//! JavaScript dynamic code generation facilities for Percival.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    rc::Rc,
};

use rpds::{RedBlackTreeMap, RedBlackTreeSet};
use thiserror::Error;

use crate::ast::{Aggregate, Clause, Literal, Program, Rule, Value};

const VAR_DEPS: &str = "__percival_deps";
const VAR_IMMUTABLE: &str = "__percival.Immutable";
const VAR_LOAD: &str = "__percival.load";
const VAR_AGGREGATES: &str = "__percival.aggregates";
const VAR_IMPORTS: &str = "__percival_imports";

const VAR_FIRST_ITERATION: &str = "__percival_first_iteration";
const VAR_OBJ: &str = "__percival_obj";
const VAR_GOAL: &str = "__percival_goal";

/// List of aggregate operators. Keep this in sync with `worker.ts`.
const OPERATORS: [&str; 5] = ["count", "sum", "mean", "min", "max"];

/// An error during code generation.
#[derive(Error, Debug)]
pub enum Error {
    /// A given variable was not found in context.
    #[error("Could not find definition of `{0:?}` in context")]
    UndefVar(VarId),

    /// Two conflicting imports were found with the same name.
    #[error("Multiple imports found with name \"{0}\"")]
    DuplicateImport(String),

    /// Tried to put an import on the left-hand side of a rule.
    #[error("Imported relation \"{0}\" cannot be used as the goal of a rule")]
    GoalImportConflict(String),

    /// Import protocol not understood in directive.
    #[error("Unknown import protocol \"{0}\"")]
    UnknownProtocol(String),

    /// Two conflicting variables were defined with the same name.
    #[error("Conflicting declaration of variable \"{0}\"")]
    DuplicateVariable(String),

    /// Unknown aggregate operator was referenced.
    #[error("Aggregate operator \"{0}\" is not in {OPERATORS:?}")]
    UnknownAggregate(String),

    /// Aggregate references relation that is declared in this cell.
    #[error("Relation \"{0}\" is queried in the same cell that it is declared")]
    CircularReference(String),
}

/// Result returned by the compiler.
pub type Result<T> = std::result::Result<T, Error>;

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
#[derive(Clone, Debug)]
struct Context {
    map: RedBlackTreeMap<VarId, String>,
    deps: Rc<BTreeSet<String>>,
    results: Rc<BTreeSet<String>>,
    imports: Rc<BTreeSet<String>>,
    counter: u32,
}

impl Context {
    fn new(prog: &Program) -> Self {
        Context {
            map: RedBlackTreeMap::new(),
            deps: Rc::new(prog.deps()),
            results: Rc::new(prog.results()),
            imports: Rc::new(prog.imports()),
            counter: 0,
        }
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
            ..self.clone()
        }
    }

    /// Check is a fact value is bound or free, given the current context.
    fn is_bound(&self, value: &Value) -> bool {
        match value {
            Value::Id(id) => self.map.contains_key(&VarId::Var(id.clone())),
            Value::Literal(_) | Value::Expr(_) | Value::Aggregate(_) => true,
        }
    }
}

/// Generates a JavaScript function body that evaluates the program.
pub fn compile(prog: &Program) -> Result<String> {
    let ctx = make_global_context(prog)?;
    let code = [
        cmp_imports(prog)?,
        cmp_decls(&ctx)?,
        cmp_main_loop(&ctx, prog)?,
        cmp_output(&ctx)?,
    ];
    Ok(code.join("\n"))
}

