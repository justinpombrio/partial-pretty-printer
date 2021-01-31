# partial-pretty-printer

A peephole-efficient pretty printer library in Rust.

------

This is a pretty printer library. You say how a document should be printed---including newline
options, indentation, and coloring---and it prints it for you.

The Partial Pretty Printer is:

- **For tree-shaped documents:** You use it to display an AST, or a JSON object, or the like. You
  don't feed it your source code. Instead, you feed it your AST and it returns formatted source code
  to you. Its tree-shaped input is called the _document_.
- **Peephole-efficient:** You can use it to display just part of a document (in practice, the part
  around the cursor in an editor). If you ask it to print 100 lines in the middle of a 100,000 line
  document, it can (typically) do that in ~100 units of work, rather than ~50,000 units of work.

It is made for [Synless](https://github.com/justinpombrio/synless), though it aims to be
general-purpose.

## Non-features

### Align

`Align` is not supported.  Say you define 2 possible layouts for a list. It can choose to align all
its elements to the opening `[`, like this:

```
let list = &[item1.foo(arg, arg, arg, arg),     |
             item2.bar(arg, arg, arg, arg)];    |
```

Or if that's too wide to fit in the screen width (marked by `|`), it can choose to split across
multiple lines, using a constant indentation:

```
let very_very_very_long_list_variable_name = &[ |
    item1.foo(arg, arg, arg, arg),              |
    item2.bar(arg, arg, arg, arg),              |
];                                              |
```

The problem is that this pretty printer would choose the aligned layout whenever it's possible, even
when it's not practical:

```
let somewhat_long_list_variable_name = &[item1  |
                                         .foo(  |
                                          arg,  |
                                          arg,  |
                                          arg,  |
                                          arg   |
                                         ),     |
                                         item2  |
                                         .bar(  |
                                          arg,  |
                                          arg,  |
                                          arg,  |
                                          arg   |
                                         )];    |
```

Also, it would be tricky to implement partial pretty printing that supports this sort of alignment. 
