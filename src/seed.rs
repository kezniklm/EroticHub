use bcrypt::{hash, DEFAULT_COST};
use chrono::Datelike;
use sqlx::{PgPool, Postgres, Transaction};

pub async fn create_admin(
    pool: &PgPool,
    username: &str,
    password: &str,
    email: &str,
) -> anyhow::Result<()> {
    let password_hash = hash(password, DEFAULT_COST)?;
    sqlx::query!(
        r#"
        INSERT INTO user_table (username, password_hash, email, is_admin)
        VALUES ($1, $2, $3, $4)
        "#,
        username,
        password_hash,
        email,
        true
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn seed_users(tx: &mut Transaction<'_, Postgres>) -> anyhow::Result<()> {
    // registered user
    sqlx::query!(
        r#"
        INSERT INTO user_table (username, password_hash, email)
        VALUES ($1, $2, $3)
        "#,
        "registered_user",
        hash("password123", DEFAULT_COST)?,
        "registered_user@seed.com"
    )
    .execute(tx.as_mut())
    .await?;

    // paying user
    let paying_user_id = sqlx::query!(
        r#"
        INSERT INTO user_table (username, password_hash, email)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        "paying_user",
        hash("password123", DEFAULT_COST)?,
        "paying_user@seed.com"
    )
    .fetch_one(tx.as_mut())
    .await?
    .id;

    let paying_member_id = sqlx::query!(
        r#"
        INSERT INTO paying_member (user_id, valid_until)
        VALUES ($1, $2)
        RETURNING id
        "#,
        paying_user_id,
        chrono::Utc::now() + chrono::Duration::days(30)
    )
    .fetch_one(tx.as_mut())
    .await?
    .id;

    let current_date = chrono::Utc::now();
    let card_expiration_date = chrono::NaiveDate::parse_from_str(
        &format!("01/{:02}/{}", current_date.month(), current_date.year() + 3),
        "%d/%m/%Y",
    )?;
    let payment_method_id = sqlx::query!(
        r#"
        INSERT INTO payment_method (paying_member_id, card_number, card_expiration_date, card_cvc)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
        paying_member_id,
        "1234567890123456",
        card_expiration_date,
        "123"
    )
    .fetch_one(tx.as_mut())
    .await?
    .id;

    sqlx::query!(
        r#"
        UPDATE paying_member
        SET payment_method_id = $1
        WHERE id = $2
        "#,
        payment_method_id,
        paying_member_id
    )
    .execute(tx.as_mut())
    .await?;

    sqlx::query!(
        r#"
        UPDATE user_table
        SET paying_member_id = $1
        WHERE id = $2
        "#,
        paying_member_id,
        paying_user_id,
    )
    .execute(tx.as_mut())
    .await?;

    // artist user
    let artist_user_id = sqlx::query!(
        r#"
        INSERT INTO user_table (username, password_hash, email)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        "artist_user",
        hash("password123", DEFAULT_COST)?,
        "artist_user@seed.com",
    )
    .fetch_one(tx.as_mut())
    .await?
    .id;

    let artist_id = sqlx::query!(
        r#"
        INSERT INTO artist (user_id, description)
        VALUES ($1, $2)
        RETURNING id
        "#,
        artist_user_id,
        "Artist Description",
    )
    .fetch_one(tx.as_mut())
    .await?
    .id;

    sqlx::query!(
        r#"
        UPDATE user_table
        SET artist_id = $1
        WHERE id = $2
        "#,
        artist_id,
        artist_user_id,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}

pub async fn seed_database(pool: &PgPool) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;

    seed_users(&mut tx).await?;

    tx.commit().await?;

    Ok(())
}
