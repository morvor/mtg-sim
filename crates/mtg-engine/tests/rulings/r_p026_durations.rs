//! Rulings batch P026 — how long copy effects last: a copy effect with no duration lasts
//! until the permanent leaves the battlefield, though a later copy effect overrides it
//! while that one applies (CR 611.2a, 613.7); "until your next turn" ends as that turn
//! begins, even if its untap step is skipped (CR 614.10); "gains haste until end of turn"
//! ends in the cleanup step (CR 514.2).

use crate::r_p026_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts `name` into exile as if `p` had exiled it from their draft pool before the game
/// with `with` (see `r_p025_exiled_cards`).
fn exiled_before_the_game(t: &mut TestGame, p: PlayerId, with: &str, name: &str) -> ObjectId {
    let side = t.g.add_to_sideboard(p, vec![mtg_engine::card::card(name)]);
    let e =
        t.g.move_object(
            side[0],
            Zone::Exile,
            mtg_engine::events::MoveCause::Effect,
            Some(p),
        )
        .expect("exiled");
    t.g.named_exiles.push((p, with.into(), e));
    e
}

/// Advances to P0's next turn's precombat main phase (through P1's turn).
fn next_turn(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
}

#[test]
fn volatile_chimera_copy_effect_has_no_duration() {
    cr!("611.2a", "613.7", "613.1a");
    ruling!(
        "Volatile Chimera",
        "The copy effect created by the activated ability doesn't have a duration. It will last until Volatile Chimera leaves the battlefield or another copy effect overwrites it."
    );
    supported("Volatile Chimera");
    let mut t = TestGame::new(2);
    exiled_before_the_game(&mut t, P0, "Volatile Chimera", "Hill Giant");
    let chimera = t.battlefield(P0, "Volatile Chimera");
    lands_for(&mut t, P0, "{1}{R}");
    t.activate(P0, chimera, 0, &[]).expect("activation");
    t.resolve_all();
    assert_eq!(name_of(&t, chimera), "Hill Giant");
    next_turn(&mut t);
    assert_eq!(name_of(&t, chimera), "Hill Giant");
    assert_eq!(t.pt(chimera), (3, 3));
    // A later copy effect overrides it while it lasts.
    let angel = t.battlefield(P1, "Serra Angel");
    cytoshape(&mut t, angel, chimera);
    assert_eq!(name_of(&t, chimera), "Serra Angel");
    next_turn(&mut t);
    assert_eq!(name_of(&t, chimera), "Hill Giant");
}

#[test]
fn shameless_charlatan_copy_effect_lasts_until_overridden() {
    cr!("611.2a", "613.7", "613.1a", "903.3");
    ruling!(
        "Shameless Charlatan",
        "The copy effect doesn't have a duration. It lasts until the commander leaves the battlefield, though another copy effect with a later timestamp (perhaps from a new activation of the ability granted by Shameless Charlatan) would override the effect for as long as it's active."
    );
    supported("Shameless Charlatan");
    let mut t = crate::r_s13_common::commander_game();
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Shameless Charlatan");
    let cmdr = t.battlefield(P0, "Isamaru, Hound of Konda");
    t.g.objects[cmdr.0 as usize].is_commander = true;
    t.g.players[0]
        .commander_names
        .push("Isamaru, Hound of Konda".into());
    t.g.recompute();
    let giant = t.battlefield(P1, "Hill Giant");
    let angel = t.battlefield(P1, "Serra Angel");
    lands_for(&mut t, P0, "{2}{U}");
    t.answer_targets(P0, &[obj(giant)]);
    activate_containing(&mut t, P0, cmdr, "becomes a copy").expect("activation");
    t.resolve_all();
    assert_eq!(name_of(&t, cmdr), "Hill Giant");
    next_turn(&mut t);
    assert_eq!(name_of(&t, cmdr), "Hill Giant");
    // A new activation overrides it.
    lands_for(&mut t, P0, "{2}{U}");
    t.answer_targets(P0, &[obj(angel)]);
    activate_containing(&mut t, P0, cmdr, "becomes a copy").expect("activation");
    t.resolve_all();
    assert_eq!(name_of(&t, cmdr), "Serra Angel");
    assert_eq!(t.pt(cmdr), (4, 4));
}

