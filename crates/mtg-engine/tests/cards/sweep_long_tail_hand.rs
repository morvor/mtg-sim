//! Putting cards from a hand onto the battlefield: "You may put a land card from your hand
//! onto the battlefield." (Sakura-Tribe Scout), "Put up to two creature cards from your
//! hand onto the battlefield." (Tooth and Nail), "... onto the battlefield tapped"
//! (Arboreal Grazer), "that player may put ... from their hand" (Braids, Conjurer Adept),
//! and the Sneak Attack family ("That creature gains haste. Sacrifice the creature at the
//! beginning of the next end step.").

use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
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

/// The candidates of the most recent "choose entities" decision asked of `p`.
fn last_choice_of(t: &TestGame, p: PlayerId) -> Vec<Entity> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(q, d)| match d {
            Decision::ChooseEntities { candidates, .. } if q == p => Some(candidates),
            _ => None,
        })
        .expect("no choice was asked")
}

#[test]
fn sakura_tribe_scout_puts_a_land_without_playing_it() {
    cr!("305.4");
    assert_supported("Sakura-Tribe Scout");
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P0, "Sakura-Tribe Scout");
    let forest = t.hand(P0, "Forest");
    t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.activate(P0, scout, 0, &[]).unwrap();
    t.resolve();
    // Only the land card could be chosen.
    assert_eq!(last_choice_of(&t, P0), vec![Entity::Object(forest)]);
    let forests = t.named_on_battlefield("Forest");
    assert_eq!(forests.len(), 1);
    assert_eq!(t.obj_now(forests[0]).controller, P0);
    assert!(!t.obj_now(forests[0]).tapped);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // It wasn't a land play: P0 can still play a land this turn.
    assert_eq!(t.g.player(P0).lands_played_this_turn, 0);
    let mountain = t.hand(P0, "Mountain");
    t.play_land(P0, mountain).unwrap();
    assert_eq!(t.named_on_battlefield("Mountain").len(), 1);
}

#[test]
fn sakura_tribe_scout_may_decline() {
    cr!("608.2d");
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P0, "Sakura-Tribe Scout");
    t.hand(P0, "Forest");
    t.answer_yes(P0, false);
    t.activate(P0, scout, 0, &[]).unwrap();
    t.resolve();
    assert!(t.in_hand(P0, "Forest"));
    assert!(t.named_on_battlefield("Forest").is_empty());
}

#[test]
fn arboreal_grazer_land_enters_tapped() {
    cr!("110.5b");
    assert_supported("Arboreal Grazer");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let grazer = t.hand(P0, "Arboreal Grazer");
    let island = t.hand(P0, "Island");
    t.cast(P0, grazer).go();
    t.resolve();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(island)]);
    t.resolve_all();
    let islands = t.named_on_battlefield("Island");
    assert_eq!(islands.len(), 1, "{}", t.dump_log());
    assert!(t.obj_now(islands[0]).tapped);
}

#[test]
fn tooth_and_nail_puts_up_to_two_creatures() {
    cr!("110.2a");
    assert_supported("Tooth and Nail");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 7);
    let bears = t.hand(P0, "Grizzly Bears");
    let dragon = t.hand(P0, "Shivan Dragon");
    t.hand(P0, "Forest");
    let tn = t.hand(P0, "Tooth and Nail");
    t.answer_choose(P0, &[Entity::Object(bears), Entity::Object(dragon)]);
    t.cast(P0, tn).modes(&[1]).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Shivan Dragon").len(), 1);
    // Only creature cards were offered, and choosing fewer was allowed.
    let asked = t
        .asked()
        .into_iter()
        .rev()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { min, max, candidates, .. } => Some((min, max, candidates)),
            _ => None,
        })
        .unwrap();
    assert_eq!((asked.0, asked.1), (0, 2));
    assert_eq!(asked.2.len(), 2);
    assert!(t.in_hand(P0, "Forest"));
}

