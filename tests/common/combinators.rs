use partial_pretty_printer::Notation;

pub fn flat(notation: Notation) -> Notation {
    Notation::Flat(Box::new(notation))
}

pub fn newline() -> Notation {
    Notation::Newline
}

pub fn lit(s: &str) -> Notation {
    Notation::Literal(s.to_string())
}

pub fn indent(i: usize, notation: Notation) -> Notation {
    Notation::Indent(i, Box::new(notation))
}

pub fn nest(i: usize, notation: Notation) -> Notation {
    Notation::Indent(
        i,
        Box::new(Notation::Concat(
            Box::new(Notation::Newline),
            Box::new(notation),
        )),
    )
}

pub fn align(notation: Notation) -> Notation {
    Notation::Align(Box::new(notation))
}
