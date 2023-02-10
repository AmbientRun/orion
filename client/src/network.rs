use std::{net::SocketAddr, sync::Arc};

use quinn::{ClientConfig, Connection, Endpoint};

/// Represents a connected client
pub struct Client {
    connection: Connection,
}

impl Client {
    pub async fn connect(addr: SocketAddr) -> anyhow::Result<Self> {
        let config = configure_client();

        // Bind this endpoint to a UDP socket on the given client address.
        let mut endpoint = Endpoint::client("0.0.0.0:0".parse::<SocketAddr>().unwrap())?;

        // Connect to the server passing in the server name which is supposed to be in the server certificate.
        let connection = endpoint.connect(addr, "localhost")?.await?;

        // Start transferring, receiving data, see data transfer page.

        Ok(Self { connection })
    }
}

// Implementation of `ServerCertVerifier` that verifies everything as trustworthy.
struct SkipServerVerification;

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl rustls::client::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

fn configure_client() -> ClientConfig {
    let crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    ClientConfig::new(Arc::new(crypto))
}
