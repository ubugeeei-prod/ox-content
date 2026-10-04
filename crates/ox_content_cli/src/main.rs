use std::{io::Write, path::Path, process::ExitCode};

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../editors/neovim");
    match ox_content_cli::run(&args, &assets) {
        Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
        Err(error) => {
            let _ = writeln!(std::io::stderr(), "{error}");
            ExitCode::FAILURE
        }
    }
}
