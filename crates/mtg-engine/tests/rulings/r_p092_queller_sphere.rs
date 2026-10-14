//! Rulings batch P092 — Spell Queller ("the exiled card's owner may cast that card without
//! paying its mana cost" when it leaves) and Detention Sphere (exiles every permanent with
//! the target's name; returns them "under their owner's control").

use crate::r_s01_common::*;
use crate::r_s02_common::{destroy, target_candidates};
use crate::r_s05_common::move_to;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// P1 casts Blaze ("Blaze deals X damage to any target.") with X = `x` at P0.
/// It's P1's turn (Blaze is a sorcery).
fn p1_blaze(t: &mut TestGame, x: i64) -> ObjectId {
    supported("Blaze");
    t.advance_to(P1, mtg_engine::turn::Step::PrecombatMain);
    t.lands(P1, "Mountain", x as usize + 1);
    let c = t.hand(P1, "Blaze");
    t.cast(P1, c).x(x).target(Entity::Player(P0)).go()
}

#[test]
fn spell_queller_uses_the_chosen_x_for_mana_value() {
    cr!("107.3", "202.3e", "115.1");
    ruling!(
        "Spell Queller",
        "For spells with {X} in their mana costs, use the value chosen for X to determine if the spell’s mana value is 4 or less."
    );
    supported("Spell Queller");
    for (x, ok) in [(3, true), (4, false)] {
        let mut t = TestGame::new(2);
        let blaze = p1_blaze(&mut t, x);
        let from = t.asked().len();
        t.enter(P0, "Spell Queller");
        t.settle();
        let cands = target_candidates(&t, P0, from);
        assert_eq!(cands.iter().any(|c| c.contains(&obj(blaze))), ok, "X={x}");
    }
}

#[test]
fn spell_queller_exiles_a_spell_that_cant_be_countered() {
    cr!("101.1", "608.2b");
    ruling!(
        "Spell Queller",
        "Spells that can’t be countered can be exiled by Spell Queller’s ability. They won’t resolve."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Swamp", 1);
    t.lands(P1, "Forest", 1);
    let decay = t.hand(P1, "Abrupt Decay");
    let decay = t.cast(P1, decay).target(bears).go();
    t.answer_targets(P0, &[obj(decay)]);
    t.enter(P0, "Spell Queller");
    t.resolve_all();
    assert!(t.in_exile("Abrupt Decay"));
    assert!(t.on_battlefield(bears));
}

#[test]
fn spell_queller_that_leaves_first_exiles_the_spell_forever() {
    cr!("603.6c", "603.10a");
    ruling!(
        "Spell Queller",
        "If Spell Queller leaves the battlefield before its “enters” ability resolves, its leaves-the-battlefield triggered ability triggers, resolves, and does nothing. Then its first triggered ability resolves and exiles the spell forever."
    );
    let mut t = TestGame::new(2);
    let blaze = p1_blaze(&mut t, 2);
    t.answer_targets(P0, &[obj(blaze)]);
    let q = t.enter(P0, "Spell Queller");
    t.settle();
    assert_eq!(t.stack_len(), 2);
    destroy(&mut t, q);
    assert_eq!(t.stack_len(), 3);
    t.resolve_all();
    assert!(t.in_exile("Blaze"));
    assert_eq!(t.life(P0), 20);
    t.advance_to(P0, mtg_engine::turn::Step::PrecombatMain);
    assert!(t.in_exile("Blaze"));
}

#[test]
fn spell_queller_lets_the_owner_cast_it_anew_right_away_with_x_0() {
    cr!("608.2g", "118.9", "107.3b", "400.7", "307.1");
    ruling!(
        "Spell Queller",
        "If the card has {X} in its mana cost, the player must choose 0 as the value of X when casting it without paying its mana cost."
    );
    ruling!(
        "Spell Queller",
        "If the exiled card’s owner casts it, the spell has no relation to the spell that player originally cast. Any choices made for the original spell or effects affecting the original spell aren’t carried over to the new one."
    );
    ruling!(
        "Spell Queller",
        "If the player casts the exiled card, they do so as part of the resolution of Spell Queller’s last ability. The player can’t wait to cast it later in the turn. Timing permissions based on the card’s type are ignored"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blaze = p1_blaze(&mut t, 3);
    t.answer_targets(P0, &[obj(blaze)]);
    let q = t.enter(P0, "Spell Queller");
    t.resolve_all();
    assert!(t.in_exile("Blaze"));
    t.advance_to(P0, mtg_engine::turn::Step::PrecombatMain);
    // It's P0's turn; P1 casts the sorcery Blaze as the leaves ability resolves, choosing
    // a new target. X is 0.
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[obj(bears)]);
    t.answer(P1, DecisionKind::X, Answer::Number(3));
    destroy(&mut t, q);
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert!(!t.in_exile("Blaze"));
    let spells: Vec<ObjectId> = t.g.stack.to_vec();
    assert_eq!(spells.len(), 1, "Blaze was cast as the ability resolved");
    let spell = spells[0];
    assert_eq!(t.obj_now(spell).chars.name, "Blaze");
    assert_eq!(crate::r_s25_common::x_of(&t, spell), Some(0));
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(t.on_battlefield(bears));
    assert!(t.in_graveyard(P1, "Blaze"));
}

#[test]
fn spell_quellers_free_cast_can_pay_additional_costs() {
    cr!("118.9", "118.9a", "118.9d", "702.33a");
    ruling!(
        "Spell Queller",
        "The player can, however, pay additional costs, such as escalate costs. If the card has any mandatory additional costs, those must be paid to cast the card."
    );
    supported("Burst Lightning");
    // Burst Lightning (kicker {4}): kicked when cast again, 4 damage.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P1, "Mountain", 1);
    let c = t.hand(P1, "Burst Lightning");
    let spell = t.cast(P1, c).target(Entity::Player(P0)).go();
    t.answer_targets(P0, &[obj(spell)]);
    let q = t.enter(P0, "Spell Queller");
    t.resolve_all();
    t.lands(P1, "Wastes", 4);
    t.answer_yes(P1, true);
    t.answer(P1, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P1, &[obj(giant)]);
    destroy(&mut t, q);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(t.in_graveyard(P1, "Burst Lightning"));
    // Tormenting Voice ("As an additional cost to cast this spell, discard a card."): the
    // discard is still required.
    supported("Tormenting Voice");
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 2);
    t.hand(P1, "Forest");
    let c = t.hand(P1, "Forest");
    let voice = t.hand(P1, "Tormenting Voice");
    let _ = c;
    // P1 casts it at sorcery speed in P1's turn; P0 flashes in the Queller.
    t.advance_to(P1, mtg_engine::turn::Step::PrecombatMain);
    let spell = t.cast(P1, voice).go();
    assert_eq!(t.graveyard_size(P1), 1);
    t.answer_targets(P0, &[obj(spell)]);
    let q = t.enter(P0, "Spell Queller");
    t.resolve_all();
    let hand = t.hand_size(P1);
    t.answer_yes(P1, true);
    destroy(&mut t, q);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Tormenting Voice"));
    // Discarded the other Forest, drew two.
    assert_eq!(t.hand_size(P1), hand - 1 + 2);
}

