//! Rulings batch S06 — endure (CR 701.63) and encore (CR 702.141).

use crate::r_s01_common::*;
use crate::r_s04_common::*;
use crate::r_s06_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Activates Krumar Initiate's "{X}{B}, {T}, Pay X life: This creature endures X" with
/// X = 2, choosing the option `choice` of the endure instruction, and resolves it.
fn krumar_endures_2(t: &mut TestGame, krumar: ObjectId, choice: usize) {
    add_mana(t, P0, ManaType::B, 1);
    add_mana(t, P0, ManaType::C, 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer(P0, DecisionKind::Option, Answer::Index(choice));
    activate_containing(t, P0, krumar, "endures").expect("activate");
    t.resolve();
}

#[test]
fn a_noncreature_permanent_can_endure_too() {
    cr!("701.63a");
    ruling!(
        "Krumar Initiate",
        "If a noncreature permanent is instructed to endure, the effect is the same. You can put +1/+1 counters on that permanent or create a Spirit token."
    );
    supported("Krumar Initiate");
    // One with the Stars makes Krumar Initiate a noncreature enchantment; it keeps its
    // ability.
    let mut t = TestGame::new(2);
    let krumar = t.battlefield(P0, "Krumar Initiate");
    attach_new(&mut t, P1, "One with the Stars", krumar);
    assert!(!is_creature(&t, krumar));
    let from = t.asked().len();
    krumar_endures_2(&mut t, krumar, 0);
    assert!(asked_since(&t, from)
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseOption { prompt, .. } if prompt == "Endure")));
    assert_eq!(t.counters(krumar, "+1/+1"), 2);
    assert!(!is_creature(&t, krumar));
    assert_eq!(t.life(P0), 18);
    assert!(tokens_of(&t, P0).is_empty());
    // Or the Spirit token.
    let mut t = TestGame::new(2);
    let krumar = t.battlefield(P0, "Krumar Initiate");
    attach_new(&mut t, P1, "One with the Stars", krumar);
    krumar_endures_2(&mut t, krumar, 1);
    assert_eq!(t.counters(krumar, "+1/+1"), 0);
    let spirits = tokens_of(&t, P0);
    assert_eq!(spirits.len(), 1);
    assert_eq!(t.pt(spirits[0]), (2, 2));
    assert!(t.g.obj(spirits[0]).chars.has_subtype("Spirit"));
}

/// Tokens `p` controls.
fn tokens_of(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token())
        .map(|o| o.id)
        .collect()
}

#[test]
fn exiling_the_encore_card_is_a_cost_paid_without_interruption() {
    cr!("702.141a", "602.2", "602.2b", "601.2h");
    ruling!(
        "Impulsive Pilferer",
        "Exiling the card with encore is a cost to activate the ability. Once you announce that you're activating it, no player may take actions until you've finished. They can't try to remove the card from your graveyard to stop you from paying the cost."
    );
    supported("Impulsive Pilferer");
    supported("Relic of Progenitus");
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Impulsive Pilferer");
    // P1 has a way to exile cards from graveyards, but gets no chance to use it while the
    // encore ability is activated.
    let relic = t.battlefield(P1, "Relic of Progenitus");
    add_mana(&mut t, P0, ManaType::R, 1);
    add_mana(&mut t, P0, ManaType::C, 3);
    let from = t.asked().len();
    activate_named(&mut t, P0, card, "Encore", 0).expect("encore");
    let asked = asked_since(&t, from);
    assert!(
        asked
            .iter()
            .all(|(p, d)| *p == P0 && !matches!(d, Decision::Priority { .. })),
        "decisions during the activation: {asked:?}"
    );
    // The card was exiled as the cost; the ability is on the stack.
    assert!(t.in_exile("Impulsive Pilferer"));
    assert!(!t.g.is_live(card));
    assert_eq!(on_stack(&t, "Encore"), 1);
    // Exiling all graveyards in response doesn't matter any more.
    t.lands(P1, "Wastes", 1);
    activate_containing(&mut t, P1, relic, "Exile all graveyards").expect("relic");
    t.resolve();
    t.resolve_all();
    let copies: Vec<ObjectId> = tokens_of(&t, P0)
        .into_iter()
        .filter(|id| t.g.obj(*id).chars.name == "Impulsive Pilferer")
        .collect();
    assert_eq!(copies.len(), 1);
    assert!(t.g.obj(copies[0]).is(CardType::Creature));
}
