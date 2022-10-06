#[cfg(any(target_os = "linux", target_os = "netbsd"))]
fn main() {
    println!("cargo:rustc-link-lib=X11");
}

#[cfg(all(
    not(target_os = "linux"),
    not(target_os = "netbsd"),
))]
fn main() {}
