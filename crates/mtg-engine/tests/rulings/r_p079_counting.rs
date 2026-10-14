//! Rulings batch P079 — counting attacking creatures (or your creatures) for a spell's
//! cost or effect: all attacking creatures whoever controls them and whatever they attack
//! (CR 506.4, 508.1k), counted as the spell resolves (CR 608.2h) or as the cost is
//! determined (CR 601.2f); and X = 0 for "divided as you choose" spells (CR 601.2d).

use crate::r_p079_common::*;
use crate::r_s01_common::{attack_with, creatures, supported, tokens};
use mtg_engine::combat::remove_from_combat;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 attacks with `n` Grizzly Bears: the first at P1, the rest at P1's planeswalker (or
/// at P1 when `pw` is false). Returns the attackers.
fn bears_attack(t: &mut TestGame, n: usize, pw: bool) -> Vec<ObjectId> {
    let jace = pw.then(|| t.battlefield(P1, "Jace Beleren"));
    let bears: Vec<ObjectId> = (0..n).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let decl: Vec<(ObjectId, Entity)> = bears
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let to = match (i, jace) {
                (0, _) | (_, None) => Entity::Player(P1),
                (_, Some(j)) => obj(j),
            };
            (*b, to)
        })
        .collect();
    attack_with(t, &decl);
    t.resolve_all();
    bears
}

#[test]
fn ancient_stone_idol_counts_every_attacking_creature() {
    cr!("601.2f", "508.1k");
    ruling!(
        "Ancient Stone Idol",
        "Ancient Stone Idol's cost is reduced for each attacking creature, not just creatures attacking you. It even counts your attacking creatures."
    );
    supported("Ancient Stone Idol");
    // P0's own two attackers (one at P1, one at P1's planeswalker): {8}.
    for (lands, ok) in [(8, true), (7, false)] {
        let mut t = TestGame::new(2);
        bears_attack(&mut t, 2, true);
        t.lands(P0, "Wastes", lands);
        let idol = t.hand(P0, "Ancient Stone Idol");
        assert_eq!(t.cast(P0, idol).try_go().is_ok(), ok, "{lands} lands");
    }
    // An opponent's attackers, cast by the defending player: {7} with three.
    let mut t = TestGame::new(2);
    bears_attack(&mut t, 3, false);
    t.lands(P1, "Wastes", 7);
    let idol = t.hand(P1, "Ancient Stone Idol");
    assert!(t.cast(P1, idol).try_go().is_ok());
}

#[test]
fn cleaving_skyrider_counts_other_players_attackers() {
    cr!("608.2h", "508.1k");
    ruling!(
        "Cleaving Skyrider",
        "Cleaving Skyrider's triggered ability counts all attacking creatures, even attacking creatures other players control."
    );
    supported("Cleaving Skyrider");
    // P1 flashes in a kicked Skyrider during P0's attack with three creatures: 3 damage.
    let mut t = TestGame::new(2);
    bears_attack(&mut t, 3, true);
    crate::r_s01_common::give_mana_for(&mut t, P1, "Cleaving Skyrider");
    t.lands(P1, "Mountain", 1);
    t.lands(P1, "Wastes", 2);
    let sky = t.hand(P1, "Cleaving Skyrider");
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.cast(P1, sky).kicked(true).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
}

#[test]
fn flurry_of_wings_counts_attackers_as_it_resolves() {
    cr!("608.2h", "508.4", "506.4");
    ruling!(
        "Flurry of Wings",
        "Flurry of Wings counts the number of creatures that are attacking as it resolves. It doesn’t matter how many creatures were originally declared as attackers that turn."
    );
    supported("Flurry of Wings");
    // Three declared (two at P1's planeswalker); one removed from combat, two put onto
    // the battlefield attacking: four.
    let mut t = TestGame::new(2);
    let bears = bears_attack(&mut t, 3, true);
    remove_from_combat(&mut t.g, bears[0]);
    enter_attacking(&mut t, P0, "Savannah Lions", Entity::Player(P1));
    enter_attacking(&mut t, P0, "Savannah Lions", Entity::Player(P1));
    cast_card(&mut t, P1, "Flurry of Wings", &[]).unwrap();
    t.resolve_all();
    assert_eq!(tokens(&t, P1).len(), 4);
    // The caster's own attackers count too.
    let mut t = TestGame::new(2);
    bears_attack(&mut t, 2, false);
    cast_card(&mut t, P0, "Flurry of Wings", &[]).unwrap();
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
}

