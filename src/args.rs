pub use clap::{Parser, Subcommand};

/// DeepL cli - machine translation at the speed of Rust
#[derive(Parser, Debug)]
#[clap(
name = "deepl", 
version(env!("CARGO_PKG_VERSION")), 
propagate_version(true)
)]
pub struct Args {
    #[clap(subcommand)]
    pub cmd: Cmd,
}

#[derive(clap::Subcommand, Debug)]
pub enum Cmd {
    /// Fetch information about account usage & limits
    Usage,
    /// Translate text
    Text(TxtOpt),
    // /// Translate a document
    // Document(DocOpt),
    // /// Fetch list of available source and target languages
    // Languages,
    // /// Create, list, and remove glossaries
    // Glossary(GlosSub),
}

/// Translate text options
#[derive(Parser, Debug)]
pub struct TxtOpt {
    /// Source language
    #[clap(short = 's', long)]
    pub source: Option<String>,
    /// Target language (required)
    #[clap(short = 't', long)]
    pub target: String,
    /// Controls sentence splitting [0,1,nonewlines]
    #[clap(long)]
    pub split_sentences: Option<String>,
    /// Text to translate (default: read from stdin)
    #[clap(long)]
    pub text: Option<String>,
    /// Show detected source language
    #[clap(long)]
    pub show_detected: bool,
    /// Preserve formatting
    #[clap(long)]
    pub preserve_formatting: bool,
    /// Formality preference [more,less,prefer_more,prefer_less]
    #[clap(long)]
    pub formality: Option<String>,
    /// Glossary id to use for translation
    #[clap(long)]
    pub glossary: Option<String>,
    /// (experimental) Translate a list of newline-separated text of variable source lang. Only applies when reading stdin, overrides --source option
    #[clap(long)]
    pub multi_lang: bool,
    /// Actvates tag handling [xml,html]
    #[clap(long)]
    pub tag_handling: Option<String>,
    /// Turn off automatic outline detection used for tag handling
    #[clap(long)]
    pub no_outline_detection: bool,
    /// Tags which always split sentences, can be formatted as a comma-separated list e.g. "head,title,body"
    #[clap(long)]
    pub splitting_tags: Option<String>,
    /// Tags which never split sentences, format like splitting-tags
    #[clap(long)]
    pub non_splitting_tags: Option<String>,
    /// Tags indicating text not to be translated, format like splitting-tags
    #[clap(long)]
    pub ignore_tags: Option<String>,
}
/*

/// Translate document options
#[derive(Parser, Debug)]
pub struct DocOpt {
    /// Source lang
    #[clap(long)]
    pub source: Option<String>,
    /// Target lang
    #[clap(long)]
    pub target: String,
    /// Path to input file
    #[clap(long)]
    pub file: String,
    /// Document filename
    #[clap(long)]
    pub filename: Option<String>,
    /// Output file
    #[clap(long)]
    pub out_file: Option<String>,
    /// Formality preference
    #[clap(long)]
    pub formality: Option<String>,
    /// Glossary id
    #[clap(long)]
    pub glossary: Option<String>,
}

#[derive(Parser, Debug)]
pub struct GlosSub {
    ///
    #[clap(subcommand)]
    pub cmd: Glos,
}

#[derive(clap::Subcommand, Debug)]
pub enum Glos {
    /// Create a new glossary
    Create(GlosNew),
    /// List available glossaries
    List,
    /// Get glossary metadata
    Get(GlosGet),
    /// Retrieve entries from a glossary
    Entries(GlosEntry),
    /// Delete a glossary
    Delete(GlosDel),
}

#[derive(Parser, Debug)]
pub struct GlosNew {
    /// Name of new glossary
    #[clap(long)]
    pub name: String,
    /// Source language
    #[clap(long)]
    pub source: String,
    /// Target language
    #[clap(long)]
    pub target: String,
    /// Path to input data. Expects source/target pairs in TSV format, one entry per line
    #[clap(long)]
    pub file: String,
    /// Interpret data from input as CSV rather than TSV
    #[clap(long)]
    pub csv: bool,
    // One or more glossary entries formatted [SRC=TRG ...]. If --file is specified, then this option is ignored.
    //TODO:
    // #[clap(long)]
    // pub entries: Option<String>,
}

#[derive(Parser, Debug)]
pub struct GlosGet {
    /// Glossary id
    pub id: String,
}

#[derive(Parser, Debug)]
pub struct GlosEntry {
    /// Glossary id
    pub id: String,
}

#[derive(Parser, Debug)]
pub struct GlosDel {
    /// Glossary id
    pub id: String,
}
*/