# エクステンション

`async-graphql`は、オリジナルのソースコードを修正する必要なく、エクステンションで拡張する能力があります。
多くの機能はこの方法で追加されており、多くのエクステンションが存在します。

## エクステンションを定義する方法

`async-graphql`のエクステンションは、関連する`Extension`トレイトを実装することで定義されます。
`Extension`トレイトは、`async-graphql`を介してGraphQLリクエストに応答するために使用されるいくつかのステップに、独自のコードを挿入することができます。
`Extensions`を使用して、アプリケーションは、着信リクエストまたは出力レスポンスに対して振る舞いを追加するために、GraphQLのリクエストサイクルをフックします。

`Extensions`は他のフレームワークのミドルウェアに似ていて、エクステンションを使用するときは、**それがすべてのGraphQLリクエストに対して実行される**ことに注意する必要があります。

すべてのステップで、現在のリクエストの実行に関するデータが`ExtensionContext`に供給されます。
コード内でそれが構築される方法を自由に確認してください。
それに関するドキュメントはすぐに公開されます。

### ミドルウェアについて

知らない人のために、ミドルウェアが何であるか深堀りしましょう。

```rust
async fn middleware(&self, ctx: &ExtensionContext<'_>, next: NextMiddleware<'_>) -> MiddlewareResult {
    // ミドルウェアのロジック

    /*
     * ミドルウェアの最後のステップで、次のミドルウェアの実行をトリガーする`next`関数を呼び出します。
     * これは、JavaScriptの`callback`のようです。
     */
    next.run(ctx).await
}
```

確認した通り、`Middleware`は最後に`next`関数を呼び出す単なる関数ですが、最初に`next.run`関数があるミドルウェアを実行することもできます。
これがややこしいところです。
ロジックを置く場所、`next.run`を呼び出す場所によって、ロジックは同じ実行順序になりません。

ロジックコードに応じて、`next.run`呼び出しの前か後で、そのロジックを処理する必要があります。
もし、ミドルウェアについてより情報が必要な場合、Webに多くの情報があります。

### クエリの処理

クエリの処理を完成させるいくつかのステップがあり、これらのフックに基づいてエクステンションを作成できます。

#### request

最初に、リクエストを受け取ったとき、それがサブスクリプションでない場合、呼び出される最初の関数は`request`で、それは最初のステップであり、着信リクエストによって呼び出された関数で、ユーザにレスポンスを出力する関数でもあります。

`request`のデフォルト実装は次のとおりです。

```rust
async fn request(&self, ctx: &ExtensionContext<'_>, next: NextRequest<'_>) -> Response {
    next.run(ctx).await
}
```

ロジックコードを置く場所に応じて、クエリが処理される開始時または終了時に実行されます。

```rust
async fn request(&self, ctx: &ExtensionContext<'_>, next: NextRequest<'_>) -> Response {
    // ここにあるコードは、`prepare_request`が実行される前に実行されます。
    let result = next.run(ctx).await;
    // このfutureの完了後のコードは、クエリが処理された後、結果をユーザーに送信する直前に実行されます。
    result
}
```

#### prepare_request

`request`の直後、フックできる`prepare_request`ライフサイクルがあります。

```rust
async fn prepare_request(
    &self,
    ctx: &ExtensionContext<'_>,
    request: Request,
    next: NextPrepareRequest<'_>,
) -> ServerResult<Request> {
    // ここのコードは、`prepare_request`が実行される前に実行され、それは`request`ライフサイクルフックの直後です。
    let result = next.run(ctx, request).await;
    // ここのコードは、`prepare_request`の直後に実行されます。
    result
}
```

#### parse_query

`parse_query`は、クエリに対するGraphQLの`ExecutableDocument`を作成して、クエリがGraphQL仕様に準拠しているか確認します。
普通、`async-graphql`で実装された仕様は、最新の安定版（2021年10月）に準拠しています。

```rust
/// クエリを解析するときに呼び出されます。
async fn parse_query(
    &self,
    ctx: &ExtensionContext<'_>,
    // 生クエリ
    query: &str,
    // 変数
    variables: &Variables,
    next: NextParseQuery<'_>,
) -> ServerResult<ExecutableDocument> {
    next.run(ctx, query, variables).await
}
```

#### validation

`validation`ステップは、（`validation_mode`に基づいて）クエリが従うべきルールを確認して、クエリがムッこうである理由に関するデータをクライアントに提供します。

```rust
/// クエリを検証するときに呼び出されます。
async fn validation(
    &self,
    ctx: &ExtensionContext<'_>,
    next: NextValidation<'_>,
) -> Result<ValidationResult, Vec<ServerError>> {
    next.run(ctx).await
}
```

#### execute

`execute`ステップは巨大で、`Query`に対してはそれぞれのリゾルバーを同時に、`Mutation`に対しては順番に呼び出すことで、クエリの実行を開始します。

