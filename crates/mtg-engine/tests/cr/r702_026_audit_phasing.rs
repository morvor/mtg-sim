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
    cr!("701.12a", "702.26b", "702.26d", "702.26e");
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
    cr!("702.26e", "611.2c");
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
    // A creature The Moment phased out doesn't phase in during its controller's untap
    // step: it's still phased out in P0's next upkeep.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let moment = t.battlefield(P0, "The Moment");
    t.lands(P0, "Plains", 2);
    t.activate(P0, moment, 0, &[Entity::Object(bears)])
        .expect("The Moment");
    t.resolve();
    assert!(phased_out(&t, bears));
    to_upkeep(&mut t, P0);
    t.settle();
    assert!(phased_out(&t, bears), "not phased in by P0's untap step");
    assert!(t.on_battlefield(moment));
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
    cr!("122.5", "702.26b", "702.58a");
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
fn a_phased_out_permanent_isnt_untapped() {
    cr!("702.26b", "702.26c", "502.3");
    supported("Goblin Sharpshooter");
    let mut t = TestGame::new(2);
    // "This creature doesn't untap during your untap step. Whenever a creature dies, untap
    // this creature. {T}: This creature deals 1 damage to any target."
    let shooter = t.battlefield(P0, "Goblin Sharpshooter");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.activate(P0, shooter, 0, &[Entity::Object(elves)])
        .expect("ping");
    t.resolve();
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    assert!(t.obj_now(shooter).tapped);
    // The untap trigger is on the stack when the Sharpshooter phases out: a phased-out
    // permanent can't be untapped.
    assert_eq!(t.stack_len(), 1);
    ripple(&mut t, P0, shooter);
    t.resolve_all();
    assert!(t.obj_now(shooter).tapped);
    // It phases in during P0's untap step, still tapped, and doesn't untap then.
    to_upkeep(&mut t, P0);
    t.settle();
    assert!(!phased_out(&t, shooter));
    assert!(t.obj_now(shooter).tapped);
    // Phased in, the trigger untaps it.
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.g.destroy(ogre, None);
    t.resolve_all();
    assert!(!t.obj_now(shooter).tapped);
}

#[test]
fn a_phased_out_permanent_doesnt_become_a_copy() {
    cr!("702.26e", "707.2");
    supported("Dimir Doppelganger");
    let mut t = TestGame::new(2);
    let doppelganger = t.battlefield(P0, "Dimir Doppelganger");
    let bears = t.graveyard(P1, "Grizzly Bears");
    // "{1}{U}{B}: Exile target creature card from a graveyard. This creature becomes a copy
    // of that card, except it has this ability." It phases out in response: the card is
    // exiled, but the copy effect doesn't include the phased-out Doppelganger.
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 2);
    t.activate(P0, doppelganger, 0, &[Entity::Object(bears)])
        .expect("ability");
    ripple(&mut t, P0, doppelganger);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    to_upkeep(&mut t, P0);
    t.settle();
    assert!(!phased_out(&t, doppelganger));
    assert_eq!(
        t.obj_now(doppelganger).chars.name.as_str(),
        "Dimir Doppelganger"
    );
    assert_eq!(pt(&mut t, doppelganger), (0, 2));
    // Phased in, the same ability makes it a copy.
    let giant = t.graveyard(P1, "Hill Giant");
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 2);
    t.activate(P0, doppelganger, 0, &[Entity::Object(giant)])
        .expect("ability");
    t.resolve_all();
    assert_eq!(t.obj_now(doppelganger).chars.name.as_str(), "Hill Giant");
}

#[test]
fn a_phased_out_permanent_doesnt_transform() {
    cr!("702.26b", "701.27a");
    supported("Gatstaf Shepherd // Gatstaf Howler");
    let mut t = TestGame::new(2);
    // "At the beginning of each upkeep, if no spells were cast last turn, transform this
    // creature." No spell was cast on turn 1: it triggers in P1's upkeep, and P0 phases
    // it out in response.
    let shepherd = t.battlefield(P0, "Gatstaf Shepherd // Gatstaf Howler");
    to_upkeep(&mut t, P1);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    ripple(&mut t, P0, shepherd);
    t.resolve_all();
    assert_eq!(t.obj_now(shepherd).chars.name.as_str(), "Gatstaf Shepherd");
    // It phases in front face up in P0's untap step (a spell was cast last turn, so it
    // doesn't transform in this upkeep either).
    to_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(!phased_out(&t, shepherd));
    assert_eq!(t.obj_now(shepherd).chars.name.as_str(), "Gatstaf Shepherd");
    assert_eq!(pt(&mut t, shepherd), (2, 2));
    // Phased in, the trigger of a turn after a turn without spells transforms it.
    to_upkeep(&mut t, P1);
    t.resolve_all();
    assert_eq!(t.obj_now(shepherd).chars.name.as_str(), "Gatstaf Howler");
}

