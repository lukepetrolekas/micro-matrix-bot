use rusqlite::{Connection, OpenFlags, NO_PARAMS};
use crate::bot::matrix::MatrixTimeline;
use crate::bot::bot::Message;

use regex::Regex;

use log::{info, trace, warn};

use dateparser::parse_with_timezone;
use chrono::offset::Local;
use std::error::Error;

pub struct Task {
    conn: rusqlite::Connection,
    sender: String,
}

#[derive(Debug)]
struct Event {
    rowid: i32,
    host: String,
    description: String,
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
            self.conn
                .execute(
                    "UPDATE matrix set next_batch = (?1) where id = 1",
                    &[next_batch],
                )
                .unwrap();
        }
    }

    pub fn parse(&mut self, room: &String, timeline: MatrixTimeline) -> Vec<Message> {
        lazy_static! {
            static ref CAL_LIST: Regex = Regex::new(r"^\s*![Cc]al\s+list").unwrap();
            static ref CAL_HELP: Regex = Regex::new(r"^\s*![Cc]al\s+help").unwrap();
            static ref CAL_ADD: Regex = Regex::new(r#"^\s*![Cc]al\s+add\s+([^\|]+)\s*\|\s*(.*)"#).unwrap();
            static ref CAL_RM: Regex = Regex::new(r"^\s*![Cc]al\s+rm\s+(\d+)").unwrap();
        }

        let mut cal_list_flag = false;
        let mut cal_help_flag = false;

        let mut messages : Vec<Message> = Vec::new();

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

                        let dt = CAL_ADD
                            .captures(&message)
                            .unwrap()
                            .get(2)
                            .unwrap().as_str();

                        let attempt_dt_parse = parse_with_timezone(dt, &Local);
                        
                        if !event.is_empty() && attempt_dt_parse.is_ok() {
                            let utc_dt = attempt_dt_parse.unwrap();

                            self.conn.execute(
                                "INSERT INTO events (host, description, dt) VALUES (?1, ?2, ?3)",
                                &[&s, event, utc_dt.timestamp().to_string().as_str()],
                            ).unwrap();

                            messages.push(Message { room: room.to_string(), message: format!("Thank you {}. Event added to list.", s)});

                            info!("Event added.");
                        } else {
                            messages.push(Message { room: room.to_string(), message: format!("{}: Your event could not be added. Please try again or invoke !cal help.", s)});
                            info!("Event addition failed.");
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

                        let t = self.conn
                            .execute(
                                "DELETE FROM events where host=?1 and rowid=?2",
                                &[&s, remove_event],
                            )
                            .unwrap();

                        match t {
                            0 => { 
                                messages.push(Message { room: room.to_string(), message: format!("Sorry, event #{} does not exist.", remove_event)}); 
                                info!("An attempt to remove occured that failed.");
                            }
                            _ => { 
                                messages.push(Message { room: room.to_string(), message: format!("Event #{} removed by {}.", remove_event, s)}); 
                                info!("Event removed.");
                            }
                        }
                    }

                }
            }
        }

        //if list message, help message...
        if cal_list_flag { 

            let mut stmt = self.conn
            .prepare("SELECT rowid, host, description FROM events")
            .unwrap();

            let events_iter = stmt
                .query_map(NO_PARAMS, |row| 
                    Ok( 
                        Event {
                            rowid: row.get(0)?,
                            host: row.get(1)?,
                            description: row.get(2)?,
                        }
                    )
                ).unwrap();	

            let mut l: String = "".to_string();
            let mut count = 0;
            
            for e in events_iter {
                count += 1;
                let q = e.unwrap();
                l = format!("{}{}\t{}\t{}\n",l, q.rowid, q.description, q.host);
            }

            if count == 0 {
                l = "\nThere are no events currently.".to_string();
            } else {           
                l = format!("Calendar List\n#\tDescription\tHost\n{}", l);
            }
        
            messages.push(Message { room: room.to_string(), message: l});
            info!("The list of events was requested.");
        }
        
        
        if cal_help_flag { 
            messages.push(Message { room: room.to_string(), message: format!("Hi I'm Erised the event bot. What do you desire? Commands: cal list, cal add <event> | <datetime>, cal rm <id>, cal help. (Use !)")}); 
            info!("Help feature was requested.")
        }

        return messages;
    }

    
}
