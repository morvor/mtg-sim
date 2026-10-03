//! CR 701.14: fight; CR 701.15: goad.

use crate::a701_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn fighting_creatures_deal_damage_equal_to_their_power_to_each_other() {
    cr!("701.14", "701.14a");
    supported("Prey Upon");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let prey = t.hand(P0, "Prey Upon");
    t.cast(P0, prey).target(giant).target(bears).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.obj(giant).damage, 2);
}

#[test]
fn if_either_creature_is_gone_neither_fights() {
    cr!("701.14b");
    supported("Prey Upon");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let prey = t.hand(P0, "Prey Upon");
    t.cast(P0, prey).target(giant).target(bears).go();
    // The Giant leaves the battlefield in response: the Bears don't fight either.
    t.g.move_object(
        giant,
        Zone::Hand(P0),
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.resolve();
    assert_eq!(t.obj(bears).damage, 0);
    // A creature that stopped being a creature doesn't fight either.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let prey = t.hand(P0, "Prey Upon");
    t.cast(P0, prey).target(giant).target(bears).go();
    run(
        &mut t,
        P1,
        None,
        Effect::Modify {
            what: Sel::All(Filter::Objects(vec![bears])),
            mods: vec![Modification::RemoveTypes(vec![CardType::Creature])],
            duration: Duration::EndOfTurn,
        },
    );
    t.resolve();
    assert_eq!(t.obj(giant).damage, 0);
    assert_eq!(t.obj(bears).damage, 0);
}

#[test]
fn a_creature_fighting_itself_deals_twice_its_power_to_itself() {
    cr!("701.14c");
    let def = oracle_card(
        "Inner Struggle",
        "Instant",
        "{0}",
        None,
        "Target creature fights target creature.",
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let s = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, s).target(giant).target(giant).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    let dmg: u32 = t
        .g
        .turn_events
        .iter()
        .filter_map(|e| match e {
            mtg_engine::events::Event::Damage { amount, .. } => Some(*amount),
            _ => None,
        })
        .sum();
    assert_eq!(dmg, 6);
}

#[test]
fn fight_damage_isnt_combat_damage() {
    cr!("701.14d");
    supported("Prey Upon");
    supported("Fog");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // "Prevent all combat damage that would be dealt this turn."
    let fog = t.hand(P0, "Fog");
    t.cast(P0, fog).go();
    t.resolve();
    let prey = t.hand(P0, "Prey Upon");
    t.cast(P0, prey).target(giant).target(bears).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.g.turn_events.iter().any(|e| matches!(
        e,
        mtg_engine::events::Event::Damage { combat: false, .. }
    )));
}

/// Advances to P1's declare attackers step, letting P1's agent declare its default
/// attacks, and returns what each creature attacked.
fn p1_attacks(t: &mut TestGame) -> Vec<(ObjectId, Entity)> {
    t.set_step(P1, Step::BeginningOfCombat);
    t.advance_to(P1, Step::DeclareAttackers);
    t.g.combat
        .as_ref()
        .map(|c| {
            c.attackers
                .iter()
                .filter_map(|a| a.target.map(|t| (a.id, t)))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

#[test]
fn a_goaded_creature_attacks_a_player_other_than_the_goader_until_the_goaders_next_turn() {
    cr!("701.15", "701.15a", "701.15b");
    supported("Disrupt Decorum");
    let mut t = TestGame::new(3);
    t.lands(P0, "Mountain", 4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let dd = t.hand(P0, "Disrupt Decorum");
    t.cast(P0, dd).go();
    t.resolve();
    assert_eq!(t.g.goaders(bears), vec![P0]);
    // On P1's turn, the Bears attack (though P1 declares no attackers), and not P0.
    let attacks = p1_attacks(&mut t);
    assert_eq!(attacks, vec![(bears, Entity::Player(P2))]);
    // The goad ends as P0's next turn begins.
    t.advance_to(P0, Step::Upkeep);
    assert!(t.g.goaders(bears).is_empty());
}

#[test]
fn a_creature_can_be_goaded_by_several_players_but_once_by_each() {
    cr!("701.15c", "701.15d");
    supported("Disrupt Decorum");
    let mut t = TestGame::new(4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let goad = |t: &mut TestGame, p: PlayerId| {
        run(
            t,
            p,
            None,
            Effect::KeywordAction {
                action: KeywordAction::Goad,
                who: PlayerRef::You,
                what: Sel::All(Filter::Objects(vec![bears])),
                n: Value::c(1),
            },
        );
    };
    goad(&mut t, P0);
    goad(&mut t, P0);
    assert_eq!(t.g.goaders(bears), vec![P0]);
    goad(&mut t, P2);
    assert_eq!(t.g.goaders(bears), vec![P0, P2]);
    // Goaded by P0 and P2, it must attack P3 if able.
    let attacks = p1_attacks(&mut t);
    assert_eq!(attacks, vec![(bears, Entity::Player(P3))]);
    let _ = Decision::Priority { actions: vec![] };
}
