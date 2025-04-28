//! # 2.2.1.2. gRPCを使って通信するバックエンドの作成
//!
//! ```sh
//! $ grpcurl -plaintext localhost:57631 list
//! book.Catalogue
//! grpc.reflection.v1.ServerReflection
//!
//! $grpcurl -plaintext localhost:57631 list book.Catalogue
//! book.Catalogue.GetBook
//!
//! $grpcurl -plaintext -d '{"id": 1}' localhost:57631 book.Catalogue.GetBook
//! {
//!   "id": 1,
//!   "title": "The Awakening",
//!   "author": "Kate Chopin",
//!   "price": 1000
//! }
//! ```
use std::net::{SocketAddr, TcpListener};

use domain::models::{Book, DUMMY_BOOKS};

mod book {
    tonic::include_proto!("book");
}

use book::{
    GetBookRequest, GetBookResponse, ListBooksResponse,
    catalogue_server::{Catalogue, CatalogueServer},
};

impl From<Book> for book::Book {
    fn from(value: Book) -> Self {
        Self {
            id: value.id,
            title: value.title,
            author: value.author,
            price: value.price,
        }
    }
}

impl From<Book> for GetBookResponse {
    fn from(value: Book) -> Self {
        GetBookResponse {
            book: Some(value.into()),
        }
    }
}

/// gRPCサーバーからのレスポンスの型
type ApiResult<T> = Result<tonic::Response<T>, tonic::Status>;

/// カタログサービス
struct CatalogueService;

#[tonic::async_trait]
impl Catalogue for CatalogueService {
    async fn get_book(
        &self,
        request: tonic::Request<GetBookRequest>,
    ) -> ApiResult<GetBookResponse> {
        let book = get_book_by_id(request.get_ref().id).await;
        match book {
            Some(book) => Ok(tonic::Response::new(GetBookResponse {
                book: Some(book.into()),
            })),
            None => Ok(tonic::Response::new(GetBookResponse { book: None })),
        }
    }

    async fn list_books(&self, _: tonic::Request<()>) -> ApiResult<ListBooksResponse> {
        let books = get_books().await;
        Ok(tonic::Response::new(ListBooksResponse {
            books: books.into_iter().map(|book| book.into()).collect(),
        }))
    }
}

/// リポジトリから書籍を取得するダミー実装
async fn get_book_by_id(id: i32) -> Option<Book> {
    Some(DUMMY_BOOKS[(id - 1) as usize].clone())
}

async fn get_books() -> Vec<Book> {
    DUMMY_BOOKS.iter().cloned().collect()
}

/// OSに利用可能なポートを割り当ててもらう。
///
/// macOSでは、gRPCのデフォルトポートである50051を`launchd`が使用しているため、別のポートを使用する。
///
/// # 戻り値
///
/// - ポート番号
fn bind_any_port() -> std::io::Result<SocketAddr> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    // let listener = TcpListener::bind("127.0.0.1:59061")?;
    listener.local_addr()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let socket = bind_any_port().map_err(|e| anyhow::anyhow!(e))?;
    let catalogue_service = CatalogueService;

    println!("Listen: {}", socket);

    let book_reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(tonic::include_file_descriptor_set!(
            "book_descriptor"
        ))
        .build_v1()
        .map_err(|e| anyhow::anyhow!(e))?;

    tonic::transport::Server::builder()
        .add_service(CatalogueServer::new(catalogue_service))
        .add_service(book_reflection_service)
        .serve(socket)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}
