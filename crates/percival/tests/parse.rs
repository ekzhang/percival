use chumsky::prelude::*;
use maplit::btreemap;

use percival::{
    ast::{Aggregate, Clause, Fact, Import, Literal, Program, Rule, Value},
    errors::format_errors,
    parser::parser,
};

#[test]
fn parse_single_rule() {
    let parser = parser();
    let result = parser.parse("tc(x, y) :- tc(x, y: z), edge(x: z, y).");
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Program {
            rules: vec![Rule {
                goal: Fact {
                    name: "tc".into(),
                    props: btreemap! {
                        "x".into() => Value::Id("x".into()),
                        "y".into() => Value::Id("y".into()),
                    },
                },
                clauses: vec![
                    Clause::Fact(Fact {
                        name: "tc".into(),
                        props: btreemap! {
                            "x".into() => Value::Id("x".into()),
                            "y".into() => Value::Id("z".into()),
                        },
                    }),
                    Clause::Fact(Fact {
                        name: "edge".into(),
                        props: btreemap! {
                            "x".into() => Value::Id("z".into()),
                            "y".into() => Value::Id("y".into()),
                        },
                    }),
                ],
            }],
            imports: vec![],
        },
    );
}

#[test]
fn parse_no_clauses() {
    let parser = parser();
    let result = parser.parse("person(name, age).");
    assert!(result.is_ok());
    let result = parser.parse("person(name, age) :-.");
    assert!(result.is_err());
}

#[test]
fn parse_literal() {
    let parser = parser();
    let result = parser.parse("person(name: \"eric\\t\", age: 20, weight: 1.234e+2).");
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Program {
            rules: vec![Rule {
                goal: Fact {
                    name: "person".into(),
                    props: btreemap! {
                        "name".into() => Value::Literal(Literal::String("eric\\t".into())),
                        "age".into() => Value::Literal(Literal::Number("20".into())),
                        "weight".into() => Value::Literal(Literal::Number("1.234e+2".into())),
                    },
                },
                clauses: vec![],
            }],
            imports: vec![],
        },
    );
}

#[test]
fn parse_err() {
    let parser = parser();
    let text = "tc(x, y) :- f(.
tc(z) :- tc(z, &).";
    let (_, errors) = parser.parse_recovery(text);
    assert!(errors.len() == 1);
    let message = format_errors(text, errors);
    assert!(message.contains("Unexpected token in input, expected "));
}

#[test]
fn parse_reserved_word() {
    let parser = parser();
    let text = "bad(x: continue).";
    let (_, errors) = parser.parse_recovery(text);
    assert!(errors.len() == 1);
    let message = format_errors(text, errors);
    assert!(message.contains("Cannot use reserved word as a variable binding"));

    let text = "bad(x: __percival_first_iteration).";
    let (_, errors) = parser.parse_recovery(text);
    assert!(errors.len() == 1);

    // It is okay to use a reserved word as a field name, just not a variable.
    let text = "ok(continue: x).";
    let (_, errors) = parser.parse_recovery(text);
    assert!(errors.is_empty());
}

#[test]
fn parse_js_expr() {
    let parser = parser();
    let result = parser.parse("ok(x: `2 * num`) :- input(x: num), `num < 10`.");
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Program {
            rules: vec![Rule {
                goal: Fact {
                    name: "ok".into(),
                    props: btreemap! {
                        "x".into() => Value::Expr("2 * num".into()),
                    },
                },
                clauses: vec![
                    Clause::Fact(Fact {
                        name: "input".into(),
                        props: btreemap! {
                            "x".into() => Value::Id("num".into()),
                        },
                    }),
                    Clause::Expr("num < 10".into()),
                ],
            }],
            imports: vec![],
        },
    );
}

#[test]
fn parse_comments() {
    let parser = parser();
    let result = parser.parse(
        "
hello(x: /* asdf */ 3) :-
    // a comment!
    world(k) /* another comment */,
    `k < 10`.
"
        .trim(),
    );
    assert!(result.is_ok());
}

#[test]
fn parse_whitespace() {
    let parser = parser();
    let result = parser.parse("\n\n\n");
    assert!(result.is_ok());
}

