#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Kind {
    String,
    Object,
    Integer,
    Decimal,
    Boolean,
    Record {
        name: String,
    }, // the full name of the record (full document name.record name)
    OrType {
        name: String,
        variant: Option<String>,
        full_variant: Option<String>,
    },
    List {
        kind: Box<Kind>,
    },
    Optional {
        kind: Box<Kind>,
    },
    UI {
        name: Option<String>,
        subsection_source: bool,
    },
    Constant {
        kind: Box<Kind>,
    },
    Void,
}

impl Kind {
    pub fn into_kind_data(self) -> KindData {
        KindData::new(self)
    }

    pub fn is_same_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::UI { name: n1, .. }, Self::UI { name: n2, .. }) => n1.eq(n2),
            (Self::OrType { name: n1, .. }, Self::OrType { name: n2, .. }) => n1.eq(n2),
            (Self::Optional { kind, .. }, _) => kind.is_same_as(other),
            (_, Self::Optional { kind: other, .. }) => self.is_same_as(other),
            (Self::List { kind: k1 }, Self::List { kind: k2 }) => k1.is_same_as(k2),
            _ => self.eq(other),
        }
    }

    pub fn string() -> Kind {
        Kind::String
    }

    pub fn integer() -> Kind {
        Kind::Integer
    }

    pub fn decimal() -> Kind {
        Kind::Decimal
    }

    pub fn boolean() -> Kind {
        Kind::Boolean
    }

    pub fn ui() -> Kind {
        Kind::UI {
            name: None,
            subsection_source: false,
        }
    }

    pub fn ui_with_name(name: &str) -> Kind {
        Kind::UI {
            name: Some(name.to_string()),
            subsection_source: false,
        }
    }

    pub fn subsection_ui() -> Kind {
        Kind::UI {
            name: None,
            subsection_source: true,
        }
    }

    pub fn object() -> Kind {
        Kind::Object
    }

    pub fn void() -> Kind {
        Kind::Void
    }

    pub fn record(name: &str) -> Kind {
        Kind::Record {
            name: name.to_string(),
        }
    }

    pub fn or_type(name: &str) -> Kind {
        Kind::OrType {
            name: name.to_string(),
            variant: None,
            full_variant: None,
        }
    }

    pub fn or_type_with_variant(name: &str, variant: &str, full_variant: &str) -> Kind {
        Kind::OrType {
            name: name.to_string(),
            variant: Some(variant.to_string()),
            full_variant: Some(full_variant.to_string()),
        }
    }

    pub fn into_list(self) -> Kind {
        Kind::List {
            kind: Box::new(self),
        }
    }

    pub fn into_optional(self) -> Kind {
        Kind::Optional {
            kind: Box::new(self),
        }
    }

    pub fn inner(self) -> Kind {
        match self {
            Kind::Optional { kind } => kind.as_ref().to_owned(),
            t => t,
        }
    }

    pub fn mut_inner(&mut self) -> &mut Kind {
        match self {
            Kind::Optional { kind } => kind,
            t => t,
        }
    }

    pub fn ref_inner(&self) -> &Kind {
        match self {
            Kind::Optional { kind } => kind,
            t => t,
        }
    }

    pub fn inner_list(self) -> Kind {
        match self {
            Kind::List { kind } => kind.as_ref().to_owned(),
            t => t,
        }
    }

    pub fn ref_inner_list(&self) -> &Kind {
        match self {
            Kind::List { kind } => kind,
            t => t,
        }
    }

    pub(crate) fn is_list(&self) -> bool {
        matches!(self, Kind::List { .. })
    }

    pub fn is_subsection_ui(&self) -> bool {
        matches!(
            self,
            Kind::UI {
                subsection_source: true,
                ..
            }
        )
    }

    pub fn is_ui(&self) -> bool {
        matches!(self, Kind::UI { .. })
    }

    pub fn is_optional(&self) -> bool {
        matches!(self, Kind::Optional { .. })
    }

    pub fn is_record(&self) -> bool {
        matches!(self, Kind::Record { .. })
    }

    pub fn is_ftd_length(&self) -> bool {
        matches!(self, Kind::OrType { name, .. } if name.eq(ftd::interpreter2::FTD_LENGTH))
    }

    pub fn is_ftd_resizing(&self) -> bool {
        matches!(self, Kind::OrType { name, .. } if name.eq(ftd::interpreter2::FTD_RESIZING))
    }

    pub fn is_ftd_resizing_fixed(&self) -> bool {
        matches!(self, Kind::OrType { name, variant, .. } if name.eq(ftd::interpreter2::FTD_RESIZING) && variant.is_some() && variant.as_ref().unwrap().starts_with(ftd::interpreter2::FTD_RESIZING_FIXED))
    }

    pub fn is_or_type(&self) -> bool {
        matches!(self, Kind::OrType { .. })
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Kind::String { .. })
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, Kind::Integer { .. })
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, Kind::Boolean { .. })
    }

    pub fn is_decimal(&self) -> bool {
        matches!(self, Kind::Decimal { .. })
    }

    pub fn is_void(&self) -> bool {
        matches!(self, Kind::Void { .. })
    }

    pub(crate) fn list_type(
        &self,
        doc_name: &str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<Kind> {
        match &self {
            Kind::List { kind } => Ok(kind.as_ref().clone()),
            t => ftd::interpreter2::utils::e2(
                format!("Expected List, found: `{:?}`", t),
                doc_name,
                line_number,
            ),
        }
    }

    pub fn get_or_type(&self) -> Option<(String, Option<String>, Option<String>)> {
        match self {
            Kind::OrType {
                name,
                variant,
                full_variant,
            } => Some((name.to_owned(), variant.to_owned(), full_variant.to_owned())),
            _ => None,
        }
    }

    pub fn get_record_name(&self) -> Option<&str> {
        match self {
            ftd::interpreter2::Kind::Record { ref name, .. } => Some(name),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct KindData {
    pub kind: Kind,
    pub caption: bool,
    pub body: bool,
}

impl KindData {
    pub fn new(kind: Kind) -> KindData {
        KindData {
            kind,
            caption: false,
            body: false,
        }
    }

    pub fn caption(self) -> KindData {
        let mut kind = self;
        kind.caption = true;
        kind
    }

    pub fn body(self) -> KindData {
        let mut kind = self;
        kind.body = true;
        kind
    }

    pub fn caption_or_body(self) -> KindData {
        let mut kind = self;
        kind.caption = true;
        kind.body = true;
        kind
    }

    pub(crate) fn into_by_ast_modifier(self, modifier: &ftd::ast::VariableModifier) -> Self {
        match modifier {
            ftd::ast::VariableModifier::Optional => self.optional(),
            ftd::ast::VariableModifier::List => self.list(),
        }
    }

    pub(crate) fn scan_ast_kind(
        var_kind: ftd::ast::VariableKind,
        known_kinds: &ftd::Map<ftd::interpreter2::Kind>,
        doc: &mut ftd::interpreter2::TDoc,
        line_number: usize,
    ) -> ftd::interpreter2::Result<()> {
        let ast_kind = var_kind.kind;
        match ast_kind.as_ref() {
            "string" | "object" | "integer" | "decimal" | "boolean" | "void" | "ftd.ui"
            | "children" => Ok(()),
            k if known_kinds.contains_key(k) => Ok(()),
            k => doc.scan_thing(k, line_number),
        }
    }

    pub(crate) fn from_ast_kind(
        var_kind: ftd::ast::VariableKind,
        known_kinds: &ftd::Map<ftd::interpreter2::Kind>,
        doc: &ftd::interpreter2::TDoc,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::StateWithThing<KindData>> {
        let mut ast_kind = var_kind.kind.clone();
        let (caption, body) = check_for_caption_and_body(&mut ast_kind);
        if ast_kind.is_empty() {
            if !(caption || body) {
                return Err(ftd::interpreter2::utils::invalid_kind_error(
                    ast_kind,
                    doc.name,
                    line_number,
                ));
            }

            let mut kind_data = KindData {
                kind: Kind::String,
                caption,
                body,
            };

            if let Some(ref modifier) = var_kind.modifier {
                kind_data = kind_data.into_by_ast_modifier(modifier);
            }

            return Ok(ftd::interpreter2::StateWithThing::new_thing(kind_data));
        }
        let kind = match ast_kind.as_ref() {
            "string" => Kind::string(),
            "object" => Kind::object(),
            "integer" => Kind::integer(),
            "decimal" => Kind::decimal(),
            "boolean" => Kind::boolean(),
            "void" => Kind::void(),
            "ftd.ui" => Kind::ui(),
            "children" => {
                if let Some(modifier) = var_kind.modifier {
                    return ftd::interpreter2::utils::e2(
                        format!("Can't add modifier `{:?}`", modifier),
                        doc.name,
                        line_number,
                    );
                }
                Kind::List {
                    kind: Box::new(Kind::subsection_ui()),
                }
            }
            k if known_kinds.contains_key(k) => known_kinds.get(k).unwrap().to_owned(),
            k => match try_ok_state!(doc.search_thing(k, line_number)?) {
                ftd::interpreter2::Thing::Record(r) => Kind::record(r.name.as_str()),
                ftd::interpreter2::Thing::Component(_) => Kind::ui(),
                ftd::interpreter2::Thing::OrType(o) => Kind::or_type(o.name.as_str()),
                ftd::interpreter2::Thing::OrTypeWithVariant { or_type, variant } => {
                    Kind::or_type_with_variant(
                        or_type.as_str(),
                        variant.name().as_str(),
                        variant.name().as_str(),
                    )
                }
                t => {
                    return ftd::interpreter2::utils::e2(
                        format!("Can't get find for `{:?}`", t),
                        doc.name,
                        line_number,
                    )
                }
            },
        };

        let mut kind_data = KindData {
            kind,
            caption,
            body,
        };

        if let Some(ref modifier) = var_kind.modifier {
            kind_data = kind_data.into_by_ast_modifier(modifier);
        }

        Ok(ftd::interpreter2::StateWithThing::new_thing(kind_data))
    }

    fn optional(self) -> KindData {
        KindData {
            kind: Kind::Optional {
                kind: Box::new(self.kind),
            },
            caption: self.caption,
            body: self.body,
        }
    }

    fn list(self) -> KindData {
        KindData {
            kind: Kind::List {
                kind: Box::new(self.kind),
            },
            caption: self.caption,
            body: self.body,
        }
    }

    pub fn is_list(&self) -> bool {
        self.kind.is_list()
    }

    pub fn is_or_type(&self) -> bool {
        self.kind.is_or_type()
    }
    pub fn is_optional(&self) -> bool {
        self.kind.is_optional()
    }

    pub fn is_string(&self) -> bool {
        self.kind.is_string()
    }

    pub fn is_integer(&self) -> bool {
        self.kind.is_integer()
    }

    pub fn is_boolean(&self) -> bool {
        self.kind.is_boolean()
    }

    pub fn is_subsection_ui(&self) -> bool {
        self.kind.is_subsection_ui()
    }

    pub fn is_ui(&self) -> bool {
        self.kind.is_ui()
    }

    pub fn is_decimal(&self) -> bool {
        self.kind.is_decimal()
    }

    pub fn is_void(&self) -> bool {
        self.kind.is_void()
    }

    pub fn inner_list(self) -> KindData {
        let kind = match self.kind {
            Kind::List { kind } => kind.as_ref().to_owned(),
            t => t,
        };
        KindData {
            kind,
            caption: self.caption,
            body: self.body,
        }
    }
}

pub fn check_for_caption_and_body(s: &mut String) -> (bool, bool) {
    use itertools::Itertools;

    let mut caption = false;
    let mut body = false;

    let mut expr = s.split_whitespace().collect_vec();

    if expr.is_empty() {
        return (caption, body);
    }

    if is_caption_or_body(expr.as_slice()) {
        caption = true;
        body = true;
        expr = expr[3..].to_vec();
    } else if is_caption(expr[0]) {
        caption = true;
        expr = expr[1..].to_vec();
    } else if is_body(expr[0]) {
        body = true;
        expr = expr[1..].to_vec();
    }

    *s = expr.join(" ");

    (caption, body)
}

pub(crate) fn is_caption_or_body(expr: &[&str]) -> bool {
    if expr.len() < 3 {
        return false;
    }
    if is_caption(expr[0]) && expr[1].eq("or") && is_body(expr[2]) {
        return true;
    }

    if is_body(expr[0]) && expr[1].eq("or") && is_caption(expr[2]) {
        return true;
    }

    false
}

pub(crate) fn is_caption(s: &str) -> bool {
    s.eq("caption")
}

pub fn is_body(s: &str) -> bool {
    s.eq("body")
}
