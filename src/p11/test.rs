use {indoc::indoc, pretty_assertions::assert_eq}; // macro

#[track_caller]
fn p(s: &str, t: Vec<ftd::p11::Section>) {
    assert_eq!(
        t,
        super::parse(s, "foo")
            .unwrap_or_else(|e| panic!("{:?}", e))
            .iter()
            .map(|v| v.without_line_number())
            .collect::<Vec<ftd::p11::Section>>(),
    )
}

#[track_caller]
fn f(s: &str, m: &str) {
    match super::parse(s, "foo") {
        Ok(r) => panic!("expected failure, found: {:?}", r),
        Err(e) => {
            let expected = m.trim();
            let f2 = e.to_string();
            let found = f2.trim();
            if expected != found {
                let patch = diffy::create_patch(expected, found);
                let f = diffy::PatchFormatter::new().with_color();
                print!(
                    "{}",
                    f.fmt_patch(&patch)
                        .to_string()
                        .replace("\\ No newline at end of file", "")
                );
                println!("expected:\n{}\nfound:\n{}\n", expected, f2);
                panic!("test failed")
            }
        }
    }
}

fn replace_extension(path: &std::path::Path, new_extension: &str) -> std::path::PathBuf {
    let mut new_path = path.to_path_buf();
    new_path.set_extension(new_extension);
    new_path
}

#[test]
fn test_all() {
    // we are storing files in folder named `t` and not inside `tests`, because `cargo test`
    // re-compiles the crate and we don't want to recompile the crate for every test
    for path in find_all_files_matching_extension_recursively("t", "ftd") {
        println!("testing: {}", path.display());
        let s = std::fs::read_to_string(&path).unwrap();
        let t = std::fs::read_to_string(replace_extension(&path, "json")).unwrap();
        p(&s, serde_json::from_str(&t).unwrap());
    }
}

fn find_all_files_matching_extension_recursively(
    dir: impl AsRef<std::path::Path>,
    extension: &str,
) -> Vec<std::path::PathBuf> {
    let mut files = vec![];
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            files.extend(find_all_files_matching_extension_recursively(
                &path, extension,
            ));
        } else {
            match path.extension() {
                Some(ext) if ext == extension => files.push(path),
                _ => continue,
            }
        }
    }
    files
}

#[test]
fn sub_section() {
    p(
        "-- foo:\n\nhello world\n-- bar:\n\n-- end: foo",
        ftd::p11::Section::with_name("foo")
            .and_body("hello world")
            .add_sub_section(ftd::p11::Section::with_name("bar"))
            .list(),
    );

    p(
        indoc!(
            "
            -- foo:

            body ho

            -- dodo:

            -- end: foo


            -- bar:

            bar body
            "
        ),
        vec![
            ftd::p11::Section::with_name("foo")
                .and_body("body ho")
                .add_sub_section(ftd::p11::Section::with_name("dodo")),
            ftd::p11::Section::with_name("bar").and_body("bar body"),
        ],
    );

    p(
        indoc!(
            "
            -- foo:

            body ho


            -- bar:

            bar body

            -- dodo:

            -- end: bar
            "
        ),
        vec![
            ftd::p11::Section::with_name("foo").and_body("body ho"),
            ftd::p11::Section::with_name("bar")
                .and_body("bar body")
                .add_sub_section(ftd::p11::Section::with_name("dodo")),
        ],
    );

    p(
        indoc!(
            "
            -- foo:

            body ho


            -- bar:

            bar body

            -- dodo:
            -- rat:

            -- end: bar
            "
        ),
        vec![
            ftd::p11::Section::with_name("foo").and_body("body ho"),
            ftd::p11::Section::with_name("bar")
                .and_body("bar body")
                .add_sub_section(ftd::p11::Section::with_name("dodo"))
                .add_sub_section(ftd::p11::Section::with_name("rat")),
        ],
    );

    p(
        indoc!(
            "
            -- foo:

            body ho


            -- bar:

            -- bar.cat:

            bar body

            -- dodo:
            -- rat:

            -- end: bar
            "
        ),
        vec![
            ftd::p11::Section::with_name("foo").and_body("body ho"),
            ftd::p11::Section::with_name("bar")
                .add_header_str("cat", "bar body")
                .add_sub_section(ftd::p11::Section::with_name("dodo"))
                .add_sub_section(ftd::p11::Section::with_name("rat")),
        ],
    );

    p(
        indoc!(
            "
            -- foo:

            body ho

            -- bar:

            bar body

            -- dodo:

            hello

            -- end: bar
            "
        ),
        vec![
            ftd::p11::Section::with_name("foo").and_body("body ho"),
            ftd::p11::Section::with_name("bar")
                .and_body("bar body")
                .add_sub_section(ftd::p11::Section::with_name("dodo").and_body("hello")),
        ],
    );

    p(
        "-- foo:\n\nhello world\n-- bar:\n\n-- end: foo",
        ftd::p11::Section::with_name("foo")
            .and_body("hello world")
            .add_sub_section(ftd::p11::Section::with_name("bar"))
            .list(),
    );

    p(
        "-- foo:\n\nhello world\n-- bar: foo\n\n-- end: foo",
        ftd::p11::Section::with_name("foo")
            .and_body("hello world")
            .add_sub_section(ftd::p11::Section::with_name("bar").and_caption("foo"))
            .list(),
    );
}

