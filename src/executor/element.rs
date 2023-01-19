#[derive(serde::Deserialize, Clone, Debug, PartialEq, serde::Serialize)]
pub enum Element {
    Row(Row),
    Column(Column),
    Text(Text),
    Integer(Text),
    Boolean(Text),
    Decimal(Text),
    Image(Image),
    Code(Code),
    Iframe(Iframe),
    TextInput(TextInput),
    Null,
}

impl Element {
    pub(crate) fn get_common(&self) -> Option<&Common> {
        match self {
            Element::Row(r) => Some(&r.common),
            Element::Column(c) => Some(&c.common),
            Element::Text(t) => Some(&t.common),
            Element::Integer(i) => Some(&i.common),
            Element::Boolean(b) => Some(&b.common),
            Element::Decimal(d) => Some(&d.common),
            Element::Image(i) => Some(&i.common),
            Element::Code(c) => Some(&c.common),
            Element::Iframe(i) => Some(&i.common),
            Element::TextInput(i) => Some(&i.common),
            Element::Null => None,
        }
    }
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct Row {
    pub container: Container,
    pub common: Common,
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct Column {
    pub container: Container,
    pub common: Common,
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Text {
    pub text: ftd::executor::Value<Rendered>,
    pub text_align: ftd::executor::Value<Option<ftd::executor::TextAlign>>,
    pub common: Common,
}

#[derive(serde::Serialize, serde::Deserialize, Eq, PartialEq, Debug, Default, Clone)]
pub struct Rendered {
    pub original: String,
    pub rendered: String,
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Image {
    pub src: ftd::executor::Value<ImageSrc>,
    pub common: Common,
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct ImageSrc {
    pub light: ftd::executor::Value<String>,
    pub dark: ftd::executor::Value<String>,
}

impl ImageSrc {
    fn from_values(
        values: ftd::Map<ftd::interpreter2::PropertyValue>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<ImageSrc> {
        let light = {
            let value = values
                .get("light")
                .ok_or(ftd::executor::Error::ParseError {
                    message: "`light` field in ftd.image-src not found".to_string(),
                    doc_id: doc.name.to_string(),
                    line_number,
                })?;
            ftd::executor::Value::new(
                value
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .string(doc.name, line_number)?,
                Some(line_number),
                vec![value.into_property(ftd::interpreter2::PropertySource::header("light"))],
            )
        };

        let dark = {
            if let Some(value) = values.get("dark") {
                ftd::executor::Value::new(
                    value
                        .clone()
                        .resolve(&doc.itdoc(), line_number)?
                        .string(doc.name, line_number)?,
                    Some(line_number),
                    vec![value.into_property(ftd::interpreter2::PropertySource::header("dark"))],
                )
            } else {
                light.clone()
            }
        };

        Ok(ImageSrc { light, dark })
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Code {
    pub text: ftd::executor::Value<Rendered>,
    pub text_align: ftd::executor::Value<Option<ftd::executor::TextAlign>>,
    pub common: Common,
}

pub fn code_from_properties(
    properties: &[ftd::interpreter2::Property],
    events: &[ftd::interpreter2::Event],
    arguments: &[ftd::interpreter2::Argument],
    condition: &Option<ftd::interpreter2::Expression>,
    doc: &ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
) -> ftd::executor::Result<Code> {
    // TODO: `text`, `lang` and `theme` cannot have condition

    let text =
        ftd::executor::value::optional_string("text", properties, arguments, doc, line_number)?;
    if text.value.is_none() && condition.is_none() {
        // TODO: Check condition if `value is not null` is there
        return ftd::executor::utils::parse_error(
            "Expected string for text property",
            doc.name,
            line_number,
        );
    }

    let lang = ftd::executor::value::string_with_default(
        "lang",
        properties,
        arguments,
        "txt",
        doc,
        line_number,
    )?;

    let theme = ftd::executor::value::string_with_default(
        "theme",
        properties,
        arguments,
        ftd::executor::code::DEFAULT_THEME,
        doc,
        line_number,
    )?;

    let text = ftd::executor::Value::new(
        ftd::executor::element::code_with_theme(
            text.value.unwrap_or_default().as_str(),
            lang.value.as_str(),
            theme.value.as_str(),
            doc.name,
        )?,
        text.line_number,
        text.properties,
    );

    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
    )?;

    Ok(Code {
        text,
        text_align: ftd::executor::TextAlign::optional_text_align(
            properties,
            arguments,
            doc,
            line_number,
            "text-align",
        )?,
        common,
    })
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Iframe {
    pub src: ftd::executor::Value<Option<String>>,
    pub srcdoc: ftd::executor::Value<Option<String>>,
    /// iframe can load lazily.
    pub loading: ftd::executor::Value<ftd::executor::Loading>,
    pub common: Common,
}

pub fn iframe_from_properties(
    properties: &[ftd::interpreter2::Property],
    events: &[ftd::interpreter2::Event],
    arguments: &[ftd::interpreter2::Argument],
    condition: &Option<ftd::interpreter2::Expression>,
    doc: &ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
) -> ftd::executor::Result<Iframe> {
    // TODO: `youtube` should not be conditional
    let srcdoc =
        ftd::executor::value::optional_string("srcdoc", properties, arguments, doc, line_number)?;

    let src = {
        let src =
            ftd::executor::value::optional_string("src", properties, arguments, doc, line_number)?;

        let youtube = ftd::executor::value::optional_string(
            "youtube",
            properties,
            arguments,
            doc,
            line_number,
        )?
        .map(|v| v.and_then(|v| ftd::executor::youtube_id::from_raw(v.as_str())));

        if [
            src.value.is_some(),
            youtube.value.is_some(),
            srcdoc.value.is_some(),
        ]
        .into_iter()
        .filter(|b| *b)
        .count()
            > 1
        {
            return ftd::executor::utils::parse_error(
                "Two or more than two values are provided among src, youtube and srcdoc.",
                doc.name,
                src.line_number.unwrap_or_else(|| {
                    youtube
                        .line_number
                        .unwrap_or_else(|| srcdoc.line_number.unwrap_or(line_number))
                }),
            );
        }
        if src.value.is_none() && youtube.value.is_none() && srcdoc.value.is_none() {
            return ftd::executor::utils::parse_error(
                "Either srcdoc or src or youtube id is required",
                doc.name,
                line_number,
            );
        }
        if src.value.is_some() {
            src
        } else {
            youtube
        }
    };

    let loading = ftd::executor::Loading::loading_with_default(
        properties,
        arguments,
        doc,
        line_number,
        "loading",
    )?;

    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
    )?;

    Ok(Iframe {
        src,
        srcdoc,
        loading,
        common,
    })
}

pub fn markup_inline(s: &str) -> Rendered {
    Rendered {
        original: s.to_string(),
        rendered: ftd::executor::markup::markup_inline(s),
    }
}

pub fn code_with_theme(
    code: &str,
    ext: &str,
    theme: &str,
    doc_id: &str,
) -> ftd::executor::Result<Rendered> {
    Ok(Rendered {
        original: code.to_string(),
        rendered: ftd::executor::code::code(
            code.replace("\n\\-- ", "\n-- ").as_str(),
            ext,
            theme,
            doc_id,
        )?,
    })
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Container {
    pub spacing: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub wrap: ftd::executor::Value<Option<bool>>,
    pub align_content: ftd::executor::Value<ftd::executor::Alignment>,
    pub spacing_mode: ftd::executor::Value<Option<ftd::executor::SpacingMode>>,
    pub children: Vec<Element>,
}

pub type Event = ftd::interpreter2::Event;

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Common {
    pub is_not_visible: bool,
    pub event: Vec<Event>,
    pub is_dummy: bool,
    pub z_index: ftd::executor::Value<Option<i64>>,
    pub left: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub right: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub top: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub bottom: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub anchor: ftd::executor::Value<Option<ftd::executor::Anchor>>,
    pub role: ftd::executor::Value<Option<ftd::executor::ResponsiveType>>,
    pub region: ftd::executor::Value<Option<ftd::executor::Region>>,
    pub cursor: ftd::executor::Value<Option<ftd::executor::Cursor>>,
    pub classes: ftd::executor::Value<Vec<String>>,
    pub padding: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub padding_left: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub padding_right: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub padding_top: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub padding_bottom: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub padding_horizontal: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub padding_vertical: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub margin: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub margin_left: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub margin_right: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub margin_top: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub margin_bottom: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub margin_horizontal: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub margin_vertical: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub border_width: ftd::executor::Value<ftd::executor::Length>,
    pub border_radius: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub border_color: ftd::executor::Value<Option<ftd::executor::Color>>,
    pub border_bottom_width: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub border_bottom_color: ftd::executor::Value<Option<ftd::executor::Color>>,
    pub border_top_width: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub border_top_color: ftd::executor::Value<Option<ftd::executor::Color>>,
    pub border_left_width: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub border_left_color: ftd::executor::Value<Option<ftd::executor::Color>>,
    pub border_right_width: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub border_right_color: ftd::executor::Value<Option<ftd::executor::Color>>,
    pub border_top_left_radius: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub border_top_right_radius: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub border_bottom_left_radius: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub border_bottom_right_radius: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub width: ftd::executor::Value<ftd::executor::Resizing>,
    pub height: ftd::executor::Value<ftd::executor::Resizing>,
    pub min_width: ftd::executor::Value<Option<ftd::executor::Resizing>>,
    pub max_width: ftd::executor::Value<Option<ftd::executor::Resizing>>,
    pub min_height: ftd::executor::Value<Option<ftd::executor::Resizing>>,
    pub max_height: ftd::executor::Value<Option<ftd::executor::Resizing>>,
    pub link: ftd::executor::Value<Option<String>>,
    pub open_in_new_tab: ftd::executor::Value<Option<bool>>,
    pub background: ftd::executor::Value<Option<ftd::executor::Background>>,
    pub color: ftd::executor::Value<Option<ftd::executor::Color>>,
    pub align_self: ftd::executor::Value<Option<ftd::executor::AlignSelf>>,
    pub data_id: String,
    pub line_number: usize,
    pub condition: Option<ftd::interpreter2::Expression>,
    pub overflow: ftd::executor::Value<Option<ftd::executor::Overflow>>,
    pub overflow_x: ftd::executor::Value<Option<ftd::executor::Overflow>>,
    pub overflow_y: ftd::executor::Value<Option<ftd::executor::Overflow>>,
    pub resize: ftd::executor::Value<Option<ftd::executor::Resize>>,
    pub white_space: ftd::executor::Value<Option<ftd::executor::WhiteSpace>>,
    pub text_transform: ftd::executor::Value<Option<ftd::executor::TextTransform>>,
}

pub fn default_column() -> Column {
    ftd::executor::Column {
        container: Default::default(),
        common: ftd::executor::Common {
            width: ftd::executor::Value::new(ftd::executor::Resizing::FillContainer, None, vec![]),
            height: ftd::executor::Value::new(ftd::executor::Resizing::FillContainer, None, vec![]),
            ..Default::default()
        },
    }
}

pub fn text_from_properties(
    properties: &[ftd::interpreter2::Property],
    events: &[ftd::interpreter2::Event],
    arguments: &[ftd::interpreter2::Argument],
    condition: &Option<ftd::interpreter2::Expression>,
    doc: &ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
) -> ftd::executor::Result<Text> {
    let text =
        ftd::executor::value::optional_string("text", properties, arguments, doc, line_number)?;
    if text.value.is_none() && condition.is_none() {
        // TODO: Check condition if `value is not null` is there
        return ftd::executor::utils::parse_error(
            "Expected string for text property",
            doc.name,
            line_number,
        );
    }
    let text = text.map(|v| ftd::executor::element::markup_inline(v.unwrap_or_default().as_str()));
    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
    )?;
    Ok(Text {
        text,
        text_align: ftd::executor::TextAlign::optional_text_align(
            properties,
            arguments,
            doc,
            line_number,
            "text-align",
        )?,
        common,
    })
}

pub fn integer_from_properties(
    properties: &[ftd::interpreter2::Property],
    events: &[ftd::interpreter2::Event],
    arguments: &[ftd::interpreter2::Argument],
    condition: &Option<ftd::interpreter2::Expression>,
    doc: &ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
) -> ftd::executor::Result<Text> {
    let value = ftd::executor::value::i64("value", properties, arguments, doc, line_number)?;
    let num = format_num::NumberFormat::new();
    let text = match ftd::executor::value::optional_string(
        "format",
        properties,
        arguments,
        doc,
        line_number,
    )?
    .value
    {
        Some(f) => value.map(|v| {
            ftd::executor::element::markup_inline(num.format(f.as_str(), v as f64).as_str())
        }),
        None => value.map(|v| ftd::executor::element::markup_inline(v.to_string().as_str())),
    };
    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
    )?;
    Ok(Text {
        text,
        common,
        text_align: ftd::executor::TextAlign::optional_text_align(
            properties,
            arguments,
            doc,
            line_number,
            "text-align",
        )?,
    })
}

pub fn decimal_from_properties(
    properties: &[ftd::interpreter2::Property],
    events: &[ftd::interpreter2::Event],
    arguments: &[ftd::interpreter2::Argument],
    condition: &Option<ftd::interpreter2::Expression>,
    doc: &ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
) -> ftd::executor::Result<Text> {
    let value = ftd::executor::value::f64("value", properties, arguments, doc, line_number)?;
    let num = format_num::NumberFormat::new();
    let text = match ftd::executor::value::optional_string(
        "format",
        properties,
        arguments,
        doc,
        line_number,
    )?
    .value
    {
        Some(f) => value.map(|v| {
            ftd::executor::element::markup_inline(num.format(f.as_str(), v as f64).as_str())
        }),
        None => value.map(|v| ftd::executor::element::markup_inline(v.to_string().as_str())),
    };
    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
    )?;
    Ok(Text {
        text,
        common,
        text_align: ftd::executor::TextAlign::optional_text_align(
            properties,
            arguments,
            doc,
            line_number,
            "text-align",
        )?,
    })
}

pub fn boolean_from_properties(
    properties: &[ftd::interpreter2::Property],
    events: &[ftd::interpreter2::Event],
    arguments: &[ftd::interpreter2::Argument],
    condition: &Option<ftd::interpreter2::Expression>,
    doc: &ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
) -> ftd::executor::Result<Text> {
    let value = ftd::executor::value::bool("value", properties, arguments, doc, line_number)?;
    let text = value.map(|v| ftd::executor::element::markup_inline(v.to_string().as_str()));
    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
    )?;
    Ok(Text {
        text,
        common,
        text_align: ftd::executor::TextAlign::optional_text_align(
            properties,
            arguments,
            doc,
            line_number,
            "text-align",
        )?,
    })
}

pub fn image_from_properties(
    properties: &[ftd::interpreter2::Property],
    events: &[ftd::interpreter2::Event],
    arguments: &[ftd::interpreter2::Argument],
    condition: &Option<ftd::interpreter2::Expression>,
    doc: &ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
) -> ftd::executor::Result<Image> {
    let src = {
        let src = ftd::executor::value::record(
            "src",
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_IMAGE_SRC,
        )?;
        ftd::executor::Value::new(
            ImageSrc::from_values(src.value, doc, line_number)?,
            Some(line_number),
            src.properties,
        )
    };

    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
    )?;
    Ok(Image { src, common })
}

#[allow(clippy::too_many_arguments)]
pub fn row_from_properties(
    properties: &[ftd::interpreter2::Property],
    events: &[ftd::interpreter2::Event],
    arguments: &[ftd::interpreter2::Argument],
    condition: &Option<ftd::interpreter2::Expression>,
    doc: &ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
    children: Vec<Element>,
) -> ftd::executor::Result<Row> {
    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
    )?;
    let container = container_from_properties(properties, arguments, doc, line_number, children)?;
    Ok(Row { container, common })
}

#[allow(clippy::too_many_arguments)]
pub fn column_from_properties(
    properties: &[ftd::interpreter2::Property],
    events: &[ftd::interpreter2::Event],
    arguments: &[ftd::interpreter2::Argument],
    condition: &Option<ftd::interpreter2::Expression>,
    doc: &ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
    children: Vec<Element>,
) -> ftd::executor::Result<Column> {
    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
    )?;
    let container = container_from_properties(properties, arguments, doc, line_number, children)?;
    Ok(Column { container, common })
}

pub fn common_from_properties(
    properties: &[ftd::interpreter2::Property],
    events: &[ftd::interpreter2::Event],
    arguments: &[ftd::interpreter2::Argument],
    condition: &Option<ftd::interpreter2::Expression>,
    doc: &ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
) -> ftd::executor::Result<Common> {
    let is_visible = if let Some(condition) = condition {
        condition.eval(&doc.itdoc())?
    } else {
        true
    };

    Ok(Common {
        is_not_visible: !is_visible,
        event: events.to_owned(),
        is_dummy: false,
        z_index: ftd::executor::value::optional_i64(
            "z-index",
            properties,
            arguments,
            doc,
            line_number,
        )?,
        left: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "left",
        )?,
        right: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "right",
        )?,
        top: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "top",
        )?,
        bottom: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "bottom",
        )?,
        anchor: ftd::executor::Anchor::optional_anchor(
            properties,
            arguments,
            doc,
            line_number,
            "anchor",
        )?,
        role: ftd::executor::ResponsiveType::optional_responsive_type(
            properties,
            arguments,
            doc,
            line_number,
            "role",
        )?,
        region: ftd::executor::Region::optional_region(
            properties,
            arguments,
            doc,
            line_number,
            "region",
        )?,
        cursor: ftd::executor::Cursor::optional_cursor(
            properties,
            arguments,
            doc,
            line_number,
            "cursor",
        )?,
        text_transform: ftd::executor::TextTransform::optional_text_transform(
            properties,
            arguments,
            doc,
            line_number,
            "text-transform",
        )?,
        classes: ftd::executor::value::string_list(
            "classes",
            properties,
            arguments,
            doc,
            line_number,
        )?,
        padding: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "padding",
        )?,
        padding_left: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "padding-left",
        )?,
        padding_right: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "padding-right",
        )?,
        padding_top: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "padding-top",
        )?,
        padding_bottom: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "padding-bottom",
        )?,
        padding_horizontal: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "padding-horizontal",
        )?,
        padding_vertical: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "padding-vertical",
        )?,
        margin: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "margin",
        )?,
        margin_left: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "margin-left",
        )?,
        margin_right: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "margin-right",
        )?,
        margin_top: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "margin-top",
        )?,
        margin_bottom: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "margin-bottom",
        )?,
        margin_horizontal: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "margin-horizontal",
        )?,
        margin_vertical: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "margin-vertical",
        )?,
        border_width: ftd::executor::Length::length_with_default(
            properties,
            arguments,
            doc,
            line_number,
            "border-width",
            ftd::executor::Length::Px(0),
        )?,
        border_radius: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "border-radius",
        )?,
        border_color: ftd::executor::Color::optional_color(
            properties,
            arguments,
            doc,
            line_number,
            "border-color",
        )?,
        border_bottom_width: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "border-bottom-width",
        )?,
        border_bottom_color: ftd::executor::Color::optional_color(
            properties,
            arguments,
            doc,
            line_number,
            "border-bottom-color",
        )?,
        border_top_width: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "border-top-width",
        )?,
        border_top_color: ftd::executor::Color::optional_color(
            properties,
            arguments,
            doc,
            line_number,
            "border-top-color",
        )?,
        border_left_width: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "border-left-width",
        )?,
        border_left_color: ftd::executor::Color::optional_color(
            properties,
            arguments,
            doc,
            line_number,
            "border-left-color",
        )?,
        border_right_width: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "border-right-width",
        )?,
        border_right_color: ftd::executor::Color::optional_color(
            properties,
            arguments,
            doc,
            line_number,
            "border-right-color",
        )?,
        border_top_left_radius: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "border-top-left-radius",
        )?,
        border_top_right_radius: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "border-top-right-radius",
        )?,
        border_bottom_left_radius: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "border-bottom-left-radius",
        )?,
        border_bottom_right_radius: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "border-bottom-right-radius",
        )?,
        width: ftd::executor::Resizing::resizing_with_default(
            properties,
            arguments,
            doc,
            line_number,
            "width",
            ftd::executor::Resizing::default(),
        )?,
        height: ftd::executor::Resizing::resizing_with_default(
            properties,
            arguments,
            doc,
            line_number,
            "height",
            ftd::executor::Resizing::default(),
        )?,
        min_width: ftd::executor::Resizing::optional_resizing(
            properties,
            arguments,
            doc,
            line_number,
            "min-width",
        )?,
        max_width: ftd::executor::Resizing::optional_resizing(
            properties,
            arguments,
            doc,
            line_number,
            "max-width",
        )?,
        min_height: ftd::executor::Resizing::optional_resizing(
            properties,
            arguments,
            doc,
            line_number,
            "min-height",
        )?,
        max_height: ftd::executor::Resizing::optional_resizing(
            properties,
            arguments,
            doc,
            line_number,
            "max-height",
        )?,
        link: ftd::executor::value::optional_string(
            "link",
            properties,
            arguments,
            doc,
            line_number,
        )?,
        open_in_new_tab: ftd::executor::value::optional_bool(
            "open-in-new-tab",
            properties,
            arguments,
            doc,
            line_number,
        )?,
        condition: condition.to_owned(),
        data_id: ftd::executor::utils::get_string_container(local_container),
        line_number,
        background: ftd::executor::Background::optional_fill(
            properties,
            arguments,
            doc,
            line_number,
            "background",
        )?,
        color: ftd::executor::Color::optional_color(
            properties,
            arguments,
            doc,
            line_number,
            "color",
        )?,
        align_self: ftd::executor::AlignSelf::optional_align_self(
            properties,
            arguments,
            doc,
            line_number,
            "align-self",
        )?,
        overflow: ftd::executor::Overflow::optional_overflow(
            properties,
            arguments,
            doc,
            line_number,
            "overflow",
        )?,
        overflow_x: ftd::executor::Overflow::optional_overflow(
            properties,
            arguments,
            doc,
            line_number,
            "overflow-x",
        )?,
        overflow_y: ftd::executor::Overflow::optional_overflow(
            properties,
            arguments,
            doc,
            line_number,
            "overflow-y",
        )?,
        resize: ftd::executor::Resize::optional_resize(
            properties,
            arguments,
            doc,
            line_number,
            "resize",
        )?,
        white_space: ftd::executor::WhiteSpace::optional_whitespace(
            properties,
            arguments,
            doc,
            line_number,
            "white-space",
        )?,
    })
}

