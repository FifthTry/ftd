pub struct HtmlUI {
    pub html: String,
    pub dependencies: String,
    pub variables: String,
    pub functions: String,
    pub variable_dependencies: String,
    pub outer_events: String,
    pub dummy_html: String,
}

impl HtmlUI {
    #[tracing::instrument(skip_all)]
    pub fn from_node_data(node_data: ftd::node::NodeData, id: &str) -> ftd::html1::Result<HtmlUI> {
        let tdoc = ftd::interpreter2::TDoc::new(
            node_data.name.as_str(),
            &node_data.aliases,
            &node_data.bag,
        );

        let functions = ftd::html1::FunctionGenerator::new(id).get_functions(&node_data)?;
        let (dependencies, var_dependencies) =
            ftd::html1::dependencies::DependencyGenerator::new(id, &node_data.node, &tdoc)
                .get_dependencies()?;
        let variable_dependencies = ftd::html1::VariableDependencyGenerator::new(id, &tdoc)
            .get_set_functions(&var_dependencies)?;
        let variables = ftd::html1::data::DataGenerator::new(&tdoc).get_data()?;
        let (html, outer_events) =
            HtmlGenerator::new(id, &tdoc).to_html_and_outer_events(node_data.node)?;

        /*for (dependency, dummy_node) in node_data.dummy_nodes {
            let dummy_html = RawHtmlGenerator::from_node(id, &tdoc, dummy_node.main);
            dbg!("dummy_nodes", &dependency, &dummy_html);
        }*/

        let dummy_html =
            ftd::html1::DummyHtmlGenerator::new(id, &tdoc).from_dummy_nodes(&node_data.dummy_nodes);
        dbg!(&dummy_html);

        for (dependency, raw_node) in node_data.raw_nodes {
            let raw_html = RawHtmlGenerator::from_node(id, &tdoc, raw_node.node);
            dbg!("raw_nodes", &dependency, &raw_html);
        }

        Ok(HtmlUI {
            html,
            dependencies,
            variables: serde_json::to_string_pretty(&variables)
                .expect("failed to convert document to json"),
            functions,
            variable_dependencies,
            outer_events,
            dummy_html,
        })
    }
}

#[derive(Debug, Default)]
pub(crate) struct RawHtmlGenerator {
    pub name: String,
    pub html: String,
    pub properties: Vec<(String, ftd::interpreter2::Property)>,
    pub properties_string: Option<String>,
    pub iteration: Option<ftd::interpreter2::Loop>,
    pub helper_html: ftd::Map<RawHtmlGenerator>,
    pub children: Vec<RawHtmlGenerator>,
}

impl RawHtmlGenerator {
    pub(crate) fn from_node(
        id: &str,
        doc: &ftd::interpreter2::TDoc,
        node: ftd::node::Node,
    ) -> RawHtmlGenerator {
        let mut dummy_html = Default::default();
        HtmlGenerator::new(id, doc).to_dummy_html(node, &mut dummy_html);
        dummy_html
    }
}

pub(crate) struct HtmlGenerator<'a> {
    pub id: String,
    pub doc: &'a ftd::interpreter2::TDoc<'a>,
}

