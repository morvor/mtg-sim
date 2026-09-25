//! CR 702.21 Ward.

use crate::common_k702_011_017::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Number of ward triggers on the stack.
fn ward_triggers_on_stack(t: &TestGame) -> usize {
    t.g.stack
        .iter()
        .filter(|id| {
            t.g.obj(**id)
                .stack
                .as_ref()
                .is_some_and(|si| matches!(&si.kind, object::StackKind::Triggered { ability, .. } if ability.text == "Ward"))
        })
        .count()
}

#[test]
fn ward_counters_a_spell_unless_its_controller_pays() {
    cr!("702.21", "702.21a");
    assert_supported("Rimeshield Frost Giant");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Rimeshield Frost Giant");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(giant).go();
    t.settle();
    // Ward triggers when the giant becomes the target of an opponent's spell.
    assert_eq!(ward_triggers_on_stack(&t), 1);
    // P1 would like to pay {3} but has no mana left.
    t.answer_yes(P1, true);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert_eq!(t.obj_now(giant).damage, 0);
}

#[test]
fn paying_the_ward_cost_lets_the_spell_resolve() {
    cr!("702.21a");
    assert_supported("Tomakul Honor Guard");
    let mut t = TestGame::new(2);
    let guard = t.battlefield(P0, "Tomakul Honor Guard");
    t.lands(P1, "Mountain", 3);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(guard).go();
    t.answer_yes(P1, true);
    t.resolve_all();
    assert!(!t.on_battlefield(guard));
    assert!(t.in_graveyard(P0, "Tomakul Honor Guard"));
    // All three Mountains were used: one for the Bolt, two for ward.
    assert!(t
        .g
        .battlefield
        .iter()
        .filter(|id| t.g.obj(**id).controller == P1)
        .all(|id| t.g.obj(*id).tapped));
}

#[test]
fn declining_to_pay_counters_the_spell() {
    cr!("702.21a");
    let mut t = TestGame::new(2);
    let guard = t.battlefield(P0, "Tomakul Honor Guard");
    t.lands(P1, "Mountain", 3);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(guard).go();
    t.answer_yes(P1, false);
    t.resolve_all();
    assert!(t.on_battlefield(guard));
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
}

#[test]
fn ward_doesnt_trigger_on_its_controllers_spells() {
    cr!("702.21a");
    let mut t = TestGame::new(2);
    let guard = t.battlefield(P0, "Tomakul Honor Guard");
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(guard).go();
    t.settle();
    assert_eq!(ward_triggers_on_stack(&t), 0);
    t.resolve_all();
    assert_eq!(t.pt(guard), (6, 4));
}

#[test]
fn ward_counters_abilities_too() {
    cr!("702.21a");
    let mut t = TestGame::new(2);
    let guard = t.battlefield(P0, "Tomakul Honor Guard");
    let pyromancer = t.battlefield(P1, "Prodigal Pyromancer");
    t.activate(P1, pyromancer, 0, &[Entity::Object(guard)])
        .unwrap();
    t.settle();
    assert_eq!(ward_triggers_on_stack(&t), 1);
    t.resolve_all();
    // The ability was countered: the 3/1 took no damage.
    assert!(t.on_battlefield(guard));
    assert_eq!(t.obj_now(guard).damage, 0);
    assert!(t.obj_now(pyromancer).tapped);
}

#[test]
fn ward_with_a_life_cost() {
    cr!("702.21a");
    assert_supported("Owlin Shieldmage");
    let mut t = TestGame::new(2);
    let owlin = t.battlefield(P0, "Owlin Shieldmage");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(owlin).go();
    t.answer_yes(P1, true);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(!t.on_battlefield(owlin));
}

#[test]
fn each_targeted_ward_permanent_triggers_and_all_must_be_paid() {
    cr!("702.21a");
    ruling!(
        "Owlin Shieldmage",
        "each of those ward abilities will trigger. If that player doesn’t pay for all of them, the spell will be countered"
    );
    let mut t = TestGame::new(2);
    let g1 = t.battlefield(P0, "Tomakul Honor Guard");
    let g2 = t.battlefield(P0, "Tomakul Honor Guard");
    let g3 = t.battlefield(P0, "Tomakul Honor Guard");
    // Seeds of Strength plus one ward payment, but not three.
    t.lands(P1, "Forest", 3);
    t.lands(P1, "Plains", 1);
    let seeds = t.hand(P1, "Seeds of Strength");
    t.cast(P1, seeds).target(g1).target(g2).target(g3).go();
    t.settle();
    assert_eq!(ward_triggers_on_stack(&t), 3);
    for _ in 0..3 {
        t.answer_yes(P1, true);
    }
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Seeds of Strength"));
    for g in [g1, g2, g3] {
        assert_eq!(t.pt(g), (3, 1));
    }
}

