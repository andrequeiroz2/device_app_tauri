use crate::api::device::device_model::DeviceType;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::str::FromStr;

/// Tipo de alias para categoria de ícone; reutiliza `DeviceType` (sensor, actuator).
pub type IconCategory = DeviceType;

pub(crate) fn parse_icon_category(s: &str) -> Result<IconCategory, String> {
    match s.trim().to_lowercase().as_str() {
        "sensor" => Ok(DeviceType::Sensor),
        "actuator" => Ok(DeviceType::Actuator),
        _ => Err(format!("categoria inválida: {}", s)),
    }
}

/// Struct de banco (não expor `id` na API).
#[derive(Debug, FromRow)]
pub struct Icon {
    pub id: i64,
    pub uuid: String,
    pub code: String,
    pub name: String,
    pub iconify_id: String,
    pub category: String,
    pub color: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Resposta pública para API (sem `id`).
#[derive(Debug, Serialize, Deserialize)]
pub struct IconPublic {
    pub uuid: String,
    pub code: String,
    pub name: String,
    pub iconify_id: String,
    pub category: String,
    pub color: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Icon> for IconPublic {
    fn from(icon: Icon) -> Self {
        Self {
            uuid: icon.uuid,
            code: icon.code,
            name: icon.name,
            iconify_id: icon.iconify_id,
            category: icon.category,
            color: icon.color,
            is_active: icon.is_active,
            created_at: icon.created_at,
            updated_at: icon.updated_at,
        }
    }
}

/// Input para criação de ícone (code gerado no backend).
#[derive(Debug, Deserialize)]
pub struct IconCreateInput {
    pub name: String,
    pub iconify_id: String,
    pub category: String,
    pub color: Option<String>,
}

impl IconCreateInput {
    /// Valida campos e retorna cor como hex (validado via IconColor) ou None.
    pub fn validate(&self) -> Result<(String, String, IconCategory, Option<String>), String> {
        let name = self.name.trim();
        let iconify_id = self.iconify_id.trim();

        if name.is_empty() {
            return Err("name is required".to_string());
        }
        if iconify_id.is_empty() {
            return Err("iconify_id is required".to_string());
        }
        if !iconify_id.contains(':') {
            return Err("iconify_id must be in format prefix:icon-name".to_string());
        }
        if name.len() > 255 {
            return Err("name is too long (max 255)".to_string());
        }

        let category = parse_icon_category(&self.category)
            .map_err(|e| e.to_string())?;

        let color_hex = match &self.color {
            Some(c) if !c.trim().is_empty() => {
                let color = IconColor::from_str(c).map_err(|e| e.to_string())?;
                Some(color.as_hex().to_string())
            }
            _ => None,
        };

        Ok((name.to_string(), iconify_id.to_string(), category, color_hex))
    }
}

/// Input para atualização de ícone.
#[derive(Debug, Deserialize)]
pub struct IconUpdateInput {
    pub uuid: String,
    pub name: Option<String>,
    pub iconify_id: Option<String>,
    pub category: Option<String>,
    pub color: Option<String>,
    pub is_active: Option<bool>,
}

/// Input para delete de ícone.
#[derive(Debug, Deserialize)]
pub struct IconDeleteInput {
    pub uuid: String,
}

impl IconUpdateInput {
    /// Valida os campos opcionais. Retorna (color_hex_validated) se color foi enviado.
    pub fn validate_color(&self) -> Result<Option<String>, String> {
        match &self.color {
            Some(c) if !c.trim().is_empty() => {
                let color = IconColor::from_str(c).map_err(|e| e.to_string())?;
                Ok(Some(color.as_hex().to_string()))
            }
            _ => Ok(None),
        }
    }

    /// Valida name se presente.
    pub fn validate_name(&self) -> Result<(), String> {
        if let Some(ref n) = self.name {
            let t = n.trim();
            if t.is_empty() {
                return Err("name cannot be empty".to_string());
            }
            if t.len() > 255 {
                return Err("name is too long (max 255)".to_string());
            }
        }
        Ok(())
    }

