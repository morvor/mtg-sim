//! Rulings batch P008 — burn that can hit planeswalkers in multiplayer games: "each
//! opponent" in Two-Headed Giant (CR 810.9, 810.7), leaving the game (CR 800.4a), and
//! emblems owned by other players (CR 114.2).

use crate::r_p007_common::*;
use crate::r_p008_common::*;
use crate::r_p014_common::thg;
use crate::r_s01_common::supported;
use crate::r_s02_common::{can_cast, destroy};
use mtg_engine::object::{CastMethod, ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const NORMAL: CastMethod = CastMethod::Normal;

#[test]
fn two_headed_giant_each_opponent_effects_hit_the_team_twice() {
    cr!("810.9", "810.7", "102.3");
    ruling!(
        "Aria of Flame",
        "In a Two-Headed Giant game, Aria of Flame’s first ability causes the opposing team to gain 20 life."
    );
    ruling!(
        "Chandra, Pyrogenius",
        "In a Two-Headed Giant game, Chandra's first ability causes 4 damage total to be dealt to the opposing team."
    );
    ruling!(
        "Chandra, Torch of Defiance",
        "In a Two-Headed Giant game, Chandra's first ability causes 4 damage total to be dealt to the opposing team."
    );
    ruling!(
        "Sarkhan, Dragonsoul",
        "In a Two-Headed Giant game, Sarkhan's first ability causes the opposing team to lose 2 life."
    );
    ruling!(
        "Goblin Chainwhirler",
        "In a Two-Headed Giant game, the Goblin Chainwhirler’s last ability causes the opposing team to lose 2 life."
    );
    // Aria of Flame enters: +20.
    supported("Aria of Flame");
    let mut t = thg();
    let before = t.life(P2);
    t.enter(P0, "Aria of Flame");
    t.settle();
    t.resolve_all();
    assert_eq!((t.life(P2), t.life(P3)), (before + 20, before + 20));
    // Chandra, Pyrogenius +2: 4 in all.
    let mut t = thg();
    let before = t.life(P2);
    let c = pw(&mut t, P0, "Chandra, Pyrogenius", 5);
    loyalty(&mut t, P0, c, 0, &[]);
    t.resolve_all();
    assert_eq!(t.life(P2), before - 4);
    // Chandra, Torch of Defiance +1, not casting the exiled card: 4 in all.
    let mut t = thg();
    let before = t.life(P2);
    t.library_top(P0, "Craw Wurm");
    let c = pw(&mut t, P0, "Chandra, Torch of Defiance", 4);
    t.answer_yes(P0, false);
    loyalty(&mut t, P0, c, 0, &[]);
    t.resolve_all();
    assert_eq!(t.life(P2), before - 4);
    // Sarkhan, Dragonsoul +2 and Goblin Chainwhirler: 2 in all.
    supported("Sarkhan, Dragonsoul");
    let mut t = thg();
    let before = t.life(P2);
    let s = pw(&mut t, P0, "Sarkhan, Dragonsoul", 5);
    loyalty(&mut t, P0, s, 0, &[]);
    t.resolve_all();
    assert_eq!(t.life(P2), before - 2);
    let mut t = thg();
    let before = t.life(P2);
    t.enter(P0, "Goblin Chainwhirler");
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P2), before - 2);
}

#[test]
fn two_headed_giant_cavalier_and_chandra_emblems() {
    cr!("810.9", "114.2", "810.7");
    ruling!(
        "Cavalier of Flame",
        "In a Two-Headed Giant game, Cavalier of Flame’s last ability causes the opposing team to be dealt X damage twice."
    );
    ruling!(
        "Chandra, Awakened Inferno",
        "In a Two-Headed Giant game, Chandra's first ability grants an emblem to each opponent. The opposing team will be dealt 1 damage twice during their upkeep."
    );
    // Cavalier of Flame dies with two land cards in P0's graveyard.
    supported("Cavalier of Flame");
    let mut t = thg();
    let before = t.life(P2);
    let theirs = pw(&mut t, P3, "Chandra, Pyrogenius", 5);
    t.graveyard(P0, "Forest");
    t.graveyard(P0, "Forest");
    let cav = t.battlefield(P0, "Cavalier of Flame");
    let from = t.g.turn_events.len();
    destroy(&mut t, cav);
    t.resolve_all();
    assert_eq!(t.life(P2), before - 4);
    assert_eq!(damage_to(&t, from, pl(P2)).0, 2);
    assert_eq!(damage_to(&t, from, pl(P3)).0, 2);
    assert_eq!(t.counters(theirs, "loyalty"), 3);
    // Chandra, Awakened Inferno +2: P2 and P3 each get an emblem.
    supported("Chandra, Awakened Inferno");
    let mut t = thg();
    let c = pw(&mut t, P0, "Chandra, Awakened Inferno", 6);
    loyalty(&mut t, P0, c, 0, &[]);
    t.resolve_all();
    let mut owners: Vec<PlayerId> = t
        .g
        .objects
        .iter()
        .filter(|o| o.kind == ObjKind::Emblem && o.zone == Zone::Command)
        .map(|o| o.owner)
        .collect();
    owners.sort();
    assert_eq!(owners, vec![P2, P3]);
    let before = t.life(P2);
    t.advance_to(P2, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P2), before - 2);
}

