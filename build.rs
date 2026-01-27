fn main() {
    // On Windows, we need to link additional system libraries for git2
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=advapi32");
        println!("cargo:rustc-link-lib=crypt32");
        println!("cargo:rustc-link-lib=secur32");
        println!("cargo:rustc-link-lib=user32");
    }
}
