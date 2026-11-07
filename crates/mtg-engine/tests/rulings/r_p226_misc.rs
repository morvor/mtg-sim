//! Rulings batch P226 — tribute (CR 702.104), triple damage and trample (CR 510.1c),
//! undergrowth, undying (CR 702.93), unearth (CR 702.84), unleash (CR 702.98) and
//! vanishing (CR 702.63).

use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, create_token, destroy};
use crate::r_s04_common::{untapped_lands, *};
use crate::r_s05_common::move_to;
use crate::r_s06_common::activate_containing;
use crate::r_s08_common::mana_value;
use crate::r_s13_common::add;
use crate::r_s17_common::{become_copy, put_onto_battlefield};
use crate::r_s18_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{StackKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn shrike_harpys_target_neednt_be_the_opponent_chosen_for_tribute() {
    cr!("702.104a", "702.104b", "603.4");
    ruling!(
        "Shrike Harpy",
        "The opponent you target with the triggered ability doesn’t have to be the same opponent you chose to pay tribute or not."
    );
    supported("Shrike Harpy");
    // "Tribute 2" / "When this creature enters, if tribute wasn't paid, target opponent
    // sacrifices a creature of their choice." P1 is chosen and declines; P2 is targeted.
    let mut t = TestGame::new(3);
    let p2_bears = t.battlefield(P2, "Grizzly Bears");
    let p1_bears = t.battlefield(P1, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Shrike Harpy");
    let harpy = t.hand(P0, "Shrike Harpy");
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(vec![Entity::Player(P1)]),
    );
    t.answer_yes(P1, false);
    t.answer_targets(P0, &[Entity::Player(P2)]);
    t.cast(P0, harpy).go();
    t.resolve_all();
    let harpy = t.named_on_battlefield("Shrike Harpy")[0];
    assert_eq!(t.counters(harpy, counters::PLUS1), 0);
    assert!(!t.on_battlefield(p2_bears));
    assert!(t.on_battlefield(p1_bears));
}

#[test]
fn city_on_fire_trample_assigns_unmodified_lethal_damage_then_triples_it() {
    cr!("510.1c", "702.19b", "614.1a");
    ruling!(
        "City on Fire",
        "While you control City on Fire, if a creature you control with trample would deal combat damage to a blocking creature, you must assign its unmodified damage. For example, a 3/3 creature with trample blocked by a 2/2 creature can have 1 damage assigned to the defending player. It will then deal 6 damage to the blocking creature (2 tripled) and 3 to the defending player (1 tripled)."
    );
    supported("City on Fire");
    // "If a source you control would deal damage to a permanent or player, it deals
    // triple that damage instead."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "City on Fire");
    let mammoth = t.battlefield(P0, "War Mammoth");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![2, 1]));
    t.attack(&[(mammoth, Entity::Player(P1))], &[(bears, mammoth)]);
    // Lethal damage for the Bears was 2, not 1 (the damage before tripling).
    let lethal: Vec<Vec<u32>> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::AssignCombatDamage { lethal, .. } => Some(lethal),
            _ => None,
        })
        .collect();
    assert_eq!(lethal, vec![vec![2]]);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 17);
    let bears_dmg = t.g.log.iter().any(|l| l.text.contains("deals 6 damage"));
    assert!(bears_dmg, "{}", t.dump_log());
}

// ---------------------------------------------------------------------------------------
// Undergrowth
// ---------------------------------------------------------------------------------------

#[test]
fn molderhulk_cast_from_the_graveyard_doesnt_count_itself_and_keeps_its_mana_value() {
    cr!("601.2f", "118.7", "202.3");
    ruling!(
        "Molderhulk",
        "If an effect allows you to cast Molderhulk from your graveyard, its undergrowth ability doesn't count itself. It's already on the stack when you determine the total cost to cast it."
    );
    ruling!(
        "Molderhulk",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as Molderhulk's undergrowth ability). The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    supported("Molderhulk");
    supported("Muldrotha, the Gravetide");
    // "Undergrowth — This spell costs {1} less to cast for each creature card in your
    // graveyard." With Muldrotha, from the graveyard with two other creature cards there:
    // {5}{B}{G}, not {4}{B}{G}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Muldrotha, the Gravetide");
    let hulk = t.graveyard(P0, "Molderhulk");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 4);
    assert!(t.cast(P0, hulk).try_go().is_err());
    t.lands(P0, "Wastes", 1);
    let spell = t.cast(P0, hulk).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(mana_value(&t, spell), 9);
    t.answer_targets(P0, &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Molderhulk").len(), 1);
    // From the hand with three creature cards in the graveyard: {4}{B}{G}, and its mana
    // value on the stack is still 9. "When this creature enters, return target land card
    // from your graveyard to the battlefield."
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.graveyard(P0, "Grizzly Bears");
    }
    let forest = t.graveyard(P0, "Forest");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 4);
    let card = t.hand(P0, "Molderhulk");
    let spell = t.cast(P0, card).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(mana_value(&t, spell), 9);
    t.answer_targets(P0, &[Entity::Object(forest)]);
    t.resolve_all();
    assert!(t.on_battlefield(forest));
}

