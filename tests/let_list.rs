mod common;

use common::{assert_pp, child, lit, nl, Tree};
use partial_pretty_printer::Notation;

#[test]
#[ignore]
fn let_list() {
    fn list(elements: Vec<Tree>) -> Tree {
        let empty = lit("[]");
        let lone = |elem| lit("[") + elem + lit("]");
        let join = |elem: Notation, accum: Notation| elem + lit(",") + (lit(" ") | nl()) + accum;
        let surround = |accum: Notation| {
            let single = lit("[") + accum.clone() + lit("]");
            let multi = lit("[") + (4 >> accum) ^ lit("]");
            single | multi
        };
        let notation = Notation::repeat(elements.len(), empty, lone, join, surround);
        Tree::new_branch(notation, elements)
    }

    fn make_let(var: &str, defn: Tree) -> Tree {
        let notation = lit("let ") + child(0) + lit(" =") + (lit(" ") | nl()) + child(1) + lit(";");
        Tree::new_branch(notation, vec![Tree::new_leaf(lit(var)), defn])
    }

    // TODO: Add a way to get this to not share lines
    fn phi() -> Tree {
        Tree::new_leaf(lit("1 + sqrt(5)") ^ lit("-----------") ^ lit("     2"))
    }

    fn num(n: &str) -> Tree {
        Tree::new_leaf(lit(n))
    }

    let doc = make_let(
        "best_numbers",
        list(vec![
            num("1025"),
            num("-58"),
            num("33297"),
            phi(),
            num("1.618281828"),
            num("23"),
        ]),
    );

    assert_pp(&doc, 80, &[""]);
}
