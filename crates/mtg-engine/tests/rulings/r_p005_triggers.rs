//! Rulings batch P005 — when abilities trigger and what they see on resolution:
//! intervening "if" clauses (CR 603.4) checked with last known information (CR 608.2h),
//! one event versus several (CR 603.2c), each life gain a separate event (CR 119.9),
//! triggers on casting (CR 601.2i) that lands never cause (CR 305.1), effects that affect
//! only what's there as they resolve (CR 611.2c), a leaves-the-battlefield trigger that
//! resolves before the enters trigger it undoes (CR 607.2a), and "up to" choices.

use crate::r_p006_common::{in_graveyard, mark, obj, pl, targets_of, triggers_from};
use crate::r_p044_common::kill;
use crate::r_p046_common::bounce;
use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s02_common::can_play_land;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const MUTATE: CastMethod = CastMethod::Keyword(KeywordKind::Mutate);

fn clues(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.has_subtype("Clue"))
        .count()
}

#[test]
fn sharp_eyed_rookie_triggers_only_for_a_bigger_creature() {
    cr!("603.4");
    ruling!(
        "Sharp-Eyed Rookie",
        "When a creature enters the battlefield under your control, check its power and toughness against Sharp-Eyed Rookie's power and toughness. If neither stat of the new creature is greater, Sharp-Eyed Rookie's triggered ability won't trigger at all."
    );
    supported("Sharp-Eyed Rookie");
    // (entering creature, triggers): Grizzly Bears 2/2 (neither greater), Hill Giant 3/3,
    // Wall of Stone 0/8 (toughness only), Goblin Piker 2/1 (neither).
    for (name, triggers) in [
        ("Grizzly Bears", false),
        ("Hill Giant", true),
        ("Wall of Stone", true),
        ("Goblin Piker", false),
    ] {
        let mut t = TestGame::new(2);
        let rookie = t.battlefield(P0, "Sharp-Eyed Rookie");
        t.enter(P0, name);
        t.settle();
        assert_eq!(triggers_from(&t, rookie), triggers as usize, "{name}");
        t.resolve_all();
        assert_eq!(
            t.counters(rookie, counters::PLUS1),
            triggers as u32,
            "{name}"
        );
        assert_eq!(clues(&t, P0), triggers as usize, "{name}");
    }
}

#[test]
fn sharp_eyed_rookie_counts_counters_the_creature_enters_with() {
    cr!("603.4", "614.1c");
    ruling!(
        "Sharp-Eyed Rookie",
        "If a creature enters the battlefield with +1/+1 counters on it, consider those counters when determining if Sharp-Eyed Rookie's triggered ability will trigger. For example, a 1/1 creature that enters the battlefield with two +1/+1 counters on it will cause a 2/2 Sharp-Eyed Rookie's ability to trigger."
    );
    supported("Salt Road Quartermasters");
    // Salt Road Quartermasters: a 1/1 that enters with two +1/+1 counters.
    let mut t = TestGame::new(2);
    let rookie = t.battlefield(P0, "Sharp-Eyed Rookie");
    let qm = t.enter(P0, "Salt Road Quartermasters");
    assert_eq!(t.pt(qm), (3, 3));
    t.settle();
    assert_eq!(triggers_from(&t, rookie), 1);
    t.resolve_all();
    assert_eq!(t.pt(rookie), (3, 3));
    assert_eq!(clues(&t, P0), 1);
}

#[test]
fn sharp_eyed_rookie_rechecks_on_resolution_with_last_known_information() {
    cr!("603.4", "608.2h");
    ruling!(
        "Sharp-Eyed Rookie",
        "If Sharp-Eyed Rookie's triggered ability triggers, the stat comparison will happen again when the ability tries to resolve. If neither stat of the new creature is greater, the ability will do nothing. If the creature that entered the battlefield leaves the battlefield before the ability tries to resolve, use its power and toughness as it last existed on the battlefield for the purposes of the comparison."
    );
    // The Rookie grows to 4/4 in response: the Hill Giant (3/3) is no longer bigger.
    let mut t = TestGame::new(2);
    let rookie = t.battlefield(P0, "Sharp-Eyed Rookie");
    t.enter(P0, "Hill Giant");
    t.settle();
    assert_eq!(triggers_from(&t, rookie), 1);
    t.g.add_counters(obj(rookie), counters::PLUS1, 2, None);
    t.g.recompute();
    t.resolve_all();
    assert_eq!(t.counters(rookie, counters::PLUS1), 2);
    assert_eq!(clues(&t, P0), 0);
    // The Hill Giant leaves in response: it was 3/3 as it last existed.
    let mut t = TestGame::new(2);
    let rookie = t.battlefield(P0, "Sharp-Eyed Rookie");
    let giant = t.enter(P0, "Hill Giant");
    t.settle();
    kill(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.counters(rookie, counters::PLUS1), 1);
    assert_eq!(clues(&t, P0), 1);
    // The Hill Giant shrinks to 1/1 (two -1/-1 counters) and then dies in response: as it
    // last existed it isn't bigger, and the ability does nothing.
    let mut t = TestGame::new(2);
    let rookie = t.battlefield(P0, "Sharp-Eyed Rookie");
    let giant = t.enter(P0, "Hill Giant");
    t.settle();
    t.g.add_counters(obj(giant), counters::MINUS1, 2, None);
    t.g.recompute();
    kill(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.counters(rookie, counters::PLUS1), 0);
    assert_eq!(clues(&t, P0), 0);
}

