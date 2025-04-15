# クイックスタート

## 依存ライブラリの追加

```toml
[dependencies]
async-graphql = "4.0"
async-graphql-actix-web = "4.0" # If you need to integrate into actix-web
async-graphql-warp = "4.0" # If you need to integrate into warp
async-graphql-tide = "4.0" # If you need to integrate into tide
```

> ```toml
> [dependencies]
> async-graphql = "7"
> async-graphql-axum = "7"
> ```

## スキーマの記述

GraphQLのスキーマは、必須となるクエリ、オプションのミューテーション、そしてオプションのサブスクリプションを含んでいます。
これらのオブジェクト型は、Rust言語の構造体を使用して記述されます。
構造体のフィールドは、GraphQLオブジェクトのフィールドに対応します。

`async-graphql`は、`i32`、`f64`、`Option<T>`、`Vec<T>`などのような一般的なデータ型をGraphQLの型にマッピングする実装を持っています。
また、GraphQLでスカラーと呼ばれる[これらの基本的な型を拡張](https://async-graphql.github.io/async-graphql/en/custom_scalars.html)できます。

ここに、`a`と`b`の合計を返す、ちょうど1つのクエリを提供する単純な例を示します。

```rust
use async_graphql::*;

struct Query;

#[Object]
impl Query {
    /// aとbの合計を返す。
    async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}
```

## クエリの実行

上記例において、ミューテーションまたはサブスクリプションはなく、クエリのみがあるため、`EmptyMutation`と`EmptySubscription`を使用してスキーマを作成して、クエリを実行するために`Schema::execute`を呼び出します。

```rust
let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
let res = schema.execut("{ add(a: 10, b: 20) }").await;
```

## JSONでクエリ結果を出力

```rust
let json = serde_json::to_string(&res);
```

## Webサーバー統合

すべての例は、examplesディレクトリ内に配置された[サブディレクトリ](https://github.com/async-graphql/examples)にあります。

```sh
git submodule update # サンプルリポジトリを更新
cd examples &&cargo run --bin [name]
```

詳細は、サブディレクトリのREADME.mdを参照してください。
