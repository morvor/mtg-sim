//! Rulings batch P056 — "Creatures you control get +N/+N until end of turn. Untap them."
//! (Great Oak Guardian, Rally to Battle, Rallying Roar): the affected set is locked in as
//! it resolves (CR 611.2c) and untapped creatures get the bonus too; Whisperwood
//! Elemental's granted ability (also locked in); Paradox Engine's cast trigger (CR 601.2i,
//! 603.3).

use crate::r_p056_common::*;
use crate::r_s01_common::supported;
use crate::r_s04_common::top_of_stack;
use crate::r_s06_common::activate_containing;
use crate::r_s29_common::cast_and_resolve;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn already_untapped_creatures_just_get_the_bonus() {
    cr!("611.2c", "701.26b");
    ruling!(
        "Great Oak Guardian",
        "Any creatures controlled by the player that are already untapped will just get +2/+2 until end of turn."
    );
    ruling!(
        "Rally to Battle",
        "Creatures you control that are already untapped get +1/+3, even though they can’t be untapped again."
    );
    ruling!(
        "Rallying Roar",
        "Untapped creatures you control get +1/+1 even though Rallying Roar doesn’t untap them."
    );
    // (card, the controller of the creatures, power/toughness bonus)
    for (spell, bonus) in [("Rally to Battle", (1, 3)), ("Rallying Roar", (1, 1))] {
        supported(spell);
        let mut t = TestGame::new(2);
        let tapped_one = t.battlefield(P0, "Grizzly Bears");
        let untapped = t.battlefield(P0, "Grizzly Bears");
        tap(&mut t, tapped_one);
        t.set_step(P0, Step::DeclareBlockers);
        cast_and_resolve(&mut t, P0, spell, &[]);
        for id in [tapped_one, untapped] {
            assert!(!tapped(&t, id), "{spell}");
            assert_eq!(t.pt(id), (2 + bonus.0, 2 + bonus.1), "{spell}");
        }
    }
    supported("Great Oak Guardian");
    let mut t = TestGame::new(2);
    let tapped_one = t.battlefield(P1, "Grizzly Bears");
    let untapped = t.battlefield(P1, "Grizzly Bears");
    tap(&mut t, tapped_one);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Great Oak Guardian");
    t.resolve_all();
    for id in [tapped_one, untapped] {
        assert!(!tapped(&t, id));
        assert_eq!(t.pt(id), (4, 4));
    }
}

#[test]
fn great_oak_guardian_targets_only_the_player() {
    cr!("115.1", "702.18a");
    ruling!(
        "Great Oak Guardian",
        "Great Oak Guardian's last ability targets only the player, not any creatures. Creatures with shroud that player controls will be affected"
    );
    let mut t = TestGame::new(2);
    let leviathan = t.battlefield(P1, "Inkwell Leviathan");
    tap(&mut t, leviathan);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Great Oak Guardian");
    t.resolve_all();
    assert!(!tapped(&t, leviathan));
    assert_eq!(t.pt(leviathan), (9, 13));
}

#[test]
fn rallying_roar_affects_only_creatures_there_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Rallying Roar",
        "Rallying Roar affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won’t get +1/+1."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_and_resolve(&mut t, P0, "Rallying Roar", &[]);
    let later = t.battlefield(P0, "Grizzly Bears");
    let stolen = t.battlefield(P1, "Grizzly Bears");
    crate::r_s06_common::give_control(&mut t, stolen, P0);
    assert_eq!(t.pt(bears), (3, 3));
    assert_eq!(t.pt(later), (2, 2));
    assert_eq!(t.pt(stolen), (2, 2));
}

#[test]
fn whisperwood_elemental_grants_only_to_creatures_there_as_it_resolves() {
    cr!("611.2c", "701.40a");
    ruling!(
        "Whisperwood Elemental",
        "The last ability grants that ability to face-up nontoken creatures you control as that ability resolves. Creatures that come under your control later in the turn won't have that ability."
    );
    supported("Whisperwood Elemental");
    let mut t = TestGame::new(2);
    let elemental = t.battlefield(P0, "Whisperwood Elemental");
    let bears = t.battlefield(P0, "Grizzly Bears");
    activate_containing(&mut t, P0, elemental, "Sacrifice").expect("activate");
    t.resolve_all();
    let later = t.battlefield(P0, "Hill Giant");
    let face_down = |t: &TestGame| t.g.permanents().filter(|o| o.face_down).count();
    t.g.destroy(later, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(
        face_down(&t),
        0,
        "the later creature didn't have the ability"
    );
    t.g.destroy(bears, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(face_down(&t), 1, "{}", t.dump_log());
}

#[test]
fn paradox_engine_trigger_resolves_even_if_the_spell_is_countered() {
    cr!("603.3", "601.2i", "701.6a");
    ruling!(
        "Paradox Engine",
        "Paradox Engine’s triggered ability resolves before the spell that caused it to trigger. The ability will resolve even if that spell is countered."
    );
    supported("Paradox Engine");
    let mut t = TestGame::new(2);
    let engine = t.battlefield(P0, "Paradox Engine");
    let stone = t.battlefield(P0, "Mind Stone");
    tap(&mut t, engine);
    tap(&mut t, stone);
    let forests = t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.settle();
    // The trigger is above the spell.
    let top = top_of_stack(&t);
    assert_ne!(top, t.g.current(bears));
    t.lands(P1, "Island", 2);
    let counter = t.hand(P1, "Counterspell");
    let spell = t.g.current(bears);
    t.cast(P1, counter).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(!tapped(&t, engine) && !tapped(&t, stone));
    // Lands aren't untapped.
    assert!(forests.iter().all(|f| tapped(&t, *f)));
}
