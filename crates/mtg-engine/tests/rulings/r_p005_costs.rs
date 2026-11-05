//! Rulings batch P005 — costs, mana abilities and the order of casting: triggered mana
//! abilities (CR 605.1b, 605.4a) and abilities that aren't mana abilities (CR 605.5a),
//! generic cost reductions (CR 118.7a) applied to an alternative cost (CR 601.2f), targets
//! chosen before costs are paid (CR 601.2c, 601.2h), sacrificing for mana while paying a
//! cost (CR 605.3a, 601.2h), and "can't cast additional spells" restrictions.

use crate::r_p006_common::{in_graveyard, obj, pl, priority_since};
use crate::r_s01_common::{supported, watch};
use crate::r_s02_common::{can_activate, can_cast, create_token, target_candidates};
use crate::r_s20_common::tap_for_mana;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::game::Game;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn badgermole_cubs_extra_mana_doesnt_use_the_stack() {
    cr!("605.1b", "605.4a");
    ruling!(
        "Badgermole Cub",
        "Badgermole Cub's last ability is a triggered mana ability. It doesn't use the stack and can't be responded to."
    );
    supported("Badgermole Cub");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Badgermole Cub");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let from = t.asked().len();
    assert!(tap_for_mana(&mut t, P0, elves, "{G}"));
    // Both mana are in the pool at once: the trigger resolved immediately, without
    // going on the stack and without anyone getting priority.
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::G), 2);
    assert!(t.g.stack.is_empty());
    t.settle();
    assert!(t.g.stack.is_empty());
    assert!(priority_since(&t, from).is_empty());
}

#[test]
fn molten_core_maestros_mana_trigger_uses_the_stack() {
    cr!("605.5a", "603.3");
    ruling!(
        "Molten-Core Maestro",
        "Molten-Core Maestro's last ability is not a mana ability. It uses the stack and can be responded to."
    );
    supported("Molten-Core Maestro");
    supported("Lava Axe");
    let mut t = TestGame::new(2);
    let maestro = t.battlefield(P0, "Molten-Core Maestro");
    t.lands(P0, "Mountain", 5);
    let axe = t.hand(P0, "Lava Axe");
    let axe = t.cast(P0, axe).target(pl(P1)).go();
    t.settle();
    // The opus trigger is on the stack above Lava Axe; no mana has been added yet.
    assert_eq!(t.g.stack.len(), 2);
    assert_ne!(*t.g.stack.last().unwrap(), axe);
    assert!(t.g.player(P0).mana_pool.is_empty());
    // A response: a +1/+1 counter is put on the Maestro while the trigger waits. The
    // trigger then adds mana equal to its power as it resolves (2 + 1 + its own 1 = 4).
    t.g.add_counters(obj(maestro), counters::PLUS1, 1, None);
    t.g.recompute();
    t.resolve();
    assert_eq!(t.counters(maestro, counters::PLUS1), 2);
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::R), 4);
    assert_eq!(t.g.stack, vec![axe]);
}

#[test]
fn sculptor_of_winter_isnt_a_mana_ability() {
    cr!("605.5a", "605.3a", "601.2g");
    ruling!(
        "Sculptor of Winter",
        "Sculptor of Winter’s activated ability is not a mana ability. It can’t be activated while casting a spell or activating an ability. It uses the stack and can be responded to."
    );
    supported("Sculptor of Winter");
    let mut t = TestGame::new(2);
    let sculptor = t.battlefield(P0, "Sculptor of Winter");
    let land = t.battlefield(P0, "Snow-Covered Forest");
    t.g.tap(land);
    let growth = t.hand(P0, "Giant Growth");
    // While paying for Giant Growth, the Sculptor can't untap the land.
    assert!(!can_cast(&mut t, P0, growth, CastMethod::Normal));
    assert!(t.cast(P0, growth).target(sculptor).try_go().is_err());
    assert!(t.obj_now(land).tapped && !t.obj_now(sculptor).tapped);
    // With priority, it's activated: it goes on the stack and the land stays tapped until
    // it resolves.
    t.activate(P0, sculptor, 0, &[obj(land)]).unwrap();
    assert_eq!(t.g.stack.len(), 1);
    assert!(t.obj_now(land).tapped);
    t.resolve();
    assert!(!t.obj_now(land).tapped);
    let growth = t.g.current(growth);
    t.cast(P0, growth).target(sculptor).go();
    t.resolve_all();
    assert_eq!(t.pt(sculptor), (5, 5));
}

