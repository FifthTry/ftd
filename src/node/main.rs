#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Node {
    pub classes: Vec<String>,
    pub events: Vec<Event>,
    pub node: String,
    pub display: String,
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
    fn from_common(
        node: &str,
        display: &str,
        common: &ftd::executor::Common,
        doc_id: &str,
    ) -> Node {
        Node {
            node: s(node),
            display: s(display),
            condition: common.condition.to_owned(),
            attrs: common.attrs(doc_id),
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
        display: &str,
    ) -> Node {
        use itertools::Itertools;

        let mut attrs = common.attrs(doc_id);
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
            display: s(display),
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
                display: "".to_string(),
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

        let mut n = Node::from_container(&self.common, &self.container, doc_id, "flex");
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
                        .align_content
                        .to_owned()
                        .map(|v| v.to_css_justify_content(true))
                        .value,
                ),
                self.container.align_content.to_owned(),
                Some(ftd::executor::Alignment::justify_content_pattern(true)),
                doc_id,
            ),
        );

        n.style.upsert(
            "justify-content",
            ftd::node::Value::from_executor_value(
                self.container
                    .spacing_mode
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.container.spacing_mode.to_owned(),
                None,
                doc_id,
            ),
        );

        n.style.check_and_insert(
            "align-items",
            ftd::node::Value::from_executor_value(
                Some(
                    self.container
                        .align_content
                        .to_owned()
                        .map(|v| v.to_css_align_items(true))
                        .value,
                ),
                self.container.align_content.to_owned(),
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

        let mut n = Node::from_container(&self.common, &self.container, doc_id, "flex");
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
                        .align_content
                        .to_owned()
                        .map(|v| v.to_css_justify_content(false))
                        .value,
                ),
                self.container.align_content.to_owned(),
                Some(ftd::executor::Alignment::justify_content_pattern(false)),
                doc_id,
            ),
        );

        n.style.upsert(
            "justify-content",
            ftd::node::Value::from_executor_value(
                self.container
                    .spacing_mode
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.container.spacing_mode.to_owned(),
                None,
                doc_id,
            ),
        );

        n.style.check_and_insert(
            "align-items",
            ftd::node::Value::from_executor_value(
                Some(
                    self.container
                        .align_content
                        .to_owned()
                        .map(|v| v.to_css_align_items(false))
                        .value,
                ),
                self.container.align_content.to_owned(),
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
        use ftd::node::utils::CheckMap;

        let node = self.common.node();
        let mut n = Node::from_common(node.as_str(), "block", &self.common, doc_id);

        n.style.check_and_insert(
            "text-align",
            ftd::node::Value::from_executor_value(
                self.text_align
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.text_align.to_owned(),
                None,
                doc_id,
            ),
        );
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
        return if self.common.link.value.is_some() {
            let mut n = Node::from_common("a", "block", &self.common, doc_id);
            n.attrs.insert(
                s("data-id"),
                ftd::node::Value::from_string(format!("{}:parent", self.common.data_id).as_str()),
            );

            let img = update_img(self, doc_id);
            n.children.push(img);
            n
        } else {
            update_img(self, doc_id)
        };

        fn update_img(image: &ftd::executor::Image, doc_id: &str) -> Node {
            let mut n = Node::from_common("img", "block", &image.common, doc_id);
            n.classes.extend(image.common.add_class());
            n.attrs.insert(
                s("src"),
                ftd::node::Value::from_executor_value(
                    Some(image.src.value.light.value.to_string()),
                    image.src.to_owned(),
                    None,
                    doc_id,
                ),
            );
            n
        }
    }
}

impl ftd::executor::Common {
    fn attrs(&self, doc_id: &str) -> ftd::Map<ftd::node::Value> {
        use ftd::node::utils::CheckMap;

        let mut d: ftd::Map<ftd::node::Value> = Default::default();

        d.check_and_insert(
            "data-id",
            ftd::node::Value::from_string(self.data_id.as_str()),
        );

        d.check_and_insert(
            "href",
            ftd::node::Value::from_executor_value(
                self.link.value.as_ref().map(ToString::to_string),
                self.link.to_owned(),
                None,
                doc_id,
            ),
        );

        d
    }

