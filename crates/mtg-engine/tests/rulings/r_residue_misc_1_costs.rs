//! Item `rulings-residue-misc-1` (pass C) — costs and divided damage: Glamdring's and
//! Doran's reductions of generic mana only (CR 601.2f), The Pride of Hull Clade and
//! Nemesis of Mortals (the total cost is locked in before mana abilities, CR 601.2a,
//! 601.2f–g), Lethal Sting's and Desperate Plea's additional costs (CR 601.2b, 601.2c,
//! 601.2h), Fatal Grudge (copied costs, CR 707.10; free casting still pays additional
//! costs, CR 118.9), Meteor Shower and Polukranos (dividing damage, CR 601.2d, 608.2b),
//! Eureka (put onto the battlefield, not cast; X is 0, CR 107.3f), Moment of Truth, and
//! Rampant Frogantua (players who have lost, CR 104.3).

use crate::r_p103_common::{obj, pl};
use crate::r_residue_misc_1_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const P4: PlayerId = PlayerId(4);
const P5: PlayerId = PlayerId(5);

// ---------------------------------------------------------------------------------------
// Cost reductions
// ---------------------------------------------------------------------------------------

const GLAMDRING: &str = "Glamdring, Foe-hammer // Gleam of Death";

/// P0's creature `name` equipped with Glamdring.
fn glamdring_on(t: &mut TestGame, name: &str) -> ObjectId {
    supported(GLAMDRING);
    let c = t.battlefield(P0, name);
    attach_new(t, P0, GLAMDRING, c);
    c
}

#[test]
fn glamdring_reduces_only_generic_mana() {
    cr!("601.2f", "118.7d");
    ruling!(
        "Glamdring, Foe-hammer // Gleam of Death",
        "Glamdring, Foe-hammer's cost reduction ability reduces only the generic mana in the spell's cost. The colored mana must still be paid."
    );
    let mut t = TestGame::new(2);
    glamdring_on(&mut t, "Craw Wurm");
    t.advance_to(P0, Step::PrecombatMain);
    let bolt = t.hand(P0, "Lightning Bolt");
    assert_eq!(cost_to_cast(&mut t, P0, bolt, &[pl(P1)]), Some(1));
    t.resolve_all();
    let div = t.hand(P0, "Divination");
    assert_eq!(cost_to_cast(&mut t, P0, div, &[]), Some(1));
}

#[test]
fn glamdring_applies_after_cost_increases_and_leaves_the_mana_value() {
    cr!("601.2f", "202.3");
    ruling!(
        "Glamdring, Foe-hammer // Gleam of Death",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Glamdring, Foe-hammer). The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    // Thalia's {1} more, then Glamdring's {1} less (Llanowar Elves, power 1): Shock costs
    // {R}.
    let mut t = TestGame::new(2);
    glamdring_on(&mut t, "Llanowar Elves");
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.advance_to(P0, Step::PrecombatMain);
    let shock = t.hand(P0, "Shock");
    assert_eq!(cost_to_cast(&mut t, P0, shock, &[pl(P1)]), Some(1));
    assert_eq!(crate::r_p001_common::mv(&t, shock), 1);
}

#[test]
fn doran_reduces_only_generic_mana_of_creatures_with_more_toughness() {
    cr!("601.2f", "118.7d");
    ruling!(
        "Doran, Besieged by Time",
        "Doran's first ability applies only to generic mana in the total cost of creature spells you cast with toughness greater than their power."
    );
    supported("Doran, Besieged by Time");
    for (name, cost) in [
        ("Wall of Roots", 1),
        ("Wall of Stone", 2),
        ("Birds of Paradise", 1),
        ("Grizzly Bears", 2),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Doran, Besieged by Time");
        t.advance_to(P0, Step::PrecombatMain);
        let c = t.hand(P0, name);
        assert_eq!(cost_to_cast(&mut t, P0, c, &[]), Some(cost), "{name}");
    }
}

#[test]
fn nemesis_of_mortals_cost_is_locked_before_mana_abilities() {
    cr!("601.2f", "601.2g");
    ruling!(
        "Nemesis of Mortals",
        "Once you declare you're casting Nemesis of Mortals or activating its ability, it's too late for players to try to change the number of creature cards in your graveyard to affect the cost of doing so."
    );
    supported("Nemesis of Mortals");
    // No creature cards in the graveyard: {4}{G}{G}. Two Forests, two Wastes and Ashnod's
    // Altar sacrificing the Bears pay it; the Bears in the graveyard don't make it cheaper.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Ashnod's Altar");
    let lands = [t.lands(P0, "Forest", 2), t.lands(P0, "Wastes", 2)].concat();
    t.advance_to(P0, Step::PrecombatMain);
    let n = t.hand(P0, "Nemesis of Mortals");
    let from = t.asked().len();
    t.cast(P0, n).go();
    assert_eq!(t.zone(n), Zone::Stack);
    assert!(!t.on_battlefield(bears));
    assert!(lands.iter().all(|l| t.obj_now(*l).tapped));
    assert!(others_asked_since(&t, from, P0).is_empty());
}

