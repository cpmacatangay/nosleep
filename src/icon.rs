//! Tray artwork, drawn at the exact pixel size the shell will use.
//!
//! Generating the icons instead of shipping `.ico` files keeps the build free of
//! resource compilation and lets the artwork be rendered crisply at whatever the
//! current display scaling asks for. The two states differ in shape (solid disc
//! versus hollow ring) as well as colour, so they remain distinguishable without
//! relying on colour perception.

use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::ptr::null_mut;

use windows_sys::Win32::Graphics::Gdi::{
    CreateBitmap, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC,
    BITMAPINFO, BITMAPV5HEADER, BI_RGB, DIB_RGB_COLORS, HGDIOBJ,
};
use windows_sys::Win32::UI::HiDpi::{GetDpiForSystem, GetSystemMetricsForDpi};
use windows_sys::Win32::UI::WindowsAndMessaging::{CreateIconIndirect, ICONINFO, SM_CXSMICON};

pub type HIcon = *mut c_void;

/// Amber disc for On, neutral ring for Off.
pub const AWAKE_RGB: (u8, u8, u8) = (240, 168, 48);
pub const ASLEEP_RGB: (u8, u8, u8) = (150, 156, 164);

#[derive(Clone, Copy)]
pub enum Style {
    Disc,
    Ring,
}

/// The size, in pixels, that the notification area will actually draw.
pub fn tray_size() -> i32 {
    // SAFETY: both calls are parameterless queries.
    unsafe {
        let dpi = GetDpiForSystem();
        let size = GetSystemMetricsForDpi(SM_CXSMICON, dpi);
        if (8..=96).contains(&size) {
            size
        } else {
            16
        }
    }
}

/// Build an `HICON` by rasterising the requested shape.
pub fn render(size: i32, style: Style, rgb: (u8, u8, u8)) -> HIcon {
    if size <= 0 {
        return null_mut();
    }
    // SAFETY: all handles are checked; every early return cleans up what it made.
    unsafe {
        let screen = GetDC(null_mut());
        if screen.is_null() {
            return null_mut();
        }
        let dc = CreateCompatibleDC(screen);

        let mut header: BITMAPV5HEADER = zeroed();
        header.bV5Size = size_of::<BITMAPV5HEADER>() as u32;
        header.bV5Width = size;
        // Negative height selects a top-down DIB, matching the raster order below.
        header.bV5Height = -size;
        header.bV5Planes = 1;
        header.bV5BitCount = 32;
        header.bV5Compression = BI_RGB;

        let mut bits: *mut c_void = null_mut();
        let colour = CreateDIBSection(
            dc,
            &header as *const BITMAPV5HEADER as *const BITMAPINFO,
            DIB_RGB_COLORS,
            &mut bits,
            null_mut(),
            0,
        );
        if colour.is_null() || bits.is_null() {
            DeleteDC(dc);
            ReleaseDC(null_mut(), screen);
            return null_mut();
        }

        paint(bits as *mut u8, size, style, rgb);

        // A monochrome mask is still required by CreateIconIndirect even when
        // the colour bitmap carries an alpha channel.
        let mask = CreateBitmap(size, size, 1, 1, std::ptr::null());
        if mask.is_null() {
            DeleteObject(colour as HGDIOBJ);
            DeleteDC(dc);
            ReleaseDC(null_mut(), screen);
            return null_mut();
        }

        let info = ICONINFO {
            fIcon: 1,
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: mask,
            hbmColor: colour,
        };
        let icon = CreateIconIndirect(&info);

        DeleteObject(colour as HGDIOBJ);
        DeleteObject(mask as HGDIOBJ);
        DeleteDC(dc);
        ReleaseDC(null_mut(), screen);
        icon
    }
}

/// Rasterise with 4x4 supersampling so the circle edge is smooth at 16 px.
fn paint(bits: *mut u8, size: i32, style: Style, (r, g, b): (u8, u8, u8)) {
    let s = size as f32;
    let centre = s / 2.0;
    let outer = s * 0.46;
    let inner = s * 0.27;
    const SAMPLES: i32 = 4;
    let per_sample = 1.0 / (SAMPLES * SAMPLES) as f32;

    for y in 0..size {
        for x in 0..size {
            let mut coverage = 0.0f32;
            for sy in 0..SAMPLES {
                for sx in 0..SAMPLES {
                    let px = x as f32 + (sx as f32 + 0.5) / SAMPLES as f32;
                    let py = y as f32 + (sy as f32 + 0.5) / SAMPLES as f32;
                    let dx = px - centre;
                    let dy = py - centre;
                    let distance = (dx * dx + dy * dy).sqrt();
                    let inside = match style {
                        Style::Disc => distance <= outer,
                        Style::Ring => distance <= outer && distance >= inner,
                    };
                    if inside {
                        coverage += per_sample;
                    }
                }
            }

            // Premultiplied BGRA, as the DIB section expects.
            let index = ((y * size + x) * 4) as usize;
            let alpha = coverage;
            // SAFETY: index is inside the buffer allocated for size*size*4 bytes.
            unsafe {
                *bits.add(index) = (b as f32 * alpha) as u8;
                *bits.add(index + 1) = (g as f32 * alpha) as u8;
                *bits.add(index + 2) = (r as f32 * alpha) as u8;
                *bits.add(index + 3) = (255.0 * alpha) as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{paint, Style};

    const SIZE: i32 = 16;

    fn pixels(style: Style) -> Vec<u8> {
        let mut buffer = vec![0u8; (SIZE * SIZE * 4) as usize];
        paint(buffer.as_mut_ptr(), SIZE, style, (255, 255, 255));
        buffer
    }

    fn alpha_at(buffer: &[u8], x: i32, y: i32) -> u8 {
        buffer[((y * SIZE + x) * 4 + 3) as usize]
    }

    #[test]
    fn disc_is_solid_in_the_middle() {
        let buffer = pixels(Style::Disc);
        assert_eq!(alpha_at(&buffer, 8, 8), 255, "disc centre must be opaque");
        assert_eq!(alpha_at(&buffer, 0, 0), 0, "corner must be transparent");
    }

    #[test]
    fn ring_is_hollow_in_the_middle() {
        let buffer = pixels(Style::Ring);
        assert_eq!(alpha_at(&buffer, 8, 8), 0, "ring centre must be transparent");
        assert!(
            alpha_at(&buffer, 8, 1) > 200,
            "ring edge must be near-opaque"
        );
        assert_eq!(alpha_at(&buffer, 0, 0), 0, "corner must be transparent");
    }

    #[test]
    fn the_two_states_do_not_look_alike() {
        let disc = pixels(Style::Disc);
        let ring = pixels(Style::Ring);
        assert_ne!(disc, ring, "On and Off artwork must differ");
    }

    #[test]
    fn colours_are_premultiplied() {
        let mut buffer = vec![0u8; (SIZE * SIZE * 4) as usize];
        paint(buffer.as_mut_ptr(), SIZE, Style::Disc, (0, 0, 255));
        let centre = ((8 * SIZE + 8) * 4) as usize;
        assert_eq!(buffer[centre], 255, "blue channel");
        assert_eq!(buffer[centre + 1], 0, "green channel");
        assert_eq!(buffer[centre + 2], 0, "red channel");
        assert_eq!(buffer[centre + 3], 255, "alpha channel");
    }
}
