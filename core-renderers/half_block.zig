const std = @import("std");

export fn generate_terminal_cells(
    in_pixels: [*]const u8,
    in_width: u32,
    in_height: u32,
    out_cells: [*]u8,
    target_width: u32,
    target_height: u32,
) void {
    if (target_width == 0 or target_height == 0) return;
    if (in_width == 0 or in_height == 0) return;

    // 出力ピクセルサイズ: 幅 = target_width, 高さ = target_height * 2 (1セル = 縦2ピクセル)
    const out_pixel_width = target_width;
    const out_pixel_height = target_height * 2;

    const scale_x: f32 = @as(f32, @floatFromInt(out_pixel_width)) / @as(f32, @floatFromInt(in_width));
    const scale_y: f32 = @as(f32, @floatFromInt(out_pixel_height)) / @as(f32, @floatFromInt(in_height));
    const scale = if (scale_x < scale_y) scale_x else scale_y;

    const drawn_width: u32 = @intFromFloat(@as(f32, @floatFromInt(in_width)) * scale);
    const drawn_height: u32 = @intFromFloat(@as(f32, @floatFromInt(in_height)) * scale);

    const off_x: u32 = (out_pixel_width - drawn_width) / 2;
    const off_y: u32 = (out_pixel_height - drawn_height) / 2;

    var cy: u32 = 0;
    while (cy < target_height) : (cy += 1) {
        const py_top: u32 = cy * 2;
        const py_bot: u32 = cy * 2 + 1;

        var cx: u32 = 0;
        while (cx < target_width) : (cx += 1) {
            const px: u32 = cx;

            var r_fg: u8 = 0;
            var g_fg: u8 = 0;
            var b_fg: u8 = 0;
            var r_bg: u8 = 0;
            var g_bg: u8 = 0;
            var b_bg: u8 = 0;

            // Top Pixel
            if (px >= off_x and px < off_x + drawn_width and py_top >= off_y and py_top < off_y + drawn_height) {
                const sx: u32 = @intFromFloat(@as(f32, @floatFromInt(px - off_x)) / scale);
                const sy: u32 = @intFromFloat(@as(f32, @floatFromInt(py_top - off_y)) / scale);
                const clamp_sx = if (sx >= in_width) in_width - 1 else sx;
                const clamp_sy = if (sy >= in_height) in_height - 1 else sy;
                const idx = (clamp_sy * in_width + clamp_sx) * 3;
                r_fg = in_pixels[idx];
                g_fg = in_pixels[idx + 1];
                b_fg = in_pixels[idx + 2];
            }

            // Bottom Pixel
            if (px >= off_x and px < off_x + drawn_width and py_bot >= off_y and py_bot < off_y + drawn_height) {
                const sx: u32 = @intFromFloat(@as(f32, @floatFromInt(px - off_x)) / scale);
                const sy: u32 = @intFromFloat(@as(f32, @floatFromInt(py_bot - off_y)) / scale);
                const clamp_sx = if (sx >= in_width) in_width - 1 else sx;
                const clamp_sy = if (sy >= in_height) in_height - 1 else sy;
                const idx = (clamp_sy * in_width + clamp_sx) * 3;
                r_bg = in_pixels[idx];
                g_bg = in_pixels[idx + 1];
                b_bg = in_pixels[idx + 2];
            }

            const out_idx = (cy * target_width + cx) * 6;
            out_cells[out_idx] = r_fg;
            out_cells[out_idx + 1] = g_fg;
            out_cells[out_idx + 2] = b_fg;
            out_cells[out_idx + 3] = r_bg;
            out_cells[out_idx + 4] = g_bg;
            out_cells[out_idx + 5] = b_bg;
        }
    }
}
