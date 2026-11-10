//! CR 601.2g / 605 audit: mana abilities during casting, with real cards. Only mana
//! abilities may be activated while a spell's costs are paid (CR 605.3a): an ability with
//! a target isn't one (CR 605.1a), one that is activated only as an instant can't be
//! activated then (CR 602.5e), one that costs mana or counters can; mana already in the
//! pool pays first and survives the reversal of an illegal cast (CR 733.1); restricted
//! mana pays any part of the total cost it's allowed to pay (CR 106.6); and split second
//! stops casting but not mana abilities (CR 702.61a–b).

use crate::r601_audit_common::*;
use mtg_engine::mana::ManaType;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn an_ability_with_a_target_isnt_a_mana_ability_and_cant_pay_for_a_spell() {
    cr!("605.1a", "605.3a", "601.2g");
    ruling!(
        "Deathrite Shaman",
        "Because the first ability requires a target, it is not a mana ability. It uses the stack and can be responded to."
    );
    supported("Deathrite Shaman");
    let mut t = TestGame::new(2);
    let shaman = t.battlefield(P0, "Deathrite Shaman");
    let forest = t.graveyard(P1, "Forest");
    let bears = t.hand(P0, "Grizzly Bears");
    // No mana is available for the Bears: the Shaman's mana comes through the stack.
    assert_eq!(t.g.max_mana_available(P0), 0);
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
    assert!(t.cast(P0, bears).try_go().is_err());
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(!is_tapped(&t, shaman));
    // Activated with priority, it uses the stack.
    assert!(t.activate(P0, shaman, 0, &[Entity::Object(forest)]).is_ok());
    assert_eq!(t.stack_len(), 1);
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve();
    assert_eq!(pool_total(&t, P0), 1);
    assert!(t.in_exile("Forest"));
}

#[test]
fn an_ability_activated_only_as_an_instant_cant_pay_for_a_spell_being_cast() {
    cr!("602.5e", "605.3a", "601.2g");
    ruling!(
        "Lion's Eye Diamond",
        "it can only be activated at times when you can cast an instant"
    );
    supported("Lion's Eye Diamond");
    let mut t = TestGame::new(2);
    let led = t.battlefield(P0, "Lion's Eye Diamond");
    let bears = t.hand(P0, "Grizzly Bears");
    assert_eq!(t.g.max_mana_available(P0), 0);
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
    assert!(t.cast(P0, bears).try_go().is_err());
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.on_battlefield(led));
}

