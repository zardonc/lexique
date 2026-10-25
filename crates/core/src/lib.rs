pub const DATABASE_SCHEMA_VERSION: u32 = 2;

// The export name is unique within this crate and has no memory access.
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
pub extern "C" fn schema_version() -> u32 {
    DATABASE_SCHEMA_VERSION
}
