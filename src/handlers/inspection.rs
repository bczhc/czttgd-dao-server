use anyhow::anyhow;
use axum::{Extension, Form};
use axum::extract::{Path, Query};
use axum::response::IntoResponse;
use futures::TryStreamExt;
use log::debug;
use serde::{Deserialize, Serialize};
use sqlx::{Database, Executor, MySql, Row};
use sqlx::database::HasArguments;
use sqlx::types::BigDecimal;
use yeet_ops::yeet;

use crate::{api_error, api_ok, ApiContext};
use crate::handlers::{InspectionDetails, InspectionForm};

use sqlx::mysql::MySqlArguments;


const FORM_INSERT_SQL: &str = r"INSERT INTO tt_inspect
(creator,
 devicecode,
 creationtime,
 spec,
 wirespeed,
 wirenum,
 breakspec,
 twbatchcode,
 trbatchcode,
 dlwarehouse,
 breakflag,
 breakpointa,
 breakpointb,
 memo,
 devicecategory,
 billflag)
    VALUE (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0)";

const FORM_UPDATE_SQL: &str = r"UPDATE tt_inspect
SET creator        = ?,
    devicecode     = ?,
    creationtime   = ?,
    spec           = ?,
    wirespeed      = ?,
    wirenum        = ?,
    breakspec      = ?,
    twbatchcode    = ?,
    trbatchcode    = ?,
    dlwarehouse    = ?,
    breakflag      = ?,
    breakpointa    = ?,
    breakpointb    = ?,
    memo           = ?,
    devicecategory = ?,
    billflag       = 0
WHERE id = ?";

macro handle_errors($r:expr) {{
    debug!("Result: {:?}", &$r);
    let err = $r.err().unwrap();
    api_error!(format!("{}", err))
}}

#[axum::debug_handler]
pub async fn post_new(
    Extension(api_context): Extension<ApiContext>,
    Form(form): Form<InspectionForm>,
) -> impl IntoResponse {
    debug!("post_new form: {:?}", form);
    let db = &api_context.db;

    let result: anyhow::Result<()> = try {
        let query = sqlx::query(FORM_INSERT_SQL);
        let query = bind_form(query, form);
        db.execute(query).await?;
        return api_ok!(());
    };
    handle_errors!(result)
}

