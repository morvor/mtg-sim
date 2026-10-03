//! Rulings batch P201 — Lich's Mirror: "If you would lose the game, instead shuffle your
//! hand, your graveyard, and all permanents you own into your library, then draw seven
//! cards and your life total becomes 20." (CR 104.3, 614.1a, 704.7, 810.8a).

use crate::r_p116_common::{set_life, two_headed_giant};
use crate::r_s01_common::{custom_card, supported};
use crate::r_s02_common::create_token;
use crate::r_s29_common::cast_and_resolve;
use mtg_engine::ability::*;
use mtg_engine::decision::Action;
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// P0 controls Lich's Mirror and three Forests; returns the Mirror.
fn mirror(t: &mut TestGame, p: PlayerId) -> ObjectId {
    supported("Lich's Mirror");
    let m = t.battlefield(p, "Lich's Mirror");
    t.lands(p, "Forest", 3);
    m
}

/// The number of cards `p` drew this turn.
fn draws(t: &TestGame, p: PlayerId) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, Event::Drew { player, .. } if *player == p))
        .count()
}

/// The Mirror did its thing for P0: P0 is still in the game with seven cards in hand,
/// 20 life, and nothing they own on the battlefield or in their graveyard (except the
/// resolving "Doom Edict" spell, which goes there after the Mirror's effect).
fn assert_mirrored(t: &TestGame, p: PlayerId) {
    assert!(!t.has_lost(p));
    assert!(t.g.result.is_none());
    assert_eq!(t.hand_size(p), 7);
    assert_eq!(t.life(p), 20);
    assert!(t
        .player(p)
        .graveyard
        .iter()
        .all(|c| t.obj(*c).chars.name == "Doom Edict"));
    assert!(t.g.permanents().all(|o| o.owner != p));
}

/// Empties `p`'s library except for its top `keep` cards.
fn trim_library(t: &mut TestGame, p: PlayerId, keep: usize) {
    let lib = t.player(p).library.clone();
    for c in lib.into_iter().skip(keep) {
        t.g.move_object(c, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    }
}

#[test]
fn lichs_mirror_replaces_each_way_of_losing() {
    cr!("104.3b", "104.3c", "704.5a", "704.5b", "704.5c", "614.1a");
    ruling!(
        "Lich's Mirror",
        "Lich's Mirror replaces the game-loss event if you would lose the game in the following ways:"
    );
    // 0 or less life.
    let mut t = TestGame::new(2);
    mirror(&mut t, P0);
    set_life(&mut t, P0, 0);
    t.settle();
    assert_mirrored(&t, P0);
    // Drawing from an empty library.
    let mut t = TestGame::new(2);
    mirror(&mut t, P0);
    for _ in 0..4 {
        t.hand(P0, "Island");
    }
    trim_library(&mut t, P0, 0);
    t.g.draw_cards(P0, 1);
    t.settle();
    assert_mirrored(&t, P0);
    // Ten poison counters: replaced once (the poison counters stay, so the next check
    // would replace it again).
    let mut t = TestGame::new(2);
    mirror(&mut t, P0);
    t.g.players[0].counters.insert(counters::POISON.into(), 10);
    t.g.check_sbas();
    assert_mirrored(&t, P0);
    assert_eq!(t.player(P0).poison(), 10);
    // An effect says you lose the game.
    let mut t = TestGame::new(2);
    mirror(&mut t, P0);
    let doom = custom_card("Doom Edict", "Sorcery", "{0}", None, "You lose the game.");
    cast_and_resolve_custom(&mut t, doom);
    assert_mirrored(&t, P0);
    // The spell itself wasn't affected: it finished resolving and went to the graveyard.
    assert!(t.in_graveyard(P0, "Doom Edict"));
}

fn cast_and_resolve_custom(t: &mut TestGame, def: mtg_engine::card::CardDef) {
    let c = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, c).go();
    t.resolve_all();
}

#[test]
fn lichs_mirror_replaces_several_reasons_once() {
    cr!("704.7", "704.5a", "704.5b");
    ruling!(
        "Lich's Mirror",
        "If, during a check of state-based actions, you'd lose the game for multiple reasons"
    );
    supported("Night's Whisper");
    let mut t = TestGame::new(2);
    mirror(&mut t, P0);
    trim_library(&mut t, P0, 1);
    set_life(&mut t, P0, 1);
    // "You draw two cards and you lose 2 life."
    t.lands(P0, "Swamp", 2);
    let before = draws(&t, P0);
    cast_and_resolve(&mut t, P0, "Night's Whisper", &[]);
    // One draw from Night's Whisper, then seven from the Mirror once.
    assert_eq!(draws(&t, P0) - before, 8);
    assert_mirrored(&t, P0);
}

