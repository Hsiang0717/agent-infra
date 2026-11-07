use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};
use imageproc::drawing::draw_hollow_rect_mut;
use imageproc::rect::Rect;

/// Default tile boundary alignment factor (e.g. 64px)
pub const TILE_ALIGN_MULTIPLE: u32 = 64;
/// Default max dimension for single screenshot tile optimization (1024px)
pub const DEFAULT_MAX_DIMENSION: u32 = 1024;

/// Optimize an image for Vision Transformer (ViT) tile consumption:
/// 1. Scales down proportionally if width or height exceeds `max_dim`.
/// 2. Snaps dimensions to multiples of `TILE_ALIGN_MULTIPLE` to eliminate fragmented patches.
/// 3. Preserves aspect ratio with zero distortion.
pub fn optimize_for_vit_tiles(img: &DynamicImage, max_dim: u32) -> DynamicImage {
    let (orig_w, orig_h) = img.dimensions();
    if orig_w == 0 || orig_h == 0 {
        return img.clone();
    }

    let mut target_w = orig_w;
    let mut target_h = orig_h;

    if orig_w > max_dim || orig_h > max_dim {
        let ratio = (max_dim as f64) / (orig_w.max(orig_h) as f64);
        target_w = ((orig_w as f64 * ratio).round() as u32).max(1);
        target_h = ((orig_h as f64 * ratio).round() as u32).max(1);
    }

    // Align to nearest multiple of TILE_ALIGN_MULTIPLE (at least 1 multiple)
    let aligned_w = ((target_w + TILE_ALIGN_MULTIPLE / 2) / TILE_ALIGN_MULTIPLE * TILE_ALIGN_MULTIPLE)
        .max(TILE_ALIGN_MULTIPLE);
    let aligned_h = ((target_h + TILE_ALIGN_MULTIPLE / 2) / TILE_ALIGN_MULTIPLE * TILE_ALIGN_MULTIPLE)
        .max(TILE_ALIGN_MULTIPLE);

    if aligned_w == orig_w && aligned_h == orig_h {
        img.clone()
    } else {
        img.resize_exact(aligned_w, aligned_h, image::imageops::FilterType::Lanczos3)
    }
}

/// Overlay hierarchical dual-grid onto image:
/// - Macro Grid: 4x4 regions named A1..D4 (bold, distinct high-contrast borders & labels).
/// - Micro Grid: 3x3 sub-regions within each macro cell (subtle, semi-transparent dashed lines).
pub fn overlay_hierarchical_grid(img: &DynamicImage) -> DynamicImage {
    let mut rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    if width < 128 || height < 128 {
        return DynamicImage::ImageRgba8(rgba);
    }

    let macro_cols = 4u32;
    let macro_rows = 4u32;
    let micro_cols = 3u32;
    let micro_rows = 3u32;

    let cell_w = width as f32 / macro_cols as f32;
    let cell_h = height as f32 / macro_rows as f32;

    let subtle_white = Rgba([255, 255, 255, 70]);
    let macro_yellow = Rgba([255, 230, 0, 230]);
    let label_bg = Rgba([20, 20, 20, 210]);
    let label_fg = Rgba([255, 230, 0, 255]);

    // 1. Draw subtle micro-grid lines (3x3 per macro cell)
    let sub_w = cell_w / micro_cols as f32;
    let sub_h = cell_h / micro_rows as f32;

    for r in 0..macro_rows {
        for c in 0..macro_cols {
            let base_x = c as f32 * cell_w;
            let base_y = r as f32 * cell_h;

            for i in 1..micro_cols {
                let x = base_x + i as f32 * sub_w;
                draw_dashed_vline(&mut rgba, x.round() as i32, base_y as i32, (base_y + cell_h) as i32, subtle_white);
            }
            for j in 1..micro_rows {
                let y = base_y + j as f32 * sub_h;
                draw_dashed_hline(&mut rgba, y.round() as i32, base_x as i32, (base_x + cell_w) as i32, subtle_white);
            }
        }
    }

    // 2. Draw macro-grid borders (4x4)
    for c in 1..macro_cols {
        let x = (c as f32 * cell_w).round() as f32;
        draw_thick_vline(&mut rgba, x as i32, 0, height as i32, 2, macro_yellow);
    }
    for r in 1..macro_rows {
        let y = (r as f32 * cell_h).round() as f32;
        draw_thick_hline(&mut rgba, y as i32, 0, width as i32, 2, macro_yellow);
    }

    // 3. Draw outer border
    draw_hollow_rect_mut(
        &mut rgba,
        Rect::at(0, 0).of_size(width, height),
        macro_yellow,
    );

    // 4. Draw Macro Labels (A1..D4) in top-left of each macro cell
    let col_letters = ['A', 'B', 'C', 'D'];
    for r in 0..macro_rows {
        for c in 0..macro_cols {
            let label = format!("{}{}", col_letters[c as usize], r + 1);
            let lx = (c as f32 * cell_w).round() as i32 + 4;
            let ly = (r as f32 * cell_h).round() as i32 + 4;

            draw_badge(&mut rgba, lx, ly, &label, label_bg, label_fg);
        }
    }

    DynamicImage::ImageRgba8(rgba)
}

