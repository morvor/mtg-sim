//! Rulings batch P069 — costs and mana of cards cast from exile: cost reductions of
//! generic mana only (CR 601.2f), mana value vs. total cost (CR 202.3), alternative and
//! additional costs (CR 118.9, 601.2b), mana "as though it were mana of any type" (CR
//! 609.4b) and snow costs (CR 107.4h), mana with or without spending restrictions (CR
//! 106.6), and costs paid by sacrificing or tapping (CR 601.2g–h, 302.6).

use crate::r_p069_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, stack_library, supported};
use crate::r_s08_common::legal_cast_methods;
use mtg_engine::decision::Answer;
use mtg_engine::designations;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn savvy_trader_reduces_only_generic_mana() {
    cr!("601.2f", "118.7d");
    ruling!(
        "Savvy Trader",
        "Savvy Trader’s last ability can’t reduce the amount of colored mana you pay for a spell. It reduces only the generic mana component of that spell’s cost."
    );
    supported("Savvy Trader");
    // Llanowar Elves ({G}) exiled by Savvy Trader still costs {G}; Think Twice's
    // flashback cost ({2}{U}) becomes {1}{U}.
    let mut t = TestGame::new(2);
    let elves = t.graveyard(P0, "Llanowar Elves");
    let tt = t.graveyard(P0, "Think Twice");
    t.answer_targets(P0, &[o(elves)]);
    t.enter(P0, "Savvy Trader");
    t.resolve_all();
    assert!(exiled(&t, elves), "{}", t.dump_log());
    must_pay(&mut t, P0, elves, ManaType::G, 1);
    mana(&mut t, P0, ManaType::U, 1);
    assert!(!castable(&mut t, P0, tt));
    mana(&mut t, P0, ManaType::C, 1);
    assert!(castable(&mut t, P0, tt));
}

#[test]
fn savvy_trader_doesnt_change_the_mana_value() {
    cr!("202.3", "601.2f");
    ruling!(
        "Savvy Trader",
        "Savvy Trader’s last ability doesn’t change the mana cost or mana value of any spell. It changes only the total cost you pay to cast spells from anywhere other than your hand."
    );
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[o(bears)]);
    t.enter(P0, "Savvy Trader");
    t.resolve_all();
    mana(&mut t, P0, ManaType::G, 1);
    let s = cast_now(&mut t, P0, bears, &[]);
    assert!(t.g.player(P0).mana_pool.is_empty());
    assert_eq!(t.g.mana_value_of(s), 2);
    // From the hand: no reduction.
    let b2 = t.hand(P0, "Grizzly Bears");
    t.resolve_all();
    mana(&mut t, P0, ManaType::G, 1);
    assert!(!castable(&mut t, P0, b2));
}

/// P0 casts Shadow of the Enemy on P1's graveyard, which holds `name`.
fn shadow(t: &mut TestGame, name: &str) -> ObjectId {
    supported("Shadow of the Enemy");
    let c = t.graveyard(P1, name);
    let s = t.hand(P0, "Shadow of the Enemy");
    mana(t, P0, ManaType::B, 6);
    t.cast(P0, s).target(P1).go();
    t.resolve_all();
    assert!(exiled(t, c), "{}", t.dump_log());
    c
}

#[test]
fn shadow_of_the_enemy_any_type_includes_colorless() {
    cr!("609.4b", "106.1b");
    ruling!(
        "Shadow of the Enemy",
        "Shadow of the Enemy uses a new template indicating that you may spend mana as though it were mana of any type to cast the exiled cards. The six types of mana are white, blue, black, red, green, and colorless."
    );
    // Thought-Knot Seer ({3}{C}) paid with red mana.
    let mut t = TestGame::new(2);
    let tks = shadow(&mut t, "Thought-Knot Seer");
    must_pay(&mut t, P0, tks, ManaType::R, 4);
    mana(&mut t, P0, ManaType::R, 4);
    cast_now(&mut t, P0, tks, &[]);
    assert!(t.g.player(P0).mana_pool.is_empty());
}