#[test]
fn ethersworn_canonist_allows_one_nonartifact_spell_and_any_artifact_spells() {
    cr!("601.2e");
    ruling!(
        "Ethersworn Canonist",
        "Each turn, each player can cast any number of artifact spells plus a maximum of one nonartifact spell."
    );
    supported("Ethersworn Canonist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ethersworn Canonist");
    t.lands(P0, "Mountain", 2);
    // Artifact spells before the nonartifact one don't count.
    for _ in 0..2 {
        let o = t.hand(P0, "Ornithopter");
        t.cast(P0, o).go();
        t.resolve_all();
    }
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(pl(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // A second nonartifact spell can't be cast...
    let shock = t.hand(P0, "Shock");
    assert!(!can_cast(&mut t, P0, shock, CastMethod::Normal));
    assert!(t.cast(P0, shock).target(pl(P1)).try_go().is_err());
    // ...but more artifact spells can.
    for _ in 0..2 {
        let o = t.hand(P0, "Ornithopter");
        t.cast(P0, o).go();
        t.resolve_all();
    }
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 4);
    // The other player has their own nonartifact spell.
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let theirs = t.hand(P1, "Shock");
    t.cast(P1, theirs).target(pl(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
}

#[test]
fn hero_of_iroas_reduces_only_generic_mana() {
    cr!("118.7a", "601.2f");
    ruling!(
        "Hero of Iroas",
        "Hero of Iroas's first ability reduces only generic mana in the spell's total cost."
    );
    supported("Hero of Iroas");
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, "Hero of Iroas");
    t.lands(P0, "Plains", 1);
    // Pacifism ({1}{W}) costs {W}.
    let bears = t.battlefield(P1, "Grizzly Bears");
    let pacifism = t.hand(P0, "Pacifism");
    t.cast(P0, pacifism).target(bears).go();
    t.resolve_all();
    assert!(t.on_battlefield(pacifism));
    // Holy Strength ({W}) still costs {W}: with no mana left it can't be cast.
    let strength = t.hand(P0, "Holy Strength");
    assert!(!can_cast(&mut t, P0, strength, CastMethod::Normal));
    assert!(t.cast(P0, strength).target(hero).try_go().is_err());
}

#[test]
fn hero_of_iroas_reduces_a_bestow_cost() {
    cr!("118.7a", "601.2f", "702.103a", "702.103b");
    ruling!(
        "Hero of Iroas",
        "Hero of Iroas's first ability will apply if you cast a card for its bestow cost."
    );
    supported("Boon Satyr");
    const BESTOW: CastMethod = CastMethod::Keyword(KeywordKind::Bestow);
    // Boon Satyr's bestow cost {3}{G}{G} costs {2}{G}{G}: four lands are enough with the
    // Hero, and not without it.
    for hero in [true, false] {
        let mut t = TestGame::new(2);
        if hero {
            t.battlefield(P0, "Hero of Iroas");
        }
        t.lands(P0, "Forest", 2);
        t.lands(P0, "Wastes", 2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let satyr = t.hand(P0, "Boon Satyr");
        assert_eq!(can_cast(&mut t, P0, satyr, BESTOW), hero);
        let r = t.cast(P0, satyr).target(bears).method(BESTOW).try_go();
        assert_eq!(r.is_ok(), hero);
        if hero {
            t.resolve_all();
            // Bestowed: an Aura on the Bears (2/2 + 4/2).
            assert_eq!(t.pt(bears), (6, 4));
        }
    }
}

#[test]
fn cauldron_of_essence_cant_target_the_creature_it_sacrifices() {
    cr!("602.2b", "601.2c", "601.2h");
    ruling!(
        "Cauldron of Essence",
        "Because targets are chosen before costs are paid, the target of Cauldron of Essence's last ability can't be the creature sacrificed to pay its cost."
    );
    supported("Cauldron of Essence");
    // No creature card in the graveyard: the ability can't be activated, even though
    // sacrificing the Bears would put one there.
    let mut t = TestGame::new(2);
    let cauldron = t.battlefield(P0, "Cauldron of Essence");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(!can_activate(&mut t, P0, cauldron));
    assert!(t.activate(P0, cauldron, 0, &[obj(bears)]).is_err());
    assert!(t.on_battlefield(bears));
    // With a Hill Giant card there, it's the only possible target; the Bears are then
    // sacrificed and stay in the graveyard.
    let giant = t.graveyard(P0, "Hill Giant");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, cauldron, 0, &[obj(giant)]).unwrap();
    assert_eq!(target_candidates(&t, P0, from), vec![vec![obj(giant)]]);
    assert!(in_graveyard(&t, bears));
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert!(in_graveyard(&t, bears));
}

#[test]
fn barrage_of_expendables_is_the_source_of_the_damage() {
    cr!("113.7a", "702.2b", "702.15b");
    ruling!(
        "Barrage of Expendables",
        "Barrage of Expendables deals the damage, not the sacrificed creature. Notably, that creature having deathtouch or lifelink provides no additional benefit."
    );
    supported("Barrage of Expendables");
    let mut t = TestGame::new(2);
    let barrage = t.battlefield(P0, "Barrage of Expendables");
    t.lands(P0, "Mountain", 1);
    // Vampire Nighthawk has deathtouch and lifelink.
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[obj(hawk)]);
    t.activate(P0, barrage, 0, &[obj(giant)]).unwrap();
    assert!(in_graveyard(&t, hawk));
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 1);
    assert!(t.on_battlefield(giant), "no deathtouch");
    assert_eq!(t.life(P0), 20, "no lifelink");
}

