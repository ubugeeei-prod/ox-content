use std::{io::Write, path::Path, process::ExitCode};

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let mut assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../editors/neovim");
    if args.first().is_some_and(|command| command == "ide")
        && let Ok(executable) = std::env::current_exe()
        && let Some(directory) = executable.parent()
    {
        let bundled = directory.join("neovim");
        if bundled.is_dir() {
            assets = bundled;
        }
    }
    match ox_content_cli::run(&args, &assets) {
        Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
        Err(error) => {
            let _ = writeln!(std::io::stderr(), "{error}");
            ExitCode::FAILURE
        }
    }
}
