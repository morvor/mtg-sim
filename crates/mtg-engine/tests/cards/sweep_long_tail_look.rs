//! Looking at another player's hand: "Look at target player's hand." (Gitaxian Probe,
//! Telepathic Spies) — the cards are shown only to the player looking, not revealed
//! (CR 701.20e) — and "Look at target
//! player's hand and choose X cards from it. That player discards those cards." (Mind
//! Warp): the caster chooses, the player discards (CR 701.9b).

use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn gitaxian_probe_looks_without_revealing_and_draws() {
    cr!("701.20e");
    assert_supported("Gitaxian Probe");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let probe = t.hand(P0, "Gitaxian Probe");
    let hand_before = t.hand_size(P0);
    t.cast(P0, probe).target(P1).go();
    t.resolve();
    // The probe left the hand, and P0 drew a card.
    assert_eq!(t.hand_size(P0), hand_before);
    assert!(t.in_graveyard(P0, "Gitaxian Probe"));
    // The card was looked at, not revealed: no reveal happened, and it's still in hand.
    assert!(t.g.reveals.revealed.is_empty());
    assert_eq!(t.zone(bolt), Zone::Hand(P1));
    assert!(t.dump_log().contains("looks at"), "{}", t.dump_log());
}

#[test]
fn telepathic_spies_targets_an_opponent() {
    cr!("701.20e", "115.1");
    assert_supported("Telepathic Spies");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let spies = t.hand(P0, "Telepathic Spies");
    t.cast(P0, spies).go();
    t.resolve();
    // The enters trigger asks for an opponent target only.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    let target_choices: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseTargets { candidates, .. } if p == P0 => Some(candidates),
            _ => None,
        })
        .collect();
    assert!(!target_choices.is_empty(), "{}", t.dump_log());
    assert!(target_choices
        .iter()
        .all(|c| c == &vec![Entity::Player(P1)]));
    assert!(t.dump_log().contains("looks at"), "{}", t.dump_log());
    // Looked at, not revealed.
    assert!(t.g.reveals.revealed.is_empty());
}

#[test]
fn gitaxian_probe_with_an_empty_hand_still_draws() {
    cr!("701.20e");
    ruling!(
        "Gitaxian Probe",
        "The targeted player may have no cards in their hand. You'll still draw a card."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let probe = t.hand(P0, "Gitaxian Probe");
    assert_eq!(t.hand_size(P1), 0);
    let lib = t.library_size(P0);
    t.cast(P0, probe).target(P1).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Gitaxian Probe"));
    assert_eq!(t.library_size(P0), lib - 1, "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn mind_warp_caster_chooses_x_cards_to_discard() {
    cr!("701.9b", "107.3");
    ruling!("Mind Warp", "You decide which cards, but they do the discarding.");
    assert_supported("Mind Warp");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 6);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bears = t.hand(P1, "Grizzly Bears");
    let forest = t.hand(P1, "Forest");
    let warp = t.hand(P0, "Mind Warp");
    t.answer_choose(P0, &[Entity::Object(bolt), Entity::Object(forest)]);
    t.cast(P0, warp).target(P1).x(2).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Lightning Bolt"), "{}", t.dump_log());
    assert!(t.in_graveyard(P1, "Forest"));
    assert_eq!(t.zone(bears), Zone::Hand(P1));
    // Looking isn't revealing.
    assert!(t.g.reveals.revealed.is_empty());
}
