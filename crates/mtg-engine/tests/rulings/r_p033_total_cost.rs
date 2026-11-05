//! Rulings batch P033 — the total cost of a spell with cost reductions (CR 601.2f, 118.9d):
//! start with the mana cost or the alternative cost being paid, add additional costs and
//! cost increases, then apply the reductions; X has its value before the reductions apply
//! (CR 107.3b); the mana value stays that of the mana cost (CR 202.3).
//!
//! Many tests use the same discriminating setup: P1's God-Pharaoh's Statue ("Spells your
//! opponents cast cost {2} more to cast") and P0's Omniscience ("You may cast spells from
//! your hand without paying their mana costs."): a spell cast for that alternative cost of
//! {0} costs {2}, and a reduction of {1} applied after the increase leaves {1} (applied
//! before it, it would reduce nothing and leave {2}).

use crate::r_p033_common::*;
use crate::r_s01_common::{supported, tapped_lands};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The Statue-and-Omniscience setup with P0's `reducers` on the battlefield: P0 casts
/// `name` from hand without paying its mana cost. Returns what it cost.
fn free_with_statue(reducers: &[&str], name: &str) -> Paid {
    let mut t = main_phase(2);
    t.battlefield(P1, "God-Pharaoh's Statue");
    t.battlefield(P0, "Omniscience");
    for r in reducers {
        t.battlefield(P0, r);
    }
    let card = t.hand(P0, name);
    crate::r_p046_common::fill_pool(&mut t, P0, 15);
    let m = free_method(&mut t, P0, card);
    cast_paying(&mut t, P0, card, m).expect("cast without paying its mana cost")
}

#[test]
fn reductions_apply_after_increases_to_the_alternative_cost_being_paid() {
    cr!("601.2f", "118.9d", "202.3");
    ruling!(
        "Cunning Nightbonder",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Cunning Nightbonder). The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    ruling!(
        "It That Heralds the End",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of It That Heralds the End). The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    ruling!(
        "Thryx, the Sudden Storm",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Thryx). The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    ruling!(
        "Starnheim Aspirant",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Starnheim Aspirant). The mana value of the spell is determined only by its mana cost, no matter what the total cost to cast the spell was."
    );
    // (reducer, spell, what it costs for Omniscience's {0} plus the Statue's {2}, what it
    // costs for its mana cost plus the Statue's {2})
    let cases = [
        // "Spells with flash you cast cost {1} less to cast and can't be countered."
        // Spectral Sailor ({U}, flash): {2}{U} - {1} (reducing {U} first would leave
        // {2}{U}).
        ("Cunning Nightbonder", "Spectral Sailor", "{1}", "{1}{U}"),
        // "Colorless spells you cast with mana value 7 or greater cost {1} less to cast."
        // Ulamog's Crusher ({8}).
        ("It That Heralds the End", "Ulamog's Crusher", "{1}", "{9}"),
        // "Spells you cast with mana value 5 or greater cost {1} less to cast and can't be
        // countered." Craw Wurm ({4}{G}{G}).
        ("Thryx, the Sudden Storm", "Craw Wurm", "{1}", "{5}{G}{G}"),
        // "Angel spells you cast cost {2} less to cast." Serra Angel ({3}{W}{W}).
        ("Starnheim Aspirant", "Serra Angel", "{0}", "{3}{W}{W}"),
    ];
    for (reducer, spell, free, normal) in cases {
        supported(reducer);
        let paid = free_with_statue(&[reducer], spell);
        assert_spent(&paid.spent, free, reducer);
        assert_eq!(paid.mv, printed_mv(spell), "{reducer}: {spell}'s mana value");
        // Positive control: without the reducer it costs {2}.
        let paid = free_with_statue(&[], spell);
        assert_spent(&paid.spent, "{2}", spell);
        // Its mana cost with the Statue's {2}.
        let mut t = main_phase(2);
        t.battlefield(P1, "God-Pharaoh's Statue");
        t.battlefield(P0, reducer);
        let card = t.hand(P0, spell);
        let paid = cast_normal(&mut t, P0, card);
        assert_spent(&paid.spent, normal, &format!("{reducer}: {spell} for its mana cost"));
        assert_eq!(paid.mv, printed_mv(spell));
    }
}

