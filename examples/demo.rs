//! Demo: render an image file in the terminal using Braille or half-block mode.
//!
//! Usage:
//!   cargo run --example demo -- <image_path> [braille|halfblock]
//!
//! Requires a True Color capable terminal (e.g. iTerm2, kitty, Alacritty, Windows Terminal).

use std::env;
use std::io::Write;
use std::process;

use image::GenericImageView;
use terminal_pixel_animation::{render_braille, render_half_block, print_braille_to_terminal, print_halfblock_to_terminal};

fn main() {
    let args: Vec<String> = env::args().collect();

    let (path, mode) = match args.len() {
        2 => (args[1].clone(), "braille".to_string()),
        3 => (args[1].clone(), args[2].clone()),
        _ => {
            eprintln!("Usage: {} <image_path> [braille|halfblock]", args[0]);
            eprintln!("  Renders an image in the terminal using Unicode characters.");
            process::exit(1);
        }
    };

    let img = match image::open(&path) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("Failed to open image '{}': {}", path, e);
            process::exit(1);
        }
    };

    let (width, height) = img.dimensions();
    let rgb_img = img.to_rgb8();
    let pixels = rgb_img.as_raw();

    // ターミナルサイズを取得 (fallback: 80x24)
    let (term_w, term_h) = get_terminal_size();

    // 出力セル数: brailleは2x4ドットなので幅は半分、halfblockは1x2ドット
    let (cell_w, cell_h) = match mode.as_str() {
        "braille" => (term_w / 2, term_h / 4),
        "halfblock" => (term_w, term_h / 2),
        _ => {
            eprintln!("Unknown mode: {}. Use 'braille' or 'halfblock'.", mode);
            process::exit(1);
        }
    };

    if cell_w == 0 || cell_h == 0 {
        eprintln!("Terminal too small to render.");
        process::exit(1);
    }

    // ターミナルをクリアしてカーソルを隠す
    print!("\x1b[2J\x1b[?25l");

    match mode.as_str() {
        "braille" => {
            match render_braille(pixels, width, height, cell_w, cell_h) {
                Ok(cells) => print_braille_to_terminal(&cells, cell_w, cell_h),
                Err(e) => {
                    eprintln!("Braille render error: {}", e);
                    restore_terminal();
                    process::exit(1);
                }
            }
        }
        "halfblock" => {
            match render_half_block(pixels, width, height, cell_w, cell_h) {
                Ok(cells) => print_halfblock_to_terminal(&cells, cell_w, cell_h),
                Err(e) => {
                    eprintln!("Half-block render error: {}", e);
                    restore_terminal();
                    process::exit(1);
                }
            }
        }
        _ => unreachable!(),
    }

    // Enterキーを待ってからターミナルを復元
    eprintln!("\n\x1b[?25hPress Enter to exit...");
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);
    restore_terminal();
}

fn restore_terminal() {
    print!("\x1b[?25h\x1b[0m");
    let _ = std::io::stdout().flush();
}

fn get_terminal_size() -> (u32, u32) {
    // TIOCGWINSZ ioctl を使ってターミナルサイズを取得
    #[repr(C)]
    struct Winsize {
        ws_row: u16,
        ws_col: u16,
        ws_xpixel: u16,
        ws_ypixel: u16,
    }

    unsafe {
        let mut ws: Winsize = std::mem::zeroed();
        let ret = libc::ioctl(1, libc::TIOCGWINSZ, &mut ws);
        if ret == 0 && ws.ws_col > 0 && ws.ws_row > 0 {
            return (ws.ws_col as u32, ws.ws_row as u32);
        }
    }

    (80, 24)
}
