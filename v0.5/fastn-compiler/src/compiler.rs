struct Compiler {
    symbols: Box<dyn fastn_compiler::SymbolStore>,
    interner: string_interner::DefaultStringInterner,
    bag: std::collections::HashMap<string_interner::DefaultSymbol, fastn_unresolved::LookupResult>,
    #[expect(unused)]
    auto_imports: Vec<fastn_section::AutoImport>,
    document: fastn_unresolved::Document,
}

impl Compiler {
    fn new(
        symbols: Box<dyn fastn_compiler::SymbolStore>,
        auto_imports: Vec<fastn_section::AutoImport>,
        document_id: &fastn_unresolved::ModuleName,
        source: &str,
    ) -> Self {
        let document = fastn_unresolved::parse(document_id, source);

        Self {
            symbols,
            interner: string_interner::StringInterner::new(),
            bag: std::collections::HashMap::new(),
            auto_imports,
            document,
        }
    }

    fn update_partially_resolved(&mut self, partially_resolved: Vec<fastn_unresolved::Definition>) {
        for definition in partially_resolved {
            let symbol = definition.symbol.unwrap();
            match definition.resolved() {
                Ok(v) => {
                    self.bag.insert(
                        symbol,
                        fastn_unresolved::LookupResult {
                            symbol,
                            definition: fastn_unresolved::UR::Resolved(v),
                        },
                    );
                }
                Err(v) => {
                    self.bag.insert(
                        symbol,
                        fastn_unresolved::LookupResult {
                            symbol,
                            definition: fastn_unresolved::UR::UnResolved(v),
                        },
                    );
                }
            }
        }
    }

    async fn fetch_unresolved_symbols(
        &mut self,
        symbols_to_fetch: &[fastn_unresolved::SymbolName],
    ) {
        let definitions = self.symbols.lookup(&mut self.interner, symbols_to_fetch);
        for definition in definitions {
            self.bag.insert(definition.symbol, definition);
        }
    }

    /// try to resolve as many symbols as possible, and return the ones that we made any progress on.
    ///
    /// this function should be called in a loop, until the list of symbols is empty.
    fn resolve_symbols(
        &mut self,
        symbols: Vec<fastn_unresolved::SymbolName>,
    ) -> ResolveSymbolsResult {
        let mut r = ResolveSymbolsResult::default();
        for symbol in symbols {
            let sym = symbol.symbol(&mut self.interner);
            let mut definition = self.bag.remove(&sym);
            match definition.as_mut().map(|v| &mut v.definition) {
                Some(fastn_unresolved::UR::UnResolved(definition)) => {
                    r.need_more_symbols
                        .extend_from_slice(&definition.resolve(&self.bag));
                }
                Some(fastn_unresolved::UR::Resolved(_)) => unreachable!(),
                _ => r.unresolvable.push(symbol),
            }
            if let Some(definition) = definition {
                self.bag.insert(sym, definition);
            }
        }

        r
    }

    /// try to make as much progress as possibly by resolving as many symbols as possible, and return
    /// the vec of ones that could not be resolved.
    ///
    /// if this returns an empty list of symbols, we can go ahead and generate the JS.
    fn resolve_document(&mut self) -> Vec<fastn_unresolved::SymbolName> {
        let mut stuck_on_symbols = vec![];

        for ci in self.document.content.iter_mut() {
            if let fastn_unresolved::UR::UnResolved(c) = ci {
                stuck_on_symbols.extend(c.resolve(&self.bag));
            }
        }

        stuck_on_symbols
    }

    async fn compile(&mut self) -> Result<fastn_compiler::Output, fastn_compiler::Error> {
        // we only make 10 attempts to resolve the document: we need a warning if we are not able to
        // resolve the document in 10 attempts.
        let mut unresolvable = vec![];
        for _ in 1..10 {
            // resolve_document can internally run in parallel.
            let unresolved_symbols = self.resolve_document();
            if unresolved_symbols.is_empty() {
                break;
            }

            self.fetch_unresolved_symbols(&unresolved_symbols).await;
            // this itself has to happen in a loop. we need a warning if we are not able to resolve all
            // symbols in 10 attempts.
            let mut r = ResolveSymbolsResult::default();
            r.need_more_symbols.extend(unresolved_symbols);

            for _ in 1..10 {
                // resolve_document can internally run in parallel.
                r = self.resolve_symbols(r.need_more_symbols);
                unresolvable.extend_from_slice(&r.unresolvable);
                self.update_partially_resolved(r.partially_resolved);
                if r.need_more_symbols.is_empty() {
                    break;
                }
                self.fetch_unresolved_symbols(&r.need_more_symbols).await;
            }

            if !unresolvable.is_empty() {
                // we were not able to resolve all symbols
            }
        }

        todo!()
    }
}

/// this is our main compiler
///
/// it should be called with a parsed document, and it returns generated JS.
///
/// on success, we return the JS, and list of warnings, and on error, we return the list of
/// diagnostics, which is an enum containing warning and error.
///
/// earlier we had strict mode here, but to simplify things, now we let the caller convert non-empty
/// warnings from OK part as error, and discard the generated JS.
pub async fn compile(
    symbols: Box<dyn fastn_compiler::SymbolStore>,
    document_id: &fastn_unresolved::ModuleName,
    source: &str,
    auto_imports: Vec<fastn_section::AutoImport>,
) -> Result<fastn_compiler::Output, fastn_compiler::Error> {
    Compiler::new(symbols, auto_imports, document_id, source)
        .compile()
        .await
}

#[derive(Default)]
struct ResolveSymbolsResult {
    partially_resolved: Vec<fastn_unresolved::Definition>,
    need_more_symbols: Vec<fastn_unresolved::SymbolName>,
    unresolvable: Vec<fastn_unresolved::SymbolName>,
}
