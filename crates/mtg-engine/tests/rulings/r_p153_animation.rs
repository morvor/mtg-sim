//! Rulings batch P153 — "scales with mana value": permanents animated with power and
//! toughness equal to their mana value. An Equipment that becomes a creature becomes
//! unattached (CR 704.5n) and can't equip (CR 301.5c); a later effect setting base power
//! and toughness overrides an earlier one (CR 613.4, layer 7b timestamps); a type-changing
//! effect's applicability can depend on another (CR 613.8); an animated permanent's
//! controller needs to have controlled it continuously since their turn began to attack
//! with it or activate its {T} abilities (CR 302.6).

use crate::r_p153_common::*;
use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s02_common::{can_activate, can_attack, destroy};
use crate::r_s04_common::is_creature;
use crate::r_s06_common::{activate_containing, attach_new, attached_to, give_control};
use mtg_engine::mana::ManaType::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Activates the equip ability of `equipment` targeting `creature`.
fn equip(t: &mut TestGame, p: PlayerId, equipment: ObjectId, creature: ObjectId) {
    t.answer_targets(p, &[obj(creature)]);
    activate_containing(t, p, equipment, "Equip").expect("equip couldn't be activated");
    t.resolve_all();
}

#[test]
fn sydri_animated_equipment_becomes_unattached_and_cant_equip() {
    cr!("704.5n", "301.5c");
    ruling!(
        "Sydri, Galvanic Genius",
        "If an Equipment attached to a creature becomes a creature, it becomes unattached. An Equipment that is also a creature can’t equip another creature."
    );
    supported("Sydri, Galvanic Genius");
    let mut t = TestGame::new(2);
    let sydri = t.battlefield(P0, "Sydri, Galvanic Genius");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let axe = attach_new(&mut t, P0, "Bonesplitter", bears);
    assert_eq!(t.pt(bears), (4, 2));
    mana(&mut t, P0, U, 1);
    t.answer_targets(P0, &[obj(axe)]);
    activate_containing(&mut t, P0, sydri, "becomes an artifact creature").unwrap();
    t.resolve_all();
    assert!(is_creature(&t, axe));
    assert_eq!(t.pt(axe), (1, 1));
    assert_eq!(attached_to(&t, axe), None);
    assert_eq!(t.pt(bears), (2, 2));
    // Equip resolves but attaches nothing.
    mana(&mut t, P0, C, 1);
    equip(&mut t, P0, axe, bears);
    assert_eq!(attached_to(&t, axe), None);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn march_of_the_machines_animated_equipment_becomes_unattached_and_cant_equip() {
    cr!("704.5n", "301.5c");
    ruling!(
        "March of the Machines",
        "If an Equipment becomes a creature, it can no longer equip a creature. If it's currently attached to a creature, it becomes unattached (but remains on the battlefield). You can activate the Equipment's equip ability, but it won't do anything."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let axe = attach_new(&mut t, P0, "Bonesplitter", bears);
    t.battlefield(P0, "March of the Machines");
    t.settle();
    assert!(is_creature(&t, axe));
    assert!(t.on_battlefield(axe));
    assert_eq!(attached_to(&t, axe), None);
    assert_eq!(t.pt(bears), (2, 2));
    mana(&mut t, P0, C, 1);
    assert!(can_activate(&mut t, P0, axe));
    equip(&mut t, P0, axe, bears);
    assert_eq!(attached_to(&t, axe), None);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn bludgeon_brawls_effect_stops_applying_to_an_artifact_that_becomes_a_creature() {
    cr!("613.8", "704.5n", "613.5");
    ruling!(
        "Bludgeon Brawl",
        "If an artifact affected by Bludgeon Brawl later becomes a creature, Bludgeon Brawl’s effect no longer applies to it. That artifact will stop being an Equipment and become unattached from any creature it was attached to; it will remain on the battlefield. The same thing if Bludgeon Brawl leaves the battlefield."
    );
    supported("Bludgeon Brawl");
    // Sol Ring (mana value 1) is an Equipment with equip {1} and "+1/+0".
    let mut t = TestGame::new(2);
    let brawl = t.battlefield(P0, "Bludgeon Brawl");
    let ring = t.battlefield(P0, "Sol Ring");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.obj_now(ring).chars.has_subtype("Equipment"));
    mana(&mut t, P0, C, 1);
    equip(&mut t, P0, ring, bears);
    assert_eq!(attached_to(&t, ring), Some(obj(bears)));
    assert_eq!(t.pt(bears), (3, 2));
    // Sydri animates the Ring: no longer an Equipment, unattached, still there.
    let sydri = t.battlefield(P0, "Sydri, Galvanic Genius");
    mana(&mut t, P0, U, 1);
    t.answer_targets(P0, &[obj(ring)]);
    activate_containing(&mut t, P0, sydri, "becomes an artifact creature").unwrap();
    t.resolve_all();
    assert!(is_creature(&t, ring));
    assert!(!t.obj_now(ring).chars.has_subtype("Equipment"));
    assert_eq!(attached_to(&t, ring), None);
    assert!(t.on_battlefield(ring));
    assert_eq!(t.pt(bears), (2, 2));
    // Bludgeon Brawl leaving: the same.
    let mut t = TestGame::new(2);
    let brawl2 = t.battlefield(P0, "Bludgeon Brawl");
    let ring = t.battlefield(P0, "Sol Ring");
    let bears = t.battlefield(P0, "Grizzly Bears");
    mana(&mut t, P0, C, 1);
    equip(&mut t, P0, ring, bears);
    assert_eq!(t.pt(bears), (3, 2));
    destroy(&mut t, brawl2);
    assert!(!t.obj_now(ring).chars.has_subtype("Equipment"));
    assert_eq!(attached_to(&t, ring), None);
    assert!(t.on_battlefield(ring));
    assert_eq!(t.pt(bears), (2, 2));
    let _ = brawl;
}

#[test]
fn chimeric_staffs_later_animation_overrides_march_of_the_machines() {
    cr!("613.4", "613.7");
    ruling!(
        "March of the Machines",
        "If you activate Chimeric Staff's ability and choose X = 5, Chimeric Staff will be a 5/5 artifact creature for the rest of the turn."
    );
    supported("Chimeric Staff");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "March of the Machines");
    let staff = t.battlefield(P0, "Chimeric Staff");
    assert!(is_creature(&t, staff));
    assert_eq!(t.pt(staff), (4, 4));
    mana(&mut t, P0, C, 5);
    answer_x(&mut t, P0, 5);
    activate_containing(&mut t, P0, staff, "becomes an X/X").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(staff), (5, 5));
    assert!(t.obj_now(staff).chars.has_subtype("Construct"));
}