#[test]
fn sage_of_the_beyond_reduces_an_escape_cost_after_the_increase() {
    cr!("601.2f", "118.9d", "702.138a", "202.3");
    ruling!(
        "Sage of the Beyond",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Sage of the Beyond). The mana value of the spell is determined only by its mana cost, no matter what the total cost to cast the spell was."
    );
    supported("Sage of the Beyond");
    supported("Ox of Agonas");
    // Sage: "Spells you cast from anywhere other than your hand cost {2} less to cast."
    // Ox of Agonas ({3}{R}{R}) escapes for {R}{R} and exiling eight other cards; with the
    // Statue's {2}: {2}{R}{R} - {2} = {R}{R} (reducing first would leave {2}{R}{R}).
    for (sage, cost) in [(true, "{R}{R}"), (false, "{2}{R}{R}")] {
        let mut t = main_phase(2);
        t.battlefield(P1, "God-Pharaoh's Statue");
        if sage {
            t.battlefield(P0, "Sage of the Beyond");
        }
        let ox = t.graveyard(P0, "Ox of Agonas");
        for _ in 0..8 {
            t.graveyard(P0, "Grizzly Bears");
        }
        let paid = cast_paying(&mut t, P0, ox, CastMethod::Keyword(KeywordKind::Escape))
            .expect("escape");
        assert_spent(&paid.spent, cost, "Ox of Agonas");
        assert_eq!(paid.mv, 5);
    }
}

