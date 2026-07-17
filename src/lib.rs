use std::fmt;
use std::os::raw::{c_int, c_uchar, c_uint};

/// レンダリング処理中に発生するエラー
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderError {
    /// 幅または高さに0が指定された場合のエラー
    InvalidDimensions,
    /// 予期せぬ内部エラー（将来の拡張用）
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

// --- FFI (Foreign Function Interface) 定義 ---
unsafe extern "C" {
    fn tvr_render_braille(
        in_pixels: *const c_uchar,
        in_width: c_uint,
        in_height: c_uint,
        out_cells: *mut c_uchar,
        target_width: c_uint,
        target_height: c_uint,
    ) -> c_int;

    fn tvr_render_halfblock(
        in_pixels: *const c_uchar,
        in_width: c_uint,
        in_height: c_uint,
        out_cells: *mut c_uchar,
        target_width: c_uint,
        target_height: c_uint,
    ) -> c_int;
}

// --- Rustユーザー向けの安全なラッパー関数 ---

/// Odinで実装されたBraille(点字)レンダラ
/// 成功すると1セルあたり8バイトのバッファを返します
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

    // 出力バッファの確保 (1セルあたり8バイト: UTF-8(4) + R(1) + G(1) + B(1) + 予約(1))
    let out_size = (target_width * target_height * 8) as usize;
    let mut out_cells = vec![0u8; out_size];

    let status = unsafe {
        tvr_render_braille(
            pixels.as_ptr(),
            in_width,
            in_height,
            out_cells.as_mut_ptr(),
            target_width,
            target_height,
        )
    };

    if status == 0 {
        Ok(out_cells)
    } else {
        Err(RenderError::InternalError)
    }
}

/// Zigで実装されたHalf-blockレンダラ
/// 成功すると1セルあたり6バイトのバッファを返します
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

    // 出力バッファの確保 (1セルあたり6バイト: R_fg, G_fg, B_fg, R_bg, G_bg, B_bg)
    let out_size = (target_width * target_height * 6) as usize;
    let mut out_cells = vec![0u8; out_size];

    let status = unsafe {
        tvr_render_halfblock(
            pixels.as_ptr(),
            in_width,
            in_height,
            out_cells.as_mut_ptr(),
            target_width,
            target_height,
        )
    };

    if status == 0 {
        Ok(out_cells)
    } else {
        Err(RenderError::InternalError)
    }
}
