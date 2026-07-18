use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir);
    let is_wasm = env::var("CARGO_CFG_TARGET_ARCH").unwrap() == "wasm32";

    println!("cargo:rerun-if-changed=core_renderers/braille.odin");
    println!("cargo:rerun-if-changed=core_renderers/half_block.zig");

    // ---- Odin (Braille) ----
    let odin_obj_file = out_path.join("braille.o");
    let odin_out_file = out_path.join("libbraille.a");

    let mut odin_args = vec![
        "build".into(),
        "core_renderers/braille.odin".into(),
        "-file".into(),
        "-build-mode:obj".into(),
        "-o:speed".into(),
        format!("-out:{}", odin_obj_file.display()),
    ];
    if is_wasm {
        odin_args.push("-target:freestanding_wasm32".into());
        // -reloc-mode:pic はELF/PE的な概念なのでwasmでは付けない
    } else {
        odin_args.push("-reloc-mode:pic".into());
    }
    let status = Command::new("odin")
        .args(&odin_args)
        .status()
        .expect("odin not found in PATH");
    assert!(status.success(), "Odin compilation failed");

    if is_wasm {
        // まずrenameステップなしでアーカイブ化。duplicate symbolが出たら要調査。
        let status = Command::new("zig")
            .args([
                "ar",
                "rcs",
                odin_out_file.to_str().unwrap(),
                odin_obj_file.to_str().unwrap(),
            ])
            .status()
            .expect("zig not found in PATH");
        assert!(status.success(), "zig ar failed (braille)");
    } else {
        let fixed_obj = out_path.join("braille_fixed.o");
        let status = Command::new("objcopy")
            .args([
                "--redefine-sym",
                "main=__odin_runtime_main",
                odin_obj_file.to_str().unwrap(),
                fixed_obj.to_str().unwrap(),
            ])
            .status()
            .expect("objcopy not found in PATH");
        assert!(status.success(), "objcopy rename failed");
        let status = Command::new("ar")
            .args([
                "rcs",
                odin_out_file.to_str().unwrap(),
                fixed_obj.to_str().unwrap(),
            ])
            .status()
            .expect("ar not found in PATH");
        assert!(status.success(), "ar failed (braille)");
    }

    // ---- Zig (Half-block) ----
    let zig_out_file = out_path.join("libhalfblock.a");
    if is_wasm {
        let zig_obj_file = out_path.join("halfblock.o");
        let status = Command::new("zig")
            .args([
                "build-obj",
                "core_renderers/half_block.zig",
                "-target",
                "wasm32-freestanding",
                "-O",
                "ReleaseFast",
                &format!("-femit-bin={}", zig_obj_file.display()),
            ])
            // -lc は外す: freestandingにはlibcが無いし、この関数はlibc不要
            .status()
            .expect("zig not found in PATH");
        assert!(status.success(), "Zig compilation failed");
        let status = Command::new("zig")
            .args([
                "ar",
                "rcs",
                zig_out_file.to_str().unwrap(),
                zig_obj_file.to_str().unwrap(),
            ])
            .status()
            .expect("zig ar failed");
        assert!(status.success(), "zig ar failed (halfblock)");
    } else {
        let status = Command::new("zig")
            .args([
                "build-lib",
                "core_renderers/half_block.zig",
                "-O",
                "ReleaseFast",
                "-lc",
                &format!("-femit-bin={}", zig_out_file.display()),
            ])
            .status()
            .expect("zig not found in PATH");
        assert!(status.success(), "Zig compilation failed");
    }

    println!("cargo:rustc-link-search=native={out_dir}");
    println!("cargo:rustc-link-lib=static=braille");
    println!("cargo:rustc-link-lib=static=halfblock");
}
