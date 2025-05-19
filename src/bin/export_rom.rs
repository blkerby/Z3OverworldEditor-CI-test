use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    /// Name of the person to greet
    #[arg(long)]
    theme: String,
}

pub fn main() -> Result<()> {
    // let args = Args::parse();
    // let theme = args.theme;
    // let state = Z3OverworldEditor::state::get_initial_state()?;
    // let global_config = crate::state::GlobalConfig::load()?;

    Ok(())
}
