//! Rulings batch P180 — tutors and defender: Mythos of Brokkos, Aether Searcher, Distant
//! Memories, Demonic Bargain, Maralen of the Mornsong, Grozoth, Demonic Tutor, Spire
//! Serpent, Sentry Oak and Wakestone Gargoyle.

use crate::r_p180_common::*;
use crate::r_s11_common::triggered_from;
use mtg_engine::card::card;
use mtg_engine::decision::Decision;
use mtg_engine::draft::{Draft, DraftInfo, DraftStyle};
use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn mythos_of_brokkos_can_return_the_card_it_searched_for() {
    cr!("608.2c", "701.23a");
    ruling!(
        "Mythos of Brokkos",
        "Because Mythos of Brokkos doesn’t target the permanent cards in your graveyard, you may choose the card you searched for."
    );
    supported("Mythos of Brokkos");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.library_top(P0, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 2);
    let mythos = t.hand(P0, "Mythos of Brokkos");
    t.answer_choose_moved(P0, &[obj(bears)]);
    t.answer_choose_moved(P0, &[obj(bears)]);
    t.cast(P0, mythos).go();
    t.resolve_all();
    let bears = t.g.current(bears);
    assert_eq!(t.zone(bears), Zone::Hand(P0));
}

/// The draft information of P0 drafting `cards` in one round (one card each pick).
fn drafted(cards: &[&str]) -> DraftInfo {
    let mut d = Draft::new(
        vec![vec![cards.iter().map(|c| card(c)).collect()]],
        DraftStyle::Conspiracy,
        1,
    );
    for _ in cards {
        d.pick(P0, &[0]).unwrap();
    }
    d.finish().1
}

/// A 2-player game in P0's main phase with the given draft information.
fn game_with(info: DraftInfo) -> TestGame {
    let mut t = TestGame::new(2);
    t.g.start.draft = info;
    t.set_step(P0, Step::PrecombatMain);
    t
}

/// Script P0 searching their hand (not the library) for `card` and casting it.
fn search_hand_and_cast(t: &mut TestGame, card: ObjectId) {
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.answer_choose(P0, &[obj(card)]);
    t.answer_yes(P0, true);
}

#[test]
fn aether_searcher_notes_across_rounds_and_works_without_itself() {
    cr!("905.2b", "608.2b");
    ruling!(
        "Aether Searcher",
        "If Aether Searcher is the last card you draft in any draft round other than the last one, you'll reveal the first card you draft in the following draft round and note its name."
    );
    ruling!(
        "Aether Searcher",
        "If Aether Searcher leaves the battlefield before its ability resolves, you'll still be able to search for the named card and cast it."
    );
    supported("Aether Searcher");
    // Last pick of round one; the first pick of round two is noted.
    let mut d = Draft::new(
        vec![vec![
            vec![card("Aether Searcher")],
            vec![card("Lightning Bolt"), card("Shock")],
        ]],
        DraftStyle::Conspiracy,
        1,
    );
    d.pick(P0, &[0]).unwrap();
    d.pick(P0, &[0]).unwrap();
    d.pick(P0, &[0]).unwrap();
    let info = d.finish().1;
    assert_eq!(info.notes.len(), 1);
    assert_eq!(info.notes[0].noted_name.as_deref(), Some("Lightning Bolt"));
    // The Searcher is gone by the time its ability resolves: Bolt is still cast.
    let mut t = game_with(info);
    let bolt = t.hand(P0, "Lightning Bolt");
    search_hand_and_cast(&mut t, bolt);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let searcher = t.enter(P0, "Aether Searcher");
    t.settle();
    destroy(&mut t, searcher);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn aether_searcher_uses_its_controllers_notes() {
    cr!("905.2b");
    ruling!(
        "Aether Searcher",
        "If an Aether Searcher owned by another player enters the battlefield under your control, you can search for a card you've named while drafting a card named Aether Searcher, not a card named by its owner."
    );
    let mut info = drafted(&["Aether Searcher", "Lightning Bolt"]);
    // The notes are P1's.
    for n in info.notes.iter_mut() {
        n.player = P1;
    }
    let mut t = game_with(info);
    let bolt = t.hand(P0, "Lightning Bolt");
    search_hand_and_cast(&mut t, bolt);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    // P1's Aether Searcher enters under P0's control.
    let id = t
        .g
        .create_card_object(card("Aether Searcher"), P1, Zone::Nowhere);
    t.g.move_object_ev(mtg_engine::replacement::MoveEv {
        obj: id,
        to: Zone::Battlefield,
        pos: mtg_engine::ability::LibraryPosition::Top,
        cause: mtg_engine::events::MoveCause::Effect,
        by: Some(P0),
        etb: mtg_engine::replacement::EtbInfo {
            controller: Some(P0),
            ..Default::default()
        },
        source: None,
    })
    .unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "P0 noted nothing");
    assert!(t.in_hand(P0, "Lightning Bolt"));
}

