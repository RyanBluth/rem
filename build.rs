use std::process::Command;
use std::env;

fn main() {
    Command::new("bash").args(&["script/build.sh"])
                        .arg(env::var("PROFILE").unwrap())
                        .status().unwrap();
}