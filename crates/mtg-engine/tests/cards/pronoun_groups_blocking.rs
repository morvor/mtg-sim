//! "Creatures blocking it" where "it" is the object the ability is about (patterns in
//! `src/oracle/patterns/pronoun_groups.rs`): the source ("Whenever ~ becomes blocked, it
//! deals 1 damage to each creature blocking it"), the creature a trigger is about
//! ("Whenever a creature you control becomes blocked, it gets +3/+3 until end of turn for
//! each creature blocking it"), or the equipped creature.

use mtg_engine::decision::Answer;
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// Declares attackers and blockers and advances to the declare blockers step, where the
/// "becomes blocked" triggers wait on the stack.
fn to_blocks(t: &mut TestGame, attackers: &[(ObjectId, Entity)], blocks: &[(ObjectId, ObjectId)]) {
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(attackers.to_vec()),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(blocks.to_vec()),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.settle();
}

#[test]
fn it_gets_bigger_for_each_creature_blocking_it() {
    cr!("509.3c", "608.2h", "603.2e");
    ruling!(
        "General Marhault Elsdragon",
        "The ability triggers only once, no matter how many creatures are blocking it. The number of creatures blocking it is determined as the ability resolves."
    );
    assert_supported(&["General Marhault Elsdragon", "Berserk Murlodont"]);
    for remove_one in [false, true] {
        let mut t = TestGame::new(2);
        let general = t.battlefield(P0, "General Marhault Elsdragon");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let wall = t.battlefield(P1, "Wall of Wood");
        let giant = t.battlefield(P1, "Hill Giant");
        to_blocks(
            &mut t,
            &[(bears, Entity::Player(P1))],
            &[(wall, bears), (giant, bears)],
        );
        // One trigger for the bears, however many creatures block them.
        assert_eq!(t.stack_len(), 1);
        if remove_one {
            t.g.move_object(wall, Zone::Graveyard(P1), MoveCause::Effect, None);
            t.g.flush_events();
        }
        t.resolve();
        let n = if remove_one { 1 } else { 2 };
        assert_eq!(t.pt(bears), (2 + 3 * n, 2 + 3 * n), "removed: {remove_one}");
        // "It" is the blocked creature, not the General.
        assert_eq!(t.pt(general), (4, 4));
    }
}

#[test]
fn it_deals_damage_to_each_creature_blocking_it() {
    cr!("509.3c", "120.3e");
    assert_supported(&["Battle-Scarred Goblin", "Fire Juggler"]);
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P0, "Battle-Scarred Goblin");
    let wall = t.battlefield(P1, "Wall of Wood");
    let giant = t.battlefield(P1, "Hill Giant");
    let bystander = t.battlefield(P1, "Grizzly Bears");
    to_blocks(
        &mut t,
        &[(goblin, Entity::Player(P1))],
        &[(wall, goblin), (giant, goblin)],
    );
    t.resolve();
    assert_eq!(t.obj_now(wall).damage, 1);
    assert_eq!(t.obj_now(giant).damage, 1);
    assert_eq!(t.obj_now(bystander).damage, 0);
}

#[test]
fn equipped_creature_deals_damage_to_each_creature_blocking_it() {
    cr!("509.3c", "301.5a");
    let mut t = TestGame::new(2);
    let torch = t.battlefield(P0, "Trailblazer's Torch");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    t.g.attach(torch, Entity::Object(bears));
    let wall = t.battlefield(P1, "Wall of Wood");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    to_blocks(
        &mut t,
        &[(bears, Entity::Player(P1)), (other, Entity::Player(P1))],
        &[(wall, bears), (elves, bears), (giant, other)],
    );
    t.resolve_all();
    // The bears deal the damage, to the creatures blocking them only.
    assert_eq!(t.obj_now(wall).damage, 2);
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    assert_eq!(t.obj_now(giant).damage, 0);
}

#[test]
fn it_deals_damage_to_target_creature_blocking_it() {
    cr!("509.3c", "115.1d");
    assert_supported(&["Goblin Javelineer", "Flowstone Salamander", "Godo's Irregulars"]);
    let mut t = TestGame::new(2);
    let javelineer = t.battlefield(P0, "Goblin Javelineer");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.answer_targets(P0, &[Entity::Object(elves)]);
    to_blocks(&mut t, &[(javelineer, Entity::Player(P1))], &[(elves, javelineer)]);
    t.resolve();
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
}
