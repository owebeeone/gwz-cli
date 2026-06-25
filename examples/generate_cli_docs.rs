use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let mode = Mode::from_args(env::args().skip(1).collect());
    let docs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/CLI.md");
    let generated = gwz::cli_reference_markdown();

    match mode {
        Mode::Stdout => {
            print!("{generated}");
        }
        Mode::Write => {
            fs::write(&docs_path, generated).unwrap_or_else(|error| {
                panic!("failed to write {}: {error}", docs_path.display());
            });
            eprintln!("wrote {}", docs_path.display());
        }
        Mode::Check => {
            let current = fs::read_to_string(&docs_path).unwrap_or_else(|error| {
                panic!("failed to read {}: {error}", docs_path.display());
            });
            if current != generated {
                eprintln!(
                    "{} is out of date; regenerate with: python scripts/generate_cli_reference.py --write",
                    docs_path.display()
                );
                std::process::exit(1);
            }
        }
    }
}

enum Mode {
    Stdout,
    Write,
    Check,
}

impl Mode {
    fn from_args(args: Vec<String>) -> Self {
        match args.as_slice() {
            [] => Self::Stdout,
            [arg] if arg == "--write" => Self::Write,
            [arg] if arg == "--check" => Self::Check,
            _ => {
                eprintln!("usage: generate_cli_docs [--write|--check]");
                std::process::exit(2);
            }
        }
    }
}
