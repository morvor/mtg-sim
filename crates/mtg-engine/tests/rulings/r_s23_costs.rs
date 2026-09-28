//! Rulings batch S23 — the total cost of a spell (CR 601.2f): additional costs are added
//! before cost reductions apply, generic-only reductions don't reduce colored mana, and
//! mana that may be spent only to cast spells can't pay costs imposed after the spell was
//! cast (CR 106.6).

use crate::r_s01_common::*;
use crate::r_s05_common::enter;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn council_of_the_absolute_reduction_applies_after_kicker_is_added() {
    cr!("601.2f", "118.8", "702.33a");
    ruling!(
        "Council of the Absolute",
        "If there are additional costs to cast the spell, such as kicker costs, apply those increases before applying cost reductions."
    );
    supported("Council of the Absolute");
    // "As this creature enters, choose a noncreature, nonland card name. ... Spells with
    // the chosen name you cast cost {2} less to cast." Burst Lightning ({R}, kicker {4})
    // kicked costs {R}{4} - {2} = {2}{R}.
    let mut t = TestGame::new(2);
    t.answer(
        P0,
        DecisionKind::Name,
        Answer::Text("Burst Lightning".into()),
    );
    enter(&mut t, P0, "Council of the Absolute");
    t.resolve_all();
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    let burst = t.hand(P0, "Burst Lightning");
    t.cast(P0, burst)
        .kicked(true)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(tapped_lands(&t, P0), 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Unkicked, it costs {R}: the reduction can't reduce its colored mana.
    let burst = t.hand(P0, "Burst Lightning");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, burst)
        .kicked(false)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(tapped_lands(&t, P0), 4);
}

#[test]
fn lyse_hext_reduces_only_generic_mana_of_noncreature_spells() {
    cr!("601.2f", "118.7", "118.7d");
    ruling!(
        "Lyse Hext",
        "The cost reduction applies only to generic mana in the total cost of noncreature spells you cast."
    );
    supported("Lyse Hext");
    // "Noncreature spells you cast cost {1} less to cast."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lyse Hext");
    // Divination ({2}{U}) costs {1}{U}.
    t.lands(P0, "Island", 2);
    let divination = t.hand(P0, "Divination");
    t.cast(P0, divination).go();
    assert_eq!(tapped_lands(&t, P0), 2);
    t.resolve_all();
    // Lightning Bolt ({R}) still costs {R}: without red mana it can't be cast.
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).target(Entity::Player(P1)).try_go().is_err());
    // Burst Lightning kicked: the total cost {4}{R} is reduced to {3}{R}.
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 3);
    let burst = t.hand(P0, "Burst Lightning");
    t.cast(P0, burst)
        .kicked(true)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(tapped_lands(&t, P0), 6);
    t.resolve_all();
    // A creature spell isn't reduced: Grizzly Bears ({1}{G}) can't be cast with a Forest.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lyse Hext");
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
}

#[test]
fn mishras_workshop_mana_cant_pay_a_cost_imposed_after_casting() {
    cr!("106.6", "601.2g", "118.12");
    ruling!(
        "Mishra's Workshop",
        "This mana may not be used to pay costs imposed after the spell is initially cast."
    );
    supported("Mishra's Workshop");
    supported("Mana Leak");
    // "{T}: Add {C}{C}{C}. Spend this mana only to cast artifact spells." P0 adds the mana
    // and casts Memnite ({0}); P1's Mana Leak asks P0 to pay {3}: the Workshop's mana
    // can't pay it.
    let mut t = TestGame::new(2);
    let workshop = t.battlefield(P0, "Mishra's Workshop");
    t.activate(P0, workshop, 0, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::C), 3);
    let memnite = t.hand(P0, "Memnite");
    let spell = t.cast(P0, memnite).go();
    t.lands(P1, "Island", 2);
    let leak = t.hand(P1, "Mana Leak");
    t.cast(P1, leak).target(Entity::Object(spell)).go();
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.in_graveyard(P0, "Memnite"));
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::C), 3);
    // Unrestricted mana can pay it.
    let mut t = TestGame::new(2);
    crate::r_s04_common::add_mana(&mut t, P0, ManaType::C, 3);
    let memnite = t.hand(P0, "Memnite");
    let spell = t.cast(P0, memnite).go();
    t.lands(P1, "Island", 2);
    let leak = t.hand(P1, "Mana Leak");
    t.cast(P1, leak).target(Entity::Object(spell)).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Memnite").len(), 1);
    assert!(t.g.player(P0).mana_pool.is_empty());
}

