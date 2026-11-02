//! Notification-area icon registration.

use std::mem::{size_of, zeroed};

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NIM_SETVERSION, NOTIFYICONDATAW,
};

use crate::icon::HIcon;
use crate::util::copy_wide;

/// One icon, so one identifier.
pub const ID: u32 = 1;

/// The version-4 callback contract. Version 4 is what gives us a distinct
/// left-click notification (`NIN_SELECT`) instead of guessing from button-up.
const VERSION_4: u32 = 4;

fn base(hwnd: HWND, callback: u32) -> NOTIFYICONDATAW {
    // SAFETY: NOTIFYICONDATAW is a plain-old-data struct with a zeroed default.
    let mut data: NOTIFYICONDATAW = unsafe { zeroed() };
    data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = hwnd;
    data.uID = ID;
    data.uCallbackMessage = callback;
    data
}

pub fn add(hwnd: HWND, callback: u32, icon: HIcon, tip: &str) -> bool {
    // SAFETY: `data` outlives the call and points at the live icon handle.
    unsafe {
        let mut data = base(hwnd, callback);
        data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_SHOWTIP;
        data.hIcon = icon;
        copy_wide(&mut data.szTip, tip);
        Shell_NotifyIconW(NIM_ADD, &data) != 0
    }
}

pub fn set_version(hwnd: HWND, callback: u32) -> bool {
    // SAFETY: as above; only the version union member is written.
    unsafe {
        let mut data = base(hwnd, callback);
        data.Anonymous.uVersion = VERSION_4;
        Shell_NotifyIconW(NIM_SETVERSION, &data) != 0
    }
}

pub fn update(hwnd: HWND, callback: u32, icon: HIcon, tip: &str) {
    // SAFETY: as above.
    unsafe {
        let mut data = base(hwnd, callback);
        data.uFlags = NIF_ICON | NIF_TIP | NIF_SHOWTIP;
        data.hIcon = icon;
        copy_wide(&mut data.szTip, tip);
        Shell_NotifyIconW(NIM_MODIFY, &data);
    }
}

pub fn remove(hwnd: HWND, callback: u32) {
    // SAFETY: deleting an icon that may not exist is harmless.
    unsafe {
        let data = base(hwnd, callback);
        Shell_NotifyIconW(NIM_DELETE, &data);
    }
}
