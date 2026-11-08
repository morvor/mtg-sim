//! Rulings batch P071 — casting cards from a graveyard: Abandoned Sarcophagus, Harness
//! the Storm, Kess, Dissident Mage, The Dawning Archaic, Efreet Flamepainter, Sins of the
//! Past and Maestros Ascendancy. Permissions to cast don't change timing or costs (CR
//! 601.3, 601.2f–h); a cast during a resolution ignores the card's type's timing (CR
//! 608.2g); "without paying its mana cost" allows no alternative cost and makes X 0 (CR
//! 118.9a, 107.3b).

use crate::r_p071_common::*;
use mtg_engine::decision::{Action, Answer};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// The methods `p` could cast `card` (followed across zone changes) with right now.
fn methods(t: &mut TestGame, p: PlayerId, card: ObjectId) -> Vec<CastMethod> {
    crate::r_p068_common::methods(t, p, card)
}

/// Whether `p` could activate an ability of `card` (followed across zone changes) now.
fn activatable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    let c = t.g.current(card);
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Activate { source, .. } if *source == c))
}

// ---------------------------------------------------------------------------
// Abandoned Sarcophagus
// ---------------------------------------------------------------------------

#[test]
fn abandoned_sarcophagus_tokens_and_permanents_without_cycling_go_to_the_graveyard() {
    cr!("614.1a", "111.7", "702.29b");
    ruling!(
        "Abandoned Sarcophagus",
        "A token permanent with cycling will go to your graveyard before ceasing to exist. It won't be exiled. Similarly, a nontoken permanent that lost cycling while it was on the battlefield will also go to your graveyard."
    );
    // A token copy of Desert Cerodon (cycling) dies: Blood Artist sees it die.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Abandoned Sarcophagus");
    t.battlefield(P0, "Blood Artist");
    let cerodon = t.hand(P0, "Desert Cerodon");
    let token = crate::r_s17_common::token_copy(&mut t, P0, cerodon)[0];
    assert!(t.g.obj(token).is_token());
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.destroy(token, None);
    t.resolve_all();
    assert_eq!(t.life(P1), 19, "{}", t.dump_log());
    // Desert Cerodon under Humility (no abilities) dies: put into the graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Abandoned Sarcophagus");
    t.battlefield(P0, "Humility");
    let cerodon = t.battlefield(P0, "Desert Cerodon");
    assert!(!t.obj_now(cerodon).has_keyword(KeywordKind::Cycling));
    t.g.destroy(cerodon, None);
    t.settle();
    assert!(t.in_graveyard(P0, "Desert Cerodon"), "{}", t.dump_log());
    // Without Humility: exiled.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Abandoned Sarcophagus");
    let cerodon = t.battlefield(P0, "Desert Cerodon");
    t.g.destroy(cerodon, None);
    t.settle();
    assert!(t.in_exile("Desert Cerodon"));
}

