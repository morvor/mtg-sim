//! Rulings batch P108 — modal spells and abilities (CR 700.2): modes are chosen as the
//! spell is cast or the ability is put on the stack (CR 601.2b, 700.2a); each mode's
//! targets are chosen separately and may be the same object (CR 700.2d); modes are
//! followed in printed order (CR 700.2), or in the order chosen for a mode chosen more
//! than once (CR 700.2h); a spell whose targets are all illegal doesn't resolve
//! (CR 608.2b); copies keep the modes (CR 707.10).

use crate::r_p108_common::*;
use crate::r_s25_common::{abilities_from, change_copy_targets, spell_copies, targets_of};
use mtg_engine::testing::*;
use mtg_engine::*;

/// The modes chosen for the spell or ability `id` on the stack.
fn modes_of(t: &TestGame, id: ObjectId) -> Vec<Option<usize>> {
    t.g.obj(id)
        .stack
        .as_deref()
        .map(|si| si.chosen.iter().map(|m| m.mode).collect())
        .unwrap_or_default()
}

/// Puts the real card `name` into `p`'s hand with lands for its cost.
fn in_hand(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    lands_for_cost(t, p, name);
    t.hand(p, name)
}

// --- Summon: Ixion ---------------------------------------------------------------------

/// P0's Summon: Ixion with one lore counter and a Grizzly Bears; a second lore counter
/// triggers chapter II, with the targets `targets` (resolved unless `respond` acts first).
fn ixion_chapter_two(targets: &[usize], respond: impl FnOnce(&mut TestGame, ObjectId)) -> TestGame {
    supported("Summon: Ixion");
    let mut t = TestGame::new(2);
    let ixion = t.battlefield(P0, "Summon: Ixion");
    t.g.objects[ixion.0 as usize]
        .counters
        .insert("lore".into(), 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let chosen: Vec<Entity> = targets
        .iter()
        .map(|i| if *i == 0 { obj(bears) } else { obj(ixion) })
        .collect();
    t.answer_targets(P0, &chosen);
    put_counters(&mut t, ixion, "lore", 1);
    assert_eq!(abilities_from(&t, ixion).len(), 1);
    respond(&mut t, bears);
    t.resolve_all();
    t
}

#[test]
fn ixion_chapter_two_with_no_targets_still_gains_life() {
    cr!("714.2b", "115.1c", "601.2c");
    ruling!(
        "Summon: Ixion",
        "You don't have to choose any targets for Summon: Ixion's second or third chapter ability."
    );
    let t = ixion_chapter_two(&[], |_, _| {});
    assert_eq!(t.life(P0), 22);
}

#[test]
fn ixion_chapter_two_with_its_only_target_gone_doesnt_resolve() {
    cr!("608.2b", "714.2b");
    ruling!(
        "Summon: Ixion",
        "if you do and all of the targets are illegal when the ability tries to resolve, it won't resolve and none of its effects will happen. You won't gain life."
    );
    let t = ixion_chapter_two(&[0], |t, bears| destroy(t, bears));
    assert_eq!(t.life(P0), 20);
    // With its target still there, the counter and the life.
    let t = ixion_chapter_two(&[0], |_, _| {});
    assert_eq!(t.life(P0), 22);
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.counters(bears, "+1/+1"), 1);
}

// --- Same target for both modes ----------------------------------------------------------

#[test]
fn artful_takedowns_modes_may_target_the_same_creature() {
    cr!("700.2d", "115.3");
    ruling!(
        "Artful Takedown",
        "two modes may each target the same creature, or they may target two different creatures."
    );
    supported("Artful Takedown");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Colossal Dreadmaw");
    let card = in_hand(&mut t, P0, "Artful Takedown");
    let spell = t
        .cast(P0, card)
        .modes(&[0, 1])
        .target(giant)
        .target(giant)
        .go();
    assert_eq!(targets_of(&t, spell), vec![obj(giant), obj(giant)]);
    t.resolve_all();
    assert!(t.obj_now(giant).tapped);
    assert_eq!(t.pt(giant), (4, 2));
    // Two different creatures.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let other = t.battlefield(P1, "Grizzly Bears");
    let card = in_hand(&mut t, P0, "Artful Takedown");
    t.cast(P0, card)
        .modes(&[0, 1])
        .target(giant)
        .target(other)
        .go();
    t.resolve_all();
    assert!(t.obj_now(giant).tapped && t.on_battlefield(giant));
    assert!(!t.on_battlefield(other));
}