/// Advances to `p`'s next upkeep (its untap step's phasing done), triggered abilities not
/// yet put on the stack.
fn to_upkeep(t: &mut TestGame, p: PlayerId) {
    if t.g.turn.active == p {
        let other = t.g.next_player(p);
        t.advance_to(other, Step::Upkeep);
    }
    t.advance_to(p, Step::Upkeep);
}

#[test]
fn phasing_in_doesnt_trigger_enters_abilities() {
    cr!("702.26d", "702.26c");
    ruling!(
        "Phyrexian Dreadnought",
        "Phasing in does not trigger \"enters\" abilities, so you don't have to sacrifice again if it phases in."
    );
    ruling!(
        "Suleiman's Legacy",
        "Does nothing to Djinns or Efreets that phase in while it is on the battlefield."
    );
    supported("Phyrexian Dreadnought");
    supported("Suleiman's Legacy");
    supported("Shimmering Efreet");
    // Phyrexian Dreadnought ("When this creature enters, sacrifice it unless you sacrifice
    // any number of creatures with total power 12 or greater") phases out and in.
    let mut t = TestGame::new(2);
    let dreadnought = t.battlefield(P0, "Phyrexian Dreadnought");
    ripple(&mut t, P0, dreadnought);
    to_upkeep(&mut t, P0);
    t.settle();
    assert!(!phased_out(&t, dreadnought));
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(dreadnought));
    // P1's Shimmering Efreet (phasing) is phased out when P0's Suleiman's Legacy enters
    // ("destroy all Djinns and Efreets"), and phases in under "Whenever a Djinn or Efreet
    // enters, destroy it": neither touches it. (Its own "Whenever this creature phases
    // in, target creature phases out" takes P0's Bears.)
    let mut t = TestGame::new(2);
    let efreet = t.battlefield(P1, "Shimmering Efreet");
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_upkeep(&mut t, P1);
    t.settle();
    assert!(phased_out(&t, efreet));
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Plains", 1);
    let legacy = t.hand(P0, "Suleiman's Legacy");
    t.cast(P0, legacy).go();
    t.resolve_all();
    assert!(t.on_battlefield(efreet));
    t.answer_targets(P1, &[Entity::Object(bears)]);
    to_upkeep(&mut t, P1);
    t.settle();
    assert!(!phased_out(&t, efreet));
    assert_eq!(t.stack_len(), 1, "only the Efreet's own phase-in trigger");
    t.resolve_all();
    assert!(t.on_battlefield(efreet) && !phased_out(&t, efreet));
    assert!(phased_out(&t, bears));
}

#[test]
fn shimmering_efreet_phasing_in_alone_phases_itself_out() {
    cr!("702.26a", "702.26c", "608.2b");
    ruling!(
        "Shimmering Efreet",
        "If it phases in and no other creatures are on the battlefield, it will phase itself back out."
    );
    ruling!(
        "Shimmering Efreet",
        "If two of these phase in, they can both target the same creature and have just that one creature phase out."
    );
    supported("Shimmering Efreet");
    // Alone: "Whenever this creature phases in, target creature phases out." has only
    // itself to target.
    let mut t = TestGame::new(2);
    let efreet = t.battlefield(P0, "Shimmering Efreet");
    to_upkeep(&mut t, P0);
    t.settle();
    assert!(phased_out(&t, efreet), "phasing: out at its controller's untap step");
    to_upkeep(&mut t, P0);
    t.settle();
    assert!(!phased_out(&t, efreet));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(phased_out(&t, efreet), "it phased itself back out");
    // Two of them phase in together and both target P1's Giant: just the Giant phases out.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Shimmering Efreet");
    let b = t.battlefield(P0, "Shimmering Efreet");
    let giant = t.battlefield(P1, "Hill Giant");
    to_upkeep(&mut t, P0);
    t.settle();
    assert!(phased_out(&t, a) && phased_out(&t, b));
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    to_upkeep(&mut t, P0);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(phased_out(&t, giant));
    assert!(!phased_out(&t, a) && !phased_out(&t, b));
}

#[test]
fn a_phase_in_trigger_sees_the_permanents_phasing_in_with_it() {
    cr!("702.26c", "702.26a", "603.2");
    ruling!(
        "King of the Oathbreakers",
        "If King of the Oathbreakers phases in at the same time as one or more other Spirits you control, King of the Oathbreakers's last ability will trigger for each of them."
    );
    supported("King of the Oathbreakers");
    supported("Clever Concealment");
    let mut t = TestGame::new(2);
    // (The Kami comes first in the battlefield's order: it phases in before the King.)
    let kami = t.battlefield(P0, "Kami of Ancient Law");
    let king = t.battlefield(P0, "King of the Oathbreakers");
    // Both phase out ("Any number of target nonland permanents you control phase out";
    // the King also phases each of them out as it becomes the target of a spell).
    t.lands(P0, "Plains", 4);
    let conceal = t.hand(P0, "Clever Concealment");
    t.cast(P0, conceal)
        .targets(&[Entity::Object(king), Entity::Object(kami)])
        .go();
    t.resolve_all();
    assert!(phased_out(&t, king) && phased_out(&t, kami));
    // They phase in together during P0's untap step: "Whenever King of the Oathbreakers or
    // another Spirit you control phases in, create a tapped 1/1 white Spirit creature
    // token with flying" triggers for each.
    to_upkeep(&mut t, P0);
    t.settle();
    assert!(!phased_out(&t, king) && !phased_out(&t, kami));
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Spirit Token").len(), 2);
}

