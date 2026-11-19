[CmdletBinding(DefaultParameterSetName='History')]
param(
    [Parameter(Mandatory)] [string]$Launcher,
    [Parameter(Mandatory)] [string]$NoticesFile,
    [Parameter(Mandatory)] [string]$OutputDirectory,
    [Parameter(Mandatory,ParameterSetName='History')] [string]$OriginalDirectory,
    [Parameter(Mandatory,ParameterSetName='Build')] [string]$GameExe,
    [Parameter(Mandatory,ParameterSetName='Build')] [ValidatePattern('^\d+\.\d+\.\d+$')] [string]$Version,
    [Parameter(ParameterSetName='Build')] [string]$AssetsDirectory,
    [string]$LicensedRuntimeInstaller
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
$launcherPath = (Resolve-Path -LiteralPath $Launcher).ProviderPath
$noticesPath = (Resolve-Path -LiteralPath $NoticesFile).ProviderPath
$destination = (New-Item -ItemType Directory -Path $OutputDirectory -Force).FullName
$catalog = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'versions.json') -Raw | ConvertFrom-Json
$scratch = New-Item -ItemType Directory -Path (Join-Path $destination ('.package-' + [guid]::NewGuid().ToString('N')))

function Read-ZipText($Entry) {
    $reader = [IO.StreamReader]::new($Entry.Open())
    try { $reader.ReadToEnd() } finally { $reader.Dispose() }
}

function Add-ZipText($Zip, [string]$Path, [string]$Text) {
    $writer = [IO.StreamWriter]::new($Zip.CreateEntry($Path).Open())
    try { $writer.Write($Text) } finally { $writer.Dispose() }
}

function Get-EntryHash($Entry) {
    $stream = $Entry.Open()
    try { (Get-FileHash -InputStream $stream -Algorithm SHA256).Hash.ToLowerInvariant() } finally { $stream.Dispose() }
}

function Invoke-PeInspection([string]$Path) {
    $report = Join-Path $scratch.FullName 'pe.txt'
    $info = [Diagnostics.ProcessStartInfo]::new($launcherPath)
    $info.UseShellExecute = $false
    foreach ($argument in @('--inspect', $Path, '--report', $report)) { $info.ArgumentList.Add($argument) }
    $process = [Diagnostics.Process]::Start($info)
    try {
        if (!$process.WaitForExit(10000)) { $process.Kill(); throw 'Tempo excedido na leitura do PE.' }
        $result = Get-Content -LiteralPath $report -Raw
        if ($process.ExitCode -ne 0 -or $result -notmatch '(?m)^machine=0x8664$') { throw "PE x64 inválido em $Path. $result" }
        $result
    } finally { $process.Dispose() }
}