#[test]
fn abandoned_sarcophagus_doesnt_let_you_cycle_from_the_graveyard() {
    cr!("702.29a", "601.3");
    ruling!(
        "Abandoned Sarcophagus",
        "Abandoned Sarcophagus doesn't grant you permission to do anything with those cards except cast them. For example, you can't cycle nonland cards with cycling from your graveyard."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Abandoned Sarcophagus");
    let cerodon = t.graveyard(P0, "Desert Cerodon");
    t.lands(P0, "Mountain", 6);
    assert!(castable(&mut t, P0, cerodon));
    assert!(!activatable(&mut t, P0, cerodon));
    let r = t.activate(P0, cerodon, 0, &[]);
    assert!(r.is_err());
    assert_eq!(t.zone(cerodon), Zone::Graveyard(P0));
}

#[test]
fn abandoned_sarcophagus_casts_typecycling_cards_and_exiles_them_unless_typecycled() {
    cr!("702.29e", "702.29f", "614.1a");
    ruling!(
        "Abandoned Sarcophagus",
        "Certain older cards have variants of cycling, such as basic landcycling or Wizardcycling. Abandoned Sarcophagus's effect lets you cast these cards and exiles them if they weren't discarded for their cycling variant's ability."
    );
    // Twisted Abomination (swampcycling {2}) is cast from the graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Abandoned Sarcophagus");
    let ta = t.graveyard(P0, "Twisted Abomination");
    t.lands(P0, "Swamp", 6);
    assert!(castable(&mut t, P0, ta));
    t.cast(P0, ta).go();
    t.resolve_all();
    assert!(t.on_battlefield(ta), "{}", t.dump_log());
    // Destroyed: exiled.
    t.g.destroy(t.g.current(ta), None);
    t.settle();
    assert!(t.in_exile("Twisted Abomination"));
    // Swampcycled: to the graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Abandoned Sarcophagus");
    t.library_top(P0, "Swamp");
    let ta = t.hand(P0, "Twisted Abomination");
    t.lands(P0, "Swamp", 2);
    t.activate(P0, ta, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Twisted Abomination"), "{}", t.dump_log());
    assert!(t.in_hand(P0, "Swamp"));
    // Discarded otherwise (Mind Rot): exiled.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Abandoned Sarcophagus");
    t.hand(P0, "Twisted Abomination");
    t.lands(P0, "Swamp", 3);
    let rot = t.hand(P0, "Mind Rot");
    t.cast(P0, rot).target(P0).go();
    t.resolve_all();
    assert!(t.in_exile("Twisted Abomination"), "{}", t.dump_log());
}

#[test]
fn abandoned_sarcophagus_cycled_instant_can_be_cast_before_the_cycling_ability_resolves() {
    cr!("117.3c", "405.5", "307.1");
    ruling!(
        "Abandoned Sarcophagus",
        "If you cycle an instant card, you may cast it from your graveyard right away before any player may take any other action. If you do, that spell will resolve before the cycling ability. If you cycle a noninstant card without flash, you can't cast it until after the cycling ability has resolved."
    );
    // Hieroglyphic Illumination (instant; cycling {U}).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Abandoned Sarcophagus");
    let hi = t.hand(P0, "Hieroglyphic Illumination");
    mana(&mut t, P0, ManaType::U, 1);
    t.activate(P0, hi, 0, &[]).unwrap();
    assert_eq!(t.zone(hi), Zone::Graveyard(P0));
    assert_eq!(t.stack_len(), 1);
    // P0 holds priority: castable now.
    mana(&mut t, P0, ManaType::U, 4);
    assert!(castable(&mut t, P0, hi));
    let hand = t.hand_size(P0);
    t.cast(P0, t.g.current(hi)).go();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    // The spell resolved first: two cards drawn, the cycling ability still on the stack.
    assert_eq!(t.hand_size(P0), hand + 2, "{}", t.dump_log());
    assert_eq!(t.stack_len(), 1);
    assert!(t.in_exile("Hieroglyphic Illumination"));
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 3);
    // Desert Cerodon (a creature card): not while the cycling ability is on the stack;
    // after it resolves, in P0's main phase.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Abandoned Sarcophagus");
    let dc = t.hand(P0, "Desert Cerodon");
    mana(&mut t, P0, ManaType::R, 2);
    t.activate(P0, dc, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 1);
    mana(&mut t, P0, ManaType::R, 6);
    assert!(!castable(&mut t, P0, dc));
    t.resolve();
    assert!(castable(&mut t, P0, dc));
}

// ---------------------------------------------------------------------------
// Harness the Storm
// ---------------------------------------------------------------------------

/// P0 controls Harness the Storm and casts `name` from hand (at `targets`), with another
/// `name` in the graveyard. Returns (the spell from hand, the card in the graveyard).
fn harness(t: &mut TestGame, name: &str, targets: &[Entity]) -> (ObjectId, ObjectId) {
    t.battlefield(P0, "Harness the Storm");
    let gy = t.graveyard(P0, name);
    let hand = t.hand(P0, name);
    let spell = t.cast(P0, hand).targets(targets).go();
    t.answer_targets(P0, &[o(gy)]);
    t.settle();
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    (spell, gy)
}

#[test]
fn harness_the_storm_allows_alternative_and_additional_costs_and_requires_mandatory_ones() {
    cr!("601.2b", "601.2f", "702.33a", "702.113a");
    ruling!(
        "Harness the Storm",
        "Because you’re paying the spell’s costs, you can pay alternative costs, such as awaken costs and surge costs from the Battle for Zendikar block. You can also pay additional costs, such as kicker costs. If the card has a mandatory additional cost, like Lightning Axe, you must pay it to cast the card."
    );
    // Kicked Burst Lightning from the graveyard.
    let mut t = TestGame::new(2);
    mana(&mut t, P0, ManaType::R, 1);
    harness(&mut t, "Burst Lightning", &[Entity::Player(P1)]);
    mana(&mut t, P0, ManaType::R, 5);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 2 - 4, "{}", t.dump_log());
    assert_eq!(t.g.players[0].mana_pool.total(), 0);
    // Lightning Axe from the graveyard: a card is discarded.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Hill Giant");
    let fodder = t.hand(P0, "Grizzly Bears");
    let fodder2 = t.hand(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::R, 1);
    t.answer_choose(P0, &[o(fodder)]);
    harness(&mut t, "Lightning Axe", &[o(bears)]);
    let giant2 = t.battlefield(P1, "Hill Giant");
    mana(&mut t, P0, ManaType::R, 1);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(fodder2)]);
    t.answer_targets(P0, &[o(giant2)]);
    t.resolve_all();
    assert_eq!(t.zone(fodder2), Zone::Graveyard(P0), "{}", t.dump_log());
    assert!(!t.on_battlefield(giant2));
    assert!(!t.on_battlefield(bears));
    // Coastal Discovery from the graveyard for its awaken cost ({5}{U}).
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Island");
    mana(&mut t, P0, ManaType::U, 4);
    harness(&mut t, "Coastal Discovery", &[]);
    mana(&mut t, P0, ManaType::U, 6);
    t.answer_yes(P0, true);
    t.answer(
        P0,
        DecisionKind::Option,
        Answer::Index(1),
    );
    t.answer_targets(P0, &[o(land)]);
    t.resolve_all();
    assert_eq!(t.counters(land, "+1/+1"), 4, "{}", t.dump_log());
    assert!(t.obj_now(land).is(CardType::Creature));
}

