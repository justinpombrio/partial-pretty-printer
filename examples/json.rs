use partial_pretty_printer::json_notation::{json_list, json_number};
use partial_pretty_printer::{pretty_print, Doc};

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

fn json_long_list_example() {
    let num_elems = 1000;
    let numbers = (0..num_elems).map(|n| json_number(n)).collect::<Vec<_>>();
    let list = json_list(numbers);

    for _i in 0..100 {
        let lines = print_region(&list, 80, &[num_elems / 2], 60);
        assert_eq!(lines.len(), 60);
    }
}

fn main() {
    json_long_list_example();
}
