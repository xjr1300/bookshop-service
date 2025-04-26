# オリジン間リソース共有（CORS）

基本的に、Webアプリケーションは、**Webブラウザによって**自分と同じ**オリジン**（プロトコル、ドメイン、ポート番号）のリソースのみアクセスできるように制御されています。

オリジン間リソース共有（`CORS`: Cross-Origin Resource Sharing）は、HTTPヘッダーベースの仕組みを使用して、あるオリジンで動作しているWebアプリに、自分と異なるオリジンのリソースへのアクセス権を与えるように**ブラウザに指示**する仕組みです。

セキュリティの観点から、Webブラウザは、スクリプトによって開始される別オリジンのHTTPリクエストを制限しており、`fetch`や`XMLHttpRequest`は、**同一オリジンポリジー (same-origin policy)**に従い、別オリジンのリソースへのアクセスを制限します。
同一オリジンポリシーがある理由は、悪意のあるWebサイトが、ユーザーの認証情報を盗んだり、他のWebサイトに対して不正なリクエストを送信したりすることを防ぐためです。

Webブラウザが、自分以外のオリジンのリソースを受け付けるようにするためには、レスポンスに適切な**CORSヘッダーを追加**する必要があります。

（仕様書では、）Webブラウザは、別オリジンから対応するメソッドの一覧を収集するために、`OPTIONS`メソッドを使用した**プリフライトリクエスト**を送信する場合があります。

## 単純リクエスト

**単純リクエスト**とは、次の条件をすべて満たすリクエストを示します。

- 許可されているメソッドが、`GET`、`HEAD`、`POST`のいずれかである。
- リクエストヘッダーが、ユーザーエージェントをによって自動的に設定されるヘッダーを除いて、次だけである。
  - `Accept`
  - `Accept-Language`
  - `Content-Language`
  - `Content-Type`
  - `Range`
- `Content-Type`が、次のいずれかである。
  - `application/x-www-form-urlencoded`
  - `multipart/form-data`
  - `text/plain`

単純リクエストの場合、Webブラウザは次のようなリクエストを送信します。

```http
GET /resources/public-data/ HTTP/1.1
Host: bar.other
User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10.14; rv:71.0) Gecko/20100101 Firefox/71.0
Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8
Accept-Language: en-us,en;q=0.5
Accept-Encoding: gzip,deflate
Connection: keep-alive
Origin: https://foo.example
```

`Origin`ヘッダーは、呼び出しが`https://foo.example`からのものであることを示します。

上記リクエストに対して、サーバーは次のようなレスポンスを返します。

```http
HTTP/1.1 200 OK
Date: Mon, 01 Dec 2008 00:23:53 GMT
Server: Apache/2
Access-Control-Allow-Origin: *
Keep-Alive: timeout=2, max=100
Connection: Keep-Alive
Transfer-Encoding: chunked
Content-Type: application/xml

[…XML データ…]
```

上記レスポンスでは、`Access-Control-Allow-Origin`ヘッダーが`*`に設定されており、すべてのドメインからのリクエストを許可していることを示します。
リクエストを`https://foo.example`からのリクエストに制限したい場合は、次のヘッダーを追加します。

```http
Access-Control-Allow-Origin: https://foo.example
```

## プリフライトリクエスト

単純リクエスト以外では、実際のリクエストを送信する前に`OPTIONS`メソッドでプリフライトリクエストを送信します。

```http
OPTIONS /doc HTTP/1.1
Host: bar.other
User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10.14; rv:71.0) Gecko/20100101 Firefox/71.0
Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8
Accept-Language: en-us,en;q=0.5
Accept-Encoding: gzip,deflate
Connection: keep-alive
Origin: https://foo.example
Access-Control-Request-Method: POST
Access-Control-Request-Headers: content-type,x-pingother
```

プリフライトリクエストに対して、サーバーは次のようなレスポンスを返します。

```http
HTTP/1.1 204 No Content
Date: Mon, 01 Dec 2008 01:15:39 GMT
Server: Apache/2
Access-Control-Allow-Origin: https://foo.example
Access-Control-Allow-Methods: POST, GET, OPTIONS
Access-Control-Allow-Headers: X-PINGOTHER, Content-Type
Access-Control-Max-Age: 86400
Vary: Accept-Encoding, Origin
Keep-Alive: timeout=2, max=100
Connection: Keep-Alive
```

Webブラウザが、異なるオリジンからのレスポンスを受け入れるためには、次のヘッダーをレスポンスに含める必要があります。

- `Access-Control-Allow-Origin`: サーバーがリクエストを許可するオリジンを指定します。
- `Access-Control-Allow-Methods`: プリフライトリクエストで指定されたリソースで許可するメソッドを指定します。
- `Access-Control-Allow-Headers`: プリフライトリクエストで指定されたリソースが受け入れ可能なヘッダーを指定します。

プリフライトリクエストが成功したら、Webブラウザは実際のリクエストを送信します。

## 資格情報

異なるオリジンへのリクエストに、クッキーやHTTP認証情報を含めることを、レスポンスヘッダーに指示することができます。
ただし、サードパーティクッキーポリシーの影響を受けます。

```http
Access-Control-Allow-Credentials: true
```