function Add-PortablePackage([string]$ReleaseVersion, [string]$GamePath, [string]$ExpectedHash) {
    $file = "ainda-vivo-v$ReleaseVersion-windows-x64.zip"
    $finalPath = Join-Path $destination $file
    $hashPath = $finalPath + '.sha256'
    if ((Test-Path -LiteralPath $finalPath) -or (Test-Path -LiteralPath $hashPath)) {
        throw "Já existe uma distribuição chamada $file. Use uma pasta nova para gerar outra cópia."
    }
    $gameHash = (Get-FileHash -LiteralPath $GamePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($ExpectedHash -and $gameHash -ne $ExpectedHash) { throw "SHA-256 do executável v$ReleaseVersion divergiu." }
    $audit = Invoke-PeInspection $GamePath
    $archive = Join-Path $scratch.FullName $file
    $zip = [IO.Compression.ZipFile]::Open($archive, [IO.Compression.ZipArchiveMode]::Create)
    try {
        $mode = [IO.Compression.CompressionLevel]::Optimal
        [void][IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $launcherPath, 'AINDA-VIVO.exe', $mode)
        [void][IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $GamePath, 'game/ainda-vivo.exe', $mode)
        [void][IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $noticesPath, 'LICENCAS.txt', $mode)
        Add-ZipText $zip 'game/version.txt' "$ReleaseVersion$([Environment]::NewLine)"
        $zip.CreateEntry('dependencies/') | Out-Null
        if ($LicensedRuntimeInstaller) {
            [void][IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $LicensedRuntimeInstaller, 'dependencies/vc_redist.x64.exe', $mode)
        }
        if ($AssetsDirectory) {
            $assets = (Resolve-Path -LiteralPath $AssetsDirectory).ProviderPath
            foreach ($item in Get-ChildItem -LiteralPath $assets -Recurse -File) {
                $relative = [IO.Path]::GetRelativePath($assets, $item.FullName).Replace('\', '/')
                [void][IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $item.FullName, ('game/assets/' + $relative), $mode)
            }
        }
        $notice = @"
AINDA VIVO v$ReleaseVersion
Windows 10/11 x64 • Pré-lançamento experimental

1. Extraia todo o ZIP para uma pasta.
2. Abra AINDA-VIVO.exe e clique em JOGAR.
3. Se faltar o Microsoft Visual C++ x64, o inicializador pede sua autorização.
   Com os requisitos já instalados, a preparação e o jogo funcionam offline.

É necessário um driver de vídeo com Vulkan ou Direct3D 12 compatível.
Se houver um erro, use DIAGNÓSTICO no inicializador. Registros em
%LOCALAPPDATA%\AindaVivo\launcher\$ReleaseVersion.

SHA-256 do executável original: $gameHash
"@
        Add-ZipText $zip 'LEIA-ME.txt' $notice
    } finally { $zip.Dispose() }
    $verify = [IO.Compression.ZipFile]::OpenRead($archive)
    try {
        if ((Get-EntryHash ($verify.GetEntry('game/ainda-vivo.exe'))) -ne $gameHash -or
            (Get-EntryHash ($verify.GetEntry('AINDA-VIVO.exe'))) -ne $launcherHash) {
            throw 'A cópia arquivada diverge dos executáveis de origem.'
        }
    } finally { $verify.Dispose() }
    $sha = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    Move-Item -LiteralPath $archive -Destination $finalPath
    "$sha  $file" | Set-Content -LiteralPath $hashPath -Encoding ascii
    [pscustomobject]@{
        Version=$ReleaseVersion; File=$finalPath; ZipSha256=$sha; ExeSha256=$gameHash
        Bytes=(Get-Item -LiteralPath $finalPath).Length
        Imports=($audit -split '\r?\n' | Where-Object { $_ -like 'imports=*' }) -replace '^imports=', ''
    }
}

try {
    $launcherHash = (Get-FileHash -LiteralPath $launcherPath -Algorithm SHA256).Hash.ToLowerInvariant()
    $launcherAudit = Invoke-PeInspection $launcherPath
    if ($launcherAudit -match '(?i)vcruntime|msvcp140|api-ms-win-crt') {
        throw 'O launcher depende de runtime externo. Compile com -C target-feature=+crt-static.'
    }
    if ($LicensedRuntimeInstaller) {
        $LicensedRuntimeInstaller = (Resolve-Path -LiteralPath $LicensedRuntimeInstaller).ProviderPath
        $signature = Get-AuthenticodeSignature -LiteralPath $LicensedRuntimeInstaller
        if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch '(?:^|,\s*)CN=Microsoft Corporation(?:,|$)') {
            throw 'O redistribuível fornecido não tem assinatura Microsoft válida.'
        }
    }
    if ($PSCmdlet.ParameterSetName -eq 'Build') {
        Add-PortablePackage $Version (Resolve-Path -LiteralPath $GameExe).ProviderPath ''
        return
    }
    foreach ($record in ($catalog | Where-Object zipSha256)) {
        $v = $record.version
        $original = Join-Path $OriginalDirectory "ainda-vivo-v$v-windows-e-codigo.zip"
        if (!(Test-Path -LiteralPath $original -PathType Leaf)) { throw "Original v$v ausente em $original." }
        if ((Get-FileHash -LiteralPath $original -Algorithm SHA256).Hash -ne $record.zipSha256) {
            throw "O ZIP original v$v não corresponde ao arquivo catalogado."
        }
        $input = [IO.Compression.ZipFile]::OpenRead($original)
        try {
            $prefix = "ainda-vivo-v$v/"
            $sourceName = if ($v -eq '0.0.1') { 'ainda-vivo.exe' } else { "ainda-vivo-v$v.exe" }
            $source = $input.GetEntry($prefix + "windows/$sourceName")
            $manifest = $input.GetEntry($prefix + 'Cargo.toml')
            if (!$source -or !$manifest -or (Read-ZipText $manifest) -notmatch "(?m)^version = `"$([regex]::Escape($v))`"$") {
                throw "Executável ou manifesto v$v inválido no ZIP original."
            }
            $stagedExe = Join-Path $scratch.FullName $sourceName
            [IO.Compression.ZipFileExtensions]::ExtractToFile($source, $stagedExe)
        } finally { $input.Dispose() }
        Add-PortablePackage $v $stagedExe $record.exeSha256
        Remove-Item -LiteralPath $stagedExe
    }
} finally {
    $scratchRoot = [IO.Path]::GetFullPath($scratch.FullName)
    $parent = [IO.Path]::GetDirectoryName($scratchRoot)
    if ($parent -eq [IO.Path]::TrimEndingDirectorySeparator([IO.Path]::GetFullPath($destination)) -and
        [IO.Path]::GetFileName($scratchRoot) -match '^\.package-[a-f0-9]{32}$') {
        Remove-Item -LiteralPath $scratchRoot -Recurse -Force
    }
}
