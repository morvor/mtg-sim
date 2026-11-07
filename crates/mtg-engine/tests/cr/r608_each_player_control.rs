//! CR 101.4, 608.2f: "each player gains control of ..." — every player's permanents are
//! determined (and chosen) before any control changes, then all the control changes
//! happen at the same time.

use crate::r600_common::CB;
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn sorcery(name: &str, body: Effect) -> mtg_engine::card::CardDef {
    CB::new(name)
        .sorcery()
        .cost("{0}")
        .spell(Body::effect(body))
        .build()
}

/// "Each player gains control of all creatures [players] control", for `whose` relative
/// to the iterated player.
fn each_player_takes(whose: PlayerRef) -> Effect {
    Effect::ForEachPlayer {
        who: PlayerRef::EachPlayer,
        effect: Box::new(Effect::GainControl {
            what: Sel::All(Filter::and(vec![
                Filter::creature(),
                Filter::InZone(ZoneKind::Battlefield),
                Filter::ControlledByPlayer(Box::new(whose)),
            ])),
            who: PlayerRef::Iterated,
            duration: Duration::Permanent,
        }),
    }
}

#[test]
fn each_player_gaining_their_opponents_creatures_swaps_them() {
    cr!("608.2f", "101.4");
    // Determined one player at a time, P1 would take back what P0 had just taken.
    let body = each_player_takes(PlayerRef::Each(PlayerFilter::OpponentOf(Box::new(
        PlayerRef::Iterated,
    ))));
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let spell = t.custom(P0, sorcery("Great Swap", body), Zone::Hand(P0));
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.obj_now(a).controller, P1);
    assert_eq!(t.obj_now(b).controller, P0);
}

#[test]
fn each_player_gaining_the_creatures_of_the_player_to_their_left_rotates_them() {
    cr!("608.2f", "101.4");
    let body = each_player_takes(PlayerRef::Seat {
        of: Box::new(PlayerRef::Iterated),
        side: SeatSide::Left,
        opponent: false,
    });
    let mut t = TestGame::new(3);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let c = t.battlefield(P2, "Craw Wurm");
    let spell = t.custom(P0, sorcery("Rotation", body), Zone::Hand(P0));
    t.cast(P0, spell).go();
    t.resolve_all();
    // P1 sits to P0's left, P2 to P1's, P0 to P2's.
    assert_eq!(t.obj_now(b).controller, P0);
    assert_eq!(t.obj_now(c).controller, P1);
    assert_eq!(t.obj_now(a).controller, P2);
}

#[test]
fn the_tokens_each_player_created_are_all_named_afterwards() {
    cr!("608.2c", "608.2f");
    // "Each player creates a Treasure token. Tap those tokens.": every player's token.
    let body = Effect::Seq(vec![
        Effect::ForEachPlayer {
            who: PlayerRef::EachPlayer,
            effect: Box::new(Effect::AsPlayer {
                who: PlayerRef::Iterated,
                effect: Box::new(Effect::CreateToken {
                    spec: mtg_engine::tokens::predefined("Treasure").expect("Treasure"),
                    count: Value::c(1),
                    controller: PlayerRef::You,
                    tapped: false,
                    attacking: false,
                }),
            }),
        },
        Effect::Tap {
            what: Sel::Var(vars::CREATED),
        },
    ]);
    let mut t = TestGame::new(3);
    let spell = t.custom(P0, sorcery("Treasure for All", body), Zone::Hand(P0));
    t.cast(P0, spell).go();
    t.resolve_all();
    let tokens = t.named_on_battlefield("Treasure Token");
    assert_eq!(tokens.len(), 3);
    assert!(tokens.iter().all(|o| t.obj_now(*o).tapped));
}
