mod common;

use common::{assert_pp, Tree};
use partial_pretty_printer::notation_constructors::{child, flat, lit};

// (1..5).each do |i| puts i end
//
// (1..5).each do |i|
//     puts i
// end
//
// -- Dissalow this layout?
// (1..5).each
//     do |i|
//         puts i
//     end
//
// (1..5)
//     .each do |i|
//         puts i
//     end
//
// object.method argument
// object.method
//     argument
// object
//     .method argument
// object
//     .method
//         argument

fn method(obj: Tree, method: &str, arg: Tree) -> Tree {
    let single = lit(".") + child(1) + lit(" ") + child(2);
    let two_lines = lit(".") + child(1) + lit(" ") + child(2);
    let multi = lit(".") + child(1) + (4 >> child(2));
    let notation = child(0) + (single | (4 >> (two_lines | multi)));
    Tree::new_branch(notation, vec![obj, var(method), arg])
}

fn ruby_do(var_name: &str, body: Tree) -> Tree {
    let single = lit("do |") + child(0) + lit("| ") + flat(child(1)) + lit(" end");
    let multi = lit("do |") + child(0) + lit("|") + (4 >> child(1)) ^ lit("end");
    let notation = single | multi;
    Tree::new_branch(notation, vec![var(var_name), body])
}

fn var(var_name: &str) -> Tree {
    Tree::new_leaf(lit(var_name))
}

#[test]
fn ruby_loop() {
    let doc = method(var("(1..5)"), "each", ruby_do("i", var("puts i")));
    assert_pp(
        &doc,
        30,
        //  0    5   10   15   20   25   30   35   40
        &[
            // force rustfmt
            "(1..5).each do |i| puts i end",
        ],
    );
    assert_pp(
        &doc,
        20,
        //  0    5   10   15   20   25   30   35   40
        &[
            // force rustfmt
            "(1..5).each do |i|",
            "    puts i",
            "end",
        ],
    );
    assert_pp(
        &doc,
        15,
        //  0    5   10   15   20   25   30   35   40
        &[
            // force rustfmt
            "(1..5)",
            "    .each",
            "        do |i|",
            "            puts i",
            "        end",
        ],
    );
}
