# 統合

`async-graphql`は、いくつか一般的なRust Webサーバーをサポートしています。

- Poem: [async-graphql-poem](https://crates.io/crates/async-graphql-poem)
- Actix-web: [async-graphql-actix-web](https://crates.io/crates/async-graphql-actix-web)
- Warp: [async-graphql-warp](https://crates.io/crates/async-graphql-warp)
- Axum: [async-graphql-axum](https://crates.io/crates/async-graphql-axum)
- Rocket: [async-graphql-rocket](https://crates.io/crates/async-graphql-rocket)

**現在使用しているサーバーが上記リストにない場合でも、似たような機能を実行することはとても簡単です。**

## Poem

### リクエスト例

```rust
use poem::Route;
use async_graphql_poem::GraphQL;

let app = Route::new()
    .at("/ws", GraphQL::new(schema));
```

### サブスクリプション例

```rust
use poem::{get, Route};
use async_graphql_poem::GraphQLSubscription;

let app = Route::new()
    .at("/ws", get(GraphQLSubscription::new(schema)));
```

### 他の例

<https://github.com/async-graphql/examples/tree/master/poem>

## Warp

`async-graphql-warp`用に、`graphql`と`graphql_subscription`の2つの`Filter`統合が提供されています。

`graphql`フィルターは、`Query`と`Mutation`リクエストで使用されます。
それはGraphQLリクエストを抽出して、`async_graphql::Schema`と`async_graphql::Request`を出力します。
後で、他のフィルターと組み合わせするか、クエリを実行するために直接`Schema::execute`を呼び出すことができます。

`graphql_subscription`は、WebSocketサブスクリプションを実装するために使用されます。
それは`wrap::Replay`を出力します。

### リクエスト例

```rust
type MySchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
let filter = async_graphql_warp::graphql(schema).and_then(|(schema, request): (MySchema, async_graphql::Request)| async move {
    // Execute query
    let resp = schema.execute(request).await;

    // Return result
    Ok::<_, Infallible>(async_graphql_warp::GraphQLResponse::from(resp))
});
warp::serve(filter).run(([0, 0, 0, 0], 8000)).await;
```

### サブスクリプション例

```rust
let schema = Schema::new(QueryRoot, EmptyMutation, SubscriptionRoot);
let filter = async_graphql_warp::graphql_subscription(schema);
warp::serve(filter).run(([0, 0, 0, 0], 8000)).await;
```

### 他の例

<https://github.com/async-graphql/examples/tree/master/warp>

## Actix-web

### リクエスト例

`actix_web::App`を定義するとき、データとしてスキーマ内に渡す必要があります。

```rust
use actix_web::{web, HttpRequest, HttpResponse};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};

async fn index(
    // ここでスキーマにアクセスできます。
    schema: web::Data<Schema<Query, EmptyMutation, EmptySubscription>>,
    request: GraphQLRequest,
) -> web::Json<GraphQLResponse> {
    web::Json(schema.execute(request.into_inner()).await.into())
}
```

### サブスクリプション例

```rust
use actix_web::{web, HttpRequest, HttpResponse};
use async_graphql_actix_web::GraphQLSubscription;

async fn index_ws(
    schema: web::Data<Schema<Query, EmptyMutation, EmptySubscription>>,
    req: HttpRequest,
    payload: web::Payload,
) -> actix_web::Result<HttpResponse> {
    GraphQLSubscription::new(Schema::clone(&*schema)).start(&req, payload)
}
```

### 他の例

<https://github.com/async-graphql/examples/tree/master/actix-web>
