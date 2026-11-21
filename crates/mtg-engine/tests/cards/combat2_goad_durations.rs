//! Goad with durations other than "until your next turn", and ending goad and suspicion
//! (CR 701.15, 701.60): Jon Irenicus, Life of the Party, Nettling Nuisance, The War Games,
//! Hot Pursuit, Vislor Turlough, Serene Sleuth, Eliminate the Impossible.

use crate::combat2_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn suspected(t: &TestGame, id: ObjectId) -> bool {
    t.g.obj(id).suspected
}

#[test]
fn jon_irenicus_goads_the_creature_it_gives_away_for_the_rest_of_the_game() {
    cr!("701.15a", "701.15b", "508.1d");
    supported("Jon Irenicus, Shattered One");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Jon Irenicus, Shattered One");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(controller(&t, bears), P1);
    assert_eq!(t.counters(bears, "+1/+1"), 2);
    assert!(t.obj_now(bears).tapped);
    assert_eq!(t.g.goaders(bears), vec![P0]);
    // It gained "This creature can't be sacrificed".
    let before = t.g.obj(bears).zone;
    t.g.sacrifice(bears, P1);
    assert_eq!(t.g.obj(bears).zone, before);
    // P1's turn: it must attack a player other than P0 (P2) if able.
    t.advance_to(P1, Step::BeginningOfCombat);
    assert!(!legal_attack(&mut t, &[]));
    assert!(!legal_attack(&mut t, &[(bears, Entity::Player(P0))]));
    assert!(legal_attack(&mut t, &[(bears, Entity::Player(P2))]));
    // A goad from an action would end as P0's next turn begins; this one doesn't.
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.g.goaders(bears), vec![P0]);
    t.advance_to(P1, Step::BeginningOfCombat);
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &[(bears, Entity::Player(P2))]));
}

#[test]
fn jon_irenicus_with_no_creature_chosen_gives_nothing_away() {
    cr!("115.1");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jon Irenicus, Shattered One");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(controller(&t, bears), P0);
    assert!(t.g.goaders(bears).is_empty());
}

#[test]
fn life_of_the_party_goads_each_opponents_copy_for_the_rest_of_the_game() {
    cr!("701.15a", "701.15b", "707.2");
    ruling!(
        "Life of the Party",
        "If something is goaded but the player who goaded it has left the game, it still must attack one of the other players if able."
    );
    supported("Life of the Party");
    let mut t = TestGame::new(3);
    let life = t.enter(P0, "Life of the Party");
    t.resolve_all();
    let copies: Vec<ObjectId> = t
        .named_on_battlefield("Life of the Party")
        .into_iter()
        .filter(|o| *o != life)
        .collect();
    assert_eq!(copies.len(), 2);
    let mut owners: Vec<PlayerId> = copies.iter().map(|c| controller(&t, *c)).collect();
    owners.sort();
    assert_eq!(owners, vec![P1, P2]);
    for c in &copies {
        assert_eq!(t.g.goaders(*c), vec![P0]);
    }
    assert!(t.g.goaders(life).is_empty());
    // Still goaded after P0's next turn has begun.
    t.advance_to(P0, Step::Upkeep);
    let p1_copy = *copies.iter().find(|c| controller(&t, **c) == P1).unwrap();
    assert_eq!(t.g.goaders(p1_copy), vec![P0]);
    // P0 leaves the game: the copy still must attack (P2) if able.
    t.g.lose_game(P0);
    t.advance_to(P1, Step::BeginningOfCombat);
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &[(p1_copy, Entity::Player(P2))]));
}

#[test]
fn nettling_nuisance_pirate_is_goaded_for_the_rest_of_the_game() {
    cr!("701.15a", "701.15b");
    supported("Nettling Nuisance");
    let mut t = TestGame::new(2);
    let faerie = t.battlefield(P0, "Nettling Nuisance");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(faerie, Entity::Player(P1))], &[]);
    t.resolve_all();
    let pirates: Vec<ObjectId> =
        t.g.permanents()
            .filter(|o| o.controller == P1 && o.is_creature())
            .map(|o| o.id)
            .collect();
    assert_eq!(pirates.len(), 1);
    let pirate = pirates[0];
    assert_eq!(t.pt(pirate), (4, 2));
    assert_eq!(t.g.goaders(pirate), vec![P0]);
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::BeginningOfCombat);
    assert_eq!(t.g.goaders(pirate), vec![P0]);
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &[(pirate, Entity::Player(P0))]));
}

#[test]
fn the_war_games_goads_the_warriors_while_it_remains_on_the_battlefield() {
    cr!("701.15b", "611.2b", "714.2b");
    supported("The War Games");
    let mut t = TestGame::new(3);
    let saga = t.enter(P0, "The War Games");
    t.resolve_all();
    for p in [P0, P1, P2] {
        let warriors: Vec<ObjectId> =
            t.g.permanents()
                .filter(|o| o.controller == p && o.is_creature())
                .map(|o| o.id)
                .collect();
        assert_eq!(warriors.len(), 3, "{p}");
        for w in warriors {
            assert!(t.obj_now(w).tapped);
            assert_eq!(t.g.goaders(w), vec![P0]);
        }
    }
    // P0's own Warriors (untapped and no longer summoning sick on P0's next turn) must
    // attack a player other than P0 if able.
    let mine: Vec<ObjectId> =
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.is_creature())
            .map(|o| o.id)
            .collect();
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::BeginningOfCombat);
    for w in &mine {
        assert_eq!(t.g.goaders(*w), vec![P0]);
    }
    assert!(!legal_attack(&mut t, &[]));
    // The Saga leaves the battlefield: the Warriors are no longer goaded.
    t.g.destroy(saga, None);
    t.settle();
    for w in &mine {
        assert!(t.g.goaders(*w).is_empty());
    }
    to_combat(&mut t, P0);
    assert!(legal_attack(&mut t, &[]));
}

