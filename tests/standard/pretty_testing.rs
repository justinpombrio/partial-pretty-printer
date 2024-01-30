use partial_pretty_printer::{
    notation_constructors::lit, pretty_print, pretty_print_to_string,
    testing::oracular_pretty_print, Notation, NotationError, PrettyDoc, ValidNotation, Width,
};

// TODO: temporary
#[allow(unused)]
pub fn punct(s: &'static str) -> Notation<()> {
    lit(s, ())
}

#[derive(Debug, Clone)]
pub struct SimpleDoc<S>(pub ValidNotation<S>);

impl<S> SimpleDoc<S> {
    pub fn new(notation: Notation<S>) -> SimpleDoc<S> {
        SimpleDoc(notation.validate().expect("Invalid notation"))
    }

    pub fn try_new(notation: Notation<S>) -> Result<SimpleDoc<S>, NotationError> {
        Ok(SimpleDoc(notation.validate()?))
    }
}

impl<'a, S> PrettyDoc<'a> for &'a SimpleDoc<S> {
    type Id = usize;
    type Style = S;
    type Mark = ();

    fn id(self) -> usize {
        // shouldn't be the default of usize
        1
    }

    fn notation(self) -> &'a ValidNotation<S> {
        &self.0
    }

    fn mark(self) -> Option<&'a ()> {
        None
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
) -> (Vec<String>, Vec<String>) {
    let (upward_printer, downward_printer) = pretty_print(doc, width, path).unwrap();
    let mut lines_above = upward_printer
        .map(|line| line.unwrap().to_string())
        .collect::<Vec<_>>();
    lines_above.reverse();
    let lines_below = downward_printer
        .map(|line| line.unwrap().to_string())
        .collect::<Vec<_>>();
    (lines_above, lines_below)
}

#[allow(unused)]
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

#[allow(unused)]
pub fn print_region<'d, D: PrettyDoc<'d>>(
    doc: D,
    width: Width,
    path: &[usize],
    rows: usize,
) -> Vec<String> {
    let (upward_printer, downward_printer) = pretty_print(doc, width, path).unwrap();
    let mut lines = upward_printer
        .map(|line| line.unwrap().to_string())
        .take(rows / 2)
        .collect::<Vec<_>>();
    lines.reverse();
    let mut lines_below = downward_printer
        .map(|line| line.unwrap().to_string())
        .take(rows / 2)
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
    compare_lines(
        &format!("IN PRETTY PRINTING WITH WIDTH {}", width),
        ("EXPECTED", oracle_result.clone()),
        ("ACTUAL", lines.join("\n")),
    );
    for path in all_paths(doc) {
        let (lines_above, mut lines_below) = print_above_and_below(doc, width, &path);
        let mut lines = lines_above;
        lines.append(&mut lines_below);
        compare_lines(
            &format!("IN PRETTY PRINTING AT PATH {:?}", path),
            ("EXPECTED", oracle_result.clone()),
            ("ACTUAL", lines.join("\n")),
        );
    }
}

#[track_caller]
pub fn assert_pp_seek<'d, D: PrettyDoc<'d>>(
    doc: D,
    width: Width,
    path: &[usize],
    expected_lines_above: &[&str],
    expected_lines_below: &[&str],
) {
    let (lines_above, lines_below) = print_above_and_below(doc, width, path);
    compare_lines(
        &format!("IN DOWNWARD PRINTING AT PATH {:?}", path),
        ("EXPECTED", expected_lines_below.join("\n")),
        ("ACTUAL", lines_below.join("\n")),
    );
    compare_lines(
        &format!("IN UPWARD PRINTING AT PATH {:?}", path),
        ("EXPECTED", expected_lines_above.join("\n")),
        ("ACTUAL", lines_above.join("\n")),
    );
}

#[track_caller]
pub fn assert_pp_region<'d, D: PrettyDoc<'d>>(
    doc: D,
    width: Width,
    path: &[usize],
    rows: usize,
    expected_lines: &[&str],
) {
    let lines = print_region(doc, width, path, rows);
    compare_lines(
        &format!("IN PRINTING {} ROWS AT PATH {:?}", rows, path),
        ("EXPECTED", expected_lines.join("\n")),
        ("ACTUAL", lines.join("\n")),
    );
}
