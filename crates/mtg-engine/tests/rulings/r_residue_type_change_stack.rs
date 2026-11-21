//! Item `rulings-residue-type-change` — interaction rulings about The Tomb of Aclazotz's
//! permission to cast a creature spell from a graveyard (CR 601.3, 400.7a, 400.7h):
//! the type it gives the spell and the permanent, timing, and several permissions.

use crate::r_residue_type_change_common::*;
use mtg_engine::ability::AbilityKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const TOMB: &str = "Tarrian's Journal // The Tomb of Aclazotz";

/// P0's Tarrian's Journal, transformed into The Tomb of Aclazotz.
fn tomb(t: &mut TestGame) -> ObjectId {
    supported(TOMB);
    let j = t.battlefield(P0, TOMB);
    assert!(mtg_engine::dfc::transform(&mut t.g, j));
    t.g.current(j)
}

/// P0 activates the Tomb's last ability (untapping it first, "somehow") and resolves it.
fn use_tomb(t: &mut TestGame, tomb: ObjectId) {
    let tomb = t.g.current(tomb);
    t.g.objects[tomb.0 as usize].tapped = false;
    t.g.recompute();
    let n = t
        .obj_now(tomb)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .count();
    t.activate(P0, tomb, n - 1, &[]).unwrap();
    t.resolve_all();
}

/// Whether P0 could cast the card now (any way).
fn castable(t: &mut TestGame, card: ObjectId) -> bool {
    t.g.turn.priority = Some(P0);
    t.g.recompute();
    let card = t.g.current(card);
    t.g.legal_actions(P0)
        .iter()
        .any(|a| matches!(a, mtg_engine::decision::Action::Cast { card: c, .. } if *c == card))
}

#[test]
fn tomb_spell_is_a_vampire_on_the_stack_and_on_the_battlefield() {
    cr!("400.7a", "601.3", "205.1b", "611.2f", "601.2a", "601.2i");
    ruling!(
        "Tarrian's Journal // The Tomb of Aclazotz",
        "The creature spell you cast will be a Vampire in addition to its other types while on the stack. The resulting creature will be a Vampire in addition to its other types."
    );
    supported("Voldaren Estate");
    supported("Edgar Markov");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let tomb = tomb(&mut t);
    t.battlefield(P0, "Edgar Markov");
    let bears = t.graveyard(P0, "Grizzly Bears");
    use_tomb(&mut t, tomb);
    // P0's only mana: two Voldaren Estates, whose colored mana ("{T}, Pay 1 life: Add one
    // mana of any color. Spend this mana only to cast a Vampire spell.") is the only {G}.
    t.battlefield(P0, "Voldaren Estate");
    t.battlefield(P0, "Voldaren Estate");
    let spell = t.cast(P0, bears).go();
    assert_eq!(t.zone(spell), Zone::Stack);
    assert!(subtype(&mut t, spell, "Vampire") && subtype(&mut t, spell, "Bear"));
    // It was a Vampire from the moment it was put on the stack (CR 611.2f): as its cost
    // was paid (with an Estate's colored mana) ...
    assert!(t.life(P0) < 20);
    // ... and as it became cast: Edgar Markov's "Whenever you cast another Vampire spell"
    // triggered.
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(crate::r_s25_common::creature_tokens(&t, P0), 1);
    let b = t.named_on_battlefield("Grizzly Bears")[0];
    assert!(subtype(&mut t, b, "Vampire") && subtype(&mut t, b, "Bear"));
    assert_eq!(t.counters(b, "finality"), 1);
    // Still a Vampire on later turns.
    t.advance_to(P1, Step::Upkeep);
    assert!(subtype(&mut t, b, "Vampire"));
}

#[test]
fn tomb_permission_is_for_a_card_chosen_later_in_the_turn() {
    cr!("601.3", "611.2a");
    ruling!(
        "Tarrian's Journal // The Tomb of Aclazotz",
        "You don't choose which creature spell you're casting while activating or resolving The Tomb of Aclazotz's last ability."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let tomb = tomb(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    use_tomb(&mut t, tomb);
    // No card was chosen while activating or resolving it.
    assert!(asked_since(&t, from)
        .iter()
        .all(|(_, d)| !matches!(d, mtg_engine::decision::Decision::ChooseEntities { .. })));
    // A creature card that reaches the graveyard afterward can be cast with it.
    destroy(&mut t, bears);
    let card = t.g.current(bears);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    t.lands(P0, "Forest", 2);
    t.cast(P0, card).go();
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears")[0];
    assert!(subtype(&mut t, b, "Vampire"));
}

#[test]
fn tomb_permission_follows_creature_timing() {
    cr!("601.3", "307.1", "117.1a");
    ruling!(
        "Tarrian's Journal // The Tomb of Aclazotz",
        "You must still follow any timing restrictions and permissions for creature spells you cast with the permission granted by The Tomb of Aclazotz's last ability."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let tomb = tomb(&mut t);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    use_tomb(&mut t, tomb);
    // Not during combat ...
    assert!(!castable(&mut t, bears));
    // ... nor in a main phase while the stack isn't empty ...
    t.set_step(P0, Step::PostcombatMain);
    let elves = t.battlefield(P0, "Llanowar Elves");
    cast_new(&mut t, P0, "Giant Growth", &[obj(elves)]);
    assert!(!castable(&mut t, bears));
    t.resolve_all();
    // ... but in a main phase with an empty stack.
    assert!(castable(&mut t, bears));
    // A creature card with flash can be cast any time it could be.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let tomb2 = self::tomb(&mut t);
    let flash = t.graveyard(P0, "Spectral Sailor");
    t.lands(P0, "Island", 1);
    use_tomb(&mut t, tomb2);
    assert!(castable(&mut t, flash));
}

#[test]
fn tomb_permissions_add_up() {
    cr!("601.3", "601.3b");
    ruling!(
        "Tarrian's Journal // The Tomb of Aclazotz",
        "If you somehow activate The Tome of Aclazotz's ability more than once during a turn, you can cast a creature spell for each ability's permission."
    );
    supported("Lurrus of the Dream-Den");
    // Two activations: a creature spell for each, and no more.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let tomb = tomb(&mut t);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let elves = t.graveyard(P0, "Llanowar Elves");
    let third = t.graveyard(P0, "Grizzly Bears");
    use_tomb(&mut t, tomb);
    use_tomb(&mut t, tomb);
    t.lands(P0, "Forest", 5);
    t.cast(P0, bears).go();
    t.resolve_all();
    assert!(castable(&mut t, elves));
    t.cast(P0, elves).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
    // Both permissions are used up (two Forests are still untapped).
    assert!(!castable(&mut t, third));
    // Lurrus's permission first, then the Tomb's for another creature spell.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let tomb = self::tomb(&mut t);
    t.battlefield(P0, "Lurrus of the Dream-Den");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let elves = t.graveyard(P0, "Llanowar Elves");
    t.lands(P0, "Forest", 3);
    t.cast(P0, bears).go();
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears")[0];
    assert!(!subtype(&mut t, b, "Vampire"));
    // Lurrus's permission is used up this turn.
    assert!(!castable(&mut t, elves));
    use_tomb(&mut t, tomb);
    assert!(castable(&mut t, elves));
    t.cast(P0, elves).go();
    t.resolve_all();
    let e = t.named_on_battlefield("Llanowar Elves")[0];
    assert!(subtype(&mut t, e, "Vampire"));
}
