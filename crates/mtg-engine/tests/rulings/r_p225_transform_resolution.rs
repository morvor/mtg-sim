//! Rulings batch P225 — "then if ..., transform it" instructions that are part of a
//! resolving ability (CR 608.2c): the condition is checked only as the ability resolves,
//! so meeting it some other way doesn't transform the permanent; intervening "if" clauses
//! (CR 603.4); costs paid during resolution (CR 608.2c, 118.3, 605.3a); mana left in a
//! pool until the step ends (CR 106.4); and replacement effects on the milled card
//! (CR 614.6).

use crate::r_s01_common::*;
use crate::r_s06_common::activate_containing;
use crate::r_s13_common::add;
use crate::r_s17_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const AMULET: &str = "Primal Amulet // Primal Wellspring";
const AZCANTA: &str = "Search for Azcanta // Azcanta, the Sunken Ruin";
const MAP: &str = "Treasure Map // Treasure Cove";
const DOCENT: &str = "Docent of Perfection // Final Iteration";
const HANWEIR: &str = "Hanweir Militia Captain // Westvale Cult Leader";
const ANGLER: &str = "Grizzled Angler // Grisly Anglerfish";
const THING: &str = "Thing in the Ice // Awoken Horror";
const RESEARCHER: &str = "Aberrant Researcher // Perfected Form";
const JACOB: &str = "Jacob Hauken, Inspector // Hauken's Insight";
const JERREN: &str = "Jerren, Corrupted Bishop // Ormendahl, the Corrupter";
const HOMUNCULUS: &str = "Curious Homunculus // Voracious Reader";
const NISSA: &str = "Nissa, Vastwood Seer // Nissa, Sage Animist";

/// Library cards so turns can pass without decking.
fn stock_library(t: &mut TestGame, p: PlayerId) {
    for _ in 0..5 {
        t.library_top(p, "Island");
    }
}

#[test]
fn meeting_a_transform_condition_some_other_way_doesnt_transform_it() {
    cr!("608.2c", "603.2");
    ruling!(
        "Primal Amulet // Primal Wellspring",
        "If a fourth charge counter is put on Primal Amulet by something other than the resolution of its ability"
    );
    ruling!(
        "Search for Azcanta // Azcanta, the Sunken Ruin",
        "If a seventh card is put into your graveyard by something other than resolving Search for Azcanta's triggered ability"
    );
    ruling!(
        "Treasure Map // Treasure Cove",
        "If a third landmark counter is put on Treasure Map by something other than the resolution of its first ability"
    );
    ruling!(
        "Docent of Perfection // Final Iteration",
        "If you control three or more Wizards while you control Docent of Perfection, it won’t transform yet."
    );
    ruling!(
        "Grizzled Angler // Grisly Anglerfish",
        "If you have a colorless creature card in your graveyard while you control Grizzled Angler, it won’t transform yet."
    );
    ruling!(
        "Thing in the Ice // Awoken Horror",
        "Removing all ice counters from Thing in the Ice some other way will not cause it to transform."
    );
    for name in [AMULET, AZCANTA, MAP, DOCENT, ANGLER, THING] {
        supported(name);
    }
    // Primal Amulet: four counters from elsewhere; the next instant transforms it.
    let mut t = TestGame::new(2);
    let amulet = t.battlefield(P0, AMULET);
    add(&mut t, amulet, "charge", 4);
    t.settle();
    assert_eq!(name_of(&t, amulet), "Primal Amulet");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(name_of(&t, amulet), "Primal Wellspring");
    // Search for Azcanta: seven cards in the graveyard; it waits for the upkeep.
    let mut t = TestGame::new(2);
    stock_library(&mut t, P0);
    stock_library(&mut t, P1);
    let search = t.battlefield(P0, AZCANTA);
    for _ in 0..7 {
        t.graveyard(P0, "Grizzly Bears");
    }
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(name_of(&t, search), "Search for Azcanta");
    // Treasure Map: three counters from elsewhere; its next activation transforms it.
    let mut t = TestGame::new(2);
    let map = t.battlefield(P0, MAP);
    add(&mut t, map, "landmark", 3);
    t.settle();
    assert_eq!(name_of(&t, map), "Treasure Map");
    assert!(with_subtype(&t, P0, "Treasure").is_empty());
    t.lands(P0, "Wastes", 1);
    activate_containing(&mut t, P0, map, "Scry").unwrap();
    t.resolve_all();
    assert_eq!(name_of(&t, map), "Treasure Cove");
    assert_eq!(with_subtype(&t, P0, "Treasure").len(), 3);
    assert_eq!(t.counters(map, "landmark"), 0);
    // Docent of Perfection: three Wizards without casting a spell.
    let mut t = TestGame::new(2);
    let docent = t.battlefield(P0, DOCENT);
    for _ in 0..3 {
        crate::r_s02_common::create_token(&mut t, P0, "Wizard");
    }
    t.settle();
    assert_eq!(name_of(&t, docent), "Docent of Perfection");
    // Grizzled Angler: a colorless creature card in the graveyard; its activation
    // transforms it.
    let mut t = TestGame::new(2);
    let angler = t.battlefield(P0, ANGLER);
    t.graveyard(P0, "Ornithopter");
    stock_library(&mut t, P0);
    t.settle();
    assert_eq!(name_of(&t, angler), "Grizzled Angler");
    activate_containing(&mut t, P0, angler, "Mill").unwrap();
    t.resolve_all();
    assert_eq!(name_of(&t, angler), "Grisly Anglerfish");
    // Thing in the Ice: its ice counters removed some other way.
    let mut t = TestGame::new(2);
    let thing = t.battlefield(P0, THING);
    add(&mut t, thing, "ice", 4);
    t.g.remove_counters(Entity::Object(thing), "ice", 4);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.counters(thing, "ice"), 0);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(name_of(&t, thing), "Thing in the Ice");
}