const PRIDE: &str = "The Pride of Hull Clade";

#[test]
fn pride_of_hull_clade_cost_is_locked_and_nobody_acts_meanwhile() {
    cr!("601.2", "601.2f", "601.2g");
    ruling!(
        "The Pride of Hull Clade",
        "Once you announce you're casting a spell, no player may take actions until the spell has been paid for. Notably, opponents can't try to reduce the total toughness of creatures you control."
    );
    ruling!(
        "The Pride of Hull Clade",
        "Once you determine the cost to cast The Pride of Hull Clade, you may activate mana abilities to pay that cost. If the total toughness of creatures you control changes while activating mana abilities, the cost to cast The Pride of Hull Clade remains what you previously determined."
    );
    supported(PRIDE);
    // Wall of Stone (0/8): {2}{G}, paid by a Forest and the Wall sacrificed to Ashnod's
    // Altar ({C}{C}); afterward the total toughness is 0.
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Stone");
    t.battlefield(P0, "Ashnod's Altar");
    t.lands(P0, "Forest", 1);
    t.advance_to(P0, Step::PrecombatMain);
    let p = t.hand(P0, PRIDE);
    let from = t.asked().len();
    t.cast(P0, p).go();
    assert_eq!(t.zone(p), Zone::Stack);
    assert!(!t.on_battlefield(wall));
    assert!(others_asked_since(&t, from, P0).is_empty());
}

