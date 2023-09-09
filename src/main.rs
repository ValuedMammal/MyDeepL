use deeprl::{
    DeepL,
    DocumentOptions,
    glos::GlossaryEntriesFormat,
    Language, 
    lang::LanguageType, 
    TextOptions,
    text::{Formality, SplitSentences, TagHandling}, 
    Document,
};
use serde_json::Value;
use std::{
    env,
    fs,
    io::{self, Read}, 
    path::PathBuf, 
    str::FromStr,
    thread, 
    time::Duration,
};
use anyhow::bail;
use crate::args::*;
pub mod args;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Create new instance of DeepL
    let Ok(key) = env::var("DEEPL_API_KEY") else {
        bail!("Please make sure DEEPL_API_KEY is set")
    };
    let dl = DeepL::new(&key);
    
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
                // else skip source lang if input contains variable lang
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
                    _ => bail!("invalid tag handling")
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
            if params.text.is_none() { 
                for ln in io::stdin().lines() {
                    text.push(ln.unwrap());
                }
            } else {
                let t = params.text.unwrap();
                if t.starts_with('-') {
                    for ln in io::stdin().lines() {
                        text.push(ln.unwrap());
                    }
                } else {
                    // Single string
                    text.push(t);
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
            let doc: Document;
            // Skip upload if doc handle present
            if let Some(document_id) = params.doc_id {
                let Some(document_key) = params.key else {
                    bail!("missing required document key")
                };
                doc = Document { document_id, document_key };
            } else {
                // Set required options
                // target_lang, in file
                let Some(trg) = params.target else {
                    bail!("missing required target language");
                };
                let Ok(target_lang) = trg.parse::<Language>() else {
                    bail!("invalid target lang")
                };
                let Some(fp) = params.file else {
                    bail!("missing document file path");
                };
                let file_path = PathBuf::from(fp);
                let mut opt = DocumentOptions::new(target_lang, file_path);
    
                // Set optional
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
                doc = dl.document_upload(opt)?;
                let Document { document_id, document_key } = &doc;
                
                let mut map = serde_json::Map::new();
                map.insert("document_id".to_string(), Value::String(document_id.clone()));
                map.insert("document_key".to_string(), Value::String(document_key.clone()));
                println!("{}", serde_json::to_string_pretty(&map)?);
            }

            // Poll status
            let mut is_done = false;
            let mut server_err: Option<String> = None;
            let mut secs = Duration::from_secs(2);
            for _ in 0..3 {
                thread::sleep(secs);

                let status = dl.document_status(&doc)?;
                println!("Status: {:?}", status.status);

                if status.is_done() {
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

            if server_err.is_some() {
                bail!("{}", server_err.unwrap());
            }
            if !is_done {
                println!("Please try download again with document id and key");
                bail!("Poll status timeout");
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
            let mut map = serde_json::Map::new();
            /*
            {
                source_languages: [
                    {
                        language: EN,
                        name: English,
                    },
                ],
                target_languages: [
                    {
                        ...
                    },
                ]
            }
            */
            
            let mut src: Vec<Value> = vec![];
            let languages = dl.languages(LanguageType::Source)?;
            for lang in languages {
                let mut map = serde_json::Map::new();
                map.insert("language".to_string(), Value::String(lang.language));
                map.insert("name".to_string(), Value::String(lang.name));
                let obj = Value::Object(map);
                src.push(obj);
            }
            map.insert("source_languages".to_string(), Value::Array(src));
            
            let mut trg: Vec<Value> = vec![];
            let languages = dl.languages(LanguageType::Target)?;
            for lang in languages {
                let mut map = serde_json::Map::new();
                map.insert("language".to_string(), Value::String(lang.language));
                map.insert("name".to_string(), Value::String(lang.name));
                map.insert("supports_formality".to_string(), Value::Bool(lang.supports_formality.unwrap()));
                let obj = Value::Object(map);
                trg.push(obj);
            }
            map.insert("target_languages".to_string(), Value::Array(trg));
            
            let json = serde_json::to_string_pretty(&map)?;
            println!("{json}");
        },
        // Glossary
        Cmd::Glossary(sub) => {
            match sub.cmd {
                Glos::Pairs => {
                    let pairs = dl.glossary_languages()?;
                    println!("{}", serde_json::to_string_pretty(&pairs)?);
                },
                Glos::List => {
                    let glossaries = dl.glossaries()?;
                    let json = serde_json::to_string_pretty(&glossaries)?;
                    println!("{json}");
                },
                Glos::Get(glos) => {
                    let glos = dl.glossary_info(&glos.id)?;
                    println!("{}", serde_json::to_string_pretty(&glos)?);
                },
                Glos::Entries(glos) => {
                    let entries = dl.glossary_entries(&glos.id)?;
                    for (k, v) in entries {
                        println!("{k} {v}");
                    }
                },
                Glos::Create(params) => {
                    let name = params.name;
                    let Ok(src) = params.source.parse::<Language>() else {
                        bail!("invalid source lang");
                    };
                    let Ok(trg) = params.target.parse::<Language>() else {
                        bail!("invalid source lang");
                    };

                    let entries: String;
                    let mut fmt_optional = true;
                    if let Some(fp) = params.file {
                        // Read in file
                        let file_path = PathBuf::from(fp);
                        entries = fs::read_to_string(file_path)?;
                    } else {
                        let Some(raw) = params.entries else {
                            bail!("missing required glossary entries");
                        };
                        if raw.starts_with('-') {
                            // Read from stdin
                            let mut s = String::new();
                            io::stdin().read_to_string(&mut s).expect("failed to read stdin");
                            entries = s;           
                        } else {
                            // else Parse cli arg "src=trg, src=trg,..."
                            fmt_optional = false;
                            let raw_entries: Vec<&str> = raw.split(',').map(|s| s.trim()).collect();
                            
                            let mut s = String::new();
                            for elem in raw_entries {
                                let mut pair: Vec<&str> = elem.split('=').map(|s| s.trim()).collect();
                                if pair.len() != 2 { continue }
                                let trg = pair.pop().unwrap();
                                let src = pair.pop().unwrap();
                                s.push_str(
                                    &format!("{src},")
                                );
                                s.push_str(
                                    &format!("{trg}\n")
                                );
                            }
                            entries = s;
                        }
                    }

                    // set entries format
                    let mut fmt = GlossaryEntriesFormat::Csv;
                    if fmt_optional && params.tsv {
                        fmt = GlossaryEntriesFormat::Tsv;
                    }

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
