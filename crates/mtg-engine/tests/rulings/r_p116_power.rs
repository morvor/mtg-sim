//! Rulings batch P116 — power-based effects: values of X locked in as an ability or spell
//! resolves (CR 608.2h), affected sets of "attacking creatures get ..." fixed at
//! resolution (CR 611.2c), doubling power and toughness (CR 701.11), intervening "if"
//! clauses about power (CR 603.4), and "leaves-the-battlefield" looks at counters
//! (CR 603.10a).

use crate::r_p116_common::*;
use mtg_engine::ability::{Effect, PlayerRef, TokenSpec, Value};
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{CardType, ColorSet};
use mtg_engine::*;

/// P0 casts Giant Growth on `target` and it resolves (only it).
fn giant_growth(t: &mut TestGame, target: ObjectId) {
    cast_new(t, P0, "Giant Growth", &[obj(target)]);
    t.resolve();
}

#[test]
fn unnatural_growths_double_one_after_the_other() {
    cr!("701.11a", "701.11b", "613.4c");
    ruling!(
        "Unnatural Growth",
        "If you control more than one Unnatural Growth, each one applies independently. For example, if you control two copies of Unnatural Growth, a 2/2 Bear Cub becomes a 4/4 creature when the first ability resolves and then becomes an 8/8 creature when the second one resolves."
    );
    supported("Unnatural Growth");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Unnatural Growth");
    t.battlefield(P0, "Unnatural Growth");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.pt(bears), (4, 4));
    t.resolve();
    assert_eq!(t.pt(bears), (8, 8));
}

#[test]
fn railway_brawler_x_is_the_power_on_resolution() {
    cr!("608.2h", "603.2");
    ruling!(
        "Railway Brawler",
        "The value of X is determined as Railway Brawler’s triggered ability resolves."
    );
    supported("Railway Brawler");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Railway Brawler");
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // In response, Giant Growth: the Bears are 5/5 as the trigger resolves.
    giant_growth(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 5);
}

/// `pump(t, target)` gives `target` +X/+0 or +X/+X where X is its power; checks that X
/// is the power as it resolves and doesn't change afterwards. Returns the (power,
/// toughness) the Bears had after a later Giant Growth.
fn x_locked_in(pump: impl Fn(&mut TestGame, ObjectId), response: bool) -> (i32, i32) {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    pump(&mut t, bears);
    if response {
        // Giant Growth resolves first: X is 5.
        giant_growth(&mut t, bears);
        t.resolve_all();
        return t.pt(bears);
    }
    t.resolve_all();
    // X was 2; a later Giant Growth doesn't change it.
    giant_growth(&mut t, bears);
    t.pt(bears)
}

#[test]
fn rush_of_blood_x_is_locked_in_on_resolution() {
    cr!("608.2h", "611.2a");
    ruling!(
        "Rush of Blood",
        "The value of X is determined when Rush of Blood resolves. The bonus won’t change later in the turn if the creature’s power changes."
    );
    supported("Rush of Blood");
    let rush = |t: &mut TestGame, b: ObjectId| {
        cast_new(t, P0, "Rush of Blood", &[obj(b)]);
    };
    // Target creature gets +X/+0.
    assert_eq!(x_locked_in(rush, false), (2 + 2 + 3, 2 + 3));
    assert_eq!(x_locked_in(rush, true), (5 + 5, 5));
}

#[test]
fn nantuko_mentor_x_is_locked_in_on_resolution() {
    cr!("608.2h", "611.2a");
    ruling!(
        "Nantuko Mentor",
        "The value of X is determined when the ability resolves. The value will not change over time."
    );
    supported("Nantuko Mentor");
    let mentor = |t: &mut TestGame, b: ObjectId| {
        let m = t.battlefield(P0, "Nantuko Mentor");
        t.lands(P0, "Forest", 3);
        t.activate(P0, m, 0, &[obj(b)]).unwrap();
    };
    // Target creature gets +X/+X.
    assert_eq!(x_locked_in(mentor, false), (2 + 2 + 3, 2 + 2 + 3));
    assert_eq!(x_locked_in(mentor, true), (5 + 5, 5 + 5));
}