#[test]
fn pride_of_hull_clade_never_below_g_and_mana_value_unchanged() {
    cr!("601.2f", "202.3");
    ruling!(
        "The Pride of Hull Clade",
        "The Pride of Hull Clade's cost reduction ability can't reduce the total cost to cast the spell below {G}."
    );
    ruling!(
        "The Pride of Hull Clade",
        "The Pride of Hull Clade's mana value doesn't change no matter what the total toughness of creatures you control is."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wall of Stone");
    t.battlefield(P0, "Wall of Stone");
    t.advance_to(P0, Step::PrecombatMain);
    let p = t.hand(P0, PRIDE);
    assert_eq!(cost_to_cast(&mut t, P0, p, &[]), Some(1));
    assert_eq!(crate::r_p001_common::mv(&t, p), 11);
}

#[test]
fn pride_of_hull_clade_counts_toughness_after_moving_to_the_stack() {
    cr!("601.2a", "601.2f");
    ruling!(
        "The Pride of Hull Clade",
        "The first step of casting a spell is to move it to the stack. If this causes the total toughness of creatures you control to change (perhaps because you control a creature whose toughness is determined by the number of cards in your hand), that new toughness will be used to determine the cost reduction."
    );
    supported("Maro");
    // Maro with three cards in hand (the Pride and two Islands): 3/3, then 2/2 once the
    // Pride is on the stack: {8}{G}.
    let mut t = TestGame::new(2);
    let maro = t.battlefield(P0, "Maro");
    t.hand(P0, "Island");
    t.hand(P0, "Island");
    t.advance_to(P0, Step::PrecombatMain);
    let p = t.hand(P0, PRIDE);
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(pt_now(&mut t, maro), (3, 3));
    assert_eq!(cost_to_cast(&mut t, P0, p, &[]), Some(9));
}

// ---------------------------------------------------------------------------------------
// Additional costs
// ---------------------------------------------------------------------------------------

#[test]
fn lethal_sting_needs_a_creature_that_can_take_the_counter() {
    cr!("601.2b", "601.2h", "122.1");
    ruling!(
        "Lethal Sting",
        "If you don’t control any creatures or can’t put any -1/-1 counters on any creature you control, you can’t cast Lethal Sting."
    );
    supported("Lethal Sting");
    supported("Melira, Sylvok Outcast");
    for (mine, ok) in [
        (&[][..], false),
        (&["Melira, Sylvok Outcast"][..], false),
        (&["Grizzly Bears"][..], true),
    ] {
        let mut t = TestGame::new(2);
        for n in mine {
            t.battlefield(P0, n);
        }
        t.battlefield(P1, "Hill Giant");
        t.advance_to(P0, Step::PrecombatMain);
        lands_for_cost(&mut t, P0, "Lethal Sting");
        let s = t.hand(P0, "Lethal Sting");
        assert_eq!(
            can_cast(&mut t, P0, s, mtg_engine::object::CastMethod::Normal),
            ok,
            "{mine:?}"
        );
    }
}

#[test]
fn desperate_plea_cant_return_the_creature_it_sacrificed() {
    cr!("601.2c", "601.2h");
    ruling!(
        "Desperate Plea",
        "Because you select targets for Desperate Plea before paying its additional cost, the first mode cannot target and return the creature you sacrificed."
    );
    supported("Desperate Plea");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.graveyard(P0, "Llanowar Elves");
    t.advance_to(P0, Step::PrecombatMain);
    lands_for_cost(&mut t, P0, "Desperate Plea");
    let s = t.hand(P0, "Desperate Plea");
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer_targets(P0, &[obj(elves)]);
    t.answer_choose(P0, &[obj(bears)]);
    t.cast(P0, s).go();
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert_eq!(
        cands,
        vec![vec![obj(elves)]],
        "the Bears weren't in the graveyard yet"
    );
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn fatal_grudge_copy_uses_the_original_sacrificed_permanent() {
    cr!("707.10", "601.2h");
    ruling!(
        "Fatal Grudge",
        "If a player copies Fatal Grudge, the characteristics of the permanent that was sacrificed to pay for the original spell are used to determine which permanents each opponent may choose."
    );
    supported("Fatal Grudge");
    // P0 sacrifices Sol Ring (an artifact) and copies the spell: P1 sacrifices an artifact
    // for each, never the Bears.
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Sol Ring");
    let a = t.battlefield(P1, "Mind Stone");
    let b = t.battlefield(P1, "Mind Stone");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[obj(ring)]);
    let g = cast_new(&mut t, P0, "Fatal Grudge", &[]);
    assert!(!t.on_battlefield(ring));
    t.answer_choose(P1, &[obj(a)]);
    t.answer_choose(P1, &[obj(b)]);
    cast_new(&mut t, P0, "Reverberate", &[obj(g)]);
    t.resolve();
    t.resolve();
    assert!(
        !t.on_battlefield(a),
        "the copy made P1 sacrifice an artifact"
    );
    t.resolve_all();
    assert!(!t.on_battlefield(b));
    assert!(t.on_battlefield(bears));
}

#[test]
fn fatal_grudge_cast_free_still_sacrifices() {
    cr!("118.9", "118.9d", "601.2h");
    ruling!(
        "Fatal Grudge",
        "If an effect allows a player to cast Fatal Grudge without paying its mana cost, that player must still pay the additional cost of sacrificing a nonland permanent."
    );
    // Master of Predicaments: P1 guesses "greater than 4" (wrong), P0 casts it free.
    let mut t = TestGame::new(2);
    let master = t.battlefield(P0, "Master of Predicaments");
    let ring = t.battlefield(P0, "Sol Ring");
    let stone = t.battlefield(P1, "Mind Stone");
    let fg = t.hand(P0, "Fatal Grudge");
    t.answer_choose(P0, &[obj(fg)]);
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(ring)]);
    t.answer_choose(P1, &[obj(stone)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(master, pl(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.zone(fg), Zone::Graveyard(P0));
    assert!(!t.on_battlefield(ring), "the Sol Ring was sacrificed");
    assert!(!t.on_battlefield(stone));
}

// ---------------------------------------------------------------------------------------
// Dividing damage
// ---------------------------------------------------------------------------------------

/// Casts Meteor Shower with `x` among `targets` dividing `split`; returns the mana spent.
fn meteor_shower(t: &mut TestGame, x: i64, targets: &[Entity], split: &[i64]) -> usize {
    supported("Meteor Shower");
    let s = t.hand(P0, "Meteor Shower");
    tap_lands(t, P0);
    fill_pool(t, P0, 15);
    let before = pool(t, P0);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(split.to_vec()));
    t.cast(P0, s).x(x).targets(targets).go();
    before - pool(t, P0)
}

#[test]
fn meteor_shower_costs_twice_x_and_deals_x_plus_1() {
    cr!("107.3", "601.2f");
    ruling!(
        "Meteor Shower",
        "A tip on how this math works: To deal a total of 1 damage, X is 0. The cost is {0} + {0} + {R} = {R}. To deal a total of 2 damage, X is 1. The cost is {1} + {1} + {R} = {2}{R}. To deal a total of 3 damage, X is 2. The cost is {2} + {2} + {R} = {4}{R}. And so on."
    );
    for (x, cost, dealt) in [(0, 1, 1), (1, 3, 2), (2, 5, 3)] {
        let mut t = TestGame::new(2);
        t.advance_to(P0, Step::PrecombatMain);
        assert_eq!(meteor_shower(&mut t, x, &[pl(P1)], &[dealt]), cost, "X {x}");
        t.resolve_all();
        assert_eq!(t.life(P1), 20 - dealt as i32, "X {x}");
    }
}

#[test]
fn meteor_shower_divides_whole_damage_as_cast_at_least_1_each() {
    cr!("601.2d", "608.2b");
    ruling!(
        "Meteor Shower",
        "You can’t choose to do fractional damage to a target."
    );
    ruling!(
        "Meteor Shower",
        "You divide the damage as you cast Meteor Shower, not as it resolves. Each target must be assigned at least 1 damage."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.advance_to(P0, Step::PrecombatMain);
    let from = t.asked().len();
    meteor_shower(&mut t, 2, &[obj(giant), pl(P1)], &[2, 1]);
    // The division was asked during casting: whole numbers, at least 1 each.
    let divs: Vec<(u32, u32)> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::Divide {
                total, min_each, ..
            } => Some((total, min_each)),
            _ => None,
        })
        .collect();
    assert_eq!(divs, vec![(3, 1)]);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(damage_on(&t, giant), 2);
    assert_eq!(t.life(P1), 19);
}

