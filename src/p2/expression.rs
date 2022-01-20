#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Boolean {
    // if: $caption is not null
    IsNotNull {
        value: ftd::PropertyValue,
    },
    // if: $caption is null
    IsNull {
        value: ftd::PropertyValue,
    },
    // if: $list is not empty
    IsNotEmpty {
        value: ftd::PropertyValue,
    },
    // if: $list is empty
    IsEmpty {
        value: ftd::PropertyValue,
    },
    // if: $caption == hello | if: $foo
    Equal {
        left: ftd::PropertyValue,
        right: ftd::PropertyValue,
    },
    // if: $caption != hello
    NotEqual {
        left: ftd::PropertyValue,
        right: ftd::PropertyValue,
    },
    // if: not $show_something
    Not {
        of: Box<Boolean>,
    },
    // if: false
    Literal {
        value: bool,
    },
    // if: $array is empty
    ListIsEmpty {
        value: ftd::PropertyValue,
    },
}

impl Boolean {
    pub fn to_condition(
        &self,
        line_number: usize,
        all_locals: &mut ftd::Map,
        arguments: &std::collections::BTreeMap<String, ftd::Value>,
        doc: &ftd::p2::TDoc,
    ) -> ftd::p1::Result<ftd::Condition> {
        let (variable, value) = match self {
            Self::Equal { left, right } => {
                let variable = resolve_variable(left, line_number, all_locals, arguments, doc)?;

                let value = match right {
                    ftd::PropertyValue::Value { value } => value.to_owned(),
                    ftd::PropertyValue::Variable { name, kind } => {
                        if let Some(arg) = arguments.get(name) {
                            if arg.kind().is_same_as(kind) {
                                arg.to_owned()
                            } else {
                                return ftd::e2(
                                    format!(
                                        "kind mismatch expected: {:?} found: {:?}",
                                        kind,
                                        arg.kind()
                                    ),
                                    doc.name,
                                    line_number,
                                );
                            }
                        } else {
                            return ftd::e2(
                                format!("argument not found {}", name),
                                doc.name,
                                line_number,
                            );
                        }
                    }
                    _ => {
                        return ftd::e2(
                            format!("{:?} must be value or argument", right),
                            doc.name,
                            line_number,
                        );
                    }
                };

                (variable, value)
            }
            Self::IsNotNull { value } => {
                let variable = resolve_variable(value, line_number, all_locals, arguments, doc)?;
                (
                    variable,
                    ftd::Value::String {
                        text: "$IsNotNull$".to_string(),
                        source: ftd::TextSource::Header,
                    },
                )
            }
            Self::IsNull { value } => {
                let variable = resolve_variable(value, line_number, all_locals, arguments, doc)?;
                (
                    variable,
                    ftd::Value::String {
                        text: "$IsNull$".to_string(),
                        source: ftd::TextSource::Header,
                    },
                )
            }
            _ => return ftd::e2(format!("{:?} must not happen", self), doc.name, line_number),
        };
        return match value.to_string() {
            None => {
                return ftd::e2(
                    format!(
                        "expected value of type String, Integer, Decimal or Boolean, found: {:?}",
                        value
                    ),
                    doc.name,
                    line_number,
                )
            }
            Some(value) => Ok(ftd::Condition { variable, value }),
        };

        fn resolve_variable(
            value: &ftd::PropertyValue,
            line_number: usize,
            all_locals: &mut ftd::Map,
            arguments: &std::collections::BTreeMap<String, ftd::Value>,
            doc: &ftd::p2::TDoc,
        ) -> ftd::p1::Result<String> {
            match value {
                ftd::PropertyValue::Reference { name, .. } => Ok(name.to_string()),
                ftd::PropertyValue::Variable { name, .. } => {
                    let (v, remaining) = name
                        .split_once('.')
                        .map(|(v, n)| (v, Some(n)))
                        .unwrap_or((name, None));
                    if let Some(string_container) = all_locals.get(v) {
                        if let Some(remaining) = remaining {
                            let bag_with_argument = {
                                let mut bag_with_argument = doc.bag.clone();
                                bag_with_argument.extend(arguments.iter().map(|(k, v)| {
                                    (
                                        format!("{}#{}", doc.name, k),
                                        ftd::p2::Thing::Variable(ftd::Variable {
                                            name: k.to_string(),
                                            value: v.to_owned(),
                                            conditions: vec![],
                                        }),
                                    )
                                }));
                                bag_with_argument
                            };
                            let doc = ftd::p2::TDoc {
                                name: doc.name,
                                aliases: doc.aliases,
                                bag: &bag_with_argument,
                            };
                            if doc.get_value(line_number, name).is_ok() {
                                return Ok(format!("@{}@{}.{}", v, string_container, remaining));
                            }
                        } else {
                            return Ok(format!("@{}@{}", v, string_container));
                        }
                    } else if name.eq("MOUSE-IN") {
                        let string_container = all_locals.get("MOUSE-IN-TEMP").unwrap().clone();
                        all_locals.insert("MOUSE-IN".to_string(), string_container.to_string());
                        return Ok(format!("@MOUSE-IN@{}", string_container));
                    }
                    return ftd::e2(
                        format!("Can't find the local variable {}", name),
                        doc.name,
                        line_number,
                    );
                }
                _ => {
                    return ftd::e2(
                        format!("{:?} must be variable or local variable", value),
                        doc.name,
                        line_number,
                    );
                }
            }
        }
    }

