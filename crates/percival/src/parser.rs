//! Parser definitions and error recovery for Percival.

use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::prelude::*;

use crate::ast::{Aggregate, Clause, Fact, Import, Literal, Program, Rule, Value};

/// Constructs a parser combinator for the Percival language.
pub fn parser() -> BoxedParser<'static, char, Program, Simple<char>> {
    let id = text::ident().labelled("ident");

    let keyword = |s: &'static str| {
        text::ident::<char, Simple<_>>().validate(move |ident, span, emit| {
            if ident != s {
                emit(Simple::custom(span, "not an import statement"))
            }
            ident
        })
    };

    let comments = {
        let single_line = just('/')
            .then_ignore(just('/'))
            .then_ignore(take_until(text::newline()));
        let multi_line = just('/')
            .then_ignore(just('*'))
            .then_ignore(take_until(just('*').then(just('/'))));
        single_line
            .or(multi_line)
            .padded()
            .repeated()
            .map_err(|e: Simple<char>| Simple::custom(e.span(), "Not a valid comment"))
    };

    let number = {
        // We only support decimal literals for now, not the full scope of numbers.
        // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Lexical_grammar#numeric_literals
        let digit = one_of('0'..='9');
        let digits = digit.then_ignore(just('_').or_not()).repeated().at_least(1);
        let sign = just('-')
            .or(just('+'))
            .map(|c| c.to_string())
            .or_not()
            .map(Option::unwrap_or_default);
        let integer = sign.chain(digits.clone());

        let fraction = just('.')
            .chain(digits.clone())
            .or_not()
            .map(Option::unwrap_or_default);
        let exponent = just('e')
            .or(just('E'))
            .chain(sign.chain(digits))
            .or_not()
            .map(Option::unwrap_or_default);
        integer
            .chain::<char, _, _>(fraction)
            .chain::<char, _, _>(exponent)
            .collect()
    };

    let string = {
        let normal_char = filter(|&c: &char| c != '"' && c != '\\' && !c.is_control());
        let hex_digit = filter(|&c: &char| c.is_ascii_hexdigit());
        let control_char = just('\\')
            .chain(
                one_of("\"\\/bfnrt".chars())
                    .map(|c| vec![c])
                    .or(just('u').chain(hex_digit.repeated().at_least(4).at_most(4))),
            )
            .collect::<String>();
        let chars = normal_char
            .map(|c| c.to_string())
            .or(control_char)
            .repeated()
            .collect();
        just('"').ignore_then(chars).then_ignore(just('"'))
    };

    let boolean = keyword("true")
        .map(|_| true)
        .or(keyword("false").map(|_| false));

    let literal = number
        .map(Literal::Number)
        .or(string.clone().map(Literal::String))
        .or(boolean.map(Literal::Boolean))
        .labelled("literal");

    let expr = just('`')
        .ignore_then(take_until(just('`')))
        .map(|(s, _)| s)
        .collect()
        .labelled("expr");

    // Declared here so that we can use it for aggregate subqueries.
    let mut clauses = Recursive::<_, Vec<Clause>, Simple<char>>::declare();

    let value = recursive(|value| {
        let aggregate = text::ident()
            .then(
                value
                    .padded()
                    .padded_by(comments)
                    .delimited_by('[', ']')
                    .padded()
                    .padded_by(comments),
            )
            .then(
                clauses
                    .clone()
                    .padded()
                    .padded_by(comments)
                    .delimited_by('{', '}'),
            )
            .map(|((operator, value), subquery)| Aggregate {
                operator,
                value: Box::new(value),
                subquery,
            });

        literal
            .map(Value::Literal)
            .or(expr.map(Value::Expr))
            .or(aggregate.map(Value::Aggregate))
            .or(id.map(Value::Id))
            .labelled("value")
    });

    let prop = id
        .then(
            just(':')
                .padded()
                .padded_by(comments)
                .ignore_then(value.clone())
                .or_not(),
        )
        .try_map(|(id, value), span| {
            let value = value.unwrap_or_else(|| Value::Id(id.clone()));
            match &value {
                Value::Id(id) if is_reserved_word(id) => Err(Simple::custom(
                    span,
                    "Cannot use reserved word as a variable binding",
                )),
                _ => Ok((id, value)),
            }
        })
        .labelled("prop");

    let fact = text::ident()
        .then(
            prop.padded()
                .padded_by(comments)
                .separated_by(just(','))
                .delimited_by('(', ')'),
        )
        .map(|(name, props)| Fact {
            name,
            props: props.into_iter().collect(),
        })
        .labelled("fact")
        .boxed(); // boxed to avoid rustc recursion limit

    let binding = id
        .then_ignore(just('=').padded().padded_by(comments))
        .then(value)
        .labelled("binding");

    let clause = fact
        .clone()
        .map(Clause::Fact)
        .or(expr.map(Clause::Expr))
        .or(binding.map(|(name, value)| Clause::Binding(name, value)))
        .labelled("clause");

    clauses.define(
        clause
            .clone()
            .padded()
            .padded_by(comments)
            .separated_by(just(',')),
    );

    let rule = fact
        .then(
            seq(":-".chars())
                .padded()
                .padded_by(comments)
                .ignore_then(clauses)
                .then_ignore(just('.'))
                .try_map(|clauses, span| {
                    if clauses.is_empty() {
                        Err(Simple::custom(span, "Rule needs at least one clause"))
                    } else {
                        Ok(clauses)
                    }
                })
                .or(just('.').padded().padded_by(comments).map(|_| Vec::new())),
        )
        .map(|(goal, clauses)| Rule { goal, clauses })
        .labelled("rule");

    let import = keyword("import")
        .ignore_then(text::ident().padded().padded_by(comments))
        .then_ignore(keyword("from"))
        .then(string.padded().padded_by(comments))
        .map(|(name, uri)| Import { name, uri });

    enum Entry {
        Rule(Rule),
        Import(Import),
    }

    let program = rule
        .map(Entry::Rule)
        .or(import.map(Entry::Import))
        .padded()
        .padded_by(comments)
        .repeated()
        .map(|entries| {
            let mut rules = Vec::new();
            let mut imports = Vec::new();
            for entry in entries {
                match entry {
                    Entry::Rule(rule) => rules.push(rule),
                    Entry::Import(import) => imports.push(import),
                }
            }
            Program { rules, imports }
        });

    program
        .padded()
        .padded_by(comments)
        .then_ignore(end())
        .boxed()
}

