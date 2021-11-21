//! Abstract syntax tree definitions for the Percival language.

use std::collections::{BTreeMap, BTreeSet};

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
    pub goal: Fact,
    /// Tail or conditional assumptions of the Horn clause.
    pub clauses: Vec<Fact>,
}

/// Literal part of a Horn clause, written in terms of relations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fact {
    /// Name of the relation being referenced.
    pub name: String,
    /// Named properties of the relation.
    pub props: BTreeMap<String, Value>,
}

/// A bound or unbound value assigned to part of a relation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    /// A simple identifier, which can be either bound or unbound.
    Id(String),
    // TODO: Expr(Expr),
}

impl Program {
    /// Returns the names of all relations produced by this program.
    pub fn results(&self) -> BTreeSet<String> {
        self.rules
            .iter()
            .map(|rule| rule.goal.name.clone())
            .collect()
    }

    /// Returns the names of all external relations that this program uses.
    pub fn deps(&self) -> BTreeSet<String> {
        let results = self.results();
        self.rules
            .iter()
            .flat_map(|rule| {
                rule.clauses.iter().filter_map(|clause| {
                    if results.contains(&clause.name) {
                        None
                    } else {
                        Some(clause.name.clone())
                    }
                })
            })
            .collect()
    }
}