#[test]
fn winterflames_modes_may_target_the_same_creature() {
    cr!("700.2d", "115.3");
    ruling!(
        "Winterflame",
        "If you choose both modes, they can each target the same creature or they can target different creatures."
    );
    supported("Winterflame");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let card = in_hand(&mut t, P0, "Winterflame");
    t.cast(P0, card)
        .modes(&[0, 1])
        .target(giant)
        .target(giant)
        .go();
    t.resolve_all();
    assert!(t.obj_now(giant).tapped);
    assert_eq!(t.obj_now(giant).damage, 2);
}

#[test]
fn winterflame_with_both_targets_illegal_doesnt_resolve() {
    cr!("608.2b", "700.2d");
    ruling!(
        "Winterflame",
        "Winterflame won’t affect any target that’s illegal as it tries to resolve. If you choose to use both modes and both targets are illegal at that time, Winterflame won’t resolve."
    );
    // One target illegal: the other is still affected.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = in_hand(&mut t, P0, "Winterflame");
    t.cast(P0, card)
        .modes(&[0, 1])
        .target(giant)
        .target(bears)
        .go();
    destroy(&mut t, giant);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    // Both illegal: it doesn't resolve (it's put into the graveyard without effect).
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = in_hand(&mut t, P0, "Winterflame");
    let spell = t
        .cast(P0, card)
        .modes(&[0, 1])
        .target(giant)
        .target(bears)
        .go();
    destroy(&mut t, giant);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Winterflame"));
    assert!(!resolved(&t, spell));
}

#[test]
fn subtle_strike_affects_the_other_target_if_one_is_illegal() {
    cr!("608.2b", "700.2d");
    ruling!(
        "Subtle Strike",
        "If you choose both modes and one target becomes illegal before Subtle Strike resolves, the other target is affected as appropriate."
    );
    supported("Subtle Strike");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Hill Giant");
    let card = in_hand(&mut t, P0, "Subtle Strike");
    t.cast(P0, card)
        .modes(&[0, 1])
        .target(theirs)
        .target(mine)
        .go();
    destroy(&mut t, theirs);
    t.resolve_all();
    assert_eq!(t.counters(mine, "+1/+1"), 1);
    // The other way around.
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Hill Giant");
    let card = in_hand(&mut t, P0, "Subtle Strike");
    t.cast(P0, card)
        .modes(&[0, 1])
        .target(theirs)
        .target(mine)
        .go();
    destroy(&mut t, mine);
    t.resolve_all();
    assert_eq!(t.pt(theirs), (1, 1));
}

#[test]
fn flash_thompson_taps_then_untaps_the_same_target() {
    cr!("700.2", "608.2c");
    ruling!(
        "Flash Thompson, Spider-Fan",
        "If you choose both modes, the first mode will happen, then the second mode will happen. In particular, this means that if you choose the same target for both modes, it will be tapped before it is untapped."
    );
    supported("Flash Thompson, Spider-Fan");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer(P0, DecisionKind::Modes, mtg_engine::decision::Answer::Indices(vec![0, 1]));
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_targets(P0, &[obj(giant)]);
    t.enter(P0, "Flash Thompson, Spider-Fan");
    t.resolve_all();
    assert!(!t.obj_now(giant).tapped, "tapped, then untapped");
}

#[test]
fn pollenbright_druid_may_target_itself() {
    cr!("115.1", "603.3d");
    ruling!(
        "Pollenbright Druid",
        "You may choose Pollenbright Druid as the target of its own ability."
    );
    supported("Pollenbright Druid");
    let mut t = TestGame::new(2);
    let d = t.enter(P0, "Pollenbright Druid");
    t.answer(P0, DecisionKind::Modes, mtg_engine::decision::Answer::Indices(vec![0]));
    t.answer_targets(P0, &[obj(d)]);
    t.resolve_all();
    assert_eq!(t.counters(d, "+1/+1"), 1);
    assert_eq!(t.pt(d), (2, 2));
}

// --- Choosing modes ----------------------------------------------------------------------

#[test]
fn fortify_affects_only_creatures_you_control_as_it_resolves() {
    cr!("611.2c", "700.2a");
    ruling!(
        "Fortify",
        "Creatures that come under your control after Fortify resolves won't get the chosen bonus."
    );
    ruling!(
        "Fortify",
        "You choose the mode as you cast Fortify, not as it resolves."
    );
    supported("Fortify");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let card = in_hand(&mut t, P0, "Fortify");
    let spell = t.cast(P0, card).modes(&[1]).go();
    // The mode is fixed on the stack before it resolves.
    assert_eq!(modes_of(&t, spell), vec![Some(1)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 4));
    let later = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(later), (3, 3));
}

