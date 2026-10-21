//! Rulings batch P126 — Equipment that attaches itself with a triggered ability ("quick
//! equip"): targets and new objects (CR 608.2b, 400.7), non-targeting attachment
//! (CR 115.1, 701.3), Equipment that can't be attached (CR 301.5c, 701.3b), control
//! changes and delayed triggers (CR 603.7), and "whenever equipped creature attacks"
//! after the attack (CR 508.3a).

use crate::r_s01_common::{attack_with, supported};
use crate::r_s05_common::{enter, move_to};
use crate::r_s06_common::{attach_new, attached_to, give_control};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[test]
fn ogre_geargrabber_attaches_after_attacks_trigger_window() {
    cr!("508.3a", "603.2", "301.5a");
    ruling!(
        "Ogre Geargrabber",
        "Any ability of the targeted Equipment that triggers “whenever equipped creature attacks” won’t trigger."
    );
    supported("Ogre Geargrabber");
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P0, "Ogre Geargrabber");
    let sword = t.battlefield(P1, "Sword of the Animist");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_targets(P0, &[obj(sword)]);
    attack_with(&mut t, &[(ogre, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.obj_now(sword).controller, P0);
    assert_eq!(attached_to(&t, sword), Some(obj(ogre)));
    assert_eq!(t.stack_len(), 0, "the Sword's attack trigger didn't trigger");
}

#[test]
fn ogre_geargrabber_gains_control_even_if_it_cant_attach() {
    cr!("301.5c", "701.3b", "610.3");
    ruling!(
        "Ogre Geargrabber",
        "If the targeted Equipment can’t be attached to Ogre Geargrabber for some reason, you’ll still gain control of it until end of turn."
    );
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P0, "Ogre Geargrabber");
    // March of the Machines makes the Bonesplitter an Equipment creature (without
    // reconfigure): it can't equip a creature.
    t.battlefield(P1, "March of the Machines");
    let sash = t.battlefield(P1, "Bonesplitter");
    t.g.recompute();
    assert!(t.obj_now(sash).is(CardType::Creature));
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_targets(P0, &[obj(sash)]);
    attack_with(&mut t, &[(ogre, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.obj_now(sash).controller, P0);
    assert_eq!(attached_to(&t, sash), None);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(sash).controller, P1);
}

#[test]
fn ogre_geargrabber_unattaches_whenever_control_is_lost() {
    cr!("603.7c", "701.3d");
    ruling!(
        "Ogre Geargrabber",
        "triggers the next time you lose control of the Equipment for any reason, not just when Ogre Geargrabber’s control-changing effect ends. The Equipment will become unattached from whatever creature it happens to be attached to at that time. It doesn’t matter if Ogre Geargrabber is still on the battlefield at this time."
    );
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P0, "Ogre Geargrabber");
    let sword = t.battlefield(P1, "Bonesplitter");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_targets(P0, &[obj(sword)]);
    attack_with(&mut t, &[(ogre, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(attached_to(&t, sword), Some(obj(ogre)));
    // Ogre Geargrabber leaves; the Equipment is moved to another creature, then another
    // effect gives control of it back to P1 during the turn.
    let bears = t.battlefield(P0, "Grizzly Bears");
    move_to(&mut t, ogre, Zone::Graveyard(P0));
    assert!(t.g.attach(t.g.current(sword), obj(bears)));
    t.g.recompute();
    give_control(&mut t, sword, P1);
    t.resolve_all();
    assert_eq!(t.obj_now(sword).controller, P1);
    assert_eq!(attached_to(&t, sword), None);
}

#[test]
fn cranial_plating_counts_itself() {
    cr!("301.5", "613.4c");
    ruling!(
        "Cranial Plating",
        "Cranial Plating is counted by its own ability, so it gives at least +1/+0."
    );
    supported("Cranial Plating");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Cranial Plating", bears);
    assert_eq!(t.pt(bears), (3, 2));
}

/// The Equipment `eq` is attached to a Grizzly Bears; the creature `name` enters and the
/// Equipment's trigger is on the stack; the entering creature leaves before it resolves.
fn entering_creature_leaves_first(eq_name: &str, name: &str) {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let eq = attach_new(&mut t, P0, eq_name, bears);
    let c = enter(&mut t, P0, name);
    assert_eq!(t.stack_len(), 1, "{eq_name} triggered");
    move_to(&mut t, c, Zone::Graveyard(P0));
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(attached_to(&t, eq), Some(obj(bears)), "{eq_name}");
    // Unattached: it stays unattached.
    let mut t = TestGame::new(2);
    let eq = t.battlefield(P0, eq_name);
    let c = enter(&mut t, P0, name);
    move_to(&mut t, c, Zone::Graveyard(P0));
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(attached_to(&t, eq), None, "{eq_name}");
}

#[test]
fn equipment_stays_put_if_the_entering_creature_left() {
    cr!("400.7", "608.2c", "701.3b");
    ruling!(
        "Inventor's Goggles",
        "If the Artificer that entered the battlefield leaves the battlefield before Inventor's Goggles becomes attached, Inventor's Goggles stays as it was. If it's already attached to another creature, it remains attached to that creature."
    );
    ruling!(
        "Hero's Blade",
        "If the legendary creature that entered the battlefield leaves the battlefield before Hero's Blade becomes attached, Hero's Blade stays as it was. If it's already attached to another creature, it remains attached to that creature."
    );
    ruling!(
        "Sai of the Shinobi",
        "or if you can't (most likely because the creature has left the battlefield by the time the triggered ability resolves), Sai of the Shinobi doesn't move."
    );
    supported("Inventor's Goggles");
    supported("Hero's Blade");
    supported("Sai of the Shinobi");
    entering_creature_leaves_first("Inventor's Goggles", "Etherium Sculptor");
    entering_creature_leaves_first("Hero's Blade", "Isamaru, Hound of Konda");
    entering_creature_leaves_first("Sai of the Shinobi", "Hill Giant");
}

#[test]
fn sai_of_the_shinobi_may_choose_not_to_attach() {
    cr!("603.5", "701.3");
    ruling!(
        "Sai of the Shinobi",
        "If you choose not to attach Sai of the Shinobi to a creature that enters the battlefield"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sai = attach_new(&mut t, P0, "Sai of the Shinobi", bears);
    enter(&mut t, P0, "Hill Giant");
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(attached_to(&t, sai), Some(obj(bears)));
}

#[test]
fn ronin_warclub_attaches_to_an_untargetable_creature() {
    cr!("115.1", "702.11b", "701.3");
    ruling!(
        "Ronin Warclub",
        "The triggered ability can attach Ronin Warclub to an untargetable creature because the ability doesn't target."
    );
    supported("Ronin Warclub");
    let mut t = TestGame::new(2);
    let club = t.battlefield(P0, "Ronin Warclub");
    let scout = enter(&mut t, P0, "Gladecover Scout");
    t.resolve_all();
    assert_eq!(attached_to(&t, club), Some(obj(scout)));
}

/// Draws a first and a second card for P0 this turn; Mantle of Tides triggers.
fn draw_two(t: &mut TestGame) {
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1, "Mantle of Tides triggered");
}

#[test]
fn mantle_of_tides_target_and_illegal_target() {
    cr!("608.2b", "115.1");
    ruling!(
        "Mantle of Tides",
        "If the target creature is an illegal target by the time Mantle of Tides's triggered ability tries to resolve, the ability won't resolve. Mantle of Tides remains attached to what it was attached to or remains unattached if it wasn't attached."
    );
    ruling!(
        "Mantle of Tides",
        "The triggered ability of Mantle of Tides can target the creature it's already attached to."
    );
    supported("Mantle of Tides");
    // Targeting the creature it's attached to.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let mantle = attach_new(&mut t, P0, "Mantle of Tides", bears);
    t.answer_targets(P0, &[obj(bears)]);
    draw_two(&mut t);
    t.resolve_all();
    assert_eq!(attached_to(&t, mantle), Some(obj(bears)));
    // The target leaves: it stays attached (or unattached).
    for attached in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P0, "Hill Giant");
        let mantle = if attached {
            attach_new(&mut t, P0, "Mantle of Tides", bears)
        } else {
            t.battlefield(P0, "Mantle of Tides")
        };
        t.answer_targets(P0, &[obj(giant)]);
        draw_two(&mut t);
        move_to(&mut t, giant, Zone::Graveyard(P0));
        t.resolve_all();
        let expect = if attached { Some(obj(bears)) } else { None };
        assert_eq!(attached_to(&t, mantle), expect);
    }
}

#[test]
fn heros_blade_and_gods_that_arent_creatures() {
    cr!("704.5n", "700.5", "301.5c");
    ruling!(
        "Hero's Blade",
        "The triggered ability will trigger when one of the Gods from Theros block enters the battlefield only if your devotion is high enough that it's a creature when it enters. If Hero's Blade is attached to a God that stops being a creature (or any creature that stops being a creature), it will become unattached."
    );
    // Low devotion: Purphoros isn't a creature as it enters; no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hero's Blade");
    enter(&mut t, P0, "Purphoros, God of the Forge");
    assert_eq!(t.stack_len(), 0);
    // Devotion to red 5 (Purphoros 1, Hill Giant 1, Shivan Dragon 2, Hill Giant 1).
    let mut t = TestGame::new(2);
    let blade = t.battlefield(P0, "Hero's Blade");
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Hill Giant");
    let dragon = t.battlefield(P0, "Shivan Dragon");
    let god = enter(&mut t, P0, "Purphoros, God of the Forge");
    assert_eq!(t.stack_len(), 1, "a creature God entered");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(attached_to(&t, blade), Some(obj(god)));
    // The devotion drops: it stops being a creature, and the Blade becomes unattached.
    move_to(&mut t, dragon, Zone::Graveyard(P0));
    t.g.recompute();
    t.settle();
    assert!(!t.obj_now(god).is(CardType::Creature));
    assert_eq!(attached_to(&t, blade), None);
}
