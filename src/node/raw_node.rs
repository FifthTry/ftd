#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct RawNode {
    pub name: String,
    pub node: ftd::node::Node,
    pub arguments: Vec<ftd::interpreter2::Argument>,
}

impl RawNode {
    pub(crate) fn from_element_constructor(
        element_constructor: ftd::executor::ElementConstructor,
        doc_id: &str,
    ) -> RawNode {
        RawNode {
            name: element_constructor.name.to_string(),
            node: element_constructor.element.to_node(doc_id, &mut vec![]),
            arguments: element_constructor.arguments,
        }
    }
    pub(crate) fn from_element_constructors(
        element_constructors: ftd::Map<ftd::executor::ElementConstructor>,
        doc_id: &str,
    ) -> ftd::Map<RawNode> {
        element_constructors
            .into_iter()
            .map(|(k, v)| (k, RawNode::from_element_constructor(v, doc_id)))
            .collect()
    }
}

/*pub struct HelperNode {
    pub name: String,
    pub properties: Vec<(String, ftd::interpreter2::Property)>,
    pub iteration: Option<ftd::interpreter2::Loop>,
    pub node: ftd::node::Node,
}*/

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct DummyNode {
    pub parent_container: Vec<usize>,
    pub start_index: usize,
    pub main: ftd::node::Node,
}

impl DummyNode {
    pub(crate) fn new(
        parent_container: Vec<usize>,
        start_index: usize,
        main: ftd::node::Node,
    ) -> DummyNode {
        DummyNode {
            parent_container,
            start_index,
            main,
        }
    }

    pub(crate) fn from_dummy_instruction(
        dummy_element: ftd::executor::DummyElement,
        doc_id: &str,
    ) -> DummyNode {
        DummyNode::new(
            dummy_element.parent_container.to_owned(),
            dummy_element.start_index,
            dummy_element.element.to_node(doc_id, &mut vec![]),
        )
    }

    pub(crate) fn from_dummy_instructions(
        dummy_instructions: ftd::Map<ftd::executor::DummyElement>,
        doc_id: &str,
    ) -> ftd::Map<DummyNode> {
        dummy_instructions
            .into_iter()
            .map(|(k, v)| (k, DummyNode::from_dummy_instruction(v, doc_id)))
            .collect()
    }
}
