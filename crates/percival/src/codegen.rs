//! JavaScript dynamic code generation facilities for Percival.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use rpds::RedBlackTreeMap;
use thiserror::Error;

use crate::ast::{Fact, Literal, Program, Rule, Value};

/// An error during code generation.
#[derive(Error, Debug)]
pub enum Error {
    /// A given variable was not found in context.
    #[error("could not find definition for `{0:?}` in context")]
    UndefVar(VarId),
}

/// Result returned by the compiler.
pub type Result<T> = std::result::Result<T, Error>;

const VAR_DEPS: &str = "__percival_deps";
const VAR_IMMUTABLE: &str = "__percival_immutable";
const VAR_FIRST_ITERATION: &str = "__percival_first_iteration";

/// An index created on a subset of relation fields.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Index {
    /// Name of the relation being indexed.
    name: String,

    /// Bound fields of the relation.
    bound: BTreeSet<String>,
}

/// Abstract identifier for variables stored in JavaScript objects.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum VarId {
    /// Active sets of current relations.
    Set(String),

    /// Index maps of current relations.
    Index(Index),

    /// Updated relations in the current iteration.
    Update(String),

    /// Updates to index maps of current relations.
    IndexUpdate(Index),

    /// New relations in the current iteration.
    New(String),

    /// A bound local variable in Datalog.
    Var(String),
}

/// Context storing mappings of [`VarId`] to their JavaScript identifiers.
///
/// This is implemented using a persistent data structure, so it can be cheaply
/// cloned to produce nested subcontexts.
#[derive(Clone, Debug, Default)]
struct Context {
    map: RedBlackTreeMap<VarId, String>,
    counter: u32,
}

impl Context {
    fn new() -> Self {
        Default::default()
    }

    /// Produce a new, globally unique symbol for compilation.
    fn gensym(&mut self, key: &str) -> String {
        let counter = self.counter;
        self.counter += 1;
        format!("__percival_{}_{}", key, counter)
    }

    /// Get an entry of the map.
    fn get(&self, key: &VarId) -> Result<String> {
        self.map
            .get(key)
            .map(String::clone)
            .ok_or_else(|| Error::UndefVar(key.clone()))
    }

    /// Add a new entry to the map, returning a new map.
    fn add(&self, key: VarId, value: String) -> Self {
        if self.map.contains_key(&key) {
            panic!("Tried to add duplicate key {:?} to context", key);
        }
        Self {
            map: self.map.insert(key, value),
            counter: self.counter,
        }
    }

    /// Check is a fact value is bound or free, given the current context.
    fn is_bound(&self, value: &Value) -> bool {
        match value {
            Value::Id(id) => self.map.contains_key(&VarId::Var(id.clone())),
            Value::Literal(_) => true,
        }
    }
}

/// Generates a JavaScript function body that evaluates the program.
pub fn compile(prog: &Program) -> Result<String> {
    let ctx = make_global_context(prog);
    let code = [
        cmp_decls(&ctx, prog)?,
        cmp_main_loop(&ctx, prog)?,
        cmp_output(&ctx, prog)?,
    ];
    Ok(code.join("\n"))
}

fn make_global_context(prog: &Program) -> Context {
    let mut ctx = Context::new();

    for name in prog.results() {
        let set_name = ctx.gensym(&name);
        let update_name = ctx.gensym(&format!("{}_update", name));
        ctx = ctx
            .add(VarId::Set(name.clone()), set_name)
            .add(VarId::Update(name), update_name);
    }

    let results = prog.results();
    for index in make_indices(prog) {
        let index_name = ctx.gensym(&format!("{}_index", index.name));
        ctx = ctx.add(VarId::Index(index.clone()), index_name);
        if results.contains(&index.name) {
            let update_name = ctx.gensym(&format!("{}_index_update", index.name));
            ctx = ctx.add(VarId::IndexUpdate(index), update_name);
        }
    }

    ctx
}

fn make_indices(prog: &Program) -> BTreeSet<Index> {
    prog.rules
        .iter()
        .flat_map(|rule| {
            let mut vars = BTreeSet::new();
            let mut indices = BTreeSet::new();
            for clause in &rule.clauses {
                let mut bound = BTreeSet::new();
                for (key, value) in &clause.props {
                    match value {
                        Value::Id(id) => {
                            if vars.contains(id) {
                                bound.insert(key.to_owned());
                            } else {
                                vars.insert(id);
                            }
                        }
                        Value::Literal(_) => {
                            bound.insert(key.to_owned());
                        }
                    }
                }
                if !bound.is_empty() {
                    indices.insert(Index {
                        name: clause.name.clone(),
                        bound,
                    });
                }
            }
            indices
        })
        .collect()
}

