use ftd::evalexpr::ContextWithMutableFunctions;

pub fn default_aliases() -> ftd::Map<String> {
    std::iter::IntoIterator::into_iter([("ftd".to_string(), "ftd".to_string())]).collect()
}

pub fn default_functions() -> ftd::Map<ftd::evalexpr::Function> {
    use ftd::evalexpr::*;

    std::iter::IntoIterator::into_iter([(
        "isempty".to_string(),
        Function::new(|argument| {
            if argument.as_empty().is_ok() {
                Ok(Value::Boolean(true))
            } else if let Ok(s) = argument.as_string() {
                Ok(Value::Boolean(s.is_empty()))
            } else if let Ok(s) = argument.as_tuple() {
                Ok(Value::Boolean(s.is_empty()))
            } else {
                Ok(Value::Boolean(false)) //todo: throw error
            }
        }),
    )])
    .collect()
}

pub fn default_context() -> ftd::interpreter2::Result<ftd::evalexpr::HashMapContext> {
    let mut context = ftd::evalexpr::HashMapContext::new();
    for (key, function) in default_functions() {
        context.set_function(key, function)?;
    }
    Ok(context)
}

pub fn default_bag() -> ftd::Map<ftd::interpreter2::Thing> {
    let record = |n: &str, r: &str| (n.to_string(), ftd::interpreter2::Kind::record(r));
    let _color = |n: &str| record(n, "ftd#color");
    std::iter::IntoIterator::into_iter([
        (
            "ftd#row".to_string(),
            ftd::interpreter2::Thing::Component(row_function()),
        ),
        (
            "ftd#column".to_string(),
            ftd::interpreter2::Thing::Component(column_function()),
        ),
        (
            "ftd#text".to_string(),
            ftd::interpreter2::Thing::Component(markup_function()),
        ),
        (
            "ftd#integer".to_string(),
            ftd::interpreter2::Thing::Component(integer_function()),
        ),
        (
            "ftd#boolean".to_string(),
            ftd::interpreter2::Thing::Component(integer_function()),
        ),
    ])
    .collect()
}

