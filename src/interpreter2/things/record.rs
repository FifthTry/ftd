#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Record {
    pub name: String,
    pub fields: Vec<Field>,
    pub line_number: usize,
}

impl Record {
    fn new(name: &str, fields: Vec<Field>, line_number: usize) -> Record {
        Record {
            name: name.to_string(),
            fields,
            line_number,
        }
    }

    pub(crate) fn scan_ast(
        ast: ftd::ast::AST,
        doc: &mut ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<()> {
        let record = ast.get_record(doc.name)?;
        Record::scan_record(record, doc)
    }

    pub(crate) fn scan_record(
        record: ftd::ast::Record,
        doc: &mut ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<()> {
        let name = doc.resolve_name(record.name.as_str());
        let known_kinds = std::iter::IntoIterator::into_iter([(
            record.name.to_string(),
            ftd::interpreter2::Kind::record(name.as_str()),
        )])
        .collect::<ftd::Map<ftd::interpreter2::Kind>>();
        Field::scan_ast_fields(record.fields, doc, &known_kinds)
    }

    pub(crate) fn from_ast(
        ast: ftd::ast::AST,
        doc: &mut ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::StateWithThing<ftd::interpreter2::Record>>
    {
        let record = ast.get_record(doc.name)?;
        Record::from_record(record, doc)
    }

    pub(crate) fn from_record(
        record: ftd::ast::Record,
        doc: &mut ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::StateWithThing<ftd::interpreter2::Record>>
    {
        let name = doc.resolve_name(record.name.as_str());
        let known_kinds = std::iter::IntoIterator::into_iter([(
            record.name.to_string(),
            ftd::interpreter2::Kind::Record {
                name: name.to_string(),
            },
        )])
        .collect::<ftd::Map<ftd::interpreter2::Kind>>();
        let fields = try_ok_state!(Field::from_ast_fields(record.fields, doc, &known_kinds)?);
        validate_record_fields(name.as_str(), &fields, doc.name)?;
        Ok(ftd::interpreter2::StateWithThing::new_thing(Record::new(
            name.as_str(),
            fields,
            record.line_number,
        )))
    }

    pub(crate) fn get_field(
        &self,
        name: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<&Field> {
        use itertools::Itertools;

        let field = self.fields.iter().filter(|v| v.name.eq(name)).collect_vec();
        if field.is_empty() {
            return ftd::interpreter2::utils::e2(
                format!(
                    "Cannot find the field `{}` for record `{}`",
                    name, self.name
                )
                .as_str(),
                doc_id,
                line_number,
            );
        }

        if field.len() > 1 {
            return ftd::interpreter2::utils::e2(
                format!(
                    "Multiple fields `{}` for record `{}` found",
                    name, self.name
                )
                .as_str(),
                doc_id,
                line_number,
            );
        }

        Ok(field.first().unwrap())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Field {
    pub name: String,
    pub kind: ftd::interpreter2::KindData,
    pub mutable: bool,
    pub value: Option<ftd::interpreter2::PropertyValue>,
    pub line_number: usize,
}

impl Field {
    pub fn new(
        name: &str,
        kind: ftd::interpreter2::KindData,
        mutable: bool,
        value: Option<ftd::interpreter2::PropertyValue>,
        line_number: usize,
    ) -> Field {
        Field {
            name: name.to_string(),
            kind,
            mutable,
            value,
            line_number,
        }
    }

    pub fn to_sources(&self) -> Vec<ftd::interpreter2::PropertySource> {
        let mut sources = vec![ftd::interpreter2::PropertySource::Header {
            name: self.name.to_string(),
            mutable: self.mutable,
        }];
        if self.is_caption() {
            sources.push(ftd::interpreter2::PropertySource::Caption);
        }

        if self.is_body() {
            sources.push(ftd::interpreter2::PropertySource::Body);
        }

        if self.is_subsection_ui() {
            sources.push(ftd::interpreter2::PropertySource::Subsection);
        }

        sources
    }

    pub fn default(name: &str, kind: ftd::interpreter2::KindData) -> Field {
        Field {
            name: name.to_string(),
            kind,
            mutable: false,
            value: None,
            line_number: 0,
        }
    }

    pub(crate) fn scan_ast_fields(
        fields: Vec<ftd::ast::Field>,
        doc: &mut ftd::interpreter2::TDoc,
        known_kinds: &ftd::Map<ftd::interpreter2::Kind>,
    ) -> ftd::interpreter2::Result<()> {
        for field in fields {
            Field::scan_ast_field(field, doc, known_kinds)?;
        }
        Ok(())
    }

    pub(crate) fn from_ast_fields(
        fields: Vec<ftd::ast::Field>,
        doc: &mut ftd::interpreter2::TDoc,
        known_kinds: &ftd::Map<ftd::interpreter2::Kind>,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::StateWithThing<Vec<Field>>> {
        let mut result = vec![];
        for field in fields {
            let field = try_ok_state!(Field::from_ast_field(field, doc, known_kinds)?);
            result.push(field);
        }
        Ok(ftd::interpreter2::StateWithThing::new_thing(result))
    }

    pub(crate) fn scan_ast_field(
        field: ftd::ast::Field,
        doc: &mut ftd::interpreter2::TDoc,
        known_kinds: &ftd::Map<ftd::interpreter2::Kind>,
    ) -> ftd::interpreter2::Result<()> {
        ftd::interpreter2::KindData::scan_ast_kind(
            field.kind,
            known_kinds,
            doc,
            field.line_number,
        )?;

        if let Some(value) = field.value {
            ftd::interpreter2::PropertyValue::scan_ast_value(value, doc)?;
        }

        Ok(())
    }

    pub(crate) fn from_ast_field(
        field: ftd::ast::Field,
        doc: &mut ftd::interpreter2::TDoc,
        known_kinds: &ftd::Map<ftd::interpreter2::Kind>,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::StateWithThing<Field>> {
        let kind = try_ok_state!(ftd::interpreter2::KindData::from_ast_kind(
            field.kind,
            known_kinds,
            doc,
            field.line_number,
        )?);

        let value = if let Some(value) = field.value {
            Some(try_ok_state!(
                ftd::interpreter2::PropertyValue::from_ast_value(
                    value,
                    doc,
                    field.mutable,
                    Some(&kind),
                )?
            ))
        } else {
            None
        };

        Ok(ftd::interpreter2::StateWithThing::new_thing(Field {
            name: field.name.to_string(),
            kind,
            mutable: field.mutable,
            value,
            line_number: field.line_number,
        }))
    }

    pub fn is_caption(&self) -> bool {
        self.kind.caption
    }

    pub fn is_subsection_ui(&self) -> bool {
        self.kind.kind.clone().inner_list().is_subsection_ui()
    }

    pub fn is_body(&self) -> bool {
        self.kind.body
    }

    pub(crate) fn for_component(
        component_name: &str,
        definition_name_with_arguments: &Option<(&str, &[Field])>,
        doc: &mut ftd::interpreter2::TDoc,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::StateWithThing<Vec<Field>>> {
        Ok(ftd::interpreter2::StateWithThing::new_thing(
            match definition_name_with_arguments {
                Some((name, arg)) if name.eq(&component_name) => arg.to_vec(),
                _ => try_ok_state!(doc.search_component(component_name, line_number)?).arguments,
            },
        ))
    }

    pub fn update_with_or_type_variant(
        &mut self,
        doc: &mut ftd::interpreter2::TDoc,
        variant: &str,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::StateWithThing<()>> {
        match self.kind.kind.mut_inner() {
            ftd::interpreter2::Kind::OrType {
                name,
                variant: v,
                full_variant,
            } => {
                let or_type = try_ok_state!(doc.search_or_type(name, self.line_number)?);
                let (variant_name, remaining) =
                    ftd::p2::utils::get_doc_name_and_remaining(variant)?;
                let or_variant = or_type
                    .variants
                    .iter()
                    .find(|v| {
                        v.name()
                            .trim_start_matches(format!("{}.", name).as_str())
                            .eq(variant_name.as_str())
                    })
                    .ok_or(ftd::interpreter2::Error::ParseError {
                        message: format!(
                            "Cannot find variant `{}` for or-type `{}`",
                            variant, name
                        ),
                        doc_id: doc.name.to_string(),
                        line_number: self.line_number,
                    })?;

                check_variant_if_constant(or_variant, remaining, doc)?;
                let variant = Some(format!("{}.{}", name, variant));

                *v = variant.clone();
                *full_variant = variant;
                Ok(ftd::interpreter2::StateWithThing::new_thing(()))
            }
            t => ftd::interpreter2::utils::e2(
                format!(
                    "Expected or-type for variant `{}`, found: `{:?}`",
                    variant, t
                ),
                doc.name,
                self.line_number,
            ),
        }
    }
}

fn validate_record_fields(
    rec_name: &str,
    fields: &[Field],
    doc_id: &str,
) -> ftd::interpreter2::Result<()> {
    if let Some(field) = fields.iter().find(|v| v.mutable) {
        return ftd::interpreter2::utils::e2(
            format!(
                "Currently, mutable field `{}` in record `{}` is not supported.",
                field.name, rec_name
            )
            .as_str(),
            doc_id,
            field.line_number,
        );
    }
    Ok(())
}

fn check_variant_if_constant(
    or_variant: &ftd::interpreter2::OrTypeVariant,
    _remaining: Option<String>,
    doc: &ftd::interpreter2::TDoc,
) -> ftd::interpreter2::Result<()> {
    match or_variant {
        ftd::interpreter2::OrTypeVariant::AnonymousRecord(_r) => {} // Todo: check on remaining for constant and throw error if found
        ftd::interpreter2::OrTypeVariant::Regular(_r) => {} // Todo: check on remaining for constant and throw error if found
        ftd::interpreter2::OrTypeVariant::Constant(c) => {
            return ftd::interpreter2::utils::e2(
                format!("Cannot pass deconstructed constant variant `{}`", c.name),
                doc.name,
                c.line_number,
            );
        }
    }
    Ok(())
}
