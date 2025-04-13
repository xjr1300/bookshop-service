# async-graphql

- [async-graphql](#async-graphql)
  - [ドキュメント](#ドキュメント)
  - [SimpleObject](#simpleobject)
    - [ジェネリックなSimpleObject](#ジェネリックなsimpleobject)
    - [入力型と出力型の使用](#入力型と出力型の使用)
    - [フィールドの平坦化](#フィールドの平坦化)
  - [Object](#object)

## ドキュメント

- [GitHub](https://github.com/async-graphql/async-graphql)
- [Docs](https://docs.rs/async-graphql/latest/async_graphql/index.html)
- [Book](https://async-graphql.github.io/async-graphql/en/index.html)

## SimpleObject

[SimpleObject](https://async-graphql.github.io/async-graphql/en/define_simple_object.html)は、GraphQLオブジェクトに構造体のすべてのフィールドをマッピングする。
もし、自動的なフィールドのマッピングが必要ない場合は、[Object](https://async-graphql.github.io/async-graphql/en/define_complex_object.html)を参照すること。

次の例は、フィールド`a`、`b`を持つ`MyObject`を定義する。
`c`は、`#[graphql(skip)]`でラベル付けされているため、GraphQLにマッピングされない。

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

### ジェネリックなSimpleObject

他の型に`SimpleObject`を再利用したい場合、ジェネリックな`SimpleObject`を定義して、具体的な方が実装される方法を指示できる。

次の例は、2つの`SimpleObject`型が作成される。

```rust
#[derive(SimpleObject)]
#[graphql(concrete(name = "SomeName", params(SomeType)))]
#[graphql(concrete(name = "SomeOtherName", params(SomeOtherType)))]
struct SomeGenericObject<T: OutputType> {
    field1: Option<T>,
    field2: String,
}
```

注意: それぞれのジェネリックパラメーターは、上記のように[OutputType](https://docs.rs/async-graphql/latest/async_graphql/trait.OutputType.html)を実装しなければならない。

生成されたスキーマは次のようになる。

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

リゾルバーメソッド、または他のオブジェクトのフィールドで、通常のジェネリックタイプとして使用する。

```rust
#[derive(SimpleObject)]
pub struct YetAnotherObject {
  a: SomeGenericObject<SomeType>,
  b: SomeGenericObject<SomeOtherType>,
}
```

感まで分割して`params()`に複数のジェネリック型を渡せる。

### 入力型と出力型の使用

// Note: You must use the input_name attribute to define a new name for the input type, otherwise a runtime error will occur.

```rust
#[derive(SimpleObject, InputObject)]
#[graphql(input_name = "MyObjInput")]
struct MyObj {
    a: i32,
    b: i32,
}
```

注意、入力型に対して新しい名前を定義するために`input_name`属性を使用しなければならず、そうでない場合は、ランタイムエラーが発生する。

### フィールドの平坦化

`#[graphql(flatten)]`を追加することで、フィールドを平坦化できる。

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

/// ParentObjectは、次と同じである。
#[derive(SimpleObject)]
pub struct Object {
    a: String,
    b: String,
    c: String,
}
```

## Object

`SimpleObject`と異なり、`Object`はその`impl`内にそれぞれのフィールドを定義したリゾルバーを持たなければならない。

**リゾルバー関数は非同期でなければならない。最初の引数は`&self`で、2番目の引数はオプショナルな`Context`でなければならず、その後にフィルド引数が続く。**

リゾルバーはフィールドの値を得るために使用される。
例えば、データベースに問い合わせて結果を返すことができる。
**関数の戻り値の型はフィールドの型である。**
もしエラーが発生した場合、エラーを返すために`async_graphql::Result`を返すこともできる。
そして、エラーメッセージはクエリの結果として送信される。

例えば、データベースコネクションプールなど、クエリ内でグローバルなデータにアクセスする必要があるかもしれない。
スキーマを作成するとき、グローバうるデータを構成するために`SchemaBuilder`を、また`Context`データを構成するために`Context::data`を使用できる。
次の`value_from_db`関数は、`Context`からデータベースコネクションを取り出す方法を示している。

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