#[test]
fn opalescence_doesnt_animate_itself_but_another_opalescence_does() {
    cr!("611.3a", "613.4");
    ruling!(
        "Opalescence",
        "Does not animate itself. But can be animated by another Opalescence."
    );
    supported("Opalescence");
    let mut t = TestGame::new(2);
    let op1 = t.battlefield(P0, "Opalescence");
    assert!(!is_creature(&t, op1));
    let op2 = t.battlefield(P0, "Opalescence");
    assert!(is_creature(&t, op1));
    assert!(is_creature(&t, op2));
    assert_eq!(t.pt(op1), (4, 4));
    assert_eq!(t.pt(op2), (4, 4));
}

#[test]
fn an_animated_deaths_oasis_sacrificed_to_its_own_ability_triggers_itself() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Death's Oasis",
        "If Death's Oasis becomes a creature, activating its last ability will cause its first ability to trigger."
    );
    supported("Death's Oasis");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Opalescence");
    let oasis = t.battlefield(P0, "Death's Oasis");
    assert!(is_creature(&t, oasis));
    assert_eq!(t.pt(oasis), (3, 3));
    t.battlefield(P0, "Hill Giant");
    let bears = t.graveyard(P0, "Grizzly Bears");
    crate::r_s01_common::stack_library(&mut t, P0, &["Forest", "Forest"]);
    mana(&mut t, P0, C, 1);
    activate_containing(&mut t, P0, oasis, "gain life").unwrap();
    assert_eq!(t.zone(oasis), Zone::Graveyard(P0));
    t.settle();
    assert_eq!(triggers_on_stack(&t, "mill two"), 1, "{}", t.dump_log());
    t.answer_choose(P0, &[obj(bears)]);
    t.resolve_all();
    // The activated ability gained life for Hill Giant (4); the trigger milled two and
    // returned Grizzly Bears (2 < 3).
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.zone(bears), Zone::Hand(P0));
    assert_eq!(t.library_size(P0), 30);
}

