use crate::persistence::entities::video::VideoVisibility;
use bcrypt::{hash, DEFAULT_COST};
use chrono::Datelike;
use sqlx::{types::BigDecimal, PgPool, Postgres, Transaction};
use std::str::FromStr;

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

async fn seed_artists(tx: &mut Transaction<'_, Postgres>) -> anyhow::Result<()> {
    // (name, description)
    let templates = [
        ("Seraphina Noir", "A photographer specializing in dark, sensual imagery with a gothic flair."),
        ("Jax Wilder", "A male model and erotic performer known for his athletic physique and captivating presence."),
        ("Lila Rouge", "A burlesque dancer and performance artist who blends classic glamour with modern sensuality."),
        ("Damon Steele", "A writer and illustrator creating erotic comics and graphic novels with intricate storylines."),
        ("Raven Ash", "A digital artist crafting fantasy-themed erotica with strong female characters."),
        ("Isabelle Bloom", "A painter using vibrant colors and expressive strokes to capture the beauty of the human form."),
        ("Kai Storm", "A musician and composer creating ambient soundscapes for sensual experiences."),
        ("Victoria Velvet", "A lingerie designer specializing in luxurious and seductive pieces."),
        ("Ethan Blaze", "A fitness model and personal trainer who shares workout routines and motivational content."),
        ("Luna Moon", "A poet and spoken word artist exploring themes of love, desire, and intimacy."),
        ("Caleb Stone", "A sculptor working with clay and stone to create erotic figures and abstract forms."),
        ("Aria Night", "A cosplayer and model known for her detailed costumes and captivating portrayals."),
        ("Julian Frost", "A filmmaker and director creating erotic short films and music videos."),
        ("Aurora Rose", "A makeup artist and body painter specializing in sensual and artistic designs."),
        ("Sebastian Grey", "A male escort and cam model offering personalized experiences and companionship."),
        ("Skye Diamond", "A jewelry designer creating custom pieces with erotic and symbolic meanings."),
        ("Phoenix Fire", "A fire performer and dancer who incorporates elements of sensuality and danger."),
        ("River Wilde", "A nature photographer capturing the raw beauty and eroticism of the natural world."),
        ("Zephyr Breeze", "An ASMR artist creating audio experiences designed to evoke relaxation and arousal."),
        ("Indigo Bloom", "A tattoo artist specializing in erotic and symbolic designs using fine lines and delicate shading.")
    ];

    for (name, description) in templates.iter() {
        let artist_user_id = sqlx::query!(
            r#"
            INSERT INTO user_table (username, password_hash, email)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
            name.to_lowercase().replace(" ", "_"),
            hash("password123", DEFAULT_COST)?,
            format!("{}@seed.com", name.to_lowercase().replace(" ", "_")),
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
            description,
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
    }

    Ok(())
}

async fn seed_categories(tx: &mut Transaction<'_, Postgres>) -> anyhow::Result<()> {
    // name
    let categories = [
        "Photography",
        "Modeling",
        "Dance",
        "Writing",
        "Digital Art",
        "Painting",
        "Music",
        "Fashion",
        "Fitness",
        "Poetry",
        "Sculpture",
        "Cosplay",
        "Film",
        "Makeup",
        "Companionship",
        "Jewelry",
        "Performance Art",
        "Photography",
        "ASMR",
        "Tattoo",
    ];

    for category in categories.iter() {
        sqlx::query!(
            r#"
            INSERT INTO video_category (name)
            VALUES ($1)
            "#,
            category,
        )
        .execute(tx.as_mut())
        .await?;
    }

    Ok(())
}

async fn seed_deals(tx: &mut Transaction<'_, Postgres>) -> anyhow::Result<()> {
    let templates: Vec<(&str, BigDecimal, i32)> = vec![
        ("Basic", BigDecimal::from_str("9.90")?, 1),
        ("Extended", BigDecimal::from_str("8.90")?, 3),
        ("Best value", BigDecimal::from_str("7.90")?, 12),
    ];

    for (name, price, duration) in templates.iter() {
        sqlx::query!(
            r#"
            INSERT INTO deal (label, price_per_month, number_of_months)
            VALUES ($1, $2, $3)
            "#,
            name,
            price,
            duration,
        )
        .execute(tx.as_mut())
        .await?;
    }

    Ok(())
}

async fn seed_videos(tx: &mut Transaction<'_, Postgres>) -> anyhow::Result<()> {
    // name, file_path, thumbnail_path, description
    let templates = [(
        "Pussy licking itself vigorously",
        "./seed_resources/videos/pussy_1.mp4",
        "./seed_resources/thumbnails/pussy_1.png",
        "I made this video with my boyfriend watching! What do you think?",
    )];

    for (name, file_path, thumbnail_path, description) in templates.iter() {
        let artist_id = sqlx::query!(
            r#"
            SELECT id
            FROM artist
            ORDER BY random()
            LIMIT 1
            "#,
        )
        .fetch_one(tx.as_mut())
        .await?
        .id;

        let random_number = rand::random::<u8>() % 3;
        let visibility = match random_number {
            0 => VideoVisibility::All,
            1 => VideoVisibility::Registered,
            _ => VideoVisibility::Paying,
        };

        let video_id = sqlx::query!(
            r#"
            INSERT INTO video (artist_id, name, file_path, thumbnail_path, description, visibility)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
            artist_id,
            name,
            file_path,
            thumbnail_path,
            description,
            visibility as VideoVisibility,
        )
        .fetch_one(tx.as_mut())
        .await?
        .id;
    }

    Ok(())
}

pub async fn seed_database(pool: &PgPool) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;

    seed_users(&mut tx).await?;
    seed_deals(&mut tx).await?;

    // videos depend on artists and categories: mind the order
    seed_artists(&mut tx).await?;
    seed_categories(&mut tx).await?;
    seed_videos(&mut tx).await?;

    tx.commit().await?;

    Ok(())
}
