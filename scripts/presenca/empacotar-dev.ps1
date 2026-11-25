param(
    [Parameter(Mandatory)] [string]$GameExe,
    [Parameter(Mandatory)] [string]$NoticesFile,
    [Parameter(Mandatory)] [string]$OutputDirectory
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$line = Select-String -LiteralPath (Join-Path $root 'Cargo.toml') -Pattern '^version = "([0-9]+\.[0-9]+\.[0-9]+-dev\.[0-9]+)"$' | Select-Object -First 1
if (-not $line) { throw 'O manifesto precisa identificar uma versão experimental -dev.' }
$version = $line.Matches[0].Groups[1].Value
$exe = (Resolve-Path -LiteralPath $GameExe).ProviderPath
$notices = (Resolve-Path -LiteralPath $NoticesFile).ProviderPath
$target = [IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Path $target -Force | Out-Null
$name = "ainda-vivo-v$version-windows-x64.zip"
$archivePath = Join-Path $target $name
if (Test-Path -LiteralPath $archivePath) { throw "O pacote já existe em $archivePath." }

function Add-Text($Archive, [string]$Name, [string]$Contents, [Text.Encoding]$Encoding) {
    $entry = $Archive.CreateEntry($Name)
    $writer = [IO.StreamWriter]::new($entry.Open(), $Encoding)
    try { $writer.Write($Contents) } finally { $writer.Dispose() }
}

$archive = [IO.Compression.ZipFile]::Open($archivePath, [IO.Compression.ZipArchiveMode]::Create)
try {
    [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($archive, $exe, 'ainda-vivo.exe', [IO.Compression.CompressionLevel]::Optimal) | Out-Null
    [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($archive, $notices, 'LICENCAS.txt') | Out-Null
    Add-Text $archive 'ABRIR DEV LAB.cmd' "@echo off`r`nstart `"`" `"%~dp0ainda-vivo.exe`" --dev-lab`r`n" ([Text.Encoding]::ASCII)
    $readme = @"
AINDA VIVO v$version
Windows x64 • Build experimental do Dev Lab da Presença

Extraia todo o pacote e abra ABRIR DEV LAB.cmd. No menu ou em campanha,
use Ctrl+I para entrar e voltar. Na aba Animação, escolha um clipe,
teste a Pista ou abra a Aproximação em primeira pessoa. Para testar
percepção e navegação, abra IA > Cenário IA. Fora do laboratório, F5
solicita a troca entre a representação anterior e a Presença física.
Para ensaiar corrida, retirada e contato, abra Caçada > Caçada livre.
Para colocar a Presença 3D passiva no corredor inicial, execute
.\ainda-vivo.exe --dev-lab --lab-campaign-preview pelo terminal.

O modelo está embutido no jogo. Blender e Rust servem à edição e à compilação.
Use o Windows 10/11 x64 com driver gráfico compatível e o runtime Visual C++ x64.
As preferências da sala ficam em %LOCALAPPDATA%\AindaVivo\dev-lab.
"@
    Add-Text $archive 'LEIA-ME.txt' $readme ([Text.UTF8Encoding]::new($false))
} finally {
    $archive.Dispose()
}

$digest = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash.ToLowerInvariant()
[IO.File]::WriteAllText("$archivePath.sha256", "$digest  $name`n", [Text.Encoding]::ASCII)
[pscustomobject]@{
    Version = $version
    Zip = $archivePath
    ZipSHA256 = $digest
    ExeSHA256 = (Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash.ToLowerInvariant()
}
