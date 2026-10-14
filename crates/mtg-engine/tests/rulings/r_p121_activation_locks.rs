//! Rulings batch P121 — effects that stop activated abilities (CR 602.5): they affect
//! only the objects they say (creatures on the battlefield, cards in graveyards), not
//! triggered or static abilities, nor turning a face-down creature face up (a special
//! action, CR 116.2b), nor an ability already on the stack (CR 113.7a); mana of any type
//! (CR 106.1b, snow isn't a type, CR 107.4h); Braided Quipu; and Arboria (CR 508.1c).

use crate::r_p121_common::*;
use crate::r_s06_common::attach_new;
use crate::r_s27_common::{activatable, can_activate_containing};
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- Ashes of the Abhorrent ------------------------------------------------------------------

#[test]
fn ashes_of_the_abhorrent_stops_activated_not_triggered_graveyard_abilities() {
    cr!("602.5", "603.6");
    ruling!(
        "Ashes of the Abhorrent",
        "Activated abilities contain a colon. They’re generally written “[Cost]: [Effect].” Some keyword abilities (such as embalm from the Amonkhet set) are activated abilities and will have colons in their reminder text. Triggered abilities (starting with “when,” “whenever,” or “at”) are unaffected by Ashes of the Abhorrent."
    );
    supported("Ashes of the Abhorrent");
    supported("Sacred Cat");
    supported("Bloodghast");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let cat = t.graveyard(P0, "Sacred Cat");
    assert!(can_activate_containing(&mut t, P0, cat, "Embalm"));
    t.battlefield(P1, "Ashes of the Abhorrent");
    assert!(activatable(&mut t, P0, cat).is_empty());
    // Bloodghast's landfall ability works from the graveyard.
    t.graveyard(P0, "Bloodghast");
    t.answer_yes(P0, true);
    let land = t.hand(P0, "Swamp");
    t.play_land(P0, land).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Bloodghast").len(), 1);
}

#[test]
fn ashes_of_the_abhorrent_destroyed_with_a_creature_still_gains_life() {
    cr!("603.10a");
    ruling!(
        "Ashes of the Abhorrent",
        "If a creature dies at the same time that Ashes of the Abhorrent is destroyed, you’ll gain 1 life."
    );
    let mut t = TestGame::new(2);
    let ashes = t.battlefield(P0, "Ashes of the Abhorrent");
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_p146_common::destroy_together(&mut t, &[ashes, bears]);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

// --- Cursed Totem and Linvala ---------------------------------------------------------------

#[test]
fn cursed_totem_lets_noncreatures_become_creatures() {
    cr!("602.5", "702.122a");
    ruling!(
        "Cursed Totem",
        "Activated abilities of noncreature permanents that would make them become creatures (such as a Vehicle's crew ability) can still be activated. Once they become creatures, any abilities they have cannot be activated."
    );
    supported("Smuggler's Copter");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cursed Totem");
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    assert!(can_activate_containing(&mut t, P0, copter, "Crew"));
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, copter, 0, &[]).expect("crew");
    t.resolve_all();
    assert!(crate::r_p037_common::is_creature(&t, copter));
    assert!(activatable(&mut t, P0, copter).is_empty());
}

