//! Cover of Winter: "If a creature would deal combat damage to you and/or one or more
//! creatures you control, prevent X of that damage, where X is the number of age counters
//! on this enchantment." X of each creature's damage; when one creature's damage to two or
//! more of those recipients is more than X, Cover of Winter's controller divides X among
//! them (register entry `per-source-prevention-division`). The division is asked once for
//! each such creature, among that creature's recipients only; nothing is asked when there's
//! nothing to divide; and an answer that isn't a division of X, at most each recipient's
//! damage, prevents X in the order the damage is replaced.

use crate::r614_audit_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0's Cover of Winter with `x` age counters.
fn cover_of_winter(t: &mut TestGame, x: u32) {
    supported("Cover of Winter");
    let cover = t.battlefield(P0, "Cover of Winter");
    if x > 0 {
        t.g.add_counters(obj(cover), "age", x, None);
    }
}

/// The recipients of each division of X that P0 was asked for.
fn divisions(t: &TestGame) -> Vec<Vec<Entity>> {
    t.asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::Divide { recipients, .. } if p == P0 => Some(recipients),
            _ => None,
        })
        .collect()
}

#[test]
fn cover_of_winter_asks_for_no_division_when_there_is_nothing_to_divide() {
    cr!("615.1a", "510.2", "702.19b");
    interp!("per-source-prevention-division");
    // One recipient: 2 of the unblocked Giant's 3 damage is prevented, with no question.
    let mut t = TestGame::new(2);
    cover_of_winter(&mut t, 2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    t.attack(&[(giant, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 19);
    assert!(divisions(&t).is_empty(), "{:?}", divisions(&t));
    // X = 0 (no age counter yet): the Dreadmaw's 3 damage to the Wall and 3 to P0 are dealt.
    let mut t = TestGame::new(2);
    cover_of_winter(&mut t, 0);
    let wurm = t.battlefield(P1, "Colossal Dreadmaw");
    let wall = t.battlefield(P0, "Wall of Wood");
    t.set_step(P1, Step::PrecombatMain);
    t.attack(&[(wurm, Entity::Player(P0))], &[(wall, wurm)]);
    assert!(t.in_graveyard(P0, "Wall of Wood"));
    assert_eq!(t.life(P0), 17);
    assert!(divisions(&t).is_empty(), "{:?}", divisions(&t));
    // X = 6, all the Dreadmaw deals: all of it is prevented.
    let mut t = TestGame::new(2);
    cover_of_winter(&mut t, 6);
    let wurm = t.battlefield(P1, "Colossal Dreadmaw");
    let wall = t.battlefield(P0, "Wall of Wood");
    t.set_step(P1, Step::PrecombatMain);
    t.attack(&[(wurm, Entity::Player(P0))], &[(wall, wurm)]);
    assert_eq!(damage_on(&t, wall), 0);
    assert_eq!(t.life(P0), 20);
    assert!(divisions(&t).is_empty(), "{:?}", divisions(&t));
}

#[test]
fn cover_of_winter_divides_x_among_a_tramplers_two_blockers_and_you() {
    cr!("615.1a", "510.2", "702.19b");
    interp!("per-source-prevention-division");
    ruling!(
        "Cover of Winter",
        "You may prevent damage that would be dealt to the first creature, the second creature, and/or you, as long as the total amount of that creature's damage that would be prevented is X."
    );
    let mut t = TestGame::new(2);
    cover_of_winter(&mut t, 3);
    let wurm = t.battlefield(P1, "Colossal Dreadmaw");
    let wall = t.battlefield(P0, "Wall of Wood");
    let memnite = t.battlefield(P0, "Memnite");
    t.set_step(P1, Step::PrecombatMain);
    // The Dreadmaw assigns lethal damage to each blocker and the rest to P0: 3 to the Wall,
    // 1 to Memnite, 2 to P0. P0 prevents 1 of each (in whatever order they're listed).
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1, 1]));
    t.attack(
        &[(wurm, Entity::Player(P0))],
        &[(wall, wurm), (memnite, wurm)],
    );
    let asked = divisions(&t);
    assert_eq!(asked.len(), 1, "{asked:?}");
    assert_eq!(asked[0].len(), 3, "{asked:?}");
    assert_eq!(damage_on(&t, wall), 2);
    assert!(t.on_battlefield(memnite));
    assert_eq!(damage_on(&t, memnite), 0);
    assert_eq!(t.life(P0), 19);
}

#[test]
fn cover_of_winter_divides_each_creatures_damage_on_its_own() {
    cr!("615.1a", "510.2", "702.19b");
    interp!("per-source-prevention-division");
    ruling!(
        "Cover of Winter",
        "This prevention is applied creature by creature"
    );
    ruling!(
        "Cover of Winter",
        "you decide how to divide the prevention effect"
    );
    let mut t = TestGame::new(2);
    cover_of_winter(&mut t, 2);
    let wurm = t.battlefield(P1, "Colossal Dreadmaw");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wall = t.battlefield(P0, "Wall of Wood");
    t.set_step(P1, Step::PrecombatMain);
    // The Dreadmaw's 3 to the Wall and 3 to P0, divided: 1 of each prevented. The unblocked
    // Bears' 2 to P0 is X of another creature's damage, all prevented, and isn't part of the
    // Dreadmaw's division.
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    t.attack(
        &[(wurm, Entity::Player(P0)), (bears, Entity::Player(P0))],
        &[(wall, wurm)],
    );
    let asked = divisions(&t);
    assert_eq!(asked.len(), 1, "{asked:?}");
    assert_eq!(asked[0].len(), 2, "{asked:?}");
    assert!(asked[0].contains(&obj(wall)) && asked[0].contains(&Entity::Player(P0)));
    assert_eq!(damage_on(&t, wall), 2);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn cover_of_winter_an_answer_that_isnt_a_division_of_x_prevents_x_in_order() {
    cr!("615.1a", "510.2", "702.19b");
    interp!("per-source-prevention-division");
    ruling!(
        "Cover of Winter",
        "as long as the total amount of that creature's damage that would be prevented is X"
    );
    // 2 + 2 is more than X; -1 isn't an amount of damage.
    for answer in [vec![2, 2], vec![-1, 3]] {
        let mut t = TestGame::new(2);
        cover_of_winter(&mut t, 2);
        let wurm = t.battlefield(P1, "Colossal Dreadmaw");
        let wall = t.battlefield(P0, "Wall of Wood");
        t.set_step(P1, Step::PrecombatMain);
        t.answer(P0, DecisionKind::Divide, Answer::Numbers(answer.clone()));
        t.attack(&[(wurm, Entity::Player(P0))], &[(wall, wurm)]);
        assert_eq!(divisions(&t).len(), 1, "{answer:?}");
        // X = 2 of the 6 is prevented, from the damage to the Wall (assigned, and replaced,
        // first): the Wall is dealt 1 and P0 3.
        assert_eq!(damage_on(&t, wall), 1, "{answer:?}");
        assert_eq!(t.life(P0), 17, "{answer:?}");
    }
}
