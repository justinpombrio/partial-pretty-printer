# partial-pretty-printer

A peephole-efficient pretty printer library in Rust.

------

This is a pretty printer library. You say how a document should be printed---including newlines
options, indentation, and coloring---and it prints it for you.

The Partial Pretty Printer is:

- **For tree-shaped documents:** You use it to display an AST, or a JSON value, or the like. You
  don't feed it your source code. Instead, you feed it your AST and it returns formatted source code
  to you. Its tree-shaped input is called the _document_. Its output is called the _rendering_ of
  that document.
- **Peephole-efficient:** You can use it to display just part of a document (in practice, the part
  around the cursor in an editor). When the document is first loaded, the PPP must do work
  proportional to the size of the document. However, after that, it never has to traverse the entire
  document again (unless you ask it to print the whole thing). For example, if you make an edit to
  the document, PPP will (more or less) only do work proportional to the size of the portion of the
  document being rendered. It does _not_ need to do work proportional to the size of the document
  whose rendering is effected by the edit.

It is made for [Synless](https://github.com/justinpombrio/synless), though it aims to be reasonably
general-purpose.
