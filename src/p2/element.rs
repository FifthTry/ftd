#[allow(clippy::too_many_arguments)]
pub fn common_from_properties(
    properties: &std::collections::BTreeMap<String, ftd::Value>,
    doc: &ftd::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd::Map,
    root_name: Option<&str>,
    reference: Option<String>,
) -> ftd::p1::Result<ftd::Common> {
    let submit = ftd::p2::utils::string_optional("submit", properties, doc.name, 0)?;
    let link = ftd::p2::utils::string_optional("link", properties, doc.name, 0)?;
    if let (Some(_), Some(_)) = (&submit, &link) {
        return ftd::e2("Cannot have both submit and link together", doc.name, 0);
    }
    let gradient_color_str =
        ftd::p2::utils::string_optional("gradient-colors", properties, doc.name, 0)?;

    let gradient_colors: Vec<ftd::Color> = match gradient_color_str {
        Some(f) => f
            .split(',')
            .flat_map(|x| color_from(Some(x.to_string()), doc.name).ok()?)
            .collect(),
        None => vec![],
    };
    let anchor = ftd::Anchor::from(
        ftd::p2::utils::string_optional("anchor", properties, doc.name, 0)?,
        doc.name,
    )?;

    let inner_default = match anchor {
        Some(ref p) => match p {
            ftd::Anchor::Parent => false,
            ftd::Anchor::Window => true,
        },
        None => false,
    };

    let arguments = {
        //remove properties
        let mut arguments_without_properties: std::collections::BTreeMap<String, ftd::Value> =
            Default::default();
        for (k, v) in properties {
            if let Some(k) = k.strip_prefix('$') {
                arguments_without_properties.insert(k.to_string(), v.to_owned());
            }
        }
        arguments_without_properties
    };

    let (cond, is_visible) = match condition {
        Some(c) => {
            let mut is_visible = true;
            if !c.eval(0, &arguments, doc)? {
                is_visible = false;
            }
            if !c.is_arg_constant() {
                (
                    Some(c.to_condition(0, all_locals, &arguments, doc.name)?),
                    is_visible,
                )
            } else {
                (None, is_visible)
            }
        }
        _ => (None, true),
    };

    Ok(ftd::Common {
        conditional_attribute: Default::default(),
        locals: Default::default(),
        condition: cond,
        is_not_visible: !is_visible,
        events: ftd::p2::Event::get_events(0, events, all_locals, properties, doc, root_name)?,
        reference,
        region: ftd::Region::from(
            ftd::p2::utils::string_optional("region", properties, doc.name, 0)?,
            doc.name,
        )?,
        padding: ftd::p2::utils::int_optional("padding", properties, doc.name, 0)?,
        padding_vertical: ftd::p2::utils::int_optional(
            "padding-vertical",
            properties,
            doc.name,
            0,
        )?,
        padding_horizontal: ftd::p2::utils::int_optional(
            "padding-horizontal",
            properties,
            doc.name,
            0,
        )?,
        padding_left: ftd::p2::utils::int_optional("padding-left", properties, doc.name, 0)?,
        padding_right: ftd::p2::utils::int_optional("padding-right", properties, doc.name, 0)?,
        padding_top: ftd::p2::utils::int_optional("padding-top", properties, doc.name, 0)?,
        padding_bottom: ftd::p2::utils::int_optional("padding-bottom", properties, doc.name, 0)?,
        border_top_radius: ftd::p2::utils::int_optional(
            "border-top-radius",
            properties,
            doc.name,
            0,
        )?,
        border_bottom_radius: ftd::p2::utils::int_optional(
            "border-bottom-radius",
            properties,
            doc.name,
            0,
        )?,
        border_left_radius: ftd::p2::utils::int_optional(
            "border-left-radius",
            properties,
            doc.name,
            0,
        )?,
        border_right_radius: ftd::p2::utils::int_optional(
            "border-right-radius",
            properties,
            doc.name,
            0,
        )?,
        width: ftd::Length::from(
            ftd::p2::utils::string_optional("width", properties, doc.name, 0)?,
            doc.name,
        )?,
        min_width: ftd::Length::from(
            ftd::p2::utils::string_optional("min-width", properties, doc.name, 0)?,
            doc.name,
        )?,
        max_width: ftd::Length::from(
            ftd::p2::utils::string_optional("max-width", properties, doc.name, 0)?,
            doc.name,
        )?,
        height: ftd::Length::from(
            ftd::p2::utils::string_optional("height", properties, doc.name, 0)?,
            doc.name,
        )?,
        min_height: ftd::Length::from(
            ftd::p2::utils::string_optional("min-height", properties, doc.name, 0)?,
            doc.name,
        )?,
        max_height: ftd::Length::from(
            ftd::p2::utils::string_optional("max-height", properties, doc.name, 0)?,
            doc.name,
        )?,
        color: color_from(
            ftd::p2::utils::string_optional("color", properties, doc.name, 0)?,
            doc.name,
        )?,
        background_color: color_from(
            ftd::p2::utils::string_optional("background-color", properties, doc.name, 0)?,
            doc.name,
        )?,
        border_color: color_from(
            ftd::p2::utils::string_optional("border-color", properties, doc.name, 0)?,
            doc.name,
        )?,
        border_width: ftd::p2::utils::int_with_default("border-width", 0, properties, doc.name, 0)?,
        border_radius: ftd::p2::utils::int_with_default(
            "border-radius",
            0,
            properties,
            doc.name,
            0,
        )?,
        data_id: ftd::p2::utils::string_optional("id", properties, doc.name, 0)?.map(|v| {
            if is_child {
                v
            } else {
                format!("{}#{}", doc.name, v)
            }
        }),
        id: None,
        overflow_x: ftd::Overflow::from(
            ftd::p2::utils::string_optional("overflow-x", properties, doc.name, 0)?,
            doc.name,
        )?,
        overflow_y: ftd::Overflow::from(
            ftd::p2::utils::string_optional("overflow-y", properties, doc.name, 0)?,
            doc.name,
        )?,
        border_top: ftd::p2::utils::int_optional("border-top", properties, doc.name, 0)?,
        border_left: ftd::p2::utils::int_optional("border-left", properties, doc.name, 0)?,
        border_right: ftd::p2::utils::int_optional("border-right", properties, doc.name, 0)?,
        border_bottom: ftd::p2::utils::int_optional("border-bottom", properties, doc.name, 0)?,
        margin_top: ftd::p2::utils::int_optional("margin-top", properties, doc.name, 0)?,
        margin_bottom: ftd::p2::utils::int_optional("margin-bottom", properties, doc.name, 0)?,
        margin_left: ftd::p2::utils::int_optional("margin-left", properties, doc.name, 0)?,
        margin_right: ftd::p2::utils::int_optional("margin-right", properties, doc.name, 0)?,
        link,
        open_in_new_tab: ftd::p2::utils::bool_with_default(
            "open-in-new-tab",
            false,
            properties,
            doc.name,
            0,
        )?,
        sticky: ftd::p2::utils::bool_with_default("sticky", false, properties, doc.name, 0)?,
        top: ftd::p2::utils::int_optional("top", properties, doc.name, 0)?,
        bottom: ftd::p2::utils::int_optional("bottom", properties, doc.name, 0)?,
        left: ftd::p2::utils::int_optional("left", properties, doc.name, 0)?,
        right: ftd::p2::utils::int_optional("right", properties, doc.name, 0)?,
        cursor: ftd::p2::utils::string_optional("cursor", properties, doc.name, 0)?,
        submit,
        shadow_offset_x: ftd::p2::utils::int_optional("shadow-offset-x", properties, doc.name, 0)?,
        shadow_offset_y: ftd::p2::utils::int_optional("shadow-offset-y", properties, doc.name, 0)?,
        shadow_size: ftd::p2::utils::int_optional("shadow-size", properties, doc.name, 0)?,
        shadow_blur: ftd::p2::utils::int_optional("shadow-blur", properties, doc.name, 0)?,
        shadow_color: color_from(
            ftd::p2::utils::string_optional("shadow-color", properties, doc.name, 0)?,
            doc.name,
        )?,
        gradient_direction: ftd::GradientDirection::from(
            ftd::p2::utils::string_optional("gradient-direction", properties, doc.name, 0)?,
            doc.name,
        )?,
        anchor,
        gradient_colors,
        background_image: ftd::p2::utils::string_optional(
            "background-image",
            properties,
            doc.name,
            0,
        )?,
        background_repeat: ftd::p2::utils::bool_with_default(
            "background-repeat",
            false,
            properties,
            doc.name,
            0,
        )?,
        background_parallax: ftd::p2::utils::bool_with_default(
            "background-parallax",
            false,
            properties,
            doc.name,
            0,
        )?,
        scale: ftd::p2::utils::decimal_optional("scale", properties, doc.name, 0)?,
        scale_x: ftd::p2::utils::decimal_optional("scale-x", properties, doc.name, 0)?,
        scale_y: ftd::p2::utils::decimal_optional("scale-y", properties, doc.name, 0)?,
        rotate: ftd::p2::utils::int_optional("rotate", properties, doc.name, 0)?,
        move_up: ftd::p2::utils::int_optional("move-up", properties, doc.name, 0)?,
        move_down: ftd::p2::utils::int_optional("move-down", properties, doc.name, 0)?,
        move_left: ftd::p2::utils::int_optional("move-left", properties, doc.name, 0)?,
        move_right: ftd::p2::utils::int_optional("move-right", properties, doc.name, 0)?,
        position: ftd::Position::from(
            match ftd::p2::utils::string_optional("position", properties, doc.name, 0)? {
                None => ftd::p2::utils::string_optional("align", properties, doc.name, 0)?,
                Some(v) => Some(v),
            },
            doc.name,
        )?,
        inner: ftd::p2::utils::bool_with_default("inner", inner_default, properties, doc.name, 0)?,
        z_index: ftd::p2::utils::int_optional("z-index", properties, doc.name, 0)?,
        slot: ftd::p2::utils::string_optional("slot", properties, doc.name, 0)?,
        grid_column: ftd::p2::utils::string_optional("grid-column", properties, doc.name, 0)?,
        grid_row: ftd::p2::utils::string_optional("grid-row", properties, doc.name, 0)?,
    })
}