#[test]
fn golgari_raiders_and_rhizome_lurcher_count_themselves_when_returned_from_the_graveyard() {
    cr!("614.1c", "614.12");
    ruling!(
        "Golgari Raiders",
        "If you return Golgari Raiders from your graveyard to the battlefield, its undergrowth ability counts itself."
    );
    ruling!(
        "Rhizome Lurcher",
        "If you return Rhizome Lurcher from your graveyard to the battlefield, its undergrowth ability counts itself."
    );
    // "Undergrowth — This creature enters with a +1/+1 counter on it for each creature
    // card in your graveyard." / "... with a number of +1/+1 counters on it equal to the
    // number of creature cards in your graveyard."
    for name in ["Golgari Raiders", "Rhizome Lurcher"] {
        supported(name);
        let mut t = TestGame::new(2);
        let card = t.graveyard(P0, name);
        t.graveyard(P0, "Grizzly Bears");
        let it = put_onto_battlefield(&mut t, P0, card, false).unwrap();
        assert_eq!(t.counters(it, counters::PLUS1), 2, "{name}");
        // Cast from the hand, it doesn't count itself.
        let mut t = TestGame::new(2);
        t.graveyard(P0, "Grizzly Bears");
        give_mana_for(&mut t, P0, name);
        let card = t.hand(P0, name);
        t.cast(P0, card).go();
        t.resolve_all();
        let it = t.named_on_battlefield(name)[0];
        assert_eq!(t.counters(it, counters::PLUS1), 1, "{name}");
    }
    // Golgari Raiders has haste.
    let mut t = TestGame::new(2);
    let raiders = t.battlefield_sick(P0, "Golgari Raiders");
    assert!(t.obj_now(raiders).has_keyword(KeywordKind::Haste));
}

#[test]
fn kraul_harpooners_bonus_is_counted_once_as_its_ability_resolves() {
    cr!("608.2h", "611.2c", "701.14a");
    ruling!(
        "Kraul Harpooner",
        "The value of X is determined only as the undergrowth ability resolves. If the number of creature cards in your graveyard changes later in the turn, Kraul Harpooner is unaffected."
    );
    supported("Kraul Harpooner");
    // "Undergrowth — When this creature enters, choose up to one target creature you
    // don't control with flying. This creature gets +X/+0 until end of turn, where X is
    // the number of creature cards in your graveyard, then you may have this creature
    // fight that creature."
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Grizzly Bears");
    let angel = t.battlefield(P1, "Serra Angel");
    t.answer_targets(P0, &[Entity::Object(angel)]);
    t.answer_yes(P0, true);
    let harpooner = t.enter(P0, "Kraul Harpooner");
    t.resolve_all();
    // 3/2 + 2/+0: 5 damage kills the 4/4 Angel; the Angel's 4 kills the Harpooner.
    assert!(t.in_graveyard(P1, "Serra Angel"));
    assert!(!t.on_battlefield(harpooner));
    // No target; later creature cards don't change the bonus.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[]);
    let harpooner = t.enter(P0, "Kraul Harpooner");
    t.resolve_all();
    assert_eq!(t.pt(harpooner), (4, 2));
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(harpooner), (4, 2));
    assert!(t.obj_now(harpooner).has_keyword(KeywordKind::Reach));
}

