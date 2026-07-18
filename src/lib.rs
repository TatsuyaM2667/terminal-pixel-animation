//! # web-pixel-animation
//!
//! Render pixel images as Unicode characters in the terminal with True Color support.
//!
//! This library provides two rendering backends implemented in high-performance systems languages:
//!
//! - **Braille renderer** (Odin): Maps each terminal cell to a 2x4 Braille dot pattern (U+2800~),
//!   achieving 8 pixels per cell with luminance-weighted color averaging and saturation boosting.
//! - **Half-block renderer** (Zig): Uses the `▀` (U+2580) character to pack 2 vertically stacked
//!   pixels per cell with independent foreground/background RGB colors.
//!
//! Both renderers maintain aspect ratio with letterboxing and support ANSI True Color output.
//!
//! # Quick Start
//!
//! ```no_run
//! use terminal_pixel_animation::{render_braille, print_braille_to_terminal};
//!
//! // Assume `pixels` is an RGB8 buffer, `width` x `height`
//! # let pixels: Vec<u8> = vec![0; 100 * 100 * 3];
//! # let (width, height) = (100u32, 100u32);
//! let cells = render_braille(&pixels, width, height, 80, 24).unwrap();
//! print_braille_to_terminal(&cells, 80, 24);
//! ```

use std::fmt;
use std::os::raw::{c_uchar, c_uint};

#[cfg(not(target_arch = "wasm32"))]
use std::io::{self, Write};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Errors that can occur during rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderError {
    /// Width or height parameter was zero.
    InvalidDimensions,
    /// An unexpected internal error occurred.
    InternalError,
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::InvalidDimensions => write!(f, "Width or height cannot be zero"),
            RenderError::InternalError => write!(f, "An internal rendering error occurred"),
        }
    }
}

impl std::error::Error for RenderError {}

#[cfg(target_arch = "wasm32")]
impl From<RenderError> for JsValue {
    fn from(e: RenderError) -> JsValue {
        JsValue::from_str(&e.to_string())
    }
}

// ---------------------------------------------------------------------------
// FFI bindings
// ---------------------------------------------------------------------------

unsafe extern "C" {
    fn generate_braille_cells(
        in_pixels: *const c_uchar,
        in_width: c_uint,
        in_height: c_uint,
        out_cells: *mut c_uchar,
        target_width: c_uint,
        target_height: c_uint,
    );

    fn generate_terminal_cells(
        in_pixels: *const c_uchar,
        in_width: c_uint,
        in_height: c_uint,
        out_cells: *mut c_uchar,
        target_width: c_uint,
        target_height: c_uint,
    );
}

// ---------------------------------------------------------------------------
// Core rendering functions
// ---------------------------------------------------------------------------

/// Render an RGB8 pixel buffer into Braille Unicode cells.
///
/// Each terminal cell maps to a 2x4 pixel region encoded as a single Braille
/// character (U+2800–U+28FF) with a luminance-weighted average RGB color.
///
/// # Output format
///
/// Each cell occupies **8 bytes**: `[utf8_0, utf8_1, utf8_2, utf8_3, R, G, B, _]`
/// - Bytes 0–3: The Braille character encoded as up to 4 bytes of UTF-8.
/// - Bytes 4–6: The averaged RGB color for the active dots.
/// - Byte 7: Reserved (zero).
///
/// # Errors
///
/// Returns [`RenderError::InvalidDimensions`] if any dimension is zero.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn render_braille(
    pixels: &[u8],
    in_width: u32,
    in_height: u32,
    target_width: u32,
    target_height: u32,
) -> Result<Vec<u8>, RenderError> {
    if target_width == 0 || target_height == 0 || in_width == 0 || in_height == 0 {
        return Err(RenderError::InvalidDimensions);
    }

    let out_size = (target_width * target_height * 8) as usize;
    let mut out_cells = vec![0u8; out_size];

    unsafe {
        generate_braille_cells(
            pixels.as_ptr(),
            in_width,
            in_height,
            out_cells.as_mut_ptr(),
            target_width,
            target_height,
        );
    }

    Ok(out_cells)
}

/// Render an RGB8 pixel buffer into half-block terminal cells.
///
/// Each terminal cell maps to a 1x2 pixel region. The top pixel becomes the
/// foreground color and the bottom pixel becomes the background color, rendered
/// using the `▀` (U+2580) character.
///
/// # Output format
///
/// Each cell occupies **6 bytes**: `[R_fg, G_fg, B_fg, R_bg, G_bg, B_bg]`
///
/// # Errors
///
/// Returns [`RenderError::InvalidDimensions`] if any dimension is zero.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn render_half_block(
    pixels: &[u8],
    in_width: u32,
    in_height: u32,
    target_width: u32,
    target_height: u32,
) -> Result<Vec<u8>, RenderError> {
    if target_width == 0 || target_height == 0 || in_width == 0 || in_height == 0 {
        return Err(RenderError::InvalidDimensions);
    }

    let out_size = (target_width * target_height * 6) as usize;
    let mut out_cells = vec![0u8; out_size];

    unsafe {
        generate_terminal_cells(
            pixels.as_ptr(),
            in_width,
            in_height,
            out_cells.as_mut_ptr(),
            target_width,
            target_height,
        );
    }

    Ok(out_cells)
}

// ---------------------------------------------------------------------------
// Terminal output helpers (native only — meaningless on wasm32-unknown-unknown)
// ---------------------------------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
fn le_bytes_to_char(bytes: &[u8]) -> Option<char> {
    if bytes.len() < 4 {
        return None;
    }
    let code_point = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    char::from_u32(code_point)
}

/// Print Braille cell data to stdout using ANSI True Color escape codes.
#[cfg(not(target_arch = "wasm32"))]
pub fn print_braille_to_terminal(cells: &[u8], target_width: u32, target_height: u32) {
    let mut out = io::stdout();
    let _ = write!(out, "\x1b[2J\x1b[H");

    for cy in 0..target_height {
        for cx in 0..target_width {
            let idx = ((cy * target_width + cx) * 8) as usize;

            let ch = le_bytes_to_char(&cells[idx..idx + 4]);

            let r = cells[idx + 4];
            let g = cells[idx + 5];
            let b = cells[idx + 6];

            match ch {
                Some(c) if c != '\0' && c != ' ' => {
                    let _ = write!(out, "\x1b[38;2;{};{};{}m{}", r, g, b, c);
                }
                _ => {
                    let _ = write!(out, " ");
                }
            }
        }
        let _ = writeln!(out, "\x1b[0m");
    }
    let _ = out.flush();
}

/// Print half-block cell data to stdout using ANSI True Color escape codes.
#[cfg(not(target_arch = "wasm32"))]
pub fn print_halfblock_to_terminal(cells: &[u8], target_width: u32, target_height: u32) {
    let mut out = io::stdout();
    let _ = write!(out, "\x1b[2J\x1b[H");

    for cy in 0..target_height {
        for cx in 0..target_width {
            let idx = ((cy * target_width + cx) * 6) as usize;

            let r_fg = cells[idx];
            let g_fg = cells[idx + 1];
            let b_fg = cells[idx + 2];
            let r_bg = cells[idx + 3];
            let g_bg = cells[idx + 4];
            let b_bg = cells[idx + 5];

            let _ = write!(
                out,
                "\x1b[48;2;{};{};{}m\x1b[38;2;{};{};{}m\u{2580}",
                r_fg, g_fg, b_fg, r_bg, g_bg, b_bg
            );
        }
        let _ = writeln!(out, "\x1b[0m");
    }
    let _ = out.flush();
}
