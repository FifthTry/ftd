async fn create_pool() -> Result<deadpool_postgres::Pool, deadpool_postgres::CreatePoolError> {
    let mut cfg = deadpool_postgres::Config::new();
    cfg.libpq_style_connection_string = Some(std::env::var("FASTN_PG_URL").unwrap());
    cfg.manager = Some(deadpool_postgres::ManagerConfig {
        recycling_method: deadpool_postgres::RecyclingMethod::Verified,
    });
    let runtime = Some(deadpool_postgres::Runtime::Tokio1);
    match std::env::var("FASTN_PG_CERTIFICATE") {
        Ok(cert) => {
            let cert = tokio::fs::read(cert).await.unwrap();
            let cert = native_tls::Certificate::from_pem(&cert).unwrap();
            let connector = native_tls::TlsConnector::builder()
                .add_root_certificate(cert)
                .build()
                .unwrap();
            let tls = postgres_native_tls::MakeTlsConnector::new(connector);
            cfg.create_pool(runtime, tls)
        }
        _ => cfg.create_pool(runtime, tokio_postgres::NoTls),
    }
}

// TODO: I am a little confused about the use of `tokio::sync` here, both sides are async, so why
//       do we need to use `tokio::sync`? Am I doing something wrong? How do I prove/verify that
//       this is correct?
static POOL_RESULT: tokio::sync::OnceCell<
    Result<deadpool_postgres::Pool, deadpool_postgres::CreatePoolError>,
> = tokio::sync::OnceCell::const_new();

async fn pool() -> &'static Result<deadpool_postgres::Pool, deadpool_postgres::CreatePoolError> {
    POOL_RESULT.get_or_init(create_pool).await
}

pub async fn process(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let (headers, query) = super::sqlite::get_p1_data("pg", &value, doc.name)?;

    super::sqlite::result_to_value(
        execute_query(query.as_str(), doc, value.line_number(), headers).await?,
        kind,
        doc,
        &value,
    )
}

type PGData = dyn postgres_types::ToSql + Sync;

struct QueryArgs {
    args: Vec<Box<PGData>>,
}

impl QueryArgs {
    fn pg_args(&self) -> Vec<&PGData> {
        self.args.iter().map(|x| x.as_ref()).collect()
    }
}

fn resolve_variable_from_doc(
    doc: &ftd::interpreter::TDoc<'_>,
    var: &str,
    e: &postgres_types::Type,
    line_number: usize,
) -> ftd::interpreter::Result<Box<PGData>> {
    let thing = match doc.get_thing(var, line_number) {
        Ok(ftd::interpreter::Thing::Variable(v)) => v.value.resolve(doc, line_number)?,
        Ok(v) => {
            return ftd::interpreter::utils::e2(
                format!("{var} is not a variable, it's a {v:?}"),
                doc.name,
                line_number,
            )
        }
        Err(e) => {
            return ftd::interpreter::utils::e2(
                format!("${var} not found in the document: {e:?}"),
                doc.name,
                line_number,
            )
        }
    };

    Ok(match (e, thing) {
        (&postgres_types::Type::TEXT, ftd::interpreter::Value::String { text, .. }) => {
            Box::new(text)
        }
        (&postgres_types::Type::INT4, ftd::interpreter::Value::Integer { value, .. }) => {
            Box::new(value as i32)
        }
        (&postgres_types::Type::INT8, ftd::interpreter::Value::Integer { value, .. }) => {
            Box::new(value)
        }
        (&postgres_types::Type::FLOAT4, ftd::interpreter::Value::Decimal { value, .. }) => {
            Box::new(value as f32)
        }
        (&postgres_types::Type::FLOAT8, ftd::interpreter::Value::Decimal { value, .. }) => {
            Box::new(value)
        }
        (&postgres_types::Type::BOOL, ftd::interpreter::Value::Boolean { value, .. }) => {
            Box::new(value)
        }
        (e, a) => {
            return ftd::interpreter::utils::e2(
                format!("for {} postgresql expected ${:?}, found {:?}", var, e, a),
                doc.name,
                line_number,
            )
        }
    })
}

