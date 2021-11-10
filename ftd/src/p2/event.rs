#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Event {
    // $event-click$: toggle foo
    // will be parsed into this Event struct
    pub name: EventName, // click
    pub action: Action,
}

impl Event {
    fn to_value(
        property: &std::collections::BTreeMap<String, Vec<ftd::PropertyValue>>,
        arguments: &std::collections::BTreeMap<String, crate::Value>,
        doc: &crate::p2::TDoc,
        root_name: Option<&str>,
    ) -> ftd::p1::Result<std::collections::BTreeMap<String, Vec<String>>> {
        let mut property_string: std::collections::BTreeMap<String, Vec<String>> =
            Default::default();
        for (s, property_values) in property {
            let mut property_values_string = vec![];
            for property_value in property_values {
                let value = property_value.resolve_with_root(arguments, doc, root_name)?;
                if let Some(v) = value.to_string() {
                    property_values_string.push(v);
                } else {
                    return ftd::e(format!("Can't convert value to string {:?}", value));
                }
            }
            property_string.insert(s.to_string(), property_values_string);
        }
        Ok(property_string)
    }

    pub fn get_events(
        events: &[Self],
        all_locals: &mut ftd_rt::Map,
        arguments: &std::collections::BTreeMap<String, crate::Value>,
        doc: &crate::p2::TDoc,
        root_name: Option<&str>,
        mouse_event: bool,
    ) -> crate::p1::Result<Vec<ftd_rt::Event>> {
        let arguments = {
            //remove properties
            let mut arguments_without_properties: std::collections::BTreeMap<String, crate::Value> =
                Default::default();
            for (k, v) in arguments {
                if let Some(k) = k.strip_prefix('$') {
                    arguments_without_properties.insert(k.to_string(), v.to_owned());
                }
            }
            arguments_without_properties
        };

        let mut event: Vec<ftd_rt::Event> = vec![];
        for e in events {
            let target = match e.action.target.strip_prefix('@') {
                Some(value) => {
                    if let Some(val) = all_locals.get(value) {
                        format!("@{}@{}", value, val)
                    } else if value.eq("mouse-in") {
                        let string_container = all_locals.get("mouse-in-temp").unwrap().clone();
                        all_locals.insert("mouse-in".to_string(), string_container.to_string());
                        format!("@mouse-in@{}", string_container)
                    } else {
                        return crate::e(format!("Can't find the local variable {}", value));
                    }
                }
                None => e.action.target.to_string(),
            };

            event.push(ftd_rt::Event {
                name: e.name.to_str().to_string(),
                action: ftd_rt::Action {
                    action: e.action.action.to_str().to_string(),
                    target,
                    parameters: ftd::p2::Event::to_value(
                        &e.action.parameters,
                        &arguments,
                        doc,
                        root_name,
                    )?,
                },
            });
        }
        if mouse_event {
            if let Some(val) = all_locals.get("mouse-in") {
                event.push(ftd_rt::Event {
                    name: "onmouseenter".to_string(),
                    action: ftd_rt::Action {
                        action: "set-value".to_string(),
                        target: format!("@mouse-in@{}", val),
                        parameters: std::array::IntoIter::new([(
                            "value".to_string(),
                            vec!["true".to_string(), "boolean".to_string()],
                        )])
                        .collect(),
                    },
                });
                event.push(ftd_rt::Event {
                    name: "onmouseleave".to_string(),
                    action: ftd_rt::Action {
                        action: "set-value".to_string(),
                        target: format!("@mouse-in@{}", val),
                        parameters: std::array::IntoIter::new([(
                            "value".to_string(),
                            vec!["false".to_string(), "boolean".to_string()],
                        )])
                        .collect(),
                    },
                });
            }
        }
        Ok(event)
    }
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub enum EventName {
    OnClick,
}

impl EventName {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::OnClick => "onclick",
        }
    }

    pub fn from_string(s: &str) -> ftd::p1::Result<Self> {
        match s {
            "click" => Ok(Self::OnClick),
            t => return crate::e(format!("{} is not a valid event", t)),
        }
    }
}

