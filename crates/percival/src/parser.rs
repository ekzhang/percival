//! Parser definitions and error recovery for Percival.

use std::fmt;

use chumsky::{prelude::*, Stream};

use crate::ast::{Aggregate, Clause, Fact, Import, Literal, Program, Rule, Value};

/// A range of character positions in a parser input.
pub type Span = std::ops::Range<usize>;

/// A token emitted from the initial lexical analysis phase.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    /// An identifier, such as for a variable.
    Ident(String),
    /// A numerical constant literal.
    Number(String),
    /// A string literal, with optional escape sequences.
    String(String),
    /// A raw JavaScript expression delimited by backquotes.
    Expr(String),
    /// A control character understood by Percival.
    Ctrl(&'static str),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Ident(s) => write!(f, "{}", s),
            Token::Number(n) => write!(f, "{}", n),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Expr(e) => write!(f, "`{}`", e),
            Token::Ctrl(c) => write!(f, "{}", c),
        }
    }
}

/// Construct a parser combinator for lexical analysis (stage 1).
///
/// If possible, prefer to use the higher-level `Grammar` API directly, rather
/// than this low-level implementation of a parser combinator.
pub fn lexer() -> BoxedParser<'static, char, Vec<(Token, Span)>, Simple<char>> {
    let ident = text::ident().labelled("ident");

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

    let expr = just('`')
        .ignore_then(take_until(just('`')))
        .map(|(s, _)| s)
        .collect()
        .labelled("expr");

    let ctrl = choice::<_, Simple<char>>((
        just::<_, _, Simple<char>>(":-"),
        just::<_, _, Simple<char>>("("),
        just::<_, _, Simple<char>>(")"),
        just::<_, _, Simple<char>>("["),
        just::<_, _, Simple<char>>("]"),
        just::<_, _, Simple<char>>("{"),
        just::<_, _, Simple<char>>("}"),
        just::<_, _, Simple<char>>(":"),
        just::<_, _, Simple<char>>("."),
        just::<_, _, Simple<char>>(","),
        just::<_, _, Simple<char>>("="),
    ));

    let token = choice((
        ident.map(Token::Ident),
        number.map(Token::Number),
        string.map(Token::String),
        expr.map(Token::Expr),
        ctrl.map(Token::Ctrl),
    ))
    .boxed()
    .recover_with(skip_then_retry_until([]));

    let comments = {
        let single_line = just("//").then_ignore(take_until(text::newline()));
        let multi_line = just("/*").then_ignore(take_until(just("*/")));
        single_line
            .or(multi_line)
            .padded()
            .repeated()
            .map_err(|e: Simple<char>| Simple::custom(e.span(), "Not a valid comment"))
    };

    token
        .padded()
        .padded_by(comments)
        .map_with_span(|tok, span| (tok, span))
        .repeated()
        .boxed()
}