pub fn boolean_function() -> ftd::interpreter2::ComponentDefinition {
    ftd::interpreter2::ComponentDefinition {
        name: "ftd#boolean".to_string(),
        arguments: [
            common_arguments(),
            vec![
                ftd::interpreter2::Argument::default(
                    "value",
                    ftd::interpreter2::Kind::boolean()
                        .into_kind_data()
                        .caption_or_body(),
                ),
                ftd::interpreter2::Argument::default(
                    "align",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "style",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "role",
                    ftd::interpreter2::Kind::record("ftd#type")
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "format",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "text-indent",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "text-align",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
            ],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: ftd::interpreter2::Component::from_name("ftd.kernel"),
        line_number: 0,
    }
}

pub fn integer_function() -> ftd::interpreter2::ComponentDefinition {
    ftd::interpreter2::ComponentDefinition {
        name: "ftd#integer".to_string(),
        arguments: [
            common_arguments(),
            vec![
                ftd::interpreter2::Argument::default(
                    "value",
                    ftd::interpreter2::Kind::integer()
                        .into_kind_data()
                        .caption_or_body(),
                ),
                ftd::interpreter2::Argument::default(
                    "align",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "style",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "role",
                    ftd::interpreter2::Kind::record("ftd#type")
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "format",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "text-indent",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "text-align",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
            ],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: ftd::interpreter2::Component::from_name("ftd.kernel"),
        line_number: 0,
    }
}

pub fn markup_function() -> ftd::interpreter2::ComponentDefinition {
    ftd::interpreter2::ComponentDefinition {
        name: "ftd#text".to_string(),
        arguments: [
            common_arguments(),
            vec![
                ftd::interpreter2::Argument::default(
                    "text",
                    ftd::interpreter2::Kind::string()
                        .into_kind_data()
                        .caption_or_body(),
                ),
                ftd::interpreter2::Argument::default(
                    "align",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "style",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "role",
                    ftd::interpreter2::Kind::record("ftd#type")
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "line-clamp",
                    ftd::interpreter2::Kind::integer()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "text-indent",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "text-align",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
            ],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: ftd::interpreter2::Component::from_name("ftd.kernel"),
        line_number: 0,
    }
}

pub fn row_function() -> ftd::interpreter2::ComponentDefinition {
    ftd::interpreter2::ComponentDefinition {
        name: "ftd#row".to_string(),
        arguments: [
            container_arguments(),
            common_arguments(),
            vec![ftd::interpreter2::Argument::default(
                "spacing",
                ftd::interpreter2::Kind::string()
                    .into_optional()
                    .into_kind_data(),
            )],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: ftd::interpreter2::Component::from_name("ftd.kernel"),
        line_number: 0,
    }
}

pub fn column_function() -> ftd::interpreter2::ComponentDefinition {
    ftd::interpreter2::ComponentDefinition {
        name: "ftd#column".to_string(),
        arguments: [
            container_arguments(),
            common_arguments(),
            vec![ftd::interpreter2::Argument::default(
                "spacing",
                ftd::interpreter2::Kind::string()
                    .into_optional()
                    .into_kind_data(),
            )],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: ftd::interpreter2::Component::from_name("ftd.kernel"),
        line_number: 0,
    }
}

fn container_arguments() -> Vec<ftd::interpreter2::Argument> {
    vec![
        ftd::interpreter2::Argument::default(
            "align",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "wrap",
            ftd::interpreter2::Kind::boolean()
                .into_optional()
                .into_kind_data(),
        ),
    ]
}

fn common_arguments() -> Vec<ftd::interpreter2::Argument> {
    vec![
        ftd::interpreter2::Argument::default(
            "padding",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "padding-vertical",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "padding-horizontal",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "padding-left",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "padding-right",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "padding-top",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "padding-bottom",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-top-radius",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-bottom-radius",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-left-radius",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-right-radius",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "width",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "min-width",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "max-width",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "height",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "min-height",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "max-height",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "region",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "color",
            ftd::interpreter2::Kind::record("ftd#color")
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "background-color",
            ftd::interpreter2::Kind::Record {
                name: "ftd#color".to_string(),
            }
            .into_optional()
            .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-color",
            ftd::interpreter2::Kind::record("ftd#color")
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-width",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-radius",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "id",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "overflow-x",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "overflow-y",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-top",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-bottom",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-left",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-right",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-top-color",
            ftd::interpreter2::Kind::record("ftd#color")
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-left-color",
            ftd::interpreter2::Kind::record("ftd#color")
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-right-color",
            ftd::interpreter2::Kind::record("ftd#color")
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-bottom-color",
            ftd::interpreter2::Kind::record("ftd#color")
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "margin-top",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "margin-bottom",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "margin-left",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "margin-right",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "link",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "submit",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "open-in-new-tab",
            ftd::interpreter2::Kind::boolean()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "sticky",
            ftd::interpreter2::Kind::boolean()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "top",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "bottom",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "left",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "right",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "cursor",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "anchor",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "gradient-direction",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "gradient-colors",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "shadow-offset-x",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "shadow-offset-y",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "shadow-blur",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "shadow-size",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "shadow-color",
            ftd::interpreter2::Kind::record("ftd#color")
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "background-image",
            ftd::interpreter2::Kind::record("ftd#image-src")
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "background-repeat",
            ftd::interpreter2::Kind::boolean()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "background-parallax",
            ftd::interpreter2::Kind::boolean()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "scale",
            ftd::interpreter2::Kind::decimal()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "scale-x",
            ftd::interpreter2::Kind::decimal()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "scale-y",
            ftd::interpreter2::Kind::decimal()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "rotate",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "move-up",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "move-down",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "move-left",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "move-right",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "position",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "z-index",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "slot",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "white-space",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-style",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "text-transform",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        /*ftd::interpreter2::Argument::default(
            "grid-column".to_string(),
            ftd::interpreter2::Kind::string().into_optional().into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "grid-row".to_string(),
            ftd::interpreter2::Kind::string().into_optional().into_kind_data(),
        ),*/
    ]
}

/*fn kernel_component() -> ftd::interpreter2::ComponentDefinition {
    ftd::interpreter2::ComponentDefinition {
        name: "ftd.kernel".to_string(),
        arguments: vec![],
        definition: ftd::interpreter2::Component {
            name: "ftd.kernel".to_string(),
            properties: vec![],
            iteration: Box::new(None),
            condition: Box::new(None),
            events: vec![],
            children: vec![],
            line_number: 0,
        },
        line_number: 0,
    }
}*/
