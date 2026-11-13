//! CR 508.1c–d / 509.1b–c audit on large boards: the maximum number of requirements
//! obeyed without breaking restrictions is the maximum over all declarations, however
//! many creatures and attack targets there are. A board too large to enumerate must not
//! make an empty (or otherwise non-maximal) declaration legal.

use crate::r506_audit_common::*;
use crate::r506_common::{declare, go_to};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 controls `n` Grizzly Bears; P1 controls Angler Turtle ("Creatures your opponents
/// control attack each combat if able"), Silent Arbiter ("No more than one creature can
/// attack each combat") and a planeswalker (so each Bear may attack two things).
fn turtle_and_arbiter(n: usize) -> (TestGame, Vec<ObjectId>) {
    supported("Angler Turtle");
    supported("Silent Arbiter");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Angler Turtle");
    t.battlefield(P1, "Silent Arbiter");
    t.battlefield(P1, "Jace Beleren");
    let bears = (0..n)
        .map(|_| t.battlefield(P0, "Grizzly Bears"))
        .collect();
    t.set_step(P0, Step::BeginningOfCombat);
    (t, bears)
}

#[test]
fn with_a_maximum_of_one_attacker_exactly_one_of_many_creatures_that_must_attack_attacks() {
    cr!("508.1c", "508.1d");
    interp!("requirement-search-budget");
    for n in [3usize, 11, 14] {
        let (mut t, _) = turtle_and_arbiter(n);
        // Not attacking obeys none of the n requirements; one Bear attacking obeys one.
        declare(&mut t, &[]);
        go_to(&mut t, Step::DeclareAttackers);
        assert_eq!(attackers(&t).len(), 1, "{n} Bears");
    }
}

#[test]
fn with_a_maximum_of_one_attacker_two_declared_attackers_are_undone() {
    cr!("508.1c", "508.1d");
    let (mut t, bears) = turtle_and_arbiter(12);
    declare(
        &mut t,
        &[(bears[0], Entity::Player(P1)), (bears[1], Entity::Player(P1))],
    );
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(attackers(&t).len(), 1);
}

#[test]
fn many_creatures_that_must_attack_and_one_that_cant_attack_alone_all_attack() {
    cr!("508.1c", "508.1d");
    // Mogg Flunkies plus 11 Bears, all required to attack (Angler Turtle), against a
    // player with a planeswalker: every creature attacks.
    supported("Mogg Flunkies");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Angler Turtle");
    t.battlefield(P1, "Jace Beleren");
    let mut mine = vec![t.battlefield(P0, "Mogg Flunkies")];
    mine.extend((0..11).map(|_| t.battlefield(P0, "Grizzly Bears")));
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[]);
    go_to(&mut t, Step::DeclareAttackers);
    let mut a = attackers(&t);
    a.sort();
    mine.sort();
    assert_eq!(a, mine);
}

#[test]
fn many_blockers_that_must_block_and_a_maximum_of_one_blocker() {
    cr!("509.1b", "509.1c");
    interp!("requirement-search-budget");
    // Grand Melee: all creatures block each combat if able (and attack: the attacker
    // attacks anyway). Silent Arbiter: no more than one creature can block each combat.
    supported("Grand Melee");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grand Melee");
    t.battlefield(P0, "Silent Arbiter");
    for _ in 0..3 {
        t.battlefield(P0, "Grizzly Bears");
    }
    for _ in 0..18 {
        t.battlefield(P1, "Grizzly Bears");
    }
    t.set_step(P0, Step::BeginningOfCombat);
    // The Arbiter attacks too (Grand Melee): it's a 1/5.
    go_to(&mut t, Step::DeclareBlockers);
    assert_eq!(attackers(&t).len(), 1);
    let blockers = t.g.blockers();
    assert_eq!(blockers.len(), 1, "exactly one creature blocks");
}

#[test]
fn many_goaded_creatures_split_their_attacks_under_crawlspace() {
    cr!("508.1c", "508.1d", "701.15b", "802.3a");
    interp!("requirement-search-budget");
    supported("Crawlspace");
    // Three players: P1 goaded P0's twelve Bears; P2 controls Crawlspace ("No more than
    // two creatures can attack you each combat"). Every Bear attacks (each obeys "attacks
    // each combat if able"); two of them attack P2 (obeying "a player other than P1"),
    // the rest P1.
    let mut t = TestGame::with_config(
        3,
        mtg_engine::game::GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    );
    t.battlefield(P2, "Crawlspace");
    let bears: Vec<ObjectId> = (0..12)
        .map(|_| t.battlefield(P0, "Grizzly Bears"))
        .collect();
    for b in &bears {
        t.g.objects[b.0 as usize].goaded_by = vec![P1];
    }
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[]);
    go_to(&mut t, Step::DeclareAttackers);
    let at = |p: PlayerId| {
        bears
            .iter()
            .filter(|b| attacks(&t, **b) == Some(Entity::Player(p)))
            .count()
    };
    assert_eq!((at(P1), at(P2)), (10, 2));
}