#[test]
fn parse_trailing_eof_comment() {
    // This example technically invalid under our grammar; however, most
    // users would usually want to allow for comments at the end of a cell.
    // To fix this, Percival programs should be terminated by newlines.
    let parser = parser();
    let result = parser.parse("// this comment has no trailing newline");
    assert!(result.is_err());

    let result = parser.parse("// this comment has a trailing newline\n");
    assert!(result.is_ok());
}

#[test]
fn parse_empty() {
    let parser = parser();
    let result = parser.parse("any() :- ok().");
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Program {
            rules: vec![Rule {
                goal: Fact {
                    name: "any".into(),
                    props: btreemap! {},
                },
                clauses: vec![Clause::Fact(Fact {
                    name: "ok".into(),
                    props: btreemap! {},
                })],
            }],
            imports: vec![],
        },
    );
}

#[test]
fn parse_imports() {
    let parser = parser();
    let result = parser.parse(
        r#"
import hello from "https://example.com/hello.json"
import barley from "npm://vega-datasets/data/barley.json"
import football from "gh://vega/vega-datasets@next/data/football.json"
"#
        .trim(),
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Program {
            rules: vec![],
            imports: vec![
                Import {
                    name: "hello".into(),
                    uri: "https://example.com/hello.json".into()
                },
                Import {
                    name: "barley".into(),
                    uri: "npm://vega-datasets/data/barley.json".into()
                },
                Import {
                    name: "football".into(),
                    uri: "gh://vega/vega-datasets@next/data/football.json".into()
                },
            ],
        },
    );
}

#[test]
fn parse_boolean() {
    let parser = parser();
    let result = parser.parse("hello(x: true, y: false).");
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Program {
            rules: vec![Rule {
                goal: Fact {
                    name: "hello".into(),
                    props: btreemap! {
                        "x".into() => Value::Literal(Literal::Boolean(true)),
                        "y".into() => Value::Literal(Literal::Boolean(false)),
                    },
                },
                clauses: vec![],
            }],
            imports: vec![],
        },
    );
}

#[test]
fn parse_import_edge_cases() {
    let parser = parser();
    let result = parser.parse("importhello from \"gh://hello\"");
    assert!(result.is_err());

    let result = parser.parse("importa(value: 3).");
    assert!(result.is_ok());
}

#[test]
fn parse_binding() {
    let parser = parser();
    let result = parser.parse(
        r#"
ok(val) :-
    attempt(x),
    val = `3 * x`.
"#,
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Program {
            rules: vec![Rule {
                goal: Fact {
                    name: "ok".into(),
                    props: btreemap! {
                        "val".into() => Value::Id("val".into()),
                    },
                },
                clauses: vec![
                    Clause::Fact(Fact {
                        name: "attempt".into(),
                        props: btreemap! {
                            "x".into() => Value::Id("x".into()),
                        },
                    }),
                    Clause::Binding("val".into(), Value::Expr("3 * x".into())),
                ],
            }],
            imports: vec![],
        },
    );
}

#[test]
fn parse_aggregate() {
    let parser = parser();
    let result = parser.parse(
        r#"
ok(value) :-
    year(year),
    value = mean[mpg] {
        cars(Year: year, mpg)
    }.
"#,
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Program {
            rules: vec![Rule {
                goal: Fact {
                    name: "ok".into(),
                    props: btreemap! {
                        "value".into() => Value::Id("value".into()),
                    },
                },
                clauses: vec![
                    Clause::Fact(Fact {
                        name: "year".into(),
                        props: btreemap! {
                            "year".into() => Value::Id("year".into()),
                        },
                    }),
                    Clause::Binding(
                        "value".into(),
                        Value::Aggregate(Aggregate {
                            operator: "mean".into(),
                            value: Box::new(Value::Id("mpg".into())),
                            subquery: vec![Clause::Fact(Fact {
                                name: "cars".into(),
                                props: btreemap! {
                                    "Year".into() => Value::Id("year".into()),
                                    "mpg".into() => Value::Id("mpg".into()),
                                },
                            }),],
                        }),
                    ),
                ],
            }],
            imports: vec![],
        },
    );
}
