#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ComponentDefinition {
    pub name: String,
    pub arguments: Vec<Argument>,
    pub definition: Component,
    pub line_number: usize,
}

impl ComponentDefinition {
    pub(crate) fn new(
        name: &str,
        arguments: Vec<Argument>,
        definition: Component,
        line_number: usize,
    ) -> ComponentDefinition {
        ComponentDefinition {
            name: name.to_string(),
            arguments,
            definition,
            line_number,
        }
    }

    pub(crate) fn from_ast(
        ast: ftd::ast::AST,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<ComponentDefinition> {
        let component_definition = ast.get_component_definition(doc.name)?;
        let name = doc.resolve_name(component_definition.name.as_str());
        let arguments =
            Argument::from_ast_fields(component_definition.arguments, doc, &Default::default())?;
        let definition_name_with_arguments =
            (component_definition.name.as_str(), arguments.as_slice());
        let definition = Component::from_ast_component(
            component_definition.definition,
            Some(definition_name_with_arguments),
            doc,
        )?;
        Ok(ComponentDefinition::new(
            name.as_str(),
            arguments,
            definition,
            component_definition.line_number,
        ))
    }

    pub fn to_value(&self, kind: &ftd::interpreter2::KindData) -> ftd::interpreter2::Value {
        ftd::interpreter2::Value::UI {
            name: self.name.to_string(),
            kind: kind.to_owned(),
            component: self.definition.to_owned(),
        }
    }
}

pub type Argument = ftd::interpreter2::Field;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Component {
    pub name: String,
    pub properties: Vec<Property>,
    pub iteration: Box<Option<Loop>>,
    pub condition: Box<Option<ftd::interpreter2::Boolean>>,
    pub events: Vec<Event>,
    pub children: Vec<Component>,
    pub line_number: usize,
}

impl Component {
    pub(crate) fn from_name(name: &str) -> Component {
        Component {
            name: name.to_string(),
            properties: vec![],
            iteration: Box::new(None),
            condition: Box::new(None),
            events: vec![],
            children: vec![],
            line_number: 0,
        }
    }