#[test]
fn activity() {
    p(
        indoc!(
            "
            -- step:
            method: GET

            -- realm.rr.activity:
            okind:
            oid:
            ekind:

            null

            -- end: step

        "
        ),
        vec![ftd::p11::Section::with_name("step")
            .add_header_str("method", "GET")
            .add_sub_section(
                ftd::p11::Section::with_name("realm.rr.activity")
                    .add_header_str("okind", "")
                    .add_header_str("oid", "")
                    .add_header_str("ekind", "")
                    .and_body("null"),
            )],
    )
}

#[test]
fn escaping() {
    p(
        indoc!(
            "
            -- hello:

            \\-- yo: whats up?
            \\-- foo: bar
        "
        ),
        ftd::p11::Section::with_name("hello")
            .and_body("-- yo: whats up?\n-- foo: bar")
            .list(),
    )
}

#[test]
fn comments() {
    p(
        indoc!(
            "
            ;; yo
            -- foo:
            ;; yo
            key: value

            body ho
            ;; yo

            -- bar:
            ;; yo
            b: ba
            ;; yo

            bar body
            ;; yo
            -- dodo:
            ;; yo
            k: v
            ;; yo

            hello
            ;; yo
            -- end: bar
            "
        ),
        vec![
            ftd::p11::Section::with_name("foo")
                .and_body("body ho")
                .add_header_str("key", "value"),
            ftd::p11::Section::with_name("bar")
                .and_body("bar body")
                .add_header_str("b", "ba")
                .add_sub_section(
                    ftd::p11::Section::with_name("dodo")
                        .add_header_str("k", "v")
                        .and_body("hello"),
                ),
        ],
    );
}
#[test]
fn two() {
    p(
        indoc!(
            "
            -- foo:
            key: value

            body ho

            -- bar:
            b: ba

            bar body
            -- dodo:
            k: v

            hello
            -- end: bar
            "
        ),
        vec![
            ftd::p11::Section::with_name("foo")
                .and_body("body ho")
                .add_header_str("key", "value"),
            ftd::p11::Section::with_name("bar")
                .and_body("bar body")
                .add_header_str("b", "ba")
                .add_sub_section(
                    ftd::p11::Section::with_name("dodo")
                        .add_header_str("k", "v")
                        .and_body("hello"),
                ),
        ],
    );
}

#[test]
fn empty_key() {
    p(
        "-- foo:\nkey: \n",
        ftd::p11::Section::with_name("foo")
            .add_header_str("key", "")
            .list(),
    );

    p(
        "-- foo:\n-- bar:\nkey:\n\n\n-- end: foo",
        ftd::p11::Section::with_name("foo")
            .add_sub_section(ftd::p11::Section::with_name("bar").add_header_str("key", ""))
            .list(),
    )
}

