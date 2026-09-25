//! CR 101: the Magic golden rules — cards override rules, "can't" beats "can", impossible
//! instructions are ignored, and simultaneous choices are made in APNAP order.

use crate::r100_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::game::{Game, GameConfig};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::{Arc, Mutex};

#[test]
fn card_text_that_contradicts_a_rule_takes_precedence() {
    // Normally a player may play only one land each turn (CR 305.2); Exploration says
    // otherwise, and the card wins.
    cr!("101.1");
    let mut t = TestGame::new(2);
    let a = t.hand(P0, "Forest");
    let b = t.hand(P0, "Forest");
    t.play_land(P0, a).unwrap();
    assert!(t.play_land(P0, b).is_err(), "one land per turn by the rules");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Exploration");
    let a = t.hand(P0, "Forest");
    let b = t.hand(P0, "Forest");
    t.play_land(P0, a).unwrap();
    t.play_land(P0, b).unwrap();
    assert_eq!(t.named_on_battlefield("Forest").len(), 2);
}

/// P0 resolves Explore ("You may play an additional land this turn.") and Turf Wound
/// ("Target player can't play lands this turn.") targeting themself, in the given order.
fn explore_and_turf_wound(explore_first: bool) -> TestGame {
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    t.lands(P0, "Mountain", 3);
    let explore = t.hand(P0, "Explore");
    let wound = t.hand(P0, "Turf Wound");
    let order = if explore_first {
        [explore, wound]
    } else {
        [wound, explore]
    };
    for s in order {
        if s == wound {
            t.cast(P0, s).target(Entity::Player(P0)).go();
        } else {
            t.cast(P0, s).go();
        }
        t.resolve();
    }
    t
}

#[test]
fn cant_takes_precedence_over_may() {
    // The rule's own example: "You may play an additional land this turn" and "You can't
    // play lands this turn" — the effect that precludes playing lands wins, whatever the
    // order.
    cr!("101.2");
    for explore_first in [true, false] {
        let mut t = explore_and_turf_wound(explore_first);
        let land = t.hand(P0, "Forest");
        assert!(t.play_land(P0, land).is_err());
        assert!(t.in_hand(P0, "Forest"));
    }
    // Explore alone does allow a second land.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let explore = t.hand(P0, "Explore");
    t.cast(P0, explore).go();
    t.resolve();
    let a = t.hand(P0, "Forest");
    let b = t.hand(P0, "Forest");
    t.play_land(P0, a).unwrap();
    t.play_land(P0, b).unwrap();
}

#[test]
fn adding_and_removing_abilities_follow_timestamps_not_the_cant_rule() {
    // Losing an ability isn't a "can't": whichever effect is more recent wins.
    cr!("101.2a");
    let flies = |t: &TestGame, id: ObjectId| t.obj_now(id).has_keyword(KeywordKind::Flying);
    // Loses flying (Canopy Claws), then gains flying (Angelic Blessing): it flies.
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P0, "Wind Drake");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Plains", 3);
    let claws = t.hand(P0, "Canopy Claws");
    t.cast(P0, claws).target(drake).go();
    t.resolve();
    assert!(!flies(&t, drake));
    let blessing = t.hand(P0, "Angelic Blessing");
    t.cast(P0, blessing).target(drake).go();
    t.resolve();
    assert!(flies(&t, drake));
    // Gains flying, then loses it: it doesn't fly.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    let blessing = t.hand(P0, "Angelic Blessing");
    t.cast(P0, blessing).target(bear).go();
    t.resolve();
    assert!(flies(&t, bear));
    t.lands(P0, "Forest", 1);
    let claws = t.hand(P0, "Canopy Claws");
    t.cast(P0, claws).target(bear).go();
    t.resolve();
    assert!(!flies(&t, bear));
}

#[test]
fn impossible_parts_of_an_instruction_are_ignored() {
    cr!("101.3");
    // "Target player discards two cards." with one card in hand: that card is discarded.
    let mut t = TestGame::new(2);
    t.hand(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    let rot = t.hand(P0, "Mind Rot");
    t.lands(P0, "Swamp", 2);
    t.cast(P0, rot).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.hand_size(P1), 0);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // "Each player sacrifices a creature of their choice." when an opponent has none: the
    // rest still happens.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    let blood = t.hand(P0, "Innocent Blood");
    t.cast(P0, blood).go();
    t.resolve();
    assert!(!t.on_battlefield(bear));
}

