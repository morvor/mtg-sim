//! Rulings batch P005 — Taigam, Master Opportunist ("Flurry — Whenever you cast your second
//! spell each turn, copy it, then exile the spell you cast with four time counters on it.
//! If it doesn't have suspend, it gains suspend."): copies of spells (CR 707.10) and
//! suspend (CR 702.62): its abilities, suspended cards, countering its triggers, and
//! casting without paying the mana cost (CR 118.9a, 601.2b).

use crate::r_p006_common::{mark, obj, pl, targets_of, top, triggers_from};
use crate::r_p044_common::kill;
use crate::r_p109_common::{divide, division_of};
use crate::r_s01_common::{give_mana_for, supported};
use crate::r_s02_common::target_candidates;
use crate::r_s04_common::next_upkeep;
use mtg_engine::decision::{Action, Answer, Decision, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const TAIGAM: &str = "Taigam, Master Opportunist";

/// P0 controls Taigam and has cast a first spell this turn (an Ornithopter).
fn taigam_game() -> (TestGame, ObjectId) {
    supported(TAIGAM);
    let mut t = TestGame::new(2);
    let taigam = t.battlefield(P0, TAIGAM);
    let orni = t.hand(P0, "Ornithopter");
    t.cast(P0, orni).go();
    t.resolve_all();
    (t, taigam)
}

/// P0 casts `name` (with mana for it) as the second spell of the turn with these targets;
/// Taigam's trigger is put on the stack. Returns the spell.
fn second_spell(t: &mut TestGame, taigam: ObjectId, name: &str, targets: &[Entity]) -> ObjectId {
    supported(name);
    give_mana_for(t, P0, name);
    let card = t.hand(P0, name);
    let b = t.cast(P0, card);
    let b = if targets.is_empty() {
        b
    } else {
        b.targets(targets)
    };
    let spell = b.go();
    t.settle();
    assert_eq!(triggers_from(t, taigam), 1);
    spell
}

/// Taigam exiles `name` (cast as the second spell, at P1) with four time counters; its
/// copy resolves. Returns the exiled card.
fn suspended_by_taigam(t: &mut TestGame, taigam: ObjectId, name: &str) -> ObjectId {
    let spell = second_spell(t, taigam, name, &[pl(P1)]);
    t.resolve_all();
    let card = t.g.current(spell);
    assert_eq!(t.zone(card), Zone::Exile);
    assert_eq!(t.counters(card, counters::TIME), 4);
    assert!(t.obj_now(card).has_keyword(KeywordKind::Suspend));
    card
}

#[test]
fn taigams_copy_resolves_and_the_original_never_does() {
    cr!("707.10", "608.2c");
    ruling!(
        "Taigam, Master Opportunist",
        "Taigam’s ability and the copy it creates resolve before the spell it’s copying. They resolve even if the original spell is countered before the copy is created. Note that Taigam’s ability exiles the original spell as the ability resolves, which will remove that spell from the stack before it can resolve."
    );
    // Normally: the ability resolves, the copy is put on the stack and the original is
    // exiled at once, before it can resolve; only the copy deals damage.
    let (mut t, taigam) = taigam_game();
    let shock = second_spell(&mut t, taigam, "Shock", &[pl(P1)]);
    t.resolve();
    assert_eq!(t.stack_len(), 1, "the copy");
    assert!(!t.g.stack.contains(&shock));
    assert_eq!(t.zone(shock), Zone::Exile);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // The original is countered (Counterspell) in response: the copy is still created
    // and resolves; the countered card stays in the graveyard.
    let (mut t, taigam) = taigam_game();
    let shock = second_spell(&mut t, taigam, "Shock", &[pl(P1)]);
    give_mana_for(&mut t, P1, "Counterspell");
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(shock).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Shock"));
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "{}", t.dump_log());
    assert!(t.in_graveyard(P0, "Shock"));
    assert!(!t.in_exile("Shock"));
}