    pub fn boolean_left_right(
        line_number: usize,
        expr: &str,
        doc_id: &str,
    ) -> ftd::p1::Result<(String, String, Option<String>)> {
        let expr: String = expr.split_whitespace().collect::<Vec<&str>>().join(" ");
        if expr == "true" || expr == "false" {
            return Ok(("Literal".to_string(), expr, None));
        }
        let (left, rest) = match expr.split_once(' ') {
            None => return Ok(("Equal".to_string(), expr.to_string(), None)),
            Some(v) => v,
        };
        if left == "not" {
            return Ok(("NotEqual".to_string(), rest.to_string(), None));
        }
        Ok(match rest {
            "is not null" => ("IsNotNull".to_string(), left.to_string(), None),
            "is null" => ("IsNull".to_string(), left.to_string(), None),
            "is not empty" => ("IsNotEmpty".to_string(), left.to_string(), None),
            "is empty" => ("IsEmpty".to_string(), left.to_string(), None),
            _ if rest.starts_with("==") => (
                "Equal".to_string(),
                left.to_string(),
                Some(rest.replace("==", "").trim().to_string()),
            ),
            _ => {
                return ftd::e2(
                    format!("'{}' is not valid condition", rest),
                    doc_id,
                    line_number,
                )
            }
        })
    }

    pub fn from_expression(
        expr: &str,
        doc: &ftd::p2::TDoc,
        arguments: &std::collections::BTreeMap<String, ftd::p2::Kind>,
        left_right_resolved_property: (Option<ftd::PropertyValue>, Option<ftd::PropertyValue>),
        line_number: usize,
    ) -> ftd::p1::Result<Self> {
        let (boolean, mut left, mut right) =
            ftd::p2::Boolean::boolean_left_right(line_number, expr, doc.name)?;
        left = doc.resolve_reference_name(line_number, left.as_str(), arguments)?;
        if let Some(ref r) = right {
            right = doc.resolve_reference_name(line_number, r, arguments).ok();
        }
        return Ok(match boolean.as_str() {
            "Literal" => Boolean::Literal {
                value: left == "true",
            },
            "IsNotNull" | "IsNull" => {
                let value = property_value(
                    &left,
                    None,
                    doc,
                    arguments,
                    left_right_resolved_property.0,
                    line_number,
                )?;
                if !value.kind().is_optional() {
                    return ftd::e2(
                        format!("'{}' is not to an optional", left),
                        doc.name,
                        line_number,
                    );
                }
                if boolean.as_str() == "IsNotNull" {
                    Boolean::IsNotNull { value }
                } else {
                    Boolean::IsNull { value }
                }
            }
            "IsNotEmpty" | "IsEmpty" => {
                let value = property_value(
                    &left,
                    None,
                    doc,
                    arguments,
                    left_right_resolved_property.0,
                    line_number,
                )?;
                if !value.kind().is_list() {
                    return ftd::e2(
                        format!("'{}' is not to a list", left),
                        doc.name,
                        line_number,
                    );
                }
                if boolean.as_str() == "IsNotEmpty" {
                    Boolean::IsNotEmpty { value }
                } else {
                    Boolean::IsEmpty { value }
                }
            }
            "NotEqual" | "Equal" => {
                if let Some(right) = right {
                    let left = property_value(
                        &left,
                        None,
                        doc,
                        arguments,
                        left_right_resolved_property.0,
                        line_number,
                    )?;
                    Boolean::Equal {
                        left: left.to_owned(),
                        right: property_value(
                            &right,
                            Some(left.kind()),
                            doc,
                            arguments,
                            left_right_resolved_property.1,
                            line_number,
                        )?,
                    }
                } else {
                    Boolean::Equal {
                        left: property_value(
                            &left,
                            Some(ftd::p2::Kind::boolean()),
                            doc,
                            arguments,
                            left_right_resolved_property.0,
                            line_number,
                        )?,
                        right: ftd::PropertyValue::Value {
                            value: ftd::Value::Boolean {
                                value: boolean.as_str() == "Equal",
                            },
                        },
                    }
                }
            }
            _ => {
                return ftd::e2(
                    format!("'{}' is not valid condition", expr),
                    doc.name,
                    line_number,
                )
            }
        });

        fn property_value(
            value: &str,
            expected_kind: Option<ftd::p2::Kind>,
            doc: &ftd::p2::TDoc,
            arguments: &std::collections::BTreeMap<String, ftd::p2::Kind>,
            loop_already_resolved_property: Option<ftd::PropertyValue>,
            line_number: usize,
        ) -> ftd::p1::Result<ftd::PropertyValue> {
            Ok(
                match ftd::PropertyValue::resolve_value(
                    line_number,
                    value,
                    expected_kind,
                    doc,
                    arguments,
                    None,
                ) {
                    Ok(v) => v,
                    Err(e) => match &loop_already_resolved_property {
                        Some(ftd::PropertyValue::Variable { .. }) => {
                            loop_already_resolved_property.clone().expect("")
                        }
                        _ => return Err(e),
                    },
                },
            )
        }
    }

