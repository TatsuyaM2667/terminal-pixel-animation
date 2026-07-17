use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Cargoが用意するビルド用の一時ディレクトリを取得
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir);

    // Odin のビルド (Braille レンダラ)
    println!("cargo:rerun-if-changed=core_renderers/braille.odin");

    let odin_lib_name = "braille";
    let odin_out_file = if cfg!(target_os = "windows") {
        out_path.join(format!("{}.lib", odin_lib_name))
    } else {
        out_path.join(format!("lib{}.a", odin_lib_name))
    };

    let status_odin = Command::new("odin")
        .args(&[
            "build",
            "core_renderers/braille.odin",
            "-file",              // ディレクトリではなく1ファイルを対象にする
            "-build-mode:static", // 静的ライブラリとして出力
            "-o:speed",           // 最適化 (高速化)
            &format!("-out:{}", odin_out_file.display()),
        ])
        .status()
        .expect("Failed to execute Odin compiler. Is 'odin' in your PATH?");

    assert!(status_odin.success(), "Odin compilation failed");

    // Zig のビルド (Half-block レンダラ)
    println!("cargo:rerun-if-changed=core_renderers/half_block.zig");

    let zig_lib_name = "halfblock";
    let zig_out_file = if cfg!(target_os = "windows") {
        out_path.join(format!("{}.lib", zig_lib_name))
    } else {
        out_path.join(format!("lib{}.a", zig_lib_name))
    };

    let status_zig = Command::new("zig")
        .args(&[
            "build-lib",
            "core_renderers/half_block.zig",
            "-O",
            "ReleaseFast", // 最適化 (高速化)
            "-lc",         // C標準ライブラリをリンク
            &format!("-femit-bin={}", zig_out_file.display()),
        ])
        .status()
        .expect("Failed to execute Zig compiler. Is 'zig' in your PATH?");

    assert!(status_zig.success(), "Zig compilation failed");

    // Cargo にライブラリのリンクを指示
    // コンパイルしたライブラリの検索パスを追加
    println!("cargo:rustc-link-search=native={}", out_dir);

    // Rustコンパイラに静的リンクを指示
    println!("cargo:rustc-link-lib=static={}", odin_lib_name);
    println!("cargo:rustc-link-lib=static={}", zig_lib_name);
}
