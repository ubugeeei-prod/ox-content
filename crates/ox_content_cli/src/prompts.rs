use crate::Result;
use dialoguer::{Confirm, Input, MultiSelect, Select, theme::ColorfulTheme};
use std::io::{IsTerminal, Write};

pub fn interactive(yes: bool) -> bool {
    !yes && std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

pub fn text(label: &str, default: &str) -> Result<String> {
    Ok(Input::with_theme(&ColorfulTheme::default())
        .with_prompt(label)
        .default(default.to_string())
        .interact_text()?)
}

pub fn select(label: &str, items: &[&str], default: usize) -> Result<String> {
    let choice = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(label)
        .items(items)
        .default(default)
        .interact_opt()?
        .ok_or("Setup cancelled")?;
    Ok(items[choice].to_string())
}

pub fn multi(label: &str, items: &[&str], defaults: &[bool]) -> Result<Vec<String>> {
    let choices = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt(label)
        .items(items)
        .defaults(defaults)
        .interact_opt()?
        .ok_or("Setup cancelled")?;
    if choices.is_empty() {
        return Err("Select at least one option".into());
    }
    Ok(choices.into_iter().map(|index| items[index].to_string()).collect())
}

pub fn confirm(label: &str) -> Result<bool> {
    Ok(Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(label)
        .default(true)
        .interact_opt()?
        .ok_or("Setup cancelled")?)
}

pub fn print(value: &str) -> Result<()> {
    writeln!(std::io::stdout(), "{value}")?;
    Ok(())
}
