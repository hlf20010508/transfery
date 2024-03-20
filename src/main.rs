/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use pico_args::Arguments;

mod env;
mod error;
mod init;
mod storage;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut args = Arguments::from_env();

    if args.contains("--init") {
        init::init().await;
    } else {
        server().await;
    }

    Ok(())
}

async fn server() {}
