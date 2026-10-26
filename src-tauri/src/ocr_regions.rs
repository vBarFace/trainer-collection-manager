//! Temporary, pixel-only OCR inputs. No paths, files, language models or storage.
use image::{imageops, GrayImage, ImageFormat, Luma, RgbaImage};
use serde::Serialize;
use std::io::Cursor;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextRegions {
    pub display_name: Option<Vec<u8>>,
    pub cp: Option<Vec<u8>>,
}

pub fn prepare(image: &RgbaImage) -> TextRegions {
    let (w, h) = image.dimensions();
    if w < 240 || h < w || h > w * 3 {
        return TextRegions {
            display_name: None,
            cp: None,
        };
    }
    // The CP label is in the upper central part of a portrait detail screen.
    // Include the CP prefix: digits alone are never sufficient OCR evidence.
    let cp = binary_crop(image, w / 4, h * 45 / 1000, w / 2, h * 55 / 1000, true);
    // Anchor the name to the green current-HP line, not the appraisal HP IV.
    // Multiple possible HP lines make the name crop ambiguous.
    let display_name = hp_line(image).and_then(|y| {
        let top = y.checked_sub(w * 12 / 100)?;
        binary_crop(image, w / 5, top, w * 3 / 5, w / 10, false)
    });
    TextRegions { display_name, cp }
}

fn hp_line(image: &RgbaImage) -> Option<u32> {
    let (w, h) = image.dimensions();
    let mut bands: Vec<(u32, u32)> = Vec::new();
    for y in h / 5..h * 7 / 10 {
        let mut start = 0;
        let mut run = 0;
        let mut found = false;
        for x in 0..=w {
            let [r, g, b, a] = if x < w {
                image.get_pixel(x, y).0
            } else {
                [0; 4]
            };
            if a >= 250 && g >= 145 && g.saturating_sub(r) >= 30 && g.saturating_sub(b) >= 15 {
                if run == 0 {
                    start = x;
                }
                run += 1;
            } else {
                if run >= w * 2 / 5
                    && start > w / 10
                    && start + run < w * 9 / 10
                    && (start + run / 2).abs_diff(w / 2) < w / 12
                {
                    found = true;
                }
                run = 0;
            }
        }
        if found {
            if let Some(last) = bands.last_mut().filter(|last| last.1 + 1 == y) {
                last.1 = y;
            } else {
                if bands.len() >= 8 {
                    return None;
                }
                bands.push((y, y));
            }
        }
    }
    let bands: Vec<_> = bands
        .into_iter()
        .filter(|(top, bottom)| bottom - top >= 1 && bottom - top < w / 25)
        .collect();
    if bands.len() == 1 {
        Some(bands[0].0)
    } else {
        None
    }
}

fn binary_crop(
    image: &RgbaImage,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    light_text: bool,
) -> Option<Vec<u8>> {
    if w == 0 || h == 0 || x + w > image.width() || y + h > image.height() {
        return None;
    }
    let mut pixels = GrayImage::from_pixel(w, h, Luma([255]));
    for row in 0..h {
        for col in 0..w {
            let [r, g, b, a] = image.get_pixel(x + col, y + row).0;
            let ink = if light_text {
                r >= 210 && g >= 210 && b >= 210
            } else {
                u32::from(r) + u32::from(g) + u32::from(b) < 480
            };
            if a >= 250 && ink {
                pixels.put_pixel(col, row, Luma([0]));
            }
        }
    }
    if light_text {
        remove_border_overlay(&mut pixels)?;
        pixels = normalize_mixed_size_line(&pixels).unwrap_or(pixels);
    }
    // Bounded upscaling and a white margin improve single-line OCR. PNG strips
    // all source metadata; intermediate pixels are dropped before returning.
    let scale = if w <= 600 { 2 } else { 1 };
    let scaled = imageops::resize(
        &pixels,
        pixels.width() * scale,
        pixels.height() * scale,
        imageops::FilterType::Nearest,
    );
    let mut padded = GrayImage::from_pixel(scaled.width() + 24, scaled.height() + 24, Luma([255]));
    imageops::replace(&mut padded, &scaled, 12, 12);
    let mut png = Cursor::new(Vec::new());
    padded.write_to(&mut png, ImageFormat::Png).ok()?;
    Some(png.into_inner())
}

