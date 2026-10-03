//! Rulings batch S04 — cycling (CR 702.29): "Cycling [cost]" means "[Cost], Discard this
//! card: Draw a card." Typecycling ("[type]cycling [cost]") searches for a card of that
//! type instead (CR 702.29e) and is cycling too (CR 702.29f). Abilities that trigger
//! "when you cycle this card" are separate from the cycling ability (CR 702.29c).

use crate::r_s01_common::*;
use crate::r_s04_common::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::object::StackKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether the stack object is an activated ability (the cycling ability), a triggered
/// ability, or a spell: "A", "T", or "S".
pub(crate) fn kinds_on_stack(t: &TestGame) -> String {
    t.g.stack
        .iter()
        .map(|id| match t.g.obj(*id).stack.as_deref().map(|si| &si.kind) {
            Some(StackKind::Activated { .. }) => 'A',
            Some(StackKind::Triggered { .. }) => 'T',
            _ => 'S',
        })
        .collect()
}

/// The candidates offered by the last search decision of `p` since decision `from`.
pub(crate) fn search_candidates(t: &TestGame, p: PlayerId, from: usize) -> Vec<Entity> {
    t.asked()[from..]
        .iter()
        .rev()
        .find_map(|(q, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *q == p => Some(candidates.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

/// P0 casts Stifle (or another counterspell for abilities) at the top object of the stack
/// on P1's behalf, and it resolves.
pub(crate) fn counter_top_with(t: &mut TestGame, caster: PlayerId, spell: &str) {
    let top = top_of_stack(t);
    give_mana_for(t, caster, spell);
    let c = t.hand(caster, spell);
    t.cast(caster, c).target(Entity::Object(top)).go();
    t.resolve();
}

/// Checks that the cycling ability of the real card `name` (cycled by P0 for its first
/// cycling cost) is an activated ability: Stifle and Squelch can target it and countering
/// it means no card is drawn; spell-only effects (Cancel, Remove Soul, Faerie Tauntings)
/// don't interact with it.
pub(crate) fn check_cycling_is_an_activated_ability(name: &str) {
    supported(name);
    // Cancel and Remove Soul can't target it; Stifle and Squelch can.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 3);
    for l in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        t.lands(P0, l, 2);
    }
    let card = t.hand(P0, name);
    cycle(&mut t, P0, card, 0).unwrap();
    assert_eq!(kinds_on_stack(&t), "A");
    let ability = Entity::Object(top_of_stack(&t));
    assert!(!spell_targets(&mut t, P1, "Cancel").contains(&ability));
    assert!(!spell_targets(&mut t, P1, "Remove Soul").contains(&ability));
    assert!(spell_targets(&mut t, P1, "Stifle").contains(&ability));
    assert!(spell_targets(&mut t, P1, "Squelch").contains(&ability));
    // Stifled, it doesn't resolve: no card is searched for or drawn, though the card was
    // discarded to pay the cost.
    let hand = t.hand_size(P0);
    let library = t.library_size(P0);
    counter_top_with(&mut t, P1, "Stifle");
    t.resolve_all();
    assert!(t.in_graveyard(P0, name));
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.library_size(P0), library);

    // Faerie Tauntings ("Whenever you cast a spell during an opponent's turn") doesn't
    // trigger when P0 cycles during P1's turn, only when P0 casts a spell.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Faerie Tauntings");
    t.lands(P0, "Wastes", 3);
    for l in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        t.lands(P0, l, 2);
    }
    t.set_step(P1, Step::PrecombatMain);
    let card = t.hand(P0, name);
    cycle(&mut t, P0, card, 0).unwrap();
    t.settle();
    assert_eq!(kinds_on_stack(&t), "A");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(P1).go();
    t.settle();
    assert_eq!(kinds_on_stack(&t), "ST");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn cycling_is_an_activated_ability_not_a_spell() {
    cr!("702.29a", "602.1", "112.1", "113.3b", "701.6a");
    ruling!(
        "Barren Moor",
        "Cycling is an activated ability. Effects that interact with activated abilities (such as Stifle or Rings of Brighthearth) will interact with cycling. Effects that interact with spells (such as Remove Soul or Faerie Tauntings) will not."
    );
    check_cycling_is_an_activated_ability("Barren Moor");
    // Pithing Needle stops activated abilities of the named card, cycling included.
    let mut t = TestGame::new(2);
    t.answer(P1, DecisionKind::Name, Answer::Text("Barren Moor".into()));
    t.enter(P1, "Pithing Needle");
    t.lands(P0, "Swamp", 1);
    let moor = t.hand(P0, "Barren Moor");
    assert!(!can_cycle(&mut t, P0, moor));
    assert!(cycle(&mut t, P0, moor, 0).is_err());
    assert!(t.in_hand(P0, "Barren Moor"));
}

#[test]
fn basic_landcycling_is_an_activated_ability_not_a_spell() {
    cr!("702.29e", "702.29f", "602.1", "701.6a");
    ruling!(
        "Fiery Fall",
        "Basic landcycling is an activated ability. Effects that interact with activated abilities (such as Stifle or Rings of Brighthearth) will interact with basic landcycling. Effects that interact with spells (such as Remove Soul or Faerie Tauntings) will not."
    );
    check_cycling_is_an_activated_ability("Fiery Fall");
}

#[test]
fn the_cycle_trigger_goes_on_the_stack_above_the_cycling_ability() {
    cr!("702.29a", "702.29c", "603.3", "405.5");
    ruling!(
        "Bant Sojourners",
        "If you cycle this card, the cycling ability goes on the stack, then the triggered ability goes on the stack on top of it. The triggered ability will resolve before you draw a card from the cycling ability."
    );
    supported("Bant Sojourners");
    // Bant Sojourners: "When you cycle this card and when this creature dies, you may
    // create a 1/1 white Soldier creature token."
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 2);
    let top = t.library_top(P0, "Grizzly Bears");
    let card = t.hand(P0, "Bant Sojourners");
    cycle(&mut t, P0, card, 0).unwrap();
    t.settle();
    assert_eq!(kinds_on_stack(&t), "AT");
    t.answer_yes(P0, true);
    t.resolve();
    // The token was created; the card hasn't been drawn yet.
    assert_eq!(with_subtype(&t, P0, "Soldier").len(), 1);
    assert_eq!(t.zone(top), mtg_engine::object::Zone::Library(P0));
    t.resolve();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn a_countered_or_fizzled_cycle_trigger_doesnt_stop_the_draw() {
    cr!("702.29a", "702.29c", "608.2b", "701.6a");
    ruling!(
        "Gempalm Incinerator",
        "The cycling ability and the triggered ability are separate. If the triggered ability doesn't resolve (because, for example, it has been countered, or all of its targets have become illegal), the cycling ability will still resolve, and you'll draw a card."
    );
    supported("Gempalm Incinerator");
    // Gempalm Incinerator: "When you cycle this card, you may have it deal X damage to
    // target creature, where X is the number of Goblins on the battlefield."
    // The trigger is countered (Stifle).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raging Goblin");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let top = t.library_top(P0, "Hill Giant");
    let card = t.hand(P0, "Gempalm Incinerator");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    cycle(&mut t, P0, card, 0).unwrap();
    t.settle();
    assert_eq!(kinds_on_stack(&t), "AT");
    counter_top_with(&mut t, P1, "Stifle");
    assert_eq!(kinds_on_stack(&t), "A");
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.zone(top), mtg_engine::object::Zone::Hand(P0));

    // The trigger's target becomes illegal.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raging Goblin");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let top = t.library_top(P0, "Hill Giant");
    let card = t.hand(P0, "Gempalm Incinerator");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    cycle(&mut t, P0, card, 0).unwrap();
    t.settle();
    t.g.destroy(bears, None);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(kinds_on_stack(&t), "A");
    t.resolve();
    assert_eq!(t.zone(top), mtg_engine::object::Zone::Hand(P0));
}

#[test]
fn a_stifled_sojourners_trigger_doesnt_stop_the_draw() {
    cr!("702.29a", "702.29c", "608.2b", "701.6a");
    ruling!(
        "Naya Sojourners",
        "The cycling ability and the triggered ability are separate. If the triggered ability doesn't resolve (due to being countered with Stifle, for example, or if all its targets have become illegal), the cycling ability will still resolve and you'll draw a card."
    );
    supported("Naya Sojourners");
    // Naya Sojourners: "When you cycle this card and when this creature dies, you may put
    // a +1/+1 counter on target creature."
    for stifle in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, "Forest", 1);
        t.lands(P0, "Wastes", 2);
        let top = t.library_top(P0, "Hill Giant");
        let card = t.hand(P0, "Naya Sojourners");
        t.answer_targets(P0, &[Entity::Object(bears)]);
        cycle(&mut t, P0, card, 0).unwrap();
        t.settle();
        assert_eq!(kinds_on_stack(&t), "AT");
        if stifle {
            counter_top_with(&mut t, P1, "Stifle");
        } else {
            t.g.destroy(bears, None);
            t.answer_yes(P0, true);
            t.resolve();
        }
        assert_eq!(kinds_on_stack(&t), "A");
        t.resolve();
        assert_eq!(t.zone(top), mtg_engine::object::Zone::Hand(P0));
        if stifle {
            assert_eq!(t.counters(bears, "+1/+1"), 0);
        }
    }
}

