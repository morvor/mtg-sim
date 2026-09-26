//! "They" / "those creatures" in "one or more" triggers (patterns in
//! `src/oracle/patterns/pronoun_groups.rs`): the objects of the batch of events the
//! ability triggered on ("Whenever one or more creatures you control attack, they gain
//! indestructible until end of turn.").

use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

fn has(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(k)
}

#[test]
fn they_are_the_creatures_that_attacked() {
    cr!("603.2c", "508.1", "611.2c");
    assert_supported(&["Angelic Guardian"]);
    let mut t = TestGame::new(2);
    let guardian = t.battlefield(P0, "Angelic Guardian");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![
            (bears, Entity::Player(P1)),
            (giant, Entity::Player(P1)),
        ]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.settle();
    // One trigger for the whole attack.
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert!(has(&t, bears, KeywordKind::Indestructible));
    assert!(has(&t, giant, KeywordKind::Indestructible));
    assert!(!has(&t, guardian, KeywordKind::Indestructible));
    assert!(!has(&t, theirs, KeywordKind::Indestructible));
}

#[test]
fn those_creatures_are_the_ones_attacking_you() {
    cr!("603.2c", "508.1");
    assert_supported(&["Sabotage Strategist"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sabotage Strategist");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let home = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.attack(
        &[(bears, Entity::Player(P0)), (giant, Entity::Player(P0))],
        &[],
    );
    assert_eq!(t.pt(bears), (1, 2));
    assert_eq!(t.pt(giant), (2, 3));
    assert_eq!(t.pt(home), (2, 2));
    assert_eq!(t.life(P0), 17);
}

#[test]
fn each_of_those_creatures_is_one_that_dealt_combat_damage() {
    cr!("603.2c", "510.2", "122.1a");
    assert_supported(&["Vulture, Feathered Fiend"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vulture, Feathered Fiend");
    let drake = t.battlefield(P0, "Wind Drake");
    let other = t.battlefield(P0, "Wind Drake");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.attack(
        &[
            (drake, Entity::Player(P1)),
            (other, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
        ],
        &[],
    );
    assert_eq!(t.counters(drake, "+1/+1"), 1);
    assert_eq!(t.counters(other, "+1/+1"), 1);
    // Without flying: not one of those creatures.
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    // One trigger for the player dealt damage.
    assert_eq!(t.hand_size(P0), hand + 1);
}
