//! Rulings batch P124 — phasing (CR 702.26): "for as long as" durations can't see a
//! phased-out permanent and end (CR 702.26f, 611.2b), "until this leaves the battlefield"
//! effects don't end when it phases out (CR 610.3, 702.26d), no leaves- or enters-the-
//! battlefield abilities trigger, tokens phase back in, and entry choices are kept.

use crate::r_p124_common::*;
use crate::r_s11_common::triggered_from;
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::Color;
use mtg_engine::*;

/// The number of triggered abilities the object (followed across zone changes) has now.
fn triggered_abilities(t: &mut TestGame, id: ObjectId) -> usize {
    t.g.recompute();
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Triggered(_)))
        .count()
}

/// P1's Mathas, Fiend Seeker ("At the beginning of your end step, put a bounty counter on
/// target creature an opponent controls. For as long as that creature has a bounty
/// counter on it, it has 'When this creature dies, each opponent draws a card and gains 2
/// life.'") puts a bounty counter on `target` in P1's end step; then it's P0's main
/// phase, and `target` has the bounty ability.
fn bounty_on(t: &mut TestGame, target: ObjectId) {
    supported("Mathas, Fiend Seeker");
    t.battlefield(P1, "Mathas, Fiend Seeker");
    let before = triggered_abilities(t, target);
    t.set_step(P1, Step::PostcombatMain);
    t.answer_targets(P1, &[obj(target)]);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(target, "bounty"), 1);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(triggered_abilities(t, target), before + 1);
}

#[test]
fn for_as_long_as_durations_end_when_the_permanent_phases_out() {
    cr!("702.26f", "611.2b", "702.26d", "702.26a");
    ruling!(
        "Teferi's Protection",
        "Any continuous effects with a \"for as long as\" duration such as that of Mathas, Fiend Seeker ignore phased-out objects."
    );
    ruling!(
        "Perch Protection",
        "Any continuous effects with a \"for as long as\" duration such as that of Mathas, Fiend Seeker ignore phased-out objects."
    );
    ruling!(
        "Clever Concealment",
        "Any continuous effects with a \"for as long as\" duration, such as that of Mind Flayer, ignore phased-out objects."
    );
    ruling!(
        "Guardian of Faith",
        "Any continuous effects with a \"for as long as\" duration, such as that of Mind Flayer, ignore phased-out objects."
    );
    ruling!(
        "Blink Dog",
        "Any continuous effects with a \"for as long as\" duration, such as that of Mind Flayer, ignore phased-out objects."
    );
    ruling!(
        "Renegade Silent",
        "Any continuous effects with a \"for as long as\" duration, such as that of Mind Flayer, ignore phased-out objects."
    );
    // Each way of phasing out P0's creature (Grizzly Bears, or the card itself when it
    // phases itself out) with the bounty: it phases back in with the counter but without
    // the ability.
    type Phase = fn(&mut TestGame, ObjectId);
    let ways: [(&str, &str, Phase); 6] = [
        ("Teferi's Protection", "Grizzly Bears", |t, _| {
            cast_resolve(t, P0, "Teferi's Protection", &[])
        }),
        ("Perch Protection", "Grizzly Bears", |t, _| {
            t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
            cast_resolve(t, P0, "Perch Protection", &[])
        }),
        ("Clever Concealment", "Grizzly Bears", |t, id| {
            cast_resolve(t, P0, "Clever Concealment", &[obj(id)])
        }),
        ("Guardian of Faith", "Grizzly Bears", |t, id| {
            t.answer_targets(P0, &[obj(id)]);
            cast_resolve(t, P0, "Guardian of Faith", &[])
        }),
        ("Blink Dog", "Blink Dog", |t, id| {
            t.lands(P0, "Plains", 4);
            t.activate(P0, id, 0, &[]).unwrap();
            t.resolve_all();
        }),
        ("Renegade Silent", "Renegade Silent", |t, _| {
            t.answer_targets(P0, &[]);
            t.advance_to(P0, Step::End);
            t.resolve_all();
        }),
    ];
    for (card, creature, phase) in ways {
        supported(card);
        let mut t = TestGame::new(2);
        let id = t.battlefield(P0, creature);
        bounty_on(&mut t, id);
        let before = triggered_abilities(&mut t, id);
        phase(&mut t, id);
        assert!(phased_out(&t, id), "{card}");
        to_next_upkeep(&mut t);
        assert!(!phased_out(&t, id), "{card}");
        assert_eq!(t.counters(id, "bounty"), 1, "{card}: the counter stays");
        assert_eq!(
            triggered_abilities(&mut t, id),
            before - 1,
            "{card}: the bounty ability ended"
        );
    }
}

