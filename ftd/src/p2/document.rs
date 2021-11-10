#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Document {
    pub data: std::collections::BTreeMap<String, ftd::p2::Thing>,
    pub name: String,
    pub instructions: Vec<ftd::Instruction>,
    pub main: ftd_rt::Column,
    pub p1: Vec<ftd::p1::Section>,
    pub aliases: std::collections::BTreeMap<String, String>,
}

impl ToString for Document {
    fn to_string(&self) -> String {
        ftd::p1::to_string(&self.p1)
    }
}

impl Document {
    fn get_data(&self) -> ftd_rt::Map {
        let mut d: ftd_rt::Map = Default::default();
        for (k, v) in self.data.iter() {
            if let ftd::p2::Thing::Variable(ftd::Variable { value, .. }) = v {
                let value = match value {
                    ftd::Value::Boolean { value } => value.to_string(),
                    ftd::Value::Integer { value } => value.to_string(),
                    ftd::Value::String { text: value, .. } => value.to_string(),
                    _ => continue,
                };
                d.insert(k.to_string(), value);
            }
        }
        d
    }

    fn get_locals(&self) -> ftd_rt::Map {
        ftd_rt::Element::get_locals(&self.main.container.children)
    }

    fn rt_data(&self) -> ftd_rt::DataDependenciesMap {
        let mut d: ftd_rt::Map = self.get_data();
        for (k, v) in self.get_locals() {
            d.insert(format!("@{}", k), v.to_string());
        }

        let mut data: ftd_rt::DataDependenciesMap = Default::default();
        for (k, v) in d {
            data.insert(
                k.to_string(),
                ftd_rt::Data {
                    value: v.to_string(),
                    dependencies: Default::default(),
                },
            );
        }
        ftd_rt::Element::get_visible_event_dependencies(&self.main.container.children, &mut data);
        ftd_rt::Element::get_value_event_dependencies(&self.main.container.children, &mut data);
        ftd_rt::Element::get_style_event_dependencies(&self.main.container.children, &mut data);

        data
    }

    pub fn rerender(&mut self, id: &str) -> crate::p1::Result<ftd_rt::Document> {
        let mut rt = ftd::RT::from(
            self.name.as_str(),
            self.aliases.clone(),
            self.data.clone(),
            self.instructions.clone(),
        );
        self.main = rt.render()?;
        let data = self.rt_data();
        Ok(ftd_rt::Document {
            data,
            html: self.html(id),
            external_children: ftd_rt::Element::get_external_children_dependencies(
                &self.main.container.children,
            ),
        })
    }

    pub fn to_rt(&self, id: &str) -> ftd_rt::Document {
        let external_children =
            ftd_rt::Element::get_external_children_dependencies(&self.main.container.children);

        ftd_rt::Document {
            data: self.rt_data(),
            html: self.html(id),
            external_children,
        }
    }

    pub fn html(&self, id: &str) -> String {
        self.main
            .to_node()
            .to_html(&Default::default(), &self.rt_data(), id)
    }

    pub fn set_string(&mut self, name: &str, value: &str) {
        let thing = ftd::p2::Thing::Variable(ftd::Variable {
            name: name.to_string(),
            value: ftd::Value::String {
                text: value.to_string(),
                source: ftd::TextSource::Header,
            },
            conditions: vec![],
        });
        self.data.insert(name.to_string(), thing);
    }

    pub fn set_bool(&mut self, name: &str, value: bool) {
        let thing = ftd::p2::Thing::Variable(ftd::Variable {
            name: name.to_string(),
            value: ftd::Value::Boolean { value },
            conditions: vec![],
        });
        self.data.insert(name.to_string(), thing);
    }

    pub fn alias(&self, doc: &str) -> Option<&str> {
        for (k, v) in self.aliases.iter() {
            if v == doc {
                return Some(k);
            }
        }

        None
    }

