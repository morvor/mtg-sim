//! "You may have ~ assign its combat damage as though it weren't blocked." (Thorn
//! Elemental, Proud Wildbonder) and "remove [creature] from combat" (Gustcloak Runner,
//! Hollowhenge Spirit).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

/// Thorn Elemental attacks P1 and Grizzly Bears blocks it; P0 answers `unblocked`.
fn thorn(unblocked: bool) -> TestGame {
    let mut t = TestGame::new(2);
    let thorn = t.battlefield(P0, "Thorn Elemental");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, unblocked);
    t.attack(&[(thorn, Entity::Player(P1))], &[(bears, thorn)]);
    t
}

#[test]
fn thorn_elemental_may_assign_all_damage_to_the_player() {
    cr!("510.1b", "510.1c");
    ruling!(
        "Thorn Elemental",
        "you choose whether you want to assign all damage to blocking creatures, or if you want to assign all of it to the player"
    );
    assert_supported("Thorn Elemental");
    let t = thorn(true);
    assert_eq!(t.life(P1), 13, "{}", t.dump_log());
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Or all of it to the blocker.
    let t = thorn(false);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "{}", t.dump_log());
}

#[test]
fn proud_wildbonder_grants_it_to_tramplers() {
    cr!("510.1b", "613.1f");
    ruling!(
        "Proud Wildbonder",
        "Proud Wildbonder’s last ability applies to itself as long as it still has trample."
    );
    assert_supported("Proud Wildbonder");
    let mut t = TestGame::new(2);
    let wb = t.battlefield(P0, "Proud Wildbonder");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.answer_yes(P0, true);
    t.attack(&[(wb, Entity::Player(P1))], &[(wall, wb)]);
    // A 4/3 trampler blocked by a 0/8 wall would deal nothing to P1 with trample alone.
    assert_eq!(t.life(P1), 16, "{}", t.dump_log());
    assert_eq!(t.obj_now(wall).damage, 0);
}

#[test]
fn gustcloak_runner_untaps_and_leaves_combat_when_blocked() {
    cr!("506.4");
    assert_supported("Gustcloak Runner");
    let mut t = TestGame::new(2);
    let runner = t.battlefield(P0, "Gustcloak Runner");
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.answer_yes(P0, true);
    t.attack(&[(runner, Entity::Player(P1))], &[(ogre, runner)]);
    // Removed from combat: the blocker deals no damage to it, and it's untapped.
    assert!(t.on_battlefield(runner), "{}", t.dump_log());
    assert!(!t.obj_now(runner).tapped);
    assert_eq!(t.obj_now(runner).damage, 0);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn hollowhenge_spirit_removes_an_attacker_from_combat() {
    cr!("506.4");
    ruling!(
        "Hollowhenge Spirit",
        "Removing an attacking creature from combat doesn’t untap that creature."
    );
    assert_supported("Hollowhenge Spirit");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 4);
    t.set_step(P1, Step::BeginningOfCombat);
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P0))]),
    );
    t.advance_to(P1, Step::DeclareBlockers);
    assert!(t.g.is_attacking(bears), "{}", t.dump_log());
    let spirit = t.hand(P0, "Hollowhenge Spirit");
    t.cast(P0, spirit).go();
    t.resolve();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(!t.g.is_attacking(bears));
    // Removing it from combat doesn't untap it.
    assert!(t.obj_now(bears).tapped);
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.life(P0), 20, "{}", t.dump_log());
}

/// Players asked the "as though it weren't blocked" question so far.
fn asked_unblocked(t: &TestGame) -> Vec<PlayerId> {
    t.asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            mtg_engine::decision::Decision::YesNo { prompt, .. }
                if prompt.contains("as though it weren't blocked") =>
            {
                Some(p)
            }
            _ => None,
        })
        .collect()
}

#[test]
fn thorn_elemental_blocked_by_banding_defending_player_decides() {
    cr!("702.22j");
    ruling!(
        "Thorn Elemental",
        "If blocked by a creature with banding, the defending player decides"
    );
    for (p1_says, p1_life) in [(false, 20), (true, 13)] {
        let mut t = TestGame::new(2);
        let thorn = t.battlefield(P0, "Thorn Elemental");
        let hero = t.battlefield(P1, "Benalish Hero");
        // The attacking player's answer is the opposite: only P1's counts.
        t.answer_yes(P0, !p1_says);
        t.answer_yes(P1, p1_says);
        t.attack(&[(thorn, Entity::Player(P1))], &[(hero, thorn)]);
        assert_eq!(asked_unblocked(&t), vec![P1], "{}", t.dump_log());
        assert_eq!(t.life(P1), p1_life, "{}", t.dump_log());
        assert_eq!(t.on_battlefield(hero), p1_says);
    }
}