    pub fn is_constant(&self) -> bool {
        let is_loop_constant = {
            let mut constant = false;
            if let ftd::p2::Boolean::Equal {
                left: ftd::PropertyValue::Variable { name, .. },
                right: ftd::PropertyValue::Value { .. },
            } = self
            {
                if name.starts_with("$loop$") {
                    constant = true;
                }
            }
            constant
        };
        (!matches!(
            self,
            Self::Equal {
                left: ftd::PropertyValue::Reference { .. },
                right: ftd::PropertyValue::Value { .. },
                ..
            }
        ) && !matches!(
            self,
            Self::Equal {
                left: ftd::PropertyValue::Variable { .. },
                right: ftd::PropertyValue::Value { .. },
                ..
            }
        ) && !matches!(self, Self::IsNotNull { .. })
            && !matches!(self, Self::IsNull { .. }))
            || is_loop_constant
    }

    pub fn is_arg_constant(&self) -> bool {
        let is_loop_constant = {
            let mut constant = false;
            if let ftd::p2::Boolean::Equal {
                left: ftd::PropertyValue::Variable { name, .. },
                right: ftd::PropertyValue::Value { .. },
            } = self
            {
                if name.starts_with("$loop$") {
                    constant = true;
                }
            }
            constant
        };
        (!matches!(
            self,
            Self::Equal {
                left: ftd::PropertyValue::Reference { .. },
                right: ftd::PropertyValue::Value { .. },
                ..
            }
        ) && !matches!(
            self,
            Self::Equal {
                left: ftd::PropertyValue::Variable { .. },
                right: ftd::PropertyValue::Value { .. },
                ..
            }
        ) && !matches!(
            self,
            Self::Equal {
                left: ftd::PropertyValue::Reference { .. },
                right: ftd::PropertyValue::Variable { .. },
                ..
            }
        ) && !matches!(self, Self::IsNotNull { .. })
            && !matches!(self, Self::IsNull { .. }))
            || is_loop_constant
    }

    pub fn eval(
        &self,
        line_number: usize,
        arguments: &std::collections::BTreeMap<String, ftd::Value>,
        doc: &ftd::p2::TDoc,
    ) -> ftd::p1::Result<bool> {
        Ok(match self {
            Self::Literal { value } => *value,
            Self::IsNotNull { value } => !value.resolve(line_number, arguments, doc)?.is_null(),
            Self::IsNull { value } => value.resolve(line_number, arguments, doc)?.is_null(),
            Self::IsNotEmpty { value } => !value.resolve(line_number, arguments, doc)?.is_empty(),
            Self::IsEmpty { value } => value.resolve(line_number, arguments, doc)?.is_empty(),
            Self::Equal { left, right } => left
                .resolve(line_number, arguments, doc)?
                .is_equal(&right.resolve(line_number, arguments, doc)?),
            _ => {
                return ftd::e2(
                    format!("unknown Boolean found: {:?}", self),
                    doc.name,
                    line_number,
                )
            }
        })
    }

    pub fn set_null(&self, line_number: usize, doc_id: &str) -> ftd::p1::Result<bool> {
        Ok(match self {
            Self::Literal { .. } | Self::IsNotEmpty { .. } | Self::IsEmpty { .. } => true,
            Self::Equal { left, right } => match (left, right) {
                (ftd::PropertyValue::Value { .. }, ftd::PropertyValue::Value { .. })
                | (ftd::PropertyValue::Value { .. }, ftd::PropertyValue::Variable { .. })
                | (ftd::PropertyValue::Variable { .. }, ftd::PropertyValue::Value { .. })
                | (ftd::PropertyValue::Variable { .. }, ftd::PropertyValue::Variable { .. }) => {
                    true
                }
                _ => false,
            },
            Self::IsNotNull { .. } | Self::IsNull { .. } => false,
            _ => {
                return ftd::e2(
                    format!("unimplemented for type: {:?}", self),
                    doc_id,
                    line_number,
                )
            }
        })
    }
}
