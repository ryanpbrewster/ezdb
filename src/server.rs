use actix::{Addr, MailboxError};
use actix_web::dev::{HttpServiceFactory, ServiceRequest};
use actix_web::{web, Error, HttpResponse};

use crate::core::{CoreActor, Policy, RestMessage};
use crate::persistence::{PersistenceError, PersistenceResult};
use crate::tokens::{DatabaseId, ProjectId};
use actix_web::error::ErrorInternalServerError;
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use actix_web_httpauth::middleware::HttpAuthentication;
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub fn rest_service() -> impl HttpServiceFactory {
    let auth = HttpAuthentication::bearer(verify_admin_auth);
    web::scope("/v0/{project_id}/{database_id}")
        .service(
            web::resource("/raw")
                .wrap(auth.clone())
                .route(web::get().to(handle_raw_get))
                .route(web::post().to(handle_raw_post)),
        )
        .service(
            web::resource("/policy")
                .wrap(auth)
                .route(web::get().to(handle_policy_get))
                .route(web::put().to(handle_policy_put)),
        )
        .service(
            web::resource("/named/{name}")
                .route(web::get().to(handle_named_get))
                .route(web::post().to(handle_named_post)),
        )
}

async fn verify_admin_auth(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    if credentials.token() != "admin" {
        return Err(AuthenticationError::from(Config::default()).into());
    }
    Ok(req)
}

async fn handle_raw_get(
    path: web::Path<(ProjectId, DatabaseId)>,
    query: String,
    srv: web::Data<Addr<CoreActor>>,
) -> Result<HttpResponse, Error> {
    wrap_output(srv.send(RestMessage::QueryRaw(query)).await)
}

async fn handle_raw_post(
    path: web::Path<(ProjectId, DatabaseId)>,
    stmt: String,
    srv: web::Data<Addr<CoreActor>>,
) -> Result<HttpResponse, Error> {
    wrap_output(srv.send(RestMessage::MutateRaw(stmt)).await)
}

async fn handle_policy_get(
    path: web::Path<(ProjectId, DatabaseId)>,
    srv: web::Data<Addr<CoreActor>>,
) -> Result<HttpResponse, Error> {
    wrap_output(srv.send(RestMessage::FetchPolicy).await)
}

async fn handle_policy_put(
    path: web::Path<(ProjectId, DatabaseId)>,
    policy: web::Json<Policy>,
    srv: web::Data<Addr<CoreActor>>,
) -> Result<HttpResponse, Error> {
    wrap_output(srv.send(RestMessage::SetPolicy(policy.into_inner())).await)
}

async fn handle_named_get(
    path: web::Path<(ProjectId, DatabaseId, String)>,
    srv: web::Data<Addr<CoreActor>>,
    params: web::Json<BTreeMap<String, Value>>,
) -> Result<HttpResponse, Error> {
    let (_project_id, _database_id, name) = path.into_inner();
    wrap_output(
        srv.send(RestMessage::QueryNamed(name, params.into_inner()))
            .await,
    )
}

async fn handle_named_post(
    path: web::Path<(ProjectId, DatabaseId, String)>,
    srv: web::Data<Addr<CoreActor>>,
    params: web::Json<BTreeMap<String, Value>>,
) -> Result<HttpResponse, Error> {
    let (_project_id, _database_id, name) = path.into_inner();
    wrap_output(
        srv.send(RestMessage::MutateNamed(name, params.into_inner()))
            .await,
    )
}

fn wrap_output(
    result: Result<PersistenceResult<String>, MailboxError>,
) -> Result<HttpResponse, Error> {
    match result {
        Ok(Ok(data)) => Ok(HttpResponse::Ok().body(data)),
        Ok(Err(e)) => {
            let payload = match e {
                PersistenceError::Unknown(msg) => json!({
                    "code": "unknown",
                    "message": msg,
                }),
                PersistenceError::NoSuchQuery(name) => json!({
                    "code": "not_found",
                    "message": "no such query",
                    "details": {
                        "name": name,
                    },
                }),
            };
            Ok(HttpResponse::BadRequest().body(payload))
        }
        Err(_) => Err(ErrorInternalServerError("oops")),
    }
}
