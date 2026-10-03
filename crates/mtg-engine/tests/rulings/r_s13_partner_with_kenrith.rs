//! Rulings batch S13 — "partner with": the rulings shared by Will Kenrith and Rowan
//! Kenrith. Will Kenrith's −8: "Target player gets an emblem with 'Whenever you cast an
//! instant or sorcery spell, copy it. You may choose new targets for the copy.'" A copy
//! copies the decisions made for the spell — modes, targets, X, division, additional
//! costs — and isn't cast (CR 707.10).

use crate::r_s01_common::*;
use crate::r_s13_common::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::deck::check_constructed;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::kw::partner::{can_be_commander, check_commander_deck};
use mtg_engine::object::{ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

/// `p` gets Will Kenrith's emblem through its −8 ability (the Will, at 8 loyalty, then
/// dies with none left).
fn will_emblem(t: &mut TestGame, p: PlayerId) {
    let will = t.battlefield(p, "Will Kenrith");
    t.g.objects[will.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 8);
    t.activate(p, will, 2, &[Entity::Player(p)])
        .expect("Will Kenrith's −8");
    t.resolve_all();
    assert!(t.in_graveyard(p, "Will Kenrith"));
}

/// Decisions of a kind asked since decision `from`.
fn count_asked(t: &TestGame, from: usize, pred: fn(&Decision) -> bool) -> usize {
    t.asked()[from..].iter().filter(|(_, d)| pred(d)).count()
}

fn damage(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).damage
}

#[test]
fn the_copy_keeps_the_division_of_damage_but_may_change_targets() {
    cr!("707.10", "707.10c", "601.2d");
    ruling!(
        "Will Kenrith",
        "If the spell or ability has damage divided as it was cast or activated (like Chandra's Pyrohelix), the division can't be changed (although the targets receiving that damage still can)."
    );
    supported("Will Kenrith");
    supported("Chandra's Pyrohelix");
    let mut t = TestGame::new(2);
    will_emblem(&mut t, P0);
    // Chandra's Pyrohelix: "deals 2 damage divided as you choose among one or two
    // targets": 1 to each of two creatures.
    let giants: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P1, "Hill Giant")).collect();
    let helix = t.hand(P0, "Chandra's Pyrohelix");
    give_mana_for(&mut t, P0, "Chandra's Pyrohelix");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    let from = t.asked().len();
    t.cast(P0, helix)
        .targets(&[Entity::Object(giants[0]), Entity::Object(giants[1])])
        .go();
    t.settle();
    // The copy's targets become the other two creatures.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(giants[2])]);
    t.answer_targets(P0, &[Entity::Object(giants[3])]);
    t.resolve_all();
    for g in &giants {
        assert_eq!(damage(&t, *g), 1);
    }
    // The damage was divided only once, for the original.
    assert_eq!(
        count_asked(&t, from, |d| matches!(d, Decision::Divide { .. })),
        1
    );
}

#[test]
fn two_emblems_copy_each_spell_twice() {
    cr!("707.10", "114.2", "114.4");
    ruling!(
        "Will Kenrith",
        "If you have two of Will's emblems, perhaps because Rowan's emblem copied Will's last ability, each one will copy a spell you cast."
    );
    let mut t = TestGame::new(2);
    will_emblem(&mut t, P0);
    will_emblem(&mut t, P0);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, bolt).target(P1).go();
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 3 * 3);
}

#[test]
fn the_copy_has_the_same_mode() {
    cr!("707.10", "700.2g");
    ruling!(
        "Will Kenrith",
        "If the spell or ability that's copied is modal (that is, it says \"Choose one —\" or the like), the copy will have the same mode. A different mode can't be chosen."
    );
    supported("Izzet Charm");
    let mut t = TestGame::new(2);
    will_emblem(&mut t, P0);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    // Izzet Charm, second mode: "Izzet Charm deals 2 damage to target creature."
    let charm = t.hand(P0, "Izzet Charm");
    give_mana_for(&mut t, P0, "Izzet Charm");
    let from = t.asked().len();
    t.cast(P0, charm).modes(&[1]).target(a).go();
    t.settle();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(
        count_asked(&t, from, |d| matches!(d, Decision::ChooseModes { .. })),
        1
    );
}

#[test]
fn the_copy_has_the_same_value_of_x() {
    cr!("707.10", "107.3");
    ruling!(
        "Will Kenrith",
        "If the spell or ability that's copied has an X whose value was determined as it was cast or activated (like Blaze does), the copy will have the same value of X."
    );
    supported("Blaze");
    let mut t = TestGame::new(2);
    will_emblem(&mut t, P0);
    // Blaze: "Blaze deals X damage to any target." X = 4.
    let blaze = t.hand(P0, "Blaze");
    t.lands(P0, "Mountain", 5);
    let from = t.asked().len();
    t.cast(P0, blaze).x(4).target(P1).go();
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 4 - 4);
    assert_eq!(
        count_asked(&t, from, |d| matches!(d, Decision::ChooseX { .. })),
        1
    );
}

