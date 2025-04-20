pub mod book {
    tonic::include_proto!("book");
    pub use catalogue_client::*;
    pub use catalogue_server::*;
}
