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

