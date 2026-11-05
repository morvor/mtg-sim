//! Rulings batch P012 — abilities that count the spells cast this turn (Sentinel Tower,
//! Nullstone Gargoyle, Shadow in the Warp): the count covers every spell cast this turn,
//! including ones cast before the permanent was on the battlefield, ones still on the
//! stack and ones countered (CR 603.2, 601.2i).

use crate::r_p012_common::*;
use crate::r_s01_common::supported;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn sentinel_tower_counts_the_instants_and_sorceries_cast_before_this_turn() {
    cr!("603.2", "601.2i");
    ruling!(
        "Sentinel Tower",
        "For example, the first instant or sorcery spell cast during your turn causes Sentinel Tower to deal 1 damage; the second causes it to deal 2 damage, and so on."
    );
    ruling!(
        "Sentinel Tower",
        "Sentinel Tower’s ability counts all instant and sorcery spells cast in the turn before the spell that caused it to trigger, even if those spells are still on the stack or have been countered."
    );
    supported("Sentinel Tower");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sentinel Tower");
    t.answer_targets(P0, &[pl(P1)]);
    cast_card(&mut t, P0, "Opt", &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // The second is countered after its trigger resolves; it still counts for the third,
    // and the third (still on the stack) counts for a fourth cast in response.
    t.answer_targets(P0, &[pl(P1)]);
    let opt = cast_card(&mut t, P0, "Opt", &[]);
    t.resolve(); // the Tower's trigger: 2
    assert!(t.g.counter(opt, None));
    assert_eq!(t.life(P1), 17);
    t.answer_targets(P0, &[pl(P1)]);
    let third = cast_card(&mut t, P0, "Opt", &[]);
    t.resolve(); // 3
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.zone(third), Zone::Stack);
    t.answer_targets(P0, &[pl(P1)]);
    cast_card(&mut t, P0, "Opt", &[]);
    t.resolve(); // 4
    assert_eq!(t.life(P1), 10);
    // Creature spells don't trigger it.
    t.resolve_all();
    cast_card(&mut t, P0, "Grizzly Bears", &[]);
    assert!(!trigger_on_top(&t));
}

#[test]
fn sentinel_towers_controller_chooses_the_target_whoever_cast_the_spell() {
    cr!("603.3d", "109.5");
    ruling!(
        "Sentinel Tower",
        "You choose the target for Sentinel Tower’s ability, regardless of who cast the spell that caused it to trigger."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sentinel Tower");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // P1 casts an instant during P0's turn; P0 targets P1's Bears.
    t.answer_targets(P0, &[obj(bears)]);
    let from = t.asked().len();
    cast_card(&mut t, P1, "Opt", &[]);
    assert!(trigger_on_top(&t));
    let tower_target_chooser: Vec<_> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, mtg_engine::decision::Decision::ChooseTargets { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(tower_target_chooser, vec![P0]);
    t.resolve();
    assert_eq!(t.obj_now(bears).damage, 1);
    // During P1's turn, P1's instant doesn't trigger it.
    t.resolve_all();
    t.set_step(P1, Step::PrecombatMain);
    cast_card(&mut t, P1, "Opt", &[]);
    assert!(!trigger_on_top(&t));
}

#[test]
fn nullstone_gargoyle_counters_only_the_first_noncreature_spell_of_the_turn() {
    cr!("603.2", "701.6a");
    ruling!(
        "Nullstone Gargoyle",
        "If another spell is cast in response to the first spell, Nullstone Gargoyle doesn’t affect that second spell."
    );
    ruling!(
        "Nullstone Gargoyle",
        "Nullstone Gargoyle cares about the first noncreature spell cast during a turn, not the first one it “sees.” If a noncreature spell was cast before Nullstone Gargoyle entered the battlefield, it won’t counter the next one."
    );
    supported("Nullstone Gargoyle");
    // A spell cast in response to the first isn't countered.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Nullstone Gargoyle");
    let first = cast_card(&mut t, P0, "Opt", &[]);
    assert!(trigger_on_top(&t));
    let second = cast_card(&mut t, P0, "Opt", &[]);
    assert!(!trigger_on_top(&t));
    t.resolve(); // the second Opt
    assert_eq!(t.zone(second), Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(t.zone(first), Zone::Graveyard(P0));
    assert_eq!(t.hand_size(P0), 1, "only the second Opt drew");

    // A noncreature spell cast before the Gargoyle was on the battlefield counts.
    let mut t = TestGame::new(2);
    cast_card(&mut t, P0, "Opt", &[]);
    t.resolve_all();
    t.battlefield(P1, "Nullstone Gargoyle");
    cast_card(&mut t, P0, "Opt", &[]);
    assert!(!trigger_on_top(&t));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn shadow_in_the_warp_counts_spells_cast_before_it_was_on_the_battlefield() {
    cr!("603.2", "601.2f");
    ruling!(
        "Shadow in the Warp",
        "Both the first and last abilities of Shadow in the Warp take into account spells cast before it was on the battlefield. For example, if an opponent cast a noncreature spell before it was on the battlefield, then casts a noncreature spell after it is put on the battlefield, Shadow in the Warp's second ability won't trigger because that is the second noncreature spell they've cast that turn."
    );
    supported("Shadow in the Warp");
    // The last ability: P1's first noncreature spell was cast before.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    cast_card(&mut t, P1, "Opt", &[]);
    t.resolve_all();
    t.battlefield(P0, "Shadow in the Warp");
    cast_card(&mut t, P1, "Opt", &[]);
    assert!(!trigger_on_top(&t));
    // Next turn, P1's first noncreature spell is punished.
    let mut t2 = TestGame::new(2);
    t2.battlefield(P0, "Shadow in the Warp");
    t2.set_step(P1, Step::PrecombatMain);
    cast_card(&mut t2, P1, "Opt", &[]);
    assert!(trigger_on_top(&t2));
    t2.resolve_all();
    assert_eq!(t2.life(P1), 18);

    // The first ability: P0's first creature spell was cast before; the next one costs
    // full price (Grizzly Bears, {1}{G}, with only a Forest: can't be cast).
    let mut t = TestGame::new(2);
    cast_card(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    t.battlefield(P0, "Shadow in the Warp");
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
    // Without the earlier creature spell it costs {G}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shadow in the Warp");
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
}