fn common_arguments() -> Vec<(String, ftd::p2::Kind)> {
    vec![
        (
            "padding".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "padding-vertical".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "padding-horizontal".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "padding-left".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "padding-right".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "padding-top".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "padding-bottom".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "border-top-radius".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "border-bottom-radius".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "border-left-radius".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "border-right-radius".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        ("width".to_string(), ftd::p2::Kind::string().into_optional()),
        (
            "min-width".to_string(),
            ftd::p2::Kind::string().into_optional(),
        ),
        (
            "max-width".to_string(),
            ftd::p2::Kind::string().into_optional(),
        ),
        (
            "height".to_string(),
            ftd::p2::Kind::string().into_optional(),
        ),
        (
            "min-height".to_string(),
            ftd::p2::Kind::string().into_optional(),
        ),
        (
            "max-height".to_string(),
            ftd::p2::Kind::string().into_optional(),
        ),
        (
            // TODO: remove this after verifying that no existing document is using this
            "explain".to_string(),
            ftd::p2::Kind::boolean().into_optional(),
        ),
        (
            "region".to_string(),
            ftd::p2::Kind::string().into_optional(),
        ),
        ("color".to_string(), ftd::p2::Kind::string().into_optional()),
        (
            "background-color".to_string(),
            ftd::p2::Kind::string().into_optional(),
        ),
        (
            "border-color".to_string(),
            ftd::p2::Kind::string().into_optional(),
        ),
        (
            "border-width".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "border-radius".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        ("id".to_string(), ftd::p2::Kind::string().into_optional()),
        (
            "overflow-x".to_string(),
            ftd::p2::Kind::string().into_optional(),
        ),
        (
            "overflow-y".to_string(),
            ftd::p2::Kind::string().into_optional(),
        ),
        (
            "border-top".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "border-bottom".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "border-left".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "border-right".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "margin-top".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "margin-bottom".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "margin-left".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "margin-right".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        ("link".to_string(), ftd::p2::Kind::string().into_optional()),
        (
            "submit".to_string(),
            ftd::p2::Kind::string().into_optional(),
        ),
        (
            "open-in-new-tab".to_string(),
            ftd::p2::Kind::boolean().into_optional(),
        ),
        (
            "sticky".to_string(),
            ftd::p2::Kind::boolean().into_optional(),
        ),
        ("top".to_string(), ftd::p2::Kind::integer().into_optional()),
        (
            "bottom".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        ("left".to_string(), ftd::p2::Kind::integer().into_optional()),
        (
            "right".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "cursor".to_string(),
            ftd::p2::Kind::string().into_optional(),
        ),
        (
            "anchor".to_string(),
            ftd::p2::Kind::string().into_optional(),
        ),
        (
            "gradient-direction".to_string(),
            ftd::p2::Kind::string().into_optional(),
        ),
        (
            "gradient-colors".to_string(),
            ftd::p2::Kind::string().into_optional(),
        ),
        (
            "shadow-offset-x".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "shadow-offset-y".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "shadow-blur".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "shadow-size".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "shadow-color".to_string(),
            ftd::p2::Kind::string().into_optional(),
        ),
        (
            "background-image".to_string(),
            ftd::p2::Kind::string().into_optional(),
        ),
        (
            "background-repeat".to_string(),
            ftd::p2::Kind::boolean().into_optional(),
        ),
        (
            "background-parallax".to_string(),
            ftd::p2::Kind::boolean().into_optional(),
        ),
        (
            "scale".to_string(),
            ftd::p2::Kind::decimal().into_optional(),
        ),
        (
            "scale-x".to_string(),
            ftd::p2::Kind::decimal().into_optional(),
        ),
        (
            "scale-y".to_string(),
            ftd::p2::Kind::decimal().into_optional(),
        ),
        (
            "rotate".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "move-up".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "move-down".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "move-left".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "move-right".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        (
            "position".to_string(),
            ftd::p2::Kind::string().into_optional(),
        ),
        (
            "inner".to_string(),
            ftd::p2::Kind::boolean().into_optional(),
        ),
        (
            "z-index".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        ),
        ("slot".to_string(), ftd::p2::Kind::string().into_optional()),
        /*(
            "grid-column".to_string(),
            ftd::p2::Kind::string().into_optional(),
        ),
        (
            "grid-row".to_string(),
            ftd::p2::Kind::string().into_optional(),
        ),*/
    ]
}

pub fn null() -> ftd::Component {
    ftd::Component {
        kernel: true,
        full_name: "ftd#null".to_string(),
        root: "ftd.kernel".to_string(),
        ..Default::default()
    }
}

pub fn container_from_properties(
    properties: &std::collections::BTreeMap<String, ftd::Value>,
    doc: &ftd::p2::TDoc,
) -> ftd::p1::Result<ftd::Container> {
    Ok(ftd::Container {
        children: Default::default(),
        external_children: Default::default(),
        open: ftd::p2::utils::string_bool_optional("open", properties, doc.name, 0)?,
        wrap: ftd::p2::utils::bool_with_default("wrap", false, properties, doc.name, 0)?,
    })
}

fn container_arguments() -> Vec<(String, ftd::p2::Kind)> {
    vec![
        ("open".to_string(), ftd::p2::Kind::string().into_optional()),
        ("align".to_string(), ftd::p2::Kind::string().into_optional()),
        ("wrap".to_string(), ftd::p2::Kind::boolean().into_optional()),
    ]
}

pub fn image_function() -> ftd::Component {
    ftd::Component {
        kernel: true,
        full_name: "ftd#image".to_string(),
        root: "ftd.kernel".to_string(),
        arguments: [
            vec![
                ("src".to_string(), ftd::p2::Kind::string()),
                (
                    "description".to_string(),
                    ftd::p2::Kind::string().into_optional(),
                ),
                ("align".to_string(), ftd::p2::Kind::string().into_optional()),
                ("crop".to_string(), ftd::p2::Kind::boolean().into_optional()),
            ],
            common_arguments(),
        ]
        .concat()
        .into_iter()
        .collect(),
        locals: Default::default(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
        condition: None,
        events: vec![],
        line_number: 0,
    }
}

pub fn image_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (ftd::Value, Option<String>)>,
    doc: &ftd::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd::Map,
    root_name: Option<&str>,
) -> ftd::p1::Result<ftd::Image> {
    let (src, reference) =
        ftd::p2::utils::string_and_ref(0, "src", properties_with_ref, all_locals, doc.name)?;
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    Ok(ftd::Image {
        src,
        description: ftd::p2::utils::string_optional("description", properties, doc.name, 0)?
            .unwrap_or_else(|| "".to_string()),
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, reference,
        )?,
        crop: ftd::p2::utils::bool_with_default("crop", false, properties, doc.name, 0)?,
    })
}

pub fn row_function() -> ftd::Component {
    ftd::Component {
        kernel: true,
        full_name: "ftd#row".to_string(),
        root: "ftd.kernel".to_string(),
        arguments: [
            container_arguments(),
            common_arguments(),
            vec![(
                "spacing".to_string(),
                ftd::p2::Kind::string().into_optional(),
            )],
        ]
        .concat()
        .into_iter()
        .collect(),
        locals: Default::default(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
        condition: None,
        events: vec![],
        line_number: 0,
    }
}

pub fn row_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (ftd::Value, Option<String>)>,
    doc: &ftd::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd::Map,
    root_name: Option<&str>,
) -> ftd::p1::Result<ftd::Row> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    Ok(ftd::Row {
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, None,
        )?,
        container: container_from_properties(properties, doc)?,
        spacing: ftd::Spacing::from(ftd::p2::utils::string_optional(
            "spacing", properties, doc.name, 0,
        )?)?,
    })
}

pub fn column_function() -> ftd::Component {
    ftd::Component {
        line_number: 0,
        kernel: true,
        full_name: "ftd#column".to_string(),
        root: "ftd.kernel".to_string(),
        arguments: [
            container_arguments(),
            common_arguments(),
            vec![(
                "spacing".to_string(),
                ftd::p2::Kind::string().into_optional(),
            )],
        ]
        .concat()
        .into_iter()
        .collect(),
        locals: Default::default(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
        condition: None,
        events: vec![],
    }
}

pub fn column_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (ftd::Value, Option<String>)>,
    doc: &ftd::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd::Map,
    root_name: Option<&str>,
) -> ftd::p1::Result<ftd::Column> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    Ok(ftd::Column {
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, None,
        )?,
        container: container_from_properties(properties, doc)?,
        spacing: ftd::Spacing::from(ftd::p2::utils::string_optional(
            "spacing", properties, doc.name, 0,
        )?)?,
    })
}

