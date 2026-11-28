#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dpapi_separates_provider_slots_and_rejects_damaged_ciphertext() {
        let ciphertext = protect(b"sample key", "deepl").unwrap();
        assert_ne!(ciphertext, b"sample key");
        assert_eq!(unprotect(&ciphertext, "deepl").unwrap(), b"sample key");
        assert!(unprotect(&ciphertext, "openai_compatible").is_err());
        assert!(unprotect(b"damaged", "deepl").is_err());
    }

    #[test]
    fn slots_survive_updates_and_clear_without_exposing_plaintext() {
        let root = std::env::temp_dir().join(format!("lexique-credentials-{}", std::process::id()));
        assert!(!root.exists());
        std::fs::create_dir_all(root.join("Lexique-data").join("db")).unwrap();
        let database = root.join("Lexique-data").join("db").join("lexique.sqlite3");
        let store = WindowsCredentials::new(&database).unwrap();
        assert!(store.read("deepl").unwrap().is_none());
        store.write("deepl", "private-deepl").unwrap();
        store.write("openai_compatible", "private-model").unwrap();
        assert_eq!(
            store.read("deepl").unwrap().as_deref(),
            Some("private-deepl")
        );
        let contents =
            std::fs::read(root.join("Lexique-data/config/service-credentials.json")).unwrap();
        assert!(!String::from_utf8_lossy(&contents).contains("private-"));
        let path = root.join("Lexique-data/config/service-credentials.json");
        let mut slots: std::collections::BTreeMap<String, String> =
            serde_json::from_slice(&contents).unwrap();
        slots.insert("deepl".into(), "not-valid-base64!".into());
        std::fs::write(path, serde_json::to_vec(&slots).unwrap()).unwrap();
        assert!(store.read("deepl").is_err());
        store.clear("deepl").unwrap();
        assert!(store.read("deepl").unwrap().is_none());
        assert_eq!(
            store.read("openai_compatible").unwrap().as_deref(),
            Some("private-model")
        );
        drop(store);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn corrupt_store_is_preserved_before_clearing_a_slot() {
        let root = std::env::temp_dir().join(format!(
            "lexique-corrupt-credentials-{}",
            std::process::id()
        ));
        assert!(!root.exists());
        std::fs::create_dir_all(root.join("Lexique-data/db")).unwrap();
        let store = WindowsCredentials::new(&root.join("Lexique-data/db/lexique.sqlite3")).unwrap();
        let directory = root.join("Lexique-data/config");
        let corrupt = b"{\"deepl\":\"truncated";
        std::fs::write(directory.join("service-credentials.json"), corrupt).unwrap();
        assert!(store.read("deepl").is_err());
        store.clear("deepl").unwrap();
        assert!(store.read("deepl").unwrap().is_none());
        let backups: Vec<_> = std::fs::read_dir(&directory)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(".credentials-corrupt-")
            })
            .collect();
        assert_eq!(backups.len(), 1);
        assert_eq!(std::fs::read(&backups[0]).unwrap(), corrupt);
        drop(store);
        std::fs::remove_dir_all(root).unwrap();
    }
}
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    path::Path,
};

use base64::{Engine, engine::general_purpose::STANDARD};
use cap_fs_ext::OpenOptionsFollowExt;
use cap_primitives::fs::FollowSymlinks;
use cap_std::fs::{Dir, MetadataExt, OpenOptions};
use windows::Win32::{
    Foundation::{HLOCAL, LocalFree},
    Security::Cryptography::{
        CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData, CryptUnprotectData,
    },
};

use crate::translation_store::{CredentialStore, valid_slot};

pub(crate) struct WindowsCredentials(Dir);

impl WindowsCredentials {
    pub(crate) fn new(database: &Path) -> Result<Self, String> {
        let data = crate::speech_cache::data_directory(database)?;
        Ok(Self(crate::speech_cache::create_child(&data, "config")?))
    }

