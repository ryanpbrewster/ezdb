use actix::{Addr, MailboxError};
use actix_web::dev::HttpServiceFactory;
use actix_web::web::Bytes;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use log::debug;

use crate::core::{CoreActor, RestMessage};
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};

pub fn rest_service() -> impl HttpServiceFactory {
    web::resource("{any:.*}")
        .route(web::get().to(handle_rest_get))
        .route(web::put().to(handle_rest_put))
        .route(web::delete().to(handle_rest_delete))
}

async fn handle_rest_get(
    req: HttpRequest,
    srv: web::Data<Addr<CoreActor>>,
) -> Result<HttpResponse, Error> {
    debug!("HTTP GET @ {}", req.path());
    let path = req.path().to_owned();
    wrap_output(srv.send(RestMessage::Get(path)).await)
}

async fn handle_rest_delete(
    req: HttpRequest,
    srv: web::Data<Addr<CoreActor>>,
) -> Result<HttpResponse, Error> {
    debug!("HTTP DELETE @ {}", req.path());
    let path = req.path().to_owned();
    wrap_output(srv.send(RestMessage::Delete(path)).await)
}

async fn handle_rest_put(
    req: HttpRequest,
    payload: Bytes,
    srv: web::Data<Addr<CoreActor>>,
) -> Result<HttpResponse, Error> {
    debug!("HTTP PUT @ {} w/ {:?}", req.path(), payload);
    let path = req.path().to_owned();
    let body = serde_json::from_slice(&payload).map_err(ErrorBadRequest)?;
    wrap_output(srv.send(RestMessage::Put(path, body)).await)
}

fn wrap_output(result: Result<Option<String>, MailboxError>) -> Result<HttpResponse, Error> {
    match result {
        Ok(Some(data)) => Ok(HttpResponse::Ok().body(data)),
        Ok(None) => Ok(HttpResponse::NoContent().finish()),
        Err(_) => Err(ErrorInternalServerError("oops")),
    }
}
