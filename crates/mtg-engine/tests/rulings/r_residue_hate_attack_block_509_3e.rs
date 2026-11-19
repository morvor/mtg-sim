//! Rulings residue `rulings-residue-hate-attack-block`: "Whenever ~ blocks or becomes
//! blocked by one or more [quality] creatures" (Serra Inquisitors, Amphibious Kavu,
//! Dwarven Soldier) and "Whenever ~ blocks one or more black creatures / two or more
//! creatures" (Rashka the Slayer, Lairwatch Giant) trigger as blockers are declared and
//! also when an effect adds the blocker that makes the count reach the number (CR 509.3e):
//! here Sorrow's Path exchanging two blockers, and creatures put onto the battlefield
//! blocking. The count is taken as of each block, so an effect adding several blocks at
//! once triggers such an ability once (register `block-count-triggers-at-the-nth-block`).

use crate::r_p079_common::cast_card;
use crate::r_residue_hate_attack_block_common::*;
use crate::r_s01_common::{supported, tokens};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 activates Sorrow's Path targeting P1's blockers `x` and `y` and everything resolves
/// (its "becomes tapped" trigger first, CR 603.3).
fn swap(t: &mut TestGame, path: ObjectId, x: ObjectId, y: ObjectId) {
    t.answer_targets(P0, &[obj(x), obj(y)]);
    t.activate(P0, path, 0, &[]).expect("Sorrow's Path");
    t.resolve_all();
}

/// As [`swap`], but stops once the exchange has resolved, with the abilities it triggered
/// on the stack.
fn swap_to_triggers(t: &mut TestGame, path: ObjectId, x: ObjectId, y: ObjectId) {
    t.answer_targets(P0, &[obj(x), obj(y)]);
    t.activate(P0, path, 0, &[]).expect("Sorrow's Path");
    t.settle();
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    // The damage trigger, then the exchange.
    t.resolve();
    t.settle();
    t.resolve();
    t.settle();
}

#[test]
fn becomes_blocked_by_one_or_more_black_creatures_when_an_exchange_adds_one() {
    cr!("509.3e");
    interp!("block-count-triggers-at-the-nth-block");
    for name in ["Serra Inquisitors", "Amphibious Kavu", "Dwarven Soldier"] {
        supported(name);
    }
    // Serra Inquisitors is blocked by a green creature; Sorrow's Path then has the black
    // creature blocking P0's Hill Giant block it instead: it's now blocked by one black
    // creature, so it triggers, though it was a blocked creature all along (unlike
    // "becomes blocked", CR 509.3c).
    let mut t = TestGame::new(2);
    let path = t.battlefield(P0, "Sorrow's Path");
    let si = t.battlefield(P0, "Serra Inquisitors");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wraith = t.battlefield(P1, "Bog Wraith");
    blocks(&mut t, &[si, giant], &[(bears, si), (wraith, giant)]);
    assert_eq!(t.stack_len(), 0, "{}", t.dump_log());
    swap(&mut t, path, bears, wraith);
    assert_eq!(blocking(&t, wraith), vec![si]);
    assert_eq!(t.pt(si), (5, 3), "{}", t.dump_log());
    // Another black creature added to the block doesn't trigger it again ("one or
    // more": the count was already reached).
    let skeletons = t.battlefield(P1, "Drudge Skeletons");
    assert!(mtg_engine::combat::block_by_effect(&mut t.g, skeletons, si));
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0, "{}", t.dump_log());
}

