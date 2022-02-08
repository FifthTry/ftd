#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Event {
    // $event-click$: toggle foo
    // will be parsed into this Event struct
    pub name: EventName, // click
    pub action: Action,
}

impl Event {
    fn to_value(
        line_number: usize,
        property: &std::collections::BTreeMap<String, Vec<ftd::PropertyValue>>,
        doc: &ftd::p2::TDoc,
    ) -> ftd::p1::Result<std::collections::BTreeMap<String, Vec<ftd::event::ParameterData>>> {
        let mut property_string: std::collections::BTreeMap<
            String,
            Vec<ftd::event::ParameterData>,
        > = Default::default();
        for (s, property_values) in property {
            let mut property_values_string = vec![];
            for property_value in property_values {
                let value = property_value.resolve(line_number, doc)?;
                let reference = get_reference(property_value, doc, line_number)?;
                if let Some(value) = value.to_string() {
                    property_values_string.push(ftd::event::ParameterData { value, reference });
                } else {
                    return ftd::e2(
                        format!("Can't convert value to string {:?}", value),
                        doc.name,
                        line_number,
                    );
                }
            }
            if !property_values_string.is_empty() {
                property_string.insert(s.to_string(), property_values_string);
            }
        }
        return Ok(property_string);

        fn get_reference(
            property_value: &ftd::PropertyValue,
            doc: &ftd::p2::TDoc,
            line_number: usize,
        ) -> ftd::p1::Result<Option<String>> {
            Ok(match property_value {
                ftd::PropertyValue::Reference { name, .. } => {
                    match doc.get_value(line_number, name)? {
                        ftd::Value::Object { values } => {
                            let mut val: std::collections::BTreeMap<String, String> =
                                Default::default();
                            for (k, v) in values.iter() {
                                if let Some(reference) = get_reference(v, doc, line_number)? {
                                    val.insert(k.to_string(), reference);
                                }
                            }
                            serde_json::to_string(&val).ok()
                        }
                        _ => Some(name.to_owned()),
                    }
                }
                _ => None,
            })
        }
    }

    pub fn get_events(
        line_number: usize,
        events: &[Self],
        doc: &ftd::p2::TDoc,
    ) -> ftd::p1::Result<Vec<ftd::Event>> {
        let mut event: Vec<ftd::Event> = vec![];
        for e in events {
            let target = match &e.action.target {
                ftd::PropertyValue::Value { value } => value.to_string().unwrap_or("".to_string()),
                ftd::PropertyValue::Reference { name, .. }
                | ftd::PropertyValue::Variable { name, .. } => name.to_string(),
            };

            event.push(ftd::Event {
                name: e.name.to_str().to_string(),
                action: ftd::Action {
                    action: e.action.action.to_str().to_string(),
                    target,
                    parameters: ftd::p2::Event::to_value(line_number, &e.action.parameters, doc)?,
                },
            });
        }
        Ok(event)
    }

    pub fn mouse_event(val: &str) -> Vec<ftd::Event> {
        let mut event: Vec<ftd::Event> = vec![];
        event.push(ftd::Event {
            name: "onmouseenter".to_string(),
            action: ftd::Action {
                action: "set-value".to_string(),
                target: val.to_string(),
                parameters: std::array::IntoIter::new([(
                    "value".to_string(),
                    vec![
                        ftd::event::ParameterData {
                            value: "true".to_string(),
                            reference: None,
                        },
                        ftd::event::ParameterData {
                            value: "boolean".to_string(),
                            reference: None,
                        },
                    ],
                )])
                .collect(),
            },
        });
        event.push(ftd::Event {
            name: "onmouseleave".to_string(),
            action: ftd::Action {
                action: "set-value".to_string(),
                target: val.to_string(),
                parameters: std::array::IntoIter::new([(
                    "value".to_string(),
                    vec![
                        ftd::event::ParameterData {
                            value: "false".to_string(),
                            reference: None,
                        },
                        ftd::event::ParameterData {
                            value: "boolean".to_string(),
                            reference: None,
                        },
                    ],
                )])
                .collect(),
            },
        });
        event
    }
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub enum EventName {
    OnClick,
    OnChange,
    OnInput,
    OnMouseEnter,
    OnMouseLeave,
}

impl EventName {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::OnClick => "onclick",
            Self::OnChange => "onchange",
            Self::OnInput => "oninput",
            Self::OnMouseEnter => "onmouseenter",
            Self::OnMouseLeave => "onmouseleave",
        }
    }

    pub fn from_string(s: &str, doc_id: &str) -> ftd::p1::Result<Self> {
        match s {
            "click" => Ok(Self::OnClick),
            "change" => Ok(Self::OnChange),
            "input" => Ok(Self::OnInput),
            "mouseenter" => Ok(Self::OnMouseEnter),
            "mouseleave" => Ok(Self::OnMouseLeave),
            t => return ftd::e2(format!("{} is not a valid event", t), doc_id, 0),
        }
    }
}

