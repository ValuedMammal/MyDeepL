use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use deeprl::{DeepL, GlossaryEntriesFormat, Language};

use super::{bail, Result};
use crate::cli::{Glos, GlosSub};

pub fn execute(dl: &DeepL, sub: GlosSub) -> Result<()> {
    match sub.cmd {
        Glos::Pairs => {
            let pairs = dl.glossary_languages()?;
            println!("{}", serde_json::to_string_pretty(&pairs)?);
        }
        Glos::List => {
            let glossaries = dl.glossaries()?;
            let json = serde_json::to_string_pretty(&glossaries)?;
            println!("{json}");
        }
        Glos::Get(glos) => {
            let glos = dl.glossary_info(&glos.id)?;
            println!("{}", serde_json::to_string_pretty(&glos)?);
        }
        Glos::Entries(glos) => {
            let entries = dl.glossary_entries(&glos.id)?;
            for (k, v) in entries {
                println!("{k} {v}");
            }
        }
        Glos::Create(params) => {
            let name = params.name;
            let Ok(src) = params.source.parse::<Language>() else {
                bail!("invalid source lang");
            };
            let Ok(trg) = params.target.parse::<Language>() else {
                bail!("invalid source lang");
            };

            let (entries, formattable) = if let Some(fp) = params.file {
                // Read in file
                let file_path = PathBuf::from(fp);
                (fs::read_to_string(file_path)?, true)
            } else {
                // Parse cli
                let Some(raw) = params.entries else {
                    bail!("missing required glossary entries");
                };
                if raw.starts_with('-') {
                    // Read from stdin
                    let mut entries = String::new();
                    io::stdin()
                        .read_to_string(&mut entries)
                        .expect("read stdin");
                    (entries, true)
                } else {
                    // reformat entries from:
                    //      "SRC=TRG,...,"
                    // to (csv):
                    //      "SRC,TRG\n..."
                    let raw_entries: Vec<&str> = raw.split(',').map(|s| s.trim()).collect();

                    let mut entries = String::new();
                    for entry in raw_entries {
                        let pair: Vec<&str> = entry.split('=').map(|s| s.trim()).collect();
                        if pair.len() != 2 {
                            continue;
                        }
                        let src = pair[0];
                        let trg = pair[1];
                        entries.push_str(&format!("{src},"));
                        entries.push_str(&format!("{trg}\n"));
                    }
                    (entries, false)
                }
            };

            // set entries format
            let fmt = if formattable && params.tsv {
                GlossaryEntriesFormat::Tsv
            } else {
                GlossaryEntriesFormat::Csv
            };

            let glos = dl.glossary_new(name, src, trg, entries, fmt)?;
            println!("{}", glos.glossary_id);
        }
        // Delete a glossary
        Glos::Delete(glos) => {
            let _ = dl.glossary_delete(&glos.id);
            println!("Done.");
        }
    }

    Ok(())
}
