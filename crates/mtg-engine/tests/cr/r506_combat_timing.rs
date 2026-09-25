//! CR 506.8: spells and abilities that may be cast/activated only before or after a
//! particular point in the combat phase.

use crate::r506_common::*;
use mtg_engine::ability::StepKind;
use mtg_engine::decision::Action;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn instant(t: &mut TestGame, p: PlayerId, name: &str, restriction: &str) -> ObjectId {
    let def = custom_card(
        name,
        "Instant",
        None,
        &format!("{restriction}\nYou gain 1 life."),
    );
    t.custom(p, def, Zone::Hand(p))
}

fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    let saved = t.g.turn.priority;
    t.g.turn.priority = Some(p);
    let ok =
        t.g.legal_actions(p)
            .iter()
            .any(|a| matches!(a, Action::Cast { card: c, .. } if *c == card));
    t.g.turn.priority = saved;
    ok
}

fn activatable(t: &mut TestGame, p: PlayerId, src: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Activate { source, .. } if *source == src))
}

#[test]
fn before_and_after_attackers_are_declared() {
    cr!("506.8", "506.8a");
    let mut t = TestGame::new(2);
    let before = instant(
        &mut t,
        P0,
        "Early Word",
        "Cast this spell only before attackers are declared.",
    );
    let after = instant(
        &mut t,
        P0,
        "Late Word",
        "Cast this spell only after attackers are declared.",
    );
    t.battlefield(P0, "Grizzly Bears");
    // Precombat main phase and beginning of combat: before.
    assert!(castable(&mut t, P0, before) && !castable(&mut t, P0, after));
    go_to(&mut t, Step::BeginningOfCombat);
    assert!(castable(&mut t, P0, before) && !castable(&mut t, P0, after));
    // The declare attackers step has begun — even though no attackers were declared.
    go_to(&mut t, Step::DeclareAttackers);
    assert!(t.g.attackers().is_empty());
    assert!(!castable(&mut t, P0, before));
    assert!(castable(&mut t, P0, after));
    go_to(&mut t, Step::PostcombatMain);
    assert!(!castable(&mut t, P0, before) && castable(&mut t, P0, after));
}

#[test]
fn before_and_after_blockers_are_declared() {
    cr!("506.8", "506.8b");
    let mut t = TestGame::new(2);
    // Rapid Fire: "Cast this spell only before blockers are declared."
    let before = t.hand(P0, "Rapid Fire");
    t.lands(P0, "Plains", 4);
    let after = instant(
        &mut t,
        P0,
        "Late Word",
        "Cast this spell only after blockers are declared.",
    );
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    // Even before combat, it's "before blockers are declared".
    assert!(castable(&mut t, P0, before) && !castable(&mut t, P0, after));
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(castable(&mut t, P0, before) && !castable(&mut t, P0, after));
    // P1 declares no blockers, but the declare blockers step has begun.
    go_to(&mut t, Step::DeclareBlockers);
    assert!(t.g.blockers().is_empty());
    assert!(!castable(&mut t, P0, before));
    assert!(castable(&mut t, P0, after));
}

#[test]
fn during_combat_spells_work_in_every_combat_phase() {
    cr!("506.8c");
    let mut t = TestGame::new(2);
    // Gorilla War Cry: "Cast this spell only during combat before blockers are declared."
    let cry = t.hand(P0, "Gorilla War Cry");
    t.lands(P0, "Mountain", 4);
    let during = instant(
        &mut t,
        P0,
        "Battle Word",
        "Cast this spell only during combat.",
    );
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(!castable(&mut t, P0, cry) && !castable(&mut t, P0, during));
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(castable(&mut t, P0, cry) && castable(&mut t, P0, during));
    // An additional combat phase follows this one.
    t.g.add_extra_combat(true);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(!castable(&mut t, P0, cry) && castable(&mut t, P0, during));
    go_to(&mut t, Step::EndOfCombat);
    assert!(!castable(&mut t, P0, cry) && castable(&mut t, P0, during));
    // Second combat: before blockers are declared again.
    go_to(&mut t, Step::BeginningOfCombat);
    assert_eq!(t.g.turn.combat_phases, 2);
    assert!(castable(&mut t, P0, cry) && castable(&mut t, P0, during));
}

#[test]
fn during_a_certain_players_combat() {
    cr!("506.8c");
    let mut t = TestGame::new(2);
    let theirs = instant(
        &mut t,
        P1,
        "Ambush Word",
        "Cast this spell only during combat on an opponent's turn.",
    );
    let mine = instant(
        &mut t,
        P0,
        "Own Word",
        "Cast this spell only during combat on an opponent's turn.",
    );
    go_to(&mut t, Step::BeginningOfCombat);
    assert!(castable(&mut t, P1, theirs));
    assert!(!castable(&mut t, P0, mine));
}