fn cmp_decls(ctx: &Context, prog: &Program) -> Result<String> {
    let mut decls = Vec::new();
    let deps = prog.deps();
    for (id, js_name) in ctx.map.iter() {
        match id {
            VarId::Set(_) | VarId::Update(_) => {
                decls.push(format!("let {} = {}.Set();", js_name, VAR_IMMUTABLE));
            }
            VarId::Index(index) => {
                decls.push(format!("let {} = {}.Map();", js_name, VAR_IMMUTABLE));
                if deps.contains(&index.name) {
                    // Initialize index in the declarations.
                    let init_index = format!(
                        "
{v} = {v}.withMutations({v} => {{
    for (let obj of {deps}.{name}) {{
        {v}.update({imm}.Map({bindings}), value => {{
            if (value === undefined) value = [];
            value.push({imm}.Map(obj));
            return value;
        }});
    }}
}});",
                        v = js_name,
                        deps = VAR_DEPS,
                        imm = VAR_IMMUTABLE,
                        name = index.name,
                        bindings = cmp_object(&index.bound, |field| Ok(format!("obj.{}", field)))?,
                    );
                    decls.push(init_index.trim().into());
                }
            }
            _ => (),
        }
    }
    Ok(decls.join("\n"))
}

fn cmp_main_loop(ctx: &Context, prog: &Program) -> Result<String> {
    let results = prog.results();
    let updates = cmp_updates(ctx, prog)?;
    let (ctx, new_decls) = cmp_new_decls(ctx, prog);
    let rules = cmp_rules(&ctx, prog)?;
    let set_update_to_new = cmp_set_update_to_new(&ctx, prog)?;
    let main_loop = format!(
        "
let {first_iter} = true;
while ({first_iter} || !({no_updates})) {{
    {updates}
    {new_decls}
    {rules}
    {set_update_to_new}
    {first_iter} = false;
}}",
        first_iter = VAR_FIRST_ITERATION,
        no_updates = results
            .iter()
            .map(|name| format!(
                "{}.size === 0 && ",
                ctx.get(&VarId::Update(name.into()))
                    .expect("could not find name in main loop no_updates")
            ))
            .collect::<Box<_>>()
            .join("")
            + "true",
        updates = updates,
        new_decls = new_decls,
        rules = rules,
        set_update_to_new = set_update_to_new,
    );
    Ok(main_loop.trim().into())
}

fn cmp_updates(ctx: &Context, prog: &Program) -> Result<String> {
    let mut updates = Vec::new();
    let results = prog.results();
    for (id, js_name) in &ctx.map {
        match id {
            VarId::Set(name) => {
                updates.push(format!(
                    "{v} = {v}.merge({upd});",
                    v = js_name,
                    upd = ctx.get(&VarId::Update(name.into()))?,
                ));
            }
            VarId::Index(index) if results.contains(&index.name) => {
                let upd_name = ctx.get(&VarId::Update(index.name.clone()))?;
                let ind_upd_name = ctx.get(&VarId::IndexUpdate(index.clone()))?;
                let code = format!(
                    "
{v} = {v}.asMutable();
let {ind_upd} = {imm}.Map().asMutable();
for (const obj of {upd}) {{
    const key = {imm}.Map({key});
    {v}.update(key, value => {{
        if (value === undefined) value = [];
        value.push(obj);
        return value;
    }});
    {ind_upd}.update(key, value => {{
        if (value === undefined) value = [];
        value.push(obj);
        return value;
    }});
}}
{v} = {v}.asImmutable();
{ind_upd} = {ind_upd}.asImmutable();
",
                    imm = VAR_IMMUTABLE,
                    v = js_name,
                    upd = upd_name,
                    ind_upd = ind_upd_name,
                    key = cmp_object(&index.bound, |field| Ok(format!("obj.get('{}')", field)))?,
                );
                updates.push(code.trim().into());
            }
            _ => (),
        }
    }
    Ok(updates.join("\n"))
}

fn cmp_new_decls(ctx: &Context, prog: &Program) -> (Context, String) {
    let mut ctx = ctx.clone();
    let mut decls = Vec::new();
    for result in prog.results() {
        let name = ctx.gensym(&format!("{}_new", result));
        decls.push(format!(
            "let {} = {}.Set().asMutable();",
            name, VAR_IMMUTABLE,
        ));
        ctx = ctx.add(VarId::New(result), name);
    }
    (ctx, decls.join("\n"))
}