impl Event {
    pub fn to_event(
        line_number: usize,
        event_name: &str,
        action: &str,
        doc: &ftd::p2::TDoc,
        arguments: &std::collections::BTreeMap<String, ftd::p2::Kind>,
    ) -> ftd::p1::Result<Self> {
        let event_name = EventName::from_string(event_name, doc.name)?;
        let action = Action::to_action(line_number, action, doc, arguments)?;
        Ok(Self {
            name: event_name,
            action,
        })
    }
}

pub struct Parameter {
    pub min: usize,
    pub max: usize,
    pub ptype: Vec<ftd::p2::Kind>,
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Action {
    pub action: ActionKind,         // toggle
    pub target: ftd::PropertyValue, // foo
    pub parameters: std::collections::BTreeMap<String, Vec<ftd::PropertyValue>>,
}

#[derive(Debug, PartialEq, Clone, serde::Deserialize)]
pub enum ActionKind {
    Toggle,
    Insert,
    Clear,
    Increment,
    Decrement,
    StopPropagation,
    PreventDefault,
    SetValue,
    MessageHost,
}

impl serde::Serialize for ActionKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_str())
    }
}

impl ActionKind {
    pub fn to_str(&self) -> &'static str {
        match self {
            ftd::p2::ActionKind::Toggle => "toggle",
            ftd::p2::ActionKind::Increment => "increment",
            ftd::p2::ActionKind::Decrement => "decrement",
            ftd::p2::ActionKind::Insert => "insert",
            ftd::p2::ActionKind::StopPropagation => "stop-propagation",
            ftd::p2::ActionKind::PreventDefault => "prevent-default",
            ftd::p2::ActionKind::SetValue => "set-value",
            ftd::p2::ActionKind::MessageHost => "message-host",
            ftd::p2::ActionKind::Clear => "clear",
        }
    }

    // pub fn from_string(s: &str, doc_id: &str) -> ftd::p1::Result<Self> {
    //     match s {
    //         "toggle" => Ok(Self::Toggle),
    //         "increment" => Ok(Self::Increment),
    //         "decrement" => Ok(Self::Decrement),
    //         "stop-propagation" => Ok(Self::StopPropagation),
    //         "prevent-default" => Ok(Self::PreventDefault),
    //         "set-value" => Ok(Self::SetValue),
    //         t => return ftd::e2(format!("{} is not a valid action kind", t), doc_id),
    //     }
    // }

    pub fn parameters(&self) -> std::collections::BTreeMap<String, ftd::p2::event::Parameter> {
        let mut parameters: std::collections::BTreeMap<String, ftd::p2::event::Parameter> =
            Default::default();
        match self {
            ftd::p2::ActionKind::Toggle
            | ftd::p2::ActionKind::StopPropagation
            | ftd::p2::ActionKind::PreventDefault
            | ftd::p2::ActionKind::Clear
            | ftd::p2::ActionKind::SetValue => {}
            ftd::p2::ActionKind::MessageHost => {
                parameters.insert(
                    "data".to_string(),
                    ftd::p2::event::Parameter {
                        min: 1,
                        max: 1,
                        ptype: vec![ftd::p2::Kind::object()],
                    },
                );
            }
            ftd::p2::ActionKind::Increment | ftd::p2::ActionKind::Decrement => {
                parameters.insert(
                    "by".to_string(),
                    ftd::p2::event::Parameter {
                        min: 1,
                        max: 1,
                        ptype: vec![ftd::p2::Kind::integer()],
                    },
                );
                parameters.insert(
                    "clamp".to_string(),
                    ftd::p2::event::Parameter {
                        min: 1,
                        max: 2,
                        ptype: vec![ftd::p2::Kind::integer(), ftd::p2::Kind::integer()],
                    },
                );
            }
            ftd::p2::ActionKind::Insert => {
                parameters.insert(
                    "value".to_string(),
                    ftd::p2::event::Parameter {
                        min: 1,
                        max: 1,
                        ptype: vec![],
                    },
                );
                parameters.insert(
                    "at".to_string(),
                    ftd::p2::event::Parameter {
                        min: 1,
                        max: 1,
                        ptype: vec![ftd::p2::Kind::string()],
                    },
                );
            }
        }
        parameters
    }
}

