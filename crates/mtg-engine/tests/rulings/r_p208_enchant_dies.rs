//! Rulings batch P208 — enchant (CR 603.10a, 603.6c, 704.5m): "when enchanted creature
//! dies" and leaves-the-battlefield triggers when the Aura and the creature leave
//! together, tokens that can't return, and triggers of Auras and enchanted creatures
//! about damage and life.

use crate::r_p208_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s04_common::top_of_stack;
use crate::r_s06_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The Aura and the creature it enchants are destroyed simultaneously, then everything
/// resolves.
fn destroy_together(t: &mut TestGame, aura: ObjectId, creature: ObjectId) {
    t.g.destroy_all(vec![creature, aura], None, false);
    t.g.flush_events();
    t.settle();
    assert!(!t.on_battlefield(aura));
    assert!(!t.on_battlefield(creature));
    t.resolve_all();
}

#[test]
fn dies_triggers_of_auras_destroyed_with_their_creature_still_trigger() {
    cr!("603.10a", "603.6c", "111.1");
    ruling!(
        "Griffin Guide",
        "If Griffin Guide and the enchanted creature go to the graveyard at the same time, Griffin Guide's last ability will trigger."
    );
    ruling!(
        "Fool's Demise",
        "If Fool's Demise and the enchanted creature go to the graveyard from the battlefield at the same time, both abilities will trigger."
    );
    supported("Griffin Guide");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let guide = attach_new(&mut t, P0, "Griffin Guide", bears);
    destroy_together(&mut t, guide, bears);
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.pt(toks[0]), (2, 2));
    assert!(has_kw(&t, toks[0], mtg_engine::keywords::KeywordKind::Flying));

    supported("Fool's Demise");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let demise = attach_new(&mut t, P0, "Fool's Demise", giant);
    destroy_together(&mut t, demise, giant);
    // The creature returns under P0's control; Fool's Demise returns to its owner's hand.
    let back = t.named_on_battlefield("Hill Giant");
    assert_eq!(back.len(), 1);
    assert_eq!(t.g.obj(back[0]).controller, P0);
    assert!(t.in_hand(P0, "Fool's Demise"));
}

#[test]
fn granted_dies_ability_triggers_when_aura_and_creature_are_destroyed_together() {
    cr!("603.10a", "113.6");
    ruling!(
        "Infernal Scarring",
        "If Infernal Scarring and the enchanted creature are destroyed at the same time, the player will draw a card."
    );
    supported("Infernal Scarring");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let scar = attach_new(&mut t, P0, "Infernal Scarring", bears);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    destroy_together(&mut t, scar, bears);
    // The creature's controller (who controlled the granted ability) draws.
    assert_eq!(t.hand_size(P1), h1 + 1);
    assert_eq!(t.hand_size(P0), h0);
}

#[test]
fn viridian_harvest_gains_life_when_put_into_the_graveyard_with_its_artifact() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Viridian Harvest",
        "If the enchanted artifact and Viridian Harvest are put into the graveyard at the same time, you’ll still gain 6 life."
    );
    supported("Viridian Harvest");
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P1, "Ornithopter");
    let harvest = attach_new(&mut t, P0, "Viridian Harvest", relic);
    destroy_together(&mut t, harvest, relic);
    assert_eq!(t.life(P0), 26);
}

#[test]
fn elephant_guide_on_an_opponents_creature_gives_you_the_elephant() {
    cr!("603.3a", "111.2");
    ruling!(
        "Elephant Guide",
        "If Elephant Guide enchants an opponent’s creature, you get the Elephant when that creature dies."
    );
    supported("Elephant Guide");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Elephant Guide", bears);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    assert!(tokens(&t, P1).is_empty());
    assert_eq!(t.obj_now(tokens(&t, P0)[0]).chars.name, "Elephant Token");
}

#[test]
fn tokens_enchanted_by_return_auras_dont_return() {
    cr!("111.7", "111.8", "704.5d");
    ruling!(
        "Minion's Return",
        "If Minion’s Return enchants a token creature, that creature won’t return to the battlefield when it dies."
    );
    ruling!(
        "Unhallowed Pact",
        "If Unhallowed Pact is enchanting a token creature, that creature can’t return to the battlefield."
    );
    for aura in ["Minion's Return", "Unhallowed Pact"] {
        supported(aura);
        let mut t = TestGame::new(2);
        let tok = create_token_named(&mut t, P1);
        attach_new(&mut t, P0, aura, tok);
        destroy(&mut t, tok);
        t.resolve_all();
        assert!(tokens(&t, P0).is_empty(), "{aura}");
        assert!(tokens(&t, P1).is_empty(), "{aura}");
        // A nontoken creature does return.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        attach_new(&mut t, P0, aura, bears);
        destroy(&mut t, bears);
        t.resolve_all();
        let back = t.named_on_battlefield("Grizzly Bears");
        assert_eq!(back.len(), 1, "{aura}");
        assert_eq!(t.g.obj(back[0]).controller, P0);
    }
}

