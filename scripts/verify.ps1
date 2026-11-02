#requires -Version 5.1
<#
    NoSleep verification.

    Static checks (safe everywhere, including a headless CI runner):
      1. the release binary exists and is under budget
      2. it is a GUI-subsystem image with no non-Windows dependencies
      3. the power API accepts the request (--diagnose exit code)

    Runtime checks (need an interactive desktop, so skipped with -StaticOnly):
      4. the process starts, registers a tray icon, refuses a second instance,
         and sits idle without burning CPU

    Everything here is read-only except starting and stopping our own process.
#>
param(
    [switch]$StaticOnly,
    [string]$Exe = ''
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($Exe)) {
    $exe = Join-Path $root 'target\release\nosleep.exe'
} else {
    $exe = if ([IO.Path]::IsPathRooted($Exe)) { $Exe } else { Join-Path $root $Exe }
}
$budgetBytes = 1MB

if (-not (Test-Path $exe)) {
    throw "Build first: cargo build --release"
}

$failures = New-Object System.Collections.Generic.List[string]
function Pass($label, $detail) { Write-Host ("  PASS  {0,-38} {1}" -f $label, $detail) -ForegroundColor Green }
function Fail($label, $detail) { Write-Host ("  FAIL  {0,-38} {1}" -f $label, $detail) -ForegroundColor Red; $failures.Add($label) }
function Info($label, $detail) { Write-Host ("  ..    {0,-38} {1}" -f $label, $detail) -ForegroundColor DarkGray }

Write-Host "`nNoSleep verification" -ForegroundColor Cyan
Write-Host ("=" * 60)

# --- 1. size -------------------------------------------------------------
$size = (Get-Item $exe).Length
if ($size -le $budgetBytes) { Pass 'binary size' ("{0:N0} bytes <= 1 MB" -f $size) }
else { Fail 'binary size' ("{0:N0} bytes > 1 MB" -f $size) }

# --- 2. image shape ------------------------------------------------------
$dumpbin = $null
$vswhere = 'C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe'
if (Test-Path $vswhere) {
    $vsPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
    if ($vsPath) {
        $dumpbin = Get-ChildItem (Join-Path $vsPath 'VC\Tools\MSVC') -Recurse -Filter dumpbin.exe -ErrorAction SilentlyContinue |
            Select-Object -First 1 -ExpandProperty FullName
    }
}

if ($dumpbin) {
    $headers = & $dumpbin /nologo /headers $exe
    if ($headers -match 'subsystem \(Windows GUI\)') { Pass 'subsystem' 'Windows GUI (no console window)' }
    else { Fail 'subsystem' 'not a GUI-subsystem image' }

    $deps = (& $dumpbin /nologo /dependents $exe) | Select-String -Pattern '^\s+\S+\.dll\s*$' | ForEach-Object { $_.ToString().Trim() }
    $osDlls = @('kernel32.dll','user32.dll','shell32.dll','gdi32.dll','advapi32.dll','comctl32.dll','ntdll.dll','msvcrt.dll','ucrtbase.dll')
    $foreign = $deps | Where-Object { $osDlls -notcontains $_.ToLower() -and $_ -notmatch '^api-ms-win-' }
    if ($foreign.Count -eq 0) { Pass 'dependencies' ("OS only: " + ($deps -join ', ')) }
    else { Fail 'dependencies' ("unexpected: " + ($foreign -join ', ')) }

    if ($deps -match 'vcruntime') { Fail 'static CRT' 'vcruntime DLL is still imported' }
    else { Pass 'static CRT' 'no VC++ redistributable needed' }
} else {
    Info 'image checks' 'dumpbin not found; skipped'
}