#[test]
fn harness_the_storm_card_can_still_be_cast_if_the_hand_spell_is_countered() {
    cr!("603.3d", "608.2b");
    ruling!(
        "Harness the Storm",
        "If the spell you cast from your hand is countered in response to Harness the Storm’s triggered ability, you can still cast the target card while the ability is resolving."
    );
    let mut t = TestGame::new(2);
    mana(&mut t, P0, ManaType::R, 1);
    let (spell, _gy) = harness(&mut t, "Lightning Bolt", &[Entity::Player(P1)]);
    let cs = t.hand(P1, "Counterspell");
    mana(&mut t, P1, ManaType::U, 2);
    t.g.turn.priority = Some(P1);
    t.cast(P1, cs).target(spell).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    mana(&mut t, P0, ManaType::R, 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "{}", t.dump_log());
}

#[test]
fn harness_the_storm_graveyard_spell_can_target_the_hand_spell_and_resolves_first() {
    cr!("608.2g", "405.5", "608.2n");
    ruling!(
        "Harness the Storm",
        "The spell you cast from your graveyard can target the spell you cast from your hand."
    );
    ruling!(
        "Harness the Storm",
        "The spell you cast from your graveyard resolves before the spell you cast from your hand. It’s put into your graveyard as it resolves."
    );
    // Unsubstantiate from the graveyard returns the Unsubstantiate cast from hand.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    mana(&mut t, P0, ManaType::U, 2);
    let (spell, gy) = harness(&mut t, "Unsubstantiate", &[o(bears)]);
    mana(&mut t, P0, ManaType::U, 2);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[o(spell)]);
    t.resolve();
    // The ability resolved; the graveyard spell is on top of the hand spell.
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    t.resolve();
    assert_eq!(t.zone(spell), Zone::Hand(P0), "{}", t.dump_log());
    assert_eq!(t.zone(gy), Zone::Graveyard(P0));
    assert!(t.g.stack.is_empty());
    assert!(t.on_battlefield(bears));
    // Burst Lightning: the graveyard one resolves first and goes back to the graveyard.
    let mut t = TestGame::new(2);
    mana(&mut t, P0, ManaType::R, 1);
    let (spell, gy) = harness(&mut t, "Burst Lightning", &[Entity::Player(P1)]);
    mana(&mut t, P0, ManaType::R, 1);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.zone(gy), Zone::Graveyard(P0), "{}", t.dump_log());
    assert_eq!(t.zone(spell), Zone::Stack);
    t.resolve();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn harness_the_storm_card_is_cast_on_resolution_or_not_at_all() {
    cr!("608.2g", "601.3");
    ruling!(
        "Harness the Storm",
        "You choose whether to cast the targeted card as the ability resolves. You can’t wait to cast it later, and there’s no way for your opponents to set up a situation where you have to cast the card but don’t want to."
    );
    let mut t = TestGame::new(2);
    mana(&mut t, P0, ManaType::R, 1);
    let (_spell, gy) = harness(&mut t, "Lightning Bolt", &[Entity::Player(P1)]);
    mana(&mut t, P0, ManaType::R, 1);
    let from = t.asked().len();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.asked()[from..].iter().any(|(p, d)| *p == P0
        && matches!(d, mtg_engine::decision::Decision::YesNo { .. })));
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.zone(gy), Zone::Graveyard(P0));
    mana(&mut t, P0, ManaType::R, 1);
    assert!(!castable(&mut t, P0, gy), "can't cast it later");
}

