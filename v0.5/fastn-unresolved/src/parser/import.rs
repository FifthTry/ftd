pub(super) fn import(
    section: fastn_section::Section,
    document: &mut fastn_unresolved::Document,
    arena: &mut fastn_section::Arena,
    package: &Option<&fastn_package::Package>,
    main_package_name: &str,
) {
    if let Some(ref kind) = section.init.kind {
        document
            .errors
            .push(kind.span().wrap(fastn_section::Error::ImportCantHaveType));
        // we will go ahead with this import statement parsing
    }

    // section.name must be exactly import.
    if section.simple_name() != Some("import") {
        document.errors.push(
            section
                .init
                .name
                .wrap(fastn_section::Error::ImportMustBeImport),
        );
        // we will go ahead with this import statement parsing
    }

    let i = match parse_import(&section, document, arena) {
        Some(v) => v,
        None => {
            // error handling is job of parse_module_name().
            return;
        }
    };

    // ensure there are no extra headers, children or body
    fastn_unresolved::utils::assert_no_body(&section, document);
    fastn_unresolved::utils::assert_no_children(&section, document);
    fastn_unresolved::utils::assert_no_extra_headers(&section, document, &["exports", "exposing"]);
    validate_import_module_in_dependencies(section, document, arena, package, &i);

    // Add import in document
    add_import(document, arena, &i);

    // Add export and exposing in document
    add_export_and_exposing(document, arena, &i, main_package_name, package);
}

fn add_import(
    document: &mut fastn_unresolved::Document,
    arena: &mut fastn_section::Arena,
    i: &Import,
) {
    add_to_document_alias(
        document,
        arena,
        i.alias.str(),
        fastn_section::SoM::Module(i.module),
    );
}

fn add_export_and_exposing(
    document: &mut fastn_unresolved::Document,
    arena: &mut fastn_section::Arena,
    i: &Import,
    main_package_name: &str,
    package: &Option<&fastn_package::Package>,
) {
    let alias = if is_main_package(package, main_package_name) {
        // Add Symbol aliases for exposing
        &i.exposing
    } else {
        // Add Symbol aliases for exports
        &i.export
    };

    let alias = match alias {
        Some(alias) => alias,
        None => return,
    };

    match alias {
        Export::All => todo!(),
        Export::Things(things) => {
            for thing in things {
                let alias = thing.alias.as_ref().unwrap_or(&thing.name).str();

                let symbol = i.module.symbol(thing.name.str(), arena);
                add_to_document_alias(document, arena, alias, fastn_section::SoM::Symbol(symbol));
            }
        }
    }
}

fn is_main_package(package: &Option<&fastn_package::Package>, main_package_name: &str) -> bool {
    match package {
        Some(package) => package.name == main_package_name,
        None => false,
    }
}

fn add_to_document_alias(
    document: &mut fastn_unresolved::Document,
    arena: &mut fastn_section::Arena,
    alias: &str,
    som: fastn_section::SoM,
) {
    match document.aliases {
        Some(id) => {
            arena
                .aliases
                .get_mut(id)
                .unwrap()
                .insert(alias.to_string(), som);
        }
        None => {
            let aliases = fastn_section::Aliases::from_iter([(alias.to_string(), som)]);
            document.aliases = Some(arena.aliases.alloc(aliases));
        }
    }
}

/// Validates that the import statement references a module in the current package or package's
/// dependencies.
fn validate_import_module_in_dependencies(
    section: fastn_section::Section,
    document: &mut fastn_unresolved::Document,
    arena: &mut fastn_section::Arena,
    package: &Option<&fastn_package::Package>,
    i: &Import,
) {
    // ensure that the import statement is for a module in dependency
    match package {
        Some(package) => {
            let imported_package_name = i.module.package(arena);

            // Check if the imported package exists in dependencies or matches the current package name
            let is_valid_import = package.dependencies.iter().any(|dep| {
                dep.name.as_str() == imported_package_name || dep.name.as_str() == package.name
            });

            if !is_valid_import {
                document.errors.push(
                    section
                        .init
                        .name
                        .wrap(fastn_section::Error::ImportPackageNotFound),
                );
            }
        }
        None => {
            document.errors.push(
                section
                    .init
                    .name
                    .wrap(fastn_section::Error::PackageNotFound),
            );
        }
    }
}