```rust
/// クエリを実行するときに呼び出されます。
async fun execute(&self, ctx: &ExtensionContext<'_>, operation_name: Option<&str>, next: NextExecute<'_>) -> Response {
    // クエリ全体の解決を開始する前に実行されます。
    let result = next.run(ctx, operation_name).await;
    // クエリ全体が解決された後に実行されます。
    result
}
```

#### resolve

`resolve`ステップは、それぞれのフィールドに対して起動されます。

```rust
/// フィールドを解決するときに呼び出されます。
async fn resolve(
    &self,
    ctx: &ExtensionContext<'_>,
    info: ResoleInfo<'_>,
    next: NextResolve<'_>,
) -> ServerResult<Option<Value>> {
    // フィールドを解決する前のロジックです。
    let result = next.run(ctx, info).await;
    // フィールドを解決した後のロジックです。
    result
}
```

#### subscribe

`subscribe`ライフサイクルは、`request`と同じ振る舞いをしますが、`Subscription`用です。

```rust
/// 購読リクエストによって呼び出されます。
fn subscribe<'s>(
    &self,
    ctx: &ExtensionContext<'_>,
    stream: BoxStream<'s, Response>,
    next: NextSubscribe<'_>,
) -> BoxStream<'s, Response> {
    next.run(ctx, stream)
}
```

## 利用可能なエクステンション

`async-graphql`には、GraphQLサーバーを強化する利用可能なエクステンションが多くあり、これらのいくつかのドキュメントを示します。

### Analyzer

*リポジトリで利用可能です。(Available in the repository)*

`analyzer`エクステンションは、それぞれのクエリのレスポンス拡張フィールドに`complexity`と`depth`を含めます。

### Apollo Persisted Queries

*リポジトリで利用可能です。(Available in the repository)*

巨大なクエリに対してネットワーク性能を改善するために、このエクステンションを有効にできます。
このエクステンションを有効にすることで、それぞれの一意なクエリは一意な識別子に関連付けされるため、クライアントはリクエストサイズを減らすために、対応するクエリ文字列の代わりに、この識別子を送信できます。

このエクステンションは、任意のキャッシュ戦略の仕様を強制しないため、キャッシュ連略を選択でき、単に`CacheStrategy`トレイトを実装する必要があるだけです。

```rust
#[async_trait::async_trait]
pub trait CacheStorage: Send + Sync + Clone + 'static {
    /// `key`でクエリをロードします。
    async fn get(&self, key: String) -> Option<String>;
    /// `key`でクエリを保存します。
    async fn set(&self, key: String, query: String);
}
```

参照: [Apollo doc - Persisted Queries](https://www.apollographql.com/docs/react/api/link/persisted-queries/)

### Apollo Tracing

*リポジトリで利用可能です。(Available in the repository)*

Apollo Tracingは、クエリの分析データを含むエクステンションです。
このエクステンションは、古くて現在非推奨となっている[Apollo Tracing Spec](https://github.com/apollographql/apollo-tracing)に準拠するように動作します。
新しいApollo Reporting Protocolを確認したい場合は、Apollo Studio用の[async-graphql Apollo studio extension](https://github.com/async-graphql/async_graphql_apollo_studio_extension)によって実装されています。

### Apollo Studio

*[async-graphql/async-graphql_apollo_studio_extension](https://github.com/async-graphql/async_graphql_apollo_studio_extension)で利用できます。*

Apollo Studioは、組織のグラフを構築、検証、そして安全にすることを支援するクラウドプラットフォームです（説明は公式ドキュメントからです）。
これは、GraphQLスキーマを監視し、チームと連携できるサービスです。
`async-graphql`は、[async-graphql-extension-apollo-tracing](https://github.com/async-graphql/async_graphql_apollo_studio_extension)と[Crates.io](https://crates.io/crates/async-graphql-extension-apollo-tracing)で利用可能な公式の[Apollo Specification](https://www.apollographql.com/docs/studio/setup-analytics/#third-party-support)を実装する拡張機能を提供します。

### Logger

*リポジトリで利用可能です。(Available in the repository)*

Loggerは、`async-graphql`にいくつかのロギング機能を追加するエクステンションです。
これは、独自のエクステンションを作成する方法を学ぶ良い例にもなります。

### OpenTelemetry

*リポジトリで利用可能です。(Available in the repository)*

OpenTelemetryは、[opentelemetry crate](https://crates.io/crates/opentelemetry)ととの統合を提供するエクステンションで、アプリケーションが`async-graphql`から分散したトレースとメトリックを収集できるようにします。

### Tracing

*リポジトリで利用可能です。(Available in the repository)*

Tracingは、いくつかのトレーシング機能を`async-graphql`に追加するエクステンションです。
少し`Logger`エクステンションと似ています。