#[test]
fn a_mana_ability_that_costs_mana_is_activated_while_paying_with_mana_made_meanwhile() {
    cr!("605.3a", "601.2g", "601.2h");
    supported("Mystic Gate");
    supported("Boomerang");
    // Boomerang {U}{U} with a Plains and Mystic Gate ({W/U}, {T}: Add {U}{U}): the Plains'
    // {W} pays the Gate's cost during the payment.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let plains = t.lands(P0, "Plains", 1)[0];
    let gate = t.battlefield(P0, "Mystic Gate");
    let boomerang = t.hand(P0, "Boomerang");
    assert!(can_cast(&mut t, P0, boomerang, CastMethod::Normal));
    t.cast(P0, boomerang).target(bears).go();
    assert!(is_tapped(&t, plains) && is_tapped(&t, gate));
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve();
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn mana_in_the_pool_before_casting_pays_first_and_survives_a_reversed_cast() {
    cr!("733.1", "601.2g", "601.2h");
    interp!("reversal-undoes-mana-abilities");
    interp!("automatic-mana-payment");
    let mut t = TestGame::new(2);
    let swamps = t.lands(P0, "Swamp", 2);
    // {B} in the pool from a Swamp tapped with priority; the other Swamp is untapped.
    tap_for_mana(&mut t, P0, swamps[0]);
    assert_eq!(pool_count(&t, P0, ManaType::B), 1);
    // Altar's Reap with no creature to sacrifice: illegal, reversed. The mana ability
    // activated before the attempt wasn't part of it.
    let reap = t.hand(P0, "Altar's Reap");
    assert!(t.cast(P0, reap).try_go().is_err());
    assert!(t.in_hand(P0, "Altar's Reap"));
    assert_eq!(pool_count(&t, P0, ManaType::B), 1);
    assert!(is_tapped(&t, swamps[0]) && !is_tapped(&t, swamps[1]));
    // With a creature: the pool's {B} pays first, the other Swamp pays the {1}.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let reap = t.g.current(reap);
    choose_objects(&mut t, P0, &[bears]);
    t.cast(P0, reap).go();
    assert_eq!(pool_total(&t, P0), 0);
    assert!(is_tapped(&t, swamps[1]));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn a_mana_ability_paid_with_a_counter_is_activated_during_payment_once_each_turn() {
    cr!("605.3a", "601.2g", "602.5b");
    supported("Wall of Roots");
    // Wall of Roots: "Put a -0/-1 counter on this creature: Add {G}. Activate only once
    // each turn." It pays for Giant Growth with no lands; a second Giant Growth can't be
    // paid for this turn.
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Roots");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let growth = t.hand(P0, "Giant Growth");
    assert!(can_cast(&mut t, P0, growth, CastMethod::Normal));
    t.cast(P0, growth).target(bears).go();
    assert_eq!(t.counters(wall, "-0/-1"), 1);
    assert_eq!(t.pt(wall), (0, 4));
    t.resolve();
    assert_eq!(t.pt(bears), (5, 5));
    let growth = t.hand(P0, "Giant Growth");
    assert!(!can_cast(&mut t, P0, growth, CastMethod::Normal));
    assert!(t.cast(P0, growth).target(bears).try_go().is_err());
    assert_eq!(t.counters(wall, "-0/-1"), 1);
}

#[test]
fn restricted_mana_pays_any_part_of_a_creature_spells_total_cost_but_nothing_else() {
    cr!("601.2h", "106.6", "118.8");
    ruling!(
        "Ancient Ziggurat",
        "This includes additional costs (such as kicker)"
    );
    supported("Ancient Ziggurat");
    supported("Goblin Bushwhacker");
    // Two Ziggurats pay Goblin Bushwhacker's {R} and its kicker {R}.
    let mut t = TestGame::new(2);
    let zigs = t.lands(P0, "Ancient Ziggurat", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let whacker = t.hand(P0, "Goblin Bushwhacker");
    t.cast(P0, whacker).kicked(true).go();
    assert!(zigs.iter().all(|z| is_tapped(&t, *z)));
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 2), "kicked: creatures get +1/+0");
    // A Ziggurat can't pay for Lightning Bolt. (The list of castable spells doesn't look
    // at mana restrictions, `castability-is-optimistic`: the attempt is what fails.)
    let mut t = TestGame::new(2);
    let zig = t.lands(P0, "Ancient Ziggurat", 1)[0];
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).target(P1).try_go().is_err());
    assert!(t.in_hand(P0, "Lightning Bolt"));
    assert!(!is_tapped(&t, zig));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn split_second_stops_casting_but_not_mana_abilities() {
    cr!("702.61a", "702.61b", "601.3", "605.3a");
    ruling!(
        "Krosan Grip",
        "Players still get priority while a card with split second is on the stack; their options are just limited to mana abilities and certain special actions."
    );
    supported("Krosan Grip");
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P1, "Sol Ring");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.lands(P1, "Mountain", 1);
    t.lands(P0, "Forest", 3);
    let grip = t.hand(P0, "Krosan Grip");
    t.cast(P0, grip).target(ring).go();
    // P1 can't cast Lightning Bolt in response, but may tap the Elves for mana.
    let bolt = t.hand(P1, "Lightning Bolt");
    assert!(!can_cast(&mut t, P1, bolt, CastMethod::Normal));
    assert!(t.cast(P1, bolt).target(P0).try_go().is_err());
    assert!(t.activate(P1, elves, 0, &[]).is_ok());
    assert_eq!(pool_count(&t, P1, ManaType::G), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Sol Ring"));
    // Once it has resolved, the Bolt may be cast.
    assert!(can_cast(&mut t, P1, bolt, CastMethod::Normal));
}
