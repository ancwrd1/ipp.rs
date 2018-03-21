use std::process::Command;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let version = match Command::new("git").args(&["describe", "--tags", "--dirty"]).output() {
        Ok(ref output) if output.status.success() => String::from_utf8_lossy(&output.stdout).trim().to_string(),
        _ => VERSION.to_string()
    };

    println!("cargo:rustc-env=GIT_VERSION={}", version);
}
