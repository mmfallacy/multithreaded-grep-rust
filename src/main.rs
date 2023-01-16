use std::process::{Command, Output};

fn grep() -> Output {
    Command::new("ls")
        .output()
        .expect(" Failed to execute grep")
}

fn main() {
    println!("Hello, world!");

    let res = grep();
    println!(
        "Success: {:?}\nOutput:\n{}",
        res.status,
        String::from_utf8_lossy(&res.stdout)
    );
}
