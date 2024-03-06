use partial_pretty_printer::{
    pretty_print, pretty_print_to_string, testing::oracular_pretty_print, Notation, NotationError,
    PrettyDoc, ValidNotation, Width,
};

#[derive(Debug, Clone)]
pub struct SimpleDoc(pub ValidNotation<(), ()>);

impl SimpleDoc {
    #[track_caller]
    pub fn new(notation: Notation<(), ()>) -> SimpleDoc {
        SimpleDoc(notation.validate().expect("Invalid notation"))
    }

    pub fn cheat_validation(notation: Notation<(), ()>) -> SimpleDoc {
        SimpleDoc(notation.cheat_validation_for_testing_only())
    }

    pub fn try_new(notation: Notation<(), ()>) -> Result<SimpleDoc, NotationError> {
        Ok(SimpleDoc(notation.validate()?))
    }
}

impl<'a> PrettyDoc<'a> for &'a SimpleDoc {
    type Id = usize;
    type Style = ();
    type StyleLabel = ();
    type Condition = ();

    fn id(self) -> usize {
        0
    }

    fn notation(self) -> &'a ValidNotation<(), ()> {
        &self.0
    }

    fn condition(self, _condition: &()) -> bool {
        false
    }

    fn node_style(self) -> () {
        ()
    }

    fn lookup_style(self, _label: ()) -> () {
        ()
    }

    fn num_children(self) -> Option<usize> {
        Some(0)
    }

    fn unwrap_text(self) -> &'a str {
        panic!("Nothing in a simple doc");
    }

    fn unwrap_child(self, _i: usize) -> Self {
        panic!("Nothing in a simple doc");
    }
}

#[track_caller]
fn compare_lines(message: &str, expected: (&'static str, String), actual: (&'static str, String)) {
    if actual.1 != expected.1 {
        eprintln!(
            "{}\n{}:\n{}\n{}:\n{}\n=========",
            message, expected.0, expected.1, actual.0, actual.1,
        );
        assert_eq!(actual, expected);
    }
}

fn print_above_and_below<'d, D: PrettyDoc<'d>>(
    doc: D,
    width: Width,
    path: &[usize],
    seek_end: bool,
) -> (Vec<String>, String, String, Vec<String>) {
    let (upward_printer, focused_line, downward_printer) =
        pretty_print(doc, width, path, seek_end).unwrap();
    let mut lines_above = upward_printer
        .map(|line| line.unwrap().to_string())
        .collect::<Vec<_>>();
    lines_above.reverse();
    let left_string = focused_line.to_left_string();
    let right_string = focused_line.to_right_string();
    let lines_below = downward_printer
        .map(|line| line.unwrap().to_string())
        .collect::<Vec<_>>();
    (lines_above, left_string, right_string, lines_below)
}

pub fn all_paths<'d, D: PrettyDoc<'d>>(doc: D) -> Vec<Vec<usize>> {
    fn recur<'d, D: PrettyDoc<'d>>(doc: D, path: &mut Vec<usize>, paths: &mut Vec<Vec<usize>>) {
        paths.push(path.clone());
        for i in 0..doc.num_children().unwrap_or(0) {
            path.push(i);
            recur(doc.unwrap_child(i), path, paths);
            path.pop();
        }
    }
    let mut paths = vec![];
    recur(doc, &mut vec![], &mut paths);
    paths
}

pub fn print_region<'d, D: PrettyDoc<'d>>(
    doc: D,
    width: Width,
    path: &[usize],
    seek_end: bool,
    rows: usize,
) -> Vec<String> {
    let (upward_printer, focused_line, downward_printer) =
        pretty_print(doc, width, path, seek_end).unwrap();
    let mut lines = upward_printer
        .map(|line| line.unwrap().to_string())
        .take(rows / 2)
        .collect::<Vec<_>>();
    lines.reverse();
    lines.push(focused_line.to_string());
    let mut lines_below = downward_printer
        .map(|line| line.unwrap().to_string())
        .take(rows / 2 - 1)
        .collect::<Vec<_>>();
    lines.append(&mut lines_below);
    lines
}

#[track_caller]
pub fn assert_pp<'d, D: PrettyDoc<'d>>(doc: D, width: Width, expected_lines: &[&str]) {
    assert_pp_impl(doc, width, Some(expected_lines));
}

