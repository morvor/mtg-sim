//! Rulings batch P068 — owners, controllers and players: leaving the game (CR 800.4a) and
//! "loses the game" triggers looking back (CR 603.10f), face-down cards in exile no one may
//! look at and a choice of a spell from several zones (CR 406.3, 406.3b, 601.3e), cards put
//! into "your graveyard" (CR 400.3), a player controlling another's search (CR 723.5,
//! 701.23b), discards replaced by exile (CR 614.6, 701.9), the drafted-card pregame action
//! (CR 607.2n) and two Classes each tracking their own exiled cards (CR 607.2a).

use crate::r_p068_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, stack_library, supported};
use mtg_engine::card::card;
use mtg_engine::decision::Answer;
use mtg_engine::facedown::can_look_at;
use mtg_engine::mana::ManaType::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Gives control of `id` (owned by another player) to `p`.
fn give_control(t: &mut TestGame, id: ObjectId, p: PlayerId) {
    let o = t.g.obj_mut(id);
    o.controller = p;
    o.base_controller = p;
    t.g.dirty = true;
    t.g.recompute();
}

/// Asserts that `card` is a face-down card in exile no player in the game may look at.
fn face_down_unseen(t: &TestGame, card: ObjectId, what: &str) {
    let c = t.g.current(card);
    assert_eq!(t.zone(c), Zone::Exile, "{what}: {}", t.dump_log());
    assert!(t.obj_now(c).face_down, "{what}");
    // The players still in the game (the one who left is no longer a player).
    for p in [P0, P1, P2] {
        if t.g.player(p).in_game() {
            assert!(!can_look_at(&t.g, p, c), "{what}: {p:?} may look");
        }
    }
}

// ---------------------------------------------------------------------------
// Enigma Sphinx
// ---------------------------------------------------------------------------

#[test]
fn enigma_sphinx_owned_by_another_player_doesnt_trigger() {
    cr!("400.3", "603.6c");
    ruling!(
        "Enigma Sphinx",
        "If you control an Enigma Sphinx that's owned by another player, it's put into that player's graveyard from the battlefield, so Enigma Sphinx's middle ability won't trigger."
    );
    supported("Enigma Sphinx");
    let mut t = TestGame::new(2);
    let es = t.battlefield(P1, "Enigma Sphinx");
    give_control(&mut t, es, P0);
    t.g.destroy(es, None);
    t.settle();
    assert!(t.g.stack.is_empty(), "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.zone(es), Zone::Graveyard(P1));
}

#[test]
fn enigma_sphinx_with_a_short_library_goes_to_the_bottom() {
    cr!("401.7");
    ruling!(
        "Enigma Sphinx",
        "If you have zero or one card left in your library when Enigma Sphinx's middle ability resolves, Enigma Sphinx is put on the bottom of your library."
    );
    for n in [0, 1] {
        let mut t = TestGame::new(2);
        t.g.players[0].library.clear();
        if n == 1 {
            t.library_top(P0, "Forest");
        }
        let es = t.battlefield(P0, "Enigma Sphinx");
        t.g.destroy(es, None);
        t.resolve_all();
        assert_eq!(t.library_size(P0), n + 1, "{}", t.dump_log());
        assert_eq!(library_pos(&t, P0, es), Some(n), "on the bottom");
    }
}

// ---------------------------------------------------------------------------
// Leaving the game
// ---------------------------------------------------------------------------

#[test]
fn hostage_taker_owner_leaving_returns_the_card_to_its_owner() {
    cr!("800.4a", "610.3", "610.3c");
    ruling!(
        "Hostage Taker",
        "In a multiplayer game, if Hostage Taker's owner leaves the game while the card is still exiled and another player owns that card, the exiled card will return to the battlefield under its owner's control. Because the one-shot effect that returns the card isn't an ability that goes on the stack, it won't cease to exist along with the leaving player's spells and abilities on the stack."
    );
    supported("Hostage Taker");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[o(bears)]);
    t.enter(P0, "Hostage Taker");
    t.resolve_all();
    assert!(exiled(&t, bears), "{}", t.dump_log());
    t.g.lose_game(P0);
    t.resolve_all();
    let b = t.g.current(bears);
    assert!(t.on_battlefield(b), "{}", t.dump_log());
    assert_eq!(t.obj_now(b).controller, P1);
}

