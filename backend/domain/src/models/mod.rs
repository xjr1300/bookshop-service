/// 書籍モデル
#[derive(Debug, Clone)]
pub struct Book {
    pub id: i32,
    pub title: String,
    pub author: String,
    pub price: i32,
}
