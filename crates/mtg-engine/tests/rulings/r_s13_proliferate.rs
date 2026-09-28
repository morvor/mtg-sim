//! Rulings batch S13 — proliferate (CR 701.34a): "To proliferate means to choose any
//! number of permanents and/or players that have a counter, then give each one additional
//! counter of each kind that permanent or player already has."

use crate::r_s01_common::*;
use crate::r_s03_common::choice_candidates;
use crate::r_s13_common::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::game::Game;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The prompt of the proliferate choice.
const PROLIFERATE: &str = "Proliferate: choose";

/// Karn's Bastion ("{4}, {T}: Proliferate.") with four Wastes to pay for it.
fn bastion(t: &mut TestGame, p: PlayerId) -> ObjectId {
    t.lands(p, "Wastes", 4);
    t.battlefield(p, "Karn's Bastion")
}

/// Activates Karn's Bastion's proliferate ability (its second activated ability).
fn activate_bastion(t: &mut TestGame, p: PlayerId, bastion: ObjectId) {
    t.activate(p, bastion, 1, &[]).expect("Karn's Bastion");
}

/// The candidates of each proliferate choice asked since decision `from`.
fn proliferate_candidates(t: &TestGame, from: usize) -> Vec<Vec<Entity>> {
    choice_candidates(t, from, PROLIFERATE)
}

/// Indices (in the decision log) of the proliferate choices.
fn proliferate_choices(t: &TestGame) -> Vec<usize> {
    t.asked()
        .iter()
        .enumerate()
        .filter(|(_, (_, d))| {
            matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.starts_with(PROLIFERATE))
        })
        .map(|(i, _)| i)
        .collect()
}

/// A creature with one +1/+1 counter and one flying counter.
fn two_kinds(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let c = t.battlefield(p, "Grizzly Bears");
    add(t, c, counters::PLUS1, 1);
    add(t, c, "flying", 1);
    c
}

#[test]
fn players_can_respond_to_proliferating_but_not_to_its_choices() {
    cr!("701.34a", "117.3c", "117.3b", "608.2c");
    ruling!(
        "Karn's Bastion",
        "Players can respond to a spell or ability whose effect includes proliferating. Once that spell or ability starts to resolve, however, and its controller chooses which permanents and players will get new counters, it's too late for anyone to respond."
    );
    ruling!(
        "Radstorm",
        "Once that spell or ability starts to resolve, however, and its controller chooses which permanents and players will get new counters, it’s too late for anyone to respond."
    );
    supported("Karn's Bastion");
    supported("Radstorm");
    // Karn's Bastion's ability is on the stack; the opponent responds by killing one of
    // the creatures with a counter (through the priority loop).
    let mut t = TestGame::new(2);
    let b = bastion(&mut t, P0);
    let doomed = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, doomed, counters::PLUS1, 1);
    let kept = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, kept, counters::PLUS1, 1);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.answer(
        P1,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: bolt,
            method: CastMethod::Normal,
        }),
    );
    t.answer_targets(P1, &[Entity::Object(doomed)]);
    t.answer_choose(P0, &[Entity::Object(kept)]);
    // Whenever P0 gets priority: the +1/+1 counters on the battlefield and the stack size.
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::Priority { .. }),
        |g: &Game| {
            let n: u32 = g.permanents().map(|o| o.counter(counters::PLUS1)).sum();
            (n, g.stack.len())
        },
    );
    activate_bastion(&mut t, P0, b);
    let from = t.asked().len();
    // Through the priority loop until the game moves on to combat.
    assert!(t.g.run_until(1000, |g| g.turn.step != Step::PrecombatMain));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // The creature killed in response wasn't there to be chosen.
    assert_eq!(
        proliferate_candidates(&t, from),
        vec![vec![Entity::Object(kept)]]
    );
    assert_eq!(t.counters(kept, counters::PLUS1), 2);
    // Once the ability resolved, the next time anyone had priority the counter was
    // already there: no one could respond between the choice and the counters.
    let i = proliferate_choices(&t)[0];
    let asked = t.asked();
    let (p, next) = &asked[i + 1];
    assert_eq!(*p, P0);
    assert!(matches!(next, Decision::Priority { .. }));
    let seen = seen.lock().unwrap();
    assert!(seen.contains(&(2, 1)), "P0 had priority with the ability on the stack");
    assert!(seen.contains(&(2, 0)));
    assert!(seen.iter().all(|(n, stack)| *stack > 0 || *n == 2));

    // Radstorm (an instant: "Storm. Proliferate."): the opponent responds the same way.
    let mut t = TestGame::new(2);
    let doomed = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, doomed, counters::PLUS1, 1);
    let kept = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, kept, counters::PLUS1, 1);
    let radstorm = t.hand(P0, "Radstorm");
    give_mana_for(&mut t, P0, "Radstorm");
    t.cast(P0, radstorm).go();
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(doomed).go();
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Object(kept)]);
    t.resolve_all();
    assert_eq!(
        proliferate_candidates(&t, from),
        vec![vec![Entity::Object(kept)]]
    );
    assert_eq!(t.counters(kept, counters::PLUS1), 2);
}