#[test]
fn unsettled_mariner_triggers_once_for_repeated_targets() {
    cr!("603.2c", "115.3");
    ruling!(
        "Unsettled Mariner",
        "If a spell or ability an opponent controls targets only you or only one permanent you control, but it targets you or that permanent multiple times, Unsettled Mariner’s ability triggers only once."
    );
    supported("Unsettled Mariner");
    supported("Seeds of Strength");
    supported("Verdant Command");
    // Seeds of Strength, cast by P1, targets P0's Grizzly Bears three times.
    let mut t = TestGame::new(2);
    let mariner = t.battlefield(P0, "Unsettled Mariner");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 1);
    t.lands(P1, "Plains", 1);
    let seeds = t.hand(P1, "Seeds of Strength");
    t.cast(P1, seeds)
        .target(bears)
        .target(bears)
        .target(bears)
        .go();
    t.settle();
    assert_eq!(triggers_from(&t, mariner), 1);
    // P1 pays {1} (an untapped Forest): the spell resolves, +3/+3.
    t.lands(P1, "Forest", 1);
    t.answer_yes(P1, true);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    // Verdant Command, cast by P1 choosing two modes that each target P0.
    let mut t = TestGame::new(2);
    let mariner = t.battlefield(P0, "Unsettled Mariner");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 1);
    t.lands(P1, "Wastes", 1);
    let cmd = t.hand(P1, "Verdant Command");
    // Modes: "Target player creates two tapped 1/1 green Squirrel creature tokens." and
    // "Target player gains 3 life."
    let cmd = t
        .cast(P1, cmd)
        .modes(&[0, 3])
        .target(pl(P0))
        .target(pl(P0))
        .go();
    assert_eq!(targets_of(&t, cmd), vec![pl(P0), pl(P0)]);
    t.settle();
    assert_eq!(triggers_from(&t, mariner), 1);
    // P1 can't pay: the Command is countered.
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Verdant Command"));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn assaultron_dominator_triggers_for_each_attacking_artifact_creature() {
    cr!("603.2c", "508.1m");
    ruling!(
        "Assaultron Dominator",
        "If multiple artifact creatures you control attack, Assaultron Dominator’s last ability triggers for each of them."
    );
    supported("Assaultron Dominator");
    let mut t = TestGame::new(2);
    let dominator = t.battlefield(P0, "Assaultron Dominator");
    let a = t.battlefield(P0, "Ornithopter");
    let b = t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Player(P0), "energy", 3, None);
    attack_with(
        &mut t,
        &[
            (dominator, pl(P1)),
            (a, pl(P1)),
            (b, pl(P1)),
            (bears, pl(P1)),
        ],
    );
    // Three artifact creatures (not the Bears).
    assert_eq!(triggers_from(&t, dominator), 3);
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    t.resolve_all();
    for id in [dominator, a, b] {
        assert_eq!(t.obj_now(id).counters.values().sum::<u32>(), 1);
    }
    assert!(t.obj_now(bears).counters.is_empty());
}

#[test]
fn spirit_mascot_triggers_once_for_cards_leaving_together() {
    cr!("603.2c", "603.10a");
    ruling!(
        "Spirit Mascot",
        "If multiple cards leave your graveyard at the same time, Spirit Mascot's ability will trigger only once."
    );
    supported("Spirit Mascot");
    supported("Tormod's Crypt");
    let mut t = TestGame::new(2);
    let mascot = t.battlefield(P0, "Spirit Mascot");
    for _ in 0..3 {
        t.graveyard(P0, "Grizzly Bears");
    }
    let crypt = t.battlefield(P1, "Tormod's Crypt");
    t.activate(P1, crypt, 0, &[pl(P0)]).unwrap();
    t.resolve();
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(triggers_from(&t, mascot), 1);
    t.resolve_all();
    assert_eq!(t.counters(mascot, counters::PLUS1), 1);
}