#[test]
fn cloud_key_reduces_flashback_and_kicked_costs() {
    cr!("601.2f", "118.9d", "702.34a", "702.33d", "202.3");
    ruling!(
        "Cloud Key",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying (such as a flashback cost), add any cost increases (such as kicker costs), then apply any cost reductions (such as that of Cloud Key's ability). The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    let mut t = main_phase(2);
    // Instant (index 3).
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.enter(P0, "Cloud Key");
    t.resolve_all();
    // Think Twice's flashback {2}{U}: {1}{U}; mana value 2.
    let tt = t.graveyard(P0, "Think Twice");
    let paid =
        cast_paying(&mut t, P0, tt, CastMethod::Keyword(KeywordKind::Flashback)).expect("flashback");
    assert_spent(&paid.spent, "{1}{U}", "Think Twice flashback");
    assert_eq!(paid.mv, 2);
    t.resolve_all();
    // Burst Lightning ({R}, kicker {4}) kicked: {4}{R} - {1} = {3}{R} (reducing {R} first
    // would leave {4}{R}); mana value 1.
    let burst = t.hand(P0, "Burst Lightning");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let paid = cast_normal(&mut t, P0, burst);
    assert_spent(&paid.spent, "{3}{R}", "kicked Burst Lightning");
    assert_eq!(paid.mv, 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn kicker_and_thalias_increase_are_added_before_reductions() {
    cr!("601.2f", "702.33d");
    ruling!(
        "Killian, Ink Duelist",
        "If a spell has additional costs, apply those increases before applying cost reductions."
    );
    ruling!(
        "Hardened Berserker",
        "If there are any additional costs to cast a spell, apply those before applying Hardened Berserker and any other cost reductions."
    );
    ruling!(
        "Arcane Melee",
        "If there are additional costs to cast a spell, such as a kicker cost or a cost imposed by another effect (such as Thalia, Guardian of Thraben’s ability), apply those increases before applying cost reductions."
    );
    ruling!(
        "Herald of the Pantheon",
        "If there are additional costs to cast an enchantment spell, apply those before applying cost reductions."
    );
    supported("Killian, Ink Duelist");
    supported("Hardened Berserker");
    supported("Arcane Melee");
    supported("Herald of the Pantheon");
    // Burst Lightning ({R}, kicker {4}): no generic mana in its mana cost, so a reduction
    // applied before the kicker would reduce nothing.
    let kicked_burst = |t: &mut TestGame, target: Entity| {
        let burst = t.hand(P0, "Burst Lightning");
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
        t.answer_targets(P0, &[target]);
        cast_normal(t, P0, burst)
    };
    // Killian ("Spells you cast that target a creature cost {2} less to cast"): kicked,
    // targeting a creature: {4}{R} - {2} = {2}{R}.
    let mut t = main_phase(2);
    t.battlefield(P0, "Killian, Ink Duelist");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let paid = kicked_burst(&mut t, obj(bears));
    assert_spent(&paid.spent, "{2}{R}", "Killian");
    // Hardened Berserker ("Whenever this creature attacks, the next spell you cast this
    // turn costs {1} less to cast"): {3}{R}.
    let mut t = main_phase(2);
    let b = t.battlefield(P0, "Hardened Berserker");
    t.attack(&[(b, Entity::Player(P1))], &[]);
    t.resolve_all();
    let paid = kicked_burst(&mut t, Entity::Player(P1));
    assert_spent(&paid.spent, "{3}{R}", "Hardened Berserker");
    // Arcane Melee ({2} less) with P1's Thalia ("Noncreature spells cost {1} more to
    // cast"): {4}{R}{1} - {2} = {3}{R}; and Lightning Bolt ({R}): {1}{R} - {2} = {R}.
    let mut t = main_phase(2);
    t.battlefield(P0, "Arcane Melee");
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let paid = kicked_burst(&mut t, Entity::Player(P1));
    assert_spent(&paid.spent, "{3}{R}", "Arcane Melee and Thalia");
    t.resolve_all();
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let paid = cast_normal(&mut t, P0, bolt);
    assert_spent(&paid.spent, "{R}", "Lightning Bolt, Arcane Melee and Thalia");
    // Herald ("Enchantment spells you cast cost {1} less to cast"): Roost of Drakes ({U},
    // kicker {2}{U}) kicked: {2}{U}{U} - {1} = {1}{U}{U}.
    let mut t = main_phase(2);
    t.battlefield(P0, "Herald of the Pantheon");
    let roost = t.hand(P0, "Roost of Drakes");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    let paid = cast_normal(&mut t, P0, roost);
    assert_spent(&paid.spent, "{1}{U}{U}", "Herald and kicked Roost of Drakes");
    assert_eq!(paid.mv, 1);
    t.resolve_all();
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Drake").len(), 1);
}

#[test]
fn reductions_apply_to_alternative_costs() {
    cr!("601.2f", "118.9d", "702.74a", "702.103a", "702.96a", "702.94a");
    ruling!(
        "Killian, Ink Duelist",
        "Killian can reduce alternative costs. For example, if you choose to pay the {1}{B} alternative cost for Baleful Mastery while targeting a creature, that cost would be reduced to {B}."
    );
    ruling!("Hardened Berserker", "The ability can apply to alternative costs to cast a spell.");
    ruling!(
        "Seal of the Guildpact",
        "The cost reduction can apply to alternative costs such as evoke costs."
    );
    ruling!(
        "Herald of the Pantheon",
        "Herald of the Pantheon can reduce alternative costs such as bestow costs."
    );
    ruling!(
        "Council of the Absolute",
        "The last ability can reduce alternative costs such as overload costs."
    );
    ruling!(
        "Arcane Melee",
        "Arcane Melee can reduce alternative costs such as miracle costs."
    );
    supported("Baleful Mastery");
    // Baleful Mastery ({3}{B}): "You may pay {1}{B} rather than pay this spell's mana
    // cost. If the {1}{B} cost was paid, an opponent draws a card. Exile target creature
    // or planeswalker."
    let mastery_alt = |t: &mut TestGame, bears: ObjectId| {
        let card = t.hand(P0, "Baleful Mastery");
        crate::r_p046_common::fill_pool(t, P0, 15);
        let alt = crate::r_s08_common::legal_cast_methods(t, P0, card)
            .into_iter()
            .find(|m| matches!(m, CastMethod::Alternative(_)))
            .expect("the {1}{B} alternative cost");
        t.answer_targets(P0, &[obj(bears)]);
        cast_paying(t, P0, card, alt).expect("alternative cost")
    };
    // Killian: {B}; the opponent draws, the Bears are exiled.
    let mut t = main_phase(2);
    t.battlefield(P0, "Killian, Ink Duelist");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let hand = t.hand_size(P1);
    let paid = mastery_alt(&mut t, bears);
    assert_spent(&paid.spent, "{B}", "Killian, Baleful Mastery");
    assert_eq!(paid.mv, 4);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.hand_size(P1), hand + 1);
    // Hardened Berserker after it attacked: {B}.
    let mut t = main_phase(2);
    let b = t.battlefield(P0, "Hardened Berserker");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.attack(&[(b, Entity::Player(P1))], &[]);
    t.resolve_all();
    let paid = mastery_alt(&mut t, bears);
    assert_spent(&paid.spent, "{B}", "Hardened Berserker, Baleful Mastery");

    // Seal of the Guildpact with blue and black chosen (index 2 of the color pairs: white
    // and blue, white and black, blue and black, ...): Mulldrifter's evoke {2}{U} is blue:
    // {1}{U}.
    let mut t = main_phase(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    let seal = t.enter(P0, "Seal of the Guildpact");
    let colors = t.obj_now(seal).choices.colors.expect("two colors");
    assert!(colors.contains(Color::Blue) && colors.contains(Color::Black));
    let md = t.hand(P0, "Mulldrifter");
    let paid = cast_paying(&mut t, P0, md, CastMethod::Keyword(KeywordKind::Evoke)).expect("evoke");
    assert_spent(&paid.spent, "{1}{U}", "Seal, evoked Mulldrifter");
    assert_eq!(paid.mv, 5);

    // Herald of the Pantheon: Boon Satyr's bestow {3}{G}{G}: {2}{G}{G}.
    let mut t = main_phase(2);
    t.battlefield(P0, "Herald of the Pantheon");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let satyr = t.hand(P0, "Boon Satyr");
    t.answer_targets(P0, &[obj(bears)]);
    let paid = cast_paying(&mut t, P0, satyr, CastMethod::Keyword(KeywordKind::Bestow)).expect("bestow");
    assert_spent(&paid.spent, "{2}{G}{G}", "Herald, bestowed Boon Satyr");
    assert_eq!(paid.mv, 3);

    // Council of the Absolute naming Cyclonic Rift: overload {6}{U}: {4}{U}.
    let mut t = main_phase(2);
    t.answer(P0, DecisionKind::Name, Answer::Text("Cyclonic Rift".into()));
    t.enter(P0, "Council of the Absolute");
    t.resolve_all();
    let p1s = t.battlefield(P1, "Mind Stone");
    let rift = t.hand(P0, "Cyclonic Rift");
    let paid = cast_paying(&mut t, P0, rift, CastMethod::Keyword(KeywordKind::Overload))
        .expect("overload");
    assert_spent(&paid.spent, "{4}{U}", "Council, overloaded Cyclonic Rift");
    assert_eq!(paid.mv, 2);
    t.resolve_all();
    assert!(t.in_hand(P1, "Mind Stone") && !t.on_battlefield(p1s));

    // Arcane Melee: Temporal Mastery's miracle {1}{U}: {U}, with one Island. Without the
    // Melee, one Island can't pay it.
    for melee in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::Upkeep);
        if melee {
            t.battlefield(P0, "Arcane Melee");
        }
        t.library_top(P0, "Temporal Mastery");
        t.lands(P0, "Island", 1);
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        t.g.draw_cards(P0, 1);
        t.g.flush_events();
        t.settle();
        t.resolve();
        let cast = t
            .g
            .find_in_zone(mtg_engine::object::Zone::Stack, "Temporal Mastery")
            .first()
            .copied();
        assert_eq!(cast.is_some(), melee, "miracle cast with one Island, Melee: {melee}");
        if melee {
            assert_eq!(tapped_lands(&t, P0), 1);
        }
    }
}

#[test]
fn x_is_chosen_before_the_reductions_and_they_reduce_it() {
    cr!("601.2f", "107.3a", "107.3b", "118.7a");
    ruling!(
        "Seal of the Guildpact",
        "If a spell you cast has {X} in its mana cost, you choose the value of X before calculating the spell's total cost. For example, if that spell's mana cost is {X}{R}{G} and you've chosen red and green, you could choose 5 as the value of X and pay {3}{R}{G} to cast the spell."
    );
    ruling!(
        "Semblance Anvil",
        "Semblance Anvil can reduce the amount you pay for a spell's cost that includes {X}. For example, Blaze is a sorcery that costs {X}{R}. If the card exiled with Semblance Anvil is a sorcery, and you want to cast Blaze with X equal to 5, you'll have to pay only {3}{R}. This is true even if the ability states that {X} must be paid with a certain color of mana (as Drain Life does)."
    );
    supported("Seal of the Guildpact");
    supported("Semblance Anvil");
    // Seal of the Guildpact with red and green (the last of the ten color pairs): Savage
    // Twister ({X}{R}{G}) with X = 5 costs {3}{R}{G}.
    let mut t = main_phase(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(9));
    let seal = t.enter(P0, "Seal of the Guildpact");
    let colors = t.obj_now(seal).choices.colors.expect("two colors");
    assert!(colors.contains(Color::Red) && colors.contains(Color::Green));
    let wurm = t.battlefield(P1, "Craw Wurm");
    let twister = t.hand(P0, "Savage Twister");
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    let paid = cast_normal(&mut t, P0, twister);
    assert_spent(&paid.spent, "{3}{R}{G}", "Savage Twister X=5");
    assert_eq!(paid.mv, 7);
    t.resolve_all();
    // (Craw Wurm, 6/4, dies.)
    assert!(t.in_graveyard(P1, "Craw Wurm") && !t.on_battlefield(wurm));

    // Semblance Anvil with a sorcery exiled (Divination): Blaze ({X}{R}) with X = 5 costs
    // {3}{R} and deals 5.
    let anvil = |t: &mut TestGame, imprint: &str| {
        let card = t.hand(P0, imprint);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(card)]);
        t.enter(P0, "Semblance Anvil");
        t.resolve_all();
        assert!(t.in_exile(imprint));
    };
    let mut t = main_phase(2);
    anvil(&mut t, "Divination");
    let blaze = t.hand(P0, "Blaze");
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let paid = cast_normal(&mut t, P0, blaze);
    assert_spent(&paid.spent, "{3}{R}", "Blaze X=5");
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    // Drain Life ({X}{1}{B}, "Spend only black mana on X.") with X = 3: {3}{1}{B} - {2},
    // paid with three black mana (the reduction applies to X's mana too), and it deals 3.
    let mut t = main_phase(2);
    anvil(&mut t, "Divination");
    let drain = t.hand(P0, "Drain Life");
    t.lands(P0, "Swamp", 3);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, drain).go();
    assert_eq!(tapped_lands(&t, P0), 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn semblance_anvil_reduces_the_total_cost_including_kicker_evoke_and_increases() {
    cr!("601.2f", "118.9d", "702.33d", "702.74a");
    ruling!(
        "Semblance Anvil",
        "Semblance Anvil takes the total cost to cast a spell into account, not just its mana cost. This includes additional costs (such as kicker) and alternative costs (such as evoke). It also includes cost increases from other sources (such as Lodestone Golem's ability)."
    );
    ruling!(
        "Semblance Anvil",
        "Semblance Anvil won't affect the part of a spell's cost represented by colored mana symbols. For example, a spell that normally costs {1}{U} to cast would cost {U} if it shared a card type with the exiled card."
    );
    ruling!(
        "Semblance Anvil",
        "Spells you cast that share multiple card types with the exiled card still cost just {2} less to cast. They don't get a greater cost reduction."
    );
    ruling!("Semblance Anvil", "Semblance Anvil's cost reduction effect is mandatory.");
    let anvil = |imprint: &str| {
        let mut t = main_phase(2);
        let card = t.hand(P0, imprint);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(card)]);
        t.enter(P0, "Semblance Anvil");
        t.resolve_all();
        assert!(t.in_exile(imprint));
        t
    };
    // A creature exiled (Grizzly Bears): Kavu Titan ({1}{G}, kicker {2}{G}) kicked:
    // {3}{G}{G} - {2} = {1}{G}{G}.
    let mut t = anvil("Grizzly Bears");
    let titan = t.hand(P0, "Kavu Titan");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    let paid = cast_normal(&mut t, P0, titan);
    assert_spent(&paid.spent, "{1}{G}{G}", "kicked Kavu Titan");
    // Mulldrifter's evoke {2}{U}: {U}.
    let mut t = anvil("Grizzly Bears");
    let md = t.hand(P0, "Mulldrifter");
    let paid = cast_paying(&mut t, P0, md, CastMethod::Keyword(KeywordKind::Evoke)).expect("evoke");
    assert_spent(&paid.spent, "{U}", "evoked Mulldrifter");
    // Coral Trickster ({1}{U}): {U}; nothing reduces its {U}.
    let trickster = t.hand(P0, "Coral Trickster");
    t.resolve_all();
    let paid = cast_normal(&mut t, P0, trickster);
    assert_spent(&paid.spent, "{U}", "Coral Trickster");
    // P1's Lodestone Golem ("Nonartifact spells cost {1} more to cast"): Grizzly Bears
    // ({1}{G}): {2}{G} - {2} = {G} (reducing first would leave {1}{G}).
    let mut t = anvil("Craw Wurm");
    t.battlefield(P1, "Lodestone Golem");
    let bears = t.hand(P0, "Grizzly Bears");
    let paid = cast_normal(&mut t, P0, bears);
    assert_spent(&paid.spent, "{G}", "Bears with Lodestone Golem");
    // An artifact creature exiled (Steel Overseer): Juggernaut (an artifact creature, {4})
    // shares both types and still costs {2}.
    let mut t = anvil("Steel Overseer");
    let jugg = t.hand(P0, "Juggernaut");
    let paid = cast_normal(&mut t, P0, jugg);
    assert_spent(&paid.spent, "{2}", "Juggernaut");
    // Mandatory: P0 isn't asked whether to apply it (a queued "no" isn't consumed), and
    // with plenty of mana the spell still costs {2} less.
    let mut t = anvil("Craw Wurm");
    let from = t.asked().len();
    t.answer_yes(P0, false);
    let wurm = t.hand(P0, "Craw Wurm");
    let paid = cast_normal(&mut t, P0, wurm);
    assert_spent(&paid.spent, "{2}{G}{G}", "Craw Wurm");
    assert!(
        t.asked()[from..]
            .iter()
            .all(|(_, d)| !matches!(d, mtg_engine::decision::Decision::YesNo { .. })),
        "asked whether to apply the reduction"
    );
}

