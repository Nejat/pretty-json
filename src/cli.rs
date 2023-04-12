use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[allow(clippy::upper_case_acronyms)]
pub struct CLI {
    /// Source JSON path
    pub source: String,

    /// Optional, output JSON Path, default will append '-pretty' to source
    pub output: Option<String>,

    /// Optional, list of object properties to output flat, use dot notation for deep properties
    #[arg(short, long, use_value_delimiter = true, value_delimiter = ',')]
    pub flat: Option<Vec<String>>,
}