#[test]
fn with_dash_dash() {
    p(
        indoc!(
            r#"
            -- hello:

            hello -- world: yo
        "#
        ),
        ftd::p11::Section::with_name("hello")
            .and_body("hello -- world: yo")
            .list(),
    );

    p(
        indoc!(
            r#"
            -- hello:

            -- realm.rr.step.body:

            {
              "body": "-- h0: Hello World\n\n-- markup:\n\ndemo cr 1\n",
              "kind": "content",
              "track": "amitu/index",
              "version": "2020-11-16T04:13:14.642892+00:00"
            }
            
            -- end: hello
        "#
        ),
        ftd::p11::Section::with_name("hello")
            .add_sub_section(
                ftd::p11::Section::with_name("realm.rr.step.body").and_body(&indoc!(
                    r#"
                        {
                          "body": "-- h0: Hello World\n\n-- markup:\n\ndemo cr 1\n",
                          "kind": "content",
                          "track": "amitu/index",
                          "version": "2020-11-16T04:13:14.642892+00:00"
                        }"#
                )),
            )
            .list(),
    );
}

#[test]
fn indented_body() {
    p(
        &indoc!(
            "
                 -- markup:

                 hello world is

                     not enough

                     lol
            "
        ),
        ftd::p11::Section::with_name("markup")
            .and_body("hello world is\n\n    not enough\n\n    lol")
            .list(),
    );
    p(
        indoc!(
            "
            -- foo:

              body ho

            yo

            -- bar:

                bar body

            "
        ),
        vec![
            ftd::p11::Section::with_name("foo").and_body("  body ho\n\nyo"),
            ftd::p11::Section::with_name("bar").and_body("    bar body"),
        ],
    );
}

#[test]
fn body_with_empty_lines() {
    p(
        indoc!(
            "
            -- foo:





            hello









            "
        ),
        vec![ftd::p11::Section::with_name("foo").and_body("hello")],
    );

    p(
        indoc!(
            "
            -- foo:
            -- bar:




            hello









            -- end: foo
            "
        ),
        vec![ftd::p11::Section::with_name("foo")
            .add_sub_section(ftd::p11::Section::with_name("bar").and_body("hello"))],
    );
}

#[test]
fn basic() {
    p(
        "-- foo: bar",
        ftd::p11::Section::with_name("foo")
            .and_caption("bar")
            .list(),
    );

    p("-- foo:", ftd::p11::Section::with_name("foo").list());

    p("-- foo: ", ftd::p11::Section::with_name("foo").list());

    p(
        "-- foo:\nkey: value",
        ftd::p11::Section::with_name("foo")
            .add_header_str("key", "value")
            .list(),
    );

    p(
        "-- foo:\nkey: value\nk2:v2",
        ftd::p11::Section::with_name("foo")
            .add_header_str("key", "value")
            .add_header_str("k2", "v2")
            .list(),
    );

    p(
        "-- foo:\n\nbody ho",
        ftd::p11::Section::with_name("foo")
            .and_body("body ho")
            .list(),
    );

    p(
        indoc!(
            "
            -- foo:

            body ho
            -- bar:

            bar body
            "
        ),
        vec![
            ftd::p11::Section::with_name("foo").and_body("body ho"),
            ftd::p11::Section::with_name("bar").and_body("bar body"),
        ],
    );

    p(
        indoc!(
            "
            -- foo:

            body ho

            yo

            -- bar:

            bar body

            "
        ),
        vec![
            ftd::p11::Section::with_name("foo").and_body("body ho\n\nyo"),
            ftd::p11::Section::with_name("bar").and_body("bar body"),
        ],
    );

    p(
        indoc!(
            "
            -- foo:

            hello
            "
        ),
        vec![ftd::p11::Section::with_name("foo").and_body("hello")],
    );

    f("invalid", "foo:1 -> SectionNotFound")
}

