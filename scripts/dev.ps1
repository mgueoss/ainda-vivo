param(
    [ValidateSet('Run', 'Build', 'Test', 'Smoke', 'Check')]
    [string]$Action = 'Run',
    [string]$BuildRoot = $env:AINDA_VIVO_BUILD_ROOT,
    [ValidateRange(1, 16)]
    [int]$Jobs = 4
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$projectRoot = Split-Path -Parent $PSScriptRoot
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw 'Rust/Cargo nao encontrado. Consulte README.md.'
}
if ([string]::IsNullOrWhiteSpace($BuildRoot)) {
    if (Test-Path -LiteralPath 'D:\') {
        $BuildRoot = 'D:\CodexBuilds\ainda-vivo-v0.0.1'
    } else {
        $BuildRoot = Join-Path $projectRoot 'target\local-build'
    }
}
$BuildRoot = [System.IO.Path]::GetFullPath($BuildRoot)
$marker = Join-Path $BuildRoot '.ainda-vivo-build'
if ((Test-Path -LiteralPath $BuildRoot) -and -not (Test-Path -LiteralPath $marker)) {
    if (@(Get-ChildItem -LiteralPath $BuildRoot -Force).Count -gt 0) {
        throw "A pasta ja contem outros arquivos: $BuildRoot. Escolha uma pasta nova com -BuildRoot."
    }
}
New-Item -ItemType Directory -Path $BuildRoot -Force | Out-Null
if (-not (Test-Path -LiteralPath $marker)) {
    Set-Content -LiteralPath $marker -Value 'Cache e compilacao do projeto AINDA VIVO v0.0.1'
}
$env:CARGO_HOME = Join-Path $BuildRoot 'cargo-home'
$env:CARGO_TARGET_DIR = Join-Path $BuildRoot 'target'
$env:TEMP = Join-Path $BuildRoot 'temp'
$env:TMP = $env:TEMP
$env:RUST_BACKTRACE = '1'
$env:RUST_LOG = 'info,wgpu_core=warn,wgpu_hal=warn,naga=warn'
New-Item -ItemType Directory -Path $env:CARGO_HOME,$env:CARGO_TARGET_DIR,$env:TEMP,(Join-Path $BuildRoot 'logs') -Force | Out-Null
Push-Location -LiteralPath $projectRoot
try {
    switch ($Action) {
        'Test' { & cargo test --workspace --locked -j $Jobs; if ($LASTEXITCODE -ne 0) { throw 'Testes falharam.' }; break }
        'Check' { & cargo fmt --all -- --check; if ($LASTEXITCODE -ne 0) { throw 'Formatacao pendente.' }; & cargo clippy --workspace --all-targets --locked -j $Jobs -- -D warnings; if ($LASTEXITCODE -ne 0) { throw 'Verificacao falhou.' }; break }
        default {
            & cargo build --locked -j $Jobs
            if ($LASTEXITCODE -ne 0) { throw 'Compilacao falhou. Veja a mensagem acima.' }
            if ($Action -eq 'Build') { Write-Host "Pronto: $env:CARGO_TARGET_DIR\debug\ainda-vivo.exe"; break }
            $binary = Join-Path $env:CARGO_TARGET_DIR 'debug\ainda-vivo.exe'
            $log = Join-Path $BuildRoot ('logs\' + (Get-Date -Format 'yyyy-MM-dd-HHmmss') + '.log')
            Write-Host "AINDA VIVO v0.0.2 | registro: $log"
            if ($Action -eq 'Smoke') {
                & $binary --smoke-test --smoke-report (Join-Path $BuildRoot 'logs\smoke-report.txt') 2>&1 | Tee-Object -FilePath $log
            } else {
                & $binary 2>&1 | Tee-Object -FilePath $log
            }
            if ($LASTEXITCODE -ne 0) { throw "O jogo terminou com erro. Consulte $log" }
        }
    }
} finally { Pop-Location }
