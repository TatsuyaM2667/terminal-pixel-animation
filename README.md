for row in 0..24 {
    for col in 0..80 {
        let idx = ((row * 80 + col) * 8) as usize;

        // Decode the Braille UTF-8 character
        let code_point = u32::from_le_bytes(cells[idx..idx+4].try_into().unwrap());
        let ch = char::from_u32(code_point).unwrap_or(' ');

        // Read the RGB color
        let (r, g, b) = (cells[idx+4], cells[idx+5], cells[idx+6]);

        // Use with any ANSI-capable renderer...
    }
}
```

## Building from Source

```bash
cargo build --release
```

The build script compiles Odin and Zig source files into static libraries and links them into the Rust crate automatically.

### Dependencies

- [Rust](https://rustup.rs/) (edition 2024)
- [Odin compiler](https://odin-lang.org/)
- [Zig compiler](https://ziglang.org/)
- `objcopy` (binutils)

## Example

```bash
cargo run --example demo -- path/to/image.png braille
cargo run --example demo -- path/to/image.png halfblock
```

## License

MIT
