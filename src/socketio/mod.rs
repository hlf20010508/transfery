/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod lobby;
mod message;
mod start_connection;
mod ws;

#[cfg(test)]
mod tests {
    use super::*;
    use actix::Actor;
    use actix_web::{test as atest, App, HttpServer};
    use actix_web_actors::ws;
    use lobby::Lobby;
    use start_connection::start_connection;

    #[atest]
    async fn test_socketio_main() {
        let chat_server = Lobby::default().start(); //create and spin up a lobby
                                                    // ws::start(actor, req, stream)
        let mut app = atest::init_service(App::new().service(start_connection)).await;
    }
}
