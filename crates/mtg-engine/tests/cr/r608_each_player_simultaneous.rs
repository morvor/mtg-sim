//! CR 101.4, 608.2e, 608.2f: an instruction several players perform ("each player ...")
//! — each makes the choices it requires in APNAP order, then the actions happen at the
//! same time; with several instructions, all the players perform the first one before
//! the second.

use crate::r703_common::oracle_card;
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn chooses_since(t: &TestGame, from: usize) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect()
}

#[test]
fn the_active_player_chooses_first_even_when_another_player_controls_the_spell() {
    cr!("101.4");
    // P1's turn; P1 casts Exhume: P1 (the active player) chooses, then P0.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let a = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    let b = t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.answer_choose(P1, &[Entity::Object(b)]);
    t.lands(P1, "Swamp", 2);
    let spell = t.hand(P1, "Exhume");
    let from = t.asked().len();
    t.cast(P1, spell).go();
    t.resolve_all();
    assert_eq!(chooses_since(&t, from), vec![P1, P0]);
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 2);
    let mut controllers: Vec<PlayerId> = bears.iter().map(|o| t.obj_now(*o).controller).collect();
    controllers.sort();
    assert_eq!(controllers, vec![P0, P1]);
}

#[test]
fn every_player_performs_an_instruction_before_anyone_performs_the_next() {
    cr!("608.2e", "608.2c");
    // "Each player mills two cards, then returns a creature card from their graveyard to
    // their hand."
    let def = oracle_card(
        "Shared Recollection",
        "Sorcery",
        "{2}",
        None,
        "Each player mills two cards, then returns a creature card from their graveyard to their hand.",
    );
    let mut t = TestGame::new(2);
    for p in [P0, P1] {
        t.library_top(p, "Grizzly Bears");
        t.library_top(p, "Forest");
    }
    t.lands(P0, "Forest", 2);
    let spell = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, spell).go();
    let from = t.g.turn_events.len();
    t.resolve_all();
    let mut milled = Vec::new();
    let mut returned = Vec::new();
    for (i, e) in t.g.turn_events.iter().enumerate().skip(from) {
        match e {
            Event::Milled { .. } => milled.push(i),
            Event::ZoneChange { to: Zone::Hand(_), .. } => returned.push(i),
            _ => {}
        }
    }
    assert_eq!(milled.len(), 2);
    assert_eq!(returned.len(), 2);
    // Both players milled before either returned a card.
    assert!(milled.iter().max() < returned.iter().min());
    for p in [P0, P1] {
        assert!(t.in_hand(p, "Grizzly Bears"));
        assert!(t.in_graveyard(p, "Forest"));
    }
}

#[test]
fn choices_are_made_before_any_player_acts() {
    cr!("101.4", "101.4b", "608.2f");
    // "Each player returns a creature card from their graveyard to their hand, then
    // discards a card": when P1 chooses which card to return, P0's chosen card is still
    // in P0's graveyard, but P1 knows P0's choice.
    let def = oracle_card(
        "Shared Regret",
        "Sorcery",
        "{2}",
        None,
        "Each player returns a creature card from their graveyard to their hand, then discards a card.",
    );
    let mut t = TestGame::new(2);
    let a = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    let b = t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.answer_choose(P1, &[Entity::Object(b)]);
    let seen: std::sync::Arc<std::sync::Mutex<Vec<(usize, Vec<(PlayerId, bool)>)>>> =
        Default::default();
    {
        struct Look {
            inner: Box<dyn mtg_engine::decision::Agent>,
            seen: std::sync::Arc<std::sync::Mutex<Vec<(usize, Vec<(PlayerId, bool)>)>>>,
        }
        impl mtg_engine::decision::Agent for Look {
            fn decide(
                &mut self,
                g: &mtg_engine::game::Game,
                p: PlayerId,
                d: &Decision,
            ) -> mtg_engine::decision::Answer {
                if matches!(d, Decision::ChooseEntities { .. }) {
                    self.seen.lock().unwrap().push((
                        g.player(P0).graveyard.len(),
                        g.known_apnap_choices(p)
                            .iter()
                            .map(|(q, c)| (*q, c.is_some()))
                            .collect(),
                    ));
                }
                self.inner.decide(g, p, d)
            }
        }
        let mut agents = t.g.agents.0.lock().unwrap();
        let inner = std::mem::replace(
            &mut agents[P1.idx()],
            Box::new(mtg_engine::decision::PassiveAgent),
        );
        agents[P1.idx()] = Box::new(Look {
            inner,
            seen: seen.clone(),
        });
    }
    t.lands(P0, "Forest", 2);
    let spell = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, spell).go();
    t.resolve_all();
    // P1's first choice: P0's two cards are still in the graveyard, and P0's public
    // choice (made in a graveyard) is known.
    let first = seen.lock().unwrap()[0].clone();
    assert_eq!(first, (2, vec![(P0, true)]));
    assert!(t.in_graveyard(P0, "Hill Giant") && t.in_graveyard(P1, "Hill Giant"));
    // Each then discarded a card (the one they returned, the only card in hand).
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.hand_size(P1), 0);
}

