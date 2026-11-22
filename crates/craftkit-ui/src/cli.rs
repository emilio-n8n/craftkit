//! Command-line parsing shared by CraftKit applications.

use std::path::PathBuf;

/// Options common to every CraftKit application.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CliOptions {
    /// Open the built-in sample document.
    pub sample: bool,
    /// Open a project file on startup.
    pub open: Option<PathBuf>,
}

/// Result of parsing the command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliAction {
    /// Start the application with these options.
    Run(CliOptions),
    /// Print help and exit.
    Help,
    /// Print the version and exit.
    Version,
}

/// Invalid command line.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CliError {
    /// `--open` had no following path.
    #[error("--open requires a project path")]
    MissingOpenPath,
    /// An argument was not recognized.
    #[error("unknown argument: {0}")]
    UnknownArgument(String),
}

/// Parse arguments, excluding the executable name.
///
/// `--help` and `--version` win over every other argument.
///
/// # Errors
///
/// Returns an error for a missing `--open` value or an unknown argument.
pub fn parse_args<I>(args: I) -> Result<CliAction, CliError>
where
    I: IntoIterator<Item = String>,
{
    let args: Vec<String> = args.into_iter().collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        return Ok(CliAction::Help);
    }
    if args.iter().any(|arg| arg == "--version" || arg == "-V") {
        return Ok(CliAction::Version);
    }

    let mut options = CliOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--sample" => options.sample = true,
            "--open" => {
                index += 1;
                let path = args.get(index).ok_or(CliError::MissingOpenPath)?;
                options.open = Some(PathBuf::from(path));
            }
            value if !value.starts_with('-') && value.ends_with(".json") => {
                options.open = Some(PathBuf::from(value));
            }
            value => return Err(CliError::UnknownArgument(value.to_owned())),
        }
        index += 1;
    }
    Ok(CliAction::Run(options))
}

/// Standard help text.
#[must_use]
pub fn help_text(binary: &str, description: &str) -> String {
    format!(
        "{description}\n\n\
         Usage: {binary} [--sample] [--open <project.json>]\n\n\
         Options:\n  \
         --sample          Open the built-in sample document\n  \
         --open <file>     Open a project file at startup\n  \
         --version         Print the version and exit\n  \
         --help            Show this help\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn parses_sample_and_open() {
        let action = parse_args(args(&["--sample", "--open", "demo.json"])).expect("parse");
        assert_eq!(
            action,
            CliAction::Run(CliOptions {
                sample: true,
                open: Some(PathBuf::from("demo.json")),
            })
        );
    }

    #[test]
    fn accepts_a_positional_project() {
        let action = parse_args(args(&["project.json"])).expect("parse");
        assert_eq!(
            action,
            CliAction::Run(CliOptions {
                sample: false,
                open: Some(PathBuf::from("project.json")),
            })
        );
    }

    #[test]
    fn help_wins() {
        assert_eq!(
            parse_args(args(&["--sample", "--help"])).expect("parse"),
            CliAction::Help
        );
    }

    #[test]
    fn rejects_missing_path_and_unknown_flag() {
        assert!(matches!(
            parse_args(args(&["--open"])),
            Err(CliError::MissingOpenPath)
        ));
        assert!(matches!(
            parse_args(args(&["--wat"])),
            Err(CliError::UnknownArgument(_))
        ));
    }
}