    pub fn find<T, F>(children: &[ftd_rt::Element], f: &F) -> Option<T>
    where
        F: Fn(&ftd_rt::Element) -> Option<T>,
    {
        fn finder<T2, F2>(elements: &[ftd_rt::Element], f: &F2) -> Option<T2>
        where
            F2: Fn(&ftd_rt::Element) -> Option<T2>,
        {
            for e in elements.iter() {
                match e {
                    ftd_rt::Element::Text(_) => {
                        if let Some(v) = f(e) {
                            return Some(v);
                        }
                    }
                    ftd_rt::Element::Input(_) => {
                        if let Some(v) = f(e) {
                            return Some(v);
                        }
                    }
                    ftd_rt::Element::Column(c) => {
                        if let Some(v) = f(e) {
                            return Some(v);
                        }

                        if let Some(t) = finder(&c.container.children, f) {
                            return Some(t);
                        }
                    }
                    ftd_rt::Element::Row(c) => {
                        if let Some(v) = f(e) {
                            return Some(v);
                        }

                        if let Some(t) = finder(&c.container.children, f) {
                            return Some(t);
                        }
                    }
                    ftd_rt::Element::Scene(c) => {
                        if let Some(v) = f(e) {
                            return Some(v);
                        }

                        if let Some(t) = finder(&c.container.children, f) {
                            return Some(t);
                        }
                    }
                    ftd_rt::Element::Image(_) => {
                        if let Some(v) = f(e) {
                            return Some(v);
                        }
                    }
                    ftd_rt::Element::IFrame(_) => {
                        if let Some(v) = f(e) {
                            return Some(v);
                        }
                    }
                    ftd_rt::Element::Decimal(_) => {
                        if let Some(v) = f(e) {
                            return Some(v);
                        }
                    }
                    ftd_rt::Element::Integer(_) => {
                        if let Some(v) = f(e) {
                            return Some(v);
                        }
                    }
                    ftd_rt::Element::Boolean(_) => {
                        if let Some(v) = f(e) {
                            return Some(v);
                        }
                    }
                    ftd_rt::Element::Null => {}
                }
            }
            None
        }

        finder(children, f)
    }

    pub fn find_text<T, F>(children: &[ftd_rt::Element], f: F) -> Option<T>
    where
        F: Fn(&ftd_rt::Text) -> Option<T>,
    {
        Self::find(children, &|e: &ftd_rt::Element| -> Option<T> {
            match e {
                ftd_rt::Element::Text(t) => f(t),
                _ => None,
            }
        })
    }

    pub fn without_render(
        name: &str,
        source: &str,
        lib: &dyn crate::p2::Library,
    ) -> crate::p1::Result<Document> {
        let mut interpreter = crate::p2::interpreter::Interpreter::new(lib);
        let instructions = interpreter.interpret(name, source)?;
        let rt = ftd::RT::from(name, interpreter.aliases, interpreter.bag, instructions);

        Ok(Document {
            main: Default::default(),
            data: rt.bag,
            instructions: rt.instructions,
            p1: interpreter.p1,
            aliases: rt.aliases,
            name: name.to_string(),
        })
    }

    pub fn from(
        name: &str,
        source: &str,
        lib: &dyn crate::p2::Library,
    ) -> crate::p1::Result<Document> {
        let mut d = Self::without_render(name, source, lib)?;

        let mut rt = ftd::RT::from(
            d.name.as_str(),
            d.aliases.clone(),
            d.data.clone(),
            d.instructions.clone(),
        );

        d.main = rt.render()?;
        Ok(d)
    }

