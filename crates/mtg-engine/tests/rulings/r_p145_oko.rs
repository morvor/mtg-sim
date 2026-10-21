//! Rulings batch P145 — Oko, the Ringleader: becoming a copy of a creature until end of
//! turn (except he has hexproof), and his −5 token copies.

use crate::r_p145_common::*;
use crate::r_p057_common::pump;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{counters, CardType};
use mtg_engine::*;

const OKO: &str = "Oko, the Ringleader";

/// Oko's beginning of combat trigger, targeting `target`, put on the stack.
fn oko_trigger(t: &mut TestGame, target: ObjectId) {
    t.answer_targets(P0, &[obj(target)]);
    into_beginning_of_combat(t, P0);
    assert_eq!(triggers_on_stack(t, "becomes a copy"), 1);
}

fn hexproof(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).has_keyword(KeywordKind::Hexproof)
}

#[test]
fn oko_copies_only_the_copiable_values() {
    cr!("707.2", "707.9b", "613.2a");
    ruling!(
        "Oko, the Ringleader",
        "Except for the listed exception, Oko copies exactly what was printed on the target creature and nothing else"
    );
    ruling!(
        "Oko, the Ringleader",
        "While Oko is a copy of a creature, he isn't a planeswalker. He won't have his loyalty abilities, and damage dealt to him won't cause him to lose loyalty counters."
    );
    supported(OKO);
    let mut t = TestGame::new(2);
    let oko = t.battlefield(P0, OKO);
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.g.objects[wurm.0 as usize].tapped = true;
    put_counters(&mut t, wurm, counters::PLUS1, 1);
    pump(&mut t, wurm, 2, 2);
    cast_resolve(&mut t, P0, "Rancor", &[obj(wurm)]);
    assert_eq!(t.pt(wurm), (11, 7));
    oko_trigger(&mut t, wurm);
    t.resolve();
    let o = t.obj_now(oko);
    assert_eq!(o.chars.name, "Craw Wurm");
    assert!(o.is(CardType::Creature) && !o.is(CardType::Planeswalker));
    assert!(!o.tapped);
    assert_eq!(t.pt(oko), (6, 4), "no counters, pump or Aura copied");
    assert!(hexproof(&t, oko));
    let loyalty = t.counters(oko, counters::LOYALTY);
    assert!(loyalty > 0);
    assert!(!can_activate(&mut t, P0, oko), "no loyalty abilities");
    // Damage to him is marked damage, not loyalty loss.
    t.set_step(P0, Step::BeginningOfCombat);
    // (Hexproof: P0 has to Shock him.)
    cast_new(&mut t, P0, "Shock", &[obj(oko)]);
    t.resolve();
    assert_eq!(t.counters(oko, counters::LOYALTY), loyalty);
    assert_eq!(t.obj_now(oko).damage, 2);
    assert!(t.on_battlefield(oko));
}

#[test]
fn oko_copies_tokens_and_copies_as_they_are_copied() {
    cr!("707.2", "707.3", "111.3");
    ruling!(
        "Oko, the Ringleader",
        "If the copied creature is a token, Oko copies the original characteristics of that token as stated by the effect that created that token, with the stated exception."
    );
    ruling!(
        "Oko, the Ringleader",
        "If the copied creature is copying something else, then Oko becomes a copy of whatever that creature copied, with the stated exception."
    );
    // A pumped Elk token from Oko's −1.
    let mut t = TestGame::new(2);
    let oko = t.battlefield(P0, OKO);
    activate_containing(&mut t, P0, oko, "Elk").expect("−1");
    t.resolve_all();
    let elk = tokens(&t, P0)[0];
    pump(&mut t, elk, 1, 1);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    oko_trigger(&mut t, elk);
    t.resolve();
    assert!(t.obj_now(oko).chars.has_subtype("Elk"));
    assert_eq!(t.pt(oko), (3, 3));
    assert!(!t.obj_now(oko).is_token());
    // A Clone copying Grizzly Bears: Oko becomes Grizzly Bears.
    let mut t = TestGame::new(2);
    let oko = t.battlefield(P0, OKO);
    let bears = t.battlefield(P0, "Grizzly Bears");
    yes(&mut t, P0);
    t.answer_choose(P0, &[obj(bears)]);
    let clone = enter(&mut t, P0, "Clone");
    t.resolve_all();
    assert_eq!(name_of(&t, clone), "Grizzly Bears");
    oko_trigger(&mut t, clone);
    t.resolve();
    assert_eq!(name_of(&t, oko), "Grizzly Bears");
    assert!(hexproof(&t, oko));
}

#[test]
fn oko_with_an_illegal_target_stays_oko() {
    cr!("608.2b");
    ruling!(
        "Oko, the Ringleader",
        "If the target creature is an illegal target as Oko's first ability tries to resolve, it won't resolve and none of its effects will happen. Oko won't gain hexproof."
    );
    let mut t = TestGame::new(2);
    let oko = t.battlefield(P0, OKO);
    let bears = t.battlefield(P0, "Grizzly Bears");
    oko_trigger(&mut t, bears);
    destroy(&mut t, bears);
    t.resolve();
    assert_eq!(name_of(&t, oko), OKO);
    assert!(t.obj_now(oko).is(CardType::Planeswalker));
    assert!(!hexproof(&t, oko));
}

#[test]
fn oko_becoming_a_copy_doesnt_enter_or_leave() {
    cr!("707.4", "603.6a");
    ruling!(
        "Oko, the Ringleader",
        "When Oko becomes a copy of the target creature, he's neither entering nor leaving the battlefield. Any enters-the-battlefield or leaves-the-battlefield abilities won't trigger."
    );
    let mut t = TestGame::new(2);
    let oko = t.battlefield(P0, OKO);
    let wall = t.battlefield(P0, "Wall of Omens");
    oko_trigger(&mut t, wall);
    t.resolve();
    assert_eq!(name_of(&t, oko), "Wall of Omens");
    assert_eq!(t.stack_len(), 0, "no enters trigger");
    assert_eq!(t.hand_size(P0), 0);
    // At end of turn he stops being a copy: no leaves trigger either, and he's Oko again.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(name_of(&t, oko), OKO);
    assert_eq!(t.g.current(oko), oko);
}

#[test]
fn oko_minus_five_tokens_copy_only_copiable_values() {
    cr!("707.2", "111.4");
    ruling!(
        "Oko, the Ringleader",
        "Each of the tokens created by Oko's last ability copies exactly what was printed on the original permanent and nothing else"
    );
    let mut t = TestGame::new(2);
    let oko = t.battlefield(P0, OKO);
    t.g.objects[oko.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 5);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    put_counters(&mut t, bears, counters::PLUS1, 2);
    pump(&mut t, bears, 1, 0);
    activate_containing(&mut t, P0, oko, "For each other").expect("−5");
    t.resolve_all();
    let copies: Vec<ObjectId> = tokens(&t, P0);
    assert_eq!(copies.len(), 1, "a copy of Grizzly Bears (Oko isn't copied)");
    let c = copies[0];
    assert_eq!(name_of(&t, c), "Grizzly Bears");
    assert!(!t.obj_now(c).tapped);
    assert_eq!(t.pt(c), (2, 2));
}