#[test]
fn an_illegal_target_for_primal_boosts_trigger_doesnt_stop_the_draw() {
    cr!("702.29a", "702.29c", "608.2b");
    ruling!(
        "Primal Boost",
        "The cycling ability and the triggered ability are separate. If the triggered ability doesn't resolve (because, for example, it has been countered, or all of its targets have become illegal), the cycling ability will still resolve and you'll draw a card."
    );
    supported("Primal Boost");
    // Primal Boost: "When you cycle this card, you may have target creature get +1/+1
    // until end of turn."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 2);
    let top = t.library_top(P0, "Hill Giant");
    let card = t.hand(P0, "Primal Boost");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    cycle(&mut t, P0, card, 0).unwrap();
    t.settle();
    assert_eq!(kinds_on_stack(&t), "AT");
    // The creature leaves the battlefield: the trigger doesn't resolve.
    t.g.destroy(bears, None);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(kinds_on_stack(&t), "A");
    t.resolve();
    assert_eq!(t.zone(top), mtg_engine::object::Zone::Hand(P0));
    // Stifled instead.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 2);
    let top = t.library_top(P0, "Hill Giant");
    let card = t.hand(P0, "Primal Boost");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    cycle(&mut t, P0, card, 0).unwrap();
    t.settle();
    counter_top_with(&mut t, P1, "Stifle");
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.zone(top), mtg_engine::object::Zone::Hand(P0));
}

