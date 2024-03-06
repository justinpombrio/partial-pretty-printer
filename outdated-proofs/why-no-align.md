### Align

`Align` is not supported.  Say you define 2 possible layouts for a list. It can
choose to align all its elements to the opening `[`, like this:

```
let list = &[item1.foo(arg, arg, arg, arg),     |
             item2.bar(arg, arg, arg, arg)];    |
```

Or if that's too wide to fit in the screen width (marked by `|`), it can choose
to split across multiple lines, using a constant indentation:

```
let very_very_very_long_list_variable_name = &[ |
    item1.foo(arg, arg, arg, arg),              |
    item2.bar(arg, arg, arg, arg),              |
];                                              |
```

The problem is that this pretty printer would choose the aligned layout whenever
it's possible, even when it's not practical:

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

Also, it would be tricky to implement partial pretty printing that supports this
sort of alignment. 
