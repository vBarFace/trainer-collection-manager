//! Conservative, experimental pixel geometry. No OCR, models or persistence.
use image::RgbaImage;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Field<T> {
    pub value: Option<T>,
    pub confidence: Confidence,
    pub method: Method,
    pub state: State,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    None,
    Tentative,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    Unavailable,
    AppraisalGeometryV1,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Detected,
    Unresolved,
    Unavailable,
}

#[derive(Debug, Serialize)]
pub struct Issue {
    pub code: &'static str,
    pub message: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Extraction {
    pub display_name: Field<String>,
    pub cp: Field<String>,
    pub attack: Field<u8>,
    pub defense: Field<u8>,
    pub hp: Field<u8>,
    pub issues: Vec<Issue>,
}

impl<T> Field<T> {
    fn text_unavailable() -> Self {
        Self {
            value: None,
            confidence: Confidence::None,
            method: Method::Unavailable,
            state: State::Unavailable,
        }
    }
}

impl Field<u8> {
    fn bar(value: Option<u8>) -> Self {
        Self {
            value,
            confidence: if value.is_some() {
                Confidence::Tentative
            } else {
                Confidence::None
            },
            method: Method::AppraisalGeometryV1,
            state: if value.is_some() {
                State::Detected
            } else {
                State::Unresolved
            },
        }
    }
}

pub fn extract(pixels: &RgbaImage) -> Extraction {
    let values = bars(pixels);
    let mut issues = vec![Issue {
        code: "text_unavailable",
        message: "Local text recognition is not available in this release. Enter the display name and visible CP manually; hidden CP stays unknown.",
    }];
    if values.is_none() {
        issues.push(Issue {
            code: "bars_unresolved",
            message: "Three complete, unambiguous appraisal bars could not be measured. Enter all three IVs manually or leave all empty.",
        });
    }
    Extraction {
        display_name: Field::text_unavailable(),
        cp: Field::text_unavailable(),
        attack: Field::bar(values.map(|v| v[0])),
        defense: Field::bar(values.map(|v| v[1])),
        hp: Field::bar(values.map(|v| v[2])),
        issues,
    }
}

// Palette gates include the neutral grey tracks used by actual appraisal UI.
// Transparent pixels never supply evidence invisible in the preview.
fn color([r, g, b, a]: [u8; 4]) -> u8 {
    if a < 250 {
        return 0;
    }
    if r >= 190
        && (95..=205).contains(&g)
        && b <= 150
        && r.saturating_sub(g) >= 35
        && g.saturating_sub(b) >= 20
    {
        1 // orange partial fill
    } else if r >= 180
        && (60..=190).contains(&g)
        && (70..=190).contains(&b)
        && r.saturating_sub(g) >= 45
        && r.saturating_sub(b) >= 35
        && g.abs_diff(b) < 65
    {
        2 // pink complete fill
    } else if (185..=242).contains(&r)
        && (185..=242).contains(&g)
        && (185..=242).contains(&b)
        && r.max(g).max(b) - r.min(g).min(b) <= 18
        || r >= 225
            && (195..=235).contains(&g)
            && (175..=225).contains(&b)
            && r.saturating_sub(g) >= 8
            && g >= b
    {
        3 // visible empty track
    } else {
        0
    }
}

#[derive(Clone, Debug)]
struct Track {
    x: u32,
    width: u32,
    top: u32,
    bottom: u32,
    value: u8,
    rows: u32,
}

fn measure(row: &[u8]) -> Option<u8> {
    let width = row.len();
    let mut empty = false;
    let mut fill = 0;
    let mut kind = 0;
    let mut separators = [false; 2];
    let mut unknown = Vec::new();
    for (x, &pixel) in row.iter().enumerate() {
        if let Some(index) = [width / 3, width * 2 / 3]
            .iter()
            .position(|&tick| x.abs_diff(tick) <= (width / 45).max(1))
        {
            // Ignore JPEG colour bleeding at separator edges, while requiring
            // actual uncoloured separator evidence inside both regions.
            separators[index] |= pixel == 0;
            continue;
        }
        match pixel {
            0 => {
                // A few mixed-colour pixels may surround the fill endpoint.
                unknown.push(x);
            }
            3 => empty = true,
            1 | 2 if !empty && (kind == 0 || kind == pixel) => {
                kind = pixel;
                fill = x + 1;
            }
            _ => return None,
        }
    }
    // Full bars must have the distinct complete-IV palette; an orange span
    // alone provides no evidence of its intended full track length.
    if !separators.iter().all(|found| *found) || (!empty && kind != 2) || (empty && kind == 2) {
        return None;
    }
    for tick in [width / 3, width * 2 / 3] {
        let tolerance = (width / 45).max(1);
        if kind == 1
            && empty
            && fill.abs_diff(tick) <= tolerance + 1
            && row[tick - tolerance - 1] == 1
            && row[tick + tolerance + 1] == 3
        {
            // An endpoint inside a separator is exactly IV5 or IV10; the
            // unpainted separator itself must not shorten the filled length.
            fill = tick;
        }
    }
    if unknown.len() > (width / 60).max(1)
        || unknown
            .iter()
            .any(|&x| x.abs_diff(fill) > (width / 60).max(1))
    {
        return None;
    }
    let steps = fill as f64 * 15.0 / width as f64;
    let value = steps.round();
    // Rounded caps and JPEG mixing can move the observed endpoint slightly.
    // Refuse anything close to the halfway point between two IVs.
    if (steps - value).abs() > 0.22 {
        return None;
    }
    Some(value as u8)
}

fn bars(pixels: &RgbaImage) -> Option<[u8; 3]> {
    let (width, height) = pixels.dimensions();
    if width < 80 || height < 80 {
        return None;
    }
    let mut tracks: Vec<Track> = Vec::new();
    let mut row = vec![0; width as usize];
    for y in 0..height {
        for (x, pixel) in row.iter_mut().enumerate() {
            *pixel = color(pixels.get_pixel(x as u32, y).0);
        }
        let mut x = 0;
        while x < width as usize {
            if row[x] == 0 {
                x += 1;
                continue;
            }
            let start = x;
            let mut end = x + 1;
            let mut gap = 0;
            while x < width as usize {
                if row[x] != 0 {
                    end = x + 1;
                    gap = 0;
                } else {
                    gap += 1;
                    if gap > (width / 100).max(1) {
                        break;
                    }
                }
                x += 1;
            }
            let length = end - start;
            if length < 60 || start < 3 || end + 3 >= width as usize {
                continue;
            }
            let Some(value) = measure(&row[start..end]) else {
                continue;
            };
            if let Some(track) = tracks.iter_mut().rev().find(|t| {
                y > t.bottom
                    && y - t.bottom <= (t.width / 15).max(1)
                    && y - t.top < t.width / 10
                    && t.x.abs_diff(start as u32) <= (t.width / 20).max(1)
                    && (t.x + t.width).abs_diff(end as u32) <= (t.width / 20).max(1)
            }) {
                // Contradictory rows invalidate the entire group, not just one IV.
                if track.value != value {
                    return None;
                }
                track.bottom = y;
                track.rows += 1;
                // Interior rows of rounded bars carry the complete width.
                if length as u32 > track.width {
                    track.x = start as u32;
                    track.width = length as u32;
                }
            } else {
                // Bounded storage even for adversarially patterned valid images.
                if tracks.len() >= 128 {
                    return None;
                }
                tracks.push(Track {
                    x: start as u32,
                    width: length as u32,
                    top: y,
                    bottom: y,
                    value,
                    rows: 1,
                });
            }
        }
    }
    // Multiple nearby agreeing scanlines establish a band despite JPEG noise.
    // A valid contradictory row invalidates it above rather than being voted away.
    if tracks.len() != 3 {
        return None;
    }
    let [a, b, c] = [&tracks[0], &tracks[1], &tracks[2]];
    if a.top < 3
        || c.bottom + 3 >= height
        || tracks.iter().any(|t| {
            let thickness = t.bottom - t.top + 1;
            thickness < 3 || t.rows < 3 || thickness * 10 > t.width
        })
        || [b, c].iter().any(|t| {
            t.x.abs_diff(a.x) > (a.width / 60).max(1)
                || t.width.abs_diff(a.width) > (a.width / 40).max(1)
                || (t.bottom - t.top + 1) * 3 < a.bottom - a.top + 1
                || t.bottom - t.top + 1 > (a.bottom - a.top + 1) * 3
        })
        || b.top <= a.bottom
        || c.top <= b.bottom
        || ((b.top + b.bottom) / 2 - (a.top + a.bottom) / 2)
            .abs_diff((c.top + c.bottom) / 2 - (b.top + b.bottom) / 2)
            > (a.width / 40).max(2)
        || b.top - a.top < (a.bottom - a.top + 1) * 2
        || b.top - a.top > a.width / 2
    {
        return None;
    }
    Some([a.value, b.value, c.value])
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    #[ignore = "Requires an explicitly supplied private reference image; never copies it"]
    fn inspect_authorized_reference() {
        let path = std::env::var_os("TCM_REFERENCE_IMAGE").expect("Set TCM_REFERENCE_IMAGE");
        let pixels = image::open(path).unwrap().to_rgba8();
        let result = extract(&pixels);
        println!("{}", serde_json::to_string(&result).unwrap());
        if let Ok(expected) = std::env::var("TCM_REFERENCE_IVS") {
            let expected: Vec<u8> = expected.split(',').map(|v| v.parse().unwrap()).collect();
            assert_eq!(expected.len(), 3);
            assert_eq!(
                [result.attack.value, result.defense.value, result.hp.value],
                [Some(expected[0]), Some(expected[1]), Some(expected[2])]
            );
        }
    }

    // Original MIT test drawings, generated in memory; no game artwork/assets.
    pub fn fixture(scale: u32, values: [u8; 3]) -> RgbaImage {
        let mut pixels = RgbaImage::from_pixel(240 * scale, 300 * scale, Rgba([255; 4]));
        for (i, value) in values.into_iter().enumerate() {
            for y in (140 + i as u32 * 25) * scale..(146 + i as u32 * 25) * scale {
                for x in 30 * scale..180 * scale {
                    let offset = x - 30 * scale;
                    let rgb = if (50 * scale..51 * scale).contains(&offset)
                        || (100 * scale..101 * scale).contains(&offset)
                    {
                        [255; 4]
                    } else if offset < u32::from(value) * 10 * scale {
                        if value == 15 {
                            [240, 110, 140, 255]
                        } else {
                            [250, 150, 60, 255]
                        }
                    } else {
                        [245, 215, 200, 255]
                    };
                    pixels.put_pixel(x, y, Rgba(rgb));
                }
            }
        }
        pixels
    }

    #[test]
    fn complete_tracks_quantize_every_iv_at_multiple_resolutions() {
        for scale in [1, 2, 4, 8] {
            for iv in 0..=15 {
                assert_eq!(bars(&fixture(scale, [iv, 0, 15])), Some([iv, 0, 15]));
            }
        }
    }

    fn rounded_fixture(scale: u32, values: [u8; 3]) -> RgbaImage {
        let mut pixels = RgbaImage::from_pixel(360 * scale, 480 * scale, Rgba([255; 4]));
        let radius = 7 * scale;
        for (index, value) in values.into_iter().enumerate() {
            for y in 0..14 * scale {
                for x in 0..240 * scale {
                    let dx = if x < radius {
                        radius - x
                    } else {
                        x.saturating_sub(240 * scale - radius - 1)
                    };
                    let dy = y.abs_diff(radius);
                    if dx * dx + dy * dy > radius * radius {
                        continue;
                    }
                    let rgb = if (80 * scale..83 * scale).contains(&x)
                        || (160 * scale..163 * scale).contains(&x)
                    {
                        [255; 4]
                    } else if x < u32::from(value) * 16 * scale {
                        if value == 15 {
                            [228, 125, 129, 255]
                        } else {
                            [243, 167, 73, 255]
                        }
                    } else {
                        [226, 226, 228, 255]
                    };
                    pixels.put_pixel(
                        40 * scale + x,
                        (200 + index as u32 * 60) * scale + y,
                        Rgba(rgb),
                    );
                }
            }
        }
        pixels
    }

    #[test]
    fn rounded_neutral_tracks_quantize_every_iv_at_multiple_scales() {
        for scale in [1, 2, 4] {
            for value in 0..=15 {
                assert_eq!(
                    bars(&rounded_fixture(scale, [value, 0, 15])),
                    Some([value, 0, 15])
                );
            }
        }
    }

    #[test]
    fn jpeg_rounding_and_compression_preserve_clear_appraisal_group() {
        for scale in [1, 2, 4] {
            let rgb = image::DynamicImage::ImageRgba8(rounded_fixture(scale, [15, 2, 2])).to_rgb8();
            let mut encoded = Vec::new();
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut encoded, 85)
                .encode_image(&rgb)
                .unwrap();
            let decoded = image::load_from_memory(&encoded).unwrap().to_rgba8();
            assert_eq!(bars(&decoded), Some([15, 2, 2]));
        }
    }