#[test]
fn a_sojourners_trigger_is_controlled_by_who_cycled_it_or_last_controlled_it() {
    cr!("702.29c", "603.3a", "603.10a", "603.6c");
    ruling!(
        "Bant Sojourners",
        "The triggered ability acts as a cycle-triggered ability or as a leaves-the-battlefield ability, as appropriate. The player who controls the triggered ability is the player who cycled the Sojourner, or the player who last controlled the Sojourner on the battlefield."
    );
    // Cycled from P1's hand: P1 controls the trigger and gets the Soldier.
    let mut t = TestGame::new(2);
    t.lands(P1, "Plains", 1);
    t.lands(P1, "Wastes", 2);
    let card = t.hand(P1, "Bant Sojourners");
    cycle(&mut t, P1, card, 0).unwrap();
    t.settle();
    assert_eq!(kinds_on_stack(&t), "AT");
    assert_eq!(t.obj(top_of_stack(&t)).controller, P1);
    t.answer_yes(P1, true);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P1, "Soldier").len(), 1);
    assert!(with_subtype(&t, P0, "Soldier").is_empty());

    // P0's Bant Sojourners, controlled by P1 (Act of Treason), dies: its leaves-the-
    // battlefield ability triggers once, controlled by P1.
    let mut t = TestGame::new(2);
    let soj = t.battlefield(P0, "Bant Sojourners");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 3);
    let treason = t.hand(P1, "Act of Treason");
    t.cast(P1, treason).target(soj).go();
    t.resolve_all();
    assert_eq!(t.obj_now(soj).controller, P1);
    t.g.destroy(soj, None);
    t.settle();
    assert!(t.in_graveyard(P0, "Bant Sojourners"));
    assert_eq!(kinds_on_stack(&t), "T");
    assert_eq!(t.obj(top_of_stack(&t)).controller, P1);
    t.answer_yes(P1, true);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P1, "Soldier").len(), 1);
    assert!(with_subtype(&t, P0, "Soldier").is_empty());
    assert!(t.g.stack.is_empty());
}

#[test]
fn a_sojourner_can_be_cycled_with_no_target_for_its_trigger() {
    cr!("702.29a", "702.29c", "603.3d");
    ruling!(
        "Naya Sojourners",
        "You can cycle this card even if there are no targets for the triggered ability. That's because the cycling ability itself has no targets."
    );
    // No creatures on the battlefield: Naya Sojourners' trigger ("put a +1/+1 counter on
    // target creature") has no target and is removed from the stack; the card is drawn.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 2);
    let top = t.library_top(P0, "Hill Giant");
    let card = t.hand(P0, "Naya Sojourners");
    assert!(can_cycle(&mut t, P0, card));
    cycle(&mut t, P0, card, 0).unwrap();
    t.settle();
    assert_eq!(kinds_on_stack(&t), "A");
    t.resolve_all();
    assert_eq!(t.zone(top), mtg_engine::object::Zone::Hand(P0));
}

#[test]
fn a_card_can_be_cycled_with_no_legal_target_for_its_cycle_trigger() {
    cr!("702.29a", "702.29c", "603.3d");
    ruling!(
        "Gempalm Incinerator",
        "You can cycle this card even if there are no legal targets for the triggered ability."
    );
    // No creatures: Gempalm Incinerator's trigger has no legal target.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let top = t.library_top(P0, "Hill Giant");
    let card = t.hand(P0, "Gempalm Incinerator");
    assert!(can_cycle(&mut t, P0, card));
    cycle(&mut t, P0, card, 0).unwrap();
    t.settle();
    assert_eq!(kinds_on_stack(&t), "A");
    t.resolve_all();
    assert_eq!(t.zone(top), mtg_engine::object::Zone::Hand(P0));
    assert!(t.in_graveyard(P0, "Gempalm Incinerator"));
}

