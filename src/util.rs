//! Small shared helpers.

/// NUL-terminated UTF-16, the encoding every Win32 `W` function expects.
pub fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Copy `text` into a fixed-size wide buffer, always NUL-terminated.
pub fn copy_wide(dst: &mut [u16], text: &str) {
    let mut n = 0;
    for unit in text.encode_utf16() {
        if n + 1 >= dst.len() {
            break;
        }
        dst[n] = unit;
        n += 1;
    }
    dst[n] = 0;
}
