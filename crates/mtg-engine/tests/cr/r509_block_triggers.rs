//! CR 509.3 (block trigger conditions) and 509.4 (creatures put onto the battlefield
//! blocking).

use crate::r506_common::*;
use mtg_engine::ability::*;
use mtg_engine::combat::{become_blocked, block_by_effect};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn with_text(name: &str, pt: (i32, i32), text: &str) -> CardDef {
    custom_card(name, "Creature — Test", Some(pt), text)
}

/// P0 attacks P1 with the given creatures; returns at the declare blockers step after
/// P1's blocks and with triggers resolved.
fn fight(t: &mut TestGame, attackers: &[ObjectId], blocks: &[(ObjectId, ObjectId)]) {
    let decl: Vec<(ObjectId, Entity)> =
        attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    declare(t, &decl);
    block(t, P1, blocks);
    go_to(t, Step::DeclareBlockers);
    t.resolve_all();
}

#[test]
fn blocks_triggers_once_per_combat() {
    cr!("509.3", "509.3a");
    let mut t = TestGame::new(2);
    let a1 = t.battlefield(P0, "Grizzly Bears");
    let a2 = t.battlefield(P0, "Grizzly Bears");
    let guard = bf(
        &mut t,
        P1,
        with_text(
            "Double Guard",
            (1, 6),
            "This creature can block an additional creature each combat.\nWhenever this creature blocks, you gain 1 life.",
        ),
    );
    fight(&mut t, &[a1, a2], &[(guard, a1), (guard, a2)]);
    assert_eq!(t.g.combat.as_ref().unwrap().blocking(guard).len(), 2);
    assert_eq!(t.life(P1), 21, "once, though it blocks two creatures");
    // An effect making it block another creature doesn't trigger again: it was already
    // blocking.
    let a3 = enter_with(
        &mut t,
        P0,
        vanilla("Late", 1, 1),
        Some(Entity::Player(P1)),
        None,
    );
    assert!(block_by_effect(&mut t.g, guard, a3));
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P1), 21);
}

#[test]
fn blocks_triggers_when_an_effect_makes_a_nonblocker_block_but_not_when_entering_blocking() {
    cr!("509.3a", "509.4");
    let text = "Whenever this creature blocks, you gain 1 life.";
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let idle = bf(&mut t, P1, with_text("Idle Guard", (1, 4), text));
    fight(&mut t, &[a], &[]);
    assert!(block_by_effect(&mut t.g, idle, a));
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P1), 21);
    // Put onto the battlefield blocking: it's blocking, but it never "blocked".
    let late = enter_with(
        &mut t,
        P1,
        with_text("Late Guard", (1, 4), text),
        None,
        Some(a),
    );
    assert!(t.g.is_blocking(late));
    assert_eq!(t.g.combat.as_ref().unwrap().blocking(late), vec![a]);
    t.resolve_all();
    assert_eq!(t.life(P1), 21);
}

#[test]
fn blocks_a_creature_triggers_for_each_attacker() {
    cr!("509.3b");
    let text = "This creature can block an additional creature each combat.\nWhenever this creature blocks a creature, you gain 1 life.";
    let mut t = TestGame::new(2);
    let a1 = t.battlefield(P0, "Grizzly Bears");
    let a2 = t.battlefield(P0, "Grizzly Bears");
    let a3 = t.battlefield(P0, "Hill Giant");
    let guard = bf(&mut t, P1, with_text("Double Guard", (1, 6), text));
    fight(&mut t, &[a1, a2, a3], &[(guard, a1), (guard, a2)]);
    assert_eq!(t.life(P1), 22);
    // An effect makes it block a third attacker: triggers again (not already blocking it).
    assert!(block_by_effect(&mut t.g, guard, a3));
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P1), 23);
    // Not when a creature enters blocking.
    enter_with(
        &mut t,
        P1,
        with_text("Late Guard", (1, 4), text),
        None,
        Some(a1),
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 23);
}

