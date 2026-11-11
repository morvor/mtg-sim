//! CR 601.2f audit: the order in which a spell's total cost is determined, with real
//! cards. The total cost is the mana cost or alternative cost plus the additional costs
//! and cost increases, minus the cost reductions (all of them after all of the increases,
//! CR 601.2f, 118.9d), with effects that directly affect the total (Trinisphere's floor)
//! applied after that; an effect that reduces an alternative keyword cost ("Flashback costs
//! you pay cost {2} less") reduces it after the cost increases applied to it (CR 118.9d,
//! `keyword-cost-reduction-on-an-alternative-cost-after-increases`); convoke and delve
//! then pay part of that total (CR 702.51b, 702.66b),
//! and the total is locked in before the mana abilities are activated and the costs paid
//! (CR 601.2g–h).

use crate::r601_audit_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::kw::offered_costs::OFFERED_ALT_COST;
use mtg_engine::mana::ManaType;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const OFFERED: CastMethod = CastMethod::Alternative(OFFERED_ALT_COST);
const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);
const EMERGE: CastMethod = CastMethod::Keyword(KeywordKind::Emerge);
const DASH: CastMethod = CastMethod::Keyword(KeywordKind::Dash);

#[test]
fn affinity_reduces_the_total_after_a_cost_increase_is_added() {
    cr!("601.2f", "702.41a", "118.7a");
    ruling!(
        "Thalia, Guardian of Thraben",
        "start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions"
    );
    supported("Thoughtcast");
    // Thoughtcast {4}{U} (affinity for artifacts) under Thalia with five artifacts:
    // {4}{U} + {1} - {5} = {U}. Reducing first would leave {1}{U}.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    for _ in 0..5 {
        t.battlefield(P0, "Ornithopter");
    }
    let island = t.lands(P0, "Island", 1)[0];
    let cast = t.hand(P0, "Thoughtcast");
    assert!(can_cast(&mut t, P0, cast, CastMethod::Normal));
    t.cast(P0, cast).go();
    assert!(is_tapped(&t, island));
    t.resolve();
    assert_eq!(t.hand_size(P0), 2);
    // With four artifacts the total is {1}{U}: one Island isn't enough.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    for _ in 0..4 {
        t.battlefield(P0, "Ornithopter");
    }
    t.lands(P0, "Island", 1);
    let cast = t.hand(P0, "Thoughtcast");
    assert!(!can_cast(&mut t, P0, cast, CastMethod::Normal));
    assert!(t.cast(P0, cast).try_go().is_err());
    assert!(t.in_hand(P0, "Thoughtcast"));
}