#[test]
fn izoni_can_sacrifice_creatures_in_response_to_its_undergrowth_trigger() {
    cr!("608.2h", "602.2", "603.3");
    ruling!(
        "Izoni, Thousand-Eyed",
        "You can activate Izoni's last ability while its undergrowth ability is on the stack. This will increase the number of Insect tokens you'll create."
    );
    supported("Izoni, Thousand-Eyed");
    // "Undergrowth — When Izoni enters, create a 1/1 black and green Insect creature
    // token for each creature card in your graveyard." / "{B}{G}, Sacrifice another
    // creature: You gain 1 life and draw a card."
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let izoni = t.enter(P0, "Izoni, Thousand-Eyed");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, izoni, "Sacrifice another creature").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(with_subtype(&t, P0, "Insect").len(), 2);
}

// ---------------------------------------------------------------------------------------
// Undying, unearth, unleash
// ---------------------------------------------------------------------------------------

#[test]
fn relentless_skaabs_returning_by_undying_isnt_cast_and_exiles_nothing() {
    cr!("702.93a", "601.2f", "118.8");
    ruling!(
        "Relentless Skaabs",
        "When Relentless Skaabs returns to the battlefield because of its undying ability, it’s not being cast. You won’t exile a creature card from your graveyard."
    );
    supported("Relentless Skaabs");
    // "As an additional cost to cast this spell, exile a creature card from your
    // graveyard." / Undying.
    let mut t = TestGame::new(2);
    let fodder = t.graveyard(P0, "Grizzly Bears");
    let kept = t.graveyard(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Relentless Skaabs");
    let card = t.hand(P0, "Relentless Skaabs");
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    t.cast(P0, card).go();
    assert_eq!(t.zone(fodder), Zone::Exile);
    t.resolve_all();
    let skaabs = t.named_on_battlefield("Relentless Skaabs")[0];
    destroy(&mut t, skaabs);
    t.resolve_all();
    let back = t.named_on_battlefield("Relentless Skaabs");
    assert_eq!(back.len(), 1);
    assert_eq!(t.counters(back[0], counters::PLUS1), 1);
    assert_eq!(t.zone(kept), Zone::Graveyard(P0));
}

#[test]
fn dregscape_slivers_unearth_is_an_ability_not_a_spell() {
    cr!("702.84a", "602.2", "701.6a", "115.1");
    ruling!(
        "Dregscape Sliver",
        "Activating a creature card’s unearth ability isn’t the same as casting the creature spell. The unearth ability is put on the stack, but the creature card is not. Spells and abilities that interact with activated abilities (such as Stifle) will interact with unearth, but spells and abilities that interact with spells (such as Cancel) will not."
    );
    supported("Dregscape Sliver");
    let mut t = TestGame::new(2);
    let sliver = t.graveyard(P0, "Dregscape Sliver");
    t.lands(P0, "Wastes", 2);
    let ability = unearth(&mut t, P0, sliver).unwrap().unwrap();
    assert_eq!(t.zone(sliver), Zone::Graveyard(P0));
    assert!(matches!(
        t.obj(ability).stack.as_deref().map(|s| &s.kind),
        Some(StackKind::Activated { .. })
    ));
    // Cancel has nothing to target; Stifle counters the ability.
    t.lands(P1, "Island", 3);
    let cancel = t.hand(P1, "Cancel");
    assert!(t.cast(P1, cancel).target(ability).try_go().is_err());
    t.clear_answers();
    let stifle = t.hand(P1, "Stifle");
    t.cast(P1, stifle).target(ability).go();
    t.resolve_all();
    assert_eq!(t.zone(sliver), Zone::Graveyard(P0));
    assert!(t.named_on_battlefield("Dregscape Sliver").is_empty());
}

#[test]
fn dregscape_sliver_grants_unearth_only_while_on_the_battlefield() {
    cr!("702.84a", "604.1");
    ruling!(
        "Dregscape Sliver",
        "Dregscape Sliver grants the unearth ability to other Sliver cards only while it’s on the battlefield."
    );
    // "Each Sliver creature card in your graveyard has unearth {2}."
    let mut t = TestGame::new(2);
    let muscle = t.graveyard(P0, "Muscle Sliver");
    t.lands(P0, "Wastes", 2);
    assert!(!can_activate(&mut t, P0, muscle));
    // In the graveyard too, it grants nothing.
    let dregscape = t.graveyard(P0, "Dregscape Sliver");
    assert!(!can_activate(&mut t, P0, muscle));
    move_to(&mut t, dregscape, Zone::Battlefield);
    assert!(can_activate(&mut t, P0, muscle));
    unearth(&mut t, P0, muscle).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Muscle Sliver").len(), 1);
}

/// The unearthed creature is exiled instead of going to its owner's hand, but an
/// effect that exiles it succeeds, and the card it returns as is a new object that
/// unearth no longer affects.
fn unearth_exile_and_return(name: &str, cost: &str) {
    supported(name);
    // Unsummon: exiled instead.
    let mut t = TestGame::new(2);
    let it = unearthed(&mut t, P0, name, cost);
    t.lands(P0, "Island", 1);
    let unsummon = t.hand(P0, "Unsummon");
    t.cast(P0, unsummon).target(it).go();
    t.resolve_all();
    assert_eq!(t.zone(it), Zone::Exile, "{name}");
    assert!(!t.in_hand(P0, name));
    // Cloudshift: exiled and returned as a new object; it stays past the end step.
    let mut t = TestGame::new(2);
    let it = unearthed(&mut t, P0, name, cost);
    t.lands(P0, "Plains", 1);
    let shift = t.hand(P0, "Cloudshift");
    t.cast(P0, shift).target(it).go();
    t.resolve_all();
    let back = t.named_on_battlefield(name);
    assert_eq!(back.len(), 1, "{name}");
    assert_ne!(back[0], it);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(back[0]), "{name}");
    let again = back[0];
    destroy(&mut t, again);
    assert!(t.in_graveyard(P0, name));
}

