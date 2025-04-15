# イントロダクション

- [Book](https://async-graphql.github.io/async-graphql/en/index.html)
- [GitHub](https://github.com/async-graphql/async-graphql)
- [Docs](https://docs.rs/async-graphql/latest/async_graphql/index.html)

`async-graphql`は、Rustで実装されたGraphQLサーバーサイドライブラリです。
`async-graphql`は、GraphQL仕様と、そのほとんどのエクステンションと完全な互換性があり、そ型安全と高い性能を提供します。

Rustでスキーマを定義でき、手続きマクロはGraphQLクエリ用にコードを自動的に生成します。
このライブラリは、Rustの構文を拡張しておらず、それはRustfmtが普通に使用できることを意味します。
これには高い価値があり、`async-graphql`を開発した理由の1つです。

## なぜこうするのか？

GraphQLとRustが好きだからです。
RustでGraphQLサーバーを実装する問題を解決する`Juniper`を使用してきました。
しかし、`Juniper`はいくつかの問題があり、その最も重要なことは、当時はasync/awaitをサポートしていないことです。
したがって、このライブラリを作成することを決心しました。

## ベンチマーク

バックグラウンドでCPUを大量に消費するプロセスがないことを確認してください。

```sh
cd benchmark
cargo bench
```

これで、`benchmark/target/criterion/report`にあるHTMLレポートを利用できます。
