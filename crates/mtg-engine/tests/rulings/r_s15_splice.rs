//! Rulings batch S15 — splice (CR 702.47): Strange Inversion, Evermind, Everdream, with
//! Glacial Ray and Unsummon.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::*;

/// The cards offered for splicing since decision `from`.
fn splice_offers(t: &TestGame, from: usize) -> Vec<ObjectId> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::OptionalCost { source, name, .. } if name == "splice" => Some(*source),
            _ => None,
        })
        .collect()
}

fn splice(t: &mut TestGame, p: PlayerId, yes: bool) {
    t.answer(p, DecisionKind::OptionalCost, Answer::Bool(yes));
}

#[test]
fn targets_for_spliced_text_are_chosen_after_the_reveal_and_may_differ() {
    cr!("601.2b", "702.47d", "601.2c");
    ruling!(
        "Strange Inversion",
        "You choose all targets for the spell after revealing cards you want to splice, including any targets required by the text of any of those cards. You may choose a different target for each instance of the word \"target\" on the resulting spell."
    );
    supported("Strange Inversion");
    supported("Glacial Ray");
    // Glacial Ray (Arcane): "Glacial Ray deals 2 damage to any target." Strange Inversion:
    // "Switch target creature's power and toughness until end of turn. Splice onto Arcane
    // {1}{R}". Glacial Ray targets P1; the spliced text targets Savannah Lions (2/1).
    let mut t = TestGame::new(2);
    let lions = t.battlefield(P0, "Savannah Lions");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 2);
    let ray = t.hand(P0, "Glacial Ray");
    let inversion = t.hand(P0, "Strange Inversion");
    splice(&mut t, P0, true);
    let from = t.asked().len();
    t.cast(P0, ray).target(P1).target(lions).go();
    let asked = t.asked();
    let reveal = asked[from..]
        .iter()
        .position(|(_, d)| matches!(d, Decision::OptionalCost { .. }))
        .expect("splice offered");
    let targets: Vec<usize> = asked[from..]
        .iter()
        .enumerate()
        .filter(|(_, (_, d))| matches!(d, Decision::ChooseTargets { .. }))
        .map(|(i, _)| i)
        .collect();
    // Two target choices, both after the reveal.
    assert_eq!(targets.len(), 2);
    assert!(targets.iter().all(|i| *i > reveal));
    assert_eq!(splice_offers(&t, from), vec![inversion]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.pt(lions), (1, 2));
    assert!(t.in_hand(P0, "Strange Inversion"));
}

#[test]
fn a_spliced_spell_whose_targets_are_all_illegal_does_nothing() {
    cr!("702.47d", "608.2b");
    ruling!(
        "Evermind",
        "If all of the spell's targets are illegal when the spell tries to resolve, it won't resolve and none of its effects will happen."
    );
    ruling!(
        "Everdream",
        "If all of the spell’s targets are illegal when the spell tries to resolve, it won’t resolve and none of its effects will happen, including those from cards spliced onto it."
    );
    supported("Evermind");
    supported("Everdream");
    supported("Unsummon");
    // Evermind ("Draw a card. Splice onto Arcane {1}{U}") spliced onto Glacial Ray, whose
    // only target is gone: no card is drawn.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    let ray = t.hand(P0, "Glacial Ray");
    t.hand(P0, "Evermind");
    splice(&mut t, P0, true);
    t.cast(P0, ray).target(bears).go();
    assert!(t.g.permanents().all(|o| !o.chars.is_land() || o.tapped));
    let hand = t.hand_size(P0);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // Everdream ("Draw a card. Splice onto instant or sorcery {2}{U}") spliced onto
    // Unsummon, whose target is gone: no card is drawn either.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 2);
    let unsummon = t.hand(P0, "Unsummon");
    t.hand(P0, "Everdream");
    splice(&mut t, P0, true);
    t.cast(P0, unsummon).target(bears).go();
    assert!(t.g.permanents().all(|o| !o.chars.is_land() || o.tapped));
    let hand = t.hand_size(P0);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.in_hand(P0, "Everdream"));
    // With the target still there, the spliced card's draw happens.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 2);
    let unsummon = t.hand(P0, "Unsummon");
    t.hand(P0, "Everdream");
    splice(&mut t, P0, true);
    t.cast(P0, unsummon).target(bears).go();
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn a_card_cant_be_spliced_onto_itself() {
    cr!("702.47a");
    ruling!(
        "Strange Inversion",
        "A card with a splice ability can't be spliced onto itself because the spell is on the stack (and not in your hand) when you reveal the cards you want to splice onto it."
    );
    supported("Strange Inversion");
    // Strange Inversion is an Arcane spell: cast alone, nothing is offered for splicing.
    let mut t = TestGame::new(2);
    let lions = t.battlefield(P0, "Savannah Lions");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 2);
    let first = t.hand(P0, "Strange Inversion");
    let from = t.asked().len();
    t.cast(P0, first).target(lions).go();
    assert!(splice_offers(&t, from).is_empty());
    t.resolve_all();
    assert_eq!(t.pt(lions), (1, 2));
    // Another Strange Inversion in hand can be spliced onto it.
    let mut t = TestGame::new(2);
    let lions = t.battlefield(P0, "Savannah Lions");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 3);
    let first = t.hand(P0, "Strange Inversion");
    let second = t.hand(P0, "Strange Inversion");
    splice(&mut t, P0, true);
    let from = t.asked().len();
    t.cast(P0, first).target(lions).target(bears).go();
    assert_eq!(splice_offers(&t, from), vec![second]);
}