/// Whether `id` is among the mana sources `p` could activate now.
fn is_mana_source(t: &TestGame, p: PlayerId, id: ObjectId) -> bool {
    mtg_engine::mana_abilities::mana_sources(&t.g, p, None)
        .iter()
        .any(|s| s.obj == t.g.current(id))
}

#[test]
fn an_animated_cactus_preserve_not_controlled_since_the_turn_began_cant_tap_or_attack() {
    cr!("302.6", "605.3a");
    ruling!(
        "Cactus Preserve",
        "If Cactus Preserve becomes a creature but you haven’t controlled it continuously since your most recent turn began, you won’t be able to activate its mana ability or attack with it that turn."
    );
    supported("Cactus Preserve");
    for stolen in [true, false] {
        let mut t = TestGame::new(2);
        let owner = if stolen { P1 } else { P0 };
        let preserve = t.battlefield(owner, "Cactus Preserve");
        t.g.objects[preserve.0 as usize].tapped = false;
        // (With no commander it would be a 0/0: two +1/+1 counters keep it alive.)
        t.g.objects[preserve.0 as usize]
            .counters
            .insert("+1/+1".into(), 2);
        t.battlefield(P0, "Forest");
        if stolen {
            give_control(&mut t, preserve, P0);
        }
        // Not a creature: its mana ability can be activated whatever the control history.
        assert!(is_mana_source(&t, P0, preserve), "stolen: {stolen}");
        mana(&mut t, P0, C, 3);
        activate_containing(&mut t, P0, preserve, "becomes an X/X").unwrap();
        t.resolve_all();
        assert!(is_creature(&t, preserve), "stolen: {stolen}");
        assert_eq!(is_mana_source(&t, P0, preserve), !stolen, "stolen: {stolen}");
        t.set_step(P0, Step::DeclareAttackers);
        assert_eq!(can_attack(&mut t, preserve), !stolen, "stolen: {stolen}");
    }
}

#[test]
fn cactus_preserve_ignores_the_restrictions_of_the_mana_it_copies() {
    cr!("605.1a", "106.6");
    ruling!(
        "Cactus Preserve",
        "It just produces one mana of the appropriate type, with no restrictions or riders."
    );
    supported("Ancient Ziggurat");
    let mut t = TestGame::new(2);
    let preserve = t.battlefield(P0, "Cactus Preserve");
    t.g.objects[preserve.0 as usize].tapped = false;
    // Ancient Ziggurat: "Add one mana of any color. Spend this mana only to cast a creature
    // spell." Cactus Preserve's red mana pays for Lightning Bolt.
    t.battlefield(P0, "Ancient Ziggurat");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert!(t.obj_now(preserve).tapped, "{}", t.dump_log());
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn cactus_preserve_checks_what_lands_could_produce_not_their_costs_or_legality() {
    cr!("605.1a");
    ruling!(
        "Cactus Preserve",
        "It doesn’t matter whether you control an artifact, whether you can pay 1 life, or whether Spire of Industry is untapped."
    );
    supported("Spire of Industry");
    let mut t = TestGame::new(2);
    let preserve = t.battlefield(P0, "Cactus Preserve");
    t.g.objects[preserve.0 as usize].tapped = false;
    // No artifact, Spire tapped: Cactus Preserve still taps for green.
    let spire = t.battlefield(P0, "Spire of Industry");
    t.g.objects[spire.0 as usize].tapped = true;
    t.g.recompute();
    let growth = t.hand(P0, "Giant Growth");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.cast(P0, growth).target(bears).go();
    assert!(t.obj_now(preserve).tapped, "{}", t.dump_log());
    t.resolve();
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(t.life(P0), 20);
}
