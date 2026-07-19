use super::commands::convert_command;
use super::{transform_pm, PmOptions};

fn all(command: &str) -> (String, String, String, String) {
    (
        convert_command(command, "npm"),
        convert_command(command, "pnpm"),
        convert_command(command, "yarn"),
        convert_command(command, "bun"),
    )
}

fn vp(command: &str) -> String {
    convert_command(command, "vp")
}

mod commands;
mod transform;
