//! Rulings batch S14 — rebound (CR 702.88): "If this spell was cast from your hand,
//! instead of putting it into your graveyard as it resolves, exile it and, at the
//! beginning of your next upkeep, you may cast this card from exile without paying its
//! mana cost." A replacement effect (CR 614.1a) of how the spell leaves the stack as it
//! resolves; its controller chooses between it and other replacement effects that would
//! apply (CR 616.1). The card is cast during the delayed trigger's resolution (CR 608.2g).

use crate::r_s01_common::*;
use crate::r_s03_common::run_effect;
use crate::r_s04_common::next_upkeep;
use crate::r_s14_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

/// Casts the rebound card `name` from P0's hand with the given targets and resolves it (and
/// anything above it). Returns the spell.
fn cast_and_resolve(t: &mut TestGame, name: &str, targets: &[Entity]) -> ObjectId {
    let spell = cast_from_hand(t, P0, name, targets);
    t.resolve_all();
    spell
}

/// P0 casts the card `card` (in any zone) by a resolving effect that lets them cast it
/// without paying its mana cost, with the given targets; returns the spell.
fn cast_by_effect(t: &mut TestGame, card: ObjectId, targets: &[Entity]) -> ObjectId {
    for e in targets {
        t.answer_targets(P0, &[*e]);
    }
    run_effect(
        t,
        None,
        P0,
        Effect::CastCard {
            who: PlayerRef::You,
            what: Sel::Target(0),
            free: true,
            optional: false,
        },
        &[Entity::Object(card)],
    );
    let spell = t.g.current(card);
    assert_eq!(t.zone(spell), Zone::Stack, "the card wasn't cast");
    spell
}

/// Goes to P0's next upkeep and returns the number of triggered abilities put on the
/// stack (the rebound triggers).
fn upkeep_triggers(t: &mut TestGame) -> usize {
    next_upkeep(t, P0);
    triggers_on_stack_now(t)
}

#[test]
fn a_copy_of_a_rebound_spell_isnt_exiled_and_doesnt_rebound() {
    cr!("702.88a", "707.10");
    ruling!(
        "Distortion Strike",
        "Rebound will have no effect on copies of spells because you don’t cast them from your hand."
    );
    supported("Distortion Strike");
    // Distortion Strike (a sorcery: target creature gets +1/+0 and can't be blocked this
    // turn) is cast from P0's hand and copied.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let spell = cast_from_hand(&mut t, P0, "Distortion Strike", &[Entity::Object(bears)]);
    run_effect(
        &mut t,
        None,
        P0,
        Effect::CopySpell {
            what: Sel::Target(0),
            count: Value::c(1),
            new_targets: false,
        },
        &[Entity::Object(spell)],
    );
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    // Only the card is exiled; only one delayed trigger.
    assert_eq!(t.zone(spell), Zone::Exile);
    assert_eq!(upkeep_triggers(&mut t), 1);
}