#[test]
fn either_the_cycling_ability_or_its_trigger_can_be_countered_alone() {
    cr!("702.29a", "702.29c", "701.6a", "603.3d");
    ruling!(
        "Titanoth Rex",
        "You can cycle a card even if it has a triggered ability from cycling that won't have a legal target. This is because the cycling ability and the triggered ability are separate. This also means that if either ability is countered (with Disallow, for example), the other ability will still resolve."
    );
    supported("Titanoth Rex");
    supported("Disallow");
    // Titanoth Rex: "When you cycle this card, put a trample counter on target creature
    // you control." With no creature, it can still be cycled.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let top = t.library_top(P0, "Hill Giant");
    let card = t.hand(P0, "Titanoth Rex");
    cycle(&mut t, P0, card, 0).unwrap();
    t.settle();
    assert_eq!(kinds_on_stack(&t), "A");
    t.resolve_all();
    assert_eq!(t.zone(top), mtg_engine::object::Zone::Hand(P0));

    // Disallow counters the cycling ability: the trigger still resolves, and no card is
    // drawn.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    let top = t.library_top(P0, "Hill Giant");
    let card = t.hand(P0, "Titanoth Rex");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    cycle(&mut t, P0, card, 0).unwrap();
    t.settle();
    assert_eq!(kinds_on_stack(&t), "AT");
    let cycling = t.g.stack[0];
    give_mana_for(&mut t, P1, "Disallow");
    let disallow = t.hand(P1, "Disallow");
    t.cast(P1, disallow).target(Entity::Object(cycling)).go();
    t.resolve();
    assert_eq!(kinds_on_stack(&t), "T");
    t.resolve_all();
    assert_eq!(t.counters(bears, "trample"), 1);
    assert_eq!(t.zone(top), mtg_engine::object::Zone::Library(P0));

    // Disallow counters the trigger: the cycling ability still resolves.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    let top = t.library_top(P0, "Hill Giant");
    let card = t.hand(P0, "Titanoth Rex");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    cycle(&mut t, P0, card, 0).unwrap();
    t.settle();
    counter_top_with(&mut t, P1, "Disallow");
    assert_eq!(kinds_on_stack(&t), "A");
    t.resolve_all();
    assert_eq!(t.counters(bears, "trample"), 0);
    assert_eq!(t.zone(top), mtg_engine::object::Zone::Hand(P0));
}

#[test]
fn cycle_triggers_of_the_card_and_of_other_permanents_resolve_before_the_draw() {
    cr!("702.29a", "702.29c", "603.3b", "405.5");
    ruling!(
        "Fractured Sanity",
        "Some cards with cycling have an ability that triggers when you cycle them, and some cards have an ability that triggers whenever you cycle any card. These triggered abilities resolve before you draw from the cycling ability."
    );
    supported("Fractured Sanity");
    // Fractured Sanity: "When you cycle this card, each opponent mills four cards."
    // Lightning Rift: "Whenever a player cycles a card, you may pay {1}. If you do, this
    // enchantment deals 2 damage to any target."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lightning Rift");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    let top = t.library_top(P0, "Hill Giant");
    let library = t.library_size(P1);
    let card = t.hand(P0, "Fractured Sanity");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cycle(&mut t, P0, card, 0).unwrap();
    t.settle();
    assert_eq!(kinds_on_stack(&t), "ATT");
    t.answer_yes(P0, true);
    t.resolve();
    t.resolve();
    // Both triggers resolved; the card isn't drawn yet.
    assert_eq!(kinds_on_stack(&t), "A");
    assert_eq!(t.library_size(P1), library - 4);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.zone(top), mtg_engine::object::Zone::Library(P0));
    t.resolve();
    assert_eq!(t.zone(top), mtg_engine::object::Zone::Hand(P0));
}

#[test]
fn cycle_and_discard_triggers_resolve_before_the_draw() {
    cr!("702.29a", "702.29c", "702.29d", "603.3b");
    ruling!(
        "Magmakin Artillerist",
        "Some cards with cycling have an ability that triggers when you cycle them, and some cards have an ability that triggers whenever you cycle any card or discard any card. These triggered abilities resolve before you draw from the cycling ability."
    );
    supported("Magmakin Artillerist");
    // Magmakin Artillerist: "Whenever you discard one or more cards, this creature deals
    // that much damage to each opponent." and "When you cycle this card, it deals 1
    // damage to each opponent."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Magmakin Artillerist");
    t.lands(P0, "Mountain", 2);
    let top = t.library_top(P0, "Hill Giant");
    let card = t.hand(P0, "Magmakin Artillerist");
    cycle(&mut t, P0, card, 0).unwrap();
    t.settle();
    assert_eq!(kinds_on_stack(&t), "ATT");
    t.resolve();
    t.resolve();
    assert_eq!(kinds_on_stack(&t), "A");
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.zone(top), mtg_engine::object::Zone::Library(P0));
    t.resolve();
    assert_eq!(t.zone(top), mtg_engine::object::Zone::Hand(P0));
}

