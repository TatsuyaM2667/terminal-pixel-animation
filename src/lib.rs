// src/lib.rs
use std::os::raw::{c_uchar, c_uint, c_void};

// --- FFI (Foreign Function Interface) 定義 ---
extern "C" {
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

/// OdinのBrailleレンダラ
pub fn render_braille(
    pixels: &[u8],
    in_width: u32,
    in_height: u32,
    target_width: u32,
    target_height: u32,
) -> Vec<u8> {
    // 出力バッファの確保 (1セルあたり8バイト)
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
    out_cells
}

/// ZigのHalf-blockレンダラ
pub fn render_half_block(
    pixels: &[u8],
    in_width: u32,
    in_height: u32,
    target_width: u32,
    target_height: u32,
) -> Vec<u8> {
    // 出力バッファの確保 (1セルあたり6バイト)
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
    out_cells
}
