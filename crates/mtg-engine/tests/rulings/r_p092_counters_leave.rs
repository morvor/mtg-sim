//! Rulings batch P092 — counters and "a creature you control leaves the battlefield"
//! triggers: Costume Closet, Ninth Bridge Patrol, The Ozolith, Broodguard Elite, Vela the
//! Night-Clad, Wight of Precinct Six, Prowling Geistcatcher.

use crate::r_p017_common::{animate, modify_no_settle};
use crate::r_p120_common::{give_minus1, give_plus1, plus1};
use crate::r_s01_common::*;
use crate::r_s02_common::{create_token, destroy};
use crate::r_s05_common::move_to;
use crate::r_s06_common::give_control;
use crate::r_s29_common::put_counters;
use mtg_engine::ability::Modification;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// P0 casts a spell (giving P0 the mana for it) and everything resolves.
fn cast_resolve(t: &mut TestGame, name: &str, targets: &[Entity]) {
    give_mana_for(t, P0, name);
    let c = t.hand(P0, name);
    let mut b = t.cast(P0, c);
    for e in targets {
        b = b.target(*e);
    }
    b.go();
    t.resolve_all();
}

/// A Costume Closet with its two +1/+1 counters, entered for real.
fn closet(t: &mut TestGame) -> ObjectId {
    let c = t.enter(P0, "Costume Closet");
    t.resolve_all();
    assert_eq!(plus1(t, c), 2);
    c
}

#[test]
fn costume_closet_moves_nothing_if_it_left_or_ran_out_of_counters() {
    cr!("608.2b", "701.12b", "122.5");
    ruling!(
        "Costume Closet",
        "If Costume Closet has left the battlefield or has no +1/+1 counters on it by the time its activated ability resolves, you won't put a +1/+1 counter on the target creature."
    );
    supported("Costume Closet");
    // It left the battlefield.
    let mut t = TestGame::new(2);
    let c = closet(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, c, 0, &[obj(bears)]).unwrap();
    destroy(&mut t, c);
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 0);
    // It lost its counters.
    let mut t = TestGame::new(2);
    let c = closet(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, c, 0, &[obj(bears)]).unwrap();
    t.g.remove_counters(obj(c), "+1/+1", 2);
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 0);
    assert!(t.on_battlefield(c));
}