#[test]
fn lichs_mirror_leaves_spells_exiled_cards_and_borrowed_permanents() {
    cr!("614.1a", "608.2c");
    ruling!(
        "Lich's Mirror",
        "Lich's Mirror doesn't affect spells on the stack, cards that have been exiled, or permanents you control but don't own."
    );
    ruling!(
        "Lich's Mirror",
        "Lich's Mirror shuffles permanents you own into your library, regardless of who controls them."
    );
    let mut t = TestGame::new(2);
    mirror(&mut t, P0);
    let exiled = t.exile(P0, "Grizzly Bears");
    // P1's Hill Giant under P0's control; P0's Grizzly Bears under P1's control.
    let giant = t.battlefield(P1, "Hill Giant");
    let lent = t.battlefield(P0, "Grizzly Bears");
    gain_control(&mut t, P0, giant);
    gain_control(&mut t, P1, lent);
    assert_eq!(t.obj_now(giant).controller, P0);
    assert_eq!(t.obj_now(lent).controller, P1);
    // P0 casts Divination; in response, P0 is dealt lethal damage... modelled as losing
    // all their life before state-based actions are checked.
    t.lands(P0, "Island", 3);
    let div = t.hand(P0, "Divination");
    let spell = t.cast(P0, div).go();
    set_life(&mut t, P0, 0);
    t.settle();
    // The spell, the exiled card and the Giant (owned by P1) stay; P0's Bears controlled by
    // P1 is shuffled into P0's library.
    assert!(t.g.stack.contains(&spell));
    assert!(t.g.is_live(exiled));
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert!(t.on_battlefield(giant));
    assert!(!t.g.is_live(lent));
    assert!(t
        .player(P0)
        .library
        .iter()
        .any(|c| t.obj(*c).chars.name == "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), 7);
    assert_eq!(t.life(P0), 20);
    // The spell then resolves as normal: P0 draws two more.
    t.resolve();
    assert_eq!(t.hand_size(P0), 9);
    assert!(t.in_graveyard(P0, "Divination"));
}

/// `p` gains control of the permanent.
fn gain_control(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    ctx.targets = vec![vec![Entity::Object(t.g.current(id))]];
    t.g.exec(
        &Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

#[test]
fn lichs_mirror_removes_tokens_you_own() {
    cr!("111.7", "111.8", "704.5d");
    ruling!(
        "Lich's Mirror",
        "Lich's Mirror shuffles tokens you own into your library, too."
    );
    let mut t = TestGame::new(2);
    mirror(&mut t, P0);
    let token = create_token(&mut t, P0, "Soldier");
    set_life(&mut t, P0, 0);
    t.settle();
    assert!(!t.g.is_live(token));
    assert!(t.g.permanents().all(|o| !o.is_token()));
    // The token ceased to exist: it isn't in the library or anywhere else.
    assert!(t.player(P0).library.iter().all(|c| !t.obj(*c).is_token()));
    assert_mirrored(&t, P0);
}

#[test]
fn lichs_mirror_doesnt_stop_an_opponent_winning() {
    cr!("104.2b", "104.3a");
    ruling!(
        "Lich's Mirror",
        "Lich's Mirror has no effect if a spell or ability (such as the one from Helix Pinnacle) states that a player \"wins the game.\""
    );
    supported("Helix Pinnacle");
    let mut t = TestGame::new(2);
    mirror(&mut t, P0);
    // P1's Helix Pinnacle with 100 tower counters: "At the beginning of your upkeep, if
    // there are 100 or more tower counters on this enchantment, you win the game."
    let pin = t.battlefield(P1, "Helix Pinnacle");
    t.g.objects[pin.0 as usize]
        .counters
        .insert("tower".into(), 100);
    t.g.dirty = true;
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(
        t.g.result,
        Some(mtg_engine::game::GameResult::Win(vec![P1]))
    );
    assert!(t.has_lost(P0));
}

#[test]
fn lichs_mirror_doesnt_stop_a_concession() {
    cr!("104.3a");
    ruling!(
        "Lich's Mirror",
        "Lich's Mirror has no effect if you concede the game."
    );
    let mut t = TestGame::new(2);
    mirror(&mut t, P0);
    t.g.perform_action(P0, Action::Concede).unwrap();
    t.settle();
    assert!(t.has_lost(P0));
    assert_eq!(
        t.g.result,
        Some(mtg_engine::game::GameResult::Win(vec![P1]))
    );
}

#[test]
fn lichs_mirror_saves_the_team_in_two_headed_giant() {
    cr!("810.8a", "810.9c", "614.1a");
    ruling!(
        "Lich's Mirror",
        "In a Two-Headed Giant game, if your team would lose the game and you control Lich's Mirror, your team won't lose."
    );
    // P0 and P1 are a team (P2 and P3 the other); P0 controls Lich's Mirror.
    // The team's life total drops to 0.
    let mut t = two_headed_giant();
    mirror(&mut t, P0);
    let hand1 = t.hand_size(P1);
    set_life(&mut t, P0, 0);
    set_life(&mut t, P1, 0);
    t.settle();
    assert!(!t.has_lost(P0) && !t.has_lost(P1));
    assert_eq!(t.hand_size(P0), 7);
    assert_eq!(t.hand_size(P1), hand1);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 20);
    // The teammate draws from an empty library: the Mirror's controller does what it says.
    let mut t = two_headed_giant();
    mirror(&mut t, P0);
    let hand1 = t.hand_size(P1);
    let gy1 = t.graveyard(P1, "Grizzly Bears");
    trim_library(&mut t, P1, 0);
    t.g.draw_cards(P1, 1);
    t.settle();
    assert!(!t.has_lost(P0) && !t.has_lost(P1));
    assert!(t.g.result.is_none());
    assert_eq!(t.hand_size(P0), 7);
    assert_eq!(t.hand_size(P1), hand1);
    assert!(t.g.is_live(gy1));
    assert_eq!(t.life(P0), 20);
    // The teammate is affected by "you lose the game".
    let mut t = two_headed_giant();
    mirror(&mut t, P0);
    let doom = custom_card("Doom Edict", "Sorcery", "{0}", None, "You lose the game.");
    let c = t.custom(P1, doom, Zone::Hand(P1));
    t.cast(P1, c).go();
    t.resolve_all();
    assert!(!t.has_lost(P0) && !t.has_lost(P1));
    assert_eq!(t.hand_size(P0), 7);
    assert_eq!(t.life(P0), 20);
}
