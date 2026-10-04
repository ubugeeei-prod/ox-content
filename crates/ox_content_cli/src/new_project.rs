use crate::{Result, files::absolute, prompts, templates};
use clap::Parser;
use std::{fs, io::Write, path::Path, process::Command};

#[derive(Parser)]
#[command(name = "oxct new", disable_version_flag = true)]
struct Options {
    directory: Option<String>,
    #[arg(long)]
    template: Option<String>,
    #[arg(long)]
    skin: Option<String>,
    #[arg(long)]
    palette: Option<String>,
    #[arg(long = "package-manager")]
    manager: Option<String>,
    #[arg(long, overrides_with = "no_install")]
    install: bool,
    #[arg(long, overrides_with = "install")]
    no_install: bool,
    #[arg(long)]
    yes: bool,
}

pub fn run(args: &[String]) -> Result<i32> {
    let mut options = Options::try_parse_from(
        std::iter::once("oxct new".to_string()).chain(args.iter().cloned()),
    )?;
    // Reject supplied choices before prompting or writing anything.
    validate(&options)?;
    let interactive = prompts::interactive(options.yes);
    if interactive {
        prompts::print("◆ Ox Content · Create a project")?;
        if options.directory.is_none() {
            options.directory = Some(prompts::text("Project directory", "my-content")?);
        }
        if options.template.is_none() {
            options.template =
                Some(prompts::select("What are you building?", &["docs", "blog", "minimal"], 0)?);
        }
        if options.skin.is_none() {
            options.skin = Some(prompts::select("Choose a skin", templates::SKINS, 8)?);
        }
        if options.palette.is_none() {
            options.palette =
                Some(prompts::select("Choose a color palette", templates::PALETTES, 26)?);
        }
        if options.manager.is_none() {
            options.manager =
                Some(prompts::select("Package manager", &["vp", "pnpm", "npm", "yarn", "bun"], 0)?);
        }
        if !options.install && !options.no_install {
            options.install = prompts::confirm("Install dependencies now?")?;
        }
    }
    let directory = options.directory.as_deref().unwrap_or("my-content");
    let template = options.template.as_deref().unwrap_or("docs");
    let skin = options.skin.as_deref().unwrap_or("editorial");
    let palette = options.palette.as_deref().unwrap_or("nord");
    let manager = options.manager.as_deref().unwrap_or("vp");
    let target = absolute(Path::new(directory))?;
    create_project(&target, template, skin, palette, manager)?;
    if options.install {
        let executable = if cfg!(windows) && ["npm", "pnpm", "yarn"].contains(&manager) {
            format!("{manager}.cmd")
        } else {
            manager.to_string()
        };
        let status = Command::new(executable).arg("install").current_dir(&target).status();
        if !status.as_ref().is_ok_and(std::process::ExitStatus::success) {
            return Err(format!("Dependency installation failed. Project files are saved in {}; retry {manager} install there. {status:?}", target.display()).into());
        }
    }
    let runner = if manager == "vp" { "vp".to_string() } else { format!("{manager} run") };
    prompts::print(&format!(
        "◆ Created {}\ncd '{}'",
        target.display(),
        directory.replace('\'', "'\\''")
    ))?;
    if !options.install {
        prompts::print(&format!("{manager} install"))?;
    }
    prompts::print(&format!("{runner} dev"))?;
    Ok(0)
}

fn validate(options: &Options) -> Result<()> {
    for (key, value, allowed) in [
        ("template", &options.template, &["docs", "blog", "minimal"][..]),
        ("skin", &options.skin, templates::SKINS),
        ("palette", &options.palette, templates::PALETTES),
        ("manager", &options.manager, &["vp", "pnpm", "npm", "yarn", "bun"][..]),
    ] {
        if let Some(value) = value
            && !allowed.contains(&value.as_str())
        {
            return Err(format!("Invalid {key}: {value}. Choose {}.", allowed.join(", ")).into());
        }
    }
    Ok(())
}

fn create_project(
    target: &Path,
    template: &str,
    skin: &str,
    palette: &str,
    manager: &str,
) -> Result<()> {
    let existed = target.exists();
    if existed && fs::read_dir(target)?.next().is_some() {
        return Err(format!(
            "Directory is not empty: {}. Choose a new or empty directory.",
            target.display()
        )
        .into());
    }
    let name =
        target.file_name().ok_or("Choose a project directory with a name")?.to_string_lossy();
    let files = templates::project_files(&name, template, skin, palette, manager)?;
    fs::create_dir_all(target)?;
    let mut written = Vec::new();
    let mut directories = Vec::new();
    let result: Result<()> = (|| {
        for (name, content) in files {
            let path = target.join(name);
            if let Some(parent) = path.parent()
                && !parent.exists()
            {
                fs::create_dir_all(parent)?;
                directories.push(parent.to_path_buf());
            }
            let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&path)?;
            written.push(path);
            file.write_all(content.as_bytes())?;
        }
        Ok(())
    })();
    if result.is_err() {
        for file in written {
            let _ = fs::remove_file(file);
        }
        for dir in directories.into_iter().rev() {
            let _ = fs::remove_dir(dir);
        }
        if !existed {
            let _ = fs::remove_dir(target);
        }
    }
    result
}