const POLUKRANOS: &str = "Polukranos, World Eater";

/// P0's Polukranos becomes monstrous with X = `x`, targeting `targets` with the division
/// `split` (the trigger then waits on the stack).
fn polukranos(t: &mut TestGame, x: i64, targets: &[Entity], split: &[i64]) -> ObjectId {
    supported(POLUKRANOS);
    let p = t.battlefield(P0, POLUKRANOS);
    t.advance_to(P0, Step::PrecombatMain);
    tap_lands(t, P0);
    fill_pool(t, P0, 15);
    t.answer(P0, DecisionKind::X, Answer::Number(x));
    t.answer_targets(P0, targets);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(split.to_vec()));
    t.activate(P0, p, 0, &[]).unwrap();
    t.resolve();
    p
}

#[test]
fn polukranos_deals_its_damage_first_infect_shrinks_the_return_damage() {
    cr!("608.2c", "702.90c", "120.3d");
    ruling!(
        "Polukranos, World Eater",
        "As Polukranos's triggered ability resolves, Polukranos deals damage first, then the target creatures do. Although no creature will die until after the ability finishes resolving, the order could matter if Polukranos has wither or infect."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let p = polukranos(&mut t, 2, &[obj(giant)], &[2]);
    grant_kw(&mut t, p, mtg_engine::keywords::KeywordKind::Infect);
    t.resolve_all();
    // The Giant got two -1/-1 counters (1/1) before dealing damage: 1 to Polukranos.
    assert_eq!(damage_on(&t, p), 1);
    assert_eq!(t.counters(giant, counters::MINUS1), 2);
}

#[test]
fn polukranos_keeps_its_division_when_a_target_becomes_illegal() {
    cr!("608.2b", "601.2d");
    ruling!(
        "Polukranos, World Eater",
        "If some, but not all, of the ability's targets become illegal, you can't change the division of damage. Damage that would've been dealt to illegal targets simply isn't dealt."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let p = polukranos(&mut t, 2, &[obj(giant), obj(bears)], &[1, 1]);
    crate::r_p046_common::bounce(&mut t, bears);
    t.resolve_all();
    assert_eq!(damage_on(&t, giant), 1, "still 1, not 2");
    // Only the Giant (power 3) deals damage back.
    assert_eq!(damage_on(&t, p), 3);
}

#[test]
fn polukranos_x_is_the_activated_abilitys_x() {
    cr!("107.3m", "701.37a");
    ruling!(
        "Polukranos, World Eater",
        "The value of X in Polukranos's last ability is equal to the value chosen for X when its activated ability was activated."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Craw Wurm");
    let p = polukranos(&mut t, 3, &[obj(giant)], &[3]);
    t.resolve_all();
    assert_eq!(t.counters(p, counters::PLUS1), 3);
    assert_eq!(damage_on(&t, giant), 3);
}

#[test]
fn polukranos_divides_as_put_on_the_stack_among_different_opponents() {
    cr!("601.2d", "603.3d");
    ruling!(
        "Polukranos, World Eater",
        "You choose the division of damage as you put the ability on the stack, not as it resolves. Each target must be assigned at least 1 damage. In multiplayer games, you may choose creatures controlled by different opponents."
    );
    let mut t = TestGame::new(3);
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P2, "Hill Giant");
    let from = t.asked().len();
    polukranos(&mut t, 2, &[obj(a), obj(b)], &[1, 1]);
    let divs: Vec<(u32, u32)> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::Divide {
                total, min_each, ..
            } => Some((total, min_each)),
            _ => None,
        })
        .collect();
    assert_eq!(divs, vec![(2, 1)]);
    assert_eq!(t.stack_len(), 1, "the trigger waits, already divided");
    t.resolve_all();
    assert_eq!((damage_on(&t, a), damage_on(&t, b)), (1, 1));
}