#[test]
fn thorn_elemental_still_blocked_with_no_blockers_left_may_hit_the_player() {
    cr!("509.1h", "510.1b", "510.1c");
    for (unblocked, p1_life) in [(true, 13), (false, 20)] {
        let mut t = TestGame::new(2);
        let thorn = t.battlefield(P0, "Thorn Elemental");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Mountain", 1);
        t.set_step(P0, Step::BeginningOfCombat);
        t.answer(
            P0,
            DecisionKind::Attackers,
            Answer::Attackers(vec![(thorn, Entity::Player(P1))]),
        );
        t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(bears, thorn)]));
        t.advance_to(P0, Step::DeclareBlockers);
        // Kill the blocker: Thorn Elemental remains blocked (CR 509.1h).
        let bolt = t.hand(P0, "Lightning Bolt");
        t.cast(P0, bolt).target(bears).go();
        t.resolve();
        assert!(t.in_graveyard(P1, "Grizzly Bears"));
        t.answer_yes(P0, unblocked);
        t.advance_to(P0, Step::EndOfCombat);
        // Blocked with no blockers, it assigns no damage — unless its controller has it
        // assign its damage as though it weren't blocked.
        assert_eq!(t.life(P1), p1_life, "{}", t.dump_log());
    }
}

#[test]
fn thorn_elemental_attacking_a_planeswalker_assigns_to_the_planeswalker() {
    cr!("510.1b");
    let mut t = TestGame::new(2);
    let thorn = t.battlefield(P0, "Thorn Elemental");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let pw = t.battlefield(P1, "Ajani Goldmane");
    t.answer_yes(P0, true);
    t.attack(&[(thorn, Entity::Object(pw))], &[(bears, thorn)]);
    // All of it to the planeswalker it's attacking, not the player or the blocker.
    assert!(!t.on_battlefield(pw), "{}", t.dump_log());
    assert_eq!(t.life(P1), 20);
    assert!(t.on_battlefield(bears));
}

#[test]
fn proud_wildbonder_declining_assigns_trample_damage_normally() {
    cr!("702.19b");
    ruling!(
        "Proud Wildbonder",
        "you choose whether you want to assign all damage to blocking creatures and assign trample damage from that as normal"
    );
    let mut t = TestGame::new(2);
    let wb = t.battlefield(P0, "Proud Wildbonder");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, false);
    t.attack(&[(wb, Entity::Player(P1))], &[(bears, wb)]);
    // Lethal damage (2) to the blocker, the rest (2) tramples over.
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "{}", t.dump_log());
    assert_eq!(t.life(P1), 18, "{}", t.dump_log());
    assert_eq!(asked_unblocked(&t), vec![P0]);
}

#[test]
fn hollowhenge_spirit_removing_a_blocker_leaves_the_attacker_blocked() {
    cr!("506.4", "509.1h", "510.1c");
    ruling!(
        "Hollowhenge Spirit",
        "Removing a blocking creature from combat doesn’t cause the creature it was blocking to become unblocked."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wall = t.battlefield(P0, "Wall of Stone");
    t.lands(P0, "Plains", 4);
    t.set_step(P1, Step::BeginningOfCombat);
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P0))]),
    );
    t.answer(P0, DecisionKind::Blockers, Answer::Blockers(vec![(wall, bears)]));
    t.advance_to(P1, Step::DeclareBlockers);
    assert!(t.g.is_blocking(wall), "{}", t.dump_log());
    let spirit = t.hand(P0, "Hollowhenge Spirit");
    t.cast(P0, spirit).go();
    t.answer_targets(P0, &[Entity::Object(wall)]);
    t.resolve_all();
    assert!(!t.g.is_blocking(wall), "{}", t.dump_log());
    assert!(t.g.is_attacking(bears));
    t.advance_to(P1, Step::EndOfCombat);
    // Still blocked, with no blockers: no combat damage to anyone.
    assert_eq!(t.life(P0), 20, "{}", t.dump_log());
    assert_eq!(t.obj_now(wall).damage, 0);
}
