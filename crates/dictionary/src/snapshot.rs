use std::{fs::File, io::Read, path::Path};

use rusqlite::{Connection, MAIN_DB};
use sha2::{Digest, Sha256};

use crate::{DictionaryError, failure};

pub(crate) fn open(
    path: &Path,
    expected: &str,
    maximum: u64,
) -> Result<Connection, DictionaryError> {
    let mut data = Vec::new();
    File::open(path)
        .and_then(|file| file.take(maximum + 1).read_to_end(&mut data))
        .map_err(|error| {
            let code = if error.kind() == std::io::ErrorKind::NotFound {
                "missing"
            } else {
                "query_failed"
            };
            failure(
                code,
                &format!("Could not read resource {}", path.display()),
                error,
            )
        })?;
    if data.len() as u64 > maximum
        || !data.starts_with(b"SQLite format 3\0")
        || format!("{:x}", Sha256::digest(&data)) != expected
    {
        return Err(failure(
            "corrupt",
            "Resource checksum mismatch",
            path.display(),
        ));
    }
    let mut connection = Connection::open_in_memory()
        .map_err(|error| failure("query_failed", "Could not open resource connection", error))?;
    // Query the verified bytes even if the bundled file is replaced during lookup.
    connection
        .deserialize_read_exact(MAIN_DB, data.as_slice(), data.len(), true)
        .map_err(|error| failure("corrupt", "Could not load read-only resource", error))?;
    connection
        .pragma_update(None, "trusted_schema", false)
        .map_err(|error| failure("query_failed", "Could not restrict resource schema", error))?;
    Ok(connection)
}