    #[test]
    fn rounded_tracks_reject_shared_occlusion_and_image_boundary_crops() {
        let pixels = rounded_fixture(1, [15, 2, 2]);
        let mut occluded = pixels.clone();
        for y in 195..340 {
            for x in 200..280 {
                occluded.put_pixel(x, y, Rgba([255; 4]));
            }
        }
        assert_eq!(bars(&occluded), None);
        for (x, y, width, height) in [(45, 0, 300, 480), (0, 0, 360, 326)] {
            assert_eq!(
                bars(&image::imageops::crop_imm(&pixels, x, y, width, height).to_image()),
                None
            );
        }
    }

    #[test]
    fn incomplete_occluded_unreadable_and_ambiguous_tracks_are_unresolved() {
        for rect in [
            (30, 165, 180, 171),
            (90, 140, 110, 146),
            (105, 140, 180, 146),
        ] {
            let mut pixels = fixture(1, [8, 12, 15]);
            for y in rect.1..rect.3 {
                for x in rect.0..rect.2 {
                    pixels.put_pixel(x, y, Rgba([255; 4]));
                }
            }
            assert_eq!(bars(&pixels), None);
        }
        let pixels = fixture(1, [8, 12, 15]);
        for (x, y, w, h) in [(31, 0, 200, 300), (0, 150, 240, 150), (0, 0, 170, 300)] {
            assert_eq!(
                bars(&image::imageops::crop_imm(&pixels, x, y, w, h).to_image()),
                None
            );
        }
        assert_eq!(bars(&RgbaImage::new(240, 300)), None);
        let mut extra = pixels.clone();
        for y in 215..221 {
            for x in 30..180 {
                if x != 80 && x != 130 {
                    extra.put_pixel(x, y, Rgba([240, 110, 140, 255]));
                }
            }
        }
        assert_eq!(bars(&extra), None);
    }

