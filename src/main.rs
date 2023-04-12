use std::fs::File;
use std::io::{BufReader, BufWriter, Error, ErrorKind, Result, Write};
use std::path::Path;

use clap::Parser;
use itertools::Itertools;
use serde_json::{Map, Value};

use crate::cli::CLI;

mod cli;

const INDENT: usize = 4;

fn main() -> Result<()> {
    let cli = CLI::parse();
    let source_json = read_json(&cli.source)?;
    let pretty_output = pretty_name(&cli);
    let output = File::create(pretty_output)?;
    let mut output = BufWriter::new(output);
    let mut property = String::default();

    write_pretty(&mut output, &source_json, 0, &mut property, &cli.flat.unwrap_or_default())
}

fn write_pretty<W: Write>(
    output: &mut W,
    value: &Value,
    level: usize,
    property: &mut String,
    flat: &[String],
) -> Result<()> {
    match value {
        Value::Null => write!(output, "null"),
        Value::Bool(value) => write!(output, "{value}"),
        Value::Number(value) => write!(output, "{value}"),
        Value::String(value) => write!(output, "{value:?}"),
        Value::Array(array) => write_pretty_array(output, array, level, property, flat),
        Value::Object(object) => write_pretty_object(output, object, level, property, flat)
    }
}

fn write_pretty_array<W: Write>(
    output: &mut W,
    array: &Vec<Value>,
    level: usize,
    property: &mut String,
    flat: &[String],
) -> Result<()> {
    let padding = level * INDENT;
    let field_padding = (level + 1) * INDENT;
    let mut idx = 0;

    let chunked = array.len() > 10 || array.iter().any(
        |itm| matches!(itm, Value::Array(_) | Value::Object(_))
    );

    if chunked {
        let max = array.len().max(1) - 1;

        let has_objects = array.iter().any(|itm|
            matches!(itm, Value::Array(items) if items.len() > 3) ||
                matches!(itm, Value::Object(_))
        );

        let chunks = if has_objects {
            1
        } else if array.iter().any(|itm| matches!(itm, Value::Array(_))) {
            5
        } else {
            10
        };

        writeln!(output, "[")?;

        for values in &array.iter().chunks(chunks) {
            write!(output, "{:field_padding$}", ' ')?;

            for value in values {
                write_pretty(output, value, level + 1, property, flat)?;

                if idx < max {
                    write!(output, ", ")?;
                }

                idx += 1;
            }

            writeln!(output)?;
        }

        write!(output, "{:padding$}]", ' ')
    } else {
        write!(output, "[")?;

        for value in array.iter() {
            write_pretty(output, value, level + 1, property, flat)?;

            if idx < array.len() - 1 {
                write!(output, ", ")?;
            }

            idx += 1;
        }

        write!(output, "]")
    }
}

fn write_pretty_object<W: Write>(
    output: &mut W,
    object: &Map<String, Value>,
    level: usize,
    property: &mut String,
    flat: &[String],
) -> Result<()> {
    let padding = level * INDENT;
    let field_padding = (level + 1) * INDENT;
    let last = object.len() - 1;
    let flatten = flat.contains(property);

    if flatten {
        write!(output, "{{ ")?;
    } else {
        writeln!(output, "{{")?;
    }

    for (idx, (field, value)) in object.iter().enumerate() {
        if flatten {
            write!(output, "\"{field}\": ")?;
        } else {
            write!(output, "{:field_padding$}\"{field}\": ", ' ')?;
        }

        let mut property = property.clone();

        if !property.is_empty() {
            property.push('.');
        }

        property.push_str(field);

        write_pretty(output, value, level + 1, &mut property, flat)?;

        if idx < last {
            write!(output, ", ")?;
        }

        if !flatten {
            writeln!(output)?;
        }
    }

    if flatten {
        write!(output, " }}")
    } else {
        write!(output, "{:padding$}}}", ' ')
    }
}

fn pretty_name(cli: &CLI) -> String {
    cli.output.clone().unwrap_or_else(|| {
        let pretty = Path::new(&cli.source);
        let extension = pretty.extension().unwrap_or_default().to_str().unwrap_or_default();
        let mut filename = pretty.file_name().unwrap().to_string_lossy().to_string();

        filename = filename.replace(extension, "");

        if filename.ends_with('.') {
            filename.pop();
        }

        filename.push_str("-pretty");

        if !extension.is_empty() {
            filename.push('.');
            filename.push_str(extension);
        }

        let pretty = pretty.with_file_name(filename);

        pretty.to_string_lossy().to_string()
    })
}

fn read_json(source: &str) -> Result<Value> {
    let file = File::open(source)?;
    let reader = BufReader::new(file);

    serde_json::from_reader(reader)
        .map_err(|err| Error::new(ErrorKind::InvalidData, format!("{err}")))
}
