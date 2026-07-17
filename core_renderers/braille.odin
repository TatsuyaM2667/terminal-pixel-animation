package braille
@(export)
generate_braille_cells :: proc "c" (
	in_pixels: [^]u8,
	in_width: u32,
	in_height: u32,
	out_cells: [^]u8,
	target_width: u32,
	target_height: u32,
) {
	if target_width == 0 || target_height == 0 do return
	if in_width == 0 || in_height == 0 do return

	out_pixel_width := target_width * 2
	out_pixel_height := target_height * 4

	scale_x := f32(out_pixel_width) / f32(in_width)
	scale_y := f32(out_pixel_height) / f32(in_height)
	scale := scale_x if scale_x < scale_y else scale_y

	drawn_width := u32(f32(in_width) * scale)
	drawn_height := u32(f32(in_height) * scale)

	off_x := (out_pixel_width - drawn_width) / 2
	off_y := (out_pixel_height - drawn_height) / 2

	dot_flags := [4][2]u32{{0x01, 0x08}, {0x02, 0x10}, {0x04, 0x20}, {0x40, 0x80}}

	for cy in 0 ..< target_height {
		for cx in 0 ..< target_width {
			braille_char: u32 = 0x2800

			// Luminance-weighted color accumulators
			wr_sum: f32 = 0.0
			wg_sum: f32 = 0.0
			wb_sum: f32 = 0.0
			lum_total: f32 = 0.0

			for dy in 0 ..< 4 {
				for dx in 0 ..< 2 {
					px := cx * 2 + u32(dx)
					py := cy * 4 + u32(dy)

					if px >= off_x &&
					   px < off_x + drawn_width &&
					   py >= off_y &&
					   py < off_y + drawn_height {
						sx := u32(f32(px - off_x) / scale)
						sy := u32(f32(py - off_y) / scale)
						csx := sx if sx < in_width else in_width - 1
						csy := sy if sy < in_height else in_height - 1

						idx := (csy * in_width + csx) * 3
						r := in_pixels[idx]
						g := in_pixels[idx + 1]
						b := in_pixels[idx + 2]

						lum := 0.299 * f32(r) + 0.587 * f32(g) + 0.114 * f32(b)


						if lum > 35.0 {
							braille_char |= dot_flags[dy][dx]

							wr_sum += f32(r) * lum
							wg_sum += f32(g) * lum
							wb_sum += f32(b) * lum
							lum_total += lum
						}
					}
				}
			}

			final_r: u8 = 0
			final_g: u8 = 0
			final_b: u8 = 0

			if lum_total > 0.0 {
				avg_r := wr_sum / lum_total
				avg_g := wg_sum / lum_total
				avg_b := wb_sum / lum_total

				gray := 0.299 * avg_r + 0.587 * avg_g + 0.114 * avg_b
				boost: f32 = 1.25
				boosted_r := gray + (avg_r - gray) * boost
				boosted_g := gray + (avg_g - gray) * boost
				boosted_b := gray + (avg_b - gray) * boost

				if boosted_r < 0.0 do boosted_r = 0.0
				if boosted_r > 255.0 do boosted_r = 255.0
				if boosted_g < 0.0 do boosted_g = 0.0
				if boosted_g > 255.0 do boosted_g = 255.0
				if boosted_b < 0.0 do boosted_b = 0.0
				if boosted_b > 255.0 do boosted_b = 255.0

				final_r = u8(boosted_r)
				final_g = u8(boosted_g)
				final_b = u8(boosted_b)
			}

			out_idx := (cy * target_width + cx) * 8

			out_cells[out_idx + 0] = u8(braille_char & 0xFF)
			out_cells[out_idx + 1] = u8((braille_char >> 8) & 0xFF)
			out_cells[out_idx + 2] = u8((braille_char >> 16) & 0xFF)
			out_cells[out_idx + 3] = u8((braille_char >> 24) & 0xFF)

			out_cells[out_idx + 4] = final_r
			out_cells[out_idx + 5] = final_g
			out_cells[out_idx + 6] = final_b
			out_cells[out_idx + 7] = 0
		}
	}
}
