$ErrorActionPreference = 'Stop'
$redist = $env:AINDA_VIVO_REDIST_PATH
$scratch = $null
try {
    if ([string]::IsNullOrWhiteSpace($redist)) {
        $scratch = Join-Path ([IO.Path]::GetTempPath()) ('AindaVivo-' + [guid]::NewGuid().ToString('N'))
        New-Item -ItemType Directory -Path $scratch | Out-Null
        $redist = Join-Path $scratch 'vc_redist.x64.exe'
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri 'https://aka.ms/vc14/vc_redist.x64.exe' -OutFile $redist -UseBasicParsing -TimeoutSec 120
    }
    $signature = Get-AuthenticodeSignature -LiteralPath $redist
    if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch '(?:^|,\s*)CN=Microsoft Corporation(?:,|$)') {
        throw 'The installer does not have a valid Microsoft signature.'
    }
    $process = Start-Process -FilePath $redist -ArgumentList '/install','/passive','/norestart' -Wait -PassThru
    Write-Output ('Microsoft Visual C++ installer exit code: ' + $process.ExitCode)
    if ($process.ExitCode -in 1641,3010) { exit 30 }
    # 1638 means another installed version is present. The launcher checks the DLLs again.
    if ($process.ExitCode -notin 0,1638) { exit 1 }
} finally {
    if ($null -ne $scratch -and (Test-Path -LiteralPath $redist -PathType Leaf)) {
        Remove-Item -LiteralPath $redist -ErrorAction SilentlyContinue
    }
    if ($null -ne $scratch -and (Test-Path -LiteralPath $scratch -PathType Container)) {
        Remove-Item -LiteralPath $scratch -ErrorAction SilentlyContinue
    }
}
