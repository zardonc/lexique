use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatus {
    pub version: String,
    pub schema_version: u32,
    pub storage: StorageKind,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageKind {
    Sqlite,
}
