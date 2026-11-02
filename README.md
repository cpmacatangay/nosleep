# NoSleep

[![release](https://img.shields.io/github/v/release/cpmacatangay/nosleep?label=download)](https://github.com/cpmacatangay/nosleep/releases/latest)
[![build](https://img.shields.io/github/actions/workflow/status/cpmacatangay/nosleep/release.yml?label=build)](https://github.com/cpmacatangay/nosleep/actions)

**v0.1.1** · Windows 10/11 (x64) · one executable under 1 MB · no installer

One click keeps your PC awake. Built for laptops whose sleep timeout is set by
policy, where every sleep costs a re-login and the work you had open.

## Install

```powershell
$d = "$env:LOCALAPPDATA\NoSleep"
New-Item -ItemType Directory -Force $d | Out-Null
Invoke-WebRequest https://github.com/cpmacatangay/nosleep/releases/latest/download/NoSleep-win64.zip -OutFile "$env:TEMP\NoSleep.zip"
Expand-Archive "$env:TEMP\NoSleep.zip" -DestinationPath $d -Force
& "$d\NoSleep.exe"
```

Or take the zip from [Releases](https://github.com/cpmacatangay/nosleep/releases/latest)
and run `NoSleep.exe` in place — from a folder, a USB stick, anywhere.

> If Windows says it protected your PC, a browser downloaded the file. Run
> `Unblock-File .\NoSleep.exe` first. The PowerShell route above never triggers it.

## Use

**Left-click** the tray icon to toggle. **Right-click** for the menu.

| State | Icon | Meaning |
| --- | --- | --- |
| Off | hollow grey ring | normal sleep |
| On | solid green disc | display and system stay awake |

Windows 11 hides new tray icons: click `^` and drag it onto the taskbar.

## How it works

While On, NoSleep holds
`SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED)`
— the same mechanism a video call uses. The display stays on, so the idle lock
screen never appears.

When Off — or when the process exits, crashes, or is killed — the request is
released and normal power behaviour resumes immediately.

### Limits

| | |
| --- | --- |
| Security policy | never touches power plans, policy, or the registry |
| Enforced inactivity lock | if IT set one, the screen locks anyway |
| Lid close, critical battery | firmware and Windows decide; NoSleep does not interfere |
| Autostart | none; launch it yourself |

## Verify

```powershell
.\NoSleep.exe --diagnose     # exit 0 = the OS accepted the wake request
```

For the real thing: unplug the charger, turn it On, leave it alone past the sleep
timeout, then confirm no sleep event was recorded.

```powershell
Get-WinEvent -FilterHashtable @{LogName='System'; Id=42} -MaxEvents 5 |
  Select-Object TimeCreated
```

## Build

Needs Rust (MSVC) and the Visual C++ build tools.

```powershell
cargo test --release
cargo build --release
powershell -ExecutionPolicy Bypass -File .\scripts\verify.ps1
```

`verify.ps1` checks the size budget, GUI subsystem, import table, wake request,
single instance, tray registration, memory and idle CPU. `-StaticOnly` skips the
parts that need an interactive desktop.

`.\scripts\package.ps1` writes a portable folder to `dist\NoSleep\` and a versioned
zip beside it.

## Release

Bump the version in one command, then tag:

```powershell
.\scripts\set-version.ps1 0.1.2
git commit -am "chore: release v0.1.2"
git tag v0.1.2
git push origin master --tags
```

`.github/workflows/release.yml` builds on `windows-latest`, runs the tests and
static checks, and attaches `NoSleep-win64.zip` to the release. It fails if the
tag, `Cargo.toml`, and the version at the top of this file disagree.

## Design

- one dependency (`windows-sys`); no GUI framework, no async runtime
- the wake request is owned by an RAII guard, so no exit path can strand it
- hidden top-level window, so the tray icon survives an Explorer restart
- statically linked CRT — no Visual C++ Redistributable needed
- icons rasterised at the shell's real pixel size; no resource compiler

Specification and history: [`specs/001-keep-awake/`](specs/001-keep-awake/).
Principles: [`.specify/memory/constitution.md`](.specify/memory/constitution.md).

## Licence

MIT
