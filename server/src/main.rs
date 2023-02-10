use std::{error::Error, f32::consts::TAU, net::SocketAddr, sync::Arc};

use glam::{vec2, Vec2};
use itertools::Itertools;
use orion_shared::Asteroid;
use quinn::{ClientConfig, Endpoint, ServerConfig};
use rand::{thread_rng, Rng};
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, registry, util::SubscriberInitExt,
};
use tracing_tree::HierarchicalLayer;

pub struct Game {
    asteroids: Vec<Asteroid>,
}

impl Game {
    pub fn new() -> Self {
        let mut rng = thread_rng();

        let asteroids = (0..16)
            .map(|_| {
                let dir = rng.gen_range(0.0..TAU);
                let vel = vec2(dir.cos(), dir.sin()) * rng.gen_range(0.0..2.0);
                Asteroid {
                    size: rng.gen_range(10.0..20.0),
                    color: rng.gen(),
                    pos: rng.gen::<Vec2>() * 512.0,
                    vel,
                }
            })
            .collect_vec();

        Self { asteroids }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ConnectedClient {}

pub struct ServerState {
    game: Game,
    clients: Vec<ConnectedClient>,
}

fn client_addr() -> SocketAddr {
    "127.0.0.1:5000".parse::<SocketAddr>().unwrap()
}

fn server_addr() -> SocketAddr {
    "127.0.0.1:5001".parse::<SocketAddr>().unwrap()
}

#[tokio::main]
async fn main() {
    registry().with(HierarchicalLayer::new(4)).init();
    let server = ServerState::new();
    server.run().await.unwrap();
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            game: Game::new(),
            clients: Vec::new(),
        }
    }

    async fn run(self) -> anyhow::Result<()> {
        let (certs, key) = generate_self_signed_cert()?;

        let config = ServerConfig::with_single_cert(vec![certs], key)?;

        // Bind this endpoint to a UDP socket on the given server address.
        let endpoint = Endpoint::server(config, server_addr())?;

        // Start iterating over incoming connections.
        tracing::info!("Listening for connections");
        while let Some(conn) = endpoint.accept().await {
            let mut connection = conn.await?;

            // Save connection somewhere, start transferring, receiving data, see DataTransfer tutorial.
        }

        Ok(())
    }
}

fn generate_self_signed_cert() -> Result<(rustls::Certificate, rustls::PrivateKey), anyhow::Error> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])?;
    let key = rustls::PrivateKey(cert.serialize_private_key_der());
    Ok((rustls::Certificate(cert.serialize_der()?), key))
}