/// Convert a grid coordinate (e.g. "B3-5" or "A1") into absolute pixel coordinates (x, y).
/// Macro: A1..D4 (4x4)
/// Micro: 1..9 (3x3 keypad order: 1=top-left, 5=center, 9=bottom-right)
pub fn resolve_grid_coordinate(target: &str, img_width: u32, img_height: u32) -> Result<(u32, u32), String> {
    let clean = target.trim().to_ascii_uppercase();
    let parts: Vec<&str> = clean.split('-').collect();

    let macro_part = parts[0];
    if macro_part.len() < 2 {
        return Err(format!("Invalid macro grid coordinate: '{}'. Expected e.g. 'B3'", macro_part));
    }

    let col_char = macro_part.chars().next().unwrap();
    let row_char = macro_part.chars().nth(1).unwrap();

    let col_idx = match col_char {
        'A' => 0,
        'B' => 1,
        'C' => 2,
        'D' => 3,
        _ => return Err(format!("Invalid column letter '{}'. Must be A, B, C, or D", col_char)),
    };

    let row_idx = match row_char {
        '1' => 0,
        '2' => 1,
        '3' => 2,
        '4' => 3,
        _ => return Err(format!("Invalid row number '{}'. Must be 1, 2, 3, or 4", row_char)),
    };

    let macro_w = img_width as f64 / 4.0;
    let macro_h = img_height as f64 / 4.0;

    let base_x = col_idx as f64 * macro_w;
    let base_y = row_idx as f64 * macro_h;

    // If micro part is provided (1..9), center on the 3x3 subcell
    if parts.len() > 1 {
        let micro_num: u32 = parts[1].parse().map_err(|_| format!("Invalid micro index '{}'", parts[1]))?;
        if micro_num < 1 || micro_num > 9 {
            return Err(format!("Micro index must be 1..9, got {}", micro_num));
        }

        let sub_r = (micro_num - 1) / 3;
        let sub_c = (micro_num - 1) % 3;

        let sub_w = macro_w / 3.0;
        let sub_h = macro_h / 3.0;

        let center_x = base_x + (sub_c as f64 + 0.5) * sub_w;
        let center_y = base_y + (sub_r as f64 + 0.5) * sub_h;

        Ok((center_x.round() as u32, center_y.round() as u32))
    } else {
        // Default to center of macro cell
        let center_x = base_x + macro_w * 0.5;
        let center_y = base_y + macro_h * 0.5;
        Ok((center_x.round() as u32, center_y.round() as u32))
    }
}

