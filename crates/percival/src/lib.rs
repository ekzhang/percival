//! Core compiler code for the Percival language.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::HashMap;

use chumsky::prelude::*;

/// A program translation unit in the Percival language.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    /// Rules that make up the program.
    pub rules: Vec<Rule>,
}

/// Represents a single Horn clause.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    /// Head or implicand of the Horn clause.
    pub head: Fact,
    /// Tail or conditional assumptions of the Horn clause.
    pub clauses: Vec<Fact>,
}

/// Literal part of a Horn clause, written in terms of relations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fact {
    /// Name of the relation being referenced.
    pub name: String,
    /// Named properties of the relation.
    pub props: HashMap<String, Value>,
}

/// A bound or unbound value assigned to part of a relation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    /// A simple identifier, which can be either bound or unbound.
    Id(String),
    // TODO: Expr(Expr),
}

/// Constructs a parser combinator for the Percival language.
pub fn parser() -> impl Parser<char, Program, Error = Simple<char>> {
    let id = text::ident().labelled("ident");

    let value = id.map(Value::Id).labelled("value");

    let prop = id
        .then(just(':').padded().ignore_then(value).or_not())
        .map(|(id, value)| (id.clone(), value.unwrap_or_else(|| Value::Id(id))))
        .labelled("prop");

    let fact = text::ident()
        .then(prop.padded().separated_by(just(',')).delimited_by('(', ')'))
        .map(|(name, props)| Fact {
            name,
            props: props.into_iter().collect(),
        })
        .labelled("fact");

    let rule = fact
        .then_ignore(seq(":-".chars()).padded())
        .then(fact.padded().separated_by(just(',')))
        .then_ignore(just('.'))
        .map(|(head, clauses)| Rule { head, clauses })
        .labelled("rule");

    rule.padded()
        .repeated()
        .map(|rules| Program { rules })
        .then_ignore(end())
}

#[cfg(test)]
mod tests {
    use maplit::hashmap;

    use super::*;

    #[test]
    fn parse_single_rule() {
        let parser = parser();
        let result = parser.parse("tc(x, y) :- tc(x, y: z), edge(x: z, y).");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Program {
                rules: vec![Rule {
                    head: Fact {
                        name: "tc".into(),
                        props: hashmap! {
                            "x".into() => Value::Id("x".into()),
                            "y".into() => Value::Id("y".into()),
                        }
                    },
                    clauses: vec![
                        Fact {
                            name: "tc".into(),
                            props: hashmap! {
                                "x".into() => Value::Id("x".into()),
                                "y".into() => Value::Id("z".into()),
                            }
                        },
                        Fact {
                            name: "edge".into(),
                            props: hashmap! {
                                "x".into() => Value::Id("z".into()),
                                "y".into() => Value::Id("y".into()),
                            }
                        },
                    ],
                }],
            },
        );
    }
}
