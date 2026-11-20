//! Rulings batch P086 — tapping and "doesn't untap" locks, and control for as long as a
//! permanent stays tapped: Tidebinder Mage, Dungeon Geists, Encase in Ice, Rust Tick,
//! Phyrexian Gremlins, Relic Barrier, Willow Satyr and Holy Justiciar.

use crate::r_p086_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0's `name` ("When this creature enters, tap target ... creature an opponent controls.
/// That creature doesn't untap during its controller's untap step for as long as you
/// control this creature.") enters targeting `target`; the trigger is left on the stack.
fn tidebinder_enters(t: &mut TestGame, name: &str, target: ObjectId) -> ObjectId {
    t.answer_targets(P0, &[obj(target)]);
    let mage = t.enter(P0, name);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    mage
}

#[test]
fn tidebinder_mage_lock_ends_for_good_when_another_player_gains_control_of_it() {
    cr!("611.2a", "502.3");
    ruling!(
        "Tidebinder Mage",
        "If another player gains control of Tidebinder Mage, the creature will no longer be affected by the ability preventing it from untapping, even if you later regain control of Tidebinder Mage."
    );
    supported("Tidebinder Mage");
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P1, "Raging Goblin");
    let mage = tidebinder_enters(&mut t, "Tidebinder Mage", goblin);
    t.resolve_all();
    assert!(is_tapped(&t, goblin));
    give_control(&mut t, mage, P1);
    give_control(&mut t, mage, P0);
    next_main(&mut t, P1);
    assert!(!is_tapped(&t, goblin));
}

#[test]
fn tidebinder_mage_lock_follows_the_creature_to_your_untap_step() {
    cr!("611.2a", "502.3");
    ruling!(
        "Tidebinder Mage",
        "If you gain control of the creature that’s being prevented from untapping, that creature won’t untap during your untap step for as long as you control Tidebinder Mage."
    );
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P1, "Raging Goblin");
    let mage = tidebinder_enters(&mut t, "Tidebinder Mage", goblin);
    t.resolve_all();
    give_control(&mut t, goblin, P0);
    next_main(&mut t, P1);
    next_main(&mut t, P0);
    assert!(is_tapped(&t, goblin));
    // Once P0 stops controlling the Mage, the goblin untaps as normal.
    destroy(&mut t, mage);
    next_main(&mut t, P0);
    assert!(!is_tapped(&t, goblin));
}

#[test]
fn tidebinder_mage_leaving_control_before_its_trigger_resolves_only_taps() {
    cr!("611.2b", "608.2c");
    ruling!(
        "Tidebinder Mage",
        "If you lose control of Tidebinder Mage between when it entered the battlefield and when the ability resolves, the target creature will become tapped. However, it won’t be affected by the other effect and will untap as normal during its controller’s untap step."
    );
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P1, "Raging Goblin");
    let mage = tidebinder_enters(&mut t, "Tidebinder Mage", goblin);
    give_control(&mut t, mage, P1);
    t.resolve_all();
    assert!(is_tapped(&t, goblin));
    give_control(&mut t, mage, P0);
    next_main(&mut t, P1);
    assert!(!is_tapped(&t, goblin));
}

#[test]
fn tidebinder_mage_control_lost_and_regained_before_its_trigger_resolves_only_taps() {
    cr!("611.2b");
    interp!("for-as-long-as-checked-continually-from-activation");
    // The "for as long as you control" duration ended while the trigger was on the stack;
    // regaining control before it resolves doesn't begin it again.
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P1, "Raging Goblin");
    let mage = tidebinder_enters(&mut t, "Tidebinder Mage", goblin);
    give_control(&mut t, mage, P1);
    give_control(&mut t, mage, P0);
    t.resolve_all();
    assert_eq!(controller(&t, mage), P0);
    assert!(is_tapped(&t, goblin));
    next_main(&mut t, P1);
    assert!(!is_tapped(&t, goblin));
}

#[test]
fn tap_and_lock_abilities_can_target_a_tapped_creature() {
    cr!("701.26a", "502.3");
    ruling!(
        "Dungeon Geists",
        "The ability can target a tapped creature. If the targeted creature is already tapped when it resolves, that creature just remains tapped and doesn’t untap during its controller’s next untap step."
    );
    ruling!(
        "Tidebinder Mage",
        "The ability can target a tapped creature. If the targeted creature is already tapped when it resolves, that creature just remains tapped and doesn’t untap during its controller’s next untap step."
    );
    for name in ["Dungeon Geists", "Tidebinder Mage"] {
        supported(name);
        let mut t = TestGame::new(2);
        let goblin = t.battlefield(P1, "Raging Goblin");
        tap(&mut t, goblin);
        tidebinder_enters(&mut t, name, goblin);
        t.resolve_all();
        assert!(is_tapped(&t, goblin), "{name}");
        next_main(&mut t, P1);
        assert!(is_tapped(&t, goblin), "{name}");
    }
}

