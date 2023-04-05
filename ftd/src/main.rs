#[allow(unreachable_code)]
#[allow(dead_code)]
fn t() {
    // The returned nodes are created in the supplied Arena, and are bound by its lifetime.
    let arena = comrak::Arena::new();

    let root = comrak::parse_document(
        &arena,
        "This is my input.\n\n1. Also my input.\n2. Certainly my input.\n",
        &comrak::ComrakOptions::default(),
    );

    dbg!(root);
    return;

    fn iter_nodes<'a, F>(node: &'a comrak::nodes::AstNode<'a>, f: &F)
    where
        F: Fn(&'a comrak::nodes::AstNode<'a>),
    {
        f(node);
        for c in node.children() {
            iter_nodes(c, f);
        }
    }

    iter_nodes(root, &|node| {
        dbg!(root);
        dbg!(node);
        // match &mut node.data.borrow_mut().value {
        //     &mut NodeValue::Text(ref mut text) => {
        //         let orig = std::mem::replace(text, vec![]);
        //         *text = String::from_utf8(orig).unwrap().replace("my", "your").as_bytes().to_vec();
        //     }
        //     _ => (),
        // }
    });
}

pub fn main() {
    // t();
    // return;

    let id = std::env::args().nth(1);

    if id.is_some() && id.as_ref().unwrap().eq("bm") {
        use std::io::Write;

        let mut log = "".to_string();
        let benchmark_dir = std::path::Path::new("./benchmark-2022/");
        for entry in std::fs::read_dir(benchmark_dir)
            .unwrap_or_else(|_| panic!("{:?} is not a directory", benchmark_dir.to_str()))
        {
            let path = entry.expect("no files inside ./benchmark-2022").path();
            let source = path
                .to_str()
                .unwrap_or_else(|| panic!("Path {:?} cannot be convert to string", path));
            let split: Vec<_> = source.split('/').collect();
            let id = split.last().expect("Filename should be present");
            if id.contains(".ftd") {
                let start = std::time::Instant::now();
                log = format!("{}Processing: {} ... ", log, id);
                let doc = std::fs::read_to_string(source).expect("cant read file");
                ftd_v2_write(id, doc.as_str());
                log = format!("{}Done {:?}\n", log, start.elapsed());
            }
        }
        let mut f =
            std::fs::File::create("./benchmark-2022/.log").expect("failed to create .html file");
        f.write_all(log.as_bytes())
            .expect("failed to write to .log file");
        return;
    }
    let new_ftd_dir = std::path::Path::new("./ftd/t/html/");

    let mut write_doc = indoc::indoc!(
        "
-- ftd.column:
padding-horizontal.px: 40
padding-vertical.px: 20

-- ftd.text: FTD Examples
role: $inherited.types.heading-hero
padding-bottom.px: 20

"
    )
    .to_string();

    if id.is_none() && new_ftd_dir.is_dir() {
        for entry in std::fs::read_dir(new_ftd_dir)
            .unwrap_or_else(|_| panic!("{:?} is not a directory", new_ftd_dir.to_str()))
        {
            let path = entry.expect("no files inside ./examples").path();
            let source = path
                .to_str()
                .unwrap_or_else(|| panic!("Path {:?} cannot be convert to string", path));
            let split: Vec<_> = source.split('/').collect();
            let id = split.last().expect("Filename should be present");
            if id.contains(".ftd") {
                let doc = std::fs::read_to_string(source).expect("cant read file");
                ftd_v2_write(id, doc.as_str());
                write_doc = format!(
                    "{}\n-- ftd.text: {} \n link: {}\nrole: $inherited.types.heading-small\n",
                    write_doc,
                    id.replace(".ftd", ""),
                    id.replace(".ftd", ".html"),
                );
            }
        }
    }

    write_doc = format!("{}\n-- end: ftd.column\n", write_doc,);

    ftd_v2_write("index.ftd", write_doc.as_str());
    std::fs::create_dir_all("./docs/ftd/t/").expect("failed to create docs folder");
    std::fs::copy("./ftd/t/test.css", "./docs/ftd/t/test.css").expect("failed to copy test.css");
    std::fs::copy("./ftd/t/test.js", "./docs/ftd/t/test.js").expect("failed to copy test.js");
    std::fs::copy("./ftd/t/web_component.js", "./docs/ftd/t/web_component.js")
        .expect("failed to copy web_component.js");
}

