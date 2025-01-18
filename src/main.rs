#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};
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
            Err(ShellError::UnknownType(t)) => println!("{t}: not found"),
            Err(ShellError::Exit(code)) => match code.as_str() {
                "0" => return Ok(()),
                _ => return Err(ShellError::Exit(code)),
            },
            Err(e) => return Err(e),
            Ok(c) => match c.execute() {
                Ok(()) => {}
                Err(ShellError::IO(_)) => {
                    println!("{}: No such file or directory", c)
                }
                Err(e) => println!("{:#?}", e),
            },
        }
    }
}

type Result<T> = std::result::Result<T, ShellError>;

#[derive(Error, Debug)]
enum ShellError {
    #[error("IOError {0}")]
    IO(#[from] std::io::Error),
    #[error("Unknown error {0}")]
    _Unknown(String),
    #[error("{0}: command not found")]
    NotImplemented(String),
    #[error("exit code {0} != 0")]
    Exit(String),
    #[error("type not known {0}")]
    UnknownType(String),
    #[error("Env error: {0}")]
    Env(#[from] std::env::VarError),
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
    Type(String, Option<String>),
    External(String, Args),
    Pwd,
    Cd(String),
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cd(p) => write!(f, "cd: {}", p),
            _ => write!(f, ""),
        }
    }
}

fn handle_input(mut input: &str) -> Vec<String> {
    let mut args = Vec::new();
    while !input.is_empty() {
        if input.starts_with('\'') {
            let (next_arg, next_input) = longest_sequence(&input[1..], true, false);
            args.push(next_arg);
            input = next_input;
        } else if input.starts_with('\"') {
            let (next_arg, next_input) = longest_sequence(&input[1..], false, true);
            args.push(next_arg);
            input = next_input;
        } else {
            let (next_arg, next_input) = longest_sequence(input, false, false);
            args.push(next_arg);
            input = next_input;
        }
    }

    args
}
fn longest_sequence(mut input: &str, mut in_quote: bool, mut in_d_quote: bool) -> (String, &str) {
    let mut res = String::new();
    loop {
        let nxt_quote = input.find('\'').unwrap_or(input.len());
        let nxt_wht = input.find(char::is_whitespace).unwrap_or(input.len());
        let nxt_d_quote = input.find('\"').unwrap_or(input.len());
        let nxt_bkslsh = input.find('\\').unwrap_or(input.len());
        if in_d_quote {
            if nxt_bkslsh < nxt_d_quote {
                let (in_between, next_input) = input.split_at(nxt_bkslsh);
                res.push_str(in_between);
                let (c, next_input) = next_input.split_at(2);
                let c = c.chars().last().unwrap();
                if c == '\\' || c == '\"' || c == '$' || c == '\n' {
                    res.push(c);
                } else {
                    res.push('\\');
                    res.push(c);
                }
                input = next_input;
                continue;
            }
            if nxt_d_quote < input.len() {
                in_d_quote = false;
                let (sub_s, next_input) = input.split_at(nxt_d_quote);
                res.push_str(sub_s);
                input = &next_input[1..];
            } else {
                //TODO
                return (input.trim().replace('\"', ""), "");
            }
        } else if in_quote {
            if nxt_quote < input.len() {
                in_quote = false;
                let (sub_s, next_input) = input.split_at(nxt_quote);
                res.push_str(sub_s);
                input = &next_input[1..];
            } else {
                //TODO
                return (input.trim().replace('\'', ""), "");
            }
        } else if nxt_bkslsh < nxt_d_quote && nxt_bkslsh < nxt_quote && nxt_bkslsh < nxt_wht {
            let (in_between, next_input) = input.split_at(nxt_bkslsh);
            res.push_str(in_between);
            let (c, next_input) = next_input.split_at(2);
            res.push(c.chars().last().unwrap());
            input = next_input;
        } else if nxt_wht < nxt_quote && nxt_wht < nxt_d_quote {
            let (r, next_in) = input.split_once(char::is_whitespace).unwrap_or(("", ""));
            res.push_str(r);
            return (res, next_in.trim_start());
        } else if nxt_d_quote < nxt_quote {
            in_d_quote = true;
            let (in_between, next_input) = input.split_at(nxt_d_quote);
            input = &next_input[1..];
            res.push_str(in_between);
        } else {
            in_quote = true;
            let (in_between, next_input) = input.split_at(nxt_quote);
            input = &next_input[1..];
            res.push_str(in_between);
        }
    }
}

impl FromStr for Command {
    type Err = ShellError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let c;
        let mut s = if let Some((command, args)) = s.split_once(char::is_whitespace) {
            let mut s = vec![command];
            c = handle_input(args);
            s.extend(c.iter().map(|s| s.as_str()));
            s.into_iter()
        } else {
            vec![s].into_iter()
        };

        match s.next() {
            Some("exit") => Err(ShellError::Exit(s.next().unwrap_or("").into())),
            Some("type") => {
                let c = s.next();
                match c {
                    Some("echo") | Some("type") | Some("exit") | Some("pwd") | Some("cd") => {
                        Ok(Self::Type(c.expect("must contain valuet").into(), None))
                    }
                    Some(c) => match std::env::var("PATH")?
                        .split(":")
                        .map(|path| format!("{}/{}", path, c))
                        .find(|path| std::fs::metadata(path).is_ok())
                    {
                        None => Err(ShellError::UnknownType(c.into())),
                        Some(p) => Ok(Self::Type(c.into(), p.into())),
                    },
                    _ => Err(ShellError::UnknownType(c.unwrap_or("").into())),
                }
            }
            Some("echo") => Ok(Self::Echo(
                Args::default().with_args(s.map(|arg| arg.to_string()).collect()),
            )),
            Some("pwd") => Ok(Self::Pwd),
            Some("cd") => Ok(Self::Cd(s.next().unwrap_or("~").trim().into())),
            Some(c) => {
                match std::env::var("PATH")?
                    .split(":")
                    .map(|path| format!("{}/{}", path, c.trim()))
                    .find(|path| std::fs::metadata(path).is_ok())
                {
                    None => Err(ShellError::NotImplemented(c.into())),
                    Some(p) => Ok(Self::External(
                        p,
                        Args::default().with_args(s.map(|arg| arg.to_string()).collect()),
                    )),
                }
            }
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
            Self::Type(c, p) => match p {
                None => println!("{c} is a shell builtin"),
                Some(p) => println!("{} is {}", c, p),
            },
            Self::NoCommand => println!(),
            Self::Pwd => println!("{}", std::env::current_dir()?.display()),
            Self::Cd(p) => {
                let p: PathBuf = if p.starts_with("~") {
                    let home = std::env::home_dir().unwrap_or(Path::new("/").into());
                    let p = p.trim_start_matches("~").trim_start_matches("/");
                    home.join(PathBuf::from(p))
                } else {
                    PathBuf::from_str(p).unwrap_or_default()
                };
                std::env::set_current_dir(p.canonicalize()?)?;
            }
            Self::External(p, args) => {
                let output = std::process::Command::new(p.split('/').last().unwrap_or(""))
                    .args(args.args.clone())
                    .output()?;

                if output.status.success() {
                    print!("{}", String::from_utf8_lossy(&output.stdout));
                } else {
                    print!("{}", String::from_utf8_lossy(&output.stderr));
                }
            }
        }
        Ok(())
    }
}