#[test]
fn increases_and_reductions_apply_to_an_offered_alternative_cost_in_that_order() {
    cr!("118.9d", "601.2f");
    ruling!(
        "Thalia, Guardian of Thraben",
        "start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions"
    );
    ruling!(
        "Fist of Suns",
        "You can pay additional costs, such as kicker, in addition to this alternative cost."
    );
    supported("Fist of Suns");
    supported("Vryn Wingmare");
    // Lightning Bolt for Fist of Suns's {W}{U}{B}{R}{G} under Thalia (+{1}) with Helm of
    // Awakening (-{1}): exactly {W}{U}{B}{R}{G}.
    let wubrg = |t: &mut TestGame| {
        for ty in [
            ManaType::W,
            ManaType::U,
            ManaType::B,
            ManaType::R,
            ManaType::G,
        ] {
            add_mana(t, P0, ty, 1);
        }
    };
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fist of Suns");
    t.battlefield(P0, "Helm of Awakening");
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    wubrg(&mut t);
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(can_cast(&mut t, P0, bolt, OFFERED));
    t.cast(P0, bolt).method(OFFERED).target(P1).go();
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve();
    assert_eq!(t.life(P1), 17);
    // Two increases and one reduction: {1}{W}{U}{B}{R}{G}. Reducing first ({W}{U}{B}{R}{G}
    // has no generic mana to reduce) would make it {2}{W}{U}{B}{R}{G}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fist of Suns");
    t.battlefield(P0, "Helm of Awakening");
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.battlefield(P1, "Vryn Wingmare");
    wubrg(&mut t);
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(!can_cast(&mut t, P0, bolt, OFFERED));
    add_mana(&mut t, P0, ManaType::C, 1);
    assert!(can_cast(&mut t, P0, bolt, OFFERED));
    t.cast(P0, bolt).method(OFFERED).target(P1).go();
    assert_eq!(pool_total(&t, P0), 0);
    // An additional cost is paid with the alternative cost (CR 118.9d): Burst Lightning
    // kicked for Fist of Suns's cost is {W}{U}{B}{R}{G} + {4}, and deals 4.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fist of Suns");
    wubrg(&mut t);
    add_mana(&mut t, P0, ManaType::C, 3);
    let burst = t.hand(P0, "Burst Lightning");
    assert!(t
        .cast(P0, burst)
        .method(OFFERED)
        .kicked(true)
        .target(P1)
        .try_go()
        .is_err());
    assert_eq!(pool_total(&t, P0), 8, "nothing paid for a reversed cast");
    add_mana(&mut t, P0, ManaType::C, 1);
    t.cast(P0, burst)
        .method(OFFERED)
        .kicked(true)
        .target(P1)
        .go();
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn trinispheres_floor_applies_to_a_spell_cast_without_paying_its_mana_cost() {
    cr!("601.2f", "118.9");
    ruling!(
        "Trinisphere",
        "Finally, apply Trinisphere's effect if the mana component of the spell's cost is less than three mana"
    );
    supported("Omniscience");
    // Grizzly Bears cast from the hand without paying its mana cost: a total of {0}, which
    // Trinisphere makes {3}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    t.battlefield(P1, "Trinisphere");
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Free));
    assert!(t.cast(P0, bears).method(CastMethod::Free).try_go().is_err());
    t.lands(P0, "Forest", 1);
    let bears = t.g.current(bears);
    assert!(can_cast(&mut t, P0, bears, CastMethod::Free));
    t.cast(P0, bears).method(CastMethod::Free).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve();
    assert_eq!(creatures_named(&t, "Grizzly Bears"), 1);
}

#[test]
fn trinispheres_floor_is_judged_on_the_total_cost_that_delve_then_pays() {
    cr!("601.2f", "702.66b");
    ruling!(
        "Treasure Cruise",
        "Delve doesn't change a spell's mana cost or mana value."
    );
    // Treasure Cruise's total cost is {7}{U}, well above three mana: Trinisphere does
    // nothing, and delve may then pay all seven generic mana.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Trinisphere");
    let cards: Vec<ObjectId> = (0..7).map(|_| t.graveyard(P0, "Grizzly Bears")).collect();
    t.lands(P0, "Island", 1);
    let cruise = t.hand(P0, "Treasure Cruise");
    assert!(can_cast(&mut t, P0, cruise, CastMethod::Normal));
    choose_objects(&mut t, P0, &cards);
    let spell = t.cast(P0, cruise).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.g.mana_value_of(spell), 8);
    t.resolve();
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn trinispheres_floor_can_be_paid_by_convoking_creatures() {
    cr!("601.2f", "702.51b");
    ruling!(
        "Gather Courage",
        "Convoke applies after the total cost is calculated."
    );
    supported("Gather Courage");
    // Gather Courage {G} costs {2}{G} under Trinisphere; three green creatures convoke
    // all of it.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Trinisphere");
    let bears: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let courage = t.hand(P0, "Gather Courage");
    assert!(can_cast(&mut t, P0, courage, CastMethod::Normal));
    choose_objects(&mut t, P0, &bears);
    t.cast(P0, courage).target(bears[0]).go();
    assert!(bears.iter().all(|b| is_tapped(&t, *b)));
    t.resolve();
    assert_eq!(t.pt(bears[0]), (4, 4));
    // Two creatures can't pay {2}{G}.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Trinisphere");
    let bears: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let courage = t.hand(P0, "Gather Courage");
    assert!(!can_cast(&mut t, P0, courage, CastMethod::Normal));
    choose_objects(&mut t, P0, &bears);
    assert!(t.cast(P0, courage).target(bears[0]).try_go().is_err());
    assert!(t.in_hand(P0, "Gather Courage"));
}