#[test]
fn strict_body() {
    // section body without headers
    f(
        indoc!(
            "-- some-section:
                This is body
                "
        ),
        "foo:2 -> start section body 'This is body' after a newline!!",
    );

    // section body with headers
    f(
        indoc!(
            "-- some-section:
                h1: v1
                This is body
                "
        ),
        "foo:3 -> start section body 'This is body' after a newline!!",
    );

    // subsection body without headers
    f(
        indoc!(
            "-- some-section:
                h1: val

                -- some-sub-section:
                This is body

                -- end: some-section
                "
        ),
        "foo:5 -> start section body 'This is body' after a newline!!",
    );

    // subsection body with headers
    f(
        indoc!(
            "-- some-section:
                h1: val

                -- some-sub-section:
                h2: val
                h3: val
                This is body

                -- end: some-section
                "
        ),
        "foo:7 -> start section body 'This is body' after a newline!!",
    );
}

#[test]
fn header_section() {
    p(
        indoc!(
            "
            -- foo:

            -- foo.bar:

            -- section:
            k1: v1

            -- section.k2:

            This is value of section k2

            -- end: foo.bar

            -- foo.body:

            bar body
            "
        ),
        ftd::p11::Section::with_name("foo")
            .and_body("bar body")
            .add_header_section(
                "bar",
                None,
                ftd::p11::Section::with_name("section")
                    .add_header_str("k1", "v1")
                    .add_header_str("k2", "This is value of section k2")
                    .list(),
            )
            .list(),
    );
}

#[test]
fn kind() {
    p(
        indoc!(
            "
            -- moo foo:

            -- too foo.bar:

            -- section:
            k1: v1

            -- section.k2:

            This is value of section k2

            -- end: foo.bar

            -- foo.body:

            bar body

            -- foo.caption:

            bar caption

            -- subsection:

            -- sub-subsection:
            
            This is sub-subsection

            -- end: subsection

            -- end: foo
            "
        ),
        ftd::p11::Section::with_name("foo")
            .kind("moo")
            .and_body("bar body")
            .and_caption("bar caption")
            .add_header_section(
                "bar",
                Some("too".to_string()),
                ftd::p11::Section::with_name("section")
                    .add_header_str("k1", "v1")
                    .add_header_str("k2", "This is value of section k2")
                    .list(),
            )
            .add_sub_section(ftd::p11::Section::with_name("subsection").add_sub_section(
                ftd::p11::Section::with_name("sub-subsection").and_body("This is sub-subsection"),
            ))
            .list(),
    );

    p(
        indoc!(
            "
            -- moo foo:

            -- foo.caption:

            bar caption

            -- too foo.bar:

            -- section:
            k1: v1

            -- section.k2:

            This is value of section k2

            -- end: foo.bar

            -- foo.body:

            bar body

            -- subsection:

            -- sub-subsection:
            
            This is sub-subsection

            -- end: subsection

            -- end: foo
            "
        ),
        ftd::p11::Section::with_name("foo")
            .kind("moo")
            .and_body("bar body")
            .and_caption("bar caption")
            .add_header_section(
                "bar",
                Some("too".to_string()),
                ftd::p11::Section::with_name("section")
                    .add_header_str("k1", "v1")
                    .add_header_str("k2", "This is value of section k2")
                    .list(),
            )
            .add_sub_section(ftd::p11::Section::with_name("subsection").add_sub_section(
                ftd::p11::Section::with_name("sub-subsection").and_body("This is sub-subsection"),
            ))
            .list(),
    );

    p(
        indoc!(
            "
            -- moo foo:

            -- foo.caption:

            bar caption

            -- foo.body:

            bar body

            -- too foo.bar:

            -- section:
            k1: v1

            -- section.k2:

            This is value of section k2

            -- end: foo.bar


            -- subsection:

            -- sub-subsection:
            
            This is sub-subsection

            -- end: subsection

            -- end: foo
            "
        ),
        ftd::p11::Section::with_name("foo")
            .kind("moo")
            .and_body("bar body")
            .and_caption("bar caption")
            .add_header_section(
                "bar",
                Some("too".to_string()),
                ftd::p11::Section::with_name("section")
                    .add_header_str("k1", "v1")
                    .add_header_str("k2", "This is value of section k2")
                    .list(),
            )
            .add_sub_section(ftd::p11::Section::with_name("subsection").add_sub_section(
                ftd::p11::Section::with_name("sub-subsection").and_body("This is sub-subsection"),
            ))
            .list(),
    );
}