#[test]
fn duergar_mine_captain_affects_only_creatures_attacking_as_it_resolves() {
    cr!("611.2c", "506.4");
    ruling!(
        "Duergar Mine-Captain",
        "This ability affects only creatures that are attacking at the time it resolves. It won’t affect creatures that attack later in the turn."
    );
    supported("Duergar Mine-Captain");
    // "{1}{R/W}, {Q}: Attacking creatures get +1/+0 until end of turn."
    let mut t = TestGame::new(2);
    let captain = t.battlefield(P0, "Duergar Mine-Captain");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let home = t.battlefield(P0, "Hill Giant");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![
            (captain, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
        ]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.settle();
    t.resolve_all();
    assert!(t.obj_now(captain).tapped);
    t.lands(P0, "Mountain", 2);
    t.activate(P0, captain, 0, &[]).unwrap();
    assert!(!t.obj_now(captain).tapped);
    t.resolve_all();
    assert_eq!(t.pt(captain), (3, 1));
    assert_eq!(t.pt(bears), (3, 2));
    assert_eq!(t.pt(home), (3, 3));
    // A creature put onto the battlefield attacking afterwards isn't affected.
    let spec = TokenSpec {
        name: "Soldier".into(),
        colors: ColorSet::NONE,
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: vec!["Soldier".into()],
        power: Some(1),
        toughness: Some(1),
        abilities: vec![],
        scryfall_name: None,
        pt_values: None,
    };
    let before = t.g.battlefield.clone();
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    t.g.exec(
        &Effect::CreateToken {
            spec,
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: true,
            attacking: true,
        },
        &mut ctx,
    );
    t.g.recompute();
    let token = *t
        .g
        .battlefield
        .iter()
        .find(|o| !before.contains(o))
        .unwrap();
    assert!(t.g.combat.as_ref().unwrap().attacker(token).is_some());
    assert_eq!(t.pt(token), (1, 1));
}

#[test]
fn duelcraft_trainer_rechecks_coven_on_resolution() {
    cr!("603.4", "702.1");
    ruling!(
        "Duelcraft Trainer",
        "Duelcraft Trainer's triggered ability checks to see if you have three or more creatures with different powers both when it triggers and as it tries to resolve."
    );
    supported("Duelcraft Trainer");
    // Duelcraft Trainer (3), Grizzly Bears (2), Llanowar Elves (1).
    let setup = |t: &mut TestGame| {
        t.battlefield(P0, "Duelcraft Trainer");
        let b = t.battlefield(P0, "Grizzly Bears");
        let e = t.battlefield(P0, "Llanowar Elves");
        (b, e)
    };
    // Two powers only: it doesn't trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Duelcraft Trainer");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // Three powers: it triggers; the Elves die in response: removed from the stack.
    let mut t = TestGame::new(2);
    let (bears, elves) = setup(&mut t);
    t.answer_targets(P0, &[obj(bears)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, elves);
    t.resolve_all();
    assert!(!t
        .obj_now(bears)
        .has_keyword(mtg_engine::keywords::KeywordKind::DoubleStrike));
    // Still three: double strike.
    let mut t = TestGame::new(2);
    let (bears, _) = setup(&mut t);
    t.answer_targets(P0, &[obj(bears)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    t.resolve_all();
    assert!(t
        .obj_now(bears)
        .has_keyword(mtg_engine::keywords::KeywordKind::DoubleStrike));
}

#[test]
fn bugenhagen_checks_for_power_seven_on_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Bugenhagen, Wise Elder",
        "Bugenhagen's second ability checks at the moment it would trigger to see if you control a creature with power 7 or greater."
    );
    supported("Bugenhagen, Wise Elder");
    let upkeep = |t: &mut TestGame| {
        t.set_step(P1, Step::End);
        t.advance_to(P0, Step::Upkeep);
        t.settle();
    };
    // No such creature: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bugenhagen, Wise Elder");
    t.battlefield(P0, "Craw Wurm");
    upkeep(&mut t);
    assert_eq!(t.stack_len(), 0);
    // A 7-power creature (Colossal Dreadmaw with a +1/+1 counter) that's gone as it
    // resolves: nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bugenhagen, Wise Elder");
    let dreadmaw2 = t.battlefield(P0, "Colossal Dreadmaw");
    put_counters(&mut t, dreadmaw2, "+1/+1", 1);
    upkeep(&mut t);
    assert_eq!(t.stack_len(), 1);
    let hand = t.hand_size(P0);
    destroy(&mut t, dreadmaw2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // Still there: draw.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bugenhagen, Wise Elder");
    let d = t.battlefield(P0, "Colossal Dreadmaw");
    put_counters(&mut t, d, "+1/+1", 1);
    upkeep(&mut t);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn garruks_packleader_sees_a_creature_entering_with_it() {
    cr!("603.6a", "603.10a");
    ruling!(
        "Garruk's Packleader",
        "If Garruk's Packleader and another creature with power 3 or greater enter under your control at the same time, the ability will trigger."
    );
    supported("Garruk's Packleader");
    let mut t = TestGame::new(2);
    t.answer_yes(P0, true);
    enter_together(&mut t, P0, &["Garruk's Packleader", "Hill Giant"]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // With a 2-power creature: no trigger.
    let mut t = TestGame::new(2);
    enter_together(&mut t, P0, &["Garruk's Packleader", "Grizzly Bears"]);
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn parish_blade_trainee_gives_minus_counters_too() {
    cr!("603.10a", "704.5f");
    ruling!(
        "Parish-Blade Trainee",
        "If Parish-Blade Trainee has -1/-1 counters on it when it dies, the last ability will include those as well. This may result in the recipient also dying."
    );
    supported("Parish-Blade Trainee");
    let mut t = TestGame::new(2);
    let trainee = t.battlefield(P0, "Parish-Blade Trainee");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    put_counters(&mut t, trainee, "-1/-1", 2);
    assert!(!t.on_battlefield(trainee));
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn evolutionary_leap_without_a_creature_card_puts_everything_back() {
    cr!("701.20a", "401.4");
    ruling!(
        "Evolutionary Leap",
        "If you don't reveal a creature card, you'll reveal all the cards from your library and then put them back in your library in a random order."
    );
    supported("Evolutionary Leap");
    // "{G}, Sacrifice a creature: Reveal cards from the top of your library until you
    // reveal a creature card. Put that card into your hand and the rest on the bottom of
    // your library in a random order." The library has no creature cards.
    let mut t = TestGame::new(2);
    let leap = t.battlefield(P0, "Evolutionary Leap");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let lib = t.library_size(P0);
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, leap, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.library_size(P0), lib);
    assert_eq!(t.hand_size(P0), hand);
}

/// Evolving Adaptive for P0, entering with its oil counter, then `oil - 1` more.
fn adaptive(t: &mut TestGame, oil: u32) -> ObjectId {
    supported("Evolving Adaptive");
    // "This creature enters with an oil counter on it. This creature gets +1/+1 for each
    // oil counter on it. Whenever another creature you control enters, if that creature
    // has greater power or toughness than this creature, put an oil counter on this
    // creature."
    let a = t.enter(P0, "Evolving Adaptive");
    t.settle();
    if oil > 1 {
        put_counters(t, a, "oil", oil - 1);
    }
    assert_eq!(t.pt(a), (oil as i32, oil as i32));
    a
}

#[test]
fn evolving_adaptive_counts_counters_the_creature_enters_with() {
    cr!("603.4", "614.1c");
    ruling!(
        "Evolving Adaptive",
        "If a creature enters the battlefield with +1/+1 counters on it, consider those counters when determining if Evolving Adaptive's ability will trigger."
    );
    // Spike Feeder: a 0/0 that enters with two +1/+1 counters.
    let mut t = TestGame::new(2);
    let a = adaptive(&mut t, 1);
    t.enter(P0, "Spike Feeder");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(a, "oil"), 2);
    // A 1/1 doesn't make it trigger.
    let mut t = TestGame::new(2);
    let a = adaptive(&mut t, 1);
    t.enter(P0, "Llanowar Elves");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.counters(a, "oil"), 1);
}

#[test]
fn evolving_adaptive_compares_again_as_each_trigger_resolves() {
    cr!("603.4", "603.2");
    ruling!(
        "Evolving Adaptive",
        "If multiple creatures enter the battlefield at the same time, Evolving Adaptive's ability may trigger multiple times, although the stat comparison will take place each time one of those abilities tries to resolve."
    );
    supported("Centaur Courser");
    let mut t = TestGame::new(2);
    let a = adaptive(&mut t, 2);
    enter_together(&mut t, P0, &["Centaur Courser", "Centaur Courser"]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.pt(a), (3, 3));
    t.resolve_all();
    assert_eq!(t.counters(a, "oil"), 3);
}

#[test]
fn evolving_adaptive_uses_last_known_stats_of_a_creature_that_left() {
    cr!("603.4", "608.2h");
    ruling!(
        "Evolving Adaptive",
        "If the ability triggers, the stat comparison will happen again when the ability tries to resolve. If neither stat of the new creature is greater, the ability will do nothing. If the creature that entered the battlefield leaves the battlefield before the ability tries to resolve, use its last known power and toughness to compare the stats."
    );
    // Hill Giant (3/3) enters and is destroyed in response: still greater.
    let mut t = TestGame::new(2);
    let a = adaptive(&mut t, 1);
    let giant = t.enter(P0, "Hill Giant");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.counters(a, "oil"), 2);
    // Evolving Adaptive grew in response: neither stat is greater, nothing happens.
    let mut t = TestGame::new(2);
    let a = adaptive(&mut t, 1);
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    put_counters(&mut t, a, "oil", 1);
    t.resolve_all();
    assert_eq!(t.counters(a, "oil"), 2);
}

#[test]
fn hulkling_compares_power_to_power_and_toughness_to_toughness() {
    cr!("603.4");
    supported("Hulkling, Burgeoning Bruiser");
    // A 2/3: Grizzly Bears (2/2) isn't greater; Hill Giant (3/3) is.
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hulkling, Burgeoning Bruiser");
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.enter(P0, "Hill Giant");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(h), (3, 4));
}

#[test]
fn primal_empathy_a_shared_greatest_power_draws() {
    cr!("608.2c", "603.3");
    ruling!(
        "Primal Empathy",
        "If the greatest power among creatures on the battlefield is shared by a creature you control and a creature you don't control, you draw a card."
    );
    supported("Primal Empathy");
    supported("High Score");
    // "At the beginning of your upkeep, draw a card if you control a creature with the
    // greatest power among creatures on the battlefield. Otherwise, put a +1/+1 counter on
    // a creature you control."
    let upkeep = |t: &mut TestGame| {
        t.set_step(P1, Step::End);
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        t.resolve_all();
    };
    // Tied at 3: P0 draws.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Primal Empathy");
    let mine = t.battlefield(P0, "Hill Giant");
    t.battlefield(P1, "Hill Giant");
    let hand = t.hand_size(P0);
    upkeep(&mut t);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.counters(mine, "+1/+1"), 0);
    // P1's creature is greater: a +1/+1 counter instead.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Primal Empathy");
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    let hand = t.hand_size(P0);
    upkeep(&mut t);
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.counters(mine, "+1/+1"), 1);
}
