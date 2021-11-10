pub fn common_from_properties(
    properties: &std::collections::BTreeMap<String, crate::Value>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd_rt::Map,
    root_name: Option<&str>,
    reference: Option<String>,
) -> crate::p1::Result<ftd_rt::Common> {
    let submit = crate::p2::utils::string_optional("submit", properties)?;
    let link = crate::p2::utils::string_optional("link", properties)?;
    if let (Some(_), Some(_)) = (&submit, &link) {
        return crate::e2(
            "Cannot have both submit and link together",
            "common_from_properties",
        );
    }
    let gradient_color_str = crate::p2::utils::string_optional("gradient-colors", properties)?;

    let gradient_colors: Vec<ftd_rt::Color> = match gradient_color_str {
        Some(f) => f
            .split(',')
            .flat_map(|x| color_from(Some(x.to_string())).ok()?)
            .collect(),
        None => vec![],
    };
    let anchor = ftd_rt::Anchor::from(crate::p2::utils::string_optional("anchor", properties)?)?;

    let inner_default = match anchor {
        Some(ref p) => match p {
            ftd_rt::Anchor::Parent => false,
            ftd_rt::Anchor::Window => true,
        },
        None => false,
    };

    Ok(ftd_rt::Common {
        conditional_attribute: Default::default(),
        locals: Default::default(),
        condition: match condition {
            Some(c) if !c.is_constant() => Some(c.to_condition(all_locals, &Default::default())?),
            _ => None,
        },
        is_not_visible: false,
        events: ftd::p2::Event::get_events(events, all_locals, properties, doc, root_name, false)?,
        reference,
        region: ftd_rt::Region::from(crate::p2::utils::string_optional("region", properties)?)?,
        padding: crate::p2::utils::int_optional("padding", properties)?,
        padding_vertical: crate::p2::utils::int_optional("padding-vertical", properties)?,
        padding_horizontal: crate::p2::utils::int_optional("padding-horizontal", properties)?,
        padding_left: crate::p2::utils::int_optional("padding-left", properties)?,
        padding_right: crate::p2::utils::int_optional("padding-right", properties)?,
        padding_top: crate::p2::utils::int_optional("padding-top", properties)?,
        padding_bottom: crate::p2::utils::int_optional("padding-bottom", properties)?,
        border_top_radius: crate::p2::utils::int_optional("border-top-radius", properties)?,
        border_bottom_radius: crate::p2::utils::int_optional("border-bottom-radius", properties)?,
        border_left_radius: crate::p2::utils::int_optional("border-left-radius", properties)?,
        border_right_radius: crate::p2::utils::int_optional("border-right-radius", properties)?,
        width: ftd_rt::Length::from(crate::p2::utils::string_optional("width", properties)?)?,
        min_width: ftd_rt::Length::from(crate::p2::utils::string_optional(
            "min-width",
            properties,
        )?)?,
        max_width: ftd_rt::Length::from(crate::p2::utils::string_optional(
            "max-width",
            properties,
        )?)?,
        height: ftd_rt::Length::from(crate::p2::utils::string_optional("height", properties)?)?,
        min_height: ftd_rt::Length::from(crate::p2::utils::string_optional(
            "min-height",
            properties,
        )?)?,
        max_height: ftd_rt::Length::from(crate::p2::utils::string_optional(
            "max-height",
            properties,
        )?)?,
        color: color_from(crate::p2::utils::string_optional("color", properties)?)?,
        background_color: color_from(crate::p2::utils::string_optional(
            "background-color",
            properties,
        )?)?,
        border_color: color_from(crate::p2::utils::string_optional(
            "border-color",
            properties,
        )?)?,
        border_width: crate::p2::utils::int_with_default("border-width", 0, properties)?,
        border_radius: crate::p2::utils::int_with_default("border-radius", 0, properties)?,
        data_id: crate::p2::utils::string_optional("id", properties)?.map(|v| {
            if is_child {
                v
            } else {
                format!("{}#{}", doc.name, v)
            }
        }),
        id: None,
        overflow_x: ftd_rt::Overflow::from(crate::p2::utils::string_optional(
            "overflow-x",
            properties,
        )?)?,
        overflow_y: ftd_rt::Overflow::from(crate::p2::utils::string_optional(
            "overflow-y",
            properties,
        )?)?,
        border_top: crate::p2::utils::int_optional("border-top", properties)?,
        border_left: crate::p2::utils::int_optional("border-left", properties)?,
        border_right: crate::p2::utils::int_optional("border-right", properties)?,
        border_bottom: crate::p2::utils::int_optional("border-bottom", properties)?,
        margin_top: crate::p2::utils::int_optional("margin-top", properties)?,
        margin_bottom: crate::p2::utils::int_optional("margin-bottom", properties)?,
        margin_left: crate::p2::utils::int_optional("margin-left", properties)?,
        margin_right: crate::p2::utils::int_optional("margin-right", properties)?,
        link,
        open_in_new_tab: crate::p2::utils::bool_with_default("open-in-new-tab", false, properties)?,
        sticky: crate::p2::utils::bool_with_default("sticky", false, properties)?,
        top: crate::p2::utils::int_optional("top", properties)?,
        bottom: crate::p2::utils::int_optional("bottom", properties)?,
        left: crate::p2::utils::int_optional("left", properties)?,
        right: crate::p2::utils::int_optional("right", properties)?,
        cursor: crate::p2::utils::string_optional("cursor", properties)?,
        submit,
        shadow_offset_x: crate::p2::utils::int_optional("shadow-offset-x", properties)?,
        shadow_offset_y: crate::p2::utils::int_optional("shadow-offset-y", properties)?,
        shadow_size: crate::p2::utils::int_optional("shadow-size", properties)?,
        shadow_blur: crate::p2::utils::int_optional("shadow-blur", properties)?,
        shadow_color: color_from(crate::p2::utils::string_optional(
            "shadow-color",
            properties,
        )?)?,
        gradient_direction: ftd_rt::GradientDirection::from(crate::p2::utils::string_optional(
            "gradient-direction",
            properties,
        )?)?,
        anchor,
        gradient_colors,
        background_image: crate::p2::utils::string_optional("background-image", properties)?,
        background_repeat: crate::p2::utils::bool_with_default(
            "background-repeat",
            false,
            properties,
        )?,
        background_parallax: crate::p2::utils::bool_with_default(
            "background-parallax",
            false,
            properties,
        )?,
        scale: crate::p2::utils::decimal_optional("scale", properties)?,
        scale_x: crate::p2::utils::decimal_optional("scale-x", properties)?,
        scale_y: crate::p2::utils::decimal_optional("scale-y", properties)?,
        rotate: crate::p2::utils::int_optional("rotate", properties)?,
        move_up: crate::p2::utils::int_optional("move-up", properties)?,
        move_down: crate::p2::utils::int_optional("move-down", properties)?,
        move_left: crate::p2::utils::int_optional("move-left", properties)?,
        move_right: crate::p2::utils::int_optional("move-right", properties)?,
        position: ftd_rt::Position::from(
            match crate::p2::utils::string_optional("position", properties)? {
                None => crate::p2::utils::string_optional("align", properties)?,
                Some(v) => Some(v),
            },
        )?,
        inner: crate::p2::utils::bool_with_default("inner", inner_default, properties)?,
    })
}

