#![allow(unused)]
use deeprl::{
    DeepL,
    Language, TextOptions, text::SplitSentences,
    text::Formality,
};
use std::{
    process, 
    env,
};
use anyhow::{Result, Context, bail};
use crate::args::*;
pub mod args;



fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Create new instance of DeepL
    let Ok(key) = env::var("DEEPL_API_KEY") else {
        bail!("Please make sure DEEPL_API_KEY is set")
    };
    let dl = DeepL::new(key);
    
    // Execute command
    match args.cmd {
        // Get usage
        Cmd::Usage => {
            let usage = dl.usage()?;
            let count = usage.character_count;
            let limit = usage.character_limit;
    
            println!("Used: {count}/{limit}");
        },
        // Translate text
        Cmd::Text(params) => {
            // Check we have a valid target lang
            let trg = params.target;
            let Ok(target_lang) = trg.parse::<Language>() else {
                bail!("invalid target lang")
            };

            // Build options
            // src, split_sent, preserve, formal, glos, tags, outline, split_tag, nonsplit_tag, ignore_tag
            let mut opt = TextOptions::new(target_lang);

            if let Some(src) = params.source {
                let Ok(source_lang) = src.parse::<Language>() else {
                    bail!("invalid source lang")
                };
                opt = opt.source_lang(source_lang);
            }

            if let Some(ss) = params.split_sentences {
                let split = match ss.as_str() {
                    "0" => SplitSentences::None,
                    "nonewlines" => SplitSentences::NoNewlines,
                    _ => SplitSentences::Default
                };
                opt = opt.split_sentences(split);
            }

            if params.preserve_formatting {
                opt = opt.preserve_formatting(true);
            }

            if let Some(f) = params.formality {
                // opt = opt.formality(Formality::from_str(&f));
            }
        },
    };
    /*
        Cmd::Document(d) => {},
        Cmd::Languages => {},
        Cmd::Glossary(g) => {},
    
    */
    Ok(())
}