impl Event {
    pub fn to_event(
        event_name: &str,
        action: &str,
        doc: &crate::p2::TDoc,
        locals: &std::collections::BTreeMap<String, crate::p2::Kind>,
        arguments: &std::collections::BTreeMap<String, crate::p2::Kind>,
    ) -> ftd::p1::Result<Self> {
        let event_name = EventName::from_string(event_name)?;
        let action = Action::to_action(action, doc, locals, arguments)?;
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
    pub action: ActionKind, // toggle
    pub target: String,     // foo
    pub parameters: std::collections::BTreeMap<String, Vec<ftd::PropertyValue>>,
}

#[derive(Debug, PartialEq, Clone, serde::Deserialize)]
pub enum ActionKind {
    Toggle,
    Increment,
    Decrement,
    StopPropagation,
    PreventDefault,
    SetValue,
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
            Self::Toggle => "toggle",
            Self::Increment => "increment",
            Self::Decrement => "decrement",
            Self::StopPropagation => "stop-propagation",
            Self::PreventDefault => "prevent-default",
            Self::SetValue => "set-value",
        }
    }

    pub fn from_string(s: &str) -> ftd::p1::Result<Self> {
        match s {
            "toggle" => Ok(Self::Toggle),
            "increment" => Ok(Self::Increment),
            "decrement" => Ok(Self::Decrement),
            "stop-propagation" => Ok(Self::StopPropagation),
            "prevent-default" => Ok(Self::PreventDefault),
            "set-value" => Ok(Self::SetValue),
            t => return crate::e(format!("{} is not a valid action kind", t)),
        }
    }

    pub fn parameters(&self) -> std::collections::BTreeMap<String, ftd::p2::event::Parameter> {
        let mut parameters: std::collections::BTreeMap<String, ftd::p2::event::Parameter> =
            Default::default();
        match self {
            ftd::p2::ActionKind::Toggle
            | ftd::p2::ActionKind::StopPropagation
            | ftd::p2::ActionKind::PreventDefault
            | ftd::p2::ActionKind::SetValue => {}
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
        }
        parameters
    }
}

impl Action {
    fn to_action(
        a: &str,
        doc: &crate::p2::TDoc,
        locals: &std::collections::BTreeMap<String, crate::p2::Kind>,
        arguments: &std::collections::BTreeMap<String, crate::p2::Kind>,
    ) -> ftd::p1::Result<Self> {
        let a: String = a.split_whitespace().collect::<Vec<&str>>().join(" ");
        return match a {
            _ if a.starts_with("toggle ") => {
                let value = a.replace("toggle ", "");
                let target = get_target(
                    value,
                    doc,
                    locals,
                    arguments,
                    Some(ftd::p2::Kind::boolean()),
                )?
                .0;
                Ok(Self {
                    action: ActionKind::Toggle,
                    target,
                    parameters: Default::default(),
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
                    return crate::e2(
                        "target not found",
                        &format!("expected `{} something` found: {}", action_string, a),
                    );
                };
                let target = get_target(
                    value,
                    doc,
                    locals,
                    arguments,
                    Some(ftd::p2::Kind::integer()),
                )?
                .0;

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
                                return crate::e(format!(
                                    "minumum number of arguments for {} are {}, found: {}",
                                    current_parameter, min, idx
                                ));
                            }
                            current_parameter = parameter.to_string();
                            min = p.min;
                            max = p.max;
                            pkind = p.ptype.to_vec();
                            idx = 0;
                            parameters.insert(current_parameter.to_string(), vec![]);
                        } else if let Some(p) = parameters.get_mut(&current_parameter) {
                            if idx >= max {
                                return crate::e(format!(
                                    "maximum number of arguments for {} are {}, found: {}",
                                    current_parameter,
                                    max,
                                    max + 1
                                ));
                            }
                            p.push(ftd::PropertyValue::resolve_value(
                                parameter,
                                pkind.get(idx).map(|k| k.to_owned()),
                                doc,
                                arguments,
                                locals,
                                None,
                                false,
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
            _ if a.eq("stop-propagation") => Ok(Self {
                action: ActionKind::StopPropagation,
                target: "".to_string(),
                parameters: Default::default(),
            }),
            _ if a.eq("prevent-default") => Ok(Self {
                action: ActionKind::PreventDefault,
                target: "".to_string(),
                parameters: Default::default(),
            }),
            _ if a.contains('=') => {
                let (part_1, part_2) = ftd::p2::utils::split(a, "=")?;
                let (target, kind) = get_target(part_1, doc, locals, arguments, None)?;
                let mut parameters: std::collections::BTreeMap<String, Vec<ftd::PropertyValue>> =
                    Default::default();

                let value = ftd::PropertyValue::resolve_value(
                    &part_2,
                    Some(kind.clone()),
                    doc,
                    arguments,
                    locals,
                    None,
                    false,
                )?;
                let kind = ftd::PropertyValue::Value {
                    value: ftd::variable::Value::String {
                        text: kind.to_string()?,
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
            t => return crate::e(format!("{} is not a valid action", t)),
        };

        fn get_target(
            value: String,
            doc: &crate::p2::TDoc,
            locals: &std::collections::BTreeMap<String, crate::p2::Kind>,
            arguments: &std::collections::BTreeMap<String, crate::p2::Kind>,
            kind: Option<crate::p2::Kind>,
        ) -> ftd::p1::Result<(String, ftd::p2::Kind)> {
            let pv = ftd::PropertyValue::resolve_value(
                &value, kind, doc, arguments, locals, None, false,
            )?;
            Ok((
                match pv {
                    ftd::PropertyValue::Reference { ref name, .. } => name.to_string(),
                    ftd::PropertyValue::LocalVariable { ref name, .. } => format!("@{}", name),
                    t => return crate::e(format!("value not expected {:?}", t)),
                },
                pv.kind(),
            ))
        }
    }
}
