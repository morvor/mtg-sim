//! Rulings batch P178 — copies of spells made by Hive Mind and Curse of Echoes: not cast,
//! mandatory or optional, "you" and "opponent" from the copy's controller, targets kept or
//! changed, and X, modes and additional costs copied (CR 707.10).

use crate::r_p178_common::*;
use crate::r_s06_common::attach_new;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// A two-player game in P1's main phase; P0 controls Hive Mind (or, if `curse`, has
/// enchanted P1 with Curse of Echoes).
fn game(curse: bool) -> TestGame {
    let mut t = TestGame::new(2);
    if curse {
        supported("Curse of Echoes");
        attach_new(&mut t, P0, "Curse of Echoes", Entity::Player(P1));
    } else {
        supported("Hive Mind");
        t.battlefield(P0, "Hive Mind");
    }
    t.set_step(P1, Step::PrecombatMain);
    t
}

#[test]
fn hive_mind_copies_any_instant_or_sorcery_and_you_means_the_copier() {
    cr!("707.10", "109.5");
    ruling!(
        "Hive Mind",
        "Hive Mind will copy any instant or sorcery spell, not just one with targets."
    );
    ruling!(
        "Hive Mind",
        "If a copy says that it affects “you,” it affects the controller of the copy, not the controller of the original spell. Similarly, if a copy says that it affects an “opponent,” it affects an opponent of the copy’s controller"
    );
    // Divination: P0's copy draws P0 two cards.
    let mut t = game(false);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    cast_targeting(&mut t, P1, "Divination", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h0 + 2);
    assert_eq!(t.hand_size(P1), h1 + 2);
    // Sizzle ("each opponent"): P0's copy hits P1.
    let mut t = game(false);
    cast_targeting(&mut t, P1, "Sizzle", &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn hive_mind_copy_is_mandatory() {
    cr!("707.10");
    ruling!(
        "Hive Mind",
        "Hive Mind’s effect is mandatory. Each other player must copy the spell whether they want to or not."
    );
    let mut t = game(false);
    for _ in 0..4 {
        t.answer_yes(P0, false);
    }
    cast_targeting(&mut t, P1, "Sizzle", &[]);
    t.resolve();
    assert_eq!(t.stack_len(), 2, "the copy is on the stack");
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "P0's copy hit P1");
}

/// P1 casts Ranger's Guile ("Target creature you control") on its Bears; P0 copies it
/// (Hive Mind, or Curse of Echoes) and asks to change its target. Returns the game, the
/// Bears and P0's creature if `p0_creature`.
fn guile(curse: bool, p0_creature: bool) -> (TestGame, ObjectId, Option<ObjectId>) {
    let mut t = game(curse);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mine = p0_creature.then(|| t.battlefield(P0, "Hill Giant"));
    if curse {
        t.answer_yes(P0, true); // copy it
    }
    t.answer_yes(P0, true); // choose new targets
    if let Some(m) = mine {
        t.answer_targets(P0, &[obj(m)]);
    }
    cast_targeting(&mut t, P1, "Ranger's Guile", &[obj(bears)]);
    t.resolve_all();
    (t, bears, mine)
}

#[test]
fn copies_keep_a_target_that_cant_be_changed_even_an_illegal_one() {
    cr!("707.10c", "115.7d", "608.2b");
    ruling!(
        "Hive Mind",
        "Each copy will have the same targets as the spell it’s copying unless its controller chooses new ones. That player may change any number of the targets, including all of them or none of them. If, for one of the targets, the player can’t choose a new legal target, then it remains unchanged (even if the current target is illegal)."
    );
    ruling!(
        "Curse of Echoes",
        "Each copy will have the same targets as the spell it’s copying unless its controller chooses new ones. That player may change any number of the targets, including all of them or none of them. If, for one of the targets, that player can’t choose a new legal target, then it remains unchanged (even if the current target is illegal)."
    );
    for curse in [false, true] {
        // P0 controls no creature: the copy keeps targeting P1's Bears, which isn't a
        // creature P0 controls, so the copy does nothing.
        let (t, bears, _) = guile(curse, false);
        assert_eq!(t.pt(bears), (3, 3), "only the original pumped the Bears");
        // P0 controls a creature: the copy can target it instead.
        let (t, bears, mine) = guile(curse, true);
        assert_eq!(t.pt(bears), (3, 3));
        assert_eq!(t.pt(mine.unwrap()), (4, 4));
    }
    // None of the targets changed: P0's copy of Lightning Bolt hits the same creature.
    for curse in [false, true] {
        let mut t = game(curse);
        let giant = t.battlefield(P0, "Craw Wurm");
        if curse {
            t.answer_yes(P0, true);
        }
        t.answer_yes(P0, false);
        cast_targeting(&mut t, P1, "Lightning Bolt", &[obj(giant)]);
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Craw Wurm"), "6 damage to the 6/4");
    }
}

#[test]
fn copies_have_the_same_x() {
    cr!("707.10", "107.3c");
    ruling!(
        "Hive Mind",
        "If the spell that’s copied has an X whose value was determined as it was cast (like Earthquake does), the copy has the same value of X."
    );
    ruling!(
        "Curse of Echoes",
        "If the original spell has an X whose value was determined as it was cast, each copy has the same value of X."
    );
    // Hive Mind: Earthquake with X = 2, copied: each player takes 4.
    let mut t = game(false);
    give_mana_for(&mut t, P1, "Earthquake");
    t.lands(P1, "Wastes", 2);
    let quake = t.hand(P1, "Earthquake");
    t.cast(P1, quake).x(2).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 16);
    assert_eq!(t.life(P1), 16);
    // Curse of Echoes: Blaze with X = 3 at P0, P0's copy retargeted at P1.
    let mut t = game(true);
    give_mana_for(&mut t, P1, "Blaze");
    t.lands(P1, "Wastes", 3);
    let blaze = t.hand(P1, "Blaze");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P1, blaze).x(3).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn copies_have_the_same_mode() {
    cr!("707.10", "700.2g");
    ruling!(
        "Hive Mind",
        "If the spell that’s copied is modal (that is, it says “Choose one —” or the like), the copy will have the same mode. A player can’t choose a different one."
    );
    ruling!(
        "Curse of Echoes",
        "If the original spell is modal (that is, it says “Choose one —” or the like), each copy will have the same mode(s). You can’t choose different ones."
    );
    for curse in [false, true] {
        let mut t = game(curse);
        // Abzan Charm, "You draw two cards and you lose 2 life."
        if curse {
            t.answer_yes(P0, true);
        }
        give_mana_for(&mut t, P1, "Abzan Charm");
        let charm = t.hand(P1, "Abzan Charm");
        let h0 = t.hand_size(P0);
        t.cast(P1, charm).modes(&[1]).go();
        t.resolve_all();
        assert_eq!(t.hand_size(P0), h0 + 2, "P0's copy has the draw mode");
        assert_eq!(t.life(P0), 18);
        assert_eq!(t.life(P1), 18);
    }
}

