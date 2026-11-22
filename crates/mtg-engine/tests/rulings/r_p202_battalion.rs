//! Rulings batch P202 — battalion (an ability word, CR 207.2c): "Whenever this creature and
//! at least two other creatures attack, ..."

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts `name` and two Grizzly Bears onto P0's battlefield and attacks P1 with all three;
/// returns (the battalion creature, the bears). The battalion trigger is on the stack.
fn battalion_attack(t: &mut TestGame, name: &str) -> (ObjectId, Vec<ObjectId>) {
    let c = t.battlefield(P0, name);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let p1 = Entity::Player(P1);
    attack_with(t, &[(c, p1), (a, p1), (b, p1)]);
    assert_eq!(triggers_on_stack(t, "at least two other creatures attack"), 1);
    (c, vec![a, b])
}

#[test]
fn makeshift_battalions_attackers_can_attack_different_defenders() {
    cr!("207.2c", "508.1b");
    ruling!(
        "Makeshift Battalion",
        "Makeshift Battalion and the other attacking creatures don't have to be attacking the same player, planeswalker, or battle."
    );
    supported("Makeshift Battalion");
    let mut t = TestGame::new(3);
    let mb = t.battlefield(P0, "Makeshift Battalion");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let jace = t.battlefield(P1, "Jace Beleren");
    attack_with(
        &mut t,
        &[
            (mb, Entity::Player(P1)),
            (a, Entity::Object(jace)),
            (b, Entity::Player(P2)),
        ],
    );
    assert_eq!(triggers_on_stack(&t, "at least two other creatures attack"), 1);
    t.resolve_all();
    assert_eq!(t.counters(mb, counters::PLUS1), 1);
}

#[test]
fn makeshift_battalions_trigger_doesnt_recheck_the_attackers() {
    cr!("207.2c", "603.2", "506.4");
    ruling!(
        "Makeshift Battalion",
        "Once Makeshift Battalion's ability has triggered, it doesn't matter how many creatures are still attacking when that ability resolves."
    );
    let mut t = TestGame::new(2);
    let (mb, bears) = battalion_attack(&mut t, "Makeshift Battalion");
    destroy(&mut t, bears[0]);
    mtg_engine::combat::remove_from_combat(&mut t.g, bears[1]);
    t.resolve_all();
    assert_eq!(t.counters(mb, counters::PLUS1), 1);
}