#[test]
fn becomes_blocked_triggers_once_and_only_if_it_was_unblocked() {
    cr!("509.3c");
    let text = "Whenever this creature becomes blocked, you gain 1 life.";
    let mut t = TestGame::new(2);
    let knight = bf(&mut t, P0, with_text("Proud Knight", (4, 4), text));
    let other = bf(&mut t, P0, with_text("Other Knight", (2, 2), text));
    let x = t.battlefield(P1, "Grizzly Bears");
    let y = t.battlefield(P1, "Grizzly Bears");
    fight(&mut t, &[knight, other], &[(x, knight), (y, knight)]);
    assert_eq!(
        t.life(P0),
        21,
        "once for two blockers; the other knight is unblocked"
    );
    // An effect makes the unblocked knight blocked: it triggers.
    assert!(become_blocked(&mut t.g, other));
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // A creature entering the battlefield blocking an already-blocked creature: no trigger.
    enter_with(&mut t, P1, vanilla("Late Guard", 1, 1), None, Some(knight));
    t.resolve_all();
    assert_eq!(t.life(P0), 22);

    // A creature put onto the battlefield blocking an unblocked creature: it triggers.
    let mut t = TestGame::new(2);
    let knight = bf(&mut t, P0, with_text("Proud Knight", (4, 4), text));
    fight(&mut t, &[knight], &[]);
    enter_with(&mut t, P1, vanilla("Late Guard", 1, 1), None, Some(knight));
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn becomes_blocked_by_a_creature_triggers_for_each_blocker() {
    cr!("509.3d");
    // Kolaghan Aspirant: "Whenever this creature becomes blocked by a creature, this
    // creature deals 1 damage to that creature."
    let mut t = TestGame::new(2);
    let asp = t.battlefield(P0, "Kolaghan Aspirant");
    let x = t.battlefield(P1, "Craw Wurm");
    let y = t.battlefield(P1, "Hill Giant");
    fight(&mut t, &[asp], &[(x, asp), (y, asp)]);
    assert_eq!(t.obj_now(x).damage, 1);
    assert_eq!(t.obj_now(y).damage, 1);
    // An effect makes another creature block it: triggers for that creature.
    let z = t.battlefield(P1, "Serra Angel");
    assert!(block_by_effect(&mut t.g, z, asp));
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.obj_now(z).damage, 1);
    // A creature put onto the battlefield blocking it: triggers.
    let w = enter_with(&mut t, P1, vanilla("Late Guard", 1, 4), None, Some(asp));
    t.resolve_all();
    assert_eq!(t.obj_now(w).damage, 1);

    // Becoming blocked by an effect rather than by a creature doesn't trigger it.
    let mut t = TestGame::new(2);
    let asp = t.battlefield(P0, "Kolaghan Aspirant");
    let x = t.battlefield(P1, "Craw Wurm");
    fight(&mut t, &[asp], &[]);
    assert!(become_blocked(&mut t.g, asp));
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.obj_now(x).damage, 0);
}