#[test]
fn shapesharer_copy_lasts_until_your_next_untap_step_even_if_skipped() {
    cr!("611.2a", "614.10");
    ruling!(
        "Shapesharer",
        "The copy effect doesn’t wear off until just before your next untap step (even if an effect will cause that untap step to be skipped)."
    );
    // "{2}{U}: Target Shapeshifter becomes a copy of target creature until your next
    // turn." Stasis: "Players skip their untap steps."
    supported("Shapesharer");
    supported("Stasis");
    let mut t = TestGame::new(2);
    let sharer = t.battlefield(P0, "Shapesharer");
    let giant = t.battlefield(P1, "Hill Giant");
    t.battlefield(P0, "Stasis");
    // A tapped land shows that P0's untap step is skipped.
    let land = t.battlefield(P0, "Wastes");
    t.g.tap(land);
    lands_for(&mut t, P0, "{2}{U}");
    t.activate(P0, sharer, 0, &[obj(sharer), obj(giant)])
        .expect("activation");
    t.resolve_all();
    assert_eq!(name_of(&t, sharer), "Hill Giant");
    // Still a copy during the opponent's turn.
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(name_of(&t, sharer), "Hill Giant");
    // P0's untap step is skipped, but the effect has ended by P0's upkeep.
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.g.turn.active, P0);
    assert!(t.obj_now(land).tapped);
    assert_eq!(name_of(&t, sharer), "Shapesharer");
}