// White phone overlays become black after inversion. Remove only tall regions
// connected to the crop border; a clipped text-sized component instead makes
// the crop unresolved so a truncated CP cannot silently become a smaller CP.
fn remove_border_overlay(pixels: &mut GrayImage) -> Option<()> {
    let (w, h) = pixels.dimensions();
    let mut visited = vec![false; (w * h) as usize];
    let mut overlays = Vec::new();
    let borders = (0..w)
        .flat_map(|x| [(x, 0), (x, h - 1)])
        .chain((0..h).flat_map(|y| [(0, y), (w - 1, y)]));
    for start in borders {
        let index = (start.1 * w + start.0) as usize;
        if visited[index] || pixels.get_pixel(start.0, start.1).0 != [0] {
            continue;
        }
        let mut component = vec![start];
        visited[index] = true;
        let mut cursor = 0;
        let (mut top, mut bottom, mut left, mut right) = (start.1, start.1, start.0, start.0);
        while cursor < component.len() {
            let (x, y) = component[cursor];
            cursor += 1;
            top = top.min(y);
            bottom = bottom.max(y);
            left = left.min(x);
            right = right.max(x);
            for (nx, ny) in [
                (x.wrapping_sub(1), y),
                (x + 1, y),
                (x, y.wrapping_sub(1)),
                (x, y + 1),
            ] {
                if nx >= w || ny >= h {
                    continue;
                }
                let index = (ny * w + nx) as usize;
                if !visited[index] && pixels.get_pixel(nx, ny).0 == [0] {
                    visited[index] = true;
                    component.push((nx, ny));
                }
            }
        }
        if (bottom - top + 1) * 5 < h * 4 {
            return None;
        }
        overlays.push((left, right));
        for (x, y) in component {
            pixels.put_pixel(x, y, Luma([255]));
        }
    }
    let ink: Vec<_> = pixels
        .enumerate_pixels()
        .filter(|(_, _, pixel)| pixel.0 == [0])
        .map(|(x, y, _)| (x, y))
        .collect();
    if let (Some(left), Some(right), Some(top), Some(bottom)) = (
        ink.iter().map(|p| p.0).min(),
        ink.iter().map(|p| p.0).max(),
        ink.iter().map(|p| p.1).min(),
        ink.iter().map(|p| p.1).max(),
    ) {
        let margin = (bottom - top + 1) * 3 / 4;
        if overlays.iter().any(|&(overlay_left, overlay_right)| {
            !(overlay_left > right && overlay_left - right >= margin
                || overlay_right < left && left - overlay_right >= margin)
        }) {
            return None;
        }
    }
    Some(())
}

