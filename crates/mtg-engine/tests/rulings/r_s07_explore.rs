//! Rulings batch S07 — explore (CR 701.44): reveal the top card of your library; a land
//! card goes to your hand, otherwise put a +1/+1 counter on the exploring permanent and
//! you may put the revealed card into your graveyard.

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s05_common::*;
use crate::r_s07_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::decision::Decision;
use mtg_engine::events::{Event, MoveCause};
use mtg_engine::game::Game;
use mtg_engine::kwa::explore::{EXPLORED, REVEALED_NONLAND};
use mtg_engine::object::Zone;
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The explore events so far: (player, permanent, what was revealed).
fn explored(t: &TestGame) -> Vec<(Option<PlayerId>, Option<ObjectId>, i32)> {
    t.g.turn_events
        .iter()
        .chain(t.g.events.iter())
        .filter_map(|e| match e {
            Event::Custom {
                name,
                player,
                obj,
                amount,
            } if name == EXPLORED => Some((*player, *obj, *amount)),
            _ => None,
        })
        .collect()
}

/// Puts real cards onto the battlefield under `p`'s control at the same time, and puts
/// the triggers on the stack.
fn enter_together(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    let moves = names
        .iter()
        .map(|n| MoveEv {
            obj: t.g.create_card_object(card(n), p, Zone::Nowhere),
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
        .collect();
    let ids = t.g.move_objects(moves).into_iter().flatten().collect();
    t.g.flush_events();
    t.settle();
    ids
}

fn is_yes_no(d: &Decision) -> bool {
    matches!(d, Decision::YesNo { .. })
}

/// The +1/+1 counters on the Grizzly Bears on the battlefield.
fn bears_counters(g: &Game) -> u32 {
    g.find_in_zone(Zone::Battlefield, "Grizzly Bears")
        .first()
        .map_or(0, |b| g.obj(*b).counter(counters::PLUS1))
}

#[test]
fn nothing_can_happen_while_a_creature_explores() {
    cr!("701.44a", "608.2");
    ruling!(
        "Path of Discovery",
        "Once an ability that causes a creature to explore begins to resolve, no player may take any other actions until it's done. Notably, opponents can't try to remove the exploring creature after you reveal a nonland card but before it receives a counter."
    );
    supported("Path of Discovery");
    // Path of Discovery: "Whenever a creature you control enters, it explores."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Path of Discovery");
    let hill = t.library_top(P0, "Hill Giant");
    let bears = enter(&mut t, P0, "Grizzly Bears");
    assert_eq!(t.stack_len(), 1);
    let seen = watch(&mut t, P0, is_yes_no, bears_counters);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.resolve();
    // When P0 chose whether to put the revealed card into the graveyard, the Bears
    // already had the counter; nobody got priority in between.
    assert_eq!(*seen.lock().unwrap(), vec![1]);
    assert_eq!(count_asked(&t, from, is_priority), 0);
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(t.zone(hill), Zone::Graveyard(P0));
}

#[test]
fn exploring_again_reveals_the_same_card() {
    cr!("701.44a");
    ruling!(
        "Path of Discovery",
        "Some spells or abilities might cause a creature to explore multiple times in a row. If you reveal a nonland card when a creature explores and leave it on top of your library, then the creature explores again immediately afterwards, you'll reveal the same card again."
    );
    // Two creatures enter at once: each explores; the nonland card revealed first is left
    // on top, and revealed again.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Path of Discovery");
    let hill = t.library_top(P0, "Hill Giant");
    let ids = enter_together(&mut t, P0, &["Grizzly Bears", "Llanowar Elves"]);
    assert_eq!(t.stack_len(), 2);
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.counters(ids[0], counters::PLUS1), 1);
    assert_eq!(t.counters(ids[1], counters::PLUS1), 1);
    assert_eq!(t.g.library_top(P0), Some(hill));
    let ex = explored(&t);
    assert_eq!(ex.len(), 2);
    assert!(ex.iter().all(|(_, _, r)| *r == REVEALED_NONLAND));
}

#[test]
fn a_noncreature_permanent_can_explore() {
    cr!("701.44a", "115.1");
    ruling!(
        "Path of Discovery",
        "In some unusual cases, noncreature permanents may explore."
    );
    supported("Spyglass Siren");
    // A creature enters with Path of Discovery; before "it explores" resolves, an effect
    // makes it a noncreature permanent. It still explores and gets the counter.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Path of Discovery");
    t.library_top(P0, "Hill Giant");
    let bears = enter(&mut t, P0, "Grizzly Bears");
    run_from(
        &mut t,
        P1,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![
                Modification::RemoveTypes(vec![CardType::Creature]),
                Modification::AddTypes(vec![CardType::Artifact]),
            ],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(bears)],
    );
    assert!(!t.obj_now(bears).is(CardType::Creature));
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(explored(&t), vec![(Some(P0), Some(bears), REVEALED_NONLAND)]);
    // A Map token's ability targets a creature: the noncreature permanent can't be chosen.
    let siren = enter(&mut t, P0, "Spyglass Siren");
    t.answer_yes(P0, false);
    t.resolve_all();
    let map = tokens_with_subtype(&t, P0, "Map")[0];
    let cands = ability_targets(&mut t, map, 0);
    assert!(!cands.contains(&Entity::Object(bears)));
    assert!(cands.contains(&Entity::Object(t.g.current(siren))));
}

