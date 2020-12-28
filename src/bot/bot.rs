use reqwest;
use rusqlite::{Connection, OpenFlags, NO_PARAMS};

use serde_derive::Deserialize;

use std::collections::HashMap;

use std::thread;
use std::time::Duration;

pub struct MatrixConfig {
    pub host: &'static str,
    pub login: &'static str,
    pub sync: &'static str,
    pub logout: &'static str,
}

pub struct Bot {
    pub username: &'static str,
    password: String,
    client: reqwest::blocking::Client,
    conn: rusqlite::Connection,
    access_token: String,
    last_batch: String,
    logged_in: bool,
    task: crate::bot::task::Task,
    config: MatrixConfig,
}

#[derive(Debug)]
pub enum MatrixError {
    LogonFailure,
    ServerFailure,
    OtherFailure,
}

#[derive(Deserialize)]
struct MatrixLoginResponse {
    access_token: String,
}

impl Bot {
    pub fn new(
        username: &'static str,
        password: String,
        db_location: String,
        task: crate::bot::task::Task,
        config: MatrixConfig,
    ) -> Bot {

        let client = reqwest::blocking::Client::new();
        let conn = Connection::open_with_flags(db_location, OpenFlags::SQLITE_OPEN_READ_WRITE).unwrap();

        Bot {
            username,
            password,
            client,
            conn,
            access_token: "".to_owned(),
            last_batch: "".to_owned(),
            logged_in: false,
            task,
            config,
        }
    }

    pub fn start(&mut self) {
        loop {
            self.login();
            match self.get_last_known_batch() {
                Some(v) => self.last_batch = v,
                None => {
                /* let uri = format!("{}{}?{}&access_token={}", 
                self.config.host, 
                self.config.sync, 
                "filter={\"room\":{\"timeline\":{\"limit\":1}}}",
                &self.access_token),*/
                }
            }

        /* complaining, rethink
        match data {
            Ok(next_batch) => let uri = format!(
                "{}{}?{}&since={}&access_token={}", 
                self.config.host.to_string(), 
                self.config.sync.to_string(), 
                "filter={\"room\":{\"timeline\":{\"limit\":1}}}", 
                &next_batch, 
                self.access_token),

        }*/

            loop {
                self.last_batch = self.sync();
                thread::sleep(Duration::from_millis(2500));
                break;
            }

            self.logout();
            println!("Rebooting politely...");
            thread::sleep(Duration::from_millis(10000));
            break;
        }
        println!("Shutting down...");
    }

    fn login(&mut self) {
        // continuously make attempt to connect.
        while self.access_token.is_empty() {
            let res = self.get_access_token();

            match res {
                Ok(v) => {
                    self.access_token = v;
                    break;
                }
                Err(e) => {
                    println!("Connection failed. {:?}", e);
                    thread::sleep(Duration::from_millis(30000));
                }
            }
        }

        println!("Connection successful.");
    }

    fn get_access_token(&mut self) -> Result<String, MatrixError> {
        let mut map = HashMap::new();
        map.insert("type", "m.login.password");
        map.insert("user", &self.username);
        map.insert("password", &self.password);

        let response_result = self
            .client
            .post(&format!("{}{}", self.config.host, self.config.login))
            .json(&map)
            .send();

        match response_result {
            Ok(resp) => {
                if resp.status().is_success() {
                    let res: std::result::Result<MatrixLoginResponse, reqwest::Error> = resp.json();   
                    match res {
                        Ok(v) => Ok(v.access_token),
                        Err(_e) => Err(MatrixError::LogonFailure),
                    }
                } else if resp.status().is_server_error() {
                    Err(MatrixError::ServerFailure)
                } else {
                    Err(MatrixError::OtherFailure)
                }
            }
            Err(_e) => {
                Err(MatrixError::OtherFailure)
            },
        }
    }

    fn get_last_known_batch(&mut self) -> Option<String> {
        let data: std::result::Result<String, rusqlite::Error> = self.conn.query_row("SELECT next_batch FROM matrix LIMIT 1", NO_PARAMS, |row| row.get(0));
        match data {
            Ok(v) => Some(v),
            Err(_e) => None,
        }
    }

    fn sync(&mut self) -> String {
        return "hello".to_owned();
    }

    fn logout(&mut self) {
        self.logged_in = false;
        self.client.post(&format!("{}{}", self.config.host, self.config.logout)).send().unwrap();
    }
}
