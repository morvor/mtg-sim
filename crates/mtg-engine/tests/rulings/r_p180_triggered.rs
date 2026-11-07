//! Rulings batch P180 — assorted triggered abilities and "attacking creatures get"
//! effects: Master Thief, Wolfkin Bond, the Invasion Lairs, Trumpet Blast, Banners Raised
//! and Nulltread Gargantuan.

use crate::r_p180_common::*;
use crate::r_s06_common::give_control;
use mtg_engine::decision::Decision;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn controller(t: &TestGame, id: ObjectId) -> PlayerId {
    t.obj_now(id).controller
}

#[test]
fn master_thiefs_control_effect_ends_when_you_stop_controlling_it() {
    cr!("611.2b");
    ruling!(
        "Master Thief",
        "If Master Thief leaves the battlefield, you no longer control it, and its control-change effect ends."
    );
    ruling!(
        "Master Thief",
        "If another player gains control of Master Thief, its control-change effect ends. Regaining control of Master Thief won’t cause you to regain control of the artifact."
    );
    supported("Master Thief");
    // It leaves the battlefield.
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P1, "Sol Ring");
    t.answer_targets(P0, &[obj(ring)]);
    let thief = t.enter(P0, "Master Thief");
    t.resolve_all();
    assert_eq!(controller(&t, ring), P0);
    destroy(&mut t, thief);
    t.g.recompute();
    assert_eq!(controller(&t, ring), P1);
    // Another player gains control of it; regaining it doesn't bring the artifact back.
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P1, "Sol Ring");
    t.answer_targets(P0, &[obj(ring)]);
    let thief = t.enter(P0, "Master Thief");
    t.resolve_all();
    assert_eq!(controller(&t, ring), P0);
    give_control(&mut t, thief, P1);
    assert_eq!(controller(&t, ring), P1);
    give_control(&mut t, thief, P0);
    assert_eq!(controller(&t, thief), P0);
    assert_eq!(controller(&t, ring), P1);
}

#[test]
fn wolfkin_bond_needs_a_legal_target_and_doesnt_enter_without_one() {
    cr!("303.4a", "608.3b");
    ruling!(
        "Wolfkin Bond",
        "If the target creature is an illegal target by the time Wolfkin Bond tries to resolve, the spell doesn't resolve."
    );
    ruling!(
        "Wolfkin Bond",
        "You need a creature for Wolfkin Bond to target as you cast it."
    );
    supported("Wolfkin Bond");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_targeting(&mut t, P0, "Wolfkin Bond", &[obj(bears)]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Wolfkin Bond"));
    assert!(t.named_on_battlefield("Wolf Token").is_empty(), "no Wolf");
    // No creature to target: it can't be cast (and so can't go onto its own Wolf).
    let mut t = TestGame::new(2);
    let bond = crate::r_s03_common::in_hand_with_mana(&mut t, P0, "Wolfkin Bond");
    t.set_step(P0, Step::PrecombatMain);
    assert!(!crate::r_s02_common::can_cast(&mut t, P0, bond, CastMethod::Normal));
    // With a creature: it enchants that creature, not the Wolf it creates.
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(crate::r_s02_common::can_cast(&mut t, P0, bond, CastMethod::Normal));
    t.cast_with(P0, bond, &[obj(bears)]).unwrap();
    t.resolve_all();
    let bond = t.named_on_battlefield("Wolfkin Bond")[0];
    assert_eq!(t.obj_now(bond).attached_to, Some(obj(bears)));
    assert_eq!(t.named_on_battlefield("Wolf Token").len(), 1);
}

#[test]
fn a_lair_can_be_tapped_for_mana_before_its_enters_ability_resolves() {
    cr!("605.3a");
    ruling!(
        "Crosis's Catacombs",
        "If you don't want to unsummon a land, you can play this card then tap it for mana before the enters ability resolves."
    );
    supported("Crosis's Catacombs");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let island = t.battlefield(P0, "Island");
    let lair = t.hand(P0, "Crosis's Catacombs");
    t.play_land(P0, lair).unwrap();
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let lair = t.g.current(lair);
    t.activate(P0, lair, 0, &[]).unwrap();
    assert!(t.obj_now(lair).tapped);
    let pool = t.g.player(P0).mana_pool.total();
    assert_eq!(pool, 1, "mana added before the ability resolves");
    // Don't return a land: sacrifice it.
    t.answer_yes(P0, false);
    t.resolve();
    assert!(t.in_graveyard(P0, "Crosis's Catacombs"));
    assert!(t.on_battlefield(island));
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
}

#[test]
fn trumpet_blast_affects_attackers_at_resolution_for_the_rest_of_the_turn() {
    cr!("611.2c");
    ruling!(
        "Trumpet Blast",
        "Even though Trumpet Blast affects only attacking creatures, the bonus will remain for the rest of the turn."
    );
    ruling!(
        "Trumpet Blast",
        "Only creatures that are attacking as Trumpet Blast resolves will receive the bonus."
    );
    supported("Trumpet Blast");
    // Cast while attacking: the bonus lasts after combat.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let home = t.battlefield(P0, "Hill Giant");
    crate::r_s01_common::attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    cast_targeting(&mut t, P0, "Trumpet Blast", &[]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    assert_eq!(t.pt(home), (3, 3));
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.pt(bears), (4, 2), "still +2/+0 after combat");
    // Cast before attackers are declared: nothing.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_targeting(&mut t, P0, "Trumpet Blast", &[]);
    t.resolve_all();
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn banners_raised_only_affects_creatures_you_control_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Banners Raised",
        "Only creatures you control when Banners Raised resolves will get the bonus."
    );
    supported("Banners Raised");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    cast_resolve(&mut t, P0, "Banners Raised", &[]);
    assert_eq!(t.pt(bears), (3, 2));
    let later = t.enter(P0, "Hill Giant");
    give_control(&mut t, theirs, P0);
    assert_eq!(t.pt(later), (3, 3));
    assert_eq!(t.pt(theirs), (3, 3));
}

#[test]
fn nulltread_gargantuan_chooses_on_resolution_and_can_tuck_itself() {
    cr!("115.1", "608.2c");
    ruling!(
        "Nulltread Gargantuan",
        "If you control no other creatures, you’ll have to put Nulltread Gargantuan on top of its owner’s library."
    );
    ruling!(
        "Nulltread Gargantuan",
        "Nulltread Gargantuan’s ability doesn’t target the creature you put on top of its owner’s library."
    );
    supported("Nulltread Gargantuan");
    // Alone: it goes on top of its owner's library.
    let mut t = TestGame::new(2);
    let g = t.enter(P0, "Nulltread Gargantuan");
    t.resolve_all();
    let g = t.g.current(g);
    assert_eq!(t.zone(g), Zone::Library(P0));
    assert_eq!(t.g.library_top(P0), Some(g));
    // It doesn't target: a creature with shroud can be chosen, as the ability resolves.
    let mut t = TestGame::new(2);
    let shroud = t.battlefield(P0, "Argothian Enchantress");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(shroud)]);
    let g = t.enter(P0, "Nulltread Gargantuan");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert!(
        !t.asked()[from..]
            .iter()
            .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })),
        "no target chosen"
    );
    t.resolve_all();
    let shroud = t.g.current(shroud);
    assert_eq!(t.g.library_top(P0), Some(shroud));
    assert!(t.on_battlefield(g));
}