#[track_caller]
pub fn assert_pp_without_expectation<'d, D: PrettyDoc<'d>>(doc: D, width: Width) {
    assert_pp_impl(doc, width, None)
}

#[track_caller]
fn assert_pp_impl<'d, D: PrettyDoc<'d>>(doc: D, width: Width, expected_lines: Option<&[&str]>) {
    let oracle_result = oracular_pretty_print(doc, width);
    if let Some(expected_lines) = expected_lines {
        compare_lines(
            "ORACLE DISAGREES WITH TEST CASE, SO TEST CASE MUST BE WRONG",
            ("ORACLE", oracle_result.clone()),
            ("TEST CASE", expected_lines.join("\n")),
        );
    }
    let lines = pretty_print_to_string(doc, width)
        .unwrap()
        .split('\n')
        .map(|s| s.to_owned())
        .collect::<Vec<_>>();
    if expected_lines.is_none() {
        compare_lines(
            &format!(
                "IN PRETTY PRINTING WITH WIDTH {}\nNOTATION\n{}",
                width,
                doc.notation()
            ),
            ("ORACLE", oracle_result.clone()),
            ("ACTUAL", lines.join("\n")),
        );
    } else {
        compare_lines(
            &format!("IN PRETTY PRINTING WITH WIDTH {}", width),
            ("EXPECTED", oracle_result.clone()),
            ("ACTUAL", lines.join("\n")),
        );
    }
    for path in all_paths(doc) {
        for seek_end in [false, true] {
            let (lines_above, left_string, right_string, lines_below) =
                print_above_and_below(doc, width, &path, seek_end);
            let lines = concat_lines(lines_above, left_string, right_string, lines_below);
            compare_lines(
                &format!(
                    "IN PRETTY PRINTING AT PATH {:?} (seek_end={})",
                    path, seek_end
                ),
                ("EXPECTED", oracle_result.clone()),
                ("ACTUAL", lines.join("\n")),
            );
        }
    }
}

fn concat_lines(
    lines_above: Vec<String>,
    left_string: String,
    right_string: String,
    lines_below: Vec<String>,
) -> Vec<String> {
    let mut lines = lines_above;
    let mut center_line = left_string;
    center_line.push_str(&right_string);
    lines.push(center_line);
    lines.extend(lines_below);
    lines
}

#[track_caller]
pub fn assert_pp_seek<'d, D: PrettyDoc<'d>>(
    doc: D,
    width: Width,
    path: &[usize],
    expected_lines: &[&str],
) {
    let (lines_above_1, left_string_1, right_string_1, lines_below_1) =
        print_above_and_below(doc, width, path, false);
    let (lines_above_2, left_string_2, right_string_2, lines_below_2) =
        print_above_and_below(doc, width, path, true);
    let start_row = lines_above_1.len();
    let end_row = lines_above_2.len();
    let start_col = left_string_1.len();
    let end_col = left_string_2.len();

    let lines_1 = concat_lines(lines_above_1, left_string_1, right_string_1, lines_below_1);
    let lines_2 = concat_lines(lines_above_2, left_string_2, right_string_2, lines_below_2);
    compare_lines(
        &format!("IN PRINTING AT PATH {:?}", path),
        ("SEEK_START", lines_1.join("\n")),
        ("SEEK_END", lines_2.join("\n")),
    );

    let mut lines_with_focus = lines_1;
    lines_with_focus[end_row].insert(end_col, ')');
    lines_with_focus[start_row].insert(start_col, '(');
    compare_lines(
        &format!("IN PRINTING AT PATH {:?}", path),
        ("EXPECTED", expected_lines.join("\n")),
        ("ACTUAL", lines_with_focus.join("\n")),
    );
}

#[track_caller]
pub fn assert_pp_region<'d, D: PrettyDoc<'d>>(
    doc: D,
    width: Width,
    path: &[usize],
    seek_end: bool,
    rows: usize,
    expected_lines: &[&str],
) {
    let lines = print_region(doc, width, path, seek_end, rows);
    compare_lines(
        &format!("IN PRINTING {} ROWS AT PATH {:?}", rows, path),
        ("EXPECTED", expected_lines.join("\n")),
        ("ACTUAL", lines.join("\n")),
    );
}
