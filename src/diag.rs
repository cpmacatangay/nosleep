//! The single diagnostic affordance: `--diagnose`.
//!
//! It acquires and immediately releases the wake request and reports whether the
//! operating system accepted it. This makes the one thing that can silently fail
//! observable without keeping the machine awake.

use std::ptr::null_mut;

use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::Storage::FileSystem::{
    GetFileType, WriteFile, FILE_TYPE_CHAR, FILE_TYPE_DISK, FILE_TYPE_PIPE,
};
use windows_sys::Win32::System::Console::{GetStdHandle, STD_OUTPUT_HANDLE};
use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION, MB_OK};

use crate::power::PowerGuard;
use crate::util::wide;

pub fn run() -> i32 {
    let mut guard = PowerGuard::new();
    let accepted = guard.acquire();
    let last_error = guard.last_error();

    let report = format!(
        "NoSleep {}\n\
         \n\
         SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED)\n\
         held      : {}\n\
         accepted  : {}\n\
         last error: {}\n\
         \n\
         The request was released again; this command does not keep the PC awake.",
        env!("CARGO_PKG_VERSION"),
        if guard.is_held() { "yes" } else { "no" },
        if accepted { "yes" } else { "NO" },
        last_error
    );

    guard.release();
    emit(&report);

    if accepted {
        0
    } else {
        2
    }
}

/// Write to whatever stdout is — a console, a pipe, or a redirected file — and
/// fall back to a dialog so that double-clicking the executable still produces
/// an answer.
fn emit(text: &str) {
    // SAFETY: the handle is only used after its type has been identified; the
    // wide buffers outlive the message-box call.
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        let kind = if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            0
        } else {
            GetFileType(handle)
        };
        let writable =
            kind == FILE_TYPE_CHAR || kind == FILE_TYPE_PIPE || kind == FILE_TYPE_DISK;

        if writable {
            let mut written = 0u32;
            WriteFile(
                handle,
                text.as_ptr(),
                text.len() as u32,
                &mut written,
                null_mut(),
            );
        } else {
            let body = wide(text);
            let title = wide("NoSleep - diagnose");
            MessageBoxW(
                null_mut(),
                body.as_ptr(),
                title.as_ptr(),
                MB_OK | MB_ICONINFORMATION,
            );
        }
    }
}
