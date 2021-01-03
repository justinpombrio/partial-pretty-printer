use partial_pretty_printer::{pretty_print, Doc};

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

fn print_above_and_below<D: Doc>(
    doc: &D,
    width: usize,
    path: &[usize],
) -> (Vec<String>, Vec<String>) {
    let path_iter = path.into_iter().map(|i| *i);
    let (upward_printer, downward_printer) = pretty_print(doc, width, path_iter);
    let mut lines_above = upward_printer
        .map(|(spaces, line)| format!("{:spaces$}{}", "", line, spaces = spaces))
        .collect::<Vec<_>>();
    lines_above.reverse();
    let lines_below = downward_printer
        .map(|(spaces, line)| format!("{:spaces$}{}", "", line, spaces = spaces))
        .collect::<Vec<_>>();
    (lines_above, lines_below)
}

fn all_paths<D: Doc>(doc: &D) -> Vec<Vec<usize>> {
    fn recur<D: Doc>(doc: &D, path: &mut Vec<usize>, paths: &mut Vec<Vec<usize>>) {
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
pub fn print_region<D: Doc>(doc: &D, width: usize, path: &[usize], rows: usize) -> Vec<String> {
    let path_iter = path.into_iter().map(|i| *i);
    let (upward_printer, downward_printer) = pretty_print(doc, width, path_iter);
    let mut lines = upward_printer
        .map(|(spaces, line)| format!("{:spaces$}{}", "", line, spaces = spaces))
        .take(rows / 2)
        .collect::<Vec<_>>();
    lines.reverse();
    let mut lines_below = downward_printer
        .map(|(spaces, line)| format!("{:spaces$}{}", "", line, spaces = spaces))
        .take(rows / 2)
        .collect::<Vec<_>>();
    lines.append(&mut lines_below);
    lines
}

#[track_caller]
pub fn assert_pp<D: Doc>(doc: &D, width: usize, expected_lines: &[&str]) {
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
pub fn assert_pp_seek<D: Doc>(
    doc: &D,
    width: usize,
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

#[test]
fn test_all_paths_fn() {
    use partial_pretty_printer::json_notation::{json_list, json_string};
    let doc = json_list(vec![
        json_list(vec![json_string("0.0"), json_string("0.1")]),
        json_string("1"),
        json_list(vec![
            json_list(vec![json_string("2.0.0")]),
            json_string("2.1"),
        ]),
    ]);
    assert_eq!(
        all_paths(&doc),
        vec![
            vec![],
            vec![0],
            vec![0, 0],
            vec![0, 1],
            vec![1],
            vec![2],
            vec![2, 0],
            vec![2, 0, 0],
            vec![2, 1]
        ]
    );
}
