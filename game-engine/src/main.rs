mod board;
mod ai;
mod rules;
mod rabbit;
mod config;

use crate::rabbit::lapin_listen;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    let config = config::get_config();
    let mut cfg = deadpool_lapin::Config::default();
    cfg.url = Some(config.rabbit.into());
    let lapin_pool = cfg.create_pool(Some(deadpool_lapin::Runtime::Tokio1)).unwrap();
    lapin_listen(lapin_pool.clone()).await;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Color {
    White,
    Red,
}

const BIT_MASK: u32 = 0b1000_0000_0000_0000_0000_0000_0000_0000;