#[test]
fn hive_mind_copy_is_kicked_if_the_original_was() {
    cr!("707.10", "702.33d");
    ruling!(
        "Hive Mind",
        "A copy’s controller can’t choose to pay any additional costs for the copy. However, effects based on any additional costs that were paid for the original spell are copied as though those same costs were paid for the copy too."
    );
    for kicked in [true, false] {
        let mut t = game(false);
        give_mana_for(&mut t, P1, "Burst Lightning");
        t.lands(P1, "Wastes", 4);
        let burst = t.hand(P1, "Burst Lightning");
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.cast(P1, burst)
            .kicked(kicked)
            .target(Entity::Player(P0))
            .go();
        t.resolve_all();
        let n = if kicked { 4 } else { 2 };
        assert_eq!(t.life(P0), 20 - n);
        assert_eq!(t.life(P1), 20 - n, "the copy is kicked: {kicked}");
    }
}

#[test]
fn curse_of_echoes_copies_are_not_cast() {
    cr!("707.10", "601.2");
    ruling!(
        "Curse of Echoes",
        "All the copies are created on the stack, so none of them are cast. Abilities that trigger when a player casts a spell won’t trigger."
    );
    let mut t = game(true);
    let hydra = t.battlefield(P0, "Managorger Hydra");
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    cast_targeting(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 14, "the original and P0's copy (same target)");
    assert_eq!(t.counters(hydra, "+1/+1"), 1, "only the cast spell counted");
}