/// A custom creature named `name` (nonland), with `text`, onto `p`'s battlefield.
fn named_creature(t: &mut TestGame, p: PlayerId, name: &str, text: &str) -> ObjectId {
    let def = custom_card(name, "Creature — Human", "{1}", Some((1, 1)), text);
    t.custom(p, def, Zone::Battlefield)
}

#[test]
fn detention_sphere_exiles_lands_and_protected_permanents_with_the_name() {
    cr!("115.1", "702.16b", "201.2");
    ruling!(
        "Detention Sphere",
        "Although the target of the \"enters\" ability must not be a land, lands with the same name as that permanent will be exiled."
    );
    ruling!(
        "Detention Sphere",
        "The \"enters\" ability has only one target. The other permanents with that name aren't targeted. For example, a permanent with protection from white will be exiled if it has the same name as the target nonland permanent."
    );
    supported("Detention Sphere");
    let mut t = TestGame::new(2);
    // A nonland "Forest" and two Forest lands.
    let fake = named_creature(&mut t, P1, "Forest", "");
    let lands = t.lands(P1, "Forest", 2);
    let target = named_creature(&mut t, P1, "Paladin", "");
    let warded = named_creature(&mut t, P1, "Paladin", "Protection from white");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(fake)]);
    t.answer_yes(P0, true);
    let sphere = t.enter(P0, "Detention Sphere");
    t.resolve_all();
    let cands = target_candidates(&t, P0, from);
    assert!(cands[0].iter().all(|e| !lands.iter().any(|l| *e == obj(*l))));
    assert!(!cands[0].contains(&obj(warded)));
    assert!(cands[0].contains(&obj(target)));
    assert_eq!(t.zone(fake), Zone::Exile);
    for l in &lands {
        assert_eq!(t.zone(*l), Zone::Exile);
    }
    // Another Sphere at a Paladin: the protected one is exiled too.
    t.answer_targets(P0, &[obj(target)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Detention Sphere");
    t.resolve_all();
    assert_eq!(t.zone(target), Zone::Exile);
    assert_eq!(t.zone(warded), Zone::Exile);
    // The first one leaves: its cards return under their owner's control.
    destroy(&mut t, sphere);
    t.resolve_all();
    assert!(t.on_battlefield(fake));
    assert_eq!(t.obj_now(fake).controller, P1);
    assert!(lands.iter().all(|l| t.on_battlefield(*l)));
    assert_eq!(t.zone(target), Zone::Exile);
}

#[test]
fn detention_sphere_that_leaves_first_exiles_indefinitely() {
    cr!("603.6c", "603.10a");
    ruling!(
        "Detention Sphere",
        "If Detention Sphere leaves the battlefield before its \"enters\" ability has resolved, its leaves-the-battlefield ability will trigger and do nothing. Then the \"enters\" ability will resolve and exile the targeted nonland permanent and other permanents with that name indefinitely."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(a)]);
    t.answer_yes(P0, true);
    let sphere = t.enter(P0, "Detention Sphere");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, sphere);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
}

#[test]
fn detention_sphere_with_an_illegal_target_exiles_nothing() {
    cr!("608.2b");
    ruling!(
        "Detention Sphere",
        "If the target nonland permanent is an illegal target when the \"enters\" ability tries to resolve, it won't resolve and none of its effects will happen. No permanents will be exiled, including those with the same name as the target."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(a)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Detention Sphere");
    t.settle();
    move_to(&mut t, a, Zone::Hand(P1));
    t.resolve_all();
    assert!(t.on_battlefield(b));
    assert!(!t.in_exile("Grizzly Bears"));
}
