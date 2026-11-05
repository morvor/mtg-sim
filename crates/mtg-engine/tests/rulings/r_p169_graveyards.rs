//! Rulings batch P169 — "whenever a card is put into an opponent's graveyard from
//! anywhere" (CR 603.6c, 603.10a, 111.7), Laboratory Maniac (CR 614.1a, 104.3b, 104.4),
//! "a library has twenty or fewer cards", mill triggers, and a creature gaining menace
//! after it was blocked (CR 509.1h).

use crate::r_p169_common::*;
use mtg_engine::game::GameResult;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Empties `p`'s library (the fillers are exiled).
fn empty_library(t: &mut TestGame, p: PlayerId) {
    crate::r_p189_common::set_library(t, p, &[]);
}

/// Exiles cards from the top of `p`'s library until it has `n` left.
fn library_down_to(t: &mut TestGame, p: PlayerId, n: usize) {
    while t.library_size(p) > n {
        let top = *t.g.player(p).library.last().unwrap();
        t.g.move_object(top, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    }
    t.g.flush_events();
    t.g.recompute();
}

#[test]
fn profane_memento_doesnt_see_a_creature_card_going_to_a_graveyard_with_it() {
    cr!("603.6c", "603.10");
    ruling!(
        "Profane Memento",
        "If Profane Memento is put into your graveyard at the same time as a creature card is put into an opponent’s graveyard, its ability won’t trigger."
    );
    supported("Profane Memento");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Profane Memento");
    t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Akroma's Vengeance", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Profane Memento"));
    assert_eq!(t.life(P0), 20);
    // Positive control: with Profane Memento staying, the same destruction triggers it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Profane Memento");
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn profane_memento_ownership_and_tokens() {
    cr!("404.2", "111.7", "603.2");
    ruling!(
        "Profane Memento",
        "Profane Memento’s ability will trigger when a nontoken creature you control but an opponent owns dies."
    );
    ruling!(
        "Profane Memento",
        "Token creatures dying won’t cause Profane Memento’s ability to trigger."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Profane Memento");
    // P0 controls a creature P1 owns: it goes to P1's graveyard.
    let stolen = t.battlefield(P1, "Grizzly Bears");
    crate::r_p117_common::gain_control(&mut t, P0, stolen);
    assert_eq!(t.obj_now(stolen).controller, P0);
    destroy(&mut t, stolen);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P0), 21);
    // A token creature an opponent owns dies: no trigger.
    let token = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    destroy(&mut t, token);
    t.resolve_all();
    assert!(!t.on_battlefield(token));
    assert_eq!(t.life(P0), 21);
}

#[test]
fn the_haunt_of_hightower_attacking_a_planeswalker_and_tokens() {
    cr!("506.2", "508.5", "111.7");
    ruling!(
        "The Haunt of Hightower",
        "If The Haunt of Hightower is attacking a planeswalker, the controller of the planeswalker is the defending player."
    );
    ruling!(
        "The Haunt of Hightower",
        "The Haunt of Hightower's last ability won't trigger when a token is put into an opponent's graveyard."
    );
    supported("The Haunt of Hightower");
    let mut t = TestGame::new(2);
    let haunt = t.battlefield(P0, "The Haunt of Hightower");
    let jace = t.battlefield(P1, "Jace Beleren");
    t.hand(P1, "Grizzly Bears");
    attack_with(&mut t, &[(haunt, obj(jace))]);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.counters(haunt, counters::PLUS1), 1);
    // A token: no counter.
    let token = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    destroy(&mut t, token);
    t.resolve_all();
    assert_eq!(t.counters(haunt, counters::PLUS1), 1);
}