// ---------------------------------------------------------------------------------------
// Eureka, Moment of Truth, Rampant Frogantua
// ---------------------------------------------------------------------------------------

#[test]
fn eureka_x_is_zero_and_nothing_is_cast_or_played() {
    cr!("107.3f", "305.4", "601.1a");
    ruling!(
        "Eureka",
        "If the card has an X in its mana cost, treat X as zero."
    );
    ruling!(
        "Eureka",
        "The cards are put onto the battlefield and will not trigger effects which trigger on such cards being “cast” or “played”."
    );
    supported("Eureka");
    supported("Endless One");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Beast Whisperer");
    let one = t.hand(P0, "Endless One");
    let forest = t.hand(P0, "Forest");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[obj(one)]);
    t.answer_choose(P1, &[]);
    t.answer_choose(P0, &[obj(forest)]);
    t.answer_choose(P1, &[]);
    t.answer_choose(P0, &[]);
    t.answer_choose(P1, &[]);
    cast_new(&mut t, P0, "Eureka", &[]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    // Endless One entered with no counters and died; the Forest wasn't a land play.
    assert!(t.in_graveyard(P0, "Endless One"));
    assert!(t.on_battlefield(forest));
    assert_eq!(t.hand_size(P0), hand - 2, "Beast Whisperer drew nothing");
    assert_eq!(t.g.players[0].lands_played_this_turn, 0);
}

#[test]
fn moment_of_truth_with_two_cards_hand_then_graveyard() {
    cr!("608.2c", "401.2");
    ruling!(
        "Moment of Truth",
        "If there are fewer than three cards in your library, look at all of the cards in your library and follow the instructions in order. For example, if you had only two cards in your library, you would look at both, put one into your hand, and then put the other one in the graveyard."
    );
    supported("Moment of Truth");
    let mut t = TestGame::new(2);
    t.g.players[0].library.clear();
    let cards = stack_library(&mut t, P0, &["Island", "Forest"]);
    t.advance_to(P0, Step::Upkeep);
    t.answer_choose(P0, &[obj(cards[0])]);
    cast_new(&mut t, P0, "Moment of Truth", &[]);
    t.resolve_all();
    assert_eq!(t.zone(cards[0]), Zone::Hand(P0));
    assert_eq!(t.zone(cards[1]), Zone::Graveyard(P0));
    assert_eq!(t.library_size(P0), 0);
}

#[test]
fn rampant_frogantua_counts_every_lost_player_whenever_and_however() {
    cr!("104.3a", "104.3c", "704.5a", "704.5b");
    ruling!(
        "Rampant Frogantua",
        "Rampant Frogantua's second ability counts every player who has lost, no matter how the player lost the game: due to a state-based action (having 0 or less life, trying to draw a card from an empty library, having 10 or more poison counters), because of a spell or effect that states they lose the game, because your awesome might has left them with no choice other than concession, or for any other reason."
    );
    ruling!(
        "Rampant Frogantua",
        "Rampant Frogantua's second ability counts players who lost the game before Rampant Frogantua entered the battlefield."
    );
    supported("Rampant Frogantua");
    let mut t = TestGame::new(6);
    // P2: no life; P3: an effect; P4: an empty library; P5: poison.
    t.g.players[2].life = 0;
    t.g.lose_game(P3);
    t.g.players[4].library.clear();
    t.g.draw_cards(P4, 1);
    t.g.add_counters(pl(P5), "poison", 10, None);
    t.settle();
    for p in [P2, P3, P4, P5] {
        assert!(t.has_lost(p), "{p:?}");
    }
    let f = t.battlefield(P0, "Rampant Frogantua");
    assert_eq!(pt_now(&mut t, f), (43, 43));
}
