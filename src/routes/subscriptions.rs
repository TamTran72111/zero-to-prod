use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        email=%form.email,
        name=%form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, HttpResponse> {
    // `web::Form` is a wrapper around `FormData`
    // `form.0` gives access to the underlying `FormData`
    let name =
        SubscriberName::parse(form.0.name).map_err(|_| HttpResponse::BadRequest().finish())?;

    let email =
        SubscriberEmail::parse(form.0.email).map_err(|_| HttpResponse::BadRequest().finish())?;

    let new_subscriber = NewSubscriber { email, name };

    insert_subscriber(&pool, &new_subscriber)
        .await
        .map_err(|_| HttpResponse::InternalServerError().finish())?;
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Saving new subscriber details in the databse",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