#[test]
fn vulturous_zombie_sees_cards_from_every_zone_but_not_tokens() {
    cr!("603.6c", "111.7", "404.1");
    ruling!(
        "Vulturous Zombie",
        "The ability doesn’t care why a card goes to your opponent’s graveyard — only that it does. The ability triggers when a card is put into your opponent’s graveyard from the stack (a spell resolves, fails to resolve, or is countered), from the battlefield (a permanent is destroyed or sacrificed), from the player’s hand (a card is discarded), from the player’s library (from a Millstone-like effect), or from any other zone."
    );
    ruling!(
        "Vulturous Zombie",
        "This ability triggers only on cards, so it won’t trigger when a token is put into your opponent’s graveyard."
    );
    supported("Vulturous Zombie");
    let mut t = TestGame::new(2);
    let vz = t.battlefield(P0, "Vulturous Zombie");
    let mut n = 0;
    let mut step = |t: &mut TestGame, what: &str| {
        t.resolve_all();
        n += 1;
        assert_eq!(t.counters(vz, counters::PLUS1), n, "{what}");
    };
    // From the stack: P1's spell resolves.
    t.set_step(P1, Step::PrecombatMain);
    cast_opt(&mut t, P1);
    step(&mut t, "resolved");
    // From the stack: countered.
    let spell = cast_opt(&mut t, P1);
    counter_spell(&mut t, spell);
    step(&mut t, "countered");
    // From the stack: fails to resolve (its only target is gone).
    let bears = t.battlefield(P0, "Grizzly Bears");
    mana(&mut t, P1, ManaType::R, 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(bears).go();
    let gone = t.g.current(bears);
    t.g.move_object(gone, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    t.g.flush_events();
    step(&mut t, "failed to resolve");
    // From the battlefield: destroyed, and sacrificed.
    let a = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, a);
    step(&mut t, "destroyed");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.g.sacrifice(b, P1);
    t.g.flush_events();
    step(&mut t, "sacrificed");
    // From the hand: discarded.
    let c = t.hand(P1, "Grizzly Bears");
    t.g.discard(P1, c, None);
    t.g.flush_events();
    step(&mut t, "discarded");
    // From the library: milled.
    t.g.mill(P1, 1);
    t.g.flush_events();
    step(&mut t, "milled");
    // From exile.
    let d = t.exile(P1, "Grizzly Bears");
    t.g.move_object(d, Zone::Graveyard(P1), mtg_engine::events::MoveCause::Effect, None);
    t.g.flush_events();
    step(&mut t, "from exile");
    // A token: no trigger.
    let token = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    destroy(&mut t, token);
    t.resolve_all();
    assert_eq!(t.counters(vz, counters::PLUS1), n);
    // Positive control for "opponent's": P0's own card doesn't count.
    let mine = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, mine);
    t.resolve_all();
    assert_eq!(t.counters(vz, counters::PLUS1), n);
}

#[test]
fn laboratory_maniac_cant_win_but_the_draw_is_still_replaced() {
    cr!("614.1a", "104.3c", "704.5b");
    ruling!(
        "Laboratory Maniac",
        "If for some reason you can't win the game (because your opponent has cast Angel's Grace this turn, for example), you won't lose for having tried to draw a card from a library with no cards in it. The draw was still replaced."
    );
    supported("Laboratory Maniac");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Laboratory Maniac");
    empty_library(&mut t, P0);
    mana(&mut t, P1, ManaType::W, 1);
    let grace = t.hand(P1, "Angel's Grace");
    t.cast(P1, grace).go();
    t.resolve_all();
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
    assert!(t.g.result.is_none());
    assert!(!t.has_lost(P0) && !t.has_lost(P1));
    // The draw was replaced: no loss for it later in the turn or the next.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.g.result.is_none());
    assert!(!t.has_lost(P0));
    // Positive control: without Angel's Grace, P0 wins.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Laboratory Maniac");
    empty_library(&mut t, P0);
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
    assert!(matches!(&t.g.result, Some(GameResult::Win(w)) if w == &vec![P0]));
}

#[test]
fn two_laboratory_maniacs_and_each_player_drawing() {
    cr!("101.4", "104.2b", "121.6");
    ruling!(
        "Laboratory Maniac",
        "If two or more players each control a Laboratory Maniac and each player is instructed to draw a number of cards, first the player whose turn it is draws that many cards. If this causes that player to win the game instead, the game is immediately over. If the game isn't over yet, repeat this process for each other player in turn order."
    );
    // Both libraries are empty: the active player wins.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Laboratory Maniac");
    t.battlefield(P1, "Laboratory Maniac");
    empty_library(&mut t, P0);
    empty_library(&mut t, P1);
    mana(&mut t, P0, ManaType::U, 2);
    let pros = t.hand(P0, "Prosperity");
    t.cast(P0, pros).x(1).go();
    t.resolve_all();
    assert!(matches!(&t.g.result, Some(GameResult::Win(w)) if w == &vec![P0]));
    // It's P1's turn and only P0's library is empty: P1 draws first without winning,
    // then P0 wins.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Laboratory Maniac");
    t.battlefield(P1, "Laboratory Maniac");
    empty_library(&mut t, P0);
    let p1_library = t.library_size(P1);
    mana(&mut t, P1, ManaType::U, 2);
    let pros = t.hand(P1, "Prosperity");
    t.cast(P1, pros).x(1).go();
    t.resolve_all();
    assert_eq!(t.library_size(P1), p1_library - 1);
    assert!(matches!(&t.g.result, Some(GameResult::Win(w)) if w == &vec![P0]));
}