# --- 3. power API --------------------------------------------------------
# Start-Process -Wait is used rather than the call operator because a GUI-
# subsystem executable is not reliably awaited by PowerShell, which would make
# the exit code meaningless. Output is redirected so the report stays readable.
$stdout = [IO.Path]::GetTempFileName()
$stderr = [IO.Path]::GetTempFileName()
try {
    $diag = Start-Process -FilePath $exe -ArgumentList '--diagnose' -NoNewWindow -Wait -PassThru `
        -RedirectStandardOutput $stdout -RedirectStandardError $stderr
    if ($diag.ExitCode -eq 0) { Pass 'wake request' 'accepted by the OS' }
    else {
        Fail 'wake request' "SetThreadExecutionState refused (exit $($diag.ExitCode))"
        Get-Content $stdout | ForEach-Object { Info 'diagnose output' $_ }
    }
}
finally {
    Remove-Item $stdout, $stderr -Force -ErrorAction SilentlyContinue
}

# --- 4. runtime ----------------------------------------------------------
if ($StaticOnly) {
    Info 'runtime checks' 'skipped (-StaticOnly)'
} else {
    Add-Type @"
using System;
using System.Runtime.InteropServices;

public static class NoSleepProbe {
    [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern IntPtr FindWindow(string lpClassName, string lpWindowName);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool PostMessage(IntPtr hWnd, uint msg, IntPtr wParam, IntPtr lParam);

    [DllImport("shell32.dll")]
    public static extern int Shell_NotifyIconGetRect(ref NOTIFYICONIDENTIFIER identifier, out RECT location);

    [StructLayout(LayoutKind.Sequential)]
    public struct NOTIFYICONIDENTIFIER {
        public int cbSize;
        public IntPtr hWnd;
        public int uID;
        public Guid guidItem;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int Left, Top, Right, Bottom; }
}
"@

    Get-Process nosleep -ErrorAction SilentlyContinue | Stop-Process -Force
    Start-Sleep -Milliseconds 300

    $process = Start-Process -FilePath $exe -PassThru
    try {
        Start-Sleep -Milliseconds 800

        if ($process.HasExited) {
            Fail 'startup' 'process exited immediately'
        } else {
            Pass 'startup' 'running'

            $hwnd = [NoSleepProbe]::FindWindow('NoSleepTrayWindow', 'NoSleep')
            if ($hwnd -ne [IntPtr]::Zero) { Pass 'hidden window' 'created' }
            else { Fail 'hidden window' 'not found' }

            # A second launch must be a no-op.
            $second = Start-Process -FilePath $exe -PassThru
            Start-Sleep -Milliseconds 700
            $count = @(Get-Process nosleep -ErrorAction SilentlyContinue).Count
            if ($count -eq 1) { Pass 'single instance' 'second launch exited' }
            else { Fail 'single instance' "$count processes running" }
            if (-not $second.HasExited) { $second | Stop-Process -Force }

            # Ask the shell whether our icon is registered. This works even when
            # the icon is tucked into the overflow area.
            if ($hwnd -ne [IntPtr]::Zero) {
                $id = New-Object NoSleepProbe+NOTIFYICONIDENTIFIER
                $id.cbSize = [Runtime.InteropServices.Marshal]::SizeOf([type]'NoSleepProbe+NOTIFYICONIDENTIFIER')
                $id.hWnd = $hwnd
                $id.uID = 1
                $rect = New-Object NoSleepProbe+RECT
                $hr = [NoSleepProbe]::Shell_NotifyIconGetRect([ref]$id, [ref]$rect)
                if ($hr -eq 0) { Pass 'tray icon' ("registered at {0},{1}" -f $rect.Left, $rect.Top) }
                else { Fail 'tray icon' ("Shell_NotifyIconGetRect -> 0x{0:X8}" -f $hr) }

                # Exercise the click path: WM_APP+1 with NIN_SELECT in the low word.
                $wmTray = 0x8001
                $ninSelect = 0x0400
                [void][NoSleepProbe]::PostMessage($hwnd, $wmTray, [IntPtr]::Zero, [IntPtr]$ninSelect)
                Start-Sleep -Milliseconds 300
                if (-not $process.HasExited) { Pass 'toggle path' 'On transition survived' }
                else { Fail 'toggle path' 'process died on toggle' }

                [void][NoSleepProbe]::PostMessage($hwnd, $wmTray, [IntPtr]::Zero, [IntPtr]$ninSelect)
                Start-Sleep -Milliseconds 300
                if (-not $process.HasExited) { Pass 'toggle path' 'Off transition survived' }
                else { Fail 'toggle path' 'process died on toggle' }
            }

            $process.Refresh()
            $ramMb = $process.WorkingSet64 / 1MB
            if ($ramMb -le 10) { Pass 'working set' ("{0:N1} MB <= 10 MB" -f $ramMb) }
            else { Fail 'working set' ("{0:N1} MB > 10 MB" -f $ramMb) }

            $cpu0 = $process.TotalProcessorTime.TotalMilliseconds
            Start-Sleep -Seconds 2
            $process.Refresh()
            $cpuDelta = $process.TotalProcessorTime.TotalMilliseconds - $cpu0
            if ($cpuDelta -lt 20) { Pass 'idle CPU' ("{0:N0} ms over 2 s" -f $cpuDelta) }
            else { Fail 'idle CPU' ("{0:N0} ms over 2 s" -f $cpuDelta) }
        }
    }
    finally {
        if (-not $process.HasExited) { $process | Stop-Process -Force }
    }
}

Write-Host ("=" * 60)
if ($failures.Count -eq 0) {
    Write-Host "ALL CHECKS PASSED" -ForegroundColor Green
    exit 0
} else {
    Write-Host ("{0} CHECK(S) FAILED: {1}" -f $failures.Count, ($failures -join ', ')) -ForegroundColor Red
    exit 1
}
