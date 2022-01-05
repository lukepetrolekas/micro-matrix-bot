use rusqlite::{Connection, OpenFlags, NO_PARAMS};
use crate::bot::matrix::MatrixTimeline;

use regex::Regex;

pub struct Task {
    conn: rusqlite::Connection,
    sender: String,
}

impl Task {
    pub fn new(db: String, sender: String) -> Task {
        Task {
            conn: Connection::open_with_flags(db, OpenFlags::SQLITE_OPEN_READ_WRITE).unwrap(),
            sender,
        }
    }

    pub fn get_last_known_batch(&mut self) -> Option<String> {
        let data: std::result::Result<String, rusqlite::Error> =
            self.conn
                .query_row("SELECT next_batch FROM matrix LIMIT 1", NO_PARAMS, |row| {
                    row.get(0)
                });
        match data {
            Ok(v) => {
                if v.trim().is_empty() {
                    return None; // blank value or space is not useful
                }

                Some(v)
            }
            Err(_e) => None,
        }
    }

    pub fn tick(&mut self, next_batch: &String) {
        //save the batch that is found into the database
        //yes this costs 1 initial re-write but we should get a lot out of it.

        if !(next_batch.is_empty()) {
            println!("updatebatch");
            self.conn
                .execute(
                    "UPDATE matrix set next_batch = (?1) where id = 1",
                    &[next_batch],
                )
                .unwrap();
        }
    }

    pub fn parse(&mut self, room: &String, timeline: MatrixTimeline) {
        //Statically created variables only created once for every iteration of interpret
        lazy_static! {
            static ref CAL_LIST: Regex = Regex::new(r"^\s*![Cc]al\s+list").unwrap();
            static ref CAL_HELP: Regex = Regex::new(r"^\s*![Cc]al\s+help").unwrap();
            static ref CAL_ADD: Regex = Regex::new(r#"^\s*![Cc]al\s+add\s+(.*)"#).unwrap();
            static ref CAL_RM: Regex = Regex::new(r"^\s*![Cc]al\s+rm\s+(\d+)").unwrap();
        }

        let mut cal_list_flag = false;
        let mut cal_help_flag = false;

        // if there is a room, there will be events in the room
        for e in timeline.events {
            // only process events with content
            if e.content.is_some() {
                let b = e.content.unwrap();
                let s = e.sender.unwrap_or("".to_owned());

                // only process ones with a body, which implies a message
                // and NOT messages sent by the bot
                if b.body.is_some() && s.ne(&self.sender) {
                    let message = b.body.unwrap();
                    println!("echo: {}", message);

                    // if the body includes !cal list then save that the list should be shown (mutliple people could invoke it and spam channel)
                    if CAL_LIST.find(&message).is_some() {
                        cal_list_flag = true;
                    }

                    // if body includes !cal help then return this information (mutliple people could invoke it and spam channel)
                    if CAL_HELP.find(&message).is_some() {
                        cal_help_flag = true;
                    }

                    // if body includes !cal add [desc] then add to list
                    if CAL_ADD.find(&message).is_some() {
                        let event = CAL_ADD
                            .captures(&message)
                            .unwrap()
                            .get(1)
                            .unwrap().as_str();

                        println!("add = {}", event);

                        //println!("{}", conn.un);
                        if !event.is_empty() {
                            self.conn.execute(
                                "INSERT INTO events (host, description) VALUES (?1, ?2)",
                                &[&s, event],
                            ).unwrap();
                        }

                    }

                     // if the body includes !cal rm [id] (only if creator and remover are the same person)
                    if CAL_RM.find(&message).is_some() {
                        let remove_event = CAL_RM
                            .captures(&message)
                            .unwrap()
                            .get(1)
                            .unwrap()
                            .as_str();
                        println!("rm = {}", remove_event);

                        let t = self.conn
                            .execute(
                                "DELETE FROM events where host=?1 and rowid=?2",
                                &[&s, remove_event],
                            )
                            .unwrap();

                        /*
                        match t {
                            0 => send(client, room, token, "No action taken as the event does not exist."),
                            _ => send(client, room, token, "Event removed."),
                        }*/
                    }

                }
            }
        }

        //if list message, help message...

        /*


        
       

        send(
            client,
            room,
            token,
            "I don't understand the request. cal (add|rm <n>|list|help)",
        );
        None

        */
    }
}