#[test]
fn ragemonger_reduces_black_and_red_mana_of_minotaur_spells() {
    cr!("601.2f", "601.2b", "702.35a", "702.33d");
    ruling!(
        "Ragemonger",
        "If a spell has hybrid mana symbols in its mana cost, you choose which half you will be paying before determining the total cost. If you choose to pay such a cost with {B} or {R}, Ragemonger can reduce that part of the cost."
    );
    ruling!(
        "Ragemonger",
        "If you control more than one Ragemonger, the cost reduction is cumulative."
    );
    ruling!(
        "Ragemonger",
        "You apply cost reduction effects after other cost modifiers, so Ragemonger can reduce additional costs or alternative costs of Minotaur spells if they include {B} and/or {R}."
    );
    supported("Ragemonger");
    // "Minotaur spells you cast cost {B}{R} less to cast. This effect reduces only the
    // amount of colored mana you pay."
    // Boros Reckoner ({R/W}{R/W}{R/W}): one hybrid symbol paid with {R} is reduced: two
    // mana left, each red or white.
    let mut t = main_phase(2);
    t.battlefield(P0, "Ragemonger");
    let reckoner = t.hand(P0, "Boros Reckoner");
    let paid = cast_normal(&mut t, P0, reckoner);
    assert_eq!(paid.spent.len(), 2, "{:?}", paid.spent);
    assert!(paid
        .spent
        .iter()
        .all(|m| matches!(m, mtg_engine::mana::ManaType::R | mtg_engine::mana::ManaType::W)));
    // Two Ragemongers: Oracle of Bones ({2}{R}{R}) costs {2}; one: {2}{R}.
    for (n, cost) in [(1, "{2}{R}"), (2, "{2}")] {
        let mut t = main_phase(2);
        for _ in 0..n {
            t.battlefield(P0, "Ragemonger");
        }
        let oracle = t.hand(P0, "Oracle of Bones");
        let paid = cast_normal(&mut t, P0, oracle);
        assert_spent(&paid.spent, cost, &format!("{n} Ragemongers"));
    }
    // An alternative cost: Skophos Reaver's "Madness {1}{R}" costs {1}: one Wastes pays
    // it (without Ragemonger it doesn't, and the Reaver goes to the graveyard).
    supported("Skophos Reaver");
    for rage in [true, false] {
        let mut t = main_phase(2);
        if rage {
            t.battlefield(P0, "Ragemonger");
        }
        t.lands(P0, "Wastes", 1);
        let reaver = t.hand(P0, "Skophos Reaver");
        t.g.discard(P0, reaver, None);
        t.g.flush_events();
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Skophos Reaver").len(), rage as usize, "{rage}");
        assert_eq!(tapped_lands(&t, P0), rage as usize);
    }
    // An additional cost: a Minotaur with "Kicker {B}{R}" ({2}) kicked: {2}{B}{R} - {B}{R}
    // = {2} (reducing first would reduce nothing).
    let def = crate::r_s01_common::custom_card(
        "Kicking Minotaur",
        "Creature — Minotaur",
        "{2}",
        Some((2, 2)),
        "Kicker {B}{R}",
    );
    let mut t = main_phase(2);
    t.battlefield(P0, "Ragemonger");
    let card = t.custom(P0, def, mtg_engine::object::Zone::Hand(P0));
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    let paid = cast_normal(&mut t, P0, card);
    assert_spent(&paid.spent, "{2}", "kicked Minotaur");
    assert!(t.obj(paid.spell).stack.as_ref().unwrap().cast.times_kicked > 0);
}
