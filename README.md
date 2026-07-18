# terminal-pixel-animation

[![crates.io](https://img.shields.io/crates/v/terminal-pixel-animation.svg)](https://crates.io/crates/terminal-pixel-animation)
[![npm (core)](https://img.shields.io/npm/v/terminal-pixel-animation.svg)](https://www.npmjs.com/package/terminal-pixel-animation)
[![npm (react)](https://img.shields.io/npm/v/terminal-pixel-animation-react.svg)](https://www.npmjs.com/package/terminal-pixel-animation-react)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Render pixel images as Unicode characters in the terminal with True Color support.
Also available as a WebAssembly module for browser use with React hooks.

RGBピクセル画像をUnicode文字に変換し、ターミナルやブラウザにTrue Colorで表示するライブラリ。React Hooks対応。

## Features

| Renderer | Language | Cells per pixel | Resolution | Best for |
|---|---|---|---|---|
| **Braille** | Odin | 2x4 (8 px/cell) | High | Detailed still images, thumbnails |
| **Half-block** | Zig | 1x2 (2 px/cell) | Medium | High frame-rate video playback |

- Aspect-ratio-preserving resize with letterboxing
- ANSI True Color (24-bit) output helpers
- Luminance-weighted color averaging with saturation boost (Braille mode)
- **WebAssembly** target for browser use via `wasm-pack` / `wasm-bindgen`
- **React Hooks** (`WasmProvider`, `useBraille`, `useHalfBlock`)

アスペクト比を保ったリサイズ、ANSI True Color出力、WebAssembly対応、React Hooks対応。

---

## Installation

### Rust (crates.io)

```toml
[dependencies]
terminal-pixel-animation = "0.2"
```

### npm

```bash
# Core WASM module / WASMコアモジュール
npm install terminal-pixel-animation

# React hooks (optional) / React Hooks（オプション）
npm install terminal-pixel-animation-react
```

> **Build dependencies:** Requires `odin`, `zig`, `objcopy`, and `wasm-pack` in your `PATH`.
>
> **ビルド時の依存関係:** Rust, Odin, Zig, objcopy (binutils), wasm-pack が必要です。

---

## Quick Start

### Native (Terminal)

```rust
use terminal_pixel_animation::{render_braille, print_braille_to_terminal};

// RGB8 pixel buffer (width * height * 3 bytes)
let pixels: Vec<u8> = load_your_image();
let (width, height) = (320u32, 240u32);

// Render into Braille cells (80 columns x 30 rows)
let cells = render_braille(&pixels, width, height, 80, 30).unwrap();

// Print to terminal
print_braille_to_terminal(&cells, 80, 30);
```

### WASM (Browser)

```js
import init, { render_braille, render_half_block } from "terminal-pixel-animation";

await init();

// RGB8 pixel buffer as Uint8Array
const cells = render_braille(rgb, videoWidth, videoHeight, cols, rows);

// Decode Braille cells
for (let i = 0; i < cells.length; i += 8) {
    const cp = cells[i] | (cells[i+1] << 8) | (cells[i+2] << 16) | (cells[i+3] << 24);
    const ch = String.fromCodePoint(cp);
    const r = cells[i+4], g = cells[i+5], b = cells[i+6];
    // Draw ch with color rgb(r, g, b) on a canvas...
}
```

### React

```tsx
import { useState, useEffect } from "react";
import { WasmProvider, useBraille } from "terminal-pixel-animation-react";

function App() {
  return (
    <WasmProvider>
      <PixelCanvas />
    </WasmProvider>
  );
}

function PixelCanvas() {
  const [pixels, setPixels] = useState<Uint8Array | null>(null);
  const { decoded, loading } = useBraille(pixels, 320, 240, 80, 30);

  if (loading) return <p>Loading WASM...</p>;
  if (!decoded) return null;

  return (
    <pre style={{ fontFamily: "monospace", lineHeight: "1", fontSize: "10px" }}>
      {decoded.map((cell, i) => (
        <span key={i} style={{ color: `rgb(${cell.r},${cell.g},${cell.b})` }}>
          {cell.char}
        </span>
      ))}
    </pre>
  );
}
```

---

## WASM Guide

### Setup

```bash
npm init -y
npm install terminal-pixel-animation
```

### Browser Usage

```js
import init, { render_braille, render_half_block } from "terminal-pixel-animation";

// Initialize WASM (call once)
await init();

// Get RGB buffer from webcam
const canvas = document.createElement("canvas");
const ctx = canvas.getContext("2d");
canvas.width = video.videoWidth;
canvas.height = video.videoHeight;
ctx.drawImage(video, 0, 0);
const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);

// Convert RGBA to RGB
const rgb = new Uint8Array(canvas.width * canvas.height * 3);
for (let i = 0, j = 0; i < imageData.data.length; i += 4, j += 3) {
    rgb[j] = imageData.data[i];
    rgb[j + 1] = imageData.data[i + 1];
    rgb[j + 2] = imageData.data[i + 2];
}

// Braille rendering
const cells = render_braille(rgb, canvas.width, canvas.height, 80, 30);

// Draw on canvas
const displayCanvas = document.getElementById("output");
const displayCtx = displayCanvas.getContext("2d");
displayCanvas.width = 80 * 8;
displayCanvas.height = 30 * 14;

for (let i = 0; i < cells.length; i += 8) {
    const cp = cells[i] | (cells[i+1] << 8) | (cells[i+2] << 16) | (cells[i+3] << 24);
    const ch = String.fromCodePoint(cp);
    const r = cells[i+4], g = cells[i+5], b = cells[i+6];

    if (ch !== "\0" && ch !== " ") {
        const col = (i / 8) % 80;
        const row = Math.floor((i / 8) / 80);
        displayCtx.fillStyle = `rgb(${r},${g},${b})`;
        displayCtx.font = "14px monospace";
        displayCtx.fillText(ch, col * 8, row * 14);
    }
}
```

### Half-block Rendering

```js
const cells = render_half_block(rgb, canvas.width, canvas.height, 80, 30);

// Cell is 6 bytes: [R_fg, G_fg, B_fg, R_bg, G_bg, B_bg]
for (let i = 0; i < cells.length; i += 6) {
    const rFg = cells[i], gFg = cells[i+1], bFg = cells[i+2];
    const rBg = cells[i+3], gBg = cells[i+4], bBg = cells[i+5];

    const col = (i / 6) % 80;
    const row = Math.floor((i / 6) / 80);

    // Upper half (foreground)
    displayCtx.fillStyle = `rgb(${rFg},${gFg},${bFg})`;
    displayCtx.fillRect(col * 8, row * 14, 8, 7);

    // Lower half (background)
    displayCtx.fillStyle = `rgb(${rBg},${gBg},${bBg})`;
    displayCtx.fillRect(col * 8, row * 14 + 7, 8, 7);
}
```

---

## React Guide

### Setup

```bash
npx create-vite@latest my-app --template react-ts
cd my-app
npm install terminal-pixel-animation terminal-pixel-animation-react
npm run dev
```

### WasmProvider

Wrap your app with `WasmProvider` at the root. The WASM module is loaded once.

```tsx
import { WasmProvider } from "terminal-pixel-animation-react";

function App() {
  return (
    <WasmProvider>
      <MyComponent />
    </WasmProvider>
  );
}
```

### useBraille Hook

```tsx
import { useBraille } from "terminal-pixel-animation-react";

function MyComponent() {
  const { cells, decoded, loading, error } = useBraille(pixels, 320, 240, 80, 30);

  if (loading) return <p>Loading...</p>;
  if (error) return <p>Error: {error.message}</p>;
  if (!decoded) return null;

  // decoded: [{ char, r, g, b }, ...] — flat array, row-major order
  return (
    <pre style={{ fontFamily: "monospace", lineHeight: "1", fontSize: "10px" }}>
      {decoded.map((cell, i) => (
        <span key={i} style={{ color: `rgb(${cell.r},${cell.g},${cell.b})` }}>
          {cell.char}
        </span>
      ))}
    </pre>
  );
}
```

### useHalfBlock Hook

```tsx
import { useHalfBlock } from "terminal-pixel-animation-react";

function MyComponent() {
  const { decoded } = useHalfBlock(pixels, 320, 240, 80, 30);

  // decoded: [{ rFg, gFg, bFg, rBg, gBg, bBg }, ...]
  return (
    <div>
      {decoded?.map((cell, i) => (
        <div
          key={i}
          style={{
            display: "inline-block",
            width: 8,
            height: 14,
            background: `linear-gradient(to bottom, rgb(${cell.rFg},${cell.gFg},${cell.bFg}) 50%, rgb(${cell.rBg},${cell.gBg},${cell.bBg}) 50%)`,
          }}
        />
      ))}
    </div>
  );
}
```

### Real-time Webcam Rendering

```tsx
import { useState, useEffect, useRef } from "react";
import { WasmProvider, useBraille } from "terminal-pixel-animation-react";

function App() {
  return (
    <WasmProvider>
      <WebcamDemo />
    </WasmProvider>
  );
}

function WebcamDemo() {
  const videoRef = useRef<HTMLVideoElement>(null);
  const [pixels, setPixels] = useState<Uint8Array | null>(null);
  const [size, setSize] = useState({ w: 0, h: 0 });

  useEffect(() => {
    navigator.mediaDevices.getUserMedia({ video: true })
      .then(stream => {
        if (videoRef.current) {
          videoRef.current.srcObject = stream;
          videoRef.current.play();
        }
      });
  }, []);

  useEffect(() => {
    const canvas = document.createElement("canvas");
    const ctx = canvas.getContext("2d")!;

    const capture = () => {
      const video = videoRef.current;
      if (!video || video.readyState < 2) return;

      canvas.width = video.videoWidth;
      canvas.height = video.videoHeight;
      ctx.drawImage(video, 0, 0);
      const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);

      const rgb = new Uint8Array(canvas.width * canvas.height * 3);
      for (let i = 0, j = 0; i < imageData.data.length; i += 4, j += 3) {
        rgb[j] = imageData.data[i];
        rgb[j + 1] = imageData.data[i + 1];
        rgb[j + 2] = imageData.data[i + 2];
      }

      setPixels(rgb);
      setSize({ w: canvas.width, h: canvas.height });
    };

    const id = requestAnimationFrame(function loop() {
      capture();
      requestAnimationFrame(loop);
    });
    return () => cancelAnimationFrame(id);
  }, []);

  return (
    <div>
      <video ref={videoRef} autoPlay playsInline style={{ display: "none" }} />
      <PixelDisplay pixels={pixels} width={size.w} height={size.h} />
    </div>
  );
}

function PixelDisplay({ pixels, width, height }: {
  pixels: Uint8Array | null; width: number; height: number;
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const { decoded } = useBraille(pixels, width, height, 100, 45);

  useEffect(() => {
    if (!decoded || !canvasRef.current) return;
    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d")!;

    canvas.width = 100 * 8;
    canvas.height = 45 * 14;
    ctx.fillStyle = "#000";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.font = "14px monospace";
    ctx.textBaseline = "top";

    for (let i = 0; i < decoded.length; i++) {
      const cell = decoded[i];
      if (cell.char !== "\0" && cell.char !== " ") {
        const col = i % 100;
        const row = Math.floor(i / 100);
        ctx.fillStyle = `rgb(${cell.r},${cell.g},${cell.b})`;
        ctx.fillText(cell.char, col * 8, row * 14);
      }
    }
  }, [decoded]);

  return <canvas ref={canvasRef} style={{ background: "#000" }} />;
}
```

---

## API Reference

### Rust (Native)

```rust
// Braille: 8 bytes per cell [utf8;4, R, G, B, _]
let cells = render_braille(&pixels, w, h, cols, rows)?;

// Half-block: 6 bytes per cell [R_fg, G_fg, B_fg, R_bg, G_bg, B_bg]
let cells = render_half_block(&pixels, w, h, cols, rows)?;

// Terminal output
print_braille_to_terminal(&cells, cols, rows);
print_halfblock_to_terminal(&cells, cols, rows);
```

### WASM (JavaScript)

```js
// Braille: Uint8Array, 8 bytes per cell
const cells = render_braille(rgb, in_width, in_height, target_cols, target_rows);

// Half-block: Uint8Array, 6 bytes per cell
const cells = render_half_block(rgb, in_width, in_height, target_cols, target_rows);
```

### React Hooks

| Hook | Arguments | Returns |
|---|---|---|
| `useBraille(pixels, w, h, cols, rows)` | `Uint8Array \| null`, 4 numbers | `{ cells, decoded, loading, error }` |
| `useHalfBlock(pixels, w, h, cols, rows)` | `Uint8Array \| null`, 4 numbers | `{ cells, decoded, loading, error }` |

---

## Building from Source

### Dependencies

- [Rust](https://rustup.rs/) (edition 2024)
- [Odin compiler](https://odin-lang.org/)
- [Zig compiler](https://ziglang.org/)
- `objcopy` (binutils)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/) (for WASM builds)

### Native

```bash
cargo build --release
```

### WASM

```bash
wasm-pack build --target bundler --out-dir pkg
```

### React hooks

```bash
cd react && npm install && npm run build
```

---

## Examples

### Native

```bash
cargo run --example demo -- path/to/image.png braille
cargo run --example demo -- path/to/image.png halfblock
```

### WASM

```bash
python3 -m http.server 8080
# Open http://localhost:8080/pkg/wasm-demo.html in your browser
```

Supports webcam live feed and video file playback.

---

## Publishing

### npm

```bash
npm login
cd pkg && npm publish
cd ../react && npm publish
```

### crates.io

```bash
cargo login <your-api-token>
cargo publish
```

---

## License

MIT