#[test]
fn a_chosen_permanent_or_player_gets_one_of_each_kind_of_counter() {
    cr!("701.34a");
    ruling!(
        "Karn's Bastion",
        "If a player or permanent has more than one kind of counter on it, and you choose for it to get additional counters, it must get one of each kind of counter it already has."
    );
    ruling!(
        "Atomize",
        "If a player or permanent has more than one kind of counter on it, and you choose for it to get additional counters, it must get one of each kind of counter it already has."
    );
    ruling!(
        "Smell Fear",
        "While proliferating, if you choose a permanent or player with multiple kinds of counters, the permanent or player gets another counter of each kind, not just one kind."
    );
    ruling!(
        "Surge Conductor",
        "If a permanent or player has more than one kind of counter on them, and you choose for that permanent or player to get additional counters, that permanent or player must get one of each kind of counter they already have."
    );
    for c in ["Karn's Bastion", "Atomize", "Smell Fear", "Surge Conductor"] {
        supported(c);
    }
    // Karn's Bastion: a creature with a +1/+1 and a flying counter, and a player with
    // energy and experience counters.
    let mut t = TestGame::new(2);
    let b = bastion(&mut t, P0);
    let c = two_kinds(&mut t, P0);
    add(&mut t, P0, counters::ENERGY, 2);
    add(&mut t, P0, counters::EXPERIENCE, 1);
    t.answer_choose(P0, &[Entity::Object(c), Entity::Player(P0)]);
    activate_bastion(&mut t, P0, b);
    t.resolve_all();
    assert_eq!(t.counters(c, counters::PLUS1), 2);
    assert_eq!(t.counters(c, "flying"), 2);
    assert_eq!(t.g.player(P0).counter(counters::ENERGY), 3);
    assert_eq!(t.g.player(P0).counter(counters::EXPERIENCE), 2);
    assert_eq!(t.pt(c), (4, 4));

    // Atomize ("Destroy target nonland permanent. Proliferate.").
    let mut t = TestGame::new(2);
    let c = two_kinds(&mut t, P0);
    let victim = t.battlefield(P1, "Grizzly Bears");
    let atomize = t.hand(P0, "Atomize");
    give_mana_for(&mut t, P0, "Atomize");
    t.answer_choose(P0, &[Entity::Object(c)]);
    t.cast(P0, atomize).target(victim).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(
        (t.counters(c, counters::PLUS1), t.counters(c, "flying")),
        (2, 2)
    );

    // Smell Fear ("Proliferate. Target creature you control fights up to one target
    // creature you don't control."): both kinds, then the 3/3 fights.
    let mut t = TestGame::new(2);
    let c = two_kinds(&mut t, P0);
    let foe = t.battlefield(P1, "Grizzly Bears");
    let smell = t.hand(P0, "Smell Fear");
    give_mana_for(&mut t, P0, "Smell Fear");
    t.answer_choose(P0, &[Entity::Object(c)]);
    t.cast(P0, smell).target(c).target(foe).go();
    t.resolve_all();
    assert_eq!(
        (t.counters(c, counters::PLUS1), t.counters(c, "flying")),
        (2, 2)
    );
    assert!(t.on_battlefield(c));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));

    // Surge Conductor ("Whenever another nontoken artifact you control enters,
    // proliferate."): the opponent, with poison and rad counters, gets one of each.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Surge Conductor");
    add(&mut t, P1, counters::POISON, 2);
    add(&mut t, P1, counters::RAD, 3);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Ornithopter");
    t.resolve_all();
    assert_eq!(t.g.player(P1).counter(counters::POISON), 3);
    assert_eq!(t.g.player(P1).counter(counters::RAD), 4);
}