#[test]
fn face_down_cards_stay_exiled_unseen_when_their_exiler_leaves() {
    cr!("800.4a", "406.3");
    ruling!(
        "Gonti, Lord of Luxury",
        "If you leave the game, the card remains exiled face down indefinitely. No player may look at it."
    );
    ruling!(
        "Rev, Tithe Extractor",
        "If you leave the game, cards exiled face down with Rev's last ability remain exiled face down indefinitely. No player may look at them."
    );
    ruling!(
        "Outrageous Robbery",
        "If you leave the game, the exiled cards remain exiled face down indefinitely. No player may look at them."
    );
    supported("Gonti, Lord of Luxury");
    supported("Rev, Tithe Extractor");
    supported("Outrageous Robbery");
    // Gonti.
    let mut t = TestGame::new(3);
    let lib = stack_library(&mut t, P1, &["Grizzly Bears", "Hill Giant", "Forest", "Island"]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[o(lib[0])]);
    t.enter(P0, "Gonti, Lord of Luxury");
    t.resolve_all();
    let c = t.g.current(lib[0]);
    assert!(can_look_at(&t.g, P0, c), "{}", t.dump_log());
    t.g.lose_game(P0);
    t.settle();
    face_down_unseen(&t, lib[0], "Gonti");
    // Rev: it deals combat damage to P1.
    let mut t = TestGame::new(3);
    let rev = t.battlefield(P0, "Rev, Tithe Extractor");
    let top = t.library_top(P1, "Grizzly Bears");
    t.answer_targets(P0, &[o(rev)]);
    attack_with(&mut t, &[(rev, Entity::Player(P1))]);
    t.resolve_all();
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert!(exiled(&t, top), "{}", t.dump_log());
    t.g.lose_game(P0);
    t.settle();
    face_down_unseen(&t, top, "Rev");
    // Outrageous Robbery with X = 2.
    let mut t = TestGame::new(3);
    let lib = stack_library(&mut t, P1, &["Grizzly Bears", "Hill Giant"]);
    let or = t.hand(P0, "Outrageous Robbery");
    pool(&mut t, P0, &[B, B, C, C]);
    t.cast(P0, or).x(2).target(P1).go();
    t.resolve_all();
    t.g.lose_game(P0);
    t.settle();
    for c in lib {
        face_down_unseen(&t, c, "Outrageous Robbery");
    }
}

