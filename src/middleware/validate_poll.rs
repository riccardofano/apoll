use std::future::{ready, Ready};

use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{web, FromRequest, HttpMessage};
use actix_web_lab::middleware::Next;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize, Clone)]
pub struct PollInfo {
    pub poll_id: Uuid,
    pub prompt: String,
}

impl FromRequest for PollInfo {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        match req.extensions().get::<PollInfo>() {
            Some(poll_info) => ready(Ok(poll_info.clone())),
            None => ready(Err(actix_web::error::ErrorBadRequest(
                "could not find PollInfo in request",
            ))),
        }
    }
}

#[tracing::instrument(name = "validate poll id middleware", skip_all)]
pub async fn validate_poll_id(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let poll_id = Uuid::parse_str(req.match_info().query("poll_id")).map_err(e404)?;
    let db_pool = req.app_data::<web::Data<PgPool>>().unwrap();

    let prompt = find_poll(db_pool, poll_id)
        .await
        .map_err(e404)?
        .ok_or_else(|| e404(anyhow::anyhow!("could not find poll_id: {}", poll_id)))?;

    req.extensions_mut().insert(PollInfo { poll_id, prompt });
    next.call(req).await
}

#[tracing::instrument(
    name = "find poll in database"
    skip(db_poll),
)]
async fn find_poll(db_poll: &PgPool, poll_id: Uuid) -> Result<Option<String>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT prompt
        FROM polls
        WHERE poll_id = $1
        "#,
        poll_id
    )
    .fetch_optional(db_poll)
    .await?;

    Ok(result.map(|r| r.prompt))
}

fn e404<T>(e: T) -> actix_web::Error
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    actix_web::error::ErrorNotFound(e)
}
