use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use yexp::handle_yaml;

#[derive(Parser, Debug)]
#[command(name = "yexp")]
#[command(bin_name = "yexp")]
#[command(version, about)]
struct YexpCli {
    #[arg(name = "path")]
    path: PathBuf,

    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let cli = YexpCli::parse();
    let value = handle_yaml(cli.path)?;

    if let Some(path) = cli.output {
        let mut file = std::fs::File::create(path).context("Failed to create file")?;
        serde_yaml::to_writer(&mut file, &value)?;
    } else {
        print!("{}", serde_yaml::to_string(&value)?);
    }

    Ok(())
}