#[test]
fn you_choose_any_number_of_them_including_none() {
    cr!("701.34a");
    ruling!(
        "Karn's Bastion",
        "You don't have to choose every permanent or player that has a counter, only the ones you want to add another counter to."
    );
    ruling!(
        "Metastatic Evangel",
        "You don't have to choose every permanent or player that has a counter—only the ones you want to add counters to."
    );
    ruling!(
        "Kilo, Apogee Mind",
        "You don’t have to choose every permanent or player that has a counter—only the ones you want to add counters to."
    );
    ruling!(
        "Unbounded Potential",
        "You don't have to choose every permanent or player that has a counter, only the ones you want to add another counter to."
    );
    for c in [
        "Karn's Bastion",
        "Metastatic Evangel",
        "Kilo, Apogee Mind",
        "Unbounded Potential",
    ] {
        supported(c);
    }
    let setup = |t: &mut TestGame| -> (ObjectId, ObjectId) {
        let a = t.battlefield(P0, "Grizzly Bears");
        add(t, a, counters::PLUS1, 1);
        let z = t.battlefield(P1, "Grizzly Bears");
        add(t, z, counters::PLUS1, 1);
        add(t, P0, counters::ENERGY, 1);
        add(t, P1, counters::POISON, 1);
        (a, z)
    };
    let totals = |t: &TestGame, a: ObjectId, z: ObjectId| {
        (
            t.counters(a, counters::PLUS1),
            t.counters(z, counters::PLUS1),
            t.g.player(P0).counter(counters::ENERGY),
            t.g.player(P1).counter(counters::POISON),
        )
    };
    // Karn's Bastion: only some of them (one permanent and one player) ...
    let mut t = TestGame::new(2);
    let b = bastion(&mut t, P0);
    let (a, z) = setup(&mut t);
    t.answer_choose(P0, &[Entity::Object(a), Entity::Player(P1)]);
    activate_bastion(&mut t, P0, b);
    t.resolve_all();
    assert_eq!(totals(&t, a, z), (2, 1, 1, 2));
    // ... or none at all.
    t.g.objects[b.0 as usize].tapped = false;
    t.lands(P0, "Wastes", 4);
    t.answer_choose(P0, &[]);
    activate_bastion(&mut t, P0, b);
    t.resolve_all();
    assert_eq!(totals(&t, a, z), (2, 1, 1, 2));
    assert_eq!(proliferate_choices(&t).len(), 2);

    // Metastatic Evangel ("Whenever another nontoken creature you control enters,
    // proliferate."): no permanents, only a player.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Metastatic Evangel");
    let (a, z) = setup(&mut t);
    t.answer_choose(P0, &[Entity::Player(P0)]);
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(totals(&t, a, z), (1, 1, 2, 1));

    // Kilo, Apogee Mind ("Whenever Kilo becomes tapped, proliferate."): nothing chosen.
    let mut t = TestGame::new(2);
    let kilo = t.battlefield(P0, "Kilo, Apogee Mind");
    let (a, z) = setup(&mut t);
    t.answer_choose(P0, &[]);
    t.g.tap(kilo);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(proliferate_choices(&t).len(), 1);
    assert_eq!(totals(&t, a, z), (1, 1, 1, 1));

    // Unbounded Potential, proliferate mode: only the opponent's creature.
    let mut t = TestGame::new(2);
    let (a, z) = setup(&mut t);
    let up = t.hand(P0, "Unbounded Potential");
    give_mana_for(&mut t, P0, "Unbounded Potential");
    t.answer_choose(P0, &[Entity::Object(z)]);
    t.cast(P0, up).modes(&[1]).go();
    t.resolve_all();
    assert_eq!(totals(&t, a, z), (1, 2, 1, 1));
}

