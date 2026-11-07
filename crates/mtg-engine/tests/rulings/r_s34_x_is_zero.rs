//! Rulings batch S34 — an object with {X} in its mana cost that isn't a spell on the
//! stack (a permanent, a card in a library, a revealed or discarded card) has X = 0 in its
//! mana value (CR 202.3e, 107.3g); a card's mana value comes from its printed mana cost,
//! and a card with no mana cost has mana value 0 (CR 202.3, 202.3a).

use crate::r_s01_common::*;
use crate::r_s04_common::{next_upkeep, spell_targets};
use crate::r_s08_common::mana_value;
use crate::r_s34_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn witherbloom_charm_destroys_a_big_endless_one() {
    cr!("202.3e", "107.3g", "115.1");
    ruling!(
        "Witherbloom Charm",
        "If a permanent has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    supported("Witherbloom Charm");
    // "• Destroy target nonland permanent with mana value 2 or less." A 5/5 Endless One
    // has mana value 0.
    let mut t = TestGame::new(2);
    let one = p1_endless_one(&mut t, 5);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    let charm = t.hand(P0, "Witherbloom Charm");
    t.cast(P0, charm).modes(&[2]).target(one).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Endless One"));
}

#[test]
fn ulamog_s_dreadsire_ward_cant_be_paid_with_a_permanent_whose_x_is_0() {
    cr!("202.3e", "107.3g", "702.21a");
    ruling!(
        "Ulamog's Dreadsire",
        "If a permanent on the battlefield has {X} in its mana cost, X is 0 when determining its mana value."
    );
    supported("Ulamog's Dreadsire");
    // "Ward—Sacrifice a permanent with mana value 1 or greater." P1's only nonland
    // permanent is a 4/4 Endless One (mana value 0): P1 can't pay, and the Lightning Bolt
    // is countered.
    let mut t = TestGame::new(2);
    let dreadsire = t.battlefield(P0, "Ulamog's Dreadsire");
    let one = p1_endless_one(&mut t, 4);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(dreadsire).go();
    t.answer_yes(P1, true);
    t.resolve_all();
    assert!(t.on_battlefield(one));
    assert_eq!(t.obj_now(dreadsire).damage, 0);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    // With Grizzly Bears (mana value 2) P1 can pay.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(dreadsire).go();
    t.answer_yes(P1, true);
    t.resolve_all();
    assert!(!t.g.is_live(bears));
    assert!(t.on_battlefield(one));
    assert_eq!(t.obj_now(dreadsire).damage, 3);
}

#[test]
fn helios_one_for_x_0_destroys_a_permanent_whose_x_is_0() {
    cr!("202.3e", "107.3g", "107.3k");
    ruling!(
        "HELIOS One",
        "If a permanent has {X} in its mana cost, X is 0 when determining its mana value."
    );
    supported("HELIOS One");
    // "{3}, {T}, Pay X {E}, Sacrifice this land: Destroy target nonland permanent with
    // mana value X. Activate only as a sorcery." With X = 0 (no energy paid), it destroys
    // a 4/4 Endless One.
    let mut t = TestGame::new(2);
    let one = p1_endless_one(&mut t, 4);
    let helios = t.battlefield(P0, "HELIOS One");
    t.lands(P0, "Wastes", 3);
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    t.answer_targets(P0, &[Entity::Object(one)]);
    crate::r_s06_common::activate_containing(&mut t, P0, helios, "Destroy target")
        .expect("activate");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Endless One"));
    assert!(t.in_graveyard(P0, "HELIOS One"));
}

#[test]
fn pest_control_destroys_a_big_endless_one() {
    cr!("202.3e", "107.3g");
    ruling!(
        "Pest Control",
        "If the mana cost of a permanent includes {X}, X is 0 for the purpose of determining its mana value."
    );
    supported("Pest Control");
    // "Destroy all nonland permanents with mana value 1 or less." The 5/5 Endless One
    // (mana value 0) is destroyed; Grizzly Bears (2) isn't.
    let mut t = TestGame::new(2);
    p1_endless_one(&mut t, 5);
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s25_common::cast_new(&mut t, P0, "Pest Control", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Endless One"));
    assert!(t.on_battlefield(bears));
}

#[test]
fn starfield_shepherd_finds_a_card_whose_x_is_0_in_the_library() {
    cr!("202.3e", "107.3g", "701.23a");
    ruling!(
        "Starfield Shepherd",
        "If a card in your library has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    supported("Starfield Shepherd");
    // "When this creature enters, search your library for a basic Plains card or a
    // creature card with mana value 1 or less, reveal it, put it into your hand, then
    // shuffle." Endless One ({X}) in the library has mana value 0.
    let mut t = TestGame::new(2);
    let one = t.library_top(P0, "Endless One");
    t.library_top(P0, "Hill Giant");
    assert_eq!(mana_value(&t, one), 0);
    t.answer_choose(P0, &[Entity::Object(one)]);
    t.enter(P0, "Starfield Shepherd");
    t.resolve_all();
    assert!(t.in_hand(P0, "Endless One"));
}

#[test]
fn dark_confidant_reveals_a_card_whose_x_is_0() {
    cr!("202.3e", "107.3g");
    ruling!(
        "Dark Confidant",
        "If a card in a player's library has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    supported("Dark Confidant");
    // "At the beginning of your upkeep, reveal the top card of your library and put that
    // card into your hand. You lose life equal to its mana value." Chalice of the Void
    // ({X}{X}) has mana value 0.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dark Confidant");
    next_upkeep(&mut t, P0);
    t.library_top(P0, "Chalice of the Void");
    t.resolve_all();
    assert!(t.in_hand(P0, "Chalice of the Void"));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn dark_tutelage_counts_the_printed_mana_symbols() {
    cr!("202.3");
    ruling!(
        "Dark Tutelage",
        "The mana value of the revealed card is determined solely by the mana symbols printed in its upper right corner. The mana value is the total amount of mana in that cost, regardless of color. For example, a card with mana cost {3}{U}{U} has mana value 5."
    );
    supported("Dark Tutelage");
    // Force of Will ({3}{U}{U}): P0 loses 5 life, though Force of Will could be cast for
    // no mana.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dark Tutelage");
    next_upkeep(&mut t, P0);
    t.library_top(P0, "Force of Will");
    t.resolve_all();
    assert!(t.in_hand(P0, "Force of Will"));
    assert_eq!(t.life(P0), 15);
}

#[test]
fn pain_seer_reveals_a_land_with_mana_value_0() {
    cr!("202.3a", "502.3");
    ruling!(
        "Pain Seer",
        "If the revealed card doesn't have a mana cost (because it's a land card, for example), its mana value is 0."
    );
    supported("Pain Seer");
    // "Inspired — Whenever this creature becomes untapped, reveal the top card of your
    // library and put that card into your hand. You lose life equal to that card's mana
    // value." A Forest: no life lost. (Hill Giant: 4.)
    for (top, life) in [("Forest", 20), ("Ancestral Vision", 20), ("Hill Giant", 16)] {
        let mut t = TestGame::new(2);
        let seer = t.battlefield(P0, "Pain Seer");
        t.g.tap(seer);
        next_upkeep(&mut t, P0);
        t.library_top(P0, top);
        t.resolve_all();
        assert!(t.in_hand(P0, top), "{top}");
        assert_eq!(t.life(P0), life, "{top}");
    }
}

#[test]
fn roadside_blowout_costs_less_targeting_an_x_creature_with_mana_value_1() {
    cr!("202.3e", "107.3g", "601.2f");
    ruling!(
        "Roadside Blowout",
        "If there’s an {X} in a permanent’s mana cost, X is 0 when determining its mana value."
    );
    supported("Roadside Blowout");
    // "This spell costs {2} less to cast if it targets a permanent with mana value 1.
    // Return target creature or Vehicle an opponent controls to its owner's hand. Draw a
    // card." P1's Ingenious Prodigy ({X}{U}) cast with X = 3 has mana value 1: {U}.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let prodigy = crate::r_s27_common::prodigy_x3(&mut t, P1);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    let blowout = t.hand(P0, "Roadside Blowout");
    let hand = t.hand_size(P0);
    t.cast(P0, blowout).target(prodigy).go();
    assert_eq!(tapped_lands(&t, P0), 1);
    t.resolve_all();
    assert!(t.in_hand(P1, "Ingenious Prodigy"));
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn daretti_s_power_counts_an_artifact_s_x_as_0() {
    cr!("202.3e", "107.3g", "604.3");
    ruling!(
        "Daretti, Rocketeer Engineer",
        "If an artifact on the battlefield has {X} in its mana cost, X is 0 when determining its mana value."
    );
    supported("Daretti, Rocketeer Engineer");
    // "Daretti's power is equal to the greatest mana value among artifacts you control."
    // Chalice of the Void cast with X = 2 has mana value 0; Bonesplitter has 1.
    let mut t = TestGame::new(2);
    let daretti = t.battlefield(P0, "Daretti, Rocketeer Engineer");
    crate::r_s27_common::chalice_x2(&mut t, P0);
    t.g.recompute();
    assert_eq!(t.pt(daretti), (0, 5));
    t.battlefield(P0, "Bonesplitter");
    t.g.recompute();
    assert_eq!(t.pt(daretti), (1, 5));
}

#[test]
fn ruinous_rampage_exiles_an_artifact_whose_x_is_0() {
    cr!("202.3e", "107.3g");
    ruling!(
        "Ruinous Rampage",
        "If an artifact has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    supported("Ruinous Rampage");
    // "• Exile all artifacts with mana value 3 or less." Chalice of the Void cast with
    // X = 2 (mana value 0) is exiled; Juggernaut (4) isn't.
    let mut t = TestGame::new(2);
    crate::r_s27_common::chalice_x2(&mut t, P0);
    let juggernaut = t.battlefield(P1, "Juggernaut");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 1);
    let rampage = t.hand(P0, "Ruinous Rampage");
    t.cast(P0, rampage).modes(&[1]).go();
    t.resolve_all();
    assert!(t.in_exile("Chalice of the Void"));
    assert!(t.on_battlefield(juggernaut));
}

#[test]
fn doc_ock_s_tentacles_ignores_a_big_endless_one() {
    cr!("202.3e", "107.3g", "603.2");
    ruling!(
        "Doc Ock's Tentacles",
        "If a creature has {X} in its mana cost, X is 0 when determining its mana value."
    );
    supported("Doc Ock's Tentacles");
    // "Whenever a creature you control with mana value 5 or greater enters, you may
    // attach this Equipment to it." A 6/6 Endless One (mana value 0) doesn't trigger it;
    // Craw Wurm (6) does.
    let mut t = TestGame::new(2);
    let tentacles = t.battlefield(P0, "Doc Ock's Tentacles");
    let one = endless_one(&mut t, P0, 6);
    assert!(crate::r_s06_common::attached_to(&t, tentacles).is_none());
    assert_eq!(t.pt(one), (6, 6));
    t.answer_yes(P0, true);
    let wurm = t.enter(P0, "Craw Wurm");
    t.resolve_all();
    assert_eq!(
        crate::r_s06_common::attached_to(&t, tentacles),
        Some(Entity::Object(wurm))
    );
    assert_eq!(t.pt(wurm), (10, 8));
}

#[test]
fn requiting_hex_destroys_a_big_endless_one() {
    cr!("202.3e", "107.3g", "115.1");
    ruling!(
        "Requiting Hex",
        "If a creature on the battlefield has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    supported("Requiting Hex");
    // "Destroy target creature with mana value 2 or less." A 5/5 Endless One.
    let mut t = TestGame::new(2);
    let one = p1_endless_one(&mut t, 5);
    let giant = t.battlefield(P1, "Hill Giant");
    let targets = spell_targets(&mut t, P0, "Requiting Hex");
    assert!(targets.contains(&Entity::Object(one)));
    assert!(!targets.contains(&Entity::Object(giant)));
    t.lands(P0, "Swamp", 1);
    let hex = t.hand(P0, "Requiting Hex");
    t.cast(P0, hex).target(one).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Endless One"));
}

#[test]
fn death_in_the_family_exiles_a_big_endless_one() {
    cr!("202.3e", "107.3g", "115.1");
    ruling!(
        "Death in the Family",
        "If a creature has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    supported("Death in the Family");
    // "Exile target creature with mana value 3 or less." A 5/5 Endless One.
    let mut t = TestGame::new(2);
    let one = p1_endless_one(&mut t, 5);
    crate::r_s25_common::cast_new(&mut t, P0, "Death in the Family", &[Entity::Object(one)]);
    t.resolve_all();
    assert!(t.in_exile("Endless One"));
}

#[test]
fn summon_kujata_deals_damage_for_a_discarded_card_with_x_0() {
    cr!("202.3e", "107.3g", "714.2b", "603.12");
    ruling!(
        "Summon: Kujata",
        "If the discarded card has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    supported("Summon: Kujata");
    // "III — Fire — Discard a card, then draw two cards. When you discard a card this
    // way, this creature deals damage equal to that card's mana value to each opponent."
    // Blaze ({X}{R}) has mana value 1.
    let mut t = TestGame::new(2);
    let kujata = t.battlefield(P0, "Summon: Kujata");
    crate::r_s19_common::add_lore(&mut t, kujata, 2);
    t.resolve_all();
    let blaze = t.hand(P0, "Blaze");
    t.answer_choose(P0, &[Entity::Object(blaze)]);
    crate::r_s19_common::add_lore(&mut t, kujata, 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Blaze"));
    assert_eq!(t.life(P1), 19);
}
