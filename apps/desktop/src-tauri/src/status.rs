use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatus {
    pub version: String,
    pub schema_version: u32,
    pub storage: StorageKind,
    pub audio_migration: lexique_storage::AudioMigrationStatus,
    pub audio_cache: Option<AudioCacheStatus>,
}

#[derive(Clone, Serialize)]
pub struct AudioCacheStatus {
    #[serde(flatten)]
    pub files: crate::speech_cache::CacheReport,
    #[serde(flatten)]
    pub usage: lexique_storage::AudioUsage,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageKind {
    Sqlite,
}