#[test]
fn whenever_you_proliferate_triggers_even_if_you_chose_nothing() {
    cr!("701.34a", "603.2");
    ruling!(
        "Ezuri, Stalker of Spheres",
        "An ability that triggers \"Whenever you proliferate\" triggers even if you chose no permanents or players while doing so."
    );
    // Ezuri, Stalker of Spheres: "When Ezuri enters, you may pay {3}. If you do,
    // proliferate twice. Whenever you proliferate, draw a card."
    supported("Ezuri, Stalker of Spheres");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ezuri, Stalker of Spheres");
    let b = bastion(&mut t, P0);
    let bear = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, bear, counters::PLUS1, 1);
    // Nothing chosen, though the Bears could have been.
    t.answer_choose(P0, &[]);
    activate_bastion(&mut t, P0, b);
    t.resolve_all();
    assert_eq!(proliferate_choices(&t).len(), 1);
    assert_eq!(t.counters(bear, counters::PLUS1), 1);
    assert_eq!(t.hand_size(P0), 1);
    // Nothing that could be chosen at all.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ezuri, Stalker of Spheres");
    let b = bastion(&mut t, P0);
    activate_bastion(&mut t, P0, b);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    // Proliferating twice (Ezuri's own enters ability, paying {3}) triggers twice.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 3);
    t.answer_yes(P0, true);
    t.enter(P0, "Ezuri, Stalker of Spheres");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn any_permanent_or_player_with_a_counter_can_be_chosen_but_no_other_cards() {
    cr!("701.34a", "122.1");
    ruling!(
        "Karn's Bastion",
        "To proliferate, you can choose any permanent that has a counter, including ones controlled by opponents. You can choose any player who has a counter, including opponents. You can't choose cards in any zone other than the battlefield, even if they have counters on them."
    );
    ruling!(
        "Metastatic Evangel",
        "When you proliferate, you can choose any permanent that has a counter, including ones controlled by opponents. You can choose any player who has a counter, including opponents. You can't choose cards in any zone other than the battlefield"
    );
    ruling!(
        "Atomize",
        "You can choose any permanent that has a counter, including ones controlled by opponents. You can choose any player who has a counter, including opponents. You can’t choose cards in any zone other than the battlefield"
    );
    ruling!(
        "Smell Fear",
        "To proliferate, you can choose any permanent that has a counter, including ones controlled by opponents, and you can choose any player who has a counter, including opponents."
    );
    ruling!(
        "Kilo, Apogee Mind",
        "When you proliferate, you can choose any permanent that has a counter, including ones controlled by opponents. You can choose any player who has a counter, including opponents. You can’t choose cards in any zone other than the battlefield"
    );
    for c in [
        "Karn's Bastion",
        "Metastatic Evangel",
        "Atomize",
        "Smell Fear",
        "Kilo, Apogee Mind",
    ] {
        supported(c);
    }
    // P0's and P1's creatures with counters, P1 with poison counters; P0's creature with
    // none, P0 with no counters, and a card in exile with time counters.
    let setup = |t: &mut TestGame| -> (ObjectId, ObjectId, Vec<Entity>) {
        let mine = t.battlefield(P0, "Grizzly Bears");
        add(t, mine, counters::PLUS1, 1);
        let theirs = t.battlefield(P1, "Grizzly Bears");
        add(t, theirs, counters::PLUS1, 2);
        t.battlefield(P0, "Memnite");
        add(t, P1, counters::POISON, 4);
        let exiled = t.exile(P0, "Rift Bolt");
        t.g.objects[exiled.0 as usize]
            .counters
            .insert(counters::TIME.into(), 3);
        let mut expected = vec![
            Entity::Object(mine),
            Entity::Object(theirs),
            Entity::Player(P1),
        ];
        expected.sort();
        (theirs, exiled, expected)
    };
    let check = |t: &TestGame, from: usize, expected: &[Entity], theirs: ObjectId, exiled: ObjectId| {
        let mut cands = proliferate_candidates(t, from).remove(0);
        cands.sort();
        assert_eq!(cands, expected);
        assert_eq!(t.counters(theirs, counters::PLUS1), 3);
        assert_eq!(t.g.player(P1).counter(counters::POISON), 5);
        assert_eq!(t.g.obj(exiled).counter(counters::TIME), 3);
    };
    // Karn's Bastion: the opponent's creature and the opponent are chosen.
    let mut t = TestGame::new(2);
    let b = bastion(&mut t, P0);
    let (theirs, exiled, expected) = setup(&mut t);
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Object(theirs), Entity::Player(P1)]);
    activate_bastion(&mut t, P0, b);
    t.resolve_all();
    check(&t, from, &expected, theirs, exiled);

    // Metastatic Evangel, triggered by another creature entering.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Metastatic Evangel");
    let (theirs, exiled, expected) = setup(&mut t);
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Object(theirs), Entity::Player(P1)]);
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    check(&t, from, &expected, theirs, exiled);

    // Atomize, destroying the Memnite.
    let mut t = TestGame::new(2);
    let (theirs, exiled, expected) = setup(&mut t);
    let memnite = t.named_on_battlefield("Memnite")[0];
    let atomize = t.hand(P0, "Atomize");
    give_mana_for(&mut t, P0, "Atomize");
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Object(theirs), Entity::Player(P1)]);
    t.cast(P0, atomize).target(memnite).go();
    t.resolve_all();
    check(&t, from, &expected, theirs, exiled);

    // Smell Fear, with no creature to fight.
    let mut t = TestGame::new(2);
    let (theirs, exiled, expected) = setup(&mut t);
    let mine = t.named_on_battlefield("Memnite")[0];
    let smell = t.hand(P0, "Smell Fear");
    give_mana_for(&mut t, P0, "Smell Fear");
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Object(theirs), Entity::Player(P1)]);
    t.cast(P0, smell).target(mine).targets(&[]).go();
    t.resolve_all();
    check(&t, from, &expected, theirs, exiled);

    // Kilo, Apogee Mind, becoming tapped.
    let mut t = TestGame::new(2);
    let kilo = t.battlefield(P0, "Kilo, Apogee Mind");
    let (theirs, exiled, expected) = setup(&mut t);
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Object(theirs), Entity::Player(P1)]);
    t.g.tap(kilo);
    t.g.flush_events();
    t.resolve_all();
    check(&t, from, &expected, theirs, exiled);
    assert_eq!(t.zone(exiled), Zone::Exile);
}