/// Cancel and Saruman's Trickery ("Counter target spell") can't target the cycling
/// ability or the cycle trigger of the real card `name`, cycled by P0 with a creature to
/// target.
fn check_spell_counters_cant_target_cycling(name: &str, x: Option<i64>) {
    supported(name);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ornithopter = t.battlefield(P1, "Ornithopter");
    t.lands(P0, "Wastes", 3);
    for l in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        t.lands(P0, l, 2);
    }
    let card = t.hand(P0, name);
    if let Some(x) = x {
        t.answer(P0, DecisionKind::X, Answer::Number(x));
    }
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Object(ornithopter)]);
    cycle(&mut t, P0, card, 0).unwrap();
    t.settle();
    assert_eq!(kinds_on_stack(&t), "AT");
    for counter in ["Cancel", "Saruman's Trickery"] {
        assert!(spell_targets(&mut t, P1, counter).is_empty(), "{counter}");
    }
    // Both abilities are still there to resolve.
    t.clear_answers();
    t.resolve_all();
    assert!(t.hand_size(P0) >= 1);
}

#[test]
fn cycling_abilities_and_cycle_triggers_arent_spells() {
    cr!("702.29a", "702.29c", "112.1", "115.1a");
    ruling!(
        "Titanoth Rex",
        "Triggered abilities from cycling a card and the cycling ability itself aren't spells. Effects that interact with spells (such as that of Cancel) won't affect them."
    );
    check_spell_counters_cant_target_cycling("Titanoth Rex", None);
    check_spell_counters_cant_target_cycling("Splendor Mare", None);
}

#[test]
fn a_rampaging_war_mammoths_cycling_and_trigger_cant_be_countered_by_spell_counters() {
    cr!("702.29a", "702.29c", "112.1");
    ruling!(
        "Rampaging War Mammoth",
        "Triggered abilities from cycling a card and the cycling ability itself aren't spells. Effects that interact with spells (such as Saruman's Trickery) won't affect them."
    );
    // Rampaging War Mammoth: "Cycling {X}{2}{R}" and "When you cycle this card, destroy up
    // to X target artifacts."
    check_spell_counters_cant_target_cycling("Rampaging War Mammoth", Some(1));
    // With X = 1, its trigger destroyed the Ornithopter.
    let mut t = TestGame::new(2);
    let ornithopter = t.battlefield(P1, "Ornithopter");
    t.lands(P0, "Mountain", 4);
    let card = t.hand(P0, "Rampaging War Mammoth");
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.answer_targets(P0, &[Entity::Object(ornithopter)]);
    cycle(&mut t, P0, card, 0).unwrap();
    t.settle();
    assert_eq!(kinds_on_stack(&t), "AT");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Ornithopter"));
}

/// The typecycling ability of the real card `name` (its first cycling ability) is
/// cycling: Lightning Rift triggers on it, and Stabilizer ("Players can't cycle cards")
/// stops it.
pub(crate) fn check_typecycling_is_cycling(name: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lightning Rift");
    t.lands(P0, "Wastes", 3);
    for l in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        t.lands(P0, l, 2);
    }
    let card = t.hand(P0, name);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cycle(&mut t, P0, card, 0).unwrap();
    t.settle();
    assert_eq!(kinds_on_stack(&t), "AT", "{name}");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "{name}");

    let mut t = TestGame::new(2);
    t.battlefield(P1, "Stabilizer");
    t.lands(P0, "Wastes", 3);
    for l in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        t.lands(P0, l, 2);
    }
    let card = t.hand(P0, name);
    assert!(!can_cycle(&mut t, P0, card), "{name}");
    assert!(cycle(&mut t, P0, card, 0).is_err(), "{name}");
    assert!(t.in_hand(P0, name));
}

#[test]
fn typecycling_is_cycling() {
    cr!("702.29e", "702.29f");
    ruling!(
        "Troll of Khazad-dûm",
        "Typecycling is a form of cycling. Any ability that triggers on a card being cycled also triggers on a card being typecycled. Any ability that stops a cycling ability from being activated also stops a typecycling ability from being activated."
    );
    check_typecycling_is_cycling("Troll of Khazad-dûm");
}

#[test]
fn basic_landcycling_is_cycling() {
    cr!("702.29e", "702.29f");
    ruling!(
        "Fiery Fall",
        "Basic landcycling is a form of cycling. Any ability that triggers on a card being cycled also triggers on a card being basic landcycled. Any ability that stops a cycling ability from being activated also stops a basic landcycling ability from being activated."
    );
    check_typecycling_is_cycling("Fiery Fall");
}