#[test]
fn multiple_ward_abilities_trigger_separately() {
    cr!("702.21a");
    ruling!(
        "Rith, Liberated Primeval",
        "If a permanent has more than one ward ability, the abilities trigger individually"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rith, Liberated Primeval");
    let dragon = t.battlefield(P0, "Archive Dragon");
    assert_eq!(keyword_count(&t, dragon, KeywordKind::Ward), 2);
    t.lands(P1, "Mountain", 3);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(dragon).go();
    t.settle();
    assert_eq!(ward_triggers_on_stack(&t), 2);
    // P1 pays {2} for the first one, but can't pay for the second.
    t.answer_yes(P1, true);
    t.answer_yes(P1, true);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert_eq!(t.obj_now(dragon).damage, 0);
}

#[test]
fn ward_resolves_even_if_the_permanent_loses_ward() {
    cr!("702.21a");
    ruling!(
        "Bronze Guardian",
        "Once a ward ability has triggered, it doesn't matter if the artifact loses ward"
    );
    assert_supported("Bronze Guardian");
    let mut t = TestGame::new(2);
    let guardian = t.battlefield(P0, "Bronze Guardian");
    let thopter = t.battlefield(P0, "Ornithopter");
    assert!(t.obj_now(thopter).has_keyword(KeywordKind::Ward));
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(thopter).go();
    t.settle();
    assert_eq!(ward_triggers_on_stack(&t), 1);
    // Bronze Guardian leaves the battlefield in response.
    t.g.destroy(guardian, None);
    t.g.recompute();
    assert!(!t.obj_now(thopter).has_keyword(KeywordKind::Ward));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert!(t.on_battlefield(thopter));
}

#[test]
fn a_spell_that_cant_be_countered_isnt_countered_by_ward() {
    cr!("702.21a");
    ruling!(
        "Raze to the Ground",
        "\"This spell can't be countered\" means it can't be countered even by the ward keyword ability"
    );
    assert_supported("Abrupt Decay");
    let mut t = TestGame::new(2);
    let guard = t.battlefield(P0, "Tomakul Honor Guard");
    t.lands(P1, "Swamp", 1);
    t.lands(P1, "Forest", 1);
    let decay = t.hand(P1, "Abrupt Decay");
    t.cast(P1, decay).target(guard).go();
    t.settle();
    assert_eq!(ward_triggers_on_stack(&t), 1);
    t.answer_yes(P1, false);
    t.resolve_all();
    assert!(!t.on_battlefield(guard));
}

#[test]
fn ward_x_is_determined_as_the_ability_resolves() {
    cr!("702.21b");
    let mut t = TestGame::new(2);
    let minthara = t.battlefield(P0, "Minthara, Merciless Soul");
    let c = mtg_engine::card::card("Minthara, Merciless Soul");
    assert!(!c
        .unsupported_text()
        .iter()
        .any(|u| u.to_lowercase().contains("ward")));
    t.g.add_counters(Entity::Player(P0), "experience", 1, None);
    t.lands(P1, "Mountain", 2);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(minthara).go();
    t.settle();
    assert_eq!(ward_triggers_on_stack(&t), 1);
    // X was 1 when it triggered; P0 gets another experience counter before it resolves.
    t.g.add_counters(Entity::Player(P0), "experience", 1, None);
    t.answer_yes(P1, true);
    t.resolve_all();
    // X is 2 now: P1's one remaining Mountain can't pay it.
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert!(t.on_battlefield(minthara));
    // No one was asked to choose X.
    assert!(!t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, decision::Decision::ChooseX { .. })));
}

#[test]
fn ward_x_paid_at_its_value_on_resolution() {
    cr!("702.21b");
    let mut t = TestGame::new(2);
    let minthara = t.battlefield(P0, "Minthara, Merciless Soul");
    t.g.add_counters(Entity::Player(P0), "experience", 2, None);
    t.lands(P1, "Mountain", 3);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(minthara).go();
    t.answer_yes(P1, true);
    t.resolve_all();
    assert!(!t.on_battlefield(minthara));
}

#[test]
fn ward_cost_based_on_power_uses_power_on_resolution() {
    cr!("702.21a", "702.21b");
    ruling!(
        "Phyrexian Fleshgorger",
        "equal to Phyrexian Fleshgorger's power at the time the ward ability resolves"
    );
    let mut t = TestGame::new(2);
    let gorger = t.battlefield(P0, "Phyrexian Fleshgorger");
    assert_eq!(t.pt(gorger).0, 7);
    t.g.player_mut(P1).life = 9;
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(gorger).go();
    t.settle();
    // In response, P0 pumps it: the ward cost becomes 10 life.
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(gorger).go();
    t.resolve();
    assert_eq!(t.pt(gorger).0, 10);
    t.answer_yes(P1, true);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert_eq!(t.life(P1), 9);
    assert_eq!(t.obj_now(gorger).damage, 0);
}