#[test]
fn moment_of_reckoning_with_every_target_illegal_doesnt_resolve() {
    cr!("608.2b", "700.2a");
    ruling!(
        "Moment of Reckoning",
        "If all targets for the chosen modes become illegal before Moment of Reckoning resolves, the spell won't resolve and none of its effects will happen. If at least one target is still legal, the spell will resolve but will have no effect on any illegal targets."
    );
    ruling!(
        "Moment of Reckoning",
        "You choose the modes as you cast Moment of Reckoning. Once modes are chosen, they can't be changed."
    );
    supported("Moment of Reckoning");
    // At least one target legal: it resolves for that one.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Grizzly Bears");
    let card = in_hand(&mut t, P0, "Moment of Reckoning");
    let spell = t.cast(P0, card).modes(&[0, 0]).target(a).target(b).go();
    assert_eq!(modes_of(&t, spell), vec![Some(0), Some(0)]);
    destroy(&mut t, a);
    let giant_card = t.g.current(a);
    t.resolve_all();
    assert!(resolved(&t, spell));
    assert!(!t.on_battlefield(b));
    assert_eq!(t.g.obj(giant_card).zone, mtg_engine::object::Zone::Graveyard(P1));
    // All targets illegal: it doesn't resolve.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Hill Giant");
    let gy = t.graveyard(P0, "Grizzly Bears");
    let card = in_hand(&mut t, P0, "Moment of Reckoning");
    let spell = t.cast(P0, card).modes(&[0, 1]).target(a).target(gy).go();
    destroy(&mut t, a);
    let gy_now = t.g.current(gy);
    t.g.move_object(
        gy_now,
        mtg_engine::object::Zone::Exile,
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.resolve_all();
    assert!(!resolved(&t, spell));
}

#[test]
fn moment_of_reckoning_copy_may_change_targets_but_not_modes() {
    cr!("707.10", "707.10c", "700.2a");
    ruling!(
        "Moment of Reckoning",
        "If Moment of Reckoning is copied, the effect that creates the copy will usually allow you to choose new targets, but you can't choose new modes."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Grizzly Bears");
    let card = in_hand(&mut t, P0, "Moment of Reckoning");
    let spell = t.cast(P0, card).modes(&[0]).target(a).go();
    let from = t.asked().len();
    change_copy_targets(&mut t, P0, &[Some(obj(b))]);
    mtg_engine::copy::copy_spell(&mut t.g, spell, P0, true).expect("copied");
    let copy = spell_copies(&t)[0];
    assert_eq!(modes_of(&t, copy), vec![Some(0)]);
    assert_eq!(targets_of(&t, copy), vec![obj(b)]);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::ChooseModes { .. })));
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
}

#[test]
fn moment_of_reckoning_follows_the_chosen_order_of_a_repeated_mode() {
    cr!("700.2h", "613.7d");
    ruling!(
        "Moment of Reckoning",
        "If the same mode is chosen more than once, you choose their relative order as you cast the spell."
    );
    // The card returned by the instance of the mode ordered first enters first (it gets
    // the earlier timestamp).
    for first_bears in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.graveyard(P0, "Grizzly Bears");
        let giant = t.graveyard(P0, "Hill Giant");
        let card = in_hand(&mut t, P0, "Moment of Reckoning");
        let (x, y) = if first_bears { (bears, giant) } else { (giant, bears) };
        t.cast(P0, card).modes(&[1, 1]).target(x).target(y).go();
        t.resolve_all();
        let (bx, gx) = (t.obj_now(bears).timestamp, t.obj_now(giant).timestamp);
        assert!(t.on_battlefield(bears) && t.on_battlefield(giant));
        assert_eq!(bx < gx, first_bears);
    }
}

#[test]
fn nothing_happens_between_a_confluences_modes() {
    cr!("608.2c", "603.3", "117.3");
    ruling!(
        "Eldrazi Confluence",
        "No player can cast spells or activate abilities in between the modes of a resolving spell. Any abilities that trigger won't be put onto the stack until a spell is done resolving."
    );
    supported("Eldrazi Confluence");
    supported("Soul Warden");
    let mut t = TestGame::new(2);
    let warden = t.battlefield(P0, "Soul Warden");
    let card = in_hand(&mut t, P0, "Eldrazi Confluence");
    t.cast(P0, card).modes(&[2, 2, 2]).go();
    t.resolve();
    // All three Scions entered during the resolution; the three triggers wait for it.
    assert_eq!(tokens_with(&t, P0, "Scion"), 3);
    assert_eq!(abilities_from(&t, warden).len(), 3);
    assert_eq!(t.life(P0), 20);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}
