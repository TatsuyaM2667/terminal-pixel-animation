# terminal-pixel-animation

Render pixel images as Unicode characters in the terminal with True Color support.

This library converts RGB pixel data into terminal-friendly Unicode art using two rendering backends written in high-performance systems languages (Odin and Zig), exposed to Rust via C FFI.

## Features

| Renderer | Language | Cells per pixel | Resolution | Best for |
|---|---|---|---|---|
| **Braille** | Odin | 2x4 (8 px/cell) | High | Detailed still images, thumbnails |
| **Half-block** | Zig | 1x2 (2 px/cell) | Medium | High frame-rate video playback |

- Aspect-ratio-preserving resize with letterboxing
- ANSI True Color (24-bit) output helpers
- Luminance-weighted color averaging with saturation boost (Braille mode)

## Installation

```toml
[dependencies]
terminal-pixel-animation = "0.1"
```

> **Note:** Requires `odin`, `zig`, and `objcopy` compilers in your `PATH` at build time.

## Quick Start

```rust
use terminal_pixel_animation::{render_braille, print_braille_to_terminal};

// Your RGB8 pixel buffer (width * height * 3 bytes)
let pixels: Vec<u8> = load_your_image();
let (width, height) = (320u32, 240u32);

// Render into Braille cells (80 columns x 30 rows)
let cells = render_braille(&pixels, width, height, 80, 30).unwrap();

// Print to terminal
print_braille_to_terminal(&cells, 80, 30);
```

## API

### Rendering

```rust
// Braille: 8 bytes per cell [utf8;4, R, G, B, _]
let cells = render_braille(&pixels, w, h, cols, rows)?;

// Half-block: 6 bytes per cell [R_fg, G_fg, B_fg, R_bg, G_bg, B_bg]
let cells = render_half_block(&pixels, w, h, cols, rows)?;
```

### Terminal Output

```rust
print_braille_to_terminal(&cells, cols, rows);
print_halfblock_to_terminal(&cells, cols, rows);
```

### Custom Rendering

The library returns raw cell buffers, so you can implement your own output:

```rust
let cells = render_braille(&pixels, w, h, 80, 24)?;

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
