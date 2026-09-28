//! Rulings batch S29 — retrace (CR 702.81a): a spell cast with retrace goes back to its
//! owner's graveyard when it resolves or is countered (CR 608.2n, 701.6a); and copies of
//! spells whose damage or counters were divided as they were cast keep the division
//! while their targets may change (CR 707.10, 707.10c, 601.2d).

use crate::r_s01_common::supported;
use crate::r_s04_common::graveyard_names;
use crate::r_s25_common::{cast_new, change_copy_targets, spell_copies};
use crate::r_s29_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const RETRACE: CastMethod = CastMethod::Keyword(KeywordKind::Retrace);

/// `p` gets Wrenn and Six's emblem ("Instant and sorcery cards in your graveyard have
/// retrace.") through its −7 ability.
fn wrenn_emblem(t: &mut TestGame, p: PlayerId) {
    let wrenn = t.battlefield(p, "Wrenn and Six");
    t.g.objects[wrenn.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 7);
    t.activate(p, wrenn, 2, &[]).expect("Wrenn and Six's −7");
    t.resolve_all();
}

#[test]
fn a_spell_cast_with_retrace_returns_to_the_graveyard_when_it_resolves_or_is_countered() {
    cr!("702.81a", "608.2n", "701.6a");
    ruling!(
        "Wrenn and Six",
        "When a spell you cast with retrace resolves or is countered, it's put back into your graveyard. You may use the retrace ability to cast it again."
    );
    supported("Wrenn and Six");
    let mut t = TestGame::new(2);
    wrenn_emblem(&mut t, P0);
    t.lands(P0, "Mountain", 1);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let f1 = t.hand(P0, "Forest");
    let f2 = t.hand(P0, "Forest");
    // Cast with retrace (discarding a Forest): it resolves, then it's back in the
    // graveyard.
    t.answer_choose(P0, &[Entity::Object(f1)]);
    t.cast(P0, bolt).method(RETRACE).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0));
    // Cast with retrace again (discarding the other Forest) and countered: back in the
    // graveyard again.
    t.lands(P0, "Mountain", 1);
    let bolt = t.g.current(bolt);
    t.answer_choose(P0, &[Entity::Object(f2)]);
    let spell = t.cast(P0, bolt).method(RETRACE).target(P1).go();
    cast_new(&mut t, P1, "Counterspell", &[Entity::Object(spell)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0));
    assert!(graveyard_names(&t, P0).contains(&"Lightning Bolt".to_string()));
}

#[test]
fn a_countered_retrace_spell_returns_to_the_graveyard() {
    cr!("702.81a", "701.6a");
    ruling!(
        "Deeproot Historian",
        "If a spell you cast with retrace is countered, it's put back into your graveyard. You may use the retrace ability to cast it again."
    );
    supported("Deeproot Historian");
    // "Merfolk and Druid cards in your graveyard have retrace." Coral Merfolk (a Merfolk
    // creature card) is cast with retrace and countered.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Deeproot Historian");
    t.lands(P0, "Island", 2);
    let merfolk = t.graveyard(P0, "Coral Merfolk");
    let forests = [t.hand(P0, "Forest"), t.hand(P0, "Forest")];
    t.answer_choose(P0, &[Entity::Object(forests[0])]);
    let spell = t.cast(P0, merfolk).method(RETRACE).go();
    cast_new(&mut t, P1, "Counterspell", &[Entity::Object(spell)]);
    t.resolve_all();
    assert_eq!(t.zone(merfolk), Zone::Graveyard(P0));
    // It can be cast with retrace again, and this time it resolves.
    t.lands(P0, "Island", 2);
    t.answer_choose(P0, &[Entity::Object(forests[1])]);
    t.cast(P0, t.g.current(merfolk)).method(RETRACE).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Coral Merfolk").len(), 1);
}

/// Four Hill Giants (3/3) for P1.
fn giants(t: &mut TestGame) -> Vec<ObjectId> {
    (0..4).map(|_| t.battlefield(P1, "Hill Giant")).collect()
}

/// P0 casts Arc Lightning ("deals 3 damage divided as you choose among one, two, or three
/// targets") dividing 2 and 1 between the first two giants.
fn arc_lightning(t: &mut TestGame, g: &[ObjectId]) -> ObjectId {
    supported("Arc Lightning");
    cast_divided(t, "Arc Lightning", &[g[0], g[1]])
}

