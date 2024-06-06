#![warn(unused_extern_crates)]
#![deny(unused_crate_dependencies)]

pub mod sql;

pub async fn handle<S: Send>(
    mut wasm_store: wasmtime::Store<S>,
    module: wasmtime::Module,
    linker: wasmtime::Linker<S>,
    path: String,
) -> wasmtime::Result<(wasmtime::Store<S>, Option<ft_sys_shared::Request>)> {
    let instance = match linker.instantiate_async(&mut wasm_store, &module).await {
        Ok(i) => i,
        Err(e) => {
            return Ok((
                wasm_store,
                Some(ft_sys_shared::Request::server_error(format!(
                    "failed to instantiate wasm module: {e:?}"
                ))),
            ));
        }
    };

    let (mut wasm_store, main) = get_entrypoint(instance, wasm_store, path.as_str());

    let main = match main {
        Ok(v) => v,
        Err(e) => {
            return Ok((
                wasm_store,
                Some(ft_sys_shared::Request {
                    uri: "server-error".to_string(),
                    method: "404".to_string(),
                    headers: vec![],
                    body: format!("no endpoint found for {path}: {e:?}").into_bytes(),
                }),
            ));
        }
    };
    main.call_async(&mut wasm_store, ()).await?;

    Ok((wasm_store, None))
}

pub fn get_entrypoint<S: Send>(
    instance: wasmtime::Instance,
    mut store: wasmtime::Store<S>,
    path: &str,
) -> (
    wasmtime::Store<S>,
    wasmtime::Result<wasmtime::TypedFunc<(), ()>>,
) {
    if let Ok(f) = instance.get_typed_func::<(), ()>(&mut store, "main_ft") {
        return (store, Ok(f));
    }

    let entrypoint = match path_to_entrypoint(path) {
        Ok(v) => v,
        Err(e) => return (store, Err(e)),
    };

    println!("main_ft not found, trying {entrypoint}");

    let r = instance.get_typed_func(&mut store, entrypoint.as_str());
    (store, r)
}

#[derive(Debug, thiserror::Error)]
pub enum PathToEndpointError {
    #[error("no wasm file found in path")]
    NoWasm,
}

#[derive(Debug, thiserror::Error)]
pub enum WasmError {
    #[error("endpoint did not return response")]
    EndpointDidNotReturnResponse,
}

pub fn path_to_entrypoint(path: &str) -> wasmtime::Result<String> {
    let path = path.split_once('?').map(|(f, _)| f).unwrap_or(path);
    match path.split_once(".wasm/") {
        Some((_, l)) => {
            let l = l.trim_end_matches('/').replace('/', "_");
            Ok(l.trim_end_matches('/').replace('-', "_") + "__entrypoint")
        }
        None => Err(PathToEndpointError::NoWasm.into()),
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SqlError {
    #[error("connection error {0}")]
    Connection(rusqlite::Error),
    #[error("Query error {0}")]
    Query(rusqlite::Error),
    #[error("Execute error {0}")]
    Execute(rusqlite::Error),
    #[error("column error {0}: {0}")]
    Column(usize, rusqlite::Error),
    #[error("row error {0}")]
    Row(rusqlite::Error),
    #[error("found blob")]
    FoundBlob,
    #[error("unknown db error")]
    UnknownDB,
}

pub fn rows_to_json(
    mut rows: rusqlite::Rows,
    count: usize,
) -> Result<Vec<Vec<serde_json::Value>>, SqlError> {
    let mut result: Vec<Vec<serde_json::Value>> = vec![];
    loop {
        match rows.next() {
            Ok(None) => break,
            Ok(Some(r)) => {
                result.push(row_to_json(r, count)?);
            }
            Err(e) => return Err(SqlError::Row(e)),
        }
    }
    Ok(result)
}

pub fn row_to_json(r: &rusqlite::Row, count: usize) -> Result<Vec<serde_json::Value>, SqlError> {
    let mut row: Vec<serde_json::Value> = Vec::with_capacity(count);
    for i in 0..count {
        match r.get::<usize, rusqlite::types::Value>(i) {
            Ok(rusqlite::types::Value::Null) => row.push(serde_json::Value::Null),
            Ok(rusqlite::types::Value::Integer(i)) => row.push(serde_json::Value::Number(i.into())),
            Ok(rusqlite::types::Value::Real(i)) => row.push(serde_json::Value::Number(
                serde_json::Number::from_f64(i).unwrap(),
            )),
            Ok(rusqlite::types::Value::Text(i)) => row.push(serde_json::Value::String(i)),
            Ok(rusqlite::types::Value::Blob(_)) => return Err(SqlError::FoundBlob),
            Err(e) => return Err(SqlError::Column(i, e)),
        }
    }
    Ok(row)
}

pub const FASTN_MOUNTPOINT: &str = "x-fastn-mountpoint";
