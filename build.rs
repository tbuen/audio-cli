use std::env;
use std::process::Command;

fn main() {
    if env::var("PROFILE").unwrap() == "debug" {
        let output = Command::new("git").arg("describe").arg("--always").arg("--tags").arg("--dirty").output().unwrap();
        let version = String::from_utf8(output.stdout).unwrap();
        println!("cargo:rustc-env=VERSION={}", version);
    } else {
        println!("cargo:rustc-env=VERSION={}", env::var("CARGO_PKG_VERSION").unwrap());
    }
}