#[test]
fn conduit_of_ruin_x_is_chosen_before_the_first_creature_spells_discount() {
    cr!("601.2f", "107.3b", "118.7a");
    ruling!(
        "Conduit of Ruin",
        "If the first creature spell you cast in a turn has {X} in its mana cost, you choose the value of X before calculating the spell's total cost. For example, if the first creature spell you cast in a turn has a mana cost of {X}{G}, you could choose 2 as the value of X and pay {G} to cast the spell."
    );
    supported("Conduit of Ruin");
    supported("Mistcutter Hydra");
    // "The first creature spell you cast each turn costs {2} less to cast." Mistcutter
    // Hydra ({X}{G}, "enters with X +1/+1 counters") with X = 2 costs {G}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Conduit of Ruin");
    t.lands(P0, "Forest", 1);
    let hydra = t.hand(P0, "Mistcutter Hydra");
    t.cast(P0, hydra).x(2).go();
    assert_eq!(tapped_lands(&t, P0), 1);
    t.resolve_all();
    assert_eq!(t.counters(hydra, "+1/+1"), 2);
}

#[test]
fn conduit_of_ruin_the_first_creature_spell_neednt_be_the_first_spell() {
    cr!("601.2f", "118.7a");
    ruling!(
        "Conduit of Ruin",
        "The first creature spell you cast each turn doesn't necessarily have to be the first spell you cast. You could cast a sorcery spell and then cast a creature spell that would get the discount."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Conduit of Ruin");
    // A sorcery first: Divination ({2}{U}) isn't discounted.
    t.lands(P0, "Island", 3);
    let divination = t.hand(P0, "Divination");
    t.cast(P0, divination).go();
    assert_eq!(tapped_lands(&t, P0), 3);
    t.resolve_all();
    // Then Hill Giant ({3}{R}) costs {1}{R}.
    t.lands(P0, "Mountain", 2);
    let giant = t.hand(P0, "Hill Giant");
    t.cast(P0, giant).go();
    assert_eq!(tapped_lands(&t, P0), 5);
    t.resolve_all();
    // The second creature spell costs its full cost: Grizzly Bears ({1}{G}) can't be
    // cast with one Forest.
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
    // Next turn, the discount is back.
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    let bears = t.g.find_in_zone(mtg_engine::object::Zone::Hand(P0), "Grizzly Bears")[0];
    let untapped_before = crate::r_s04_common::untapped_lands(&t, P0);
    t.cast(P0, bears).go();
    assert_eq!(
        crate::r_s04_common::untapped_lands(&t, P0),
        untapped_before - 1
    );
}

#[test]
fn sage_of_the_beyond_reduces_only_generic_mana_of_spells_cast_from_elsewhere() {
    cr!("601.2f", "118.7a", "702.34a");
    ruling!(
        "Sage of the Beyond",
        "The cost reduction applies only to generic mana in the costs of spells you cast from anywhere other than your hand. It can't reduce requirements of a specific color of mana."
    );
    supported("Sage of the Beyond");
    supported("Think Twice");
    supported("Lingering Souls");
    // "Spells you cast from anywhere other than your hand cost {2} less to cast."
    let flashback = mtg_engine::object::CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Flashback);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sage of the Beyond");
    // Think Twice's flashback cost {2}{U} is reduced to {U}.
    t.lands(P0, "Island", 1);
    let tt = t.graveyard(P0, "Think Twice");
    t.cast(P0, tt).method(flashback.clone()).go();
    assert_eq!(tapped_lands(&t, P0), 1);
    t.resolve_all();
    // Lingering Souls' flashback cost {1}{B} is reduced to {B}, not less.
    let souls = t.graveyard(P0, "Lingering Souls");
    assert!(t.cast(P0, souls).method(flashback.clone()).try_go().is_err());
    t.lands(P0, "Swamp", 1);
    t.cast(P0, souls).method(flashback).go();
    assert_eq!(tapped_lands(&t, P0), 2);
    t.resolve_all();
    // Cast from the hand, Think Twice costs its full {1}{U}.
    t.lands(P0, "Island", 1);
    let tt = t.hand(P0, "Think Twice");
    assert!(t.cast(P0, tt).try_go().is_err());
    t.lands(P0, "Island", 1);
    t.cast(P0, tt).go();
    assert_eq!(tapped_lands(&t, P0), 4);
}
