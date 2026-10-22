//! CR 701.35: detain.

use crate::a701_028_071_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_detained_permanent_cant_attack_block_or_activate_until_your_next_turn() {
    cr!("701.35a");
    ruling!(
        "Lyev Skyknight",
        "No one can activate any activated abilities, including mana abilities, of a detained permanent."
    );
    supported("Lyev Decree");
    // "Detain up to two target creatures your opponents control."
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P1, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let free = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 2);
    let decree = t.hand(P0, "Lyev Decree");
    t.cast(P0, decree)
        .targets(&[Entity::Object(elves), Entity::Object(bears)])
        .go();
    t.resolve_all();
    // During P1's turn: they can't attack, and the Elves' mana ability can't be activated.
    t.advance_to(P1, Step::PrecombatMain);
    assert!(!t.g.can_attack(bears));
    assert!(!t.g.can_attack(elves));
    assert!(t.g.can_attack(free));
    assert!(t.activate(P1, elves, 0, &[]).is_err());
    assert_eq!(t.g.player(P1).mana_pool.total(), 0);
    // During P0's next turn it ends: they can block again.
    t.advance_to(P0, Step::Upkeep);
    assert!(t.g.can_block_at_all(bears));
    assert!(t.g.can_block_at_all(elves));
    // And in P1's following turn they can attack and tap for mana.
    t.advance_to(P1, Step::PrecombatMain);
    assert!(t.g.can_attack(bears));
    assert!(t.activate(P1, elves, 0, &[]).is_ok());
}

#[test]
fn a_detained_noncreature_permanent_that_becomes_a_creature_cant_block() {
    cr!("701.35a");
    ruling!(
        "Lyev Skyknight",
        "If a noncreature permanent is detained and later turns into a creature, it won't be able to attack or block."
    );
    supported("Lyev Skyknight");
    // "When this creature enters, detain target nonland permanent an opponent controls."
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P1, "Mind Stone");
    t.answer_targets(P0, &[Entity::Object(stone)]);
    t.enter(P0, "Lyev Skyknight");
    t.resolve_all();
    // It becomes a creature later: it still can't block.
    run(
        &mut t,
        P1,
        None,
        mtg_engine::ability::Effect::Modify {
            what: mtg_engine::ability::Sel::Target(0),
            mods: vec![
                mtg_engine::ability::Modification::AddTypes(vec![CardType::Creature]),
                mtg_engine::ability::Modification::SetPT(
                    Some(mtg_engine::ability::Value::c(2)),
                    Some(mtg_engine::ability::Value::c(2)),
                ),
            ],
            duration: mtg_engine::ability::Duration::EndOfTurn,
        },
        &[Entity::Object(stone)],
    );
    assert!(t.obj(stone).is_creature());
    assert!(!t.g.can_block_at_all(stone));
    // Nor can its mana ability be activated.
    assert!(t.activate(P1, stone, 0, &[]).is_err());
}