    fn style(&self, doc_id: &str, _classes: &mut [String]) -> ftd::Map<ftd::node::Value> {
        use ftd::node::utils::CheckMap;

        let mut d: ftd::Map<ftd::node::Value> = Default::default();

        d.check_and_insert("text-decoration", ftd::node::Value::from_string("none"));

        if self.is_not_visible {
            d.check_and_insert("display", ftd::node::Value::from_string("none"));
        }

        d.check_and_insert("box-sizing", ftd::node::Value::from_string("border-box"));

        d.check_and_insert(
            "width",
            ftd::node::Value::from_executor_value(
                Some(self.width.to_owned().map(|v| v.to_css_string()).value),
                self.width.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "align-self",
            ftd::node::Value::from_executor_value(
                self.align_self
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.align_self.to_owned(),
                None,
                doc_id,
            ),
        );

        // html and css name only
        d.check_and_insert(
            "overflow",
            ftd::node::Value::from_executor_value(
                self.overflow
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.overflow.to_owned(),
                None,
                doc_id,
            ),
        );

        // html and css name only
        d.check_and_insert(
            "overflow-x",
            ftd::node::Value::from_executor_value(
                self.overflow_x
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.overflow_x.to_owned(),
                None,
                doc_id,
            ),
        );

        // html and css name only
        d.check_and_insert(
            "overflow-y",
            ftd::node::Value::from_executor_value(
                self.overflow_y
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.overflow_y.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "background-color",
            ftd::node::Value::from_executor_value(
                self.background
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.background.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "color",
            ftd::node::Value::from_executor_value(
                self.color
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.color.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-color",
            ftd::node::Value::from_executor_value(
                self.border_color
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.border_color.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "height",
            ftd::node::Value::from_executor_value(
                Some(self.height.to_owned().map(|v| v.to_css_string()).value),
                self.height.to_owned(),
                None,
                doc_id,
            ),
        );

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
            "margin",
            ftd::node::Value::from_executor_value(
                self.margin.value.as_ref().map(|v| v.to_css_string()),
                self.margin.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "margin-left",
            ftd::node::Value::from_executor_value(
                self.margin_horizontal
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string()),
                self.margin_horizontal.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "margin-right",
            ftd::node::Value::from_executor_value(
                self.margin_horizontal
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string()),
                self.margin_horizontal.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "margin-top",
            ftd::node::Value::from_executor_value(
                self.margin_vertical
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string()),
                self.margin_vertical.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "margin-bottom",
            ftd::node::Value::from_executor_value(
                self.margin_vertical
                    .value
                    .as_ref()
                    .map(|v| v.to_css_string()),
                self.margin_vertical.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "margin-top",
            ftd::node::Value::from_executor_value(
                self.margin_top.value.as_ref().map(|v| v.to_css_string()),
                self.margin_top.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "margin-bottom",
            ftd::node::Value::from_executor_value(
                self.margin_bottom.value.as_ref().map(|v| v.to_css_string()),
                self.margin_bottom.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "margin-left",
            ftd::node::Value::from_executor_value(
                self.margin_left.value.as_ref().map(|v| v.to_css_string()),
                self.margin_left.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "margin-right",
            ftd::node::Value::from_executor_value(
                self.margin_right.value.as_ref().map(|v| v.to_css_string()),
                self.margin_right.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "min-width",
            ftd::node::Value::from_executor_value(
                self.min_width.value.as_ref().map(|v| v.to_css_string()),
                self.min_width.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "max-width",
            ftd::node::Value::from_executor_value(
                self.max_width.value.as_ref().map(|v| v.to_css_string()),
                self.max_width.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "min-height",
            ftd::node::Value::from_executor_value(
                self.min_height.value.as_ref().map(|v| v.to_css_string()),
                self.min_height.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "max-height",
            ftd::node::Value::from_executor_value(
                self.max_height.value.as_ref().map(|v| v.to_css_string()),
                self.max_height.to_owned(),
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
            "border-bottom-width",
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
            "border-top-width",
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
            "border-left-width",
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
            "border-right-width",
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
            "border-bottom-width",
            ftd::node::Value::from_executor_value(
                self.border_bottom_width
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.border_bottom_width.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-bottom-color",
            ftd::node::Value::from_executor_value(
                self.border_bottom_color
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.border_bottom_color.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-top-width",
            ftd::node::Value::from_executor_value(
                self.border_top_width
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.border_top_width.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-top-color",
            ftd::node::Value::from_executor_value(
                self.border_top_color
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.border_top_color.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-left-width",
            ftd::node::Value::from_executor_value(
                self.border_left_width
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.border_left_width.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-left-color",
            ftd::node::Value::from_executor_value(
                self.border_left_color
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.border_left_color.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-right-width",
            ftd::node::Value::from_executor_value(
                self.border_right_width
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.border_right_width.to_owned(),
                None,
                doc_id,
            ),
        );

        d.check_and_insert(
            "border-right-color",
            ftd::node::Value::from_executor_value(
                self.border_right_color
                    .to_owned()
                    .map(|v| v.map(|v| v.to_css_string()))
                    .value,
                self.border_right_color.to_owned(),
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
        if self.link.value.is_some() {
            s("a")
        } else {
            s("div")
        }
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
