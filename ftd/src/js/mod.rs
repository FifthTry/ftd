#![allow(dead_code)]

#[cfg(test)]
#[macro_use]
mod ftd_test_helpers;
mod element;
mod utils;
mod value;

pub use element::{Common, Element};
pub use value::Value;

pub fn document_into_js_ast(document: ftd::interpreter::Document) -> Vec<fastn_js::Ast> {
    use itertools::Itertools;
    let doc = ftd::interpreter::TDoc::new(&document.name, &document.aliases, &document.data);
    let mut asts = vec![ftd::js::from_tree(document.tree.as_slice(), &doc)];
    let default_thing_name = ftd::interpreter::default::default_bag()
        .into_iter()
        .map(|v| v.0)
        .collect_vec();

    for (key, thing) in document.data.iter() {
        if default_thing_name.contains(key) {
            continue;
        }
        if let ftd::interpreter::Thing::Component(c) = thing {
            asts.push(c.to_ast(&doc));
        } else if let ftd::interpreter::Thing::Variable(v) = thing {
            asts.push(v.to_ast(&doc));
        } else if let ftd::interpreter::Thing::Function(f) = thing {
            asts.push(f.to_ast());
        }
    }
    asts
}

impl ftd::interpreter::Function {
    pub fn to_ast(&self) -> fastn_js::Ast {
        use itertools::Itertools;

        fastn_js::udf_with_params(
            self.name.as_str(),
            self.expression
                .iter()
                .map(|e| {
                    fastn_grammar::evalexpr::build_operator_tree(e.expression.as_str()).unwrap()
                })
                .collect_vec(),
            self.arguments
                .iter()
                .map(|v| v.name.to_string())
                .collect_vec(),
        )
    }
}

impl ftd::interpreter::Variable {
    pub fn to_ast(&self, doc: &ftd::interpreter::TDoc) -> fastn_js::Ast {
        if let Ok(value) = self.value.value(doc.name, self.value.line_number()) {
            if self.kind.is_record() {
                let record = doc
                    .get_record(self.name.as_str(), self.line_number)
                    .unwrap();
                let record_fields = value
                    .record_fields(doc.name, self.value.line_number())
                    .unwrap();
                let mut fields = vec![];
                for field in record.fields {
                    if let Some(value) = record_fields.get(field.name.as_str()) {
                        fields.push((field.name.to_string(), value.to_fastn_js_value()));
                    } else {
                        fields.push((
                            field.name.to_string(),
                            field
                                .get_default_value()
                                .unwrap()
                                .to_set_property_value_with_none(),
                        ));
                    }
                }
                return fastn_js::Ast::RecordInstance(fastn_js::RecordInstance {
                    name: self.name.to_string(),
                    fields: fastn_js::SetPropertyValue::Value(fastn_js::Value::Record { fields }),
                });
            } else if self.kind.is_list() {
                // Todo: It should be only for Mutable not Static
                return fastn_js::Ast::MutableList(fastn_js::MutableList {
                    name: self.name.to_string(),
                    value: self.value.to_fastn_js_value(),
                });
            } else if self.mutable {
                return fastn_js::Ast::MutableVariable(fastn_js::MutableVariable {
                    name: self.name.to_string(),
                    value: self.value.to_fastn_js_value(),
                });
            }
        }
        fastn_js::Ast::StaticVariable(fastn_js::StaticVariable {
            name: self.name.to_string(),
            value: self.value.to_fastn_js_value(),
        })
    }
}

impl ftd::interpreter::ComponentDefinition {
    pub fn to_ast(&self, doc: &ftd::interpreter::TDoc) -> fastn_js::Ast {
        use itertools::Itertools;

        let mut statements = vec![];
        statements.extend(self.definition.to_component_statements(
            "parent",
            0,
            doc,
            Some(self.name.to_string()),
            true,
        ));
        fastn_js::component_with_params(
            self.name.as_str(),
            statements,
            self.arguments
                .iter()
                .map(|v| v.name.to_string())
                .collect_vec(),
        )
    }
}

pub fn from_tree(
    tree: &[ftd::interpreter::Component],
    doc: &ftd::interpreter::TDoc,
) -> fastn_js::Ast {
    let mut statements = vec![];
    for (index, component) in tree.iter().enumerate() {
        statements.extend(component.to_component_statements("parent", index, doc, None, false))
    }
    fastn_js::component0("main", statements)
}

impl ftd::interpreter::Component {
    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        component_definition_name: Option<String>,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        use itertools::Itertools;

        let loop_alias = self.iteration.clone().map(|v| v.alias);
        let mut component_statements = if self.is_loop() || self.condition.is_some() {
            self.to_component_statements_(
                "root",
                0,
                doc,
                component_definition_name.clone(),
                true,
                loop_alias.clone(),
            )
        } else {
            self.to_component_statements_(
                parent,
                index,
                doc,
                component_definition_name.clone(),
                should_return,
                None,
            )
        };

        if let Some(condition) = self.condition.as_ref() {
            component_statements = vec![fastn_js::ComponentStatement::ConditionalComponent(
                fastn_js::ConditionalComponent {
                    deps: condition
                        .references
                        .values()
                        .flat_map(|v| {
                            v.get_deps(component_definition_name.clone(), loop_alias.clone())
                        })
                        .collect_vec(),
                    condition: condition.update_node_with_variable_reference_js(
                        component_definition_name,
                        loop_alias,
                    ),
                    statements: component_statements,
                    parent: parent.to_string(),
                    should_return: self.is_loop() || should_return,
                },
            )]
        }

        if let Some(iteration) = self.iteration.as_ref() {
            component_statements = vec![fastn_js::ComponentStatement::ForLoop(fastn_js::ForLoop {
                list_variable: iteration.on.to_fastn_js_value(),
                statements: component_statements,
                parent: parent.to_string(),
                should_return,
            })]
        }

        component_statements
    }

    fn to_component_statements_(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        component_definition_name: Option<String>,
        should_return: bool,
        loop_alias: Option<String>,
    ) -> Vec<fastn_js::ComponentStatement> {
        use itertools::Itertools;
        if ftd::js::element::is_kernel(self.name.as_str()) {
            ftd::js::Element::from_interpreter_component(self, doc).to_component_statements(
                parent,
                index,
                doc,
                component_definition_name,
                loop_alias,
                should_return,
            )
        } else if let Ok(component_definition) =
            doc.get_component(self.name.as_str(), self.line_number)
        {
            let arguments = component_definition
                .arguments
                .iter()
                .map(|v| {
                    v.get_value(self.properties.as_slice())
                        .to_set_property_value(
                            component_definition_name.clone(),
                            loop_alias.clone(),
                        )
                })
                .collect_vec();
            // Todo: Add event
            /*for event in self.events.iter() {
                component_statements.push(fastn_js::ComponentStatement::AddEventHandler(
                    event.to_event_handler_js(element_name, doc, component_definition_name.clone()),
                ));
            }*/
            vec![fastn_js::ComponentStatement::InstantiateComponent(
                fastn_js::InstantiateComponent {
                    name: self.name.to_string(),
                    arguments,
                    parent: parent.to_string(),
                    should_return,
                },
            )]
        } else {
            panic!("Can't find, {}", self.name)
        }
    }
}
