# partial-pretty-printer

This is a pretty printing library for formatting source code in any language.

You provide declarative rules for how to display each sort of node in a
document, including line break options, indentation, and coloring. The Partial
Pretty Printer prints the document, picking a good layout that fits in your
desired line width if possible.

The Partial Pretty Printer is:

- **Peephole-efficient:** It lets you display just _part of_ a document. If you
  ask it to print 50 lines in the middle of a 100,000 line document, it can
  typically do that in ~50 units of work, rather than ~50,000 units of work.
  This library was designed to support the
  [Synless](https://github.com/justinpombrio/synless) editor, which needs to
  re-render a screenfull of document on every keystroke, even if the screen
  width has changed.

- **Fast:** Even when printing every line of a document, it can e.g. print
  ~400,000 lines of JSON per second (excluding terminal IO time).

- **Expressive:** The combinators that it uses are a
  [variation](https://justinpombrio.net/2024/02/23/a-twist-on-Wadlers-printer.html)
  on Wadler's
  [Prettier Printer](http://homepages.inf.ed.ac.uk/wadler/papers/prettier/prettier.pdf).
  It's therefore about as expressive as other libraries based on Wadler's
  algorithm, like [JS Prettier](https://prettier.io).

- **Flexible:** It can apply user-defined styles to the text and use
  user-defined predicates (like "is this node a comment") to choose between
  layouts.

- **Also secretly a terminal UI library:** See the (optional) `pane` module for
  how to arrange multiple documents in a window.

There's a demo JSON formatter implemented using the Partial Pretty Printer,
which you can try out with:

> cargo run --release --example json -- examples/pokemon.json --width 100
