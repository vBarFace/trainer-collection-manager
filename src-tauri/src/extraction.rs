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

// Narrow palette gates deliberately prefer unresolved over guessing. Transparent
// source pixels must never supply evidence invisible in the preview.
fn color([r, g, b, a]: [u8; 4]) -> u8 {
    if a < 250 {
        return 0;
    }
    if r >= 225 && (95..=185).contains(&g) && b <= 115 && r.saturating_sub(g) >= 50 {
        1 // orange partial fill
    } else if r >= 220 && (75..=155).contains(&g) && (95..=175).contains(&b) {
        2 // pink complete fill
    } else if r >= 225
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
}

fn measure(row: &[u8]) -> Option<u8> {
    let width = row.len();
    let mut empty = false;
    let mut fill = 0;
    let mut kind = 0;
    let mut separators = [false; 2];
    for (x, &pixel) in row.iter().enumerate() {
        match pixel {
            0 => {
                // Only the two known segment separators may interrupt a track.
                let Some(index) = [width / 3, width * 2 / 3]
                    .iter()
                    .position(|&tick| x.abs_diff(tick) <= (width / 150).max(1))
                else {
                    return None;
                };
                separators[index] = true;
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
    let steps = fill as f64 * 15.0 / width as f64;
    let value = steps.round();
    // At most ~1 pixel of endpoint uncertainty, and never close to half a step.
    if (steps - value).abs() > (15.0 / width as f64).min(0.15) {
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
            if let Some(track) = tracks
                .iter_mut()
                .rev()
                .find(|t| t.bottom + 1 == y && t.x == start as u32 && t.width == length as u32)
            {
                // Contradictory rows invalidate the entire group, not just one IV.
                if track.value != value {
                    return None;
                }
                track.bottom = y;
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
                });
            }
        }
    }
    // Do not discard short contradictory fragments: an occlusion splitting a
    // track into a long and a short band must invalidate the complete result.
    if tracks.len() != 3 {
        return None;
    }
    let [a, b, c] = [&tracks[0], &tracks[1], &tracks[2]];
    if a.top < 3
        || c.bottom + 3 >= height
        || tracks.iter().any(|t| {
            let thickness = t.bottom - t.top + 1;
            thickness < 3 || thickness * 12 > t.width
        })
        || [b, c]
            .iter()
            .any(|t| t.x != a.x || t.width != a.width || t.bottom - t.top != a.bottom - a.top)
        || b.top <= a.bottom
        || c.top <= b.bottom
        || (b.top - a.top).abs_diff(c.top - b.top) > 1
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
