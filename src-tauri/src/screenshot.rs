//! Consent-only, temporary screenshot handling. No renderer-supplied paths.
use image::{ImageError, ImageFormat, ImageReader, Limits};
use serde::Serialize;
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;

pub const MAX_FILE_BYTES: u64 = 10 * 1024 * 1024;
pub const MAX_SIDE: u32 = 4096;
pub const MAX_PIXELS: u64 = 12_000_000;
pub const MAX_DECODE_BYTES: u64 = 64 * 1024 * 1024;
const PREVIEW_SIDE: u32 = 1600;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub png: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub extraction: crate::extraction::Extraction,
    pub text_regions: crate::ocr_regions::TextRegions,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Error {
    pub code: &'static str,
    pub message: &'static str,
}

fn invalid_image() -> Error {
    Error {
        code: "invalid_image",
        message: "Select a valid, fully decodable PNG or JPEG image.",
    }
}

fn resource_limit() -> Error {
    Error {
        code: "image_limit",
        message: "Image exceeds the limits: 10 MiB, 4096 pixels per side, 12 million pixels, or 64 MiB decoding budget.",
    }
}

fn image_error(error: ImageError) -> Error {
    if matches!(error, ImageError::Limits(_)) {
        resource_limit()
    } else {
        invalid_image()
    }
}

fn dimensions(width: u32, height: u32) -> Result<(), Error> {
    if width == 0 || height == 0 {
        return Err(invalid_image());
    }
    if width > MAX_SIDE || height > MAX_SIDE || u64::from(width) * u64::from(height) > MAX_PIXELS {
        return Err(resource_limit());
    }
    Ok(())
}

fn limits() -> Limits {
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_SIDE);
    limits.max_image_height = Some(MAX_SIDE);
    limits.max_alloc = Some(MAX_DECODE_BYTES);
    limits
}

fn reader(bytes: &[u8], format: ImageFormat) -> ImageReader<Cursor<&[u8]>> {
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    reader.limits(limits());
    reader
}

fn decode_preview(bytes: &[u8]) -> Result<Preview, Error> {
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err(resource_limit());
    }
    let format = image::guess_format(bytes).map_err(|_| invalid_image())?;
    if !matches!(format, ImageFormat::Png | ImageFormat::Jpeg) {
        return Err(invalid_image());
    }
    // Parse dimensions with decoder limits before allocating the pixel buffer.
    let (width, height) = reader(bytes, format)
        .into_dimensions()
        .map_err(image_error)?;
    dimensions(width, height)?;
    let decoded = reader(bytes, format).decode().map_err(image_error)?;
    dimensions(decoded.width(), decoded.height())?;
    // Reuse validated full-resolution pixels, before preview downsampling.
    let rgba = decoded.to_rgba8();
    let extraction = crate::extraction::extract(&rgba);
    let text_regions = crate::ocr_regions::prepare(&rgba);
    drop(rgba);
    // A fresh pixel-only PNG discards EXIF, text, profiles, names and other
    // metadata. The smaller preview also bounds IPC and renderer memory.
    let pixels = if width > PREVIEW_SIDE || height > PREVIEW_SIDE {
        decoded.thumbnail(PREVIEW_SIDE, PREVIEW_SIDE).to_rgba8()
    } else {
        decoded.to_rgba8()
    };
    drop(decoded);
    let mut png = Cursor::new(Vec::new());
    pixels
        .write_to(&mut png, ImageFormat::Png)
        .map_err(image_error)?;
    Ok(Preview {
        png: png.into_inner(),
        width,
        height,
        extraction,
        text_regions,
    })
}

fn read_selected(path: &Path) -> Result<Preview, Error> {
    let unreadable = || Error {
        code: "image_read",
        message:
            "The selected image could not be read. Select an accessible local file and try again.",
    };
    let file = File::open(path).map_err(|_| unreadable())?;
    let metadata = file.metadata().map_err(|_| unreadable())?;
    if !metadata.is_file() {
        return Err(invalid_image());
    }
    if metadata.len() > MAX_FILE_BYTES {
        return Err(resource_limit());
    }
    // Enforce the cap again while reading in case the file grows after stat.
    let mut bytes = Vec::new();
    file.take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| unreadable())?;
    decode_preview(&bytes)
}

