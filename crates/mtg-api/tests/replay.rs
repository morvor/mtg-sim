//! Determinism and replay (docs/PROPERTIES.md, property 4): a game played through the
//! protocol and replayed from its transcript's answers asks the same requests, in the
//! same order, and ends the same way.

mod common;

use mtg_api::replay::{replay, replay_check, Transcript};
use mtg_api::ProtocolOptions;
use mtg_engine::*;

fn config(seed: u64) -> GameConfig {
    GameConfig {
        seed,
        max_turns: 20,
        ..Default::default()
    }
}

#[test]
fn session_games_replay_from_their_transcripts() {
    for seed in 0..3u64 {
        let n = if seed == 2 { 3 } else { 2 };
        let decks: Vec<_> = (0..n).map(|i| common::random_deck(seed * 10 + i)).collect();
        let (summary, _) = replay_check(config(seed), decks, seed + 100, 50_000)
            .unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        assert!(summary.requests > 20, "seed {seed}: a real game was played");
    }
}

#[test]
fn a_replay_notices_a_different_game() {
    let decks = vec![common::simple_deck(), common::simple_deck()];
    let (_, text) = replay_check(config(5), decks.clone(), 6, 50_000).unwrap();
    let t = Transcript::parse(&text);
    // Another seed shuffles the libraries differently: the first request already differs.
    let (_, d) = replay(config(6), decks.clone(), &t, ProtocolOptions::default());
    assert!(
        d.first().is_some_and(|d| d.contains("differs")),
        "divergences: {d:?}"
    );
    // The same game replays.
    let (_, d) = replay(config(5), decks, &t, ProtocolOptions::default());
    assert!(d.is_empty(), "{d:?}");
}

/// The example client, if Python is available.
fn python() -> Option<String> {
    ["python3", "python"]
        .into_iter()
        .find(|p| {
            std::process::Command::new(p)
                .arg("--version")
                .output()
                .is_ok_and(|o| o.status.success())
        })
        .map(str::to_string)
}

#[test]
fn a_child_process_game_replays_from_its_transcript_file() {
    let Some(py) = python() else {
        eprintln!("python3 not found: skipping the child process replay test");
        return;
    };
    let client = format!(
        "{}/../../examples/random_client.py",
        env!("CARGO_MANIFEST_DIR")
    );
    let dir = std::env::temp_dir().join(format!("mtg-api-replay-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("transcript.txt");
    let _ = std::fs::remove_file(&path);
    let decks = vec![common::random_deck(70), common::random_deck(71)];
    let mut agents: Vec<Box<dyn Agent>> = Vec::new();
    let mut conns = Vec::new();
    for seat in 0..2u8 {
        let cmd = format!("MTG_CLIENT_SEED={} {py} {client}", 7 + seat as u64);
        let (agent, conn) =
            mtg_api::spawn_external(&cmd, PlayerId(seat), ProtocolOptions::default()).unwrap();
        conn.transcript(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .unwrap(),
        );
        agents.push(Box::new(agent));
        conns.push(conn);
    }
    let mut g = Game::new(config(8), decks.clone(), agents);
    mtg_api::prepare_game(&mut g);
    g.run();
    for (seat, mut conn) in conns.into_iter().enumerate() {
        mtg_api::Transport::game_over(&mut conn, &mtg_api::GameOver::new(&g, PlayerId(seat as u8)));
        conn.shut_down();
    }
    let text = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    let t = Transcript::parse(&text);
    assert!(t.seats.values().all(|s| s.exchanges.len() > 5));
    let (turns, d) = replay(config(8), decks, &t, ProtocolOptions::default());
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(turns, g.turn.number);
}
