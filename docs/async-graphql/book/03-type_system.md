# 型システム

- [型システム](#型システム)
  - [SimpleObject](#simpleobject)
    - [ユーザー定義リゾルバー](#ユーザー定義リゾルバー)
    - [ジェネリックなSimpleObject](#ジェネリックなsimpleobject)
    - [入力型と出力型の使用](#入力型と出力型の使用)
    - [フィールドの平坦化](#フィールドの平坦化)
  - [Object](#object)
    - [Context](#context)
      - [データの蓄積](#データの蓄積)
      - [スキーマデータ](#スキーマデータ)
      - [リクエストデータ](#リクエストデータ)
      - [ヘッダー](#ヘッダー)
      - [選択と先読み](#選択と先読み)
    - [エラーハンドリング](#エラーハンドリング)
      - [サブスクリプションのエラー](#サブスクリプションのエラー)
    - [オブジェクトのマージ](#オブジェクトのマージ)
      - [サブスクリプションのマージ](#サブスクリプションのマージ)
    - [導出されたフィールド](#導出されたフィールド)
      - [ラッパー型](#ラッパー型)
        - [例](#例)
  - [列挙型](#列挙型)
    - [外部列挙型のラップ](#外部列挙型のラップ)
  - [インターフェイス](#インターフェイス)
  - [手作業でインターフェイスを登録する](#手作業でインターフェイスを登録する)
  - [ユニオン](#ユニオン)
    - [ネストしたユニオンの平坦化](#ネストしたユニオンの平坦化)
  - [入力オブジェクト](#入力オブジェクト)
    - [ジェネリックなInputObject](#ジェネリックなinputobject)
    - [機密データの編集](#機密データの編集)
    - [フィールドの平坦化](#フィールドの平坦化-1)
  - [OneofObject](#oneofobject)
  - [デフォルト値](#デフォルト値)
    - [オブジェクトのフィールド](#オブジェクトのフィールド)
    - [入力オブジェクトのフィールド](#入力オブジェクトのフィールド)

`async-graphql`は、GraphQLオブジェクトからRust構造体への変換を実装しているため、使用することは簡単です。

## SimpleObject

[SimpleObject](https://async-graphql.github.io/async-graphql/en/define_simple_object.html)は、GraphQLオブジェクトに構造体のすべてのフィールドをマッピングします。
もし、自動的なフィールドのマッピングが必要ない場合は、[Object](https://async-graphql.github.io/async-graphql/en/define_complex_object.html)を参照してください。

次の例は、フィールド`a`、`b`を持つ`MyObject`を定義します。
`c`は、`#[graphql(skip)]`でラベル付けされているため、GraphQLにマッピングされません。

```rust
use async_graphql::SimpleObject;

#[derive(SimpleObject)]
struct MyObject {
    a: i32,
    b: String,
    #[graphql(skip)]
    c: String,
}
```

### ユーザー定義リゾルバー

GraphQLオブジェクトのほとんどのフィールドは、単純に構造体のメンバーの値を返すことがよくありますが、いくつかのフィールドは計算されます。
この場合、すべてのリゾルバーを手書きしない限り、[Object](https://async-graphql.github.io/async-graphql/en/define_complex_object.html)マクロは使用できません。

[ComplexObject](https://docs.rs/async-graphql/latest/async_graphql/attr.ComplexObject.html)マクロは、`SimpleObject`マクロと連携して機能します。
`SimpleObject`導出マクロは非計算フィールドを定義して、`ComplexObject`マクロは計算フィールドに対してユーザー定義リゾルバーを記述できるようにします。

`ComplexObject`マクロを追加されたリゾルバーは、[Object](https://async-graphql.github.io/async-graphql/en/define_complex_object.html)のリゾルバーと同じルールに従います。

```rust
/// 注意: `ComplexObject`マクロを有効にしたい場合、この`complex`属性が要求されます。
#[derive(SimpleObject)]
#[graphql(complex)]
struct MyObj {
    a: i32,
    b: i32,
}

#[ComplexObject]
impl MyObj {
    async fn c(&self) -> i32 {
        self.a + self.b
    }
}
```

### ジェネリックなSimpleObject

他の型に`SimpleObject`を再利用したい場合、ジェネリックな`SimpleObject`を定義して、具体的な型が実装される方法を指示できます。

次の例は、2つの`SimpleObject`型が作成されます。

```rust
#[derive(SimpleObject)]
#[graphql(concrete(name = "SomeName", params(SomeType)))]
#[graphql(concrete(name = "SomeOtherName", params(SomeOtherType)))]
struct SomeGenericObject<T: OutputType> {
    field1: Option<T>,
    field2: String,
}
```

注意: それぞれのジェネリックパラメーターは、上記のように[OutputType](https://docs.rs/async-graphql/latest/async_graphql/trait.OutputType.html)を実装しなければなりません。

生成されたスキーマは次のようになります。

```graphql
type SomeName {
  field1: SomeType
  field2: String!
}

type SomeOtherName {
  field1: SomeOtherType
  field2: String!
}
```

リゾルバーメソッド、または他のオブジェクトのフィールドで、通常のジェネリックタイプとして使用します。

```rust
#[derive(SimpleObject)]
pub struct YetAnotherObject {
  a: SomeGenericObject<SomeType>,
  b: SomeGenericObject<SomeOtherType>,
}
```

カンマで分割して`params()`に複数のジェネリック型を渡せます。

### 入力型と出力型の使用

```rust
/// 入力型に新しい名前を定義するために`input_name`属性を使用する必要があり、そうでない場合はランタイムエラーが発生します。
#[derive(SimpleObject, InputObject)]
#[graphql(input_name = "MyObjInput")]
struct MyObj {
    a: i32,
    b: i32,
}
```

### フィールドの平坦化

`#[graphql(flatten)]`を追加することで、フィールドを平坦化できます。

```rust
#[derive(SimpleObject)]
pub struct ChildObject {
    b: String,
    c: String,
}

#[derive(SimpleObject)]
pub struct ParentObject {
    a: String,
    #[graphql(flatten)]
    child: ChildObject,
}

/// ParentObjectは次と同じです。
#[derive(SimpleObject)]
pub struct Object {
    a: String,
    b: String,
    c: String,
}
```

## Object

`SimpleObject`と異なり、`Object`はその`impl`内にそれぞれのフィールドを定義したリゾルバーを持たなければなりません。

**リゾルバー関数は非同期でなければなりません。最初の引数は`&self`で、2番目の引数はオプショナルな`Context`でなければならず、その後にフィールド引数が続きます。**

リゾルバーはフィールドの値を得るために使用されます。
例えば、データベースに問い合わせて結果を返すことができます。
**関数の戻り値の型はフィールドの型です。**
もし関数でエラーが発生した場合、エラーを返すために`async_graphql::Result`を返すこともできます。
そして、エラーメッセージはクエリの結果として送信されます。

例えば、データベースコネクションプールなど、クエリ内でグローバルなデータにアクセスする必要があるかもしれません。
スキーマを作成するとき、グローバルデータを構成するために`SchemaBuilder`を、また`Context`データを構成するために`Context::data`を使用できます。
次の`value_from_db`関数は、`Context`からデータベースコネクションを取り出す方法を示しています。

```rust
use async_graphql::*;

struct MyObject {
    value: i32,
}

#[Object]
impl MyObject {
    async fn value(&self) -> String {
        self.value.to_string()
    }

    async fn value_from_db(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Id of object")] id: i64
    ) -> Result<String> {
        let conn = ctx.data::<DbPool>()?.take();
        Ok(conn.query_something(id)?.name)
    }
}
```

### Context

[Context](https://docs.rs/async-graphql/latest/async_graphql/context/type.Context.html)の主な目的は、スキーマに付属したグローバルデータと、実際に処理されているクエリに関連するデータを取得することです。

#### データの蓄積

環境変数、データベースコネクションプール、それぞれのクエリに必要になるかもしれない何でも、`Context`内にグローバルデータを入れられます。

データは`Send`と`Sync`を実装していなければなりません。

`ctx.data::<TypeOfYourData>()`を単に呼び出すことで、クリエ以内のデータを要求できます。

**もし、リゾルバー関数の戻り値が`Context`から借用されている場合、引数のライフタイムを明示的に記述する必要があることに注意してくだささい。**

次の例は、`Context`内のデータを借用する方法を示しています。

```rust
use async_graphql::*;

struct Query;

#[Object]
impl Query {
    async fn borrow_from_context_data<'ctx>(
        &self,
        ctx: &Context<'ctx>,
    ) -> Result<&'ctx String> {
        ctx.data::<String>()
    }
}
```

#### スキーマデータ

スキーマを作成するときにコンテキスト内にデータを入れることができ、それはコネクションプールのように変更されないデータにとって便利です。

それをアプリケーションで記述する例を示します。

```rust
let schema = Schema::build(Query::default(), EmptyMutation, EmptySubscription)
    .data(env_struct)
    .data(s3_storage)
    .data(db_core)
    .finish();
```

#### リクエストデータ

リクエストの実行時にコンテキスト内にデータを入れることができ、例えばそれは認証データにとって便利です。

`wrap`ルーティングでの小さいな例を示します。

```rust
let graphql_host = wrap::post()
    .and(wrap::path("graphql"))
    .and(wrap::header::optional("Authorization"))
    .and(schema_filter)
    .and_then(|auth: Option<String>, (scheme, mut request): (Schema<Query, EmptyMutation, EmptySubscription>, async_graphql::Request) | async move {
        // ヘッダーから得た認証データを取得するためになにかします。
        let your_auth_data = AuthInfo { token: auth };
        let response = schema.execute(request.data(your_auth_data)).await;
        Ok::<_, Infallible>(async_graphql_warp::GraphQLResponse::from(response))
    });
```

#### ヘッダー

コンテキストを使用すると、ヘッダーを挿入または追加できます。

```rust
#[Object]
impl Query {
    async fn greet(&self, ctx: &Context<'_>) -> String {
        // `http`定数を使用してヘッダーを挿入できます。
        let was_in_headers = ctx.insert_http_header(ACCESS_CONTROL_ALLOW_ORIGIN, "*");

        // `&str`を使用して挿入することもできます。
        let was_in_headers = ctx.insert_http_header("Custom-Header", "1234");

        // もし、同じキーを持つ複数のヘッダーが挿入された場合、前のものが最も最新のもので上書きされます。
        // もし、同じキーを持つ複数のヘッダーが必要な場合、後続のヘッダー用に`append_http_header`を
        // 使用してください。
        let was_in_headers = ctx.append_http_header("Custom-Header", "Hello World");

        String::from("Hello world")
    }
}
```

#### 選択と先読み

データ処理を最適化するために、サブクエリ内で要求されるフィールドを知りたいことがよくあります。
フィールドやサブフィールド間を移動する[SelectionField](https://docs.rs/async-graphql/latest/async_graphql/context/struct.SelectionField.html)を与える`ctx.field()`を使用して、クエリ全体のフィールドを読み込みできます。

もし、クエリまたはサブクエリ間の検索を実行したい場合、手動で`SelectionField`を使用する必要はなく、選択を実行するために`ctx.look_ahead()`を使用できます。

```rust
use async_graphql::*;

#[derive(SimpleObject)]
struct Detail {
    c: i32,
    d: i32,
}

#[derive(SimpleObject)]
struct MyObj {
    a: i32,
    b: i32,
    detail: Detail,
}

struct Query;

#[Object]
impl Query {
    async fn obj(&self, ctx: &Context<'_>) -> MyObj {
        if ctx.look_ahead().field("a").exist() {
            // これは`obj { a }`のようなクエリです。
        } else if ctx.look_ahead().field("detail").field("c").exists() {
            // これは`obj { detail { c } }`のようなクエリです。
        } else {
            // このクエリは`a`がありません。
        }
        unimplemented!()
    }
}
```

### エラーハンドリング

リゾルバーは、次の定義を持つ`Result`を返せます。

```rust
type Result<T> = std::result::Result<T, Error>;
```

`std::fmt::Display`を実装した任意の`Error`は、`async_graphql::Error`に変換でき、エラーメッセージを拡張できます。

> ```rust
> /// `async-graphql::Error`は、`Display + Send + Sync`を実装した任意の型を受け入れます。
> #[cfg(not(feature = "custom-error-conversion"))]
> impl<T: Display + Send + Sync + 'static> From<T> for Error {
>     fn from(e: T) -> Self {
>         Self {
>             message: e.to_string(),
>             source: Some(Arc::new(e)),
>             extensions: None,
>         }
>     }
> }
> ```

次の例は、入力文字列を整数にパースする方法を示しています。
パースが失敗したとき、エラーを返し、エラーメッセージを付与します。
詳細は、この`Book`の[Error Extensions](https://async-graphql.github.io/async-graphql/en/error_extensions.html)セクションを参照してください。

```rust
use async_graphql::*;

struct Query;

#[Object]
impl Query {
    async fn parse_with_extensions(&self, input: String) -> Result<i32> {
        Ok(
            "234a"
            .parse()
            .map_err(|e: ParseIntError| err.extend_with(|_, e| e.set("code", 400)))?)
    }
}
```

#### サブスクリプションのエラー

エラーは、次の形式の戻り値の型を使用して、サブスクリプションのリゾルバーからも返されます。

```rust
async fn my_subscription_resolver(&self) -> impl Stream<Item = Result<MyItem, MyError>> { ... }
```

`MyError`構造体は、`Subscription`マクロによる制限によって、`Clone`実装を持たなければならないことに注意してください。
これを達成する1つの方法は、`#[derive(Clone)]`を使用してカスタムエラー型を作成することで、[ここを確認](https://github.com/async-graphql/async-graphql/issues/845#issuecomment-1090933464)してください。

### オブジェクトのマージ

通常、Rustにおいて同じ型に対して複数の実装を作成できますが、手続き型マクロの制限のために、同じ型に対して複数の`Object`実装を作成できません。
例えば、次のコードはコンパイルに失敗します。

```rust
#[Object]
impl Query {
    async fn users(&self) ->Vec<User> {
        toto!()
    }
}

#[Object]
impl Query {
    async fn movies(&self) ->Vec<Movie> {
        toto!()
    }
}
```

代わりに、`#[derive(MergeObject)]`マクロは、2つまたはそれ以上の`#[Object]`実装を1つにマージすることで、複数モジュールまたはファイル間にオブジェクトのリゾルバーを分割できるようにします。

**TIP:** `#[MergeObject]`内でも、すべての`#[Object]`は一意な名前を必要とするため、マージするそれぞれのObjectに独自の名前を付ける必要があります。

**注意:** これはクエリとミューテーションで機能します。サブスクリプションは、下の「サブスクリプションのマージ」を参照してください。

```rust
#[derive(Default)]
struct UserQuery;

#[Object]
impl UserQuery {
    async fn users(&self) -> Vec<User> {
        todo!()
    }
}

#[derive(Default)]
struct MovieQuery;

#[Object]
impl MovieQuery {
    async fn movies(&self) -> Vec<Movie> {
        todo!()
    }
}

#[derive(MergedObject, Default)]
struct Query(UserQuery, MovieQuery);

let schema = Schema::new(
    Query::default(),
    EmptyMutation,
    EmptySubscription,
);
```

---

⚠️ `MergedObject`はインターフェイス内で使用できません。

---

#### サブスクリプションのマージ

`MergeObject`にしたがって、分割した`#[Subscription]`ブロックをマージするために、`MergeSubscription`を導出、または`#[MergedSubscription]`を使用できます。

Objectのマージと同様に、それぞれのサブスクリプションブロックは一意な名前を要求します。

```rust
#[derive(Default)]
struct Subscription1;

#[Subscription]
impl Subscription1 {
    async fn events1(&self) -> impl Stream<Item = i32> {
        futures_until::stream::iter(0..10)
    }
}

#[derive(Default)]
struct Subscription2;

#[Subscription]
impl Subscription2 {
    async fn events2(&self) -> impl Stream<Item = i32> {
        futures_until::stream::iter(10..20)
    }
}

#[derive(MergedSubscription, Default)]
struct Subscription(Subscription1, Subscription2);

let schema = Schema::new(
    Query::default(),
    EmptyMutation,
    Subscription::default()
);
```

### 導出されたフィールド

2つのフィールドが同じ問い合わせロジックを持つが、出力型が異なることがよくあります。
`async-graphql`において、それに対して導出フィールドを作成できます。

次の例において、`RFC2822`書式で時間フォーマットを出力する`data_rfc2822`フィールドをすでに持っており、新しい`date_rfc3339`フィールドを導出するためにそれを再利用します。

```rust
struct DateRFC3339(chrono::DateTime<Utc>);
struct DateRFC2822(chrono::DateTime<Utc>);

#[Scalar]
impl ScalarType for DateRFC3339 {
    fn parse(value: Value) -> InputValueResult<Self> { todo!() }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_rfc3339())
    }
}

#[Scalar]
impl ScalarType for DateRFC2822 {
    fn parse(value: Value) -> InputValueResult<Self> { todo!() }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_rfc2822())
    }
}

impl From<DateRFC2822> for DateRFC3339 {
    fn from(value: DateRFC2882) -> Self {
        DateRFC3339(value.0)
    }
}

struct Query;

#[Object]
impl Query {
    #[graphql(derived(name = "date_rfc3339", into = "DateRFC3339"))]
    async fn date_rfc2822(&self, arg: String) -> DateFRC2822 {
        todo!()
    }
}
```

上記は次のようにGraphQLをレンダリングします。

```graphql
type Query {
    date_rfc2822(arg: String): DateRFC2822!
    date_rfc3339(arg: String): DateRFC3339!
}
```

#### ラッパー型

派生フィールドはすべてを簡単に扱うことはできません。
Rustの[孤児ルール](https://doc.rust-lang.org/book/traits.html#rules-for-implementing-traits)は、トレイトまたはトレイトを実装する型のどちらかが、同じクレート内でimplとして定義されていなければならないことを要求するため、次のコードはコンパイルできません。

```rust
impl From<Vec<U>> for Vec<T> {
    ...
}
```

したがって、`Vec`や`Option`のような既存のラッパー型構造体に対して、派生フィールドを生成できません。
しかし、`From<U> for T`を実装したとき、`From<Vec<U>> for Vec<T>`と`From<Option<U>> for Option<T>`を実装できるべきです。
ラッパー構造体間で`Into`トレイト実装を使用する代わりに、呼び出す関数を定義する`with`パラメーターを含めました。

##### 例

```rust
#[derive(Serialize, Deserialize, Clone)]
struct ValueDerived(String);

#[derive(Serialize, Deserialize, Clone)]
struct ValueDerived2(String);

scalar!(ValueDerived);
scalar!(ValueDerived2);

impl From<ValueDerived> for ValueDerived2 {
    fn from(value: ValueDerived) -> Self {
        Self(value.0)
    }
}

fn option_to_option<T, U: From<T>>(value: Option<T>) -> Option<U> {
    value.map(|x| x.into())
}

#[derive(SimpleObject)]
struct TestObj {
    #[graphql(derived(owned, name = "value2", into = "Option<ValueDerived2>", with = "option_to_option"))]
    pub value1: Option<ValueDerived>,
}
```

## 列挙型

`Enum`を定義することは簡単で、ここに例を示します。

**`async-graphql`は、自動でそれぞれのアイテムの名前を、GraphQLのCONSTANT_CASEに変換します。名前を変更するために`name`を使用できます。**

```rust
use async_graphql::*;

/// スターウォーズ三部作の1つの映画
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum Episode {
    /// 1977年公開
    NewHope,

    /// 1980年公開
    Empire,

    /// 1983年公開
    Jedi,
}
```

### 外部列挙型のラップ

Rustの[孤児ルール](https://doc.rust-lang.org/book/traits.html#rules-for-implementing-traits)は、トレイトまたはトレイトを実装する型のどちらかが、同じクレート内でimplとして定義されていなければならないことを要求するため、外部の列挙型をGraphQLに公開することができません。
`Enum`型を提供するために、一般的な回避策は、既存の外部列挙型と同等な新しい列挙型を作成することです。

```rust
use async_graphql::*;

/// 外部列挙型と同等なものを提供します。
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum LocalEnum {
    A,
    B,
    C,
}

/// 外部の型とローカルなGraphQL列挙型の変換インターフェイスです。
impl From<remote_crate::RemoteEnum> for LocalEnum {
    fn from(value: remote_create::RemoteEnum) -> Self {
        match value {
            remote_crate::RemoteEnum::A => Self::A,
            remote_crate::RemoteEnum::B => Self::B,
            remote_crate::RemoteEnum::C => Self::C,
        }
    }
}
```

その処理は退屈で、ローカルと外部列挙型の同期を維持するために複数の手順を要求します。
`async-graphql`は、`Enum`を導出した後の追加の属性を介して、`From<remote_crate::RemoteEnum> for LocalEnum`を生成する便利な機能と、`From<LocalEnum> for remote_crate::RemoteEnum`の反対方向も同様に提供しています。

```rust
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
#[graphql(remote = "remote_crate::RemoteEnum")]
enum LocalEnum {
    A,
    B,
    C,
}
```

## インターフェイス

`Interface`は一般的なフィールドを持つ`Object`を抽象化するために使用されます。
`async-graphql`は、ラッパーとしてそれを実装しています。
そのラッパーは、このインターフェイスを実装する`Object`にフィールド解決を転送します。
したがって、`Object`のフィールド型と引数は、`Interface`と一致しなければなりません。

また、`async-graphql`は、`Object`から`Interface`への自動変換を実装するため、`Into::into()`を呼び出す必要があるだけです。

インターフェイスのフィールド名は、スキーマ定義用にキャメルケースに変換されます。
例えば、スネークケースなGraphQLフィールド名のように、もし必要があれば、`name`と`method`属性を使用できます。

- `name`と`method`が互いに存在するとき、`name`はGraphQLフィールド名で、`method`はリゾルバー関数名です。
- `name`飲みが存在するとき、`name.to_camel_case()`はGraphQLフィールド名で、`name`はリゾルバー関数名です。

```rust
use async_graphql::*;

struct Circle {
    radius: f32,
}

#[Object]
impl Circle {
    async fn area(&self) -> f32 {
        std::f32::consts::PI * self.radius * self.radius
    }

    async fn scale(&self, s: f32) -> Shape {
        Circle { radius: self.radius * s }.into();
    }

    #[graphql(name = "short_description")]
    async fn short_description(&self) -> String {
        "Circle".to_string()
    }
}

struct Square {
    width: f32,
}

#[Object]
impl Square {
    async fn area(&self) -> f32 {
        self.width * self.width
    }

    async fn scale(&self, s: f32) -> Shape {
        Square { width: self.width * s }.into()
    }

    #[graphql(name = "short_description")]
    async fn short_description(&self) -> String {
        "Square".to_string()
    }
}

#[derive(Interface)]
#[graphql(
    field(name = "area", ty = "f32"),
    field(name = "scale", ty = "Shape", arg(name = "s", ty = "f32")),
    field(name = "short_description", method = "short_description", ty = "String")
)]
enum Shape {
    Circle(Circle),
    Square(Square),
}
```

## 手作業でインターフェイスを登録する

`async-graphql`は、初期化フェーズ内で`Schema`から直接または間接的に参照される型すべてを縦断して登録します。
もし、インターフェイスが参照されていない場合、それはレジストリに存在せず、次の例において、`MyObject`は`MyInterface`を実装していても、`MyInterface`はスキーマで参照されていないため、`MyInterface`型はレジストリに存在しません。

```rust
#[derive(Interface)]
#[graphql(
    field(name = "name", ty = "String"),
)]
enum MyInterface {
    MyObject(MyObject),
}

#[derive(SimpleObject)]
struct MyObject {
    name: String,
}

struct Query;

#[Object]
impl Query {
    async fn obj(&self) -> MyObject {
        todo!()
    }
}

type MySchema = Schema<Query, EmptyMutation, EmptySubscription>;
```

`Schema`を構築するときに、手作業で`MyInterface`型を登録する必要があります。

```rust
Schema::build(Query, EmptyMutation, EmptySubscription)
    .register_output_type::<MyInterface>()
    .finish();
```

## ユニオン

`Union`の定義は`Interface`と似ていますが、**フィールドを持つことはできません。**
実装は`async-graphql`と非常によく似ています。
`async-graphql`の観点から、`Union`は`Interface`のサブセットです。

次の例は、少し`Interface`の定義を修正して、フィールドを削除しました。

```rust
use async_graphql::*;

struct Circle {
    radius: f32,
}

#[Object]
impl circle {
    async fn area(&self) -> f32 {
        std::f32::consts::PI * self.radius * self.radius
    }

    async fn scale(&self, s: f32) -> Shape {
        Circle {radius: self.radius * s }.into()
    }
}

struct Square {
    width: f32,
}

#[Object]
impl Square {
    async fn area(&self) -> f32 {
        self.width * self.width
    }

    async fn scale(&self, s: f32) -> Shape {
        Square { width: self.width * s }.into()
    }
}

#[derive(Union)]
enum Shape {
    Circle(Circle),
    Square(Square),
}
```

### ネストしたユニオンの平坦化

GraphQLの制限は、他のユニオン型からユニオン型を作成できません。
すべてのメンバーは、`Object`である必要があります。
ネストされたユニオンをサポートするために、ユニオンのメンバーを「平坦化」して、親のユニオン内にそれらのメンバーを持ち込めます。
これは、平坦化したいそれぞれのメンバーに`#[graphql(flatten)]`を適用することで実現できます。

```rust
#[derive(async_graphql::Union)]
pub enum TopLevelUnion {
    A(A),

    // ユニオンメンバーを平坦化することなしに、コンパイルできません。
    #[graphql(flatten)]
    B(B),
}

#[derive(async_graphql::SimpleObject)]
pub struct A {
    a: i32,
    // ...
}

#[derive(async_graphql::Union)]
pub enum B {
    C(C),
    D(C),
}

#[derive(async_graphql::SimpleObject)]
pub struct C {
    c: i32,
    // ...
}

#[derive(async_graphql::SimpleObject)]
pub struct D {
    d: i32,
    // ...
}
```

上記例は、トップレベルのユニオンを次の同等なものに変換します。

```rust
#[derive(async_graphql::Union)]
pub enum TopLevelUnion {
    A(A),
    C(C),
    D(D),
}
```

## 入力オブジェクト

`Object`を引数として使用でき、GraphQLはそれを`InputObject`と呼びます。

`InputObject`の定義は`SimpleObject`と似ていますが、`SimpleObject`は出力としてのみ使用でき、`InputObject`は入力としてのみ使用できます。

説明を追加、またはフィールドを名前変更するために、オプションで`#[graphql]`属性を追加できます。

```rust
use async_graphql::*;

#[derive(InputObject)]
struct Coordinate {
    latitude: f64,
    longitude: f64,
}

struct Mutation;

#[Object]
impl Mutation {
    async fn users_at_location(&self, coordinate:Coordinate, radius: f64) -> Vec<User> {
        // 座標をデータベースに記述します。
    }
}
```

### ジェネリックなInputObject

もし、他の型に`InputObject`を再利用したい場合、ジェネリックな`InputObject`を定義して、その具体的な型が実装されるべき方法を指定できます。

次の例において、2つの`InputObject`型が作成されます。

```rust
#[derive(InputObject)]
#[graphql(concrete(name = "SomeName", params(SomeType)))]
#[graphql(concrete(name = "SomeOtherName", params(SomeOtherType)))]
pub struct SomeGenericInput<T: InputType> {
    field: Option<T>,
    field2: String,
}
```

注意️： それぞれのジェネリックパラメーターは、上記のように`InputType`を実装していなければなりません。

生成されたスキーマは次のようになります。

```graphql
input SomeName {
  field1: SomeType
  field2: String!
}

input SomeOtherName {
  field1: SomeOtherType
  field2: String!
}
```

他の入力オブジェクトのリゾルバーメソッドまたはフィールドで、通常のジェネリック型として使用します。

```rust
#[derive(InputObject)]
pub struct YetAnotherInput {
    a: SomeGenericInput<SomeType>,
    b: SomeGenericInput<SomeOtherType>,
}
```

カンマで区切って複数のジェネリック型を`params()`に渡せます。

### 機密データの編集

もし、入力の一部が機密だと考えられ、それを編集したい場合、`secret`ディレクティブでそれに印をつけます。

```rust
#[derive(InputObject)]
pub struct CredentialsInput {
    username: String,
    #[graphql(secret)]
    password: String,
}
```

### フィールドの平坦化

フィールド型からその親にキーをインラインにするために、フィールドに`#[graphql(flatten)]`を追加できます。

```rust
#[derive(InputObject)]
pub struct ChildInput {
    b: String,
    c: String,
}

#[derive(InputObject)]
pub struct ParentInput {
    s: String,
    #[graphql(flatten)]
    child: ChildInput,
}

/// ParentInputは次と同じです。
#[derive(InputObject)]
pub struct Input {
    a: String,
    b: String,
    c: String,
}
```

## OneofObject

`OneofObject`は、`InputObject`の特別な型で、そのフィールドの1つのみ設定されなければならず、非nullです。
いくつかの可能性のある入力型を選択させたいときに、特に便利です。

この機能はまだ[RFC](https://github.com/graphql/graphql-spec/pull/825)であるため、GraphQL使用の公式になっていませんが、`async-graphql`はそれをすでにサポートしています。

```rust
use async_graphql::*;

#[derive(OneofObject)]
enum UserBy {
    Email(String),
    RegistrationNumber(i64),
    Address(Address),
}

#[derive(InputObject)]
struct Address {
    street: String,
    house_number: String,
    city: String,
    zip: String,
}

struct Query;

#[Object]
impl Query {
    async fn search_users(&self, by: Vec<UserBy>) -> Vec<User> {
        // ... 検索してユーザーのリストを返します ...
    }
}
```

確認したように、`OneofObject`は`enum`で表現され、そのそれぞれのバリアントは他の`InputType`を含んでいます。
これは、バリアントとして`InputObject`も使用できることを意味します。

## デフォルト値

入力値型にデフォルト値を定義できます。
下にいくつかの例を示します。

### オブジェクトのフィールド

```rust
use async_graphql::*;

struct Query;

fn my_default() -> i32 {
    30
}

#[Object]
impl Query {
    // valueパラメーターのデフォルト値は0で、それはi32::default()を呼び出します。
    async fn test1(&self, #[graphql(default)] value: i32) -> i32 { todo!() }

    // valueパラメーターのデフォルト値は10です。
    async fn test2(&self, #[graphql(default = 10)] value: i32) -> i32 { todo!() }

    // valueパラメーターのデフォルト値は、my_default関数の戻り値を使用して、その値は30です。
    async fn test3(&self, #[graphql(default_with = "my_default()")] value: i32) -> i32 { todo!() }
}

### インターフェイスのフィールド

```rust
use async_graphql::*;

#[derive(Interface)]
#[graphql(
    field(name = "test1", ty = "i32", arg(name = "value", ty = "i32", default)),
    field(name = "test2", ty = "i32", arg(name = "value", ty = "i32", default = 10)),
    field(name = "test3", ty = "i32", arg(name = "value", ty = "i32", default_with = "my_default()")),
)]
enum MyInterface {
    MyObj(MyObj),
}
```

### 入力オブジェクトのフィールド

```rust
use async_graphql::*;

#[derive(InputObject)]
struct MyInputObject {
    #[graphql(default)]
    value1: i32,

    #[graphql(default = 10)]
    value2: i32,

    #[graphql(default_with = "my_default()")]
    value3: i32,
}
```
