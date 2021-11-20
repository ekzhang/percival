//! JavaScript dynamic code generation facilities for Percival.

use std::collections::BTreeSet;

use rpds::RedBlackTreeMap;

use crate::ast::{Program, Rule, Value};

const VAR_DEPS: &str = "__percival_deps";
const VAR_IMMUTABLE: &str = "__percival_immutable";
const VAR_FIRST_ITERATION: &str = "__percival_first_iteration";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Index {
    /// Name of the relation being indexed.
    name: String,

    /// Bound fields of the relation.
    bound: BTreeSet<String>,
}

/// Abstract identifier for variables stored in JavaScript objects.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum VarId {
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
    fn get(&self, key: &VarId) -> Option<String> {
        self.map.get(key).map(String::clone)
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
}

/// Generates a JavaScript function body that evaluates the program.
pub fn compile(prog: &Program) -> String {
    let ctx = make_global_context(prog);
    let code = [
        cmp_decls(&ctx, prog),
        cmp_main_loop(&ctx, prog),
        cmp_output(&ctx, prog),
    ];
    code.join("\n")
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
    for index in make_indices(&prog) {
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

fn cmp_decls(ctx: &Context, prog: &Program) -> String {
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
        {v}.update({imm}.Map({{{bound_contents}}}), value => {{
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
                        bound_contents = index
                            .bound
                            .iter()
                            .map(|field| format!("{f}: obj.{f}", f = field))
                            .collect::<Vec<_>>()
                            .join(", "),
                    );
                    decls.push(init_index.trim().into());
                }
            }
            _ => (),
        }
    }
    decls.join("\n")
}

fn cmp_main_loop(ctx: &Context, prog: &Program) -> String {
    let results = prog.results();
    let updates = cmp_updates(ctx, prog);
    let (ctx, new_decls) = cmp_new_decls(ctx, prog);
    let rules = "cmp_rules(&ctx, prog)";
    let set_update_to_new = "cmp_set_update_to_new(&ctx, prog)";
    let main_loop = format!(
        "
let {first_iter} = false;
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
            .collect::<Vec<_>>()
            .join("")
            + "true",
        updates = updates,
        new_decls = new_decls,
        rules = rules,
        set_update_to_new = set_update_to_new,
    );
    main_loop.trim().into()
}

fn cmp_updates(ctx: &Context, prog: &Program) -> String {
    let mut updates = Vec::new();
    let results = prog.results();
    for (id, js_name) in &ctx.map {
        match id {
            VarId::Set(name) => {
                updates.push(format!(
                    "{v} = {v}.merge({upd});",
                    v = js_name,
                    upd = ctx.get(&VarId::Update(name.into())).unwrap(),
                ));
            }
            VarId::Index(index) if results.contains(&index.name) => {
                let upd_name = ctx.get(&VarId::Update(index.name.clone())).unwrap();
                let ind_upd_name = ctx.get(&VarId::IndexUpdate(index.clone())).unwrap();
                let code = format!(
                    "
{v} = {v}.asMutable();
let {ind_upd} = {imm}.Map().asMutable();
for (const obj of {upd}) {{
    const key = {imm}.Map(...TODO...);
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
                );
                updates.push(code.trim().into());
            }
            _ => (),
        }
    }
    updates.join("\n")
}

fn cmp_new_decls(ctx: &Context, prog: &Program) -> (Context, String) {
    let mut ctx = ctx.clone();
    let mut decls = Vec::new();
    for result in prog.results() {
        let name = ctx.gensym(&format!("{}_new", result));
        decls.push(format!("let {} = Immutable.Set().asMutable();", name));
        ctx = ctx.add(VarId::New(result), name);
    }
    (ctx, decls.join("\n"))
}

fn cmp_output(ctx: &Context, prog: &Program) -> String {
    let fields: Vec<_> = prog
        .results()
        .into_iter()
        .map(|name| {
            format!(
                "{}: {}.toJS()",
                name,
                ctx.get(&VarId::Set(name.clone()))
                    .expect("output result set not found in context")
            )
        })
        .collect();
    format!("return {{{}}};", fields.join(", "))
}