#[test]
fn landcycling_is_cycling() {
    cr!("702.29e", "702.29f");
    ruling!(
        "Valley Rannet",
        "Landcycling is a form of cycling. Any ability that triggers on a card being cycled also triggers on a card being landcycled. Any ability that stops a cycling ability from being activated also stops a landcycling ability from being activated."
    );
    check_typecycling_is_cycling("Valley Rannet");
}

/// P0 typecycles the real card `name` (its first cycling ability) with P1's Cosi's
/// Trickster watching for shuffles, choosing `pick` (or nothing): the candidates offered.
/// Asserts that no card was drawn and that P0 shuffled.
pub(crate) fn typecycle(t: &mut TestGame, name: &str, pick: Option<ObjectId>) -> Vec<Entity> {
    let trickster = t.battlefield(P1, "Cosi's Trickster");
    t.lands(P0, "Wastes", 3);
    for l in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        t.lands(P0, l, 2);
    }
    let top = t.library_top(P0, "Grizzly Bears");
    let card = t.hand(P0, name);
    let hand = t.hand_size(P0);
    cycle(t, P0, card, 0).unwrap();
    let from = t.asked().len();
    t.answer_choose(P0, &pick.map(Entity::Object).into_iter().collect::<Vec<_>>());
    t.resolve();
    let offered = search_candidates(t, P0, from);
    // Nothing was drawn: the top card is still in the library.
    assert_eq!(t.zone(top), mtg_engine::object::Zone::Library(P0));
    assert_eq!(t.hand_size(P0), hand - 1 + pick.map_or(0, |_| 1));
    // P0 shuffled their library: Cosi's Trickster's ability triggered.
    t.answer_yes(P1, true);
    t.resolve_all();
    assert_eq!(t.counters(trickster, "+1/+1"), 1);
    offered
}

#[test]
fn landcycling_searches_for_a_land_card_of_the_type_instead_of_drawing() {
    cr!("702.29e", "701.23a", "701.24a");
    ruling!(
        "Valley Rannet",
        "Unlike the normal cycling ability, landcycling doesn’t allow you to draw a card. Instead, it lets you search your library for a land card of the specified land type, reveal it, put it into your hand, then shuffle your library."
    );
    ruling!(
        "Valley Rannet",
        "A landcycling ability lets you search for any card in your library with the stated land type. It doesn’t have to be a basic land."
    );
    // Valley Rannet: "Mountaincycling {2}, forestcycling {2}".
    let mut t = TestGame::new(2);
    let mountain = t.library_top(P0, "Mountain");
    let stomping = t.library_top(P0, "Stomping Ground");
    let snowy = t.library_top(P0, "Snow-Covered Mountain");
    let forest = t.library_top(P0, "Forest");
    let grave = t.library_top(P0, "Watery Grave");
    let offered = typecycle(&mut t, "Valley Rannet", Some(stomping));
    // Mountaincycling offers the cards with the Mountain type, basic or not.
    for c in [mountain, stomping, snowy] {
        assert!(offered.contains(&Entity::Object(c)));
    }
    for c in [forest, grave] {
        assert!(!offered.contains(&Entity::Object(c)));
    }
    assert!(t.in_hand(P0, "Stomping Ground"));
}

#[test]
fn landcycling_can_find_a_nonbasic_land_or_nothing() {
    cr!("702.29e", "701.23b");
    ruling!(
        "Valley Rannet",
        "You can choose to find any card with the appropriate land type, including nonbasic lands. You can also choose not to find a card, even if there is a land card with the appropriate type in your library."
    );
    let mut t = TestGame::new(2);
    let mountain = t.library_top(P0, "Mountain");
    let stomping = t.library_top(P0, "Stomping Ground");
    let offered = typecycle(&mut t, "Valley Rannet", None);
    assert!(offered.contains(&Entity::Object(mountain)));
    assert!(offered.contains(&Entity::Object(stomping)));
    // Nothing was found; both are still in the library.
    assert_eq!(t.zone(mountain), mtg_engine::object::Zone::Library(P0));
    assert_eq!(t.zone(stomping), mtg_engine::object::Zone::Library(P0));
    assert!(t.in_graveyard(P0, "Valley Rannet"));
}

