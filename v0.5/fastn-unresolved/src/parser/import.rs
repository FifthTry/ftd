pub(super) fn import(
    source: &str,
    section: fastn_section::Section,
    document: &mut fastn_unresolved::Document,
) {
    if let Some(ref kind) = section.init.name.kind {
        document
            .errors
            .push(kind.span().wrap(fastn_section::Error::ImportCantHaveType));
        // we will go ahead with this import statement parsing
    }

    // section.name must be exactly import.
    if section.name(source) != "import" {
        document.errors.push(
            section
                .init
                .name
                .name
                .name
                .wrap(fastn_section::Error::ImportMustBeImport),
        );
        // we will go ahead with this import statement parsing
    }

    let i = match parse_import(&section, source, document) {
        Some(v) => v,
        None => {
            // error handling is job of parse_module_name().
            return;
        }
    };

    // ensure there are no extra headers, children or body
    fastn_unresolved::utils::assert_no_body(&section, document);
    fastn_unresolved::utils::assert_no_children(&section, document);
    fastn_unresolved::utils::assert_no_extra_headers(
        source,
        &section,
        document,
        &["exports", "exposing"],
    );
    document.imports.push(i);
}

fn parse_import(
    section: &fastn_section::Section,
    source: &str,
    document: &mut fastn_unresolved::Document,
) -> Option<fastn_unresolved::Import> {
    let caption = match section.caption_as_plain_string(source) {
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
    let (module, alias) = match caption.split_once(" as ") {
        Some((module, alias)) => (module, Some(alias)),
        None => (caption, None),
    };

    let (package, module) = match module.split_once("/") {
        Some((package, module)) => (package, module),
        None => ("", module),
    };

    Some(fastn_unresolved::Import {
        package: fastn_unresolved::PackageName(package.to_string()),
        module: fastn_unresolved::ModuleName(module.to_string()),
        alias: alias.map(|v| fastn_unresolved::Identifier(v.to_string())),
        export: parse_field("export", source, section, document),
        exposing: parse_field("exposing", source, section, document),
    })
}

fn parse_field(
    field: &str,
    source: &str,
    section: &fastn_section::Section,
    _document: &mut fastn_unresolved::Document,
) -> Option<fastn_unresolved::Export> {
    let header = match section.header_as_plain_string(field, source) {
        Some(v) => v,
        None => return dbg!(None),
    };

    Some(fastn_unresolved::Export::Things(
        header.split(",").map(aliasable).collect(),
    ))
}

fn aliasable(s: &str) -> fastn_unresolved::AliasableIdentifier {
    let (name, alias) = match s.split_once(" as ") {
        Some((name, alias)) => (name.into(), Some(alias.into())),
        None => (s.into(), None),
    };

    fastn_unresolved::AliasableIdentifier { name, alias }
}

#[cfg(test)]
mod tests {
    fn tester(
        mut d: fastn_unresolved::Document,
        source: &str,
        expected: serde_json::Value,
    ) -> bool {
        assert!(d.content.is_empty());
        assert!(d.definitions.is_empty());
        assert_eq!(d.imports.len(), 1);

        fastn_section::JDebug::debug(&d.imports.pop().unwrap(), source) == expected
    }

    fastn_unresolved::tt!(super::import, tester);

    #[test]
    fn import() {
        t!("-- import: foo", { "import": "foo" });
        t!("-- import: foo.fifthtry.site/bar", { "import": "foo.fifthtry.site/bar" });
        t!("-- import: foo as f", { "import": "foo=>f" });
    }

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
