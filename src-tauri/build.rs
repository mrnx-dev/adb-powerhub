fn main() {
    let git_commit = option_env!("GIT_COMMIT").unwrap_or("unknown");
    println!("cargo:rustc-env=GIT_COMMIT={}", git_commit);
    tauri_build::build()
}