/// Resolves the rebound spell `name` cast from P0's hand while P1 controls `other` (a
/// replacement effect that would exile the spell instead of it going to P0's graveyard),
/// with P0 choosing rebound (`rebound`) or the other effect. Returns whether the card was
/// exiled and whether it rebounded at P0's next upkeep.
fn rebound_or_other(name: &str, other: &str, rebound: bool) -> (bool, bool) {
    let mut t = TestGame::new(2);
    t.battlefield(P1, other);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.answer(
        P0,
        DecisionKind::Replacement,
        Answer::Index(usize::from(!rebound)),
    );
    let spell = cast_and_resolve(&mut t, name, &[Entity::Object(bears)]);
    let offered: Vec<Vec<String>> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseReplacement { options } if *p == P0 => Some(options.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(offered.len(), 1, "P0 chooses which replacement applies");
    assert_eq!(offered[0].len(), 2);
    assert!(offered[0][0].contains("Rebound"));
    assert!(offered[0][1].contains(other));
    let exiled = t.zone(spell) == Zone::Exile;
    let triggers = upkeep_triggers(&mut t);
    (exiled, triggers == 1)
}

#[test]
fn with_leyline_of_the_void_the_caster_chooses_rebound_or_the_leyline() {
    cr!("616.1", "616.1e", "702.88a");
    ruling!(
        "Prey's Vengeance",
        "If a replacement effect would cause a spell with rebound that you cast from your hand to be put somewhere else instead of your graveyard (such as Leyline of the Void might), you choose whether to apply the rebound effect or the other effect as the spell resolves."
    );
    supported("Prey's Vengeance");
    supported("Leyline of the Void");
    // P1's Leyline of the Void: "If a card would be put into an opponent's graveyard from
    // anywhere, exile it instead." Either way Prey's Vengeance is exiled; only rebound
    // gives the delayed trigger.
    assert_eq!(
        rebound_or_other("Prey's Vengeance", "Leyline of the Void", true),
        (true, true)
    );
    assert_eq!(
        rebound_or_other("Prey's Vengeance", "Leyline of the Void", false),
        (true, false)
    );
}

#[test]
fn with_rest_in_peace_the_caster_chooses_rebound_or_rest_in_peace() {
    cr!("616.1", "616.1e", "702.88a");
    ruling!(
        "Artful Maneuver",
        "If a replacement effect (such as the one created by Rest in Peace) would cause a spell with rebound that you cast from your hand to be put somewhere other than into your graveyard as it resolves, you can choose whether to apply the rebound effect or the other effect as the spell resolves."
    );
    supported("Artful Maneuver");
    supported("Rest in Peace");
    assert_eq!(
        rebound_or_other("Artful Maneuver", "Rest in Peace", true),
        (true, true)
    );
    assert_eq!(
        rebound_or_other("Artful Maneuver", "Rest in Peace", false),
        (true, false)
    );
}

/// Orders rebound's delayed triggers so that Distortion Strike's is put on the stack last
/// (and resolves first).
fn strike_resolves_first(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    let Decision::Order { items, .. } = d else {
        return None;
    };
    let (mut strike, mut others): (Vec<usize>, Vec<usize>) =
        (0..items.len()).partition(|i| items[*i].contains("Distortion Strike"));
    others.append(&mut strike);
    Some(Answer::Indices(others))
}

#[test]
fn rebound_triggers_are_ordered_ignore_sorcery_timing_but_not_rule_of_law() {
    cr!("702.88a", "603.3b", "608.2g", "601.3");
    ruling!(
        "Distortion Strike",
        "At the beginning of your upkeep, all delayed triggered abilities created by rebound effects trigger. You may handle them in any order. If you want to cast a card this way, you do so as part of the resolution of its delayed triggered ability. Timing restrictions based on the card’s type (if it’s a sorcery) are ignored. Other restrictions are not (such as the one from Rule of Law)."
    );
    supported("Rule of Law");
    // P0 casts Distortion Strike (a sorcery) and Prey's Vengeance (an instant) from hand;
    // both are exiled. P1 then puts Rule of Law ("Each player can't cast more than one
    // spell each turn") onto the battlefield (or, as a control, doesn't).
    for rule_of_law in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let strike = cast_and_resolve(&mut t, "Distortion Strike", &[Entity::Object(bears)]);
        let vengeance = cast_and_resolve(&mut t, "Prey's Vengeance", &[Entity::Object(bears)]);
        assert_eq!(t.zone(strike), Zone::Exile);
        assert_eq!(t.zone(vengeance), Zone::Exile);
        if rule_of_law {
            t.battlefield(P1, "Rule of Law");
        }
        // Both trigger at P0's upkeep; P0 orders them, Distortion Strike's on top.
        crate::r_s03_common::respond(&mut t, P0, strike_resolves_first);
        let from = t.asked().len();
        assert_eq!(upkeep_triggers(&mut t), 2);
        assert_eq!(
            t.asked()[from..]
                .iter()
                .filter(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. }))
                .count(),
            1
        );
        // Distortion Strike's trigger resolves: the sorcery is cast during the upkeep.
        t.answer_targets(P0, &[Entity::Object(bears)]);
        t.resolve();
        assert_eq!(t.g.turn.step, mtg_engine::turn::Step::Upkeep);
        assert_eq!(t.zone(strike), Zone::Stack, "sorcery timing is ignored");
        assert_eq!(t.zone(vengeance), Zone::Exile);
        t.resolve();
        assert_eq!(t.pt(bears), (3, 2));
        // Then Prey's Vengeance's trigger resolves: Rule of Law stops P0 from casting a
        // second spell this turn (an instant, so it's not a matter of timing). It stays
        // in exile for good.
        t.answer_targets(P0, &[Entity::Object(bears)]);
        t.resolve_all();
        if rule_of_law {
            assert_eq!(t.zone(vengeance), Zone::Exile);
            assert_eq!(t.pt(bears), (3, 2));
            assert_eq!(upkeep_triggers(&mut t), 0);
            assert_eq!(t.zone(vengeance), Zone::Exile);
        } else {
            assert!(t.in_graveyard(P0, "Prey's Vengeance"));
            assert_eq!(t.pt(bears), (5, 4));
        }
    }
}