pub fn external_font_from_properties(
    properties: &std::collections::BTreeMap<String, ftd::Value>,
    doc: &ftd::p2::TDoc,
) -> ftd::p1::Result<Option<ftd::ExternalFont>> {
    let font_option = ftd::p2::utils::string_optional("font", properties, doc.name, 0)?;
    let font_url_option = ftd::p2::utils::string_optional("font-url", properties, doc.name, 0)?;

    match (font_option, font_url_option) {
        (Some(font), Some(font_url)) => {
            let name_opt = font.split(',').next();
            let name = match name_opt {
                Some(f) => f.to_string(),
                None => {
                    return ftd::e2(
                        "Something went wrong while parsing font vector",
                        doc.name,
                        0,
                    );
                }
            };

            Ok(Some(ftd::ExternalFont {
                url: font_url,
                display: ftd::FontDisplay::from(
                    ftd::p2::utils::string_optional("font-display", properties, doc.name, 0)?,
                    doc.name,
                )?,
                name,
            }))
        }
        _ => Ok(None),
    }
}

#[allow(dead_code)]
#[allow(unused_variables)]
pub fn text_render(
    tf: &ftd::TextFormat,
    text: String,
    source: ftd::TextSource,
    theme: String,
    doc_id: &str,
) -> ftd::p1::Result<ftd::Rendered> {
    Ok(match (source, tf) {
        (ftd::TextSource::Body, ftd::TextFormat::Markdown) => ftd::markdown(text.as_str()),
        (_, ftd::TextFormat::Markdown) => ftd::markdown_line(text.as_str()),
        (_, ftd::TextFormat::Code { lang }) => {
            ftd::code_with_theme(text.as_str(), lang.as_str(), theme.as_str(), doc_id)?
        }
        (_, ftd::TextFormat::Text) => ftd::Rendered {
            original: text.clone(),
            rendered: text,
        },
    })
}

