use std::path::PathBuf;

/// ビルドする前に実行される関数
///
/// Protocol Buffersで使用するデータ構造やサービスの仕様を、Rustのデータ構造でファイルに出力する。
/// ファイルは`target/debug/catalogue-service-<alpha-numeric>/out`ディレクトリに出力される。
/// 出力されたデータ構造は次のように利用する。
///
/// ```
/// mod book {
///     tonic::include_proto!("book");
/// }
///
/// use book:: {
///     catalogue_server::{Catalogue, CatalogueServer},
///     GetBookRequest, GetBookResponse,
/// };
/// ```
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let descriptor_path =
        PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("book_descriptor.bin");
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(&descriptor_path)
        .compile_protos(&["proto/book/catalogue.proto"], &["proto/book"])?;

    Ok(())
}
