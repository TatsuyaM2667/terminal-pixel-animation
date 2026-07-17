use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir);

    // ============================================================
    // Odin のビルド (Braille レンダラ)
    // ============================================================
    println!("cargo:rerun-if-changed=core_renderers/braille.odin");

    let odin_lib_name = "braille";
    let odin_obj_file = out_path.join(format!("{}.o", odin_lib_name));
    let odin_out_file = if cfg!(target_os = "windows") {
        out_path.join(format!("{}.lib", odin_lib_name))
    } else {
        out_path.join(format!("lib{}.a", odin_lib_name))
    };

    // Step 1: オブジェクトファイルとしてビルド
    let status_odin = Command::new("odin")
        .args([
            "build",
            "core_renderers/braille.odin",
            "-file",
            "-build-mode:obj",
            "-o:speed",
            "-reloc-mode:pic",
            &format!("-out:{}", odin_obj_file.display()),
        ])
        .status()
        .expect("Failed to execute Odin compiler. Is 'odin' in your PATH?");
    assert!(status_odin.success(), "Odin compilation failed");

    // Step 2: main シンボルをリネームしてリンカ衝突を回避
    let fixed_obj = out_path.join(format!("{}_fixed.o", odin_lib_name));
    let status_rename = Command::new("objcopy")
        .args([
            "--redefine-sym",
            "main=__odin_runtime_main",
            odin_obj_file.to_str().unwrap(),
            fixed_obj.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to execute objcopy. Is 'objcopy' in your PATH?");
    assert!(status_rename.success(), "objcopy rename failed");

    // Step 3: 静的ライブラリを作成
    let status_ar = Command::new("ar")
        .args([
            "rcs",
            odin_out_file.to_str().unwrap(),
            fixed_obj.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to execute ar");
    assert!(status_ar.success(), "ar failed");

    // ============================================================
    // Zig のビルド (Half-block レンダラ)
    // ============================================================
    println!("cargo:rerun-if-changed=core_renderers/half_block.zig");

    let zig_lib_name = "halfblock";
    let zig_out_file = if cfg!(target_os = "windows") {
        out_path.join(format!("{}.lib", zig_lib_name))
    } else {
        out_path.join(format!("lib{}.a", zig_lib_name))
    };

    let status_zig = Command::new("zig")
        .args([
            "build-lib",
            "core_renderers/half_block.zig",
            "-O",
            "ReleaseFast",
            "-lc",
            &format!("-femit-bin={}", zig_out_file.display()),
        ])
        .status()
        .expect("Failed to execute Zig compiler. Is 'zig' in your PATH?");
    assert!(status_zig.success(), "Zig compilation failed");

    // ============================================================
    // リンク設定
    // ============================================================
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static={}", odin_lib_name);
    println!("cargo:rustc-link-lib=static={}", zig_lib_name);
}
