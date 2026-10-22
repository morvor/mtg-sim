//! Rulings on strive ("This spell costs [cost] more to cast for each target beyond the
//! first"): choosing the number of targets, partially illegal targets, and copies.

use crate::r_s01_common::*;
use crate::r_s07_common::resolved;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The targets chosen for the spell (or copy) `id` on the stack, first slot.
fn chosen_targets(t: &TestGame, id: ObjectId) -> Vec<Entity> {
    t.g.obj(id).stack.as_ref().unwrap().chosen[0].targets[0].clone()
}

#[test]
fn a_strive_spell_may_have_no_targets_and_cant_target_the_same_creature_twice() {
    cr!("601.2c", "115.3");
    ruling!(
        "Ajani's Presence",
        "You choose how many targets each spell with a strive ability has and what those targets are as you cast it. It's legal to cast such a spell with no targets, although this is rarely a good idea. You can't choose the same target more than once for a single strive spell."
    );
    supported("Ajani's Presence");
    // Ajani's Presence ({W}): "Strive — This spell costs {2}{W} more to cast for each
    // target beyond the first. Any number of target creatures each get +1/+1 and gain
    // indestructible until end of turn."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Plains", 1);
    // No targets: legal, for {W}; it resolves and does nothing.
    let from = t.asked().len();
    let p = t.hand(P0, "Ajani's Presence");
    let spell = t.cast(P0, p).targets(&[]).go();
    let asked: Vec<(u32, u32)> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { min, max, .. } => Some((*min, *max)),
            _ => None,
        })
        .collect();
    assert_eq!(asked, vec![(0, 2)]);
    assert!(chosen_targets(&t, spell).is_empty());
    t.resolve_all();
    assert!(resolved(&t, spell));
    assert_eq!((t.pt(a), t.pt(b)), ((2, 2), (3, 3)));
    // Choosing the same creature twice isn't a legal choice of targets: the spell never
    // targets it twice (nor pays for a second target).
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Wastes", 2);
    let p = t.hand(P0, "Ajani's Presence");
    let spell = t.cast(P0, p).targets(&[a.into(), a.into()]).go();
    let chosen = chosen_targets(&t, spell);
    assert!(chosen.len() <= 1, "targets {chosen:?}");
    assert_eq!(tapped_lands(&t, P0), 2);
    t.resolve_all();
    assert!(t.pt(a).0 <= 3, "the Bears got +1/+1 at most once");
    // Two different creatures: {W} plus {2}{W}.
    let a_before = t.pt(a);
    t.lands(P0, "Plains", 1);
    let p = t.hand(P0, "Ajani's Presence");
    let spell = t.cast(P0, p).targets(&[a.into(), b.into()]).go();
    assert_eq!(chosen_targets(&t, spell).len(), 2);
    assert_eq!(tapped_lands(&t, P0), 6);
    t.resolve_all();
    assert_eq!(t.pt(a), (a_before.0 + 1, a_before.1 + 1));
    assert_eq!(t.pt(b), (4, 4));
    assert!(t.obj_now(b).chars.has_keyword(KeywordKind::Indestructible));
}

