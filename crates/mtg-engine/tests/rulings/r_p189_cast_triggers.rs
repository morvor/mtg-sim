//! Rulings batch P189 — "whenever you cast" triggers (and "becomes the target"
//! triggers) resolve before the spell that caused them, and resolve even if that spell is
//! countered (CR 603.3, 601.2i, 405.2).

use crate::r_p076_common::mana;
use crate::r_p189_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s21_common::legal_blocks;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` casts Opt (adding {U} first) and returns the spell.
fn cast_opt(t: &mut TestGame, p: PlayerId) -> ObjectId {
    mana(t, p, ManaType::U, 1);
    let opt = t.hand(p, "Opt");
    t.cast(p, opt).go()
}

/// P1 casts Counterspell targeting `spell` (on top of the stack) and it resolves.
fn counter_now(t: &mut TestGame, spell: ObjectId) {
    mana(t, P1, ManaType::U, 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(spell).go();
    t.resolve();
}

#[test]
fn incursion_specialist_triggers_once_and_resolves_first() {
    cr!("603.2", "603.3", "601.2i");
    ruling!(
        "Incursion Specialist",
        "Incursion Specialist’s ability can trigger only once each turn. The ability will resolve before the second spell resolves. It doesn’t matter if the first spell you cast that turn has resolved, was countered, or is still on the stack."
    );
    supported("Incursion Specialist");
    let mut t = TestGame::new(2);
    let spec = t.battlefield(P0, "Incursion Specialist");
    // The first spell is still on the stack.
    let first = cast_opt(&mut t, P0);
    t.settle();
    assert_eq!(triggers_of(&t, spec), 0);
    let second = cast_opt(&mut t, P0);
    t.settle();
    assert_eq!(stack_triggers_from(&t, spec).len(), 1);
    assert_eq!(*t.g.stack.last().unwrap(), stack_triggers_from(&t, spec)[0]);
    t.resolve();
    assert_eq!(t.pt(spec), (3, 3));
    assert!(t.g.stack.contains(&second) && t.g.stack.contains(&first));
    t.resolve_all();
    // A third spell doesn't trigger it.
    cast_opt(&mut t, P0);
    t.resolve_all();
    assert_eq!(triggers_of(&t, spec), 1);
}

#[test]
fn matterbending_mage_trigger_resolves_even_if_spell_countered() {
    cr!("603.3", "601.2i", "509.1b");
    ruling!(
        "Matterbending Mage",
        "Matterbending Mage's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    supported("Matterbending Mage");
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, "Matterbending Mage");
    let bears = t.battlefield(P1, "Grizzly Bears");
    mana(&mut t, P0, ManaType::R, 1);
    let banefire = t.hand(P0, "Banefire");
    let spell = t.cast(P0, banefire).target(Entity::Player(P1)).x(0).go();
    t.settle();
    assert_eq!(stack_triggers_from(&t, mage).len(), 1);
    assert_eq!(*t.g.stack.last().unwrap(), stack_triggers_from(&t, mage)[0]);
    // The spell is countered while the trigger is waiting.
    counter_now(&mut t, spell);
    assert!(t.in_graveyard(P0, "Banefire"));
    t.resolve_all();
    attack_with(&mut t, &[(mage, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(bears, mage)]));
}

#[test]
fn razzle_dazzler_counts_spells_cast_before_it_entered() {
    cr!("603.2");
    ruling!(
        "Razzle-Dazzler",
        "Spells that were cast before Razzle-Dazzler entered the battlefield count. If Razzle-Dazzler was the first spell you cast this turn, the next spell you cast this turn is your second spell."
    );
    supported("Razzle-Dazzler");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, ManaType::U, 2);
    let card = t.hand(P0, "Razzle-Dazzler");
    t.cast(P0, card).go();
    t.resolve_all();
    let rd = t.named_on_battlefield("Razzle-Dazzler")[0];
    cast_opt(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(rd, counters::PLUS1), 1);
    assert_eq!(triggers_of(&t, rd), 1);
}

#[test]
fn shay_cormac_bounty_trigger_resolves_even_if_spell_countered() {
    cr!("603.3", "603.2", "601.2c");
    ruling!(
        "Shay Cormac",
        "Shay Cormac’s second ability resolves before the spell or ability that caused it to trigger. It resolves even if that spell or ability is countered."
    );
    supported("Shay Cormac");
    let mut t = TestGame::new(2);
    let shay = t.battlefield(P0, "Shay Cormac");
    let bears = t.battlefield(P1, "Grizzly Bears");
    mana(&mut t, P0, ManaType::R, 1);
    let shock = t.hand(P0, "Shock");
    let spell = t.cast(P0, shock).target(bears).go();
    t.settle();
    assert_eq!(*t.g.stack.last().unwrap(), stack_triggers_from(&t, shay)[0]);
    counter_now(&mut t, spell);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.counters(bears, "bounty"), 1);
}

#[test]
fn arixmethes_trigger_resolves_before_the_spell_even_if_countered() {
    cr!("603.3", "601.2c", "601.2i");
    ruling!(
        "Arixmethes, Slumbering Isle",
        "Arixmethes's triggered ability resolves before the spell that caused it to trigger, but after targets have been chosen for that spell. It resolves even if that spell is countered."
    );
    supported("Arixmethes, Slumbering Isle");
    let mut t = TestGame::new(2);
    let arix = t.enter(P0, "Arixmethes, Slumbering Isle");
    t.settle();
    assert_eq!(t.counters(arix, "slumber"), 5);
    let bears = t.battlefield(P1, "Grizzly Bears");
    mana(&mut t, P0, ManaType::R, 1);
    let shock = t.hand(P0, "Shock");
    t.answer_yes(P0, true);
    let spell = t.cast(P0, shock).target(bears).go();
    t.settle();
    // The spell's target was chosen before the trigger went on the stack above it.
    let si = t.obj(spell).stack.clone().unwrap();
    assert_eq!(si.chosen[0].targets[0], vec![Entity::Object(bears)]);
    assert_eq!(*t.g.stack.last().unwrap(), stack_triggers_from(&t, arix)[0]);
    counter_now(&mut t, spell);
    t.resolve_all();
    assert_eq!(t.counters(arix, "slumber"), 4);
    assert!(t.on_battlefield(bears));
}