fn make_global_context(prog: &Program) -> Result<Context> {
    let mut ctx = Context::new(prog);

    if ctx.imports.len() < prog.imports.len() {
        // Some duplicate import during parsing, find and return it.
        let mut names = BTreeSet::new();
        for import in &prog.imports {
            if names.contains(&import.name) {
                return Err(Error::DuplicateImport(import.name.clone()));
            }
            names.insert(import.name.clone());
        }
        unreachable!("At least one import must be duplicated");
    }

    for name in Rc::clone(&ctx.imports).iter() {
        if ctx.results.contains(name) {
            return Err(Error::GoalImportConflict(name.clone()));
        }
        let set_name = ctx.gensym(name);
        ctx = ctx.add(VarId::Set(name.clone()), set_name);
    }

    for name in Rc::clone(&ctx.deps).iter() {
        let set_name = ctx.gensym(name);
        ctx = ctx.add(VarId::Set(name.clone()), set_name);
    }

    for name in Rc::clone(&ctx.results).iter() {
        let set_name = ctx.gensym(name);
        let update_name = ctx.gensym(&format!("{}_update", name));
        ctx = ctx
            .add(VarId::Set(name.clone()), set_name)
            .add(VarId::Update(name.clone()), update_name);
    }

    for index in make_indices(prog) {
        let index_name = ctx.gensym(&format!("{}_index", index.name));
        ctx = ctx.add(VarId::Index(index.clone()), index_name);
        if ctx.results.contains(&index.name) {
            let update_name = ctx.gensym(&format!("{}_index_update", index.name));
            ctx = ctx.add(VarId::IndexUpdate(index), update_name);
        }
    }

    Ok(ctx)
}

fn make_indices(prog: &Program) -> BTreeSet<Index> {
    fn walk_clause<'a>(
        indices: &mut BTreeSet<Index>,
        vars: &mut RedBlackTreeSet<&'a str>,
        clause: &'a Clause,
    ) {
        match clause {
            Clause::Fact(fact) => {
                for value in fact.props.values() {
                    walk_value(indices, vars, value);
                }
                let mut bound = BTreeSet::new();
                for (key, value) in &fact.props {
                    match value {
                        Value::Id(id) => {
                            if vars.contains(&id[..]) {
                                bound.insert(key.to_owned());
                            } else {
                                *vars = vars.insert(id);
                            }
                        }
                        Value::Literal(_) | Value::Expr(_) | Value::Aggregate(_) => {
                            bound.insert(key.to_owned());
                        }
                    }
                }
                if !bound.is_empty() {
                    indices.insert(Index {
                        name: fact.name.clone(),
                        bound,
                    });
                }
            }
            Clause::Expr(_) => (),
            Clause::Binding(_, value) => {
                walk_value(indices, vars, value);
            }
        }
    }

    fn walk_clauses<'a>(
        indices: &mut BTreeSet<Index>,
        vars: &mut RedBlackTreeSet<&'a str>,
        clauses: &'a [Clause],
    ) {
        for clause in clauses {
            walk_clause(indices, vars, clause);
        }
    }

    fn walk_value(indices: &mut BTreeSet<Index>, vars: &RedBlackTreeSet<&str>, value: &Value) {
        if let Value::Aggregate(aggregate) = value {
            let mut vars = vars.clone();
            walk_clauses(indices, &mut vars, &aggregate.subquery);
            walk_value(indices, &vars, &aggregate.value);
        }
    }

    fn walk_rule(indices: &mut BTreeSet<Index>, rule: &Rule) {
        let mut vars = RedBlackTreeSet::new();
        walk_clauses(indices, &mut vars, &rule.clauses);
        for value in rule.goal.props.values() {
            walk_value(indices, &vars, value);
        }
    }

    let mut indices = BTreeSet::new();
    for rule in &prog.rules {
        walk_rule(&mut indices, rule);
    }
    indices
}

