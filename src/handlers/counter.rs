use sqlx::Row;

use crate::{include_sql, MySqlPool};

pub async fn increase(db: &MySqlPool) -> anyhow::Result<i32> {
    let row = sqlx::query(include_sql!("inspection-counter"))
        .fetch_one(db)
        .await?;
    let num: i32 = row.try_get("num")?;
    let mut increased = num + 1;
    if increased == 1000 {
        // reset to 001
        increased = 1
    };

    sqlx::query(include_sql!("counter-update"))
        .bind(increased)
        .execute(db)
        .await?;

    Ok(num)
}
