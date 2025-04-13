use std::sync::LazyLock;

/// 書籍モデル
#[derive(Debug, Clone)]
pub struct Book {
    pub id: i32,
    pub title: String,
    pub author: String,
    pub price: i32,
}

/// ダミーの書籍データ
pub static DUMMY_BOOKS: LazyLock<[Book; 2]> = LazyLock::new(|| {
    [
        Book {
            id: 1,
            title: "The Awakening".into(),
            author: "Kate Chopin".into(),
            price: 1_000,
        },
        Book {
            id: 2,
            title: "City of Glass".into(),
            author: "Paul Auster".into(),
            price: 2_000,
        },
    ]
});
