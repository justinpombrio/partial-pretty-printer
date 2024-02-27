use partial_pretty_printer::{
    examples::{
        json::{json_dict, json_list, json_number, json_string, Json},
        BasicStyle, Color,
    },
    pretty_print, Line, Width,
};

#[derive(Debug)]
struct RichChar {
    ch: char,
    id: u32,
    style: BasicStyle,
}

#[derive(Debug)]
struct RichText {
    lines: Vec<Vec<RichChar>>,
}

fn print(doc: &Json, width: Width) -> RichText {
    let (upward_printer, focused_line, downward_printer) =
        pretty_print(doc, width, &[], false).unwrap();
    let mut rich_text = RichText::new();
    let mut lines_above = upward_printer.collect::<Vec<_>>();
    lines_above.reverse();
    for line in lines_above {
        rich_text.push_line(line.unwrap());
    }
    rich_text.push_line(Line::from(focused_line));
    for line in downward_printer {
        rich_text.push_line(line.unwrap());
    }
    rich_text
}

impl RichChar {
    fn style_char(&self) -> char {
        let ch = match self.style.color {
            Color::White => 'w',
            Color::Red => 'r',
            Color::Green => 'g',
            Color::Yellow => 'y',
            Color::Blue => 'b',
            Color::Magenta => 'm',
            Color::Cyan => 'c',
        };
        if self.style.bold {
            ch.to_ascii_uppercase()
        } else {
            ch
        }
    }
}

impl RichText {
    fn new() -> Self {
        RichText { lines: Vec::new() }
    }

    fn push_line<'d>(&mut self, line: Line<'d, &'d Json>) {
        let mut chars = Vec::new();
        for segment in line.segments {
            for ch in segment.str.chars() {
                chars.push(RichChar {
                    ch,
                    id: segment.doc_id,
                    style: segment.style.clone(),
                });
            }
        }
        self.lines.push(chars);
    }

    fn display_text(&self) -> String {
        let mut s = String::new();
        for line in &self.lines {
            for rich_char in line {
                s.push(rich_char.ch);
            }
            s.push('\n');
        }
        // Get rid of the trailing newline
        s.pop();
        s
    }

    fn display_ids(&self) -> String {
        let mut s = String::new();
        for line in &self.lines {
            for rich_char in line {
                s.push(char::from_digit(rich_char.id % 36, 36).unwrap());
            }
            s.push('\n');
        }
        s.pop();
        s
    }

    fn display_styles(&self) -> String {
        let mut s = String::new();
        for line in &self.lines {
            for rich_char in line {
                s.push(rich_char.style_char());
            }
            s.push('\n');
        }
        s.pop();
        s
    }
}

fn assert_str_eq(expected: &str, actual: &str) {
    if expected != actual {
        println!("EXPECTED:\n{}", expected);
        println!("ACTUAL:\n{}", actual);
        println!("END");
        assert_eq!(expected, actual);
    }
}

fn make_json_dictionary() -> Json {
    Json::reset_id();
    json_dict(vec![
        (
            "Name",
            json_string("Alice").with_style(BasicStyle::new().bold()),
        ),
        ("Age", json_number(42.0)),
        ("Pets", json_list(Vec::new())),
        (
            "Favorites",
            json_list(vec![
                json_string("chocolate"),
                json_string("lemon").with_style(BasicStyle::new().color(Color::Yellow)),
                json_string("almond"),
            ])
            .with_style(BasicStyle::new().color(Color::Red).bold())
            .with_style_override("open", BasicStyle::new().color(Color::Cyan))
            .with_style_override("close", BasicStyle::new().color(Color::Blue)),
        ),
    ])
    .with_style_override("open", BasicStyle::new().color(Color::Cyan))
    .with_style_override("close", BasicStyle::new().color(Color::Blue))
}

#[test]
fn test_json_marks() {
    let json = make_json_dictionary();
    let rich_text = print(&json, 27);
    assert_str_eq(
        &[
            r#"{"#,
            r#"    "Name": "Alice","#,
            r#"    "Age": 42,"#,
            r#"    "Pets": [],"#,
            r#"    "Favorites": ["#,
            r#"        "chocolate","#,
            r#"        "lemon","#,
            r#"        "almond""#,
            r#"    ]"#,
            r#"}"#,
        ]
        .join("\n"),
        &rich_text.display_text(),
    );

    assert_str_eq(
        &[
            r#"g"#,
            r#"gggg888888991111111g"#,
            r#"ggggaaaaabb22g"#,
            r#"ggggccccccdd33g"#,
            r#"ggggeeeeeeeeeeeff7"#,
            r#"gggg7777444444444447"#,
            r#"gggg777755555557"#,
            r#"gggg777766666666"#,
            r#"gggg7"#,
            r#"g"#,
        ]
        .join("\n"),
        &rich_text.display_ids(),
    );

    assert_str_eq(
        &[
            r#"c"#,
            r#"wwwwmmmmmmwwMMMMMMMw"#,
            r#"wwwwmmmmmwwbbw"#,
            r#"wwwwmmmmmmwwwww"#,
            r#"wwwwmmmmmmmmmmmwwC"#,
            r#"wwwwRRRRMMMMMMMMMMMR"#,
            r#"wwwwRRRRMMMMMMMR"#,
            r#"wwwwRRRRMMMMMMMM"#,
            r#"wwwwB"#,
            r#"b"#,
        ]
        .join("\n"),
        &rich_text.display_styles(),
    );

    let rich_text = print(&json, 90);
    assert_str_eq(
        r#"{"Name": "Alice", "Age": 42, "Pets": [], "Favorites": ["chocolate", "lemon", "almond"]}"#,
        &rich_text.display_text(),
    );

    assert_str_eq(
        r#"g888888991111111ggaaaaabb22ggccccccdd33ggeeeeeeeeeeeff74444444444477555555577666666667g"#,
        &rich_text.display_ids(),
    );

    assert_str_eq(
        r#"cmmmmmmwwMMMMMMMwwmmmmmwwbbwwmmmmmmwwwwwwmmmmmmmmmmmwwCMMMMMMMMMMMRRMMMMMMMRRMMMMMMMMBb"#,
        &rich_text.display_styles(),
    );
}