/// Construct a parser combinator for syntactic analysis (stage 2).
///
/// If possible, prefer to use the higher-level `Grammar` API directly, rather
/// than this low-level implementation of a parser combinator.
pub fn parser() -> BoxedParser<'static, Token, Program, Simple<Token>> {
    use Token::*;

    let ident = select! { Ident(id) => id };

    let literal = select! {
        Number(n) => Literal::Number(n),
        String(s) => Literal::String(s),
        Ident(b) if b == "true" => Literal::Boolean(true),
        Ident(b) if b == "false" => Literal::Boolean(false),
    }
    .labelled("literal");

    // Declared here so that we can use it for aggregate subqueries.
    let mut clauses = Recursive::<_, Vec<Clause>, Simple<Token>>::declare();

    let value = recursive(|value| {
        let aggregate = ident
            .then(value.delimited_by(Ctrl("["), Ctrl("]")))
            .then(clauses.clone().delimited_by(Ctrl("{"), Ctrl("}")))
            .map(|((operator, value), subquery)| Aggregate {
                operator,
                value: Box::new(value),
                subquery,
            });

        choice((
            aggregate.map(Value::Aggregate),
            literal.map(Value::Literal),
            select! {
                Expr(e) => Value::Expr(e),
                Ident(id) => Value::Id(id),
            },
        ))
        .labelled("value")
    });

    let prop = ident
        .then(just(Ctrl(":")).ignore_then(value.clone()).or_not())
        .try_map(|(id, value), span| {
            let value = value.unwrap_or_else(|| Value::Id(id.clone()));
            match &value {
                Value::Id(name) if is_reserved_word(name) => Err(Simple::custom(
                    span,
                    "Cannot use reserved word as a variable binding",
                )),
                _ => Ok((id, value)),
            }
        })
        .labelled("prop");

    let fact = ident
        .then(
            prop.separated_by(just(Ctrl(",")))
                .delimited_by(Ctrl("("), Ctrl(")")),
        )
        .map(|(name, props)| Fact {
            name,
            props: props.into_iter().collect(),
        })
        .labelled("fact");

    let expr = select! { Expr(e) => e };

    let binding = ident
        .then_ignore(just(Ctrl("=")))
        .then(value)
        .labelled("binding");

    let clause = choice((
        fact.clone().map(Clause::Fact),
        expr.map(Clause::Expr),
        binding.map(|(name, value)| Clause::Binding(name, value)),
    ))
    .labelled("clause");

    clauses.define(clause.clone().separated_by(just(Ctrl(","))));

    let rule = fact
        .then(
            just(Ctrl(":-"))
                .ignore_then(clauses)
                .then_ignore(just(Ctrl(".")))
                .try_map(|clauses, span| {
                    if clauses.is_empty() {
                        Err(Simple::custom(span, "Rule needs at least one clause"))
                    } else {
                        Ok(clauses)
                    }
                })
                .or(just(Ctrl(".")).to(Vec::new())),
        )
        .map(|(goal, clauses)| Rule { goal, clauses })
        .labelled("rule");

    let import = select! { Ident(k) if k == "import" => () }
        .ignore_then(ident)
        .then_ignore(select! { Ident(k) if k == "from" => () })
        .then(select! { String(s) => s })
        .map(|(name, uri)| Import { name, uri });

    enum Entry {
        Rule(Rule),
        Import(Import),
    }

    let program = choice((rule.map(Entry::Rule), import.map(Entry::Import)))
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

    program.then_ignore(end()).boxed()
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

/// An end-to-end grammar, combining lexing and parsing stages.
#[derive(Clone)]
pub struct Grammar {
    lexer: BoxedParser<'static, char, Vec<(Token, Span)>, Simple<char>>,
    parser: BoxedParser<'static, Token, Program, Simple<Token>>,
}

impl Grammar {
    /// Construct a new grammar for the Percival language.
    pub fn new() -> Self {
        Self {
            lexer: lexer(),
            parser: parser(),
        }
    }

    /// Parse an input source file, returning the program or a list of errors.
    pub fn parse(&self, src: &str) -> Result<Program, Vec<Simple<String>>> {
        let (tokens, errs) = self.lexer.parse_recovery(src);
        let mut errs: Vec<_> = errs.into_iter().map(|e| e.map(|c| c.to_string())).collect();

        if let Some(tokens) = tokens {
            // println!("Tokens = {:?}", tokens);
            let len = src.chars().count();
            let stream = Stream::from_iter(len..len + 1, tokens.into_iter());
            let (prog, parse_errs) = self.parser.parse_recovery(stream);
            match prog {
                Some(prog) if errs.is_empty() && parse_errs.is_empty() => Ok(prog),
                _ => {
                    errs.extend(parse_errs.into_iter().map(|e| e.map(|c| c.to_string())));
                    Err(errs)
                }
            }
        } else {
            Err(errs)
        }
    }
}

impl Default for Grammar {
    fn default() -> Self {
        Self::new()
    }
}
