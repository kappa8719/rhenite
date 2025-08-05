use serde::{Deserialize, Serialize};
use snowflake::Snowflake;
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};
use tauri::State;
use thiserror::Error;

#[cfg(feature = "tauri-host")]
#[derive(Default)]
pub(super) struct CertificateManager(pub Mutex<HashMap<CertificateHandle, native_tls::Certificate>>);

#[cfg(feature = "tauri-host")]
impl CertificateManager {
    pub fn acquire(&self) -> MutexGuard<'_, HashMap<CertificateHandle, native_tls::Certificate>> {
        self.0.lock().unwrap_or_else(|error| error.into_inner())
    }
}

#[derive(Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct CertificateHandle(u64);

impl CertificateHandle {
    pub async fn from_pem(bytes: &[u8]) -> Self {
        crate::tauri::invoke(
            "__create_certificate_from_pem",
            &__CreateCertificateArgs {
                bytes: bytes.to_vec(),
            },
        )
        .await
    }

    pub async fn from_der(bytes: &[u8]) -> Self {
        crate::tauri::invoke(
            "__create_certificate_from_der",
            &__CreateCertificateArgs {
                bytes: bytes.to_vec(),
            },
        )
        .await
    }
}

#[derive(Serialize, Deserialize, Error, Debug)]
pub(super) enum CreateCertificateError {
    #[error("native_tls: {0}")]
    NativeTls(String),
}

#[cfg(feature = "tauri-host")]
impl From<native_tls::Error> for CreateCertificateError {
    fn from(err: native_tls::Error) -> Self {
        Self::NativeTls(err.to_string())
    }
}

#[derive(Serialize, Deserialize)]
struct __CreateCertificateArgs {
    pub bytes: Vec<u8>,
}

#[cfg(feature = "tauri-host")]
#[tauri::command]
pub(super) async fn __create_certificate_from_pem(
    snowflake: State<'_, Snowflake>,
    manager: State<'_, CertificateManager>,
    bytes: Vec<u8>,
) -> Result<CertificateHandle, CreateCertificateError> {
    let certificate = native_tls::Certificate::from_pem(bytes.as_slice())?;
    let mut certificates = manager.acquire();
    let handle = CertificateHandle(snowflake.next_id());
    certificates.insert(handle, certificate);

    Ok(handle)
}

#[cfg(feature = "tauri-host")]
#[tauri::command]
pub(super) async fn __create_certificate_from_der(
    snowflake: State<'_, Snowflake>,
    manager: State<'_, CertificateManager>,
    bytes: Vec<u8>,
) -> Result<CertificateHandle, CreateCertificateError> {
    let certificate = native_tls::Certificate::from_der(bytes.as_slice())?;
    let mut certificates = manager.acquire();
    let handle = CertificateHandle(snowflake.next_id());
    certificates.insert(handle, certificate);

    Ok(handle)
}