#[test]
fn an_unearthed_creature_that_an_effect_exiles_comes_back_as_a_new_object() {
    cr!("702.84a", "400.7", "614.1a");
    ruling!(
        "First-Sphere Gargantua",
        "If a creature returned to the battlefield by the unearth ability would leave it for any reason, it's exiled instead—unless the spell or ability that's causing the creature to leave the battlefield is actually trying to exile it. In that case, the spell or ability succeeds at exiling the creature. If the spell or ability later returns the creature card to the battlefield (as Astral Drift might, for example), the creature card will return as a new object with no relation to its previous existence. The unearth effect will no longer apply to it."
    );
    ruling!(
        "Dregscape Sliver",
        "If a creature returned to the battlefield by the unearth ability would leave it for any reason, it’s exiled instead—unless the spell or ability that’s causing the creature to leave the battlefield is actually trying to exile it. In that case, the spell or ability succeeds at exiling the creature. If the spell or ability later returns the creature card to the battlefield (as Astral Drift might, for example), the creature card will return as a new object with no relation to its previous existence. The unearth effect will no longer apply to it."
    );
    unearth_exile_and_return("First-Sphere Gargantua", "{2}{B}");
    unearth_exile_and_return("Dregscape Sliver", "{2}");
}

#[test]
fn lokhust_heavy_destroyer_may_be_sacrificed_to_itself_and_an_unearthed_one_is_exiled() {
    cr!("702.84a", "701.21a", "614.1a");
    ruling!(
        "Lokhust Heavy Destroyer",
        "When its triggered ability resolves, you may sacrifice Lokhust Heavy Destroyer itself. If you control no other creatures, you'll have to sacrifice it. If it was unearthed, it will be exiled."
    );
    supported("Lokhust Heavy Destroyer");
    // "Enmitic Exterminator — When this creature enters, each player sacrifices a
    // creature of their choice." Unearthed with no other creature: it's sacrificed, and
    // exiled.
    let mut t = TestGame::new(2);
    let p1_bears = t.battlefield(P1, "Grizzly Bears");
    let card = t.graveyard(P0, "Lokhust Heavy Destroyer");
    lands_for(&mut t, P0, "{5}{B}{B}{B}");
    unearth(&mut t, P0, card).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Exile);
    assert!(!t.on_battlefield(p1_bears));
    // With another creature, P0 may choose to sacrifice Lokhust itself.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lokhust = t.enter(P0, "Lokhust Heavy Destroyer");
    t.answer_choose(P0, &[Entity::Object(lokhust)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Lokhust Heavy Destroyer"));
    assert!(t.on_battlefield(bears));
}

#[test]
fn chaos_imps_have_trample_with_any_plus_one_counter() {
    cr!("702.98a", "613.1f");
    ruling!(
        "Chaos Imps",
        "Chaos Imps will have trample if it has any +1/+1 counter on it, not just the one put on it by the unleash ability."
    );
    supported("Chaos Imps");
    // "Unleash" / "This creature has trample as long as it has a +1/+1 counter on it."
    // Entering without the unleash counter: no trample; a +1/+1 counter from another
    // effect gives it trample (and stops it from blocking).
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Chaos Imps");
    let card = t.hand(P0, "Chaos Imps");
    t.answer_yes(P0, false);
    t.cast(P0, card).go();
    t.resolve_all();
    let imps = t.named_on_battlefield("Chaos Imps")[0];
    assert_eq!(t.counters(imps, counters::PLUS1), 0);
    assert!(!t.obj_now(imps).has_keyword(KeywordKind::Trample));
    assert!(t.g.can_block_at_all(imps));
    add(&mut t, imps, counters::PLUS1, 1);
    t.g.recompute();
    assert!(t.obj_now(imps).has_keyword(KeywordKind::Trample));
    assert!(t.obj_now(imps).has_keyword(KeywordKind::Flying));
    assert!(!t.g.can_block_at_all(imps));
}

// ---------------------------------------------------------------------------------------
// Vanishing
// ---------------------------------------------------------------------------------------

/// Advances to P0's next upkeep and resolves what triggers.
fn p0_upkeep(t: &mut TestGame) {
    next_upkeep(t, P0);
    t.resolve_all();
}

#[test]
fn becoming_a_copy_of_dreamtide_whale_needs_time_counters_to_vanish() {
    cr!("702.63a", "707.2", "707.4");
    ruling!(
        "Dreamtide Whale",
        "If a permanent that's already on the battlefield without time counters on it becomes a copy of a permanent with vanishing, it will stay on the battlefield indefinitely (unless time counters are somehow put on it later). If a permanent with one or more time counters on it becomes a copy of a permanent with vanishing, it will vanish as normal. If a permanent enters the battlefield as a copy of a permanent with vanishing, it enters with the appropriate number of time counters and will vanish as normal."
    );
    supported("Dreamtide Whale");
    let mut t = TestGame::new(2);
    let whale = t.battlefield(P0, "Dreamtide Whale");
    // No time counters: it stays.
    let bears = t.battlefield(P0, "Grizzly Bears");
    become_copy(&mut t, bears, whale);
    assert_eq!(t.obj_now(bears).chars.name, "Dreamtide Whale");
    assert_eq!(t.counters(bears, counters::TIME), 0);
    // One time counter: it vanishes at the next upkeep.
    let other = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, other, counters::TIME, 1);
    become_copy(&mut t, other, whale);
    // Entering as a copy (Clone): two time counters.
    t.lands(P0, "Island", 4);
    let clone = t.hand(P0, "Clone");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(whale)]);
    let clone = {
        t.cast(P0, clone).go();
        t.resolve_all();
        t.named_on_battlefield("Dreamtide Whale")
            .into_iter()
            .find(|o| ![whale, bears, other].contains(o))
            .unwrap()
    };
    assert_eq!(t.counters(clone, counters::TIME), 2);
    p0_upkeep(&mut t);
    assert!(t.on_battlefield(bears));
    assert!(!t.on_battlefield(other));
    assert_eq!(t.counters(clone, counters::TIME), 1);
    p0_upkeep(&mut t);
    assert!(t.on_battlefield(bears));
    assert!(!t.on_battlefield(clone));
}

#[test]
fn a_whale_whose_sacrifice_trigger_is_countered_stays_with_no_more_vanishing_triggers() {
    cr!("702.63a", "603.4", "701.6a");
    ruling!(
        "Dreamtide Whale",
        "If the last time counter is removed from a permanent with vanishing and the sacrifice ability is countered, that permanent will remain on the battlefield indefinitely with no time counters on it. As long as the permanent doesn't have any time counters on it, neither of vanishing's two triggered abilities will trigger again."
    );
    let mut t = TestGame::new(2);
    let whale = t.battlefield(P0, "Dreamtide Whale");
    add(&mut t, whale, counters::TIME, 1);
    next_upkeep(&mut t, P0);
    t.resolve();
    assert_eq!(t.counters(whale, counters::TIME), 0);
    // The sacrifice trigger is on the stack; Stifle counters it.
    assert_eq!(t.stack_len(), 1);
    t.lands(P1, "Island", 1);
    let stifle = t.hand(P1, "Stifle");
    let sac = top_of_stack(&t);
    t.cast(P1, stifle).target(sac).go();
    t.resolve_all();
    assert!(t.on_battlefield(whale));
    // The next upkeep: no triggers.
    next_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(whale));
}
