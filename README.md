# terminal-pixel-animation

ターミナル上でTrue Colorをサポートし、Unicode文字としてピクセル画像を描画するRustライブラリです。

このライブラリは、C FFI経由でRustから利用可能な、高速なシステムプログラミング言語（OdinとZig）で記述された2つのレンダリングバックエンドを使用し、RGBピクセルデータをターミナルフレンドリーなUnicodeアートに変換します。

## 特徴

| レンダラー | 言語 | 1セルあたりのピクセル | 解像度 | 用途 |
| --- | --- | --- | --- | --- |
| **点字 (Braille)** | Odin | 2x4 (8 px/cell) | 高 | 詳細な静止画、サムネイル |
| **ハーフブロック** | Zig | 1x2 (2 px/cell) | 中 | 高フレームレートの動画再生 |

* アスペクト比を維持したリサイズ機能（レターボックス付き）
* ANSI True Color (24-bit) 出力ヘルパー
* 輝度加重による色平均化と彩度ブースト（点字モードのみ）

## インストール

`Cargo.toml` に以下を追加してください。

```toml
[dependencies]
terminal-pixel-animation = "0.1"

```

> **注意:** ビルド時に `odin`、`zig`、および `objcopy` コンパイラが PATH に通っている必要があります。

## クイックスタート

```rust
use terminal_pixel_animation::{render_braille, print_braille_to_terminal};

// RGB8ピクセルバッファ (幅 * 高さ * 3バイト)
let pixels: Vec<u8> = load_your_image();
let (width, height) = (320u32, 240u32);

// 点字セルにレンダリング (80列 x 30行)
let cells = render_braille(&pixels, width, height, 80, 30).unwrap();

// ターミナルに出力
print_braille_to_terminal(&cells, 80, 30);

```

## API

### レンダリング

```rust
// 点字: 1セルあたり8バイト [utf8;4, R, G, B, _]
let cells = render_braille(&pixels, w, h, cols, rows)?;

// ハーフブロック: 1セルあたり6バイト [R_fg, G_fg, B_fg, R_bg, G_bg, B_bg]
let cells = render_half_block(&pixels, w, h, cols, rows)?;

```

### ターミナル出力

```rust
print_braille_to_terminal(&cells, cols, rows);
print_halfblock_to_terminal(&cells, cols, rows);

```

### カスタムレンダリング

ライブラリは生のセルバッファを返すため、独自の出力方法を実装することも可能です。

```rust
let cells = render_braille(&pixels, w, h, 80, 24)?;

for row in 0..24 {
    for col in 0..80 {
        let idx = ((row * 80 + col) * 8) as usize;

        // 点字のUTF-8文字をデコード
        let code_point = u32::from_le_bytes(cells[idx..idx+4].try_into().unwrap());
        let ch = char::from_u32(code_point).unwrap_or(' ');

        // RGBカラーを読み取り
        let (r, g, b) = (cells[idx+4], cells[idx+5], cells[idx+6]);

        // あらゆるANSI対応レンダラーで使用可能...
    }
}

```

## ソースからのビルド

```bash
cargo build --release

```

ビルドスクリプトがOdinおよびZigのソースファイルを静的ライブラリとしてコンパイルし、Rustクレートへ自動的にリンクします。

### 依存関係

* [Rust](https://rustup.rs/) (edition 2024)
* [Odin compiler](https://odin-lang.org/)
* [Zig compiler](https://ziglang.org/)
* `objcopy` (binutils)

## サンプル実行

```bash
cargo run --example demo -- path/to/image.png braille
cargo run --example demo -- path/to/image.png halfblock

```

## ライセンス

MIT