pub fn container_from_properties(
    properties: &[ftd::interpreter2::Property],
    arguments: &[ftd::interpreter2::Argument],
    doc: &ftd::executor::TDoc,
    line_number: usize,
    children: Vec<Element>,
) -> ftd::executor::Result<Container> {
    Ok(Container {
        spacing: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "spacing",
        )?,
        wrap: ftd::executor::value::optional_bool("wrap", properties, arguments, doc, line_number)?,
        align_content: ftd::executor::Alignment::alignment_with_default(
            properties,
            arguments,
            doc,
            line_number,
            "align-content",
            ftd::executor::Alignment::TopLeft,
        )?,
        spacing_mode: ftd::executor::SpacingMode::optional_spacing_mode(
            properties,
            arguments,
            doc,
            line_number,
            "spacing-mode",
        )?,
        children,
    })
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct TextInput {
    pub placeholder: ftd::executor::Value<Option<String>>,
    pub value: ftd::executor::Value<Option<String>>,
    pub multiline: ftd::executor::Value<bool>,
    pub default_value: ftd::executor::Value<Option<String>>,
    pub type_: ftd::executor::Value<Option<ftd::executor::TextInputType>>,
    pub common: Common,
}

pub fn text_input_from_properties(
    properties: &[ftd::interpreter2::Property],
    events: &[ftd::interpreter2::Event],
    arguments: &[ftd::interpreter2::Argument],
    condition: &Option<ftd::interpreter2::Expression>,
    doc: &ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
) -> ftd::executor::Result<TextInput> {
    // TODO: `youtube` should not be conditional
    let placeholder = ftd::executor::value::optional_string(
        "placeholder",
        properties,
        arguments,
        doc,
        line_number,
    )?;

    let value =
        ftd::executor::value::optional_string("value", properties, arguments, doc, line_number)?;

    let multiline = ftd::executor::value::bool_with_default(
        "multiline",
        properties,
        arguments,
        false,
        doc,
        line_number,
    )?;

    let default_value = ftd::executor::value::optional_string(
        "default-value",
        properties,
        arguments,
        doc,
        line_number,
    )?;

    let type_ =
        ftd::executor::TextInputType::optional_text_input_type(properties, arguments, doc, line_number, "type")?;

    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
    )?;

    Ok(TextInput {
        placeholder,
        value,
        multiline,
        default_value,
        common,
        type_,
    })
}