fn parse_import(
    section: &fastn_section::Section,
    document: &mut fastn_unresolved::Document,
    arena: &mut fastn_section::Arena,
) -> Option<Import> {
    let caption = match section.caption_as_plain_span() {
        Some(v) => v,
        None => {
            document.errors.push(
                section
                    .span()
                    .wrap(fastn_section::Error::ImportMustHaveCaption),
            );
            return None;
        }
    };

    // section.caption must be single text block, parsable as a module-name.
    //       module-name must be internally able to handle aliasing.
    let (raw_module, alias) = match caption.str().split_once(" as ") {
        Some((module, alias)) => (module, Some(alias)),
        None => (caption.str(), None),
    };

    let (package, module) = match raw_module.rsplit_once("/") {
        Some((package, module)) => (package, Some(module)),
        None => (raw_module, None),
    };

    // Determine the alias: prioritize explicit alias, fallback to module name, then package name
    let alias = alias
        .or(module)
        .unwrap_or_else(|| match package.rsplit_once(".") {
            Some((_, alias)) => alias,
            None => package,
        });

    Some(Import {
        module: if let Some(module) = module {
            fastn_section::Module::new(
                caption.inner_str(package).str(),
                Some(caption.inner_str(module).str()),
                arena,
            )
        } else {
            fastn_section::Module::new(caption.inner_str(package).str(), None, arena)
        },
        alias: fastn_section::Identifier {
            name: caption.inner_str(alias),
        },
        export: parse_field("export", section, document),
        exposing: parse_field("exposing", section, document),
    })
}

#[derive(Debug, Clone, PartialEq)]
pub enum Export {
    #[expect(unused)]
    All,
    Things(Vec<AliasableIdentifier>),
}

/// is this generic enough?
#[derive(Debug, Clone, PartialEq)]
pub struct AliasableIdentifier {
    pub alias: Option<fastn_section::Identifier>,
    pub name: fastn_section::Identifier,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub module: fastn_section::Module,
    pub alias: fastn_section::Identifier,
    pub export: Option<Export>,
    pub exposing: Option<Export>,
}

fn parse_field(
    field: &str,
    section: &fastn_section::Section,
    _document: &mut fastn_unresolved::Document,
) -> Option<Export> {
    let header = section.header_as_plain_span(field)?;

    Some(Export::Things(
        header
            .str()
            .split(",")
            .map(|v| aliasable(header, v.trim()))
            .collect(),
    ))
}

fn aliasable(span: &fastn_section::Span, s: &str) -> AliasableIdentifier {
    let (name, alias) = match s.split_once(" as ") {
        Some((name, alias)) => (
            span.inner_str(name).into(),
            Some(span.inner_str(alias).into()),
        ),
        None => (span.inner_str(s).into(), None),
    };

    AliasableIdentifier { name, alias }
}

#[cfg(test)]
mod tests {
    mod main_package {
        fastn_unresolved::tt!(super::import_in_main_package_function, super::tester);

        #[test]
        fn import() {
            // import without exposing or export
            t!("-- import: foo", { "import": "foo" });
            t!("-- import: foo.fifthtry.site/bar", { "import": "foo.fifthtry.site/bar=>bar" });
            t!("-- import: foo as f", { "import": "foo=>f" });

            // import with exposing
            t!("-- import: foo\nexposing: bar", { "import": "foo", "symbols": ["foo#bar"] });
            t!("-- import: foo as f\nexposing: bar", { "import": "foo=>f", "symbols": ["foo#bar"] });
            t!(
                "-- import: foo as f\nexposing: bar, moo",
                { "import": "foo=>f", "symbols": ["foo#bar", "foo#moo"] }
            );
            t!(
                "-- import: foo as f\nexposing: bar as b, moo",
                { "import": "foo=>f", "symbols": ["foo#bar=>b", "foo#moo"] }
            );

            // import with export
            t!("-- import: foo\nexport: bar", { "import": "foo" });

            // import with both exposing or export
            t!(
                "-- import: foo\nexposing: bar\nexport: moo",
                { "import": "foo", "symbols": ["foo#bar"] }
            );
        }
    }

    mod other_package {
        fastn_unresolved::tt!(super::import_in_other_package_function, super::tester);

        #[test]
        fn import() {
            // import without exposing or export
            t!("-- import: foo", { "import": "foo" });

            // import with export
            t!("-- import: foo\nexport: bar", { "import": "foo", "symbols": ["foo#bar"] });

            // import with exposing
            t!("-- import: foo\nexposing: bar", { "import": "foo" });

            // import with both exposing or export
            t!(
                "-- import: foo\nexposing: bar\nexport: moo",
                { "import": "foo", "symbols": ["foo#moo"] }
            );
        }
    }

