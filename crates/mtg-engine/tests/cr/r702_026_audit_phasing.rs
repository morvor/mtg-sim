//! CR 702.26 audit: phasing. A phased-out permanent is treated as though it doesn't exist
//! and can't affect or be affected by anything (702.26b) — also when an instruction names
//! it without targeting it ("this creature fights", "exchange control of this creature",
//! "that creature"), and a continuous effect from a resolving ability leaves it out of the
//! set of objects it affects even when it names it specifically (702.26e). Phasing isn't a
//! zone change (702.26d): an effect that applied before it phased out applies again once
//! it phases in, unless it expired meanwhile (702.26f).

use mtg_engine::card::card;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Asserts the compiler fully supports `name`.
fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

fn phased_out(t: &TestGame, id: ObjectId) -> bool {
    t.g.obj(t.g.current(id)).phased_out
}

fn damage_on(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).damage
}

fn pt(t: &mut TestGame, id: ObjectId) -> (i32, i32) {
    t.g.recompute();
    t.pt(id)
}

/// `p` casts Reality Ripple ("Target artifact, creature, or land phases out.") on
/// `target`, on top of whatever is on the stack, and it resolves.
fn ripple(t: &mut TestGame, p: PlayerId, target: ObjectId) {
    supported("Reality Ripple");
    t.lands(p, "Island", 2);
    let spell = t.hand(p, "Reality Ripple");
    t.cast(p, spell).target(target).go();
    t.resolve();
    assert!(phased_out(t, target));
}

#[test]
fn a_creature_that_phased_out_doesnt_fight() {
    cr!("701.14b", "702.26b", "702.26d");
    supported("Brash Taunter");
    let mut t = TestGame::new(2);
    let taunter = t.battlefield(P0, "Brash Taunter");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 3);
    // "{2}{R}, {T}: This creature fights another target creature." In response, the
    // Taunter phases out: treated as though it's not on the battlefield, it doesn't fight.
    t.activate(P0, taunter, 0, &[Entity::Object(giant)])
        .expect("fight ability");
    ripple(&mut t, P0, taunter);
    t.resolve_all();
    assert_eq!(damage_on(&t, giant), 0, "the phased-out Taunter deals no damage");
    assert_eq!(damage_on(&t, taunter), 0, "and is dealt none");
    assert_eq!(t.life(P1), 20, "so its 'whenever it's dealt damage' has nothing to see");
    // Unphased, the same fight happens (and the Taunter passes the 3 damage it's dealt on).
    let mut t = TestGame::new(2);
    let taunter = t.battlefield(P0, "Brash Taunter");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 3);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, taunter, 0, &[Entity::Object(giant)])
        .expect("fight ability");
    t.resolve_all();
    assert_eq!(damage_on(&t, giant), 1);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn a_phased_out_creature_cant_be_exchanged() {
    cr!("701.12a", "702.26b", "702.26d");
    supported("Phyrexian Infiltrator");
    let mut t = TestGame::new(2);
    let infiltrator = t.battlefield(P0, "Phyrexian Infiltrator");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 4);
    // "{2}{U}{U}: Exchange control of this creature and target creature." The Infiltrator
    // phases out in response: it can't change control, so no part of the exchange happens.
    t.activate(P0, infiltrator, 0, &[Entity::Object(giant)])
        .expect("exchange ability");
    ripple(&mut t, P0, infiltrator);
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.obj_now(giant).controller, P1, "the Giant stays with P1");
    assert_eq!(t.obj_now(infiltrator).controller, P0);
    // It phases in during P0's next untap step, still P0's; the Giant is still P1's.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.g.recompute();
    assert!(!phased_out(&t, infiltrator));
    assert_eq!(t.obj_now(infiltrator).controller, P0);
    assert_eq!(t.obj_now(giant).controller, P1);
}

