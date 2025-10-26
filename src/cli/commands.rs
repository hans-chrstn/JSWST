use crate::cli::Args;
use crate::{
    CaptureMode, CaptureOptions, OutputFormat, Result, ScreenshotError, capture, config::Config,
    export::Exporter, processing::ImageProcessor,
};
use tracing::{error, info};

pub async fn execute(args: Args) -> Result<()> {
    let config = Config::load().unwrap_or_default();

    if let Some(command) = args.command {
        return execute_subcommand(command, &config).await;
    }

    execute_capture(args, config).await
}

async fn execute_capture(args: Args, config: Config) -> Result<()> {
    let mode = args.parse_mode().unwrap_or(config.default_mode);

    let format = args.parse_format().unwrap_or(config.default_format);

    let options = CaptureOptions {
        delay: args
            .delay
            .or(Some(config.delay_seconds))
            .map(std::time::Duration::from_secs),
        include_cursor: args.cursor || config.include_cursor,
        monitor_index: args.monitor,
        region: args.parse_region(),
    };

    let backend = capture::create_backend()?;

    if !args.quiet {
        info!("Capturing screenshot in {:?} mode...", mode);
    }

    let screenshot = backend.capture(mode, &options).await?;

    if !args.quiet {
        info!(
            "Captured {}x{} screenshot",
            screenshot.width(),
            screenshot.height()
        );
    }

    let output_path = args.output.unwrap_or_else(|| {
        let filename = match format {
            OutputFormat::Png => format!("{}.png", config.generate_filename()),
            OutputFormat::Jpeg => format!("{}.jpg", config.generate_filename()),
            OutputFormat::Webp => format!("{}.webp", config.generate_filename()),
            OutputFormat::Clipboard => "clipboard".to_string(),
        };
        config.save_directory.join(filename)
    });

    if format != OutputFormat::Clipboard {
        let _file_size = Exporter::save(&screenshot, &output_path, format)?;

        if !args.quiet {
            println!("{}", output_path.display());

            if args.json {
                let json = serde_json::to_string_pretty(&screenshot.metadata)?;
                println!("{}", json);
            }
        }

        if args.clipboard || config.auto_copy_to_clipboard {
            Exporter::copy_to_clipboard(&screenshot)?;
            if !args.quiet {
                info!("Copied to clipboard");
            }
        }
    } else {
        Exporter::copy_to_clipboard(&screenshot)?;
        if !args.quiet {
            info!("Copied to clipboard");
        }
    }

    Ok(())
}

async fn execute_subcommand(command: crate::cli::args::Commands, config: &Config) -> Result<()> {
    use crate::cli::args::Commands;

    match command {
        #[cfg(feature = "gui")]
        Commands::Gui => {
            info!("GUI mode requires implementing the UI module");
            Err(ScreenshotError::Config(
                "GUI mode not yet implemented".to_string(),
            ))
        }

        #[cfg(feature = "gui")]
        Commands::Edit { file } => {
            info!(
                "Editor mode requires implementing the UI module: {}",
                file.display()
            );
            Err(ScreenshotError::Config(
                "Editor mode not yet implemented".to_string(),
            ))
        }

        Commands::List { what } => {
            let backend = capture::create_backend()?;
            match what.as_str() {
                "displays" | "monitors" => {
                    let displays = backend.get_displays().await?;
                    for (i, display) in displays.iter().enumerate() {
                        println!(
                            "[{}] {} - {}x{} @ ({}, {}) scale: {}",
                            i,
                            display.name,
                            display.width,
                            display.height,
                            display.x,
                            display.y,
                            display.scale
                        );
                    }
                }
                "windows" => {
                    if let Some(window) = backend.get_activate_window().await? {
                        println!(
                            "Active: {} ({}) - {}x{} @ ({}, {})",
                            window.title,
                            window.app_id,
                            window.width,
                            window.height,
                            window.x,
                            window.y
                        );
                    } else {
                        println!("No active window found");
                    }
                }
                _ => {
                    error!("Unknown list type: {}", what);
                }
            }
            Ok(())
        }

        Commands::Completions { shell } => {
            generate_completions(&shell);
            Ok(())
        }

        Commands::Config { show, reset, edit } => {
            if reset {
                let default_config = Config::default();
                default_config.save()?;
                info!("Configuration reset to defaults");
            } else if edit {
                let config_dir =
                    directories::ProjectDirs::from("com", "wayland", "screenshot-tool")
                        .ok_or_else(|| {
                            ScreenshotError::Config("Cannot find config dir".to_string())
                        })?;
                let config_path = config_dir.config_dir().join("config.toml");

                let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
                std::process::Command::new(editor)
                    .arg(&config_path)
                    .status()?;
            } else {
                let config_str = toml::to_string_pretty(config)
                    .map_err(|e| ScreenshotError::Config(e.to_string()))?;
                println!("{}", config_str);
            }
            Ok(())
        }

        Commands::Process {
            input,
            output,
            border,
            shadow,
            resize,
            blur,
        } => {
            info!("Processing image: {}", input.display());

            let img = image::open(&input)?;
            let mut screenshot =
                crate::Screenshot::new(img.to_rgba8(), CaptureMode::Screen, OutputFormat::Png);

            if let Some(width) = border {
                screenshot =
                    ImageProcessor::add_border(&screenshot, width, image::Rgba([0, 0, 0, 255]))?;
            }

            if let Some(offset) = shadow {
                screenshot = ImageProcessor::add_shadow(&screenshot, offset)?;
            }

            if let Some(size_str) = resize {
                let parts: Vec<&str> = size_str.split(',').collect();
                if parts.len() == 2 {
                    let width = parts[0]
                        .parse()
                        .map_err(|_| ScreenshotError::Config("Invalid width".to_string()))?;
                    let height = parts[1]
                        .parse()
                        .map_err(|_| ScreenshotError::Config("Invalid height".to_string()))?;
                    screenshot = ImageProcessor::resize(&screenshot, width, height)?;
                }
            }

            if let Some(sigma) = blur {
                screenshot = ImageProcessor::blur(&screenshot, sigma)?;
            }

            let format = if output.extension().and_then(|e| e.to_str()) == Some("jpg") {
                OutputFormat::Jpeg
            } else if output.extension().and_then(|e| e.to_str()) == Some("webp") {
                OutputFormat::Webp
            } else {
                OutputFormat::Png
            };

            Exporter::save(&screenshot, &output, format)?;
            info!("Saved to: {}", output.display());

            Ok(())
        }
    }
}

fn generate_completions(shell: &str) {
    use clap::CommandFactory;
    use clap_complete::{Shell, generate};

    let shell = match shell {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "powershell" => Shell::PowerShell,
        _ => {
            eprintln!("Unknown shell: {}", shell);
            return;
        }
    };

    let mut cmd = Args::command();
    generate(shell, &mut cmd, "wst", &mut std::io::stdout());
}
