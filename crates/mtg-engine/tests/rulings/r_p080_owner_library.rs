//! Rulings batch P080 — "target creature's owner puts it on their choice of the top or
//! bottom of their library": the owner chooses (also for a melded permanent, whose cards
//! all go to the same place, CR 712.21), and Aetherspouts' choices in APNAP order
//! (CR 101.4).

use crate::r_p079_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s06_common::give_control;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The top/bottom choices asked since `from`: (player, chose from options with "Bottom").
fn top_or_bottom_asks(t: &TestGame, from: usize) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| {
            matches!(d, Decision::ChooseOption { options, .. }
                if options.iter().any(|o| o.contains("Bottom")))
        })
        .map(|(p, _)| *p)
        .collect()
}

fn bottom_name(t: &TestGame, p: PlayerId, from_bottom: usize) -> String {
    let lib = &t.g.player(p).library;
    t.g.obj(lib[from_bottom]).chars.name.to_string()
}

/// P0 casts the spell at the P1-owned Grizzly Bears, which attacks P0 (P1's turn);
/// P1 answers "bottom". Returns the decisions' askers.
fn put_away(name: &str) -> (TestGame, Vec<PlayerId>) {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    t.resolve_all();
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    let from = t.asked().len();
    if name.contains(" // ") {
        // Lagoon Breach, the Adventure.
        let c = t.hand(P0, name);
        t.lands(P0, "Island", 2);
        t.answer_targets(P0, &[obj(bears)]);
        t.cast(P0, c).method(CastMethod::Half(1)).go();
    } else {
        cast_card(&mut t, P0, name, &[obj(bears)]).unwrap();
    }
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Library(P1), "{name}");
    assert_eq!(bottom_name(&t, P1, 0), "Grizzly Bears", "{name}");
    let asks = top_or_bottom_asks(&t, from);
    (t, asks)
}

#[test]
fn the_owner_chooses_top_or_bottom() {
    cr!("608.2c", "401.4");
    ruling!(
        "Uneasy Partings",
        "The creature's owner chooses whether to put it on the top or bottom of their library."
    );
    ruling!(
        "Misleading Motes",
        "The creature's owner chooses whether to put it on the top or bottom of their library."
    );
    ruling!(
        "Horned Loch-Whale // Lagoon Breach",
        "The creature's owner chooses whether to put it on the top or bottom of their library."
    );
    ruling!(
        "Dire Downdraft",
        "The creature’s owner chooses whether to put it on the top or bottom of their library."
    );
    for name in [
        "Uneasy Partings",
        "Misleading Motes",
        "Horned Loch-Whale // Lagoon Breach",
        "Dire Downdraft",
    ] {
        supported(name);
        let (_, asks) = put_away(name);
        assert_eq!(asks, vec![P1], "{name}");
    }
}

/// Melds Fang, Fearless l'Cie and Vanille, Cheerful l'Cie (owned by P0) into Ragnarok,
/// Divine Deliverance with Vanille's first-main-phase trigger.
fn ragnarok(t: &mut TestGame) -> ObjectId {
    t.battlefield(P0, "Fang, Fearless l'Cie");
    t.battlefield(P0, "Vanille, Cheerful l'Cie");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 3);
    t.answer_yes(P0, true);
    t.set_step(P0, Step::Draw);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    let r = t.named_on_battlefield("Ragnarok, Divine Deliverance");
    assert_eq!(r.len(), 1);
    r[0]
}

#[test]
fn a_melded_permanents_cards_all_go_on_top_or_all_on_the_bottom() {
    cr!("712.21", "401.4", "608.2c");
    ruling!(
        "Uneasy Partings",
        "If multiple cards are put into the library this way (such as when the spell targets a melded creature), that creature's owner puts all the cards on top or all the cards on the bottom."
    );
    ruling!(
        "Misleading Motes",
        "If multiple cards are put into the library this way (such as when the spell targets a melded permanent), that creature's owner puts all the cards on top or all the cards on the bottom."
    );
    ruling!(
        "Dire Downdraft",
        "If multiple cards are put into the library this way (such as when the spell targets a melded permanent), that creature’s owner puts all the cards on top or all the cards on the bottom."
    );
    for name in ["Uneasy Partings", "Misleading Motes", "Dire Downdraft"] {
        for bottom in [false, true] {
            let mut t = TestGame::new(2);
            let r = ragnarok(&mut t);
            t.clear_answers();
            t.answer(P0, DecisionKind::Option, Answer::Index(bottom as usize));
            let from = t.asked().len();
            cast_card(&mut t, P1, name, &[obj(r)]).unwrap();
            t.resolve_all();
            assert_eq!(top_or_bottom_asks(&t, from), vec![P0], "{name}: one choice");
            let lib = t.g.player(P0).library.clone();
            let ends: Vec<String> = if bottom {
                lib[..2].iter()
            } else {
                lib[lib.len() - 2..].iter()
            }
            .map(|id| t.g.obj(*id).chars.name.to_string())
            .collect();
            assert!(
                ends.iter().any(|n| n.starts_with("Fang"))
                    && ends.iter().any(|n| n.starts_with("Vanille")),
                "{name} bottom: {bottom}: {ends:?}"
            );
        }
    }
}

#[test]
fn aetherspouts_active_player_chooses_first_then_turn_order() {
    cr!("101.4", "608.2c");
    ruling!(
        "Aetherspouts",
        "The owner of each attacking creature chooses whether to put it on the top or bottom of their library. The active player (the player whose turn it is) makes all of their choices first, followed by each other player in turn order."
    );
    // P2 attacks P0 with its own Hill Giant and P1's Grizzly Bears (stolen); both owners
    // choose, the active player P2 first (not in seat order).
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P2, "Hill Giant");
    give_control(&mut t, bears, P2);
    t.g.objects[bears.0 as usize].summoning_sick = false;
    t.set_step(P2, Step::BeginningOfCombat);
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P0)), (giant, Entity::Player(P0))],
    );
    t.resolve_all();
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.answer(P2, DecisionKind::Option, Answer::Index(0));
    let from = t.asked().len();
    cast_card(&mut t, P0, "Aetherspouts", &[]).unwrap();
    t.resolve_all();
    assert_eq!(top_or_bottom_asks(&t, from), vec![P2, P1]);
    assert_eq!(bottom_name(&t, P1, 0), "Grizzly Bears");
    let lib2 = &t.g.player(P2).library;
    assert_eq!(t.g.obj(*lib2.last().unwrap()).chars.name, "Hill Giant");
}
