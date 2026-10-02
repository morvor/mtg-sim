//! Rulings batch P184 — Taigam, Ojutai Master: "Whenever you cast an instant or sorcery
//! spell from your hand, if Taigam attacked this turn, that spell gains rebound." Rebound
//! (CR 702.88) granted by a resolving triggered ability, and how it then behaves.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s04_common::next_upkeep;
use crate::r_s14_common::{cast_from_hand, triggers_on_stack_now};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0's Taigam attacks P1 unblocked; the game moves on to P0's postcombat main phase.
fn taigam_attacked() -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let taigam = t.battlefield(P0, "Taigam, Ojutai Master");
    attack_with(&mut t, &[(taigam, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.advance_to(P0, Step::PostcombatMain);
    (t, taigam)
}

/// Goes to P0's next upkeep; the number of triggered abilities then on the stack.
fn upkeep_triggers(t: &mut TestGame) -> usize {
    next_upkeep(t, P0);
    triggers_on_stack_now(t)
}

/// Puts the item named `name` (a trigger whose text contains it) last in P0's ordering of
/// simultaneous triggers, so that it resolves first.
fn shock_resolves_first(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    let Decision::Order { items, .. } = d else {
        return None;
    };
    let (mut first, mut others): (Vec<usize>, Vec<usize>) =
        (0..items.len()).partition(|i| items[*i].contains("Shock"));
    others.append(&mut first);
    Some(Answer::Indices(others))
}

#[test]
fn taigams_rebound_triggers_are_ordered_and_ignore_sorcery_timing_but_not_other_restrictions() {
    cr!("702.88a", "603.3b", "608.2g", "601.3");
    ruling!(
        "Taigam, Ojutai Master",
        "At the beginning of your upkeep, all delayed triggered abilities created by rebound effects trigger. You may handle them in any order. A spell you cast this way resolves before you can cast the next one, so one rebounded spell can't target another. Timing permissions based on the card's type (if it's a sorcery) are ignored. Other restrictions, such as \"Cast [this spell] only during combat,\" must be followed."
    );
    supported("Taigam, Ojutai Master");
    supported("Volcanic Hammer");
    supported("Panic");
    let (mut t, _) = taigam_attacked();
    // A sorcery and an instant cast from hand gain rebound and are exiled.
    let hammer = cast_from_hand(&mut t, P0, "Volcanic Hammer", &[Entity::Player(P1)]);
    t.resolve_all();
    let shock = cast_from_hand(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 12);
    assert_eq!(t.zone(hammer), Zone::Exile);
    assert_eq!(t.zone(shock), Zone::Exile);
    // P0 orders the two delayed triggers: Shock's resolves first.
    crate::r_s03_common::respond(&mut t, P0, shock_resolves_first);
    assert_eq!(upkeep_triggers(&mut t), 2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    // Shock is cast; Volcanic Hammer is still in exile, so it can't be targeted.
    assert_eq!(t.zone(shock), Zone::Stack);
    assert_eq!(t.zone(hammer), Zone::Exile);
    t.resolve();
    assert_eq!(t.life(P1), 10);
    // Then the sorcery is cast during the upkeep.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    assert_eq!(t.g.turn.step, Step::Upkeep);
    assert_eq!(t.zone(hammer), Zone::Stack, "sorcery timing is ignored");
    t.resolve_all();
    assert_eq!(t.life(P1), 7);
    // Panic ("Cast this spell only during combat before blockers are declared") cast
    // during combat gains rebound, but can't be cast during the upkeep: it stays exiled.
    let mut t = TestGame::new(2);
    let taigam = t.battlefield(P0, "Taigam, Ojutai Master");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(taigam, Entity::Player(P1))]);
    let panic = cast_from_hand(&mut t, P0, "Panic", &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.zone(panic), Zone::Exile);
    assert_eq!(upkeep_triggers(&mut t), 1);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.zone(panic), Zone::Exile);
}

#[test]
fn taigams_rebound_is_optional_and_never_offered_again() {
    cr!("702.88a", "603.5");
    ruling!(
        "Taigam, Ojutai Master",
        "Casting the card again due to rebound's delayed triggered ability is optional. If you choose not to cast the card, or if you can't (perhaps because there are no legal targets available), the card will stay exiled. You won't get another chance to cast it on a future turn."
    );
    supported("Giant Growth");
    for how in ["decline", "no target"] {
        // P0 casts Giant Growth ("Target creature gets +3/+3 until end of turn") on
        // Taigam; it gains rebound.
        let (mut t, taigam) = taigam_attacked();
        let growth = cast_from_hand(&mut t, P0, "Giant Growth", &[Entity::Object(taigam)]);
        t.resolve_all();
        assert_eq!(t.zone(growth), Zone::Exile);
        if how == "decline" {
            t.answer_yes(P0, false);
        } else {
            // No creature is left to target.
            destroy(&mut t, taigam);
        }
        assert_eq!(upkeep_triggers(&mut t), 1);
        t.resolve_all();
        assert_eq!(t.zone(growth), Zone::Exile, "{how}");
        if how == "decline" {
            assert_eq!(t.pt(taigam), (3, 4));
        }
        assert_eq!(upkeep_triggers(&mut t), 0);
        assert_eq!(t.zone(growth), Zone::Exile);
    }
}

