//! Rulings batch S31 — cards outside the game and the command zone: a wish can't get an
//! exiled card, which is still in one of the game's zones (CR 400.11); a commander that
//! died may go to the command zone (CR 903.9a), and then its own "you may exile it" can't
//! find it (CR 400.7).

use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s04_common::add_mana;
use crate::r_s13_common::{commander, commander_game};
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn living_wish_cant_get_an_exiled_card() {
    cr!("400.11", "400.11a");
    ruling!(
        "Living Wish",
        "You can't acquire exiled cards because those cards are still in one of the game's zones."
    );
    supported("Living Wish");
    // "You may reveal a creature or land card you own from outside the game and put it
    // into your hand. Exile Living Wish." P0's Grizzly Bears is in exile; Hill Giant is in
    // P0's sideboard.
    let mut t = TestGame::new(2);
    let exiled = t.exile(P0, "Grizzly Bears");
    let outside = t.custom(P0, (*card("Hill Giant")).clone(), Zone::Outside(P0));
    add_mana(&mut t, P0, ManaType::G, 2);
    let wish = t.hand(P0, "Living Wish");
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Object(exiled)]);
    t.cast(P0, wish).go();
    t.resolve_all();
    // The exiled Bears was never offered, and stays in exile.
    let offered_exiled = t.asked()[from..].iter().any(|(_, d)| match d {
        Decision::ChooseEntities { candidates, .. } => candidates.contains(&Entity::Object(exiled)),
        _ => false,
    });
    assert!(!offered_exiled);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert!(!t.in_hand(P0, "Grizzly Bears"));
    assert_ne!(t.zone(outside), Zone::Exile);
    assert!(t.in_exile("Living Wish"));
}

/// Iname, Life Aspect (P0's commander) on the battlefield and Kami of Ancient Law (a
/// Spirit) in P0's graveyard. Iname: "When Iname dies, you may exile it. If you do,
/// return any number of target Spirit cards from your graveyard to your hand."
fn iname_game() -> (TestGame, ObjectId, ObjectId) {
    supported("Iname, Life Aspect");
    let mut t = commander_game();
    let iname = commander(&mut t, P0, "Iname, Life Aspect");
    let iname =
        t.g.move_object(
            iname,
            Zone::Battlefield,
            mtg_engine::events::MoveCause::Effect,
            Some(P0),
        )
        .expect("Iname enters");
    t.settle();
    let kami = t.graveyard(P0, "Kami of Ancient Law");
    (t, iname, kami)
}

#[test]
fn iname_put_into_the_command_zone_cant_be_exiled() {
    cr!("903.9a", "400.7", "603.10a");
    ruling!(
        "Iname, Life Aspect",
        "If Iname is your commander in a game of Commander, you may choose to put it in the command zone after it dies, but if you do, you won't be able to exile it."
    );
    let (mut t, iname, kami) = iname_game();
    // P0 puts Iname into the command zone; its ability then can't exile it, so the Kami
    // stays in the graveyard.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(kami)]);
    t.answer_yes(P0, true);
    destroy(&mut t, iname);
    t.resolve_all();
    assert_eq!(t.zone(iname), Zone::Command);
    assert_eq!(t.zone(kami), Zone::Graveyard(P0));
}

#[test]
fn iname_exiled_by_its_ability_may_then_go_to_the_command_zone() {
    cr!("903.9a", "603.10a");
    ruling!(
        "Iname, Life Aspect",
        "If you do exile it, you may put it into the command zone from exile after resolving its ability."
    );
    let (mut t, iname, kami) = iname_game();
    // P0 leaves Iname in the graveyard, exiles it with its ability (returning the Kami),
    // then puts it into the command zone from exile.
    t.answer_yes(P0, false);
    t.answer_targets(P0, &[Entity::Object(kami)]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    destroy(&mut t, iname);
    t.resolve_all();
    assert!(t.in_hand(P0, "Kami of Ancient Law"));
    assert_eq!(t.zone(iname), Zone::Command);
}
