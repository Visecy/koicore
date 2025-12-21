use anyhow::{Context, Result};
use clap::{Parser as ClapParser, Subcommand};
use koicore::Command;
use koicore::parser::{BufReadWrapper, FileInputSource, Parser, ParserConfig};
use koicore::writer::{Writer, WriterConfig};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;

#[derive(ClapParser)]
#[command(author, version, about = "CLI tool for KoiLang parsing and conversion", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert KoiLang to JSON
    ToJson {
        /// Input KoiLang file (defaults to stdin)
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Output JSON file (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Pretty print JSON
        #[arg(short, long)]
        pretty: bool,
    },
    /// Convert JSON to KoiLang
    FromJson {
        /// Input JSON file (defaults to stdin)
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Output KoiLang file (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::ToJson {
            input,
            output,
            pretty,
        } => {
            let config = ParserConfig::default();
            let mut commands = Vec::new();

            if let Some(path) = input {
                let source = FileInputSource::new(&path)
                    .with_context(|| format!("Failed to open input file: {:?}", path))?;
                let mut parser = Parser::new(source, config);
                while let Some(command) = parser
                    .next_command()
                    .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?
                {
                    commands.push(command);
                }
            } else {
                let stdin = std::io::stdin();
                let source = BufReadWrapper(stdin.lock());
                let mut parser = Parser::new(source, config);
                while let Some(command) = parser
                    .next_command()
                    .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?
                {
                    commands.push(command);
                }
            }

            let json = if pretty {
                serde_json::to_string_pretty(&commands)?
            } else {
                serde_json::to_string(&commands)?
            };

            if let Some(path) = output {
                File::create(&path)
                    .with_context(|| format!("Failed to create output file: {:?}", path))?
                    .write_all(json.as_bytes())?;
            } else {
                std::io::stdout().write_all(json.as_bytes())?;
                println!(); // Add newline if stdout
            }
        }
        Commands::FromJson { input, output } => {
            let commands: Vec<Command> = if let Some(path) = input {
                let file = File::open(&path)
                    .with_context(|| format!("Failed to open input file: {:?}", path))?;
                serde_json::from_reader(BufReader::new(file))
                    .with_context(|| "Failed to parse JSON")?
            } else {
                serde_json::from_reader(std::io::stdin().lock())
                    .with_context(|| "Failed to parse JSON")?
            };

            let config = WriterConfig::default();
            let mut buffer = Vec::new();
            let mut writer = Writer::new(&mut buffer, config);

            for cmd in commands {
                writer
                    .write_command(&cmd)
                    .context("Failed to write command")?;
            }

            if let Some(path) = output {
                File::create(&path)
                    .with_context(|| format!("Failed to create output file: {:?}", path))?
                    .write_all(&buffer)?;
            } else {
                std::io::stdout().write_all(&buffer)?;
            }
        }
    }

    Ok(())
}
