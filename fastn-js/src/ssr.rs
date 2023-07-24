pub fn ssr_str(js: &str) -> String {
    let all_js = fastn_js::all_js_with_test();
    let js = format!("{all_js}{js}");

    dbg!(&js);

    #[cfg(target_os = "windows")]
    {
        rquickjs::Context::full(&rquickjs::Runtime::new().unwrap())
            .unwrap()
            .with(|ctx| ctx.eval::<String, _>(js).unwrap())
    }
    #[cfg(not(target_os = "windows"))]
    {
        // Added logging support from console from within context
        let context = quick_js::Context::builder()
            .console(
                |level: quick_js::console::Level, args: Vec<quick_js::JsValue>| {
                    eprintln!("{}: {:?}", level, args);
                },
            )
            .build()
            .unwrap();
        context.eval_as::<String>(js.as_str()).unwrap()
    }
}

pub fn ssr(ast: &[fastn_js::Ast]) -> String {
    let js = format!("{}\nfastn_virtual.ssr(main)", fastn_js::to_js(ast, false));
    ssr_str(&js)
}

pub fn ssr_with_js_string(js: &str) -> String {
    let js = format!("{}\nfastn_virtual.ssr(main)", js);
    ssr_str(&js)
}