#[test]
fn harness_the_storm_targets_as_the_trigger_is_put_on_the_stack() {
    cr!("603.3d", "115.1");
    ruling!(
        "Harness the Storm",
        "You must choose a target for Harness the Storm’s ability (if one exists) immediately after you’ve cast an instant or sorcery spell. There’s no way for that spell to be countered and then targeted by Harness the Storm once it’s in the graveyard."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Harness the Storm");
    let bolt = t.hand(P0, "Lightning Bolt");
    mana(&mut t, P0, ManaType::R, 1);
    let spell = t.cast(P0, bolt).target(P1).go();
    t.settle();
    // No Lightning Bolt in the graveyard: no trigger (no legal target).
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    let cs = t.hand(P1, "Counterspell");
    mana(&mut t, P1, ManaType::U, 2);
    t.g.turn.priority = Some(P1);
    t.cast(P1, cs).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert!(t.g.stack.is_empty());
    assert_eq!(t.life(P1), 20);
}

// ---------------------------------------------------------------------------
// Kess, Dissident Mage
// ---------------------------------------------------------------------------

/// The method that casts `card` from P0's graveyard with Kess's permission (not
/// flashback).
fn kess_method(t: &mut TestGame, card: ObjectId) -> CastMethod {
    methods(t, P0, card)
        .into_iter()
        .find(|m| *m != CastMethod::Keyword(KeywordKind::Flashback))
        .expect("Kess's permission")
}

#[test]
fn kess_sorcery_resolving_in_your_main_phase_can_be_cast_again_before_anyone_acts() {
    cr!("117.3b", "307.1");
    ruling!(
        "Kess, Dissident Mage",
        "If a sorcery card is put into your graveyard during your main phase and the stack is empty, you have a chance to cast it before any player may attempt to remove that card from your graveyard."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kess, Dissident Mage");
    let div = t.hand(P0, "Divination");
    mana(&mut t, P0, ManaType::U, 3);
    t.cast(P0, div).go();
    // Both players pass; Divination resolves; the next player to get priority is P0.
    let ok = t.g.run_until(1_000, |g| {
        g.stack.is_empty() && g.turn.priority.is_some()
    });
    assert!(ok);
    assert_eq!(t.g.turn.priority, Some(P0));
    assert_eq!(t.zone(div), Zone::Graveyard(P0));
    mana(&mut t, P0, ManaType::U, 3);
    assert!(castable(&mut t, P0, div));
}

#[test]
fn kess_spell_returned_by_buyback_is_a_new_object_not_exiled_later() {
    cr!("400.7", "702.27a");
    ruling!(
        "Kess, Dissident Mage",
        "If a spell cast this way is put into another zone after it resolves (perhaps because it had buyback), it's considered a new object. If that card later moves to your graveyard, it won't be exiled."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kess, Dissident Mage");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let cap = t.graveyard(P0, "Capsize");
    mana(&mut t, P0, ManaType::U, 6);
    let m = kess_method(&mut t, cap);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, cap).method(m).target(bears).go();
    t.resolve_all();
    assert_eq!(t.zone(cap), Zone::Hand(P0), "{}", t.dump_log());
    assert_eq!(t.zone(bears), Zone::Hand(P1));
    // Cast again from hand without buyback: to the graveyard.
    let b2 = t.battlefield(P1, "Grizzly Bears");
    mana(&mut t, P0, ManaType::U, 3);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.cast(P0, t.g.current(cap)).target(b2).go();
    t.resolve_all();
    assert_eq!(t.zone(cap), Zone::Graveyard(P0), "{}", t.dump_log());
}

#[test]
fn kess_isnt_used_up_by_flashback_and_a_new_kess_gives_a_new_cast() {
    cr!("702.34a", "400.7");
    ruling!(
        "Kess, Dissident Mage",
        "If you cast a card from your graveyard using another permission (such as that of flashback), Kess's effect doesn't apply. You can cast another instant or sorcery card from your graveyard."
    );
    ruling!(
        "Kess, Dissident Mage",
        "If you cast one instant or sorcery card from your graveyard and then have a new Kess come under your control in the same turn, you may cast another instant or sorcery card from your graveyard that turn."
    );
    let mut t = TestGame::new(2);
    let kess = t.battlefield(P0, "Kess, Dissident Mage");
    let tt = t.graveyard(P0, "Think Twice");
    let div = t.graveyard(P0, "Divination");
    let opt = t.graveyard(P0, "Opt");
    // Think Twice by flashback ({2}{U}).
    mana(&mut t, P0, ManaType::U, 3);
    t.cast(P0, tt)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .go();
    t.resolve_all();
    assert!(t.in_exile("Think Twice"));
    // Divination with Kess.
    mana(&mut t, P0, ManaType::U, 3);
    assert!(castable(&mut t, P0, div), "flashback didn't use Kess");
    let m = kess_method(&mut t, div);
    t.cast(P0, div).method(m).go();
    t.resolve_all();
    assert!(t.in_exile("Divination"));
    mana(&mut t, P0, ManaType::U, 1);
    assert!(!castable(&mut t, P0, opt), "once each turn");
    // A new Kess: Opt.
    t.g.destroy(kess, None);
    t.settle();
    t.battlefield(P0, "Kess, Dissident Mage");
    assert!(castable(&mut t, P0, opt), "{}", t.dump_log());
}

#[test]
fn kess_spell_is_exiled_even_if_kess_changes_control_after_its_cast() {
    cr!("601.2i", "614.1a");
    ruling!(
        "Kess, Dissident Mage",
        "Once you begin to cast the card, losing control of Kess won't affect the spell. You can finish casting it as normal and it will be exiled if it would be put into your graveyard."
    );
    let mut t = TestGame::new(2);
    let kess = t.battlefield(P0, "Kess, Dissident Mage");
    let div = t.graveyard(P0, "Divination");
    mana(&mut t, P0, ManaType::U, 3);
    let m = kess_method(&mut t, div);
    t.cast(P0, div).method(m).go();
    // In response, P1 gains control of Kess (Ray of Command).
    let ray = t.hand(P1, "Ray of Command");
    mana(&mut t, P1, ManaType::U, 4);
    t.g.turn.priority = Some(P1);
    t.cast(P1, ray).target(kess).go();
    t.resolve();
    assert_eq!(t.obj_now(kess).controller, P1, "{}", t.dump_log());
    t.resolve_all();
    assert!(t.in_exile("Divination"), "{}", t.dump_log());
}

#[test]
fn kess_card_follows_normal_timing_and_is_paid_for_possibly_with_an_alternative_cost() {
    cr!("307.1", "601.2f", "118.9");
    ruling!(
        "Kess, Dissident Mage",
        "You must follow the normal timing permissions and restrictions of the card you cast from your graveyard."
    );
    ruling!(
        "Kess, Dissident Mage",
        "You must pay the costs to cast that card. If it has an alternative cost, you may cast it for that cost instead."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kess, Dissident Mage");
    let div = t.graveyard(P0, "Divination");
    must_pay(&mut t, P0, div, ManaType::U, 3);
    sorcery_timing_only(&mut t, P0, div, ManaType::U, 3);
    // Invigorate's alternative cost (with a Forest: an opponent gains 3 life).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kess, Dissident Mage");
    t.battlefield(P0, "Forest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let inv = t.graveyard(P0, "Invigorate");
    let alt = methods(&mut t, P0, inv)
        .into_iter()
        .find(|m| matches!(m, CastMethod::Alternative(_)))
        .expect("the alternative cost");
    t.cast(P0, inv).method(alt).target(bears).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 23, "{}", t.dump_log());
    assert_eq!(t.pt(bears), (6, 6));
    assert!(t.in_exile("Invigorate"));
}

// ---------------------------------------------------------------------------
// Cast as an ability resolves: Efreet Flamepainter, The Dawning Archaic
// ---------------------------------------------------------------------------

#[test]
fn efreet_flamepainter_casts_the_sorcery_in_combat_or_not_at_all() {
    cr!("608.2g", "307.1");
    ruling!(
        "Efreet Flamepainter",
        "Efreet Flamepainter’s triggered ability lets you cast the spell immediately, ignoring timing restrictions. It doesn’t let you choose to cast it at a later time."
    );
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let e = t.battlefield(P0, "Efreet Flamepainter");
        let div = t.graveyard(P0, "Divination");
        // (Double strike: it triggers twice.)
        t.answer_targets(P0, &[o(div)]);
        t.answer_yes(P0, yes);
        t.answer_yes(P0, yes);
        let hand = t.hand_size(P0);
        attack_with(&mut t, &[(e, Entity::Player(P1))]);
        block_and_finish(&mut t, P1, &[]);
        t.resolve_all();
        if yes {
            assert_eq!(t.hand_size(P0), hand + 2, "{}", t.dump_log());
            assert!(t.in_exile("Divination"));
        } else {
            assert_eq!(t.hand_size(P0), hand, "{}", t.dump_log());
            assert_eq!(t.zone(div), Zone::Graveyard(P0));
            t.set_step(P0, Step::PostcombatMain);
            mana(&mut t, P0, ManaType::U, 3);
            assert!(!castable(&mut t, P0, div), "later");
        }
    }
}

/// P0's The Dawning Archaic attacks; its trigger targets `card` in P0's graveyard.
fn archaic_attacks(t: &mut TestGame, card: ObjectId) {
    let a = t.battlefield(P0, "The Dawning Archaic");
    t.answer_targets(P0, &[o(card)]);
    attack_with(t, &[(a, Entity::Player(P1))]);
}

#[test]
fn dawning_archaic_free_cast_has_x_0() {
    cr!("107.3b", "118.9");
    ruling!(
        "The Dawning Archaic",
        "If the target of The Dawning Archaic's last ability has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    let mut t = TestGame::new(2);
    let blaze = t.graveyard(P0, "Blaze");
    archaic_attacks(&mut t, blaze);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.in_exile("Blaze"), "{}", t.dump_log());
    assert_eq!(t.life(P1), 20);
}

#[test]
fn dawning_archaic_total_cost_applies_increases_then_reductions_and_keeps_mana_value() {
    cr!("601.2f", "202.3", "118.7d");
    ruling!(
        "The Dawning Archaic",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of The Dawning Archaic's first ability). The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    // Eleven instants in the graveyard and Sphere of Resistance: {10} + {1} − {11} = {0}.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sphere of Resistance");
    for _ in 0..11 {
        t.graveyard(P0, "Opt");
    }
    let a = t.hand(P0, "The Dawning Archaic");
    assert!(castable(&mut t, P0, a), "{}", t.dump_log());
    let spell = t.cast(P0, a).go();
    assert_eq!(t.g.obj(spell).chars.mana_value(), 10);
    t.resolve_all();
    assert!(t.on_battlefield(a));
    // Ten instants: {1}.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sphere of Resistance");
    for _ in 0..10 {
        t.graveyard(P0, "Opt");
    }
    let a = t.hand(P0, "The Dawning Archaic");
    assert!(!castable(&mut t, P0, a));
    mana(&mut t, P0, ManaType::C, 1);
    assert!(castable(&mut t, P0, a));
}

#[test]
fn dawning_archaic_casts_on_resolution_ignoring_type_timing_with_additional_costs_only() {
    cr!("608.2g", "118.9a", "601.2b");
    ruling!(
        "The Dawning Archaic",
        "You cast the spell while The Dawning Archaic's last ability is resolving and still on the stack. You can't wait to cast it later in the turn. Timing restrictions based on the card's types are ignored. If you cast a spell \"without paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the spell has any mandatory additional costs, you must pay those to cast the spell."
    );
    // Bone Splinters (a sorcery; sacrifice a creature) in combat: Grizzly Bears sacrificed,
    // P1's Hill Giant destroyed.
    let mut t = TestGame::new(2);
    let bs = t.graveyard(P0, "Bone Splinters");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    archaic_attacks(&mut t, bs);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(bears)]);
    t.answer_targets(P0, &[o(giant)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "{}", t.dump_log());
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.in_exile("Bone Splinters"));
    // Declined: it stays in the graveyard and can't be cast later.
    let mut t = TestGame::new(2);
    let div = t.graveyard(P0, "Divination");
    archaic_attacks(&mut t, div);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.zone(div), Zone::Graveyard(P0));
    t.set_step(P0, Step::PostcombatMain);
    mana(&mut t, P0, ManaType::U, 3);
    assert!(!castable(&mut t, P0, div));
    // Invigorate: its alternative cost isn't offered.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forest");
    let inv = t.graveyard(P0, "Invigorate");
    archaic_attacks(&mut t, inv);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(!crate::r_p068_common::asked_method(&t, from), "{}", t.dump_log());
    assert_eq!(t.life(P1), 20);
    assert!(t.in_exile("Invigorate"));
}

