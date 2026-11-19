[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string]$PackageDirectory,
    [Parameter(Mandatory)] [string]$ValidationReport,
    [string[]]$Version,
    [string]$OutputReport
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$repo = 'mgueoss/ainda-vivo'
$api = "https://api.github.com/repos/$repo"
$catalog = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'versions.json') -Raw | ConvertFrom-Json
$recommended = ($catalog | Sort-Object { [version]$_.version } -Descending | Select-Object -First 1).version
if ($Version) {
    $catalog = @($catalog | Where-Object { $_.version -in $Version })
    if ($catalog.Count -ne $Version.Count) { throw 'Uma das versões selecionadas não está no catálogo.' }
}
$validation = Get-Content -LiteralPath $ValidationReport -Raw | ConvertFrom-Json
$marker = '<!-- ainda-vivo-distribuicao-v1 -->'
$env:GIT_TERMINAL_PROMPT = '0'
$env:GCM_INTERACTIVE = 'Never'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../..')).ProviderPath

$requests = foreach ($item in $catalog) {
    $tag = "v$($item.version)"
    $filename = "ainda-vivo-$tag-windows-x64.zip"
    $zip = Join-Path $PackageDirectory $filename
    $checksum = $zip + '.sha256'
    $digest = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLowerInvariant()
    $line = (Get-Content -LiteralPath $checksum -Raw).Trim()
    if ($line -cne "$digest  $filename") { throw "Hash local de $tag inválido." }
    if (!($validation | Where-Object { $_.Version -eq $item.version -and $_.Status -eq 'PASS' -and $_.ZipSha256 -eq $digest })) {
        throw "Falta validação nativa do pacote atual de $tag."
    }
    $reference = git -C $root rev-parse "refs/tags/$tag"
    if ($LASTEXITCODE -ne 0) { throw "Tag local não encontrada: $tag." }
    $manifest = git -C $root show ("${tag}:Cargo.toml")
    if ($LASTEXITCODE -ne 0 -or ($manifest -join [char]10) -notmatch ('(?m)^version = "' + [regex]::Escape($item.version) + '"$')) {
        throw "O manifesto da tag $tag informa outra versão."
    }
    [pscustomobject]@{ Item=$item; Tag=$tag; Hash=$digest; Filename=$filename; Zip=$zip; Checksum=$checksum; Ref=$reference }
}
$refs = @($requests | ForEach-Object { 'refs/tags/' + $_.Tag })
$remote = @(git -C $root ls-remote --refs origin @refs)
if ($LASTEXITCODE -ne 0) { throw 'Não foi possível ler as tags do remoto.' }
foreach ($request in $requests) {
    if (("$($request.Ref)" + [char]9 + "refs/tags/$($request.Tag)") -notin $remote) {
        throw "A tag remota $($request.Tag) diverge ou está ausente."
    }
}

# Keep the Git Credential Manager token in memory and out of URLs, stdout and files.
$requestText = @('protocol=https', 'host=github.com', "path=$repo.git", '', '') -join [char]10
$credential = $requestText | git -C $root credential fill 2>$null
$secret = $credential | Where-Object { $_.StartsWith('password=') } | Select-Object -First 1
if (!$secret) { throw 'A sessão do Git não forneceu uma credencial para publicação.' }
$headers = @{
    Authorization = ('Bearer ' + $secret.Substring(9))
    'X-GitHub-Api-Version' = '2022-11-28'
    Accept = 'application/vnd.github+json'
}
$metadata = Invoke-RestMethod -Uri $api -Headers $headers -TimeoutSec 30
if ($metadata.full_name -cne $repo -or !$metadata.permissions.push) { throw 'A conta não tem acesso de escrita ao repositório esperado.' }

$handler = [Net.Http.HttpClientHandler]::new()
$handler.AllowAutoRedirect = $true
$downloads = [Net.Http.HttpClient]::new($handler)
$downloads.Timeout = [TimeSpan]::FromSeconds(40)

function Confirm-Download($Asset, [string]$File, [string]$Digest, [bool]$IsZip) {
    if ($Asset.state -ne 'uploaded' -or [long]$Asset.size -ne (Get-Item -LiteralPath $File).Length) {
        throw "Asset remoto incompleto: $($Asset.name)."
    }
    $digestProperty = $Asset.PSObject.Properties['digest']
    $remoteDigest = if ($digestProperty) { $digestProperty.Value } else { $null }
    if ($remoteDigest -and $remoteDigest -ne "sha256:$Digest") {
        throw "O hash do GitHub diverge para $($Asset.name)."
    }
    $message = [Net.Http.HttpRequestMessage]::new([Net.Http.HttpMethod]::Get, "$api/releases/assets/$($Asset.id)")
    $message.Headers.Authorization = [Net.Http.Headers.AuthenticationHeaderValue]::Parse($headers.Authorization)
    $message.Headers.UserAgent.ParseAdd('AindaVivo-Distribution/1.0')
    $message.Headers.Accept.ParseAdd('application/octet-stream')
    $message.Headers.Add('X-GitHub-Api-Version', $headers['X-GitHub-Api-Version'])
    if ($IsZip -and $remoteDigest) { $message.Headers.Range = [Net.Http.Headers.RangeHeaderValue]::new(0, 3) }
    $response = $null
    try {
        $response = $downloads.Send($message, [Net.Http.HttpCompletionOption]::ResponseHeadersRead)
        if (!$response.IsSuccessStatusCode) {
            throw "Download de $($Asset.name) retornou HTTP $([int]$response.StatusCode)."
        }
        $stream = $response.Content.ReadAsStream()
        try {
            if (!$IsZip -or !$remoteDigest) {
                $sha = [Security.Cryptography.SHA256]::HashData($stream)
                if ([Convert]::ToHexString($sha).ToLowerInvariant() -ne $Digest) {
                    throw "Download com digest incorreto: $($Asset.name)."
                }
            } else {
                $bytes = [byte[]]::new(4)
                $stream.ReadExactly($bytes)
                if ([Convert]::ToHexString($bytes) -ne '504B0304') {
                    throw "A resposta de download não é um ZIP: $($Asset.name)."
                }
            }
        } finally { $stream.Dispose() }
    } finally {
        if ($response) { $response.Dispose() }
        $message.Dispose()
    }
}

