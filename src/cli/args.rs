use crate::{CaptureMode, OutputFormat};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "wst")]
#[command(author = "Wayland Screenshot Tool")]
#[command(version = "1.0")]
#[command(about = "Modern screenshot tool for Wayland", long_about = None)]
pub struct Args {
    #[arg(value_name = "MODE")]
    pub mode: Option<String>,

    #[arg(value_name = "OUTPUT")]
    pub output: Option<PathBuf>,

    #[arg(short, long, value_name = "FORMAT")]
    pub format: Option<String>,

    #[arg(short, long, value_name = "SECONDS")]
    pub delay: Option<u64>,

    #[arg(short = 'c', long)]
    pub clipboard: bool,

    #[arg(long)]
    pub cursor: bool,

    #[arg(short, long)]
    pub quiet: bool,

    #[arg(short, long)]
    pub json: bool,

    #[arg(short = 'x', long)]
    pub headless: bool,

    #[arg(short, long, value_name = "REGION")]
    pub region: Option<String>,

    #[arg(short, long, value_name = "INDEX")]
    pub monitor: Option<usize>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[cfg(feature = "gui")]
    Gui,

    #[cfg(feature = "gui")]
    Edit {
        file: PathBuf,
    },

    List {
        #[arg(default_value = "displays")]
        what: String,
    },

    Completions {
        shell: String,
    },

    Config {
        #[arg(long)]
        show: bool,

        #[arg(long)]
        reset: bool,

        #[arg(long)]
        edit: bool,
    },

    Process {
        input: PathBuf,

        output: PathBuf,

        #[arg(long)]
        border: Option<u32>,

        #[arg(long)]
        shadow: Option<u32>,

        #[arg(long)]
        resize: Option<String>,

        #[arg(long)]
        blur: Option<f32>,
    },
}

impl Args {
    pub fn parse_mode(&self) -> Option<CaptureMode> {
        self.mode.as_ref().and_then(|m| m.parse().ok())
    }

    pub fn parse_format(&self) -> Option<OutputFormat> {
        self.format.as_ref().and_then(|f| f.parse().ok())
    }

    pub fn parse_region(&self) -> Option<crate::Region> {
        self.region.as_ref().and_then(|r| {
            let parts: Vec<&str> = r.split(',').collect();
            if parts.len() == 4 {
                let x = parts[0].parse().ok()?;
                let y = parts[1].parse().ok()?;
                let width = parts[2].parse().ok()?;
                let height = parts[3].parse().ok()?;
                Some(crate::Region::new(x, y, width, height))
            } else {
                None
            }
        })
    }
}