#[test]
fn blocked_by_a_number_of_creatures() {
    cr!("509.3e");
    let text = "Whenever this creature becomes blocked by two or more creatures, you gain 1 life.";
    let mut t = TestGame::new(2);
    let a = bf(&mut t, P0, with_text("Gang Target", (5, 5), text));
    let b = bf(&mut t, P0, with_text("Lone Target", (5, 5), text));
    let x = t.battlefield(P1, "Grizzly Bears");
    let y = t.battlefield(P1, "Grizzly Bears");
    let z = t.battlefield(P1, "Grizzly Bears");
    fight(&mut t, &[a, b], &[(x, a), (y, a), (z, b)]);
    assert_eq!(t.life(P0), 21, "only the creature blocked by two triggers");
    // An effect adding a second blocker makes the other trigger.
    let w = t.battlefield(P1, "Hill Giant");
    assert!(block_by_effect(&mut t.g, w, b));
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn trigger_characteristics_are_checked_when_blocking_happens() {
    cr!("509.3f");
    // CR 509.3f example: "Whenever this creature becomes blocked by a white creature, ..."
    // blocked by a black creature later turned white doesn't trigger.
    let text = "Whenever this creature becomes blocked by a white creature, you gain 1 life.";
    let mut t = TestGame::new(2);
    let a = bf(&mut t, P0, with_text("Pale Hunter", (3, 3), text));
    let black = t.battlefield(P1, "Walking Corpse");
    fight(&mut t, &[a], &[(black, a)]);
    apply(
        &mut t,
        P1,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::SetColors(ColorSet::single(Color::White))],
            duration: Duration::EndOfTurn,
        },
        &[black],
    );
    assert!(t.obj_now(black).chars.colors.contains(Color::White));
    t.resolve_all();
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P0), 20);
    // Blocked by a white creature: triggers.
    let mut t = TestGame::new(2);
    let a = bf(&mut t, P0, with_text("Pale Hunter", (3, 3), text));
    let white = t.battlefield(P1, "Benalish Hero");
    fight(&mut t, &[a], &[(white, a)]);
    assert_eq!(t.life(P0), 21);
    // "Whenever this creature blocks" with a characteristic: checked when it blocks.
    let watcher = triggered(TriggerCond::Blocks(Filter::Color(Color::White)), gain(1));
    let mut t = TestGame::new(2);
    bf(
        &mut t,
        P1,
        custom_with("White Eye", "Enchantment", None, vec![watcher]),
    );
    let a = t.battlefield(P0, "Grizzly Bears");
    let black = t.battlefield(P1, "Walking Corpse");
    fight(&mut t, &[a], &[(black, a)]);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn attacks_and_isnt_blocked() {
    cr!("509.3g");
    let text = "Whenever this creature attacks and isn't blocked, you gain 1 life.";
    // Triggers if no creatures are declared as blockers for it, even if it entered
    // attacking.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[(a, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    let late = enter_with(
        &mut t,
        P0,
        with_text("Sly Raider", (2, 2), text),
        Some(Entity::Player(P1)),
        None,
    );
    assert!(t.g.is_attacking(late));
    go_to(&mut t, Step::DeclareBlockers);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);

    // Blocked, then its blocker is removed from combat: no trigger.
    let mut t = TestGame::new(2);
    let raider = bf(&mut t, P0, with_text("Sly Raider", (2, 2), text));
    let x = t.battlefield(P1, "Grizzly Bears");
    fight(&mut t, &[raider], &[(x, raider)]);
    mtg_engine::combat::remove_from_combat(&mut t.g, x);
    t.resolve_all();
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn creature_entering_blocking_isnt_subject_to_block_restrictions_or_requirements() {
    cr!("509.4b");
    let mut t = TestGame::new(2);
    // A flying menace attacker; P1 has a block tax; the entering creature can't block.
    let flyer = bf(
        &mut t,
        P0,
        custom_card(
            "Sky Brute",
            "Creature — Ogre",
            Some((3, 3)),
            "Flying\nMenace",
        ),
    );
    bf(
        &mut t,
        P0,
        custom_with(
            "Blockade Tax",
            "Enchantment",
            None,
            vec![restriction(Restriction::BlockCost {
                blockers: Filter::creature(),
                cost: Cost::mana(mtg_engine::mana::ManaCost::parse("{3}").unwrap()),
            })],
        ),
    );
    fight(&mut t, &[flyer], &[]);
    let pacifist = custom_card(
        "Timid Guard",
        "Creature — Human",
        Some((1, 1)),
        "This creature can't block.",
    );
    let g = enter_with(&mut t, P1, pacifist, None, Some(flyer));
    assert!(t.g.is_blocking(g));
    assert!(is_blocked(&t, flyer));
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(g));
}

#[test]
fn controller_chooses_what_a_creature_entering_blocking_blocks() {
    cr!("509.4");
    let mut t = TestGame::new(3);
    let a1 = t.battlefield(P0, "Grizzly Bears");
    let a2 = t.battlefield(P0, "Hill Giant");
    let a3 = t.battlefield(P0, "Craw Wurm");
    declare(
        &mut t,
        &[
            (a1, Entity::Player(P1)),
            (a2, Entity::Player(P1)),
            (a3, Entity::Player(P2)),
        ],
    );
    go_to(&mut t, Step::DeclareBlockers);
    // P1 chooses among the creatures attacking P1 (not the one attacking P2).
    t.answer_choose(P1, &[Entity::Object(a2)]);
    let chosen = mtg_engine::combat::choose_attacker_to_block(&mut t.g, P1).unwrap();
    assert_eq!(chosen, a2);
    let asked = t.asked();
    let cands = asked
        .iter()
        .rev()
        .find_map(|(p, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities { candidates, .. } if *p == P1 => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(cands, vec![Entity::Object(a1), Entity::Object(a2)]);
    let late = enter_with(&mut t, P1, vanilla("Late Guard", 1, 4), None, Some(chosen));
    assert_eq!(t.g.combat.as_ref().unwrap().blocking(late), vec![a2]);
}