#[test]
fn taigams_copy_has_the_originals_targets_modes_and_division() {
    cr!("707.10");
    ruling!(
        "Taigam, Master Opportunist",
        "The copy will have the same targets and modes as the original spell. If the spell divides damage or distributes counters among a number of targets, those choices apply to the copy."
    );
    // Forked Bolt: 2 damage divided between P1 and a Grizzly Bears, 1 each.
    let (mut t, taigam) = taigam_game();
    let bears = t.battlefield(P1, "Grizzly Bears");
    divide(&mut t, P0, &[1, 1]);
    let bolt = second_spell(&mut t, taigam, "Forked Bolt", &[pl(P1), obj(bears)]);
    assert_eq!(division_of(&t, bolt), vec![1, 1]);
    t.resolve();
    let copy = top(&t);
    assert_ne!(copy, bolt);
    assert_eq!(targets_of(&t, copy), vec![pl(P1), obj(bears)]);
    assert_eq!(division_of(&t, copy), vec![1, 1]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.obj_now(bears).damage, 1);
    // Boros Charm, third mode: "Target creature gains double strike until end of turn."
    let (mut t, taigam) = taigam_game();
    let bears = t.battlefield(P0, "Grizzly Bears");
    supported("Boros Charm");
    give_mana_for(&mut t, P0, "Boros Charm");
    let charm = t.hand(P0, "Boros Charm");
    t.cast(P0, charm).modes(&[2]).target(bears).go();
    t.settle();
    assert_eq!(triggers_from(&t, taigam), 1);
    t.resolve();
    assert_eq!(targets_of(&t, top(&t)), vec![obj(bears)]);
    t.resolve_all();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::DoubleStrike));
    assert_eq!(t.life(P1), 20, "not the 4-damage mode");
}

#[test]
fn suspend_is_three_abilities_and_a_card_that_gains_it_uses_two() {
    cr!("702.62a", "702.62b");
    ruling!(
        "Taigam, Master Opportunist",
        "Suspend is a keyword that represents three abilities. The first is a static ability that allows you to exile the card from your hand with the specified number of time counters (the number before the dash) on it by paying its suspend cost (listed after the dash). The second is a triggered ability that removes a time counter from the suspended card at the beginning of each of your upkeeps. The third is a triggered ability that gives you the option to cast the card when the last time counter is removed. Note that if a card is exiled and then “gains suspend,” only the two triggered abilities are relevant."
    );
    supported("Rift Bolt");
    // The first ability: Rift Bolt (suspend 1—{R}) is exiled from hand with one time
    // counter by paying {R}.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let rift = t.hand(P0, "Rift Bolt");
    t.take_action(P0, Action::Special(SpecialAction::Suspend { card: rift }));
    let rift = t.g.current(rift);
    assert_eq!(t.zone(rift), Zone::Exile);
    assert_eq!(t.counters(rift, counters::TIME), 1);
    // The second and third: at the next upkeep the counter is removed and it's cast.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // A card that gained suspend in exile: a time counter comes off each upkeep, and
    // after the fourth it's cast.
    let (mut t, taigam) = taigam_game();
    let card = suspended_by_taigam(&mut t, taigam, "Shock");
    assert_eq!(t.life(P1), 18, "the copy");
    for left in [3, 2, 1] {
        next_upkeep(&mut t, P0);
        t.resolve_all();
        assert_eq!(t.counters(card, counters::TIME), left);
    }
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert!(t.in_graveyard(P0, "Shock"));
}

