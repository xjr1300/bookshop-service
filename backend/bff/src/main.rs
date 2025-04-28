use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;

use async_graphql::Context;
use async_graphql::http::GraphiQLSource;
use async_graphql::{EmptyMutation, EmptySubscription, FieldResult, Schema, SimpleObject};
use async_graphql_axum::GraphQL;
use axum::http::{HeaderValue, Method};
use axum::response::IntoResponse;
use axum::{Router, routing};
use clap::Parser;
use hyper::header;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

use catalogue::book::{Book as CatalogueBook, CatalogueClient, GetBookRequest};

#[derive(SimpleObject)]
struct BookObject {
    pub id: i32,
    pub title: String,
    pub author: String,
    pub price: i32,
}

impl From<CatalogueBook> for BookObject {
    fn from(value: CatalogueBook) -> Self {
        Self {
            id: value.id,
            title: value.title,
            author: value.author,
            price: value.price,
        }
    }
}

struct Query;

#[async_graphql::Object]
impl Query {
    async fn book(&self, ctx: &Context<'_>, id: i32) -> FieldResult<Option<BookObject>> {
        let mut app_state = ctx.data::<SharedState>()?.lock().await;
        let response = app_state
            .catalogue_client
            .get_book(GetBookRequest { id })
            .await?;
        Ok(response.into_inner().book.map(|book| book.into()))
    }

    async fn books(&self, ctx: &Context<'_>) -> FieldResult<Vec<BookObject>> {
        let mut app_state = ctx.data::<SharedState>()?.lock().await;
        let response = app_state.catalogue_client.list_books(()).await?;
        Ok(response
            .into_inner()
            .books
            .iter()
            .map(|b| b.clone().into())
            .collect())
    }
}

async fn graphiql() -> impl IntoResponse {
    axum::response::Html(GraphiQLSource::build().endpoint("/").finish())
}

/// コマンドライン引数
#[derive(Debug, Parser)]
struct Args {
    /// カタログサービスリッスンポート
    #[clap(long)]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let catalogue_address = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), args.port);

    let catalogue_client = CatalogueClient::connect(format!("http://{}", catalogue_address))
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    let app_state = Arc::new(Mutex::new(AppState { catalogue_client }));

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(app_state)
        .finish();

    let origins = [HeaderValue::from_static("http://localhost:5173")];
    let cors_layer = CorsLayer::new()
        .allow_origin(origins)
        //.allow_credentials(true)
        .allow_headers([header::CONTENT_TYPE])
        .allow_methods([Method::GET, Method::POST]);

    let app = Router::new()
        .route(
            "/",
            routing::get(graphiql).post_service(GraphQL::new(schema.clone())),
        )
        .route_service("/graphql", GraphQL::new(schema))
        .layer(cors_layer);

    println!("GraphiQL IDE: http://localhost:4000");

    axum::serve(TcpListener::bind("127.0.0.1:4000").await.unwrap(), app)
        .await
        .unwrap();

    Ok(())
}

#[derive(Debug, Clone)]
struct AppState {
    /// カタログサービスアドレス
    catalogue_client: CatalogueClient<tonic::transport::Channel>,
}

type SharedState = Arc<Mutex<AppState>>;