#[test]
fn flurry_of_wings_does_nothing_outside_combat() {
    cr!("506.4", "511.3");
    ruling!(
        "Flurry of Wings",
        "If Flurry of Wings is cast before attackers have been declared or after combat ends, it won’t do anything."
    );
    for step in [Step::BeginningOfCombat, Step::PostcombatMain] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        if step == Step::PostcombatMain {
            t.attack(&[(bears, Entity::Player(P1))], &[]);
            crate::r_s21_common::go_to(&mut t, step);
            assert_eq!(t.life(P1), 18);
        } else {
            t.set_step(P0, step);
        }
        cast_card(&mut t, P0, "Flurry of Wings", &[]).unwrap();
        t.resolve_all();
        assert!(tokens(&t, P0).is_empty(), "{step:?}");
    }
}

#[test]
fn galadhrim_ambush_counts_all_attackers_whatever_they_attack() {
    cr!("608.2h", "506.4", "508.1k");
    ruling!(
        "Galadhrim Ambush",
        "Galadhrim Ambush counts all attacking creatures no matter which player, planeswalker, or battle they're attacking, including attacking creatures you control"
    );
    supported("Galadhrim Ambush");
    // P0 casts it during their own combat: two at P1, one at P1's planeswalker.
    let mut t = TestGame::new(2);
    bears_attack(&mut t, 3, true);
    cast_card(&mut t, P0, "Galadhrim Ambush", &[]).unwrap();
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 3);
    // And P1, against P0's attackers at P1 and P1's planeswalker.
    let mut t = TestGame::new(2);
    bears_attack(&mut t, 2, true);
    cast_card(&mut t, P1, "Galadhrim Ambush", &[]).unwrap();
    t.resolve_all();
    assert_eq!(tokens(&t, P1).len(), 2);
}

#[test]
fn outflank_counts_your_creatures_as_it_resolves() {
    cr!("608.2h");
    ruling!(
        "Outflank",
        "Count the number of creatures you control as Outflank resolves to determine how much damage Outflank deals."
    );
    supported("Outflank");
    // P1 controls one creature when casting Outflank at P0's attacking Hill Giant, then
    // gets two more before it resolves: 3 damage kills the Giant.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    cast_card(&mut t, P1, "Outflank", &[obj(giant)]).unwrap();
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    assert_eq!(creatures(&t, P1).len(), 3);
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    // With only the one creature, 1 damage.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    cast_card(&mut t, P1, "Outflank", &[obj(giant)]).unwrap();
    t.resolve_all();
    assert_eq!(crate::r_s07_common::damage_on(&t, giant), 1);
}

#[test]
fn divided_damage_with_x_zero_has_no_targets() {
    cr!("601.2d", "601.2c", "115.1a");
    ruling!(
        "Rolling Thunder",
        "If X is 0, you can’t choose any targets."
    );
    ruling!("Rock Slide", "If X is 0, you can’t choose any targets.");
    ruling!(
        "Infernal Harvest",
        "If X is 0, you can’t choose any targets."
    );
    for name in ["Rolling Thunder", "Rock Slide", "Infernal Harvest"] {
        supported(name);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let giant = t.battlefield(P0, "Hill Giant");
        attack_with(&mut t, &[(giant, Entity::Player(P1))]);
        // Sorceries are cast in P0's main phase; Rock Slide at the attacking Giant.
        let target = if name == "Rock Slide" { giant } else { bears };
        if name != "Rock Slide" {
            t.g.combat = None;
            t.set_step(P0, Step::PrecombatMain);
        }
        crate::r_s01_common::give_mana_for(&mut t, P0, name);
        let card = t.hand(P0, name);
        t.answer_targets(P0, &[obj(target)]);
        let spell = t.cast(P0, card).x(0).try_go();
        let targets: Vec<Entity> = match spell {
            Ok(s) => t
                .obj_now(s)
                .stack
                .as_ref()
                .unwrap()
                .chosen
                .iter()
                .flat_map(|m| m.targets.iter().flatten().copied())
                .collect(),
            Err(e) => panic!("{name}: {e:?}"),
        };
        assert!(targets.is_empty(), "{name}: {targets:?}");
        t.resolve_all();
        assert_eq!(crate::r_s07_common::damage_on(&t, target), 0, "{name}");
    }
}
