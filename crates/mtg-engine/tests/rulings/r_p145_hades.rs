//! Rulings batch P145 — Hades, Sorcerer of Eld: "If a card or token would be put into
//! your graveyard from anywhere, exile it instead", and playing cards from the graveyard.

use crate::r_p079_common::back_face;
use crate::r_p145_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

const EMET: &str = "Emet-Selch, Unsundered // Hades, Sorcerer of Eld";

fn hades(t: &mut TestGame) -> ObjectId {
    supported(EMET);
    let h = back_face(t, P0, EMET);
    assert_eq!(name_of(t, h), "Hades, Sorcerer of Eld");
    h
}

#[test]
fn hades_destroyed_or_killed_by_your_own_spell() {
    cr!("614.1a", "608.2n", "704.5g");
    ruling!(
        "Emet-Selch, Unsundered // Hades, Sorcerer of Eld",
        "If a Hades, Sorcerer of Eld you control is destroyed by a spell you own, Hades will be exiled and then the spell will be put into your graveyard."
    );
    // Murder: Hades is exiled as the spell resolves; then Murder goes to the graveyard
    // (Hades is gone).
    let mut t = TestGame::new(2);
    let h = hades(&mut t);
    cast_resolve(&mut t, P0, "Murder", &[obj(h)]);
    assert_eq!(t.zone(h), Zone::Exile);
    assert!(t.in_graveyard(P0, "Murder"));
    // Fireball for 6: the spell is put into the graveyard (exiled, Hades is still there)
    // before state-based actions destroy Hades (exiled).
    let mut t = TestGame::new(2);
    let h = hades(&mut t);
    t.lands(P0, "Mountain", 7);
    let fireball = t.hand(P0, "Fireball");
    t.cast(P0, fireball).x(6).target(obj(h)).go();
    t.resolve_all();
    assert_eq!(t.zone(h), Zone::Exile);
    assert!(t.in_exile("Fireball"));
    assert!(!t.in_graveyard(P0, "Fireball"));
}

#[test]
fn hades_dies_triggers_of_your_permanents_dont_trigger() {
    cr!("614.1a", "700.4", "603.6c");
    ruling!(
        "Emet-Selch, Unsundered // Hades, Sorcerer of Eld",
        "While you control Hades, Sorcerer of Eld, abilities that trigger whenever a permanent you own is put into your graveyard from the battlefield"
    );
    supported("Doomed Traveler");
    let mut t = TestGame::new(2);
    hades(&mut t);
    let traveler = t.battlefield(P0, "Doomed Traveler");
    destroy(&mut t, traveler);
    assert_eq!(t.zone(traveler), Zone::Exile);
    assert_eq!(t.stack_len(), 0);
    // An opponent's creature still dies normally.
    let theirs = t.battlefield(P1, "Doomed Traveler");
    destroy(&mut t, theirs);
    assert_eq!(triggers_on_stack(&t, "dies"), 1);
}

#[test]
fn hades_discarded_cards_are_still_discarded() {
    cr!("701.9a", "702.35a", "616.1", "701.50a");
    ruling!(
        "Emet-Selch, Unsundered // Hades, Sorcerer of Eld",
        "If you discard a card while you control Hades, Sorcerer of Eld, abilities that function when a card is discarded (such as madness) still work, even though that card never reaches your graveyard."
    );
    // Madness: P0 applies madness rather than Hades (CR 616.1), so Fiery Temper is exiled
    // by madness and its trigger lets P0 cast it.
    let mut t = TestGame::new(2);
    hades(&mut t);
    let temper = t.hand(P0, "Fiery Temper");
    t.answer(
        P0,
        DecisionKind::Replacement,
        mtg_engine::decision::Answer::Index(1),
    );
    discard(&mut t, P0, temper);
    assert_eq!(t.zone(temper), Zone::Exile);
    assert_eq!(stacked_triggers(&t), 1, "the madness trigger");
    // Connive: Ledger Shredder finds the discarded nonland card in exile.
    let mut t = TestGame::new(2);
    hades(&mut t);
    let shredder = t.battlefield(P0, "Ledger Shredder");
    let bears = t.hand(P0, "Grizzly Bears");
    t.library_top(P0, "Island");
    cast_new(&mut t, P0, "Dark Ritual", &[]);
    t.resolve_all();
    cast_new(&mut t, P0, "Ornithopter", &[]);
    discard_these(&mut t, P0, &[bears]);
    t.resolve();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.counters(shredder, counters::PLUS1), 1);
}

#[test]
fn hades_graveyard_cards_follow_costs_and_timing() {
    cr!("601.2", "305.2", "307.1");
    ruling!(
        "Emet-Selch, Unsundered // Hades, Sorcerer of Eld",
        "You pay all costs and follow all timing rules for cards played with the permission granted by Hades, Sorcerer of Eld's second ability."
    );
    let mut t = TestGame::new(2);
    hades(&mut t);
    let divination = t.graveyard(P0, "Divination");
    let forest = t.graveyard(P0, "Forest");
    let swamp = t.graveyard(P0, "Swamp");
    // No mana: it can't be cast.
    assert!(!can_cast(&mut t, P0, divination, Default::default()));
    t.lands(P0, "Island", 3);
    assert!(can_cast(&mut t, P0, divination, Default::default()));
    // A sorcery needs an empty stack in a main phase.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_cast(&mut t, P0, divination, Default::default()));
    t.set_step(P0, Step::PrecombatMain);
    // A land from the graveyard is the turn's land play.
    assert!(can_play_land(&mut t, P0, forest));
    t.play_land(P0, forest).expect("land");
    assert!(!can_play_land(&mut t, P0, swamp));
    // Not on an opponent's turn.
    t.set_step(P1, Step::PrecombatMain);
    t.g.turn.priority = Some(P0);
    assert!(!can_cast(&mut t, P0, divination, Default::default()));
}