#[test]
fn encase_in_ice_creature_can_be_untapped_by_other_effects() {
    cr!("502.3", "701.26b");
    ruling!(
        "Encase in Ice",
        "The creature can still be untapped by other spells and abilities."
    );
    supported("Encase in Ice");
    supported("Burst of Energy");
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P1, "Raging Goblin");
    t.answer_targets(P0, &[obj(goblin)]);
    cast_new(&mut t, P0, "Encase in Ice", &[]);
    t.resolve_all();
    assert_eq!(
        attached_to(&t, t.named_on_battlefield("Encase in Ice")[0]),
        Some(obj(goblin))
    );
    assert!(is_tapped(&t, goblin));
    cast_new(&mut t, P1, "Burst of Energy", &[obj(goblin)]);
    t.resolve_all();
    assert!(!is_tapped(&t, goblin));
    // It still doesn't untap during its controller's untap step.
    tap(&mut t, goblin);
    next_main(&mut t, P1);
    assert!(is_tapped(&t, goblin));
}

#[test]
fn rust_tick_untapping_doesnt_untap_the_artifact_right_away() {
    cr!("611.2a", "502.3");
    ruling!(
        "Rust Tick",
        "If Rust Tick untaps or leaves the battlefield, its effect will end. This has no immediate visible effect on the affected artifact. (It doesn’t untap immediately, for example.) The artifact will just untap as normal during its controller’s next untap step."
    );
    supported("Rust Tick");
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let tick = t.battlefield(P0, "Rust Tick");
        let myr = t.battlefield(P1, "Ornithopter");
        t.lands(P0, "Wastes", 1);
        activate(&mut t, P0, tick, "Tap target artifact", &[obj(myr)]);
        t.resolve_all();
        assert!(is_tapped(&t, myr));
        // Locked while the Tick stays tapped.
        next_main(&mut t, P1);
        assert!(is_tapped(&t, myr));
        if leaves {
            destroy(&mut t, tick);
        } else {
            untap(&mut t, tick);
        }
        assert!(is_tapped(&t, myr), "leaves={leaves}");
        next_main(&mut t, P1);
        assert!(!is_tapped(&t, myr), "leaves={leaves}");
    }
}

#[test]
fn artifact_tappers_can_target_a_tapped_artifact() {
    cr!("115.1", "701.26a");
    ruling!("Phyrexian Gremlins", "Can target a tapped artifact.");
    ruling!("Relic Barrier", "Can target a tapped artifact.");
    for name in ["Phyrexian Gremlins", "Relic Barrier"] {
        supported(name);
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, name);
        let thopter = t.battlefield(P1, "Ornithopter");
        tap(&mut t, thopter);
        assert!(
            ability_targets(&mut t, src, 0).contains(&obj(thopter)),
            "{name}"
        );
        activate(&mut t, P0, src, "Tap target artifact", &[obj(thopter)]);
        t.resolve_all();
        assert!(is_tapped(&t, thopter), "{name}");
    }
    // Phyrexian Gremlins' lock still applies to an artifact that was already tapped.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Phyrexian Gremlins");
    let thopter = t.battlefield(P1, "Ornithopter");
    tap(&mut t, thopter);
    activate(&mut t, P0, g, "Tap target artifact", &[obj(thopter)]);
    t.resolve_all();
    next_main(&mut t, P1);
    assert!(is_tapped(&t, thopter));
}

// --- Willow Satyr -----------------------------------------------------------------------------

/// P0's Willow Satyr is activated targeting `target`; the ability is left on the stack.
fn satyr(t: &mut TestGame, target: ObjectId) -> ObjectId {
    let satyr = t.battlefield(P0, "Willow Satyr");
    activate(t, P0, satyr, "Gain control", &[obj(target)]);
    satyr
}

#[test]
fn willow_satyr_leaving_the_battlefield_ends_the_control_change() {
    cr!("611.2a", "108.4");
    ruling!(
        "Willow Satyr",
        "If Willow Satyr leaves the battlefield, you no longer control it, so the duration of its control-change effect ends."
    );
    supported("Willow Satyr");
    let mut t = TestGame::new(2);
    let ayumi = t.battlefield(P1, "Ayumi, the Last Visitor");
    let s = satyr(&mut t, ayumi);
    t.resolve_all();
    assert_eq!(controller(&t, ayumi), P0);
    destroy(&mut t, s);
    assert_eq!(controller(&t, ayumi), P1);
}

