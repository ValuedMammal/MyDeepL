use deeprl::{
    DeepL,
    DocumentOptions,
    glos::GlossaryEntriesFormat,
    Language, 
    lang::LanguageType, 
    TextOptions,
    text::{Formality, SplitSentences, TagHandling},
};
use std::{
    env,
    fs,
    io::{self, Read}, 
    path::PathBuf, 
    str::FromStr,
    thread, 
    time::Duration,
};
use anyhow::{Context, bail};
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
        // Usage
        Cmd::Usage => {
            let usage = dl.usage()?;
            let count = usage.character_count;
            let limit = usage.character_limit;
    
            println!("Used: {count}/{limit}");
        },
        // Text
        Cmd::Text(params) => {
            // Check we have a valid target lang
            let Ok(target_lang) = params.target.parse::<Language>() else {
                bail!("invalid target lang")
            };

            // Set optional
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
        // Document
        Cmd::Document(params) => {
            let Ok(target_lang) = params.target.parse::<Language>() else {
                bail!("invalid target lang")
            };
            let file_path = params.file.into();

            let mut opt = DocumentOptions::new(target_lang, file_path);

            // Set optional fields
            // filename, src, formal, glos
            if let Some(name) = params.filename {
                opt = opt.filename(name);
            }
            if let Some(src) = params.source {
                let Ok(source_lang) = src.parse::<Language>() else {
                    bail!("invalid source lang")
                };
                opt = opt.source_lang(source_lang);
            }
            if let Some(f) = params.formality {
                opt = opt.formality(Formality::from_str(&f).unwrap());
            }
            if let Some(g) = params.glossary {
                opt = opt.glossary_id(g);
            }
            
            // Upload
            println!("Uploading file...");
            let doc = dl.document_upload(opt)?;

            // Poll status
            let mut is_done = false;
            let mut server_err: Option<String> = None;
            let mut secs = Duration::from_secs(2);
            for _ in 0..3 {
                thread::sleep(secs);

                let status = dl.document_status(&doc)?;
                let state = status.status;
                println!("Status: {state:?}");

                if state.is_done() {
                    is_done = true;
                    break
                }
                if let Some(msg) = status.error_message {
                    server_err = Some(msg);
                    break
                }
                // basic backoff
                secs *= 2;
            }

            if !is_done {
                bail!("Poll status timeout");
                //TODO: prompt user to try download again
            }
            if server_err.is_some() {
                bail!("{}", server_err.unwrap());
            }

            // Download
            let mut out_file: Option<PathBuf> = None;
            if let Some(out) = params.out_file {
                out_file = Some(PathBuf::from(out));
            }
            println!("Retrieving results...");
            let result = dl.document_download(doc, out_file)?;
            print!("New translation file: {result:?}");
        },
        // Languages
        Cmd::Languages => {
            println!("Fetching source languages...");
            let languages = dl.languages(LanguageType::Source)?;
            for lang in languages {
                let code = lang.language;
                let name = lang.name;
                println!("{code} {name}");
            }            
            
            println!("Fetching target languages...");
            let languages = dl.languages(LanguageType::Target)?;
            for lang in languages {
                let code = lang.language;
                let name = lang.name;
                println!("{code} {name}");
            }
        },
        // Glossary
        Cmd::Glossary(sub) => {
            match sub.cmd {
                Glos::List => {
                    let glossaries = dl.glossaries()?
                        .glossaries;

                    if glossaries.is_empty() {
                        println!("None");
                    } else {
                        for glos in glossaries {
                            //TODO
                            // let json = serde_json::to_string_pretty(&glos)?;
                            println!("{glos:?}");
                        }
                    }
                },
                Glos::Get(glos) => {
                    let glossary = dl.glossary_info(&glos.id)?;
                    //println!("{}", serde_json::to_string_pretty(&glos)?);
                    println!("{glossary:?}");
                },
                Glos::Entries(glos) => {
                    let entries = dl.glossary_entries(&glos.id)?;
                    print!("{entries}");
                },
                Glos::Create(params) => {
                    let name = params.name;
                    let Ok(src) = params.source.parse::<Language>() else {
                        bail!("invalid source lang");
                    };
                    let Ok(trg) = params.target.parse::<Language>() else {
                        bail!("invalid source lang");
                    };

                    let file_path = PathBuf::from(params.file);
                    let entries = fs::read_to_string(file_path)?;
                    //TODO: read from stdin
                    // or as cli option
                    let fmt = if params.csv { GlossaryEntriesFormat::Csv } else { GlossaryEntriesFormat::Tsv };

                    let glos = dl.glossary_new(name, src, trg, entries, fmt)?;
                    println!("{}", glos.glossary_id);
                },
                // Delete a glossary
                Glos::Delete(glos) => {
                    let _ = dl.glossary_del(&glos.id);
                    println!("Done");
                },
            }
        },
    };
    
    Ok(())
}
