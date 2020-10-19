#[cfg(target_os = "linux")]
fn main() {
    println!("cargo:rustc-link-lib=X11");
}

#[cfg(not(target_os = "linux"))]
fn main() {}
