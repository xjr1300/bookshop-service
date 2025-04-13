use async_graphql::{
    EmptyMutation, EmptySubscription, FieldResult, Object, Schema, SimpleObject,
    http::GraphiQLSource,
};
use async_graphql_axum::GraphQL;
use axum::{Router, response::IntoResponse, routing::get};
use tokio::net::TcpListener;

use domain::models::{Book, DUMMY_BOOKS};

#[derive(SimpleObject)]
struct BookObject {
    pub id: i32,
    pub title: String,
    pub author: String,
    pub price: i32,
}

impl From<Book> for BookObject {
    fn from(value: Book) -> Self {
        Self {
            id: value.id,
            title: value.title,
            author: value.author,
            price: value.price,
        }
    }
}

struct Query;

#[Object]
impl Query {
    async fn book(&self, id: i32) -> FieldResult<Option<BookObject>> {
        if (1..=2).contains(&id) {
            let index = (id - 1) as usize;
            return Ok(Some(DUMMY_BOOKS[index].clone().into()));
        }
        Ok(None)
    }

    async fn books(&self) -> FieldResult<Vec<BookObject>> {
        Ok(DUMMY_BOOKS.iter().map(|b| b.clone().into()).collect())
    }
}

async fn graphiql() -> impl IntoResponse {
    axum::response::Html(GraphiQLSource::build().endpoint("/").finish())
}

#[tokio::main]
async fn main() {
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription).finish();
    let app = Router::new().route("/", get(graphiql).post_service(GraphQL::new(schema)));

    println!("GraphiQL IDE: http://localhost:4000");

    axum::serve(TcpListener::bind("127.0.0.1:4000").await.unwrap(), app)
        .await
        .unwrap();
}
