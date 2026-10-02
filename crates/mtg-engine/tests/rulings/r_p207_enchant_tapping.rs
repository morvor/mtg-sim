//! Rulings batch P207 — enchant (CR 303.4, 702.5): Auras about tapped and untapped
//! permanents — "doesn't untap" Auras on untapped creatures, "becomes tapped" triggers,
//! "enchant tapped creature", enchant restrictions that check land subtypes, and an
//! untap ability on a land that also doesn't untap during its controller's untap step.

use crate::r_s01_common::*;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s04_common::*;
use crate::r_s05_common::*;
use crate::r_s06_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 casts the Aura `name` targeting `target` and resolves it. Whether it was cast.
fn cast_aura(t: &mut TestGame, name: &str, target: ObjectId) -> Option<ObjectId> {
    supported(name);
    let aura = in_hand_with_mana(t, P0, name);
    let r = t.cast(P0, aura).target(target).try_go().ok();
    t.resolve_all();
    r.map(|_| t.g.current(aura))
}

/// Untaps `id` as an effect would.
fn untap(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.untap(id);
    t.g.flush_events();
    t.settle();
}

/// P1's Icy Manipulator taps `id`.
fn icy(t: &mut TestGame, id: ObjectId) {
    supported("Icy Manipulator");
    let icy = t.battlefield(P1, "Icy Manipulator");
    t.lands(P1, "Wastes", 1);
    t.activate(P1, icy, 0, &[Entity::Object(t.g.current(id))])
        .unwrap();
    t.resolve_all();
}

// ---------------------------------------------------------------------------------------
// "Doesn't untap" Auras on untapped permanents
// ---------------------------------------------------------------------------------------

/// The Aura `name` is cast on P1's untapped creature (or artifact); the creature stays
/// untapped until it becomes tapped, then doesn't untap during P1's untap step.
fn doesnt_untap_on_an_untapped(name: &str, target_card: &str) {
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, target_card);
    assert!(!t.obj_now(target).tapped);
    let aura = cast_aura(&mut t, name, target).expect("the Aura couldn't be cast");
    assert_eq!(attached_to(&t, aura), Some(Entity::Object(target)));
    assert!(!t.obj_now(target).tapped, "{name} tapped it");
    t.g.tap(target);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(target).tapped, "{name}: it untapped");
}

#[test]
fn doesnt_untap_auras_may_enchant_untapped_creatures() {
    cr!("303.4a", "302.6", "502.3");
    ruling!(
        "Bonds of Quicksilver",
        "Bonds of Quicksilver may target and may enchant an untapped creature."
    );
    ruling!(
        "Controlled Instincts",
        "Controlled Instincts may target and may enchant an untapped creature."
    );
    ruling!(
        "Ice Over",
        "Ice Over may target and may enchant an untapped creature."
    );
    ruling!(
        "Coma Veil",
        "Coma Veil may target and may enchant an untapped artifact or creature."
    );
    doesnt_untap_on_an_untapped("Bonds of Quicksilver", "Grizzly Bears");
    // (Controlled Instincts: enchant red or green creature.)
    doesnt_untap_on_an_untapped("Controlled Instincts", "Grizzly Bears");
    doesnt_untap_on_an_untapped("Ice Over", "Grizzly Bears");
    doesnt_untap_on_an_untapped("Coma Veil", "Grizzly Bears");
    doesnt_untap_on_an_untapped("Coma Veil", "Mind Stone");
}

#[test]
fn claustrophobia_can_enchant_a_tapped_or_untapped_creature() {
    cr!("303.4a", "302.6", "502.3");
    ruling!(
        "Claustrophobia",
        "Claustrophobia can target and enchant a tapped or untapped creature."
    );
    for tapped in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        if tapped {
            t.g.tap(bears);
        }
        let aura = cast_aura(&mut t, "Claustrophobia", bears).unwrap();
        assert_eq!(attached_to(&t, aura), Some(Entity::Object(bears)));
        assert!(t.obj_now(bears).tapped);
        t.advance_to(P1, Step::Upkeep);
        assert!(t.obj_now(bears).tapped);
    }
}

