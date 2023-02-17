use std::net::SocketAddr;

use glam::{Vec2, Vec3};

pub struct Asteroid {
    pub color: Vec3,
    pub radius: f32,
    pub pos: Vec2,
    pub vel: Vec2,
    pub rot: f32,
    pub ang_vel: f32,
}

pub fn server_addr() -> SocketAddr {
    "127.0.0.1:5001".parse::<SocketAddr>().unwrap()
}