/// P0 casts the real spell `name` with its one target slot holding `targets`, dividing 2
/// and 1 among them.
fn cast_divided(t: &mut TestGame, name: &str, targets: &[ObjectId]) -> ObjectId {
    crate::r_s25_common::lands_for_cost(t, P0, name);
    divide(t, P0, &[2, 1]);
    let card = t.hand(P0, name);
    let targets: Vec<Entity> = targets.iter().map(|x| Entity::Object(*x)).collect();
    t.cast(P0, card).targets(&targets).go()
}

/// P0 casts Lutri, the Spellchaser, whose trigger copies `spell`, changing the copy's
/// targets to `new`; everything resolves.
fn lutri_copies(t: &mut TestGame, spell: ObjectId, new: &[ObjectId]) {
    cast_new(t, P0, "Lutri, the Spellchaser", &[]);
    t.answer_targets(P0, &[Entity::Object(spell)]);
    let new: Vec<Option<Entity>> = new.iter().map(|x| Some(Entity::Object(*x))).collect();
    change_copy_targets(t, P0, &new);
    // Lutri resolves; its trigger resolves, creating the copy.
    t.resolve();
    t.resolve();
    assert_eq!(spell_copies(t).len(), 1);
    t.resolve_all();
}

#[test]
fn lutris_copy_keeps_the_division_but_may_change_targets() {
    cr!("707.10", "707.10c", "601.2d");
    ruling!(
        "Lutri, the Spellchaser",
        "If the spell has damage divided as it was cast (like Mythos of Vadrok), the division can't be changed, although the targets receiving that damage still can. The same is true of spells that distribute counters."
    );
    supported("Lutri, the Spellchaser");
    // "When Lutri enters, if you cast it, copy target instant or sorcery spell you
    // control. You may choose new targets for the copy."
    let mut t = TestGame::new(2);
    let g = giants(&mut t);
    let arc = arc_lightning(&mut t, &g);
    lutri_copies(&mut t, arc, &[g[2], g[3]]);
    let dmg: Vec<u32> = g.iter().map(|x| damage_marked(&t, *x)).collect();
    assert_eq!(dmg, vec![2, 1, 2, 1]);
    // Counters: Defend the Celestus ("Distribute three +1/+1 counters among one, two, or
    // three target creatures you control."), 2 and 1, copied onto two other creatures.
    supported("Defend the Celestus");
    let mut t = TestGame::new(2);
    let bears: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let defend = cast_divided(&mut t, "Defend the Celestus", &[bears[0], bears[1]]);
    lutri_copies(&mut t, defend, &[bears[2], bears[3]]);
    let n: Vec<u32> = bears
        .iter()
        .map(|b| t.counters(*b, counters::PLUS1))
        .collect();
    assert_eq!(n, vec![2, 1, 2, 1]);
}

#[test]
fn mercurial_spelldancers_copy_keeps_the_division_but_may_change_targets() {
    cr!("707.10", "707.10c", "603.7a");
    ruling!(
        "Mercurial Spelldancer",
        "If the spell has damage divided as it was put onto the stack, the division can't be changed, although the targets receiving that damage still can. The same is true of spells that distribute counters."
    );
    supported("Mercurial Spelldancer");
    // "Whenever this creature deals combat damage to a player, you may remove two oil
    // counters from it. If you do, when you next cast an instant or sorcery spell this
    // turn, copy that spell. You may choose new targets for the copy."
    let mut t = TestGame::new(2);
    let g = giants(&mut t);
    let dancer = t.battlefield(P0, "Mercurial Spelldancer");
    put_counters(&mut t, dancer, "oil", 2);
    t.answer_yes(P0, true);
    t.attack(&[(dancer, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.counters(dancer, "oil"), 0);
    t.advance_to(P0, Step::PostcombatMain);
    arc_lightning(&mut t, &g);
    t.settle();
    change_copy_targets(
        &mut t,
        P0,
        &[Some(Entity::Object(g[2])), Some(Entity::Object(g[3]))],
    );
    t.resolve_all();
    let dmg: Vec<u32> = g.iter().map(|x| damage_marked(&t, *x)).collect();
    assert_eq!(dmg, vec![2, 1, 2, 1]);
}