fn resolve_variable_from_headers(
    doc: &ftd::interpreter::TDoc<'_>,
    headers: &ftd::ast::HeaderValues,
    var: &str,
    e: &postgres_types::Type,
    doc_name: &str,
    line_number: usize,
) -> ftd::interpreter::Result<Option<Box<PGData>>> {
    let header = match headers.optional_header_by_name(var, doc_name, line_number)? {
        Some(v) => v,
        None => return Ok(None),
    };

    if let ftd::ast::VariableValue::String { value, .. } = &header.value {
        if let Some(stripped) = value.strip_prefix('$') {
            return resolve_variable_from_doc(doc, stripped, e, line_number).map(Some);
        }
    }

    fn friendlier_error<T, E: ToString>(
        r: Result<T, E>,
        var: &str,
        val: &str,
        into: &str,
        doc_name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<T> {
        match r {
            Ok(r) => Ok(r),
            Err(e) => ftd::interpreter::utils::e2(
                format!(
                    "failed to parse `{var}: {val}` into {into}: {e}",
                    e = e.to_string()
                ),
                doc_name,
                line_number,
            ),
        }
    }

    Ok(match (e, &header.value) {
        (&postgres_types::Type::TEXT, ftd::ast::VariableValue::String { value, .. }) => {
            Some(Box::new(value.to_string()))
        }
        (&postgres_types::Type::INT4, ftd::ast::VariableValue::String { value, .. }) => {
            Some(Box::new(friendlier_error(
                value.parse::<i32>(),
                var,
                value,
                "i32",
                doc_name,
                line_number,
            )?))
        }
        (&postgres_types::Type::INT8, ftd::ast::VariableValue::String { value, .. }) => {
            Some(Box::new(friendlier_error(
                value.parse::<i64>(),
                var,
                value,
                "i64",
                doc_name,
                line_number,
            )?))
        }
        (&postgres_types::Type::FLOAT4, ftd::ast::VariableValue::String { value, .. }) => {
            Some(Box::new(friendlier_error(
                value.parse::<f32>(),
                var,
                value,
                "f32",
                doc_name,
                line_number,
            )?))
        }
        (&postgres_types::Type::FLOAT8, ftd::ast::VariableValue::String { value, .. }) => {
            Some(Box::new(friendlier_error(
                value.parse::<f64>(),
                var,
                value,
                "f64",
                doc_name,
                line_number,
            )?))
        }
        (&postgres_types::Type::BOOL, ftd::ast::VariableValue::String { value, .. }) => {
            Some(Box::new(friendlier_error(
                value.parse::<bool>(),
                var,
                value,
                "bool",
                doc_name,
                line_number,
            )?))
        }
        (e, a) => {
            return ftd::interpreter::utils::e2(
                format!("for {} postgresql expected ${:?}, found {:?}", var, e, a),
                doc.name,
                line_number,
            )
        }
    })
}

fn prepare_args(
    query_args: Vec<String>,
    expected_args: &[postgres_types::Type],
    doc: &ftd::interpreter::TDoc<'_>,
    line_number: usize,
    headers: ftd::ast::HeaderValues,
) -> ftd::interpreter::Result<QueryArgs> {
    if expected_args.len() != query_args.len() {
        return ftd::interpreter::utils::e2(
            format!(
                "expected {} arguments, found {}",
                expected_args.len(),
                query_args.len()
            ),
            doc.name,
            line_number,
        );
    }
    let mut args = vec![];
    for (e, a) in expected_args.iter().zip(query_args) {
        args.push(
            match resolve_variable_from_headers(doc, &headers, &a, e, doc.name, line_number)? {
                Some(v) => v,
                None => resolve_variable_from_doc(doc, &a, e, line_number)?,
            },
        );
    }
    Ok(QueryArgs { args })
}

async fn execute_query(
    query: &str,
    doc: &ftd::interpreter::TDoc<'_>,
    line_number: usize,
    headers: ftd::ast::HeaderValues,
) -> ftd::interpreter::Result<Vec<Vec<serde_json::Value>>> {
    let (query, query_args) = super::sql::extract_arguments(query)?;
    let client = pool().await.as_ref().unwrap().get().await.unwrap();

    let stmt = client.prepare_cached(query.as_str()).await.unwrap();
    let args = prepare_args(query_args, stmt.params(), doc, line_number, headers)?;

    let rows = client.query(&stmt, &args.pg_args()).await.unwrap();
    let mut result: Vec<Vec<serde_json::Value>> = vec![];

    for r in rows {
        result.push(row_to_json(r, doc.name, line_number)?)
    }

    Ok(result)
}

fn row_to_json(
    r: tokio_postgres::Row,
    doc_name: &str,
    line_number: usize,
) -> ftd::interpreter::Result<Vec<serde_json::Value>> {
    let columns = r.columns();
    let mut row: Vec<serde_json::Value> = Vec::with_capacity(columns.len());
    for (i, column) in columns.iter().enumerate() {
        match column.type_() {
            &postgres_types::Type::BOOL => row.push(serde_json::Value::Bool(r.get(i))),
            &postgres_types::Type::INT2 => {
                row.push(serde_json::Value::Number(r.get::<usize, i16>(i).into()))
            }
            &postgres_types::Type::INT4 => {
                row.push(serde_json::Value::Number(r.get::<usize, i32>(i).into()))
            }
            &postgres_types::Type::INT8 => {
                row.push(serde_json::Value::Number(r.get::<usize, i64>(i).into()))
            }
            &postgres_types::Type::FLOAT4 => row.push(serde_json::Value::Number(
                serde_json::Number::from_f64(r.get::<usize, f32>(i) as f64).unwrap(),
            )),
            &postgres_types::Type::FLOAT8 => row.push(serde_json::Value::Number(
                serde_json::Number::from_f64(r.get::<usize, f64>(i)).unwrap(),
            )),
            &postgres_types::Type::TEXT => row.push(serde_json::Value::String(r.get(i))),
            &postgres_types::Type::CHAR => row.push(serde_json::Value::String(r.get(i))),
            &postgres_types::Type::VARCHAR => row.push(serde_json::Value::String(r.get(i))),
            &postgres_types::Type::JSON => row.push(r.get(i)),

            t => {
                return ftd::interpreter::utils::e2(
                    format!("type {} not yet supported", t),
                    doc_name,
                    line_number,
                )
            }
        }
    }

    Ok(row)
}

/*
FASTN_PG_URL=postgres://amitu@localhost/amitu fastn serve
 */

/*
CREATE TABLE users (
    id SERIAL,
    name TEXT,
    department TEXT
);

INSERT INTO "users" (name, department) VALUES ('jack', 'design');
INSERT INTO "users" (name, department) VALUES ('jill', 'engineering');

 */

/*
-- import: fastn/processors as pr

-- record person:
integer id:
string name:
string department:


-- integer id: 1

-- ftd.integer: $id

-- person list people:
$processor$: pr.pg

SELECT * FROM "users" where id >= $id ;


-- ftd.text: data from db

-- ftd.text: $p.name
$loop$: $people as $p



-- integer int_2:
$processor$: pr.pg

SELECT 20::int2;

-- ftd.integer: $int_2

-- integer int_4:
$processor$: pr.pg

SELECT 40::int4;

-- ftd.integer: $int_4

-- integer int_8:
$processor$: pr.pg

SELECT 80::int8;

-- ftd.integer: $int_8






-- decimal d_4:
$processor$: pr.pg
val: 4.01
note: `SELECT $val::FLOAT8` should work but doesn't

SELECT 1.0::FLOAT8;

-- ftd.decimal: $d_4


-- decimal d_8:
$processor$: pr.pg

SELECT 80.0::FLOAT8;

-- ftd.decimal: $d_8
*/

/*
PREPARE my_query AS
SELECT * FROM "users" where id >= $1;
SELECT parameter_types FROM pg_prepared_statements WHERE name = 'my_query';
DEALLOCATE my_query;
 */