// ---------------------------------------------------------------------------------------
// "Becomes tapped" triggers
// ---------------------------------------------------------------------------------------

#[test]
fn brink_of_disaster_on_a_tapped_permanent_waits_for_it_to_become_tapped() {
    cr!("303.4a", "701.26a", "603.2");
    ruling!(
        "Brink of Disaster",
        "Brink of Disaster may target and may enchant a permanent that’s already tapped. It won’t do anything until the enchanted permanent changes from being untapped to being tapped."
    );
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    t.g.tap(land);
    let aura = cast_aura(&mut t, "Brink of Disaster", land).unwrap();
    assert_eq!(attached_to(&t, aura), Some(Entity::Object(land)));
    assert!(t.on_battlefield(land));
    untap(&mut t, land);
    assert!(t.on_battlefield(land));
    // Tapped by an opponent's effect.
    icy(&mut t, land);
    assert!(!t.on_battlefield(land));
    assert!(t.in_graveyard(P1, "Forest"));
}

#[test]
fn cryoshatter_on_a_tapped_creature_waits_for_it_to_become_tapped() {
    cr!("303.4a", "701.26a", "603.2");
    ruling!(
        "Cryoshatter",
        "Cryoshatter may target and may enchant a creature that’s already tapped. An ability that triggers when a creature “becomes tapped” won’t trigger until the creature changes from being untapped to being tapped."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.tap(giant);
    cast_aura(&mut t, "Cryoshatter", giant).unwrap();
    assert_eq!(t.pt(giant), (-2, 3));
    assert!(t.on_battlefield(giant));
    // P1's untap step: it untaps (nothing happens), then it attacks and becomes tapped.
    t.advance_to(P1, Step::PrecombatMain);
    assert!(!t.obj_now(giant).tapped);
    assert!(t.on_battlefield(giant));
    attack_with(&mut t, &[(giant, Entity::Player(P0))]);
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
}

/// The Aura `name` enchants P1's Forest (or `land`); the land becomes tapped by P1's
/// mana ability and by an Icy Manipulator: the trigger happens each time.
fn land_tapped_trigger(name: &str, land: &str, per: i32, check: fn(&TestGame) -> i32) {
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, land);
    cast_aura(&mut t, name, forest).unwrap();
    let start = check(&t);
    // Tapped by an opponent's effect, not for mana.
    icy(&mut t, forest);
    assert_eq!(check(&t) - start, per, "{name}: not after Icy Manipulator");
    // Tapped for mana by its controller.
    untap(&mut t, forest);
    t.g.turn.priority = Some(P1);
    let gg = t.hand(P1, "Giant Growth");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.cast(P1, gg).target(bears).go();
    t.resolve_all();
    assert_eq!(check(&t) - start, 2 * per, "{name}: not when tapped for mana");
}

#[test]
fn becomes_tapped_land_auras_trigger_for_any_reason() {
    cr!("701.26a", "603.2", "106.12");
    ruling!(
        "Chronic Flooding",
        "Chronic Flooding’s ability will trigger whenever the enchanted land becomes tapped for any reason, not just because its controller taps it for mana."
    );
    ruling!(
        "Contaminated Ground",
        "Contaminated Ground's last ability triggers whenever the enchanted land becomes tapped for any reason, not just when it's tapped for mana."
    );
    ruling!(
        "Corrupted Roots",
        "Corrupted Roots doesn’t care why the enchanted land becomes tapped. If that land changes from being untapped to being tapped for any reason, the ability will trigger."
    );
    // Chronic Flooding: its controller mills three cards.
    land_tapped_trigger("Chronic Flooding", "Forest", 3, |t| {
        -(t.library_size(P1) as i32)
    });
    // Contaminated Ground (the land becomes a Swamp, which taps for {B}): its controller
    // loses 2 life.
    land_tapped_trigger_swamp("Contaminated Ground");
    // Corrupted Roots: its controller loses 2 life.
    land_tapped_trigger("Corrupted Roots", "Forest", 2, |t| -t.life(P1));
}

