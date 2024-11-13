use ftd::interpreter::FieldExt;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct WebComponentDefinition {
    pub name: String,
    pub arguments: Vec<fastn_type::Argument>,
    pub js: fastn_type::PropertyValue,
    pub line_number: usize,
}

impl WebComponentDefinition {
    pub(crate) fn new(
        name: &str,
        arguments: Vec<fastn_type::Argument>,
        js: fastn_type::PropertyValue,
        line_number: usize,
    ) -> WebComponentDefinition {
        WebComponentDefinition {
            name: name.to_string(),
            arguments,
            js,
            line_number,
        }
    }

    pub(crate) fn scan_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        let web_component_definition = ast.get_web_component_definition(doc.name)?;

        fastn_type::Argument::scan_ast_fields(
            web_component_definition.arguments,
            doc,
            &Default::default(),
        )?;

        Ok(())
    }

    pub(crate) fn from_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<WebComponentDefinition>> {
        use ftd::interpreter::PropertyValueExt;

        let web_component_definition = ast.get_web_component_definition(doc.name)?;
        let name = doc.resolve_name(web_component_definition.name.as_str());

        let js = try_ok_state!(fastn_type::PropertyValue::from_ast_value(
            ftd_ast::VariableValue::String {
                line_number: web_component_definition.line_number(),
                value: web_component_definition.js,
                source: ftd_ast::ValueSource::Default,
                condition: None
            },
            doc,
            false,
            Some(&fastn_type::Kind::string().into_kind_data()),
        )?);

        let arguments = try_ok_state!(fastn_type::Argument::from_ast_fields(
            web_component_definition.name.as_str(),
            web_component_definition.arguments,
            doc,
            &Default::default(),
        )?);

        Ok(ftd::interpreter::StateWithThing::new_thing(
            WebComponentDefinition::new(
                name.as_str(),
                arguments,
                js,
                web_component_definition.line_number,
            ),
        ))
    }
}