fn common_arguments() -> Vec<(String, crate::p2::Kind)> {
    vec![
        (
            "padding".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "padding-vertical".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "padding-horizontal".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "padding-left".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "padding-right".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "padding-top".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "padding-bottom".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "border-top-radius".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "border-bottom-radius".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "border-left-radius".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "border-right-radius".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "width".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "min-width".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "max-width".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "height".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "min-height".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "max-height".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            // TODO: remove this after verifying that no existing document is using this
            "explain".to_string(),
            crate::p2::Kind::boolean().into_optional(),
        ),
        (
            "region".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "color".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "background-color".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "border-color".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "border-width".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "border-radius".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        ("id".to_string(), crate::p2::Kind::string().into_optional()),
        (
            "overflow-x".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "overflow-y".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "border-top".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "border-bottom".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "border-left".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "border-right".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "margin-top".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "margin-bottom".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "margin-left".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "margin-right".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "link".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "submit".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "open-in-new-tab".to_string(),
            crate::p2::Kind::boolean().into_optional(),
        ),
        (
            "sticky".to_string(),
            crate::p2::Kind::boolean().into_optional(),
        ),
        (
            "top".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "bottom".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "left".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "right".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "cursor".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "anchor".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "gradient-direction".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "gradient-colors".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "shadow-offset-x".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "shadow-offset-y".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "shadow-blur".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "shadow-size".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "shadow-color".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "background-image".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "background-repeat".to_string(),
            crate::p2::Kind::boolean().into_optional(),
        ),
        (
            "background-parallax".to_string(),
            crate::p2::Kind::boolean().into_optional(),
        ),
        (
            "scale".to_string(),
            crate::p2::Kind::decimal().into_optional(),
        ),
        (
            "scale-x".to_string(),
            crate::p2::Kind::decimal().into_optional(),
        ),
        (
            "scale-y".to_string(),
            crate::p2::Kind::decimal().into_optional(),
        ),
        (
            "rotate".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "move-up".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "move-down".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "move-left".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "move-right".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "position".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "inner".to_string(),
            crate::p2::Kind::boolean().into_optional(),
        ),
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
    properties: &std::collections::BTreeMap<String, crate::Value>,
    _doc: &crate::p2::TDoc,
) -> crate::p1::Result<ftd_rt::Container> {
    Ok(ftd_rt::Container {
        children: Default::default(),
        external_children: Default::default(),
        open: crate::p2::utils::string_bool_optional("open", properties)?,
        spacing: crate::p2::utils::int_optional("spacing", properties)?,
        wrap: crate::p2::utils::bool_with_default("wrap", false, properties)?,
    })
}

fn container_arguments() -> Vec<(String, crate::p2::Kind)> {
    vec![
        (
            "open".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "spacing".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "align".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "wrap".to_string(),
            crate::p2::Kind::boolean().into_optional(),
        ),
    ]
}

pub fn image_function() -> crate::Component {
    crate::Component {
        kernel: true,
        full_name: "ftd#image".to_string(),
        root: "ftd.kernel".to_string(),
        arguments: [
            vec![
                ("src".to_string(), crate::p2::Kind::string()),
                (
                    "description".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "align".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "crop".to_string(),
                    crate::p2::Kind::boolean().into_optional(),
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

pub fn image_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd_rt::Map,
    root_name: Option<&str>,
) -> crate::p1::Result<ftd_rt::Image> {
    let (src, reference) =
        crate::p2::utils::string_and_ref("src", properties_with_ref, all_locals)?;
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    Ok(ftd_rt::Image {
        src,
        description: crate::p2::utils::string_optional("description", properties)?
            .unwrap_or_else(|| "".to_string()),
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, reference,
        )?,
        crop: crate::p2::utils::bool_with_default("crop", false, properties)?,
    })
}

pub fn row_function() -> crate::Component {
    crate::Component {
        kernel: true,
        full_name: "ftd#row".to_string(),
        root: "ftd.kernel".to_string(),
        arguments: [container_arguments(), common_arguments()]
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

pub fn row_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd_rt::Map,
    root_name: Option<&str>,
) -> crate::p1::Result<ftd_rt::Row> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    Ok(ftd_rt::Row {
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, None,
        )?,
        container: container_from_properties(properties, doc)?,
    })
}

pub fn column_function() -> crate::Component {
    crate::Component {
        kernel: true,
        full_name: "ftd#column".to_string(),
        root: "ftd.kernel".to_string(),
        arguments: [container_arguments(), common_arguments()]
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
    properties_with_ref: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd_rt::Map,
    root_name: Option<&str>,
) -> crate::p1::Result<ftd_rt::Column> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    Ok(ftd_rt::Column {
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, None,
        )?,
        container: container_from_properties(properties, doc)?,
    })
}

pub fn external_font_from_properties(
    properties: &std::collections::BTreeMap<String, crate::Value>,
    _doc: &crate::p2::TDoc,
) -> crate::p1::Result<Option<ftd_rt::ExternalFont>> {
    let font_option = crate::p2::utils::string_optional("font", properties)?;
    let font_url_option = crate::p2::utils::string_optional("font-url", properties)?;

    match (font_option, font_url_option) {
        (Some(font), Some(font_url)) => {
            let name_opt = font.split(',').next();
            let name = match name_opt {
                Some(f) => f.to_string(),
                None => return crate::e("Something went wrong while parsing font vector"),
            };

            Ok(Some(ftd_rt::ExternalFont {
                url: font_url,
                display: ftd_rt::FontDisplay::from(crate::p2::utils::string_optional(
                    "font-display",
                    properties,
                )?)?,
                name,
            }))
        }
        _ => Ok(None),
    }
}

#[allow(unused_variables)]
pub fn text_render(
    tf: &ftd_rt::TextFormat,
    text: String,
    source: crate::TextSource,
    theme: String,
) -> crate::p1::Result<ftd_rt::Rendered> {
    Ok(match (source, tf) {
        (ftd::TextSource::Body, ftd_rt::TextFormat::Markdown) => ftd::markdown(text.as_str()),
        (_, ftd_rt::TextFormat::Markdown) => ftd::markdown_line(text.as_str()),
        (_, ftd_rt::TextFormat::Latex) => ftd::latex(text.as_str())?,
        (_, ftd_rt::TextFormat::Code { lang }) => {
            ftd::code_with_theme(text.as_str(), lang.as_str(), theme.as_str())?
        }
        (_, ftd_rt::TextFormat::Text) => ftd_rt::Rendered {
            original: text.clone(),
            rendered: text,
        },
    })
}

pub fn iframe_function() -> crate::Component {
    crate::Component {
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#iframe".to_string(),
        arguments: [
            vec![
                ("src".to_string(), crate::p2::Kind::string().into_optional()),
                (
                    "youtube".to_string(),
                    crate::p2::Kind::string().into_optional(),
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
    properties_with_ref: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd_rt::Map,
    root_name: Option<&str>,
) -> crate::p1::Result<ftd_rt::IFrame> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    let src = match (
        crate::p2::utils::string_optional("src", properties)?,
        crate::p2::utils::string_optional("youtube", properties)?
            .and_then(|id| crate::youtube_id::from_raw(id.as_str())),
    ) {
        (Some(src), None) => src,
        (None, Some(id)) => id,
        (Some(_), Some(_)) => return crate::e("both src and youtube id provided"),
        (None, None) => return crate::e("src or youtube id is required"),
    };

    Ok(ftd_rt::IFrame {
        src,
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, None,
        )?,
    })
}

pub fn text_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd_rt::Map,
    root_name: Option<&str>,
) -> crate::p1::Result<ftd_rt::Text> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    let format = ftd_rt::TextFormat::from(
        crate::p2::utils::string_optional("format", properties)?,
        crate::p2::utils::string_optional("lang", properties)?,
    )?;

    let (text, source, reference) =
        crate::p2::utils::string_and_source_and_ref("text", properties_with_ref, all_locals)?;
    let font_str = crate::p2::utils::string_optional("font", properties)?;

    let font: Vec<ftd_rt::NamedFont> = match font_str {
        Some(f) => f
            .split(',')
            .flat_map(|x| ftd_rt::NamedFont::from(Some(x.to_string())))
            .collect(),
        None => vec![],
    };
    Ok(ftd_rt::Text {
        line: source != ftd::TextSource::Body,
        text: text_render(
            &format,
            text,
            source,
            crate::p2::utils::string_with_default(
                "theme",
                crate::render::DEFAULT_THEME,
                properties,
            )?,
        )?,
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, reference,
        )?,
        text_align: ftd_rt::TextAlign::from(crate::p2::utils::string_optional(
            "text-align",
            properties,
        )?)?,
        style: ftd_rt::Style::from(crate::p2::utils::string_optional("style", properties)?)?,
        format,
        size: crate::p2::utils::int_optional("size", properties)?,
        external_font: external_font_from_properties(properties, doc)?,
        font,
        line_height: crate::p2::utils::int_optional("line-height", properties)?,
        line_clamp: crate::p2::utils::int_optional("line-clamp", properties)?,
    })
}

pub fn integer_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd_rt::Map,
    root_name: Option<&str>,
) -> crate::p1::Result<ftd_rt::Text> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    let font_str = crate::p2::utils::string_optional("font", properties)?;
    let num = format_num::NumberFormat::new();
    let text = match crate::p2::utils::string_optional("format", properties)? {
        Some(f) => num.format(
            f.as_str(),
            crate::p2::utils::int("value", properties)? as f64,
        ),
        None => crate::p2::utils::int("value", properties)?.to_string(),
    };
    let reference = ftd::p2::utils::complete_reference(
        &properties_with_ref.get("value").expect("").1,
        all_locals,
    );

    let font: Vec<ftd_rt::NamedFont> = match font_str {
        Some(f) => f
            .split(',')
            .flat_map(|x| ftd_rt::NamedFont::from(Some(x.to_string())))
            .collect(),
        None => vec![],
    };

    Ok(ftd_rt::Text {
        text: crate::markdown_line(text.as_str()),
        line: false,
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, reference,
        )?,
        text_align: ftd_rt::TextAlign::from(crate::p2::utils::string_optional(
            "text-align",
            properties,
        )?)?,
        style: ftd_rt::Style::from(crate::p2::utils::string_optional("style", properties)?)?,
        format: Default::default(),
        size: crate::p2::utils::int_optional("size", properties)?,
        external_font: external_font_from_properties(properties, doc)?,
        font,
        line_height: crate::p2::utils::int_optional("line-height", properties)?,
        line_clamp: crate::p2::utils::int_optional("line-clamp", properties)?,
    })
}