#[test]
fn willow_satyr_untapped_or_stolen_before_resolution_gains_nothing() {
    cr!("611.2b");
    interp!("for-as-long-as-checked-continually-from-activation");
    ruling!(
        "Willow Satyr",
        "If Willow Satyr stops being tapped before its ability resolves — even if it becomes tapped again right away — you won’t gain control of the targeted creature at all. The same is true if you lose control of Willow Satyr before the ability resolves."
    );
    // Untapped (and tapped again) in response.
    let mut t = TestGame::new(2);
    let ayumi = t.battlefield(P1, "Ayumi, the Last Visitor");
    let s = satyr(&mut t, ayumi);
    untap(&mut t, s);
    tap(&mut t, s);
    t.resolve_all();
    assert_eq!(controller(&t, ayumi), P1);
    // Control of the Satyr lost in response — and also when it's regained before the
    // ability resolves: the duration ended before the effect began (CR 611.2b).
    for regain in [false, true] {
        let mut t = TestGame::new(2);
        let ayumi = t.battlefield(P1, "Ayumi, the Last Visitor");
        let s = satyr(&mut t, ayumi);
        give_control(&mut t, s, P1);
        if regain {
            give_control(&mut t, s, P0);
        }
        t.resolve_all();
        assert_eq!(controller(&t, ayumi), P1, "regain={regain}");
    }
}

#[test]
fn willow_satyr_effect_doesnt_care_whether_the_permanent_stays_a_legal_target() {
    cr!("611.2c", "608.2b");
    ruling!(
        "Willow Satyr",
        "Once the ability resolves, it doesn’t care whether the targeted permanent remains a legal target for the ability, only that it’s on the battlefield. The effect will continue to apply to it even if it gains shroud, stops being a creature, stops being legendary, or would no longer be a legal target for any other reason."
    );
    let mut t = TestGame::new(2);
    let ayumi = t.battlefield(P1, "Ayumi, the Last Visitor");
    satyr(&mut t, ayumi);
    t.resolve_all();
    assert_eq!(controller(&t, ayumi), P0);
    // It gains shroud (Lightning Greaves), then stops being a creature (Song of the
    // Dryads: "Enchanted permanent is a colorless Forest land.").
    attach_new(&mut t, P1, "Lightning Greaves", obj(ayumi));
    t.g.recompute();
    assert!(crate::r_s06_common::has_kw(
        &t,
        ayumi,
        mtg_engine::keywords::KeywordKind::Shroud
    ));
    attach_new(&mut t, P1, "Song of the Dryads", obj(ayumi));
    t.g.recompute();
    assert!(!t.obj_now(ayumi).is(mtg_engine::types::CardType::Creature));
    assert_eq!(controller(&t, ayumi), P0);
}

#[test]
fn olivia_voldaren_control_lost_and_regained_before_resolution_gains_nothing() {
    cr!("611.2b");
    interp!("for-as-long-as-checked-continually-from-activation");
    // "{3}{B}{B}: Gain control of target Vampire for as long as you control Olivia
    // Voldaren." Control of Olivia lost and regained while the ability is on the stack.
    supported("Olivia Voldaren");
    for regain in [false, true] {
        let mut t = TestGame::new(2);
        let olivia = t.battlefield(P0, "Olivia Voldaren");
        let child = t.battlefield(P1, "Child of Night");
        t.lands(P0, "Swamp", 5);
        activate(&mut t, P0, olivia, "Gain control", &[obj(child)]);
        give_control(&mut t, olivia, P1);
        if regain {
            give_control(&mut t, olivia, P0);
        }
        t.resolve_all();
        assert_eq!(controller(&t, child), P1, "regain={regain}");
    }
    // Keeping control, the Vampire is gained.
    let mut t = TestGame::new(2);
    let olivia = t.battlefield(P0, "Olivia Voldaren");
    let child = t.battlefield(P1, "Child of Night");
    t.lands(P0, "Swamp", 5);
    activate(&mut t, P0, olivia, "Gain control", &[obj(child)]);
    t.resolve_all();
    assert_eq!(controller(&t, child), P0);
}

#[test]
fn holy_justiciar_taps_then_exiles_an_untapped_zombie() {
    cr!("608.2c", "701.26a");
    ruling!(
        "Holy Justiciar",
        "If the creature is an untapped Zombie, it will become tapped, then exiled."
    );
    supported("Holy Justiciar");
    let mut t = TestGame::new(2);
    let j = t.battlefield(P0, "Holy Justiciar");
    let zombie = t.battlefield(P1, "Walking Corpse");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    activate(&mut t, P0, j, "Tap target creature", &[obj(zombie)]);
    t.resolve_all();
    assert!(t.in_exile("Walking Corpse"));
    // It became tapped first.
    assert!(t.g.turn_events.iter().chain(t.g.events.iter()).any(|e| matches!(
        e,
        mtg_engine::events::Event::Tapped { obj, .. } if *obj == zombie
    )));
    // A creature that isn't a Zombie is just tapped.
    untap(&mut t, j);
    t.lands(P0, "Plains", 3);
    activate(&mut t, P0, j, "Tap target creature", &[obj(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears) && is_tapped(&t, bears));
}
