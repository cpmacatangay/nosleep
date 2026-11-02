//! Opt-in autostart.
//!
//! This is the only persistent change the program is allowed to make. It writes
//! a single value under `HKCU\...\CurrentVersion\Run`, only when the user ticks
//! "Start with Windows", and removes it when they untick it. No other registry
//! key is touched, and nothing is written by merely running the app.

use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
#[cfg(test)]
use windows_sys::Win32::System::Registry::RegDeleteKeyW;
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW,
    RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ,
};

use crate::util::wide;

/// The per-user startup key. `HKCU` means no administrator rights and no effect
/// on other accounts.
const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

/// Value name. Deliberately plain, so it is identifiable in Task Manager.
const VALUE_NAME: &str = "NoSleep";

/// Is autostart currently registered?
pub fn is_enabled() -> bool {
    stored_path().is_some()
}

/// The command line recorded in the registry, exactly as Windows will run it.
pub fn stored_path() -> Option<String> {
    read_value(RUN_KEY, VALUE_NAME)
}

/// Register this executable to run at sign-in.
pub fn enable() -> bool {
    match current_command() {
        Some(command) => write_value(RUN_KEY, VALUE_NAME, &command),
        None => false,
    }
}

/// Remove the entry. Succeeds when there was nothing to remove.
pub fn disable() -> bool {
    delete_value(RUN_KEY, VALUE_NAME)
}

/// Correct a stale path after the executable has been moved.
///
/// Windows stores an absolute path, so relocating the binary would otherwise
/// leave an entry pointing at nothing. Called once at startup; a single read in
/// the common case where the path is already right.
pub fn heal() {
    let Some(stored) = stored_path() else {
        return;
    };
    let Some(want) = current_command() else {
        return;
    };
    if !stored.eq_ignore_ascii_case(&want) {
        let _ = write_value(RUN_KEY, VALUE_NAME, &want);
    }
}

/// The current executable wrapped in quotes, so paths with spaces still parse
/// when Windows builds the command line.
fn current_command() -> Option<String> {
    let path = std::env::current_exe().ok()?;
    let text = path.to_str()?;
    if text.contains('"') {
        return None;
    }
    Some(format!("\"{text}\""))
}

// --- registry plumbing -----------------------------------------------------

fn open_key(key_path: &str, write: bool) -> Option<HKEY> {
    let path = wide(key_path);
    let mut key: HKEY = null_mut();
    let access = if write { KEY_READ | KEY_WRITE } else { KEY_READ };
    // SAFETY: `path` outlives the call; `key` is written only on success.
    let status = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, path.as_ptr(), 0, access, &mut key) };
    if status == ERROR_SUCCESS {
        Some(key)
    } else {
        None
    }
}

fn create_key(key_path: &str) -> Option<HKEY> {
    let path = wide(key_path);
    let mut key: HKEY = null_mut();
    let mut disposition = 0u32;
    // SAFETY: as above. A null security descriptor inherits the user's default.
    let status = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            path.as_ptr(),
            0,
            null(),
            0,
            KEY_READ | KEY_WRITE,
            null(),
            &mut key,
            &mut disposition,
        )
    };
    if status == ERROR_SUCCESS {
        Some(key)
    } else {
        None
    }
}

fn read_value(key_path: &str, value_name: &str) -> Option<String> {
    let key = open_key(key_path, false)?;
    let name = wide(value_name);

    let mut kind = 0u32;
    let mut size = 0u32;
    // SAFETY: sizes and handles are valid; a null data pointer asks for the size.
    let mut status = unsafe {
        RegQueryValueExW(key, name.as_ptr(), null(), &mut kind, null_mut(), &mut size)
    };
    if status != ERROR_SUCCESS || size == 0 || kind != REG_SZ {
        // SAFETY: `key` was opened by `open_key`.
        unsafe { RegCloseKey(key) };
        return None;
    }

    let mut buffer = vec![0u8; size as usize];
    // SAFETY: `buffer` is exactly `size` bytes, as required by the previous call.
    status = unsafe {
        RegQueryValueExW(
            key,
            name.as_ptr(),
            null(),
            &mut kind,
            buffer.as_mut_ptr(),
            &mut size,
        )
    };
    // SAFETY: the handle is closed exactly once.
    unsafe { RegCloseKey(key) };

    if status != ERROR_SUCCESS || kind != REG_SZ {
        return None;
    }

    let mut units: Vec<u16> = buffer
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    while units.last() == Some(&0) {
        units.pop();
    }
    String::from_utf16(&units).ok()
}