#[test]
fn apple_of_eden_cards_stay_exiled_if_you_leave_before_the_end_step() {
    cr!("800.4a", "406.3", "603.7");
    ruling!(
        "Apple of Eden, Isu Relic",
        "If you leave the game before the second delayed trigger resolves, the cards remain exiled face down indefinitely. No player may look at them."
    );
    supported("Apple of Eden, Isu Relic");
    let mut t = TestGame::new(3);
    let apple = t.battlefield(P0, "Apple of Eden, Isu Relic");
    let hand = [t.hand(P1, "Grizzly Bears"), t.hand(P1, "Lightning Bolt")];
    t.activate(P0, apple, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(hand.iter().all(|c| exiled(&t, *c)), "{}", t.dump_log());
    t.g.lose_game(P0);
    t.settle();
    // The next end step passes, and the next turn begins.
    t.advance_to_step(Step::End);
    t.resolve_all();
    t.advance_to_step(Step::Upkeep);
    for c in hand {
        face_down_unseen(&t, c, "Apple of Eden");
    }
}

#[test]
fn share_the_spoils_borrowed_from_a_player_who_loses_still_triggers() {
    cr!("603.10f", "603.3a", "800.4a", "611.3b");
    // Pins the CR, not the card's ruling of 2021-07-23 ("If you control an opponent's Share
    // the Spoils and that player loses the game, Share the Spoils leaves the game with them
    // and the first ability won't trigger."), which contradicts CR 603.10f: it is left
    // uncited, pending the coordinator's conflict decision (docs/RULING_DISCREPANCIES.md).
    supported("Share the Spoils");
    // P0 controls P2's Share the Spoils; P2 loses. It leaves the game with P2, but its
    // "whenever an opponent loses the game" ability looks back in time and triggers, under
    // P0's control.
    let mut t = TestGame::new(3);
    let sts = t.battlefield(P2, "Share the Spoils");
    give_control(&mut t, sts, P0);
    let bears = t.library_top(P0, "Grizzly Bears");
    let forest = t.library_top(P1, "Forest");
    t.g.lose_game(P2);
    assert!(!t.on_battlefield(sts), "{}", t.dump_log());
    t.settle();
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    let trigger = *t.g.stack.last().unwrap();
    assert_eq!(t.obj_now(trigger).controller, P0);
    t.resolve_all();
    // The top card of each library still in the game is exiled.
    assert!(exiled(&t, bears) && exiled(&t, forest), "{}", t.dump_log());
    assert_eq!(t.g.exile.len(), 2, "{}", t.dump_log());
    // No one may play them: the enchantment that let players do so is gone.
    for p in [P0, P1] {
        t.set_step(p, Step::PrecombatMain);
        mana(&mut t, p, G, 2);
        assert!(!castable(&mut t, p, bears), "{p:?}: {}", t.dump_log());
        assert!(!playable_land(&mut t, p, forest), "{p:?}");
    }
    // Had its owner stayed (P1's Share the Spoils, P2 losing), P0 could cast the Bears.
    let mut t = TestGame::new(3);
    let sts = t.battlefield(P1, "Share the Spoils");
    give_control(&mut t, sts, P0);
    let bears = t.library_top(P0, "Grizzly Bears");
    t.g.lose_game(P2);
    t.resolve_all();
    assert!(t.on_battlefield(sts) && exiled(&t, bears), "{}", t.dump_log());
    mana(&mut t, P0, G, 2);
    assert!(castable(&mut t, P0, bears), "{}", t.dump_log());
}

#[test]
fn kaya_emblem_casts_a_face_down_exiled_card_only_if_you_may_look_at_it() {
    cr!("406.3", "406.3b");
    ruling!(
        "Kaya the Inexorable",
        "If you own a face-down card in exile, Kaya’s emblem will let you cast it only if you can look at the card (and only if it’s legendary). If you aren’t allowed to look at the card, you can’t cast it, even if you know it’s legendary."
    );
    supported("Kaya the Inexorable");
    for allowed in [false, true] {
        let mut t = TestGame::new(2);
        let kaya = t.battlefield(P0, "Kaya the Inexorable");
        t.g.obj_mut(kaya).counters.insert("loyalty".into(), 7);
        t.activate(P0, kaya, 2, &[]).unwrap();
        t.resolve_all();
        // P1's Gonti exiles P0's Isamaru face down.
        let isamaru = t.library_top(P0, "Isamaru, Hound of Konda");
        t.answer_targets(P1, &[Entity::Player(P0)]);
        t.answer_choose(P1, &[o(isamaru)]);
        t.enter(P1, "Gonti, Lord of Luxury");
        t.resolve_all();
        let c = t.g.current(isamaru);
        assert!(t.obj_now(c).face_down && exiled(&t, c), "{}", t.dump_log());
        assert!(!can_look_at(&t.g, P0, c));
        if allowed {
            mtg_engine::zones::allow_look(&mut t.g, P0, c);
        }
        let from = t.asked().len();
        t.answer_choose(P0, &[o(c)]);
        t.answer_yes(P0, true);
        t.advance_to(P1, Step::Upkeep);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        assert_eq!(was_offered(&t, from, c), allowed, "{}", t.dump_log());
        let on_bf = t.g.current(isamaru);
        assert_eq!(t.on_battlefield(on_bf), allowed, "{}", t.dump_log());
    }
}

#[test]
fn kaya_emblem_judges_each_face_as_the_spell_it_would_be() {
    cr!("601.3e", "712.11c");
    // Extus, Oriq Overlord (a legendary creature) // Awaken the Blood Avatar (a sorcery that
    // isn't legendary), in P0's graveyard: the emblem casts only the legendary face.
    supported("Kaya the Inexorable");
    let mut t = TestGame::new(2);
    let kaya = t.battlefield(P0, "Kaya the Inexorable");
    t.g.obj_mut(kaya).counters.insert("loyalty".into(), 7);
    t.activate(P0, kaya, 2, &[]).unwrap();
    t.resolve_all();
    let extus = t.graveyard(P0, "Extus, Oriq Overlord // Awaken the Blood Avatar");
    let from = t.asked().len();
    t.answer_choose(P0, &[o(extus)]);
    t.answer_yes(P0, true);
    // The back face, were P0 asked which face to cast.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(was_offered(&t, from, extus), "{}", t.dump_log());
    assert!(t.named_on_battlefield("Avatar Token").is_empty(), "{}", t.dump_log());
    assert_eq!(
        t.named_on_battlefield("Extus, Oriq Overlord").len(),
        1,
        "{}",
        t.dump_log()
    );
}

// ---------------------------------------------------------------------------
// Dauthi Voidwalker, Opposition Agent
// ---------------------------------------------------------------------------

#[test]
fn dauthi_voidwalker_discards_still_count_and_the_card_is_found_in_exile() {
    cr!("614.6", "701.9a", "603.2");
    ruling!(
        "Dauthi Voidwalker",
        "If your opponent discards a card while you control Dauthi Voidwalker, abilities that function when that card is discarded still work, even though that card never reaches that player's graveyard. In addition, spells or abilities that check the characteristics of the discarded card can find that card in exile."
    );
    supported("Dauthi Voidwalker");
    // P1's Pyromancy ("{3}, Discard a card at random: This enchantment deals damage to any
    // target equal to the mana value of the discarded card.") discards Hill Giant; P0's
    // Megrim ("Whenever an opponent discards a card, Megrim deals 2 damage to that
    // player.") triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dauthi Voidwalker");
    t.battlefield(P0, "Megrim");
    let pyro = t.battlefield(P1, "Pyromancy");
    let giant = t.hand(P1, "Hill Giant");
    mana(&mut t, P1, C, 3);
    t.g.turn.priority = Some(P1);
    t.activate(P1, pyro, 0, &[Entity::Player(P0)]).unwrap();
    assert!(exiled(&t, giant), "{}", t.dump_log());
    assert_eq!(t.counters(giant, "void"), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 16, "4 damage: {}", t.dump_log());
    assert_eq!(t.life(P1), 18, "Megrim");
}

#[test]
fn opposition_agent_may_find_nothing_only_for_a_stated_quality() {
    cr!("701.23b", "723.5");
    ruling!(
        "Opposition Agent",
        "If the search instruction states a quality of the card to find, such as a color or card type, you may have the opponent not find any cards. If the search is simply for a number of cards without stating a quality, you must have them find that many cards (or as many as possible, if their library doesn't contain that many cards)."
    );
    // Rampant Growth ("a basic land card"): P0 has P1 find nothing (P1 would have found the
    // Forest: P0 makes the choice).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Opposition Agent");
    let forest = t.library_top(P1, "Forest");
    t.set_step(P1, Step::PrecombatMain);
    let rg = t.hand(P1, "Rampant Growth");
    pool(&mut t, P1, &[G, C]);
    t.answer_choose(P0, &[]);
    t.answer_choose(P1, &[o(forest)]);
    t.cast(P1, rg).go();
    t.resolve_all();
    assert!(in_library(&t, forest, P1), "{}", t.dump_log());
    assert!(t.g.exile.is_empty());
    // Demonic Tutor ("a card"): a card is found (and exiled) anyway.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Opposition Agent");
    t.set_step(P1, Step::PrecombatMain);
    let dt = t.hand(P1, "Demonic Tutor");
    pool(&mut t, P1, &[B, C]);
    t.answer_choose(P0, &[]);
    t.answer_choose(P1, &[]);
    t.cast(P1, dt).go();
    t.resolve_all();
    assert_eq!(t.g.exile.len(), 1, "{}", t.dump_log());
}

// ---------------------------------------------------------------------------
// Arcane Savant, Rogue Class
// ---------------------------------------------------------------------------

#[test]
fn several_arcane_savants_each_exile_a_drafted_card() {
    cr!("607.2n");
    ruling!(
        "Arcane Savant",
        "If you have more than one Arcane Savant in your deck, you may reveal any number of them and exile that many instant or sorcery cards you drafted that aren't in your deck."
    );
    supported("Arcane Savant");
    for reveal in [2usize, 1] {
        let mut t = TestGame::new(2);
        t.library_top(P0, "Arcane Savant");
        t.library_top(P0, "Arcane Savant");
        let side = t.g.add_to_sideboard(
            P0,
            vec![card("Lightning Bolt"), card("Shock"), card("Divination")],
        );
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[o(side[0])]);
        t.answer_yes(P0, reveal == 2);
        t.answer_choose(P0, &[o(side[1])]);
        mtg_engine::opening_hand::before_shuffle_actions(&mut t.g);
        let named = t
            .g
            .named_exiles
            .iter()
            .filter(|(p, n, _)| *p == P0 && n == "Arcane Savant")
            .count();
        assert_eq!(named, reveal);
        assert_eq!(t.g.exile.len(), reveal);
    }
}

#[test]
fn two_rogue_classes_track_their_exiled_cards_separately() {
    cr!("607.2a", "716.2a");
    ruling!(
        "Rogue Class",
        "If you have more than one Rogue Class on the battlefield, they each track exiled cards individually."
    );
    supported("Rogue Class");
    let mut t = TestGame::new(2);
    let classes = [
        t.battlefield(P0, "Rogue Class"),
        t.battlefield(P0, "Rogue Class"),
    ];
    for c in classes {
        mtg_engine::classes::set_level(&mut t.g, c, 3);
    }
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lib = stack_library(&mut t, P1, &["Divination", "Divination"]);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert!(lib.iter().all(|c| exiled(&t, *c)), "{}", t.dump_log());
    t.set_step(P0, Step::PostcombatMain);
    let playable = |t: &mut TestGame| {
        lib.iter()
            .filter(|c| {
                mana(t, P0, U, 3);
                castable(t, P0, **c)
            })
            .count()
    };
    assert_eq!(playable(&mut t), 2, "{}", t.dump_log());
    // One Class leaves: only the card the other exiled can still be played.
    move_to(&mut t, classes[0], Zone::Graveyard(P0));
    assert_eq!(playable(&mut t), 1, "{}", t.dump_log());
}
