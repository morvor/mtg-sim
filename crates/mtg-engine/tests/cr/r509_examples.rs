//! The worked examples of CR 509 (the declare blockers step), played out as tests. The
//! 509.1b and 509.1c examples are cited by tests in `r509_declare_blockers.rs`.

use crate::r506_common::*;
use mtg_engine::ability::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// A 0/5 attacker (so combat damage kills no blocker) with the example's ability.
fn pale_hunter() -> CardDef {
    custom_card(
        "Pale Hunter",
        "Creature — Test",
        Some((0, 5)),
        "Whenever this creature becomes blocked by a white creature, destroy that creature at end of combat.",
    )
}

#[test]
fn blocked_by_a_black_creature_later_turned_white_doesnt_trigger() {
    cr!("509.3f", "603.2");
    cr_example!("509.3f", "becomes blocked by a white creature");
    let mut t = TestGame::new(2);
    let a = bf(&mut t, P0, pale_hunter());
    let black = t.battlefield(P1, "Walking Corpse");
    declare(&mut t, &[(a, Entity::Player(P1))]);
    block(&mut t, P1, &[(black, a)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert_eq!(t.stack_len(), 0);
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
    t.resolve_all();
    go_to(&mut t, Step::PostcombatMain);
    assert!(t.on_battlefield(black), "{}", t.dump_log());

    // Blocked by a white creature: it triggers, and the blocker is destroyed at end of
    // combat.
    let mut t = TestGame::new(2);
    let a = bf(&mut t, P0, pale_hunter());
    let white = t.battlefield(P1, "Benalish Hero");
    declare(&mut t, &[(a, Entity::Player(P1))]);
    block(&mut t, P1, &[(white, a)]);
    go_to(&mut t, Step::DeclareBlockers);
    t.resolve_all();
    go_to(&mut t, Step::CombatDamage);
    assert!(t.on_battlefield(white));
    go_to(&mut t, Step::EndOfCombat);
    t.resolve_all();
    assert!(!t.on_battlefield(white), "{}", t.dump_log());
    assert!(t.in_graveyard(P1, "Benalish Hero"));
}
