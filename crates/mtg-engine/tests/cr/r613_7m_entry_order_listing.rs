//! CR 613.7m: "If two or more objects would receive a timestamp simultaneously, such as by
//! entering a zone simultaneously or becoming attached simultaneously, their relative
//! timestamps are determined in APNAP order (see rule 101.4). Objects controlled by the
//! active player (or owned by the active player, if they have no controller) have an earlier
//! relative timestamp in the order of that player's choice, followed by each other player in
//! turn order."
//!
//! The "Order objects entering the battlefield" decision lists a player's objects in an
//! order that doesn't depend on where the players sit: by owner in APNAP order, then in the
//! order the objects were created; with no answer, that's their timestamp order. (Found by
//! the seat symmetry property, game 405 of `--properties --seed 1`: the listing followed the
//! order the moves were proposed in, an order of object ids, and ids follow seats.)

use mtg_engine::ability::LibraryPosition;
use mtg_engine::decision::Decision;
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn enter_under(p: PlayerId, objs: &[ObjectId]) -> Vec<MoveEv> {
    objs.iter()
        .map(|o| MoveEv {
            obj: *o,
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(p),
            etb: EtbInfo {
                controller: Some(p),
                ..Default::default()
            },
            source: None,
        })
        .collect()
}

/// The items of the "Order objects entering the battlefield" decisions asked so far, by
/// card name.
fn entry_listings(t: &TestGame) -> Vec<Vec<String>> {
    t.asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::Order { prompt, items } if prompt.contains("entering the battlefield") => {
                Some(
                    items
                        .iter()
                        .map(|s| s.split(" #").next().unwrap_or_default().to_string())
                        .collect(),
                )
            }
            _ => None,
        })
        .collect()
}

#[test]
fn objects_entering_together_are_listed_the_same_wherever_the_players_sit() {
    cr!("613.7m");
    interp!("simultaneous-entries-listed-by-owner-then-creation");
    // Player A (active) owns a Hill Giant in their graveyard, player B a Grizzly Bears in
    // theirs; both enter the battlefield at once under A's control. The cards are created
    // seat by seat, so which has the smaller id depends on the seats; the moves are proposed
    // in id order. Either way A is asked to order Hill Giant (owned by the active player)
    // before Grizzly Bears, and with no answer Hill Giant is the older.
    for a in [P0, P1] {
        let b = if a == P0 { P1 } else { P0 };
        let mut t = TestGame::new(2);
        t.set_step(a, Step::PrecombatMain);
        let mut cards: Vec<ObjectId> = [P0, P1]
            .into_iter()
            .map(|p| {
                t.graveyard(
                    p,
                    if p == a {
                        "Hill Giant"
                    } else {
                        "Grizzly Bears"
                    },
                )
            })
            .collect();
        cards.sort();
        t.g.move_objects(enter_under(a, &cards));
        assert_eq!(
            entry_listings(&t),
            vec![vec!["Hill Giant".to_string(), "Grizzly Bears".to_string()]],
            "A = {a:?}"
        );
        let giant = t.g.current(cards[usize::from(a != P0)]);
        let bears = t.g.current(cards[usize::from(b != P0)]);
        assert_eq!(t.zone(giant), Zone::Battlefield);
        assert_eq!(t.zone(bears), Zone::Battlefield);
        assert!(t.obj(giant).timestamp < t.obj(bears).timestamp, "A = {a:?}");
    }
}

#[test]
fn one_owners_objects_entering_together_are_listed_in_creation_order() {
    cr!("613.7m");
    interp!("simultaneous-entries-listed-by-owner-then-creation");
    // Three of the active player's cards proposed in reverse: listed (and, with no answer,
    // timestamped) in the order they were created, whatever the order of the proposal.
    let mut t = TestGame::new(2);
    let names = ["Grizzly Bears", "Hill Giant", "Ornithopter"];
    let cards: Vec<ObjectId> = names.iter().map(|n| t.graveyard(P0, n)).collect();
    let reversed: Vec<ObjectId> = cards.iter().rev().copied().collect();
    t.g.move_objects(enter_under(P0, &reversed));
    assert_eq!(
        entry_listings(&t),
        vec![names.iter().map(|n| n.to_string()).collect::<Vec<_>>()]
    );
    let ts: Vec<Timestamp> = cards
        .iter()
        .map(|c| t.obj(t.g.current(*c)).timestamp)
        .collect();
    assert!(ts[0] < ts[1] && ts[1] < ts[2], "{ts:?}");
}