#[test]
fn basic_landcycling_finds_a_basic_land_of_any_type() {
    cr!("702.29e", "701.23a", "205.4c");
    ruling!(
        "Fiery Fall",
        "Unlike the normal cycling ability, basic landcycling doesn't allow you to draw a card. Instead, it lets you search your library for a basic land card. You don't choose the type of basic land card you'll find until you're performing the search. After you choose a basic land card in your library, you reveal it, put it into your hand, then shuffle your library."
    );
    ruling!(
        "Traumatic Visions",
        "Unlike the normal cycling ability, basic landcycling doesn’t allow you to draw a card. Instead, it lets you search your library for a basic land card. You don’t choose the type of basic land card you’ll find until you’re performing the search. After you choose a basic land card in your library, you reveal it, put it into your hand, then shuffle your library."
    );
    for name in ["Fiery Fall", "Traumatic Visions"] {
        let mut t = TestGame::new(2);
        let plains = t.library_top(P0, "Plains");
        let island = t.library_top(P0, "Island");
        let snowy = t.library_top(P0, "Snow-Covered Swamp");
        let wastes = t.library_top(P0, "Wastes");
        let grave = t.library_top(P0, "Watery Grave");
        let savannah = t.library_top(P0, "Savannah");
        let offered = typecycle(&mut t, name, Some(snowy));
        // Any basic land card: whatever its basic land type (or none, Wastes).
        for c in [plains, island, snowy, wastes] {
            assert!(offered.contains(&Entity::Object(c)), "{name}");
        }
        // Not nonbasic lands, though they have basic land types.
        for c in [grave, savannah] {
            assert!(!offered.contains(&Entity::Object(c)), "{name}");
        }
        assert!(t.in_hand(P0, "Snow-Covered Swamp"));
    }
}

#[test]
fn basic_landcycling_can_find_nothing() {
    cr!("702.29e", "701.23b");
    ruling!(
        "Fiery Fall",
        "You can choose not to find a basic land card, even if there is one in your library."
    );
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    let offered = typecycle(&mut t, "Fiery Fall", None);
    assert!(offered.contains(&Entity::Object(forest)));
    assert_eq!(t.zone(forest), mtg_engine::object::Zone::Library(P0));
}

#[test]
fn a_discard_trigger_doesnt_allow_discarding_cards() {
    cr!("603.2", "701.9a", "514.1");
    ruling!(
        "Faith of the Devoted",
        "An ability that triggers whenever you discard a card doesn't give you permission to discard cards. You'll need another effect that instructs or allows you to discard them."
    );
    supported("Faith of the Devoted");
    // Faith of the Devoted: "Whenever you cycle or discard a card, you may pay {1}. If you
    // do, each opponent loses 2 life and you gain 2 life."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Faith of the Devoted");
    t.lands(P0, "Wastes", 1);
    let cards: Vec<ObjectId> = (0..8).map(|_| t.hand(P0, "Craw Wurm")).collect();
    // No legal action discards a card from P0's hand.
    t.g.turn.priority = Some(P0);
    t.g.recompute();
    for a in t.g.legal_actions(P0) {
        let source = match a {
            Action::Activate { source, .. } => source,
            Action::Cast { card, .. } | Action::PlayLand { card } => card,
            _ => continue,
        };
        assert!(!cards.contains(&source), "{a:?}");
    }
    // The cleanup step's discard to hand size is an instruction to discard: it triggers.
    t.answer_yes(P0, true);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.hand_size(P0), 7);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn a_cycle_or_discard_trigger_triggers_once_for_a_cycled_card() {
    cr!("702.29d", "603.2c");
    ruling!(
        "Horror of the Broken Lands",
        "An ability that triggers whenever you \"cycle or discard\" a card triggers only once if you cycle a card. The ability \"Whenever you discard a card\" is functionally identical to this ability; cycling is mentioned for clarity."
    );
    supported("Horror of the Broken Lands");
    // Horror of the Broken Lands: "Whenever you cycle or discard another card, this
    // creature gets +2/+1 until end of turn."
    let mut t = TestGame::new(2);
    let horror = t.battlefield(P0, "Horror of the Broken Lands");
    t.lands(P0, "Swamp", 1);
    let moor = t.hand(P0, "Barren Moor");
    cycle(&mut t, P0, moor, 0).unwrap();
    t.settle();
    assert_eq!(kinds_on_stack(&t), "AT");
    t.resolve_all();
    assert_eq!(t.pt(horror), (6, 5));
    // Discarding without cycling triggers it just the same, once per card.
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Hill Giant");
    let hand = t.hand_size(P0);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Swamp", 3);
    let rot = t.hand(P1, "Mind Rot");
    t.cast(P1, rot).target(P0).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 2);
    assert_eq!(t.pt(horror), (10, 7));
}

/// P0 activates Rune of Protection: Red's first ability choosing `source` as it resolves;
/// returns the sources P0 was offered.
fn rune_choosing(t: &mut TestGame, rune: ObjectId, source: ObjectId) -> Vec<Entity> {
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Object(source)]);
    t.activate(P0, rune, 0, &[]).unwrap();
    t.resolve();
    t.asked()[from..]
        .iter()
        .find_map(|(p, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if *p == P0 && prompt.contains("source") => Some(candidates.clone()),
            _ => None,
        })
        .expect("no source choice")
}