#[test]
fn aether_searcher_ignores_type_timing_but_cant_play_a_land() {
    cr!("905.2b");
    ruling!(
        "Aether Searcher",
        "When casting a card this way, ignore timing restrictions based on the card's type."
    );
    ruling!(
        "Aether Searcher",
        "If the card you draft after Aether Searcher is a land card, you won't be able to play that land using Aether Searcher's ability."
    );
    // A creature card is cast while the ability resolves (the stack isn't empty), on the
    // opponent's turn.
    let mut t = game_with(drafted(&["Aether Searcher", "Grizzly Bears"]));
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.hand(P0, "Grizzly Bears");
    search_hand_and_cast(&mut t, bears);
    t.enter(P0, "Aether Searcher");
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bears)), Zone::Battlefield);
    // "Cast this spell only during combat" is followed.
    let mut t = game_with(drafted(&["Aether Searcher", "Mandate of Peace"]));
    let mandate = t.hand(P0, "Mandate of Peace");
    search_hand_and_cast(&mut t, mandate);
    t.enter(P0, "Aether Searcher");
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(mandate)), Zone::Hand(P0));
    // A land can't be played this way.
    let mut t = game_with(drafted(&["Aether Searcher", "Forest"]));
    let forest = t.hand(P0, "Forest");
    search_hand_and_cast(&mut t, forest);
    t.enter(P0, "Aether Searcher");
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(forest)), Zone::Hand(P0));
}

#[test]
fn distant_memories_opponents_decide_in_turn_order_about_a_face_up_card() {
    cr!("101.4", "406.3");
    ruling!(
        "Distant Memories",
        "If no opponent has you put the card into your hand, the card remains exiled."
    );
    ruling!(
        "Distant Memories",
        "In multiplayer formats, your opponents each make the decision in turn order."
    );
    ruling!(
        "Distant Memories",
        "The card is exiled face up. Each opponent will know what the card is when deciding whether to have you put it in your hand."
    );
    supported("Distant Memories");
    for p2_yes in [false, true] {
        let mut t = TestGame::with_config(3, GameConfig::free_for_all());
        t.set_step(P0, Step::PrecombatMain);
        let bolt = t.library_top(P0, "Lightning Bolt");
        t.answer_choose_moved(P0, &[obj(bolt)]);
        let face_up = watch_face_up(&mut t, bolt);
        t.answer_yes(P1, false);
        t.answer_yes(P2, p2_yes);
        let from = t.asked().len();
        let hand = t.hand_size(P0);
        cast_resolve(&mut t, P0, "Distant Memories", &[]);
        let askers: Vec<PlayerId> = t.asked()[from..]
            .iter()
            .filter(|(p, d)| *p != P0 && matches!(d, Decision::YesNo { .. }))
            .map(|(p, _)| *p)
            .collect();
        assert_eq!(askers, vec![P1, P2]);
        assert_eq!(face_up.lock().unwrap().clone(), vec![true, true]);
        let bolt = t.g.current(bolt);
        if p2_yes {
            assert_eq!(t.zone(bolt), Zone::Hand(P0));
            assert_eq!(t.hand_size(P0), hand + 1);
        } else {
            assert_eq!(t.zone(bolt), Zone::Exile);
            assert_eq!(t.hand_size(P0), hand + 3);
        }
    }
}

