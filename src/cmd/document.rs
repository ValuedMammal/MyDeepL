use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use deeprl::{DeepL, Document, DocumentOptions, Formality, Language};
use serde_json::Value;

use super::{bail, Result};
use crate::cli::DocParams;

/// Execute document command.
pub fn execute(dl: &DeepL, params: DocParams) -> Result<()> {
    let doc: Document;
    // Skip upload if doc handle present
    if let Some(document_id) = params.doc_id {
        let Some(document_key) = params.key else {
            bail!("missing required document key")
        };
        doc = Document {
            document_id,
            document_key,
        };
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
        if let Some(fml) = params.formality {
            let formality = match fml.as_str() {
                "more" => Formality::More,
                "less" => Formality::Less,
                "prefer_more" => Formality::PreferMore,
                "prefer_less" => Formality::PreferLess,
                _ => Formality::Default,
            };
            opt = opt.formality(formality);
        }
        if let Some(g) = params.glossary {
            opt = opt.glossary_id(g);
        }

        // Upload
        println!("Uploading file...");
        doc = dl.document_upload(opt)?;
        let Document {
            document_id,
            document_key,
        } = &doc;

        let mut map = serde_json::Map::new();
        map.insert(
            "document_id".to_string(),
            Value::String(document_id.clone()),
        );
        map.insert(
            "document_key".to_string(),
            Value::String(document_key.clone()),
        );
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
            break;
        }
        if let Some(msg) = status.error_message {
            server_err = Some(msg);
            break;
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

    Ok(())
}
