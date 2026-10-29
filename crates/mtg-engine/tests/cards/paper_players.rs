//! Player verbs and filters of older Oracle wordings (cards4-paper-effects, family
//! `player-actions`): "You destroy four lands you control, then target opponent destroys
//! four lands they control", "If that player doesn't, they tap all lands ...", "you both
//! own and control".

use crate::combat2_common::supported;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn burning_of_xinye_each_player_chooses_their_own_lands_to_destroy() {
    cr!("701.8a", "608.2d");
    interp!("player-destroys-chosen-permanents");
    ruling!(
        "Burning of Xinye",
        "That player chooses which of their lands to destroy. Neither of you can choose to destroy a land with indestructible."
    );
    ruling!(
        "Burning of Xinye",
        "The lands that will be destroyed aren’t chosen until Burning of Xinye resolves."
    );
    supported("Burning of Xinye");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let mine = t.lands(P0, "Mountain", 7);
    let theirs = t.lands(P1, "Forest", 4);
    let citadel = t.battlefield(P1, "Darksteel Citadel");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Burning of Xinye");
    t.answer_choose(
        P0,
        &mine[..4].iter().map(|o| Entity::Object(*o)).collect::<Vec<_>>(),
    );
    t.answer_choose(
        P1,
        &theirs.iter().map(|o| Entity::Object(*o)).collect::<Vec<_>>(),
    );
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve();
    for l in &mine[..4] {
        assert!(!t.on_battlefield(*l));
    }
    for l in &mine[4..] {
        assert!(t.on_battlefield(*l));
    }
    for l in &theirs {
        assert!(!t.on_battlefield(*l));
    }
    // The indestructible land couldn't be chosen; it survives anyway.
    assert!(t.on_battlefield(citadel));
    // 4 damage to each creature.
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(!t.on_battlefield(bears));
    // The opponent was asked to choose among their destructible lands only.
    let asked = t.asked();
    assert!(asked.iter().any(|(p, d)| *p == P1
        && matches!(d, Decision::ChooseEntities { candidates, .. }
            if !candidates.contains(&Entity::Object(citadel)) && candidates.len() == 4)));
}

#[test]
fn power_sink_taps_the_unpaying_players_lands_with_mana_abilities() {
    cr!("118.12", "605.1a");
    ruling!(
        "Power Sink",
        "Only lands that actually have mana abilities will get tapped."
    );
    supported("Power Sink");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let forests = t.lands(P1, "Forest", 4);
    let desert = t.battlefield(P1, "Maze of Ith");
    let mine = t.lands(P0, "Island", 3);
    let sink = t.hand(P0, "Power Sink");
    let bears = t.hand(P1, "Grizzly Bears");
    let spell = t.cast(P1, bears).go();
    // P1 paid for the Bears with two Forests; two stay untapped.
    assert_eq!(
        forests.iter().filter(|f| !t.obj_now(**f).tapped).count(),
        2
    );
    t.answer_yes(P1, false);
    t.cast(P0, sink).x(2).target(Entity::Object(spell)).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(forests.iter().all(|f| t.obj_now(*f).tapped));
    // Maze of Ith has no mana ability: not tapped.
    assert!(!t.obj_now(desert).tapped);
    // Power Sink's controller's lands are unaffected beyond paying for it.
    assert_eq!(mine.iter().filter(|l| t.obj_now(**l).tapped).count(), 3);
}

#[test]
fn power_sink_paid_nothing_happens() {
    cr!("118.12");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let forests = t.lands(P1, "Forest", 4);
    t.lands(P0, "Island", 3);
    let sink = t.hand(P0, "Power Sink");
    let bears = t.hand(P1, "Grizzly Bears");
    let spell = t.cast(P1, bears).go();
    t.answer_yes(P1, true);
    t.cast(P0, sink).x(1).target(Entity::Object(spell)).go();
    t.resolve();
    // Paid {1} with a third Forest; the fourth stays untapped and the spell resolves.
    assert_eq!(
        forests.iter().filter(|f| !t.obj_now(**f).tapped).count(),
        1
    );
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn obelisk_of_undoing_needs_a_permanent_you_both_own_and_control() {
    cr!("115.1", "108.3");
    supported("Obelisk of Undoing");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let o = t.battlefield(P0, "Obelisk of Undoing");
    t.lands(P0, "Plains", 6);
    // Owned by P1, controlled by P0: not a legal target.
    let stolen = t.battlefield(P1, "Hill Giant");
    t.g.objects[stolen.0 as usize].controller = P0;
    t.battlefield(P0, "Grizzly Bears");
    // The answer naming it isn't accepted (the engine falls back to a legal target).
    let _ = t.activate(P0, o, 0, &[Entity::Object(stolen)]);
    t.resolve();
    assert!(t.on_battlefield(stolen));
    assert!(!t.in_hand(P1, "Hill Giant"));
    // Owned and controlled by P0: returned to hand.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let o = t.battlefield(P0, "Obelisk of Undoing");
    t.lands(P0, "Plains", 6);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, o, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}
