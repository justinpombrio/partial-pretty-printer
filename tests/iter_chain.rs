mod common;

use common::{assert_pp, child, flat, lit, Tree};

fn method(obj: Tree, method: &str, arg: Tree) -> Tree {
    // foobaxxle.bar(arg)
    //
    // -- Disallowing this layout:
    // foobaxxle.bar(
    //     arg
    // )
    //
    // foobaxxle
    //     .bar(arg)
    //
    // foobaxxle
    //     .bar(
    //         arg
    //      )

    let single = lit(".") + lit(method) + lit("(") + flat(child(1).clone()) + lit(")");
    let two_lines = lit(".") + lit(method) + lit("(") + flat(child(1).clone()) + lit(")");
    let multi = lit(".") + lit(method) + lit("(") + (4 >> child(1)) ^ lit(")");
    let notation = child(0) + (single | (4 >> (two_lines | multi)));
    Tree::new_branch(notation, vec![obj, arg])
}

fn closure(var: &str, body: Tree) -> Tree {
    let single = lit("|") + lit(var) + lit("| { ") + child(0) + lit(" }");
    let multi = lit("|") + lit(var) + lit("| {") + (4 >> child(0)) ^ lit("}");
    let notation = single | multi;
    Tree::new_branch(notation, vec![body])
}

fn times(arg1: Tree, arg2: Tree) -> Tree {
    let notation = child(0) + lit(" * ") + child(1);
    Tree::new_branch(notation, vec![arg1, arg2])
}

fn var(var: &str) -> Tree {
    Tree::new_leaf(lit(var))
}

#[test]
fn iter_chain_iter_map_collect() {
    let doc = var("some_vec");
    let doc = method(doc, "iter", var(""));
    let doc = method(doc, "map", closure("elem", times(var("elem"), var("elem"))));
    let doc = method(doc, "collect", var(""));

    assert_pp(
        &doc,
        80,
        //0    5   10   15   20   25   30   35   40   45   50   55   60
        &["some_vec.iter().map(|elem| { elem * elem }).collect()"],
    );
    assert_pp(
        &doc,
        50,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            // force rustfmt
            "some_vec.iter().map(|elem| { elem * elem })",
            "    .collect()",
        ],
    );
    assert_pp(
        &doc,
        40,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            // force rustfmt
            "some_vec.iter()",
            "    .map(|elem| { elem * elem })",
            "    .collect()",
        ],
    );
    assert_pp(
        &doc,
        30,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            // force rustfmt
            "some_vec.iter()",
            "    .map(",
            "        |elem| { elem * elem }",
            "    ).collect()",
        ],
    );
}

#[test]
fn iter_chain_long_method_body() {
    let doc = var("some_vec");
    let doc = method(doc, "map", closure("elem", times(var("elem"), var("elem"))));

    assert_pp(
        &doc,
        31,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            // force rustfmt
            "some_vec",
            "    .map(",
            "        |elem| { elem * elem }",
            "    )",
        ],
    );
}

#[test]
fn iter_chain_long_methods() {
    let doc = var("some_vec");
    let doc = method(doc, "call_the_map_method", closure("elem", var("elem")));
    let doc = method(doc, "call_the_map_method", closure("elem", var("elem")));

    assert_pp(
        &doc,
        41,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            // force rustfmt
            "some_vec",
            "    .call_the_map_method(|elem| { elem })",
            "    .call_the_map_method(|elem| { elem })",
        ],
    );
    assert_pp(
        &doc,
        35,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            // force rustfmt
            "some_vec",
            "    .call_the_map_method(",
            "        |elem| { elem }",
            "    )",
            "    .call_the_map_method(",
            "        |elem| { elem }",
            "    )",
        ],
    );
}