#[test]
fn braids_lets_each_player_put_a_card_on_their_upkeep() {
    cr!("110.2a", "305.4");
    ruling!(
        "Braids, Conjurer Adept",
        "Braids's effect doesn't count as playing a land if you put one onto the battlefield with it."
    );
    assert_supported("Braids, Conjurer Adept");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Braids, Conjurer Adept");
    let island = t.hand(P1, "Island");
    let bolt = t.hand(P1, "Lightning Bolt");
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(island)]);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    // The opponent chose (among artifact, creature, and land cards) and controls it.
    assert_eq!(last_choice_of(&t, P1), vec![Entity::Object(island)]);
    let islands = t.named_on_battlefield("Island");
    assert_eq!(islands.len(), 1, "{}", t.dump_log());
    assert_eq!(t.obj_now(islands[0]).controller, P1);
    assert_eq!(t.zone(bolt), Zone::Hand(P1));
    assert_eq!(t.g.player(P1).lands_played_this_turn, 0);
}

#[test]
fn sneak_attack_haste_then_sacrifice_at_end_step() {
    cr!("603.7a");
    assert_supported("Sneak Attack");
    let mut t = TestGame::new(2);
    let sneak = t.battlefield(P0, "Sneak Attack");
    t.lands(P0, "Mountain", 1);
    let dragon = t.hand(P0, "Shivan Dragon");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(dragon)]);
    t.activate(P0, sneak, 0, &[]).unwrap();
    t.resolve();
    let d = t.named_on_battlefield("Shivan Dragon");
    assert_eq!(d.len(), 1, "{}", t.dump_log());
    assert!(t.obj_now(d[0]).has_keyword(KeywordKind::Haste));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Shivan Dragon"), "{}", t.dump_log());
}

#[test]
fn sneak_attack_creature_that_left_and_returned_isnt_sacrificed() {
    cr!("400.7", "603.7a");
    ruling!(
        "Sneak Attack",
        "If that creature has left the battlefield, even if it came back, you don't sacrifice it."
    );
    let mut t = TestGame::new(2);
    let sneak = t.battlefield(P0, "Sneak Attack");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Plains", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, sneak, 0, &[]).unwrap();
    t.resolve();
    let b = t.named_on_battlefield("Grizzly Bears")[0];
    // Flicker it: it's a new object (CR 400.7).
    let shift = t.hand(P0, "Cloudshift");
    t.cast(P0, shift).target(b).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(
        t.named_on_battlefield("Grizzly Bears").len(),
        1,
        "{}",
        t.dump_log()
    );
}

#[test]
fn purphoros_red_or_artifact_creature_card() {
    cr!("110.2a");
    assert_supported("Purphoros, Bronze-Blooded");
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Purphoros, Bronze-Blooded");
    t.lands(P0, "Mountain", 3);
    let goblin = t.hand(P0, "Goblin Piker");
    let golem = t.hand(P0, "Bronze Sable");
    t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(golem)]);
    t.activate(P0, p, 0, &[]).unwrap();
    t.resolve();
    // A red creature card or an artifact creature card, not a green one.
    let mut offered = last_choice_of(&t, P0);
    offered.sort();
    let mut expected = vec![Entity::Object(goblin), Entity::Object(golem)];
    expected.sort();
    assert_eq!(offered, expected);
    assert_eq!(t.named_on_battlefield("Bronze Sable").len(), 1);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Bronze Sable"), "{}", t.dump_log());
}

#[test]
fn braids_permanent_put_during_upkeep_misses_that_upkeeps_trigger() {
    cr!("603.2");
    ruling!(
        "Braids, Conjurer Adept",
        "it won't trigger during that upkeep"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Braids, Conjurer Adept");
    let djinn = t.hand(P1, "Juzám Djinn");
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(djinn)]);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Juzám Djinn").len(), 1, "{}", t.dump_log());
    t.advance_to(P1, Step::Draw);
    // "At the beginning of your upkeep" had already happened: no damage this turn.
    assert_eq!(t.life(P1), 20, "{}", t.dump_log());
    // It triggers on P1's next upkeep.
    t.answer_yes(P1, false);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 19, "{}", t.dump_log());
}