#[test]
fn plus_one_and_minus_one_counters_are_removed_in_pairs() {
    cr!("704.5q", "704.3", "701.34a");
    ruling!(
        "Yawgmoth, Thran Physician",
        "If a permanent ever has both +1/+1 counters and -1/-1 counters on it at the same time, they're removed in pairs as a state-based action so that the permanent has only one of those kinds of counters on it."
    );
    supported("Yawgmoth, Thran Physician");
    // Yawgmoth: "Pay 1 life, Sacrifice another creature: Put a -1/-1 counter on up to one
    // target creature and draw a card." and "{B}{B}, Discard a card: Proliferate."
    let mut t = TestGame::new(2);
    let yawgmoth = t.battlefield(P0, "Yawgmoth, Thran Physician");
    let fodder = t.battlefield(P0, "Memnite");
    let bear = t.battlefield(P1, "Grizzly Bears");
    add(&mut t, bear, counters::PLUS1, 2);
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    t.activate(P0, yawgmoth, 0, &[Entity::Object(bear)])
        .expect("Yawgmoth's first ability");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Memnite"));
    // The -1/-1 counter and one +1/+1 counter were removed together.
    assert_eq!(t.counters(bear, counters::PLUS1), 1);
    assert_eq!(t.counters(bear, counters::MINUS1), 0);
    assert_eq!(t.pt(bear), (3, 3));
    // So proliferating gives it only another +1/+1 counter.
    t.lands(P0, "Swamp", 2);
    let discard = t.hand(P0, "Memnite");
    t.answer_choose(P0, &[Entity::Object(discard)]);
    t.answer_choose(P0, &[Entity::Object(bear)]);
    t.activate(P0, yawgmoth, 1, &[]).expect("Yawgmoth's proliferate");
    t.resolve_all();
    assert_eq!(t.counters(bear, counters::PLUS1), 2);
    assert_eq!(t.counters(bear, counters::MINUS1), 0);
}

#[test]
fn a_proliferate_spell_whose_targets_are_all_illegal_doesnt_resolve() {
    cr!("608.2b", "701.34a");
    ruling!(
        "Atomize",
        "Some spells and abilities that cause you to proliferate may require targets. If each target chosen is an illegal target as that spell or ability tries to resolve, it won’t resolve. You won’t proliferate."
    );
    supported("Atomize");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, bear, counters::PLUS1, 1);
    let victim = t.battlefield(P1, "Memnite");
    let atomize = t.hand(P0, "Atomize");
    give_mana_for(&mut t, P0, "Atomize");
    t.answer_choose(P0, &[Entity::Object(bear)]);
    t.cast(P0, atomize).target(victim).go();
    // The target leaves the battlefield before Atomize resolves.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(victim).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Atomize"));
    assert!(proliferate_choices(&t).is_empty());
    assert_eq!(t.counters(bear, counters::PLUS1), 1);
}