#[test]
fn search_for_azcanta_may_transform_without_milling_the_card() {
    cr!("701.25a", "608.2c");
    ruling!(
        "Search for Azcanta // Azcanta, the Sunken Ruin",
        "you may transform Search for Azcanta while resolving its triggered ability even if you choose not to put the top card of your library into your graveyard"
    );
    supported(AZCANTA);
    let mut t = TestGame::new(2);
    stock_library(&mut t, P0);
    stock_library(&mut t, P1);
    let search = t.battlefield(P0, AZCANTA);
    for _ in 0..7 {
        t.graveyard(P0, "Grizzly Bears");
    }
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let top = *t.g.player(P0).library.last().unwrap();
    t.answer(P0, DecisionKind::Surveil, Answer::Split(vec![top], vec![]));
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.zone(top), Zone::Library(P0));
    assert_eq!(t.graveyard_size(P0), 7);
    assert_eq!(name_of(&t, search), "Azcanta, the Sunken Ruin");
}

#[test]
fn hanweir_militia_captain_checks_on_resolution_and_doesnt_transform_back() {
    cr!("603.4");
    ruling!(
        "Hanweir Militia Captain // Westvale Cult Leader",
        "If you don't control four or more creatures as Hanweir Militia Captain's ability resolves, it won't transform."
    );
    ruling!(
        "Hanweir Militia Captain // Westvale Cult Leader",
        "controlling fewer than four creatures won't cause it to transform back"
    );
    supported(HANWEIR);
    let mut t = TestGame::new(2);
    stock_library(&mut t, P0);
    stock_library(&mut t, P1);
    let captain = t.battlefield(P0, HANWEIR);
    let bears: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    crate::r_s02_common::destroy(&mut t, bears[0]);
    t.resolve_all();
    assert_eq!(name_of(&t, captain), "Hanweir Militia Captain");
    // Four creatures at both times: it transforms; with fewer later, it stays.
    t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(name_of(&t, captain), "Westvale Cult Leader");
    for b in t.named_on_battlefield("Grizzly Bears") {
        crate::r_s02_common::destroy(&mut t, b);
    }
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(name_of(&t, captain), "Westvale Cult Leader");
}

#[test]
fn aberrant_researcher_transforms_even_if_the_milled_card_is_exiled_instead() {
    cr!("701.17a", "614.6", "608.2c");
    ruling!(
        "Aberrant Researcher // Perfected Form",
        "If a replacement effect causes the top card of your library to go to a zone other than your graveyard"
    );
    ruling!(
        "Aberrant Researcher // Perfected Form",
        "No player may take any action between the two steps of Aberrant Researcher's triggered ability."
    );
    supported(RESEARCHER);
    supported("Rest in Peace");
    for rip in [false, true] {
        let mut t = TestGame::new(2);
        stock_library(&mut t, P1);
        let r = t.battlefield(P0, RESEARCHER);
        if rip {
            t.battlefield(P0, "Rest in Peace");
        }
        for _ in 0..3 {
            t.library_top(P0, "Island");
        }
        t.library_top(P0, "Lightning Bolt");
        t.advance_to(P1, Step::Upkeep);
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        // One resolution: milled and transformed.
        t.resolve();
        assert_eq!(name_of(&t, r), "Perfected Form", "rip: {rip}");
        assert_eq!(t.in_exile("Lightning Bolt"), rip);
    }
}

