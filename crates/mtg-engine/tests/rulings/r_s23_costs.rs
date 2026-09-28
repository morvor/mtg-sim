//! Rulings batch S23 — the total cost of a spell (CR 601.2f): additional costs are added
//! before cost reductions apply, generic-only reductions don't reduce colored mana, and
//! mana that may be spent only to cast spells can't pay costs imposed after the spell was
//! cast (CR 106.6).

use crate::r_s01_common::*;
use crate::r_s05_common::enter;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
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
