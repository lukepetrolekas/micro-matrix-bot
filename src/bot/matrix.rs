use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Debug)]
pub enum MatrixError {
    LogonFailure,
    ServerFailure,
    OtherFailure,
}

#[derive(Deserialize)]
pub struct MatrixLoginResponse {
    pub access_token: String,
}

#[derive(Deserialize)]
pub struct MatrixNextBatchResponse {
    pub next_batch: String,
    pub rooms: Option<MatrixRoomCollection>, //This is because if there is no events, the delta has no room info.
}

#[derive(Deserialize)]
pub struct MatrixRoomCollection {
    pub join: HashMap<String, MatrixRoom>,
}

#[derive(Deserialize)]
pub struct MatrixRoom {
    pub timeline: MatrixTimeline,
}

#[derive(Deserialize)]
pub struct MatrixTimeline {
    pub events: Vec<MatrixEvent>
}

#[derive(Deserialize)]
pub struct MatrixEvent {
    pub content: Option<MatrixContent>,
    pub sender: Option<String>,
}

#[derive(Deserialize)]
pub struct MatrixContent {
    pub body: Option<String>,
    pub msgtype: Option<String>,
}
