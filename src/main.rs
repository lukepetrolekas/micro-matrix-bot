use std::env;
use rusqlite::{Connection, OpenFlags, NO_PARAMS};

#[macro_use]
extern crate lazy_static;

use log::LevelFilter;
use log::{info, trace, warn};

mod bot;

fn main() {

    simple_logging::log_to_file("test.log", LevelFilter::Info);
    simple_logging::log_to_file("test.log", LevelFilter::Warn);
    simple_logging::log_to_file("test.log", LevelFilter::Trace);

    const CONFIG: bot::bot::MatrixConfig  = bot::bot::MatrixConfig {
        protocol: "https",
        host: "matrix.org", 
        sync: "/_matrix/client/v3/sync", 
        login: "/_matrix/client/v3/login",
        rooms: "/_matrix/client/v3/rooms",
        send_message: "/send/m.room.message",
        logout: "/_matrix/client/v3/logout"
    };

    let mut password="".to_owned();
    let mut db="".to_owned();

    match env::var("BOT_PASSWORD") {
        Ok(v) => { 
            password = v; 
        },
        Err(e) => { 
            warn!("Couldn't read the environment variable BOT_PASSWORD\n{}", e); 
        },
    };

    match env::var("BOT_DATABASE_LOCATION") {
        Ok(v) => { 
            db = v; 
        },
        Err(e) => { 
            warn!("Couldn't find the database at BOT_DATABASE_LOCATION\n{}", e); 
        },
    };

    let task : bot::task::Task = bot::task::Task::new(db, format!("@{}:{}", "erised", &CONFIG.host));
    let mut b : bot::bot::Bot = bot::bot::Bot::new("erised", password, task, &CONFIG);

    trace!("Starting program.");
    b.start();
    trace!("Ending program.");
}
