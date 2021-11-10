#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct RT {
    pub name: String,
    pub aliases: std::collections::BTreeMap<String, String>,
    pub bag: std::collections::BTreeMap<String, crate::p2::Thing>,
    pub instructions: Vec<ftd::Instruction>,
}

impl RT {
    pub fn from(
        name: &str,
        aliases: std::collections::BTreeMap<String, String>,
        bag: std::collections::BTreeMap<String, crate::p2::Thing>,
        instructions: Vec<ftd::Instruction>,
    ) -> Self {
        Self {
            name: name.to_string(),
            aliases,
            bag,
            instructions,
        }
    }

    pub fn set_bool(&mut self, variable: &str, value: bool) -> crate::p1::Result<bool> {
        match self.bag.get(variable) {
            Some(ftd::p2::Thing::Variable(v)) => match v.value {
                ftd::Value::Boolean { value: old } => {
                    let conditions = v.conditions.to_vec();
                    self.bag.insert(
                        variable.to_string(),
                        ftd::p2::Thing::Variable(ftd::Variable {
                            name: variable.to_string(),
                            value: ftd::Value::Boolean { value },
                            conditions,
                        }),
                    );
                    Ok(old)
                }
                ref t => crate::e2(
                    format!("{} is not a boolean", variable),
                    format!("{:?}", t).as_str(),
                ),
            },
            Some(t) => crate::e2(
                format!("{} is not a variable", variable),
                format!("{:?}", t).as_str(),
            ),
            None => crate::e(format!("{} not found", variable)),
        }
    }

    pub fn render(&mut self) -> crate::p1::Result<ftd_rt::Column> {
        let mut main = self.render_();
        if let Ok(main) = &mut main {
            ftd_rt::Element::set_id(&mut main.container.children, &[], None);
        }
        main
    }

    pub fn render_(&mut self) -> crate::p1::Result<ftd_rt::Column> {
        let mut main = ftd::p2::interpreter::default_column();
        let mut invocations = Default::default();
        let mut element = ftd::execute_doc::ExecuteDoc {
            name: self.name.as_str(),
            aliases: &self.aliases,
            bag: &self.bag,
            instructions: &self.instructions,
            arguments: &Default::default(),
            invocations: &mut invocations,
            root_name: None,
        }
        .execute(&[], &mut Default::default(), None)?
        .children;

        ftd_rt::Element::renesting_on_region(&mut element);
        ftd::p2::document::set_region_id(&mut element);
        ftd::p2::document::default_scene_children_position(&mut element);

        main.container.children = element;
        store_invocations(&mut self.bag, invocations);
        Ok(main)
    }
}

pub(crate) fn store_invocations(
    bag: &mut std::collections::BTreeMap<String, crate::p2::Thing>,
    invocations: std::collections::BTreeMap<
        String,
        Vec<std::collections::BTreeMap<String, crate::Value>>,
    >,
) {
    for (k, v) in invocations.into_iter() {
        match bag.get_mut(k.as_str()).unwrap() {
            crate::p2::Thing::Component(ref mut c) => {
                if !c.kernel {
                    c.invocations.extend(v)
                }
            }
            _ => unreachable!(),
        }
    }
}