/// Contaminated Ground makes the land a Swamp: it's tapped by Icy Manipulator and for
/// {B} (Disfigure).
fn land_tapped_trigger_swamp(name: &str) {
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    cast_aura(&mut t, name, land).unwrap();
    assert!(t.obj_now(land).chars.has_subtype("Swamp"));
    icy(&mut t, land);
    assert_eq!(t.life(P1), 18);
    untap(&mut t, land);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let disfigure = t.hand(P1, "Disfigure");
    t.g.turn.priority = Some(P1);
    t.cast(P1, disfigure).target(bears).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

// ---------------------------------------------------------------------------------------
// Enchant restrictions
// ---------------------------------------------------------------------------------------

#[test]
fn corrupted_roots_checks_forest_or_plains_subtypes_continuously() {
    cr!("303.4a", "303.4d", "704.5m", "305.7");
    ruling!(
        "Corrupted Roots",
        "Corrupted Roots can enchant only a Forest or a Plains. It checks the enchanted land’s subtypes, not its name. If at any time the enchanted land is neither a Forest nor a Plains (due to Unstable Frontier’s ability, perhaps), Corrupted Roots is put into its owner’s graveyard as a state-based action."
    );
    supported("Savannah");
    supported("Contaminated Ground");
    // Savannah (Forest Plains) can be enchanted; a Mountain can't.
    let mut t = TestGame::new(2);
    let mountain = t.battlefield(P1, "Mountain");
    assert!(cast_aura(&mut t, "Corrupted Roots", mountain).is_none());
    let savannah = t.battlefield(P1, "Savannah");
    let roots = cast_aura(&mut t, "Corrupted Roots", savannah).unwrap();
    assert_eq!(attached_to(&t, roots), Some(Entity::Object(savannah)));
    // Contaminated Ground makes it a Swamp (and nothing else): Roots falls off.
    cast_aura(&mut t, "Contaminated Ground", savannah).unwrap();
    assert!(!t.obj_now(savannah).chars.has_subtype("Forest"));
    assert!(!t.on_battlefield(roots));
    assert!(t.in_graveyard(P0, "Corrupted Roots"));
}

#[test]
fn entangling_vines_falls_off_a_creature_that_becomes_untapped() {
    cr!("303.4a", "303.4d", "704.5m");
    ruling!(
        "Entangling Vines",
        "Entangling Vines can enchant only a creature that’s tapped. If at any time the enchanted creature is untapped, Entangling Vines is put into its owner’s graveyard as a state-based action."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(cast_aura(&mut t, "Entangling Vines", bears).is_none());
    t.g.tap(bears);
    let vines = cast_aura(&mut t, "Entangling Vines", bears).unwrap();
    assert_eq!(attached_to(&t, vines), Some(Entity::Object(bears)));
    // It doesn't untap during P1's untap step.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(vines));
    // An effect untaps it: the Vines is put into the graveyard.
    untap(&mut t, bears);
    assert!(!t.on_battlefield(vines));
    assert!(t.in_graveyard(P0, "Entangling Vines"));
}

// ---------------------------------------------------------------------------------------
// Urban Burgeoning
// ---------------------------------------------------------------------------------------

#[test]
fn doesnt_untap_during_your_untap_step_doesnt_stop_urban_burgeoning() {
    cr!("502.3", "613.1f");
    ruling!(
        "Urban Burgeoning",
        "Effects that state that the enchanted land doesn’t untap during your untap step won’t apply during another player’s untap step."
    );
    supported("Urban Burgeoning");
    supported("Choke");
    // Choke: "Islands don't untap during their controllers' untap steps."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Choke");
    let island = t.battlefield(P0, "Island");
    attach_new(&mut t, P0, "Urban Burgeoning", island);
    t.g.tap(island);
    // P1's untap step: it untaps.
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(island).tapped);
    // P0's untap step: Choke applies.
    t.g.tap(island);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(island).tapped);
}