    pub fn get_children(
        &self,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<Vec<Component>> {
        let property = if let Some(property) = self
            .properties
            .iter()
            .find(|v| v.value.kind().inner_list().is_subsection_ui())
        {
            property
        } else {
            return Ok(vec![]);
        };

        let value = property.value.clone().resolve(doc, property.line_number)?;
        if let ftd::interpreter2::Value::UI { component, .. } = value {
            return Ok(vec![component]);
        }
        if let ftd::interpreter2::Value::List { data, kind } = value {
            if kind.is_ui() {
                let mut children = vec![];
                for value in data {
                    let value = value.resolve(doc, property.line_number)?;
                    if let ftd::interpreter2::Value::UI { component, .. } = value {
                        children.push(component);
                    }
                }
                return Ok(children);
            }
        }

        Ok(vec![])
    }

    pub(crate) fn is_loop(&self) -> bool {
        self.iteration.is_some()
    }

    pub(crate) fn from_ast(
        ast: ftd::ast::AST,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<Component> {
        let component_invocation = ast.get_component_invocation(doc.name)?;
        Component::from_ast_component(component_invocation, None, doc)
    }

    pub(crate) fn from_ast_component(
        ast_component: ftd::ast::Component,
        definition_name_with_arguments: Option<(&str, &[Argument])>,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<Component> {
        let name = doc.resolve_name(ast_component.name.as_str());

        let mut loop_object_name_and_kind = None;
        let iteration = if let Some(v) = ast_component.iteration {
            let iteration = Loop::from_ast_loop(v, definition_name_with_arguments, doc)?;
            loop_object_name_and_kind = Some((
                iteration.alias.to_string(),
                iteration.loop_object_as_argument(doc)?,
            ));
            Some(iteration)
        } else {
            None
        };

        let condition = if let Some(v) = ast_component.condition {
            Some(ftd::interpreter2::Boolean::from_ast_condition(
                v,
                definition_name_with_arguments,
                &loop_object_name_and_kind,
                doc,
            )?)
        } else {
            None
        };

        let events = Event::from_ast_events(
            ast_component.events,
            definition_name_with_arguments,
            &loop_object_name_and_kind,
            doc,
        )?;

        let properties = Property::from_ast_properties_and_children(
            ast_component.properties,
            ast_component.children,
            ast_component.name.as_str(),
            definition_name_with_arguments,
            &loop_object_name_and_kind,
            doc,
            ast_component.line_number,
        )?;

        Ok(Component {
            name,
            properties,
            iteration: Box::new(iteration),
            condition: Box::new(condition),
            events,
            children: vec![],
            line_number: ast_component.line_number,
        })
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum PropertySource {
    Caption,
    Body,
    Header { name: String, mutable: bool },
    Subsection,
}

impl PropertySource {
    pub fn is_equal(&self, other: &PropertySource) -> bool {
        match self {
            PropertySource::Caption | PropertySource::Body | PropertySource::Subsection => {
                self.eq(other)
            }
            PropertySource::Header { name, .. } => matches!(other, PropertySource::Header {
                    name: other_name, ..
               } if other_name.eq(name)),
        }
    }
}

impl From<ftd::ast::PropertySource> for PropertySource {
    fn from(item: ftd::ast::PropertySource) -> Self {
        match item {
            ftd::ast::PropertySource::Caption => PropertySource::Caption,
            ftd::ast::PropertySource::Body => PropertySource::Body,
            ftd::ast::PropertySource::Header { name, mutable } => {
                PropertySource::Header { name, mutable }
            }
        }
    }
}

impl Default for PropertySource {
    fn default() -> PropertySource {
        PropertySource::Caption
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Property {
    pub value: ftd::interpreter2::PropertyValue,
    pub source: ftd::interpreter2::PropertySource,
    pub condition: Option<ftd::interpreter2::Boolean>,
    pub line_number: usize,
}

impl Property {
    pub(crate) fn resolve(
        &self,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<Option<ftd::interpreter2::Value>> {
        Ok(match self.condition {
            Some(ref condition) if !condition.eval(doc)? => None,
            _ => Some(self.value.clone().resolve(doc, self.line_number)?),
        })
    }

    fn from_ast_properties_and_children(
        ast_properties: Vec<ftd::ast::Property>,
        ast_children: Vec<ftd::ast::Component>,
        component_name: &str,
        definition_name_with_arguments: Option<(&str, &[Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter2::Argument)>,
        doc: &ftd::interpreter2::TDoc,
        line_number: usize,
    ) -> ftd::interpreter2::Result<Vec<Property>> {
        let mut properties = Property::from_ast_properties(
            ast_properties,
            component_name,
            definition_name_with_arguments,
            loop_object_name_and_kind,
            doc,
            line_number,
        )?;

        validate_children_kind_property_against_children(
            properties.as_slice(),
            ast_children.as_slice(),
            doc.name,
        )?;

        if let Some(property) = Property::from_ast_children(
            ast_children,
            component_name,
            definition_name_with_arguments,
            doc,
        )? {
            properties.push(property)
        }

        return Ok(properties);

        fn validate_children_kind_property_against_children(
            properties: &[Property],
            ast_children: &[ftd::ast::Component],
            doc_id: &str,
        ) -> ftd::interpreter2::Result<()> {
            use itertools::Itertools;

            let properties = properties
                .iter()
                .filter(|v| v.value.kind().inner_list().is_subsection_ui())
                .collect_vec();

            if properties.is_empty() {
                return Ok(());
            }

            let first_property = properties.first().unwrap();

            if properties.len() > 1 {
                return ftd::interpreter2::utils::e2(
                    "Can't pass multiple children",
                    doc_id,
                    first_property.line_number,
                );
            }

            if !ast_children.is_empty() {
                return ftd::interpreter2::utils::e2(
                    "Can't have children passed in both subsection and header",
                    doc_id,
                    first_property.line_number,
                );
            }

            if first_property.condition.is_some() {
                return ftd::interpreter2::utils::e2(
                    "Not supporting condition for children",
                    doc_id,
                    first_property.line_number,
                );
            }

            Ok(())
        }
    }

    fn get_argument_for_children(component_arguments: &[Argument]) -> Option<&Argument> {
        component_arguments
            .iter()
            .find(|v| v.kind.kind.clone().inner_list().is_subsection_ui())
    }

    fn from_ast_children(
        ast_children: Vec<ftd::ast::Component>,
        component_name: &str,
        definition_name_with_arguments: Option<(&str, &[Argument])>,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<Option<Property>> {
        if ast_children.is_empty() {
            return Ok(None);
        }

        let line_number = ast_children.first().unwrap().line_number;
        let component_arguments = Argument::for_component(
            component_name,
            &definition_name_with_arguments,
            doc,
            line_number,
        )?;

        let _argument = Property::get_argument_for_children(&component_arguments).ok_or(
            ftd::interpreter2::Error::ParseError {
                message: "Subsection is unexpected".to_string(),
                doc_id: doc.name.to_string(),
                line_number,
            },
        )?;

        let children = {
            let mut children = vec![];
            for child in ast_children {
                children.push(Component::from_ast_component(
                    child,
                    definition_name_with_arguments,
                    doc,
                )?);
            }
            children
        };

        let value = ftd::interpreter2::PropertyValue::Value {
            value: ftd::interpreter2::Value::List {
                data: children
                    .into_iter()
                    .map(|v| ftd::interpreter2::PropertyValue::Value {
                        line_number: v.line_number,
                        value: ftd::interpreter2::Value::UI {
                            name: v.name.to_string(),
                            kind: ftd::interpreter2::Kind::subsection_ui().into_kind_data(),
                            component: v,
                        },
                        is_mutable: false,
                    })
                    .collect(),
                kind: ftd::interpreter2::Kind::subsection_ui().into_kind_data(),
            },
            is_mutable: false,
            line_number,
        };

        Ok(Some(Property {
            value,
            source: ftd::interpreter2::PropertySource::Subsection,
            condition: None,
            line_number,
        }))
    }

    fn from_ast_properties(
        ast_properties: Vec<ftd::ast::Property>,
        component_name: &str,
        definition_name_with_arguments: Option<(&str, &[Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter2::Argument)>,
        doc: &ftd::interpreter2::TDoc,
        line_number: usize,
    ) -> ftd::interpreter2::Result<Vec<Property>> {
        let mut properties = vec![];
        let component_arguments = Argument::for_component(
            component_name,
            &definition_name_with_arguments,
            doc,
            line_number,
        )?;
        for property in ast_properties {
            properties.push(Property::from_ast_property(
                property,
                component_name,
                component_arguments.as_slice(),
                definition_name_with_arguments,
                loop_object_name_and_kind,
                doc,
            )?);
        }
        Ok(properties)
    }

    fn from_ast_property(
        ast_property: ftd::ast::Property,
        component_name: &str,
        component_arguments: &[Argument],
        definition_name_with_arguments: Option<(&str, &[Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter2::Argument)>,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<Property> {
        let argument = Property::get_argument_for_property(
            &ast_property,
            component_name,
            component_arguments,
            doc,
        )?;

        let value = ftd::interpreter2::PropertyValue::from_ast_value_with_argument(
            ast_property.value.to_owned(),
            doc,
            argument.mutable,
            Some(&argument.kind),
            definition_name_with_arguments,
            loop_object_name_and_kind,
        )?;

        let condition = if let Some(ref v) = ast_property.condition {
            Some(ftd::interpreter2::Boolean::from_ast_condition(
                ftd::ast::Condition::new(v, ast_property.line_number),
                definition_name_with_arguments,
                loop_object_name_and_kind,
                doc,
            )?)
        } else {
            None
        };

        if ast_property.value.is_null() && !argument.kind.is_optional() {
            return ftd::interpreter2::utils::e2(
                format!(
                    "Excepted Value for argument {} in component {}",
                    argument.name, component_name
                ),
                doc.name,
                ast_property.line_number,
            );
        }

        Ok(Property {
            value,
            source: ast_property.source.into(),
            condition,
            line_number: ast_property.line_number,
        })
    }

    fn get_argument_for_property(
        ast_property: &ftd::ast::Property,
        component_name: &str,
        component_argument: &[Argument],
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<Argument> {
        match &ast_property.source {
            ftd::ast::PropertySource::Caption => component_argument
                .iter()
                .find(|v| v.is_caption())
                .ok_or(ftd::interpreter2::Error::ParseError {
                    message: format!(
                        "Caption type argument not found for component `{}`",
                        component_name
                    ),
                    doc_id: doc.name.to_string(),
                    line_number: ast_property.line_number,
                })
                .map(ToOwned::to_owned),
            ftd::ast::PropertySource::Body => component_argument
                .iter()
                .find(|v| v.is_body())
                .ok_or(ftd::interpreter2::Error::ParseError {
                    message: format!(
                        "Body type argument not found for component `{}`",
                        component_name
                    ),
                    doc_id: doc.name.to_string(),
                    line_number: ast_property.line_number,
                })
                .map(ToOwned::to_owned),
            ftd::ast::PropertySource::Header { name, mutable } => {
                let argument = component_argument.iter().find(|v| v.name.eq(name)).ok_or(
                    ftd::interpreter2::Error::ParseError {
                        message: format!(
                            "Header type {} {} argument not found for component `{}`",
                            name, mutable, component_name
                        ),
                        doc_id: doc.name.to_string(),
                        line_number: ast_property.line_number,
                    },
                )?;
                if !argument.mutable.eq(mutable) {
                    let mutable = if argument.mutable {
                        "mutable"
                    } else {
                        "immutable"
                    };
                    return ftd::interpreter2::utils::e2(
                        format!("Expected `{}` for {}", mutable, argument.name),
                        doc.name,
                        ast_property.line_number,
                    );
                }
                Ok(argument.to_owned())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Loop {
    pub on: ftd::interpreter2::PropertyValue,
    pub alias: String,
    pub line_number: usize,
}

impl Loop {
    fn new(on: ftd::interpreter2::PropertyValue, alias: &str, line_number: usize) -> Loop {
        Loop {
            on,
            alias: alias.to_string(),
            line_number,
        }
    }

    pub(crate) fn loop_object_as_argument(
        &self,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Argument> {
        let kind = self.loop_object_kind(doc.name)?;
        Ok(ftd::interpreter2::Argument {
            name: self.alias.to_string(),
            kind: ftd::interpreter2::KindData::new(kind),
            mutable: self.on.is_mutable(),
            value: Some(self.on.to_owned()),
            line_number: self.on.line_number(),
        })
    }

    pub(crate) fn loop_object_kind(
        &self,
        doc_id: &str,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Kind> {
        let kind = self.on.kind();
        match kind {
            ftd::interpreter2::Kind::List { kind } => Ok(kind.as_ref().to_owned()),
            t => ftd::interpreter2::utils::e2(
                format!("Expected list kind, found: {:?}", t),
                doc_id,
                self.line_number,
            ),
        }
    }

    fn from_ast_loop(
        ast_loop: ftd::ast::Loop,
        definition_name_with_arguments: Option<(&str, &[Argument])>,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<Loop> {
        let on = ftd::interpreter2::PropertyValue::from_string_with_argument(
            ast_loop.on.as_str(),
            doc,
            None,
            false,
            ast_loop.line_number,
            definition_name_with_arguments,
            &None,
        )?;

        Ok(Loop::new(on, ast_loop.alias.as_str(), ast_loop.line_number))
    }

    pub fn children(
        &self,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<(
        Vec<ftd::interpreter2::PropertyValue>,
        ftd::interpreter2::KindData,
    )> {
        let value = self.on.clone().resolve(doc, self.line_number)?;
        if let ftd::interpreter2::Value::List { data, kind } = value {
            Ok((data, kind))
        } else {
            ftd::interpreter2::utils::e2(
                format!("Expected list type data, found: {:?}", self.on),
                doc.name,
                self.line_number,
            )
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Event {
    pub name: ftd::interpreter2::EventName,
    pub action: ftd::interpreter2::FunctionCall,
    line_number: usize,
}

impl Event {
    fn from_ast_event(
        ast_event: ftd::ast::Event,
        definition_name_with_arguments: Option<(&str, &[Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter2::Argument)>,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<Event> {
        let action = ftd::interpreter2::FunctionCall::from_string(
            ast_event.action.as_str(),
            doc,
            false,
            definition_name_with_arguments,
            loop_object_name_and_kind,
            ast_event.line_number,
        )?;

        let event_name = ftd::interpreter2::EventName::from_string(
            ast_event.name.as_str(),
            doc.name,
            ast_event.line_number,
        )?;

        Ok(Event {
            name: event_name,
            action,
            line_number: ast_event.line_number,
        })
    }

    fn from_ast_events(
        ast_events: Vec<ftd::ast::Event>,
        definition_name_with_arguments: Option<(&str, &[Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter2::Argument)>,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<Vec<Event>> {
        let mut events = vec![];
        for event in ast_events {
            events.push(Event::from_ast_event(
                event,
                definition_name_with_arguments,
                loop_object_name_and_kind,
                doc,
            )?);
        }
        Ok(events)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum EventName {
    Click,
}

impl EventName {
    pub(crate) fn from_string(
        e: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::EventName> {
        match e {
            "click" => Ok(EventName::Click),
            t => ftd::interpreter2::utils::e2(
                format!("`{}` event not found", t),
                doc_id,
                line_number,
            ),
        }
    }
}