pub fn iframe_function() -> ftd::Component {
    ftd::Component {
        line_number: 0,
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#iframe".to_string(),
        arguments: [
            vec![
                ("src".to_string(), ftd::p2::Kind::string().into_optional()),
                (
                    "youtube".to_string(),
                    ftd::p2::Kind::string().into_optional(),
                ),
            ],
            common_arguments(),
        ]
        .concat()
        .into_iter()
        .collect(),
        locals: Default::default(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
        condition: None,
        events: vec![],
    }
}

pub fn iframe_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (ftd::Value, Option<String>)>,
    doc: &ftd::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd::Map,
    root_name: Option<&str>,
) -> ftd::p1::Result<ftd::IFrame> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    let src = match (
        ftd::p2::utils::string_optional("src", properties, doc.name, 0)?,
        ftd::p2::utils::string_optional("youtube", properties, doc.name, 0)?
            .and_then(|id| ftd::youtube_id::from_raw(id.as_str())),
    ) {
        (Some(src), None) => src,
        (None, Some(id)) => id,
        (Some(_), Some(_)) => return ftd::e2("both src and youtube id provided", doc.name, 0),
        (None, None) => return ftd::e2("src or youtube id is required", doc.name, 0),
    };

    Ok(ftd::IFrame {
        src,
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, None,
        )?,
    })
}

