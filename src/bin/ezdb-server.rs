use actix::Actor;
use actix_web::{middleware, App, HttpServer};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct CliOptions {
    #[structopt(long = "host", default_value = "localhost")]
    host: String,
    #[structopt(long = "port", default_value = "9000")]
    port: usize,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::builder().format_timestamp_nanos().init();
    let opts: CliOptions = CliOptions::from_args();
    let addr = format!("{}:{}", opts.host, opts.port);

    let core = ezdb::core::CoreActor::default().start();
    HttpServer::new(move || {
        App::new()
            .data(core.clone())
            .wrap(middleware::Logger::default())
            .service(ezdb::server::rest_service())
    })
    .bind(&addr)?
    .run()
    .await
}
