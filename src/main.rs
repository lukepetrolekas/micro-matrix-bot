use std::env;

mod bot;

fn main() {
    const CONFIG: bot::bot::MatrixConfig  = bot::bot::MatrixConfig { 
        host: "https://matrix.org/", 
        sync: "_matrix/client/r0/sync", 
        login: "_matrix/client/r0/login", 
        logout: "_matrix/client/r0/logout"
    };

    match env::var("BOT_PASSWORD") {
        Ok(v) => { 
            let password = v; 

            let task : bot::task::Task = bot::task::Task::new();
            let mut b : bot::bot::Bot = bot::bot::Bot::new("Erised", password, CONFIG, task);

            b.start();
        },
        Err(e) => { 
            println!("Couldn't read the environment variable BOT_PASSWORD\n{}", e); 
        },
    };
}