    #[test]
    fn rejects_non_quantized_reversed_transparent_and_contradictory_evidence() {
        let mut pixels = fixture(1, [8, 12, 15]);
        for y in 140..146 {
            for x in 110..115 {
                pixels.put_pixel(x, y, Rgba([250, 150, 60, 255]));
            }
        }
        assert_eq!(bars(&pixels), None);
        let pixels = fixture(1, [8, 12, 15]);
        assert_eq!(bars(&image::imageops::flip_horizontal(&pixels)), None);
        let mut transparent = pixels.clone();
        for p in transparent.pixels_mut() {
            p.0[3] = 0;
        }
        assert_eq!(bars(&transparent), None);
        let mut contradictory = pixels;
        for x in 110..120 {
            contradictory.put_pixel(x, 143, Rgba([250, 150, 60, 255]));
        }
        assert_eq!(bars(&contradictory), None);
    }

    #[test]
    fn shared_overlay_cannot_redefine_track_length_and_plain_rectangles_are_not_bars() {
        for values in [[2, 4, 6], [15, 15, 15]] {
            let mut pixels = fixture(1, values);
            for y in 135..200 {
                for x in 130..180 {
                    pixels.put_pixel(x, y, Rgba([255; 4]));
                }
            }
            assert_eq!(bars(&pixels), None);
        }
        let mut pixels = fixture(1, [15, 15, 15]);
        for top in [140, 165, 190] {
            for y in top..top + 6 {
                for x in [80, 130] {
                    pixels.put_pixel(x, y, Rgba([240, 110, 140, 255]));
                }
            }
        }
        assert_eq!(bars(&pixels), None);
    }

