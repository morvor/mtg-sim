//! Rulings batch P162 — "can't be blocked by more than one creature" with menace,
//! optional additional costs and illegal targets, "whenever you gain life" per life-gain
//! event, and swap removal (Divergent Transformations): sequential entering and
//! triggers waiting until it finishes resolving.

use crate::r_p076_common::mana;
use crate::r_p162_common::*;
use crate::r_s01_common::{attack_with, stack_library, supported};
use crate::r_s02_common::destroy;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn sinuous_predator_with_menace_cant_be_blocked() {
    cr!("509.1b", "702.111b");
    ruling!(
        "Kessig Prowler // Sinuous Predator",
        "If Sinuous Predator gains menace, it can't be blocked by only one creature and it can't be blocked by more than one creature, so it simply can't be blocked."
    );
    supported("Kessig Prowler // Sinuous Predator");
    supported("Imposing Visage");
    let mut t = TestGame::new(2);
    let pred =
        crate::r_s17_common::enter_transformed(&mut t, P0, "Kessig Prowler // Sinuous Predator");
    t.g.objects[pred.0 as usize].summoning_sick = false;
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let legal = |t: &mut TestGame, decl: &[(ObjectId, ObjectId)]| {
        t.g.recompute();
        let opts = mtg_engine::combat::block_options(&t.g, &[P1]);
        mtg_engine::combat::block_declaration_legal(&t.g, &opts, decl)
    };
    attack_with(&mut t, &[(pred, Entity::Player(P1))]);
    assert!(legal(&mut t, &[(a, pred)]));
    assert!(!legal(&mut t, &[(a, pred), (b, pred)]));
    let visage = t.battlefield(P0, "Imposing Visage");
    t.g.attach(visage, Entity::Object(pred));
    assert!(!legal(&mut t, &[(a, pred)]));
    assert!(!legal(&mut t, &[(a, pred), (b, pred)]));
    assert!(legal(&mut t, &[]));
}

#[test]
fn requiting_hex_illegal_target_no_life_even_with_blight_paid() {
    cr!("608.2b", "601.2f");
    ruling!(
        "Requiting Hex",
        "If the target creature is an illegal target as Requiting Hex tries to resolve, it won't resolve and none of its effects will happen. You won't gain life even if the additional cost was paid."
    );
    supported("Requiting Hex");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    mana(&mut t, P0, ManaType::B, 1);
    let c = t.hand(P0, "Requiting Hex");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, c).target(bears).go();
    assert_eq!(t.counters(mine, counters::MINUS1), 1);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // With a legal target, the life is gained.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    mana(&mut t, P0, ManaType::B, 1);
    let c = t.hand(P0, "Requiting Hex");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, c).target(bears).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P0), 22);
}

#[test]
fn cradle_of_vitality_triggers_per_life_gain_event() {
    cr!("119.9", "603.2", "603.3d");
    ruling!(
        "Cradle of Vitality",
        "Cradle of Vitality triggers just once for each life-gaining event, whether it’s 1 life from Deathgreeter or 8 life from Soul’s Grace. You pay the cost just once, but the targeted creature will get a number of counters equal to the amount of life you gained."
    );
    ruling!(
        "Cradle of Vitality",
        "You choose a target when the ability triggers. You don’t choose whether to pay {1}{W} until the ability resolves."
    );
    supported("Cradle of Vitality");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cradle of Vitality");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    let from = t.asked().len();
    t.g.gain_life(P0, 4);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // The target was chosen as it was put on the stack; no payment question yet.
    let asked = &t.asked()[from..];
    assert!(asked
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    assert!(!asked
        .iter()
        .any(|(_, d)| matches!(d, Decision::YesNo { .. } | Decision::OptionalCost { .. })));
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 4);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 2);
}

#[test]
fn cradle_of_vitality_triggers_for_each_lifelink_creature() {
    cr!("119.9", "702.15b", "510.2");
    ruling!(
        "Cradle of Vitality",
        "If multiple creatures you control with lifelink deal damage simultaneously, each instance is a single life-gain event and the ability will trigger separately for each of them."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cradle of Vitality");
    let a = t.battlefield(P0, "Vampire Nighthawk");
    let b = t.battlefield(P0, "Vampire Nighthawk");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![]),
    );
    t.advance_to(P0, mtg_engine::turn::Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P0), 24);
    assert_eq!(
        crate::r_s01_common::triggers_on_stack(&t, "+1/+1 counter"),
        2
    );
}

/// P0 casts Divergent Transformations on `a` then `b`.
fn divergent(t: &mut TestGame, a: ObjectId, b: ObjectId) {
    mana(t, P0, ManaType::R, 1);
    mana(t, P0, ManaType::C, 5);
    let c = t.hand(P0, "Divergent Transformations");
    t.cast(P0, c)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.settle();
    t.g.resolve_top();
}

#[test]
fn divergent_transformations_enters_sequentially_and_triggers_wait() {
    cr!("603.3", "608.2c", "603.6a");
    ruling!(
        "Divergent Transformations",
        "Any abilities that trigger during the resolution of Divergent Transformations will wait to be put onto the stack until Divergent Transformations finishes resolving. An ability that triggers on the first creature entering the battlefield may target the second creature and vice versa."
    );
    ruling!(
        "Divergent Transformations",
        "The two creatures that are put onto the battlefield are put onto the battlefield sequentially. Triggered abilities of the second won't see the first enter the battlefield."
    );
    supported("Divergent Transformations");
    // P0's Flametongue Kavu enters first; its trigger can target P1's creature that
    // entered second.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let ftk = t.library_top(P0, "Flametongue Kavu");
    let giant = t.library_top(P1, "Hill Giant");
    divergent(&mut t, mine, theirs);
    assert!(t.on_battlefield(ftk) && t.on_battlefield(giant));
    // Triggers wait: nothing was put on the stack mid-resolution.
    let from = t.asked().len();
    assert_eq!(t.stack_len(), 0);
    let giant = t.g.current(giant);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.settle();
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(cands.iter().any(|c| c.contains(&Entity::Object(giant))));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // Sequential: Soul Warden entering second sees nothing; entering first, it sees the
    // second creature.
    for (order, gained) in [(["Grizzly Bears", "Soul Warden"], 0), (["Soul Warden", "Grizzly Bears"], 1)] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Hill Giant");
        let b = t.battlefield(P0, "Hill Giant");
        crate::r_s11_common::empty_library(&mut t, P0);
        stack_library(&mut t, P0, &[order[0], "Forest", order[1], "Forest"]);
        divergent(&mut t, a, b);
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Soul Warden").len(), 1);
        assert_eq!(t.life(P0), 20 + gained, "{order:?}");
    }
}

#[test]
fn divergent_transformations_with_no_creature_cards_left() {
    cr!("701.20a", "701.24a");
    ruling!(
        "Divergent Transformations",
        "If the controller of a creature exiled this way has no creature cards left in their library, that player reveals the cards in their library, then shuffles it."
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    crate::r_s11_common::empty_library(&mut t, P1);
    let lands = stack_library(&mut t, P1, &["Forest", "Island", "Swamp"]);
    t.library_top(P0, "Hill Giant");
    divergent(&mut t, mine, theirs);
    t.resolve_all();
    assert_eq!(t.zone(theirs), Zone::Exile);
    for l in lands {
        assert_eq!(t.zone(l), Zone::Library(P1));
    }
    assert_eq!(t.library_size(P1), 3);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}
