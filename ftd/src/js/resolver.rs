#[derive(Debug)]
pub struct ResolverData<'a> {
    pub component_definition_name: &'a Option<String>,
    pub loop_alias: &'a Option<String>,
    pub inherited_variable_name: &'a str,
    pub device: &'a Option<fastn_js::DeviceType>,
}

impl<'a> ResolverData<'a> {
    pub(crate) fn none() -> ResolverData<'a> {
        ResolverData {
            component_definition_name: &None,
            loop_alias: &None,
            inherited_variable_name: fastn_js::INHERITED_VARIABLE,
            device: &None,
        }
    }

    pub(crate) fn new_with_component_definition_name(
        component_definition_name: &'a Option<String>,
    ) -> ResolverData<'a> {
        let mut rdata = ResolverData::none();
        rdata.component_definition_name = component_definition_name;
        rdata
    }

    pub(crate) fn clone_with_default_inherited_variable(&self) -> ResolverData<'a> {
        ResolverData {
            component_definition_name: self.component_definition_name,
            loop_alias: self.loop_alias,
            inherited_variable_name: fastn_js::INHERITED_VARIABLE,
            device: self.device,
        }
    }

    pub(crate) fn clone_with_new_inherited_variable(
        &self,
        inherited_variable_name: &'a str,
    ) -> ResolverData<'a> {
        ResolverData {
            component_definition_name: self.component_definition_name,
            loop_alias: self.loop_alias,
            inherited_variable_name,
            device: self.device,
        }
    }

    pub(crate) fn clone_with_new_loop_alias(
        &self,
        loop_alias: &'a Option<String>,
    ) -> ResolverData<'a> {
        ResolverData {
            component_definition_name: self.component_definition_name,
            loop_alias,
            inherited_variable_name: self.inherited_variable_name,
            device: self.device,
        }
    }
}