    /// Valida iconify_id se presente.
    pub fn validate_iconify_id(&self) -> Result<(), String> {
        if let Some(ref i) = self.iconify_id {
            let t = i.trim();
            if t.is_empty() || !t.contains(':') {
                return Err("iconify_id must be in format prefix:icon-name".to_string());
            }
        }
        Ok(())
    }

    /// Valida category se presente.
    pub fn validate_category(&self) -> Result<(), String> {
        if let Some(ref c) = self.category {
            parse_icon_category(c).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

/// Filtro de status (active = só ativos, all = todos).
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum IconStatusFilter {
    #[default]
    Active,
    All,
}

/// Parâmetros de listagem (filtro por category + status + paginação).
#[derive(Debug, Deserialize, Clone, Default)]
pub struct IconListParams {
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub status: Option<IconStatusFilter>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub page_size: Option<u32>,
}

/// Resposta paginada da listagem de ícones.
#[derive(Debug, Serialize)]
pub struct IconListResponse {
    pub items: Vec<IconPublic>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}

/// Dados para insert (handler constrói a partir de IconCreateInput).
#[derive(Debug)]
pub struct IconCreateDB {
    pub uuid: String,
    pub code: String,
    pub name: String,
    pub iconify_id: String,
    pub category: String,
    pub color: Option<String>,
}

/// Dados para update parcial (handler constrói a partir de IconUpdateInput).
#[derive(Debug, Default)]
pub struct IconUpdateDB {
    pub name: Option<String>,
    pub iconify_id: Option<String>,
    pub code: Option<String>,
    pub category: Option<String>,
    pub color: Option<String>,
    pub is_active: Option<bool>,
}

/// Paleta de cores permitidas para ícones. O banco armazena o hex (TEXT);
/// validação garante que apenas valores do enum são aceitos.
/// API: serializa como hex (#E53935), deserializa de hex ou nome (red, blue).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconColor {
    Red,
    Blue,
    Green,
    Orange,
    Purple,
    Yellow,
    Cyan,
    Indigo,
    DeepPurple,
    Teal,
    Grey,
}

impl IconColor {
    /// Retorna o valor hex para uso no frontend (Iconify).
    pub fn as_hex(&self) -> &'static str {
        match self {
            Self::Red => "#E53935",
            Self::Blue => "#1E88E5",
            Self::Green => "#43A047",
            Self::Orange => "#FB8C00",
            Self::Purple => "#8E24AA",
            Self::Yellow => "#FDD835",
            Self::Cyan => "#00ACC1",
            Self::Indigo => "#5C6BC0",
            Self::DeepPurple => "#7B1FA2",
            Self::Teal => "#26A69A",
            Self::Grey => "#78909C",
        }
    }

    /// Lista todas as cores para o frontend (dropdown, etc).
    pub fn all() -> &'static [IconColor] {
        &[
            IconColor::Red,
            IconColor::Blue,
            IconColor::Green,
            IconColor::Orange,
            IconColor::Purple,
            IconColor::Yellow,
            IconColor::Cyan,
            IconColor::Indigo,
            IconColor::DeepPurple,
            IconColor::Teal,
            IconColor::Grey,
        ]
    }
}

impl FromStr for IconColor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_lowercase();
        match normalized.as_str() {
            "#e53935" | "red" => Ok(Self::Red),
            "#1e88e5" | "blue" => Ok(Self::Blue),
            "#43a047" | "green" => Ok(Self::Green),
            "#fb8c00" | "orange" => Ok(Self::Orange),
            "#8e24aa" | "purple" => Ok(Self::Purple),
            "#fdd835" | "yellow" => Ok(Self::Yellow),
            "#00acc1" | "cyan" => Ok(Self::Cyan),
            "#5c6bc0" | "indigo" => Ok(Self::Indigo),
            "#7b1fa2" | "deeppurple" => Ok(Self::DeepPurple),
            "#26a69a" | "teal" => Ok(Self::Teal),
            "#78909c" | "grey" | "gray" => Ok(Self::Grey),
            _ => Err(format!("cor inválida: {}", s)),
        }
    }
}

impl Serialize for IconColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_hex())
    }
}

impl<'de> Deserialize<'de> for IconColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}