fn create_token_named(t: &mut TestGame, p: PlayerId) -> ObjectId {
    crate::r_s02_common::create_token(t, p, "Soldier")
}

#[test]
fn pain_for_all_triggers_on_lethal_damage() {
    cr!("603.10a", "120.3", "704.5g");
    ruling!(
        "Pain for All",
        "If lethal damage is dealt to the enchanted creature, Pain for All’s last ability still triggers."
    );
    supported("Pain for All");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    attach_new(&mut t, P0, "Pain for All", bears);
    damage(&mut t, giant, 3, bears);
    assert!(!t.on_battlefield(bears));
    t.resolve_all();
    // The Bears (as it last existed) deals 3 damage to each opponent.
    assert_eq!(t.life(P1), 17);
}

#[test]
fn screams_from_within_returns_attached_to_a_creature_you_choose() {
    cr!("303.4f", "303.4g", "608.2c");
    ruling!(
        "Screams from Within",
        "If Screams from Within returns to the battlefield from your graveyard, you choose the creature that Screams from Within enchants"
    );
    supported("Screams from Within");
    // The opponent has no other creatures: it must enchant one of yours.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    attach_new(&mut t, P0, "Screams from Within", bears);
    destroy(&mut t, bears);
    t.resolve_all();
    let s = t.named_on_battlefield("Screams from Within");
    assert_eq!(s.len(), 1, "{}", t.dump_log());
    assert_eq!(attached_to(&t, s[0]), Some(Entity::Object(giant)));
    assert_eq!(t.pt(giant), (2, 2));
    // Nothing it can enchant: it stays in the graveyard.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Screams from Within", bears);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.named_on_battlefield("Screams from Within").is_empty());
    assert!(t.in_graveyard(P0, "Screams from Within"));
}

#[test]
fn light_of_promise_counter_comes_too_late_for_simultaneous_lethal_damage() {
    cr!("510.2", "704.5g", "603.3");
    ruling!(
        "Light of Promise",
        "If enchanted creature is dealt lethal damage at the same time that you gain life, it won't receive a counter from its ability in time to save it."
    );
    supported("Light of Promise");
    supported("Healer's Hawk");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hawk = t.battlefield(P0, "Healer's Hawk");
    let giant = t.battlefield(P1, "Hill Giant");
    attach_new(&mut t, P0, "Light of Promise", bears);
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (hawk, Entity::Player(P1))],
    );
    block_and_finish(&mut t, P1, &[(giant, bears)]);
    assert_eq!(t.life(P0), 21);
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn celestial_mantle_doubles_after_simultaneous_lifelink() {
    cr!("510.2", "702.15b", "603.3");
    ruling!(
        "Celestial Mantle",
        "If a creature dealing combat damage at the same time as the enchanted creature has lifelink, the life gained due to lifelink happens before Celestial Mantle’s triggered ability resolves."
    );
    supported("Celestial Mantle");
    supported("Healer's Hawk");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hawk = t.battlefield(P0, "Healer's Hawk");
    attach_new(&mut t, P0, "Celestial Mantle", bears);
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (hawk, Entity::Player(P1))],
    );
    block_and_finish(&mut t, P1, &[]);
    // 20 + 1 (lifelink), then doubled.
    assert_eq!(t.life(P0), 42);
    assert_eq!(t.life(P1), 20 - 5 - 1);
}

#[test]
fn armadillo_cloak_on_an_opponents_creature_gains_you_life_unless_you_lost() {
    cr!("603.3a", "704.5a", "510.2");
    ruling!(
        "Armadillo Cloak",
        "If Armadillo Cloak enchants a creature you don’t control, you’ll gain life when it deals damage, as long as that damage hasn’t already caused you to lose the game."
    );
    supported("Armadillo Cloak");
    for start in [20, 3] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        t.g.players[P0.idx()].life = start;
        let bears = t.battlefield(P1, "Grizzly Bears");
        attach_new(&mut t, P0, "Armadillo Cloak", bears);
        assert_eq!(t.pt(bears), (4, 4));
        attack_with(&mut t, &[(bears, Entity::Player(P0))]);
        t.answer(
            P0,
            DecisionKind::Blockers,
            mtg_engine::decision::Answer::Blockers(vec![]),
        );
        t.g.run_until(10_000, |g| {
            g.turn.step == Step::EndOfCombat || g.players[0].has_lost
        });
        if start == 20 {
            // Dealt 4, then gains 4 (P0 controls the Cloak's trigger).
            assert_eq!(t.life(P0), 20);
            assert_eq!(t.life(P1), 20);
        } else {
            assert!(t.has_lost(P0));
        }
    }
}

