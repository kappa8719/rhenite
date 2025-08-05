#![cfg(feature = "tauri")]

mod certificate;
mod ipc;
mod socket;

pub use certificate::CertificateHandle;
use serde::Serialize;
use snowflake::Snowflake;
pub use socket::SocketHandle;
use tauri::plugin::TauriPlugin;
use tauri::{generate_handler, Manager, Runtime};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
unsafe extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
    async fn __invoke_without_args(cmd: &str) -> JsValue;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
    async fn __invoke(cmd: &str, args: JsValue) -> JsValue;
}

const fn invoke_serializer() -> serde_wasm_bindgen::Serializer {
    serde_wasm_bindgen::Serializer::new().serialize_large_number_types_as_bigints(true)
}

async fn invoke_without_args<T>(cmd: &str) -> T
where
    T: for<'a> serde::de::Deserialize<'a>,
{
    serde_wasm_bindgen::from_value(
        __invoke_without_args(format!("plugin:rhenite-tauri|{cmd}").as_str()).await,
    )
    .unwrap()
}

async fn invoke<T>(cmd: &str, args: &impl Serialize) -> T
where
    T: for<'a> serde::de::Deserialize<'a>,
{
    let serializer = invoke_serializer();
    let serialized_args = args.serialize(&serializer).unwrap();

    serde_wasm_bindgen::from_value(
        __invoke(
            format!("plugin:rhenite-tauri|{cmd}").as_str(),
            serialized_args,
        )
        .await,
    )
    .unwrap()
}

#[cfg(feature = "tauri-host")]
pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    use crate::tauri::certificate::CertificateManager;
    use crate::tauri::socket::SocketManager;
    
    tauri::plugin::Builder::new("rhenite-tauri")
        .invoke_handler(generate_handler![
            socket::__create_socket,
            certificate::__create_certificate_from_pem,
            certificate::__create_certificate_from_der
        ])
        .setup(|app, api| {
            app.manage(Snowflake::new(0));
            app.manage(SocketManager::default());
            app.manage(CertificateManager::default());

            Ok(())
        })
        .build()
}
