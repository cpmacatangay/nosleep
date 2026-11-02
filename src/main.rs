//! NoSleep - a one-click wake lock for Windows.
//!
//! Entering this module: a hidden top-level window, one tray icon, and one thread
//! that owns the wake request. Nothing else.

#![windows_subsystem = "windows"]

mod app;
mod autostart;
mod diag;
mod icon;
mod menu;
mod power;
mod state;
mod tray;
mod util;

use std::cell::{Cell, RefCell};
use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE, HWND, LPARAM, LRESULT, WPARAM,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassW, RegisterWindowMessageW, TranslateMessage, MSG, WNDCLASSW, WM_APP, WM_DESTROY,
    WM_ENDSESSION, WM_QUERYENDSESSION, WM_TIMER, WS_POPUP,
};

use app::{Action, App};

const WM_TRAY: u32 = WM_APP + 1;

thread_local! {
    /// Single-threaded by design: the window procedure and the wake request must
    /// live on the same thread for `SetThreadExecutionState` to apply.
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
    static TASKBAR_CREATED: Cell<u32> = const { Cell::new(0) };
}

fn with_app<R>(f: impl FnOnce(&mut App) -> R) -> Option<R> {
    APP.with(|cell| cell.borrow_mut().as_mut().map(f))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--diagnose") {
        std::process::exit(diag::run());
    }

    // SAFETY: called before any window exists, which is the documented
    // requirement for setting DPI awareness.
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }

    // The guard is held for the whole process lifetime; a second launch returns
    // here and exits without creating a second icon.
    let _instance = match SingleInstance::acquire() {
        Some(guard) => guard,
        None => return,
    };

    // SAFETY: single-threaded Win32 setup and message loop.
    unsafe {
        run();
    }
}

unsafe fn run() {
    let instance = GetModuleHandleW(null());
    let class_name = util::wide("NoSleepTrayWindow");
    let window_name = util::wide("NoSleep");

    let taskbar_text = util::wide("TaskbarCreated");
    let taskbar_message = RegisterWindowMessageW(taskbar_text.as_ptr());
    TASKBAR_CREATED.with(|cell| cell.set(taskbar_message));

    let window_class = WNDCLASSW {
        style: 0,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: null_mut(),
        hCursor: null_mut(),
        hbrBackground: null_mut(),
        lpszMenuName: null(),
        lpszClassName: class_name.as_ptr(),
    };
    RegisterClassW(&window_class);

    // A hidden *top-level* window, deliberately not a message-only window: only
    // top-level windows receive the TaskbarCreated broadcast, and without it the
    // icon could never come back after Explorer restarts.
    let hwnd = CreateWindowExW(
        0,
        class_name.as_ptr(),
        window_name.as_ptr(),
        WS_POPUP,
        0,
        0,
        0,
        0,
        null_mut(),
        null_mut(),
        instance,
        null(),
    );
    if hwnd.is_null() {
        return;
    }

    APP.with(|cell| {
        let mut slot = cell.borrow_mut();
        *slot = Some(App::new());
        if let Some(app) = slot.as_mut() {
            app.attach(hwnd, WM_TRAY);
        }
    });

    // If the executable was moved since autostart was enabled, point the entry at
    // where it lives now. A single read when autostart is off.
    autostart::heal();

    let mut message: MSG = std::mem::zeroed();
    while GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
        TranslateMessage(&message);
        DispatchMessageW(&message);
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let taskbar_message = TASKBAR_CREATED.with(|cell| cell.get());
    if taskbar_message != 0 && message == taskbar_message {
        with_app(|app| app.re_add());
        return 0;
    }

    match message {
        WM_TRAY => {
            let action = with_app(|app| app.on_tray_event(lparam as u32)).unwrap_or(Action::None);
            match action {
                Action::Toggle => {
                    with_app(|app| app.toggle());
                }
                Action::ShowMenu => {
                    let awake = with_app(|app| app.state.is_on()).unwrap_or(false);
                    // Read fresh, so the tick reflects the registry rather than a
                    // cached flag that could have drifted.
                    let autostart_on = autostart::is_enabled();
                    // The menu runs a nested message loop, so it is shown with no
                    // borrow of the app state held.
                    match menu::show(hwnd, awake, autostart_on) {
                        menu::CMD_TOGGLE => {
                            with_app(|app| app.toggle());
                        }
                        menu::CMD_AUTOSTART => {
                            // A failure leaves the stored state unchanged; the next
                            // time the menu opens it shows the truth.
                            if autostart_on {
                                let _ = autostart::disable();
                            } else {
                                let _ = autostart::enable();
                            }
                        }
                        menu::CMD_QUIT => quit(hwnd),
                        _ => {}
                    }
                }
                Action::None => {}
            }
            0
        }
        WM_QUERYENDSESSION => 1,
        WM_TIMER => {
            with_app(|app| app.on_timer(wparam));
            0
        }
        WM_ENDSESSION => {
            with_app(|app| app.shutdown());
            0
        }
        WM_DESTROY => {
            with_app(|app| app.shutdown());
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

unsafe fn quit(hwnd: HWND) {
    with_app(|app| app.shutdown());
    DestroyWindow(hwnd);
}

/// Keeps a named mutex alive so only one copy of the program can run.
struct SingleInstance(HANDLE);

impl SingleInstance {
    fn acquire() -> Option<Self> {
        // SAFETY: the name buffer outlives the call; the returned handle is
        // owned by the guard.
        unsafe {
            let name = util::wide("Local\\NoSleep.SingleInstance.v1");
            let handle = CreateMutexW(null(), 0, name.as_ptr());
            if handle.is_null() {
                // Could not even create a mutex; running anyway is friendlier
                // than refusing to work.
                return Some(Self(handle));
            }
            if GetLastError() == ERROR_ALREADY_EXISTS {
                CloseHandle(handle);
                return None;
            }
            Some(Self(handle))
        }
    }
}

impl Drop for SingleInstance {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: the handle was created by us and is closed exactly once.
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
}
