use anyhow::Result;
use dashmap::DashMap;
use futures::{stream::SplitStream, SinkExt, StreamExt};
use std::{fmt, net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

const MAX_MESSAGES: usize = 128;

// State：包含所有连接的客户端的状态。
#[derive(Debug, Default)]
struct State {
    peers: DashMap<SocketAddr, mpsc::Sender<Arc<Message>>>,
}

#[derive(Debug)]
struct Peer {
    username: String,
    stream: SplitStream<Framed<TcpStream, LinesCodec>>,
}

#[derive(Debug)]
enum Message {
    UserJoined(String),
    UserLeft(String),
    Chat { sender: String, content: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await?;
    info!("Starting chat server on {}", addr);

    let state = Arc::new(State::default());

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Accepted connection from: {}", addr);
        let state_cloned = state.clone();

        tokio::spawn(async move {
            if let Err(r) = handle_client(state_cloned, stream, addr).await {
                warn!("Failed to handle client {}: {}", addr, r);
            }
        });
    }
}

async fn handle_client(state: Arc<State>, stream: TcpStream, addr: SocketAddr) -> Result<()> {
    let mut stream: Framed<TcpStream, LinesCodec> = Framed::new(stream, LinesCodec::new());

    let username = prompt_for_username(&mut stream).await?;
    let mut peer = state.add(addr, username, stream).await;

    // 广播用户加入
    broadcast_user_joined(&state, &peer.username, addr).await;

    // 接收客户端发送的消息
    while let Some(line) = peer.stream.next().await {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                warn!("Failed to read from {}: {}", addr, e);
                break;
            }
        };

        // 用户退出
        if line.trim() == "exit!" {
            info!("User {} requested to exit", peer.username);
            break;
        }
        broadcast_chat_message(&state, &peer.username, line, addr).await;
    }

    // 用户离开
    handle_user_exit(&state, addr, peer).await;

    Ok(())
}

// 提示用户输入用户名
async fn prompt_for_username(stream: &mut Framed<TcpStream, LinesCodec>) -> Result<String> {
    stream.send("Enter your username:").await?;

    match stream.next().await {
        Some(Ok(username)) => Ok(username),
        Some(Err(e)) => Err(e.into()),
        None => Err(anyhow::anyhow!("No username provided")),
    }
}

// 广播用户加入
async fn broadcast_user_joined(state: &Arc<State>, username: &str, addr: SocketAddr) {
    let message = Arc::new(Message::user_joined(username));
    info!("{}", message);
    state.broadcast(addr, message).await;
}

// 广播聊天消息
async fn broadcast_chat_message(
    state: &Arc<State>,
    username: &str,
    content: String,
    addr: SocketAddr,
) {
    let message = Arc::new(Message::chat(username, content));
    state.broadcast(addr, message).await;
}

// 用户离开
async fn handle_user_exit(state: &Arc<State>, addr: SocketAddr, peer: Peer) {
    state.peers.remove(&addr);
    let message = Arc::new(Message::user_left(&peer.username));
    info!("{}", message);
    state.broadcast(addr, message).await;
    drop(peer); // 确保资源释放
}

impl State {
    async fn add(
        &self,
        addr: SocketAddr,
        username: String,
        stream: Framed<TcpStream, LinesCodec>,
    ) -> Peer {
        let (tx, mut rx) = mpsc::channel(MAX_MESSAGES);

        self.peers.insert(addr, tx);

        let (mut stream_sender, stream_receiver) = stream.split();
        // receive messages from others, and send them to the client

        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = stream_sender.send(message.to_string()).await {
                    warn!("Failed to send message to {}: {}", addr, e);
                    break;
                }
            }
        });

        Peer {
            username,
            stream: stream_receiver,
        }
    }

    // async fn broadcast1(&self, addr: SocketAddr, message: Arc<Message>) {
    //     for peer in self.peers.iter() {
    //         if peer.key() == &addr {
    //             // 表示当前连接的客户端，不需要广播消息
    //             continue;
    //         }

    //         if let Err(e) = peer.value().send(message.clone()).await {
    //             warn!("Failed to send message to {}: {}", peer.key(), e);
    //             // 如果发送失败，那么表明该客户端已经断开连接，需要从map中移除
    //             self.peers.remove(peer.key());
    //         }
    //     }
    // }

    async fn broadcast(&self, addr: SocketAddr, message: Arc<Message>) {
        let tasks = self
            .peers
            .iter()
            .filter(|peer| peer.key() != &addr)
            .map(|peer| {
                let message_cloned = message.clone();

                async move {
                    if let Err(e) = peer.value().send(message_cloned).await {
                        warn!("Failed to send message to {}: {}", peer.key(), e);
                        // 如果发送失败，那么表明该客户端已经断开连接，需要从map中移除
                        self.peers.remove(peer.key());
                    }
                }
            })
            .collect::<Vec<_>>();

        futures::future::join_all(tasks).await;
    }
}

impl Message {
    fn user_joined(username: &str) -> Self {
        let content = format!("{} has joined the chat", username);
        Self::UserJoined(content)
    }

    fn user_left(username: &str) -> Self {
        let content = format!("{} has left the chat", username);
        Self::UserLeft(content)
    }

    fn chat(sender: impl Into<String>, content: impl Into<String>) -> Self {
        Self::Chat {
            sender: sender.into(),
            content: content.into(),
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UserJoined(content) => write!(f, "[{}]", content),
            Self::UserLeft(content) => write!(f, "[{} :(]", content),
            Self::Chat { sender, content } => write!(f, "{}: {}", sender, content),
        }
    }
}
