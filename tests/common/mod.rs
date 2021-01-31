use partial_pretty_printer::{
    notation_constructors::lit, pretty_print, pretty_print_to_string, Notation, PrettyDoc,
    PrettyDocContents, Style, Width,
};

#[allow(unused)]
pub fn punct(s: &'static str) -> Notation {
    lit(s, Style::plain())
}

#[derive(Debug, Clone)]
pub struct SimpleDoc(pub Notation);

impl PrettyDoc for SimpleDoc {
    type Id = usize;

    fn id(&self) -> usize {
        0
    }

    fn notation(&self) -> &Notation {
        &self.0
    }

    fn contents(&self) -> PrettyDocContents<SimpleDoc> {
        PrettyDocContents::Children(&[])
    }
}

fn compare_lines(message: &str, actual: &[String], expected: &[&str]) {
    if actual != expected {
        eprintln!(
            "{}\nEXPECTED:\n{}\nACTUAL:\n{}\n=========",
            message,
            expected.join("\n"),
            actual.join("\n"),
        );
        assert_eq!(actual, expected);
    }
}

fn print_above_and_below<D: PrettyDoc>(
    doc: &D,
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
pub fn all_paths<D: PrettyDoc>(doc: &D) -> Vec<Vec<usize>> {
    fn recur<D: PrettyDoc>(doc: &D, path: &mut Vec<usize>, paths: &mut Vec<Vec<usize>>) {
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
pub fn print_region<D: PrettyDoc>(
    doc: &D,
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

#[allow(unused)]
#[track_caller]
pub fn assert_pp<D: PrettyDoc>(doc: &D, width: Width, expected_lines: &[&str]) {
    let lines = pretty_print_to_string(doc, width)
        .split('\n')
        .map(|s| s.to_owned())
        .collect::<Vec<_>>();
    compare_lines("IN PRETTY PRINTING", &lines, expected_lines);
    for path in all_paths(doc) {
        let (lines_above, mut lines_below) = print_above_and_below(doc, width, &path);
        let mut lines = lines_above;
        lines.append(&mut lines_below);
        compare_lines(
            &format!("IN PRETTY PRINTING AT PATH {:?}", path),
            &lines,
            expected_lines,
        );
    }
}

#[allow(unused)]
#[track_caller]
pub fn assert_pp_seek<D: PrettyDoc>(
    doc: &D,
    width: Width,
    path: &[usize],
    expected_lines_above: &[&str],
    expected_lines_below: &[&str],
) {
    let (lines_above, lines_below) = print_above_and_below(doc, width, path);
    compare_lines(
        &format!("IN DOWNWARD PRINTING AT PATH {:?}", path),
        &lines_below,
        expected_lines_below,
    );
    compare_lines(
        &format!("IN UPWARD PRINTING AT PATH {:?}", path),
        &lines_above,
        expected_lines_above,
    );
}

#[allow(unused)]
#[track_caller]
pub fn assert_pp_region<D: PrettyDoc>(
    doc: &D,
    width: Width,
    path: &[usize],
    rows: usize,
    expected_lines: &[&str],
) {
    let lines = print_region(doc, width, path, rows);
    compare_lines(
        &format!("IN PRINTING {} ROWS AT PATH {:?}", rows, path),
        &lines,
        expected_lines,
    );
}