#[test]
fn proliferating_several_times_chooses_anew_each_time() {
    cr!("701.34a", "107.3a");
    ruling!(
        "Expansion Algorithm",
        "If you proliferate multiple times, you don't have to choose the same set of players and/or permanents to get additional counters each time."
    );
    // Expansion Algorithm: "Proliferate X times."
    supported("Expansion Algorithm");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, a, counters::PLUS1, 1);
    let b = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, b, counters::PLUS1, 1);
    add(&mut t, P1, counters::POISON, 1);
    let ea = t.hand(P0, "Expansion Algorithm");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 3);
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.answer_choose(P0, &[Entity::Object(b), Entity::Player(P1)]);
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.cast(P0, ea).x(3).go();
    t.resolve_all();
    assert_eq!(proliferate_choices(&t).len(), 3);
    assert_eq!(t.counters(a, counters::PLUS1), 3);
    assert_eq!(t.counters(b, counters::PLUS1), 3);
    assert_eq!(t.g.player(P1).counter(counters::POISON), 2);
    // X = 0: no proliferating at all.
    let mut t = TestGame::new(2);
    let ea = t.hand(P0, "Expansion Algorithm");
    let a = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, a, counters::PLUS1, 1);
    t.lands(P0, "Island", 2);
    t.cast(P0, ea).x(0).go();
    t.resolve_all();
    assert!(proliferate_choices(&t).is_empty());
    assert_eq!(t.counters(a, counters::PLUS1), 1);
}

#[test]
fn no_one_can_respond_between_one_proliferate_and_the_next() {
    cr!("701.34a", "608.2c", "117.3b");
    ruling!(
        "Expansion Algorithm",
        "While proliferating multiple times, players can't respond between proliferating the first time and proliferating the second time, and so on."
    );
    supported("Expansion Algorithm");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, a, counters::PLUS1, 1);
    let ea = t.hand(P0, "Expansion Algorithm");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 2);
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: ea,
            method: CastMethod::Normal,
        }),
    );
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.answer_choose(P0, &[Entity::Object(a)]);
    // Through the priority loop until Expansion Algorithm has resolved.
    assert!(t.g.run_until(1000, |g| g.stack.is_empty()
        && g.players[0]
            .graveyard
            .iter()
            .any(|c| g.obj(*c).chars.name == "Expansion Algorithm")));
    assert_eq!(t.counters(a, counters::PLUS1), 3);
    let choices = proliferate_choices(&t);
    assert_eq!(choices.len(), 2);
    let asked = t.asked();
    // Both players had priority with the spell on the stack ...
    let before: Vec<PlayerId> = asked[..choices[0]]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Priority { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert!(before.contains(&P1));
    // ... but no one was asked anything between the two proliferate actions.
    assert_eq!(choices[1], choices[0] + 1);
}

#[test]
fn proliferate_twice_compiles_to_two_proliferate_actions() {
    // "proliferate twice" and "proliferate X times" (the new pattern) on the cards that
    // use them.
    cr!("701.34a");
    for c in [
        "Agent Frank Horrigan",
        "Contagion Engine",
        "Ezuri, Stalker of Spheres",
        "Expansion Algorithm",
        "Roalesk, Apex Hybrid",
    ] {
        supported(c);
    }
    // Contagion Engine: "{4}, {T}: Proliferate twice."
    let mut t = TestGame::new(2);
    let engine = t.battlefield(P0, "Contagion Engine");
    t.lands(P0, "Wastes", 4);
    let bear = t.battlefield(P1, "Grizzly Bears");
    add(&mut t, bear, counters::MINUS1, 1);
    t.answer_choose(P0, &[Entity::Object(bear)]);
    t.answer_choose(P0, &[Entity::Object(bear)]);
    t.activate(P0, engine, 0, &[]).expect("Contagion Engine");
    t.resolve_all();
    assert_eq!(proliferate_choices(&t).len(), 2);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}