#[test]
fn amalia_triggers_for_each_lifelink_creatures_damage() {
    cr!("119.9", "702.15b", "510.2", "603.2c");
    ruling!(
        "Amalia Benavides Aguirre",
        "If multiple creatures with lifelink you control deal combat damage at the same time, the damage dealt by each of those creatures is a separate life-gaining event and Amalia Benavides Aguirre's ability will trigger that many times."
    );
    supported("Amalia Benavides Aguirre");
    let mut t = TestGame::new(2);
    let amalia = t.battlefield(P0, "Amalia Benavides Aguirre");
    // Two Vampire Nighthawks (lifelink) and a Grizzly Bears attack, unblocked.
    let a = t.battlefield(P0, "Vampire Nighthawk");
    let b = t.battlefield(P0, "Vampire Nighthawk");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(a, pl(P1)), (b, pl(P1)), (bears, pl(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P0), 24);
    assert_eq!(triggers_from(&t, amalia), 2);
}

#[test]
fn historic_cast_triggers_ignore_land_plays_and_transformations() {
    cr!("305.1", "601.2i", "701.27a");
    ruling!(
        "Basim Ibn Ishaq",
        "Lands are never cast, so abilities that trigger “whenever you cast a historic spell” won’t trigger if you play a legendary land. They also won’t trigger if a card on the battlefield transforms into a card with the legendary supertype, the artifact card type, or the Saga subtype."
    );
    supported("Basim Ibn Ishaq");
    supported("Abstergo Entertainment");
    supported("Legion's Landing // Adanto, the First Fort");
    let mut t = TestGame::new(2);
    let basim = t.battlefield(P0, "Basim Ibn Ishaq");
    // Playing a legendary land.
    let land = t.hand(P0, "Abstergo Entertainment");
    assert!(can_play_land(&mut t, P0, land));
    t.play_land(P0, land).unwrap();
    t.settle();
    assert!(t.on_battlefield(land));
    assert_eq!(triggers_from(&t, basim), 0);
    // Legion's Landing transforms into Adanto, the First Fort (a legendary land) when P0
    // attacks with three or more creatures.
    let landing = t.battlefield(P0, "Legion's Landing // Adanto, the First Fort");
    let attackers: Vec<_> = (0..3).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let hand = t.hand_size(P0);
    attack_with(
        &mut t,
        &attackers.iter().map(|a| (*a, pl(P1))).collect::<Vec<_>>(),
    );
    t.resolve_all();
    assert!(t.obj_now(landing).chars.is_legendary());
    assert_eq!(t.obj_now(landing).chars.name, "Adanto, the First Fort");
    assert_eq!(triggers_from(&t, basim), 0);
    assert_eq!(t.hand_size(P0), hand, "Basim drew nothing");
    block_and_finish(&mut t, P1, &[]);
    // A historic spell does trigger it.
    let orni = t.hand(P0, "Ornithopter");
    t.advance_to(P0, Step::PostcombatMain);
    t.cast(P0, orni).go();
    t.settle();
    assert_eq!(triggers_from(&t, basim), 1);
}

#[test]
fn throne_warden_checks_the_monarch_as_the_end_step_begins_and_on_resolution() {
    cr!("603.4");
    ruling!(
        "Throne Warden",
        "The last ability of Throne Warden checks to see if you're the monarch as your end step begins. If you're not, the ability won't trigger at all. You won't be able to do anything that would make you the monarch during your end step in time to have that ability trigger. The ability will also check to see if you're the monarch as it tries to resolve. If you're not the monarch at that time, the ability will have no effect."
    );
    supported("Throne Warden");
    supported("Queen Marchesa");
    supported("Palace Jailer");
    // Not the monarch: no trigger; becoming the monarch during the end step is too late.
    let mut t = TestGame::new(2);
    let warden = t.battlefield(P0, "Throne Warden");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_from(&t, warden), 0);
    t.enter(P0, "Queen Marchesa");
    t.resolve_all();
    assert_eq!(t.g.monarch, Some(P0));
    assert_eq!(triggers_from(&t, warden), 0);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.counters(warden, counters::PLUS1), 0);
    // The monarch: it triggers; P1 becomes the monarch in response (Palace Jailer).
    for lose_it in [false, true] {
        let mut t = TestGame::new(2);
        let warden = t.battlefield(P0, "Throne Warden");
        // The Jailer's exile takes this Grizzly Bears, not the Warden.
        let bears = t.battlefield(P0, "Grizzly Bears");
        mtg_engine::designations::become_monarch(&mut t.g, P0);
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(triggers_from(&t, warden), 1);
        if lose_it {
            t.answer_targets(P1, &[obj(bears)]);
            t.enter(P1, "Palace Jailer");
            t.settle();
        }
        t.resolve_all();
        assert!(t.on_battlefield(warden), "{lose_it}");
        assert_eq!(t.g.monarch, Some(if lose_it { P1 } else { P0 }));
        let expected = if lose_it { 0 } else { 1 };
        assert_eq!(t.counters(warden, counters::PLUS1), expected, "{lose_it}");
    }
}

