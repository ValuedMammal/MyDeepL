use std::env;

use anyhow::Context;
use clap::Parser;
use deeprl::DeepL;

use crate::cli::{Args, Cmd};

mod cli;
mod cmd;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let key = env::var("DEEPL_API_KEY").context("must set DEEPL_API_KEY")?;
    let dl = DeepL::new(&key);

    match args.cmd {
        // Get usage
        Cmd::Usage => {
            let usage = dl.usage()?;
            let count = usage.character_count;
            let limit = usage.character_limit;

            println!("Used: {count}/{limit}");
        }
        Cmd::Text(params) => cmd::text::execute(&dl, params)?,
        Cmd::Document(params) => cmd::document::execute(&dl, params)?,
        Cmd::Languages => cmd::languages::execute(&dl)?,
        Cmd::Glossary(glos) => cmd::glossary::execute(&dl, glos)?,
    }

    Ok(())
}