impl Action {
    fn to_action(
        line_number: usize,
        a: &str,
        doc: &ftd::p2::TDoc,
        arguments: &std::collections::BTreeMap<String, ftd::p2::Kind>,
    ) -> ftd::p1::Result<Self> {
        let a: String = a.split_whitespace().collect::<Vec<&str>>().join(" ");
        return match a {
            _ if a.starts_with("toggle ") => {
                let value = a.replace("toggle ", "");
                let target = get_target(
                    line_number,
                    value,
                    doc,
                    arguments,
                    Some(ftd::p2::Kind::boolean()),
                )?;
                Ok(Self {
                    action: ActionKind::Toggle,
                    target,
                    parameters: Default::default(),
                })
            }
            _ if a.starts_with("clear ") => {
                let value = a.replace("clear ", "");
                let target = get_target(line_number, value, doc, arguments, None)?;
                let kind = target.kind();
                if !kind.is_list() && !kind.is_optional() {
                    return ftd::e2(
                        format!(
                            "clear should have target of kind: `list` or `optional`, found: {:?}",
                            kind
                        ),
                        doc.name,
                        line_number,
                    );
                }
                Ok(Self {
                    action: ActionKind::Clear,
                    target,
                    parameters: Default::default(),
                })
            }
            _ if a.starts_with("message-host") => {
                let value = a.replace("message-host", "").trim().to_string();
                let parameters = if value.starts_with('$') {
                    let mut parameters: std::collections::BTreeMap<
                        String,
                        Vec<ftd::PropertyValue>,
                    > = Default::default();
                    if let Some(p) = ActionKind::MessageHost.parameters().get("data") {
                        parameters.insert(
                            "data".to_string(),
                            vec![ftd::PropertyValue::resolve_value(
                                line_number,
                                value.as_str(),
                                p.ptype.get(0).map(|k| k.to_owned()),
                                doc,
                                arguments,
                                None,
                            )?],
                        );
                    }
                    parameters
                } else {
                    Default::default()
                };

                let target = ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: if value.is_empty() {
                            "ftd_message".to_string()
                        } else {
                            value
                        },
                        source: ftd::TextSource::Header,
                    },
                };

                Ok(Self {
                    action: ActionKind::MessageHost,
                    target,
                    parameters,
                })
            }
            _ if a.starts_with("increment ") || a.starts_with("decrement ") => {
                let (action_kind, action_string) = if a.starts_with("increment ") {
                    (ActionKind::Increment, "increment")
                } else {
                    (ActionKind::Decrement, "decrement")
                };

                let vector: Vec<&str> = a.split(' ').filter(|x| !x.is_empty()).collect();
                let value = if let Some(val) = vector.get(1) {
                    val.to_string()
                } else {
                    return ftd::e2(
                        format!(
                            "target not found, expected `{} something` found: {}",
                            action_string, a
                        ),
                        doc.name,
                        line_number,
                    );
                };
                let target = get_target(
                    line_number,
                    value,
                    doc,
                    arguments,
                    Some(ftd::p2::Kind::integer()),
                )?;

                let parameters = {
                    let mut parameters: std::collections::BTreeMap<
                        String,
                        Vec<ftd::PropertyValue>,
                    > = Default::default();
                    let mut current_parameter = "".to_string();
                    let (mut min, mut max, mut idx) = (0, 0, 0);
                    let mut pkind = vec![];
                    for parameter in vector[2..].iter() {
                        if let Some(p) = action_kind.parameters().get(*parameter) {
                            if min > idx {
                                return ftd::e2(
                                    format!(
                                        "minumum number of arguments for {} are {}, found: {}",
                                        current_parameter, min, idx
                                    ),
                                    doc.name,
                                    line_number,
                                );
                            }
                            current_parameter = parameter.to_string();
                            min = p.min;
                            max = p.max;
                            pkind = p.ptype.to_vec();
                            idx = 0;
                            parameters.insert(current_parameter.to_string(), vec![]);
                        } else if let Some(p) = parameters.get_mut(&current_parameter) {
                            if idx >= max {
                                return ftd::e2(
                                    format!(
                                        "maximum number of arguments for {} are {}, found: {}",
                                        current_parameter,
                                        max,
                                        max + 1
                                    ),
                                    doc.name,
                                    line_number,
                                );
                            }
                            p.push(ftd::PropertyValue::resolve_value(
                                line_number,
                                parameter,
                                pkind.get(idx).map(|k| k.to_owned()),
                                doc,
                                arguments,
                                None,
                            )?);
                            idx += 1;
                        }
                    }
                    parameters
                };

                Ok(Self {
                    action: action_kind,
                    target,
                    parameters,
                })
            }
            _ if a.starts_with("insert into ") => {
                let vector: Vec<&str> = a.split(' ').filter(|x| !x.is_empty()).collect();
                let value = if let Some(val) = vector.get(2) {
                    val.to_string()
                } else {
                    return ftd::e2(
                        format!(
                            "target not found, expected `insert into <something>` found: {}",
                            a
                        ),
                        doc.name,
                        line_number,
                    );
                };
                let target = get_target(line_number, value.clone(), doc, arguments, None)?;
                let kind = target.kind();
                let expected_value_kind = if let ftd::p2::Kind::List { kind, .. } = kind {
                    kind.as_ref().to_owned()
                } else {
                    return ftd::e2(
                        format!(
                            "expected target `{}` kind is list found: `{:?}`",
                            value, kind
                        ),
                        doc.name,
                        line_number,
                    );
                };
                let parameters = {
                    let mut parameters: std::collections::BTreeMap<
                        String,
                        Vec<ftd::PropertyValue>,
                    > = Default::default();
                    let mut current_parameter = "".to_string();
                    let (mut min, mut max, mut idx) = (0, 0, 0);
                    let mut pkind = vec![];
                    for parameter in vector[3..].iter() {
                        if let Some(p) = ActionKind::Insert.parameters().get(*parameter) {
                            if min > idx {
                                return ftd::e2(
                                    format!(
                                        "minumum number of arguments for {} are {}, found: {}",
                                        current_parameter, min, idx
                                    ),
                                    doc.name,
                                    line_number,
                                );
                            }
                            current_parameter = parameter.to_string();
                            min = p.min;
                            max = p.max;
                            pkind = p.ptype.to_vec();
                            idx = 0;
                            parameters.insert(current_parameter.to_string(), vec![]);
                        } else if let Some(p) = parameters.get_mut(&current_parameter) {
                            if idx >= max {
                                return ftd::e2(
                                    format!(
                                        "maximum number of arguments for {} are {}, found: {}",
                                        current_parameter,
                                        max,
                                        max + 1
                                    ),
                                    doc.name,
                                    line_number,
                                );
                            }
                            let value = if parameter.eq(&"$VALUE") {
                                ftd::PropertyValue::Value {
                                    value: ftd::variable::Value::String {
                                        text: parameter.to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                }
                            } else {
                                ftd::PropertyValue::resolve_value(
                                    line_number,
                                    parameter,
                                    pkind.get(idx).map(|k| k.to_owned()),
                                    doc,
                                    arguments,
                                    None,
                                )?
                            };
                            if !value.kind().inner().eq(&expected_value_kind) {
                                return ftd::e2(
                                    format!(
                                        "expected value kind: `{:?}` found: `{:?}`",
                                        value.kind(),
                                        expected_value_kind
                                    ),
                                    doc.name,
                                    line_number,
                                );
                            }
                            p.push(value);
                            idx += 1;
                        }
                    }
                    parameters
                };

                Ok(Self {
                    action: ActionKind::Insert,
                    target,
                    parameters,
                })
            }
            _ if a.eq("stop-propagation") => Ok(Self {
                action: ActionKind::StopPropagation,
                target: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                parameters: Default::default(),
            }),
            _ if a.eq("prevent-default") => Ok(Self {
                action: ActionKind::PreventDefault,
                target: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                parameters: Default::default(),
            }),
            _ if a.contains('=') => {
                let (part_1, part_2) = ftd::p2::utils::split(a, "=")?;
                let target = get_target(line_number, part_1, doc, arguments, None)?;
                let kind = target.kind();
                let mut parameters: std::collections::BTreeMap<String, Vec<ftd::PropertyValue>> =
                    Default::default();

                let value = {
                    if part_2.eq("$VALUE") || part_2.eq("$MOUSE-IN") {
                        ftd::PropertyValue::Value {
                            value: ftd::variable::Value::String {
                                text: part_2,
                                source: ftd::TextSource::Header,
                            },
                        }
                    } else {
                        ftd::PropertyValue::resolve_value(
                            line_number,
                            &part_2,
                            Some(kind.clone()),
                            doc,
                            arguments,
                            None,
                        )?
                    }
                };
                let kind = ftd::PropertyValue::Value {
                    value: ftd::variable::Value::String {
                        text: kind.to_string(line_number, doc.name)?,
                        source: ftd::TextSource::Header,
                    },
                };

                parameters.insert("value".to_string(), vec![value, kind]);
                Ok(Self {
                    action: ActionKind::SetValue,
                    target,
                    parameters,
                })
            }
            t => {
                return ftd::e2(
                    format!("{} is not a valid action", t),
                    doc.name,
                    line_number,
                )
            }
        };

        fn get_target(
            line_number: usize,
            value: String,
            doc: &ftd::p2::TDoc,
            arguments: &std::collections::BTreeMap<String, ftd::p2::Kind>,
            kind: Option<ftd::p2::Kind>,
        ) -> ftd::p1::Result<ftd::PropertyValue> {
            ftd::PropertyValue::resolve_value(line_number, &value, kind, doc, arguments, None)
        }
    }
}
