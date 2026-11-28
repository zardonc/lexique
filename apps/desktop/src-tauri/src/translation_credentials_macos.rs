use security_framework::os::macos::keychain::{SecKeychain, SecPreferencesDomain};

use crate::translation_store::{CredentialStore, valid_slot};

const SERVICE: &str = "io.github.zardonc.lexique.translation";
const ITEM_NOT_FOUND: i32 = -25300;

pub(crate) struct MacCredentials(SecKeychain);

impl MacCredentials {
    pub(crate) fn new() -> Result<Self, String> {
        SecKeychain::default_for_domain(SecPreferencesDomain::User)
            .map(Self)
            .map_err(|error| format!("Could not open user Keychain (OS code {})", error.code()))
    }
}

impl CredentialStore for MacCredentials {
    fn read(&self, slot: &str) -> Result<Option<String>, String> {
        valid_slot(slot)?;
        match self.0.find_generic_password(SERVICE, slot) {
            Ok((password, _)) => String::from_utf8(password.as_ref().to_vec())
                .map(Some)
                .map_err(|_| "Unreadable credential".into()),
            Err(error) if error.code() == ITEM_NOT_FOUND => Ok(None),
            Err(error) => Err(format!("Unreadable credential (OS code {})", error.code())),
        }
    }

    fn write(&self, slot: &str, secret: &str) -> Result<(), String> {
        valid_slot(slot)?;
        self.0
            .set_generic_password(SERVICE, slot, secret.as_bytes())
            .map_err(|error| format!("Could not save credential (OS code {})", error.code()))
    }

    fn clear(&self, slot: &str) -> Result<(), String> {
        valid_slot(slot)?;
        match self.0.find_generic_password(SERVICE, slot) {
            Ok((_, item)) => item.delete(),
            Err(error) if error.code() == ITEM_NOT_FOUND => return Ok(()),
            Err(error) => {
                return Err(format!(
                    "Could not clear credential (OS code {})",
                    error.code()
                ));
            }
        }
        match self.0.find_generic_password(SERVICE, slot) {
            Err(error) if error.code() == ITEM_NOT_FOUND => Ok(()),
            _ => Err("Could not clear credential".into()),
        }
    }
}
