#![cfg(any(feature = "tauri"))]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod tauri;

#[cfg(feature = "tauri")]
pub type WebSocket = tauri::SocketHandle;
#[cfg(feature = "tauri")]
pub type Certificate = tauri::CertificateHandle;

#[derive(Serialize, Deserialize)]
pub struct SocketOptions {
    uri: String,
    headers: HashMap<String, String>,
    root_certificates: Vec<Certificate>,
}

impl SocketOptions {
    pub fn new(uri: String) -> SocketOptions {
        Self {
            uri,
            headers: HashMap::new(),
            root_certificates: Vec::new(),
        }
    }

    pub fn builder() -> SocketOptionsBuilder {
        SocketOptionsBuilder::new()
    }
}

pub struct SocketOptionsBuilder {
    uri: String,
    headers: HashMap<String, String>,
    root_certificates: Vec<Certificate>,
}

impl SocketOptionsBuilder {
    pub fn new() -> SocketOptionsBuilder {
        Self {
            uri: String::new(),
            headers: HashMap::new(),
            root_certificates: Vec::new(),
        }
    }

    pub fn uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = uri.into();
        self
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn add_root_certificate(mut self, cert: Certificate) -> Self {
        self.root_certificates.push(cert);
        self
    }

    pub fn build(self) -> SocketOptions {
        SocketOptions {
            uri: self.uri,
            headers: self.headers,
            root_certificates: self.root_certificates,
        }
    }
}
