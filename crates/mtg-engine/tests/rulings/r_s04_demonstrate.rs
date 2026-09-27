//! Rulings batch S04 — demonstrate (CR 702.144): "When you cast this spell, you may copy
//! it and you may choose new targets for the copy. If you copy the spell, choose an
//! opponent. That player copies the spell and may choose new targets for that copy."

use crate::r_s01_common::*;
use crate::r_s04_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::{ObjKind, StackKind};
use mtg_engine::testing::*;
use mtg_engine::*;

/// The spells on the stack, bottom first: (controller, is a copy, targets).
fn spells(g: &mtg_engine::game::Game) -> Vec<(PlayerId, bool, Vec<Entity>)> {
    g.stack
        .iter()
        .filter(|s| matches!(g.obj(**s).kind, ObjKind::Card | ObjKind::SpellCopy))
        .filter(|s| {
            matches!(
                g.obj(**s).stack.as_deref().map(|si| &si.kind),
                Some(StackKind::Spell)
            )
        })
        .map(|s| {
            let o = g.obj(*s);
            let targets = o
                .stack
                .as_deref()
                .map(|si| {
                    si.chosen
                        .iter()
                        .flat_map(|c| c.targets.iter().flatten().copied())
                        .collect()
                })
                .unwrap_or_default();
            (o.controller, o.kind == ObjKind::SpellCopy, targets)
        })
        .collect()
}

#[test]
fn the_opponents_copy_resolves_first_then_yours_then_the_original() {
    cr!("702.144a", "707.10", "405.5");
    ruling!(
        "Healing Technique",
        "This means that if you cast a spell with demonstrate and both you and an opponent copy it, the opponent’s copy will resolve first, then your copy will resolve, and finally the original spell will resolve."
    );
    supported("Healing Technique");
    // Healing Technique: "Return target card from your graveyard to your hand. You gain
    // life equal to that card's mana value. Exile Healing Technique."
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P0, "Hill Giant");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let wurm = t.graveyard(P1, "Craw Wurm");
    give_mana_for(&mut t, P0, "Healing Technique");
    let spell = t.hand(P0, "Healing Technique");
    t.cast(P0, spell).target(giant).go();
    // P0 copies it (new target: the Bears) and chooses P1, whose copy targets the Wurm.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[Entity::Object(wurm)]);
    t.settle();
    t.resolve();
    let s = spells(&t.g);
    assert_eq!(
        s.iter().map(|(p, c, _)| (*p, *c)).collect::<Vec<_>>(),
        vec![(P0, false), (P0, true), (P1, true)]
    );
    // P1's copy resolves first.
    t.resolve();
    assert!(t.in_hand(P1, "Craw Wurm"));
    assert_eq!((t.life(P0), t.life(P1)), (20, 26));
    assert_eq!(t.zone(bears), mtg_engine::object::Zone::Graveyard(P0));
    // Then P0's copy.
    t.resolve();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.zone(giant), mtg_engine::object::Zone::Graveyard(P0));
    // And finally the original.
    t.resolve();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(t.life(P0), 26);
    assert!(t.in_exile("Healing Technique"));
    assert!(t.g.stack.is_empty());
}

#[test]
fn the_opponent_chooses_new_targets_knowing_the_other_targets() {
    cr!("702.144a", "707.10c", "601.2c");
    ruling!(
        "Replication Technique",
        "If the spell requires targets, you choose the target of the original spell as you cast it. If you create a copy of the spell, you may choose new targets for the copy as you create that copy. Similarly, the opponent you chose to create a copy may choose new targets for that copy as it's created. In other words, your opponent will know the targets of your original spell and your copy when choosing the new targets, if any, for their copy."
    );
    supported("Replication Technique");
    // Replication Technique: "Create a token that's a copy of target permanent you
    // control."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    give_mana_for(&mut t, P0, "Replication Technique");
    let spell = t.hand(P0, "Replication Technique");
    // What P1 sees on the stack when choosing the new target for their copy.
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseTargets { .. }),
        spells,
    );
    let from = t.asked().len();
    t.cast(P0, spell).target(bears).go();
    // The original's target was chosen as it was cast, before the demonstrate trigger.
    assert_eq!(spells(&t.g), vec![(P0, false, vec![Entity::Object(bears)])]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[Entity::Object(wurm)]);
    t.settle();
    t.resolve();
    // P0 chose the copy's target before P1 chose theirs.
    let order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P0, P0, P1]);
    let seen = seen.lock().unwrap().clone();
    assert_eq!(
        seen,
        vec![vec![
            (P0, false, vec![Entity::Object(bears)]),
            (P0, true, vec![Entity::Object(giant)]),
            (P1, true, vec![Entity::Object(bears)]),
        ]]
    );
    t.resolve_all();
    // Each copy made a token of its new target for its controller.
    let tokens_named = |p: PlayerId, name: &str| {
        tokens(&t, p)
            .into_iter()
            .filter(|x| t.obj(*x).chars.name == name)
            .count()
    };
    assert_eq!(tokens_named(P0, "Grizzly Bears"), 1);
    assert_eq!(tokens_named(P0, "Hill Giant"), 1);
    assert_eq!(tokens_named(P1, "Craw Wurm"), 1);
}
