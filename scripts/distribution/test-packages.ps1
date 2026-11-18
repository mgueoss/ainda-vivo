param(
    [Parameter(Mandatory)] [string]$PackageDirectory,
    [Parameter(Mandatory)] [string]$WorkDirectory,
    [Alias('Version')] [string[]]$SelectedVersion
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Add-Type -AssemblyName System.IO.Compression.FileSystem
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class LaunchValidation {
    [DllImport("user32.dll")] public static extern IntPtr GetDlgItem(IntPtr h, int id);
    [DllImport("user32.dll")] public static extern bool IsWindowEnabled(IntPtr h);
    [DllImport("user32.dll")] public static extern IntPtr SendMessageW(IntPtr h, uint m, IntPtr w, IntPtr l);
}
'@

$base = (New-Item -ItemType Directory -Path $WorkDirectory -Force).FullName
$run = (New-Item -ItemType Directory -Path (Join-Path $base ([DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-ffff')))).FullName
$env:AINDA_VIVO_LAUNCHER_DATA_DIR = Join-Path $run 'diagnostico'
$env:AINDA_VIVO_DATA_DIR = Join-Path $run 'preferencias'
$env:RUST_LOG = 'info,wgpu_core=warn,wgpu_hal=error,naga=warn'
$catalog = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'versions.json') -Raw | ConvertFrom-Json
if ($SelectedVersion) {
    $catalog = @($catalog | Where-Object { $_.version -in $SelectedVersion })
    if ($catalog.Count -ne $SelectedVersion.Count) { throw 'Uma das versões selecionadas não está no catálogo.' }
}

function Start-Launcher([string]$Exe, [string[]]$Arguments) {
    $info = [Diagnostics.ProcessStartInfo]::new($Exe)
    $info.UseShellExecute = $false
    $info.WorkingDirectory = [IO.Path]::GetDirectoryName($Exe)
    foreach ($argument in $Arguments) { $info.ArgumentList.Add($argument) }
    [Diagnostics.Process]::Start($info)
}

function Wait-Windows([string]$Step, [scriptblock]$Check) {
    for ($try = 0; $try -lt 80; $try++) {
        $value = & $Check
        if ($value) { return $value }
        Start-Sleep -Milliseconds 175
    }
    throw "Tempo excedido na etapa $Step."
}

$results = foreach ($record in $catalog) {
    $version = $record.version
    $filename = "ainda-vivo-v$version-windows-x64.zip"
    $archive = Join-Path $PackageDirectory $filename
    $checksum = ((Get-Content -LiteralPath ($archive + '.sha256') -Raw).Trim() -split '\s+')
    $actual = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($checksum.Count -ne 2 -or $checksum[0] -ne $actual -or $checksum[1] -ne $filename) {
        throw "SHA-256 do pacote v$version inválido."
    }
    $zip = [IO.Compression.ZipFile]::OpenRead($archive)
    try {
        $allowed = @('AINDA-VIVO.exe', 'game/ainda-vivo.exe', 'game/version.txt', 'dependencies/', 'LEIA-ME.txt', 'LICENCAS.txt') | Sort-Object
        $entries = @($zip.Entries | ForEach-Object FullName | Sort-Object)
        $required = @($allowed | Where-Object { $_ -cnotin $entries })
        $unexpected = @($entries | Where-Object {
            $_ -cnotin $allowed -and $_ -cne 'dependencies/vc_redist.x64.exe' -and $_ -cnotlike 'game/assets/*'
        })
        if ($required -or $unexpected) { throw "Conteúdo inesperado em $filename." }
    } finally { $zip.Dispose() }
    $folder = Join-Path $run "v$version"
    [IO.Compression.ZipFile]::ExtractToDirectory($archive, $folder)
    $launcher = Join-Path $folder 'AINDA-VIVO.exe'
    $binary = Join-Path $folder 'game\ainda-vivo.exe'
    if ((Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash -ne $record.exeSha256) {
        throw "Binário extraído v$version difere do original."
    }
    $report = Join-Path $run "v$version-verificacao.txt"
    $elapsed = [Diagnostics.Stopwatch]::StartNew()
    $check = Start-Launcher $launcher @('--verify', '--report', $report)
    try {
        if (!$check.WaitForExit(15000)) { $check.Kill(); throw 'A verificação não terminou.' }
        $elapsed.Stop()
        $proof = Get-Content -LiteralPath $report -Raw
        if ($check.ExitCode -ne 0 -or $proof -notmatch "(?m)^version=$([regex]::Escape($version))$" -or
            $proof -notmatch '(?m)^state=Ready$') {
            throw "Verificação v$version falhou: $proof"
        }
    } finally { $check.Dispose() }

    $game = $null
    $gui = Start-Launcher $launcher @()
    try {
        $button = Wait-Windows "v$version, preparação do launcher" {
            $gui.Refresh()
            $window = $gui.MainWindowHandle
            if ($window -ne [IntPtr]::Zero -and $gui.MainWindowTitle -like "*v$version") {
                $play = [LaunchValidation]::GetDlgItem($window, 101)
                if ($play -ne [IntPtr]::Zero -and [LaunchValidation]::IsWindowEnabled($play)) { $play }
            }
        }
        # Send the native BN_CLICKED notification directly, independent of desktop focus.
        [void][LaunchValidation]::SendMessageW($gui.MainWindowHandle, 0x0111, [IntPtr]101, $button)
        $game = Wait-Windows "v$version, janela do jogo" {
            Get-Process -Name 'ainda-vivo' -ErrorAction SilentlyContinue |
                Where-Object { $_.Path -eq $binary -and $_.MainWindowTitle -like "*v$version*" } |
                Select-Object -First 1
        }
        $title = $game.MainWindowTitle
        $logs = Join-Path $env:AINDA_VIVO_LAUNCHER_DATA_DIR $version
        $engine = Wait-Windows "v$version, inicialização do renderizador" {
            $file = Join-Path $logs 'jogo.log'
            if (Test-Path -LiteralPath $file) {
                Select-String -LiteralPath $file -Pattern 'AdapterInfo' | Select-Object -First 1 -ExpandProperty Line
            }
        }
        [pscustomobject]@{
            Version=$version; Status='PASS'; ExeSha256=$record.exeSha256; ZipSha256=$actual
            Window=$title; VerificationMs=$elapsed.ElapsedMilliseconds; Renderer=$engine
        }
    } finally {
        if ($game -and !$game.HasExited -and $game.CloseMainWindow()) { [void]$game.WaitForExit(5000) }
        if (!$gui.HasExited -and $gui.CloseMainWindow()) { [void]$gui.WaitForExit(5000) }
        $gui.Dispose()
    }
}

$missing = (New-Item -ItemType Directory -Path (Join-Path $run 'pacote-sem-jogo\game') -Force).Parent.FullName
$exe = Join-Path $missing 'AINDA-VIVO.exe'
$sampleVersion = $catalog[0].version
Copy-Item -LiteralPath (Join-Path $run "v$sampleVersion\AINDA-VIVO.exe") -Destination $exe
$sampleVersion | Set-Content -LiteralPath (Join-Path $missing 'game\version.txt') -Encoding ascii
$errorReport = Join-Path $run 'pacote-sem-jogo.txt'
$check = Start-Launcher $exe @('--verify', '--report', $errorReport)
try {
    if (!$check.WaitForExit(10000)) { $check.Kill(); throw 'O teste de pacote incompleto não terminou.' }
    if ($check.ExitCode -ne 1 -or (Get-Content -LiteralPath $errorReport -Raw) -notmatch '(?m)^state=Blocked$') {
        throw 'O inicializador aceitou um pacote sem executável.'
    }
} finally { $check.Dispose() }

$results | ConvertTo-Json -AsArray -Depth 3 | Set-Content -LiteralPath (Join-Path $run 'validacao.json') -Encoding utf8
Write-Host "Evidências e logs em $run. Pacote sem executável bloqueado."
$results