/// Run on a blocking worker: the only source of a file path is this native
/// single-file picker. The handle and source bytes are dropped before return.
pub fn select(parent: &tauri::WebviewWindow) -> Result<Option<Preview>, Error> {
    rfd::FileDialog::new()
        .set_parent(parent)
        .set_title("Select one screenshot for review")
        .add_filter("PNG or JPEG screenshot", &["png", "jpg", "jpeg"])
        .pick_file()
        .map(|path| read_selected(&path))
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, ImageBuffer, Rgba};

    #[test]
    #[ignore = "requires an explicit private TCM_REFERENCE_SCREENSHOT; no fixture retained"]
    fn inspect_local_extraction_for_ocr() {
        let path =
            std::env::var_os("TCM_REFERENCE_SCREENSHOT").expect("explicit reference path required");
        let preview = read_selected(Path::new(&path)).unwrap();
        println!(
            "TCM_OCR_INPUT={}",
            serde_json::json!({"extraction": preview.extraction, "textRegions": preview.text_regions})
        );
    }

    fn encoded(image: &DynamicImage, format: ImageFormat) -> Vec<u8> {
        let mut output = Cursor::new(Vec::new());
        image.write_to(&mut output, format).unwrap();
        output.into_inner()
    }

    #[test]
    fn accepts_png_and_jpeg_content_and_returns_pixel_only_png() {
        for format in [ImageFormat::Png, ImageFormat::Jpeg] {
            let bytes = encoded(&DynamicImage::new_rgb8(12, 24), format);
            let preview = decode_preview(&bytes).unwrap();
            assert_eq!((preview.width, preview.height), (12, 24));
            assert_eq!(image::guess_format(&preview.png).unwrap(), ImageFormat::Png);
            assert_eq!(image::load_from_memory(&preview.png).unwrap().width(), 12);
        }
    }

    #[test]
    fn extracts_before_preview_downsampling_and_jpeg_never_changes_known_ivs() {
        let image = DynamicImage::ImageRgba8(crate::extraction::tests::fixture(8, [0, 14, 15]));
        let png = decode_preview(&encoded(&image, ImageFormat::Png)).unwrap();
        assert_eq!(png.extraction.attack.value, Some(0));
        assert_eq!(png.extraction.defense.value, Some(14));
        assert_eq!(png.extraction.hp.value, Some(15));
        assert_eq!(image::load_from_memory(&png.png).unwrap().height(), 1600);
        // JPEG support may conservatively resolve to unknown after compression.
        let jpeg = decode_preview(&encoded(&image.to_rgb8().into(), ImageFormat::Jpeg)).unwrap();
        assert!(jpeg.extraction.attack.value.is_none() || jpeg.extraction.attack.value == Some(0));
        assert!(
            jpeg.extraction.defense.value.is_none() || jpeg.extraction.defense.value == Some(14)
        );
        assert!(jpeg.extraction.hp.value.is_none() || jpeg.extraction.hp.value == Some(15));
    }

    #[test]
    fn rejects_empty_fake_unsupported_and_truncated_content() {
        for bytes in [b"".as_slice(), b"a file called screenshot.png", b"GIF89a"] {
            assert_eq!(decode_preview(bytes).unwrap_err(), invalid_image());
        }
        let png = encoded(&DynamicImage::new_rgb8(12, 24), ImageFormat::Png);
        assert_eq!(decode_preview(&png[..35]).unwrap_err(), invalid_image());
        let jpeg = encoded(&DynamicImage::new_rgb8(12, 24), ImageFormat::Jpeg);
        assert_eq!(decode_preview(&jpeg[..20]).unwrap_err(), invalid_image());
    }

    #[test]
    fn enforces_byte_dimension_pixel_and_decode_limits() {
        assert_eq!(
            decode_preview(&vec![0; MAX_FILE_BYTES as usize + 1]).unwrap_err(),
            resource_limit()
        );
        assert!(dimensions(4000, 3000).is_ok());
        assert!(dimensions(4096, 1).is_ok());
        assert_eq!(dimensions(4097, 1), Err(resource_limit()));
        assert_eq!(dimensions(1, 4097), Err(resource_limit()));
        assert_eq!(dimensions(4000, 3001), Err(resource_limit()));
        assert_eq!(dimensions(0, 1), Err(invalid_image()));
        let oversized = encoded(&DynamicImage::new_rgb8(4097, 1), ImageFormat::Png);
        assert_eq!(decode_preview(&oversized).unwrap_err(), resource_limit());
        let too_many_pixels = encoded(&DynamicImage::new_luma8(4000, 3001), ImageFormat::Png);
        assert_eq!(
            decode_preview(&too_many_pixels).unwrap_err(),
            resource_limit()
        );
        let high_depth =
            DynamicImage::ImageRgba16(ImageBuffer::<Rgba<u16>, Vec<u16>>::new(4000, 3000));
        let bytes = encoded(&high_depth, ImageFormat::Png);
        assert_eq!(decode_preview(&bytes).unwrap_err(), resource_limit());
    }

    #[test]
    fn preview_dimensions_are_bounded_without_changing_original_dimensions() {
        let bytes = encoded(&DynamicImage::new_rgb8(1800, 900), ImageFormat::Png);
        let preview = decode_preview(&bytes).unwrap();
        assert_eq!((preview.width, preview.height), (1800, 900));
        let image = image::load_from_memory(&preview.png).unwrap();
        assert_eq!((image.width(), image.height()), (1600, 800));
    }

    #[test]
    fn source_metadata_is_not_included_in_preview() {
        let jpeg = encoded(&DynamicImage::new_rgb8(12, 24), ImageFormat::Jpeg);
        let private_comment = b"private screenshot metadata";
        let mut with_comment = jpeg[..2].to_vec();
        with_comment.extend_from_slice(&[0xff, 0xfe]);
        with_comment.extend_from_slice(&((private_comment.len() + 2) as u16).to_be_bytes());
        with_comment.extend_from_slice(private_comment);
        with_comment.extend_from_slice(&jpeg[2..]);
        let preview = decode_preview(&with_comment).unwrap();
        assert!(!preview
            .png
            .windows(private_comment.len())
            .any(|window| window == private_comment));
    }

    #[test]
    fn content_validation_ignores_extension_and_leaves_original_unchanged() {
        let path = std::env::temp_dir().join(format!(
            "tcm-image-{}-{}.txt",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let original = encoded(&DynamicImage::new_rgb8(12, 24), ImageFormat::Png);
        std::fs::write(&path, &original).unwrap();
        let preview = read_selected(&path).unwrap();
        assert_eq!((preview.width, preview.height), (12, 24));
        assert_eq!(std::fs::read(&path).unwrap(), original);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn unreadable_errors_do_not_contain_the_selected_path() {
        let error = read_selected(Path::new("missing-private-screenshot-path.png")).unwrap_err();
        assert_eq!(error.code, "image_read");
        assert!(!error.message.contains("missing-private"));
    }
}
