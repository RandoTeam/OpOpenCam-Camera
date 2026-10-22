fn main() {
    // Camera2 NDK and media NDK are only available on Android. Host builds are
    // used for formatting, linting and pure-module tests.
    //
    // `AImageReader_*` lives in libmediandk.so, while the ACamera* API lives in
    // libcamera2ndk.so. Both must be linked explicitly; without mediandk the
    // .so loads but fails with `cannot locate symbol "AImageReader_new"`.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android") {
        println!("cargo:rustc-link-lib=camera2ndk");
        println!("cargo:rustc-link-lib=mediandk");
    }
}
