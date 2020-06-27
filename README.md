# partial-pretty-printer

A peephole-efficient pretty printer library in Rust.

------

This is a pretty printer library. You say how a document should be printed (including newlines
options, indentation, and coloring), and it prints it for you. It caters for displaying a document
that is being edited.

The Partial Pretty Printer is:

- **For tree-shaped documents:** You use it to display an AST, or a JSON value, or the like. It does
  not take text as an input. You don't feed it your source code; you feed it your AST and it returns
  formatted source code to you. Its tree-shaped input is called the _document_. Its output is called
  the _rendering_ of that document.
- **Peephole-efficient:** You can use it to display just part of a document (in practice, the part
  around the cursor in an editor). When the document is first loaded, the PPP must do work
  proportional to the size of the document. However, after that, it never has to traverse the entire
  document again (unless you ask it to print the whole thing at once). For example, if you make an
  edit to the document that changes the indent of every line of the rendering, PPP will
  (approximately) only do work proportional to the size of the portion of the document being
  rendered.

It is made for [Synless](https://github.com/justinpombrio/synless), though it aims to be reasonably
general-purpose.
