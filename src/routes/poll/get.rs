use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn show_poll(path: web::Path<Uuid>, db_pool: web::Data<PgPool>) -> HttpResponse {
    let poll_id = path.into_inner();
    let _ = match validate_poll_id(&db_pool, poll_id).await {
        Ok(p) => match p {
            Some(p) => p,
            None => return HttpResponse::NotFound().finish(),
        },
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    HttpResponse::Ok().body(poll_id.to_string())
}

pub async fn validate_poll_id(
    db_pool: &PgPool,
    poll_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT prompt
        FROM polls
        WHERE poll_id = $1
        "#,
        poll_id
    )
    .fetch_optional(db_pool)
    .await?;

    Ok(result.map(|r| r.prompt))
}
