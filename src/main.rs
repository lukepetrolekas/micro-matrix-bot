use std::env;

mod bot;

fn main() {
    const CONFIG: bot::bot::MatrixConfig  = bot::bot::MatrixConfig { 
        host: "https://matrix.org/", 
        sync: "_matrix/client/r0/sync", 
        login: "_matrix/client/r0/login", 
        logout: "_matrix/client/r0/logout"
    };

    let mut password="".to_owned();
    let mut db="".to_owned();

    match env::var("BOT_PASSWORD") {
        Ok(v) => { 
            password = v; 
        },
        Err(e) => { 
            println!("Couldn't read the environment variable BOT_PASSWORD\n{}", e); 
        },
    };

    match env::var("BOT_DATABASE_LOCATION") {
        Ok(v) => { 
            db = v; 
        },
        Err(e) => { 
            println!("Couldn't find the database at BOT_DATABASE_LOCATION\n{}", e); 
        },
    };
  
    let task : bot::task::Task = bot::task::Task::new("events.db".to_owned());
    let mut b : bot::bot::Bot = bot::bot::Bot::new("Erised", password, "events.db".to_owned(), task, CONFIG);

    b.start();
}