#[test]
fn cryptoplasm_copy_lasts_until_overwritten() {
    cr!("611.2a", "613.7");
    ruling!(
        "Cryptoplasm",
        "The copy effect lasts indefinitely. Often, it will last until it is overwritten by another copy effect (if it copies another creature on a future turn, perhaps.)"
    );
    supported("Cryptoplasm");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let angel = t.battlefield(P1, "Serra Angel");
    let c = t.battlefield(P0, "Cryptoplasm");
    t.set_step(P1, Step::End);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(giant)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(name_of(&t, c), "Hill Giant");
    // Through the opponent's turn.
    t.advance_to(P1, Step::End);
    assert_eq!(name_of(&t, c), "Hill Giant");
    t.answer_targets(P0, &[obj(angel)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(name_of(&t, c), "Serra Angel");
    assert_eq!(
        abilities_with(&t, c, "become a copy of another target creature"),
        1
    );
}

#[test]
fn cemetery_puca_copy_has_no_duration_and_is_overwritten() {
    cr!("611.2a", "613.7", "707.9a");
    ruling!(
        "Cemetery Puca",
        "This effect has no duration. If you use the ability, Cemetery Puca will remain a copy of the creature until it leaves the battlefield or you use the ability again. If it becomes a copy of a different creature card, the new copy will overwrite the old copy."
    );
    supported("Cemetery Puca");
    let mut t = TestGame::new(2);
    let puca = t.battlefield(P0, "Cemetery Puca");
    let giant = t.battlefield(P1, "Hill Giant");
    let angel = t.battlefield(P1, "Serra Angel");
    lands_for(&mut t, P0, "{1}{1}");
    t.answer_yes(P0, true);
    kill(&mut t, giant);
    t.resolve_all();
    assert_eq!(name_of(&t, puca), "Hill Giant");
    next_turn(&mut t);
    assert_eq!(name_of(&t, puca), "Hill Giant");
    t.answer_yes(P0, true);
    kill(&mut t, angel);
    t.resolve_all();
    assert_eq!(name_of(&t, puca), "Serra Angel");
    assert_eq!(t.pt(puca), (4, 4));
    assert_eq!(
        abilities_with(&t, puca, "becomes a copy of that creature"),
        1
    );
}

#[test]
fn dimir_doppelganger_new_copy_overwrites_the_old() {
    cr!("707.9a", "613.7", "611.2a");
    ruling!(
        "Dimir Doppelganger",
        "This creature becomes an exact copy of a copied card, except that it also has Dimir Doppelganger's activated ability. If it becomes a copy of a different creature card, the new copy will overwrite the old copy."
    );
    supported("Dimir Doppelganger");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Dimir Doppelganger");
    let giant = t.graveyard(P1, "Hill Giant");
    let angel = t.graveyard(P1, "Serra Angel");
    lands_for(&mut t, P0, "{1}{U}{B}");
    t.activate(P0, d, 0, &[obj(giant)]).expect("activation");
    t.resolve_all();
    assert_eq!(name_of(&t, d), "Hill Giant");
    assert_eq!(t.pt(d), (3, 3));
    assert_eq!(abilities_with(&t, d, "becomes a copy of that card"), 1);
    lands_for(&mut t, P0, "{1}{U}{B}");
    t.activate(P0, d, 0, &[obj(angel)]).expect("activation");
    t.resolve_all();
    assert_eq!(name_of(&t, d), "Serra Angel");
    assert_eq!(t.pt(d), (4, 4));
    assert!(has_kw(&t, d, KeywordKind::Flying));
    assert_eq!(abilities_with(&t, d, "becomes a copy of that card"), 1);
}

#[test]
fn corpse_dance_and_echo_chamber_grant_haste_only_until_end_of_turn() {
    cr!("514.2", "611.2a");
    ruling!(
        "Corpse Dance",
        "The effect only grants Haste until the end of the current turn. If the creature somehow manages to stick around until a later turn, it will no longer have Haste."
    );
    ruling!(
        "Echo Chamber",
        "The effect only grants Haste until the end of the current turn. If the creature somehow manages to stick around until a later turn, it will no longer have Haste."
    );
    // Corpse Dance cast in P0's end step: "the next end step" is P1's, so the creature is
    // still around in P1's upkeep, without haste.
    supported("Corpse Dance");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Hill Giant");
    lands_for(&mut t, P0, "{2}{B}");
    let dance = t.hand(P0, "Corpse Dance");
    t.set_step(P0, Step::End);
    t.cast(P0, dance).go();
    t.resolve_all();
    let giant = t.named_on_battlefield("Hill Giant")[0];
    assert!(has_kw(&t, giant, KeywordKind::Haste));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(giant));
    assert!(!has_kw(&t, giant, KeywordKind::Haste));
    // Echo Chamber's token survives its delayed trigger, countered by Stifle.
    supported("Echo Chamber");
    supported("Stifle");
    let mut t = TestGame::new(2);
    let chamber = t.battlefield(P0, "Echo Chamber");
    let giant = t.battlefield(P1, "Hill Giant");
    lands_for(&mut t, P0, "{4}");
    t.answer_targets(P1, &[obj(giant)]);
    let before = t.g.battlefield.clone();
    t.activate(P0, chamber, 0, &[]).expect("activation");
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    assert!(has_kw(&t, tok, KeywordKind::Haste));
    t.set_step(P0, Step::PostcombatMain);
    lands_for(&mut t, P0, "{U}");
    let stifle = t.hand(P0, "Stifle");
    t.g.run_until(1000, |g| g.turn.step == Step::End && !g.stack.is_empty());
    let trigger = *t.g.stack.last().unwrap();
    t.cast(P0, stifle).target(obj(trigger)).go();
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(tok));
    assert!(!has_kw(&t, tok, KeywordKind::Haste));
}

#[test]
fn unstable_shapeshifter_copy_effect_isnt_targeted() {
    cr!("115.1", "702.11b", "707.9a");
    ruling!("Unstable Shapeshifter", "The copy effect is not targeted.");
    // Gladecover Scout (hexproof) enters under the opponent's control: P0's Shapeshifter
    // becomes a copy of it anyway.
    supported("Unstable Shapeshifter");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Unstable Shapeshifter");
    let scout = t.enter(P1, "Gladecover Scout");
    t.settle();
    t.resolve_all();
    assert!(has_kw(&t, scout, KeywordKind::Hexproof));
    assert_eq!(name_of(&t, s), "Gladecover Scout");
    assert_eq!(abilities_with(&t, s, "becomes a copy of that creature"), 1);
}