fn bind_form(
    query: sqlx::query::Query<MySql, MySqlArguments>,
    form: InspectionForm,
) -> sqlx::query::Query<MySql, MySqlArguments>
{
    let break_flag = form.break_type == 0;
    let position_b = form
        .break_position_b
        .as_ref()
        .map(|x| x.parse::<BigDecimal>().unwrap());

    return query
        .bind(form.creator)
        .bind(form.machine_number)
        .bind(form.creation_time)
        .bind(form.product_specs)
        .bind(form.wire_speed)
        .bind(form.wire_number)
        .bind(form.break_specs)
        .bind(form.copper_wire_no)
        .bind(form.copper_stick_no)
        .bind(form.repo_no)
        .bind(break_flag)
        .bind(form.break_position_a)
        .bind(position_b)
        .bind(form.comments)
        .bind(form.machine_category);
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InspectionSummary {
    id: u32,
    machine_number: u32,
    cause: Option<String>,
    break_spec: String,
    product_spec: Option<String>,
    creator: String,
    creation_time: String,
    /// 0: 已初检 1: 已终检
    checking_state: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QuerySummaryQuery {
    filter: String,
    stage: u32,
}

#[axum::debug_handler]
pub async fn query_summary(
    Extension(api_context): Extension<ApiContext>,
    Query(api_query): Query<QuerySummaryQuery>,
) -> impl IntoResponse {
    let db = &api_context.db;

    let r: anyhow::Result<()> = try {
        let mut query = sqlx::query(
            r"SELECT
       t1.breakreasona,
       t1.breakreasonb,
       t1.spec,
       t1.breakspec,
       t1.creator,
       t1.creationtime,
       t1.billflag,
       t1.devicecode,
       t1.id
FROM tt_inspect t1
         INNER JOIN tt_machine t2 ON t1.devicecode = t2.machinenumber
WHERE t2.stage = ?
  AND t1.deleteflag = 0
  AND (
    t1.creator LIKE CONCAT('%', ?, '%')
        OR t1.breakreasona LIKE CONCAT('%', ?, '%')
        OR t1.breakreasonb LIKE CONCAT('%', ?, '%')
        OR t1.spec LIKE CONCAT('%', ?, '%')
        OR t1.creationtime LIKE CONCAT('%', ?, '%')
        OR CONCAT(devicecode, '号机台') LIKE CONCAT('%', ?, '%')
    )",
        );
        query = query.bind(api_query.stage as i32);
        for _ in 0..6 {
            query = query.bind(&api_query.filter);
        }
        let mut stream = query.fetch(db);
        let mut collected = vec![];
        while let Some(row) = stream.try_next().await? {
            let break_cause_a: Option<String> = row.try_get(0)?;
            let break_cause_b: Option<String> = row.try_get(1)?;
            let spec: Option<String> = row.try_get(2)?;
            let break_spec: String = row.try_get(3)?;
            let creator: String = row.try_get(4)?;
            let creation_time: String = row.try_get(5)?;
            let checking_flag: i32 = row.try_get(6)?;
            let device_code: i32 = row.try_get(7)?;
            let id: i32 = row.try_get(8)?;
            collected.push(InspectionSummary {
                cause: match checking_flag {
                    0 => break_cause_a,
                    1 => break_cause_b,
                    2 => None,
                    _ => yeet!(anyhow!("Invalid 'billflag' field")),
                },
                product_spec: spec,
                break_spec,
                creator,
                creation_time,
                checking_state: checking_flag as u8,
                machine_number: device_code as u32,
                id: id as u32,
            });
        }
        return api_ok!(collected);
    };
    handle_errors!(r)
}

#[axum::debug_handler]
pub async fn query_details(
    Extension(api_context): Extension<ApiContext>,
    path: Path<(u32,)>,
) -> impl IntoResponse {
    let id = path.0 .0;
    let db = &api_context.db;

    let result: anyhow::Result<()> = try {
        let one = sqlx::query(
            r"SELECT devicecode,
       creator,
       creationtime,
       billflag,
       spec,
       wirenum,
       breakspec,
       twbatchcode,
       trbatchcode,
       dlwarehouse,
       tgproducttime,
       breakflag,
       breakpointb,
       breakpointa,
       reasontype,
       breakreasona,
       memo,
       inspector,
       inspecttime,
       breakreasonb,
       wirespeed,
       devicecategory
FROM tt_inspect
WHERE deleteflag = 0
  AND id = ?",
        )
        .bind(id as i32)
        .fetch_one(db)
        .await?;

        let details = InspectionDetails {
            device_code: one.try_get::<i32, _>(0)? as u32,
            creator: one.try_get::<String, _>(1)?,
            creation_time: one.try_get::<String, _>(2)?,
            inspection_flag: one.try_get::<i32, _>(3)? as u32,
            product_spec: one.try_get::<Option<String>, _>(4)?,
            wire_num: one.try_get::<Option<i32>, _>(5)?.map(|num| num as u32),
            break_spec: one.try_get::<String, _>(6)?,
            wire_batch_code: one.try_get(7)?,
            stick_batch_code: one.try_get(8)?,
            warehouse: one.try_get::<Option<String>, _>(9)?,
            product_time: one.try_get::<Option<String>, _>(10)?,
            break_flag: one.try_get::<&str, _>(11)? == "1",
            breakpoint_b: one.try_get::<Option<BigDecimal>, _>(12)?,
            breakpoint_a: one.try_get::<Option<String>, _>(13)?,
            cause_type: one.try_get(14)?,
            break_cause_a: one.try_get(15)?,
            comments: one.try_get(16)?,
            inspector: one.try_get::<Option<String>, _>(17)?,
            inspection_time: one.try_get(18)?,
            break_cause_b: one.try_get(19)?,
            wire_speed: one.try_get::<Option<i32>, _>(20)?.map(|x| x as u32),
            device_category: one.try_get(21)?,
        };

        return api_ok!(details);
    };
    api_error!(format!("{:?}", result))
}

#[axum::debug_handler]
pub async fn update(
    Extension(api_context): Extension<ApiContext>,
    Path(path): Path<(u32,)>,
    Form(form): Form<InspectionForm>,
) -> impl IntoResponse {
    let id = path.0;
    let db = &api_context.db;

    let result: anyhow::Result<()> = try {
        let query = sqlx::query(FORM_UPDATE_SQL);
        let query = bind_form(query, form);
        // WHERE id = ?
        let query = query.bind(id as i32);
        query.execute(db).await?;
        return api_ok!(());
    };
    handle_errors!(result)
}
