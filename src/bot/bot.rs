use reqwest;
use std::collections::HashMap;

use std::thread;
use std::time::Duration;

use rand;

use crate::bot::matrix::*;
use crate::bot::task::Task;

use log::{info, trace, warn};


//Authorization: Bearer TheTokenHere

pub struct MatrixConfig {
    pub protocol: &'static str,
    pub host: &'static str,
    pub login: &'static str,
    pub sync: &'static str,
    pub rooms: &'static str,
    pub send_message: &'static str,
    pub logout: &'static str,
}

pub struct Bot {
    pub username: &'static str,
    password: String,
    client: reqwest::blocking::Client,
    task: Task,
    access_token: String,
    logged_in: bool,
    config: &'static MatrixConfig,
}

pub struct Message {
    pub room: String,
    pub message: String
}

impl Bot {
    pub fn new(
        username: &'static str,
        password: String,
        task: Task,
        config: &'static MatrixConfig,
    ) -> Bot {
        let client = reqwest::blocking::Client::new();


        Bot {
            username,
            password,
            client,
            task,
            access_token: "".to_owned(),
            logged_in: false,
            config,
        }
    }

    pub fn start(&mut self) {
        loop {
            self.login();
            info!("Login successful."); 

            let mut curr_next_batch = self.task.get_last_known_batch().unwrap_or("".to_owned());

            //if a batch has been found
            loop {
                match self.sync(curr_next_batch.clone()) {
                    Some(v) => { curr_next_batch = v.clone(); }
                    None => { break; }
                }

                self.task.tick(&curr_next_batch);
                thread::sleep(Duration::from_millis(5000));
            }

            // attempt to recover?
            self.logout();
            info!("Rebooting politely...");
            thread::sleep(Duration::from_millis(10000));
        }
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
                    warn!("Connection failed. {:?}", e);
                    thread::sleep(Duration::from_millis(30000));
                }
            }
        }

        info!("Connection successful.");
    }

    fn get_access_token(&mut self) -> Result<String, MatrixError> {
        let mut map = HashMap::new();
        map.insert("type", "m.login.password");
        map.insert("user", &self.username);
        map.insert("password", &self.password);

        let response_result = self
            .client
            .post(&format!(
                "{}://{}{}",
                self.config.protocol, self.config.host, self.config.login
            ))
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
            Err(_e) => Err(MatrixError::OtherFailure),
        }
    }

    fn sync(&mut self, next_batch: String) -> Option<String> {
        let mut url = format!(
            "{}://{}{}",
            &self.config.protocol, &self.config.host, &self.config.sync
        );

        if !next_batch.is_empty() {
            url = format!(
                "{}?set_presence=online&since={}&timeout=25",
                url, &next_batch
            );
        }

        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send();

        // if it fails, oh well, try again.
        if resp.is_err() {
            return Some(next_batch);
        }

        let v: MatrixNextBatchResponse = resp.unwrap().json().unwrap();
        
        if v.rooms.is_some() {
            for r in v.rooms.unwrap().join {
                let messages: Vec<Message> = self.task.parse(&r.0, r.1.timeline);
                // send the messages requested.
                for m in messages {
                    self.write(&m.room, &m.message)
                }
            }
        }

        return Some(v.next_batch);
    }

    fn logout(&mut self) {
        self.logged_in = false;
        self.client
            .post(&format!(
                "{}://{}{}",
                self.config.protocol, self.config.host, self.config.logout
            ))
            .send()
            .unwrap();
    }

    fn write(&mut self, room: &str, message: &str) {
        let request_url = format!(
            "{}://{}{}/{}{}/{}",
            &self.config.protocol,
            &self.config.host,
            &self.config.rooms,
            room,
            &self.config.send_message,
            rand::random::<i32>()
        );

        let mut map = HashMap::new();
        map.insert("msgtype", "m.text");
        map.insert("body", message);

        let resx: std::result::Result<reqwest::blocking::Response, reqwest::Error> = self
            .client
            .put(&request_url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(&map)
            .send();

        info!("{}", &resx.unwrap().status());
    }
}