    fn load(&self) -> Result<BTreeMap<String, String>, String> {
        let mut options = OpenOptions::new();
        options.read(true).follow(FollowSymlinks::No);
        let file = match self.0.open_with("service-credentials.json", &options) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(BTreeMap::new());
            }
            Err(error) => {
                return Err(format!(
                    "Could not open credentials (OS code {:?})",
                    error.raw_os_error()
                ));
            }
        };
        let metadata = file.metadata().map_err(|error| {
            format!(
                "Could not inspect credentials (OS code {:?})",
                error.raw_os_error()
            )
        })?;
        if !metadata.is_file() || metadata.file_attributes() & 0x400 != 0 || metadata.len() > 65536
        {
            return Err("Invalid credentials file".into());
        }
        let mut content = Vec::new();
        file.take(65537)
            .read_to_end(&mut content)
            .map_err(|error| {
                format!(
                    "Could not read credentials (OS code {:?})",
                    error.raw_os_error()
                )
            })?;
        if content.len() > 65536 {
            return Err("Invalid credentials file".into());
        }
        serde_json::from_slice(&content).map_err(|_| "Corrupt credentials file".into())
    }

    fn load_for_update(&self) -> Result<BTreeMap<String, String>, String> {
        match self.load() {
            Ok(slots) => Ok(slots),
            Err(error) if error == "Corrupt credentials file" => {
                let mut input_options = OpenOptions::new();
                input_options.read(true).follow(FollowSymlinks::No);
                let input = self
                    .0
                    .open_with("service-credentials.json", &input_options)
                    .map_err(|error| {
                        format!(
                            "Could not preserve credentials (OS code {:?})",
                            error.raw_os_error()
                        )
                    })?;
                let metadata = input.metadata().map_err(|error| {
                    format!(
                        "Could not inspect credentials (OS code {:?})",
                        error.raw_os_error()
                    )
                })?;
                if !metadata.is_file()
                    || metadata.file_attributes() & 0x400 != 0
                    || metadata.len() > 65536
                {
                    return Err("Invalid credentials file".into());
                }
                let nonce = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|_| "Could not preserve credentials")?
                    .as_nanos();
                let backup = format!(".credentials-corrupt-{}-{nonce}.json", std::process::id());
                let mut output_options = OpenOptions::new();
                output_options
                    .write(true)
                    .create_new(true)
                    .follow(FollowSymlinks::No);
                let mut output = self
                    .0
                    .open_with(&backup, &output_options)
                    .map_err(|error| {
                        format!(
                            "Could not preserve credentials (OS code {:?})",
                            error.raw_os_error()
                        )
                    })?;
                let copied =
                    std::io::copy(&mut input.take(65537), &mut output).map_err(|error| {
                        format!(
                            "Could not preserve credentials (OS code {:?})",
                            error.raw_os_error()
                        )
                    })?;
                if copied > 65536 {
                    return Err("Invalid credentials file".into());
                }
                output.sync_all().map_err(|error| {
                    format!(
                        "Could not preserve credentials (OS code {:?})",
                        error.raw_os_error()
                    )
                })?;
                Ok(BTreeMap::new())
            }
            Err(error) => Err(error),
        }
    }

    fn save(&self, slots: &BTreeMap<String, String>) -> Result<(), String> {
        let content = serde_json::to_vec(slots).map_err(|_| "Could not encode credentials")?;
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| "Could not create credentials file")?
            .as_nanos();
        let temporary = format!(".credentials-{}-{nonce}.tmp", std::process::id());
        let mut options = OpenOptions::new();
        options
            .write(true)
            .create_new(true)
            .follow(FollowSymlinks::No);
        let mut file = self.0.open_with(&temporary, &options).map_err(|error| {
            format!(
                "Could not create credentials file (OS code {:?})",
                error.raw_os_error()
            )
        })?;
        let result = (|| {
            file.write_all(&content).map_err(|error| {
                format!(
                    "Could not write credentials (OS code {:?})",
                    error.raw_os_error()
                )
            })?;
            file.sync_all().map_err(|error| {
                format!(
                    "Could not sync credentials (OS code {:?})",
                    error.raw_os_error()
                )
            })?;
            drop(file);
            self.0
                .rename(&temporary, &self.0, "service-credentials.json")
                .map_err(|error| {
                    format!(
                        "Could not publish credentials (OS code {:?})",
                        error.raw_os_error()
                    )
                })?;
            Ok(())
        })();
        let _ = self.0.remove_file(&temporary);
        result
    }
}

impl CredentialStore for WindowsCredentials {
    fn read(&self, slot: &str) -> Result<Option<String>, String> {
        valid_slot(slot)?;
        let _lock = crate::speech_cache::lock(&self.0)?;
        let slots = self.load()?;
        slots
            .get(slot)
            .map(|encoded| {
                let encrypted = STANDARD
                    .decode(encoded)
                    .map_err(|_| "Unreadable credential")?;
                let plain = unprotect(&encrypted, slot)?;
                String::from_utf8(plain).map_err(|_| "Unreadable credential".into())
            })
            .transpose()
    }

    fn write(&self, slot: &str, secret: &str) -> Result<(), String> {
        valid_slot(slot)?;
        let encrypted = protect(secret.as_bytes(), slot)?;
        let _lock = crate::speech_cache::lock(&self.0)?;
        let mut slots = self.load_for_update()?;
        slots.insert(slot.into(), STANDARD.encode(encrypted));
        self.save(&slots)
    }

    fn clear(&self, slot: &str) -> Result<(), String> {
        valid_slot(slot)?;
        let _lock = crate::speech_cache::lock(&self.0)?;
        let mut slots = self.load_for_update()?;
        slots.remove(slot);
        self.save(&slots)
    }
}

fn crypt(input: &[u8], slot: &str, encrypt: bool) -> Result<Vec<u8>, String> {
    let input = CRYPT_INTEGER_BLOB {
        cbData: input
            .len()
            .try_into()
            .map_err(|_| "Credential is too large")?,
        pbData: input.as_ptr().cast_mut(),
    };
    let entropy = CRYPT_INTEGER_BLOB {
        cbData: slot
            .len()
            .try_into()
            .map_err(|_| "Invalid credential slot")?,
        pbData: slot.as_ptr().cast_mut(),
    };
    let mut output = CRYPT_INTEGER_BLOB::default();
    // DPAPI allocates output with LocalAlloc; both input buffers remain alive through the call.
    unsafe {
        if encrypt {
            CryptProtectData(
                &input,
                None,
                Some(&entropy),
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        } else {
            CryptUnprotectData(
                &input,
                None,
                Some(&entropy),
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        }
        .map_err(|error| format!("Credential protection failed (OS code {})", error.code().0))?;
        let bytes = if output.cbData == 0 {
            Vec::new()
        } else {
            std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec()
        };
        let _ = LocalFree(Some(HLOCAL(output.pbData.cast())));
        Ok(bytes)
    }
}

fn protect(input: &[u8], slot: &str) -> Result<Vec<u8>, String> {
    crypt(input, slot, true)
}

fn unprotect(input: &[u8], slot: &str) -> Result<Vec<u8>, String> {
    crypt(input, slot, false)
}
