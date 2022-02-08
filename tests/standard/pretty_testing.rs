use partial_pretty_printer::{
    notation_constructors::lit, pretty_print, pretty_print_to_string,
    testing::oracular_pretty_print, Notation, PrettyDoc, Style, Width,
};

#[allow(unused)]
pub fn punct(s: &'static str) -> Notation {
    lit(s, Style::plain())
}

#[derive(Debug, Clone)]
pub struct SimpleDoc(pub Notation);

impl<'a> PrettyDoc<'a> for &'a SimpleDoc {
    type Id = usize;

    fn id(self) -> usize {
        0
    }

    fn notation(self) -> &'a Notation {
        &self.0
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
fn compare_lines(message: &str, expected: String, actual: String, doc: Option<&str>) {
    if actual != expected {
        if let Some(doc) = doc {
            eprintln!(
                "{}\nDOC:\n{}\nEXPECTED:\n{}\nACTUAL:\n{}\n=========",
                message, doc, expected, actual,
            );
        } else {
            eprintln!(
                "{}\nEXPECTED:\n{}\nACTUAL:\n{}\n=========",
                message, expected, actual,
            );
        }
        assert_eq!(actual, expected);
    }
}

fn print_above_and_below<'d, D: PrettyDoc<'d>>(
    doc: D,
    width: Width,
    path: &[usize],
) -> (Vec<String>, Vec<String>) {
    let (upward_printer, downward_printer) = pretty_print(doc, width, path);
    let mut lines_above = upward_printer
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    lines_above.reverse();
    let lines_below = downward_printer
        .map(|line| line.to_string())
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
    let (upward_printer, downward_printer) = pretty_print(doc, width, path);
    let mut lines = upward_printer
        .map(|line| line.to_string())
        .take(rows / 2)
        .collect::<Vec<_>>();
    lines.reverse();
    let mut lines_below = downward_printer
        .map(|line| line.to_string())
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
    let contents = if expected_lines.is_some() {
        None
    } else {
        Some(format!("{}", doc.notation()))
    };
    let oracle_result = oracular_pretty_print(doc, width);
    if let Some(expected_lines) = expected_lines {
        compare_lines(
            "ORACLE DISAGREES WITH TEST CASE, SO TEST CASE MUST BE WRONG",
            oracle_result.clone(),
            expected_lines.join("\n"),
            None,
        );
    }
    let lines = pretty_print_to_string(doc, width)
        .split('\n')
        .map(|s| s.to_owned())
        .collect::<Vec<_>>();
    compare_lines(
        &format!("IN PRETTY PRINTING WITH WIDTH {}", width),
        oracle_result.clone(),
        lines.join("\n"),
        contents.as_ref().map(|s| s.as_str()),
    );
    for path in all_paths(doc) {
        let (lines_above, mut lines_below) = print_above_and_below(doc, width, &path);
        let mut lines = lines_above;
        lines.append(&mut lines_below);
        compare_lines(
            &format!("IN PRETTY PRINTING WITH WIDTH {} AT PATH {:?}", width, path),
            oracle_result.clone(),
            lines.join("\n"),
            contents.as_ref().map(|s| s.as_str()),
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
        expected_lines_below.join("\n"),
        lines_below.join("\n"),
        None,
    );
    compare_lines(
        &format!("IN UPWARD PRINTING AT PATH {:?}", path),
        expected_lines_above.join("\n"),
        lines_above.join("\n"),
        None,
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
        expected_lines.join("\n"),
        lines.join("\n"),
        None,
    );
}
