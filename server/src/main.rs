use std::{error::Error, f32::consts::TAU, net::SocketAddr, sync::Arc};

use glam::{vec2, Vec2};
use itertools::Itertools;
use orion_shared::Asteroid;
use rand::{thread_rng, Rng};
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, registry, util::SubscriberInitExt,
};
use tracing_tree::HierarchicalLayer;

fn main() {
    registry().with(HierarchicalLayer::new(4)).init();

    tracing::info!("It works...");
}