#[test]
fn a_source_of_damage_can_be_a_spell_or_an_object_an_ability_on_the_stack_refers_to() {
    cr!("609.7a", "615.1a", "615.3");
    ruling!(
        "Rune of Protection: Red",
        "A source of damage is a permanent, a spell on the stack (including one that creates a permanent), or any object referred to by an object on the stack. A source doesn't need to be capable of dealing damage to be a legal choice."
    );
    supported("Rune of Protection: Red");
    // Rune of Protection: Red: "{W}: The next time a red source of your choice would deal
    // damage to you this turn, prevent that damage."
    // A spell on the stack: Lightning Bolt. Fervor, a red enchantment that can't deal
    // damage, is a legal choice too.
    let mut t = TestGame::new(2);
    let rune = t.battlefield(P0, "Rune of Protection: Red");
    let fervor = t.battlefield(P1, "Fervor");
    t.lands(P0, "Plains", 1);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(P0).go();
    let offered = rune_choosing(&mut t, rune, spell);
    assert!(offered.contains(&Entity::Object(spell)));
    assert!(offered.contains(&Entity::Object(fervor)));
    t.resolve_all();
    assert_eq!(t.life(P0), 20);

    // A permanent spell: Ball Lightning. The permanent it becomes deals no combat damage
    // to P0 the first time.
    let mut t = TestGame::new(2);
    let rune = t.battlefield(P0, "Rune of Protection: Red");
    t.lands(P0, "Plains", 1);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 3);
    let ball = t.hand(P1, "Ball Lightning");
    let spell = t.cast(P1, ball).go();
    rune_choosing(&mut t, rune, spell);
    t.resolve_all();
    let ball = t.g.current(ball);
    assert!(t.on_battlefield(ball));
    attack_with(&mut t, &[(ball, Entity::Player(P0))]);
    block_and_finish(&mut t, P0, &[]);
    assert_eq!(t.life(P0), 20);

    // An object referred to by an ability on the stack: Mogg Fanatic, sacrificed to pay
    // for "It deals 1 damage to any target", is in the graveyard.
    let mut t = TestGame::new(2);
    let rune = t.battlefield(P0, "Rune of Protection: Red");
    t.lands(P0, "Plains", 1);
    let fanatic = t.battlefield(P1, "Mogg Fanatic");
    t.activate(P1, fanatic, 0, &[Entity::Player(P0)]).unwrap();
    assert!(t.in_graveyard(P1, "Mogg Fanatic"));
    let offered = rune_choosing(&mut t, rune, fanatic);
    assert!(offered.contains(&Entity::Object(fanatic)));
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn a_cycle_trigger_uses_the_x_of_the_cycling_cost() {
    cr!("107.3e", "702.29c");
    ruling!(
        "Rampaging War Mammoth",
        "You can choose 0 as the value of X in Rampaging War Mammoth's cycling cost. In that case, its triggered ability won't have any targets and won't destroy any artifacts."
    );
    // Rampaging War Mammoth: "Cycling {X}{2}{R}" and "When you cycle this card, destroy up
    // to X target artifacts."
    for x in [0, 2] {
        let mut t = TestGame::new(2);
        let artifacts = [
            t.battlefield(P1, "Ornithopter"),
            t.battlefield(P1, "Sol Ring"),
            t.battlefield(P1, "Mind Stone"),
        ];
        t.lands(P0, "Mountain", 5);
        let card = t.hand(P0, "Rampaging War Mammoth");
        t.answer(P0, DecisionKind::X, Answer::Number(x));
        t.answer_targets(P0, &[Entity::Object(artifacts[0]), Entity::Object(artifacts[1])]);
        cycle(&mut t, P0, card, 0).unwrap();
        assert_eq!(untapped_lands(&t, P0), 5 - 3 - x as usize);
        t.settle();
        t.resolve_all();
        let destroyed = artifacts.iter().filter(|a| !t.on_battlefield(**a)).count();
        assert_eq!(destroyed, x as usize);
        assert!(t.on_battlefield(artifacts[2]));
    }
    // Webstrike Elite: "Cycling {X}{G}{G}" and "When you cycle this card, destroy up to
    // one target artifact or enchantment with mana value X."
    supported("Webstrike Elite");
    let mut t = TestGame::new(2);
    let ornithopter = t.battlefield(P1, "Ornithopter");
    let ring = t.battlefield(P1, "Sol Ring");
    let stone = t.battlefield(P1, "Mind Stone");
    t.lands(P0, "Forest", 4);
    let card = t.hand(P0, "Webstrike Elite");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(stone)]);
    cycle(&mut t, P0, card, 0).unwrap();
    t.settle();
    let offered = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .unwrap_or_default();
    assert_eq!(offered, vec![Entity::Object(stone)]);
    t.resolve_all();
    assert!(!t.on_battlefield(stone));
    assert!(t.on_battlefield(ornithopter) && t.on_battlefield(ring));
}
