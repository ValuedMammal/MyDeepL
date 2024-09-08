use deeprl::{DeepL, LanguageType};
use serde_json::Map;
use serde_json::Value;

use super::Result;

pub fn execute(dl: &DeepL) -> Result<()> {
    let mut map = Map::new();
    /*
    "source_languages": [
        {
            "language": EN,
            "name": English,
        },
    ],
    "target_languages": [
        {
            "language": EN-US,
            "name": English (American),
            "supports_formality": false
        },
    ]
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
        map.insert(
            "supports_formality".to_string(),
            Value::Bool(lang.supports_formality.unwrap()),
        );
        let obj = Value::Object(map);
        trg.push(obj);
    }
    map.insert("target_languages".to_string(), Value::Array(trg));

    let json = serde_json::to_string_pretty(&map)?;
    println!("{json}");

    Ok(())
}
