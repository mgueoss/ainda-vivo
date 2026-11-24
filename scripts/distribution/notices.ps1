param(
    [Parameter(Mandatory)] [string]$OutputFile,
    [string]$LicenseSourceDirectory = (Join-Path $PSScriptRoot 'licenses'),
    [string[]]$Features = @()
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = (Resolve-Path (Join-Path $PSScriptRoot '../..')).ProviderPath
Push-Location -LiteralPath $root
try {
    $featureArgs = if ($Features.Count -gt 0) { @('--features', ($Features -join ',')) } else { @() }
    $metadata = cargo metadata --locked --offline --filter-platform x86_64-pc-windows-msvc --format-version 1 @featureArgs
    if ($LASTEXITCODE -ne 0) { throw 'Não foi possível consultar as dependências no cache Cargo.' }
    $metadata = $metadata | ConvertFrom-Json -Depth 100
    $toolchain = rustc --print sysroot
    if ($LASTEXITCODE -ne 0) { throw 'Não foi possível localizar as licenças da biblioteca padrão Rust.' }
} finally { Pop-Location }

$graph = @{}
$queue = [Collections.Generic.Queue[string]]::new()
$selected = [Collections.Generic.HashSet[string]]::new()
foreach ($node in $metadata.resolve.nodes) { $graph[$node.id] = $node }
foreach ($member in $metadata.workspace_members) { $queue.Enqueue($member) }
while ($queue.Count) {
    $id = $queue.Dequeue()
    if (!$selected.Add($id)) { continue }
    foreach ($dependency in $graph[$id].deps) {
        if ($dependency.dep_kinds | Where-Object kind -ne 'dev') { $queue.Enqueue($dependency.pkg) }
    }
}

$rust = Get-ChildItem -LiteralPath (Join-Path $toolchain 'share/doc/rust/html/static.files') -Filter 'LICENSE-*.txt'
$apache = $rust | Where-Object Name -like 'LICENSE-APACHE-*' | Select-Object -First 1
if (!$apache) { throw 'Licença Apache-2.0 ausente dos arquivos de licença da distribuição Rust.' }

$sections = [Collections.Generic.List[object]]::new()
$ids = [Collections.Generic.Dictionary[string,int]]::new()
function Index-License([string]$Path) {
    $text = [IO.File]::ReadAllText($Path)
    if (!$text.Trim()) { throw "Texto de licença vazio: $Path." }
    $key = $text.Trim().Replace([string][char]13, '')
    if ($ids.ContainsKey($key)) { return $ids[$key] }
    $number = $sections.Count + 1
    $ids.Add($key, $number)
    $sections.Add([pscustomobject]@{ Number=$number; Text=$text })
    $number
}

$destination = [IO.Path]::GetFullPath($OutputFile)
$writer = [IO.StreamWriter]::new($destination, $false, [Text.UTF8Encoding]::new($false))
try {
    $writer.WriteLine("AINDA VIVO | Avisos de terceiros (Windows x64)")
    $writer.WriteLine("Inventário conservador das dependências da árvore Cargo Windows e da biblioteca padrão Rust.")
    $writer.WriteLine("As licenças alternativas declaradas pelo autor aparecem como publicadas no manifesto.")
    $writer.WriteLine("A presença de um nome na árvore de compilação não implica que todo o código esteja no binário.")
    $writer.WriteLine()
    $rustRefs = @($rust | ForEach-Object { Index-License $_.FullName })
    $writer.WriteLine("Rust Standard Library | MIT OR Apache-2.0 | https://www.rust-lang.org/policies/licenses | textos: $($rustRefs -join ', ')")
    $apacheRef = Index-License $apache.FullName
    $packages = $metadata.packages | Where-Object { $_.source -and $selected.Contains($_.id) } | Sort-Object name,version
    foreach ($package in $packages) {
        $folder = Split-Path -Parent $package.manifest_path
        $files = @(Get-ChildItem -LiteralPath $folder -File | Where-Object Name -Match '^(LICENSE|COPYING|NOTICE|COPYRIGHT|LICENCE)(?:[_.-]|$)' | ForEach-Object FullName)
        if ($package.license_file) {
            $files += (Join-Path $folder $package.license_file)
        }
        $refs = @($files | Sort-Object -Unique | ForEach-Object { Index-License $_ })
        if (!$refs) {
            $supplement = switch -Wildcard ($package.name) {
                'taffy' { "taffy-$($package.version).txt" }
                'encase_derive*' { 'mit0.txt' }
                'hexf-parse' { 'cc0.txt' }
            }
            if ($supplement) {
                $refs = @(Index-License (Join-Path $LicenseSourceDirectory $supplement))
            } elseif ($package.license -match 'Apache-2\.0') {
                $refs = @($apacheRef)
            } else {
                throw "Revise a licença ausente de $($package.name) $($package.version)."
            }
        }
        $url = if ($package.repository) { $package.repository } else { "https://crates.io/crates/$($package.name)/$($package.version)" }
        $writer.WriteLine("$($package.name) $($package.version) | $($package.license) | $url | textos: $($refs -join ', ')")
    }
    foreach ($section in $sections) {
        $writer.WriteLine()
        $writer.WriteLine("======== Texto $($section.Number) ========")
        $writer.WriteLine($section.Text)
    }
} finally { $writer.Dispose() }
Write-Output $destination
