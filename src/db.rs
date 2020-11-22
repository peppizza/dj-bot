use sqlx::postgres::PgPool;

#[derive(Debug)]
pub enum UserPerm {
    Admin,
    DJ,
    User,
    None,
}

pub async fn get_user_perms(pool: &PgPool, user_id: i64) -> anyhow::Result<UserPerm> {
    let rec = match sqlx::query!(
        r#"
        SELECT permlevel
        FROM perms
        WHERE id = $1"#,
        user_id
    )
    .fetch_optional(pool)
    .await?
    {
        Some(row) => row,
        None => return Ok(UserPerm::None),
    };

    Ok(perm_level_to_user_perm(rec.permlevel))
}

pub async fn set_user_perms(
    pool: &PgPool,
    user_id: i64,
    perm_level: i16,
) -> anyhow::Result<UserPerm> {
    let rec = sqlx::query!(
        r#"
        INSERT INTO perms (id, permlevel)
        VALUES($1, $2)
        ON CONFLICT (id)
        DO
            UPDATE SET permlevel = EXCLUDED.permlevel
        RETURNING permlevel
        "#,
        user_id,
        perm_level
    )
    .fetch_one(pool)
    .await?;

    Ok(perm_level_to_user_perm(rec.permlevel))
}

fn perm_level_to_user_perm(perm_level: i16) -> UserPerm {
    match perm_level {
        0 => UserPerm::None,
        1 => UserPerm::User,
        2 => UserPerm::DJ,
        3 => UserPerm::Admin,
        _ => unreachable!(),
    }
}
