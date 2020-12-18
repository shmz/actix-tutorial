use super::schema::memos;
use serde::Serialize;
use chrono::NaiveDateTime;

#[derive(Queryable, Serialize)]
pub struct Memo {
    pub id: i32,
    pub content: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "memos"]
pub struct NewMemo {
    pub content: String,
    pub created_at: NaiveDateTime,
}
