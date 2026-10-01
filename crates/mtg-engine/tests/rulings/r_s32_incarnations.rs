//! Rulings batch S32 — the Incarnations' "When Dread is put into a graveyard from anywhere,
//! shuffle it into its owner's library.": a triggered ability that functions from the
//! graveyard (CR 113.6k, 603.6c), so it triggers from any zone, doesn't look back in time
//! (CR 603.10a), and does nothing if the card has left the graveyard (CR 400.7).

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s06_common::attach_new;
use crate::r_s25_common::cast_new;
use crate::r_s32_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const DREAD: &str = "Dread";

#[test]
fn it_triggers_when_put_into_a_graveyard_from_any_zone() {
    cr!("113.6k", "603.6c", "701.9a");
    ruling!(
        "Dread",
        "The last ability triggers when the Incarnation is put into its owner’s graveyard from any zone, not just from on the battlefield."
    );
    supported(DREAD);
    // Discarded from P0's hand by P1's Mind Rot.
    let mut t = TestGame::new(2);
    t.hand(P0, DREAD);
    t.hand(P0, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Mind Rot", &[Entity::Player(P0)]);
    t.resolve_all();
    assert!(!t.in_graveyard(P0, DREAD));
    assert!(in_library(&t, P0, DREAD), "discarded: shuffled in");
    // Milled from P0's library by Thought Scour.
    let mut t = TestGame::new(2);
    t.library_top(P0, DREAD);
    cast_new(&mut t, P0, "Thought Scour", &[Entity::Player(P0)]);
    t.resolve_all();
    assert!(!t.in_graveyard(P0, DREAD));
    assert!(in_library(&t, P0, DREAD), "milled: shuffled in");
    // Destroyed on the battlefield.
    let mut t = TestGame::new(2);
    let dread = t.battlefield(P0, DREAD);
    destroy(&mut t, dread);
    t.resolve_all();
    assert!(in_library(&t, P0, DREAD), "destroyed: shuffled in");
}

#[test]
fn it_triggers_from_the_graveyard_with_the_abilities_the_card_has_there() {
    cr!("603.10a", "113.6k", "613.1f");
    ruling!(
        "Dread",
        "If the Incarnation had lost this ability while on the battlefield (due to Lignify, for example) and then was destroyed, the ability would still trigger"
    );
    ruling!(
        "Dread",
        "Although this ability triggers when the Incarnation is put into a graveyard from the battlefield, it doesn’t *specifically* trigger on leaving the battlefield, so it doesn’t behave like other leaves-the-battlefield abilities. The ability will trigger from the graveyard."
    );
    supported("Lignify");
    supported("Yixlid Jailer");
    // Lignify ("Enchanted creature is a 0/4 Treefolk with no abilities"): on the
    // battlefield Dread has no abilities, but the card in the graveyard does.
    let mut t = TestGame::new(2);
    let dread = t.battlefield(P0, DREAD);
    attach_new(&mut t, P0, "Lignify", dread);
    assert!(t.obj(dread).chars.abilities.is_empty());
    destroy(&mut t, dread);
    t.resolve_all();
    assert!(in_library(&t, P0, DREAD), "triggered from the graveyard");
    // Yixlid Jailer ("Cards in graveyards lose all abilities."): the card in the graveyard
    // has no such ability, so it doesn't trigger, and Dread stays there.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Yixlid Jailer");
    let dread = t.battlefield(P0, DREAD);
    destroy(&mut t, dread);
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert!(t.in_graveyard(P0, DREAD));
    assert!(!in_library(&t, P0, DREAD));
}

#[test]
fn removed_from_the_graveyard_or_never_put_there_it_isnt_shuffled_in() {
    cr!("400.7", "614.6");
    ruling!(
        "Dread",
        "If the Incarnation is removed from the graveyard after the ability triggers but before it resolves, it will remain in its new zone when its owner shuffles their library."
    );
    supported("Rest in Peace");
    // Exiled from the graveyard with the trigger on the stack.
    let mut t = TestGame::new(2);
    let dread = t.battlefield(P0, DREAD);
    destroy(&mut t, dread);
    assert_eq!(t.stack_len(), 1);
    let card = t.g.current(dread);
    cast_new(&mut t, P1, "Cremate", &[Entity::Object(card)]);
    t.resolve_all();
    assert!(t.in_exile(DREAD));
    assert!(!in_library(&t, P0, DREAD));
    // Rest in Peace exiles it instead of putting it into the graveyard: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Rest in Peace");
    let dread = t.battlefield(P0, DREAD);
    destroy(&mut t, dread);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.zone(t.g.current(dread)), Zone::Exile);
    assert!(!in_library(&t, P0, DREAD));
}
