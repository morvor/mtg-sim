//! Players' attacks in intervening "if" clauses (CR 508.6, 603.4): Norn's Decree,
//! Mirkwood Trapper, Ever-Watching Threshold, Undercellar Sweep, Jin Sakai.

use crate::combat2_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn multi(n: usize) -> TestGame {
    TestGame::with_config(
        n,
        GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    )
}

#[test]
fn norns_decree_draws_when_a_poisoned_player_is_attacked() {
    cr!("508.6", "603.4", "122.1f");
    supported("Norn's Decree");
    let mut t = multi(3);
    t.battlefield(P2, "Norn's Decree");
    let a = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Player(P1), "poison", 1, None);
    let hand = t.hand_size(P0);
    to_combat(&mut t, P0);
    declare(&mut t, P0, &[(a, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn norns_decree_doesnt_draw_when_no_attacked_player_is_poisoned() {
    cr!("603.4");
    let mut t = multi(3);
    t.battlefield(P2, "Norn's Decree");
    let a = t.battlefield(P0, "Grizzly Bears");
    // The poisoned player isn't attacked.
    t.g.add_counters(Entity::Player(P2), "poison", 1, None);
    let hand = t.hand_size(P0);
    to_combat(&mut t, P0);
    declare(&mut t, P0, &[(a, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn mirkwood_trapper_pumps_only_when_its_controller_isnt_attacked() {
    cr!("508.6", "603.4");
    supported("Mirkwood Trapper");
    let mut t = multi(3);
    t.battlefield(P2, "Mirkwood Trapper");
    let a = t.battlefield(P0, "Grizzly Bears");
    to_combat(&mut t, P0);
    t.answer_choose(P0, &[Entity::Object(a)]);
    declare(&mut t, P0, &[(a, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(a), (4, 2));
    // Attacking the Trapper's controller: no pump (its other ability gives -2/-0).
    let mut t = multi(3);
    t.battlefield(P2, "Mirkwood Trapper");
    let a = t.battlefield(P0, "Grizzly Bears");
    to_combat(&mut t, P0);
    t.answer_targets(P2, &[Entity::Object(a)]);
    t.answer_choose(P0, &[Entity::Object(a)]);
    declare(&mut t, P0, &[(a, Entity::Player(P2))]);
    t.resolve_all();
    assert_eq!(t.pt(a), (0, 2));
}

#[test]
fn ever_watching_threshold_draws_when_attacked_or_a_planeswalker_is() {
    cr!("508.6", "603.4");
    supported("Ever-Watching Threshold");
    for target_pw in [false, true] {
        let mut t = multi(3);
        t.battlefield(P2, "Ever-Watching Threshold");
        let pw = t.battlefield(P2, "Jace Beleren");
        let a = t.battlefield(P0, "Grizzly Bears");
        let hand = t.hand_size(P2);
        to_combat(&mut t, P0);
        let target = if target_pw {
            Entity::Object(pw)
        } else {
            Entity::Player(P2)
        };
        declare(&mut t, P0, &[(a, target)]);
        t.resolve_all();
        assert_eq!(t.hand_size(P2), hand + 1);
    }
    // Attacking someone else: no draw.
    let mut t = multi(3);
    t.battlefield(P2, "Ever-Watching Threshold");
    let a = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P2);
    to_combat(&mut t, P0);
    declare(&mut t, P0, &[(a, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.hand_size(P2), hand);
}

fn soldiers(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.has_subtype("Soldier"))
        .count()
}

#[test]
fn undercellar_sweep_creates_soldiers_when_the_attacked_player_has_the_initiative() {
    cr!("508.6", "603.4", "725.1");
    supported("Undercellar Sweep");
    let mut t = multi(3);
    t.battlefield(P0, "Undercellar Sweep");
    // P1 takes the initiative with their own Sweep.
    t.enter(P1, "Undercellar Sweep");
    t.resolve_all();
    assert_eq!(t.g.initiative, Some(P1));
    let a = t.battlefield(P0, "Grizzly Bears");
    to_combat(&mut t, P0);
    declare(&mut t, P0, &[(a, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(soldiers(&t, P0), 2);
}

#[test]
fn undercellar_sweep_does_nothing_if_nobody_involved_has_the_initiative() {
    cr!("603.4");
    let mut t = multi(3);
    t.battlefield(P0, "Undercellar Sweep");
    t.enter(P2, "Undercellar Sweep");
    t.resolve_all();
    let a = t.battlefield(P0, "Grizzly Bears");
    to_combat(&mut t, P0);
    declare(&mut t, P0, &[(a, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(soldiers(&t, P0), 0);
}

#[test]
fn jin_sakai_triggers_only_for_a_lone_attacker_on_that_player() {
    cr!("508.6", "603.4", "700.2");
    supported("Jin Sakai, Ghost of Tsushima");
    let mut t = multi(3);
    t.battlefield(P0, "Jin Sakai, Ghost of Tsushima");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Hill Giant");
    to_combat(&mut t, P0);
    t.answer(
        P0,
        DecisionKind::Modes,
        mtg_engine::decision::Answer::Indices(vec![0]),
    );
    declare(
        &mut t,
        P0,
        &[
            (a, Entity::Player(P1)),
            (b, Entity::Player(P2)),
            (c, Entity::Player(P2)),
        ],
    );
    t.resolve_all();
    use mtg_engine::keywords::KeywordKind;
    assert!(t.obj_now(a).has_keyword(KeywordKind::DoubleStrike));
    assert!(!t.obj_now(b).has_keyword(KeywordKind::DoubleStrike));
    assert!(!t.obj_now(c).has_keyword(KeywordKind::DoubleStrike));
}

// ---------------------------------------------------------------------------
// Creatures banded with ~ (CR 702.22).
// ---------------------------------------------------------------------------

#[test]
fn icatian_skirmishers_gives_first_strike_to_its_band() {
    cr!("702.22c", "508.1e");
    supported("Icatian Skirmishers");
    use mtg_engine::keywords::KeywordKind;
    let mut t = TestGame::new(2);
    let sk = t.battlefield(P0, "Icatian Skirmishers");
    let mate = t.battlefield(P0, "Grizzly Bears");
    let alone = t.battlefield(P0, "Hill Giant");
    to_combat(&mut t, P0);
    t.answer_choose(P0, &[Entity::Object(mate)]);
    declare(
        &mut t,
        P0,
        &[
            (sk, Entity::Player(P1)),
            (mate, Entity::Player(P1)),
            (alone, Entity::Player(P1)),
        ],
    );
    t.resolve_all();
    assert!(t.obj_now(mate).has_keyword(KeywordKind::FirstStrike));
    assert!(!t.obj_now(alone).has_keyword(KeywordKind::FirstStrike));
}

#[test]
fn urzas_engine_gives_trample_to_its_band() {
    cr!("702.22c");
    supported("Urza's Engine");
    use mtg_engine::keywords::KeywordKind;
    let mut t = TestGame::new(2);
    let engine = t.battlefield(P0, "Urza's Engine");
    let mate = t.battlefield(P0, "Grizzly Bears");
    let alone = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Plains", 6);
    t.activate(P0, engine, 0, &[]).unwrap();
    t.resolve();
    to_combat(&mut t, P0);
    t.answer_choose(P0, &[Entity::Object(mate)]);
    declare(
        &mut t,
        P0,
        &[
            (engine, Entity::Player(P1)),
            (mate, Entity::Player(P1)),
            (alone, Entity::Player(P1)),
        ],
    );
    t.activate(P0, engine, 1, &[]).unwrap();
    t.resolve();
    assert!(t.obj_now(mate).has_keyword(KeywordKind::Trample));
    assert!(!t.obj_now(alone).has_keyword(KeywordKind::Trample));
}