#[test]
fn emerges_reduction_is_applied_before_trinispheres_floor() {
    cr!("601.2f", "702.119a");
    ruling!(
        "Wretched Gryff",
        "its total cost is reduced by an amount of generic mana equal to the sacrificed creature’s mana value"
    );
    ruling!(
        "Trinisphere",
        "start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions. Finally, apply Trinisphere's effect"
    );
    supported("Wretched Gryff");
    // Wretched Gryff's emerge cost {5}{U} sacrificing Craw Wurm (mana value 6) is reduced to
    // {U}: one mana, so Trinisphere makes the total {2}{U}. One Island isn't enough.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Trinisphere");
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.lands(P0, "Island", 1);
    let gryff = t.hand(P0, "Wretched Gryff");
    assert!(!can_cast(&mut t, P0, gryff, EMERGE));
    choose_objects(&mut t, P0, &[wurm]);
    assert!(t.cast(P0, gryff).method(EMERGE).try_go().is_err());
    assert!(t.in_hand(P0, "Wretched Gryff"));
    assert!(t.on_battlefield(wurm));
    t.clear_answers();
    t.lands(P0, "Wastes", 2);
    let gryff = t.g.current(gryff);
    assert!(can_cast(&mut t, P0, gryff, EMERGE));
    choose_objects(&mut t, P0, &[wurm]);
    let spell = t.cast(P0, gryff).method(EMERGE).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert!(t.in_graveyard(P0, "Craw Wurm"));
    assert_eq!(t.g.mana_value_of(spell), 7);
    // Without Trinisphere, the one Island pays the reduced emerge cost.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.lands(P0, "Island", 1);
    let gryff = t.hand(P0, "Wretched Gryff");
    choose_objects(&mut t, P0, &[wurm]);
    t.cast(P0, gryff).method(EMERGE).go();
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn a_cost_increaser_sacrificed_for_mana_while_paying_doesnt_change_the_locked_in_total() {
    cr!("601.2f", "601.2g", "601.2h");
    supported("Ashnod's Altar");
    // Thalia (the caster's own) taxes Lightning Bolt to {1}{R}. The only way to pay the {1}
    // is Ashnod's Altar sacrificing Thalia; the total was locked in before, so it's still
    // {1}{R} and the Bolt is cast with one {C} left over.
    let mut t = TestGame::new(2);
    let thalia = t.battlefield(P0, "Thalia, Guardian of Thraben");
    t.battlefield(P0, "Ashnod's Altar");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    assert!(!t.on_battlefield(thalia));
    assert!(t.in_graveyard(P0, "Thalia, Guardian of Thraben"));
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(pool_count(&t, P0, ManaType::C), 1);
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn a_flashback_cost_reduction_comes_off_after_the_cost_increases() {
    cr!("601.2f", "118.7a", "118.9d", "702.34a");
    interp!("keyword-cost-reduction-on-an-alternative-cost-after-increases");
    ruling!(
        "Thalia, Guardian of Thraben",
        "start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions"
    );
    supported("Catalyst Stone");
    supported("Deep Analysis");
    // Catalyst Stone ("Flashback costs you pay cost {2} less") and Thalia on Deep
    // Analysis's flashback cost {1}{U}, Pay 3 life: Thalia's {1} is applied to the
    // flashback cost (CR 118.9d), and the reduction comes off after it: {1}{U} + {1} - {2}
    // = {U}, Pay 3 life. (Reducing the flashback cost alone first would lose the second
    // {1} and leave {1}{U}.)
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Catalyst Stone");
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Island", 1);
    let analysis = t.graveyard(P0, "Deep Analysis");
    assert!(can_cast(&mut t, P0, analysis, FLASHBACK));
    t.lands(P0, "Wastes", 1);
    t.cast(P0, analysis).method(FLASHBACK).target(P0).go();
    assert_eq!(untapped_lands(&t, P0), 1);
    assert_eq!(t.life(P0), 17);
    t.resolve();
    assert_eq!(t.hand_size(P0), 2);
    assert!(t.in_exile("Deep Analysis"));
    // Without Thalia the flashback cost is {U}, Pay 3 life.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Catalyst Stone");
    t.lands(P0, "Island", 1);
    let analysis = t.graveyard(P0, "Deep Analysis");
    assert!(can_cast(&mut t, P0, analysis, FLASHBACK));
    t.cast(P0, analysis).method(FLASHBACK).target(P0).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    // An opponent's Catalyst Stone ("Flashback costs your opponents pay cost {2} more")
    // with Helm of Awakening: Think Twice's {2}{U} + {2} - {1} = {3}{U}.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Catalyst Stone");
    t.battlefield(P0, "Helm of Awakening");
    t.lands(P0, "Island", 3);
    let twice = t.graveyard(P0, "Think Twice");
    assert!(!can_cast(&mut t, P0, twice, FLASHBACK));
    t.lands(P0, "Island", 1);
    assert!(can_cast(&mut t, P0, twice, FLASHBACK));
    t.cast(P0, twice).method(FLASHBACK).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    // Both effects on one flashback cost, each player's Catalyst Stone: {1}{U} + {2} - {2}
    // = {1}{U}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Catalyst Stone");
    t.battlefield(P1, "Catalyst Stone");
    t.lands(P0, "Island", 1);
    let analysis = t.graveyard(P0, "Deep Analysis");
    assert!(!can_cast(&mut t, P0, analysis, FLASHBACK));
    t.lands(P0, "Wastes", 1);
    assert!(can_cast(&mut t, P0, analysis, FLASHBACK));
    t.cast(P0, analysis).method(FLASHBACK).target(P0).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn a_dash_cost_reduction_comes_off_after_the_cost_increases() {
    cr!("601.2f", "118.7a", "118.9d", "702.109a");
    interp!("keyword-cost-reduction-on-an-alternative-cost-after-increases");
    ruling!(
        "Warbringer",
        "If a card’s dash cost includes one generic mana, that cost will be reduced by {1}."
    );
    supported("Warbringer");
    supported("Mardu Scout");
    // Warbringer ("Dash costs you pay cost {2} less") and Sphere of Resistance on Mardu
    // Scout's dash cost {1}{R}: {1}{R} + {1} - {2} = {R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Warbringer");
    t.battlefield(P1, "Sphere of Resistance");
    t.lands(P0, "Mountain", 1);
    let scout = t.hand(P0, "Mardu Scout");
    assert!(can_cast(&mut t, P0, scout, DASH));
    t.lands(P0, "Wastes", 1);
    t.cast(P0, scout).method(DASH).go();
    assert_eq!(untapped_lands(&t, P0), 1);
    t.resolve();
    assert!(t.on_battlefield(scout));
    assert!(t.obj_now(scout).has_keyword(KeywordKind::Haste));
    // With Lodestone Golem as well: {1}{R} + {2} - {2} = {1}{R} (reducing the dash cost
    // alone first would leave {R} + {2} = {2}{R}).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Warbringer");
    t.battlefield(P1, "Sphere of Resistance");
    t.battlefield(P1, "Lodestone Golem");
    t.lands(P0, "Mountain", 1);
    let scout = t.hand(P0, "Mardu Scout");
    assert!(!can_cast(&mut t, P0, scout, DASH));
    t.lands(P0, "Wastes", 1);
    assert!(can_cast(&mut t, P0, scout, DASH));
    t.lands(P0, "Wastes", 1);
    t.cast(P0, scout).method(DASH).go();
    assert_eq!(untapped_lands(&t, P0), 1);
    t.resolve();
    assert!(t.on_battlefield(scout));
}

#[test]
fn an_alternative_keyword_cost_reduction_never_reduces_colored_mana() {
    cr!("601.2f", "118.7a", "118.9d", "702.34a", "702.109a");
    interp!("keyword-cost-reduction-on-an-alternative-cost-after-increases");
    ruling!(
        "Warbringer",
        "Warbringer’s first ability can’t affect the colored mana requirement of a dash cost."
    );
    // Two Warbringers ({4} less) and Sphere of Resistance on Mardu Scout's dash cost:
    // {1}{R} + {1} - {4} = {R}; the excess is lost, the {R} stays.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Warbringer");
    t.battlefield(P0, "Warbringer");
    t.battlefield(P1, "Sphere of Resistance");
    let scout = t.hand(P0, "Mardu Scout");
    assert!(!can_cast(&mut t, P0, scout, DASH));
    t.lands(P0, "Wastes", 1);
    assert!(!can_cast(&mut t, P0, scout, DASH));
    t.lands(P0, "Mountain", 1);
    assert!(can_cast(&mut t, P0, scout, DASH));
    t.cast(P0, scout).method(DASH).go();
    assert_eq!(untapped_lands(&t, P0), 1);
    t.resolve();
    assert!(t.on_battlefield(scout));
    // Two Catalyst Stones and Thalia on Deep Analysis: {1}{U} + {1} - {4} = {U}, and the
    // 3 life is still paid.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Catalyst Stone");
    t.battlefield(P0, "Catalyst Stone");
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Wastes", 2);
    let analysis = t.graveyard(P0, "Deep Analysis");
    assert!(!can_cast(&mut t, P0, analysis, FLASHBACK));
    t.lands(P0, "Island", 1);
    assert!(can_cast(&mut t, P0, analysis, FLASHBACK));
    t.cast(P0, analysis).method(FLASHBACK).target(P0).go();
    assert_eq!(untapped_lands(&t, P0), 2);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn a_flashback_cost_reduction_reduces_an_additional_cost_paid_with_it() {
    cr!("118.9d", "601.2f", "702.34a", "702.27a");
    interp!("keyword-cost-reduction-on-an-alternative-cost-after-increases");
    supported("Snapcaster Mage");
    supported("Capsize");
    // Snapcaster Mage gives Capsize flashback {1}{U}{U}. Its buyback cost {3} is applied to
    // that alternative cost (CR 118.9d), so Catalyst Stone's reduction comes off it too:
    // {1}{U}{U} + {3} - {2} = {2}{U}{U}. (Reducing the flashback cost alone would leave
    // {U}{U} + {3} = {3}{U}{U}.) Flashback exiles it rather than buyback returning it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Catalyst Stone");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let capsize = t.graveyard(P0, "Capsize");
    t.lands(P0, "Island", 2);
    let mage = t.hand(P0, "Snapcaster Mage");
    t.answer_targets(P0, &[Entity::Object(capsize)]);
    t.cast(P0, mage).go();
    t.resolve_all();
    assert!(t.obj_now(capsize).has_keyword(KeywordKind::Flashback));
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 2);
    let capsize = t.g.current(capsize);
    t.cast(P0, capsize)
        .method(FLASHBACK)
        .kicked(true)
        .target(bears)
        .go();
    assert_eq!(optional_costs_asked(&t, P0), vec!["buyback".to_string()]);
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.in_exile("Capsize"));
}

#[test]
fn delve_pays_the_generic_mana_a_cost_increase_added() {
    cr!("601.2f", "702.66a", "702.66b");
    ruling!(
        "Treasure Cruise",
        "you can't exile more than seven cards from your graveyard to cast Treasure Cruise unless an effect has increased its cost"
    );
    // Under Thalia, Treasure Cruise's total cost is {8}{U}: eight cards may be exiled.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let cards: Vec<ObjectId> = (0..8).map(|_| t.graveyard(P0, "Grizzly Bears")).collect();
    t.lands(P0, "Island", 1);
    let cruise = t.hand(P0, "Treasure Cruise");
    assert!(can_cast(&mut t, P0, cruise, CastMethod::Normal));
    choose_objects(&mut t, P0, &cards);
    t.cast(P0, cruise).go();
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(untapped_lands(&t, P0), 0);
    // Without the increase, eight is more than the seven generic mana: that choice isn't
    // legal, and the default exiles only what the mana can't pay (seven).
    let mut t = TestGame::new(2);
    let cards: Vec<ObjectId> = (0..8).map(|_| t.graveyard(P0, "Grizzly Bears")).collect();
    t.lands(P0, "Island", 1);
    let cruise = t.hand(P0, "Treasure Cruise");
    choose_objects(&mut t, P0, &cards);
    t.cast(P0, cruise).go();
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn convoke_may_tap_creatures_that_came_under_your_control_this_turn() {
    cr!("702.51a", "601.2h");
    ruling!(
        "Stoke the Flames",
        "You can tap any untapped creature you control to convoke a spell, even one you haven't controlled continuously since the beginning of your most recent turn."
    );
    // Four summoning-sick Hill Giants (red) pay all of Stoke the Flames's {2}{R}{R}.
    let mut t = TestGame::new(2);
    let giants: Vec<ObjectId> = (0..4)
        .map(|_| t.battlefield_sick(P0, "Hill Giant"))
        .collect();
    let stoke = t.hand(P0, "Stoke the Flames");
    assert!(can_cast(&mut t, P0, stoke, CastMethod::Normal));
    choose_objects(&mut t, P0, &giants);
    t.cast(P0, stoke).target(P1).go();
    assert!(giants.iter().all(|g| is_tapped(&t, *g)));
    t.resolve();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn reductions_apply_to_the_total_cost_with_x_and_kicker_included() {
    cr!("601.2f", "118.8", "107.3a");
    ruling!(
        "Burst Lightning",
        "add any cost increases (such as kicker), then apply any cost reductions"
    );
    // Blaze with X = 3 under Helm of Awakening: {3}{R} - {1} = {2}{R}, and it deals 3.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Helm of Awakening");
    t.lands(P0, "Mountain", 3);
    let blaze = t.hand(P0, "Blaze");
    let spell = t.cast(P0, blaze).x(3).target(P1).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(x_of(&t, spell), Some(3));
    assert_eq!(t.g.mana_value_of(spell), 4);
    t.resolve();
    assert_eq!(t.life(P1), 17);
    // Burst Lightning kicked: {R} + {4} - {1} = {3}{R}, and it deals 4.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Helm of Awakening");
    t.lands(P0, "Mountain", 4);
    let burst = t.hand(P0, "Burst Lightning");
    t.cast(P0, burst).kicked(true).target(P1).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn a_colored_only_reduction_applies_after_an_increase_and_leaves_generic_mana_alone() {
    cr!("601.2f");
    ruling!(
        "Edgewalker",
        "You apply cost reduction effects after other cost modifiers"
    );
    supported("Edgewalker");
    supported("Orzhov Pontiff");
    // Orzhov Pontiff {1}{W}{B} under Sphere of Resistance with Edgewalker ("Cleric spells
    // you cast cost {W}{B} less ... reduces only the amount of colored mana you pay"):
    // {1}{W}{B} + {1} - {W}{B} = {2}, payable with two Wastes.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Edgewalker");
    t.battlefield(P1, "Sphere of Resistance");
    t.lands(P0, "Wastes", 1);
    let pontiff = t.hand(P0, "Orzhov Pontiff");
    assert!(!can_cast(&mut t, P0, pontiff, CastMethod::Normal));
    t.lands(P0, "Wastes", 1);
    assert!(can_cast(&mut t, P0, pontiff, CastMethod::Normal));
    t.cast(P0, pontiff).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    // Without the increase: {1}, and the reduction can't reduce it further.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Edgewalker");
    let pontiff = t.hand(P0, "Orzhov Pontiff");
    assert!(!can_cast(&mut t, P0, pontiff, CastMethod::Normal));
    t.lands(P0, "Wastes", 1);
    assert!(can_cast(&mut t, P0, pontiff, CastMethod::Normal));
}

#[test]
fn an_activated_abilitys_increase_is_applied_before_a_reduction_that_keeps_one_mana() {
    cr!("602.2b", "601.2f");
    ruling!(
        "Training Grounds",
        "Since activating Urabrask's activated ability would now cost {2}{R}, Training Grounds reduces that cost back to {R}."
    );
    supported("Training Grounds");
    supported("Suppression Field");
    supported("Prodigal Sorcerer");
    // Prodigal Sorcerer's {T} ability under Suppression Field (+{2}) with Training Grounds
    // (-{2}, but not below one mana): {2} - {2} keeping one mana = {1}. Reducing first
    // would leave nothing to reduce and the increase would then make it {2}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Training Grounds");
    t.battlefield(P1, "Suppression Field");
    let tim = t.battlefield(P0, "Prodigal Sorcerer");
    assert!(t.activate(P0, tim, 0, &[Entity::Player(P1)]).is_err());
    assert!(!is_tapped(&t, tim));
    let island = t.lands(P0, "Island", 1)[0];
    assert!(t.activate(P0, tim, 0, &[Entity::Player(P1)]).is_ok());
    assert!(is_tapped(&t, tim) && is_tapped(&t, island));
    t.resolve();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn a_reduction_reduces_the_x_part_of_an_activation_cost() {
    cr!("602.2b", "601.2f", "107.3a", "118.7a");
    ruling!(
        "Training Grounds",
        "If you control Training Grounds and you activate the ability with X equal to 5, you'll have to pay only {3}{B}{B}."
    );
    supported("Drana, Kalastria Bloodchief");
    // Drana's {X}{B}{B} with X = 5 under Training Grounds: {3}{B}{B}, five lands; Wall of
    // Stone (0/8) gets -0/-5.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Training Grounds");
    let drana = t.battlefield(P0, "Drana, Kalastria Bloodchief");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Wastes", 3);
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(5));
    assert!(t.activate(P0, drana, 0, &[Entity::Object(wall)]).is_ok());
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve();
    assert_eq!(t.pt(wall), (0, 3));
    assert_eq!(t.pt(drana), (9, 4));
    // The reduction leaves the {B}{B} alone (CR 118.7a): one Swamp and four Wastes can't
    // pay {3}{B}{B}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Training Grounds");
    let drana = t.battlefield(P0, "Drana, Kalastria Bloodchief");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 4);
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(5));
    assert!(t.activate(P0, drana, 0, &[Entity::Object(wall)]).is_err());
    assert_eq!(untapped_lands(&t, P0), 5);
    // Without Training Grounds, five lands pay only X = 3.
    let mut t = TestGame::new(2);
    let drana = t.battlefield(P0, "Drana, Kalastria Bloodchief");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Wastes", 3);
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(5));
    assert!(t.activate(P0, drana, 0, &[Entity::Object(wall)]).is_err());
    assert_eq!(untapped_lands(&t, P0), 5);
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(3));
    assert!(t.activate(P0, drana, 0, &[Entity::Object(wall)]).is_ok());
    t.resolve();
    assert_eq!(t.pt(wall), (0, 5));
}
