use actix::{Addr, MailboxError};
use actix_web::dev::HttpServiceFactory;
use actix_web::{web, Error, HttpResponse};
use log::debug;

use crate::core::{CoreActor, RestMessage};
use actix_web::error::ErrorInternalServerError;

pub fn rest_service() -> impl HttpServiceFactory {
    web::resource("/v0/{name}").route(web::get().to(handle_rest_get))
}

async fn handle_rest_get(
    path: web::Path<String>,
    srv: web::Data<Addr<CoreActor>>,
) -> Result<HttpResponse, Error> {
    debug!("HTTP GET @ {}", path);
    wrap_output(srv.send(RestMessage::Get(path.into_inner())).await)
}

fn wrap_output(result: Result<Option<String>, MailboxError>) -> Result<HttpResponse, Error> {
    match result {
        Ok(Some(data)) => Ok(HttpResponse::Ok().body(data)),
        Ok(None) => Ok(HttpResponse::NoContent().finish()),
        Err(_) => Err(ErrorInternalServerError("oops")),
    }
}
