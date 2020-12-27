use reqwest;
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
    access_token: String,
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
        config: MatrixConfig,
        task: crate::bot::task::Task,
    ) -> Bot {
        let client = reqwest::blocking::Client::new();
        Bot {
            username,
            password,
            client,
            access_token: "".to_owned(),
            logged_in: false,
            task,
            config,
        }
    }

    pub fn start(&mut self) {
        loop {
            self.login();
            
            loop {
                println!("I'm running!!!");
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

    pub fn logout(&mut self) {
        self.logged_in = false;
        self.client.post(&format!("{}{}", self.config.host, self.config.logout)).send().unwrap();
    }
}
