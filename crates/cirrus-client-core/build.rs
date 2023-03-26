use build_target;

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

fn main() {
    let target_os = build_target::target_os().unwrap();

    if target_os == build_target::Os::Android {
        add_lib("c++_shared", false);
    }
}
