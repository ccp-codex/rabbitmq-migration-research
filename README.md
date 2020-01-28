# Research RabbitMQ Migration Strategies

When moving "locations" of a RabbitMQ cluster, we're considering three options given that we're running two clusters at the same time, one at the new location and one at the old location:

- Using the [Federation](https://www.rabbitmq.com/federation.html) plugin
- Using the [Shovel](https://www.rabbitmq.com/shovel.html) plugin
- Clustering both clusters together, and let HA queues do its job while killing each old node

Our plan was to test each one but seems the first option solves the problem, so the test stopped there.

## Federation Setup

Reading the documentation my only concern is whether it's possible to setup federation links in both directions, without causing messages to bounce back-and-forth forever.

Running `federation.sh` will spin up two RabbitMQ containers, setup a vhost `fed-test` and create federation links in both directions for all exchanges in this vhost.

Use `cleanup.sh` to remove all docker resources at any stage.

## Federation Test

To test the setup I added a simple application. Which creates a publisher and a consumer on both nodes, a good result would be that both consumers receive messages published by both publishers _without repetition_.

And sure it behaves:

```
$ RUST_LOG=info cargo run
2020-01-28T14:58:43.776 INFO  testapp > connecting to node A
2020-01-28T14:58:44.426 INFO  testapp > connecting to node B
2020-01-28T14:58:45.088 INFO  testapp > consuming from node A
2020-01-28T14:58:45.090 INFO  testapp > publishing from node B
2020-01-28T14:58:45.090 INFO  testapp > consuming from node B
2020-01-28T14:58:45.090 INFO  testapp > publishing from node A
2020-01-28T14:58:50.100 INFO  testapp > -> B, received: Msg 10 from node B
2020-01-28T14:58:50.100 INFO  testapp > -> A, received: Msg 0 from node A
2020-01-28T14:58:50.102 INFO  testapp > -> B, received: Msg 0 from node A
2020-01-28T14:58:50.103 INFO  testapp > -> A, received: Msg 10 from node B
2020-01-28T14:58:55.103 INFO  testapp > -> A, received: Msg 1 from node A
2020-01-28T14:58:55.103 INFO  testapp > -> B, received: Msg 11 from node B
2020-01-28T14:58:55.106 INFO  testapp > -> A, received: Msg 11 from node B
2020-01-28T14:58:55.108 INFO  testapp > -> B, received: Msg 1 from node A
```

For example: "Msg 0" which is sent from node A, got received by both consumers, and not repeated ever after. Same goes for other messages.

So it looks like this would solve our problem.

## Considerations

One thing to notice though, is that messages are duplicated on both clusters (nodes). This could be a problem if the application is consuming (and publishing) from both clusters. Say if you have a deployment in Kubernetes and rolling update it to point it to the new cluster, the application would need to/should handle deduplication itself.
