#[derive(Debug, Clone)]
pub enum MeasuredNotation {
    Empty(Pos),
    Literal(Pos, String),
    Newline(Pos),
    Indent(Span, usize, Box<MeasuredNotation>),
    Flat(Span, Box<MeasuredNotation>),
    Concat(
        Span,
        Box<MeasuredNotation>,
        Box<MeasuredNotation>,
        KnownLineLengths,
    ),
    Choice(Span, ChoiceInner),
}

#[derive(Debug, Clone)]
pub struct ChoiceInner {
    left: Box<MeasuredNotation>,
    left_shapes: Shapes,
    right: Box<MeasuredNotation>,
    right_shapes: Shapes,
}
