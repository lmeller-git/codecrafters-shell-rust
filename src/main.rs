#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    fmt::Display,
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
    str::FromStr,
};
use termion::{event::Key, input::TermRead, raw::IntoRawMode};
use thiserror::Error;

fn main() -> Result<()> {
    let mut std_out = io::stdout().into_raw_mode()?;
    let stdin = io::stdin();
    write!(std_out, "$ ")?;
    std_out.flush()?;
    let mut input = String::new();
    let mut display_possibilities = false;
    let mut completions = Vec::default();
    let mut in_d_quote = false;
    let mut in_quote = false;
    std_out.flush()?;
    for k in stdin.keys() {
        match k.as_ref().unwrap() {
            Key::Char('\n') => {
                if in_quote || in_d_quote {
                    //TODO
                    //write!(std_out, " ")?;
                    //std_out.flush()?;
                    continue;
                }
                writeln!(std_out, "\r")?;
                match Command::from_str(&input) {
                    Err(ShellError::NotImplemented(e)) => println!("\r{e}: command not found"),
                    Err(ShellError::UnknownType(t)) => println!("\r{t}: not found"),
                    Err(ShellError::Exit(code)) => match code.as_str() {
                        "0" => return Ok(()),
                        _ => return Err(ShellError::Exit(code)),
                    },
                    Err(e) => return Err(e),
                    Ok(c) => match c.execute() {
                        Ok(()) => {}
                        Err(ShellError::IO(_)) => {
                            println!("\r{}: No such file or directory", c)
                        }
                        Err(e) => println!("\r{:#?}", e),
                    },
                }
                write!(std_out, "\r$ ")?;
                std_out.flush()?;
                input.clear();
            }
            Key::Char('\'') => {
                input.push('\'');
                write!(std_out, "\'")?;
                std_out.flush()?;
                // if !in_d_quote {
                // in_quote = !in_quote;
                // }
            }
            Key::Char('\"') => {
                input.push('\"');
                write!(std_out, "\"")?;
                std_out.flush()?;
                // in_d_quote = !in_d_quote;
            }
            Key::Backspace => {
                if !input.is_empty() {
                    if let Some(c) = input.pop() {
                        if c == '\'' {
                            in_quote = !in_quote;
                        } else if c == '\"' {
                            in_d_quote = !in_d_quote;
                        }
                    }
                    write!(std_out, "\r$ {} \r$ {}", input, input)?;
                    std_out.flush()?;
                }
            }
            Key::Char('\t') => {
                if display_possibilities {
                    writeln!(std_out)?;
                    for c in &completions {
                        writeln!(std_out, "\r{}", c)?;
                    }
                    write!(std_out, "\r$ {}", input)?;
                    std_out.flush()?;
                    display_possibilities = false;
                    completions.clear();
                    continue;
                }
                if input.is_empty() {
                    continue;
                }
                let mut a_completions = auto_complete(&input);
                match a_completions.len() {
                    1 => {
                        input = a_completions.first().unwrap().clone();
                        write!(std_out, "\r$ {} \r$ {}", input, input)?;
                        std_out.flush()?;
                    }
                    0 => {
                        write!(std_out, "{}", 7 as char)?;
                        std_out.flush()?;
                    }
                    _ => {
                        completions.append(&mut a_completions);
                        display_possibilities = true;
                    }
                }
            }
            Key::Char(c) => {
                write!(std_out, "{}", c)?;
                std_out.flush()?;
                input.push(*c)
            }
            _ => {}
        }
    }
    Ok(())
}