#[test]
fn cursed_totem_doesnt_stop_cards_in_other_zones_or_other_abilities() {
    cr!("602.5", "702.29a", "702.84a", "603.2");
    ruling!(
        "Cursed Totem",
        "Cursed Totem doesn't stop players from activating abilities of creature cards that aren't on the battlefield, such as cycling or unearth."
    );
    ruling!(
        "Cursed Totem",
        "Cursed Totem stops players from activating mana abilities of creatures."
    );
    ruling!(
        "Cursed Totem",
        "Other abilities of creatures, including static abilities and triggered abilities, are unaffected."
    );
    supported("Scrapwork Mutt");
    supported("Soul Warden");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cursed Totem");
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Swamp", 1);
    let horror = t.hand(P0, "Horror of the Broken Lands");
    assert!(crate::r_s04_common::can_cycle(&mut t, P0, horror));
    let mutt = t.graveyard(P0, "Scrapwork Mutt");
    assert!(can_activate_containing(&mut t, P0, mutt, "Unearth"));
    let elves = t.battlefield(P0, "Llanowar Elves");
    assert!(activatable(&mut t, P0, elves).is_empty());
    assert!(t.activate(P0, elves, 0, &[]).is_err());
    // Static (Goblin King) and triggered (Soul Warden) abilities of creatures work.
    t.battlefield(P0, "Goblin King");
    let goblin = t.battlefield(P0, "Raging Goblin");
    assert_eq!(t.pt(goblin), (2, 2));
    t.battlefield(P0, "Soul Warden");
    t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn linvala_affects_only_creatures_on_the_battlefield() {
    cr!("602.5", "702.84a", "603.2");
    ruling!(
        "Linvala, Keeper of Silence",
        "Linvala’s last ability affects only creatures on the battlefield. Activated abilities that work in other zones (such as bloodrush or unearth) can still be activated. Triggered abilities (starting with “when,” “whenever,” or “at”) are unaffected."
    );
    supported("Linvala, Keeper of Silence");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Linvala, Keeper of Silence");
    t.lands(P0, "Mountain", 2);
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    assert!(activatable(&mut t, P0, sorcerer).is_empty());
    let mutt = t.graveyard(P0, "Scrapwork Mutt");
    assert!(can_activate_containing(&mut t, P0, mutt, "Unearth"));
    t.battlefield(P0, "Soul Warden");
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

// --- Detainment Spell -------------------------------------------------------------------------

#[test]
fn detainment_spell_doesnt_stop_turning_face_up() {
    cr!("116.2b", "602.5", "702.37e");
    ruling!(
        "Detainment Spell",
        "Morph isn't an activated ability, so Detainment Spell doesn't prevent face-down creatures from being able to turn face up."
    );
    supported("Detainment Spell");
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 3);
    t.lands(P0, "Forest", 2);
    let hydra = t.hand(P0, "Hooded Hydra");
    let fd = t
        .cast(P0, hydra)
        .method(mtg_engine::object::CastMethod::FaceDown(
            mtg_engine::keywords::KeywordKind::Morph,
        ))
        .go();
    t.resolve_all();
    let fd = t.g.current(fd);
    assert!(t.obj_now(fd).face_down);
    attach_new(&mut t, P1, "Detainment Spell", fd);
    // Mana for the morph cost {3}{G}{G}.
    t.lands(P0, "Wastes", 3);
    t.lands(P0, "Forest", 2);
    assert!(crate::r_s11_common::turn_face_up(&mut t, P0, fd));
    t.resolve_all();
    assert!(!t.obj_now(fd).face_down);
    assert_eq!(plus1(&t, fd), 5);
}

#[test]
fn detainment_spell_moved_in_response_doesnt_stop_the_ability() {
    cr!("113.7a", "602.5");
    ruling!(
        "Detainment Spell",
        "Responding to a creature's activated ability being activated by moving Detainment Spell onto that creature won't affect the ability that was just activated."
    );
    let mut t = TestGame::new(2);
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let spell = attach_new(&mut t, P1, "Detainment Spell", bears);
    t.activate(P0, sorcerer, 0, &[Entity::Player(P1)])
        .expect("ping");
    t.lands(P1, "Plains", 2);
    t.activate(P1, spell, 0, &[obj(sorcerer)]).expect("move");
    t.resolve();
    assert_eq!(
        crate::r_s06_common::attached_to(&t, spell),
        Some(obj(sorcerer))
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

// --- Sharkey ----------------------------------------------------------------------------------

#[test]
fn sharkey_spends_mana_of_any_type_but_not_nonsnow_for_snow() {
    cr!("106.1b", "107.4h", "609.4b");
    ruling!(
        "Sharkey, Tyrant of the Shire",
        "Sharkey, Tyrant of the Shire's last ability uses a new template indicating that you may spend mana as though it were mana of any type to activate Sharkey, Tyrant of the Shire's abilities. The six types of mana are white, blue, black, red, green, and colorless."
    );
    ruling!(
        "Sharkey, Tyrant of the Shire",
        "Snow mana is not a type of mana. Sharkey, Tyrant of the Shire's last ability won't let you pay a snow cost using mana produced by a nonsnow source."
    );
    supported("Sharkey, Tyrant of the Shire");
    supported("Kessig Wolf Run");
    supported("Mouth of Ronom");
    // Kessig Wolf Run's {X}{R}{G} paid with blue mana.
    let mut t = TestGame::new(2);
    let sharkey = t.battlefield(P0, "Sharkey, Tyrant of the Shire");
    t.battlefield(P1, "Kessig Wolf Run");
    t.lands(P0, "Island", 2);
    assert!(can_activate_containing(&mut t, P0, sharkey, "trample"));
    // Mouth of Ronom's {4}{S}: not with mana from nonsnow Islands, even with enough.
    let mut t = TestGame::new(2);
    let sharkey = t.battlefield(P0, "Sharkey, Tyrant of the Shire");
    t.battlefield(P1, "Mouth of Ronom");
    t.lands(P0, "Island", 5);
    assert!(!can_activate_containing(&mut t, P0, sharkey, "damage"));
    let mut t = TestGame::new(2);
    let sharkey = t.battlefield(P0, "Sharkey, Tyrant of the Shire");
    t.battlefield(P1, "Mouth of Ronom");
    t.lands(P0, "Island", 4);
    t.lands(P0, "Snow-Covered Island", 1);
    assert!(can_activate_containing(&mut t, P0, sharkey, "damage"));
}

// --- Braided Quipu ----------------------------------------------------------------------------

#[test]
fn braided_quipu_goes_to_the_bottom_of_a_small_library_front_face_up() {
    cr!("401.4", "712.8a", "400.7");
    ruling!(
        "Braided Net // Braided Quipu",
        "After drawing cards for Braided Quipu's ability, if you have two or fewer cards in your library, you'll put Braided Quipu on the bottom of your library."
    );
    ruling!(
        "Braided Net // Braided Quipu",
        "Once Braided Quipu is back in your library, Braided Net will be the face that's up again."
    );
    let mut t = TestGame::new(2);
    let quipu = enter_with(&mut t, P0, "Braided Net // Braided Quipu", 0, true);
    let quipu = t.g.current(quipu);
    t.g.objects[quipu.0 as usize].summoning_sick = false;
    crate::r_p189_common::set_library(&mut t, P0, &["Grizzly Bears", "Hill Giant", "Opt"]);
    t.lands(P0, "Island", 4);
    t.activate(P0, quipu, 0, &[]).expect("activate");
    t.resolve_all();
    // One artifact: one card drawn, two left; Quipu goes to the bottom.
    let lib = t.g.player(P0).library.clone();
    assert_eq!(lib.len(), 3);
    let bottom = t.g.obj(lib[0]);
    assert_eq!(bottom.chars.name, "Braided Net");
}

// --- Arboria ----------------------------------------------------------------------------------

/// Arboria is on the battlefield; P0's turn 1 (doing nothing else than `during`) ends and
/// P1's turn begins, in its beginning of combat step.
fn arboria_p0_turn(t: &mut TestGame, during: impl FnOnce(&mut TestGame)) {
    t.battlefield(P1, "Arboria");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    during(t);
    t.advance_to(P1, Step::BeginningOfCombat);
}

#[test]
fn arboria_doesnt_stop_attacks_on_planeswalkers() {
    cr!("508.1c", "506.3");
    ruling!(
        "Arboria",
        "Arboria doesn’t stop creatures from attacking planeswalkers, regardless of what their controllers did or didn’t do during their last turns."
    );
    supported("Arboria");
    let mut t = TestGame::new(2);
    let pw = t.battlefield(P0, "Jace Beleren");
    let bears = t.battlefield(P1, "Grizzly Bears");
    arboria_p0_turn(&mut t, |_| {});
    assert!(!crate::r_s09_common::legal_attack(
        &mut t,
        &[(bears, Entity::Player(P0))]
    ));
    assert!(crate::r_s09_common::legal_attack(&mut t, &[(bears, obj(pw))]));
}

#[test]
fn arboria_cares_who_put_the_permanent_onto_the_battlefield() {
    cr!("508.1c");
    ruling!(
        "Arboria",
        "Arboria’s effect cares whether a player put a nontoken permanent onto the battlefield. It’s unusual for an ability to care who put a permanent onto the battlefield, as opposed to under whose control it entered."
    );
    /// During P0's turn, `by` puts a Grizzly Bears onto the battlefield under
    /// `controller`'s control.
    fn put(t: &mut TestGame, by: PlayerId, controller: PlayerId) {
        let id =
            t.g.create_card_object(mtg_engine::card::card("Grizzly Bears"), by, Zone::Nowhere);
        t.g.move_object_ev(MoveEv {
            obj: id,
            to: Zone::Battlefield,
            pos: mtg_engine::ability::LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(by),
            etb: EtbInfo {
                controller: Some(controller),
                ..Default::default()
            },
            source: None,
        })
        .expect("entered");
        t.g.flush_events();
        t.settle();
    }
    for (by, controller, attackable) in [(P0, P1, true), (P1, P0, false)] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        arboria_p0_turn(&mut t, |t| put(t, by, controller));
        assert_eq!(
            crate::r_s09_common::legal_attack(&mut t, &[(bears, Entity::Player(P0))]),
            attackable,
            "put by {by:?} under {controller:?}'s control"
        );
    }
}
