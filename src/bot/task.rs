use rusqlite::{Connection, OpenFlags, NO_PARAMS};

pub struct Task {
    conn: rusqlite::Connection,
}

impl Task {
    pub fn new(db: String) -> Task {
        Task {
            conn: Connection::open_with_flags(db, OpenFlags::SQLITE_OPEN_READ_WRITE).unwrap(),
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

    pub fn parse(&mut self, room: &String, sender: String, message: String) {
        println!("message: {}", message);
    }
}