#[test]
fn the_legend_rule_applies_once_a_phased_out_legend_phases_in() {
    cr!("702.26b", "702.26c", "704.5j");
    let mut t = TestGame::new(2);
    let first = t.battlefield(P0, "Isamaru, Hound of Konda");
    ripple(&mut t, P0, first);
    // A second Isamaru while the first is phased out: one legend on the battlefield.
    t.lands(P0, "Plains", 1);
    let second = t.hand(P0, "Isamaru, Hound of Konda");
    t.cast(P0, second).go();
    t.resolve_all();
    let second = t.g.current(second);
    assert!(t.on_battlefield(first) && t.on_battlefield(second));
    // The first phases in during P0's untap step: P0 keeps one of them.
    t.answer_choose(P0, &[Entity::Object(second)]);
    to_upkeep(&mut t, P0);
    t.settle();
    assert!(t.on_battlefield(second));
    assert!(!t.on_battlefield(first));
    assert!(t.in_graveyard(P0, "Isamaru, Hound of Konda"));
}

#[test]
fn phasing_out_isnt_dying_for_morbid() {
    cr!("702.26d");
    supported("Tragic Slip");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    ripple(&mut t, P0, bears);
    // "Target creature gets -1/-1 until end of turn. Morbid — That creature gets -13/-13
    // until end of turn instead if a creature died this turn."
    t.lands(P0, "Swamp", 2);
    let slip = t.hand(P0, "Tragic Slip");
    t.cast(P0, slip).target(giant).go();
    t.resolve_all();
    assert_eq!(pt(&mut t, giant), (2, 2));
    // Once a creature has died, the same spell is -13/-13.
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.g.destroy(ogre, None);
    t.settle();
    let slip = t.hand(P0, "Tragic Slip");
    t.cast(P0, slip).target(giant).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

/// P1 casts Lightning Bolt at P0's Grizzly Bears in P1's main phase, and P0 delays it
/// with Ertai's Meddling (X = 1): the Bolt is exiled with a delay counter, to be put back
/// onto the stack with its target at P1's next upkeep. Returns the Bears.
fn delayed_bolt_at_bears(t: &mut TestGame) -> ObjectId {
    supported("Ertai's Meddling");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(bears).go();
    t.lands(P0, "Island", 2);
    let em = t.hand(P0, "Ertai's Meddling");
    t.cast(P0, em).x(1).target(spell).go();
    t.resolve();
    assert!(t.in_exile("Lightning Bolt"));
    bears
}

#[test]
fn a_permanent_that_phased_out_and_in_is_still_the_object_a_spell_targets() {
    cr!("702.26d", "400.7", "608.2b");
    ruling!(
        "Ertai's Meddling",
        "A targeted spell which is delayed will still succeed even if its target has phased out and back in again."
    );
    // The Bears phase out (and back in during P0's untap step) before the Bolt returns:
    // phasing isn't a zone change, so they're still its target.
    let mut t = TestGame::new(2);
    let bears = delayed_bolt_at_bears(&mut t);
    ripple(&mut t, P0, bears);
    to_upkeep(&mut t, P0);
    t.settle();
    assert!(!phased_out(&t, bears));
    to_upkeep(&mut t, P1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // Exiled and returned instead (Cloudshift), they're a new object: the Bolt has no
    // legal target.
    let mut t = TestGame::new(2);
    let bears = delayed_bolt_at_bears(&mut t);
    t.lands(P0, "Plains", 1);
    let cs = t.hand(P0, "Cloudshift");
    t.cast(P0, cs).target(bears).go();
    t.resolve();
    to_upkeep(&mut t, P1);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(bears)));
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
}

#[test]
fn a_phased_out_source_still_deals_its_abilitys_damage() {
    cr!("113.7a", "702.26b");
    interp!("phased-out-source-still-acts");
    supported("Prodigal Pyromancer");
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    // "{T}: This creature deals 1 damage to any target." It phases out in response:
    // treated as though it doesn't exist, it's like a source that has left the
    // battlefield, and it still deals the damage.
    t.activate(P0, pyro, 0, &[Entity::Player(P1)])
        .expect("ability");
    ripple(&mut t, P0, pyro);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
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
    assert!(!t.g.is_blocking(giant), "removed from combat");
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
