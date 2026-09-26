//! CR 702.68 Frenzy.

use crate::common_k702_011_017::{assert_supported, attack_with, bf, custom_card};
use crate::common_k702_018_026::{declare_blocks, triggers_on_stack};
use crate::common_k702_052_066::run_effect;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn frenzy_pumps_an_attacker_that_isnt_blocked() {
    cr!("702.68", "702.68a");
    assert_supported("Frenzy Sliver");
    let mut t = TestGame::new(2);
    // Frenzy Sliver: 1/1, "All Sliver creatures have frenzy 1."
    let sliver = t.battlefield(P0, "Frenzy Sliver");
    assert_eq!(t.obj_now(sliver).chars.keyword_count(KeywordKind::Frenzy), 1);
    t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(sliver, Entity::Player(P1))]);
    // Nothing has triggered yet: it triggers once blockers are declared.
    assert_eq!(triggers_on_stack(&t, "Frenzy 1"), 0);
    declare_blocks(&mut t, P1, &[]);
    assert_eq!(t.g.turn.step, Step::DeclareBlockers);
    assert_eq!(triggers_on_stack(&t, "Frenzy 1"), 1);
    t.resolve_all();
    // +1/+0 only.
    assert_eq!(t.pt(sliver), (2, 1));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
    // Until end of turn.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(sliver), (1, 1));
}

#[test]
fn frenzy_doesnt_trigger_for_a_blocked_attacker() {
    cr!("702.68a");
    let mut t = TestGame::new(2);
    let sliver = t.battlefield(P0, "Frenzy Sliver");
    let wall = t.battlefield(P1, "Wall of Wood");
    attack_with(&mut t, &[(sliver, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(wall, sliver)]);
    assert_eq!(triggers_on_stack(&t, "Frenzy 1"), 0);
    t.resolve_all();
    assert_eq!(t.pt(sliver), (1, 1));
}

#[test]
fn frenzy_needs_the_creature_to_be_attacking() {
    cr!("702.68a");
    let mut t = TestGame::new(2);
    let sliver = t.battlefield(P0, "Frenzy Sliver");
    let other = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(other, Entity::Player(P1))]);
    // Frenzy Sliver isn't attacking: it doesn't trigger.
    declare_blocks(&mut t, P1, &[]);
    assert_eq!(triggers_on_stack(&t, "Frenzy 1"), 0);
    assert_eq!(t.pt(sliver), (1, 1));
}

#[test]
fn frenzy_triggers_for_a_creature_put_onto_the_battlefield_attacking() {
    cr!("702.68a");
    ruling!(
        "Frenzy Sliver",
        "It will trigger even if that creature was put onto the battlefield attacking"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sliver = t.hand(P0, "Frenzy Sliver");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    let mut to = Destination::battlefield();
    to.attacking = true;
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Move {
            what: Sel::Target(0),
            to,
        },
        &[Entity::Object(sliver)],
    );
    let sliver = t.g.current(sliver);
    assert!(t.g.is_attacking(sliver));
    declare_blocks(&mut t, P1, &[]);
    assert_eq!(triggers_on_stack(&t, "Frenzy 1"), 1);
    t.resolve_all();
    assert_eq!(t.pt(sliver), (2, 1));
}

#[test]
fn each_instance_of_frenzy_triggers_separately() {
    cr!("702.68b");
    ruling!(
        "Frenzy Sliver",
        "Abilities that Slivers grant, as well as power/toughness boosts, are cumulative."
    );
    let mut t = TestGame::new(2);
    // Two Frenzy Slivers: each Sliver has frenzy 1 twice.
    let a = t.battlefield(P0, "Frenzy Sliver");
    t.battlefield(P0, "Frenzy Sliver");
    assert_eq!(t.obj_now(a).chars.keyword_count(KeywordKind::Frenzy), 2);
    attack_with(&mut t, &[(a, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[]);
    assert_eq!(triggers_on_stack(&t, "Frenzy 1"), 2);
    t.resolve_all();
    assert_eq!(t.pt(a), (3, 1));
}

#[test]
fn frenzy_n_gives_plus_n() {
    cr!("702.68a", "702.68b");
    let def = custom_card(
        "Frenzied Raider",
        "Creature — Human Berserker",
        Some((2, 2)),
        "Frenzy 2\nFrenzy 1",
    );
    let mut t = TestGame::new(2);
    let raider = bf(&mut t, P0, def);
    attack_with(&mut t, &[(raider, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[]);
    assert_eq!(triggers_on_stack(&t, "Frenzy 2"), 1);
    assert_eq!(triggers_on_stack(&t, "Frenzy 1"), 1);
    t.resolve_all();
    assert_eq!(t.pt(raider), (5, 2));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 15);
}

#[test]
fn a_frenzy_sliver_that_isnt_a_sliver_doesnt_have_frenzy() {
    cr!("702.68a");
    ruling!(
        "Frenzy Sliver",
        "If the creature type of a Sliver changes so it’s no longer a Sliver, it will no longer be affected by its own ability."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Frenzy Sliver");
    let b = t.battlefield(P0, "Frenzy Sliver");
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveAllCreatureTypes],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(a)],
    );
    assert_eq!(t.obj_now(a).chars.keyword_count(KeywordKind::Frenzy), 0);
    // Its ability still gives the other Sliver frenzy.
    assert_eq!(t.obj_now(b).chars.keyword_count(KeywordKind::Frenzy), 2);
}