pub fn decimal_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd_rt::Map,
    root_name: Option<&str>,
) -> crate::p1::Result<ftd_rt::Text> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    let font_str = crate::p2::utils::string_optional("font", properties)?;
    let num = format_num::NumberFormat::new();
    let text = match crate::p2::utils::string_optional("format", properties)? {
        Some(f) => num.format(f.as_str(), crate::p2::utils::decimal("value", properties)?),
        None => crate::p2::utils::decimal("value", properties)?.to_string(),
    };

    let reference = ftd::p2::utils::complete_reference(
        &properties_with_ref.get("value").expect("").1,
        all_locals,
    );

    let font: Vec<ftd_rt::NamedFont> = match font_str {
        Some(f) => f
            .split(',')
            .flat_map(|x| ftd_rt::NamedFont::from(Some(x.to_string())))
            .collect(),
        None => vec![],
    };
    Ok(ftd_rt::Text {
        text: crate::markdown_line(text.as_str()),
        line: false,
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, reference,
        )?,
        text_align: ftd_rt::TextAlign::from(crate::p2::utils::string_optional(
            "text-align",
            properties,
        )?)?,
        style: ftd_rt::Style::from(crate::p2::utils::string_optional("style", properties)?)?,
        format: Default::default(),
        size: crate::p2::utils::int_optional("size", properties)?,
        external_font: external_font_from_properties(properties, doc)?,
        font,
        line_height: crate::p2::utils::int_optional("line-height", properties)?,
        line_clamp: crate::p2::utils::int_optional("line-clamp", properties)?,
    })
}