// ---------------------------------------------------------------------------
// Sins of the Past, Maestros Ascendancy
// ---------------------------------------------------------------------------

/// P0 casts Sins of the Past targeting `card` in P0's graveyard; it resolves.
fn sins(t: &mut TestGame, card: ObjectId) {
    let s = t.hand(P0, "Sins of the Past");
    mana(t, P0, ManaType::B, 6);
    t.cast(P0, s).target(card).go();
    t.resolve_all();
    assert!(t.in_exile("Sins of the Past"));
}

#[test]
fn sins_of_the_past_card_is_cast_from_the_graveyard_with_x_0_and_no_alternative_costs() {
    cr!("107.3b", "118.9a", "601.2b", "601.2c");
    ruling!(
        "Sins of the Past",
        "The instant or sorcery card is cast from your graveyard, not your hand. You choose modes, pay additional costs, choose targets, etc. for the spell as normal when casting it. Any X in the mana cost will be 0. Alternative costs can’t be paid."
    );
    // Blaze: X is 0.
    let mut t = TestGame::new(2);
    let blaze = t.graveyard(P0, "Blaze");
    sins(&mut t, blaze);
    let m = methods(&mut t, P0, blaze);
    assert_eq!(m, vec![CastMethod::Free], "{}", t.dump_log());
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    let spell = t.cast(P0, blaze).method(CastMethod::Free).target(P1).go();
    assert_eq!(
        t.g.obj(spell).stack.as_ref().unwrap().cast.from,
        Some(mtg_engine::ability::ZoneKind::Graveyard)
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "{}", t.dump_log());
    // Izzet Charm: a mode and its target are chosen as normal.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let charm = t.graveyard(P0, "Izzet Charm");
    sins(&mut t, charm);
    t.cast(P0, charm)
        .method(CastMethod::Free)
        .modes(&[1])
        .target(bears)
        .go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "{}", t.dump_log());
    // Village Rites: the sacrifice is paid.
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Llanowar Elves");
    let rites = t.graveyard(P0, "Village Rites");
    sins(&mut t, rites);
    t.answer_choose(P0, &[o(elf)]);
    let hand = t.hand_size(P0);
    t.cast(P0, rites).method(CastMethod::Free).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Llanowar Elves"), "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), hand + 2);
    // Invigorate: only without paying its mana cost, not for its alternative cost.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forest");
    t.battlefield(P0, "Grizzly Bears");
    let inv = t.graveyard(P0, "Invigorate");
    sins(&mut t, inv);
    assert_eq!(methods(&mut t, P0, inv), vec![CastMethod::Free]);
    // (From the hand, the alternative cost is available.)
    let inv2 = t.hand(P0, "Invigorate");
    assert!(methods(&mut t, P0, inv2)
        .iter()
        .any(|m| matches!(m, CastMethod::Alternative(_))));
}