// ---------------------------------------------------------------------------
// APNAP order
// ---------------------------------------------------------------------------

/// An agent that records what a player knew about the other players' simultaneous
/// choices (and whether a chosen card was still in its hand) whenever it had to choose,
/// then answers like the scripted agent.
struct Observer {
    inner: ScriptedAgent,
    seen: Arc<Mutex<Vec<Seen>>>,
}

#[derive(Clone, Debug)]
struct Seen {
    player: PlayerId,
    known: Vec<(PlayerId, Option<Vec<ObjectId>>)>,
    p0_hand: usize,
    p0_graveyard: usize,
}

impl Agent for Observer {
    fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
        if matches!(d, Decision::ChooseEntities { .. }) {
            self.seen.lock().unwrap().push(Seen {
                player: p,
                known: g.known_apnap_choices(p),
                p0_hand: g.player(P0).hand.len(),
                p0_graveyard: g.player(P0).graveyard.len(),
            });
        }
        self.inner.decide(g, p, d)
    }
}

fn observe(t: &mut TestGame) -> Arc<Mutex<Vec<Seen>>> {
    let seen = Arc::new(Mutex::new(Vec::new()));
    for p in t.g.player_ids() {
        let inner = ScriptedAgent {
            player: p,
            script: t.script.clone(),
        };
        t.g.set_agent(
            p,
            Box::new(Observer {
                inner,
                seen: seen.clone(),
            }),
        );
    }
    seen
}

#[test]
fn simultaneous_choices_are_made_in_apnap_order_then_happen_at_once() {
    // The rule's example: "Each player sacrifices a creature."
    cr!("101.4");
    ruling!(
        "Earth-Cult Elemental",
        "Once all choices have been made, all permanents are sacrificed simultaneously."
    );
    let mut t = TestGame::new(3);
    t.set_step(P1, Step::PrecombatMain);
    // "Whenever this creature or another creature dies, target player loses 1 life and
    // you gain 1 life."
    t.battlefield(P0, "Blood Artist");
    let bears1 = t.battlefield(P1, "Grizzly Bears");
    let bears2 = t.battlefield(P2, "Grizzly Bears");
    let seen = observe(&mut t);
    t.lands(P1, "Swamp", 1);
    let blood = t.hand(P1, "Innocent Blood");
    t.cast(P1, blood).go();
    t.g.resolve_top();
    // The active player (P1) chose first, then P2, then P0.
    let order: Vec<PlayerId> = seen.lock().unwrap().iter().map(|s| s.player).collect();
    assert_eq!(order, vec![P1, P2, P0]);
    assert!(!t.on_battlefield(bears1) && !t.on_battlefield(bears2));
    // Blood Artist died at the same time as both Bears, so it saw them die: three
    // triggers.
    t.settle();
    assert_eq!(t.stack_len(), 3);
}

#[test]
fn later_players_know_the_choices_of_earlier_players() {
    cr!("101.4b");
    ruling!(
        "Earth-Cult Elemental",
        "Players later in turn order will know which permanents were picked by earlier players while they make this choice."
    );
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P1, "Wind Drake");
    t.answer_choose(P0, &[Entity::Object(bear)]);
    let seen = observe(&mut t);
    t.lands(P0, "Swamp", 1);
    let blood = t.hand(P0, "Innocent Blood");
    t.cast(P0, blood).go();
    t.resolve();
    let seen = seen.lock().unwrap().clone();
    assert_eq!(seen[0].player, P0);
    assert!(seen[0].known.is_empty());
    assert_eq!(seen[1].player, P1);
    assert_eq!(seen[1].known, vec![(P0, Some(vec![bear]))]);
}

