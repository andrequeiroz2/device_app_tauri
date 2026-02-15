use std::fs;
use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD as b64;
use image::{ImageFormat, DynamicImage};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};
use tracing::instrument;

const MAX_IMAGE_BYTES: usize = 5 * 1024 * 1024;
const THUMB_SIZE: u32 = 320;

pub struct SavedImage {
    pub image_path: String,
    pub thumb_path: String,
    pub original_name: String,
    pub mime: String,
    pub size_bytes: i64,
    pub checksum_sha256: String,
}

#[derive(Debug)]
pub struct ImagePayload<'a> {
    pub data_base64: &'a str,
    pub original_name: &'a str,
    pub mime: &'a str,
    pub size_bytes: usize,
}

#[instrument(skip(app_handle, payload))]
pub fn save_image_with_thumb(
    app_handle: &AppHandle,
    user_uuid: &str,
    location_uuid: &str,
    payload: ImagePayload,
) -> Result<SavedImage, String> {
    validate_image_meta(&payload)?;

    let bytes = decode_base64(payload.data_base64)?;
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err("Image exceeds 5 MB limit".to_string());
    }

    let format = infer_format(&bytes, payload.mime)?;

    let dyn_img = image::load_from_memory_with_format(&bytes, format)
        .map_err(|e| format!("Invalid image data: {}", e))?;

    let checksum = compute_checksum(&bytes);

    let base_dir = image_dir(app_handle, user_uuid, location_uuid)?;
    fs::create_dir_all(&base_dir).map_err(|e| format!("Failed to create dir: {}", e))?;

    let image_ext = ext_for_format(format);
    let thumb_ext = "webp";

    let image_path = base_dir.join(format!("original.{}", image_ext));
    let thumb_path = base_dir.join(format!("thumb.{}", thumb_ext));

    save_original(&dyn_img, format, &image_path)?;
    save_thumb(&dyn_img, &thumb_path)?;

    Ok(SavedImage {
        image_path: image_path.to_string_lossy().to_string(),
        thumb_path: thumb_path.to_string_lossy().to_string(),
        original_name: payload.original_name.to_string(),
        mime: payload.mime.to_string(),
        size_bytes: bytes.len() as i64,
        checksum_sha256: checksum,
    })
}

fn validate_image_meta(payload: &ImagePayload) -> Result<(), String> {
    let mime = payload.mime.to_lowercase();
    let allowed = ["image/png", "image/jpeg", "image/webp"];
    if !allowed.contains(&mime.as_str()) {
        return Err("Unsupported image format. Use PNG, JPG or WEBP".to_string());
    }
    if payload.size_bytes > MAX_IMAGE_BYTES {
        return Err("Image exceeds 5 MB limit".to_string());
    }
    Ok(())
}

fn decode_base64(data: &str) -> Result<Vec<u8>, String> {
    b64.decode(data.trim()).map_err(|e| format!("Invalid base64: {}", e))
}

fn infer_format(bytes: &[u8], mime: &str) -> Result<ImageFormat, String> {
    if mime.eq_ignore_ascii_case("image/png") {
        Ok(ImageFormat::Png)
    } else if mime.eq_ignore_ascii_case("image/jpeg") || mime.eq_ignore_ascii_case("image/jpg") {
        Ok(ImageFormat::Jpeg)
    } else if mime.eq_ignore_ascii_case("image/webp") {
        Ok(ImageFormat::WebP)
    } else {
        image::guess_format(bytes).map_err(|e| format!("Cannot detect image format: {}", e))
    }
}

fn ext_for_format(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpg",
        ImageFormat::WebP => "webp",
        ImageFormat::Gif => "gif",
        ImageFormat::Bmp => "bmp",
        _ => "img",
    }
}

fn save_original(img: &DynamicImage, format: ImageFormat, path: &Path) -> Result<(), String> {
    let mut file = fs::File::create(path).map_err(|e| format!("Write image error: {}", e))?;
    img.write_to(&mut file, format)
        .map_err(|e| format!("Encode image error: {}", e))
}

fn save_thumb(img: &DynamicImage, path: &Path) -> Result<(), String> {
    let thumb = img.thumbnail(THUMB_SIZE, THUMB_SIZE);
    let mut file = fs::File::create(path).map_err(|e| format!("Write thumb error: {}", e))?;
    thumb
        .write_to(&mut file, ImageFormat::WebP)
        .map_err(|e| format!("Encode thumb error: {}", e))
}

fn compute_checksum(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    format!("{:x}", digest)
}

fn image_dir(app_handle: &AppHandle, user_uuid: &str, location_uuid: &str) -> Result<PathBuf, String> {
    let mut base = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("app_data_dir error: {}", e))?;
    base.push("locations");
    base.push(user_uuid);
    base.push(location_uuid);
    Ok(base)
}