/// Compose up to 4 frames into a single 768x768 storyboard (2x2 grid)
/// Perfect for Flash multimodal analysis of user journey / UX transition.
pub fn compose_storyboard_768(frames: &[DynamicImage]) -> DynamicImage {
    let canvas_size = 768u32;
    let cell_size = 384u32;
    let mut canvas = RgbaImage::new(canvas_size, canvas_size);

    // Dark slate background for clean contrast
    let bg_color = Rgba([15, 18, 24, 255]);
    for pixel in canvas.pixels_mut() {
        *pixel = bg_color;
    }

    let positions = [
        (0u32, 0u32),         // Step 1: Top-Left
        (cell_size, 0u32),     // Step 2: Top-Right
        (0u32, cell_size),     // Step 3: Bottom-Left
        (cell_size, cell_size) // Step 4: Bottom-Right
    ];

    let border_color = Rgba([50, 60, 80, 255]);
    let step_badge_bg = Rgba([0, 120, 215, 230]);
    let step_badge_fg = Rgba([255, 255, 255, 255]);

    for (i, frame) in frames.iter().take(4).enumerate() {
        let (cx, cy) = positions[i];
        let resized = frame.resize_exact(cell_size, cell_size, image::imageops::FilterType::Triangle);

        image::imageops::overlay(&mut canvas, &resized, cx as i64, cy as i64);

        // Frame dividing border
        draw_hollow_rect_mut(
            &mut canvas,
            Rect::at(cx as i32, cy as i32).of_size(cell_size, cell_size),
            border_color,
        );

        // Step Badge in top-left of each tile (e.g. "[Step 1]")
        let step_label = format!("STEP {}", i + 1);
        draw_badge(&mut canvas, (cx + 8) as i32, (cy + 8) as i32, &step_label, step_badge_bg, step_badge_fg);
    }

    DynamicImage::ImageRgba8(canvas)
}

// ─── Helper Drawing Functions ───────────────────────────────────────────────

fn draw_badge(img: &mut RgbaImage, x: i32, y: i32, text: &str, bg: Rgba<u8>, fg: Rgba<u8>) {
    let char_w = 7;
    let char_h = 10;
    let padding = 3;
    let badge_w = (text.len() as i32 * char_w) + (padding * 2);
    let badge_h = char_h + (padding * 2);

    // Fill background badge
    for py in y..(y + badge_h) {
        for px in x..(x + badge_w) {
            if px >= 0 && px < img.width() as i32 && py >= 0 && py < img.height() as i32 {
                img.put_pixel(px as u32, py as u32, bg);
            }
        }
    }

    // Draw simple monospace bitmap glyphs
    for (idx, ch) in text.chars().enumerate() {
        let gx = x + padding + (idx as i32 * char_w);
        let gy = y + padding;
        draw_simple_glyph(img, gx, gy, ch, fg);
    }
}

fn draw_dashed_vline(img: &mut RgbaImage, x: i32, y_start: i32, y_end: i32, color: Rgba<u8>) {
    if x < 0 || x >= img.width() as i32 {
        return;
    }
    for y in y_start..y_end {
        if y >= 0 && y < img.height() as i32 && (y / 4) % 2 == 0 {
            blend_pixel(img, x as u32, y as u32, color);
        }
    }
}

fn draw_dashed_hline(img: &mut RgbaImage, y: i32, x_start: i32, x_end: i32, color: Rgba<u8>) {
    if y < 0 || y >= img.height() as i32 {
        return;
    }
    for x in x_start..x_end {
        if x >= 0 && x < img.width() as i32 && (x / 4) % 2 == 0 {
            blend_pixel(img, x as u32, y as u32, color);
        }
    }
}

fn draw_thick_vline(img: &mut RgbaImage, x: i32, y_start: i32, y_end: i32, thickness: i32, color: Rgba<u8>) {
    for t in 0..thickness {
        let px = x + t;
        if px >= 0 && px < img.width() as i32 {
            for y in y_start..y_end {
                if y >= 0 && y < img.height() as i32 {
                    img.put_pixel(px as u32, y as u32, color);
                }
            }
        }
    }
}