    #[test]
    fn text_is_explicitly_unavailable_even_when_bars_are_detected() {
        // These are contract checks, NOT text recognition/accuracy tests.
        for values in [[0, 0, 0], [15, 15, 15], [7, 8, 9]] {
            let result = extract(&fixture(1, values));
            assert!(result.display_name.value.is_none());
            assert!(result.cp.value.is_none());
            assert_eq!(result.attack.value, Some(values[0]));
            let json = serde_json::to_value(result).unwrap();
            assert_eq!(json["displayName"]["state"], "unavailable");
            assert_eq!(json["cp"]["confidence"], "none");
            assert_eq!(json["attack"]["confidence"], "tentative");
        }
    }

    #[test]
    fn busy_images_hit_a_bounded_candidate_limit_without_suggestions() {
        let mut pixels = RgbaImage::from_pixel(240, 3000, Rgba([255; 4]));
        for band in 0..129 {
            for y in 10 + band * 20..16 + band * 20 {
                for x in 30..180 {
                    if x != 80 && x != 130 {
                        pixels.put_pixel(x, y, Rgba([240, 110, 140, 255]));
                    }
                }
            }
        }
        let result = extract(&pixels);
        assert!(result.attack.value.is_none());
        assert!(result.defense.value.is_none());
        assert!(result.hp.value.is_none());
    }
}