impl<'a> HtmlGenerator<'a> {
    pub fn new(id: &str, doc: &'a ftd::interpreter2::TDoc<'a>) -> HtmlGenerator<'a> {
        HtmlGenerator {
            id: id.to_string(),
            doc,
        }
    }

    pub fn to_dummy_html(
        &self,
        node: ftd::node::Node,
        dummy_html: &mut RawHtmlGenerator,
    ) -> ftd::html1::Result<()> {
        if let Some(raw_data) = node.raw_data {
            dummy_html.iteration = raw_data.iteration;
            dummy_html.properties_string = ftd::html1::utils::to_properties_string(
                self.id.as_str(),
                raw_data.properties.as_slice(),
                self.doc,
                node.node.as_str(),
            );
            dummy_html.properties = raw_data.properties;
            dummy_html.html = node.node.to_string();
            dummy_html.name = node.node.to_string();
            for child in node.children {
                let mut child_dummy_html = Default::default();
                self.to_dummy_html(child, &mut child_dummy_html);
                dummy_html.children.push(child_dummy_html);
            }
        } else {
            let data = self.to_dummy_html_(node, dummy_html)?;
            dummy_html.html = data.0;
        }

        Ok(())
    }

    pub fn to_html_and_outer_events(
        &self,
        node: ftd::node::Node,
    ) -> ftd::html1::Result<(String, String)> {
        let (html, events) = self.to_html_(node)?;
        Ok((html, ftd::html1::utils::events_to_string(events)))
    }

    pub fn to_dummy_html_(
        &self,
        node: ftd::node::Node,
        dummy_html: &mut RawHtmlGenerator,
    ) -> ftd::html1::Result<(String, Vec<(String, String, String)>)> {
        if node.is_null() {
            return Ok(("".to_string(), vec![]));
        }

        if let Some(raw_data) = node.raw_data {
            let number = ftd::html1::utils::get_new_number(
                &dummy_html
                    .helper_html
                    .iter()
                    .map(|v| v.0.to_string())
                    .collect(),
                node.node.as_str(),
            );
            let node_name = format!("{}_{}", node.node, number);
            dummy_html
                .helper_html
                .insert(node_name.to_string(), Default::default());
            let helper_dummy_html = dummy_html.helper_html.get_mut(node_name.as_str()).unwrap();
            helper_dummy_html.iteration = raw_data.iteration;
            helper_dummy_html.properties_string = ftd::html1::utils::to_properties_string(
                self.id.as_str(),
                raw_data.properties.as_slice(),
                self.doc,
                node_name.as_str(),
            );
            helper_dummy_html.properties = raw_data.properties;
            helper_dummy_html.html = node_name.to_string();
            helper_dummy_html.name = node_name.to_string();
            for child in node.children {
                let mut child_dummy_html = Default::default();
                self.to_dummy_html(child, &mut child_dummy_html);
                dummy_html.children.push(child_dummy_html);
            }
            return Ok((node_name.to_string(), vec![]));
        }

        let style = format!(
            "style=\"{}\"",
            self.style_to_html(&node, /*self.visible*/ true)
        );
        let classes = self.class_to_html(&node);

        let mut outer_events = vec![];
        let attrs = {
            let mut attr = self.attrs_to_html(&node);
            let events = self.group_by_js_event(&node.events)?;
            for (name, actions) in events {
                if name.eq("onclickoutside") || name.starts_with("onglobalkey") {
                    let event = format!(
                        "window.ftd.handle_event(event, '{}', '{}', this)",
                        self.id, actions
                    );
                    outer_events.push((
                        ftd::html1::utils::full_data_id(self.id.as_str(), node.data_id.as_str()),
                        name,
                        event,
                    ));
                } else {
                    let event = format!(
                        "window.ftd.handle_event(event, '{}', '{}', this)",
                        self.id,
                        actions.replace('\"', "&quot;")
                    );
                    attr.push(' ');
                    attr.push_str(&format!("{}={}", name, quote(&event)));
                }
            }
            attr
        };

        let body = match node.text.value.as_ref() {
            Some(v) => v.to_string(),
            None => node
                .children
                .into_iter()
                .map(|v| match self.to_html_(v) {
                    Ok((html, events)) => {
                        outer_events.extend(events);
                        Ok(html)
                    }
                    Err(e) => Err(e),
                })
                .collect::<ftd::html1::Result<Vec<String>>>()?
                .join(""),
        };

        Ok((
            format!(
                "<{node} {attrs} {style} {classes}>{body}</{node}>",
                node = node.node.as_str(),
                attrs = attrs,
                style = style,
                classes = classes,
                body = body,
            ),
            outer_events,
        ))
    }

    #[allow(clippy::type_complexity)]
    pub fn to_html_(
        &self,
        node: ftd::node::Node,
    ) -> ftd::html1::Result<(String, Vec<(String, String, String)>)> {
        if node.is_null() {
            return Ok(("".to_string(), vec![]));
        }

        let style = format!(
            "style=\"{}\"",
            self.style_to_html(&node, /*self.visible*/ true)
        );
        let classes = self.class_to_html(&node);

        let mut outer_events = vec![];
        let attrs = {
            let mut attr = self.attrs_to_html(&node);
            let events = self.group_by_js_event(&node.events)?;
            for (name, actions) in events {
                if name.eq("onclickoutside") || name.starts_with("onglobalkey") {
                    let event = format!(
                        "window.ftd.handle_event(event, '{}', '{}', this)",
                        self.id, actions
                    );
                    outer_events.push((
                        ftd::html1::utils::full_data_id(self.id.as_str(), node.data_id.as_str()),
                        name,
                        event,
                    ));
                } else {
                    let event = format!(
                        "window.ftd.handle_event(event, '{}', '{}', this)",
                        self.id,
                        actions.replace('\"', "&quot;")
                    );
                    attr.push(' ');
                    attr.push_str(&format!("{}={}", name, quote(&event)));
                }
            }
            attr
        };

        let body = match node.text.value.as_ref() {
            Some(v) => v.to_string(),
            None => node
                .children
                .into_iter()
                .map(|v| match self.to_html_(v) {
                    Ok((html, events)) => {
                        outer_events.extend(events);
                        Ok(html)
                    }
                    Err(e) => Err(e),
                })
                .collect::<ftd::html1::Result<Vec<String>>>()?
                .join(""),
        };

        Ok((
            format!(
                "<{node} {attrs} {style} {classes}>{body}</{node}>",
                node = node.node.as_str(),
                attrs = attrs,
                style = style,
                classes = classes,
                body = body,
            ),
            outer_events,
        ))
    }

    pub fn style_to_html(&self, node: &ftd::node::Node, visible: bool) -> String {
        let mut styles: ftd::Map<String> = node
            .style
            .clone()
            .into_iter()
            .filter_map(|(k, v)| v.value.map(|v| (k, v)))
            .collect();
        if !visible {
            styles.insert("display".to_string(), "none".to_string());
        }
        styles
            .iter()
            .map(|(k, v)| format!("{}: {}", *k, escape(v))) // TODO: escape needed?
            .collect::<Vec<String>>()
            .join("; ")
    }

    pub fn class_to_html(&self, node: &ftd::node::Node) -> String {
        if node.classes.is_empty() {
            return "".to_string();
        }
        format!(
            "class=\"{}\"",
            node.classes
                .iter()
                .map(|k| k.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }

    fn attrs_to_html(&self, node: &ftd::node::Node) -> String {
        node.attrs
            .iter()
            .filter_map(|(k, v)| {
                if k.eq("class") {
                    return None;
                }
                v.value.as_ref().map(|v| {
                    if k.eq("checked") {
                        if v.eq("true") {
                            return s("checked");
                        }
                        return s("");
                    }
                    let v = if k.eq("data-id") {
                        ftd::html1::utils::full_data_id(self.id.as_str(), v)
                    } else {
                        v.to_string()
                    };
                    format!("{}={}", *k, quote(v.as_str()))
                })
            }) // TODO: escape needed?
            .collect::<Vec<String>>()
            .join(" ")
    }
}

fn s(s: &str) -> String {
    s.to_string()
}

pub fn escape(s: &str) -> String {
    let s = s.replace('>', "\\u003E");
    let s = s.replace('<', "\\u003C");
    s.replace('&', "\\u0026")
}

fn quote(i: &str) -> String {
    format!("{:?}", i)
}