pub fn ftd_v2_interpret_helper(
    name: &str,
    source: &str,
) -> ftd::interpreter::Result<ftd::interpreter::Document> {
    let mut s = ftd::interpreter::interpret(name, source)?;
    let document;
    loop {
        match s {
            ftd::interpreter::Interpreter::Done { document: doc } => {
                document = doc;
                break;
            }
            ftd::interpreter::Interpreter::StuckOnImport {
                module, state: st, ..
            } => {
                let mut source = "".to_string();
                let mut foreign_variable = vec![];
                let mut foreign_function = vec![];
                if module.eq("test") {
                    foreign_variable.push("var".to_string());
                    foreign_function.push("fn".to_string());
                }
                if let Ok(value) = std::fs::read_to_string(format!("./ftd/t/html/{}.ftd", module)) {
                    source = value;
                }
                let document = ftd::interpreter::ParsedDocument::parse_with_line_number(
                    module.as_str(),
                    source.as_str(),
                    0,
                )?;

                s = st.continue_after_import(
                    module.as_str(),
                    document,
                    foreign_variable,
                    foreign_function,
                    0,
                )?;
            }
            ftd::interpreter::Interpreter::StuckOnProcessor {
                state, ast, module, ..
            } => {
                let variable_definition = ast.clone().get_variable_definition(module.as_str())?;
                let processor = variable_definition.processor.unwrap();
                let value = ftd::interpreter::Value::String {
                    text: variable_definition
                        .value
                        .caption()
                        .unwrap_or(processor)
                        .to_uppercase()
                        .to_string(),
                };
                s = state.continue_after_processor(value, ast)?;
            }
            ftd::interpreter::Interpreter::StuckOnForeignVariable {
                state,
                module,
                variable,
                ..
            } => {
                if module.eq("test") {
                    let value = ftd::interpreter::Value::String {
                        text: variable.to_uppercase().to_string(),
                    };
                    s = state.continue_after_variable(module.as_str(), variable.as_str(), value)?;
                } else {
                    return ftd::interpreter::utils::e2(
                        format!("Unknown module {}", module),
                        module.as_str(),
                        0,
                    );
                }
            }
        }
    }
    Ok(document)
}

fn ftd_v2_write(id: &str, s: &str) {
    use std::io::Write;
    let start = std::time::Instant::now();
    print!("Processing: {} ... ", id);
    let doc = ftd_v2_interpret_helper("foo", s).unwrap_or_else(|e| panic!("{:?}", e));
    let executor =
        ftd::executor::ExecuteDoc::from_interpreter(doc).unwrap_or_else(|e| panic!("{:?}", e));
    let node = ftd::node::NodeData::from_rt(executor);
    let html_ui = ftd::html::HtmlUI::from_node_data(node, "main", false)
        .unwrap_or_else(|e| panic!("{:?}", e));
    let ftd_js = std::fs::read_to_string("./ftd/build.js").expect("build.js not found");
    let html_str = ftd::html::utils::trim_all_lines(
        std::fs::read_to_string("./ftd/build.html")
            .expect("cant read ftd.html")
            .replace(
                "__ftd_meta_data__",
                ftd::html::utils::get_meta_data(&html_ui.html_data).as_str(),
            )
            .replace(
                "__ftd_doc_title__",
                html_ui.html_data.title.unwrap_or_default().as_str(),
            )
            .replace("__ftd_data__", html_ui.variables.as_str())
            .replace("__ftd_external_children__", "{}")
            .replace("__ftd__", html_ui.html.as_str())
            .replace("__ftd_js__", ftd_js.as_str())
            .replace("__extra_js__", html_ui.js.as_str())
            .replace("__base_url__", "/fastn/")
            .replace("__extra_css__", html_ui.css.as_str())
            .replace(
                "__ftd_functions__",
                format!(
                    "{}\n{}\n{}\n{}\n{}\n{}\n{}",
                    html_ui.functions.as_str(),
                    html_ui.dependencies.as_str(),
                    html_ui.variable_dependencies.as_str(),
                    html_ui.dummy_html.as_str(),
                    html_ui.raw_html.as_str(),
                    html_ui.mutable_variable,
                    html_ui.immutable_variable
                )
                .as_str(),
            )
            .replace("__ftd_body_events__", html_ui.outer_events.as_str())
            .replace("__ftd_css__", ftd::css())
            .replace("__ftd_element_css__", "")
            .as_str(),
    );
    std::fs::create_dir_all("./docs").expect("failed to create docs folder");
    let mut f = std::fs::File::create(format!("./docs/{}", id.replace(".ftd", ".html")))
        .expect("failed to create .html file");
    f.write_all(html_str.as_bytes())
        .expect("failed to write to .html file");
    let duration = start.elapsed();
    println!("Done {:?}", duration);
}
