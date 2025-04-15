# ユーティリティ

## フィールドガード

`Object`、`SimpleObject`、`ComplexObject`そして`Subscription`のフィールドに対して`guard`を定義でき、それはリゾルバー関数を呼び出す前に実行され、もしそれが失敗した場合はエラーが返されます。

```rust
#[derive(Eq, PartialEq, Copy, Clone)]
enum Role {
    Admin,
    Guest,
}

struct RoleGuard {
    role: Role,
}

impl RoleGuard {
    fn new(role: Role) -> Self {
        Self { role }
    }
}

impl Guard for RoleGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        if ctx.data_opt::<Role>() == Some(&self.role) {
            Ok(())
        } else {
            Err("Forbidden".into())
        }
    }
}
```

`guard`属性を付けてそれを使用します。

```rust
#[derive(SimpleObject)]
struct Query {
    // Adminのみ許可
    #[graphql(guard = "RoleGuard::new(Role::Admin)")]
    value1: i32,
    // AdminとGuestに許可
    #[graphql(guard = "RoleGuard::new(Role::Admin).or(RoleGuard::new(Role::Guest))")]
    value2: i32,
}
```

## パラメーター値の使用

ガードはフィールドパラメーターを使用する必要があることがよくあり、次のようにガードを作成するときにパラメーター値を渡す必要があります。

```rust
struct EqGuard {
    expect: i32,
    actual:i32,
}

impl EqGuard {
    fn new(expect: i32, actual: i32) -> Self {
        Self { expect, actual }
    }
}

impl Guard for EqGuard {
    async fn check(&self, _ctx: &Context<'_>) -> Result<()> {
        if self.expect != self.actual {
            Err("Forbidden".into())
        } else {
          Ok(())
        }
    }
}

struct Query;

#[Object]
impl Query {
    #[graphql(guard = "EqGuard::new(100, value)")]
    async fn get(&self, value: i32) -> i32 {
        value
    }
}
```

## 入力値バリデーター

`async-graphql`は、組み込みで一般的なバリデータをいくつか持っており、`Object`フィールドまたは`InputObject`のフィールドのパラメーターでそれらを使用できます。

- `maximum=N`: 数値はNよりも大きくできない。
- `minimum=N`: 数値はNよりお小さくできない。
- `multiple_of=N`: 数値はNの倍数でなくてはならない。
- `max_items=N`: リストの長さがNよりも大きくならない。
- `min_items=N`: リストの長さがNよりも小さくならない。
- `max_length=N`: 文字列の長さがNよりも大きくならない。
- `min_length=N`: 文字列の長さがNよりも小さくならない。
- `chars_max_length=N`: ユニコード文字の数がNよりも大きくならない。
- `chars_min_length=N` ユニコード文字の数がNよりも小さくならない。
- `email`: 妥当なメールアドレス
- `url`: 妥当なURL
- `ip`: 妥当なIPアドレス
- `regex=RE`: 正規表現REにマッチ

```rust
use async_graphql::*;

struct Query;

#[Object]
impl Query {
    // nameの長さが5と等しいか大きく、10と等しいか小さい
    async fn input(&self, #[graphql(validator(min_length = 5, max_length = 10))] name: String) -> i32 {
        ...
    }
}
```

## リストの全てのメンバーの確認

`list`属性を有効にでき、バリデーターはリスト内のすべてのメンバーをチェックします。

```rust
use async_graphql::*;

struct Query;

#[Object]
impl Query {
    async fn input(&self, #[graphql(validator(list, max_length = 10))] names: Vec<String>) -> Result<i32> {
    }
}
```

## カスタムバリデーター

```rust
struct MyValidator {
    expect: i32,
}

impl MyValidator {
    pub fn new(n: i32) -> Self {
        MyValidator { expect: n }
    }
}

impl CustomValidator<i32> for MyValidator {
    fn check(&self, value: &i32) -> Result<(), InputValueError<i32>> {
        if *value == self.expect {
            Ok(())
        } else {
            Err(InputValueError::custom(format!("expect 100, actual {}", value)))
        }
    }
}

struct Query;

#[Object]
impl Query {
    // nは100と等しくなければならない。
    async fn value(
        &self,
        #[graphql(validator(custom = "MyValidator::new(100)"))] n: i32,
    ) -> i32 {
        n
    }
}
```

## キャッシュ制御

プロダクション環境は、性能を改善するためにキャッシュに依存することがよくあります。

GraphQLクエリは複数のリゾルバー関数を呼び出し、それぞれのリゾルバーは異なるキャッシュ定義を持てます。
いくつかは数秒だけキャッシュするかもしれず、いくつかは数時間キャッシュするかもしれず、いくつかはすべてのユーザーに対して同じで、いくつかはそれぞれのセッションに対して異なるかもしれません。

