# マイクロサービス開発・運用　実践ガイド

Rustでの実装を試行します。

## Protocol Buffers

- Protocol Buffersのバイナリ形式は、データをフィールド名ではなく番号で識別
  - メッセージサイズが小さくなり、軽量かつ高速になる
  - 同じフィールド番号を使用すれば互換性を保てる
- 互換性を維持するために次にを守ること
  - 既存のフィールド番号を変更してはならない。
  - 削除したフィールドのフィールド番号を再利用しない。

```proto
message User {
  string username = 1;
  int32  age      = 2;
}
```

上記メッセージは、次のレイアウトのバイナリで送信される。

```text
[フィールド番号:1][型情報]["username"]
[フィールド番号:2][型情報][age]
```

## ビルドスクリプト

- `build.rs`は、パッケージのルートに配置する**ビルドスクリプト**と呼ばれる特殊なファイルで、パッケージをビルドする前に実行される
- `build.rs`は、`cargo`がパッケージをビルドする前に自動的に実行
- ビルドスクリプトの目的は、コード生成、環境変数の設定、外部ライブラリのビルドなど、通常の`cargo`のビルドでは実行できないことを行うこと
- 環境変数`OUT_DIR`は、もしパッケージにビルドスクリプトがある場合、それにビルドスクリプトがその出力を配置するフォルダに設定（`OUT_DIR`は、コンパイル中にのみ設定される）
- `OUT_DIR`で示されるディレクトリには、ビルドスクリプトによって生成されるすべての出力と中間生成物が配置される
- `OUT_DIR`で示されるディレクトリは、ビルド中のパッケージのビルドディレクトリ内にあり、該当するパッケージで一意である

## BFF(Backend for Frontend)

- BFFは、背後にある複数サービスとの通信を集約して、Webブラウザで動作するWebアプリなどのフロントエンド（クライアント）に対してAPIを公開
- 従来のバックエンドは、データベースなどのミドルウェアに対してデータを取得または更新する機能と、クライアントからリクエストを受け取る機能を持つ
- BFFは、上記2つの機能を分担して、独立して開発できるようにしたアーキテクチャ

## 2.2.4.3 DockerとKubernetesを使ったマイクロサービスのデプロイ

本書と異なり、本リポジトリは次のディレクトリ構成を持つ。

```text
bookshop-service
├── backend
│   ├── bff         # BFF
│   ├── catalogue   # カタログサービス
│   ├── domain
│   └── target
└── frontend        # フロントエンド
```

カタログサービス、BFF、フロントエンドのDockerファイルの配置場所と内容を次に示す。

> 本書ではセキュリティを考慮して`root`ユーザーでサービスを起動しないように、ルートレスコンテナイメージを使用していたが、うまく動作しなかったため、`debian:bookworm-slim`を使用している。

- backend/Dockerfile.catalogue

```dockerfile
FROM rust:1.86-slim AS builder
WORKDIR /backend
COPY . ./
RUN apt update && apt install -y protobuf-compiler
RUN cargo build --package=catalogue --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /backend/target/release/catalogue .
EXPOSE 50051
CMD ["./catalogue", "--port=50051"]
```

- backend/Dockerfile.bff

```dockerfile
FROM rust:1.86-slim AS builder
WORKDIR /backend
COPY . .
RUN apt update && apt install -y protobuf-compiler
RUN cargo build --package=bff --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /backend/target/release/bff .
EXPOSE 4000
CMD ["./bff", "--catalogue-port=50051"]
```

- frontend/Dockerfile

```dockerfile
FROM node:23-bookworm-slim AS builder
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm install
COPY public ./public
COPY src ./src
COPY index.html ./
COPY tsconfig.* ./
COPY vite.config.ts ./
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
```

Dockerイメージは次の通り作成する。

```sh
cd backend
docker image build . -t bookshop-service/catalogue:0.1 -f ./Dockerfile.catalogue
docker image build . -t bookshop-service/bff:0.1 -f ./Dockerfile.bff
cd ../frontend
docker image build -t bookshop-service/frontend:0.1 .
```

### kubernetesクラスターで利用するコマンド

```sh
# kubernetesクラスター作成
# デフォルトの`kind`クラスターを作成
kind create cluster

# kubernetesクラスターの確認
kind get clusters

# kubernetesクラスターの削除
kind delete cluster

# catalogueコンテナイメージのロード
kind load docker-image bookshop-service/catalogue:0.1
# bffコンテナイメージのロード
kind load docker-image bookshop-service/bff:0.1
# frontendコンテナイメージのロード
kind load docker-image bookshop-service/frontend:0.1

# catalogueサービスのデプロイ
kubectl apply -f bookshop-demo/catalogue/k8s/catalogue.yaml
# bffサービスのデプロイ
kubectl apply -f bookshop-demo/bff/k8s/bff.yaml
# frontendサービスのデプロイ
kubectl apply -f bookshop-demo/frontend/k8s/frontend.yaml

# pod一覧
kubectl get pods
# サービス一覧
kubectl get services
# エンドポイント確認
kubectl get endpoints
# podの詳細
kubectl describe pod <pod-name>
# ログ出力
kubectl logs <pod-name>

# ポートフォーワード
# ローカルからfrontendサービスにアクセスするためのポートフォワーディング
kubectl port-forward service/frontend 8080:80
# ローカルからbffサービスにアクセスするためのポートフォワーディング
# ローカルのブラウザからbffサービスにアクセスするため、ポートフォワーディングが必要
kubectl port-forward service/frontend 4000:4000
```
