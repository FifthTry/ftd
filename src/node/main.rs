#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Node {
    pub classes: Vec<String>,
    pub events: Vec<Event>,
    pub node: String,
    pub condition: Option<ftd::interpreter2::Expression>,
    pub attrs: ftd::Map<ftd::node::Value>,
    pub style: ftd::Map<ftd::node::Value>,
    pub children: Vec<Node>,
    pub text: ftd::node::Value,
    pub null: bool,
    pub data_id: String,
    pub line_number: usize,
}

pub type Event = ftd::executor::Event;

impl Node {
    fn from_common(node: &str, common: &ftd::executor::Common, doc_id: &str) -> Node {
        Node {
            node: s(node),
            condition: common.condition.to_owned(),
            attrs: common.attrs(),
            style: common.style(doc_id, &mut []),
            children: vec![],
            text: Default::default(),
            classes: vec![],
            null: common.is_dummy,
            events: common.event.clone(),
            data_id: common.data_id.clone(),
            line_number: common.line_number,
        }
    }

    fn from_container(
        common: &ftd::executor::Common,
        container: &ftd::executor::Container,
        doc_id: &str,
    ) -> Node {
        use itertools::Itertools;

        let mut attrs = common.attrs();
        attrs.extend(container.attrs());
        let mut classes = container.add_class();
        let mut style = common.style(doc_id, &mut classes);
        style.extend(container.style(doc_id));

        let node = common.node();

        Node {
            node: s(node.as_str()),
            attrs,
            style,
            classes,
            condition: common.condition.to_owned(),
            text: Default::default(),
            children: container
                .children
                .iter()
                .map(|v| v.to_node(doc_id))
                .collect_vec(),
            null: common.is_dummy,
            events: common.event.clone(),
            data_id: common.data_id.to_string(),
            line_number: common.line_number,
        }
    }

    pub(crate) fn is_null(&self) -> bool {
        self.null
    }
}

impl ftd::executor::Element {
    pub fn to_node(&self, doc_id: &str) -> Node {
        match self {
            ftd::executor::Element::Row(r) => r.to_node(doc_id),
            ftd::executor::Element::Column(c) => c.to_node(doc_id),
            ftd::executor::Element::Text(t) => t.to_node(doc_id),
            ftd::executor::Element::Integer(t) => t.to_node(doc_id),
            ftd::executor::Element::Boolean(t) => t.to_node(doc_id),
            ftd::executor::Element::Image(i) => i.to_node(doc_id),
            ftd::executor::Element::Null => Node {
                classes: vec![],
                events: vec![],
                node: "".to_string(),
                condition: None,
                attrs: Default::default(),
                style: Default::default(),
                children: vec![],
                text: Default::default(),
                null: true,
                data_id: "".to_string(),
                line_number: 0,
            },
        }
    }
}

impl ftd::executor::Row {
    pub fn to_node(&self, doc_id: &str) -> Node {
        use ftd::node::utils::CheckMap;

        let mut n = Node::from_container(&self.common, &self.container, doc_id);
        if !self.common.is_not_visible {
            n.style
                .insert(s("display"), ftd::node::Value::from_string("flex"));
        }

        n.style
            .insert(s("flex-direction"), ftd::node::Value::from_string("row"));

        n.style.insert(
            s("align-items"),
            ftd::node::Value::from_string("flex-start"),
        );

        n.style.insert(
            s("justify-content"),
            ftd::node::Value::from_string("flex-start"),
        );

        n.style.check_and_insert(
            "justify-content",
            ftd::node::Value::from_executor_value(
                Some(
                    self.container
                        .align
                        .to_owned()
                        .map(|v| v.to_css_justify_content(true))
                        .value,
                ),
                self.container.align.to_owned(),
                Some(ftd::executor::Alignment::justify_content_pattern(true)),
                doc_id,
            ),
        );

        n.style.check_and_insert(
            "align-items",
            ftd::node::Value::from_executor_value(
                Some(
                    self.container
                        .align
                        .to_owned()
                        .map(|v| v.to_css_align_items(true))
                        .value,
                ),
                self.container.align.to_owned(),
                Some(ftd::executor::Alignment::align_item_pattern(true)),
                doc_id,
            ),
        );

        n.children = {
            let mut children = vec![];
            for child in self.container.children.iter() {
                let child_node = child.to_node(doc_id);
                children.push(child_node);
            }
            children
        };
        n
    }
}

impl ftd::executor::Column {
    pub fn to_node(&self, doc_id: &str) -> Node {
        use ftd::node::utils::CheckMap;

        let mut n = Node::from_container(&self.common, &self.container, doc_id);
        if !self.common.is_not_visible {
            n.style
                .insert(s("display"), ftd::node::Value::from_string("flex"));
        }
        n.style
            .insert(s("flex-direction"), ftd::node::Value::from_string("column"));

        n.style.insert(
            s("align-items"),
            ftd::node::Value::from_string("flex-start"),
        );

        n.style.insert(
            s("justify-content"),
            ftd::node::Value::from_string("flex-start"),
        );

        n.style.check_and_insert(
            "justify-content",
            ftd::node::Value::from_executor_value(
                Some(
                    self.container
                        .align
                        .to_owned()
                        .map(|v| v.to_css_justify_content(false))
                        .value,
                ),
                self.container.align.to_owned(),
                Some(ftd::executor::Alignment::justify_content_pattern(false)),
                doc_id,
            ),
        );

        n.style.check_and_insert(
            "align-items",
            ftd::node::Value::from_executor_value(
                Some(
                    self.container
                        .align
                        .to_owned()
                        .map(|v| v.to_css_align_items(false))
                        .value,
                ),
                self.container.align.to_owned(),
                Some(ftd::executor::Alignment::align_item_pattern(false)),
                doc_id,
            ),
        );

        n.children = {
            let mut children = vec![];
            for child in self.container.children.iter() {
                let child_node = child.to_node(doc_id);
                children.push(child_node);
            }
            children
        };
        n
    }
}