/// Checks if a token is reserved, which cannot be used as an identifier.
///
/// See [https://262.ecma-international.org/6.0/#sec-reserved-words] for
/// JavaScript reserved words. The rest of the tokens listed here are prohibited
/// for internal reasons, or because they mean other things in the context of
/// the Percival language.
fn is_reserved_word(name: &str) -> bool {
    match name {
        // Reserved words in the ECMAScript standard
        "break" | "do" | "in" | "typeof" | "case" | "else" | "instanceof" | "var" | "catch"
        | "export" | "new" | "void" | "class" | "extends" | "return" | "while" | "const"
        | "finally" | "super" | "with" | "continue" | "for" | "switch" | "yield" | "debugger"
        | "function" | "this" | "default" | "if" | "throw" | "delete" | "import" | "try"
        | "enum" | "await" | "implements" | "package" | "protected" | "interface" | "private"
        | "public" | "null" | "true" | "false" | "let" => true,

        // Internal names, reserved to avoid conflicts
        _ => name.starts_with("__percival"),
    }
}

/// Format parser errors into a human-readable message.
pub fn format_errors(src: &str, errors: Vec<Simple<char>>) -> String {
    let mut reports = vec![];

    for e in errors {
        let e = e.map(|tok| tok.to_string());
        let report = Report::build(ReportKind::Error, (), e.span().start);

        let report = match e.reason() {
            chumsky::error::SimpleReason::Unclosed { span, delimiter } => report
                .with_message(format!(
                    "Unclosed delimiter {}",
                    delimiter.fg(Color::Yellow)
                ))
                .with_label(
                    Label::new(span.clone())
                        .with_message(format!(
                            "Unclosed delimiter {}",
                            delimiter.fg(Color::Yellow)
                        ))
                        .with_color(Color::Yellow),
                )
                .with_label(
                    Label::new(e.span())
                        .with_message(format!(
                            "Must be closed before this {}",
                            e.found()
                                .unwrap_or(&"end of file".to_string())
                                .fg(Color::Red)
                        ))
                        .with_color(Color::Red),
                ),
            chumsky::error::SimpleReason::Unexpected => report
                .with_message(format!(
                    "{}, expected {}",
                    if e.found().is_some() {
                        "Unexpected token in input"
                    } else {
                        "Unexpected end of input"
                    },
                    if e.expected().len() == 0 {
                        "end of input".to_string()
                    } else {
                        e.expected()
                            .map(|x| x.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    }
                ))
                .with_label(
                    Label::new(e.span())
                        .with_message(format!(
                            "Unexpected token {}",
                            e.found()
                                .unwrap_or(&"end of file".to_string())
                                .fg(Color::Red)
                        ))
                        .with_color(Color::Red),
                ),
            chumsky::error::SimpleReason::Custom(msg) => report.with_message(msg).with_label(
                Label::new(e.span())
                    .with_message(format!("{}", msg.fg(Color::Red)))
                    .with_color(Color::Red),
            ),
        };

        let mut buf = vec![];
        report.finish().write(Source::from(&src), &mut buf).unwrap();
        reports.push(std::str::from_utf8(&buf[..]).unwrap().to_string());
    }

    reports.join("\n")
}

#[cfg(test)]
mod tests {
    use chumsky::prelude::*;
    use maplit::btreemap;

    use super::{format_errors, parser};
    use crate::ast::{Aggregate, Clause, Fact, Import, Literal, Program, Rule, Value};

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
        assert!(message.contains("Unexpected token in input, expected )"));
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
}