    pub fn get_heading<F>(children: &[ftd_rt::Element], f: &F) -> Option<ftd_rt::Rendered>
    where
        F: Fn(&ftd_rt::Region) -> bool,
    {
        if let Some(t) = Self::find_text(children, |t| {
            if t.common.region.as_ref().map(f).unwrap_or(false) {
                Some(t.text.clone())
            } else {
                None
            }
        }) {
            return Some(t);
        }
        if let Some(t) = Self::find(children, &|e| match e {
            ftd_rt::Element::Column(t) => {
                if t.common.region.as_ref().map(f).unwrap_or(false) {
                    Some(t.container.children.clone())
                } else {
                    None
                }
            }
            ftd_rt::Element::Row(t) => {
                if t.common.region.as_ref().map(f).unwrap_or(false) {
                    Some(t.container.children.clone())
                } else {
                    None
                }
            }
            _ => None,
        }) {
            if let Some(t) = Self::find_text(&t, |t| {
                if t.common
                    .region
                    .as_ref()
                    .map(|r| r.is_title())
                    .unwrap_or(false)
                {
                    Some(t.text.clone())
                } else {
                    None
                }
            }) {
                return Some(t);
            };
            return Self::find_text(&t, |t| if t.line { Some(t.text.clone()) } else { None });
        }
        None
    }

    pub fn title(&self) -> Option<ftd_rt::Rendered> {
        // find the text of first primary heading
        if let Some(t) =
            Self::get_heading(&self.main.container.children, &|r| r.is_primary_heading())
        {
            return Some(t);
        }

        // find any heading
        if let Some(t) = Self::get_heading(&self.main.container.children, &|r| r.is_heading()) {
            return Some(t);
        }

        // find any text with caption
        if let Some(t) = Self::find_text(&self.main.container.children, |t| {
            if t.line {
                Some(t.text.clone())
            } else {
                None
            }
        }) {
            return Some(t);
        }

        None
    }

    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> crate::p1::Result<T> {
        let v = self.json(key)?;
        Ok(serde_json::from_value(v)?)
    }

    pub fn name(&self, k: &str) -> String {
        if k.contains('#') {
            k.to_string()
        } else {
            format!("{}#{}", self.name.as_str(), k)
        }
    }

