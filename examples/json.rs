use clap::Parser;
use partial_pretty_printer::doc_examples::json::{
    json_array, json_bool, json_null, json_number, json_object, json_object_pair, json_string, Json,
};
use partial_pretty_printer::doc_examples::Color;
use partial_pretty_printer::{pretty_print, FocusTarget, Line, Width};
use std::fmt;
use std::process;

const DEFAULT_WIDTH: Width = 80;

/// Reformat JSON documents.
#[derive(Parser, Debug)]
#[command(version, about)]
struct CommandLineArgs {
    /// JSON file to parse and then pretty print.
    #[arg()]
    filename: String,
    /// Maximum line width. Defaults to 80.
    #[arg(short, long)]
    width: Option<Width>,
}

fn json_to_doc(json: serde_json::Value) -> Json {
    use serde_json::Value::{Array, Bool, Null, Number, Object, String};

    match json {
        Null => json_null(),
        Bool(b) => json_bool(b),
        Number(n) => json_number(n.as_f64().expect("Failed to convert number to f64")),
        String(s) => json_string(&s),
        Array(elems) => json_array(elems.into_iter().map(json_to_doc).collect::<Vec<_>>()),
        Object(entries) => json_object(
            entries
                .into_iter()
                .map(|(key, val)| json_object_pair(&key, json_to_doc(val)))
                .collect::<Vec<_>>(),
        ),
    }
}

fn pretty_print_json<'a>(doc: &'a Json, width: Width) -> Vec<Line<'a, &'a Json>> {
    let mut lines = Vec::new();
    let (_prev_lines, focused_line, next_lines) =
        unwrap(pretty_print(doc, width, &[], FocusTarget::Start, None));
    lines.push(Line::from(focused_line));
    for line in next_lines {
        lines.push(unwrap(line));
    }
    lines
}

fn lines_to_string<'d>(lines: Vec<Line<'d, &'d Json>>) -> Result<String, fmt::Error> {
    use std::fmt::Write;
    use termion::color;
    use Color::*;

    let mut string = String::new();
    let w = &mut string;
    for line in lines {
        for segment in line.segments {
            match segment.style.color {
                White => write!(w, "{}", color::Fg(color::White))?,
                Black => write!(w, "{}", color::Fg(color::Black))?,
                Red => write!(w, "{}", color::Fg(color::Red))?,
                Green => write!(w, "{}", color::Fg(color::Green))?,
                Yellow => write!(w, "{}", color::Fg(color::Yellow))?,
                Blue => write!(w, "{}", color::Fg(color::Blue))?,
                Magenta => write!(w, "{}", color::Fg(color::Magenta))?,
                Cyan => write!(w, "{}", color::Fg(color::Cyan))?,
            };
            write!(w, "{}", segment.str)?;
        }
        writeln!(w)?;
    }
    write!(w, "{}", color::Fg(color::Reset))?;
    Ok(string)
}

/// Like `.unwrap()`, but prints errors with Display.
fn unwrap<T, E: fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(ok) => ok,
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    }
}

fn main() {
    use std::fs::File;
    use std::io::BufReader;
    use std::time::Instant;

    // Get the filename to parse from the command line args
    let args = CommandLineArgs::parse();
    let width = args.width.unwrap_or(DEFAULT_WIDTH);
    let filename = args.filename;

    // Parse the file into json using serde
    let start = Instant::now();
    let file = unwrap(File::open(filename));
    let reader = BufReader::new(file);
    let json = unwrap(serde_json::from_reader(reader));
    let ms_to_parse = start.elapsed().as_millis();

    // Convert it to a Notation
    let start = Instant::now();
    let doc = json_to_doc(json);
    let ms_to_construct = start.elapsed().as_millis();

    // Pretty print the Notation
    let start = Instant::now();
    let lines = pretty_print_json(&doc, width);
    let num_lines = lines.len() as u128;
    let string = unwrap(lines_to_string(lines));
    let ms_to_pretty_print = start.elapsed().as_millis();

    // Print it to the terminal
    let start = Instant::now();
    println!("{}", string);
    let ms_to_output = start.elapsed().as_millis();

    let ms_total = ms_to_parse + ms_to_construct + ms_to_pretty_print + ms_to_output;
    let pp_speed = if ms_to_pretty_print == 0 {
        0
    } else {
        num_lines * 1000 / ms_to_pretty_print
    };
    let overall_speed = if ms_total == 0 {
        0
    } else {
        num_lines * 1000 / ms_total
    };

    // Print timing info to stderr
    eprintln!("Time to parse file as Json:    {} ms", ms_to_parse);
    eprintln!("Time to construct document:    {} ms", ms_to_construct);
    eprintln!("Time to pretty print document: {} ms", ms_to_pretty_print);
    eprintln!("Time to print to terminal:     {} ms", ms_to_output);
    eprintln!();
    eprintln!("Pretty printing speed:         {} lines/sec", pp_speed);
    eprintln!("Overall speed:                 {} lines/sec", overall_speed);
}
