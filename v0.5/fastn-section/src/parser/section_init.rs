pub fn section_init(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::SectionInit> {
    scanner.skip_spaces();
    let dashdash = scanner.token("--")?;
    scanner.skip_spaces();
    let name = fastn_section::kinded_name(scanner)?;
    scanner.skip_spaces();
    let colon = scanner.token(":")?;
    Some(fastn_section::SectionInit {
        dashdash,
        name,
        colon,
    })
}

#[cfg(test)]
mod test {
    fastn_section::tt!(super::section_init);

    #[test]
    fn section_init() {
        t!("-- foo:", {"name": {"name": "foo"}});
        t!("-- foo: ", {"name": {"name": "foo"}}, " ");
        t!("-- foo: hello", {"name": {"name": "foo"}}, " hello");
        t!("-- integer foo: hello", {"name": {"name": "foo", "kind": "integer"}}, " hello");
        t!("-- integer héllo: foo", {"name": {"name": "héllo", "kind": "integer"}}, " foo");
        // t!("-- list<integer> foo:", {"name": {"name": "foo", "kind": "integer"}}, "");
    }
}
