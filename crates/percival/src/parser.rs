//! Parser definitions and error recovery for Percival.

use chumsky::prelude::*;

use crate::ast::{Aggregate, Clause, Fact, Import, Literal, Program, Rule, Value};

/// Constructs a parser combinator for the Percival language.
pub fn parser() -> BoxedParser<'static, char, Program, Simple<char>> {
    let id = text::ident().labelled("ident");

    let comments = {
        let single_line = just("//").then_ignore(take_until(text::newline()));
        let multi_line = just("/*").then_ignore(take_until(just("*/")));
        single_line
            .or(multi_line)
            .padded()
            .repeated()
            .map_err(|e: Simple<char>| Simple::custom(e.span(), "Not a valid comment"))
    };

    let number = {
        // We only support decimal literals for now, not the full scope of numbers.
        // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Lexical_grammar#numeric_literals
        let digit = one_of("0123456789");
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
                one_of("\"\\/bfnrt")
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

    let boolean = choice((
        text::keyword("true").to(true),
        text::keyword("false").to(false),
    ));

    let literal = choice((
        number.map(Literal::Number),
        string.clone().map(Literal::String),
        boolean.map(Literal::Boolean),
    ))
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

        choice((
            literal.map(Value::Literal),
            expr.map(Value::Expr),
            aggregate.map(Value::Aggregate),
            id.map(Value::Id),
        ))
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

    let clause = choice((
        fact.clone().map(Clause::Fact),
        expr.map(Clause::Expr),
        binding.map(|(name, value)| Clause::Binding(name, value)),
    ))
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
            just(":-")
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
                .or(just('.').padded().padded_by(comments).to(Vec::new())),
        )
        .map(|(goal, clauses)| Rule { goal, clauses })
        .labelled("rule");

    let import = text::keyword("import")
        .ignore_then(text::ident().padded().padded_by(comments))
        .then_ignore(text::keyword("from"))
        .then(string.padded().padded_by(comments))
        .map(|(name, uri)| Import { name, uri });

    enum Entry {
        Rule(Rule),
        Import(Import),
    }

    let program = choice((rule.map(Entry::Rule), import.map(Entry::Import)))
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
