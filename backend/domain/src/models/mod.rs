use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

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

/// 注文商品
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderItem {
    pub item_id: u64,
    pub quantity: u32,
    pub unit_price: u32,
}

/// 注文イベント
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderEvent {
    pub id: String,
    pub customer_id: String,
    pub customer_name: String,
    pub order_items: Vec<OrderItem>,
}