#[test]
fn blocks_one_or_more_or_two_or_more_when_an_exchange_adds_them() {
    cr!("509.3e");
    interp!("block-count-triggers-at-the-nth-block");
    for name in ["Rashka the Slayer", "Lairwatch Giant", "Amphibious Kavu"] {
        supported(name);
    }
    // Rashka blocks a green creature; the exchange has it block P0's black Bog Wraith.
    let mut t = TestGame::new(2);
    let path = t.battlefield(P0, "Sorrow's Path");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let wraith = t.battlefield(P0, "Bog Wraith");
    let rashka = t.battlefield(P1, "Rashka the Slayer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    blocks(&mut t, &[wurm, wraith], &[(rashka, wurm), (bears, wraith)]);
    assert_eq!(t.stack_len(), 0);
    swap(&mut t, path, rashka, bears);
    assert_eq!(blocking(&t, rashka), vec![wraith]);
    assert_eq!(t.pt(rashka), (4, 5), "{}", t.dump_log());
    // The same for Amphibious Kavu's "blocks ... one or more blue and/or black
    // creatures".
    let mut t = TestGame::new(2);
    let path = t.battlefield(P0, "Sorrow's Path");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let wraith = t.battlefield(P0, "Bog Wraith");
    let kavu = t.battlefield(P1, "Amphibious Kavu");
    let bears = t.battlefield(P1, "Grizzly Bears");
    blocks(&mut t, &[wurm, wraith], &[(kavu, wurm), (bears, wraith)]);
    assert_eq!(t.stack_len(), 0);
    swap(&mut t, path, kavu, bears);
    assert_eq!(t.pt(kavu), (5, 5), "{}", t.dump_log());
    // Lairwatch Giant blocks one creature; the exchange has it block the two Hill Giants
    // the other Lairwatch Giant blocked: it now blocks two creatures, so it triggers,
    // once (as the second is added), though one effect added both.
    let mut t = TestGame::new(2);
    let path = t.battlefield(P0, "Sorrow's Path");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let h1 = t.battlefield(P0, "Hill Giant");
    let h2 = t.battlefield(P0, "Hill Giant");
    let a = t.battlefield(P1, "Lairwatch Giant");
    let b = t.battlefield(P1, "Lairwatch Giant");
    blocks(&mut t, &[wurm, h1, h2], &[(a, wurm), (b, h1), (b, h2)]);
    // Only the Lairwatch Giant blocking two creatures triggered.
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    t.resolve_all();
    assert!(!t.obj_now(a).has_keyword(KeywordKind::FirstStrike));
    swap_to_triggers(&mut t, path, a, b);
    assert_eq!(blocking(&t, a).len(), 2);
    assert_eq!(t.stack_len(), 1, "one trigger: {}", t.dump_log());
    t.resolve_all();
    assert!(
        t.obj_now(a).has_keyword(KeywordKind::FirstStrike),
        "{}",
        t.dump_log()
    );
}

#[test]
fn two_black_creatures_added_by_one_effect_trigger_once() {
    // With Brave the Sands, Amphibious Kavu can block two creatures: the exchange has it
    // block both of the Grizzly Bears' black attackers at once. Counted as of each block
    // (0, then 1, then 2 black creatures), it triggers once: +3/+3, not +6/+6 or nothing.
    cr!("509.3e");
    interp!("block-count-triggers-at-the-nth-block");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Brave the Sands");
    let path = t.battlefield(P0, "Sorrow's Path");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let w1 = t.battlefield(P0, "Bog Wraith");
    let w2 = t.battlefield(P0, "Bog Wraith");
    let kavu = t.battlefield(P1, "Amphibious Kavu");
    let bears = t.battlefield(P1, "Grizzly Bears");
    blocks(
        &mut t,
        &[wurm, w1, w2],
        &[(kavu, wurm), (bears, w1), (bears, w2)],
    );
    assert_eq!(t.stack_len(), 0, "{}", t.dump_log());
    swap_to_triggers(&mut t, path, kavu, bears);
    assert_eq!(blocking(&t, kavu).len(), 2);
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.pt(kavu), (5, 5));
}

#[test]
fn two_colorless_creatures_put_onto_the_battlefield_blocking_at_once_trigger_once() {
    // Mirror Match with Doubling Season puts two copies of the attacking Ornithopter
    // (colorless) onto the battlefield blocking it at once: Hedron Blade's "becomes
    // blocked by one or more colorless creatures" triggers once, as the first is added.
    cr!("509.3e");
    interp!("block-count-triggers-at-the-nth-block");
    supported("Hedron Blade");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Doubling Season");
    let thopter = t.battlefield(P0, "Ornithopter");
    crate::r_s06_common::attach_new(&mut t, P0, "Hedron Blade", thopter);
    blocks(&mut t, &[thopter], &[]);
    cast_card(&mut t, P1, "Mirror Match", &[]).expect("Mirror Match");
    t.resolve();
    t.settle();
    let toks = tokens(&t, P1);
    assert_eq!(toks.len(), 2, "{}", t.dump_log());
    assert!(toks.iter().all(|x| blocking(&t, *x) == vec![thopter]));
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    t.resolve_all();
    assert!(t.obj_now(thopter).has_keyword(KeywordKind::Deathtouch));
}

