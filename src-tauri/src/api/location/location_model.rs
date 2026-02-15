use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tracing::instrument;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Location {
    pub id: i64,
    pub uuid: String,
    pub user_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub address: String,
    pub is_active: bool,
    pub image_path: Option<String>,
    pub thumb_path: Option<String>,
    pub image_original_name: Option<String>,
    pub image_mime: Option<String>,
    pub image_size_bytes: Option<i64>,
    pub image_checksum_sha256: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocationPublic {
    pub uuid: String,
    pub name: String,
    pub description: Option<String>,
    pub address: String,
    pub is_active: bool,
    pub image_path: Option<String>,
    pub thumb_path: Option<String>,
    pub image_original_name: Option<String>,
    pub image_mime: Option<String>,
    pub image_size_bytes: Option<i64>,
    pub image_checksum_sha256: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct LocationCreateInput {
    pub name: String,
    pub address: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LocationImageInput {
    pub data_base64: String,
    pub original_name: String,
    pub mime: String,
    pub size_bytes: usize,
}

#[derive(Debug, Deserialize)]
pub struct LocationCreateCommandInput {
    pub location: LocationCreateInput,
    pub image: Option<LocationImageInput>,
}

#[derive(Debug, Deserialize)]
pub struct LocationDeleteInput {
    pub uuid: String,
}

#[derive(Debug, Deserialize)]
pub struct LocationListParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct LocationListResponse {
    pub items: Vec<LocationPublic>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug)]
pub struct LocationCreateDB {
    pub uuid: String,
    pub user_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub address: String,
    pub is_active: bool,
    pub image_path: Option<String>,
    pub thumb_path: Option<String>,
    pub image_original_name: Option<String>,
    pub image_mime: Option<String>,
    pub image_size_bytes: Option<i64>,
    pub image_checksum_sha256: Option<String>,
}

impl LocationCreateInput {
    /// Validate mandatory fields and sanitize for insertion.
    #[instrument(skip(self))]
    pub fn validate(&self) -> Result<(), String> {
        let name = self.name.trim();
        let address = self.address.trim();

        if name.is_empty() {
            return Err("Name is required".to_string());
        }
        if address.is_empty() {
            return Err("Address is required".to_string());
        }
        if name.len() > 255 {
            return Err("Name is too long (max 255)".to_string());
        }
        if address.len() > 512 {
            return Err("Address is too long (max 512)".to_string());
        }

        Ok(())
    }

    /// Build the DB struct after validation; image fields can be filled later.
    pub fn to_db(
        &self,
        user_id: i64,
        image_paths: Option<(String, String)>,
        image_meta: Option<(String, String, i64, String)>,
    ) -> LocationCreateDB {
        let (image_path, thumb_path) = image_paths
            .map(|(img, thumb)| (Some(img), Some(thumb)))
            .unwrap_or((None, None));

        let (image_original_name, image_mime, image_size_bytes, image_checksum_sha256) =
            image_meta
                .map(|(name, mime, size, checksum)| {
                    (Some(name), Some(mime), Some(size), Some(checksum))
                })
                .unwrap_or((None, None, None, None));

        LocationCreateDB {
            uuid: uuid::Uuid::new_v4().to_string(),
            user_id,
            name: self.name.trim().to_string(),
            description: self.description.as_ref().map(|d| d.trim().to_string()),
            address: self.address.trim().to_string(),
            is_active: true,
            image_path,
            thumb_path,
            image_original_name,
            image_mime,
            image_size_bytes,
            image_checksum_sha256,
        }
    }
}

