#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Record {
    pub name: String,
    pub fields: std::collections::BTreeMap<String, ftd::p2::Kind>,
    pub instances: std::collections::BTreeMap<String, Vec<Invocation>>,
}

type Invocation = std::collections::BTreeMap<String, ftd::PropertyValue>;

impl Record {
    pub fn variant_name(&self) -> Option<&str> {
        self.name.split_once(".").map(|(_, r)| r)
    }

    pub fn fields(
        &self,
        p1: &crate::p1::Section,
        doc: &crate::p2::TDoc,
    ) -> crate::p1::Result<std::collections::BTreeMap<String, crate::PropertyValue>> {
        let mut fields: std::collections::BTreeMap<String, crate::PropertyValue> =
            Default::default();
        self.assert_no_extra_fields(&p1.header, &p1.caption, &p1.body)?;
        for (name, kind) in self.fields.iter() {
            let value = match (p1.sub_section_by_name(name), kind.inner()) {
                (Ok(v), crate::p2::Kind::String { .. }) => crate::PropertyValue::Value {
                    value: crate::Value::String {
                        text: v.body()?,
                        source: crate::TextSource::Body,
                    },
                },
                (Ok(v), crate::p2::Kind::Record { name }) => {
                    let record = doc.get_record(name)?;
                    crate::PropertyValue::Value {
                        value: crate::Value::Record {
                            name: doc.resolve_name(record.name.as_str())?,
                            fields: record.fields_from_sub_section(v, doc)?,
                        },
                    }
                }
                (Ok(_), _) => {
                    return crate::e(format!(
                        "'{:?}' ('{}') can not be a sub-section",
                        kind, name
                    ));
                }
                (
                    Err(crate::p1::Error::NotFound { .. }),
                    crate::p2::Kind::List { kind: list_kind },
                ) => match list_kind.as_ref() {
                    crate::p2::Kind::OrType { name: or_type_name } => {
                        let e = doc.get_or_type(or_type_name)?;
                        let mut values: Vec<crate::Value> = vec![];
                        for s in p1.sub_sections.0.iter() {
                            if s.is_commented {
                                continue;
                            }
                            for v in e.variants.iter() {
                                let variant = v.variant_name().expect("record.fields").to_string();
                                if s.name == format!("{}.{}", name, variant.as_str()) {
                                    values.push(crate::Value::OrType {
                                        variant,
                                        name: e.name.to_string(),
                                        fields: v.fields_from_sub_section(s, doc)?,
                                    })
                                }
                            }
                        }
                        crate::PropertyValue::Value {
                            value: crate::Value::List {
                                kind: list_kind.inner().to_owned(),
                                data: values,
                            },
                        }
                    }
                    crate::p2::Kind::Record { .. } => {
                        let mut list = crate::Value::List {
                            kind: list_kind.inner().to_owned(),
                            data: vec![],
                        };
                        for (k, v) in p1.header.0.iter() {
                            if *k != *name || k.starts_with('/') {
                                continue;
                            }
                            let reference = if v.starts_with("ref ") {
                                ftd_rt::get_name("ref", v)?
                            } else {
                                v
                            };
                            list = doc.get_value(reference)?;
                        }
                        crate::PropertyValue::Value { value: list }
                    }
                    crate::p2::Kind::String { .. } => {
                        let mut values: Vec<crate::Value> = vec![];
                        for (k, v) in p1.header.0.iter() {
                            if *k != *name || k.starts_with('/') {
                                continue;
                            }
                            values.push(crate::Value::String {
                                text: v.to_string(),
                                source: ftd::TextSource::Header,
                            });
                        }
                        crate::PropertyValue::Value {
                            value: crate::Value::List {
                                kind: list_kind.inner().to_owned(),
                                data: values,
                            },
                        }
                    }
                    crate::p2::Kind::Integer { .. } => return ftd::e("unexpected integer"),
                    t => return ftd::e2("not yet implemented 123", t),
                },
                (Err(crate::p1::Error::NotFound { .. }), _) => kind.read_section(
                    &p1.header,
                    &p1.caption,
                    &p1.body_without_comment(),
                    name,
                    doc,
                )?,
                (
                    Err(crate::p1::Error::MoreThanOneSubSections { .. }),
                    crate::p2::Kind::List { kind: list_kind },
                ) => {
                    let mut values: Vec<crate::Value> = vec![];
                    for s in p1.sub_sections.0.iter() {
                        if s.name != *name || s.is_commented {
                            continue;
                        }
                        let v = match list_kind.inner().string_any() {
                            crate::p2::Kind::Record { name } => {
                                let record = doc.get_record(name.as_str())?;
                                crate::Value::Record {
                                    name: doc.resolve_name(record.name.as_str())?,
                                    fields: record.fields_from_sub_section(s, doc)?,
                                }
                            }
                            k => {
                                match k.read_section(
                                    &s.header,
                                    &s.caption,
                                    &s.body_without_comment(),
                                    s.name.as_str(),
                                    doc,
                                )? {
                                    crate::PropertyValue::Value { value: v } => v,
                                    _ => unimplemented!(),
                                }
                            }
                        };
                        values.push(v);
                    }
                    crate::PropertyValue::Value {
                        value: crate::Value::List {
                            kind: list_kind.inner().to_owned(),
                            data: values,
                        },
                    }
                }
                (Err(e), _) => return Err(e),
            };
            fields.insert(name.to_string(), value);
        }
        Ok(fields)
    }