#[test]
fn cards_chosen_in_a_hidden_zone_stay_face_down_as_theyre_chosen() {
    cr!("101.4a", "101.4b");
    let mut t = TestGame::new(2);
    let a = t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Wind Drake");
    t.hand(P1, "Llanowar Elves");
    t.answer_choose(P0, &[Entity::Object(a)]);
    let seen = observe(&mut t);
    // "+1: Each player discards a card."
    let lili = t.battlefield(P0, "Liliana of the Veil");
    t.activate(P0, lili, 0, &[]).unwrap();
    t.resolve();
    let seen = seen.lock().unwrap().clone();
    assert_eq!(seen.len(), 2);
    // When P1 chose, P0's card was still face down in P0's hand, and P1 knew only that
    // P0 had chosen.
    assert_eq!(seen[1].player, P1);
    assert_eq!(seen[1].p0_hand, 2);
    assert_eq!(seen[1].p0_graveyard, 0);
    assert_eq!(seen[1].known, vec![(P0, None)]);
    // P0 knows their own choice.
    t.g.apnap_choices.push(mtg_engine::apnap::ApnapChoice {
        player: P0,
        chosen: vec![a],
        hidden: true,
    });
    assert_eq!(t.g.known_apnap_choices(P0), vec![(P0, Some(vec![a]))]);
    // Both cards were discarded afterwards.
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
}

#[test]
fn several_choices_at_once_are_made_in_the_order_specified() {
    // Cataclysm: "Each player chooses from among the permanents they control an artifact,
    // a creature, an enchantment, and a land, then sacrifices the rest." The artifact is
    // chosen first; an artifact creature chosen as the artifact can't also be the
    // creature.
    cr!("101.4c", "101.4");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let anthem = t.battlefield(P0, "Glorious Anthem");
    let plains: Vec<ObjectId> = t.lands(P0, "Plains", 4);
    let drake = t.battlefield(P1, "Wind Drake");
    let island = t.battlefield(P1, "Island");
    t.answer_choose(P0, &[Entity::Object(thopter)]);
    t.answer_choose(P0, &[Entity::Object(bear)]);
    let cata = t.hand(P0, "Cataclysm");
    t.cast(P0, cata).go();
    t.resolve();
    // P0's choices: artifact among {Ornithopter}; creature among the others.
    let asked: Vec<(PlayerId, Vec<Entity>)> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some((p, candidates)),
            _ => None,
        })
        .collect();
    assert_eq!(asked[0], (P0, vec![Entity::Object(thopter)]));
    assert_eq!(
        asked[1],
        (P0, vec![Entity::Object(bear), Entity::Object(elves)])
    );
    assert!(asked[2..].iter().any(|(p, _)| *p == P1));
    for kept in [thopter, bear, anthem, plains[0]] {
        assert!(t.on_battlefield(kept));
    }
    assert!(!t.on_battlefield(elves));
    assert_eq!(t.named_on_battlefield("Plains").len(), 1);
    assert!(t.on_battlefield(drake) && t.on_battlefield(island));
}

#[test]
fn while_starting_the_game_the_starting_player_is_the_active_player() {
    // Mulligan declarations begin with the starting player (CR 103.5), who is treated as
    // the active player (CR 101.4e).
    cr!("101.4e");
    let mut t = pregame(
        GameConfig {
            starting_player: Some(P2),
            ..Default::default()
        },
        vec![fillers(20), fillers(20), fillers(20)],
    );
    t.g.start();
    let order: Vec<PlayerId> = t
        .asked()
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::Mulligan { .. }))
        .map(|(p, _)| p)
        .collect();
    assert_eq!(order, vec![P2, P0, P1]);
}


#[test]
fn a_nonactive_players_choice_that_makes_an_earlier_player_choose_restarts_apnap_order() {
    cr!("101.4d");
    // Three players choose at once (P1 is active). P2's choice makes the active player P1
    // and the later player P0 choose again: P1 chooses next (APNAP order restarts for all
    // outstanding choices), then P0's two choices.
    let mut t = TestGame::new(3);
    t.set_step(P1, Step::PrecombatMain);
    let requests = vec![(P0, "first"), (P1, "first"), (P2, "first")];
    let mut made: Vec<(PlayerId, &'static str)> = Vec::new();
    t.g.apnap_round(requests, |g, p, what| {
        // Each choice is a real decision the player makes.
        g.ask_yes_no(p, None, what, true);
        made.push((p, what));
        if p == P2 && what == "first" {
            vec![(P0, "caused"), (P1, "caused")]
        } else {
            vec![]
        }
    });
    assert_eq!(
        made,
        vec![
            (P1, "first"),
            (P2, "first"),
            (P1, "caused"),
            (P0, "first"),
            (P0, "caused"),
        ]
    );
    let asked: Vec<PlayerId> = t
        .asked()
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .map(|(p, _)| p)
        .collect();
    assert_eq!(asked, vec![P1, P2, P1, P0, P0]);
}
