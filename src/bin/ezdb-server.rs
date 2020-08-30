use actix::Actor;
use actix_web::{middleware, App, HttpServer};
use std::path::PathBuf;
use structopt::StructOpt;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::builder().format_timestamp_nanos().init();
    let opts: CliOptions = CliOptions::from_args();
    let addr = format!("{}:{}", opts.host, opts.port);

    let persistence = match opts.db_dir {
        None => ezdb::persistence::SqliteFactory::InMemory,
        Some(dir) => ezdb::persistence::SqliteFactory::FileSystem { dir },
    };
    let core = ezdb::core::RoutingActor::new(persistence).start();
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
    db_dir: Option<PathBuf>,
}
