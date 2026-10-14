//! roundtrip-gaps-3 (docs/agents/items/roundtrip-gaps-3.md): the parser bugs behind the
//! new renderer gap clusters (history conditions, cast history, opt-in choices, one-use
//! replacement effects).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn shape_of_the_wiitigo_looks_at_whether_the_enchanted_creature_attacked() {
    cr!("608.2c", "303.4");
    // "At the beginning of your upkeep, put a +1/+1 counter on enchanted creature if it
    // attacked or blocked since your last upkeep. Otherwise, remove a +1/+1 counter from
    // it." "It" is the enchanted creature, not the Aura (which never attacks).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Shape of the Wiitigo");
    assert!(t.g.attach(aura, Entity::Object(bears)));
    t.g.add_counters(Entity::Object(bears), "+1/+1", 2, None);
    t.set_step(P0, Step::PrecombatMain);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 3, "{}", t.dump_log());
}

#[test]
fn regna_counts_life_a_teammate_gained() {
    cr!("102.4", "603.4");
    // "At the beginning of each end step, if your team gained life this turn, create two
    // 1/1 white Warrior creature tokens." In Two-Headed Giant, your teammate gaining life
    // is your team gaining life.
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    t.battlefield(P0, "Regna, the Redeemer");
    t.set_step(P0, Step::PrecombatMain);
    t.g.gain_life(P1, 3);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(warriors(&t), 2, "{}", t.dump_log());
}

fn warriors(t: &TestGame) -> usize {
    t.named_on_battlefield("Warrior Token").len()
}

#[test]
fn regna_ignores_an_opponents_life_gain() {
    cr!("102.4");
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    t.battlefield(P0, "Regna, the Redeemer");
    t.set_step(P0, Step::PrecombatMain);
    t.g.gain_life(P2, 3);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(warriors(&t), 0, "{}", t.dump_log());
}

/// `p` casts Reanimate on `card` (in some graveyard).
fn reanimate(t: &mut TestGame, p: PlayerId, card: ObjectId) {
    t.lands(p, "Swamp", 1);
    let r = t.hand(p, "Reanimate");
    t.cast(p, r).target(Entity::Object(card)).go();
    t.resolve_all();
}

#[test]
fn prized_amalgam_ignores_a_creature_from_another_players_graveyard() {
    cr!("400.3", "603.4");
    // "Whenever a creature enters, if it entered from your graveyard or you cast it from
    // your graveyard, return this card from your graveyard to the battlefield tapped at
    // the beginning of the next end step." A card in an opponent's graveyard isn't in
    // yours.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Prized Amalgam");
    let bears = t.graveyard(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    reanimate(&mut t, P1, bears);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.in_graveyard(P0, "Prized Amalgam"), "{}", t.dump_log());
}

#[test]
fn prized_amalgam_returns_for_a_creature_from_your_graveyard() {
    cr!("603.4");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Prized Amalgam");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    reanimate(&mut t, P1, bears);
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.named_on_battlefield("Prized Amalgam").len(), 1, "{}", t.dump_log());
}

#[test]
fn mog_sees_the_cards_every_player_discarded() {
    cr!("101.4", "608.2c");
    // "At the beginning of your end step, each player may discard a card. Each player who
    // discarded a card this way draws a card. If a creature card was discarded this way,
    // you create a 1/2 white Moogle creature token with lifelink. Then if a noncreature
    // card was discarded this way, put a +1/+1 counter on each Moogle you control."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mog, Moogle Warrior");
    t.hand(P0, "Grizzly Bears");
    t.hand(P1, "Forest");
    t.answer_yes(P0, true);
    t.answer_yes(P1, true);
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "{}", t.dump_log());
    assert!(t.in_graveyard(P1, "Forest"), "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.hand_size(P1), 1);
    let moogles = t.named_on_battlefield("Moogle Token");
    assert_eq!(moogles.len(), 1, "{}", t.dump_log());
    assert_eq!(t.counters(moogles[0], "+1/+1"), 1, "{}", t.dump_log());
}

#[test]
fn behemoth_of_vault_0_destroys_the_dies_triggers_target() {
    cr!("603.12", "115.1");
    ruling!(
        "Behemoth of Vault 0",
        "You choose the target of Behemoth of Vault 0’s last ability when it triggers"
    );
    // "When this creature dies, you may pay an amount of {E} equal to target nonland
    // permanent's mana value. When you do, destroy that permanent."
    let mut t = TestGame::new(2);
    let behemoth = t.battlefield(P0, "Behemoth of Vault 0");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(Entity::Player(P0), "energy", 3, None);
    t.set_step(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    t.g.destroy(behemoth, None);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "{}", t.dump_log());
    assert_eq!(t.g.player(P0).counter("energy"), 1);
}

#[test]
fn tempt_with_glory_counts_an_opponent_who_accepted_with_no_creatures() {
    cr!("118.12", "101.4");
    ruling!(
        "Tempt with Glory",
        "the effect happens again for you a number of times equal to the number of opponents who accepted"
    );
    // "Put a +1/+1 counter on each creature you control. Each opponent may put a +1/+1
    // counter on each creature they control. For each opponent who does, put a +1/+1
    // counter on each creature you control." Putting a counter on each of no creatures is
    // still doing it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Plains", 6);
    let c = t.hand(P0, "Tempt with Glory");
    t.answer_yes(P1, true);
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 2, "{}", t.dump_log());
}

#[test]
fn fyndhorn_druid_remembers_being_blocked_after_it_dies() {
    cr!("603.10a", "603.4");
    // "When this creature dies, if it was blocked this turn, you gain 4 life."
    let mut t = TestGame::new(2);
    let druid = t.battlefield(P0, "Fyndhorn Druid");
    let wall = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::PrecombatMain);
    let life = t.life(P0);
    t.attack(&[(druid, Entity::Player(P1))], &[(wall, druid)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Fyndhorn Druid"), "{}", t.dump_log());
    assert_eq!(t.life(P0), life + 4, "{}", t.dump_log());
}

#[test]
fn wave_of_rats_returns_after_dealing_combat_damage_to_a_player() {
    cr!("603.10a", "603.4");
    // "When this creature dies, if it dealt combat damage to a player this turn, return it
    // to the battlefield under its owner's control."
    let mut t = TestGame::new(2);
    let rats = t.battlefield(P0, "Wave of Rats");
    t.set_step(P0, Step::PrecombatMain);
    t.attack(&[(rats, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    let now = t.g.current(rats);
    t.g.destroy(now, None);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Wave of Rats").len(), 1, "{}", t.dump_log());
}

#[test]
fn samut_draws_for_a_creature_that_entered_this_turn() {
    cr!("603.4");
    // "Whenever a creature you control deals combat damage to a player, if that creature
    // entered this turn, draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Samut, Vizier of Naktamun");
    let old = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    let new = t.enter(P0, "Raging Goblin");
    t.settle();
    let hand = t.hand_size(P0);
    t.attack(&[(old, Entity::Player(P1)), (new, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1, "{}", t.dump_log());
}

#[test]
fn barrin_draws_after_a_permanent_was_returned_to_your_hand() {
    cr!("603.4", "400.7");
    // "At the beginning of your end step, if a permanent was put into your hand from the
    // battlefield this turn, draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Barrin, Tolarian Archmage");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::PostcombatMain);
    t.lands(P0, "Island", 1);
    let u = t.hand(P0, "Unsummon");
    t.cast(P0, u).target(Entity::Object(bears)).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1, "{}", t.dump_log());
}
