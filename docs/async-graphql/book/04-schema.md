# スキーマ

基本的な型を定義した後、それらを組み合わせてスキーマを定義する必要があります。
スキーマは、クエリオブジェクト、ミューテーションオブジェクト、そしてサブスクリプションオブジェクトの3つの型で構成され、ミューテーションオブジェクトとサブスクリプションオブジェクトはオプションです。

スキーマが作成されたとき、`async-graphql`はすべてのオブジェクトグラフを縦断して、すべての型を登録します。
これは、もしGraphQLオブジェクトが定義されていても参照されない場合、そのオブジェクトはスキーマに公開されないことを意味します。

## クエリとミューテーション

### クエリルートオブジェクト

クエリルートオブジェクトは、他のオブジェクトと同様の定義を持つGraphQLオブジェクトです。
クエリオブジェクトのすべてのフィールドのリゾルバー関数は、同時に実行されます。

```rust
use async_graphql::*;

struct Query;

#[Object]
impl Query {
  async fn user(&self, username: String) -> Result<Option<User>> {
    // データベースからユーザーを検索します。
  }
}
```

### ミューテーションルートオブジェクト

また、ミューテーションルートオブジェクトもGraphQLオブジェクトですが、それは順番に実行されます。
他のミューテーションが後続するミューテーションは、最初のミューテーションが完了した後でのみ、実行されます。

次のミューテーションルートオブジェクトは、ユーザーの登録とログインの例を提供しています。

```rust
use async_graphql::*;

struct Mutation;

#[Object]
impl Mutation {
    async fn sign_up(&self, username: String, password: String) -> Result<bool> {
        // ユーザーを登録します。
    }

    async fn login(&self, username: String, password: String) -> Result<String> {
        // ユーザーがログインします（トークンを生成します）。
    }
}
```

## サブスクリプション

サブスクリプションルートオブジェクトの定義は、他のルートオブジェクトとは少し異なります。
そのリゾルバー関数は、常に[Stream](https://docs.rs/futures-core/~0.3/futures_core/stream/trait.Stream.html)または`Result<Stream>`を返し、そのフィールドパラメーターは通常データフィルタ条件に使用されます。

次の例は、整数ストリームを購読して、それは1秒に1つの整数を生成します。
パラメーター`step`は、1をデフォルトに持つ整数のステップサイズを指定します。

```rust
use async_graphql::*;

struct Subscription;

#[Subscription]
impl Subscription {
    async fn integers(&self, #[graphql(default = 1)] step: i32) -> impl Stream<Item = i32> {
        let mut value = 0;
        tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(Duration::from_secs(1)))
            .map(move |_| {
                value += step;
                value
            })
    }
}
```

## SDLエクスポート

`Schema::sdl()`メソッドを使用して、スキーマをスキーマ定義言語（SDL）にエクスポートできます。

```rust
use async_graphql::*;

struct Query;

#[Object]
impl Query {
    async fn add(&self, u: i32, v: i32) -> i32 {
        u + v
    }
}

let schema = Schema::build(Query, EmptyMutation, EmptySubscription).finish();

// Print the schema in SDL format
println!("{}", &schema.sdl());
```
