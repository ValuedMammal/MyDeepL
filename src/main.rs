#![allow(unused)]
use deeprl::{
    DeepL,
    Language, 
    TextOptions, 
    text::{SplitSentences, TagHandling},
    text::Formality,
};
use std::{
    process, 
    env,
    str::FromStr,
    io::{self, Read},
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

            if !params.multi_lang {
                if let Some(src) = params.source {
                    let Ok(source_lang) = src.parse::<Language>() else {
                        bail!("invalid source lang")
                    };
                    opt = opt.source_lang(source_lang);
                }
                // skip source lang if input contains various langs
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
                opt = opt.formality(Formality::from_str(&f).unwrap());
                // Ok to unwrap, as `from_str` defaults to Default
            }

            if let Some(g) = params.glossary {
                opt = opt.glossary_id(g);
            }

            if let Some(t) = params.tag_handling {
                let tag = match t.to_lowercase().as_str() {
                    "xml" => TagHandling::Xml,
                    "html" => TagHandling::Html,
                    _ => {
                        bail!("invalid tag handling");
                    }
                };
                opt = opt.tag_handling(tag);
            }

            if params.no_outline_detection {
                opt = opt.outline_detection(false);
            }

            if let Some(split) = params.splitting_tags {
                opt = opt.splitting_tags(split);
            }

            if let Some(non) = params.non_splitting_tags {
                opt = opt.non_splitting_tags(non);
            }
            
            if let Some(ig) = params.ignore_tags {
                opt = opt.ignore_tags(ig);
            }

            // Get input text and call translate api
            let mut text: Vec<String> = vec![];

            if let Some(t) = params.text {
                text.push(t);
                //TODO: if t[0] == '-', read stdin
            } else {
                // read stdin
                let mut s = String::new();
                io::stdin().read_to_string(&mut s)
                    .context("failed to read stdin")?;

                // experimental multi-lang input
                if params.multi_lang {
                    text = s.split('\n').map(|s| s.to_owned()).collect();
                } else {
                    // send a single text param
                    text.push(s);
                }
            }

            let result = dl.translate(opt, text)?;
            for t in result.translations {
                if params.show_detected {
                    println!("Detected: {}", t.detected_source_language);
                }
                println!("{}", t.text);
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