fn auto_complete(input: &str) -> Vec<String> {
    let mut completions = Vec::new();
    for s in ["echo ", "type ", "cd ", "exit "] {
        if s.starts_with(input) {
            completions.push(s.into());
        }
    }
    completions
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
        if nxt_bkslsh == input.len()
            && nxt_d_quote == input.len()
            && nxt_wht == input.len()
            && nxt_quote == input.len()
        {
            res.push_str(input);
            input = "";
            return (res, input);
        }
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

trait OutPut {
    fn print(&self, input: &str) -> Result<()> {
        self.write(input)
    }

    fn println(&self, input: &str) -> Result<()> {
        self.write(&format!("{}\n", input))
    }
    fn write(&self, input: &str) -> Result<()>;
}

#[derive(Default, Debug)]
struct StdOut {
    to: Vec<(PathBuf, Mode)>,
}
#[derive(Default, Debug)]
struct StdErr {
    to: Vec<(PathBuf, Mode)>,
}

impl OutPut for StdOut {
    fn write(&self, input: &str) -> Result<()> {
        if self.to.is_empty() {
            for line in input.lines() {
                println!("\r{}", line);
            }
        }
        for (f, mode) in &self.to {
            let mut file = match mode {
                Mode::Append => OpenOptions::new().append(true).create(true).open(f)?,
                Mode::Overwrite => File::create(f)?,
                Mode::Out => {
                    for line in input.lines() {
                        println!("\r{}", line);
                    }
                    continue;
                }
            };
            file.write_all(input.as_bytes())?;
        }
        Ok(())
    }
}

impl OutPut for StdErr {
    fn write(&self, input: &str) -> Result<()> {
        if self.to.is_empty() {
            print!("{}", input);
        }
        for (f, mode) in &self.to {
            let mut file = match mode {
                Mode::Append => OpenOptions::new().append(true).create(true).open(f)?,
                Mode::Overwrite => File::create(f)?,
                Mode::Out => {
                    print!("{}", input);
                    continue;
                }
            };
            file.write_all(input.as_bytes())?;
        }
        Ok(())
    }
}

#[derive(Default, Debug)]
enum Mode {
    Append,
    Overwrite,
    #[default]
    Out,
}

fn get_redirection(mut input: &str) -> (StdOut, StdErr, &str) {
    let mut stderr = StdErr::default();
    let mut stdout = StdOut::default();
    let mut in_d_quote = false;
    let mut in_quote = false;
    let mut input_trimmed = false;
    let mut i = 0;
    let mut input_ = input.chars().peekable();
    let old_input = input;
    while let Some(c) = input_.next() {
        i += 1;
        match c {
            '\"' => in_d_quote = !in_d_quote,
            '\'' => in_quote = !in_quote,
            '\\' => {
                if let Some(n) = input_.peek() {
                    if *n == '>' {
                        _ = input_.next();
                        i += 1;
                    }
                }
            }
            '>' => {
                if in_quote || in_d_quote {
                    continue;
                }

                if !input_trimmed {
                    input = &input[..i - 1];
                    input_trimmed = true;
                }
                if let Some(n) = input_.peek() {
                    if *n == '>' {
                        _ = input_.next();
                        i += 1;
                        stdout
                            .to
                            .push((get_file_name(old_input[i..].trim_start()), Mode::Append));
                    } else {
                        stdout
                            .to
                            .push((get_file_name(old_input[i..].trim_start()), Mode::Overwrite));
                    }
                }
            }
            '1' => {
                if in_quote || in_d_quote {
                    continue;
                }
                if let Some(n) = input_.peek() {
                    if *n == '>' {
                        if !input_trimmed {
                            input = &input[..i - 1];
                            input_trimmed = true;
                        }
                        _ = input_.next();
                        i += 1;
                        if let Some(n) = input_.peek() {
                            if *n == '>' {
                                _ = input_.next();
                                i += 1;
                                stdout.to.push((
                                    get_file_name(old_input[i..].trim_start()),
                                    Mode::Append,
                                ));
                            } else {
                                stdout.to.push((
                                    get_file_name(old_input[i..].trim_start()),
                                    Mode::Overwrite,
                                ));
                            }
                        }
                    }
                }
            }
            '2' => {
                if in_quote || in_d_quote {
                    continue;
                }
                if let Some(n) = input_.peek() {
                    if *n == '>' {
                        if !input_trimmed {
                            input = &input[..i - 1];
                            input_trimmed = true;
                        }
                        _ = input_.next();
                        i += 1;
                        if let Some(n) = input_.peek() {
                            if *n == '>' {
                                _ = input_.next();
                                i += 1;
                                stderr.to.push((
                                    get_file_name(old_input[i..].trim_start()),
                                    Mode::Append,
                                ));
                            } else {
                                stderr.to.push((
                                    get_file_name(old_input[i..].trim_start()),
                                    Mode::Overwrite,
                                ));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    (stdout, stderr, input)
}

fn get_file_name(input: &str) -> PathBuf {
    //TODO error handling
    let file_name = handle_input(input);
    let p = file_name.first().unwrap();
    let p: PathBuf = if p.starts_with("~") {
        let home = std::env::home_dir().unwrap_or(Path::new("/").into());
        let p = p.trim_start_matches("~").trim_start_matches("/");
        home.join(PathBuf::from(p))
    } else {
        PathBuf::from_str(p).unwrap_or_default()
    };
    p //.canonicalize().unwrap()
}

#[derive(Default)]
struct Args {
    args: Vec<String>,
    out: StdOut,
    err: StdErr,
}

impl Args {
    fn with_args(mut self, mut args: Vec<String>) -> Self {
        self.args.append(&mut args);
        self
    }
    fn with_stdout(mut self, out: StdOut) -> Self {
        self.out = out;
        self
    }
    fn with_stderr(mut self, err: StdErr) -> Self {
        self.err = err;
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

impl FromStr for Command {
    type Err = ShellError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (out, err, s) = get_redirection(s);

        let c = handle_input(s);
        let mut s = c.iter().map(|c| c.as_str());

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
                Args::default()
                    .with_args(s.map(|arg| arg.to_string()).collect())
                    .with_stdout(out)
                    .with_stderr(err),
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
                        Args::default()
                            .with_args(s.map(|arg| arg.to_string()).collect())
                            .with_stdout(out)
                            .with_stderr(err),
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
                let mut out_str = String::new();
                for arg in &args.args {
                    out_str.push_str(arg);
                    out_str.push(' ');
                }
                args.out.println(&out_str)?;
                args.err.print("")?;
            }
            Self::Type(c, p) => match p {
                None => println!("\r{c} is a shell builtin"),
                Some(p) => println!("\r{} is {}", c, p),
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

                args.out
                    .print(&format!("{}", String::from_utf8_lossy(&output.stdout)))?;

                args.err
                    .print(&format!("{}", String::from_utf8_lossy(&output.stderr)))?;
            }
        }
        Ok(())
    }
}
