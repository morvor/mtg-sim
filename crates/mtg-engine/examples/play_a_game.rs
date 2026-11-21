//! Plays one game between two random agents and prints its log.
//!
//! `cargo run --release -p mtg-engine --example play_a_game`

use mtg_engine::agents::RandomAgent;
use mtg_engine::*;
use std::sync::Arc;

fn deck(cards: &[(usize, &str)]) -> Vec<Arc<CardDef>> {
    cards
        .iter()
        .flat_map(|&(n, name)| std::iter::repeat_n(card(name), n))
        .collect()
}

fn main() {
    let red = deck(&[
        (24, "Mountain"),
        (20, "Lightning Bolt"),
        (16, "Raging Goblin"),
    ]);
    let green = deck(&[(24, "Forest"), (20, "Grizzly Bears"), (16, "Giant Growth")]);
    let agents: Vec<Box<dyn Agent>> =
        vec![Box::new(RandomAgent::new(1)), Box::new(RandomAgent::new(2))];
    let config = GameConfig {
        seed: 7,
        ..Default::default()
    };
    let mut game = Game::new(config, vec![red, green], agents);
    game.logging = true;
    let result = game.run();
    for line in &game.log {
        println!("[T{}] {}", line.turn, line.text);
    }
    println!("{result:?} after {} turns", game.turn.number);
}
