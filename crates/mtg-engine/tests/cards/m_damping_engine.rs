//! Damping Engine (hand-written, `src/cards/damping_engine.rs`).

use mtg_engine::decision::SpecialAction;
use mtg_engine::testing::*;
use mtg_engine::*;

fn specials(t: &mut TestGame, p: PlayerId) -> Vec<SpecialAction> {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .into_iter()
        .filter_map(|a| match a {
            Action::Special(s) => Some(s),
            _ => None,
        })
        .collect()
}

#[test]
fn the_player_with_the_most_permanents_cant_play_lands_or_cast_permanents() {
    cr!("305.2", "601.3");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Damping Engine");
    t.lands(P0, "Forest", 3);
    let bears = t.hand(P0, "Grizzly Bears");
    let land = t.hand(P0, "Forest");
    assert!(t.cast(P0, bears).try_go().is_err());
    assert!(t.play_land(P0, land).is_err());
    // Instants aren't affected.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn sacrificing_a_permanent_ignores_it_until_end_of_turn() {
    cr!("116.2d");
    ruling!("Damping Engine", "This is a special action that doesn’t use the stack");
    let mut t = TestGame::new(2);
    let engine = t.battlefield(P1, "Damping Engine");
    t.lands(P0, "Forest", 4);
    let bears = t.hand(P0, "Grizzly Bears");
    // The other player isn't affected and can't take the action.
    assert!(specials(&mut t, P1).is_empty());
    let sa = specials(&mut t, P0);
    assert_eq!(
        sa,
        vec![SpecialAction::Other {
            name: "card:Damping Engine:sacrifice a permanent to ignore it".into(),
            obj: Some(engine),
        }]
    );
    t.g.take_action(P0, Action::Special(sa[0].clone()));
    assert_eq!(t.graveyard_size(P0), 1);
    // Still the most permanents, but ignoring the effect.
    t.cast(P0, bears).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Only once each turn.
    assert!(specials(&mut t, P0).is_empty());
}