#[test]
fn paladin_elizabeth_taggerdys_trigger_doesnt_recheck_the_attackers() {
    cr!("207.2c", "603.2", "506.4");
    ruling!(
        "Paladin Elizabeth Taggerdy",
        "Once Paladin Elizabeth Taggerdy’s battalion ability has triggered, it doesn’t matter how many creatures are still attacking when that ability resolves."
    );
    supported("Paladin Elizabeth Taggerdy");
    let mut t = TestGame::new(2);
    let (_, bears) = battalion_attack(&mut t, "Paladin Elizabeth Taggerdy");
    let hand = t.hand_size(P0);
    destroy(&mut t, bears[0]);
    destroy(&mut t, bears[1]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn paladin_elizabeth_taggerdys_x_uses_last_known_power() {
    cr!("207.2c", "608.2h", "113.7a");
    ruling!(
        "Paladin Elizabeth Taggerdy",
        "If Paladin Elizabeth Taggerdy is no longer on the battlefield when its battalion ability resolves, use its power as it last existed on the battlefield to determine the value of X."
    );
    let mut t = TestGame::new(2);
    let hill_giant = t.hand(P0, "Hill Giant"); // mana value 4: too big
    let courser_card = t.hand(P0, "Centaur Courser"); // mana value 3
    let (paladin, _) = battalion_attack(&mut t, "Paladin Elizabeth Taggerdy");
    destroy(&mut t, paladin);
    assert!(!t.on_battlefield(paladin));
    t.answer_choose(P0, &[Entity::Object(courser_card)]);
    t.resolve_all();
    let courser = t.named_on_battlefield("Centaur Courser");
    assert_eq!(courser.len(), 1);
    assert!(t.obj_now(courser[0]).tapped);
    assert!(t.g.is_attacking(courser[0]));
    assert!(!t.on_battlefield(t.g.current(hill_giant)));
}

#[test]
fn sentinel_sarah_lyons_trigger_works_without_her_or_the_other_attackers() {
    cr!("207.2c", "603.2", "113.7a");
    ruling!(
        "Sentinel Sarah Lyons",
        "Once Sentinel Sarah Lyons’s battalion ability has triggered, it doesn’t matter how many creatures are still attacking when that ability resolves. It also doesn’t matter whether or not Sentinel Sarah Lyons is still attacking or on the battlefield."
    );
    supported("Sentinel Sarah Lyons");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Ornithopter");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let (sarah, bears) = battalion_attack(&mut t, "Sentinel Sarah Lyons");
    destroy(&mut t, sarah);
    destroy(&mut t, bears[0]);
    mtg_engine::combat::remove_from_combat(&mut t.g, bears[1]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn sentinel_sarah_lyons_counts_artifacts_as_the_ability_resolves() {
    cr!("207.2c", "608.2h");
    ruling!(
        "Sentinel Sarah Lyons",
        "Use the number of artifacts you control at the time Sentinel Sarah Lyons’s battalion ability resolves to determine how much damage to deal."
    );
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    battalion_attack(&mut t, "Sentinel Sarah Lyons");
    // One artifact as it triggered; it leaves, and three others arrive, before it resolves.
    destroy(&mut t, thopter);
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Ornithopter");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn frontline_medic_doesnt_protect_creatures_arriving_later() {
    cr!("207.2c", "611.2c");
    ruling!(
        "Frontline Medic",
        "Creatures that come under your control after Frontline Medic's battalion ability resolves will not be granted indestructible by this effect."
    );
    // (The battalion ability compiles; its other ability doesn't matter here.)
    let mut t = TestGame::new(2);
    let (medic, bears) = battalion_attack(&mut t, "Frontline Medic");
    t.resolve_all();
    let late = t.battlefield(P0, "Hill Giant");
    t.g.recompute();
    for id in [medic, bears[0], bears[1]] {
        assert!(t.obj_now(id).chars.has_keyword(KeywordKind::Indestructible));
    }
    assert!(!t.obj_now(late).chars.has_keyword(KeywordKind::Indestructible));
    destroy(&mut t, late);
    destroy(&mut t, medic);
    assert!(!t.on_battlefield(late));
    assert!(t.on_battlefield(medic));
}

#[test]
fn firemane_avenger_gains_life_even_if_the_damage_is_prevented() {
    cr!("207.2c", "615.1", "608.2c");
    ruling!(
        "Firemane Avenger",
        "If Firemane Avenger's battalion ability resolves but some or all of the damage is prevented, you'll still gain 3 life."
    );
    supported("Firemane Avenger");
    supported("Mending Hands");
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(target)]);
    battalion_attack(&mut t, "Firemane Avenger");
    // In response, P1 prevents the next 4 damage to the Hill Giant.
    t.lands(P1, "Plains", 1);
    let mh = t.hand(P1, "Mending Hands");
    t.cast(P1, mh).target(target).go();
    t.resolve();
    t.resolve_all();
    assert_eq!(t.obj_now(target).damage, 0);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn firemane_avenger_gains_no_life_if_its_target_is_illegal() {
    cr!("207.2c", "608.2b");
    ruling!(
        "Firemane Avenger",
        "If the target permanent or player is an illegal target when Firemane Avenger's battalion ability tries to resolve, the ability won't resolve and none of its effects will happen. You won't gain life."
    );
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(target)]);
    battalion_attack(&mut t, "Firemane Avenger");
    destroy(&mut t, target);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}