function Get-Assets($Release) {
    Invoke-RestMethod -Uri "$api/releases/$($Release.id)/assets?per_page=100" -Headers $headers -TimeoutSec 30 | ForEach-Object { $_ }
}

function Confirm-Assets($Release, $Request) {
    $assets = @(Get-Assets $Release)
    foreach ($file in @($Request.Zip, $Request.Checksum)) {
        $name = [IO.Path]::GetFileName($file)
        $asset = $assets | Where-Object name -ceq $name | Select-Object -First 1
        if (!$asset) { throw "Está faltando o asset remoto $name." }
        $digest = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant()
        Confirm-Download $asset $file $digest ($file -eq $Request.Zip)
    }
}

$published = [Collections.Generic.List[object]]::new()
try {
    foreach ($request in $requests) {
        $tag = $request.Tag
        $item = $request.Item
        # Include drafts and paginate, as the tag endpoint does not find unpublished drafts.
        $release = $null
        for ($page = 1; !$release; $page++) {
            $batch = @(Invoke-RestMethod -Uri "$api/releases?per_page=100&page=$page" -Headers $headers -TimeoutSec 30 | ForEach-Object { $_ })
            $release = $batch | Where-Object tag_name -ceq $tag | Select-Object -First 1
            if ($batch.Count -lt 100) { break }
        }
        $outcome = 'existente'
        if (!$release) {
            $title = "AINDA VIVO $tag — $($item.title)"
            if ($item.version -eq $recommended) { $title += ' · recomendada para testes' }
            $body = @"
## O que mudou

$($item.changelog)

## Jogar no Windows x64

1. Baixe **$($request.Filename)** e extraia o ZIP inteiro.
2. Abra **AINDA-VIVO.exe** e clique em **JOGAR**.
3. Se faltar o runtime Visual C++ x64, autorize a instalação oficial da Microsoft dentro do inicializador.

Com o runtime e o driver Vulkan ou Direct3D 12 já disponíveis, o jogo abre offline. Use **DIAGNÓSTICO** para consultar os logs. Build experimental.

**SHA-256 do ZIP**: $($request.Hash)

## Limitações

$($item.known)

$marker
"@
            $data = @{ tag_name=$tag; name=$title; body=$body; draft=$true; prerelease=$true; make_latest='false' } | ConvertTo-Json
            $release = Invoke-RestMethod -Method Post -Uri "$api/releases" -Headers $headers -TimeoutSec 60 -ContentType 'application/json; charset=utf-8' -Body $data
        }
        if ($release.draft -and $release.body -like "*$marker*" -and $release.body -like "*$($request.Hash)*") {
            $uploadBase = ($release.upload_url -split '\{')[0]
            if ($uploadBase -ne "https://uploads.github.com/repos/$repo/releases/$($release.id)/assets") {
                throw "Destino inesperado de upload para $tag."
            }
            $present = @(Get-Assets $release)
            foreach ($file in @($request.Zip, $request.Checksum)) {
                $name = [IO.Path]::GetFileName($file)
                if ($present | Where-Object name -ceq $name) { continue }
                $type = if ($file -eq $request.Zip) { 'application/zip' } else { 'text/plain' }
                Write-Host "Enviando $name"
                $url = $uploadBase + '?name=' + [Uri]::EscapeDataString($name)
                Invoke-RestMethod -Method Post -Uri $url -Headers $headers -InFile $file -ContentType $type -TimeoutSec 240 | Out-Null
            }
            Confirm-Assets $release $request
            $release = Invoke-RestMethod -Method Patch -Uri "$api/releases/$($release.id)" -Headers $headers -TimeoutSec 60 -ContentType 'application/json; charset=utf-8' -Body '{"draft":false,"prerelease":true,"make_latest":"false"}'
            $outcome = 'publicada'
        } elseif ($release.draft) {
            throw "A release $tag já tem rascunho diferente. A publicação foi interrompida."
        }
        if ($release.draft -or !$release.prerelease) { throw "O estado remoto de $tag não é de pré-lançamento." }
        Confirm-Assets $release $request
        $record = [pscustomobject]@{
            Version=$tag; Status=$outcome; Url=$release.html_url
            ZipSha256=$request.Hash; Prerelease=$release.prerelease; DownloadVerified=$true
        }
        $published.Add($record)
        if ($OutputReport) {
            $published | ConvertTo-Json -AsArray -Depth 4 | Set-Content -LiteralPath $OutputReport -Encoding utf8
        }
        $record
    }
} finally {
    $downloads.Dispose()
}
