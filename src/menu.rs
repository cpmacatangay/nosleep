//! The right-click menu.
//!
//! Shown with `TPM_RETURNCMD` so the result is a plain value instead of a
//! `WM_COMMAND` message, which keeps the window procedure free of a second
//! dispatch path.

use windows_sys::Win32::Foundation::{HWND, POINT};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, DestroyMenu, GetCursorPos, PostMessageW, SetForegroundWindow,
    TrackPopupMenu, MF_CHECKED, MF_SEPARATOR, MF_STRING, MF_UNCHECKED, TPM_NONOTIFY, TPM_RETURNCMD,
    TPM_RIGHTBUTTON, WM_NULL,
};

use crate::util::wide;

pub const CMD_TOGGLE: usize = 1;
pub const CMD_QUIT: usize = 2;

/// Show the menu at the pointer and return the chosen command, or 0.
pub fn show(hwnd: HWND, awake: bool) -> usize {
    // SAFETY: every handle is created here and destroyed here; the wide strings
    // outlive the calls that borrow them.
    unsafe {
        let menu = CreatePopupMenu();
        if menu.is_null() {
            return 0;
        }

        let toggle_text = wide(if awake { "Keeping awake" } else { "Keep awake" });
        let quit_text = wide("Quit");
        let check = if awake { MF_CHECKED } else { MF_UNCHECKED };

        AppendMenuW(menu, MF_STRING | check, CMD_TOGGLE, toggle_text.as_ptr());
        AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
        AppendMenuW(menu, MF_STRING, CMD_QUIT, quit_text.as_ptr());

        let mut point = POINT { x: 0, y: 0 };
        GetCursorPos(&mut point);

        // Without this the menu can stay on screen after a click elsewhere.
        SetForegroundWindow(hwnd);
        let command = TrackPopupMenu(
            menu,
            TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTBUTTON,
            point.x,
            point.y,
            0,
            hwnd,
            std::ptr::null(),
        );
        PostMessageW(hwnd, WM_NULL, 0, 0);
        DestroyMenu(menu);

        command as usize
    }
}