#[test]
fn a_suspended_card_has_suspend_is_exiled_and_has_a_time_counter() {
    cr!("702.62b", "115.1");
    ruling!(
        "Taigam, Master Opportunist",
        "If an effect refers to a “suspended card,” that means a card that (1) has suspend, (2) is in exile, and (3) has one or more time counters on it."
    );
    supported("Timecrafting");
    let (mut t, taigam) = taigam_game();
    let suspended = suspended_by_taigam(&mut t, taigam, "Shock");
    // Exiled with suspend but no time counter; exiled with time counters but no suspend.
    let no_counter = t.exile(P0, "Rift Bolt");
    let no_suspend = t.exile(P0, "Lightning Bolt");
    t.g.add_counters(obj(no_suspend), counters::TIME, 2, None);
    // A card with suspend in a hand isn't suspended either.
    t.hand(P0, "Rift Bolt");
    t.recompute();
    // Timecrafting: "Remove X time counters from target permanent or suspended card."
    give_mana_for(&mut t, P0, "Timecrafting");
    t.lands(P0, "Mountain", 1);
    let tc = t.hand(P0, "Timecrafting");
    let from = mark(&t);
    t.cast(P0, tc).modes(&[0]).x(1).target(suspended).go();
    let cards: Vec<Entity> = target_candidates(&t, P0, from)
        .concat()
        .into_iter()
        .filter(|e| matches!(e, Entity::Object(o) if t.g.obj(*o).zone != Zone::Battlefield))
        .collect();
    assert_eq!(cards, vec![obj(suspended)]);
    assert!(!cards.contains(&obj(no_counter)) && !cards.contains(&obj(no_suspend)));
    t.resolve_all();
    assert_eq!(t.counters(suspended, counters::TIME), 3);
}

#[test]
fn removing_the_last_time_counter_any_way_triggers_the_cast() {
    cr!("702.62a", "603.2");
    ruling!(
        "Taigam, Master Opportunist",
        "When the last time counter is removed, the second triggered ability of suspend (the one that lets you cast the card) triggers. It doesn’t matter why the last time counter was removed or what effect removed it."
    );
    supported("Timecrafting");
    let (mut t, taigam) = taigam_game();
    let card = suspended_by_taigam(&mut t, taigam, "Shock");
    // Timecrafting for X = 4, on P0's turn, removes all four.
    give_mana_for(&mut t, P0, "Timecrafting");
    t.lands(P0, "Mountain", 4);
    let tc = t.hand(P0, "Timecrafting");
    t.cast(P0, tc).modes(&[0]).x(4).target(card).go();
    t.resolve();
    assert_eq!(t.counters(card, counters::TIME), 0);
    assert_eq!(triggers_from(&t, card), 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

/// The suspend trigger on the stack is countered by P1's Stifle.
fn stifle_top(t: &mut TestGame) {
    supported("Stifle");
    give_mana_for(t, P1, "Stifle");
    let stifle = t.hand(P1, "Stifle");
    let ability = top(t);
    t.cast(P1, stifle).target(ability).go();
    t.resolve();
    assert!(!t.g.stack.contains(&ability));
}

#[test]
fn countering_the_upkeep_trigger_removes_no_counter() {
    cr!("702.62a", "701.6a");
    ruling!(
        "Taigam, Master Opportunist",
        "If the first triggered ability of suspend (the one that removes time counters) is countered, no time counter is removed. The ability will trigger again at the beginning of the card’s owner’s next upkeep."
    );
    let (mut t, taigam) = taigam_game();
    let card = suspended_by_taigam(&mut t, taigam, "Shock");
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_from(&t, card), 1);
    stifle_top(&mut t);
    t.resolve_all();
    assert_eq!(t.counters(card, counters::TIME), 4);
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_from(&t, card), 1);
    t.resolve_all();
    assert_eq!(t.counters(card, counters::TIME), 3);
}