#[test]
fn a_strive_spell_affects_only_its_legal_targets_and_doesnt_resolve_if_all_are_illegal() {
    cr!("608.2b");
    ruling!(
        "Harness by Force",
        "If all of the spell's targets are illegal when the spell tries to resolve, it won't resolve and none of its effects will happen. If one or more of its targets are legal when it tries to resolve, the spell will resolve and affect only those legal targets. It will have no effect on any illegal targets."
    );
    supported("Harness by Force");
    supported("Blossoming Defense");
    // Harness by Force ({1}{R}{R}): "Strive — This spell costs {2}{R} more to cast for
    // each target beyond the first. Gain control of any number of target creatures until
    // end of turn. Untap those creatures. They gain haste until end of turn."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 6);
    let h = t.hand(P0, "Harness by Force");
    let spell = t
        .cast(P0, h)
        .targets(&[bears.into(), giant.into()])
        .go();
    // In response, P1 gives the Hill Giant hexproof with Blossoming Defense.
    t.lands(P1, "Forest", 1);
    let bd = t.hand(P1, "Blossoming Defense");
    t.cast(P1, bd).target(giant).go();
    t.resolve();
    t.resolve();
    assert!(resolved(&t, spell));
    assert_eq!(t.obj_now(bears).controller, P0);
    assert!(t.obj_now(bears).chars.has_keyword(KeywordKind::Haste));
    assert_eq!(t.obj_now(giant).controller, P1);
    assert!(!t.obj_now(giant).chars.has_keyword(KeywordKind::Haste));

    // All of its targets illegal: it doesn't resolve.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let h = t.hand(P0, "Harness by Force");
    let spell = t.cast(P0, h).targets(&[bears.into()]).go();
    t.lands(P1, "Forest", 1);
    let bd = t.hand(P1, "Blossoming Defense");
    t.cast(P1, bd).target(bears).go();
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert!(t.in_graveyard(P0, "Harness by Force"));
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn a_copy_of_a_strive_spell_keeps_its_number_of_targets() {
    cr!("707.10c", "115.7d");
    ruling!(
        "Ajani's Presence",
        "If such a spell is copied, and the effect that copies the spell allows a player to choose new targets for the copy, the number of targets can't be changed. The player may change any number of the targets, including all of them or none of them. If, for one of the targets, the player can't choose a new legal target, then it remains unchanged (even if the current target is illegal)."
    );
    supported("Twincast");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Gray Ogre");
    t.lands(P0, "Plains", 4);
    let p = t.hand(P0, "Ajani's Presence");
    let spell = t.cast(P0, p).targets(&[a.into(), b.into()]).go();
    // Twincast: "Copy target instant or sorcery spell. You may choose new targets for the
    // copy." The copy's first target becomes Gray Ogre; its second stays Hill Giant.
    t.lands(P0, "Island", 2);
    let tc = t.hand(P0, "Twincast");
    t.cast(P0, tc).target(spell).go();
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[c.into()]);
    t.answer_targets(P0, &[]);
    t.resolve();
    let copy = *t.g.stack.last().unwrap();
    assert_ne!(copy, spell);
    assert_eq!(
        chosen_targets(&t, copy),
        vec![Entity::Object(c), Entity::Object(b)]
    );
    // Each target was asked about separately, one new target at a time.
    let maxes: Vec<u32> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { max, .. } => Some(*max),
            _ => None,
        })
        .collect();
    assert_eq!(maxes, vec![1, 1]);
    t.resolve_all();
    assert_eq!(t.pt(a), (3, 3));
    assert_eq!(t.pt(b), (5, 5));
    assert_eq!(t.pt(c), (3, 3));
}

#[test]
fn a_target_of_the_copy_with_no_new_legal_choice_stays_even_if_illegal() {
    cr!("707.10c", "115.7d", "115.7e");
    ruling!(
        "Ajani's Presence",
        "If, for one of the targets, the player can't choose a new legal target, then it remains unchanged (even if the current target is illegal)."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Plains", 4);
    let p = t.hand(P0, "Ajani's Presence");
    let spell = t.cast(P0, p).targets(&[a.into(), b.into()]).go();
    // The Bears leave: the spell's first target is illegal, and the only other creature is
    // already its second target, so it can't be chosen for the first one (CR 115.3).
    crate::r_s02_common::destroy(&mut t, a);
    t.lands(P0, "Island", 2);
    let tc = t.hand(P0, "Twincast");
    t.cast(P0, tc).target(spell).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[b.into()]);
    t.answer_targets(P0, &[]);
    t.resolve();
    let copy = *t.g.stack.last().unwrap();
    assert_eq!(
        chosen_targets(&t, copy),
        vec![Entity::Object(a), Entity::Object(b)]
    );
    t.resolve_all();
    assert_eq!(t.pt(b), (5, 5));
}