    pub fn only_instance<T>(&self, record: &str) -> crate::p1::Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let v = self.instances::<T>(record)?;
        if v.is_empty() {
            return Ok(None);
        }
        if v.len() > 1 {
            return crate::e(format!(
                "more than one instances({}) of {} found",
                v.len(),
                record
            ));
        }
        Ok(Some(v.into_iter().next().unwrap())) // unwrap okay because v not empty
    }

    pub fn instances<T>(&self, record: &str) -> crate::p1::Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let name = self.name(record);
        let thing = match self.data.get(name.as_str()) {
            Some(t) => t,
            None => return Ok(vec![]),
        };

        let json = match thing {
            crate::p2::Thing::Record(r) => {
                let mut a = vec![];
                for c in match r.instances.get(self.name.as_str()) {
                    Some(v) => v.iter(),
                    None => return Ok(vec![]),
                } {
                    a.push(self.object_to_json(None, c)?);
                }
                serde_json::Value::Array(a)
            }
            t => return ftd::e2("not a record", t),
        };

        Ok(serde_json::from_value(json)?)
    }

    #[cfg(calls)]
    pub fn calls<T: serde::de::DeserializeOwned>(
        &self,
        component: &str,
    ) -> crate::p1::Result<Vec<T>> {
        let component = self.name(component);
        let thing = match self.data.get(component.as_str()) {
            Some(t) => t,
            None => return Ok(vec![]),
        };

        let json = match thing {
            crate::p2::Thing::Component(c) => {
                let mut a = vec![];
                for c in c.invocations.iter() {
                    a.push(self.object2_to_json(c)?);
                }
                serde_json::Value::Array(a)
            }
            t => panic!("{:?} is not a component", t),
        };

        Ok(serde_json::from_value(json)?)
    }

    pub fn json(&self, key: &str) -> crate::p1::Result<serde_json::Value> {
        let key = self.name(key);
        let thing = match self.data.get(key.as_str()) {
            Some(v) => v,
            None => {
                return Err(crate::p1::Error::NotFound {
                    key: key.to_string(),
                })
            }
        };

        match thing {
            crate::p2::Thing::Variable(v) => self.value_to_json(&v.value),
            t => panic!("{:?} is not a variable", t),
        }
    }

    fn value_to_json(&self, v: &crate::Value) -> crate::p1::Result<serde_json::Value> {
        Ok(match v {
            crate::Value::Integer { value } => {
                serde_json::Value::Number(serde_json::Number::from(*value))
            }
            crate::Value::Boolean { value } => serde_json::Value::Bool(*value),
            crate::Value::Decimal { value } => {
                serde_json::Value::Number(serde_json::Number::from_f64(*value).unwrap())
                // TODO: remove unwrap
            }
            crate::Value::String { text, .. } => serde_json::Value::String(text.to_owned()),
            crate::Value::Record { fields, .. } => self.object_to_json(None, fields)?,
            crate::Value::OrType {
                variant, fields, ..
            } => self.object_to_json(Some(variant), fields)?,
            crate::Value::List { data, .. } => self.list_to_json(data)?,
            crate::Value::None { .. } => serde_json::Value::Null,
            _ => return ftd::e2("unhandled value found(value_to_json)", v),
        })
    }

    fn list_to_json(&self, data: &[crate::Value]) -> crate::p1::Result<serde_json::Value> {
        let mut list = vec![];
        for item in data.iter() {
            list.push(self.value_to_json(item)?)
        }
        Ok(serde_json::Value::Array(list))
    }

    #[cfg(calls)]
    fn object2_to_json(
        &self,
        fields: &std::collections::BTreeMap<String, crate::Value>,
    ) -> crate::p1::Result<serde_json::Value> {
        let mut map = serde_json::Map::new();
        for (k, v) in fields.iter() {
            map.insert(k.to_string(), self.value_to_json(v)?);
        }
        Ok(serde_json::Value::Object(map))
    }

    fn object_to_json(
        &self,
        variant: Option<&String>,
        fields: &std::collections::BTreeMap<String, crate::PropertyValue>,
    ) -> crate::p1::Result<serde_json::Value> {
        let mut map = serde_json::Map::new();
        if let Some(v) = variant {
            map.insert("type".to_string(), serde_json::Value::String(v.to_owned()));
        }
        for (k, v) in fields.iter() {
            map.insert(k.to_string(), self.property_value_to_json(v)?);
        }
        Ok(serde_json::Value::Object(map))
    }

    fn property_value_to_json(
        &self,
        v: &crate::PropertyValue,
    ) -> crate::p1::Result<serde_json::Value> {
        match v {
            crate::PropertyValue::Value { value, .. } => self.value_to_json(value),
            crate::PropertyValue::Reference { name, .. } => self.json(name),
            _ => unreachable!(),
        }
    }
}

pub fn set_region_id(elements: &mut Vec<ftd_rt::Element>) {
    let mut map: std::collections::BTreeMap<usize, String> = Default::default();
    for element in elements.iter_mut() {
        match element {
            ftd_rt::Element::Column(ftd_rt::Column { container, .. })
            | ftd_rt::Element::Row(ftd_rt::Row { container, .. }) => {
                set_region_id(&mut container.children);
                if let Some((_, _, ref mut e)) = container.external_children {
                    set_region_id(e);
                }
            }
            _ => continue,
        }
    }

    for (idx, element) in elements.iter().enumerate() {
        match element {
            ftd_rt::Element::Column(ftd_rt::Column { common, .. })
            | ftd_rt::Element::Row(ftd_rt::Row { common, .. }) => {
                if common.region.as_ref().filter(|v| v.is_heading()).is_some()
                    && common.data_id.is_none()
                {
                    if let Some(h) =
                        ftd::p2::Document::get_heading(vec![element.clone()].as_slice(), &|r| {
                            r.is_heading()
                        })
                    {
                        map.insert(idx, slug::slugify(h.original));
                    }
                }
            }
            _ => continue,
        }
    }
    for (idx, s) in map {
        elements[idx].get_mut_common().unwrap().id = Some(s);
    }
}