pub fn text_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (ftd::Value, Option<String>)>,
    doc: &ftd::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd::Map,
    root_name: Option<&str>,
) -> ftd::p1::Result<ftd::Text> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);

    let (text, source, reference) = ftd::p2::utils::string_and_source_and_ref(
        0,
        "text",
        properties_with_ref,
        all_locals,
        doc.name,
        condition,
    )?;
    let font_str = ftd::p2::utils::string_optional("font", properties, doc.name, 0)?;

    let font: Vec<ftd::NamedFont> = match font_str {
        Some(f) => f
            .split(',')
            .flat_map(|x| ftd::NamedFont::from(Some(x.to_string())))
            .collect(),
        None => vec![],
    };
    Ok(ftd::Text {
        line: source != ftd::TextSource::Body,
        text: if source == ftd::TextSource::Body {
            ftd::markdown(text.as_str())
        } else {
            ftd::markdown_line(text.as_str())
        },
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, reference,
        )?,
        text_align: ftd::TextAlign::from(
            ftd::p2::utils::string_optional("text-align", properties, doc.name, 0)?,
            doc.name,
        )?,
        style: ftd::Style::from(
            ftd::p2::utils::string_optional("style", properties, doc.name, 0)?,
            doc.name,
        )?,
        size: ftd::p2::utils::int_optional("size", properties, doc.name, 0)?,
        external_font: external_font_from_properties(properties, doc)?,
        font,
        line_height: ftd::p2::utils::int_optional("line-height", properties, doc.name, 0)?,
        line_clamp: ftd::p2::utils::int_optional("line-clamp", properties, doc.name, 0)?,
    })
}

pub fn text_block_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (ftd::Value, Option<String>)>,
    doc: &ftd::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd::Map,
    root_name: Option<&str>,
) -> ftd::p1::Result<ftd::TextBlock> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);

    let (text, source, reference) = ftd::p2::utils::string_and_source_and_ref(
        0,
        "text",
        properties_with_ref,
        all_locals,
        doc.name,
        condition,
    )?;
    let font_str = ftd::p2::utils::string_optional("font", properties, doc.name, 0)?;

    let font: Vec<ftd::NamedFont> = match font_str {
        Some(f) => f
            .split(',')
            .flat_map(|x| ftd::NamedFont::from(Some(x.to_string())))
            .collect(),
        None => vec![],
    };
    Ok(ftd::TextBlock {
        line: source != ftd::TextSource::Body,
        text: if source == ftd::TextSource::Body {
            ftd::markdown(text.as_str())
        } else {
            ftd::markdown_line(text.as_str())
        },
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, reference,
        )?,
        text_align: ftd::TextAlign::from(
            ftd::p2::utils::string_optional("text-align", properties, doc.name, 0)?,
            doc.name,
        )?,
        style: ftd::Style::from(
            ftd::p2::utils::string_optional("style", properties, doc.name, 0)?,
            doc.name,
        )?,
        size: ftd::p2::utils::int_optional("size", properties, doc.name, 0)?,
        external_font: external_font_from_properties(properties, doc)?,
        font,
        line_height: ftd::p2::utils::int_optional("line-height", properties, doc.name, 0)?,
        line_clamp: ftd::p2::utils::int_optional("line-clamp", properties, doc.name, 0)?,
    })
}

pub fn code_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (ftd::Value, Option<String>)>,
    doc: &ftd::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd::Map,
    root_name: Option<&str>,
) -> ftd::p1::Result<ftd::Code> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);

    let (text, _, reference) = ftd::p2::utils::string_and_source_and_ref(
        0,
        "text",
        properties_with_ref,
        all_locals,
        doc.name,
        condition,
    )?;
    let font_str = ftd::p2::utils::string_optional("font", properties, doc.name, 0)?;

    let font: Vec<ftd::NamedFont> = match font_str {
        Some(f) => f
            .split(',')
            .flat_map(|x| ftd::NamedFont::from(Some(x.to_string())))
            .collect(),
        None => vec![],
    };

    Ok(ftd::Code {
        text: ftd::code_with_theme(
            text.as_str(),
            ftd::p2::utils::string_optional("lang", properties, doc.name, 0)?
                .unwrap_or_else(|| "txt".to_string())
                .as_str(),
            ftd::p2::utils::string_with_default(
                "theme",
                ftd::render::DEFAULT_THEME,
                properties,
                doc.name,
                0,
            )?
            .as_str(),
            doc.name,
        )?,
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, reference,
        )?,
        text_align: ftd::TextAlign::from(
            ftd::p2::utils::string_optional("text-align", properties, doc.name, 0)?,
            doc.name,
        )?,
        style: ftd::Style::from(
            ftd::p2::utils::string_optional("style", properties, doc.name, 0)?,
            doc.name,
        )?,
        size: ftd::p2::utils::int_optional("size", properties, doc.name, 0)?,
        external_font: external_font_from_properties(properties, doc)?,
        font,
        line_height: ftd::p2::utils::int_optional("line-height", properties, doc.name, 0)?,
        line_clamp: ftd::p2::utils::int_optional("line-clamp", properties, doc.name, 0)?,
    })
}

