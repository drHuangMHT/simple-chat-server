use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        resp: Responder<()>,
    },
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();

    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        tx.send(Command::Get {
            key: "hello".into(),
            resp: resp_tx,
        })
        .await
        .unwrap();
        println!("Got = {:?}", resp_rx.await.unwrap())
    });
    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        tx2.send(Command::Set {
            key: "foo".into(),
            val: "bar".into(),
            resp: resp_tx,
        })
        .await
        .unwrap();
        println!("Got = {:?}", resp_rx.await.unwrap())
    });
    let mgr = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();
        while let Some(cmd) = rx.recv().await {
            use Command::*;
            match cmd {
                Get { key, resp } => {resp.send(client.get(&key).await).unwrap();}
                Set { key, val, resp } => {resp.send(client.set(&key, val).await).unwrap();}
            }
        }
    });
    t1.await.unwrap();
    t2.await.unwrap();
    mgr.await.unwrap();
}