fn cmp_rules(ctx: &Context, prog: &Program) -> Result<String> {
    Ok(prog
        .rules
        .iter()
        .map(|rule| cmp_rule(ctx, rule))
        .collect::<Result<Box<_>>>()?
        .join("\n"))
}

fn cmp_rule(ctx: &Context, rule: &Rule) -> Result<String> {
    let mut ctx = ctx.clone();

    let mut clauses = Vec::new();
    for clause in &rule.clauses {
        clauses.push(cmp_clause(&mut ctx, clause)?);
    }

    let goal = format!(
        "
let goal = {imm}.Map({goal});
if (!{set}.includes(goal)) {new}.add(goal);
",
        imm = VAR_IMMUTABLE,
        goal = cmp_fields(&ctx, &rule.goal.props)?,
        set = ctx.get(&VarId::Set(rule.goal.name.clone())).unwrap(),
        new = ctx.get(&VarId::New(rule.goal.name.clone())).unwrap(),
    );

    let mut code = String::from("{\n");
    for clause in &clauses {
        code += clause;
        code += "\n";
    }
    code += goal.trim();
    code += &"\n}".repeat(clauses.len() + 1);
    Ok(code)
}

fn cmp_clause(ctx: &mut Context, clause: &Fact) -> Result<String> {
    let mut bound_fields = BTreeMap::new();
    let mut setters = Vec::new();
    for (key, value) in &clause.props {
        if ctx.is_bound(value) {
            bound_fields.insert(key.clone(), value.clone());
        } else {
            match value {
                Value::Id(id) => {
                    let name = ctx.gensym(id);
                    setters.push(format!("let {} = obj.get('{}');", name, key));
                    *ctx = ctx.add(VarId::Var(id.clone()), name);
                }
                Value::Literal(_) => unreachable!("literal values are always bound"),
            }
        }
    }

    if bound_fields.is_empty() {
        // No bound fields, just iterate over the set.
        let code = format!(
            "
for (const obj of {set}) {{
    {setters}
",
            set = ctx
                .get(&VarId::Set(clause.name.clone()))
                .unwrap_or_else(|_| format!("{}.{}", VAR_DEPS, clause.name)),
            setters = setters.join("\n"),
        );
        Ok(code.trim().into())
    } else {
        // At least one field is bound, so we use an index instead.
        let index = Index {
            name: clause.name.clone(),
            bound: bound_fields.keys().cloned().collect(),
        };
        let code = format!(
            "
for (const obj of {index}.get({imm}.Map({bindings})) ?? []) {{
    {setters}
",
            imm = VAR_IMMUTABLE,
            index = ctx.get(&VarId::Index(index))?,
            bindings = cmp_fields(ctx, &bound_fields)?,
            setters = setters.join("\n"),
        );
        Ok(code.trim().into())
    }
}

fn cmp_fields(ctx: &Context, props: &BTreeMap<String, Value>) -> Result<String> {
    cmp_object(props.keys(), |key| {
        let value = props.get(key).unwrap();
        cmp_value(ctx, value)
    })
}

fn cmp_value(ctx: &Context, value: &Value) -> Result<String> {
    Ok(match value {
        Value::Id(id) => ctx.get(&VarId::Var(id.clone()))?,
        Value::Literal(Literal::Number(n)) => n.clone(),
        Value::Literal(Literal::String(s)) => format!("\"{}\"", s),
    })
}

fn cmp_set_update_to_new(ctx: &Context, prog: &Program) -> Result<String> {
    let setters = prog
        .results()
        .into_iter()
        .map(|name| {
            Ok(format!(
                "{} = {}.asImmutable();",
                ctx.get(&VarId::Update(name.clone()))?,
                ctx.get(&VarId::New(name))?,
            ))
        })
        .collect::<Result<Box<_>>>()?;
    Ok(setters.join("\n"))
}

fn cmp_output(ctx: &Context, prog: &Program) -> Result<String> {
    let obj = cmp_object(&prog.results(), |name| {
        Ok(format!("{}.toJS()", ctx.get(&VarId::Set(name.clone()))?))
    })?;
    Ok(format!("return {};", obj))
}

fn cmp_object<T: Copy + Display, U: Display>(
    fields: impl IntoIterator<Item = T>,
    value_fn: impl Fn(T) -> Result<U>,
) -> Result<String> {
    let fields = fields
        .into_iter()
        .map(|field| value_fn(field).map(|value| format!("{}: {}", field, value)))
        .collect::<Result<Box<_>>>()?;
    Ok(format!("{{{}}}", fields.join(", ")))
}