impl ftd::executor::Text {
    pub fn to_node(&self, doc_id: &str) -> Node {
        let node = self.common.node();
        let mut n = Node::from_common(node.as_str(), &self.common, doc_id);
        n.classes.extend(self.common.add_class());
        n.classes.push("ft_md".to_string());
        n.text = ftd::node::Value::from_executor_value(
            Some(self.text.value.rendered.to_string()),
            self.text.clone(),
            None,
            doc_id,
        );
        n
    }
}

impl ftd::executor::Image {
    pub fn to_node(&self, doc_id: &str) -> Node {
        let node = s("img");
        let mut n = Node::from_common(node.as_str(), &self.common, doc_id);
        n.classes.extend(self.common.add_class());
        n.attrs.insert(
            s("src"),
            ftd::node::Value::from_executor_value(
                Some(self.src.value.light.value.to_string()),
                self.src.to_owned(),
                None,
                doc_id,
            ),
        );
        n
    }
}

impl ftd::executor::Common {
    fn attrs(&self) -> ftd::Map<ftd::node::Value> {
        // TODO: Implement attributes
        std::iter::IntoIterator::into_iter([(
            "data-id".to_string(),
            ftd::node::Value::from_string(self.data_id.as_str()),
        )])
        .collect()
    }

    fn style(&self, doc_id: &str, _classes: &mut [String]) -> ftd::Map<ftd::node::Value> {
        use ftd::node::utils::CheckMap;

        let mut d: ftd::Map<ftd::node::Value> = Default::default();

        d.check_and_insert("text-decoration", ftd::node::Value::from_string("none"));

        if self.is_not_visible {
            d.check_and_insert("display", ftd::node::Value::from_string("none"));
        }

        d.check_and_insert(
            "padding",
            ftd::node::Value::from_executor_value(
                self.padding.value.as_ref().map(|v| v.to_css_string()),
                self.padding.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "padding-left",
            ftd::node::Value::from_executor_value(
                self.padding_horizontal
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string()),
                self.padding_horizontal.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "padding-right",
            ftd::node::Value::from_executor_value(
                self.padding_horizontal
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string()),
                self.padding_horizontal.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "padding-top",
            ftd::node::Value::from_executor_value(
                self.padding_vertical
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string()),
                self.padding_vertical.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "padding-bottom",
            ftd::node::Value::from_executor_value(
                self.padding_vertical
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string()),
                self.padding_vertical.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "padding-top",
            ftd::node::Value::from_executor_value(
                self.padding_top.value.as_ref().map(|v| v.to_css_string()),
                self.padding_top.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "padding-bottom",
            ftd::node::Value::from_executor_value(
                self.padding_bottom
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string()),
                self.padding_bottom.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "padding-left",
            ftd::node::Value::from_executor_value(
                self.padding_left.value.as_ref().map(|v| v.to_css_string()),
                self.padding_left.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "padding-right",
            ftd::node::Value::from_executor_value(
                self.padding_right.value.as_ref().map(|v| v.to_css_string()),
                self.padding_right.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-style",
            ftd::node::Value::from_executor_value(
                Some(s("solid")),
                ftd::executor::Value::new(None::<String>, None, vec![]),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-width",
            ftd::node::Value::from_executor_value(
                Some(
                    self.border_width
                        .to_owned()
                        .map(|v| v.to_css_string())
                        .value,
                ),
                self.border_width.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-radius",
            ftd::node::Value::from_executor_value(
                self.border_radius.value.as_ref().map(|v| v.to_css_string()),
                self.border_radius.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-top-left-radius",
            ftd::node::Value::from_executor_value(
                self.border_top_left_radius
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string()),
                self.border_top_left_radius.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-top-right-radius",
            ftd::node::Value::from_executor_value(
                self.border_top_right_radius
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string()),
                self.border_top_right_radius.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-bottom-left-radius",
            ftd::node::Value::from_executor_value(
                self.border_bottom_left_radius
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string()),
                self.border_bottom_left_radius.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-bottom-right-radius",
            ftd::node::Value::from_executor_value(
                self.border_bottom_right_radius
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string()),
                self.border_bottom_right_radius.to_owned(),
                None,
                doc_id,
            ),
        );

        d
    }

    fn add_class(&self) -> Vec<String> {
        // TODO: Implement add_class
        Default::default()
    }

    fn node(&self) -> String {
        s("div")
    }
}

impl ftd::executor::Container {
    fn attrs(&self) -> ftd::Map<ftd::node::Value> {
        // TODO: Implement attributes
        Default::default()
    }

    fn add_class(&self) -> Vec<String> {
        // TODO: Implement add_class
        Default::default()
    }

    fn style(&self, doc_id: &str) -> ftd::Map<ftd::node::Value> {
        use ftd::node::utils::CheckMap;

        let mut d: ftd::Map<ftd::node::Value> = Default::default();

        d.check_and_insert(
            "gap",
            ftd::node::Value::from_executor_value(
                self.spacing.value.as_ref().map(|v| v.to_css_string()),
                self.spacing.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "flex-wrap",
            ftd::node::Value::from_executor_value(
                self.wrap
                    .value
                    .as_ref()
                    .map(|v| ftd::node::utils::wrap_to_css(*v)),
                self.wrap.to_owned(),
                Some((s("if ({0}) {\"wrap\"} else {\"nowrap\"}"), true)),
                doc_id,
            ),
        );

        d
    }
}

fn s(s: &str) -> String {
    s.to_string()
}
