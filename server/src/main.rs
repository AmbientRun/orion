use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, registry, util::SubscriberInitExt,
};
use tracing_tree::HierarchicalLayer;

fn main() {
    registry().with(HierarchicalLayer::new(4)).init();

    tracing::info!("It works...");
}
