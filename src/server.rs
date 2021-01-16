use actix::Addr;
use actix_web::dev::{HttpServiceFactory, ServiceRequest};
use actix_web::{web, Error, HttpResponse};

use crate::core::{DataMessage, EzdbMessage, Policy, RoutingActor};
use crate::persistence::{PersistenceError, PersistenceResult};
use crate::tokens::{DatabaseAddress, DatabaseId, ProjectId};
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
    srv: web::Data<Addr<RoutingActor>>,
) -> Result<HttpResponse, Error> {
    let (project_id, database_id) = path.into_inner();
    Ok(wrap_output(
        handle_message(
            srv.get_ref(),
            DatabaseAddress {
                project_id,
                database_id,
            },
            EzdbMessage::Data(DataMessage::QueryRaw(query)),
        )
        .await,
    ))
}

async fn handle_raw_post(
    path: web::Path<(ProjectId, DatabaseId)>,
    stmt: String,
    srv: web::Data<Addr<RoutingActor>>,
) -> Result<HttpResponse, Error> {
    let (project_id, database_id) = path.into_inner();
    Ok(wrap_output(
        handle_message(
            srv.get_ref(),
            DatabaseAddress {
                project_id,
                database_id,
            },
            EzdbMessage::Data(DataMessage::MutateRaw(stmt)),
        )
        .await,
    ))
}

async fn handle_policy_get(
    path: web::Path<(ProjectId, DatabaseId)>,
    srv: web::Data<Addr<RoutingActor>>,
) -> Result<HttpResponse, Error> {
    let (project_id, database_id) = path.into_inner();
    Ok(wrap_output(
        handle_message(
            srv.get_ref(),
            DatabaseAddress {
                project_id,
                database_id,
            },
            EzdbMessage::Data(DataMessage::FetchPolicy),
        )
        .await,
    ))
}

async fn handle_policy_put(
    path: web::Path<(ProjectId, DatabaseId)>,
    policy: web::Json<Policy>,
    srv: web::Data<Addr<RoutingActor>>,
) -> Result<HttpResponse, Error> {
    let (project_id, database_id) = path.into_inner();
    Ok(wrap_output(
        handle_message(
            srv.get_ref(),
            DatabaseAddress {
                project_id,
                database_id,
            },
            EzdbMessage::Data(DataMessage::SetPolicy(policy.into_inner())),
        )
        .await,
    ))
}

async fn handle_named_get(
    path: web::Path<(ProjectId, DatabaseId, String)>,
    srv: web::Data<Addr<RoutingActor>>,
    params: web::Json<BTreeMap<String, Value>>,
) -> Result<HttpResponse, Error> {
    let (project_id, database_id, name) = path.into_inner();
    Ok(wrap_output(
        handle_message(
            srv.get_ref(),
            DatabaseAddress {
                project_id,
                database_id,
            },
            EzdbMessage::Data(DataMessage::QueryNamed(name, params.into_inner())),
        )
        .await,
    ))
}

async fn handle_named_post(
    path: web::Path<(ProjectId, DatabaseId, String)>,
    srv: web::Data<Addr<RoutingActor>>,
    params: web::Json<BTreeMap<String, Value>>,
) -> Result<HttpResponse, Error> {
    let (project_id, database_id, name) = path.into_inner();
    Ok(wrap_output(
        handle_message(
            srv.get_ref(),
            DatabaseAddress {
                project_id,
                database_id,
            },
            EzdbMessage::Data(DataMessage::MutateNamed(name, params.into_inner())),
        )
        .await,
    ))
}

async fn handle_message(
    router: &Addr<RoutingActor>,
    db_addr: DatabaseAddress,
    msg: EzdbMessage,
) -> PersistenceResult<String> {
    let core = router.send(db_addr).await??;
    core.send(msg).await?
}

fn wrap_output(result: PersistenceResult<String>) -> HttpResponse {
    match result {
        Ok(data) => HttpResponse::Ok().body(data),
        Err(e) => {
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
                PersistenceError::Interrupted => json!({
                    "code": "interrupted",
                    "message": "Operation was interrupted",
                }),
                PersistenceError::Busy => json!({
                    "code": "busy",
                    "message": "Database is busy, back off and try again",
                }),
            };
            HttpResponse::BadRequest().body(payload)
        }
    }
}