#[test]
fn without_during_combat_only_the_first_combat_phase_counts() {
    cr!("506.8d");
    let mut t = TestGame::new(2);
    let before = t.hand(P0, "Rapid Fire");
    t.lands(P0, "Plains", 4);
    let after = instant(
        &mut t,
        P0,
        "Late Word",
        "Cast this spell only after attackers are declared.",
    );
    let bears = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    t.g.add_extra_combat(true);
    go_to(&mut t, Step::EndOfCombat);
    go_to(&mut t, Step::BeginningOfCombat);
    assert_eq!(t.g.turn.combat_phases, 2);
    // Before blockers are declared in the second combat, but after the first combat's
    // declare blockers step: not castable.
    assert!(!castable(&mut t, P0, before));
    // "After attackers are declared" refers to the first combat: castable.
    assert!(castable(&mut t, P0, after));
}

#[test]
fn before_a_skipped_point_means_before_the_declare_attackers_step_ends() {
    cr!("506.8e");
    let mut t = TestGame::new(2);
    let before = t.hand(P0, "Rapid Fire");
    t.lands(P0, "Plains", 4);
    t.battlefield(P0, "Grizzly Bears");
    // No attackers: the declare blockers and combat damage steps are skipped.
    go_to(&mut t, Step::DeclareAttackers);
    assert!(t.g.attackers().is_empty());
    assert!(
        castable(&mut t, P0, before),
        "still in the declare attackers step"
    );
    go_to(&mut t, Step::EndOfCombat);
    assert!(!castable(&mut t, P0, before));
}

#[test]
fn before_a_point_of_a_skipped_combat_phase_means_before_precombat_main_ends() {
    cr!("506.8e");
    let mut t = TestGame::new(2);
    let before = t.hand(P0, "Rapid Fire");
    t.lands(P0, "Plains", 4);
    t.g.players[0].skips.push(StepKind::Combat);
    assert!(castable(&mut t, P0, before));
    go_to(&mut t, Step::PostcombatMain);
    assert_eq!(t.g.turn.combat_phases, 0, "the combat phase was skipped");
    assert!(!castable(&mut t, P0, before));
}

#[test]
fn during_combat_after_blockers_with_skipped_declare_blockers_step() {
    cr!("506.8f");
    // Chaotic Strike: "Cast this spell only during combat after blockers are declared."
    let mut t = TestGame::new(2);
    let strike = t.hand(P0, "Chaotic Strike");
    t.lands(P0, "Mountain", 2);
    t.battlefield(P0, "Grizzly Bears");
    go_to(&mut t, Step::EndOfCombat);
    assert!(!castable(&mut t, P0, strike));

    // With the declare blockers step, it can be cast after it begins.
    let mut t = TestGame::new(2);
    let strike = t.hand(P0, "Chaotic Strike");
    t.lands(P0, "Mountain", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(!castable(&mut t, P0, strike));
    go_to(&mut t, Step::DeclareBlockers);
    assert!(castable(&mut t, P0, strike));
    go_to(&mut t, Step::EndOfCombat);
    assert!(castable(&mut t, P0, strike));
    go_to(&mut t, Step::PostcombatMain);
    assert!(!castable(&mut t, P0, strike));
}

#[test]
fn activated_abilities_follow_the_same_timing_rules() {
    cr!("506.8g");
    let mut t = TestGame::new(2);
    let early = bf(
        &mut t,
        P0,
        custom_card(
            "Early Totem",
            "Artifact",
            None,
            "{T}: You gain 1 life. Activate only before blockers are declared.",
        ),
    );
    let late = bf(
        &mut t,
        P0,
        custom_card(
            "Late Totem",
            "Artifact",
            None,
            "{T}: You gain 1 life. Activate only during combat after blockers are declared.",
        ),
    );
    let pre_attack = bf(
        &mut t,
        P0,
        custom_card(
            "Muster Totem",
            "Artifact",
            None,
            "{T}: You gain 1 life. Activate only before attackers are declared.",
        ),
    );
    // Precombat main: before blockers (and attackers) are declared.
    assert!(activatable(&mut t, P0, early));
    assert!(activatable(&mut t, P0, pre_attack));
    assert!(!activatable(&mut t, P0, late));
    t.battlefield(P0, "Grizzly Bears");
    // No attackers: the declare blockers step is skipped.
    go_to(&mut t, Step::DeclareAttackers);
    assert!(activatable(&mut t, P0, early));
    assert!(!activatable(&mut t, P0, pre_attack));
    go_to(&mut t, Step::EndOfCombat);
    assert!(!activatable(&mut t, P0, early));
    assert!(!activatable(&mut t, P0, late));

    let mut t = TestGame::new(2);
    let late = bf(
        &mut t,
        P0,
        custom_card(
            "Late Totem",
            "Artifact",
            None,
            "{T}: You gain 1 life. Activate only during combat after blockers are declared.",
        ),
    );
    let bears = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(activatable(&mut t, P0, late));
}