#[test]
fn screeching_scorchbeast_triggers_once_when_several_players_mill() {
    cr!("701.17a", "603.2c");
    ruling!(
        "Screeching Scorchbeast",
        "If more than one player is instructed to mill at the same time (by the activated ability of Raul, Trouble Shooter, for example), as long as at least one nonland card is milled, Screeching Scorchbeast’s last ability will trigger just once."
    );
    supported("Screeching Scorchbeast");
    supported("Raul, Trouble Shooter");
    let mut t = TestGame::new(2);
    let beast = t.battlefield(P0, "Screeching Scorchbeast");
    let raul = t.battlefield(P0, "Raul, Trouble Shooter");
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.activate(P0, raul, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(t.graveyard_size(P1), 1);
    assert_eq!(triggers_of(&t, beast), 1);
}

#[test]
fn isleback_spawn_gets_its_bonus_once_for_any_library() {
    cr!("611.3a");
    ruling!(
        "Isleback Spawn",
        "It doesn't matter which library has twenty or fewer cards in it, and you don't have to specify a library. If multiple libraries have twenty or fewer cards in them, Isleback Spawn will still get its bonus only once."
    );
    supported("Isleback Spawn");
    let mut t = TestGame::new(2);
    let spawn = t.battlefield(P0, "Isleback Spawn");
    assert_eq!(t.pt(spawn), (4, 8));
    // An opponent's library.
    library_down_to(&mut t, P1, 20);
    assert_eq!(t.pt(spawn), (8, 16));
    // Both libraries: still once.
    library_down_to(&mut t, P0, 20);
    assert_eq!(t.pt(spawn), (8, 16));
    // Only its controller's library.
    let mut t = TestGame::new(2);
    let spawn = t.battlefield(P0, "Isleback Spawn");
    library_down_to(&mut t, P0, 21);
    assert_eq!(t.pt(spawn), (4, 8));
    library_down_to(&mut t, P0, 20);
    assert_eq!(t.pt(spawn), (8, 16));
}

#[test]
fn nimana_skitter_sneak_gaining_menace_after_blocks_doesnt_unblock_it() {
    cr!("509.1h", "506.4");
    ruling!(
        "Nimana Skitter-Sneak",
        "Once a single creature has blocked Nimana Skitter-Sneak, causing it to gain menace won’t cause it to become unblocked or remove that blocking creature from combat."
    );
    supported("Nimana Skitter-Sneak");
    let mut t = TestGame::new(2);
    let nimana = t.battlefield(P0, "Nimana Skitter-Sneak");
    let bears = t.battlefield(P1, "Grizzly Bears");
    for _ in 0..7 {
        t.graveyard(P1, "Lightning Bolt");
    }
    crate::r_p076_common::unblockable_after_blocked(&mut t, nimana, bears, |t| {
        t.graveyard(P1, "Lightning Bolt");
        t.g.recompute();
        assert!(has_kw(t, nimana, mtg_engine::keywords::KeywordKind::Menace));
        assert_eq!(t.pt(nimana), (4, 4));
    });
    assert!(t.on_battlefield(nimana));
    // Positive control: with eight cards before blocks, a single blocker is illegal.
    let mut t = TestGame::new(2);
    let nimana = t.battlefield(P0, "Nimana Skitter-Sneak");
    let bears = t.battlefield(P1, "Grizzly Bears");
    for _ in 0..8 {
        t.graveyard(P1, "Lightning Bolt");
    }
    attack_with(&mut t, &[(nimana, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(bears, nimana)]));
}