pub fn color_from(l: Option<String>) -> ftd::p1::Result<Option<ftd_rt::Color>> {
    use std::str::FromStr;

    let v = match l {
        Some(v) => v,
        None => return Ok(None),
    };

    match css_color_parser::Color::from_str(v.as_str()) {
        Ok(v) => Ok(Some(ftd_rt::Color {
            r: v.r,
            g: v.g,
            b: v.b,
            alpha: v.a,
        })),
        Err(e) => return crate::e(format!("{} is not a valid color: {:?}", v, e)),
    }
}

pub fn boolean_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd_rt::Map,
    root_name: Option<&str>,
) -> crate::p1::Result<ftd_rt::Text> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    let font_str = crate::p2::utils::string_optional("font", properties)?;
    let value = crate::p2::utils::bool("value", properties)?;
    let text = if value {
        crate::p2::utils::string_with_default("true", "true", properties)?
    } else {
        crate::p2::utils::string_with_default("false", "false", properties)?
    };

    let reference = ftd::p2::utils::complete_reference(
        &properties_with_ref.get("value").expect("").1,
        all_locals,
    );

    let font: Vec<ftd_rt::NamedFont> = match font_str {
        Some(f) => f
            .split(',')
            .flat_map(|x| ftd_rt::NamedFont::from(Some(x.to_string())))
            .collect(),
        None => vec![],
    };

    Ok(ftd_rt::Text {
        text: crate::markdown_line(text.as_str()),
        line: false,
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, reference,
        )?,
        text_align: ftd_rt::TextAlign::from(crate::p2::utils::string_optional(
            "text-align",
            properties,
        )?)?,
        style: ftd_rt::Style::from(crate::p2::utils::string_optional("style", properties)?)?,
        format: Default::default(),
        size: crate::p2::utils::int_optional("size", properties)?,
        external_font: external_font_from_properties(properties, doc)?,
        font,
        line_height: crate::p2::utils::int_optional("line-height", properties)?,
        line_clamp: crate::p2::utils::int_optional("line-clamp", properties)?,
    })
}