#[test]
fn a_creature_that_left_the_battlefield_still_explores() {
    cr!("701.44a", "701.44c");
    ruling!(
        "Path of Discovery",
        "If a resolving spell or ability instructs a specific creature to explore but that creature has left the battlefield, the creature still explores. If you reveal a nonland card this way, you won't put a +1/+1 counter on anything, but you may put the revealed card into your graveyard. Effects that trigger \"whenever a creature explores\" trigger as appropriate."
    );
    supported("Wildgrowth Walker");
    // Wildgrowth Walker: "Whenever a creature you control explores, put a +1/+1 counter on
    // this creature and you gain 3 life."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Path of Discovery");
    let walker = t.battlefield(P0, "Wildgrowth Walker");
    let hill = t.library_top(P0, "Hill Giant");
    let bears = enter(&mut t, P0, "Grizzly Bears");
    // Wildgrowth Walker didn't enter; only the Bears' trigger is on the stack.
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, bears);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(explored(&t), vec![(Some(P0), Some(bears), REVEALED_NONLAND)]);
    assert_eq!(t.zone(hill), Zone::Graveyard(P0));
    assert_eq!(t.counters(walker, counters::PLUS1), 1);
    assert_eq!(t.life(P0), 23);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn a_creature_that_left_the_battlefield_still_explores_branchwalker() {
    cr!("701.44a", "701.44c");
    ruling!(
        "Merfolk Branchwalker",
        "If a resolving spell or ability instructs a specific creature to explore but that creature has left the battlefield, the creature still explores. If you reveal a nonland card this way, you won't put a +1/+1 counter on anything, but you may put the revealed card into your graveyard. Effects that trigger \"whenever a creature you control explores\" trigger if appropriate."
    );
    supported("Merfolk Branchwalker");
    // Merfolk Branchwalker: "When this creature enters, it explores." It leaves the
    // battlefield before the ability resolves.
    let mut t = TestGame::new(2);
    let walker = t.battlefield(P0, "Wildgrowth Walker");
    let hill = t.library_top(P0, "Hill Giant");
    let bw = enter(&mut t, P0, "Merfolk Branchwalker");
    assert_eq!(t.stack_len(), 1);
    t.g.move_object(bw, Zone::Hand(P0), MoveCause::Effect, None);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(explored(&t), vec![(Some(P0), Some(bw), REVEALED_NONLAND)]);
    assert_eq!(t.g.library_top(P0), Some(hill));
    assert_eq!(t.counters(walker, counters::PLUS1), 1);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn map_tokens() {
    cr!("111.10s", "701.44a", "602.5d");
    ruling!(
        "Get Lost",
        "Map tokens are a kind of predefined token. Each one is a colorless artifact with the artifact subtype Map and the ability \"{1}, {T}, Sacrifice this artifact: Target creature you control explores. Activate only as a sorcery.\""
    );
    supported("Get Lost");
    // Get Lost: "Destroy target creature, enchantment, or planeswalker. Its controller
    // creates two Map tokens."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 2);
    let gl = t.hand(P0, "Get Lost");
    t.cast(P0, gl).target(giant).go();
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    let maps = tokens_with_subtype(&t, P1, "Map");
    assert_eq!(maps.len(), 2);
    let o = t.obj_now(maps[0]);
    assert!(o.chars.colors.is_colorless());
    assert!(o.is(CardType::Artifact));
    assert!(!o.is(CardType::Creature));
    assert!(o.chars.has_subtype("Map"));
    // P1 uses one in their main phase: their creature explores.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Wastes", 2);
    t.library_top(P1, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    assert!(can_activate(&mut t, P1, maps[0]));
    t.answer_yes(P1, false);
    t.activate(P1, maps[0], 0, &[Entity::Object(bears)]).unwrap();
    assert!(!t.on_battlefield(maps[0]));
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    // Only as a sorcery: not in P0's turn, nor with something on the stack.
    t.set_step(P0, Step::PrecombatMain);
    assert!(!can_activate(&mut t, P1, maps[1]));
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(!can_activate(&mut t, P1, maps[1]));
}
