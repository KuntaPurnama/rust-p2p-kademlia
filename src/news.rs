use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct News {
    pub id: String,
    pub title: String,
    pub author: String,
    pub description: String,
    pub date: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NewsMode {
    ALL,
    ByTitle(String),
    ById(String),
    ByAuthor(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddNews {
    pub news: News,
    pub requester_peer_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewsResponse {
    pub mode: NewsMode,
    pub responser_peer_id: String,
    pub requester_peer_id: String,
    pub data: Vec<News>
}

pub enum EventType {
    Response(NewsResponse),
    Input(String)
}
