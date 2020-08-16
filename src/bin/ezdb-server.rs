use actix::Actor;
use actix_web::{middleware, App, HttpServer};
use std::path::PathBuf;
use structopt::StructOpt;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::builder().format_timestamp_nanos().init();
    let opts: CliOptions = CliOptions::from_args();
    let addr = format!("{}:{}", opts.host, opts.port);

    let persistence = match opts.db_file {
        None => ezdb::persistence::SqlitePersistence::in_memory().unwrap(),
        Some(db_file) => ezdb::persistence::SqlitePersistence::from_file(&db_file).unwrap(),
    };
    let core = ezdb::core::CoreActor::new(persistence).start();
    HttpServer::new(move || {
        App::new()
            .data(core.clone())
            .wrap(middleware::Logger::default())
            .service(ezdb::server::rest_service())
    })
    .bind(&addr)?
    .run()
    .await?;
    Ok(())
}

#[derive(StructOpt, Debug)]
struct CliOptions {
    #[structopt(long, default_value = "localhost")]
    host: String,
    #[structopt(long, default_value = "9000")]
    port: usize,
    #[structopt(long, parse(from_os_str))]
    db_file: Option<PathBuf>,
}