fn write_value(key_path: &str, value_name: &str, data: &str) -> bool {
    let Some(key) = create_key(key_path) else {
        return false;
    };
    let name = wide(value_name);

    let mut units: Vec<u16> = data.encode_utf16().collect();
    units.push(0);
    let bytes = (units.len() * 2) as u32;

    // SAFETY: the buffer lives for the duration of the call and is declared with
    // its true byte length, terminator included.
    let status = unsafe {
        RegSetValueExW(
            key,
            name.as_ptr(),
            0,
            REG_SZ,
            units.as_ptr() as *const u8,
            bytes,
        )
    };
    // SAFETY: the handle is closed exactly once.
    unsafe { RegCloseKey(key) };

    status == ERROR_SUCCESS
}

fn delete_value(key_path: &str, value_name: &str) -> bool {
    // A missing key means there is nothing stored, which is the desired state.
    let Some(key) = open_key(key_path, true) else {
        return true;
    };
    let name = wide(value_name);

    // SAFETY: valid handle and NUL-terminated name.
    let status = unsafe { RegDeleteValueW(key, name.as_ptr()) };
    // SAFETY: the handle is closed exactly once.
    unsafe { RegCloseKey(key) };

    status == ERROR_SUCCESS || status == ERROR_FILE_NOT_FOUND
}

/// Remove a whole key. Tests only: the product never deletes keys.
#[cfg(test)]
fn delete_key(key_path: &str) -> bool {
    let path = wide(key_path);
    // SAFETY: valid parent handle and NUL-terminated path.
    let status = unsafe { RegDeleteKeyW(HKEY_CURRENT_USER, path.as_ptr()) };
    status == ERROR_SUCCESS || status == ERROR_FILE_NOT_FOUND
}

#[cfg(test)]
mod tests {
    use super::{delete_key, delete_value, read_value, write_value};

    // A throwaway key outside the real Run key, so tests never touch autostart.
    const KEY_A: &str = "Software\\NoSleepAutostartTestA";
    const KEY_B: &str = "Software\\NoSleepAutostartTestB";
    const VALUE: &str = "Entry";

    #[test]
    fn round_trips_a_value() {
        let _ = delete_key(KEY_A);
        assert_eq!(read_value(KEY_A, VALUE), None, "starts empty");

        assert!(write_value(KEY_A, VALUE, "\"C:\\Program Files\\NoSleep\\NoSleep.exe\""));
        assert_eq!(
            read_value(KEY_A, VALUE).as_deref(),
            Some("\"C:\\Program Files\\NoSleep\\NoSleep.exe\"")
        );

        // Writing again replaces rather than duplicates.
        assert!(write_value(KEY_A, VALUE, "\"C:\\Other\\NoSleep.exe\""));
        assert_eq!(
            read_value(KEY_A, VALUE).as_deref(),
            Some("\"C:\\Other\\NoSleep.exe\"")
        );

        assert!(delete_value(KEY_A, VALUE));
        assert_eq!(read_value(KEY_A, VALUE), None);
        // Deleting again is still success: the desired state is "absent".
        assert!(delete_value(KEY_A, VALUE));

        assert!(delete_key(KEY_A), "cleanup");
    }

    #[test]
    fn reading_a_missing_value_is_none() {
        let _ = delete_key(KEY_B);
        assert_eq!(read_value(KEY_B, VALUE), None);
    }

    #[test]
    fn an_empty_value_is_not_mistaken_for_a_path() {
        let _ = delete_key(KEY_B);
        assert!(write_value(KEY_B, VALUE, ""));
        assert_eq!(read_value(KEY_B, VALUE).as_deref(), Some(""));
        assert!(delete_key(KEY_B), "cleanup");
    }

    #[test]
    fn the_real_entry_lives_in_the_per_user_run_key() {
        // The tests above exercise the plumbing with throwaway keys; this pins the
        // one string that decides where the product actually writes.
        assert_eq!(
            super::RUN_KEY,
            "Software\\Microsoft\\Windows\\CurrentVersion\\Run"
        );
        assert_eq!(super::VALUE_NAME, "NoSleep");
    }
}
