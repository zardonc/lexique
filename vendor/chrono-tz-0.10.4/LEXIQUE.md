# Timezone data

Chrono-tz 0.10.4 retains its upstream runtime and API. Its bundled zone inputs and prebuilt tables use IANA 2026f, including Vancouver's permanent UTC−7 and Cuba's November 2026 rules.

- Runtime: [chrono-tz 0.10.4](https://github.com/chronotope/chrono-tz/tree/e15d1f308a1ac2fdf3f1bf19c325dc202d418081), MIT OR Apache-2.0; see LICENSE.
- Data: [IANA tzdata2026f](https://data.iana.org/time-zones/releases/tzdata2026f.tar.gz), released 2026-10-27. SHA-256: `01b4a91cec5167ed48c11e8ab78f2048cead78ce94b0cafcad0ae8c5a3044d3d`. Data licensing is retained in tz/LICENSE.
- Generator: upstream chrono-tz-build 0.5.0 and parse-zoneinfo 0.4.1, fixed by this directory's Cargo.lock.

Reproduce or verify the tables from this directory with `cargo test --locked --test codegen`. The upstream test writes the tables and succeeds when they match the checked-in output. Run generator checks separately from application performance measurements.

The root Cargo patch selects these same tables for native and WASM builds. New scheduling records identify timezone database 2026f. Existing absolute due timestamps and rating records remain unchanged. Requests explicitly replaying a different database version require their recorded core build, as specified by the scheduling API.