#[test]
fn taigam_killed_in_response_still_gives_rebound() {
    cr!("702.88a", "603.4", "113.7a");
    ruling!(
        "Taigam, Ojutai Master",
        "If Taigam is killed in response to you casting a spell, the ability will still trigger and your spell will gain rebound if Taigam attacked."
    );
    let (mut t, taigam) = taigam_attacked();
    let shock = cast_from_hand(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(triggers_on_stack_now(&t), 1);
    destroy(&mut t, taigam);
    assert!(t.in_graveyard(P0, "Taigam, Ojutai Master"));
    t.resolve_all();
    assert_eq!(t.zone(shock), Zone::Exile);
    assert_eq!(upkeep_triggers(&mut t), 1);
}

#[test]
fn taigam_leaving_stops_new_rebound_but_not_rebound_already_gained() {
    cr!("702.88a", "603.7c", "613.1f");
    ruling!(
        "Taigam, Ojutai Master",
        "If Taigam leaves the battlefield after it attacks, its last ability won't trigger whenever you cast an instant or sorcery spell from your hand this turn. If it leaves the battlefield after the spell has gained rebound, that spell will still have rebound. Once the card is exiled, rebound's delayed triggered ability will trigger at the beginning of your next upkeep even if you no longer control Taigam."
    );
    let (mut t, taigam) = taigam_attacked();
    let first = cast_from_hand(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.settle();
    // The ability resolves: the spell gains rebound. Then Taigam leaves.
    t.resolve();
    assert_eq!(t.zone(first), Zone::Stack);
    destroy(&mut t, taigam);
    t.resolve_all();
    assert_eq!(t.zone(first), Zone::Exile, "the spell kept rebound");
    // Another spell cast this turn doesn't trigger anything.
    let second = cast_from_hand(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(triggers_on_stack_now(&t), 0);
    t.resolve_all();
    assert_eq!(t.zone(second), Zone::Graveyard(P0));
    // The exiled card's delayed trigger still happens without Taigam.
    assert_eq!(upkeep_triggers(&mut t), 1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 11);
}

#[test]
fn taigam_a_spell_that_moves_itself_doesnt_rebound() {
    cr!("702.88a", "608.2n");
    ruling!(
        "Taigam, Ojutai Master",
        "If a spell moves itself into another zone as part of its resolution (as Teferi's Protection, All Suns' Dawn, and Beacon of Unrest do), rebound won't get a chance to apply."
    );
    supported("Beacon of Unrest");
    let (mut t, _) = taigam_attacked();
    let bears = t.graveyard(P1, "Grizzly Bears");
    let beacon = cast_from_hand(&mut t, P0, "Beacon of Unrest", &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(matches!(t.zone(beacon), Zone::Library(_)));
    assert!(!t.in_exile("Beacon of Unrest"));
    assert_eq!(upkeep_triggers(&mut t), 0);
}

#[test]
fn taigam_a_spell_that_doesnt_resolve_doesnt_rebound() {
    cr!("702.88a", "608.2b", "701.6a");
    ruling!(
        "Taigam, Ojutai Master",
        "If a spell with rebound that you cast from your hand doesn't resolve for any reason (either because another spell or ability counters it or because all its targets are illegal as it tries to resolve), none of its effects will happen, including rebound. The spell will be put into its owner's graveyard and you won't get to cast it again on your next turn."
    );
    for how in ["countered", "illegal target"] {
        let (mut t, taigam) = taigam_attacked();
        let bears = t.battlefield(P1, "Grizzly Bears");
        let shock = cast_from_hand(&mut t, P0, "Shock", &[Entity::Object(bears)]);
        // Taigam's ability resolves: the spell has rebound. (Taigam then leaves, so that
        // its first ability doesn't stop the spell from being countered.)
        t.resolve();
        destroy(&mut t, taigam);
        if how == "countered" {
            cast_from_hand(&mut t, P1, "Cancel", &[Entity::Object(shock)]);
        } else {
            cast_from_hand(&mut t, P1, "Unsummon", &[Entity::Object(bears)]);
        }
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Shock"), "{how}");
        assert_eq!(upkeep_triggers(&mut t), 0, "{how}");
    }
}

#[test]
fn taigam_a_card_cast_from_exile_goes_to_the_graveyard() {
    cr!("702.88a", "608.2n");
    ruling!(
        "Taigam, Ojutai Master",
        "If you cast a card from exile, it will go to its owner's graveyard when it resolves, fails to resolve, or is countered. It won't go back to exile."
    );
    for how in ["resolves", "countered"] {
        let (mut t, taigam) = taigam_attacked();
        let shock = cast_from_hand(&mut t, P0, "Shock", &[Entity::Player(P1)]);
        t.resolve_all();
        assert_eq!(t.zone(shock), Zone::Exile);
        assert_eq!(upkeep_triggers(&mut t), 1);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.resolve();
        let recast = t.g.current(shock);
        assert_eq!(t.zone(recast), Zone::Stack);
        if how == "countered" {
            // (Without Taigam, whose first ability would stop it from being countered.)
            destroy(&mut t, taigam);
            cast_from_hand(&mut t, P1, "Cancel", &[Entity::Object(recast)]);
            t.resolve();
            assert!(t.in_graveyard(P0, "Shock"), "countered");
        }
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Shock"), "{how}");
        assert_eq!(upkeep_triggers(&mut t), 0);
    }
}
