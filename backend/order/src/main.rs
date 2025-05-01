//! ```sh
//! docker run -it --rm --name rabbitmq -p 5672:5672 -p 15672:15672 rabbitmq:4.1-management
//! ```

use deadpool_lapin::Runtime;
use deadpool_lapin::lapin::BasicProperties;
use deadpool_lapin::lapin::options::{BasicPublishOptions, QueueDeclareOptions};
use deadpool_lapin::lapin::types::FieldTable;
use uuid::Uuid;

use domain::models::{OrderEvent, OrderItem};
use utils::amqp::amqp_config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let num_cpus = num_cpus::get();
    let config = amqp_config(num_cpus);
    let pool = config.create_pool(Some(Runtime::Tokio1))?;
    let conn = pool.get().await?;
    let channel = conn.create_channel().await?;

    // passiveをtrueに設定すると、キューを宣言するときに、キューが存在しない場合、エラーが発生する。
    // durableをtrueに設定すると、メッセージブローカーが再起動しても、キューが保持される。
    // exclusiveをtrueに設定すると、キューは1つの接続のみで使用され、接続が切れたとき、そのキューが削除される。
    // auto_deleteをtrueに設定すると、1つ以上のコンシューマーが存在するキューは、最後のコンシューマーが購読を解除すると削除される。
    // nowaitをtrueに設定すると、キュー作成の確認応答を待たずに処理を継続する。したがってキューが正常化どうかを確認しない。非同期で操作したいときに使用する。
    // let options = QueueDeclareOptions {
    //     passive: false,     // キューが存在しない場合は作成
    //     durable: false,     // メッセージブローカーが再起動したとき、キューを削除
    //     exclusive: false,   // 排他的に使用しない
    //     auto_delete: false, // コンシューマーがいなくなっても、キューを削除しない
    //     nowait: false,      // キュー作成の確認応答を待つ
    // };
    // let queue = channel
    //     .queue_declare("order", options, FieldTable::default())
    //     .await?;
    let queue = channel
        .queue_declare(
            "order",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let order_event = order_event();
    let body = serde_json::to_string(&order_event)?;
    let body = body.as_bytes();

    // エクスチェンジを指定しない場合、デフォルトエクスチェンジにメッセージを送信する。
    // デフォルトエクスチェンジは、ルーティングキーをメッセージを送信するキューの名前と判断して、ルーティングキーと一致するキューにメッセージを送信する。
    //
    // BasicPublishOptionsは、メッセージを送信するためのオプションを指定するための構造体
    //     mandatory: メッセージがどのキューにもルーティングされなかったかどうかをパブリッシャーに通知する。
    //     immediate: キューに準備完了状態のコンシューマーが存在する場合、そのいずれかのキューにメッセージがルーティングされる。
    //                そのコンシューマーが受信確認応答前にクラッシュした場合、メッセージは再キューイングされるか、そのキューの他のコンシューマーに配信される。
    //                クラッシュが発生していない場合は、メッセージは確認応答され、すべて通常どおりに処理される。
    channel
        .basic_publish(
            "",                    // デフォルトエクスチェンジにメッセージを送信
            queue.name().as_str(), // ルーティングキー（キュー名）
            BasicPublishOptions::default(),
            body,
            BasicProperties::default().with_content_type("application/json".into()),
        )
        .await?;
    println!("Sent order event: {:?}", order_event);

    Ok(())
}

fn order_event() -> OrderEvent {
    let id = Uuid::new_v4().to_string();
    let customer_id = Uuid::new_v4().to_string();
    let customer_name = "Taro Yamada".to_string();
    let order_items = vec![
        OrderItem {
            item_id: 1000,
            quantity: 2,
            unit_price: 1000,
        },
        OrderItem {
            item_id: 1001,
            quantity: 1,
            unit_price: 2000,
        },
    ];

    OrderEvent {
        id,
        customer_id,
        customer_name,
        order_items,
    }
}
