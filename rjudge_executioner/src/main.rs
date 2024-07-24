use core::str;

use amqprs::{
    callbacks::{
        ChannelCallback, ConnectionCallback, DefaultChannelCallback, DefaultConnectionCallback,
    },
    channel::{
        BasicAckArguments, BasicConsumeArguments, QueueBindArguments, QueueDeclareArguments,
    },
    connection::{Connection, OpenConnectionArguments},
    consumer::DefaultConsumer,
};
use piston_rs::Executor;
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
        // .manual_ack(false) // auto ack
        .finish();

    let (_ctag, mut rx) = channel.basic_consume_rx(args).await.unwrap();

    tokio::spawn(async move {
        let client = piston_rs::Client::with_url("http://localhost:2000/api/v2");

        while let Some(msg) = rx.recv().await {
            if let Some(payload) = msg.content {
                channel
                    .basic_ack(BasicAckArguments::new(
                        msg.deliver.unwrap().delivery_tag(),
                        false,
                    ))
                    .await
                    .unwrap();

                let executor: Executor = match serde_json::from_slice(&payload) {
                    Ok(executor) => executor,
                    Err(e) => {
                        println!("[x] Failed to parse payload: {:?}", e);
                        continue;
                    }
                };
                println!("[x] Received {:?}", executor);
                if let Ok(response) = client.execute(&executor).await {
                    println!("[x] Piston Response: {:?}", response);
                } else {
                    println!("[x] Failed to install package");
                }
            }
        }
    });

    println!("[x] consume forever…, ctrl+c to exit");
    let guard = Notify::new();
    guard.notified().await;
}