    pub fn add_instance(
        &mut self,
        p1: &crate::p1::Section,
        doc: &crate::p2::TDoc,
    ) -> crate::p1::Result<()> {
        let fields = self.fields(p1, doc)?;
        self.instances
            .entry(doc.name.to_string())
            .or_default()
            .push(fields);
        Ok(())
    }

    pub fn create(
        &self,
        p1: &crate::p1::Section,
        doc: &crate::p2::TDoc,
    ) -> crate::p1::Result<crate::Value> {
        Ok(crate::Value::Record {
            name: doc.resolve_name(self.name.as_str())?,
            fields: self.fields(p1, doc)?,
        })
    }

    pub fn fields_from_sub_section(
        &self,
        p1: &crate::p1::SubSection,
        doc: &crate::p2::TDoc,
    ) -> crate::p1::Result<std::collections::BTreeMap<String, crate::PropertyValue>> {
        let mut fields: std::collections::BTreeMap<String, crate::PropertyValue> =
            Default::default();
        self.assert_no_extra_fields(&p1.header, &p1.caption, &p1.body)?;
        for (name, kind) in self.fields.iter() {
            fields.insert(
                name.to_string(),
                kind.read_section(
                    &p1.header,
                    &p1.caption,
                    &p1.body_without_comment(),
                    name,
                    doc,
                )?,
            );
        }
        Ok(fields)
    }

    fn assert_no_extra_fields(
        &self,
        p1: &crate::p1::Header,
        _caption: &Option<String>,
        _body: &Option<String>,
    ) -> crate::p1::Result<()> {
        // TODO: handle caption
        // TODO: handle body
        for (k, _) in p1.0.iter() {
            if k.starts_with('/') {
                continue;
            }

            if !self.fields.contains_key(k) && k != "type" && k != "$processor$" {
                return crate::e(format!(
                    "unknown key passed: '{}' to '{}', allowed: {:?}",
                    k,
                    self.name,
                    self.fields.keys()
                ));
            }
        }
        Ok(())
    }

