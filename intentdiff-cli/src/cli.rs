use clap::{Parser, ValueEnum};
use std::path::PathBuf;

use intentdiff_core::SignalStrength;

#[derive(Parser, Debug)]
#[command(
    name = "intentdiff",
    version,
    about = "Compare infrastructure by behavior, not syntax."
)]
pub struct Cli {
    pub left: PathBuf,
    pub right: PathBuf,

    #[arg(long, value_enum)]
    fail_on: Option<SeverityArg>,

    #[arg(long, value_enum, default_value = "terminal")]
    format: OutputFormat,
}

impl Cli {
    pub fn left(&self) -> &PathBuf {
        &self.left
    }

    pub fn right(&self) -> &PathBuf {
        &self.right
    }

    pub fn fail_on(&self) -> Option<SignalStrength> {
        self.fail_on.clone().map(Into::into)
    }

    pub fn format(&self) -> Format {
        self.format.clone().into()
    }
}

pub enum Format {
    Terminal,
    Markdown,
}

impl From<OutputFormat> for Format {
    fn from(value: OutputFormat) -> Self {
        match value {
            OutputFormat::Terminal => Format::Terminal,
            OutputFormat::Markdown => Format::Markdown,
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Terminal,
    Markdown,
}

#[derive(Debug, Clone, ValueEnum)]
enum SeverityArg {
    Informational,
    Warning,
    Critical,
}

impl From<SeverityArg> for SignalStrength {
    fn from(value: SeverityArg) -> Self {
        match value {
            SeverityArg::Informational => SignalStrength::Informational,
            SeverityArg::Warning => SignalStrength::Warning,
            SeverityArg::Critical => SignalStrength::Critical,
        }
    }
}