#[test]
fn rebound_needs_the_spell_to_be_cast_from_its_controllers_hand() {
    cr!("702.88a", "702.85a", "601.2a");
    ruling!(
        "Virulent Swipe",
        "If you cast a spell with rebound from anywhere other than your hand (such as from your graveyard due to Sins of the Past, from your library due to cascade, or from your opponent’s hand due to Sen Triplets), rebound won’t have any effect. If you do cast it from your hand, rebound will work regardless of whether you paid its mana cost (for example, if you cast it from your hand due to Maelstrom Archangel)."
    );
    supported("Virulent Swipe");
    supported("Bloodbraid Elf");
    // From P0's graveyard, by an effect: it goes to the graveyard as it resolves.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let card = t.graveyard(P0, "Virulent Swipe");
    cast_by_effect(&mut t, card, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    assert!(t.in_graveyard(P0, "Virulent Swipe"));
    assert_eq!(upkeep_triggers(&mut t), 0);
    // Cascaded into from P0's library by Bloodbraid Elf.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.library_top(P0, "Virulent Swipe");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    cast_from_hand(&mut t, P0, "Bloodbraid Elf", &[]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    assert!(t.in_graveyard(P0, "Virulent Swipe"));
    assert_eq!(upkeep_triggers(&mut t), 0);
    // From P1's hand, cast by P0: it goes to P1's graveyard.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let card = t.hand(P1, "Virulent Swipe");
    cast_by_effect(&mut t, card, &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Virulent Swipe"));
    assert_eq!(upkeep_triggers(&mut t), 0);
    // From P0's own hand without paying its mana cost: rebound works.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let card = t.hand(P0, "Virulent Swipe");
    let spell = cast_by_effect(&mut t, card, &[Entity::Object(bears)]);
    assert_eq!(
        t.g.obj(spell).stack.as_ref().unwrap().cast.method,
        CastMethod::Free
    );
    t.resolve_all();
    assert_eq!(t.zone(spell), Zone::Exile);
    assert_eq!(upkeep_triggers(&mut t), 1);
}

/// P0 casts `name` from hand targeting a Grizzly Bears; it's countered by Cancel
/// (`countered`), or the Bears leave the battlefield in response. The card ends up in
/// P0's graveyard and doesn't rebound.
fn doesnt_resolve(name: &str, countered: bool) {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let spell = cast_from_hand(&mut t, P0, name, &[Entity::Object(bears)]);
    if countered {
        cast_from_hand(&mut t, P1, "Cancel", &[Entity::Object(spell)]);
    } else {
        cast_from_hand(&mut t, P1, "Unsummon", &[Entity::Object(bears)]);
    }
    t.resolve_all();
    assert!(t.in_graveyard(P0, name));
    assert_eq!(upkeep_triggers(&mut t), 0);
    assert!(t.in_graveyard(P0, name));
}

#[test]
fn a_rebound_spell_that_doesnt_resolve_goes_to_the_graveyard() {
    cr!("702.88a", "608.2b", "701.6a");
    ruling!(
        "Virulent Swipe",
        "If a spell with rebound that you cast from your hand doesn’t resolve for any reason (due being countered by a spell like Cancel, or because all of its targets are illegal), rebound has no effect. The spell is simply put into your graveyard. You won’t get to cast it again next turn."
    );
    ruling!(
        "Artful Maneuver",
        "If a spell with rebound that you cast from your hand doesn’t resolve for any reason (either because another spell or ability counters it or because all its targets are illegal as it tries to resolve), none of its effects will happen, including rebound."
    );
    doesnt_resolve("Virulent Swipe", true);
    doesnt_resolve("Virulent Swipe", false);
    doesnt_resolve("Artful Maneuver", true);
    doesnt_resolve("Artful Maneuver", false);
}

#[test]
fn a_countered_rebound_spell_without_targets_goes_to_the_graveyard() {
    cr!("702.88a", "701.6a");
    ruling!(
        "Blossoming Calm",
        "If a spell with rebound that you cast from your hand doesn't resolve for any reason, including being countered, that spell won't resolve and none of its effects will happen, including rebound."
    );
    supported("Blossoming Calm");
    // Blossoming Calm: you gain hexproof until your next turn and 2 life. Rebound.
    let mut t = TestGame::new(2);
    let spell = cast_from_hand(&mut t, P0, "Blossoming Calm", &[]);
    cast_from_hand(&mut t, P1, "Cancel", &[Entity::Object(spell)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(t.in_graveyard(P0, "Blossoming Calm"));
    assert_eq!(upkeep_triggers(&mut t), 0);
}

/// P0 casts `name` (targeting a Grizzly Bears), it's exiled, and at the next upkeep P0
/// declines to cast it (`decline`), or can't (no creature to target), or the trigger is
/// countered by Stifle (`stifle`). The card stays in exile for good.
fn not_cast_again(name: &str, how: &str) {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let spell = cast_and_resolve(&mut t, name, &[Entity::Object(bears)]);
    assert_eq!(t.zone(spell), Zone::Exile);
    match how {
        "decline" => {
            t.answer_yes(P0, false);
        }
        "no target" => crate::r_s02_common::destroy(&mut t, bears),
        _ => {}
    }
    assert_eq!(upkeep_triggers(&mut t), 1);
    if how == "stifle" {
        let trigger = *t.g.stack.last().unwrap();
        t.lands(P1, "Island", 1);
        let stifle = t.hand(P1, "Stifle");
        t.cast_with(P1, stifle, &[Entity::Object(trigger)]).unwrap();
    }
    t.resolve_all();
    assert_eq!(t.zone(spell), Zone::Exile, "{how}");
    if how != "no target" {
        assert_eq!(t.pt(bears), (2, 2), "{how}");
    }
    assert_eq!(upkeep_triggers(&mut t), 0);
    assert_eq!(t.zone(spell), Zone::Exile);
}

#[test]
fn a_rebound_card_not_cast_from_exile_stays_there() {
    cr!("702.88a", "701.6a");
    ruling!(
        "Prey's Vengeance",
        "If you are unable to cast a card from exile this way, or you choose not to, nothing happens when the delayed triggered ability resolves. The card remains exiled for the rest of the game, and you won’t get another chance to cast the card. The same is true if the ability is countered (due to Stifle, perhaps)."
    );
    ruling!(
        "Artful Maneuver",
        "Casting the card again due to the delayed triggered ability is optional. If you choose not to cast the card, or if you can’t (perhaps because there are no legal targets available), the card will stay exiled. You won’t get another chance to cast it on a future turn."
    );
    supported("Stifle");
    for how in ["decline", "no target", "stifle"] {
        not_cast_again("Prey's Vengeance", how);
        not_cast_again("Artful Maneuver", how);
    }
}

#[test]
fn a_resolving_rebound_spell_is_exiled_directly_from_the_stack() {
    cr!("702.88a", "614.1a", "603.2");
    ruling!(
        "Emerge Unscathed",
        "If you cast a spell with rebound from your hand and it resolves, it isn’t put into your graveyard. Rather, it’s exiled directly from the stack. Effects that care about cards being put into your graveyard won’t do anything."
    );
    supported("Emerge Unscathed");
    supported("Energy Field");
    // P0's Energy Field: "When a card is put into your graveyard from anywhere, sacrifice
    // this enchantment." Emerge Unscathed resolves and is exiled: nothing triggers.
    let mut t = TestGame::new(2);
    let field = t.battlefield(P0, "Energy Field");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let spell = cast_and_resolve(&mut t, "Emerge Unscathed", &[Entity::Object(bears)]);
    assert_eq!(t.zone(spell), Zone::Exile);
    assert!(t.on_battlefield(field));
    // Cast again from exile at the next upkeep, it goes to the graveyard as it resolves:
    // Energy Field's ability triggers.
    t.answer_targets(P0, &[Entity::Object(bears)]);
    assert_eq!(upkeep_triggers(&mut t), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Emerge Unscathed"));
    assert!(!t.on_battlefield(field));
}

/// P0 casts `name` from hand targeting a Grizzly Bears and it's exiled; at the next upkeep
/// it's cast again from exile, and then resolves (`how` = "resolves"), is countered
/// ("countered"), or its target becomes illegal ("illegal target"). It ends up in P0's
/// graveyard, and doesn't rebound again.
fn cast_from_exile_then(name: &str, how: &str) {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let spell = cast_and_resolve(&mut t, name, &[Entity::Object(bears)]);
    assert_eq!(t.zone(spell), Zone::Exile);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    assert_eq!(upkeep_triggers(&mut t), 1);
    t.resolve();
    let recast = t.g.current(spell);
    assert_eq!(t.zone(recast), Zone::Stack);
    match how {
        "countered" => {
            cast_from_hand(&mut t, P1, "Cancel", &[Entity::Object(recast)]);
        }
        "illegal target" => {
            cast_from_hand(&mut t, P1, "Unsummon", &[Entity::Object(bears)]);
        }
        _ => {}
    }
    t.resolve_all();
    assert!(t.in_graveyard(P0, name), "{how}");
    assert_eq!(upkeep_triggers(&mut t), 0);
}

#[test]
fn a_card_cast_from_exile_with_rebound_goes_to_the_graveyard_afterwards() {
    cr!("702.88a", "608.2n", "701.6a", "608.2b");
    ruling!(
        "Prey's Vengeance",
        "If you cast a card from exile this way, it will go to your graveyard when it resolves, fails to resolve, or is countered. It won’t go back to exile."
    );
    ruling!(
        "Artful Maneuver",
        "If you cast a card from exile this way, it will go to its owner’s graveyard when it resolves, fails to resolve, or is countered. It won’t go back to exile."
    );
    for how in ["resolves", "countered", "illegal target"] {
        cast_from_exile_then("Prey's Vengeance", how);
        cast_from_exile_then("Artful Maneuver", how);
    }
}

#[test]
fn cast_this_spell_only_during_combat_is_followed_at_the_upkeep() {
    cr!("702.88a", "608.2g", "601.3");
    ruling!(
        "Taigam's Strike",
        "At the beginning of your upkeep, all delayed triggered abilities created by rebound effects trigger. You may handle them in any order. If you want to cast a card this way, you do so as part of the resolution of its delayed triggered ability. Timing restrictions based on the card’s type (if it’s a sorcery) are ignored. Other restrictions, such as “Cast [this spell] only during combat,” must be followed."
    );
    supported("Taigam's Strike");
    // Taigam's Strike (a sorcery) is cast again during the upkeep.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let strike = cast_and_resolve(&mut t, "Taigam's Strike", &[Entity::Object(bears)]);
    assert_eq!(t.zone(strike), Zone::Exile);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    assert_eq!(upkeep_triggers(&mut t), 1);
    t.resolve();
    assert_eq!(t.zone(t.g.current(strike)), Zone::Stack);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    // A rebound instant that can be cast only during combat (before blockers are
    // declared) can't be cast during the upkeep: it stays exiled.
    let combat_trick = custom_card(
        "Combat Rebound",
        "Instant",
        "{W}",
        None,
        "Cast this spell only during combat before blockers are declared.\nTarget creature gets +2/+2 until end of turn.\nRebound",
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    let card = t.custom(P0, combat_trick, Zone::Hand(P0));
    t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
    let spell = t.cast_with(P0, card, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(spell), Zone::Exile);
    assert_eq!(t.pt(bears), (4, 4));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    assert_eq!(upkeep_triggers(&mut t), 1);
    t.resolve_all();
    assert_eq!(t.zone(spell), Zone::Exile);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn rebound_works_from_hand_without_paying_the_mana_cost() {
    cr!("702.88a", "118.9");
    ruling!(
        "Artful Maneuver",
        "As long as you cast a spell with rebound from your hand, rebound will work regardless of whether you paid its mana cost or an alternative cost you were permitted to pay."
    );
    ruling!(
        "Artful Maneuver",
        "If you cast a spell with rebound from any zone other than your hand (including your opponent’s hand), rebound will have no effect."
    );
    // From P0's hand without paying its mana cost: exiled, and cast again next upkeep.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let card = t.hand(P0, "Artful Maneuver");
    let spell = cast_by_effect(&mut t, card, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.zone(spell), Zone::Exile);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    assert_eq!(upkeep_triggers(&mut t), 1);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    // From P1's hand or P0's graveyard: no rebound.
    for zone in ["opponent's hand", "graveyard"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let card = if zone == "graveyard" {
            t.graveyard(P0, "Artful Maneuver")
        } else {
            t.hand(P1, "Artful Maneuver")
        };
        cast_by_effect(&mut t, card, &[Entity::Object(bears)]);
        t.resolve_all();
        assert_eq!(t.pt(bears), (4, 4));
        assert!(!t.in_exile("Artful Maneuver"), "{zone}");
        assert_eq!(upkeep_triggers(&mut t), 0, "{zone}");
    }
}

#[test]
fn casting_it_again_is_optional_or_prohibited_and_it_then_stays_exiled() {
    cr!("702.88a", "601.3", "608.2n");
    ruling!(
        "Blossoming Calm",
        "Casting the card again due to rebound's delayed triggered ability is optional. If you choose not to cast the card, or if you can't because an effect prohibits it, the card will stay exiled. You won't get another chance to cast it on a future turn. If you do cast the card, it's put into its owner's graveyard as normal once it resolves."
    );
    // Declined: it stays exiled.
    let mut t = TestGame::new(2);
    let calm = cast_and_resolve(&mut t, "Blossoming Calm", &[]);
    assert_eq!(t.zone(calm), Zone::Exile);
    assert_eq!(t.life(P0), 22);
    t.answer_yes(P0, false);
    assert_eq!(upkeep_triggers(&mut t), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(upkeep_triggers(&mut t), 0);
    assert_eq!(t.zone(calm), Zone::Exile);
    // Prohibited: P1's Rule of Law, after P0 cast another spell during the upkeep (the
    // other rebound card, whose trigger resolves first).
    let mut t = TestGame::new(2);
    let calm = cast_and_resolve(&mut t, "Blossoming Calm", &[]);
    let other = cast_and_resolve(&mut t, "Blossoming Calm", &[]);
    t.battlefield(P1, "Rule of Law");
    assert_eq!(upkeep_triggers(&mut t), 2);
    t.resolve_all();
    let exiled: Vec<ObjectId> = [calm, other]
        .into_iter()
        .filter(|c| t.zone(*c) == Zone::Exile)
        .collect();
    assert_eq!(exiled.len(), 1);
    assert_eq!(
        t.graveyard_size(P0),
        1,
        "the one cast went to the graveyard"
    );
    assert_eq!(t.life(P0), 26);
    assert_eq!(upkeep_triggers(&mut t), 0);
    assert_eq!(t.zone(exiled[0]), Zone::Exile);
}

/// The replacement-effect choices P0 was asked since decision `from`.
fn replacement_choices(t: &TestGame, from: usize) -> Vec<Vec<String>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseReplacement { options } if *p == P0 => Some(options.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn buyback_and_rest_in_peace_are_chosen_between_like_rebound() {
    cr!("616.1", "702.27a");
    supported("Whispers of the Muse");
    // Whispers of the Muse ({U}: draw a card; buyback {5}) with P1's Rest in Peace: P0
    // chooses buyback (the card returns to P0's hand) or Rest in Peace (it's exiled).
    for buyback in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Rest in Peace");
        t.lands(P0, "Island", 6);
        let card = t.hand(P0, "Whispers of the Muse");
        let from = t.asked().len();
        t.answer(
            P0,
            DecisionKind::Replacement,
            Answer::Index(usize::from(!buyback)),
        );
        t.cast(P0, card).kicked(true).go();
        t.resolve_all();
        assert_eq!(replacement_choices(&t, from).len(), 1);
        assert_eq!(t.in_hand(P0, "Whispers of the Muse"), buyback);
        assert_eq!(t.in_exile("Whispers of the Muse"), !buyback);
    }
}

#[test]
fn flashback_exiles_the_card_whatever_else_would_apply() {
    cr!("702.34a", "616.1");
    supported("Think Twice");
    // Think Twice cast with flashback while P1 controls Rest in Peace: it's exiled, with
    // nothing to choose ("exile it instead of putting it anywhere else" applies after any
    // other replacement effect anyway).
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Rest in Peace");
    t.lands(P0, "Island", 3);
    let card = t.graveyard(P0, "Think Twice");
    let from = t.asked().len();
    t.cast(P0, card)
        .method(CastMethod::Keyword(
            mtg_engine::keywords::KeywordKind::Flashback,
        ))
        .go();
    t.resolve_all();
    assert!(replacement_choices(&t, from).is_empty());
    assert!(t.in_exile("Think Twice"));
}
