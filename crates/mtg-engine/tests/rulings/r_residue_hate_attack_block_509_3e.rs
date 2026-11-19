//! Rulings residue `rulings-residue-hate-attack-block`: "Whenever ~ blocks or becomes
//! blocked by one or more [quality] creatures" (Serra Inquisitors, Amphibious Kavu,
//! Dwarven Soldier) and "Whenever ~ blocks one or more black creatures / two or more
//! creatures" (Rashka the Slayer, Lairwatch Giant) trigger as blockers are declared and
//! also when an effect adds the blocker that makes the count reach the number (CR 509.3e):
//! here Sorrow's Path exchanging two blockers, and a creature put onto the battlefield
//! blocking.

use crate::r_residue_hate_attack_block_common::*;
use crate::r_s01_common::supported;
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

#[test]
fn becomes_blocked_by_one_or_more_black_creatures_when_an_exchange_adds_one() {
    cr!("509.3e", "509.1h");
    for name in ["Serra Inquisitors", "Amphibious Kavu", "Dwarven Soldier"] {
        supported(name);
    }
    // Serra Inquisitors is blocked by a green creature; Sorrow's Path then has the black
    // creature blocking P0's Hill Giant block it instead: it's now blocked by one black
    // creature, so it triggers (the attacker stayed blocked all along, CR 509.1h).
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
    cr!("509.3e", "509.3a");
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
    // the other Lairwatch Giant blocked: it now blocks two creatures, so it triggers
    // (once, as the second is added).
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
    swap(&mut t, path, a, b);
    assert_eq!(blocking(&t, a).len(), 2);
    assert!(
        t.obj_now(a).has_keyword(KeywordKind::FirstStrike),
        "{}",
        t.dump_log()
    );
}

#[test]
fn becomes_blocked_by_a_black_creature_put_onto_the_battlefield_blocking() {
    // Aetherplasm (blue) blocks Serra Inquisitors, returns to hand and puts a black
    // creature card onto the battlefield blocking it: the Inquisitors are now blocked by
    // one black creature (CR 509.3e, 509.4).
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