#[test]
fn ice_cage_trigger_resolves_before_the_targeting_spell() {
    cr!("603.3b", "405.5", "608.2b");
    ruling!(
        "Ice Cage",
        "If the enchanted creature becomes the target of a spell or ability, Ice Cage's ability triggers and is put on the stack on top of that spell or ability. Ice Cage's ability will resolve (causing Ice Cage to be destroyed) first."
    );
    supported("Ice Cage");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let cage = attach_new(&mut t, P0, "Ice Cage", giant);
    let shock = crate::r_p209_common::cast_spell(&mut t, P0, "Shock", &[Entity::Object(giant)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_ne!(top_of_stack(&t), shock);
    t.g.resolve_top();
    t.settle();
    assert!(!t.on_battlefield(cage));
    assert_eq!(t.zone(shock), mtg_engine::object::Zone::Stack);
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 2);
}

#[test]
fn trespassers_curse_trigger_order_follows_apnap() {
    cr!("101.4", "603.3b");
    ruling!(
        "Trespasser's Curse",
        "If a creature enters the battlefield under enchanted player’s control and causes an ability controlled by that player to trigger, the triggered ability of Trespasser’s Curse resolves first if it’s that player’s turn, and resolves last if it’s your turn."
    );
    supported("Trespasser's Curse");
    supported("Elvish Visionary");
    for active in [P1, P0] {
        let mut t = TestGame::new(2);
        t.set_step(active, Step::PrecombatMain);
        attach_new(&mut t, P0, "Trespasser's Curse", P1);
        t.enter(P1, "Elvish Visionary");
        t.g.flush_events();
        t.settle();
        assert_eq!(t.stack_len(), 2);
        let top = top_of_stack(&t);
        let top_controller = t.g.obj(top).controller;
        if active == P1 {
            assert_eq!(top_controller, P0, "the Curse resolves first on P1's turn");
        } else {
            assert_eq!(top_controller, P1, "the Curse resolves last on P0's turn");
        }
        let h1 = t.hand_size(P1);
        t.g.resolve_top();
        t.settle();
        if active == P1 {
            assert_eq!(t.life(P1), 19);
            assert_eq!(t.hand_size(P1), h1);
        } else {
            assert_eq!(t.life(P1), 20);
            assert_eq!(t.hand_size(P1), h1 + 1);
        }
        t.resolve_all();
        assert_eq!((t.life(P0), t.life(P1)), (21, 19));
    }
}

#[test]
fn consuming_fervor_counters_stay_and_two_fervors_trigger_twice() {
    cr!("122.1", "113.6", "611.3a");
    ruling!(
        "Consuming Fervor",
        "If Consuming Fervor is removed, the enchanted creature keeps the -1/-1 counters but loses the +3/+3 bonus."
    );
    ruling!(
        "Consuming Fervor",
        "If one creature is enchanted with two Consuming Fervors, it has the triggered ability twice and it gets two -1/-1 counters at the beginning of your upkeep."
    );
    supported("Consuming Fervor");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let f1 = attach_new(&mut t, P0, "Consuming Fervor", giant);
    attach_new(&mut t, P0, "Consuming Fervor", giant);
    assert_eq!(t.pt(giant), (9, 9));
    crate::r_s04_common::next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::MINUS1), 2);
    assert_eq!(t.pt(giant), (7, 7));
    destroy(&mut t, f1);
    assert_eq!(t.counters(giant, counters::MINUS1), 2);
    assert_eq!(t.pt(giant), (4, 4));
}

#[test]
fn convenient_target_leaving_doesnt_end_suspected() {
    cr!("701.60c", "701.60a");
    ruling!(
        "Convenient Target",
        "If Convenient Target leaves the battlefield, the creature it was enchanting will still be suspected"
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let ct = cast_aura_leave_trigger(&mut t, P0, "Convenient Target", giant);
    t.resolve_all();
    assert!(t.obj_now(giant).suspected);
    destroy(&mut t, ct);
    assert!(t.obj_now(giant).suspected);
    assert!(has_kw(&t, giant, mtg_engine::keywords::KeywordKind::Menace));
    assert_eq!(t.pt(giant), (3, 3));
}
