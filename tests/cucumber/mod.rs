mod content;
mod creation;
mod preparation;
mod world;

use cucumber_rust::WorldInit;
use world::IncrementalWorld;

const DIR: bool = true;
const PW: bool = false;

fn main() {
    let runner = IncrementalWorld::init(&["./features"]);

    // You may choose any executor you like (Tokio, async-std, etc)
    // You may even have an async main, it doesn't matter. The point is that
    // Cucumber is composable. :)
    futures::executor::block_on(runner.run());
}
