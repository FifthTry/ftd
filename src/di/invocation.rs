#[derive(Debug, PartialEq)]
pub struct Invocation {
    name: String,
    properties: Vec<ftd::di::Property>,
    children: Vec<ftd::di::DI>,
}

impl Invocation {
    pub(crate) fn is_invocation(section: &ftd::p11::Section) -> bool {
        if ftd::di::Import::is_import(section) || ftd::di::Record::is_record(section) {
            return false;
        }
        section.kind.is_none()
    }

    pub(crate) fn from_p1(
        section: &ftd::p11::Section,
        doc_id: &str,
    ) -> ftd::di::Result<Invocation> {
        if !Self::is_invocation(section) {
            return ftd::di::parse_error(
                format!("Section is not `invocation`, found `{:?}`", section),
                doc_id,
                section.line_number,
            );
        }

        let properties = ftd::di::Property::from_p1(section, doc_id)?;
        let children = ftd::di::DI::from_sections(section.sub_sections.as_slice(), doc_id)?;

        Ok(Invocation {
            name: section.name.to_string(),
            properties,
            children,
        })
    }

    #[cfg(test)]
    pub(crate) fn new(name: &str) -> Invocation {
        Invocation {
            name: name.to_string(),
            properties: vec![],
            children: vec![],
        }
    }

    #[cfg(test)]
    pub(crate) fn add_body(self, s: &str) -> Invocation {
        let mut invocation = self;
        invocation.properties.push(ftd::di::Property::from_body(s));
        invocation
    }

    #[cfg(test)]
    pub(crate) fn add_value_property(
        self,
        key: &str,
        kind: Option<String>,
        value: Option<String>,
    ) -> Invocation {
        let mut invocation = self;
        invocation.properties.push(ftd::di::Property::from_kv(
            &ftd::p11::header::KV::new(key, kind, value, 0),
            ftd::di::Source::Header,
        ));
        invocation
    }

    #[cfg(test)]
    pub(crate) fn add_di_property(
        self,
        key: &str,
        kind: Option<String>,
        di: Vec<ftd::di::DI>,
    ) -> Invocation {
        let mut invocation = self;
        invocation.properties.push(ftd::di::Property::from_di_list(
            key,
            kind,
            di,
            ftd::di::Source::Header,
        ));
        invocation
    }

    #[cfg(test)]
    pub(crate) fn add_caption_str(self, s: &str) -> Invocation {
        let mut invocation = self;
        invocation
            .properties
            .push(ftd::di::Property::from_caption_str(s));
        invocation
    }

    #[cfg(test)]
    pub(crate) fn add_child(self, di: ftd::di::DI) -> Invocation {
        let mut invocation = self;
        invocation.children.push(di);
        invocation
    }
}
