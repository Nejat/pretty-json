use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter, Error, ErrorKind, Result, Write};
use std::path::Path;

use itertools::Itertools;
use serde_json::{Map, Value};

const INDENT: usize = 4;
const USAGE: &str = "usage: pretty-json <source> [output]";

fn main() -> Result<()> {
    let source = env::args().nth(1).ok_or_else(|| Error::new(ErrorKind::Other, USAGE))?;
    let source_json = read_json(&source)?;
    let pretty_output = pretty_name(&source)?;
    let output = File::create(pretty_output)?;
    let mut output = BufWriter::new(output);

    write_pretty(&mut output, &source_json, 0)
}

fn write_pretty<W: Write>(output: &mut W, value: &Value, level: usize) -> Result<()> {
    match value {
        Value::Null => write!(output, "null"),
        Value::Bool(value) => write!(output, "{value}"),
        Value::Number(value) => write!(output, "{value}"),
        Value::String(value) => write!(output, "{value:?}"),
        Value::Array(array) => write_pretty_array(output, &array, level),
        Value::Object(object) => write_pretty_object(output, &object, level)
    }
}

fn write_pretty_array<W: Write>(output: &mut W, array: &Vec<Value>, level: usize) -> Result<()> {
    let padding = level * INDENT;
    let field_padding = (level + 1) * INDENT;

    let chunked = array.len() > 10 || array.iter().any(
        |itm| matches!(itm, Value::Array(_) | Value::Object(_))
    );

    if chunked {
        let mut idx = 0;
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
                write_pretty(output, &value, level + 1)?;

                if idx < max {
                    write!(output, ", ")?;
                }

                idx += 1;
            }

            writeln!(output)?;
        }

        write!(output, "{:padding$}]", ' ')
    } else {
        let mut idx = 0;

        write!(output, "[")?;

        for value in array.iter() {
            write_pretty(output, &value, level + 1)?;

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
) -> Result<()> {
    let padding = level * INDENT;
    let field_padding = (level + 1) * INDENT;
    let last = object.len() - 1;

    writeln!(output, "{{")?;

    for (idx, (field, value)) in object.iter().enumerate() {
        write!(output, "{:field_padding$}\"{field}\": ", ' ')?;

        write_pretty(output, value, level + 1)?;

        if idx < last {
            write!(output, ", ")?;
        }

        writeln!(output)?;
    }

    write!(output, "{:padding$}}}", ' ')
}

fn pretty_name(source: &String) -> Result<String> {
    env::args().nth(2)
        .or_else(|| {
            let pretty = Path::new(&source);
            let extension = pretty.extension().unwrap_or_default().to_str().unwrap_or_default();
            let mut filename = pretty.file_name().unwrap().to_string_lossy().to_string();

            filename = filename.replace(extension, "");

            if filename.ends_with('.') {
                filename.pop();
            }

            filename.push_str("-pretty");

            if extension.len() > 0 {
                filename.push('.');
                filename.push_str(extension);
            }

            let pretty = pretty.with_file_name(filename);

            Some(pretty.to_string_lossy().to_string())
        })
        .ok_or_else(|| Error::new(ErrorKind::Other, USAGE))
}

fn read_json(source: &String) -> Result<Value> {
    let file = File::open(&source)?;
    let reader = BufReader::new(file);

    serde_json::from_reader(reader)
        .map_err(|err| Error::new(ErrorKind::InvalidData, format!("{err}")))
}
