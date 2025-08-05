#[tokio::main]
async fn main() {
    let socket = rhenite::WebSocket::new(
        rhenite::SocketOptions::builder()
            .uri("")
            .header("", "")
            .build(),
    )
    .await;

    socket.close().await;
}
