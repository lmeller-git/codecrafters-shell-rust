#[allow(unused_imports)]
use std::io::{self, Write};
use std::str::FromStr;
use thiserror::Error;

fn main() -> Result<()> {
    // Uncomment this block to pass the first stage
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        match Command::from_str(&input) {
            Err(ShellError::NotImplemented(e)) => println!("{e}: command not found"),
            Err(ShellError::Exit(code)) => match code.as_str() {
                "0" => return Ok(()),
                _ => return Err(ShellError::Exit(code)),
            },
            Err(e) => return Err(e),
            Ok(c) => c.execute()?,
        }
    }
}

type Result<T> = std::result::Result<T, ShellError>;

#[derive(Error, Debug)]
enum ShellError {
    #[error("IOError {0}")]
    IO(#[from] std::io::Error),
    #[error("Unknown error {0}")]
    Unknown(String),
    #[error("{0}: command not found")]
    NotImplemented(String),
    #[error("exit code {0} != 0")]
    Exit(String),
}

#[derive(Default)]
struct Args {
    args: Vec<String>,
}

impl Args {
    fn with_args(mut self, mut args: Vec<String>) -> Self {
        self.args.append(&mut args);
        self
    }
}

#[derive(Default)]
enum Command {
    Echo(Args),
    #[default]
    NoCommand,
    Type(String),
}

impl FromStr for Command {
    type Err = ShellError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut s = s.split_whitespace();
        match s.next() {
            Some("exit") => Err(ShellError::Exit(s.next().unwrap_or("").into())),
            Some("type") => {
                let c = s.next();
                match c {
                    Some("echo") | Some("type") | Some("exit") => {
                        return Ok(Self::Type(c.expect("must contain valuet").into()))
                    }
                    _ => return Err(ShellError::NotImplemented(c.unwrap_or("").into())),
                }
            }
            Some("echo") => Ok(Self::Echo(
                Args::default().with_args(s.map(|arg| arg.to_string()).collect()),
            )),
            Some(c) => Err(ShellError::NotImplemented(c.into())),
            None => Ok(Self::NoCommand),
        }
    }
}

impl Command {
    fn execute(&self) -> Result<()> {
        match self {
            Self::Echo(args) => {
                for arg in &args.args {
                    print!("{} ", arg);
                }
                println!();
            }
            Self::Type(c) => println!("{c} is a shell builtin"),
            Self::NoCommand => println!(),
        }
        Ok(())
    }
}
