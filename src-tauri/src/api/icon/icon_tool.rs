/// Gera o `code` a partir do `iconify_id` (parte após o `:`).
/// Ex.: "mdi:thermometer" -> "thermometer"
pub fn compose_icon_code(iconify_id: &str) -> String {
    iconify_id
        .rsplit_once(':')
        .map(|(_, name)| name.to_string())
        .unwrap_or_else(|| iconify_id.to_string())
}
