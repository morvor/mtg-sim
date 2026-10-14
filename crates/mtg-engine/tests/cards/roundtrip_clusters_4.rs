//! roundtrip-clusters-4 (docs/agents/items/roundtrip-clusters-4.md): cards of the 3- and
//! 4-card wording clusters, checked in play where the rendering looked doubtful.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn severance_priest_token_uses_the_card_exiled_by_its_enter_ability() {
    cr!("607.2a");
    // "When this creature enters, target opponent reveals their hand. You may choose a
    // nonland card from it. If you do, exile that card. When this creature leaves the
    // battlefield, the exiled card's owner creates an X/X white Spirit creature token,
    // where X is the mana value of the exiled card."
    let mut t = TestGame::new(2);
    let giant = t.hand(P1, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    let priest = t.enter(P0, "Severance Priest");
    t.settle();
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"), "{}", t.dump_log());
    t.g.sacrifice(priest, P0);
    t.settle();
    t.resolve_all();
    let spirits = t.named_on_battlefield("Spirit Token");
    assert_eq!(spirits.len(), 1, "{}", t.dump_log());
    assert_eq!(t.obj_now(spirits[0]).controller, P1);
    assert_eq!(t.pt(spirits[0]), (4, 4));
}

#[test]
fn desecration_demon_each_opponent_may_sacrifice_one_counter() {
    cr!("101.4", "608.2c");
    ruling!(
        "Desecration Demon",
        "Each opponent in turn order may choose to sacrifice a creature"
    );
    // "At the beginning of each combat, any opponent may sacrifice a creature. If a player
    // does, tap this creature and put a +1/+1 counter on it."
    let mut t = TestGame::new(3);
    let demon = t.battlefield(P0, "Desecration Demon");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P2, "Grizzly Bears");
    t.answer_choose(P1, &[Entity::Object(a)]);
    t.answer_choose(P2, &[Entity::Object(b)]);
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to_step(Step::BeginningOfCombat);
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b), "{}", t.dump_log());
    assert!(t.obj_now(demon).tapped);
    assert_eq!(t.counters(demon, "+1/+1"), 1);
}

#[test]
fn desecration_demon_untouched_when_no_one_sacrifices() {
    cr!("608.2c");
    let mut t = TestGame::new(3);
    let demon = t.battlefield(P0, "Desecration Demon");
    let a = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P1, &[]);
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to_step(Step::BeginningOfCombat);
    t.resolve_all();
    assert!(t.on_battlefield(a));
    assert!(!t.obj_now(demon).tapped);
    assert_eq!(t.counters(demon, "+1/+1"), 0);
}

#[test]
fn underworld_fires_exiles_a_creature_it_damaged_that_dies() {
    cr!("614.1a");
    ruling!(
        "Underworld Fires",
        "if they would die for any reason that turn"
    );
    // "Underworld Fires deals 1 damage to each creature and each planeswalker. If a
    // permanent dealt damage this way would die this turn, exile it instead."
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P1, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 2);
    let fires = t.hand(P0, "Underworld Fires");
    if let Err(e) = t.cast_with(P0, fires, &[]) {
        panic!("cast: {e:?}\n{}", t.dump_log());
    }
    t.resolve_all();
    assert!(!t.on_battlefield(elf));
    assert!(t.in_exile("Llanowar Elves"), "{}", t.dump_log());
    // The Giant survived the damage, and still dies into exile this turn.
    t.g.destroy(giant, None);
    t.settle();
    assert!(t.in_exile("Hill Giant"), "{}", t.dump_log());
}

#[test]
fn mammoth_harness_gives_first_strike_to_the_other_creature() {
    cr!("509.3c");
    // "Enchanted creature loses flying. Whenever enchanted creature blocks or becomes
    // blocked by a creature, the other creature gains first strike until end of turn."
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Forest", 4);
    let harness = t.hand(P0, "Mammoth Harness");
    if let Err(e) = t.cast_with(P0, harness, &[Entity::Object(attacker)]) {
        panic!("cast: {e:?}\n{}", t.dump_log());
    }
    t.resolve_all();
    t.attack(&[(attacker, Entity::Player(P1))], &[(blocker, attacker)]);
    t.resolve_all();
    let has_first_strike =
        |t: &TestGame, o| t.obj_now(o).has_keyword(mtg_engine::keywords::KeywordKind::FirstStrike);
    assert!(has_first_strike(&t, blocker), "{}", t.dump_log());
    assert!(!has_first_strike(&t, attacker));
}

#[test]
fn refurbished_familiar_draws_for_each_opponent_who_cant_discard() {
    cr!("701.9a");
    // "When this creature enters, each opponent discards a card. For each opponent who
    // can't, you draw a card."
    let mut t = TestGame::new(3);
    t.hand(P1, "Grizzly Bears");
    let before = t.hand_size(P0);
    t.enter(P0, "Refurbished Familiar");
    t.settle();
    t.resolve_all();
    // P1 discarded the Bears; P2 had no card to discard.
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), before + 1, "{}", t.dump_log());
}

#[test]
fn trial_of_agony_controller_picks_the_creature_dealt_damage() {
    cr!("115.1", "608.2d");
    // "Choose two target creatures controlled by the same opponent. That player chooses one
    // of those creatures. Trial of Agony deals 5 damage to that creature, and the other
    // can't block this turn."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 1);
    let trial = t.hand(P0, "Trial of Agony");
    t.answer_choose(P1, &[Entity::Object(b)]);
    if let Err(e) = t.cast_with(P0, trial, &[Entity::Object(a), Entity::Object(b)]) {
        panic!("cast: {e:?}\n{}", t.dump_log());
    }
    t.resolve_all();
    assert!(t.on_battlefield(a));
    assert!(!t.on_battlefield(b), "{}", t.dump_log());
}