#[test]
fn costume_closet_keeps_its_counter_if_the_target_is_illegal() {
    cr!("608.2b", "701.12b");
    ruling!(
        "Costume Closet",
        "If the creature becomes an illegal target or can't have a +1/+1 counter put onto it for some other reason, you won't remove a +1/+1 counter from Costume Closet."
    );
    let mut t = TestGame::new(2);
    let c = closet(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, c, 0, &[obj(bears)]).unwrap();
    move_to(&mut t, bears, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(plus1(&t, c), 2);
}

#[test]
fn an_empty_costume_closet_stays_and_does_nothing() {
    cr!("602.1", "608.2b");
    ruling!(
        "Costume Closet",
        "Once Costume Closet runs out of +1/+1 counters, it remains on the battlefield. However, activating its second ability won't do anything."
    );
    let mut t = TestGame::new(2);
    let c = closet(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    for _ in 0..2 {
        t.activate(P0, c, 0, &[obj(bears)]).unwrap();
        t.resolve_all();
        let cc = t.g.current(c);
        t.g.untap(cc);
    }
    assert_eq!(plus1(&t, c), 0);
    assert_eq!(plus1(&t, bears), 2);
    assert!(t.on_battlefield(c));
    t.activate(P0, c, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 2);
    assert!(t.on_battlefield(c));
}

#[test]
fn costume_closets_counters_matter_only_if_it_becomes_a_creature() {
    cr!("122.1a", "613.4c");
    ruling!(
        "Costume Closet",
        "The +1/+1 counters on Costume Closet won't affect it unless it somehow becomes a creature."
    );
    let mut t = TestGame::new(2);
    let c = closet(&mut t);
    assert!(!t.obj_now(c).is_creature());
    animate(&mut t, c, 1);
    assert_eq!(t.pt(c), (3, 3));
}

#[test]
fn ninth_bridge_patrol_dealt_lethal_damage_with_another_creature_isnt_saved() {
    cr!("704.3", "603.2");
    ruling!(
        "Ninth Bridge Patrol",
        "If Ninth Bridge Patrol is dealt lethal damage at the same time as another creature you control, it won't receive a counter from its ability in time to save it."
    );
    supported("Ninth Bridge Patrol");
    supported("Pyroclasm");
    let mut t = TestGame::new(2);
    let patrol = t.battlefield(P0, "Ninth Bridge Patrol");
    t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, "Pyroclasm", &[]);
    assert!(!t.on_battlefield(patrol));
    assert!(t.in_graveyard(P0, "Ninth Bridge Patrol"));
}

#[test]
fn ninth_bridge_patrol_counts_creatures_leaving_to_any_zone_but_not_phasing_or_type_loss() {
    cr!("603.6c", "603.10a", "702.26d", "702.26b");
    ruling!(
        "Ninth Bridge Patrol",
        "Ninth Bridge Patrol's ability doesn't care where the creature went or whether it's a creature in its new zone. It may have died, been exiled, returned to your hand, and so on. It won't trigger if an object remains on the battlefield but ceases to be a creature. It won't trigger if a creature phases out."
    );
    let mut t = TestGame::new(2);
    let patrol = t.battlefield(P0, "Ninth Bridge Patrol");
    for (i, to) in [Zone::Graveyard(P0), Zone::Exile, Zone::Hand(P0), Zone::Library(P0)]
        .into_iter()
        .enumerate()
    {
        let bears = t.battlefield(P0, "Grizzly Bears");
        move_to(&mut t, bears, to);
        t.resolve_all();
        assert_eq!(plus1(&t, patrol), i as u32 + 1, "{to:?}");
    }
    // An animated land stops being a creature: no trigger.
    let land = t.battlefield(P0, "Forest");
    animate(&mut t, land, 2);
    modify_no_settle(
        &mut t,
        land,
        vec![Modification::RemoveTypes(vec![CardType::Creature])],
    );
    t.settle();
    t.resolve_all();
    assert!(!t.obj_now(land).is_creature());
    assert_eq!(plus1(&t, patrol), 4);
    // A creature phases out: no trigger.
    let bears = t.battlefield(P0, "Grizzly Bears");
    mtg_engine::keyword_impls::phase_out(&mut t.g, vec![bears]);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(plus1(&t, patrol), 4);
}

/// An Ozolith with two +1/+1 counters and the beginning-of-combat trigger on the stack
/// targeting `target`.
fn ozolith_trigger(t: &mut TestGame, target: ObjectId) -> ObjectId {
    let oz = t.battlefield(P0, "The Ozolith");
    give_plus1(t, oz, 2);
    t.answer_targets(P0, &[obj(target)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    oz
}

#[test]
fn the_ozolith_leaving_before_its_combat_trigger_resolves_moves_nothing() {
    cr!("608.2b", "122.5");
    ruling!(
        "The Ozolith",
        "If The Ozolith leaves the battlefield after the last ability triggers but before it resolves, you can't move any counters from it onto the target creature."
    );
    supported("The Ozolith");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let oz = ozolith_trigger(&mut t, bears);
    destroy(&mut t, oz);
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 0);
}

#[test]
fn the_ozolith_keeps_its_counters_if_its_target_is_illegal() {
    cr!("608.2b");
    ruling!(
        "The Ozolith",
        "If the target creature is an illegal target by the time The Ozolith's last ability tries to resolve, the ability won't resolve. You won't remove any counters from The Ozolith."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let oz = ozolith_trigger(&mut t, bears);
    move_to(&mut t, bears, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(plus1(&t, oz), 2);
}

#[test]
fn the_ozolith_copies_counters_and_leave_triggers_still_see_them() {
    cr!("603.10a", "122.5");
    ruling!(
        "The Ozolith",
        "Rather, you put the same number of each kind of counter the creature had onto The Ozoloith."
    );
    ruling!(
        "Broodguard Elite",
        "For example, if you control The Ozolith (a card from the Ikoria: Lair of Behemoths set) when Broodguard Elite leaves the battlefield, you’ll put the appropriate number of each kind of counter onto both The Ozolith and the target creature."
    );
    supported("Broodguard Elite");
    let mut t = TestGame::new(2);
    let oz = t.battlefield(P0, "The Ozolith");
    let elite = t.battlefield(P0, "Broodguard Elite");
    give_plus1(&mut t, elite, 3);
    put_counters(&mut t, elite, "oil", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    destroy(&mut t, elite);
    t.resolve_all();
    assert_eq!(plus1(&t, oz), 3);
    assert_eq!(t.counters(oz, "oil"), 1);
    assert_eq!(plus1(&t, bears), 3);
    assert_eq!(t.counters(bears, "oil"), 1);
}

#[test]
fn two_ozoliths_each_get_the_counters_and_leave_triggers_use_the_old_count() {
    cr!("603.10a", "122.5");
    ruling!(
        "The Ozolith",
        "Notably, if you somehow control a second The Ozolith, each one will receive the same number and kinds of counters that were on the creature that left the battlefield."
    );
    supported("Mirror Gallery");
    // Mirror Gallery turns off the legend rule, so P0 keeps both Ozoliths.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirror Gallery");
    let a = t.battlefield(P0, "The Ozolith");
    let b = t.battlefield(P0, "The Ozolith");
    t.settle();
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
    let elite = t.battlefield(P0, "Broodguard Elite");
    give_plus1(&mut t, elite, 2);
    put_counters(&mut t, elite, "oil", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    destroy(&mut t, elite);
    t.resolve_all();
    for oz in [a, b] {
        assert_eq!(plus1(&t, oz), 2);
        assert_eq!(t.counters(oz, "oil"), 1);
    }
    // Broodguard Elite's own trigger still sees the counters it had.
    assert_eq!(plus1(&t, bears), 2);
    assert_eq!(t.counters(bears, "oil"), 1);
}

#[test]
fn broodguard_elite_puts_all_kinds_of_its_counters() {
    cr!("603.10a", "122.1");
    ruling!(
        "Broodguard Elite",
        "Broodguard Elite’s second ability doesn’t cause you to move counters from Broodguard Elite onto the target creature. Rather, you put the same number of each kind of counter Broodguard Elite had when it left the battlefield onto the target creature."
    );
    ruling!(
        "Broodguard Elite",
        "Broodguard Elite’s second ability puts all counters that were on Broodguard Elite onto the target creature, not just its +1/+1 counters."
    );
    let mut t = TestGame::new(2);
    let elite = t.battlefield(P0, "Broodguard Elite");
    give_plus1(&mut t, elite, 2);
    put_counters(&mut t, elite, "flying", 1);
    put_counters(&mut t, elite, "oil", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    // It's bounced (it leaves without dying).
    move_to(&mut t, elite, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 2);
    assert_eq!(t.counters(bears, "flying"), 1);
    assert_eq!(t.counters(bears, "oil"), 2);
}

#[test]
fn broodguard_elite_killed_by_minus_counters_passes_both_kinds() {
    cr!("704.3", "704.5f", "704.5q", "603.10a");
    ruling!(
        "Broodguard Elite",
        "If enough -1/-1 counters are put on Broodguard Elite at the same time to make its toughness 0 or less, its second ability will see all of the +1/+1 counters it had when it died as well as the -1/-1 counters it had"
    );
    ruling!(
        "Broodguard Elite",
        "If Broodguard Elite has -1/-1 counters on it when it leaves the battlefield, that ability will include those as well. This may result in the recipient of the counters dying."
    );
    // A 3/3 recipient: it gets two +1/+1 and three -1/-1 counters, ending with one -1/-1.
    let mut t = TestGame::new(2);
    let elite = t.battlefield(P0, "Broodguard Elite");
    give_plus1(&mut t, elite, 2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    give_minus1(&mut t, elite, 3);
    assert!(!t.on_battlefield(elite));
    t.resolve_all();
    assert_eq!(t.pt(giant), (2, 2));
    assert_eq!(t.counters(giant, "-1/-1"), 1);
    // A 1/1 recipient dies.
    let mut t = TestGame::new(2);
    let elite = t.battlefield(P0, "Broodguard Elite");
    give_plus1(&mut t, elite, 2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.answer_targets(P0, &[obj(elves)]);
    give_minus1(&mut t, elite, 3);
    t.resolve_all();
    assert!(!t.on_battlefield(elves));
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
}

#[test]
fn vela_triggers_for_each_creature_leaving_with_it() {
    cr!("603.6c", "603.10a");
    ruling!(
        "Vela the Night-Clad",
        "If Vela leaves the battlefield at the same time as other creatures you control, its ability will trigger for each of those creatures."
    );
    supported("Vela the Night-Clad");
    supported("Wrath of God");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vela the Night-Clad");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, "Wrath of God", &[]);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn wight_of_precinct_six_dies_with_an_opposing_creature() {
    cr!("704.3", "704.5g");
    ruling!(
        "Wight of Precinct Six",
        "If Wight of Precinct Six is dealt lethal damage at the same time as a creature an opponent controls, they’re destroyed at the same time. It won’t get an additional +1/+1 from its ability in time to save it."
    );
    supported("Wight of Precinct Six");
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Hill Giant");
    let wight = t.battlefield(P0, "Wight of Precinct Six");
    assert_eq!(t.pt(wight), (2, 2));
    t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, "Pyroclasm", &[]);
    assert!(!t.on_battlefield(wight));
    assert!(t.in_graveyard(P0, "Wight of Precinct Six"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn wight_of_precinct_six_is_1_1_off_the_battlefield() {
    cr!("611.3a", "113.6");
    ruling!(
        "Wight of Precinct Six",
        "Wight of Precinct Six’s ability applies only while it’s on the battlefield. In all other zones, it’s a 1/1 creature."
    );
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Hill Giant");
    t.graveyard(P1, "Grizzly Bears");
    let in_hand = t.hand(P0, "Wight of Precinct Six");
    let in_gy = t.graveyard(P0, "Wight of Precinct Six");
    let on_bf = t.battlefield(P0, "Wight of Precinct Six");
    t.g.recompute();
    assert_eq!(t.pt(in_hand), (1, 1));
    assert_eq!(t.pt(in_gy), (1, 1));
    assert_eq!(t.pt(on_bf), (3, 3));
}

#[test]
fn prowling_geistcatcher_counts_a_token_you_control_but_dont_own() {
    cr!("701.21a", "111.2");
    ruling!(
        "Prowling Geistcatcher",
        "If you sacrifice a creature token you control but don't own, you will still put a +1/+1 counter on Prowling Geistcatcher."
    );
    supported("Prowling Geistcatcher");
    let mut t = TestGame::new(2);
    let gc = t.battlefield(P0, "Prowling Geistcatcher");
    let token = create_token(&mut t, P1, "Goblin");
    give_control(&mut t, token, P0);
    let token = t.g.current(token);
    t.g.sacrifice(token, P0);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(plus1(&t, gc), 1);
}

#[test]
fn prowling_geistcatcher_exiles_and_returns_a_card_you_dont_own() {
    cr!("701.21a", "607.2a", "610.3");
    ruling!(
        "Prowling Geistcatcher",
        "If you sacrifice a nontoken creature you control but don't own, Prowling Geistcatcher will still cause you to exile that card from its owner's graveyard. The card will be returned to the battlefield under your control with the rest of the exiled cards when Prowling Geistcatcher's last ability resolves."
    );
    let mut t = TestGame::new(2);
    let gc = t.battlefield(P0, "Prowling Geistcatcher");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Hill Giant");
    give_control(&mut t, bears, P0);
    for c in [bears, mine] {
        let c = t.g.current(c);
        t.g.sacrifice(c, P0);
        t.g.flush_events();
        t.resolve_all();
    }
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(mine), Zone::Exile);
    destroy(&mut t, gc);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P0);
    assert_eq!(t.obj_now(bears).owner, P1);
    assert!(t.on_battlefield(mine));
}