#[test]
fn shadow_of_the_enemy_cant_pay_snow_with_nonsnow_mana() {
    cr!("107.4h", "609.4b");
    ruling!(
        "Shadow of the Enemy",
        "Snow mana is not a type of mana. Shadow of the Enemy won't let you pay a snow cost using mana produced by a nonsnow source."
    );
    // Icehide Golem ({S}): a Forest's mana can't pay for it, a Snow-Covered Forest's can.
    let mut t = TestGame::new(2);
    let golem = shadow(&mut t, "Icehide Golem");
    let forest = t.battlefield(P0, "Forest");
    assert!(!castable(&mut t, P0, golem));
    t.g.tap(forest);
    t.battlefield(P0, "Snow-Covered Forest");
    assert!(castable(&mut t, P0, golem));
}

#[test]
fn boiling_rock_rioter_can_tap_itself_while_summoning_sick() {
    cr!("302.6", "602.5a");
    ruling!(
        "Boiling Rock Rioter",
        "Since Boiling Rock Rioter's activated ability doesn't have a tap symbol in its cost, you can tap a creature (including Boiling Rock Rioter itself) that hasn't been under your control since your most recent turn began to pay the cost."
    );
    supported("Boiling Rock Rioter");
    let mut t = TestGame::new(2);
    let r = t.battlefield_sick(P0, "Boiling Rock Rioter");
    let card = t.graveyard(P1, "Grizzly Bears");
    t.answer_choose(P0, &[o(r)]);
    t.activate(P0, r, 0, &[o(card)]).unwrap();
    assert!(t.obj_now(r).tapped);
    t.resolve_all();
    assert!(exiled(&t, card), "{}", t.dump_log());
}