    pub fn from_p1(
        p1_name: &str,
        p1_header: &crate::p1::Header,
        doc: &crate::p2::TDoc,
    ) -> crate::p1::Result<Self> {
        let name = ftd_rt::get_name("record", p1_name)?;
        let full_name = doc.format_name(name);
        let mut fields = std::collections::BTreeMap::new();
        let object_kind = (
            name,
            crate::p2::Kind::Record {
                name: full_name.clone(),
            },
        );
        for (k, v) in p1_header.0.iter() {
            if k.starts_with('/') {
                continue;
            }
            let v = normalise_value(v)?;
            validate_key(k)?;
            fields.insert(
                k.to_string(),
                crate::p2::Kind::from(v.as_str(), doc, Some(object_kind.clone()))?,
            );
        }
        assert_fields_valid(&fields)?;
        return Ok(Record {
            name: full_name,
            fields,
            instances: Default::default(),
        });

        fn normalise_value(s: &str) -> crate::p1::Result<String> {
            // TODO: normalise spaces in v
            Ok(s.to_string())
        }

        fn validate_key(_k: &str) -> crate::p1::Result<()> {
            // TODO: ensure k in valid (only alphanumeric, _, and -)
            Ok(())
        }
    }
}

fn assert_fields_valid(
    fields: &std::collections::BTreeMap<String, crate::p2::Kind>,
) -> crate::p1::Result<()> {
    let mut caption_field: Option<String> = None;
    let mut body_field: Option<String> = None;
    for (name, kind) in fields.iter() {
        if let crate::p2::Kind::String { caption, body, .. } = kind {
            if *caption {
                match &caption_field {
                    Some(c) => {
                        return crate::e(format!("both {} and {} are caption fields", name, c));
                    }
                    None => caption_field = Some(name.to_string()),
                }
            }
            if *body {
                match &body_field {
                    Some(c) => {
                        return crate::e(format!("both {} and {} are body fields", name, c));
                    }
                    None => body_field = Some(name.to_string()),
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::test::*;

    #[test]
    fn record() {
        let sourabh: super::Invocation = std::array::IntoIter::new([
            (
                s("name"),
                crate::PropertyValue::Value {
                    value: crate::Value::String {
                        text: "Sourabh Garg".to_string(),
                        source: crate::TextSource::Caption,
                    },
                },
            ),
            (
                s("address"),
                crate::PropertyValue::Value {
                    value: crate::Value::String {
                        text: "Ranchi".to_string(),
                        source: crate::TextSource::Header,
                    },
                },
            ),
            (
                s("bio"),
                crate::PropertyValue::Value {
                    value: crate::Value::String {
                        text: "Frontend developer at fifthtry.".to_string(),
                        source: crate::TextSource::Body,
                    },
                },
            ),
            (
                s("age"),
                crate::PropertyValue::Value {
                    value: crate::Value::Integer { value: 28 },
                },
            ),
        ])
        .collect();

        let mut bag = crate::p2::interpreter::default_bag();
        bag.insert(
            "foo/bar#abrar".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "abrar".to_string(),
                value: crate::Value::Record {
                    name: "foo/bar#person".to_string(),
                    fields: abrar(),
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#person".to_string(),
            crate::p2::Thing::Record(crate::p2::Record {
                name: "foo/bar#person".to_string(),
                fields: person_fields(),
                instances: std::array::IntoIter::new([(
                    s("foo/bar"),
                    vec![abrar(), sourabh.clone()],
                )])
                .collect(),
            }),
        );
        bag.insert(
            "foo/bar#x".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "x".to_string(),
                value: crate::Value::Integer { value: 20 },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#employee".to_string(),
            crate::p2::Thing::Record(crate::p2::Record {
                name: "foo/bar#employee".to_string(),
                fields: std::array::IntoIter::new([
                    (s("eid"), crate::p2::Kind::string()),
                    (
                        s("who"),
                        crate::p2::Kind::Record {
                            name: s("foo/bar#person"),
                        },
                    ),
                ])
                .collect(),
                instances: Default::default(),
            }),
        );
        bag.insert(
            "foo/bar#abrar_e".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "abrar_e".to_string(),
                value: crate::Value::Record {
                    name: "foo/bar#employee".to_string(),
                    fields: std::array::IntoIter::new([
                        (
                            s("eid"),
                            crate::PropertyValue::Value {
                                value: crate::Value::String {
                                    text: "E04".to_string(),
                                    source: crate::TextSource::Header,
                                },
                            },
                        ),
                        (
                            s("who"),
                            crate::PropertyValue::Reference {
                                name: s("foo/bar#abrar"),
                                kind: crate::p2::Kind::Record {
                                    name: s("foo/bar#person"),
                                },
                            },
                        ),
                    ])
                    .collect(),
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#sourabh".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "sourabh".to_string(),
                value: crate::Value::Record {
                    name: "foo/bar#employee".to_string(),
                    fields: std::array::IntoIter::new([
                        (
                            s("eid"),
                            crate::PropertyValue::Value {
                                value: crate::Value::String {
                                    text: "E05".to_string(),
                                    source: crate::TextSource::Body,
                                },
                            },
                        ),
                        (
                            s("who"),
                            crate::PropertyValue::Value {
                                value: crate::Value::Record {
                                    name: "foo/bar#person".to_string(),
                                    fields: sourabh,
                                },
                            },
                        ),
                    ])
                    .collect(),
                },
                conditions: vec![],
            }),
        );

        p!(
            "
            -- record person:
            name: caption
            address: string
            bio: body
            age: integer

            -- var x: 10

            -- person: Abrar Khan2
            address: Bihar2
            age: ref x

            Software developer working at fifthtry2.

            -- person: Sourabh Garg
            address: Ranchi
            age: 28

            Frontend developer at fifthtry.

            -- var abrar: Abrar Khan
            type: person
            address: Bihar
            age: ref x

            Software developer working at fifthtry.

            -- record employee:
            eid: string
            who: person

            -- var abrar_e:
            type: employee
            eid: E04
            who: ref abrar

            -- var sourabh:
            type: employee

            --- eid:

            E05

            --- who: Sourabh Garg
            address: Ranchi
            age: 28

            Frontend developer at fifthtry.

            -- x: 20

            -- abrar: Abrar Khan2
            address: Bihar2
            age: ref x

            Software developer working at fifthtry2.
            ",
            (bag, crate::p2::interpreter::default_column()),
        );
    }

    #[test]
    fn list() {
        let b = |source: ftd::TextSource| {
            let mut bag = default_bag();

            bag.insert(
                "foo/bar#person".to_string(),
                crate::p2::Thing::Record(crate::p2::Record {
                    name: "foo/bar#person".to_string(),
                    fields: std::array::IntoIter::new([
                        (s("name"), crate::p2::Kind::caption()),
                        (
                            s("friends"),
                            crate::p2::Kind::List {
                                kind: Box::new(crate::p2::Kind::string()),
                            },
                        ),
                    ])
                    .collect(),
                    instances: Default::default(),
                }),
            );

            bag.insert(
                "foo/bar#abrar".to_string(),
                crate::p2::Thing::Variable(crate::Variable {
                    name: "abrar".to_string(),
                    value: crate::Value::Record {
                        name: "foo/bar#person".to_string(),
                        fields: std::array::IntoIter::new([
                            (
                                s("name"),
                                crate::PropertyValue::Value {
                                    value: crate::Value::String {
                                        text: "Abrar Khan".to_string(),
                                        source: crate::TextSource::Caption,
                                    },
                                },
                            ),
                            (
                                s("friends"),
                                crate::PropertyValue::Value {
                                    value: crate::Value::List {
                                        kind: crate::p2::Kind::string(),
                                        data: vec![
                                            crate::Value::String {
                                                text: "Deepak Angrula".to_string(),
                                                source: source.clone(),
                                            },
                                            crate::Value::String {
                                                text: "Amit Upadhyay".to_string(),
                                                source: source.clone(),
                                            },
                                            crate::Value::String {
                                                text: "Saurabh Garg".to_string(),
                                                source,
                                            },
                                        ],
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                    conditions: vec![],
                }),
            );
            bag
        };

        p!(
            "
            -- record person:
            name: caption
            friends: list string

            -- var abrar: Abrar Khan
            type: person
            friends: Deepak Angrula
            friends: Amit Upadhyay
            friends: Saurabh Garg
            ",
            (b(ftd::TextSource::Header), default_column()),
        );

        p!(
            "
            -- record person:
            name: caption
            friends: list string

            -- var abrar: Abrar Khan
            type: person

            --- friends: Deepak Angrula
            --- friends: Amit Upadhyay
            --- friends: Saurabh Garg
            ",
            (b(ftd::TextSource::Caption), default_column()),
        );
    }

    #[test]
    fn list_of_records() {
        let mut bag = default_bag();

        bag.insert(
            s("foo/bar#point"),
            crate::p2::Thing::Record(crate::p2::Record {
                name: s("foo/bar#point"),
                fields: std::array::IntoIter::new([
                    (s("x"), crate::p2::Kind::integer()),
                    (s("y"), crate::p2::Kind::integer()),
                ])
                .collect(),
                instances: Default::default(),
            }),
        );

        bag.insert(
            "foo/bar#person".to_string(),
            crate::p2::Thing::Record(crate::p2::Record {
                name: s("foo/bar#person"),
                fields: std::array::IntoIter::new([
                    (s("name"), crate::p2::Kind::caption()),
                    (
                        s("points"),
                        crate::p2::Kind::List {
                            kind: Box::new(crate::p2::Kind::Record {
                                name: s("foo/bar#point"),
                            }),
                        },
                    ),
                ])
                .collect(),
                instances: Default::default(),
            }),
        );

        bag.insert(
            "foo/bar#abrar".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "abrar".to_string(),
                value: crate::Value::Record {
                    name: "foo/bar#person".to_string(),
                    fields: std::array::IntoIter::new([
                        (
                            s("name"),
                            crate::PropertyValue::Value {
                                value: crate::Value::String {
                                    text: "Abrar Khan".to_string(),
                                    source: crate::TextSource::Caption,
                                },
                            },
                        ),
                        (
                            s("points"),
                            crate::PropertyValue::Value {
                                value: crate::Value::List {
                                    kind: crate::p2::Kind::Record {
                                        name: s("foo/bar#point"),
                                    },
                                    data: vec![
                                        crate::Value::Record {
                                            name: "foo/bar#point".to_string(),
                                            fields: std::array::IntoIter::new([
                                                (
                                                    s("x"),
                                                    crate::PropertyValue::Value {
                                                        value: crate::Value::Integer { value: 10 },
                                                    },
                                                ),
                                                (
                                                    s("y"),
                                                    crate::PropertyValue::Value {
                                                        value: crate::Value::Integer { value: 20 },
                                                    },
                                                ),
                                            ])
                                            .collect(),
                                        },
                                        crate::Value::Record {
                                            name: "foo/bar#point".to_string(),
                                            fields: std::array::IntoIter::new([
                                                (
                                                    s("x"),
                                                    crate::PropertyValue::Value {
                                                        value: crate::Value::Integer { value: 0 },
                                                    },
                                                ),
                                                (
                                                    s("y"),
                                                    crate::PropertyValue::Value {
                                                        value: crate::Value::Integer { value: 0 },
                                                    },
                                                ),
                                            ])
                                            .collect(),
                                        },
                                        crate::Value::Record {
                                            name: "foo/bar#point".to_string(),
                                            fields: std::array::IntoIter::new([
                                                (
                                                    s("x"),
                                                    crate::PropertyValue::Value {
                                                        value: crate::Value::Integer { value: 1 },
                                                    },
                                                ),
                                                (
                                                    s("y"),
                                                    crate::PropertyValue::Value {
                                                        value: crate::Value::Integer { value: 22 },
                                                    },
                                                ),
                                            ])
                                            .collect(),
                                        },
                                    ],
                                },
                            },
                        ),
                    ])
                    .collect(),
                },
                conditions: vec![],
            }),
        );

        p!(
            "
            -- record point:
            x: integer
            y: integer

            -- record person:
            name: caption
            points: list point

            -- var abrar: Abrar Khan
            type: person

            --- points:
            x: 10
            y: 20

            --- points:
            x: 0
            y: 0

            --- points:
            x: 1
            y: 22
            ",
            (bag, default_column()),
        );
    }

    #[test]
    fn list_of_or_types() {
        let mut bag = default_bag();

        bag.insert(s("foo/bar#entity"), entity());
        bag.insert(
            s("foo/bar#sale"),
            crate::p2::Thing::Record(crate::p2::Record {
                name: s("foo/bar#sale"),
                fields: std::array::IntoIter::new([
                    (
                        s("party"),
                        crate::p2::Kind::List {
                            kind: Box::new(crate::p2::Kind::OrType {
                                name: s("foo/bar#entity"),
                            }),
                        },
                    ),
                    (s("value"), crate::p2::Kind::integer()),
                ])
                .collect(),
                instances: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#jan"),
            crate::p2::Thing::Variable(crate::Variable {
                name: s("jan"),
                value: crate::Value::Record {
                    name: s("foo/bar#sale"),
                    fields: std::array::IntoIter::new([
                        (
                            s("value"),
                            crate::PropertyValue::Value {
                                value: crate::Value::Integer { value: 2000 },
                            },
                        ),
                        (
                            s("party"),
                            crate::PropertyValue::Value {
                                value: crate::Value::List {
                                    kind: crate::p2::Kind::OrType {
                                        name: s("foo/bar#entity"),
                                    },
                                    data: vec![
                                        crate::Value::OrType {
                                            name: s("foo/bar#entity"),
                                            variant: s("person"),
                                            fields: std::array::IntoIter::new([
                                                (
                                                    s("address"),
                                                    crate::PropertyValue::Value {
                                                        value: crate::Value::String {
                                                            text: s("123 Lane"),
                                                            source: crate::TextSource::Header,
                                                        },
                                                    },
                                                ),
                                                (
                                                    s("bio"),
                                                    crate::PropertyValue::Value {
                                                        value: crate::Value::String {
                                                            text: s("Owner of Jack Russo\'s Bar"),
                                                            source: crate::TextSource::Body,
                                                        },
                                                    },
                                                ),
                                                (
                                                    s("name"),
                                                    crate::PropertyValue::Value {
                                                        value: crate::Value::String {
                                                            text: s("Jack Russo"),
                                                            source: crate::TextSource::Caption,
                                                        },
                                                    },
                                                ),
                                                (
                                                    s("age"),
                                                    crate::PropertyValue::Value {
                                                        value: crate::Value::Integer { value: 24 },
                                                    },
                                                ),
                                            ])
                                            .collect(),
                                        },
                                        crate::Value::OrType {
                                            name: s("foo/bar#entity"),
                                            variant: s("company"),
                                            fields: std::array::IntoIter::new([
                                                (
                                                    s("industry"),
                                                    crate::PropertyValue::Value {
                                                        value: crate::Value::String {
                                                            text: s("Widgets"),
                                                            source: crate::TextSource::Header,
                                                        },
                                                    },
                                                ),
                                                (
                                                    s("name"),
                                                    crate::PropertyValue::Value {
                                                        value: crate::Value::String {
                                                            text: s("Acme Inc"),
                                                            source: crate::TextSource::Caption,
                                                        },
                                                    },
                                                ),
                                            ])
                                            .collect(),
                                        },
                                    ],
                                },
                            },
                        ),
                    ])
                    .collect(),
                },
                conditions: vec![],
            }),
        );

        p!(
            "
            -- or-type entity:

            --- person:
            name: caption
            address: string
            bio: body
            age: integer

            --- company:
            name: caption
            industry: string

            -- record sale:
            party: list entity
            value: integer

            -- var jan:
            type: sale
            value: 2000

            --- party.person: Jack Russo
            address: 123 Lane
            age: 24

            Owner of Jack Russo's Bar

            --- party.company: Acme Inc
            industry: Widgets
            ",
            (bag, default_column()),
        );
    }
}
