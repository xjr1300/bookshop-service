# 高度なトピック

## カスタムスカラー

`async-graphql`において、ほとんどの一般的なスカラー型は組み込まれていますが、独自のスカラー型を作成することもできます。

`async_graphql::Scalar`を使用すると、実装時にスカラーのサポートを追加できます。
単にパースして出力する関数を実装する必要があるのみです。

次の例は、64ビット整数スカラーを定義して、その入力と出力は文字列です。

```rust
use async_graphql::*;

struct StringNumber(i64);

#[Scalar]
impl ScalarType for StringNumber {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(value) = &value {
            // 整数値をパース
            Ok(value.parse().map(StringNumber)?)
        } else {
            // 型がマッチしない場合
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_string())
    }
}
```

### スカラーを定義するscalar!マクロの使用

型が`serde::Serialize`と`serde::Deserialize`を実装している場合、より単純にスカラーを定義するためにこのマクロを使用できます。

```rust
#[derive(Serialize, Deserialize)]
struct MyValue {
    a: i32,
    b: HashMap<String, i32>,
}

scalar!(MyValue);

// `MV`に名前を変更する場合
// scalar!(MyValue, "MV");

// `MV`に名前を変更し、説明を追加する場合
// scalar!(MyValue, "MV", "This is my value");
```

## N+1クエリの最適化

一部のGraphQLクエリが、100個のデータベースクエリを実行する可能性があり、ほとんどが繰り返しデータであることに気づいたことがあるでしょうか？
その理由と修正方法を確認しましょう。

### クエリ解決

次のような単純なクエリがあることを想像してください。

```graphql
query {
  todos {
    users {
      name
    }
  }
}
```

そして、`User`リソルバーは次のようになっています。

```rust
struct User {
    id: u64,
}

#[Object]
impl User {
    async fn name(&self, ctx: &Context<'_>) -> Result<String> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let (name,): (String,) = sqlx::query_as("SELECT name FROM user WHERE id = $1")
            .bind(self.id)
            .fetch_one(pool)
            .await?;
        Ok(name)
    }
}
```

クエリエグゼキューターは、`SELECT * FROM todo`を実行してN個のTodoを返す`Todos`リゾルバーを呼び出します。
その後、それぞれのTodoに対して同時に`User`リゾルバーが`SELECT * FROM user WHERE id = toto.user_id`を呼び出します。

例えば・・・

```sql
SELECT id, todo, user_id FROM todo
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
SELECT name FROM user WHERE id = $1
```

多く`SELECT name FROM user WHERE id = $1`を実行した後、ほとんどの`Todo`オブジェクトは同じユーザーに属するため、これらのコードを最適化する必要があります。

### データローダー

