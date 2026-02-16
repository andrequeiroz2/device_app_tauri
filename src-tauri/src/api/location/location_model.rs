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
pub struct LocationUpdateInput {
    pub uuid: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub address: Option<String>,
    pub is_active: Option<bool>,
    pub image: Option<LocationImageInput>,
}

/// Filter options for location status
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LocationStatusFilter {
    /// Show only active locations (is_active = true)
    Active,
    /// Show all locations (active and inactive)
    All,
}

impl Default for LocationStatusFilter {
    fn default() -> Self {
        LocationStatusFilter::Active
    }
}


#[derive(Debug, Deserialize, Clone, Default)]
pub struct LocationFilter {
    #[serde(default)]
    pub status: LocationStatusFilter,
    
}

impl LocationFilter {
    /// Returns true if we should show all locations (including inactive)
    pub fn show_all(&self) -> bool {
        matches!(self.status, LocationStatusFilter::All)
    }
}

#[derive(Debug, Deserialize)]
pub struct LocationListParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    #[serde(default)]
    pub filter: LocationFilter,
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

#[derive(Debug)]
pub struct LocationUpdateDB {
    pub name: Option<String>,
    pub description: Option<String>,
    pub address: Option<String>,
    pub is_active: Option<bool>,
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

impl LocationUpdateInput {
    #[instrument(skip(self))]
    pub fn validate(&self) -> Result<(), String> {

        let has_updates = self.name.is_some()
            || self.description.is_some()
            || self.address.is_some()
            || self.is_active.is_some()
            || self.image.is_some();

        if !has_updates {
            return Err("At least one field must be provided for update".to_string());
        }

        if self.uuid.trim().is_empty() {
            return Err("UUID is required".to_string());
        }

        if let Some(ref name) = self.name {
            if name.trim().is_empty() {
                return Err("Name cannot be empty or contain only spaces".to_string());
            }

            if name != name.trim() {
                return Err("Name cannot have leading or trailing spaces".to_string());
            }
            
            if name.len() > 255 {
                return Err("Name is too long (max 255)".to_string());
            }
        }

        if let Some(ref address) = self.address {

            if address.trim().is_empty() {
                return Err("Address cannot be empty or contain only spaces".to_string());
            }

            if address != address.trim() {
                return Err("Address cannot have leading or trailing spaces".to_string());
            }

            if address.len() > 512 {
                return Err("Address is too long (max 512)".to_string());
            }
        }

        if let Some(ref description) = self.description {
            if !description.trim().is_empty() {
                if description != description.trim() {
                    return Err("Description cannot have leading or trailing spaces".to_string());
                }
            }
        }

        Ok(())
    }

    /// Convert to LocationUpdateDB for database operations.
    pub fn to_db(&self) -> LocationUpdateDB {
        LocationUpdateDB {
            name: self.name.as_ref().map(|n| n.trim().to_string()),
            description: self.description.as_ref().map(|d| d.trim().to_string()),
            address: self.address.as_ref().map(|a| a.trim().to_string()),
            is_active: self.is_active,
            image_path: None,
            thumb_path: None,
            image_original_name: None,
            image_mime: None,
            image_size_bytes: None,
            image_checksum_sha256: None,
        }
    }
}

impl LocationUpdateDB {
    /// Uses the same SavedImage struct from location_storage.
    pub fn with_saved_image(mut self, saved: &crate::api::location::location_storage::SavedImage) -> Self {
        self.image_path = Some(saved.image_path.clone());
        self.thumb_path = Some(saved.thumb_path.clone());
        self.image_original_name = Some(saved.original_name.clone());
        self.image_mime = Some(saved.mime.clone());
        self.image_size_bytes = Some(saved.size_bytes);
        self.image_checksum_sha256 = Some(saved.checksum_sha256.clone());

        self
    }
}

