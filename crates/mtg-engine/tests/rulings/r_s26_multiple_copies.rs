//! Rulings batch S26 — effects that copy a spell more than once: each copy may get its
//! own new targets, and a target that can't be changed to a legal one stays (CR 707.10c,
//! 115.7d).

use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s26_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn storm_kings_thunder_copies_can_each_get_different_targets() {
    cr!("707.10c", "115.7d", "603.7b");
    ruling!(
        "Storm King's Thunder",
        "If there are multiple copies, you may change the targets of each of them to different legal targets."
    );
    supported("Storm King's Thunder");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Mountain", 6);
    // X = 2: the next instant or sorcery spell is copied twice.
    let thunder = t.hand(P0, "Storm King's Thunder");
    t.cast(P0, thunder).x(2).go();
    t.resolve_all();
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    // The first copy gets the Bears, the second the Elves.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(elves)]);
    t.settle();
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 2);
    let mut targets: Vec<Entity> = copies
        .iter()
        .flat_map(|c| targets_on_stack(&t, *c))
        .collect();
    targets.sort_by_key(|e| format!("{e:?}"));
    let mut want = vec![Entity::Object(bears), Entity::Object(elves)];
    want.sort_by_key(|e| format!("{e:?}"));
    assert_eq!(targets, want);
    t.resolve_all();
    assert!(!t.g.is_live(bears) && !t.g.is_live(elves));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn storm_kings_thunder_copies_keep_targets_that_cant_be_changed() {
    cr!("707.10c", "115.7d");
    ruling!(
        "Storm King's Thunder",
        "Each of the copies will have the same targets as the spell it's copying unless you choose new ones. You may change any number of the targets, including all of them or none of them. If, for one of the targets, you can't choose a new legal target, then it remains unchanged (even if the current target is illegal)."
    );
    supported("Storm King's Thunder");
    supported("Shatter");
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P1, "Sol Ring");
    t.lands(P0, "Mountain", 7);
    let thunder = t.hand(P0, "Storm King's Thunder");
    t.cast(P0, thunder).x(2).go();
    t.resolve_all();
    let shatter = t.hand(P0, "Shatter");
    let shatter = t.cast(P0, shatter).target(ring).go();
    t.settle();
    // Sol Ring is destroyed before the copies are made: there's no other artifact to
    // choose, so each copy keeps the (now illegal) Sol Ring as its target.
    destroy(&mut t, ring);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 2);
    for c in copies {
        assert_eq!(targets_on_stack(&t, c), vec![Entity::Object(ring)]);
    }
    assert_eq!(targets_on_stack(&t, shatter), vec![Entity::Object(ring)]);
    t.resolve_all();
    assert!(t.g.stack.is_empty());
}

/// The Elemental tokens Young Pyromancer made for `p` (one per instant or sorcery spell
/// cast).
fn elementals(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.name == "Elemental Token")
        .count()
}

#[test]
fn complete_the_circuit_copies_arent_cast() {
    cr!("707.10", "601.2i", "601.3b");
    ruling!(
        "Complete the Circuit",
        "A copy of a spell is created on the stack, so it's not \"cast.\" Abilities that trigger when a player casts a spell won't trigger."
    );
    supported("Complete the Circuit");
    supported("Young Pyromancer");
    supported("Sign in Blood");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Pyromancer");
    t.lands(P0, "Underground Sea", 8);
    // Cast during P0's upkeep (it's an instant). Before it resolves, a sorcery can't be
    // cast then.
    t.set_step(P0, Step::Upkeep);
    let early = t.hand(P0, "Sign in Blood");
    assert!(t.cast(P0, early).target(P1).try_go().is_err());
    t.clear_answers();
    let circuit = t.hand(P0, "Complete the Circuit");
    t.cast(P0, circuit).go();
    t.resolve_all();
    assert_eq!(elementals(&t, P0), 1);
    // Sign in Blood is a sorcery, cast in the upkeep as though it had flash; the two
    // copies aren't cast.
    let sign = t.hand(P0, "Sign in Blood");
    let hand = t.hand_size(P0) - 1;
    t.cast(P0, sign).target(P1).go();
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 6);
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(elementals(&t, P0), 2);
}

#[test]
fn complete_the_circuit_copies_have_the_same_mode() {
    cr!("707.10", "700.2g");
    ruling!(
        "Complete the Circuit",
        "If the spell that's copied is modal (that is, it includes a choice from a bulleted list of effects), the copy will have the same mode. A different mode can't be chosen."
    );
    supported("Complete the Circuit");
    supported("Boros Charm");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plateau", 2);
    t.lands(P0, "Island", 6);
    let circuit = t.hand(P0, "Complete the Circuit");
    t.cast(P0, circuit).go();
    t.resolve_all();
    // Boros Charm's first mode (4 damage to target player): both copies deal 4 too.
    let charm = t.hand(P0, "Boros Charm");
    let charm = t.cast(P0, charm).modes(&[0]).target(P1).go();
    t.answer(
        P0,
        DecisionKind::Modes,
        mtg_engine::decision::Answer::Indices(vec![2]),
    );
    t.answer(
        P0,
        DecisionKind::Modes,
        mtg_engine::decision::Answer::Indices(vec![2]),
    );
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    t.settle();
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 2);
    for c in &copies {
        assert_eq!(modes_on_stack(&t, *c), modes_on_stack(&t, charm));
    }
    t.resolve_all();
    assert_eq!(t.life(P1), 8);
    assert!(t.on_battlefield(bears));
}

#[test]
fn thousand_year_storm_copies_have_the_effects_of_the_kicker_paid() {
    cr!("707.10", "707.2", "702.33d");
    ruling!(
        "Thousand-Year Storm",
        "You can't choose to pay any additional costs for the copies. However, effects based on any additional costs that were paid for the original spell are copied as though those same costs were paid for the copy too."
    );
    supported("Thousand-Year Storm");
    supported("Burst Lightning");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thousand-Year Storm");
    t.lands(P0, "Volcanic Island", 7);
    // The first instant this turn: no copies.
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    t.resolve_all();
    assert!(spell_copies(&t).is_empty());
    // The second, kicked: one copy, kicked too (4 damage each).
    let burst = t.hand(P0, "Burst Lightning");
    t.cast(P0, burst).kicked(true).target(P1).go();
    t.answer_yes(P0, false);
    t.settle();
    t.resolve();
    assert_eq!(spell_copies(&t).len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 12);
    // Not kicked: the copies (two now) aren't kicked either.
    t.lands(P0, "Volcanic Island", 1);
    let burst = t.hand(P0, "Burst Lightning");
    t.cast(P0, burst).kicked(false).target(P1).go();
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    t.settle();
    t.resolve();
    assert_eq!(spell_copies(&t).len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 6);
}
