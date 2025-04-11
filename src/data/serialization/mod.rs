mod pack_info;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum TextComponent {
    String(String),
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ResourceLocation(String);