//! Command-line surface for the competitor runner.
//!
//! Three modes share one binary because they share its dependency graph: the
//! benchmark rows, the CommonMark conformance scores, and the `--normalize`
//! filter the JS conformance sweep pipes its own engines through.

/// What the binary was asked to do.
pub enum CliAction {
    Run { runs: u32 },
    Conformance { spec_path: String },
    Normalize,
    Help,
}

impl CliAction {
    fn flag_name(&self) -> &'static str {
        match self {
            CliAction::Run { .. } => "--runs",
            CliAction::Conformance { .. } => "--conformance",
            CliAction::Normalize => "--normalize",
            CliAction::Help => "--help",
        }
    }
}

/// Parses the whole argument list before picking an action, so a typo after a
/// mode flag (`--conformance spec.txt --unknown`) is reported instead of
/// silently running that mode.
pub fn parse_args(args: &[String]) -> Result<CliAction, String> {
    let mut runs = 1u32;
    let mut runs_requested = false;
    let mut action: Option<CliAction> = None;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--normalize" {
            select_action(&mut action, CliAction::Normalize)?;
        } else if arg == "--conformance" {
            index += 1;
            let value = args
                .get(index)
                .ok_or_else(|| "--conformance requires a spec.txt path".to_string())?;
            select_action(&mut action, CliAction::Conformance { spec_path: value.clone() })?;
        } else if let Some(value) = arg.strip_prefix("--conformance=") {
            select_action(&mut action, CliAction::Conformance { spec_path: value.to_string() })?;
        } else if arg == "--runs" {
            index += 1;
            let value =
                args.get(index).ok_or_else(|| "--runs requires a positive integer".to_string())?;
            runs = parse_positive_integer(value)?;
            runs_requested = true;
        } else if let Some(value) = arg.strip_prefix("--runs=") {
            runs = parse_positive_integer(value)?;
            runs_requested = true;
        } else if arg == "--help" || arg == "-h" {
            select_action(&mut action, CliAction::Help)?;
        } else {
            return Err(format!("Unknown argument: {arg}"));
        }
        index += 1;
    }
    match action {
        // `--runs` only means something for the benchmark rows: the other modes
        // read a fixture or stdin once, so accepting it would silently ignore a
        // requested repeat count. Checked after the whole list is parsed, so the
        // rejection is independent of flag order.
        Some(action @ (CliAction::Conformance { .. } | CliAction::Normalize)) if runs_requested => {
            Err(format!("--runs cannot be combined with {}", action.flag_name()))
        }
        Some(action) => Ok(action),
        None => Ok(CliAction::Run { runs }),
    }
}

/// Records a mode flag, rejecting a second one: the modes read different inputs
/// and write different output formats, so combining them has no meaning.
fn select_action(slot: &mut Option<CliAction>, action: CliAction) -> Result<(), String> {
    match slot {
        Some(existing) => {
            Err(format!("{} cannot be combined with {}", existing.flag_name(), action.flag_name()))
        }
        None => {
            *slot = Some(action);
            Ok(())
        }
    }
}

/// Positive-integer parsing with the JS harness' strictness: the canonical
/// re-rendering must equal the input, so `+5`, `05`, or `5x` are rejected.
fn parse_positive_integer(value: &str) -> Result<u32, String> {
    match value.parse::<u32>() {
        Ok(parsed) if parsed >= 1 && parsed.to_string() == value => Ok(parsed),
        _ => Err(format!("--runs requires a positive integer, got `{value}`")),
    }
}

pub fn print_usage() {
    println!(
        "Usage: ox-content-native-competitors [--runs <count>]

Options:
  --runs <count> Use the median result from repeated runs
  --conformance <spec.txt> Score each engine against a CommonMark spec fixture
  --normalize    Normalize length-prefixed HTML records on stdin (used by the JS sweep)
  -h, --help     Show this help message"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<CliAction, String> {
        parse_args(&args.iter().map(|arg| (*arg).to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn runs_is_rejected_with_conformance_in_either_order() {
        let expected = Some("--runs cannot be combined with --conformance".to_string());
        assert_eq!(parse(&["--conformance", "spec.txt", "--runs", "5"]).err(), expected);
        assert_eq!(parse(&["--runs", "5", "--conformance", "spec.txt"]).err(), expected);
        assert_eq!(parse(&["--conformance=spec.txt", "--runs=5"]).err(), expected);
        assert_eq!(parse(&["--runs=5", "--conformance=spec.txt"]).err(), expected);
    }

    #[test]
    fn runs_is_rejected_with_normalize_in_either_order() {
        let expected = Some("--runs cannot be combined with --normalize".to_string());
        assert_eq!(parse(&["--normalize", "--runs", "5"]).err(), expected);
        assert_eq!(parse(&["--runs", "5", "--normalize"]).err(), expected);
        assert_eq!(parse(&["--runs=5", "--normalize"]).err(), expected);
    }

    #[test]
    fn mode_flags_still_parse_without_runs() {
        assert!(matches!(parse(&["--normalize"]), Ok(CliAction::Normalize)));
        assert!(matches!(
            parse(&["--conformance", "spec.txt"]),
            Ok(CliAction::Conformance { spec_path }) if spec_path == "spec.txt"
        ));
        assert!(matches!(parse(&["--runs", "5"]), Ok(CliAction::Run { runs: 5 })));
        assert!(matches!(parse(&[]), Ok(CliAction::Run { runs: 1 })));
    }
}