#[test]
fn for_as_long_as_you_control_ends_when_the_source_phases_out() {
    cr!("702.26f", "611.2b", "702.26e");
    // Mind Flayer: "When Mind Flayer enters, gain control of target creature for as long
    // as you control Mind Flayer." P0's Mind Flayer phases out (Clever Concealment): P1
    // gets the Bears back for good.
    supported("Mind Flayer");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let flayer = t.enter(P0, "Mind Flayer");
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    cast_resolve(&mut t, P0, "Clever Concealment", &[obj(flayer)]);
    assert!(phased_out(&t, flayer));
    t.g.recompute();
    assert_eq!(t.obj_now(bears).controller, P1);
    to_next_upkeep(&mut t);
    assert!(!phased_out(&t, flayer));
    t.g.recompute();
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn until_this_leaves_the_battlefield_doesnt_end_when_it_phases_out() {
    cr!("610.3", "702.26d");
    ruling!(
        "Teferi's Protection",
        "Any one-shot effects that are waiting \"until [this] leaves the battlefield,\" such as that of Banishing Light, won't happen when a permanent phases out."
    );
    ruling!(
        "Perch Protection",
        "Any one-shot effects that are waiting \"until [this] leaves the battlefield,\" such as that of Banishing Light, won't happen when a permanent phases out."
    );
    for card in ["Teferi's Protection", "Perch Protection"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        let light = t.enter(P0, "Banishing Light");
        t.resolve_all();
        assert_eq!(t.zone(bears), Zone::Exile);
        if card == "Perch Protection" {
            t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
        }
        cast_resolve(&mut t, P0, card, &[]);
        assert!(phased_out(&t, light));
        assert_eq!(t.zone(bears), Zone::Exile, "{card}");
        to_next_upkeep(&mut t);
        assert!(!phased_out(&t, light));
        assert_eq!(t.zone(bears), Zone::Exile, "{card}");
        // Once it really leaves, the Bears return.
        destroy(&mut t, light);
        assert_eq!(t.zone(bears), Zone::Battlefield, "{card}");
    }
}

#[test]
fn phasing_doesnt_trigger_leaves_or_enters_abilities() {
    cr!("702.26d", "603.6a", "603.6c");
    ruling!(
        "Clever Concealment",
        "Phasing out doesn't cause permanents to leave or re-enter the battlefield, so no \"leaves the battlefield\" or \"enters the battlefield\" abilities will trigger."
    );
    ruling!(
        "March of Swirling Mist",
        "Phasing out doesn't cause permanents to leave or re-enter the battlefield, so no “leaves the battlefield” or “enters the battlefield” abilities will trigger."
    );
    supported("Fiend Hunter");
    // Fiend Hunter: "When this creature enters, you may exile another target creature.
    // When this creature leaves the battlefield, return the exiled card to the
    // battlefield under its owner's control." P1's Soul Warden: "Whenever another creature
    // enters, you gain 1 life."
    for card in ["Clever Concealment", "March of Swirling Mist"] {
        supported(card);
        let mut t = TestGame::new(2);
        let warden = t.battlefield(P1, "Soul Warden");
        let giant = t.battlefield(P1, "Hill Giant");
        t.answer_targets(P0, &[obj(giant)]);
        t.answer_yes(P0, true);
        let hunter = t.enter(P0, "Fiend Hunter");
        t.resolve_all();
        assert_eq!(t.zone(giant), Zone::Exile);
        let life = t.life(P1);
        if card == "March of Swirling Mist" {
            t.lands(P0, "Island", 2);
            let march = t.hand(P0, card);
            t.cast(P0, march).x(1).target(hunter).go();
            t.resolve_all();
        } else {
            cast_resolve(&mut t, P0, card, &[obj(hunter)]);
        }
        assert!(phased_out(&t, hunter), "{card}");
        assert_eq!(t.zone(giant), Zone::Exile, "{card}");
        to_next_upkeep(&mut t);
        assert!(!phased_out(&t, hunter), "{card}");
        assert_eq!(t.stack_len(), 0, "{card}");
        assert_eq!(triggered_from(&t, warden), 0, "{card}");
        assert_eq!(t.life(P1), life, "{card}");
        assert_eq!(t.zone(giant), Zone::Exile, "{card}");
    }
}

#[test]
fn perch_protection_tokens_and_entry_choices_phase_back_in() {
    cr!("702.26d", "702.26a", "607.2a", "111.1");
    ruling!(
        "Perch Protection",
        "If a token is phased out, it will phase in as your next untap step begins."
    );
    ruling!(
        "Perch Protection",
        "Choices made for permanents as they entered are remembered when they phase in."
    );
    supported("Chameleon Spirit");
    // Chameleon Spirit: "As this creature enters, choose a color. Chameleon Spirit's power
    // and toughness are each equal to the number of permanents of the chosen color your
    // opponents control."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    let green = Color::ALL.iter().position(|c| *c == Color::Green).unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(green));
    let spirit = t.enter(P0, "Chameleon Spirit");
    t.settle();
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    cast_resolve(&mut t, P0, "Perch Protection", &[]);
    // (Phased-out permanents aren't among the permanents P0 controls: look at the zone.)
    let birds: Vec<ObjectId> = t
        .g
        .battlefield
        .iter()
        .copied()
        .filter(|o| t.g.obj(*o).is_token())
        .collect();
    assert_eq!(birds.len(), 4);
    assert!(birds.iter().all(|b| phased_out(&t, *b)));
    assert!(phased_out(&t, spirit));
    // P1 takes the extra turn (and its next turn), then P0's untap step phases them in.
    to_next_upkeep(&mut t);
    assert!(birds.iter().all(|b| t.on_battlefield(*b) && !phased_out(&t, *b)));
    assert!(!phased_out(&t, spirit));
    assert_eq!(t.obj_now(spirit).choices.color, Some(Color::Green));
    t.g.recompute();
    assert_eq!(t.pt(spirit), (2, 2));
}

#[test]
fn for_as_long_as_it_has_a_counter_ends_when_it_phases_out() {
    cr!("702.26f", "611.2b", "702.26d");
    // Aquitect's Will: "Put a flood counter on target land. That land is an Island in
    // addition to its other types for as long as it has a flood counter on it." P0's
    // Forest phases out with Teferi's Protection: it phases in with the counter, but the
    // effect couldn't see it and ended.
    supported("Aquitect's Will");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    cast_resolve(&mut t, P0, "Aquitect's Will", &[obj(forest)]);
    t.g.recompute();
    assert!(t.obj_now(forest).chars.has_subtype("Island"));
    cast_resolve(&mut t, P0, "Teferi's Protection", &[]);
    assert!(phased_out(&t, forest));
    to_next_upkeep(&mut t);
    assert!(!phased_out(&t, forest));
    assert_eq!(t.counters(forest, "flood"), 1);
    t.g.recompute();
    assert!(!t.obj_now(forest).chars.has_subtype("Island"));
}