`async-graphql`は、それぞれのリゾルバーに対してキャッシュ時間とスコープを定義する機構を提供します。

`Object`またはそのフィールドにキャッシュパラメーターを定義できます。
次の例は、キャッシュ制御パラメーターの2つの使用方法を示しています。

キャッシュの有効期限（秒単位）を制御するために`max_age`パラメーターを使用でき、またキャッシュのスコープを制御するために`public`と`private`を使用できます。
それを指定しなかったとき、スコープはデフォルトで`public`です。

> | スコープ | 説明 |
> | --- | --- |
> | `public` | レスポンスがCDN、プロキシーサーバー、またはブラウザにキャッシュされることを許可します。ユーザー固有の情報が含まれていない場合に使用します。 |
> | `private` | レスポンスがCDN、プロキシーサーバーにキャッシュされることを許可しません。主にブラウザだけにキャッシュします。 |

複数のリゾルバーの問い合わせをしているとき、すべてのキャッシュ制御パラメーターの結果は組み合わされ、`max_age`の最小値が採用されます。
もし、オブジェクトまたはフィールドのスコープが`private`の場合、結果は`private`になります。

クエリ結果からマージされたキャッシュ制御結果を得るために`QueryResponse`を使用して、対応するHTTPヘッダーを取得するために`CacheControl::value`を呼び出せます。

```rust
#[Object(cache_control(max_age = 60))]
impl Query {
    #[graphql(cache_control(max_age = 30))]
    async fn value1(&self) -> i32 {
        1
    }

    #[graphql(cache_control(private))]
    async fn value2(&self) -> i32 {
        2
    }

    async fn value3(&self) -> i32 {
        3
    }
}
```

次はそれぞれのクエリに対応するキャッシュ制御結果です。

```graphql
# max_age=30
{ value1 }
```

```graphql
# max_age=30, private
{ value1 value2 }
```

```graphql
# max_age=60
{ value3 }
```

## カーソルコネクション

