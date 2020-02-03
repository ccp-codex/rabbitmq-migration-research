use anyhow::Result;
use futures::{future::FutureExt, stream::StreamExt};
use lapin::{options::*, types::FieldTable, BasicProperties, Connection, ConnectionProperties, ExchangeKind};
use log::info;
use std::time::Duration;
use tokio::time::delay_for;

#[tokio::main]
async fn main() -> Result<()> {
  pretty_env_logger::init_timed();

  // opening connections and channels
  info!("connecting to node A");
  let node_a = "amqp://guest:guest@localhost:5001/fed-test";
  let conn_a = Connection::connect(node_a, ConnectionProperties::default())
    .await
    .expect("can't connect to node A");
  let pub_channel_a = conn_a
    .create_channel()
    .await
    .expect("can't open publish channel on node A");
  let con_channel_a = conn_a
    .create_channel()
    .await
    .expect("can't open consume channel on node A");

  // info!("connecting to node B");
  // let node_b = "amqp://guest:guest@localhost:5002/fed-test";
  // let conn_b = Connection::connect(node_b, ConnectionProperties::default())
  //   .await
  //   .expect("can't connect to node B");
  // let pub_channel_b = conn_b
  //   .create_channel()
  //   .await
  //   .expect("can't open publish channel on node B");
  // let con_channel_b = conn_b
  //   .create_channel()
  //   .await
  //   .expect("can't open consume channel on node B");

  // expected to use same exchange and queue names on both nodes
  let exchange_name = "fed-test-exchange";
  let queue_name = "fed-test-queue";

  // declaring exchanges
  pub_channel_a
    .exchange_declare(
      exchange_name,
      ExchangeKind::Topic,
      ExchangeDeclareOptions {
        durable: true,
        ..Default::default()
      },
      FieldTable::default(),
    )
    .wait()
    .expect("can't declare exchange on node A");

  // pub_channel_b
  //   .exchange_declare(
  //     exchange_name,
  //     ExchangeKind::Topic,
  //     ExchangeDeclareOptions {
  //       durable: true,
  //       ..Default::default()
  //     },
  //     FieldTable::default(),
  //   )
  //   .wait()
  //   .expect("can't declare exchange on node B");

  // declare queues and bindings
  let queue_a = con_channel_a
    .queue_declare(
      queue_name,
      QueueDeclareOptions {
        exclusive: true,
        auto_delete: true,
        ..Default::default()
      },
      FieldTable::default(),
    )
    .await
    .expect("can't declare queue on node A");

  con_channel_a
    .queue_bind(
      queue_name,
      exchange_name,
      "#",
      QueueBindOptions::default(),
      FieldTable::default(),
    )
    .wait()
    .expect("can't bind queue on node A");

  // let queue_b = con_channel_b
  //   .queue_declare(
  //     queue_name,
  //     QueueDeclareOptions {
  //       exclusive: true,
  //       auto_delete: true,
  //       ..Default::default()
  //     },
  //     FieldTable::default(),
  //   )
  //   .await
  //   .expect("can't declare queue on node B");

  // con_channel_b
  //   .queue_bind(
  //     queue_name,
  //     exchange_name,
  //     "#",
  //     QueueBindOptions::default(),
  //     FieldTable::default(),
  //   )
  //   .wait()
  //   .expect("can't bind queue on node B");

  // start consumers
  let consumer_a = con_channel_a
    .clone()
    .basic_consume(
      &queue_a,
      "consumer-a",
      BasicConsumeOptions::default(),
      FieldTable::default(),
    )
    .await
    .expect("can't consume from node A");

  tokio::spawn(async move {
    info!("consuming from node A");

    consumer_a
      .for_each(move |delivery| {
        let msg = delivery.expect("failed to receive from node A");
        info!("-> A, received: {}", String::from_utf8(msg.data).unwrap());
        con_channel_a
          .basic_ack(msg.delivery_tag, BasicAckOptions::default())
          .map(|_| ())
      })
      .await
  });

  // let consumer_b = con_channel_b
  //   .clone()
  //   .basic_consume(
  //     &queue_b,
  //     "consumer-b",
  //     BasicConsumeOptions::default(),
  //     FieldTable::default(),
  //   )
  //   .await
  //   .expect("can't consume from node B");

  // tokio::spawn(async move {
  //   info!("consuming from node B");

  //   consumer_b
  //     .for_each(move |delivery| {
  //       let msg = delivery.expect("failed to receive from node B");
  //       info!("-> B, received: {}", String::from_utf8(msg.data).unwrap());
  //       con_channel_b
  //         .basic_ack(msg.delivery_tag, BasicAckOptions::default())
  //         .map(|_| ())
  //     })
  //     .await
  // });

  // start publishing on both nodes
  //tokio::spawn(async move {
  info!("publishing from node A");

  let mut cnt = 0;

  loop {
    delay_for(Duration::from_millis(5000)).await;

    let payload_a = format!("Msg {} from node A", cnt).into_bytes();
    cnt += 1;

    pub_channel_a
      .basic_publish(
        exchange_name,
        "route.a",
        BasicPublishOptions::default(),
        payload_a.to_vec(),
        BasicProperties::default(),
      )
      .await
      .unwrap();
  }
  //});

  // info!("publishing from node B");
  // let mut cnt = 10;

  // loop {
  //   delay_for(Duration::from_millis(5000)).await;

  //   let payload_b = format!("Msg {} from node B", cnt).into_bytes();
  //   cnt += 1;

  //   pub_channel_b
  //     .basic_publish(
  //       exchange_name,
  //       "route.b",
  //       BasicPublishOptions::default(),
  //       payload_b.to_vec(),
  //       BasicProperties::default(),
  //     )
  //     .await
  //     .unwrap();
  // }
}