fn draw_thick_hline(img: &mut RgbaImage, y: i32, x_start: i32, x_end: i32, thickness: i32, color: Rgba<u8>) {
    for t in 0..thickness {
        let py = y + t;
        if py >= 0 && py < img.height() as i32 {
            for x in x_start..x_end {
                if x >= 0 && x < img.width() as i32 {
                    img.put_pixel(x as u32, py as u32, color);
                }
            }
        }
    }
}

fn blend_pixel(img: &mut RgbaImage, x: u32, y: u32, src: Rgba<u8>) {
    let dst = img.get_pixel(x, y);
    let alpha = src[3] as f32 / 255.0;
    let inv_alpha = 1.0 - alpha;

    let r = (src[0] as f32 * alpha + dst[0] as f32 * inv_alpha).round() as u8;
    let g = (src[1] as f32 * alpha + dst[1] as f32 * inv_alpha).round() as u8;
    let b = (src[2] as f32 * alpha + dst[2] as f32 * inv_alpha).round() as u8;

    img.put_pixel(x, y, Rgba([r, g, b, 255]));
}

/// Lightweight built-in 5x7 bitmap font renderer for badges (zero external font file required)
fn draw_simple_glyph(img: &mut RgbaImage, x: i32, y: i32, ch: char, color: Rgba<u8>) {
    let bitmap: [u8; 7] = match ch {
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        'C' => [0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111],
        'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        'S' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
        '3' => [0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => [0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110],
        _ => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
    };

    for (row, byte) in bitmap.iter().enumerate() {
        for col in 0..5 {
            if (byte >> (4 - col)) & 1 == 1 {
                let px = x + col;
                let py = y + row as i32;
                if px >= 0 && px < img.width() as i32 && py >= 0 && py < img.height() as i32 {
                    img.put_pixel(px as u32, py as u32, color);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_grid_coordinate_macro_and_micro() {
        let (w, h) = (1920, 1080);
        // A1 center: col 0 (0..480), row 0 (0..270) -> center is (240, 135)
        let (x, y) = resolve_grid_coordinate("A1", w, h).unwrap();
        assert_eq!((x, y), (240, 135));

        // D4 center: col 3 (1440..1920), row 3 (810..1080) -> center is (1680, 945)
        let (x, y) = resolve_grid_coordinate("D4", w, h).unwrap();
        assert_eq!((x, y), (1680, 945));

        // B2-5 (B2 center subcell):
        // B2 is col 1 (480..960), row 1 (270..540).
        // 5 is keypad center -> (480 + 240, 270 + 135) = (720, 405)
        let (x, y) = resolve_grid_coordinate("B2-5", w, h).unwrap();
        assert_eq!((x, y), (720, 405));

        // B2-1 (B2 top-left subcell: sub_r 0, sub_c 0):
        // sub_w = 480/3 = 160, sub_h = 270/3 = 90
        // center = (480 + 80, 270 + 45) = (560, 315)
        let (x, y) = resolve_grid_coordinate("B2-1", w, h).unwrap();
        assert_eq!((x, y), (560, 315));
    }

    #[test]
    fn test_optimize_for_vit_tiles_scaling_and_alignment() {
        let img = DynamicImage::ImageRgba8(RgbaImage::new(1920, 1080));
        let opt = optimize_for_vit_tiles(&img, 1024);
        let (w, h) = opt.dimensions();
        assert!(w <= 1024);
        assert!(h <= 1024);
        assert_eq!(w % TILE_ALIGN_MULTIPLE, 0);
        assert_eq!(h % TILE_ALIGN_MULTIPLE, 0);
    }

    #[test]
    fn test_compose_storyboard_768() {
        let f1 = DynamicImage::ImageRgba8(RgbaImage::new(500, 300));
        let f2 = DynamicImage::ImageRgba8(RgbaImage::new(500, 300));
        let f3 = DynamicImage::ImageRgba8(RgbaImage::new(500, 300));
        let f4 = DynamicImage::ImageRgba8(RgbaImage::new(500, 300));

        let sb = compose_storyboard_768(&[f1, f2, f3, f4]);
        assert_eq!(sb.dimensions(), (768, 768));
    }
}
