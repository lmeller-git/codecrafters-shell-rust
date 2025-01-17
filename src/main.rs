#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // Uncomment this block to pass the first stage
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let mut temp = input.split_whitespace();
        if let Some(code) = temp.next() {
            if code == "exit" {
                return;
            }
        }
        println!("{}: command not found", input.trim());
    }
}