#[test]
fn jacob_hauken_draws_and_exiles_before_the_payment_choice() {
    cr!("608.2c", "118.3");
    ruling!(
        "Jacob Hauken, Inspector // Hauken's Insight",
        "You draw a card and exile a card before choosing whether to pay."
    );
    supported(JACOB);
    let mut t = TestGame::new(2);
    let jacob = t.battlefield(P0, JACOB);
    t.library_top(P0, "Island");
    let card = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 4);
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::YesNo { .. } | Decision::OptionalCost { .. }),
        |g| (g.player(P0).hand.len(), g.exile.len()),
    );
    t.answer_choose(P0, &[Entity::Object(card)]);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    activate_containing(&mut t, P0, jacob, "Draw a card").unwrap();
    t.resolve_all();
    assert_eq!(seen.lock().unwrap().first().copied(), Some((1, 1)));
    assert_eq!(name_of(&t, jacob), "Hauken's Insight");
}

#[test]
fn jerren_needs_exactly_thirteen_life_on_trigger_and_resolution() {
    cr!("603.4", "605.3a", "118.3");
    ruling!(
        "Jerren, Corrupted Bishop // Ormendahl, the Corrupter",
        "If your life total isn't 13 at that point, the ability will have no effect."
    );
    ruling!(
        "Jerren, Corrupted Bishop // Ormendahl, the Corrupter",
        "you will still be able to finish paying the cost and Jerren will still transform"
    );
    supported(JERREN);
    let setup = |life: i32| {
        let mut t = TestGame::new(2);
        t.g.players[0].life = life;
        let jerren = t.battlefield(P0, JERREN);
        t.lands(P0, "Wastes", 4);
        t.lands(P0, "Swamp", 1);
        t.lands(P0, "Caves of Koilos", 1);
        t.advance_to(P0, Step::End);
        t.settle();
        (t, jerren)
    };
    // 12 life: no trigger.
    let (t, _) = setup(12);
    assert_eq!(t.stack_len(), 0);
    // 13 life, then 12 before it resolves: nothing happens.
    let (mut t, jerren) = setup(13);
    assert_eq!(t.stack_len(), 1);
    t.g.lose_life(P0, 1);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.resolve_all();
    assert_eq!(name_of(&t, jerren), "Jerren, Corrupted Bishop");
    assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 6);
    // 13 life on resolution: paying with Caves of Koilos (which deals 1 damage to P0 as
    // it's tapped for {B}) still transforms Jerren.
    let (mut t, jerren) = setup(13);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.resolve_all();
    assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 0);
    assert_eq!(t.life(P0), 12);
    assert_eq!(name_of(&t, jerren), "Ormendahl, the Corrupter");
}

#[test]
fn curious_homunculus_mana_made_in_response_lasts_through_the_upkeep() {
    cr!("106.4", "605.3a", "601.2g");
    ruling!(
        "Curious Homunculus // Voracious Reader",
        "If you activate Curious Homunculus's mana ability in response to its triggered ability"
    );
    supported(HOMUNCULUS);
    supported("Warping Wail");
    for cast in [true, false] {
        let mut t = TestGame::new(2);
        stock_library(&mut t, P0);
        stock_library(&mut t, P1);
        let h = t.battlefield(P0, HOMUNCULUS);
        for _ in 0..3 {
            t.graveyard(P0, "Lightning Bolt");
        }
        let wail = t.hand(P0, "Warping Wail");
        t.advance_to(P1, Step::Upkeep);
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        activate_containing(&mut t, P0, h, "Add {C}").unwrap();
        assert_eq!(t.g.player(P0).mana_pool.count(ManaType::C), 1);
        t.resolve_all();
        assert_eq!(name_of(&t, h), "Voracious Reader");
        assert_eq!(t.g.player(P0).mana_pool.count(ManaType::C), 1);
        if cast {
            // Warping Wail ({1}{C}, {1} less): paid with that {C}.
            t.cast(P0, wail).modes(&[2]).go();
            t.resolve_all();
            assert_eq!(with_subtype(&t, P0, "Scion").len(), 1);
            assert_eq!(t.g.player(P0).mana_pool.count(ManaType::C), 0);
        } else {
            t.advance_to(P0, Step::Draw);
            assert_eq!(t.g.player(P0).mana_pool.count(ManaType::C), 0);
        }
    }
}

#[test]
fn nissa_entering_with_seven_lands_isnt_exiled_or_transformed() {
    cr!("603.2");
    ruling!(
        "Nissa, Vastwood Seer // Nissa, Sage Animist",
        "If she enters the battlefield while you control seven or more lands, she won't automatically be exiled and transform."
    );
    supported(NISSA);
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 7);
    t.answer_yes(P0, false);
    let nissa = t.enter(P0, NISSA);
    t.resolve_all();
    assert!(t.on_battlefield(nissa));
    assert_eq!(name_of(&t, nissa), "Nissa, Vastwood Seer");
    // The next land does it.
    t.enter(P0, "Forest");
    t.resolve_all();
    assert_eq!(name_of(&t, nissa), "Nissa, Sage Animist");
}
