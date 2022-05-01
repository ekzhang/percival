# Filetypes
RUST = $(shell find crates -name '*.rs')
PATCHES = $(shell find patches -name '*.patch')

# Repeated paths
percival_wasm_pkg = crates/percival-wasm/pkg
percival_bindings = crates/percival/bindings

node_modules: package.json package-lock.json $(PATCHES) $(percival_wasm_pkg)
	npm install

crates/percival-wasm/pkg: $(RUST)
	#	Build WASM
	wasm-pack build --target web crates/percival-wasm
	# Build AST types
	cargo test ast::export_bindings
	mkdir -p $(percival_wasm_pkg)/ast
	for ts in $(percival_bindings)/* ; do \
		cp $$ts $(percival_wasm_pkg)/ast/"$$(basename "$${ts%.ts}.d.ts")" ; \
	done
	# Patch WASM types to use AST types
	sed -i '' -e 's~ast(): any~ast(): import("./ast/Program").Program | undefined~' $(percival_wasm_pkg)/percival_wasm.d.ts
