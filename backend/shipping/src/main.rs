use deadpool_lapin::Runtime;
use deadpool_lapin::lapin::options::{BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions};
use deadpool_lapin::lapin::types::FieldTable;
use domain::models::OrderEvent;
use futures_lite::StreamExt as _;

use utils::amqp::amqp_config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let num_cpus = num_cpus::get();
    let config = amqp_config(num_cpus);
    let pool = config.create_pool(Some(Runtime::Tokio1))?;
    let conn = pool.get().await?;
    let channel = conn.create_channel().await?;
    let queue = channel
        .queue_declare(
            "order",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    // BasicConsumeOptionsは、メッセージを受信するためのオプションを指定するための構造体
    //    no_local: 同じ接続で送信されたメッセージを受信しない（RabbitMQは未対応）
    //    no_ack: メッセージを受信したとき、確認応答をしない。
    //    exclusive: コンシューマーがキューの唯一の利用者になる。
    //    no_wait: コンシューマーの確認応答を待たずに処理を続ける。
    let mut consumer = channel
        .basic_consume(
            queue.name().as_str(),
            "", // コンシューマータグ、コンシューマーの識別子、省略すると自動生成される。
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            let body = String::from_utf8_lossy(&delivery.data);
            println!("Received data: {}", body);
            let order_event: OrderEvent = serde_json::from_str(&body)?;
            println!("Received OrderEvent: {:?}", order_event);

            delivery.ack(BasicAckOptions::default()).await?;
        } else {
            println!("Error receiving message");
        }
    }

    Ok(())
}
