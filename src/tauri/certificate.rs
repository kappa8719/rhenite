use native_tls::Certificate;
use serde::{Deserialize, Serialize};
use snowflake::Snowflake;
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};
use tauri::State;
use thiserror::Error;

#[derive(Default)]
pub(super) struct CertificateManager(pub Mutex<HashMap<CertificateHandle, Certificate>>);

impl CertificateManager {
    pub fn acquire(&self) -> MutexGuard<'_, HashMap<CertificateHandle, Certificate>> {
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

impl From<native_tls::Error> for CreateCertificateError {
    fn from(err: native_tls::Error) -> Self {
        Self::NativeTls(err.to_string())
    }
}

#[derive(Serialize, Deserialize)]
struct __CreateCertificateArgs {
    pub bytes: Vec<u8>,
}

#[tauri::command]
pub(super) async fn __create_certificate_from_pem(
    snowflake: State<'_, Snowflake>,
    manager: State<'_, CertificateManager>,
    bytes: Vec<u8>,
) -> Result<CertificateHandle, CreateCertificateError> {
    let certificate = Certificate::from_pem(bytes.as_slice())?;
    let mut certificates = manager.acquire();
    let handle = CertificateHandle(snowflake.next_id());
    certificates.insert(handle, certificate);

    Ok(handle)
}

#[tauri::command]
pub(super) async fn __create_certificate_from_der(
    snowflake: State<'_, Snowflake>,
    manager: State<'_, CertificateManager>,
    bytes: Vec<u8>,
) -> Result<CertificateHandle, CreateCertificateError> {
    let certificate = Certificate::from_der(bytes.as_slice())?;
    let mut certificates = manager.acquire();
    let handle = CertificateHandle(snowflake.next_id());
    certificates.insert(handle, certificate);

    Ok(handle)
}