/// Records, at each yes/no decision of P1 and P2, whether the card `card` (followed
/// across zones) is face up in exile.
fn watch_face_up(t: &mut TestGame, card: ObjectId) -> crate::r_s01_common::Seen<bool> {
    use std::sync::{Arc, Mutex};
    let seen: crate::r_s01_common::Seen<bool> = Arc::new(Mutex::new(Vec::new()));
    for p in [P1, P2] {
        struct W {
            inner: Box<dyn mtg_engine::decision::Agent>,
            card: ObjectId,
            seen: crate::r_s01_common::Seen<bool>,
        }
        impl mtg_engine::decision::Agent for W {
            fn decide(
                &mut self,
                g: &mtg_engine::game::Game,
                p: PlayerId,
                d: &Decision,
            ) -> mtg_engine::decision::Answer {
                if matches!(d, Decision::YesNo { .. }) {
                    let c = g.current(self.card);
                    self.seen
                        .lock()
                        .unwrap()
                        .push(g.obj(c).zone == Zone::Exile && !g.obj(c).face_down);
                }
                self.inner.decide(g, p, d)
            }
        }
        let mut agents = t.g.agents.0.lock().unwrap();
        let inner = std::mem::replace(
            &mut agents[p.idx()],
            Box::new(mtg_engine::decision::PassiveAgent),
        );
        agents[p.idx()] = Box::new(W {
            inner,
            card,
            seen: seen.clone(),
        });
    }
    seen
}

#[test]
fn demonic_bargain_must_find_a_card_and_keeps_the_thirteen_exiled() {
    cr!("701.23d");
    ruling!(
        "Demonic Bargain",
        "If there are any cards left in your library as you search, you must choose one of them."
    );
    ruling!(
        "Demonic Bargain",
        "If there are thirteen or fewer cards remaining in your library as this spell begins to resolve, you will exile your entire library."
    );
    ruling!("Demonic Bargain", "The thirteen cards remain in exile.");
    supported("Demonic Bargain");
    // More than thirteen cards: one is found even if P0 tries to find none.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.g.players[0].library.clear();
    for _ in 0..15 {
        t.library_top(P0, "Grizzly Bears");
    }
    t.answer_choose(P0, &[]);
    let hand = t.hand_size(P0);
    cast_resolve(&mut t, P0, "Demonic Bargain", &[]);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.library_size(P0), 1);
    let exiled = t.g.exile.iter().filter(|c| t.g.obj(**c).owner == P0).count();
    assert_eq!(exiled, 13);
    // Thirteen cards: all exiled, nothing to put into the hand.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.g.players[0].library.clear();
    for _ in 0..13 {
        t.library_top(P0, "Grizzly Bears");
    }
    let hand = t.hand_size(P0);
    cast_resolve(&mut t, P0, "Demonic Bargain", &[]);
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.library_size(P0), 0);
}

#[test]
fn maralen_stops_every_draw_and_its_trigger_is_mandatory() {
    cr!("121.1", "504.1");
    ruling!(
        "Maralen of the Mornsong",
        "Maralen prevents players from drawing cards for any reason"
    );
    ruling!(
        "Maralen of the Mornsong",
        "The effect of the triggered ability is mandatory. The life loss isn’t a payment"
    );
    supported("Maralen of the Mornsong");
    let mut t = TestGame::new(2);
    let maralen = t.battlefield(P1, "Maralen of the Mornsong");
    t.set_step(P1, Step::End);
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.library_top(P0, "Grizzly Bears");
    t.g.players[0].life = 2;
    t.answer_choose(P0, &[]);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::Draw);
    t.resolve_all();
    t.settle();
    // No draw; one card searched for even though P0 tried to find none; 3 life lost.
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(
        !t.g.turn_events.iter().any(
            |e| matches!(e, mtg_engine::events::Event::Drew { player, .. } if *player == P0)
        ),
        "the draw step's draw didn't happen"
    );
    assert_eq!(triggered_from(&t, maralen), 1);
    assert!(t.has_lost(P0), "3 life lost at 2 life");
    let _ = bolt;
    // Drawing by an effect does nothing either.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Maralen of the Mornsong");
    let hand = t.hand_size(P0);
    cast_resolve(&mut t, P0, "Divination", &[]);
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn maralen_triggers_for_each_player_of_the_active_team() {
    cr!("810.2", "810.4");
    ruling!(
        "Maralen of the Mornsong",
        "In a Two-Headed Giant game, Maralen’s triggered ability will trigger twice per draw step"
    );
    let mut t = TestGame::with_config(4, GameConfig::two_headed_giant(vec![0, 0, 1, 1]));
    let maralen = t.battlefield(P2, "Maralen of the Mornsong");
    t.set_step(P2, Step::End);
    let life = t.life(P0);
    t.advance_to(P0, Step::Draw);
    t.settle();
    assert_eq!(triggered_from(&t, maralen), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), life - 6, "the team loses 3 twice");
}

