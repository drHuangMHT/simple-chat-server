use mini_redis::{Connection, Frame};
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    sync::{Arc, Mutex},
};
use tokio::net::{TcpListener, TcpStream};
type ShardedDb = Arc<Vec<Mutex<HashMap<String, Vec<u8>>>>>;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let mut shard_count = 0;
    let mut shards = vec![Mutex::new(HashMap::new())];
    while shard_count < 8 {
        shards.push(Mutex::new(HashMap::new()));
        shard_count += 1
    }
    let db = Arc::new(shards);
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let db = db.clone();
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}
async fn process(socket: TcpStream, db: ShardedDb) {
    use mini_redis::Command::{self, Get, Set};
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                let mut shard = db[get_index(cmd.key(), 8)].lock().unwrap();
                shard.insert(cmd.key().into(), cmd.value().to_vec());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                let shard = db[get_index(cmd.key(), 8)].lock().unwrap();
                if let Some(value) = shard.get(cmd.key()) {
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented: {:?}", cmd),
        };
        connection.write_frame(&response).await.unwrap();
    }
}

fn get_index<T:Hash>(target:T,shard_count:u64)->usize
{
    let mut hasher = DefaultHasher::new();
    target.hash(&mut hasher);
    let i = hasher.finish() % shard_count;
    i.try_into().unwrap()
}