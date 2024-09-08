use std::io::{self, Read};

use deeprl::{DeepL, Formality, Language, SplitSentences, TagHandling, TextOptions};

use super::{bail, Result};
use crate::cli::TextParams;

/// Execute text command.
pub fn execute(dl: &DeepL, params: TextParams) -> Result<()> {
    // Check we have a valid target lang
    let Ok(target_lang) = params.target.parse::<Language>() else {
        bail!("invalid target lang")
    };

    // Set optional
    // src, split_sent, preserve, formal, glos, tags, outline, split_tag, nonsplit_tag, ignore_tag
    let mut opt = TextOptions::new(target_lang);

    // skip source lang if input contains variable lang
    if !params.multi_lang {
        if let Some(src) = params.source {
            let Ok(source_lang) = src.parse::<Language>() else {
                bail!("invalid source lang")
            };
            opt = opt.source_lang(source_lang);
        }
    }

    let mut split_sentences: Option<SplitSentences> = None;
    if let Some(ss) = params.split_sentences {
        let split = match ss.as_str() {
            "0" => SplitSentences::None,
            "nonewlines" => SplitSentences::NoNewlines,
            _ => SplitSentences::Default,
        };
        opt = opt.split_sentences(split);
        split_sentences = Some(split);
    }
    if params.preserve_formatting {
        opt = opt.preserve_formatting(true);
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
    if let Some(t) = params.tag_handling {
        let tag = match t.to_lowercase().as_str() {
            "xml" => TagHandling::Xml,
            "html" => TagHandling::Html,
            _ => bail!("invalid tag handling"),
        };
        opt = opt.tag_handling(tag);
    }
    if params.no_outline_detection {
        opt = opt.outline_detection(false);
    }
    if let Some(tags) = params.splitting_tags {
        let tags: Vec<_> = tags.split(",").map(ToOwned::to_owned).collect();
        opt = opt.splitting_tags(tags);
    }
    if let Some(tags) = params.non_splitting_tags {
        let tags: Vec<_> = tags.split(",").map(ToOwned::to_owned).collect();
        opt = opt.non_splitting_tags(tags);
    }
    if let Some(tags) = params.ignore_tags {
        let tags: Vec<_> = tags.split(",").map(ToOwned::to_owned).collect();
        opt = opt.ignore_tags(tags);
    }

    // Get input text and call translate api
    let mut text: Vec<String> = vec![];
    let input = match params.text {
        None => {
            // read stdin
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).unwrap();
            buf
        }
        Some(t) => {
            if t.starts_with('-') {
                // read stdin
                let mut buf = String::new();
                io::stdin().read_to_string(&mut buf).unwrap();
                buf
            } else {
                // text cli option
                t
            }
        }
    };

    // TODO: this seems wrong. instead of applying split-sentence logic (which is handled by the
    // server) we should just split input on '\n' by default
    match split_sentences {
        None | Some(SplitSentences::Default) => {
            // split lines (default)
            // send many for separate translation
            for ln in input.lines() {
                text.push(ln.to_string());
            }
        }
        Some(SplitSentences::None | SplitSentences::NoNewlines) => {
            // no split
            text.push(input);
        }
    }

    opt = opt.text(text);

    let result = dl.translate(opt)?;
    for t in result.translations {
        if params.show_detected {
            println!("Detected: {}", t.detected_source_language);
        }
        println!("{}", t.text);
    }

    Ok(())
}