pub fn integer_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (ftd::Value, Option<String>)>,
    doc: &ftd::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd::Map,
    root_name: Option<&str>,
) -> ftd::p1::Result<ftd::Text> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    let font_str = ftd::p2::utils::string_optional("font", properties, doc.name, 0)?;
    let num = format_num::NumberFormat::new();
    let text = match ftd::p2::utils::string_optional("format", properties, doc.name, 0)? {
        Some(f) => num.format(
            f.as_str(),
            ftd::p2::utils::int("value", properties, doc.name, 0)? as f64,
        ),
        None => ftd::p2::utils::int("value", properties, doc.name, 0)?.to_string(),
    };
    let reference = ftd::p2::utils::complete_reference(
        &properties_with_ref.get("value").expect("").1,
        all_locals,
    );

    let font: Vec<ftd::NamedFont> = match font_str {
        Some(f) => f
            .split(',')
            .flat_map(|x| ftd::NamedFont::from(Some(x.to_string())))
            .collect(),
        None => vec![],
    };

    Ok(ftd::Text {
        text: ftd::markdown_line(text.as_str()),
        line: false,
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, reference,
        )?,
        text_align: ftd::TextAlign::from(
            ftd::p2::utils::string_optional("text-align", properties, doc.name, 0)?,
            doc.name,
        )?,
        style: ftd::Style::from(
            ftd::p2::utils::string_optional("style", properties, doc.name, 0)?,
            doc.name,
        )?,
        size: ftd::p2::utils::int_optional("size", properties, doc.name, 0)?,
        external_font: external_font_from_properties(properties, doc)?,
        font,
        line_height: ftd::p2::utils::int_optional("line-height", properties, doc.name, 0)?,
        line_clamp: ftd::p2::utils::int_optional("line-clamp", properties, doc.name, 0)?,
    })
}

pub fn decimal_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (ftd::Value, Option<String>)>,
    doc: &ftd::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd::Map,
    root_name: Option<&str>,
) -> ftd::p1::Result<ftd::Text> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    let font_str = ftd::p2::utils::string_optional("font", properties, doc.name, 0)?;
    let num = format_num::NumberFormat::new();
    let text = match ftd::p2::utils::string_optional("format", properties, doc.name, 0)? {
        Some(f) => num.format(
            f.as_str(),
            ftd::p2::utils::decimal("value", properties, doc.name, 0)?,
        ),
        None => ftd::p2::utils::decimal("value", properties, doc.name, 0)?.to_string(),
    };

    let reference = ftd::p2::utils::complete_reference(
        &properties_with_ref.get("value").expect("").1,
        all_locals,
    );

    let font: Vec<ftd::NamedFont> = match font_str {
        Some(f) => f
            .split(',')
            .flat_map(|x| ftd::NamedFont::from(Some(x.to_string())))
            .collect(),
        None => vec![],
    };
    Ok(ftd::Text {
        text: ftd::markdown_line(text.as_str()),
        line: false,
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, reference,
        )?,
        text_align: ftd::TextAlign::from(
            ftd::p2::utils::string_optional("text-align", properties, doc.name, 0)?,
            doc.name,
        )?,
        style: ftd::Style::from(
            ftd::p2::utils::string_optional("style", properties, doc.name, 0)?,
            doc.name,
        )?,
        size: ftd::p2::utils::int_optional("size", properties, doc.name, 0)?,
        external_font: external_font_from_properties(properties, doc)?,
        font,
        line_height: ftd::p2::utils::int_optional("line-height", properties, doc.name, 0)?,
        line_clamp: ftd::p2::utils::int_optional("line-clamp", properties, doc.name, 0)?,
    })
}

pub fn color_from(l: Option<String>, doc_id: &str) -> ftd::p1::Result<Option<ftd::Color>> {
    use std::str::FromStr;

    let v = match l {
        Some(v) => v,
        None => return Ok(None),
    };

    match css_color_parser::Color::from_str(v.as_str()) {
        Ok(v) => Ok(Some(ftd::Color {
            r: v.r,
            g: v.g,
            b: v.b,
            alpha: v.a,
        })),
        Err(e) => return ftd::e2(format!("{} is not a valid color: {:?}", v, e), doc_id, 0),
    }
}

pub fn boolean_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (ftd::Value, Option<String>)>,
    doc: &ftd::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd::Map,
    root_name: Option<&str>,
) -> ftd::p1::Result<ftd::Text> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    let font_str = ftd::p2::utils::string_optional("font", properties, doc.name, 0)?;
    let value = ftd::p2::utils::bool("value", properties, doc.name, 0)?;
    let text = if value {
        ftd::p2::utils::string_with_default("true", "true", properties, doc.name, 0)?
    } else {
        ftd::p2::utils::string_with_default("false", "false", properties, doc.name, 0)?
    };

    let reference = ftd::p2::utils::complete_reference(
        &properties_with_ref.get("value").expect("").1,
        all_locals,
    );

    let font: Vec<ftd::NamedFont> = match font_str {
        Some(f) => f
            .split(',')
            .flat_map(|x| ftd::NamedFont::from(Some(x.to_string())))
            .collect(),
        None => vec![],
    };

    Ok(ftd::Text {
        text: ftd::markdown_line(text.as_str()),
        line: false,
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, reference,
        )?,
        text_align: ftd::TextAlign::from(
            ftd::p2::utils::string_optional("text-align", properties, doc.name, 0)?,
            doc.name,
        )?,
        style: ftd::Style::from(
            ftd::p2::utils::string_optional("style", properties, doc.name, 0)?,
            doc.name,
        )?,
        size: ftd::p2::utils::int_optional("size", properties, doc.name, 0)?,
        external_font: external_font_from_properties(properties, doc)?,
        font,
        line_height: ftd::p2::utils::int_optional("line-height", properties, doc.name, 0)?,
        line_clamp: ftd::p2::utils::int_optional("line-clamp", properties, doc.name, 0)?,
    })
}