クエリをグループ化して、重複したクエリを排除する必要があります。
`DataLoader`はこれを行えます。
[facebook](https://github.com/facebook/dataloader)は、リクエストスコープバッチとキャッシング解決を提供しています。

> facebook gives a request-scope batch and caching solution.

次は、クエリを最適化する`DataLoader`を使用する単純な例で、[GitHubに完全なコード例があります](https://github.com/async-graphql/examples/tree/master/tide/dataloader-postgres)。

```rust
use std::sync::Arc;

use async_graphql::*;
use async_graphql::dataloader::*;

struct UserNameLoader {
    pool: sqlx::PgPool,
}

impl Loader<u64> for UserNameLoader {
    type Value = String;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[u64]) -> Result<HashMap<u64, Self::Value>, Self::Error> {
        Ok(sqlx::query_as("SELECT name FROM WHERE id = ANY($1)")
            .bind(keys)
            .fetch(&self.pool)
            .map_ok(|name: String| name)
            .map_err(Arc::new)
            .try_collect().await?)
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
struct User {
    id: u64,
}

#[ComplexObject]
impl User {
    async fn name(&self, ctx: &Context<'_>) -> Result<String> {
        let loader = ctx.data_unchecked::<DataLoader<UserNameLoader>>();
        let name: Option<String> = loader.load_one(self.id).await?;
        mane.ok_or_else(|| "Not found".into())
    }
}
```

`ctx`内で`UserNameLoader`を公開するために、例えば`async_std::task::spawn`のようなタスク生成者といっしょに、スキーマにそれを登録する必要があります。

```rust
let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
    .data(DataLoader::new(
        UserNameLoader,
        async_std::task::spawn, // または`tokio::spawn`
    ))
    .finish();
```

最後に、たった2つのSQLが、結果を問い合わせするために必要とされます。

```sql
SELECT id, todo, user_id FROM todo
SELECT name FROM user WHERE id IN (1, 2, 3, 4)
```

### 複数のデータ型の実装

次のように、同じ`Loader`に対して複数のデータ型を実装できます。

```rust
struct PostgresLoader {
    pool: sqlx::PgPool,
}

impl Loader<UserId> for PostgresLoader {
    type Value = User;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[UserId]) -> Result<HashMap<UserId, Self::Value>, Self::Error> {
        // データベースからユーザーをロード
    }
}

impl Loader<TodoId> for PostgresLoader {
    type Value = Todo;
    type Error = sqlx::Error;

    async fn load(&self, keys: &[TodoId]) -> Result<HashMap<TodoId, Self::Value>, Self::Error> {
        // データベースからTodoをロード
    }
}
```

## カスタムディレクティブ

GraphQLには、実行可能ディレクティブと型システムディレクティブの2種類のディレクティブがあります。
実行可能ディレクティブは、クライアントが操作内で動作を変更するために使用されます（組み込みの`@include`ディレクティブや`@skip`ディレクティブなど）。
型システムディレクティブは、型に関する追加情報を提供し、サーバーの動作を変更する可能性があります（`@deprecated`ディレクティブや`@oneOf`ディレクティブなど）。
`async-graphql`では、両方のタイプのカスタムディレクティブを宣言できますが、それぞれに異なる制限があります。

### 実行可能ディレクティブ

カスタム実行可能ディレクティブを作成するには、`CustomDirective`トレイトを実装し、`Directive`マクロを使用して、ディレクティブのパラメータを受け取り、ディレクティブのインスタンスを返すファクトリー関数を生成する必要があります。

現在、`async-graphql`は`FIELD`にあるカスタム実行可能ディレクティブのみをサポートしています。

```rust
struct ConcatDirective {
    value: String,
}

#[async_trait::async_trait]
impl CustomDirective for ConcatDirective {
    async fn resolve_field(&self, _ctx: &Context<'_>, resolve: ResolveFut<'_>) -> ServerResult<Option<Value>> {
        resolve.await.map(|value| {
            value.map(|value| match value {
                Value::String(str) => Value::String(str + &self.value),
                _ => value,
            })
        })
    }
}

#[Directive(location = "Field")]
fn concat(value: String) -> impl CustomDirective {
    ConcatDirective { value }
}
```

スキーマを構築するときにディレクティブを登録します。

```rust
let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
    .directive(concat)
    .finish();
```

### 型システムディレクティブ

型システムディレクティブを作成するために、関数で`#[TypeDirective]`マクロを使用できます。

```rust
#[TypeDirective(
    location = "FieldDefinition",
    location = "Object",
)]
fn testDirective(scope: String, input: u32, opt: Option<u64>) {}
```

現在、`FieldDefinition`と`Object`の場所のみがサポートされており、どちらか一方、または両方を選択できます。
ディレクティブを宣言した後、（関数をインポートした後）関連する場所に次のように適用できます。

```rust
#[derive(SimpleObject)]
#[graphql(
    directive = testDirective::apply("simple object type".to_string(), 1, Some(3))
)]
struct SimpleValue {
    #[graphql(
        directive = testDirective::apply("field and param with \" symbol".to_string(), 2, Some(3))
    )]
    some_data: String,
}
```

この例は次のようなスキーマを生成します。

```graphql
type SimpleValue @testDirective(scope: "simple object type", input: 1, opt: 3) {
  someData: String! @testDirective(scope: "field and param with \" symbol", input: 2, opt: 3)
}

directive @testDirective(scope: String!, input: Int!, opt: Int) on FIELD_DEFINITION | OBJECT
```

注意: Apollo Federationの`@composeDirective`で型システムディレクティブを使用するには、[Federationのドキュメント](https://async-graphql.github.io/async-graphql/en/apollo_federation#composeDirective)を参照してください。

### Apolloフェデレーション

Apolloフェデレーションは、複数のGraphQLサーバー、またはサブグラフを1つのスーパーグラフに組み合わせるGraphQLアーキテクチャです。
詳細は[公式ドキュメント](https://www.apollographql.com/docs/apollo-server/federation/)で確認できます。

---

フェデーレーションの完全な例は、[フェデレーション例](https://github.com/async-graphql/examples/tree/master/federation)を確認してください。

---

#### フェデレーションサポートを有効化

`async-graphql`は、Apolloフェデレーションのバージョン2のすべての機能をサポートしています。
もし、`#[graphql(entity)]`を付与されたすべてのリゾルバーがスキーマ内で発見された場合、サポートが自動的に有効になります。
これを手作業で有効にするために、`SchemaBuilder`で`enable_federation`メソッドを使用してください。

```rust
fn main() {
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .enable_federation()
        .finish();
    // 選択したサーバーを起動
}
```

これは、フェデレーションバージョン2を有効にする[`@link`ディレクティブ](https://www.apollographql.com/docs/federation/federated-types/federated-directives#link)をスキーマに定義します。

#### エンティティと@key

[エンティティ](https://www.apollographql.com/docs/federation/entities)はフェデレーションの重要な機能であり、それは複数のサブグラフが同じ型にフィールドを提供できるようにします。
エンティティは、少なくとも1つ以上の[`@key`ディレクティブ](https://www.apollographql.com/docs/federation/entities#1-define-a-key)を持つGraphQLの型です。
型に対して`@key`を作成するために、`#[graphql(entity)]`属性を使用して参照リゾルバーを作成します。
このリゾルバーは、`Query`構造体で定義されなければなりませんが、スキーマのフィールドとして表示されません。

---

参照リゾルバーは個々のエンティティを探しますが、実装で**[dataloader](https://async-graphql.github.io/async-graphql/en/dataloader.html)を使用することが重要**です。
フェデレーションルーターはバッチでエンティティを探して、すぐにそれはN+1性能問題を引き起こす可能性があります。

---

#### 例

```rust
struct Query;

#[Object]
impl Query {
    #[graphql(entity)]
    async fn find_user_by_id(&self, id: ID) -> User {
        User { id }
    }

    #[graphql(entity)]
    async fn find_user_by_id_with_username(&self, #[graphql(key)] id: ID, username: String) -> User {
        User { id }
    }

    #[graphql(entity)]
    async fn find_user_by_id_and_username(&self, id: ID, username: String) -> User {
        User { id }
    }
}
```

**`User`オブジェクトを検索する、これら3つの検索関数の違いに注意してください。**

- `find_user_by_id`: `User`オブジェクトを見つけるために`id`を使用して、`User`のキーは`id`です。
- `find_user_by_id_with_username`: `User`オブジェクトを見つけるために`id`を使用して、`User`のキーは`id`で、`User`オブジェクトの`username`フィールドの値が要求されます（例えば、`@external`と`@requires`を介して）。
- `find_user_by_id_and_username`: `User`オブジェクトを見つけるために`id`と`username`を使用して、`User`のキーは`id`と`username`です。

スキーマの結果は次のようになります。

```graphql
type Query {
  # これらのフィールドはユーザーに公開されず、それらはエンティティを解決するためにルーターによってのみ使用されます。
  _entities(representations: [_Any!]!): [_Entity]!
  _service: _Service!
}

type User @key(fields: "id") @key(fields: "id username") {
  id: ID!
}
```

#### 複合主キーの定義

1つの主キーは複数のフィールド、さらにネストしたフィールドで構成でき、ネストした主キーを実装するために`InputObject`を使用できます。

次の例において、`User`オブジェクトの主キーは`key { a b }`です。

```rust
#[derive(InputObject)]
struct NestedKey {
    a: i32,
    b: i32,
}

struct Query;

#[Object]
impl Query {
    #[graphql(entity)]
    async fn find_user_by_key(&self, key: NestedKey) -> User {
        let NestedKey { a, b } = key;
        User { key: Key {a, b}}
    }
}
```

スキーマの結果は次のようになります。

```graphql
type Query {
  # These fields will not be exposed to users, they are only used by the router to resolve entities
  _entities(representations: [_Any!]!): [_Entity]!
  _service: _Service!
}

type User @key(fields: "key { a b }") {
  key: Key!
}

type Key {
  a: Int!
  b: Int!
}
```

#### 解決しないエンティティの作成

エンティティを参照する必要があるが、エンティティにフィールドを追加したくないときがあります。
これは、分離したサブグラフからのデータをリンクしたいが、どちらのサブグラフにもすべてのデータが含まれていないとき、特に便利です。

もし、Apolloドキュメントにある[製品とレビューサブグラフ例](https://www.apollographql.com/docs/federation/entities/#referencing-an-entity-without-contributing-fields)を実装したい場合、レビューサブグラフに対して次の型を作成します。

```rust
#[derive(SimpleObject)]
struct Review {
    product: Product,
    score: u64,
}

#[derive(SimpleObject)]
#[graphql(unresolvable)]
struct Product {
    id: u64,
}
```

これは、レビューサブグラフの`Product`型に`@key(fields: "id", resolvable: false)`を追加します。

複合キーにネストされたフィールドがあるなど、より複雑なエンティティのキーに対して、ディレクティブ内のフィールドを上書きできます。

```rust
#[derive(SimpleObject)]
#[graphql(unresolvable = "id organization { id }")]
struct User {
    id: u64,
    organization: Organization,
}

#[derive(SimpleObject)]
struct Organization {
    id: u64,
}
```

しかし、これらのフィールドが存在することを確認されないことに注意することが重要です。

#### @shareable

複数のサブグラフが解決できることを示すために、型やフィールドに[`@shareableディレクティブ](https://www.apollographql.com/docs/federation/federated-types/federated-directives#shareable)を適用します。