/// P0 casts `name` for its mutate cost onto `target`, putting it on top; the merged
/// creature's mutate triggers are on the stack.
fn mutate_onto(t: &mut TestGame, name: &str, target: ObjectId, mana: &[(ManaType, u32)]) {
    supported(name);
    for (ty, n) in mana {
        t.g.players[0].mana_pool.add_type(*ty, *n);
    }
    let card = t.hand(P0, name);
    t.cast(P0, card).method(MUTATE).target(target).go();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.resolve();
}

#[test]
fn regal_leosaur_affects_only_creatures_there_as_it_resolves() {
    cr!("611.2c", "702.140d");
    ruling!(
        "Regal Leosaur",
        "Regal Leosaur’s last ability affects only other creatures you control at the time it resolves. Creatures you begin to control later in the turn won’t get +2/+1."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    mutate_onto(
        &mut t,
        "Regal Leosaur",
        bears,
        &[(ManaType::R, 2), (ManaType::C, 1)],
    );
    assert_eq!(t.stack_len(), 1);
    // An Ornithopter enters in response: it's there as the ability resolves.
    let early = t.enter(P0, "Ornithopter");
    t.resolve_all();
    assert_eq!(t.pt(giant), (5, 4));
    assert_eq!(t.pt(early), (2, 3));
    // The mutated creature itself (Regal Leosaur on top, 2/2) doesn't: "other".
    assert_eq!(t.pt(bears), (2, 2));
    // One entering later in the turn doesn't get +2/+1.
    let late = t.enter(P0, "Ornithopter");
    assert_eq!(t.pt(late), (0, 2));
}

#[test]
fn essence_symbiote_gains_life_even_without_the_creature() {
    cr!("609.3", "702.140d");
    ruling!(
        "Essence Symbiote",
        "You gain 2 life when Essence Symbiote’s ability resolves, even if you can’t put a +1/+1 counter on the mutated creature (most likely because it’s left the battlefield)."
    );
    supported("Essence Symbiote");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Essence Symbiote");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Porcuparrot (mutate {2}{R}) on top of the Bears.
    mutate_onto(
        &mut t,
        "Porcuparrot",
        bears,
        &[(ManaType::R, 1), (ManaType::C, 2)],
    );
    assert_eq!(t.stack_len(), 1);
    // The mutated creature is returned to hand in response.
    bounce(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert!(t.in_hand(P0, "Porcuparrot") && t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn obsessive_astronomer_may_discard_no_cards() {
    cr!("731.1a", "608.2d");
    ruling!(
        "Obsessive Astronomer",
        "You may choose to discard no cards when the Obsessive Astronomer's triggered ability resolves."
    );
    supported("Obsessive Astronomer");
    let mut t = TestGame::new(2);
    let astronomer = t.enter(P0, "Obsessive Astronomer");
    assert_eq!(t.g.day, Some(true));
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Hill Giant");
    t.g.set_day(false);
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_from(&t, astronomer), 1);
    let (hand, library) = (t.hand_size(P0), t.library_size(P0));
    let from = mark(&t);
    t.answer_choose(P0, &[]);
    t.resolve_all();
    assert!(t.asked()[from..].iter().any(|(p, d)| *p == P0
        && matches!(
            d,
            mtg_engine::decision::Decision::ChooseEntities { min: 0, .. }
        )));
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.library_size(P0), library, "drew nothing");
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn tidehollow_scullers_leave_trigger_resolves_first_and_does_nothing() {
    cr!("603.6c", "607.2a", "608.2c");
    ruling!(
        "Tidehollow Sculler",
        "If Tidehollow Sculler leaves the battlefield before its first ability has resolved, its second ability triggers. This ability does nothing when it resolves. Then its first ability resolves and exiles the chosen card indefinitely."
    );
    supported("Tidehollow Sculler");
    let mut t = TestGame::new(2);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.hand(P1, "Mountain");
    t.answer_targets(P0, &[pl(P1)]);
    let sculler = t.enter(P0, "Tidehollow Sculler");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, sculler);
    assert_eq!(
        t.stack_len(),
        2,
        "the leave trigger above the enter trigger"
    );
    // The leave trigger resolves: there's no exiled card yet.
    t.resolve();
    assert_eq!(t.hand_size(P1), 2);
    // The enter trigger exiles the Lightning Bolt, for good.
    t.answer_choose(P0, &[obj(bolt)]);
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert_eq!(t.hand_size(P1), 1);
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert!(in_graveyard(&t, sculler));
}