pub fn text_function(is_text_block: bool) -> ftd::Component {
    let full_name = if is_text_block {
        "ftd#text-block"
    } else {
        "ftd#text"
    };

    ftd::Component {
        line_number: 0,
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: full_name.to_string(),
        arguments: [
            vec![
                ("text".to_string(), ftd::p2::Kind::caption_or_body()),
                ("align".to_string(), ftd::p2::Kind::string().into_optional()),
                ("style".to_string(), ftd::p2::Kind::string().into_optional()),
                ("size".to_string(), ftd::p2::Kind::integer().into_optional()),
                (
                    "font-url".to_string(),
                    ftd::p2::Kind::string().into_optional(),
                ),
                ("font".to_string(), ftd::p2::Kind::string().into_optional()),
                (
                    "font-display".to_string(),
                    ftd::p2::Kind::string().into_optional(),
                ),
                (
                    "line-height".to_string(),
                    ftd::p2::Kind::integer().into_optional(),
                ),
                (
                    "line-clamp".to_string(),
                    ftd::p2::Kind::integer().into_optional(),
                ),
                (
                    "text-align".to_string(),
                    ftd::p2::Kind::string().into_optional(),
                ),
            ],
            common_arguments(),
        ]
        .concat()
        .into_iter()
        .collect(),
        locals: Default::default(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
        condition: None,
        events: vec![],
    }
}

pub fn code_function() -> ftd::Component {
    ftd::Component {
        line_number: 0,
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#code".to_string(),
        arguments: [
            vec![
                ("text".to_string(), ftd::p2::Kind::caption_or_body()),
                ("align".to_string(), ftd::p2::Kind::string().into_optional()),
                ("style".to_string(), ftd::p2::Kind::string().into_optional()),
                ("lang".to_string(), ftd::p2::Kind::string().into_optional()),
                ("theme".to_string(), ftd::p2::Kind::string().into_optional()),
                ("size".to_string(), ftd::p2::Kind::integer().into_optional()),
                (
                    "font-url".to_string(),
                    ftd::p2::Kind::string().into_optional(),
                ),
                ("font".to_string(), ftd::p2::Kind::string().into_optional()),
                (
                    "font-display".to_string(),
                    ftd::p2::Kind::string().into_optional(),
                ),
                (
                    "line-height".to_string(),
                    ftd::p2::Kind::integer().into_optional(),
                ),
                (
                    "line-clamp".to_string(),
                    ftd::p2::Kind::integer().into_optional(),
                ),
                (
                    "text-align".to_string(),
                    ftd::p2::Kind::string().into_optional(),
                ),
            ],
            common_arguments(),
        ]
        .concat()
        .into_iter()
        .collect(),
        locals: Default::default(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
        condition: None,
        events: vec![],
    }
}

pub fn integer_function() -> ftd::Component {
    ftd::Component {
        line_number: 0,
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#integer".to_string(),
        arguments: [
            vec![
                ("value".to_string(), ftd::p2::Kind::integer()),
                ("align".to_string(), ftd::p2::Kind::string().into_optional()),
                ("style".to_string(), ftd::p2::Kind::string().into_optional()),
                (
                    "format".to_string(),
                    ftd::p2::Kind::string().into_optional(),
                ),
                ("size".to_string(), ftd::p2::Kind::integer().into_optional()),
                (
                    "font-url".to_string(),
                    ftd::p2::Kind::string().into_optional(),
                ),
                ("font".to_string(), ftd::p2::Kind::string().into_optional()),
                (
                    "font-display".to_string(),
                    ftd::p2::Kind::string().into_optional(),
                ),
                (
                    "line-height".to_string(),
                    ftd::p2::Kind::string().into_optional(),
                ),
            ],
            common_arguments(),
        ]
        .concat()
        .into_iter()
        .collect(),
        locals: Default::default(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
        condition: None,
        events: vec![],
    }
}

pub fn decimal_function() -> ftd::Component {
    ftd::Component {
        line_number: 0,
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#decimal".to_string(),
        arguments: [
            vec![
                ("value".to_string(), ftd::p2::Kind::decimal()),
                ("align".to_string(), ftd::p2::Kind::string().into_optional()),
                ("style".to_string(), ftd::p2::Kind::string().into_optional()),
                (
                    "format".to_string(),
                    ftd::p2::Kind::string().into_optional(),
                ),
                ("size".to_string(), ftd::p2::Kind::integer().into_optional()),
                (
                    "font-url".to_string(),
                    ftd::p2::Kind::string().into_optional(),
                ),
                ("font".to_string(), ftd::p2::Kind::string().into_optional()),
                (
                    "font-display".to_string(),
                    ftd::p2::Kind::string().into_optional(),
                ),
                (
                    "line-height".to_string(),
                    ftd::p2::Kind::string().into_optional(),
                ),
            ],
            common_arguments(),
        ]
        .concat()
        .into_iter()
        .collect(),
        locals: Default::default(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
        condition: None,
        events: vec![],
    }
}

pub fn scene_function() -> ftd::Component {
    let arguments = {
        let mut arguments: std::collections::BTreeMap<String, ftd::p2::Kind> = [
            container_arguments(),
            common_arguments(),
            vec![(
                "spacing".to_string(),
                ftd::p2::Kind::string().into_optional(),
            )],
        ]
        .concat()
        .into_iter()
        .collect();
        arguments.remove("spacing");
        arguments.remove("wrap");
        arguments
    };

    ftd::Component {
        line_number: 0,
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#scene".to_string(),
        arguments,
        locals: Default::default(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
        condition: None,
        events: vec![],
    }
}

pub fn grid_function() -> ftd::Component {
    let arguments: std::collections::BTreeMap<String, ftd::p2::Kind> = [
        container_arguments(),
        common_arguments(),
        vec![
            ("slots".to_string(), ftd::p2::Kind::string()),
            (
                "slot-widths".to_string(),
                ftd::p2::Kind::string().into_optional(),
            ),
            (
                "slot-heights".to_string(),
                ftd::p2::Kind::string().into_optional(),
            ),
            (
                "spacing".to_string(),
                ftd::p2::Kind::integer().into_optional(),
            ),
            (
                "inline".to_string(),
                ftd::p2::Kind::boolean().into_optional(),
            ),
        ],
    ]
    .concat()
    .into_iter()
    .collect();

    ftd::Component {
        line_number: 0,
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#grid".to_string(),
        arguments,
        locals: Default::default(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
        condition: None,
        events: vec![],
    }
}

pub fn boolean_function() -> ftd::Component {
    ftd::Component {
        line_number: 0,
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#boolean".to_string(),
        arguments: [
            vec![
                ("value".to_string(), ftd::p2::Kind::boolean()),
                ("align".to_string(), ftd::p2::Kind::string().into_optional()),
                ("style".to_string(), ftd::p2::Kind::string().into_optional()),
                (
                    "format".to_string(),
                    ftd::p2::Kind::string().into_optional(),
                ),
                ("size".to_string(), ftd::p2::Kind::integer().into_optional()),
                (
                    "font-url".to_string(),
                    ftd::p2::Kind::string().into_optional(),
                ),
                ("font".to_string(), ftd::p2::Kind::string().into_optional()),
                (
                    "font-display".to_string(),
                    ftd::p2::Kind::string().into_optional(),
                ),
                (
                    "line-height".to_string(),
                    ftd::p2::Kind::string().into_optional(),
                ),
                ("true".to_string(), ftd::p2::Kind::string().into_optional()),
                ("false".to_string(), ftd::p2::Kind::string().into_optional()),
            ],
            common_arguments(),
        ]
        .concat()
        .into_iter()
        .collect(),
        locals: Default::default(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
        condition: None,
        events: vec![],
    }
}

pub fn input_function() -> ftd::Component {
    ftd::Component {
        line_number: 0,
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#input".to_string(),
        arguments: [
            vec![(
                "placeholder".to_string(),
                ftd::p2::Kind::string().into_optional(),
            )],
            common_arguments(),
        ]
        .concat()
        .into_iter()
        .collect(),
        locals: Default::default(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
        condition: None,
        events: vec![],
    }
}

pub fn input_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (ftd::Value, Option<String>)>,
    doc: &ftd::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd::Map,
    root_name: Option<&str>,
) -> ftd::p1::Result<ftd::Input> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    Ok(ftd::Input {
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, None,
        )?,
        placeholder: ftd::p2::utils::string_optional("placeholder", properties, doc.name, 0)?,
    })
}

pub fn scene_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (ftd::Value, Option<String>)>,
    doc: &ftd::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd::Map,
    root_name: Option<&str>,
) -> ftd::p1::Result<ftd::Scene> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    Ok(ftd::Scene {
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, None,
        )?,
        container: container_from_properties(properties, doc)?,
        spacing: ftd::Spacing::from(ftd::p2::utils::string_optional(
            "spacing", properties, doc.name, 0,
        )?)?,
    })
}

pub fn grid_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (ftd::Value, Option<String>)>,
    doc: &ftd::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd::Map,
    root_name: Option<&str>,
) -> ftd::p1::Result<ftd::Grid> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    Ok(ftd::Grid {
        slots: match ftd::p2::utils::string_optional("slots", properties, doc.name, 0)? {
            Some(val) => val,
            None => return ftd::e2("expected slots", doc.name, 0),
        },
        slot_widths: ftd::p2::utils::string_optional("slot-widths", properties, doc.name, 0)?,
        slot_heights: ftd::p2::utils::string_optional("slot-heights", properties, doc.name, 0)?,
        spacing: ftd::p2::utils::int_optional("spacing", properties, doc.name, 0)?,
        spacing_vertical: ftd::p2::utils::int_optional(
            "spacing-vertical",
            properties,
            doc.name,
            0,
        )?,
        spacing_horizontal: ftd::p2::utils::int_optional(
            "spacing-horizontal",
            properties,
            doc.name,
            0,
        )?,
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, None,
        )?,
        container: container_from_properties(properties, doc)?,
        inline: ftd::p2::utils::bool_with_default("inline", false, properties, doc.name, 0)?,
        auto_flow: ftd::p2::utils::string_optional("auto-flow", properties, doc.name, 0)?,
    })
}
