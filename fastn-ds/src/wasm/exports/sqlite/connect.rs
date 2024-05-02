pub async fn connect(
    mut caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let db_url = fastn_ds::wasm::helpers::get_str(ptr, len, &mut caller)?;
    println!("sqlite_connect: {db_url}");

    caller.data_mut().sqlite_connect(db_url.as_str()).await
}

impl fastn_ds::wasm::Store {
    pub async fn sqlite_connect(&mut self, db_url: &str) -> wasmtime::Result<i32> {
        let db = rusqlite::Connection::open(db_url)?;
        self.sqlite = Some(std::sync::Arc::new(async_lock::Mutex::new(db)));
        Ok(0)
    }
}
