// # note on error handling.
//
// we can handle error in this parser in such a way that our rest of parsers that come after,
// like router parser, and the actual compiler, can run even if there are some issues (error, not
// warning) encountered in this phase.
//
// but for simplicity’s sake, we are going to not do that now, and return either a package object
// if there are no errors (maybe warnings).
//
// even within this parser, we bail early if any one FASTN.ftd is found with errors in it, this is
// kind of acceptable as all FASTN.ftd files, other the one in the current package, must not have
// errors because they are published dependencies.
//
// though this is not strictly true, say if one of the dependencies needed some system, and main
// package has not provided it, then dependency can also have errors.

#[derive(Debug, Default)]
pub struct State {
    name: fastn_package::UR<(), String>,
    systems: Vec<fastn_package::UR<String, fastn_package::System>>,
    dependencies: Vec<fastn_package::UR<String, fastn_package::Dependency>>,
    pub auto_imports: Vec<fastn_package::AutoImport>,
    apps: Vec<fastn_package::UR<String, fastn_package::App>>,
    packages: std::collections::HashMap<String, fastn_package::Package>,
    pub diagnostics: Vec<fastn_section::Spanned<fastn_section::Diagnostic>>,
    // if both a/FASTN.ftd and b/FASTN.ftd need x/FASTN.ftd, this will contain x => [a, b].
    // this will reset on every "continue after".
    pub waiting_for: std::collections::HashMap<String, Vec<String>>,
}

type PResult<T> = std::result::Result<
    (T, Vec<fastn_section::Spanned<fastn_section::Warning>>),
    Vec<fastn_section::Spanned<fastn_section::Diagnostic>>,
>;
type NResult = Result<(fastn_section::Document, Vec<String>), std::sync::Arc<std::io::Error>>;

impl fastn_package::Package {
    pub fn reader() -> fastn_continuation::Result<State> {
        fastn_continuation::Result::Stuck(Default::default(), vec!["FASTN.ftd".to_string()])
    }
}

impl fastn_continuation::Continuation for State {
    // we return a package object if we parsed, even a partial package.
    type Output = PResult<fastn_package::MainPackage>;
    type Needed = Vec<String>; // vec of file names
    type Found = Vec<(Option<String>, NResult)>;

    fn continue_after(
        mut self,
        n: Vec<(Option<String>, NResult)>,
    ) -> fastn_continuation::Result<Self> {
        match self.name {
            // if the name is not resolved means this is the first attempt.
            fastn_package::UR::UnResolved(()) => {
                assert_eq!(n.len(), 1);
                assert_eq!(n[0].0, None);
                assert!(self.waiting_for.is_empty());

                match n.into_iter().next() {
                    Some((_name, Ok((doc, file_list)))) => {
                        let _package = match parse_package(doc, file_list) {
                            Ok((package, warnings)) => {
                                self.diagnostics.extend(
                                    warnings
                                        .into_iter()
                                        .map(|v| v.map(fastn_section::Diagnostic::Warning)),
                                );
                                package
                            }
                            Err(diagnostics) => {
                                self.diagnostics.extend(diagnostics);
                                return fastn_continuation::Result::Done(Err(self.diagnostics));
                            }
                        };
                        // self.name = fastn_package::UR::Resolved();
                        todo!()
                    }
                    Some((_name, Err(_))) => {
                        // Ok(None) means we failed to find a file named FASTN.ftd.
                        // Err(e) means we failed to parse the content of FASTN.ftd.
                        todo!()
                    }
                    None => unreachable!("we did a check for this already, list has 1 element"),
                }
            }
            // even if we failed to find name, we still continue to process as many dependencies,
            // etc. as possible.
            // so this case handles both name found and name error cases.
            _ => {
                todo!()
            }
        }
    }
}

fn parse_package(
    doc: fastn_section::Document,
    file_list: Vec<String>,
) -> PResult<fastn_package::Package> {
    let warnings = vec![];
    let mut errors = vec![];

    let mut package = fastn_package::Package {
        name: "".to_string(),
        dependencies: vec![],
        auto_imports: vec![],
        favicon: None,
        urls: vec![],
        file_list,
    };

    for section in doc.sections.iter() {
        let span = section.span();

        match section.simple_name() {
            Some("package") => {
                match section.simple_caption() {
                    Some(name) => package.name = name.to_string(),
                    None => {
                        // we do not bail at this point,
                        // missing package name is just a warning for now
                        errors.push(span.wrap(fastn_section::Error::PackageNameNotInCaption));
                    }
                };

                for header in section.headers.iter() {
                    match header.name() {
                        "favicon" => match header.simple_value() {
                            Some(v) => package.favicon = Some(v.to_string()),
                            None => errors.push(
                                header
                                    .name_span()
                                    .wrap(fastn_section::Error::ArgumentValueRequired),
                            ),
                        },
                        // TODO: default-language, language-<code> etc
                        _ => errors.push(
                            header
                                .name_span()
                                .wrap(fastn_section::Error::ExtraArgumentFound),
                        ),
                    }
                }
            }
            Some("dependency") => {
                match section.simple_caption() {
                    Some(name) => {
                        package.dependencies.push(fastn_package::Dependency {
                            name: name.to_string(),
                            capabilities: vec![],
                            dependencies: vec![],
                            auto_imports: vec![],
                        });
                    }
                    None => {
                        // we do not bail at this point,
                        // missing package name is just a warning for now
                        errors.push(span.wrap(fastn_section::Error::PackageNameNotInCaption));
                    }
                };

                for header in section.headers.iter() {
                    header
                        .name_span()
                        .wrap(fastn_section::Error::ExtraArgumentFound);
                }
            }
            Some("auto-import") => {
                todo!()
            }
            Some("app") => {
                todo!()
            }
            Some("urls") => {
                todo!()
            }
            Some(_t) => {
                errors.push(
                    section
                        .span()
                        .wrap(fastn_section::Error::UnexpectedSectionInPackageFile),
                );
            }
            None => {
                // we found a section without name.
                // this is an error that Document must already have collected it, nothing to do.
                return Err(doc.diagnostics());
            }
        }
    }

    if package.name.is_empty()
        && !errors
            .iter()
            .any(|v| v.value == fastn_section::Error::PackageDeclarationMissing)
    {
        // we do not bail at this point, missing package name is just a warning for now
        errors.push(
            fastn_section::Span::default().wrap(fastn_section::Error::PackageDeclarationMissing),
        );
    }

    Ok((package, warnings))
}