pub fn default_scene_children_position(elements: &mut Vec<ftd_rt::Element>) {
    for element in elements {
        if let ftd_rt::Element::Scene(scene) = element {
            for child in &mut scene.container.children {
                check_and_set_default_position(child);
            }
            if let Some((_, _, ref mut ext_children)) = scene.container.external_children {
                for child in ext_children {
                    check_and_set_default_position(child);
                }
            }
        }
        match element {
            ftd_rt::Element::Scene(ftd_rt::Scene { container, .. })
            | ftd_rt::Element::Row(ftd_rt::Row { container, .. })
            | ftd_rt::Element::Column(ftd_rt::Column { container, .. }) => {
                default_scene_children_position(&mut container.children);
                if let Some((_, _, ref mut ext_children)) = container.external_children {
                    default_scene_children_position(ext_children);
                }
            }
            _ => {}
        }
    }

    fn check_and_set_default_position(child: &mut ftd_rt::Element) {
        if let Some(common) = child.get_mut_common() {
            if common.top.is_none() && common.bottom.is_none() {
                common.top = Some(0);
            }
            if common.left.is_none() && common.right.is_none() {
                common.left = Some(0);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::test::*;

    #[test]
    fn variable_from_other_doc() {
        let bag = super::Document::from(
            "foo/bar",
            indoc::indoc!(
                "
            -- import: fifthtry/ft
            -- ft.toc:

            foo is the toc
            "
            ),
            &ftd::p2::TestLibrary {},
        )
        .unwrap();

        pretty_assertions::assert_eq!(
            bag.get::<String>("fifthtry/ft#toc").unwrap(),
            "foo is the toc"
        );
    }

    #[test]
    fn meta() {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        #[serde(tag = "type")]
        enum Someone {
            Username { username: String },
            Who { who: String },
        }

        #[derive(Debug, PartialEq, serde::Deserialize)]
        struct Meta {
            license: String,
            reader: Vec<Someone>,
        }

        let bag = super::Document::from(
            "foo/bar",
            indoc::indoc!(
                "
                -- or-type someone:

                --- Username:
                username: caption

                --- Who:
                who: caption

                -- record meta_type:
                license: string
                reader: list someone

                -- list meta:
                type: meta_type

                -- meta:
                license: BSD

                --- reader.Username: foo
                --- reader.Who: everyone
            "
            ),
            &ftd::p2::TestLibrary {},
        )
        .unwrap();

        pretty_assertions::assert_eq!(
            bag.get::<Vec<Meta>>("meta").unwrap(),
            vec![Meta {
                license: s("BSD"),
                reader: vec![
                    Someone::Username { username: s("foo") },
                    Someone::Who { who: s("everyone") }
                ],
            }]
        )
    }

    #[test]
    #[cfg(calls)]
    #[ignore] // TODO: this is buggy
    fn calls() {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        struct PR {
            number: i64,
            title: String,
        }

        let bag = super::Document::from(
            "foo/bar",
            indoc::indoc!(
                "
                -- component pr:
                $number: integer
                $title: caption
                component: ftd.text
                text: ref $title

                -- pr: some pr
                number: 24

                -- pr: some other pr
                number: 224
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .unwrap();

        pretty_assertions::assert_eq!(
            bag.instances::<PR>("pr").unwrap(),
            vec![
                PR {
                    number: 24,
                    title: s("some pr")
                },
                PR {
                    number: 224,
                    title: s("some other pr")
                }
            ]
        )
    }

    #[test]
    fn instances() {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        struct PR {
            number: i64,
            title: String,
        }

        let bag = super::Document::from(
            "foo/bar",
            indoc::indoc!(
                "
                -- record pr:
                number: integer
                title: caption

                -- pr: some pr
                number: 24

                -- pr: some other pr
                number: 224
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .unwrap();

        pretty_assertions::assert_eq!(
            bag.instances::<PR>("pr").unwrap(),
            vec![
                PR {
                    number: 24,
                    title: s("some pr")
                },
                PR {
                    number: 224,
                    title: s("some other pr")
                }
            ]
        )
    }
}
