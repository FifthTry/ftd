pub struct DependencyGenerator<'a> {
    pub id: &'a str,
    pub node: &'a ftd::node::Node,
    pub doc: &'a ftd::interpreter2::TDoc<'a>,
}

impl<'a> DependencyGenerator<'a> {
    pub(crate) fn new(
        id: &'a str,
        node: &'a ftd::node::Node,
        doc: &'a ftd::interpreter2::TDoc,
    ) -> DependencyGenerator<'a> {
        DependencyGenerator { id, node, doc }
    }

    pub(crate) fn get_dependencies(&self) -> ftd::html1::Result<String> {
        let dependencies = self.get_dependencies_()?;
        if dependencies.trim().is_empty() {
            return Ok("".to_string());
        }
        Ok(format!(
            indoc::indoc! {"
                            function node_change_{id}(data){{
                                {dependencies}
                            }}
        
                        "},
            id = self.id,
            dependencies = dependencies.trim(),
        ))
    }

    fn get_dependencies_(&self) -> ftd::html1::Result<String> {
        let node_data_id = ftd::html1::utils::full_data_id(self.id, self.node.data_id.as_str());
        let mut result = vec![];
        let default = self
            .node
            .text
            .properties
            .iter()
            .find(|v| v.condition.is_none());
        if let Some(default) = default {
            if let Some(value_string) = self.get_formatted_dep_string_from_property_value(
                &default.value,
                &self.node.text.pattern,
            )? {
                let value = format!(
                    "document.querySelector(`[data-id=\"{}\"]`).innerHTML = {};",
                    node_data_id, value_string
                );
                result.push(value);
            }
        }

        for (key, attribute) in self.node.attrs.iter() {
            let default = attribute.properties.iter().find(|v| v.condition.is_none());
            if let Some(default) = default {
                if let Some(value_string) = self.get_formatted_dep_string_from_property_value(
                    &default.value,
                    &attribute.pattern,
                )? {
                    let value = format!(
                        "document.querySelector(`[data-id=\"{}\"]`).setAttribute(\"{}\", {});",
                        node_data_id, key, value_string
                    );
                    result.push(value);
                }
            }
        }

        for (key, attribute) in self.node.style.iter() {
            let default = attribute.properties.iter().find(|v| v.condition.is_none());
            if let Some(default) = default {
                if let Some(value_string) = self.get_formatted_dep_string_from_property_value(
                    &default.value,
                    &attribute.pattern,
                )? {
                    let value = format!(
                        "document.querySelector(`[data-id=\"{}\"]`).style[\"{}\"] = {};",
                        node_data_id, key, value_string
                    );
                    result.push(value);
                }
            }
        }

        for children in self.node.children.iter() {
            let value =
                DependencyGenerator::new(self.id, children, self.doc).get_dependencies_()?;
            if !value.trim().is_empty() {
                result.push(value.trim().to_string());
            }
        }
        Ok(result.join("\n"))
    }

    fn get_formatted_dep_string_from_property_value(
        &self,
        property_value: &ftd::interpreter2::PropertyValue,
        pattern: &Option<String>,
    ) -> ftd::html1::Result<Option<String>> {
        let value_string = match property_value {
            ftd::interpreter2::PropertyValue::Reference { name, .. } => {
                format!("data[\"{}\"]", name)
            }
            ftd::interpreter2::PropertyValue::FunctionCall(function_call) => {
                let action = serde_json::to_string(&ftd::html1::Action::from_function_call(
                    function_call,
                    self.id,
                    self.doc,
                )?)
                .unwrap();
                format!(
                    "window.ftd.handle_function(event, '{}', '{}', this)",
                    self.id, action
                )
            }
            _ => return Ok(None),
        };

        Ok(Some(match pattern {
            Some(p) => format!("\"{}\".format({})", p, value_string),
            None => value_string,
        }))
    }
}
