# Portable pack: exe + WebView2Loader (+ any remaining non-system DLLs found by objdump).
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
    if (Test-Path $p) { $exeSrc = $p; break }
  }
  if ($exeSrc) { break }
}
if (-not $exeSrc) { throw "Release exe not found. Run npm run tauri:build first." }

$gcc = Get-Command gcc -ErrorAction SilentlyContinue
$mingwBin = if ($env:DSEE_GCC -and (Test-Path $env:DSEE_GCC)) {
  Split-Path $env:DSEE_GCC -Parent
} elseif ($gcc) {
  Split-Path $gcc.Source -Parent
} else { $null }

$out = Join-Path $repo "release\$tag"
if (Test-Path $out) { Remove-Item $out -Recurse -Force }
New-Item -ItemType Directory -Force -Path $out | Out-Null
Copy-Item $exeSrc (Join-Path $out "$tag.exe") -Force

$wv = Join-Path (Split-Path $exeSrc) "WebView2Loader.dll"
if (Test-Path $wv) { Copy-Item $wv $out -Force }

function Get-Imports([string]$path) {
  if (-not (Get-Command objdump -ErrorAction SilentlyContinue)) { return @() }
  objdump -p $path 2>$null |
    Select-String "DLL Name" |
    ForEach-Object { ($_ -split ":\s*")[-1].Trim() } |
    Sort-Object -Unique
}

$systemDll = [System.Collections.Generic.HashSet[string]]::new(
  [string[]]@(
    "kernel32.dll","ntdll.dll","user32.dll","gdi32.dll","advapi32.dll","shell32.dll",
    "ole32.dll","oleaut32.dll","combase.dll","comctl32.dll","dwmapi.dll","imm32.dll",
    "shlwapi.dll","bcryptprimitives.dll","ws2_32.dll","secur32.dll","crypt32.dll",
    "winmm.dll","version.dll","setupapi.dll","cfgmgr32.dll","powrprof.dll","userenv.dll",
    "uxtheme.dll","msvcrt.dll","ucrtbase.dll","normaliz.dll","rpcrt4.dll","sechost.dll",
    "bcrypt.dll","ncrypt.dll","wintrust.dll","imagehlp.dll","dbghelp.dll","psapi.dll",
    "iphlpapi.dll","dnsapi.dll","mswsock.dll","winhttp.dll","urlmon.dll","wininet.dll",
    "propsys.dll","dwmapi.dll","dxgi.dll","d3d11.dll","mfplat.dll","mf.dll"
  ),
  [StringComparer]::OrdinalIgnoreCase
)

$have = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
(Get-ChildItem $out -File).Name | ForEach-Object { [void]$have.Add($_) }
$queue = New-Object System.Collections.Generic.Queue[string]
(Get-ChildItem $out -File).FullName | ForEach-Object { $queue.Enqueue($_) }
$unresolved = New-Object System.Collections.Generic.List[string]

while ($queue.Count -gt 0) {
  $f = $queue.Dequeue()
  foreach ($dll in (Get-Imports $f)) {
    if ($systemDll.Contains($dll)) { continue }
    if ($dll.StartsWith("api-ms-win-", [StringComparison]::OrdinalIgnoreCase)) { continue }
    if ($dll.StartsWith("ext-ms-", [StringComparison]::OrdinalIgnoreCase)) { continue }
    if ($have.Contains($dll)) { continue }
    $src = $null
    if ($mingwBin) {
      $cand = Join-Path $mingwBin $dll
      if (Test-Path $cand) { $src = $cand }
    }
    if (-not $src) {
      $near = Join-Path (Split-Path $exeSrc) $dll
      if (Test-Path $near) { $src = $near }
    }
    if ($src) {
      Copy-Item $src (Join-Path $out $dll) -Force
      [void]$have.Add($dll)
      $queue.Enqueue((Join-Path $out $dll))
      Write-Host "bundled $dll"
    } else {
      $unresolved.Add("$dll (from $(Split-Path $f -Leaf))")
    }
  }
}

if ($unresolved.Count -gt 0) {
  Write-Host "WARNING unresolved imports:"
  $unresolved | Sort-Object -Unique | ForEach-Object { Write-Host "  $_" }
}

# Verify no MinGW runtime left unexpected if static link worked
$mingwLeft = @(Get-ChildItem $out -Filter "lib*.dll" | Select-Object -ExpandProperty Name)
if ($mingwLeft.Count -gt 0) {
  Write-Host "MinGW DLLs still shipped: $($mingwLeft -join ', ')"
} else {
  Write-Host "OK: no MinGW lib*.dll (static OpenMP/gcc)"
}

$zip = Join-Path $repo "release\$tag.zip"
if (Test-Path $zip) { Remove-Item $zip -Force }
Compress-Archive -Path (Join-Path $out "*") -DestinationPath $zip -Force
Write-Host "Exe source: $exeSrc"
Write-Host "Portable folder: $out"
Write-Host "Zip: $zip"
Get-ChildItem $out | Format-Table Name, Length -AutoSize