    #[track_caller]
    fn tester(
        d: fastn_unresolved::Document,
        expected: serde_json::Value,
        arena: &fastn_section::Arena,
    ) {
        assert!(d.content.is_empty());
        assert!(d.definitions.is_empty());
        assert!(d.aliases.is_some());

        assert_eq!(
            fastn_unresolved::JIDebug::idebug(&AliasesID(d.aliases.unwrap()), arena),
            expected
        )
    }

    fn import_in_main_package_function(
        section: fastn_section::Section,
        document: &mut fastn_unresolved::Document,
        arena: &mut fastn_section::Arena,
        _package: &Option<&fastn_package::Package>,
    ) {
        let package = fastn_package::Package {
            name: "main".to_string(),
            dependencies: vec![],
            auto_imports: vec![],
            favicon: None,
        };

        super::import(section, document, arena, &Some(&package), "main");
    }

    fn import_in_other_package_function(
        section: fastn_section::Section,
        document: &mut fastn_unresolved::Document,
        arena: &mut fastn_section::Arena,
        _package: &Option<&fastn_package::Package>,
    ) {
        let package = fastn_package::Package {
            name: "other".to_string(),
            dependencies: vec![],
            auto_imports: vec![],
            favicon: None,
        };

        super::import(section, document, arena, &Some(&package), "main");
    }

    mod fail {

        fastn_unresolved::tt!(super::import_in_other_package_function, super::tester);
        #[test]
        #[should_panic]
        fn failing_tests() {
            t!("-- import: foo as f\nexposing: x", { "import": "foo=>f", "exposing": ["x"] });
            t!("-- import: foo\nexposing: x", { "import": "foo", "exposing": ["x"] });
            t!("-- import: foo\nexposing: x, y, z", { "import": "foo", "exposing": ["x", "y", "z"] });
            t!("-- import: foo as f\nexposing: x as y", { "import": "foo as f", "exposing": ["x=>y"] });
            t!("-- import: foo as f\nexposing: x as y, z", { "import": "foo as f", "exposing": ["x=>y", "z"] });
            t!("-- import: foo as f\nexport: x", { "import": "foo=>f", "export": ["x"] });
            t!("-- import: foo\nexport: x", { "import": "foo", "export": ["x"] });
            t!("-- import: foo\nexport: x, y, z", { "import": "foo", "export": ["x", "y", "z"] });
            t!("-- import: foo as f\nexport: x as y", { "import": "foo as f", "export": ["x=>y"] });
            t!("-- import: foo as f\nexport: x as y, z", { "import": "foo as f", "export": ["x=>y", "z"] });
            t!("-- import: foo as f\nexport: x\nexposing: y", { "import": "foo=>f", "export": ["x"], "exposing": ["y"] });
            t!("-- import: foo\nexport: x\nexposing: y", { "import": "foo", "export": ["x"], "exposing": ["y"] });
            t!("-- import: foo\nexport: x, y, z\nexposing: y", { "import": "foo", "export": ["x", "y", "z"], "exposing": ["y"] });
            t!("-- import: foo as f\nexport: x as y\nexposing: y", { "import": "foo as f", "export": ["x=>y"], "exposing": ["y"] });
            t!("-- import: foo as f\nexport: x as y, z\nexposing: y", { "import": "foo as f", "export": ["x=>y", "z"], "exposing": ["y"] });
        }
    }

    #[derive(Debug)]
    struct AliasesID(fastn_section::AliasesID);
    impl fastn_unresolved::JIDebug for AliasesID {
        fn idebug(&self, arena: &fastn_section::Arena) -> serde_json::Value {
            let aliases = arena.aliases.get(self.0).unwrap();
            let mut o = serde_json::Map::new();
            let mut symbols: Vec<String> = vec![];
            for (key, value) in aliases {
                match value {
                    fastn_section::SoM::Module(m) => {
                        let module_name = m.str(arena);
                        if module_name.eq("ftd") {
                            continue;
                        }

                        if module_name.eq(key) {
                            o.insert("import".into(), module_name.into());
                        } else {
                            o.insert("import".into(), format!("{module_name}=>{key}").into());
                        }
                    }
                    fastn_section::SoM::Symbol(s) => {
                        let symbol_name = s.str(arena).to_string();
                        if symbol_name.ends_with(format!("#{key}").as_str()) {
                            symbols.push(symbol_name)
                        } else {
                            symbols.push(format!("{symbol_name}=>{key}"));
                        }
                    }
                }
            }

            if !symbols.is_empty() {
                symbols.sort();
                o.insert(
                    "symbols".into(),
                    serde_json::Value::Array(symbols.into_iter().map(Into::into).collect()),
                );
            }

            serde_json::Value::Object(o)
        }
    }
}