#[test]
fn two_headed_giant_sweepers_hit_the_teammates_permanents() {
    cr!("810.9", "109.5");
    ruling!(
        "Immolating Gyre",
        "In a Two-Headed Giant game, Immolating Gyre deals damage to your teammate’s creatures and planeswalkers."
    );
    ruling!(
        "Omnath, Locus of Creation",
        "In a Two-Headed Giant game, Omnath's second ability causes the opposing team to lose 8 life the third time it resolves. Each other player's planeswalkers—including your teammate's—are dealt 4 damage."
    );
    // Immolating Gyre with two instants in the graveyard.
    let mut t = thg();
    t.graveyard(P0, "Shock");
    t.graveyard(P0, "Opt");
    let mate = t.battlefield(P1, "Grizzly Bears");
    let mate_pw = pw(&mut t, P1, "Chandra, Pyrogenius", 5);
    let mine = t.battlefield(P0, "Hill Giant");
    cast_x(&mut t, P0, "Immolating Gyre", NORMAL, None, &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(mate));
    assert_eq!(t.counters(mate_pw, "loyalty"), 3);
    assert_eq!(dmg(&t, mine), 0);
    // Omnath: the third landfall this turn.
    supported("Omnath, Locus of Creation");
    let mut t = thg();
    t.battlefield(P0, "Omnath, Locus of Creation");
    let mate_pw = pw(&mut t, P1, "Chandra, Pyrogenius", 5);
    let their_pw = pw(&mut t, P2, "Chandra, Pyrogenius", 5);
    let before = t.life(P2);
    for _ in 0..3 {
        t.enter(P0, "Forest");
        t.settle();
        t.resolve_all();
    }
    assert_eq!(t.life(P2), before - 8);
    assert_eq!(t.counters(mate_pw, "loyalty"), 1);
    assert_eq!(t.counters(their_pw, "loyalty"), 1);
}

#[test]
fn chandra_awakened_inferno_emblems_stay_after_chandras_owner_leaves() {
    cr!("800.4a", "114.2");
    ruling!(
        "Chandra, Awakened Inferno",
        "If you leave a multiplayer game, your opponents keep any emblems Chandra gave them. Each opponent owns the emblem Chandra gives them. In multiplayer, your opponents continue to burn after you and Chandra leave the game."
    );
    let mut t = TestGame::new(3);
    let c = pw(&mut t, P0, "Chandra, Awakened Inferno", 6);
    loyalty(&mut t, P0, c, 0, &[]);
    t.resolve_all();
    t.g.lose_game(P0);
    t.settle();
    assert!(!t.on_battlefield(c));
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    t.advance_to(P2, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P2), 19);
}

#[test]
fn nicol_bolas_reanimated_permanents_are_exiled_when_you_leave() {
    cr!("800.4a");
    ruling!(
        "Nicol Bolas, the Ravager // Nicol Bolas, the Arisen",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, any permanents you control from Nicol Bolas's third loyalty ability that came from other players' graveyards are exiled."
    );
    supported("Nicol Bolas, the Ravager // Nicol Bolas, the Arisen");
    let mut t = TestGame::new(3);
    let b = t.battlefield(P0, "Nicol Bolas, the Ravager // Nicol Bolas, the Arisen");
    assert!(mtg_engine::dfc::transform(&mut t.g, b));
    let b = t.g.current(b);
    t.g.objects[b.0 as usize]
        .counters
        .insert("loyalty".into(), 7);
    let wurm = t.graveyard(P1, "Craw Wurm");
    loyalty(&mut t, P0, b, 2, &[obj(wurm)]);
    t.resolve_all();
    let wurm = t.g.current(wurm);
    assert!(t.on_battlefield(wurm));
    assert_eq!(t.obj_now(wurm).controller, P0);
    t.g.lose_game(P0);
    t.settle();
    assert_eq!(t.zone(wurm), Zone::Exile);
}

#[test]
fn theater_of_horrors_works_after_the_opponent_who_lost_life_leaves() {
    cr!("800.4a", "608.2h");
    ruling!(
        "Theater of Horrors",
        "In a multiplayer game, if an opponent loses life during your turn and then leaves the game, you can play cards exiled with Theater of Horrors."
    );
    supported("Theater of Horrors");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Theater of Horrors");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.set_step(P2, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let bears = t.g.current(bears);
    assert_eq!(t.zone(bears), Zone::Exile);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Forest", 2);
    assert!(!can_cast(&mut t, P0, bears, NORMAL), "no opponent lost life");
    let src = t.battlefield(P0, "Raging Goblin");
    hit_player(&mut t, src, P1, 1);
    t.g.lose_game(P1);
    t.settle();
    assert!(can_cast(&mut t, P0, bears, NORMAL));
}
