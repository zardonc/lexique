use std::io::Cursor;

use base64::{Engine, engine::general_purpose::STANDARD};
use image::{DynamicImage, ImageFormat, ImageReader, Limits};
use serde::Serialize;

use crate::selection::{self, SelectionError, SelectionEvent};

pub(crate) const MAX_BYTES: usize = 16 * 1024 * 1024;
pub(crate) const MAX_SIDE: u32 = 4096;

#[derive(Clone, Serialize)]
pub struct OcrImage {
    width: u32,
    height: u32,
    data_url: String,
}

pub(crate) fn encode(
    image: DynamicImage,
    max_side: u32,
    max_bytes: usize,
) -> Result<OcrImage, SelectionError> {
    if image.width() == 0
        || image.height() == 0
        || image.width() > max_side
        || image.height() > max_side
    {
        return Err(SelectionError::new("image_limit", ""));
    }
    let mut encoded = Cursor::new(Vec::new());
    image
        .write_to(&mut encoded, ImageFormat::Png)
        .map_err(|error| {
            SelectionError::new(
                "image_invalid",
                format!("Could not encode preview: {error}"),
            )
        })?;
    if encoded.get_ref().len() > max_bytes {
        return Err(SelectionError::new("image_limit", ""));
    }
    Ok(OcrImage {
        width: image.width(),
        height: image.height(),
        data_url: format!(
            "data:image/png;base64,{}",
            STANDARD.encode(encoded.into_inner())
        ),
    })
}

fn decode(bytes: &[u8]) -> Result<OcrImage, SelectionError> {
    if bytes.is_empty() || bytes.len() > MAX_BYTES {
        return Err(SelectionError::new("image_limit", ""));
    }
    let format = image::guess_format(bytes).map_err(|error| {
        SelectionError::new(
            "image_invalid",
            format!("Could not read image format: {error}"),
        )
    })?;
    if !matches!(format, ImageFormat::Png | ImageFormat::Jpeg) {
        return Err(SelectionError::new("image_invalid", "PNG/JPEG only"));
    }
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_SIDE);
    limits.max_image_height = Some(MAX_SIDE);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let decoded = reader.decode().map_err(|error| {
        let code = if matches!(error, image::ImageError::Limits(_)) {
            "image_limit"
        } else {
            "image_invalid"
        };
        SelectionError::new(code, format!("Could not decode image: {error}"))
    })?;
    encode(decoded, MAX_SIDE, MAX_BYTES)
}

pub(crate) fn finish(
    app: &tauri::AppHandle,
    request: u64,
    result: Result<OcrImage, SelectionError>,
    recognize: bool,
) -> Result<(), String> {
    match result {
        Ok(image) => selection::deliver(
            app,
            SelectionEvent::Image {
                request,
                image,
                recognize,
            },
        ),
        Err(error) => selection::finish(app, request, Err(error)),
    }
}

#[tauri::command]
pub async fn ocr_image(
    app: tauri::AppHandle,
    request: tauri::ipc::Request<'_>,
) -> Result<(), String> {
    let bytes = match request.body() {
        tauri::ipc::InvokeBody::Raw(bytes) if bytes.len() <= MAX_BYTES => bytes.clone(),
        _ => return Err("Image import requires at most 16 MiB of binary PNG/JPEG data".into()),
    };
    let identity = selection::begin(&app)?;
    tauri::async_runtime::spawn_blocking(move || finish(&app, identity, decode(&bytes), false))
        .await
        .map_err(|error| format!("Image import task failed: {error}"))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_boundary_accepts_french_png_and_refuses_damaged_or_oversized_images() {
        let sample = include_bytes!("../../e2e/fixtures/ocr-french.png");
        let preview = decode(sample).unwrap();
        assert_eq!((preview.width, preview.height), (948, 159));
        assert!(preview.data_url.starts_with("data:image/png;base64,"));
        assert_eq!(decode(b"GIF89a").err().unwrap().code, "image_invalid");
        assert_eq!(decode(&sample[..32]).err().unwrap().code, "image_invalid");
        assert_eq!(
            decode(&vec![0; MAX_BYTES + 1]).err().unwrap().code,
            "image_limit"
        );
        let mut bytes = Cursor::new(Vec::new());
        DynamicImage::new_rgb8(4097, 1)
            .write_to(&mut bytes, ImageFormat::Png)
            .unwrap();
        assert_eq!(decode(bytes.get_ref()).err().unwrap().code, "image_limit");
    }
}
