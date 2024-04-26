pub async fn execute(
    mut caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let q: fastn_ds::wasm::exports::sqlite::Query =
        fastn_ds::wasm::helpers::get_json(ptr, len, &mut caller).await?;
    let res = caller.data_mut().sqlite_execute(q).await?;
    fastn_ds::wasm::helpers::send_json(res, &mut caller).await
}

impl fastn_ds::wasm::Store {
    async fn sqlite_execute(
        &mut self,
        q: fastn_ds::wasm::exports::sqlite::Query,
    ) -> wasmtime::Result<Result<usize, ft_sys_shared::DbError>> {
        let conn = if let Some(ref mut conn) = self.sqlite {
            conn
        } else {
            todo!()
        };

        let conn = conn.lock().await;
        Ok(
            match conn.execute(q.sql.as_str(), rusqlite::params_from_iter(q.binds)) {
                Ok(u) => Ok(u),
                Err(_) => todo!(),
            },
        )
    }
}