Relayのカーソルコネクション仕様は、クエリページングに対して一貫性のある方法を提供するために設計されています。
詳細は、[GraphQL Cursor connections Specification](https://facebook.github.io/relay/graphql/connections.htm)を参照してください。

`async-graphql`でカーソルコネクションを定義することは非常に簡単で、`connection::query`関数を呼び出して、クロージャー内でデータを問い合わせするだけです。

```rust
use async_graphql::*;
use async_graphql::types::connection::*;

struct Query;

#[Object]
impl Query {
    async fn numbers(&self,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<Connection<usize, i32, EmptyFields, EmptyFields>> {
        query(after, before, first, last, |after, before, first, last| async move {
            let mut start = after.map(|after| after + 1).unwrap_or(0);
            let mut end = before.unwrap_or(10000);
            if let Some(first) = first {
                end = (start + first).min(end);
            }
            if let Some(last) = last {
                start = if last > end - start {
                     end
                } else {
                    end - last
                };
            }
            let mut connection = Connection::new(start > 0, end < 10000);
            connection.edges.extend(
                (start..end).into_iter().map(|n|
                    Edge::with_additional_fields(n, n as i32, EmptyFields)
            ));
            Ok::<_, async_graphql::Error>(connection)
        }).await
    }
}
```

## エラーエクステンション

[graphql-spec](https://spec.graphql.org/June2018/#example-fce18)を引用すると・・・

---

GraphQLサービスは、キー拡張を使用してエラーに追加エントリを提供する場合があります。
このエントリを設定する場合、その値はマップである必要があります。
このエントリは、実装者が必要に応じてエラーに追加情報を追加できるように予約されており、その内容には追加の制限はありません。

---

### 例

クイックスタートとして[async-graphql example](https://github.com/async-graphql/examples/blob/master/actix-web/error-extensions/src/main.rs)を確認することを推奨します。

### 一般的な概念

`async-graphql`において、すべてのユーザーが直面するエラーは、`std::fmt::Display`によって公開されるエラーメッセージをデフォルトで提供することにより、`Error`型にキャストされます。
しかし、実際に、`Error`はエラーを拡張できる追加の情報を提供しています。

リゾルバーは次のようになります。

```rust
async fn parse_with_extensions(&self) -> Result<i32, Error> {
    Err(Error::new("MyMessage").extend_with(|_, e| e.set("details", "CAN_NOT_FETCH")))
}
```

そして、返されるレスポンスは次のようになります。

```json
{
  "errors": [
    {
      "message": "MyMessage",
      "locations": [ ... ],
      "path": [ ... ],
      "extensions": {
        "details": "CAN_NOT_FETCH",
      }
    }
  ]
}
```

### エラーエクステンション

手作業で新しい`Error`を構築することは、面倒になります。
それが、`async-graphql`がエクステンションでエラーを適切な`Error`にキャストするために2つの便利なトレイトを提供する理由です。

任意のエラーに対してエクステンションを提供する最も簡単な方法は、エラーで`extend_with`を呼び出すことです。
これは、与えられたエクステンションを使用して、その場で任意のエラーを`Error`に変換します。

```rust
use std::num::ParseIntError;

async fn parse_with_extensions(&self) -> Result<i32> {
    Ok(
      "234a"
      .parse()
      .map_err(|err: ParseIntError | err.extend_with(|_err, e| e.set("code", 404)))?
    )
}
```

#### カスタムエラーに対してエラーエクステンションを実装する

もし、エラーにエクステンションをあちこちで付与していることに気づいた場合、直接カスタムエラー型にトレイトを実装することを検討したいかもしれません。

```rust
#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("Could not find a resource")]
    NotFound,

    #[error("Server Error")]
    ServerError(String),

    #[error("No Extensions")]
    ErrorWithoutExtensions,
}

impl ErrorExtensions for MyError {
    // 基礎となるエクステンションを定義します。
    fn extend(&self) -> Error {
        Error::new(format!("{}", self)).extend_with(|err, e| match self {
            MyError::NotFound => e.set("code", "NOT FOUND"),
            MyError::ServerError(reason) => e.set("reason", reason.clone()),
            MyError::ErrorWithoutExtensions => {}
        })
    }
}
```

このように、提供されたエクステンションとともに、エラーメッセージを配信するために、単にエラーに`extend`を呼び出す必要があるのみです。

```rust
async fn parse_with_extensions_result(&self) -> Result<i32> {
    // Err(MyError::NotFound.extend())
    // Or
    Err(MyError::NotFound.extend_with(|_, e| e.set("on_the_fly", "some_more_info")))
}
```

```json
{
  "errors": [
    {
      "message": "NotFound",
      "locations": [ ... ],
      "path": [ ... ],
      "extensions": {
        "code": "NOT_FOUND",
        "on_the_fly": "some_more_info"
      }
    }
  ]
}
```

### ResultExt

`ResultExt`トレイトは、結果に直接`extend_err`を呼び出すことを可能にします。
したがって、上記コードは冗長が少なくなります。

```rust
use async_graphql::*;

async fn parse_with_extensions(&self) -> Result<i32> {
    Ok(
        "234a"
        .parse()
        .extend_err(|_, e| e.set("code", 404))?
    )
}
```

#### チェインしたエクステンション

`ErrorExtensions`と`ResultExt`は、`&E where E:std::fmt::Display`を実装しているため、互いにエクステンションをチェインでいます。

```rust
use async_graphql::*;

async fn parse_with_extensions(&self) -> Result<i32> {
    match "234a".parse() {
        Ok(n) => Ok(n),
        Err(e) => Err(e
            .extend_with(|_, e| e.set("code", 404))
            .extend_with(|_, e| e.set("details", "some more info.."))
            // キーは前のキーを上書きします。
            .extend_with(|_, e| e.set("code", 500))
        ),
    }
}
```

予期されるレスポンスは次のとおりです。

```json
{
  "errors": [
    {
      "message": "MyMessage",
      "locations": [ ... ],
      "path": [ ... ],
      "extensions": {
        "details": "some more info...",
        "code": 500,
      }
    }
  ]
}
```

#### 落とし穴

Rustは、まだ安定化したトレイトの特殊化を提供していません。
これが、`ErrorExtensions`が`E: stf::fmt::Display`の代わりに、`&E where E: std::fmt::Display`を実装している理由です。
いくつかの特殊化は[Autoref-based stable specialization](https://github.com/dtolnay/case-studies/blob/master/autoref-specialization/README.md)を介して提供されています。
この欠点は、下のコードがコンパイル**できない**ことです。

```rust
async fn parse_with_extensions_result(&self) -> Result<i32> {
    // the trait `error::ErrorExtensions` is not implemented
    // for `std::num::ParseIntError`
    "234a".parse().extend_err(|_, e| e.set("code", 404))
}
```

しかし、次は機能します。

```rust
async fn parse_with_extensions_result(&self) -> Result<i32> {
    // does work because ErrorExtensions is implemented for &ParseIntError
    "234a"
      .parse()
      .map_err(|ref e: ParseIntError| e.extend_with(|_, e| e.set("code", 404)))
}
```

### Apolloトレーシング

Apolloトレーシングは、クエリのそれぞれの段階で、性能分析結果を提供します。
これは、`Schema`のエクステンションで、性能分析結果は`QueryResponse`内に蓄積されます。

Apolloトレーシングエクステンションを有効にするために、`Schema`を作成したときにエクステンションを追加します。

```rust
use async_graphql::*;
use async_graphql::extensions::ApolloTracing;


let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
    .extension(ApolloTracing) // Enable ApolloTracing extension
    .finish();
```

### クエリの複雑さと深さ

⚠️GraphQLはデータをクエリする強力な方法を提供しますが、APIクライアントの手に大きな権限を与えると、サービス拒否攻撃のリスクに晒されます。
許可するクエリの複雑さと深さを制限することにより、`async-graphql`でそのリスクを軽減できます。

#### 高価なクエリ

ブログ投稿のリストさせるスキーマを考えてください。
それぞれのブログ投稿は、関連する他の投稿があります。

```graphql
type Query {
  posts(count: Int = 10): [Post!]!
}

type Post {
  title: String!
  text: String!
  related(count: Int = 10): [Post!]!
}
```

とても大きなレスポンスを引き起こすクエリを作成することは、あまり難しくありません。

```graphql
{
    posts(count: 100) {
        related(count: 100) {
            related(count: 100) {
                related(count: 100) {
                    title
                }
            }
        }
    }
}
```

レスポンスのサイズは、`related`フィールドの階層が深くなるにつれて、指数関数的に増加します。
運が良いことに、`async-graphql`はそのようなクエリを回避する方法を提供しています。

#### クエリの深さを制限する

深さは、フィールドのネストしたレベルの数で、次は深さ3のクエリです。

```graphql
{
    a {
        b {
            c
        }
    }
}
```

`Schema`を作成するときに深さを制限できます。
もし、クエリがこの制限を超過した場合、エラーが発生して、メッセージ`Query is nested to deep`が返されます。

```rust
let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
    .limit_depth(5) // 最大深さを5に制限
    .finish();
```

#### 複雑なクエリを制限する

複雑性は、クエリ内のフィールドの数です。
それぞれのフィールドのデフォルトの複雑さは`1`です。
下は、複雑性`6`のクエリです。

```graphql
{
    a b c {
        d {
            e f
        }
    }
}
```

`Schema`を作成するときに複雑性を制限できます。
もし、クエリがこの制限を超過した場合、エラーが発生して、`Query is too complex`が返されます。

```rust
let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
    .limit_complexity(5) // 最大複雑性を5に制限
    .finish();
```

#### カスタム複雑性計算

非リスト型とリスト型フィールドに対して、複雑性をカスタマイズする方法が2つあります。

次のコードにおいて、`value`フィールドの複雑性は`5`です。
`values`フィールドの複雑性は`count * child_complexity`で、`child_complexity`はサブクエリの複雑性を表現する特別な変数で、`count`はフィールドのパラメーターで、`values`フィールドの複雑性を計算するために使用され、戻り値の型は`usize`でなければなりません。

```rust
struct Query;

#[Object]
impl Query {
    #[graphql(complexity = 5)]
    async fn value(&self) -> i32 {
        todo!()
    }

    #[graphql(complexity = "count * child_complexity")]
    async fn values(&self, count: usize) -> i32 {
        todo!()
    }
}
```

**注意: 複雑性の計算は、実行段階でなく検証段階で行われるため、制限を超えたクエリの部分実行について心配する必要はありません。

### イントロスペクションにおけるコンテンツの非表示

デフォルトで、すべての型とフィールドはないせいで表示されます。
しかし、不必要な誤解を避けるために、ユーザーに応じて一部のコンテンツを非表示にしたい場合があります。
型やフィールドに`visible`属性を追加して、それを行います。

```rust
use async_graphql::*;

#[derive(SimpleObject)]
struct MyObj {
    // This field will be visible in introspection.
    a: i32,

    // This field is always hidden in introspection.
    #[graphql(visible = false)]
    b: i32,

    // This field calls the `is_admin` function, which
    // is visible if the return value is `true`.
    #[graphql(visible = "is_admin")]
    c: i32,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
enum MyEnum {
    // このアイテムはイントロスペクションで表示されます。
    A,

    // このアイテムはイントロスペクションで非表示になります。
    #[graphql(visible = false)]
    B,

    // このアイテムは`is_admin`関数を呼び出し、それは戻り値が`true`の場合に表示されます。
    #[graphql(visible = "is_admin")]
    C,
}

struct IsAdmin(bool);

fn is_admin(ctx: &Context<'_>) -> bool {
    ctx.data_unchecked::<IsAdmin>().0
}
```