#[test]
fn an_exchange_of_black_blockers_triggers_becomes_blocked_by_again() {
    // Serra Inquisitors is blocked by a black creature (it triggers), and the exchange
    // swaps that blocker for another black creature: its black blocker was removed from
    // combat, so the new one brings the number of black creatures blocking it from zero
    // to one again, and it triggers again (provisional: "becomes blocked", CR 509.3c,
    // wouldn't trigger again for a creature that stayed blocked).
    cr!("509.3e", "506.4");
    interp!("block-count-triggers-at-the-nth-block");
    let mut t = TestGame::new(2);
    let path = t.battlefield(P0, "Sorrow's Path");
    let si = t.battlefield(P0, "Serra Inquisitors");
    let giant = t.battlefield(P0, "Hill Giant");
    let wraith = t.battlefield(P1, "Bog Wraith");
    let skeletons = t.battlefield(P1, "Drudge Skeletons");
    blocks(&mut t, &[si, giant], &[(wraith, si), (skeletons, giant)]);
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.pt(si), (5, 3));
    swap_to_triggers(&mut t, path, wraith, skeletons);
    assert_eq!(blocking(&t, skeletons), vec![si]);
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.pt(si), (7, 3));
}

#[test]
fn dwarven_soldier_blocks_or_is_blocked_by_orcs() {
    // "One or more Orcs" (the creature type alone): blocking an Orc, and blocked by two
    // Orcs (once); a blocker that isn't an Orc doesn't trigger it.
    cr!("509.3e");
    supported("Dwarven Soldier");
    let mut t = TestGame::new(2);
    let orc = t.battlefield(P0, "Unyielding Krumar");
    let soldier = t.battlefield(P1, "Dwarven Soldier");
    blocks(&mut t, &[orc], &[(soldier, orc)]);
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.pt(soldier), (2, 3));
    let mut t = TestGame::new(2);
    let soldier = t.battlefield(P0, "Dwarven Soldier");
    let o1 = t.battlefield(P1, "Unyielding Krumar");
    let o2 = t.battlefield(P1, "Unyielding Krumar");
    blocks(&mut t, &[soldier], &[(o1, soldier), (o2, soldier)]);
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.pt(soldier), (2, 3));
    let mut t = TestGame::new(2);
    let soldier = t.battlefield(P0, "Dwarven Soldier");
    let bears = t.battlefield(P1, "Grizzly Bears");
    blocks(&mut t, &[soldier], &[(bears, soldier)]);
    assert_eq!(t.stack_len(), 0, "{}", t.dump_log());
}

#[test]
fn becomes_blocked_by_a_black_creature_put_onto_the_battlefield_blocking() {
    // Aetherplasm (blue) blocks Serra Inquisitors, returns to hand and puts a black
    // creature card onto the battlefield blocking it: the Inquisitors are now blocked by
    // one black creature (CR 509.3e; such a creature counts for "becomes blocked by",
    // as in CR 509.3c–d).
    cr!("509.3e", "509.4");
    supported("Aetherplasm");
    let mut t = TestGame::new(2);
    let si = t.battlefield(P0, "Serra Inquisitors");
    let plasm = t.battlefield(P1, "Aetherplasm");
    let wraith = t.hand(P1, "Bog Wraith");
    t.answer_yes(P1, true);
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(wraith)]);
    blocks(&mut t, &[si], &[(plasm, si)]);
    t.resolve_all();
    let wraith_now = t.named_on_battlefield("Bog Wraith");
    assert_eq!(wraith_now.len(), 1, "{}", t.dump_log());
    assert_eq!(blocking(&t, wraith_now[0]), vec![si]);
    assert_eq!(t.pt(si), (5, 3), "{}", t.dump_log());
}

#[test]
fn a_creature_put_onto_the_battlefield_blocking_never_blocked() {
    // Aetherplasm puts Amphibious Kavu onto the battlefield blocking P0's black Bog
    // Wraith: the Kavu is blocking but never "blocked" for trigger events (CR 509.4), so
    // its "blocks ... one or more blue and/or black creatures" doesn't trigger.
    cr!("509.4", "509.3e");
    supported("Aetherplasm");
    let mut t = TestGame::new(2);
    let wraith = t.battlefield(P0, "Bog Wraith");
    let plasm = t.battlefield(P1, "Aetherplasm");
    let kavu = t.hand(P1, "Amphibious Kavu");
    t.answer_yes(P1, true);
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(kavu)]);
    blocks(&mut t, &[wraith], &[(plasm, wraith)]);
    t.resolve_all();
    let kavu_now = t.named_on_battlefield("Amphibious Kavu");
    assert_eq!(kavu_now.len(), 1, "{}", t.dump_log());
    assert_eq!(blocking(&t, kavu_now[0]), vec![wraith]);
    assert_eq!(t.pt(kavu_now[0]), (2, 2), "{}", t.dump_log());
}
