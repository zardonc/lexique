# Native backup dependency notices

The desktop backup container uses `zip` 9.5.1 with default features disabled and Stored entries. Native file selection uses `tauri-plugin-dialog` 2.8.4. Both versions and the new transitive packages below are pinned by the workspace Cargo.lock. This directory contains the original notices from those exact published crate archives; application code from these packages has not been modified.

The desktop resource mapping and Windows portable packager deliver this directory as `backup-notices/`. It covers the packages added for this feature, including platform-specific dependencies. Existing resource and application dependency notices retain their separate delivery paths.

| Package | Selected license | Upstream |
| --- | --- | --- |
| zip 9.5.1 | MIT | [zip-rs/zip2](https://github.com/zip-rs/zip2/tree/v9.5.1) |
| tauri-plugin-dialog 2.8.4 | MIT | [Tauri plugins workspace](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/dialog) |
| tauri-plugin-fs 2.7.1 | MIT | [Tauri plugins workspace](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/fs) |
| rfd 0.17.2 | MIT | [PolyMeilex/rfd](https://github.com/PolyMeilex/rfd) |
| cfg_aliases 0.2.2 | MIT | [katharostech/cfg_aliases](https://github.com/katharostech/cfg_aliases) |
| constant_time_eq 0.6.1 | MIT-0 | [cesarb/constant_time_eq](https://codeberg.org/cesarb/constant_time_eq) |
| nix 0.31.3 | MIT | [nix-rust/nix](https://github.com/nix-rust/nix) |
| typed-path 0.13.0 | MIT | [chipsenkbeil/typed-path](https://github.com/chipsenkbeil/typed-path) |
| zeroize 1.9.1 | MIT | [RustCrypto/utils](https://github.com/RustCrypto/utils) |
| zeroize_derive 1.5.0 | MIT | [RustCrypto/utils](https://github.com/RustCrypto/utils) |

Each package subdirectory preserves its selected license. `cfg_aliases` also includes its upstream third-party notice, and `rfd` includes the MIT notice for its XDG window-identifier implementation. The Tauri SPDX copyright records are preserved beside the selected MIT licenses. The optional Apache-2.0 and CC0 alternatives are not selected for this delivery.

The filesystem plugin is a native transitive dependency of the dialog plugin. The main WebView receives only Lexique's restricted backup commands; it is not granted filesystem, dialog or shell plugin permissions.