pub fn text_function() -> crate::Component {
    crate::Component {
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#text".to_string(),
        arguments: [
            vec![
                ("text".to_string(), crate::p2::Kind::caption_or_body()),
                (
                    "align".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "style".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "format".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "lang".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "theme".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "size".to_string(),
                    crate::p2::Kind::integer().into_optional(),
                ),
                (
                    "font-url".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "font".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "font-display".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "line-height".to_string(),
                    crate::p2::Kind::integer().into_optional(),
                ),
                (
                    "line-clamp".to_string(),
                    crate::p2::Kind::integer().into_optional(),
                ),
                (
                    "text-align".to_string(),
                    crate::p2::Kind::string().into_optional(),
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

pub fn integer_function() -> crate::Component {
    crate::Component {
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#integer".to_string(),
        arguments: [
            vec![
                ("value".to_string(), crate::p2::Kind::integer()),
                (
                    "align".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "style".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "format".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "size".to_string(),
                    crate::p2::Kind::integer().into_optional(),
                ),
                (
                    "font-url".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "font".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "font-display".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "line-height".to_string(),
                    crate::p2::Kind::string().into_optional(),
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

pub fn decimal_function() -> crate::Component {
    crate::Component {
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#decimal".to_string(),
        arguments: [
            vec![
                ("value".to_string(), crate::p2::Kind::decimal()),
                (
                    "align".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "style".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "format".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "size".to_string(),
                    crate::p2::Kind::integer().into_optional(),
                ),
                (
                    "font-url".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "font".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "font-display".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "line-height".to_string(),
                    crate::p2::Kind::string().into_optional(),
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

pub fn scene_function() -> crate::Component {
    let arguments = {
        let mut arguments: std::collections::BTreeMap<String, ftd::p2::Kind> =
            [container_arguments(), common_arguments()]
                .concat()
                .into_iter()
                .collect();
        arguments.remove("spacing");
        arguments.remove("wrap");
        arguments
    };

    crate::Component {
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

pub fn boolean_function() -> crate::Component {
    crate::Component {
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#boolean".to_string(),
        arguments: [
            vec![
                ("value".to_string(), crate::p2::Kind::boolean()),
                (
                    "align".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "style".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "format".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "size".to_string(),
                    crate::p2::Kind::integer().into_optional(),
                ),
                (
                    "font-url".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "font".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "font-display".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "line-height".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "true".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "false".to_string(),
                    crate::p2::Kind::string().into_optional(),
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

pub fn input_function() -> crate::Component {
    crate::Component {
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#input".to_string(),
        arguments: [
            vec![(
                "placeholder".to_string(),
                crate::p2::Kind::string().into_optional(),
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
    properties_with_ref: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd_rt::Map,
    root_name: Option<&str>,
) -> crate::p1::Result<ftd_rt::Input> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    Ok(ftd_rt::Input {
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, None,
        )?,
        placeholder: crate::p2::utils::string_optional("placeholder", properties)?,
    })
}

pub fn scene_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd_rt::Map,
    root_name: Option<&str>,
) -> crate::p1::Result<ftd_rt::Scene> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    Ok(ftd_rt::Scene {
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, None,
        )?,
        container: container_from_properties(properties, doc)?,
    })
}