#[test]
fn with_shared_team_turns_the_active_team_chooses_first() {
    cr!("805.6", "101.4");
    // Two-Headed Giant (P0 and P1 against P2 and P3), on P1's team's turn: P1 casts
    // Exhume. Both players of the active team choose before either player of the other
    // team, then all four creatures enter together.
    let mut t = TestGame::with_config(
        4,
        mtg_engine::game::GameConfig::two_headed_giant(vec![0, 0, 1, 1]),
    );
    t.set_step(P1, Step::PrecombatMain);
    let mut picks = Vec::new();
    for p in [P0, P1, P2, P3] {
        let bears = t.graveyard(p, "Grizzly Bears");
        t.graveyard(p, "Hill Giant");
        t.answer_choose(p, &[Entity::Object(bears)]);
        picks.push(bears);
    }
    t.lands(P1, "Swamp", 2);
    let spell = t.hand(P1, "Exhume");
    let from = t.asked().len();
    t.cast(P1, spell).go();
    t.resolve_all();
    let order = chooses_since(&t, from);
    assert_eq!(order.len(), 4, "{order:?}");
    assert!(order[..2].contains(&P0) && order[..2].contains(&P1), "{order:?}");
    assert!(order[2..].contains(&P2) && order[2..].contains(&P3), "{order:?}");
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 4);
}

#[test]
fn each_player_uses_their_own_x_in_their_next_instruction() {
    cr!("608.2e", "107.3");
    // "Each player draws a card, then loses X life, where X is the number of cards in
    // their hand": everyone draws, then everyone loses life — each player as much as their
    // own X, not the last player's.
    use crate::r600_common::CB;
    use mtg_engine::ability::*;
    let body = Effect::ForEachPlayer {
        who: PlayerRef::EachPlayer,
        effect: Box::new(Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect: Box::new(Effect::Seq(vec![
                Effect::Draw {
                    who: PlayerRef::You,
                    n: Value::c(1),
                },
                Effect::SetX {
                    value: Value::HandSize(PlayerRef::You),
                },
                Effect::LoseLife {
                    who: PlayerRef::You,
                    n: Value::X,
                },
            ])),
        }),
    };
    let def = CB::new("Shared Burden")
        .sorcery()
        .cost("{0}")
        .spell(Body::effect(body))
        .build();
    let mut t = TestGame::new(3);
    t.hand(P1, "Forest");
    for _ in 0..3 {
        t.hand(P2, "Island");
    }
    let spell = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, spell).go();
    let from = t.g.turn_events.len();
    t.resolve_all();
    let mut draws = Vec::new();
    let mut losses = Vec::new();
    for (i, e) in t.g.turn_events.iter().enumerate().skip(from) {
        match e {
            Event::Drew { .. } => draws.push(i),
            Event::LifeLost { .. } => losses.push(i),
            _ => {}
        }
    }
    assert_eq!((draws.len(), losses.len()), (3, 3));
    assert!(draws.iter().max() < losses.iter().min());
    // Hands after drawing: P0 1 card, P1 2, P2 4.
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P2), 16);
}
