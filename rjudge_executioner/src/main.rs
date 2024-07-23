use core::str;

use amqprs::{
    callbacks::{
        ChannelCallback, ConnectionCallback, DefaultChannelCallback, DefaultConnectionCallback,
    },
    channel::{BasicConsumeArguments, QueueBindArguments, QueueDeclareArguments},
    connection::{Connection, OpenConnectionArguments},
    consumer::DefaultConsumer,
};
use tokio::sync::Notify;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    let connection = Connection::open(&OpenConnectionArguments::new(
        "localhost",
        5672,
        "guest",
        "guest",
    ))
    .await
    .unwrap();

    connection
        .register_callback(DefaultConnectionCallback)
        .await
        .unwrap();

    let channel = connection.open_channel(None).await.unwrap();
    channel
        .register_callback(DefaultChannelCallback)
        .await
        .unwrap();

    let (queue_name, _, _) = channel
        .queue_declare(QueueDeclareArguments::durable_client_named("submission"))
        .await
        .unwrap()
        .unwrap();

    let routing_key = "amqprs.example";
    let exchange_name = "amq.topic";
    channel
        .queue_bind(QueueBindArguments::new(
            &queue_name,
            &exchange_name,
            routing_key,
        ))
        .await
        .unwrap();

    let args = BasicConsumeArguments::new(&queue_name, "basic_consumer")
        .manual_ack(false)
        .finish();

    let (_ctag, mut rx) = channel.basic_consume_rx(args).await.unwrap();

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Some(payload) = msg.content {
                println!("[x] Received {:?}", str::from_utf8(&payload).unwrap());
            }
        }
    });

    println!("[x] consume forever…, ctrl+c to exit");
    let guard = Notify::new();
    guard.notified().await;
}
