use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::broadcast,
};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("localhost:10000").await.unwrap();
    let (tx, _rx) = broadcast::channel::<String>(10);
    loop {
        let tx = tx.clone();
        let mut rx = tx.subscribe();
        let (mut socket, addr) = listener.accept().await.unwrap();
        tokio::spawn(async move {

            let (reader, mut writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            loop {
                tokio::select! {
                    _this = reader.read_line(&mut line) =>{
                        tx.send(format!("{}: {}",addr,line.clone())).unwrap();line.clear();
                    },
                    queue = rx.recv() =>{
                        let msg = queue.as_ref().unwrap().split(": ").collect::<Vec<&str>>();
                        if msg[0] != addr.to_string(){
                        writer.write_all(queue.unwrap().as_bytes()).await.unwrap();}
                    }
                }
            }
        });
    }
}
