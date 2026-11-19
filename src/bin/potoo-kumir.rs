use std::io::{self, Write};

use potoo_kumir_interpreter::reference::{Host, Interpreter, Profile, parse};

struct CliHost;

impl Host for CliHost {
    fn input(&mut self) -> Result<String, String> {
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .map_err(|error| error.to_string())?;
        Ok(line.trim_end_matches(['\r', '\n']).to_owned())
    }

    fn output(&mut self, text: &str) -> Result<(), String> {
        print!("{text}");
        io::stdout().flush().map_err(|error| error.to_string())
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut profile = Profile::Classic;
    let mut path = None;
    while let Some(argument) = args.next() {
        if argument == "--profile" {
            let value = args
                .next()
                .ok_or_else(|| "`--profile` requires classic or high-grade".to_owned())?;
            profile = Profile::parse(&value)?;
        } else if let Some(value) = argument.strip_prefix("--profile=") {
            profile = Profile::parse(value)?;
        } else if argument == "--help" || argument == "-h" {
            println!("usage: potoo-kumir [--profile classic|high-grade] <source-file>");
            return Ok(());
        } else if path.replace(argument).is_some() {
            return Err("only one source file can be provided".to_owned());
        }
    }
    let path = path.ok_or_else(|| {
        "usage: potoo-kumir [--profile classic|high-grade] <source-file>".to_owned()
    })?;
    let source = std::fs::read_to_string(&path).map_err(|error| format!("{path}: {error}"))?;
    let program = parse(&source, profile).map_err(|diagnostics| {
        diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    Interpreter::new(CliHost)
        .run(&program)
        .map_err(|error| error.to_string())?;
    Ok(())
}
