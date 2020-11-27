use sqlx::postgres::PgPool;

#[derive(Debug)]
pub enum UserPerm {
    Admin,
    DJ,
    User,
    None,
}

impl From<i16> for UserPerm {
    fn from(i: i16) -> Self {
        match i {
            0 => Self::None,
            1 => Self::User,
            2 => Self::DJ,
            3 => Self::Admin,
            _ => unreachable!(),
        }
    }
}

pub async fn get_user_perms(
    pool: &PgPool,
    guild_id: i64,
    user_id: i64,
) -> anyhow::Result<Option<UserPerm>> {
    let rec = match sqlx::query!(
        r#"
        SELECT perm_level
        FROM perms
        WHERE guild_id = $1 AND user_id = $2"#,
        guild_id,
        user_id
    )
    .fetch_optional(pool)
    .await?
    {
        Some(row) => row,
        None => return Ok(None),
    };

    Ok(Some(rec.perm_level.into()))
}

pub async fn set_user_perms(
    pool: &PgPool,
    guild_id: i64,
    user_id: i64,
    perm_level: i16,
) -> anyhow::Result<UserPerm> {
    let rec = sqlx::query!(
        r#"
        INSERT INTO perms (guild_id, user_id, perm_level) VALUES ($1, $2, $3)
        ON CONFLICT (guild_id, user_id)
        DO UPDATE SET perm_level = $3
        RETURNING perm_level
        "#,
        guild_id,
        user_id,
        perm_level
    )
    .fetch_one(pool)
    .await?;

    Ok(rec.perm_level.into())
}
