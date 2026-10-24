# Assemble portable folder: versioned exe + required DLLs (+ zip).
$ErrorActionPreference = "Stop"
$repo = Split-Path $PSScriptRoot -Parent

$ver = (Get-Content (Join-Path $repo "package.json") -Raw | ConvertFrom-Json).version
$tag = "DSEE_HX_RE_" + ($ver -replace "\.", "_")

$candidates = @()
if ($env:CARGO_TARGET_DIR) { $candidates += $env:CARGO_TARGET_DIR }
$cargoCfg = Join-Path $repo ".cargo\config.toml"
if (Test-Path $cargoCfg) {
  $m = Select-String -Path $cargoCfg -Pattern 'target-dir\s*=\s*"([^"]+)"' | Select-Object -First 1
  if ($m) { $candidates += $m.Matches[0].Groups[1].Value }
}
$candidates += @(
  (Join-Path $repo "target"),
  "C:\Users\Le_Fa\AppData\Local\Temp\cursor-sandbox-cache\5385ea66da679e562821c9d7fa08ac13\cargo-target",
  "C:\Users\Le_Fa\.cache\dsee-hx-cargo-target"
)

$exeSrc = $null
foreach ($td in $candidates) {
  if (-not $td) { continue }
  foreach ($rel in @(
    "x86_64-pc-windows-gnu\release\DSEE_HX_RE.exe",
    "release\DSEE_HX_RE.exe"
  )) {
    $p = Join-Path $td $rel
    if (Test-Path $p) {
      $exeSrc = $p
      break
    }
  }
  if ($exeSrc) { break }
}
if (-not $exeSrc) { throw "Release exe not found. Run npm run tauri:build first." }

$dllSrc = Join-Path (Split-Path $exeSrc) "WebView2Loader.dll"
$out = Join-Path $repo "release\$tag"
if (Test-Path $out) { Remove-Item $out -Recurse -Force }
New-Item -ItemType Directory -Force -Path $out | Out-Null
Copy-Item $exeSrc (Join-Path $out "$tag.exe") -Force
if (Test-Path $dllSrc) { Copy-Item $dllSrc $out -Force }

$gcc = Get-Command gcc -ErrorAction SilentlyContinue
$mingwBin = if ($env:DSEE_GCC -and (Test-Path $env:DSEE_GCC)) {
  Split-Path $env:DSEE_GCC -Parent
} elseif ($gcc) {
  Split-Path $gcc.Source -Parent
} else { $null }

$runtime = @(
  "libgomp-1.dll",
  "libgcc_s_seh-1.dll",
  "libwinpthread-1.dll",
  "libstdc++-6.dll",
  "libssp-0.dll"
)
if ($mingwBin) {
  foreach ($d in $runtime) {
    $p = Join-Path $mingwBin $d
    if (Test-Path $p) { Copy-Item $p $out -Force }
  }
}

$zip = Join-Path $repo "release\$tag.zip"
if (Test-Path $zip) { Remove-Item $zip -Force }
Compress-Archive -Path (Join-Path $out "*") -DestinationPath $zip -Force
Write-Host "Exe source: $exeSrc"
Write-Host "Portable folder: $out"
Write-Host "Zip: $zip"
Get-ChildItem $out | Format-Table Name, Length -AutoSize
