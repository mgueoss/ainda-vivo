param(
    [Parameter(Mandatory)] [string]$OriginalDirectory,
    [Parameter(Mandatory)] [string]$WorkDirectory,
    [switch]$Create
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Add-Type -AssemblyName System.IO.Compression.FileSystem
$repo = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '../..')).ProviderPath
$gitDir = git -C $repo rev-parse --absolute-git-dir
if ($LASTEXITCODE -ne 0) { throw 'Repositório Git indisponível.' }
$work = (New-Item -ItemType Directory -Path $WorkDirectory -Force).FullName
$catalog = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'versions.json') -Raw | ConvertFrom-Json

function CodePath([string]$Path) {
    $Path -match '^(?:Cargo\.(?:toml|lock)$|src/|crates/|\.cargo/)'
}

function ZipText($Entry) {
    $reader = [IO.StreamReader]::new($Entry.Open())
    try { $reader.ReadToEnd().Replace("`r`n", "`n") } finally { $reader.Dispose() }
}

function Compare-Code($Original, [string]$Prefix, [string]$Reference) {
    $comparePath = Join-Path $work (([guid]::NewGuid().ToString('N')) + '.zip')
    git -C $repo archive --format=zip --output=$comparePath $Reference
    if ($LASTEXITCODE -ne 0) { throw "Git não conseguiu arquivar $Reference." }
    $gitArchive = [IO.Compression.ZipFile]::OpenRead($comparePath)
    try {
        $paths = @($Original.Entries | Where-Object {
            $_.Name -and $_.FullName.StartsWith($Prefix) -and (CodePath $_.FullName.Substring($Prefix.Length))
        } | ForEach-Object { $_.FullName.Substring($Prefix.Length) } | Sort-Object)
        $gitPaths = @($gitArchive.Entries | Where-Object { $_.Name -and (CodePath $_.FullName) } |
            ForEach-Object FullName | Sort-Object)
        if (($paths -join '/') -ne ($gitPaths -join '/')) {
            throw "$Reference tem uma lista de arquivos de código diferente do ZIP de $Prefix."
        }
        foreach ($path in $paths) {
            if ((ZipText ($Original.GetEntry($Prefix + $path))) -cne (ZipText ($gitArchive.GetEntry($path)))) {
                throw "$Reference diverge do ZIP de $Prefix em $path."
            }
        }
        $paths.Count
    } finally {
        $gitArchive.Dispose()
        Remove-Item -LiteralPath $comparePath
    }
}

function Restore-Source($Zip, [string]$Prefix, [string]$Folder) {
    foreach ($entry in $Zip.Entries) {
        if (!$entry.Name -or !$entry.FullName.StartsWith($Prefix)) { continue }
        $name = $entry.FullName.Substring($Prefix.Length)
        if ($name -notmatch '^(?:\.cargo/|src/|crates/|docs/|scripts/|(?:Cargo\.toml|Cargo\.lock|\.gitignore|README\.md|JOGAR\.cmd|CHANGELOG\.md)$)') { continue }
        $target = [IO.Path]::GetFullPath((Join-Path $Folder $name))
        if (!$target.StartsWith(($Folder + [IO.Path]::DirectorySeparatorChar), [StringComparison]::OrdinalIgnoreCase)) {
            throw "Caminho de arquivo inválido no ZIP de $Prefix."
        }
        New-Item -ItemType Directory -Path ([IO.Path]::GetDirectoryName($target)) -Force | Out-Null
        [IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $target)
    }
}

foreach ($item in ($catalog | Where-Object zipSha256)) {
    $tag = 'v' + $item.version
    $prefix = "ainda-vivo-$tag/"
    $original = Join-Path $OriginalDirectory "ainda-vivo-$tag-windows-e-codigo.zip"
    if ((Get-FileHash -LiteralPath $original -Algorithm SHA256).Hash -ne $item.zipSha256) {
        throw "O ZIP de $tag diverge do catálogo."
    }
    $zip = [IO.Compression.ZipFile]::OpenRead($original)
    try {
        $known = git -C $repo tag --list $tag
        $reference = if ($known) { "$tag^{commit}" } else { $item.commit }
        $kind = if ($item.commit) { 'commit existente' } else { 'snapshot recuperado do ZIP' }
        if (!$reference -and !$Create) {
            [pscustomobject]@{Tag=$tag; Reference=''; Source=$kind; VerifiedFiles=0}
            continue
        }
        if (!$reference) {
            $recovery = (New-Item -ItemType Directory -Path (Join-Path $work ($tag + '-' + [guid]::NewGuid().ToString('N')))).FullName
            Restore-Source $zip $prefix $recovery
            $oldIndex = $env:GIT_INDEX_FILE
            $env:GIT_INDEX_FILE = Join-Path $work ([guid]::NewGuid().ToString('N') + '.index')
            $recoveryIndex = $env:GIT_INDEX_FILE
            Push-Location -LiteralPath $recovery
            try {
                git --git-dir=$gitDir --work-tree=$recovery read-tree --empty
                if ($LASTEXITCODE -ne 0) { throw 'Não foi possível criar índice isolado.' }
                git -c core.autocrlf=false -c core.safecrlf=false --git-dir=$gitDir --work-tree=$recovery add -f -- .
                if ($LASTEXITCODE -ne 0) { throw 'Não foi possível indexar o snapshot.' }
                $tree = git --git-dir=$gitDir write-tree
                if ($LASTEXITCODE -ne 0) { throw 'Não foi possível gravar a árvore.' }
                $reference = git --git-dir=$gitDir commit-tree $tree -m "Recupera snapshot arquivado da $tag" -m "Fonte: arquivo original $([IO.Path]::GetFileName($original)). SHA-256: $($item.zipSha256). Snapshot importado sem ancestralidade histórica presumida."
                if ($LASTEXITCODE -ne 0) { throw 'Não foi possível gravar o commit de arquivo.' }
            } finally {
                Pop-Location
                $env:GIT_INDEX_FILE = $oldIndex
                if (Test-Path -LiteralPath $recoveryIndex) { Remove-Item -LiteralPath $recoveryIndex }
            }
        }
        $count = Compare-Code $zip $prefix $reference
        $objectId = git -C $repo rev-parse "$reference^{commit}"
        if ($LASTEXITCODE -ne 0) { throw "Referência Git inválida para $tag." }
        if (!$known -and $Create) {
            git -C $repo tag -a $tag $objectId -m "AINDA VIVO $tag. Fonte conferida com o ZIP original SHA-256 $($item.zipSha256)."
            if ($LASTEXITCODE -ne 0) { throw "Não foi possível criar a tag $tag." }
        }
        [pscustomobject]@{Tag=$tag; Reference=$objectId; Source=$kind; VerifiedFiles=$count}
    } finally { $zip.Dispose() }
}
