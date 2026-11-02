<#
    Build a transferable, self-contained copy of NoSleep.

    Output:
      dist\NoSleep\NoSleep.exe     the application
      dist\NoSleep\README.txt      one-page instructions for the target machine
      dist\NoSleep\SHA256.txt      checksum to confirm the copy survived transfer
      dist\NoSleep-<version>-win64.zip

    Nothing here installs anything; the output directory can be copied to a USB
    stick and run in place.
#>

param(
    [switch]$SkipBuild,
    [string]$Binary = ''
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Push-Location $root
try {
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"

    if (-not $SkipBuild) {
        Write-Host 'Building release...' -ForegroundColor Cyan
        & cargo build --release --quiet
        if ($LASTEXITCODE -ne 0) { throw 'cargo build failed' }
    }

    if ([string]::IsNullOrWhiteSpace($Binary)) {
        $source = Join-Path $root 'target\release\nosleep.exe'
    } else {
        $source = if ([IO.Path]::IsPathRooted($Binary)) { $Binary } else { Join-Path $root $Binary }
    }
    if (-not (Test-Path $source)) { throw "Release binary not found: $source" }

    $version = (& cargo metadata --no-deps --format-version 1 | ConvertFrom-Json).packages[0].version
    $dist = Join-Path $root 'dist'
    $stage = Join-Path $dist 'NoSleep'
    if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
    New-Item -ItemType Directory -Force -Path $stage | Out-Null

    Copy-Item $source (Join-Path $stage 'NoSleep.exe')

    $exe = Get-Item (Join-Path $stage 'NoSleep.exe')
    $sha = [System.Security.Cryptography.SHA256]::Create()
    $stream = [System.IO.File]::OpenRead($exe.FullName)
    try { $hash = ([BitConverter]::ToString($sha.ComputeHash($stream))).Replace('-', '') }
    finally { $stream.Dispose(); $sha.Dispose() }

    $readme = @"
NoSleep $version - one-click wake lock for Windows
==================================================

WHAT IT DOES
  Left-click the tray icon once: this PC stops sleeping and its display stays on.
  Left-click again: normal sleep behaviour returns.
  Right-click the icon: menu (toggle, Quit).

HOW TO START
  1. Copy this folder anywhere, for example  %LOCALAPPDATA%\NoSleep\
     It also runs directly from a USB stick; it writes nothing.
  2. Double-click NoSleep.exe.
  3. If Windows shows "Windows protected your PC", click More info -> Run anyway.
     If that link is missing, this command removes the downloaded-file mark:

         Unblock-File .\NoSleep.exe

  4. Windows 11 hides new tray icons. Click the "^" overflow arrow and drag the
     NoSleep icon onto the taskbar, or use:
         Settings -> Personalisation -> Taskbar -> Other system tray icons

HOW TO STOP IT
  Right-click the icon -> Quit.
  Nothing is installed and nothing is left behind: deleting NoSleep.exe is a
  complete uninstall. Killing the process from Task Manager also restores
  normal power behaviour immediately.

TO CONFIRM IT WORKS
  Open a terminal in this folder and run:

      .\NoSleep.exe --diagnose

  "accepted : yes" and exit code 0 mean the operating system accepted the wake
  request. To test the real thing, unplug the charger, turn it On, leave the
  machine alone for longer than its sleep timeout, then check:

      Get-WinEvent -FilterHashtable @{LogName='System'; Id=42} -MaxEvents 5 |
        Select-Object TimeCreated

  No recent event 42 means the machine did not sleep.

WHAT IT DOES NOT DO
  - It does not change power plans, policy, or the registry.
  - It cannot defeat an enforced inactivity lock (if IT set one, the screen
    locks anyway).
  - It does not prevent sleep when the lid is closed, or hibernation at
    critically low battery.
  - It does not start with Windows. Launch it yourself.

REQUIREMENTS
  Windows 10 1809+ or Windows 11, 64-bit. Nothing else - no runtime, no
  installer, no administrator rights.

VERIFY THE COPY
  File    : NoSleep.exe
  Bytes   : $($exe.Length)
  SHA256  : $hash
"@
    Set-Content -Path (Join-Path $stage 'README.txt') -Value $readme -Encoding UTF8
    Set-Content -Path (Join-Path $stage 'SHA256.txt') -Value ("$hash  NoSleep.exe") -Encoding ASCII

    $zip = Join-Path $dist ("NoSleep-$version-win64.zip")
    if (Test-Path $zip) { Remove-Item $zip -Force }
    Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip -CompressionLevel Optimal

    Write-Host ''
    Write-Host 'Portable build ready' -ForegroundColor Green
    Write-Host ("  folder : {0}" -f $stage)
    Write-Host ("  exe    : {0:N0} bytes" -f $exe.Length)
    Write-Host ("  zip    : {0:N0} bytes" -f (Get-Item $zip).Length)
    Write-Host ("  sha256 : {0}" -f $hash)
}
finally {
    Pop-Location
}
