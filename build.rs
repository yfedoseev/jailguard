//! Build script for jailguard.
//!
//! Currently a no-op. The C ABI header at `include/jailguard.h` is
//! committed to git and regenerated explicitly via `make regen-c-header`
//! (which invokes the standalone `cbindgen` binary). This avoids the
//! quirks of running cbindgen from build.rs (where macro expansion is
//! incomplete and several symbols come through as opaque).

fn main() {
    println!("cargo:rerun-if-changed=src/c_api.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");

    // napi-rs build helper — only runs when the `napi` feature is on.
    #[cfg(feature = "napi")]
    napi_build::setup();
}