#[test]
fn an_effect_naming_a_phased_out_permanent_doesnt_affect_it_once_it_phases_in() {
    cr!("702.26e", "702.26f", "611.2c");
    ruling!(
        "The Moment",
        "A creature phased out by The Moment doesn't phase in during its controller's untap step as normal. Rather, it phases in immediately after The Moment leaves the battlefield."
    );
    supported("Nantuko Shade");
    supported("The Moment");
    let mut t = TestGame::new(2);
    let shade = t.battlefield(P0, "Nantuko Shade");
    let moment = t.battlefield(P0, "The Moment");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Plains", 2);
    // A first "{B}: This creature gets +1/+1 until end of turn" resolves while it's
    // phased in.
    t.activate(P0, shade, 0, &[]).expect("pump");
    t.resolve();
    assert_eq!(pt(&mut t, shade), (3, 2));
    // A second one is on the stack when The Moment ("Untap target creature you control.
    // It phases out until The Moment leaves the battlefield.") phases the Shade out; it
    // resolves with the Shade phased out, which isn't among the objects it affects.
    t.activate(P0, shade, 0, &[]).expect("pump");
    t.activate(P0, moment, 0, &[Entity::Object(shade)])
        .expect("The Moment");
    t.resolve();
    assert!(phased_out(&t, shade));
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    // The Moment is destroyed: the Shade phases in, with the first bonus (which hasn't
    // expired) and not the second.
    t.lands(P0, "Mountain", 2);
    let shatter = t.hand(P0, "Shatter");
    t.cast(P0, shatter).target(moment).go();
    t.resolve_all();
    assert!(!t.on_battlefield(moment));
    assert!(!phased_out(&t, shade));
    assert_eq!(pt(&mut t, shade), (3, 2));
}

#[test]
fn counters_stay_on_a_permanent_that_phased_out_in_response_to_removing_one() {
    cr!("702.26b", "702.26d", "702.63a");
    supported("Keldon Marauders");
    let mut t = TestGame::new(2);
    // Vanishing 2: it enters with two time counters (its enters trigger hits P1).
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let marauders = t.enter(P0, "Keldon Marauders");
    t.resolve_all();
    assert_eq!(t.counters(marauders, "time"), 2);
    // P0's upkeep: "remove a time counter from it" is on the stack when it phases out.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    ripple(&mut t, P0, marauders);
    t.resolve_all();
    assert_eq!(
        t.counters(marauders, "time"),
        2,
        "no counter is removed from a phased-out permanent"
    );
    // It phases in with both counters during P0's next untap step, and loses one then.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert!(!phased_out(&t, marauders));
    assert_eq!(t.counters(marauders, "time"), 2);
    t.resolve_all();
    assert_eq!(t.counters(marauders, "time"), 1);
    assert!(t.on_battlefield(marauders));
}

#[test]
fn no_counter_moves_onto_a_creature_that_phased_out() {
    cr!("122.5", "702.26b", "702.95a");
    supported("Simic Initiate");
    let mut t = TestGame::new(2);
    // Graft 1: a 0/0 that enters with a +1/+1 counter.
    let initiate = t.enter(P0, "Simic Initiate");
    t.resolve_all();
    assert_eq!(t.counters(initiate, counters::PLUS1), 1);
    // "Whenever another creature enters, you may move a +1/+1 counter from this creature
    // onto it." The Bears phase out in response: the counter can't be put on them, so it
    // isn't removed from the Initiate either.
    t.answer_yes(P0, true);
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    ripple(&mut t, P0, bears);
    t.resolve_all();
    assert!(t.on_battlefield(initiate), "the Initiate keeps its counter and lives");
    assert_eq!(t.counters(initiate, counters::PLUS1), 1);
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
    // A creature that stays phased in gets the counter (and the Initiate, now 0/0, dies).
    t.answer_yes(P0, true);
    let giant = t.enter(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::PLUS1), 1);
    assert!(t.in_graveyard(P0, "Simic Initiate"));
}

#[test]
fn damage_to_a_creature_that_phased_out_isnt_dealt() {
    cr!("702.26b", "506.4");
    supported("Inferno Elemental");
    let mut t = TestGame::new(2);
    let elemental = t.battlefield(P0, "Inferno Elemental");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(elemental, Entity::Player(P1))]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(giant, elemental)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.settle();
    // "Whenever this creature blocks or becomes blocked by a creature, this creature deals
    // 3 damage to that creature." In response, the blocking Giant phases out (and is
    // removed from combat): the damage to it isn't dealt.
    assert_eq!(t.stack_len(), 1);
    ripple(&mut t, P1, giant);
    t.resolve_all();
    assert_eq!(damage_on(&t, giant), 0);
    // The Giant phases in during P1's untap step, undamaged, and is still there.
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert!(!phased_out(&t, giant));
    assert!(t.on_battlefield(giant));
    // Unphased, the trigger kills the blocking Giant.
    let mut t = TestGame::new(2);
    let elemental = t.battlefield(P0, "Inferno Elemental");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(elemental, Entity::Player(P1))]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(giant, elemental)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}