#[test]
fn sins_of_the_past_card_is_exiled_even_if_countered_and_cast_once() {
    cr!("614.1a", "701.6a");
    ruling!(
        "Sins of the Past",
        "The instant or sorcery card never gets put back into your graveyard, even if it’s countered. You can’t cast it more than once."
    );
    let mut t = TestGame::new(2);
    let div = t.graveyard(P0, "Divination");
    sins(&mut t, div);
    let spell = t.cast(P0, div).method(CastMethod::Free).go();
    let cs = t.hand(P1, "Counterspell");
    mana(&mut t, P1, ManaType::U, 2);
    t.g.turn.priority = Some(P1);
    t.cast(P1, cs).target(spell).go();
    t.resolve_all();
    assert!(t.in_exile("Divination"), "{}", t.dump_log());
    assert!(!castable(&mut t, P0, div));
    // Resolved: exiled, and not castable again.
    let mut t = TestGame::new(2);
    let div = t.graveyard(P0, "Divination");
    sins(&mut t, div);
    t.cast(P0, div).method(CastMethod::Free).go();
    t.resolve_all();
    assert!(t.in_exile("Divination"));
    assert!(!castable(&mut t, P0, div));
}

#[test]
fn maestros_ascendancy_adventure_is_exiled_by_the_adventure_rule_and_castable_later() {
    cr!("715.3d", "616.1", "616.1e");
    ruling!(
        "Maestros Ascendancy",
        "If you cast an Adventure spell from your graveyard this way, it's exiled as it resolves with the replacement effect created by the Adventure rules and you may cast the creature card from exile later."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Maestros Ascendancy");
    let elf = t.battlefield(P0, "Llanowar Elves");
    let bg = t.graveyard(P0, "Bonecrusher Giant // Stomp");
    mana(&mut t, P0, ManaType::R, 2);
    let m = methods(&mut t, P0, bg);
    let stomp = m
        .iter()
        .find(|m| matches!(m, CastMethod::Half(1)))
        .cloned()
        .unwrap_or_else(|| panic!("Stomp from the graveyard: {m:?}"));
    t.answer_choose(P0, &[o(elf)]);
    t.cast(P0, bg).method(stomp).target(P1).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Llanowar Elves"), "{}", t.dump_log());
    assert_eq!(t.life(P1), 18);
    assert!(exiled(&t, bg), "{}", t.dump_log());
    // Bonecrusher Giant can be cast from exile.
    mana(&mut t, P0, ManaType::R, 3);
    assert!(castable(&mut t, P0, bg), "{}", t.dump_log());
    t.cast(P0, t.g.current(bg)).go();
    t.resolve_all();
    assert!(t.on_battlefield(bg));
}