#[test]
fn can_be_your_commander_matters_only_in_commander() {
    cr!("903.3a", "702.124j");
    ruling!(
        "Will Kenrith",
        "The last abilities of Will and Rowan apply to Commander games only. They have no effect in other games."
    );
    // "Will Kenrith can be your commander." A planeswalker without that can't be.
    assert!(can_be_commander(&card("Will Kenrith"), false));
    assert!(!can_be_commander(&card("Jace Beleren"), false));
    let pair: Vec<Arc<CardDef>> = vec![card("Will Kenrith"), card("Rowan Kenrith")];
    let mut d = pair.clone();
    d.extend((0..98).map(|_| card("Wastes")));
    assert!(check_commander_deck(&d, &pair, &[], false).is_empty());
    // Elsewhere it's just a planeswalker card: any number of copies up to four, with no
    // commander.
    let mut d: Vec<Arc<CardDef>> = (0..4).map(|_| card("Will Kenrith")).collect();
    d.extend((0..56).map(|_| card("Wastes")));
    assert!(check_constructed(&d).is_empty());
    // And in a two-player game it's in the library like any card, not the command zone.
    let lib: Vec<Arc<CardDef>> = std::iter::once(card("Will Kenrith"))
        .chain((0..39).map(|_| card("Wastes")))
        .collect();
    let other: Vec<Arc<CardDef>> = (0..40).map(|_| card("Wastes")).collect();
    let mut t = pregame(
        mtg_engine::game::GameConfig {
            skip_mulligans: true,
            ..Default::default()
        },
        vec![lib, other],
    );
    t.g.start();
    assert!(t.g.find_in_zone(Zone::Command, "Will Kenrith").is_empty());
}

#[test]
fn the_copy_isnt_cast_so_cast_triggers_dont_see_it() {
    cr!("707.10", "603.2");
    ruling!(
        "Will Kenrith",
        "The copy is created on the stack, so it's not \"cast\" or \"activated.\" Abilities that trigger when a player casts a spell or activates an ability (such as either emblem's own ability) won't trigger."
    );
    supported("Young Pyromancer");
    // Young Pyromancer: "Whenever you cast an instant or sorcery spell, create a 1/1 red
    // Elemental creature token."
    let mut t = TestGame::new(2);
    will_emblem(&mut t, P0);
    t.battlefield(P0, "Young Pyromancer");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, bolt).target(P1).go();
    t.settle();
    t.resolve_all();
    // One copy (the emblem didn't trigger for its own copy), one Elemental.
    assert_eq!(t.life(P1), 14);
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn a_copy_of_a_kicked_spell_is_kicked() {
    cr!("707.10", "702.33d");
    ruling!(
        "Will Kenrith",
        "The controller of a copied spell can't choose to pay any alternative or additional costs for the copy. However, effects based on any alternative or additional costs that were paid for the original spell are copied as though those same costs were paid for the copy."
    );
    supported("Burst Lightning");
    // Burst Lightning, kicked: 4 damage, and so is its copy.
    let mut t = TestGame::new(2);
    will_emblem(&mut t, P0);
    let burst = t.hand(P0, "Burst Lightning");
    t.lands(P0, "Mountain", 5);
    let from = t.asked().len();
    t.cast(P0, burst).kicked(true).target(P1).go();
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 4 - 4);
    assert_eq!(
        count_asked(&t, from, |d| matches!(d, Decision::OptionalCost { .. })),
        1
    );
    // Not kicked: 2 each.
    let mut t = TestGame::new(2);
    will_emblem(&mut t, P0);
    let burst = t.hand(P0, "Burst Lightning");
    t.lands(P0, "Mountain", 5);
    t.cast(P0, burst).kicked(false).target(P1).go();
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn the_emblem_copies_a_spell_countered_before_its_ability_resolves() {
    cr!("707.10", "608.2h", "701.6a");
    ruling!(
        "Will Kenrith",
        "The ability of either Kenrith's emblem can copy the spell or ability even if that spell or ability is countered before the emblem's triggered ability resolves."
    );
    let mut t = TestGame::new(2);
    will_emblem(&mut t, P0);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, bolt).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // The opponent counters the Bolt in response to the emblem's trigger.
    let counter = t.hand(P1, "Counterspell");
    t.lands(P1, "Island", 2);
    t.cast(P1, counter).target(bolt).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    t.resolve_all();
    // The copy was made anyway (as the Bolt last existed on the stack) and dealt 3.
    assert_eq!(t.life(P1), 17);
}

#[test]
fn the_copy_resolves_before_the_original() {
    cr!("707.10", "405.5", "608.1");
    ruling!(
        "Will Kenrith",
        "The copy of the spell created by Will's emblem resolves before the original spell."
    );
    let mut t = TestGame::new(2);
    will_emblem(&mut t, P0);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    let spell = t.cast(P0, bolt).target(P1).go();
    t.settle();
    // The emblem's trigger resolves: the copy is put on the stack on top of the Bolt.
    t.resolve();
    assert_eq!(t.stack_len(), 2);
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.obj(top).kind, ObjKind::SpellCopy);
    // It resolves first: 3 damage, with the original Bolt still on the stack.
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.g.stack, vec![spell]);
    t.resolve();
    assert_eq!(t.life(P1), 14);
}
