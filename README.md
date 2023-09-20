# MyDeepL
DeepL command line utility

### Usage

```
$ export DEEPL_API_KEY=<YOUR_KEY>
```

```
$ deepl --help

DeepL cli - machine translation at the speed of Rust

Usage: deepl <COMMAND>

Commands:
  usage      Get account usage & limits
  text       Translate text
  document   Translate a document
  languages  Fetch list of available source and target languages
  glossary   Create, list, and remove glossaries
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

```
$ echo 'the red crab' | deepl text -s 'en' -t 'fr'
# le crabe rouge
```