#[test]
fn hot_pursuit_goads_the_suspect_while_it_remains_on_the_battlefield() {
    cr!("701.60a", "701.15b", "611.2b");
    supported("Grizzly Bears");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let hp = t.enter(P0, "Hot Pursuit");
    t.resolve_all();
    assert!(suspected(&t, bears));
    assert_eq!(t.g.goaders(bears), vec![P0]);
    t.g.destroy(hp, None);
    t.settle();
    assert!(suspected(&t, bears));
    assert!(t.g.goaders(bears).is_empty());
}

#[test]
fn hot_pursuit_gone_before_its_trigger_resolves_only_suspects() {
    cr!("611.2b", "701.60a");
    ruling!(
        "Hot Pursuit",
        "If Hot Pursuit leaves the battlefield before its first triggered ability resolves, the target creature will become suspected, but it won’t become goaded at all."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let hp = t.enter(P0, "Hot Pursuit");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.g.destroy(hp, None);
    t.resolve_all();
    assert!(suspected(&t, bears));
    assert!(t.g.goaders(bears).is_empty());
}

#[test]
fn vislor_turlough_is_goaded_for_as_long_as_the_opponent_controls_it() {
    cr!("701.15b", "611.2b");
    supported("Vislor Turlough");
    supported("Act of Treason");
    let mut t = TestGame::new(3);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    let vislor = t.enter(P0, "Vislor Turlough");
    t.resolve_all();
    assert_eq!(controller(&t, vislor), P2);
    assert_eq!(t.g.goaders(vislor), vec![P0]);
    // Goaded across P0's next turn too.
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.g.goaders(vislor), vec![P0]);
    // P1 takes it: the goad ends, and doesn't come back when P2 gets it back.
    t.advance_to(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 3);
    let act = t.hand(P1, "Act of Treason");
    t.cast(P1, act).target(vislor).go();
    t.resolve();
    assert_eq!(controller(&t, vislor), P1);
    assert!(t.g.goaders(vislor).is_empty());
    t.advance_to(P2, Step::Upkeep);
    assert_eq!(controller(&t, vislor), P2);
    assert!(t.g.goaders(vislor).is_empty());
}

#[test]
fn vislor_turlough_kept_isnt_goaded() {
    cr!("701.15a");
    let mut t = TestGame::new(2);
    t.answer_yes(P0, false);
    let vislor = t.enter(P0, "Vislor Turlough");
    t.resolve_all();
    assert_eq!(controller(&t, vislor), P0);
    assert!(t.g.goaders(vislor).is_empty());
}

#[test]
fn serene_sleuth_investigates_for_goaded_creatures_then_ends_their_goad() {
    cr!("701.15a", "701.16a");
    supported("Serene Sleuth");
    supported("Disrupt Decorum");
    let mut t = TestGame::new(2);
    let sleuth = t.battlefield(P0, "Serene Sleuth");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    // P1 goads all of P0's creatures.
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 4);
    let dd = t.hand(P1, "Disrupt Decorum");
    t.cast(P1, dd).go();
    t.resolve_all();
    for c in [sleuth, bears, elves] {
        assert_eq!(t.g.goaders(c), vec![P1]);
    }
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let clues =
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.chars.has_subtype("Clue"))
            .count();
    assert_eq!(clues, 3);
    for c in [sleuth, bears, elves] {
        assert!(t.g.goaders(c).is_empty());
    }
    // Nothing has to attack now.
    assert!(legal_attack(&mut t, &[]));
}

#[test]
fn eliminate_the_impossible_ends_suspicion_of_opponents_creatures() {
    cr!("701.60a", "701.60c");
    supported("Eliminate the Impossible");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let mine = t.battlefield(P0, "Grizzly Bears");
    for c in [bears, mine] {
        mtg_engine::kwa::suspect_detain::suspect(&mut t.g, c);
    }
    t.settle();
    assert!(suspected(&t, bears) && suspected(&t, mine));
    t.lands(P0, "Island", 2);
    let spell = t.hand(P0, "Eliminate the Impossible");
    t.cast(P0, spell).go();
    t.resolve();
    assert!(!suspected(&t, bears));
    assert!(!suspected(&t, giant));
    // Only the opponents' creatures.
    assert!(suspected(&t, mine));
    assert_eq!(t.pt(bears), (0, 2));
    assert_eq!(t.pt(giant), (1, 3));
    // No longer suspected: it can block again (CR 701.60c).
    assert!(t.g.can_block_at_all(bears));
}