#[test]
fn barrage_of_expendables_can_target_the_creature_it_sacrifices() {
    cr!("602.2b", "601.2c", "601.2h", "608.2b");
    ruling!(
        "Barrage of Expendables",
        "Because targets are chosen before costs are paid, it is possible to choose a creature as the target of the ability and then sacrifice that creature. The ability won't resolve when it tries to resolve and no damage will be dealt."
    );
    let mut t = TestGame::new(2);
    let barrage = t.battlefield(P0, "Barrage of Expendables");
    t.lands(P0, "Mountain", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, barrage, 0, &[obj(bears)]).unwrap();
    assert!(in_graveyard(&t, bears));
    assert_eq!(t.g.stack.len(), 1);
    let ability = *t.g.stack.last().unwrap();
    let from = t.g.log.len();
    t.resolve_all();
    assert!(!t.g.stack.contains(&ability));
    assert!(
        t.g.log[from..].iter().all(|l| !l.text.contains("damage")),
        "{}",
        t.dump_log()
    );
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
}

#[test]
fn makeshift_munitions_cant_sacrifice_one_permanent_twice() {
    cr!("605.3a", "601.2h", "701.21a");
    ruling!(
        "Makeshift Munitions",
        "If an artifact or creature has an ability allowing you to sacrifice it for mana, you can't sacrifice that permanent to generate mana and also to pay the sacrifice cost for the activated ability of Makeshift Munitions."
    );
    supported("Makeshift Munitions");
    // A Treasure is P0's only mana and only artifact: it can pay {1} or be the sacrifice,
    // not both.
    let mut t = TestGame::new(2);
    let munitions = t.battlefield(P0, "Makeshift Munitions");
    let treasure = create_token(&mut t, P0, "Treasure");
    assert!(t.activate(P0, munitions, 0, &[pl(P1)]).is_err());
    assert!(t.on_battlefield(treasure));
    assert_eq!(t.life(P1), 20);
    // With an Ornithopter to sacrifice, the Treasure's mana pays {1}.
    let orni = t.battlefield(P0, "Ornithopter");
    t.answer_choose(P0, &[obj(orni)]);
    t.activate(P0, munitions, 0, &[pl(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert!(!t.on_battlefield(treasure) && !t.on_battlefield(orni));
}

/// Whether the Grizzly Bears are on the battlefield (as P1 is asked for priority).
fn bears_on_battlefield(g: &Game) -> bool {
    g.battlefield
        .iter()
        .any(|id| g.obj(*id).chars.name == "Grizzly Bears")
}

#[test]
fn collateral_damage_can_be_responded_to_only_after_its_sacrifice() {
    cr!("601.2h", "601.2i");
    ruling!(
        "Collateral Damage",
        "Players can respond to Collateral Damage only after it’s been cast and all its costs have been paid. No one can try to destroy the creature you sacrificed to stop you from casting this spell."
    );
    supported("Collateral Damage");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let spell = t.hand(P0, "Collateral Damage");
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::Priority { .. }),
        bears_on_battlefield,
    );
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: spell,
            method: CastMethod::Normal,
        }),
    );
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_targets(P0, &[pl(P1)]);
    let ok = t.g.run_until(1_000, |g| g.player(P1).life < 20);
    assert!(ok, "{}", t.dump_log());
    assert!(in_graveyard(&t, bears));
    // P1 had priority while the spell was on the stack, and never while the Bears were
    // still on the battlefield.
    let seen = seen.lock().unwrap();
    assert!(!seen.is_empty());
    assert!(seen.iter().all(|on| !on), "{seen:?}");
}
