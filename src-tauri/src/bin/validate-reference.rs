//! Development-only stdin validator; shares the production catalog contract.
use std::io::{self, Read};
use trainer_collection_manager::reference::Catalog;

fn main() {
    let mut bytes = Vec::new();
    let result = io::stdin()
        .take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Could not read reference snapshot")
        .and_then(|_| Catalog::load(&bytes).map(|_| ()));
    if let Err(message) = result {
        eprintln!("{message}");
        std::process::exit(1);
    }
    println!("Reference snapshot validated (schema 1)");
}
