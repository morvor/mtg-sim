//! Rulings batch P028 — the God-Eternals' "When [this] dies or is put into exile from the
//! battlefield, you may put it into its owner's library third from the top.": the card is
//! followed only within its new zone (CR 400.7, 608.2h); a commander may go to the command
//! zone first (CR 903.9a); the trigger's controller decides (CR 603.3a).

use crate::r_p028_common::obj;
use crate::r_s01_common::supported;
use crate::r_s06_common::give_control;
use crate::r_s25_common::cast_new;
use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The position of the card `id` (followed across zone changes) in `p`'s library, from the
/// top (0 is the top card).
fn from_top(t: &TestGame, p: PlayerId, id: ObjectId) -> Option<usize> {
    let now = t.g.current(id);
    t.g.player(p).library.iter().rev().position(|c| *c == now)
}

fn kill(t: &mut TestGame, id: ObjectId) {
    t.g.destroy(id, None);
    t.g.flush_events();
    t.settle();
}

#[test]
fn a_god_flickered_returns_at_once_and_the_trigger_does_nothing() {
    cr!("400.7", "603.6c", "608.2h");
    ruling!(
        "God-Eternal Oketra",
        "If an effect exiles the God and immediately returns it to the battlefield, its last ability triggers but will have no effect. However, if an effect exiles it and would return it to the battlefield at a later time, the God's ability may return that card to its owner's library first. If it does, the effect that exiled it won't return it later."
    );
    supported("God-Eternal Oketra");
    supported("Cloudshift");
    // Cloudshift ("Exile target creature you control, then return that card to the
    // battlefield under your control."): the God is a new object on the battlefield.
    let mut t = TestGame::new(2);
    let god = t.battlefield(P0, "God-Eternal Oketra");
    cast_new(&mut t, P0, "Cloudshift", &[obj(god)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("God-Eternal Oketra").len(), 1);
    assert_eq!(from_top(&t, P0, god), None);
    // Fiend Hunter exiles it until it leaves: the God goes to the library third from the
    // top, and doesn't come back when the Hunter leaves.
    let mut t = TestGame::new(2);
    let god = t.battlefield(P1, "God-Eternal Oketra");
    let hunter = cast_new(&mut t, P0, "Fiend Hunter", &[]);
    t.answer_targets(P0, &[obj(god)]);
    t.answer_yes(P0, true);
    t.answer_yes(P1, true);
    t.resolve_all();
    assert_eq!(from_top(&t, P1, god), Some(2));
    let hunter = t.g.current(hunter);
    kill(&mut t, hunter);
    t.resolve_all();
    assert!(t.named_on_battlefield("God-Eternal Oketra").is_empty());
    assert_eq!(from_top(&t, P1, god), Some(2));
}

#[test]
fn a_god_that_left_its_new_zone_stays_where_it_went() {
    cr!("400.7", "608.2h");
    ruling!(
        "God-Eternal Oketra",
        "If one of these Gods leaves the graveyard or exile while its last ability is on the stack, it will remain in its new zone, even if that zone is a graveyard or exile."
    );
    let mut t = TestGame::new(2);
    let god = t.battlefield(P0, "God-Eternal Oketra");
    kill(&mut t, god);
    assert_eq!(t.stack_len(), 1);
    // In response, the card is exiled from the graveyard.
    let card = t.g.current(god);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    t.g.exile_object(card, None);
    t.g.flush_events();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_exile("God-Eternal Oketra"));
    assert_eq!(from_top(&t, P0, god), None);
}

#[test]
fn a_commander_god_may_go_to_the_command_zone_first() {
    cr!("903.9a", "704.6d", "608.2h");
    ruling!(
        "God-Eternal Oketra",
        "If one of these Gods dies and it's your commander, you may put it into the command zone before its ability puts it into your library. If you save your commander this way, it won't put it into your library. The same is true if it would be exiled."
    );
    let mut t = TestGame::with_config(2, GameConfig::commander_game());
    let god = t.battlefield(P0, "God-Eternal Oketra");
    t.g.objects[god.0 as usize].is_commander = true;
    t.g.players[0].commander_names.push("God-Eternal Oketra".into());
    // Yes to the command zone (a state-based action), then yes to the trigger.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    kill(&mut t, god);
    t.resolve_all();
    assert_eq!(t.g.find_in_zone(Zone::Command, "God-Eternal Oketra").len(), 1);
    assert_eq!(from_top(&t, P0, god), None);
}

#[test]
fn the_controller_of_another_player_s_god_decides() {
    cr!("603.3a", "603.10a");
    ruling!(
        "God-Eternal Oketra",
        "If you control another player's God when it dies, you decide whether to put that card into its owner's library."
    );
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let god = t.battlefield(P1, "God-Eternal Oketra");
        give_control(&mut t, god, P0);
        assert_eq!(t.obj_now(god).controller, P0);
        t.answer_yes(P0, yes);
        // P1 would put it back; P0 decides.
        t.answer_yes(P1, !yes);
        kill(&mut t, god);
        let from = t.asked().len();
        t.resolve_all();
        assert!(t.asked()[from..]
            .iter()
            .all(|(p, d)| *p == P0 || !matches!(d, mtg_engine::decision::Decision::YesNo { .. })));
        if yes {
            assert_eq!(from_top(&t, P1, god), Some(2));
        } else {
            assert!(t.in_graveyard(P1, "God-Eternal Oketra"));
        }
    }
}