#[test]
fn grozoth_finds_cards_with_mana_value_exactly_nine() {
    cr!("202.3", "701.23a");
    ruling!(
        "Grozoth",
        "The search ability lets you find cards whose mana values are each equal to exactly 9"
    );
    supported("Grozoth");
    let mut t = TestGame::new(2);
    let other = t.library_top(P0, "Grozoth");
    let crusher = t.library_top(P0, "Ulamog's Crusher");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[obj(other), obj(crusher), obj(bears)]);
    let from = t.asked().len();
    t.enter(P0, "Grozoth");
    t.resolve_all();
    let cands = crate::r_s03_common::choice_candidates(&t, from, "");
    assert!(cands.iter().any(|c| c.contains(&obj(other))));
    assert!(cands.iter().all(|c| !c.contains(&obj(crusher)) && !c.contains(&obj(bears))));
    assert_eq!(t.zone(t.g.current(other)), Zone::Hand(P0));
    assert_eq!(t.zone(t.g.current(crusher)), Zone::Library(P0));
}

#[test]
fn demonic_tutor_puts_the_card_into_the_hand_without_drawing() {
    cr!("121.1", "701.23a");
    ruling!(
        "Demonic Tutor",
        "This card is put directly into your hand. It is not drawn."
    );
    supported("Demonic Tutor");
    // Maralen: "Players can't draw cards." The card still goes to the hand.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P1, "Maralen of the Mornsong");
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.answer_choose_moved(P0, &[obj(bolt)]);
    cast_resolve(&mut t, P0, "Demonic Tutor", &[]);
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Hand(P0));
    assert_eq!(t.g.history.draws_proposed.get(&P0).copied().unwrap_or(0), 0);
}

#[test]
fn spire_serpent_keeps_attacking_after_metalcraft_ends() {
    cr!("506.4a");
    ruling!(
        "Spire Serpent",
        "If Spire Serpent has been declared as an attacker, and you stop controlling three or more artifacts later in combat, Spire Serpent continues being an attacking creature until combat ends."
    );
    supported("Spire Serpent");
    let mut t = TestGame::new(2);
    let serpent = t.battlefield(P0, "Spire Serpent");
    let rings: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Ornithopter")).collect();
    crate::r_s01_common::attack_with(&mut t, &[(serpent, Entity::Player(P1))]);
    destroy(&mut t, rings[0]);
    assert_eq!(t.pt(serpent), (3, 5));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn sentry_oak_triggers_at_the_beginning_of_combat_on_its_controllers_turn() {
    cr!("603.2b");
    ruling!(
        "Sentry Oak",
        "Specifically, this ability triggers when the beginning of combat step starts during your turn."
    );
    supported("Sentry Oak");
    let mut t = TestGame::new(2);
    let oak = t.battlefield(P0, "Sentry Oak");
    t.set_step(P1, Step::PrecombatMain);
    t.advance_to(P1, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(triggered_from(&t, oak), 0, "not on the opponent's turn");
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(triggered_from(&t, oak), 0);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(triggered_from(&t, oak), 1);
}

#[test]
fn wakestone_gargoyle_lets_itself_attack() {
    cr!("702.3b");
    ruling!(
        "Wakestone Gargoyle",
        "Wakestone Gargoyle’s ability allows itself to attack."
    );
    supported("Wakestone Gargoyle");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Wakestone Gargoyle");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, g));
    t.lands(P0, "Plains", 2);
    crate::r_s06_common::activate_containing(&mut t, P0, g, "can attack").unwrap();
    t.resolve_all();
    assert!(can_attack(&mut t, g));
}