// CP uses small prefix letters next to larger digits. Normalize glyph sizes
// geometrically before OCR rather than accepting the engine's low-score guess
// or substituting guessed letters/digits. No character identity is inferred.
fn normalize_mixed_size_line(pixels: &GrayImage) -> Option<GrayImage> {
    let (w, h) = pixels.dimensions();
    let mut visited = vec![false; (w * h) as usize];
    let mut boxes = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let index = (y * w + x) as usize;
            if visited[index] || pixels.get_pixel(x, y).0 != [0] {
                continue;
            }
            let mut points = vec![(x, y)];
            visited[index] = true;
            let (mut left, mut right, mut top, mut bottom) = (x, x, y, y);
            let mut cursor = 0;
            while cursor < points.len() {
                let (px, py) = points[cursor];
                cursor += 1;
                left = left.min(px);
                right = right.max(px);
                top = top.min(py);
                bottom = bottom.max(py);
                for (nx, ny) in [
                    (px.wrapping_sub(1), py),
                    (px + 1, py),
                    (px, py.wrapping_sub(1)),
                    (px, py + 1),
                ] {
                    if nx >= w || ny >= h {
                        continue;
                    }
                    let index = (ny * w + nx) as usize;
                    if !visited[index] && pixels.get_pixel(nx, ny).0 == [0] {
                        visited[index] = true;
                        points.push((nx, ny));
                    }
                }
            }
            if points.len() >= 4 && bottom - top + 1 >= h / 8 {
                boxes.push((left, top, right - left + 1, bottom - top + 1));
            }
            if boxes.len() > 21 {
                return None;
            }
        }
    }
    if boxes.len() < 3 {
        return None;
    }
    boxes.sort_by_key(|b| b.0);
    let height = boxes.iter().map(|b| b.3).max()?;
    if boxes[..2].iter().any(|b| b.3 * 100 > height * 85)
        || boxes[2..].iter().any(|b| b.3 * 100 < height * 85)
        || boxes
            .windows(2)
            .any(|pair| pair[0].0 + pair[0].2 > pair[1].0)
    {
        return None;
    }
    let gap = (height / 6).max(2);
    let widths: Vec<_> = boxes.iter().map(|b| (b.2 * height / b.3).max(1)).collect();
    let width = widths.iter().sum::<u32>() + gap * (boxes.len() as u32 - 1);
    if width > 4096 {
        return None;
    }
    let mut output = GrayImage::from_pixel(width, height, Luma([255]));
    let mut x = 0;
    for (b, width) in boxes.iter().zip(widths) {
        let glyph = imageops::crop_imm(pixels, b.0, b.1, b.2, b.3).to_image();
        let glyph = imageops::resize(&glyph, width, height, imageops::FilterType::Nearest);
        imageops::replace(&mut output, &glyph, x as i64, 0);
        x += width + gap;
    }
    Some(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn name_requires_a_unique_current_hp_anchor_and_supported_portrait_layout() {
        let mut image = RgbaImage::from_pixel(480, 1000, Rgba([255; 4]));
        assert!(prepare(&image).display_name.is_none());
        for y in 450..456 {
            for x in 120..360 {
                image.put_pixel(x, y, Rgba([90, 220, 170, 255]));
            }
        }
        assert_eq!(hp_line(&image), Some(450));
        assert!(prepare(&image).display_name.is_some());
        for y in 550..556 {
            for x in 120..360 {
                image.put_pixel(x, y, Rgba([90, 220, 170, 255]));
            }
        }
        assert!(prepare(&image).display_name.is_none());
        assert!(prepare(&RgbaImage::new(800, 400)).cp.is_none());
    }

    #[test]
    fn crops_are_bounded_metadata_free_pngs_and_transparency_is_not_text() {
        let image = RgbaImage::from_pixel(480, 1000, Rgba([0, 0, 0, 0]));
        let png = binary_crop(&image, 10, 20, 100, 30, false).unwrap();
        let decoded = image::load_from_memory(&png).unwrap().to_luma8();
        assert_eq!(decoded.dimensions(), (224, 84));
        assert!(decoded.pixels().all(|p| p.0 == [255]));
        assert!(binary_crop(&image, 470, 990, 100, 30, false).is_none());
    }

    #[test]
    fn removes_tall_border_overlays_but_rejects_clipped_text() {
        let mut pixels = GrayImage::from_pixel(100, 40, Luma([255]));
        for y in 0..40 {
            for x in 85..100 {
                pixels.put_pixel(x, y, Luma([0]));
            }
        }
        for y in 10..30 {
            for x in 30..36 {
                pixels.put_pixel(x, y, Luma([0]));
            }
        }
        assert!(remove_border_overlay(&mut pixels).is_some());
        assert_eq!(pixels.get_pixel(99, 20).0, [255]);
        assert_eq!(pixels.get_pixel(32, 20).0, [0]);
        for y in 10..30 {
            pixels.put_pixel(0, y, Luma([0]));
        }
        assert!(remove_border_overlay(&mut pixels).is_none());
    }

    #[test]
    fn normalizes_two_small_prefix_glyphs_without_guessing_characters() {
        let mut pixels = GrayImage::from_pixel(100, 40, Luma([255]));
        for (x, top, bottom) in [(5, 20, 35), (25, 20, 35), (45, 5, 35), (65, 5, 35)] {
            for y in top..bottom {
                for col in x..x + 6 {
                    pixels.put_pixel(col, y, Luma([0]));
                }
            }
        }
        let normalized = normalize_mixed_size_line(&pixels).unwrap();
        assert_eq!(normalized.height(), 30);
        let mut background = RgbaImage::from_pixel(480, 1000, Rgba([255; 4]));
        for y in 450..456 {
            for x in 100..480 {
                background.put_pixel(x, y, Rgba([90, 220, 170, 255]));
            }
        }
        assert!(hp_line(&background).is_none());
    }

    #[test]
    fn overlay_touching_last_glyph_cannot_turn_a_cp_into_a_shorter_integer() {
        let mut pixels = GrayImage::from_pixel(160, 60, Luma([255]));
        // Two remaining glyphs plus a last glyph joined to a tall right overlay.
        for x in [40..50, 60..70, 80..90] {
            for col in x {
                for y in 15..45 {
                    pixels.put_pixel(col, y, Luma([0]));
                }
            }
        }
        for y in 0..60 {
            for x in 88..160 {
                pixels.put_pixel(x, y, Luma([0]));
            }
        }
        assert!(remove_border_overlay(&mut pixels).is_none());
    }
}
