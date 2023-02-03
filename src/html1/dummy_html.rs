pub(crate) struct DummyHtmlGenerator<'a> {
    pub id: String,
    pub doc: &'a ftd::interpreter2::TDoc<'a>,
}

impl<'a> DummyHtmlGenerator<'a> {
    pub fn new(id: &str, doc: &'a ftd::interpreter2::TDoc<'a>) -> DummyHtmlGenerator<'a> {
        DummyHtmlGenerator {
            id: id.to_string(),
            doc,
        }
    }

    pub fn as_string_from_dummy_nodes(
        &self,
        dummy_nodes: &ftd::Map<ftd::node::DummyNode>,
    ) -> String {
        let mut dummy_dependency = "".to_string();
        for (dependency, dummy_node) in dummy_nodes {
            dummy_dependency = format!(
                "{}\n{}",
                dummy_dependency,
                self.as_string_from_dummy_node(dummy_node, dependency)
            )
        }
        if dummy_dependency.trim().is_empty() {
            "".to_string()
        } else {
            format!(
                "window.dummy_data_{} = {{}};\n{}",
                self.id, dummy_dependency
            )
        }
    }

    pub fn as_string_from_dummy_node(
        &self,
        dummy_node: &ftd::node::DummyNode,
        dependency: &str,
    ) -> String {
        let dummy_html = ftd::html1::RawHtmlGenerator::from_node(
            self.id.as_str(),
            self.doc,
            dummy_node.main.to_owned(),
        );

        if let Some(iteration) = dummy_html.iteration {
            format!(
                indoc::indoc! {"
                    window.dummy_data_{id}[\"{dependency}\"] = function(all_data, index) {{
                        function dummy_data(list, all_data, index) {{
                            let new_data = {{
                                \"{alias}\": list[index],
                                \"LOOP__COUNTER\": index
                            }};
                            let data = {{...new_data, ...all_data}};
                            {arguments}
                            data = {{...args, ...all_data}};
                            if (!!\"{node}\".trim() && !!window[\"raw_nodes_{id}\"] && !!window.raw_nodes_{id}[\"{node}\"]) {{
                                data[\"{node}\"] = window.raw_nodes_{id}[\"{node}\"](data);
                            }}
                            return \"{html}\".replace_format(data);
                        }}
                        
                        let list = resolve_reference(\"{dependency}\", all_data);
                        if (index !== null && index !== undefined) {{
                            if (index.toString().toUpperCase() === \"LAST\") {{
                                index = list.length - 1;
                            }} else if (index.toString().toUpperCase() === \"START\") {{
                                index = 0;
                            }}
                           return [dummy_data(list, all_data, index), \"{data_id}\", {start_index}];
                        }}
                        let htmls = [];
                        for (var i = 0; i < list.length; i++) {{
                            htmls.push(dummy_data(list, all_data, i));
                         }}
                         return [htmls, \"{data_id}\", {start_index}];
                    }}"
                },
                dependency = dependency,
                alias = iteration.alias,
                arguments = dummy_html.properties_string.unwrap_or_default(),
                node = dummy_html.name,
                html = dummy_html.html,
                id = self.id,
                data_id = ftd::html1::utils::full_data_id(
                    self.id.as_str(),
                    ftd::executor::utils::get_string_container(
                        dummy_node.parent_container.as_slice()
                    )
                    .as_str()
                ),
                start_index = dummy_node.start_index
            )
        } else {
            format!(
                indoc::indoc! {"
                    window.dummy_data_{id}[\"{dependency}\"] = function(all_data){{
                        {arguments}
                        let data = {{...args, ...all_data}};
                        if (!!\"{node}\".trim() && !!window[\"raw_nodes_{id}\"] && !!window.raw_nodes_{id}[\"{node}\"]) {{
                            data[\"{node}\"] = window.raw_nodes_{id}[\"{node}\"](data);
                        }}
                        let html = \"{html}\".replace_format(data);
                        return [html, \"{data_id}\", {start_index}]
                    }}"
                },
                dependency = dependency,
                arguments = dummy_html.properties_string.unwrap_or_default(),
                node = dummy_html.name,
                html = dummy_html.html,
                id = self.id,
                data_id = ftd::html1::utils::full_data_id(
                    self.id.as_str(),
                    ftd::executor::utils::get_string_container(
                        dummy_node.parent_container.as_slice()
                    )
                    .as_str()
                ),
                start_index = dummy_node.start_index
            )
        }
    }
}

pub(crate) struct HelperHtmlGenerator<'a> {
    pub id: String,
    pub doc: &'a ftd::interpreter2::TDoc<'a>,
}

impl<'a> HelperHtmlGenerator<'a> {
    pub fn new(id: &str, doc: &'a ftd::interpreter2::TDoc<'a>) -> HelperHtmlGenerator<'a> {
        HelperHtmlGenerator {
            id: id.to_string(),
            doc,
        }
    }

    pub fn as_string_from_raw_nodes(&self, raw_nodes: &ftd::Map<ftd::node::RawNode>) -> String {
        let mut raw_dependency = "".to_string();
        for (dependency, raw_node) in raw_nodes {
            raw_dependency = format!(
                "{}\n{}",
                raw_dependency,
                self.as_string_from_raw_node(raw_node, dependency)
            )
        }
        if raw_dependency.trim().is_empty() {
            "".to_string()
        } else {
            format!("window.raw_nodes_{} = {{}};\n{}", self.id, raw_dependency)
        }
    }

    pub fn as_string_from_raw_node(
        &self,
        raw_node: &ftd::node::RawNode,
        dependency: &str,
    ) -> String {
        let raw_html = ftd::html1::RawHtmlGenerator::from_node(
            self.id.as_str(),
            self.doc,
            raw_node.node.to_owned(),
        );

        let argument_string = ftd::html1::utils::to_argument_string(
            self.id.as_str(),
            raw_node.arguments.as_slice(),
            self.doc,
            dependency,
        );

        format!(
            indoc::indoc! {"
                window.raw_nodes_{id}[\"{dependency}\"] = function(all_data){{
                    {arguments}
                    let data = {{...args, ...all_data}};
                    if (!!\"{node}\".trim() && !!window[\"raw_nodes_{id}\"] && !!window.raw_nodes_{id}[\"{node}\"]) {{
                        data[\"{node}\"] = window.raw_nodes_{id}[\"{node}\"](data);
                    }}
                    let html = \"{html}\".replace_format(data);
                    return html;
                }}"
            },
            dependency = dependency,
            arguments = argument_string.unwrap_or_default(),
            node = raw_html.name,
            html = raw_html.html.replace('\"', "\\\""),
            id = self.id,
        )
    }
}
