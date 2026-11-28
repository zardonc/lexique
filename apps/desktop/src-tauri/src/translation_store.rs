pub(crate) trait CredentialStore {
    fn read(&self, slot: &str) -> Result<Option<String>, String>;
    fn write(&self, slot: &str, secret: &str) -> Result<(), String>;
    fn clear(&self, slot: &str) -> Result<(), String>;
}

pub(crate) fn valid_slot(slot: &str) -> Result<(), String> {
    if matches!(slot, "deepl" | "openai_compatible") {
        Ok(())
    } else {
        Err("Invalid credential slot".into())
    }
}