#[test]
fn dream_harvest_free_casts_allow_additional_but_no_other_alternative_costs() {
    cr!("118.9a", "601.2b", "702.33d");
    ruling!(
        "Dream Harvest",
        "Since you are using an alternative cost to cast the spells, you can't pay any other alternative costs. You can, however, pay additional costs, such as kicker costs. If the card has any mandatory additional costs, you must pay those."
    );
    supported("Dream Harvest");
    let mut t = TestGame::new(2);
    let cards = stack_library(
        &mut t,
        P1,
        &["Burst Lightning", "Bone Splinters", "Snuff Out"],
    );
    let dh = t.hand(P0, "Dream Harvest");
    mana(&mut t, P0, ManaType::U, 7);
    t.cast(P0, dh).go();
    t.resolve_all();
    for c in &cards {
        assert!(exiled(&t, *c), "{}", t.dump_log());
    }
    // Snuff Out: only without paying its mana cost, not for its "pay 4 life" cost.
    t.battlefield(P0, "Swamp");
    let giant = t.battlefield(P1, "Hill Giant");
    let methods = legal_cast_methods(&mut t, P0, cards[2]);
    assert_eq!(methods, vec![CastMethod::Free], "{methods:?}");
    // Burst Lightning, kicked for {4}: 4 damage.
    mana(&mut t, P0, ManaType::C, 4);
    let b = t.g.current(cards[0]);
    t.cast(P0, b)
        .method(CastMethod::Free)
        .kicked(true)
        .target(P1)
        .go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Bone Splinters needs a creature to sacrifice.
    assert!(!castable(&mut t, P0, cards[1]));
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[o(bears)]);
    let b = t.g.current(cards[1]);
    t.cast(P0, b).method(CastMethod::Free).target(giant).go();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn kotis_free_casts_allow_kicker() {
    cr!("118.9a", "601.2b", "702.33d", "608.2g");
    ruling!(
        "Kotis, the Fangkeeper",
        "Since you are using an alternative cost to cast the spells, you can’t pay any other alternative costs. You can, however, pay additional costs, such as kicker costs. If the spells have any mandatory additional costs, you must pay those."
    );
    supported("Kotis, the Fangkeeper");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kotis, the Fangkeeper");
    stack_library(&mut t, P1, &["Burst Lightning", "Bone Splinters"]);
    let giant = t.battlefield(P1, "Hill Giant");
    // Mana for the kicker during the combat damage step.
    t.lands(P0, "Wastes", 4);
    crate::r_s22_common::choose_names_when_offered(
        &mut t,
        P0,
        &["Burst Lightning", "Bone Splinters"],
    );
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    // Bone Splinters: Kotis itself is the only creature P0 can sacrifice.
    t.answer_choose(P0, &[o(k)]);
    t.answer_targets(P0, &[o(giant)]);
    attack_with(&mut t, &[(k, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    // 2 combat damage, then a kicked Burst Lightning.
    assert_eq!(t.life(P1), 20 - 2 - 4, "{}", t.dump_log());
    assert!(t.in_graveyard(P1, "Bone Splinters"), "{}", t.dump_log());
    assert!(t.in_graveyard(P0, "Kotis, the Fangkeeper"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

/// Casts Crabomination for its emerge cost sacrificing `artifact`, with `pool` mana added
/// first. Returns the result of the cast.
fn emerge(t: &mut TestGame, artifact: ObjectId, pool: &[(ManaType, u32)]) -> bool {
    supported("Crabomination");
    for (ty, n) in pool {
        mana(t, P0, *ty, *n);
    }
    let crab = t.hand(P0, "Crabomination");
    t.answer_choose(P0, &[o(artifact)]);
    t.cast(P0, crab)
        .method(CastMethod::Keyword(KeywordKind::Emerge))
        .try_go()
        .is_ok()
}

#[test]
fn crabomination_sacrificed_artifact_can_make_mana_but_misses_cast_triggers() {
    cr!("601.2g", "601.2h", "601.2i", "702.119a");
    ruling!(
        "Crabomination",
        "The artifact chosen to be sacrificed is still on the battlefield up through the time that you activate mana abilities. Its abilities may affect the spell's cost, be activated to generate mana, and so on. However, if it has an ability that triggers when a spell is cast, it will have been sacrificed before that ability can trigger."
    );
    // Mind Stone (mana value 2): {3}{B}{B} left, with {B}{B}{C}{C} in the pool and Mind
    // Stone's own {C}.
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Mind Stone");
    assert!(
        emerge(&mut t, stone, &[(ManaType::B, 2), (ManaType::C, 2)]),
        "{}",
        t.dump_log()
    );
    assert!(t.in_graveyard(P0, "Mind Stone"));
    assert!(t.g.player(P0).mana_pool.is_empty());
    // Oketra's Monument ("Whenever you cast a creature spell, create a 1/1 white Warrior
    // creature token with vigilance."): sacrificed before it could trigger.
    let mut t = TestGame::new(2);
    let mon = t.battlefield(P0, "Oketra's Monument");
    assert!(emerge(&mut t, mon, &[(ManaType::B, 2), (ManaType::C, 2)]));
    t.settle();
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    t.resolve_all();
    assert!(t
        .g
        .permanents()
        .all(|p| !p.chars.has_subtype("Warrior")));
}

#[test]
fn crabomination_mana_value_of_back_faces_melded_permanents_and_copies() {
    cr!("712.8g", "712.4c", "202.3", "707.2");
    ruling!(
        "Crabomination",
        "The mana value of the back face of a double-faced card is the mana value of its front face. The mana value of a melded permanent is the sum of the mana values of its front faces. An artifact that's a copy of either has a mana value of 0."
    );
    // Edgar Markov's Coffin, the back face of Edgar, Charmed Groom ({2}{W}{B}): 4.
    let mut t = TestGame::new(2);
    let coffin = crate::r_s17_common::enter_transformed(
        &mut t,
        P0,
        "Edgar, Charmed Groom // Edgar Markov's Coffin",
    );
    assert_eq!(t.g.mana_value_of(coffin), 4);
    assert!(emerge(&mut t, coffin, &[(ManaType::B, 2), (ManaType::C, 1)]));
    // A Sol Ring that's a copy of it: 0 ({5}{B}{B} left).
    let mut t = TestGame::new(2);
    let coffin = crate::r_s17_common::enter_transformed(
        &mut t,
        P1,
        "Edgar, Charmed Groom // Edgar Markov's Coffin",
    );
    let ring = t.battlefield(P0, "Sol Ring");
    t.g.tap(ring);
    crate::r_s17_common::become_copy(&mut t, ring, coffin);
    assert_eq!(t.obj_now(ring).chars.name, "Edgar Markov's Coffin");
    assert_eq!(t.g.mana_value_of(ring), 0);
    assert!(!emerge(&mut t, ring, &[(ManaType::B, 2), (ManaType::C, 4)]));
    let mut t = TestGame::new(2);
    let coffin = crate::r_s17_common::enter_transformed(
        &mut t,
        P1,
        "Edgar, Charmed Groom // Edgar Markov's Coffin",
    );
    let ring = t.battlefield(P0, "Sol Ring");
    t.g.tap(ring);
    crate::r_s17_common::become_copy(&mut t, ring, coffin);
    assert!(emerge(&mut t, ring, &[(ManaType::B, 2), (ManaType::C, 5)]));
    // Mishra, Lost to Phyrexia, melded from Mishra, Claimed by Gix ({2}{B}{R}) and
    // Phyrexian Dragon Engine ({3}): 7.
    let mut t = TestGame::new(2);
    let mishra = t.exile(P0, "Mishra, Claimed by Gix");
    let engine = t.exile(P0, "Phyrexian Dragon Engine");
    let m = mtg_engine::merge::meld(
        &mut t.g,
        mishra,
        engine,
        mtg_engine::card::card("Mishra, Lost to Phyrexia"),
        P0,
    )
    .expect("meld");
    t.g.recompute();
    assert_eq!(t.g.mana_value_of(m), 7);
    // A Sol Ring copying it: 0.
    let ring = t.battlefield(P0, "Sol Ring");
    crate::r_s17_common::become_copy(&mut t, ring, m);
    assert_eq!(t.g.mana_value_of(ring), 0);
}

#[test]
fn urianger_reduces_only_generic_mana() {
    cr!("601.2f", "118.7d");
    ruling!(
        "Urianger Augurelt",
        "The cost reduction applies only to generic mana in the total cost of spells you cast with the permission granted by Urianger Augurelt's last ability."
    );
    let mut t = TestGame::new(2);
    let u = t.battlefield(P0, "Urianger Augurelt");
    let cards = stack_library(&mut t, P0, &["Lightning Bolt", "Divination"]);
    for _ in 0..2 {
        t.answer_yes(P0, true);
        t.g.obj_mut(u).tapped = false;
        t.activate(P0, u, 0, &[]).unwrap();
        t.resolve_all();
    }
    assert!(exiled(&t, cards[0]) && exiled(&t, cards[1]), "{}", t.dump_log());
    t.g.obj_mut(u).tapped = false;
    t.activate(P0, u, 1, &[]).unwrap();
    t.resolve_all();
    // Lightning Bolt still costs {R}; Divination costs {U}.
    must_pay(&mut t, P0, cards[0], ManaType::R, 1);
    must_pay(&mut t, P0, cards[1], ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 1);
    assert!(!castable(&mut t, P0, cards[1]));
}

#[test]
fn uvilda_reduction_doesnt_reduce_colored_mana() {
    cr!("601.2f", "118.7d");
    ruling!(
        "Uvilda, Dean of Perfection // Nassari, Dean of Expression",
        "The cost reduction given by Uvilda, Dean of Perfection won’t reduce any colored mana requirements in the spell’s cost."
    );
    // Divination ({2}{U}) costs {U}: not castable with no mana, castable with an Island.
    for islands in [0, 1] {
        let mut t = TestGame::new(2);
        let u = t.battlefield(P0, "Uvilda, Dean of Perfection // Nassari, Dean of Expression");
        let div = t.hand(P0, "Divination");
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[o(div)]);
        t.activate(P0, u, 0, &[]).unwrap();
        t.resolve_all();
        assert_eq!(t.counters(div, "refine"), 3, "{}", t.dump_log());
        t.lands(P0, "Island", islands);
        for _ in 0..3 {
            t.answer_yes(P0, true);
        }
        let hand = t.hand_size(P0);
        let d = t.g.current(div);
        t.g.remove_counters(Entity::Object(d), "refine", 3);
        t.settle();
        t.resolve_all();
        if islands == 0 {
            assert!(exiled(&t, div));
            assert_eq!(t.hand_size(P0), hand);
        } else {
            assert_eq!(t.hand_size(P0), hand + 2, "{}", t.dump_log());
        }
    }
}

#[test]
fn forgers_foundry_delayed_trigger_works_after_it_leaves() {
    cr!("603.7a", "603.7c");
    ruling!(
        "Forger's Foundry",
        "The delayed triggered ability will trigger whether Forger's Foundry is still on the battlefield or not."
    );
    supported("Forger's Foundry");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Forger's Foundry");
    t.activate(P0, f, 0, &[]).unwrap();
    t.g.destroy(f, None);
    t.settle();
    assert!(t.in_graveyard(P0, "Forger's Foundry"));
    t.library_top(P0, "Island");
    let opt = t.hand(P0, "Opt");
    t.answer_yes(P0, true);
    t.cast(P0, opt).go();
    t.resolve_all();
    assert!(exiled(&t, opt), "{}", t.dump_log());
}

#[test]
fn forgers_foundry_mana_can_be_spent_on_anything() {
    cr!("106.6");
    ruling!(
        "Forger's Foundry",
        "The mana added by Forger's Foundry can be spent on anything, not just instant and sorcery spells with mana value 3 or less."
    );
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Forger's Foundry");
    t.activate(P0, f, 0, &[]).unwrap();
    // Mind Stone ({2}), an artifact spell.
    mana(&mut t, P0, ManaType::C, 1);
    let stone = t.hand(P0, "Mind Stone");
    t.cast(P0, stone).go();
    assert!(t.g.player(P0).mana_pool.is_empty());
    t.resolve_all();
    assert!(t.on_battlefield(stone));
}

#[test]
fn ice_cauldron_mana_pays_additional_costs() {
    cr!("106.6", "601.2f", "702.33a");
    ruling!(
        "Ice Cauldron",
        "The mana can be used to pay for additional costs to cast the spell."
    );
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Ice Cauldron");
    let bl = t.hand(P0, "Burst Lightning");
    mana(&mut t, P0, ManaType::R, 5);
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    t.answer_choose(P0, &[o(bl)]);
    t.activate(P0, c, 0, &[]).unwrap();
    t.resolve_all();
    assert!(exiled(&t, bl));
    assert!(t.g.player(P0).mana_pool.is_empty());
    t.g.untap(c);
    t.activate(P0, c, 1, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.mana.len(), 5);
    let b = t.g.current(bl);
    t.cast(P0, b).kicked(true).target(P1).go();
    assert!(t.g.player(P0).mana_pool.is_empty());
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.counters(c, counters::CHARGE), 0);
}

#[test]
fn reanimate_life_loss_uses_the_card_in_the_graveyard() {
    cr!("608.2h", "722.3a");
    ruling!(
        "Grave Researcher // Reanimate",
        "The amount of life you lose as Reanimate resolves is determined by the mana value of the card in your graveyard, not the creature once it's on the battlefield."
    );
    supported("Grave Researcher // Reanimate");
    // The prepared copy of Reanimate returns Clone ({3}{U}), which enters as a copy of
    // Grizzly Bears (mana value 2): P0 loses 4 life.
    let mut t = TestGame::new(2);
    let gr = t.battlefield(P0, "Grave Researcher // Reanimate");
    t.battlefield(P1, "Grizzly Bears");
    let clone = t.graveyard(P0, "Clone");
    designations::become_prepared(&mut t.g, gr);
    let copy = t.obj(gr).prepared.expect("prepared");
    mana(&mut t, P0, ManaType::B, 1);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(t.named_on_battlefield("Grizzly Bears")[0])]);
    t.cast(P0, copy).target(clone).go();
    t.resolve_all();
    assert!(t.on_battlefield(clone), "{}", t.dump_log());
    assert_eq!(t.obj_now(clone).chars.name, "Grizzly Bears");
    assert_eq!(t.life(P0), 16);
}

#[test]
fn reanimate_card_life_loss_uses_the_card_in_the_graveyard() {
    cr!("608.2h");
    ruling!(
        "Reanimate",
        "The amount of life you lose is determined by the mana value of the card in your graveyard, not the creature once it's on the battlefield."
    );
    // The same with Reanimate itself, on a Clone in P1's graveyard.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let clone = t.graveyard(P1, "Clone");
    let r = t.hand(P0, "Reanimate");
    mana(&mut t, P0, ManaType::B, 1);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(bears)]);
    t.cast(P0, r).target(clone).go();
    t.resolve_all();
    assert_eq!(t.obj_now(clone).chars.name, "Grizzly Bears");
    assert_eq!(t.g.mana_value_of(t.g.current(clone)), 2);
    assert_eq!(t.life(P0), 16);
}
