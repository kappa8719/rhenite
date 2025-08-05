use crate::SocketOptions;
use crate::tauri::ipc;
use crate::tauri::ipc::Message;
use futures_util::{SinkExt, StreamExt};
use http::{HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use snowflake::Snowflake;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Mutex, MutexGuard};

#[cfg(feature = "tauri-host")]
use tauri::{AppHandle, Emitter, Runtime, State};

#[derive(Clone, Debug)]
pub(super) struct SocketInstance {
    transmitter: tokio::sync::mpsc::UnboundedSender<ipc::Message>,
}

#[derive(Default)]
pub(super) struct SocketManager(Mutex<HashMap<SocketHandle, SocketInstance>>);

impl SocketManager {
    pub fn sockets(&self) -> MutexGuard<'_, HashMap<SocketHandle, SocketInstance>> {
        self.0.lock().unwrap_or_else(|error| error.into_inner())
    }

    pub fn get(&self, handle: SocketHandle) -> Option<SocketInstance> {
        let sockets = self.sockets();
        sockets.get(&handle).map(Clone::clone)
    }
}

#[derive(Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct SocketHandle(u64);

impl SocketHandle {
    pub async fn new(options: SocketOptions) -> Self {
        crate::tauri::invoke("__create_socket", &__CreateSocketArgs { options }).await
    }

    pub fn id(&self) -> u64 {
        self.0
    }

    fn event_name(&self) -> String {
        format!("__rhenite_socket/{}", self.id())
    }

    pub async fn send(&self, message: ipc::Message) {
        crate::tauri::invoke::<()>(
            "__send_message_to_socket",
            &__SendMessageArgs {
                handle: self.clone(),
                message: message.into(),
            },
        )
        .await;
    }

    pub async fn close(&self) {
        self.send(ipc::Message::Close(None)).await;
    }

    pub async fn close_with_reason(&self, frame: ipc::CloseFrame) {
        self.send(ipc::Message::Close(Some(frame))).await;
    }
}

#[derive(Serialize, Deserialize)]
struct __CreateSocketArgs {
    options: SocketOptions,
}

#[derive(Serialize, Deserialize)]
struct __SendMessageArgs {
    handle: SocketHandle,
    message: Message,
}

#[cfg(feature = "tauri-host")]
#[tauri::command]
pub(super) async fn __create_socket<R: Runtime>(
    app: AppHandle<R>,
    snowflake: State<'_, Snowflake>,
    socket_manager: State<'_, SocketManager>,
    certificate_manager: State<'_, super::certificate::CertificateManager>,
    options: SocketOptions,
) -> Result<SocketHandle, ()> {
    use tungstenite::protocol::WebSocketConfig;

    let mut request = tungstenite::handshake::client::Request::builder()
        .uri(options.uri)
        .body(())
        .unwrap();

    for (k, v) in options.headers {
        request.headers_mut().insert(
            HeaderName::from_str(k.as_str()).unwrap(),
            HeaderValue::from_str(v.as_str()).unwrap(),
        );
    }

    let mut builder = native_tls::TlsConnector::builder();

    // Add certificates
    {
        let certificates = certificate_manager.0.lock().unwrap();
        for handle in options.root_certificates {
            let Some(certificate) = certificates.get(&handle) else {
                continue;
            };

            builder.add_root_certificate(certificate.clone());
        }
    }

    let connector = tokio_tungstenite::Connector::NativeTls(builder.build().unwrap());

    let (stream, _) = tokio_tungstenite::connect_async_tls_with_config(
        request,
        Some(WebSocketConfig::default()),
        true,
        Some(connector),
    )
    .await
    .unwrap();

    let (mut sink, stream) = stream.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    let handle = SocketHandle(snowflake.next_id());

    // handle incoming
    tokio::spawn({
        let event_name = handle.event_name();

        stream.for_each({
            let event_name = event_name;

            move |message| {
                let app = app.clone();
                let event_name = event_name.clone();

                async move {
                    let Ok(message) = message else {
                        return;
                    };

                    let _ = app.emit(event_name.as_str(), Message::from(message));
                }
            }
        })
    });

    // handle outgoing
    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            let _ = sink.send(message.into()).await;
        }
    });

    let mut sockets = socket_manager.0.lock().unwrap();
    sockets.insert(handle, SocketInstance { transmitter: tx });

    Ok(handle)
}

#[cfg(feature = "tauri-host")]
#[tauri::command]
pub(super) async fn __send_message_to_socket(
    socket_manager: State<'_, SocketManager>,
    handle: SocketHandle,
    message: Message,
) -> Result<(), ()> {
    let Some(socket) = socket_manager.get(handle) else {
        return Err(());
    };

    if let Err(_) = socket.transmitter.send(message.into()) {
        return Err(());
    }

    Ok(())
}
