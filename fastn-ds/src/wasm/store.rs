pub struct Store {
    pub req: ft_sys_shared::Request,
    pub ud: Option<ft_sys_shared::UserData>,
    pub clients: std::sync::Arc<async_lock::Mutex<Vec<Conn>>>,
    pub pg_pools: actix_web::web::Data<scc::HashMap<String, deadpool_postgres::Pool>>,
    pub sqlite: Option<std::sync::Arc<async_lock::Mutex<rusqlite::Connection>>>,
    pub response: Option<Response>,
    pub db_url: String,

    table: wasmtime::component::ResourceTable,
    ctx: wasmtime_wasi::WasiCtx,
}

impl wasmtime_wasi::WasiView for Store {
    fn table(&mut self) -> &mut wasmtime::component::ResourceTable {
        &mut self.table
    }
    fn ctx(&mut self) -> &mut wasmtime_wasi::WasiCtx {
        &mut self.ctx
    }
}

#[derive(Debug)]
pub enum Response {
    /// When wasm worker sent HTTP response.
    Http(ft_sys_shared::Request),
    /// When wasm worker asked to render and parse a string.
    Ftd(FtdResponse),
}

#[derive(serde::Deserialize, Debug)]
pub struct FtdResponse {
    /// This is the ID of the file, relative to the package in which wasm worker
    /// is present.
    pub ftd: String,
    /// The request data processor will have access to this data as well, so wasm
    /// worker can put some data in it and ftd file can read it back.
    pub request_data: serde_json::Value,
}

pub struct Conn {
    pub client: deadpool::managed::Object<deadpool_postgres::Manager>,
}

impl Store {
    pub fn new(
        req: ft_sys_shared::Request,
        ud: Option<ft_sys_shared::UserData>,
        pg_pools: actix_web::web::Data<scc::HashMap<String, deadpool_postgres::Pool>>,
        db_url: String,
    ) -> Store {
        let mut builder = wasmtime_wasi::WasiCtxBuilder::new();

        Self {
            req,
            ud,
            response: None,
            clients: Default::default(),
            pg_pools,
            db_url,
            sqlite: None,

            ctx: builder.build(),
            table: Default::default(),
        }
    }
}
