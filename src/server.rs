use actix::{Addr, MailboxError};
use actix_web::dev::HttpServiceFactory;
use actix_web::{web, Error, HttpResponse};

use crate::core::{CoreActor, Policy, RestMessage};
use crate::persistence::PersistenceResult;
use actix_web::error::ErrorInternalServerError;
use serde_json::Value;
use std::collections::BTreeMap;

pub fn rest_service() -> impl HttpServiceFactory {
    web::scope("/v0")
        .service(
            web::resource("/raw")
                .route(web::get().to(handle_raw_get))
                .route(web::post().to(handle_raw_post)),
        )
        .service(
            web::resource("/policy")
                .route(web::get().to(handle_policy_get))
                .route(web::put().to(handle_policy_put)),
        )
        .service(
            web::resource("/named/{name}")
                .route(web::get().to(handle_named_get))
                .route(web::post().to(handle_named_post)),
        )
}

async fn handle_raw_get(
    query: String,
    srv: web::Data<Addr<CoreActor>>,
) -> Result<HttpResponse, Error> {
    wrap_output(srv.send(RestMessage::QueryRaw(query)).await)
}

async fn handle_raw_post(
    stmt: String,
    srv: web::Data<Addr<CoreActor>>,
) -> Result<HttpResponse, Error> {
    wrap_output(srv.send(RestMessage::MutateRaw(stmt)).await)
}

async fn handle_policy_get(srv: web::Data<Addr<CoreActor>>) -> Result<HttpResponse, Error> {
    wrap_output(srv.send(RestMessage::FetchPolicy).await)
}

async fn handle_policy_put(
    policy: web::Json<Policy>,
    srv: web::Data<Addr<CoreActor>>,
) -> Result<HttpResponse, Error> {
    wrap_output(srv.send(RestMessage::SetPolicy(policy.into_inner())).await)
}

async fn handle_named_get(
    path: web::Path<String>,
    srv: web::Data<Addr<CoreActor>>,
    params: web::Json<BTreeMap<String, Value>>,
) -> Result<HttpResponse, Error> {
    wrap_output(
        srv.send(RestMessage::QueryNamed(
            path.into_inner(),
            params.into_inner(),
        ))
        .await,
    )
}

async fn handle_named_post(
    path: web::Path<String>,
    srv: web::Data<Addr<CoreActor>>,
    params: web::Json<BTreeMap<String, Value>>,
) -> Result<HttpResponse, Error> {
    wrap_output(
        srv.send(RestMessage::MutateNamed(
            path.into_inner(),
            params.into_inner(),
        ))
        .await,
    )
}

fn wrap_output(
    result: Result<PersistenceResult<String>, MailboxError>,
) -> Result<HttpResponse, Error> {
    match result {
        Ok(Ok(data)) => Ok(HttpResponse::Ok().body(data)),
        Ok(Err(e)) => Ok(HttpResponse::BadRequest().body(format!("{:?}", e))),
        Err(_) => Err(ErrorInternalServerError("oops")),
    }
}
