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
        "BBC",
        "Modeling",
        "Dance",
        "Writing",
        "Painting",
        "Music",
        "Fashion",
        "Fitness",
        "Poetry",
        "Sculpture",
        "Cosplay",
        "Film",
        "Makeup",
        "Jewelry",
        "ASMR",
        "Tattoo",
        "MILF",
        "Asian",
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
    let templates = [
        (
        "Pussy licking itself vigorously",
        "./resources/videos/pussy_1.mp4",
        "./resources/thumbnails/pussy_1.png",
        "I made this video with my boyfriend watching! What do you think?",
    ),
    (
        "Touching my pussy at the family house",
        "./resources/videos/pussy_2.mp4",
        "./resources/thumbnails/pussy_2.png",
        "I was alone at home and I decided to record myself touching my pussy. I hope you like it!",
    ),
    (
        "Big black pussy alone",
        "./resources/videos/pussy_3.mp4",
        "./resources/thumbnails/pussy_3.png",
        "I recorded this video with my new camera. I hope you like it!",
    ),
    (
        "New toy for my pussy!",
        "./resources/videos/pussy_4.mp4",
        "./resources/thumbnails/pussy_4.png",
        "This is my new toy! I enjoyed it so much!",
    ),
    (
        "Pussy licking in public",
        "./resources/videos/pussy_5.mp4",
        "./resources/thumbnails/pussy_5.png",
        "I recorded this video in a public place. I am so bad!",
    ),
    (
        "Nasty white bitch",
        "./resources/videos/bitch_1.mp4",
        "./resources/thumbnails/bitch_1.png",
        "I found this and couldn't keep it out of my mouth",
    ),
    (
        "Young bitch gets rough rubbing",
        "./resources/videos/bitch_2.mp4",
        "./resources/thumbnails/bitch_2.png",
        "She likes it rough! Look at her face"
    ),
    (
        "Praise this big alpha cock!",
        "./resources/videos/cock_1.mp4",
        "./resources/thumbnails/cock_1.png",
        "How do you like my cock? Leave a comment!",
    ),
    (
        "Don't know what to do with my cock",
        "./resources/videos/cock_2.mp4",
        "./resources/thumbnails/cock_2.png",
        "Check my bio for details"
    ),
    (
        "This cock is trapped in a cage! Hardcore",
        "./resources/videos/cock_3.mp4",
        "./resources/thumbnails/cock_3.png",
        "I almost couldn't do it...",
    ),
    (
        "Two cocks moaning",
        "./resources/videos/cock_4.mp4",
        "./resources/thumbnails/cock_4.png",
        "I love when these two get together to film with me!",
    )
    ];

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

        let category_id = sqlx::query!(
            r#"
            SELECT id
            FROM video_category
            ORDER BY random()
            LIMIT 1
            "#,
        )
        .fetch_one(tx.as_mut())
        .await?
        .id;

        sqlx::query!(
            r#"
            INSERT INTO video_category_video (video_id, category_id)
            VALUES ($1, $2)
            "#,
            video_id,
            category_id,
        )
        .execute(tx.as_mut())
        .await?;
    }

    Ok(())
}

async fn seed_comments(tx: &mut Transaction<'_, Postgres>) -> anyhow::Result<()> {
    // content
    let contents = [
        "This is amazing!",
        "I love this video!",
        "Great work!",
        "I can't stop watching this!",
        "Wow!",
        "This is so hot!",
        "I need more of this!",
        "Incredible!",
        "I'm speechless!",
        "This is perfect!",
        "Booooring!",
        "I don't like this!",
        "This is not what I expected!",
        "I'm disappointed!",
        "I want my money back!",
        "This is trash!",
        "I would like to have you for myself!",
        "I'm in love!",
        "Send nudes!",
        "I want to see more!",
        "Please, be my girlfriend!",
        "I'm your biggest fan!",
    ];

    for content in contents.iter() {
        let video_id = sqlx::query!(
            r#"
            SELECT id
            FROM video
            ORDER BY random()
            LIMIT 1
            "#,
        )
        .fetch_one(tx.as_mut())
        .await?
        .id;

        let user_id = sqlx::query!(
            r#"
            SELECT id
            FROM user_table
            ORDER BY random()
            LIMIT 1
            "#,
        )
        .fetch_one(tx.as_mut())
        .await?
        .id;

        sqlx::query!(
            r#"
            INSERT INTO comment (video_id, user_id, content)
            VALUES ($1, $2, $3)
            "#,
            video_id,
            user_id,
            content,
        )
        .execute(tx.as_mut())
        .await?;
    }

    Ok(())
}

async fn seed_favorites(tx: &mut Transaction<'_, Postgres>) -> anyhow::Result<()> {
    let video_id = sqlx::query!(
        r#"
        SELECT id
        FROM video
        ORDER BY random()
        LIMIT 1
        "#,
    )
    .fetch_one(tx.as_mut())
    .await?
    .id;

    let user_id = sqlx::query!(
        r#"
        SELECT id
        FROM user_table
        ORDER BY random()
        LIMIT 1
        "#,
    )
    .fetch_one(tx.as_mut())
    .await?
    .id;

    let already_exists = sqlx::query!(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM favorite
            WHERE video_id = $1 AND user_id = $2
        ) AS "exists!"
        "#,
        video_id,
        user_id,
    )
    .fetch_one(tx.as_mut())
    .await?
    .exists;

    if already_exists {
        return Ok(());
    }

    sqlx::query!(
        r#"
        INSERT INTO favorite (video_id, user_id)
        VALUES ($1, $2)
        "#,
        video_id,
        user_id,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}

pub async fn seed_database(pool: &PgPool) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;

    seed_users(&mut tx).await?;
    seed_deals(&mut tx).await?;
    seed_artists(&mut tx).await?;
    seed_categories(&mut tx).await?;
    for _ in 0..10 {
        seed_videos(&mut tx).await?;
    }
    for _ in 0..20 {
        seed_comments(&mut tx).await?;
    }
    for _ in 0..50 {
        seed_favorites(&mut tx).await?;
    }

    tx.commit().await?;

    Ok(())
}
