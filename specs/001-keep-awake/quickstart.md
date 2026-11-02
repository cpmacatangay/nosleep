# Quickstart: NoSleep

## Build

```powershell
cargo build --release
```

Produces `target\release\nosleep.exe` (~216 KB).

## Run

```powershell
.\target\release\nosleep.exe
```

The process stays in the notification area. It writes nothing and opens no window.

## Automated checks

```powershell
cargo test --release                                  # state machine + icon rasteriser
powershell -ExecutionPolicy Bypass -File .\scripts\verify.ps1
```

`verify.ps1` asserts: size ≤ 1 MB; GUI subsystem; imports limited to OS DLLs; no
`vcruntime` dependency; wake request accepted; process starts; a second instance
exits; the shell reports the tray icon as registered; the toggle path survives;
working set ≤ 10 MB; idle CPU ≈ 0.

## Acceptance protocol (target machine)

This is the part that cannot be automated without administrator rights, and the
part that actually matters. Run it on the machine whose sleep policy you are
working around, **on battery**.

| # | Step | Pass condition |
|---|---|---|
| 1 | `nosleep.exe --diagnose` | `accepted : yes`, exit code 0 |
| 2 | Start the app, unplug the charger, click the icon once (icon turns solid green) | Icon changes, tooltip reads "keeping this PC awake" |
| 3 | Do not touch the machine for 1.5× the configured sleep timeout | Screen stays on; no lock screen; session intact |
| 4 | `Get-WinEvent -FilterHashtable @{LogName='System'; Id=42; StartTime=(Get-Date).AddHours(-1)}` | No sleep event during the window |
| 5 | Click the icon once (icon returns to the ring) and leave it | Display turns off and the machine sleeps within one timeout |
| 6 | Start the app again, turn it On, then kill it from Task Manager | Power behaviour returns to normal immediately |
| 7 | Start it twice | Only one tray icon; the second process exits |

### Definitive check, if you have an elevated prompt

While the app is On, in an **administrator** terminal:

```powershell
powercfg /requests
```

`nosleep.exe` should be listed under both `DISPLAY:` and `SYSTEM:`.

This is the fastest objective proof, but it needs elevation, which the app itself
never asks for.

## Known limitations

- The tray icon is hidden in the Windows 11 overflow area by default. Promote it
  once via *Settings → Taskbar → Other system tray icons*.
- If the machine's policy enforces an inactivity lock, the screen will lock anyway.
  No user-mode application can prevent that, and NoSleep does not try.
- Lid close and critical-battery hibernation are unaffected by design.
- The executable is unsigned. Copy it via USB or run `Unblock-File .\nosleep.exe`
  to avoid the SmartScreen "unrecognized app" prompt on first launch.