#[test]
fn countering_the_cast_trigger_leaves_it_exiled_and_not_suspended() {
    cr!("702.62a", "702.62b", "701.6a");
    ruling!(
        "Taigam, Master Opportunist",
        "If the second triggered ability is countered, the card can’t be cast. It remains exiled with no time counters on it, and it’s no longer suspended."
    );
    let (mut t, taigam) = taigam_game();
    let card = suspended_by_taigam(&mut t, taigam, "Shock");
    t.g.remove_counters(Entity::Object(card), counters::TIME, 3);
    t.recompute();
    next_upkeep(&mut t, P0);
    t.resolve();
    assert_eq!(t.counters(card, counters::TIME), 0);
    assert_eq!(triggers_from(&t, card), 1, "the cast trigger");
    stifle_top(&mut t);
    let from = mark(&t);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(!t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. })));
    assert_eq!(t.zone(card), Zone::Exile);
    assert_eq!(t.counters(card, counters::TIME), 0);
    assert_eq!(t.life(P1), 18);
    // No longer suspended: nothing triggers at the next upkeep.
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_from(&t, card), 0);
    assert_eq!(t.zone(card), Zone::Exile);
}

/// P0's upkeep with `card` (exiled by Taigam) about to lose its last time counter: the
/// counters are taken down to one, and the upkeep trigger resolves, so the cast trigger
/// is on the stack.
fn last_counter_removed(t: &mut TestGame, card: ObjectId) {
    t.g.remove_counters(Entity::Object(card), counters::TIME, 3);
    t.recompute();
    next_upkeep(t, P0);
    t.resolve();
    assert_eq!(triggers_from(t, card), 1);
}

#[test]
fn casting_without_paying_the_mana_cost_allows_additional_costs_only() {
    cr!("118.9a", "601.2b", "601.2f", "702.62d");
    ruling!(
        "Taigam, Master Opportunist",
        "If you cast a card “without paying its mana cost,” such as with suspend, you can’t choose to cast it for any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, you must pay those if you want to cast the card."
    );
    // An optional additional cost may be paid: Burst Lightning kicked ({4}) deals 4.
    let (mut t, taigam) = taigam_game();
    let card = suspended_by_taigam(&mut t, taigam, "Burst Lightning");
    assert_eq!(t.life(P1), 18, "the copy");
    last_counter_removed(&mut t, card);
    t.lands(P0, "Wastes", 4);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    // A mandatory additional cost must be paid: Collateral Damage ("As an additional cost
    // to cast this spell, sacrifice a creature.") can't be cast without a creature...
    let (mut t, taigam) = taigam_game();
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    let card = suspended_by_taigam(&mut t, taigam, "Collateral Damage");
    assert_eq!(t.life(P1), 17, "the copy");
    // P0's other creatures (Taigam and the Ornithopter) are gone too.
    for id in t.g.battlefield.clone() {
        if t.g.obj(id).is(CardType::Creature) {
            kill(&mut t, id);
        }
    }
    last_counter_removed(&mut t, card);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Exile);
    assert_eq!(t.life(P1), 17);
    // ...and with one, it's sacrificed.
    let (mut t, taigam) = taigam_game();
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    let card = suspended_by_taigam(&mut t, taigam, "Collateral Damage");
    last_counter_removed(&mut t, card);
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_choose(P0, &[obj(giant)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.life(P1), 14);
    // No alternative cost: Boon Satyr (bestow {3}{G}{G}) is cast as a creature spell, not
    // bestowed, even with a creature to enchant and mana for its bestow cost.
    let (mut t, taigam) = taigam_game();
    let spell = second_spell(&mut t, taigam, "Boon Satyr", &[]);
    t.resolve_all();
    let card = t.g.current(spell);
    assert_eq!(t.zone(card), Zone::Exile);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 5);
    last_counter_removed(&mut t, card);
    let from = mark(&t);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve_all();
    let satyrs: Vec<ObjectId> = t
        .named_on_battlefield("Boon Satyr")
        .into_iter()
        .filter(|id| !t.g.obj(*id).is_token())
        .collect();
    assert_eq!(satyrs.len(), 1);
    assert!(t.g.obj(satyrs[0]).is(CardType::Creature));
    assert_eq!(t.g.obj(satyrs[0]).attached_to, None);
    assert_eq!(t.pt(bears), (2, 2));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseCastingMethod { .. })));
}
