// ref: https://github.com/rust-windowing/android-ndk-rs/issues/167#issuecomment-1001103028
// ref: https://github.com/katyo/oboe-rs/issues/28
fn add_lib(_name: impl AsRef<str>, _static: bool) {
    #[cfg(not(feature = "test"))]
    println!(
        "cargo:rustc-link-lib={}{}",
        if _static { "static=" } else { "" },
        _name.as_ref()
    );
}

#[cfg(target_os = "android")]
fn main() {
    // println!("cargo:rustc-link-lib==c++_shared")
    add_lib("c++_shared", false);
}

#[cfg(not(target_os = "android"))]
fn main() {
    add_lib("c++_shared", false);
}