fn cmp_imports(prog: &Program) -> Result<String> {
    if prog.imports.is_empty() {
        return Ok("".into());
    }
    let mut fields = Vec::new();
    for import in &prog.imports {
        let index = import.uri.find("://");
        let index = index.ok_or_else(|| Error::UnknownProtocol("<none>".into()))?;
        let (protocol, address) = import.uri.split_at(index + 3);
        let url = match protocol {
            "http://" | "https://" => import.uri.clone(),
            "gh://" => format!("https://cdn.jsdelivr.net/gh/{}", address),
            "npm://" => format!("https://cdn.jsdelivr.net/npm/{}", address),
            _ => return Err(Error::UnknownProtocol(protocol.into())),
        };
        fields.push(format!(
            "{}: await {}(\"{}\"),\n",
            import.name, VAR_LOAD, url,
        ));
    }
    Ok(format!(
        "const {} = {{\n{}}};",
        VAR_IMPORTS,
        fields.join(""),
    ))
}

fn cmp_decls(ctx: &Context) -> Result<String> {
    let mut decls = Vec::new();
    for (id, js_name) in &ctx.map {
        match id {
            VarId::Set(name) | VarId::Update(name) => {
                decls.push(format!("let {} = {}.Set();", js_name, VAR_IMMUTABLE));
                if ctx.deps.contains(name) || ctx.imports.contains(name) {
                    // Initialize sets - need to move to Immutable.Map objects.
                    let source = if ctx.deps.contains(name) {
                        VAR_DEPS
                    } else {
                        VAR_IMPORTS
                    };
                    let init_set = format!(
                        "
{v} = {v}.withMutations({v} => {{
    for (const {obj} of {source}.{name}) {{
        {v}.add({imm}.Map({obj}));
    }}
}});
",
                        v = js_name,
                        obj = VAR_OBJ,
                        source = source,
                        imm = VAR_IMMUTABLE,
                        name = name,
                    );
                    decls.push(init_set.trim().into());
                }
            }
            VarId::Index(index) => {
                decls.push(format!("let {} = {}.Map();", js_name, VAR_IMMUTABLE));
                if ctx.deps.contains(&index.name) || ctx.imports.contains(&index.name) {
                    // Initialize index in the declarations.
                    let source = if ctx.deps.contains(&index.name) {
                        VAR_DEPS
                    } else {
                        VAR_IMPORTS
                    };
                    let init_index = format!(
                        "
{v} = {v}.withMutations({v} => {{
    for (const {obj} of {source}.{name}) {{
        {v}.update({imm}.Map({bindings}), value => {{
            if (value === undefined) value = [];
            value.push({imm}.Map({obj}));
            return value;
        }});
    }}
}});",
                        v = js_name,
                        obj = VAR_OBJ,
                        source = source,
                        imm = VAR_IMMUTABLE,
                        name = index.name,
                        bindings = cmp_object(&index.bound, |field| {
                            Ok(format!("{}.{}", VAR_OBJ, field))
                        })?,
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
    let updates = cmp_updates(ctx)?;
    let (ctx, new_decls) = cmp_new_decls(ctx);
    let rules = cmp_rules(&ctx, prog)?;
    let set_update_to_new = cmp_set_update_to_new(&ctx)?;
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
        no_updates = ctx
            .results
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

fn cmp_updates(ctx: &Context) -> Result<String> {
    let mut updates = Vec::new();
    for (id, js_name) in &ctx.map {
        match id {
            VarId::Update(name) => {
                updates.push(format!(
                    "{v} = {v}.merge({upd});",
                    v = ctx.get(&VarId::Set(name.into()))?,
                    upd = js_name,
                ));
            }
            VarId::Index(index) if ctx.results.contains(&index.name) => {
                let upd_name = ctx.get(&VarId::Update(index.name.clone()))?;
                let ind_upd_name = ctx.get(&VarId::IndexUpdate(index.clone()))?;
                let code = format!(
                    "
{v} = {v}.asMutable();
let {ind_upd} = {imm}.Map().asMutable();
for (const {obj} of {upd}) {{
    const key = {imm}.Map({key});
    {v}.update(key, value => {{
        if (value === undefined) value = [];
        value.push({obj});
        return value;
    }});
    {ind_upd}.update(key, value => {{
        if (value === undefined) value = [];
        value.push({obj});
        return value;
    }});
}}
{v} = {v}.asImmutable();
{ind_upd} = {ind_upd}.asImmutable();
",
                    imm = VAR_IMMUTABLE,
                    obj = VAR_OBJ,
                    v = js_name,
                    upd = upd_name,
                    ind_upd = ind_upd_name,
                    key = cmp_object(&index.bound, |field| {
                        Ok(format!("{}.get('{}')", VAR_OBJ, field))
                    })?,
                );
                updates.push(code.trim().into());
            }
            _ => (),
        }
    }
    Ok(updates.join("\n"))
}

fn cmp_new_decls(ctx: &Context) -> (Context, String) {
    let mut ctx = ctx.clone();
    let mut decls = Vec::new();
    for result in Rc::clone(&ctx.results).iter() {
        let name = ctx.gensym(&format!("{}_new", result));
        decls.push(format!(
            "let {} = {}.Set().asMutable();",
            name, VAR_IMMUTABLE,
        ));
        ctx = ctx.add(VarId::New(result.clone()), name);
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

/// Compile a single Datalog rule into a collection of loops.
fn cmp_rule(ctx: &Context, rule: &Rule) -> Result<String> {
    let fact_positions: Vec<_> = rule
        .clauses
        .iter()
        .enumerate()
        .filter_map(|(i, clause)| match clause {
            Clause::Fact(fact) if ctx.results.contains(&fact.name) => Some(i),
            _ => None,
        })
        .collect();

    if fact_positions.is_empty() {
        // Will not change, so we only need to evaluate it once
        let eval_loop = cmp_rule_incremental(ctx, rule, None)?;
        Ok(format!(
            "if ({first_iter}) {{\n{eval_loop}\n}}",
            first_iter = VAR_FIRST_ITERATION,
            eval_loop = eval_loop
        ))
    } else {
        // Rule has one or more facts, so we use semi-naive evaluation
        let variants = fact_positions
            .into_iter()
            .map(|update_position| cmp_rule_incremental(ctx, rule, Some(update_position)))
            .collect::<Result<Box<_>>>()?;
        Ok(variants.join("\n"))
    }
}

/// Compile a single incremental semi-naive evaluation loop for a rule.
fn cmp_rule_incremental(
    ctx: &Context,
    rule: &Rule,
    update_position: Option<usize>,
) -> Result<String> {
    let mut ctx = ctx.clone();

    let mut clauses = Vec::new();
    for (i, clause) in rule.clauses.iter().enumerate() {
        let only_update = update_position == Some(i);
        clauses.push(cmp_clause(&mut ctx, clause, only_update, false)?);
    }

    let goal = format!(
        "
let {goal} = {imm}.Map({goal_obj});
if (!{set}.includes({goal})) {new}.add({goal});
",
        goal = VAR_GOAL,
        imm = VAR_IMMUTABLE,
        goal_obj = cmp_fields(&ctx, &rule.goal.props)?,
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

fn cmp_clause(
    ctx: &mut Context,
    clause: &Clause,
    only_update: bool,
    is_subquery: bool,
) -> Result<String> {
    match clause {
        Clause::Fact(fact) => {
            if is_subquery && ctx.results.contains(&fact.name) {
                return Err(Error::CircularReference(fact.name.clone()));
            }

            let mut bound_fields = BTreeMap::new();
            let mut setters = Vec::new();
            for (key, value) in &fact.props {
                if ctx.is_bound(value) {
                    bound_fields.insert(key.clone(), value.clone());
                } else {
                    match value {
                        Value::Id(id) => {
                            // Use the same name for the variable in JavaScript.
                            let name = id.clone();
                            setters.push(format!("const {} = {}.get('{}');", name, VAR_OBJ, key));
                            *ctx = ctx.add(VarId::Var(id.clone()), name);
                        }
                        Value::Literal(_) | Value::Expr(_) | Value::Aggregate(_) => {
                            unreachable!("literal and expression values are always bound")
                        }
                    }
                }
            }

            if bound_fields.is_empty() {
                // No bound fields, just iterate over the set.
                let name = fact.name.clone();
                let set = ctx.get(&if !only_update {
                    VarId::Set(name)
                } else {
                    VarId::Update(name)
                })?;

                let code = format!(
                    "
for (const {obj} of {set}) {{
    {setters}
",
                    obj = VAR_OBJ,
                    set = set,
                    setters = setters.join("\n"),
                );
                Ok(code.trim().into())
            } else {
                // At least one field is bound, so we use an index instead.
                let index = Index {
                    name: fact.name.clone(),
                    bound: bound_fields.keys().cloned().collect(),
                };
                let index = ctx.get(&if !only_update {
                    VarId::Index(index)
                } else {
                    VarId::IndexUpdate(index)
                })?;

                let code = format!(
                    "
for (const {obj} of {index}.get({imm}.Map({bindings})) ?? []) {{
    {setters}
",
                    obj = VAR_OBJ,
                    imm = VAR_IMMUTABLE,
                    index = index,
                    bindings = cmp_fields(ctx, &bound_fields)?,
                    setters = setters.join("\n"),
                );
                Ok(code.trim().into())
            }
        }

        Clause::Expr(expr) => {
            assert!(!only_update);
            Ok(format!("if ({}) {{", expr))
        }

        Clause::Binding(name, value) => {
            assert!(!only_update);
            let key = VarId::Var(name.clone());
            if ctx.map.contains_key(&key) {
                return Err(Error::DuplicateVariable(name.clone()));
            }
            *ctx = ctx.add(VarId::Var(name.clone()), name.clone());
            Ok(format!("{{\nconst {} = {};", name, cmp_value(ctx, value)?))
        }
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
        Value::Literal(Literal::Boolean(b)) => b.to_string(),
        Value::Expr(e) => format!("({})", e),
        Value::Aggregate(aggregate) => cmp_aggregate(ctx, aggregate)?,
    })
}

fn cmp_aggregate(ctx: &Context, aggregate: &Aggregate) -> Result<String> {
    if !OPERATORS.contains(&&aggregate.operator[..]) {
        return Err(Error::UnknownAggregate(aggregate.operator.clone()));
    }
    let mut ctx = ctx.clone(); // Create a new context for this aggregate.
    let results_var = ctx.gensym("results");

    let subquery_loop = {
        let mut clauses = Vec::new();
        for clause in &aggregate.subquery {
            clauses.push(cmp_clause(&mut ctx, clause, false, true)?);
        }

        let goal = format!(
            "{results}.push({value});",
            results = results_var,
            value = cmp_value(&ctx, &aggregate.value)?,
        );

        let mut code = String::new();
        for clause in &clauses {
            code += clause;
            code += "\n";
        }
        code += &goal;
        code += &"\n}".repeat(clauses.len());
        code
    };

    let code = format!(
        "{agg}.{op}((() => {{
    const {results} = [];
    {subquery_loop}
    return {results};
}})())",
        agg = VAR_AGGREGATES,
        op = aggregate.operator,
        results = results_var,
        subquery_loop = subquery_loop,
    );

    Ok(code)
}

fn cmp_set_update_to_new(ctx: &Context) -> Result<String> {
    let setters = ctx
        .results
        .iter()
        .map(|name| {
            Ok(format!(
                "{} = {}.asImmutable();",
                ctx.get(&VarId::Update(name.clone()))?,
                ctx.get(&VarId::New(name.clone()))?,
            ))
        })
        .collect::<Result<Box<_>>>()?;
    Ok(setters.join("\n"))
}

fn cmp_output(ctx: &Context) -> Result<String> {
    let outputs: BTreeSet<String> = ctx
        .results
        .iter()
        .chain(ctx.imports.iter())
        .cloned()
        .collect();
    let obj = cmp_object(&outputs, |name| {
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
