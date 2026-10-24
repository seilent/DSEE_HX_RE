# Isolated smoke test: simulate a PC without MinGW / VS Build Tools on PATH.
# Checks DLL closure, then launches the portable exe with a scrubbed PATH.
#
#   .\scripts\smoke-isolated.ps1            # DLL check + launch (close app to finish)
#   .\scripts\smoke-isolated.ps1 -CheckOnly # DLL check only
param([switch]$CheckOnly)
$ErrorActionPreference = "Stop"
$repo = Split-Path $PSScriptRoot -Parent
$ver = (Get-Content (Join-Path $repo "package.json") -Raw | ConvertFrom-Json).version
$packDir = Join-Path $repo "release\DSEE_HX_RE_v$ver"
$exe = Get-ChildItem $packDir -Filter "DSEE_HX_RE_*.exe" | Select-Object -First 1
if (-not $exe) { throw "Pack not found: $packDir — run npm run tauri:pack first." }

Write-Host "== DLL closure =="
$system = [System.Collections.Generic.HashSet[string]]::new(
  [string[]]@(
    "kernel32.dll","ntdll.dll","user32.dll","gdi32.dll","advapi32.dll","shell32.dll",
    "ole32.dll","oleaut32.dll","combase.dll","comctl32.dll","dwmapi.dll","imm32.dll",
    "shlwapi.dll","bcryptprimitives.dll","ws2_32.dll","secur32.dll","crypt32.dll",
    "winmm.dll","version.dll","setupapi.dll","cfgmgr32.dll","powrprof.dll","userenv.dll",
    "uxtheme.dll","msvcrt.dll","ucrtbase.dll","normaliz.dll","rpcrt4.dll","sechost.dll",
    "bcrypt.dll","ncrypt.dll","wintrust.dll","imagehlp.dll","dbghelp.dll","psapi.dll",
    "iphlpapi.dll","dnsapi.dll","mswsock.dll","winhttp.dll","urlmon.dll","wininet.dll",
    "propsys.dll","dxgi.dll","d3d11.dll","mfplat.dll","mf.dll","gdiplus.dll",
    "oleacc.dll","winspool.drv","cryptbase.dll","sspicli.dll","nsi.dll"
  ),
  [StringComparer]::OrdinalIgnoreCase
)
$local = @{}
Get-ChildItem $packDir -File | ForEach-Object { $local[$_.Name.ToLowerInvariant()] = $_.FullName }
$missing = New-Object System.Collections.Generic.List[string]
$queue = New-Object System.Collections.Generic.Queue[string]
$local.Values | ForEach-Object { $queue.Enqueue($_) }
$seen = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)

while ($queue.Count -gt 0) {
  $f = $queue.Dequeue()
  if (-not (Get-Command objdump -ErrorAction SilentlyContinue)) { break }
  foreach ($line in (objdump -p $f 2>$null | Select-String "DLL Name")) {
    $dll = ($line -split ":\s*")[-1].Trim()
    if (-not $seen.Add($dll)) { continue }
    # api-ms-win-* / ext-ms-* are OS API-set forwarders (Win10+), not shippable DLLs.
    if ($system.Contains($dll) -or $dll -match '^(api|ext)-ms-') { continue }
    $key = $dll.ToLowerInvariant()
    if ($local.ContainsKey($key)) {
      $queue.Enqueue($local[$key])
    } else {
      $missing.Add("$dll  (needed by $(Split-Path $f -Leaf))")
    }
  }
}
if ($missing.Count -gt 0) {
  Write-Host "MISSING:" -ForegroundColor Red
  $missing | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
  exit 1
}
Write-Host "OK — all non-system DLLs present beside exe." -ForegroundColor Green
if ($CheckOnly) { exit 0 }

# Copy to a temp folder so we aren't accidentally relying on repo-adjacent files.
$iso = Join-Path $env:TEMP ("dsee-hx-iso-" + [guid]::NewGuid().ToString("n").Substring(0, 8))
New-Item -ItemType Directory -Path $iso | Out-Null
Copy-Item (Join-Path $packDir "*") $iso -Force
$isoExe = Get-ChildItem $iso -Filter "DSEE_HX_RE_*.exe" | Select-Object -First 1

# Scrub PATH: keep only Windows system dirs (no MinGW / cargo / scoop / etc.)
$sysPath = @(
  "$env:SystemRoot\System32",
  "$env:SystemRoot",
  "$env:SystemRoot\System32\Wbem",
  "$env:SystemRoot\System32\WindowsPowerShell\v1.0"
) -join ";"

Write-Host ""
Write-Host "== Launch with scrubbed PATH (no console) =="
Write-Host "  dir:  $iso"
Write-Host "  exe:  $($isoExe.Name)"
Write-Host "  PATH: System32 only (no MinGW)"
Write-Host "  Close the app window when done. Temp folder will be removed."
Write-Host ""

# ProcessStartInfo: scrub PATH without a visible cmd.exe flash.
$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = $isoExe.FullName
$psi.WorkingDirectory = $iso
$psi.UseShellExecute = $false
$psi.CreateNoWindow = $true
$psi.EnvironmentVariables["PATH"] = $sysPath
$psi.EnvironmentVariables["PATHEXT"] = ".COM;.EXE;.BAT;.CMD"
$p = [System.Diagnostics.Process]::Start($psi)
$p.WaitForExit()
$code = $p.ExitCode
Remove-Item $iso -Recurse -Force -ErrorAction SilentlyContinue
Write-Host "Exit code: $code"
if ($null -ne $code -and $code -ne 0) { exit $code }
