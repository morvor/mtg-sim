//! Rulings batch P197 — characteristic-changing effects with no duration (CR 611.2a):
//! they outlast the turn and their source and end only when the object leaves (400.7) or a
//! later effect overrides them (613.7); subtypes that need a card type (205.3d); text
//! changes (612); colors (105); and the cards' own conditions.

use crate::r_p197_common::*;
use crate::r_s06_common::activate_containing;
use mtg_engine::ability::Modification;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::{Event, MoveCause};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{CardType, Color, Supertype};
use mtg_engine::*;

fn two_turns(t: &mut TestGame) {
    pass_turn(t);
    pass_turn(t);
}

fn is_creature(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).is(CardType::Creature)
}

fn has_kw(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(k)
}

/// Moves the permanent to its owner's hand and back onto the battlefield: a new object.
fn flicker(t: &mut TestGame, id: ObjectId) -> ObjectId {
    let owner = t.obj_now(id).owner;
    let name = t.obj_now(id).chars.name.clone();
    bounce(t, id);
    let card = t.g.find_in_zone(Zone::Hand(owner), &name)[0];
    t.g.move_object(card, Zone::Battlefield, MoveCause::Effect, None);
    t.g.recompute();
    t.g.current(card)
}

/// P0's Smuggler's Copter crewed by a Grizzly Bears: an artifact creature until end of
/// turn.
fn crewed_copter(t: &mut TestGame) -> ObjectId {
    let copter = t
        .named_on_battlefield("Smuggler's Copter")
        .first()
        .copied()
        .unwrap_or_else(|| t.battlefield(P0, "Smuggler's Copter"));
    let crew = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(crew)]);
    activate_containing(t, P0, copter, "Crew").expect("crew");
    t.resolve_all();
    assert!(is_creature(t, copter));
    copter
}

#[test]
fn opal_titan_gets_no_protection_from_a_colorless_spell() {
    cr!("105.2c", "702.16a", "603.4");
    ruling!(
        "Opal Titan",
        "If the spell was colorless (an artifact), this card does not get any form of Protection."
    );
    supported("Opal Titan");
    for (spell, color) in [("Ornithopter", None), ("Grizzly Bears", Some(Color::Green))] {
        let mut t = TestGame::new(2);
        let titan = t.battlefield(P0, "Opal Titan");
        let green = t.battlefield(P0, "Llanowar Elves");
        let gray = t.battlefield(P0, "Memnite");
        t.set_step(P1, Step::PrecombatMain);
        lots(&mut t, P1);
        let c = t.hand(P1, spell);
        t.cast(P1, c).go();
        t.settle();
        t.resolve_all();
        assert!(is_creature(&t, titan), "{spell}");
        assert_eq!(t.pt(titan), (4, 4));
        assert!(t.obj_now(titan).chars.has_subtype("Giant"));
        assert_eq!(t.g.protected_from(titan, green), color.is_some(), "{spell}");
        assert!(!t.g.protected_from(titan, gray), "{spell}");
    }
}

#[test]
fn an_added_creature_subtype_disappears_while_the_permanent_isnt_a_creature() {
    cr!("205.3d", "611.2a", "613.1d");
    ruling!(
        "Archangel Elspeth",
        "If the target of the second ability stops being a creature after the ability has resolved, it will also stop being an Angel until it becomes a creature again, if applicable."
    );
    ruling!(
        "Breach the Multiverse",
        "The creatures you control will be Phyrexians until they leave the battlefield. Any of them that stops being a creature will also stop being a Phyrexian until it becomes a creature again, if applicable."
    );
    supported("Smuggler's Copter");
    for card in ["Archangel Elspeth", "Breach the Multiverse"] {
        supported(card);
        let sub = if card == "Breach the Multiverse" {
            "Phyrexian"
        } else {
            "Angel"
        };
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let copter = crewed_copter(&mut t);
        if card == "Archangel Elspeth" {
            let el = walker(&mut t, P0, card, 4);
            act(&mut t, P0, el, 1, &[obj(copter)]);
        } else {
            cast_at(&mut t, P0, card, &[]);
            t.resolve_all();
        }
        assert!(t.obj_now(copter).chars.has_subtype(sub), "{card}");
        pass_turn(&mut t);
        assert!(!is_creature(&t, copter));
        assert!(!t.obj_now(copter).chars.has_subtype(sub), "{card}");
        // Crewed again on P0's next turn: a creature, and an Angel / Phyrexian again.
        t.advance_to(P0, Step::PrecombatMain);
        crewed_copter(&mut t);
        assert!(t.obj_now(copter).chars.has_subtype(sub), "{card}");
        // Leaving the battlefield ends it.
        let again = flicker(&mut t, copter);
        crewed_copter(&mut t);
        assert!(!t.obj_now(again).chars.has_subtype(sub), "{card}");
    }
}

#[test]
fn tezzerets_minus_two_keeps_a_vehicles_printed_power_and_toughness() {
    cr!("301.7", "613.4b", "611.2a");
    ruling!(
        "Tezzeret, Betrayer of Flesh",
        "If you activate Tezzeret's second loyalty ability targeting a Vehicle, it will have its printed power and toughness."
    );
    supported("Tezzeret, Betrayer of Flesh");
    let mut t = TestGame::new(2);
    let tez = walker(&mut t, P0, "Tezzeret, Betrayer of Flesh", 5);
    let copter = t.battlefield(P0, "Smuggler's Copter");
    act(&mut t, P0, tez, 1, &[obj(copter)]);
    assert!(is_creature(&t, copter));
    assert_eq!(t.pt(copter), (3, 3));
    two_turns(&mut t);
    assert!(is_creature(&t, copter));
    assert_eq!(t.pt(copter), (3, 3));
    // A non-Vehicle artifact is 4/4.
    let mut t = TestGame::new(2);
    let tez = walker(&mut t, P0, "Tezzeret, Betrayer of Flesh", 5);
    let ring = t.battlefield(P0, "Sol Ring");
    act(&mut t, P0, tez, 1, &[obj(ring)]);
    assert_eq!(t.pt(ring), (4, 4));
}

#[test]
fn trench_gorger_stays_six_six_if_you_dont_search() {
    cr!("608.2c", "603.5");
    ruling!(
        "Trench Gorger",
        "If you choose not to search, Trench Gorger will simply be 6/6."
    );
    supported("Trench Gorger");
    let mut t = TestGame::new(2);
    crate::r_s01_common::stack_library(&mut t, P0, &["Island", "Island"]);
    let lib = t.library_size(P0);
    t.answer_yes(P0, false);
    let g = t.enter(P0, "Trench Gorger");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.pt(g), (6, 6));
    assert_eq!(t.library_size(P0), lib);
}

#[test]
fn olivia_mobilized_haste_ends_the_counter_and_type_remain() {
    cr!("611.2a", "514.2");
    ruling!(
        "Olivia, Mobilized for War",
        "If you discard a card, the creature only gains haste until end of turn, but the +1/+1 counter and Vampire subtype remain indefinitely."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Olivia, Mobilized for War");
    let discard = t.hand(P0, "Island");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(discard)]);
    let bears = t.enter(P0, "Grizzly Bears");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Island"));
    assert!(has_kw(&t, bears, KeywordKind::Haste));
    two_turns(&mut t);
    assert!(!has_kw(&t, bears, KeywordKind::Haste));
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert!(t.obj_now(bears).chars.has_subtype("Vampire"));
}

/// Ashiok exiles `cards` (top of P1's library) with its +2, then (two turns later) puts
/// the one named `pick` onto the battlefield with −X.
fn ashiok_puts(t: &mut TestGame, cards: &[&str], pick: &str, x: i64) -> ObjectId {
    supported("Ashiok, Nightmare Weaver");
    let a = walker(t, P0, "Ashiok, Nightmare Weaver", 5);
    let lib = crate::r_s01_common::stack_library(t, P1, cards);
    t.set_step(P0, Step::PrecombatMain);
    act(t, P0, a, 0, &[pl(P1)]);
    let i = cards.iter().position(|c| *c == pick).unwrap();
    let card = t.g.current(lib[i]);
    assert_eq!(t.zone(card), Zone::Exile);
    two_turns(t);
    t.set_step(P0, Step::PrecombatMain);
    t.answer(P0, DecisionKind::X, Answer::Number(x));
    t.answer_choose(P0, &[obj(card)]);
    act(t, P0, a, 1, &[]);
    t.g.current(card)
}

#[test]
fn ashiok_puts_a_bestow_card_in_as_a_creature_and_x_is_zero() {
    cr!("702.103f", "107.3m", "110.2");
    ruling!(
        "Ashiok, Nightmare Weaver",
        "If you put a creature card with bestow onto the battlefield, it will be a creature on the battlefield, not an Aura."
    );
    ruling!(
        "Ashiok, Nightmare Weaver",
        "If you put a creature card with {X} in its mana cost onto the battlefield, the value of that X is 0."
    );
    let mut t = TestGame::new(2);
    let satyr = ashiok_puts(&mut t, &["Boon Satyr", "Island", "Island"], "Boon Satyr", 3);
    assert!(t.on_battlefield(satyr));
    assert_eq!(ctl(&t, satyr), P0);
    let o = t.obj_now(satyr);
    assert!(o.is(CardType::Creature));
    assert!(!o.chars.has_subtype("Aura"));
    assert!(o.attached_to.is_none());
    assert!(o.chars.has_subtype("Nightmare"));
    assert_eq!(t.pt(satyr), (4, 2));
    // Endless One (mana value 0 in exile): enters with no counters and dies as a 0/0.
    let mut t = TestGame::new(2);
    let one = ashiok_puts(
        &mut t,
        &["Endless One", "Island", "Island"],
        "Endless One",
        0,
    );
    let entered = t.g.turn_events.iter().any(|e| {
        matches!(e, Event::ZoneChange { to: Zone::Battlefield, new, .. } if t.g.current(*new) == one)
    });
    assert!(entered);
    assert!(t.in_graveyard(P1, "Endless One"));
    assert!(t
        .g
        .turn_events
        .iter()
        .all(|e| !matches!(e, Event::CountersAdded { .. })));
}

#[test]
fn ageless_sentinels_becomes_a_bird_giant() {
    cr!("205.1b", "611.2a", "509.1");
    ruling!(
        "Ageless Sentinels",
        "It becomes both creature type Giant and creature type Bird."
    );
    supported("Ageless Sentinels");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Ageless Sentinels");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, Step::BeginningOfCombat);
    t.attack(&[(bears, pl(P0))], &[(wall, bears)]);
    t.resolve_all();
    assert_eq!(creature_types(&t, wall), vec!["Bird", "Giant"]);
    assert!(!has_kw(&t, wall, KeywordKind::Defender));
    two_turns(&mut t);
    assert_eq!(creature_types(&t, wall), vec!["Bird", "Giant"]);
}

#[test]
fn kenku_artificer_keeps_the_artifacts_abilities() {
    cr!("613.1d", "613.1f");
    ruling!(
        "Kenku Artificer",
        "Kenku Artificer doesn't remove any abilities the target artifact has."
    );
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Sol Ring");
    t.answer_targets(P0, &[obj(ring)]);
    t.enter(P0, "Kenku Artificer");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert!(is_creature(&t, ring));
    assert_eq!(t.pt(ring), (3, 3));
    assert!(has_kw(&t, ring, KeywordKind::Flying));
    assert!(has_ability(&t, ring, "{C}{C}"));
}

#[test]
fn reanimated_zombies_keep_their_colors_and_types() {
    cr!("611.2a", "613.1d", "613.1e", "105.3");
    ruling!(
        "Necromantic Selection",
        "Necromantic Selection doesn't overwrite any of the creature's colors or types. Rather, it adds another color and another creature type."
    );
    ruling!(
        "Rise from the Grave",
        "Rise from the Grave doesn’t overwrite any previous colors or types. Rather, it adds another color and another subtype."
    );
    supported("Necromantic Selection");
    supported("Rise from the Grave");
    for card in ["Necromantic Selection", "Rise from the Grave"] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let elves = if card == "Rise from the Grave" {
            let e = t.graveyard(P1, "Llanowar Elves");
            cast_at(&mut t, P0, card, &[obj(e)]);
            e
        } else {
            let e = t.battlefield(P1, "Llanowar Elves");
            t.answer_choose_moved(P0, &[obj(e)]);
            cast_at(&mut t, P0, card, &[]);
            e
        };
        t.resolve_all();
        let e = t.g.current(elves);
        assert!(t.on_battlefield(e), "{card}");
        assert_eq!(ctl(&t, e), P0);
        let c = &t.obj_now(e).chars;
        assert!(
            c.colors.contains(Color::Black) && c.colors.contains(Color::Green),
            "{card}"
        );
        assert_eq!(
            creature_types(&t, e),
            vec!["Druid", "Elf", "Zombie"],
            "{card}"
        );
        two_turns(&mut t);
        assert_eq!(
            creature_types(&t, e),
            vec!["Druid", "Elf", "Zombie"],
            "{card}"
        );
    }
}

#[test]
fn new_blood_changes_types_in_the_type_line_and_text_but_not_the_name() {
    cr!("612.2", "201.4b");
    ruling!(
        "New Blood",
        "New Blood changes the creature type each time it appears in the creature's type line and/or rules text. It doesn't change the name of the card or any instances of the word being used as part of a card's name."
    );
    supported("Goblin King");
    let mut t = TestGame::new(2);
    let king = t.battlefield(P1, "Goblin King");
    let gob = vanilla(&mut t, P0, "Test Goblin", "Creature — Goblin", (1, 1));
    let vamp = t.battlefield(P0, "Vampire Interloper");
    let v2 = vanilla(&mut t, P0, "Test Vampire", "Creature — Vampire", (1, 1));
    assert_eq!(t.pt(gob), (2, 2));
    choose_words(&mut t, P0, &["Goblin", "Vampire"]);
    cast_at(&mut t, P0, "New Blood", &[obj(king)]);
    t.resolve_all();
    assert!(t.obj_now(vamp).tapped);
    assert_eq!(ctl(&t, king), P0);
    assert_eq!(t.obj_now(king).chars.name, "Goblin King");
    assert!(t.obj_now(king).chars.has_subtype("Vampire"));
    assert!(!t.obj_now(king).chars.has_subtype("Goblin"));
    // "Other Vampires get +1/+1": the Goblin no longer gets it, the Vampires do.
    assert_eq!(t.pt(gob), (1, 1));
    assert_eq!(t.pt(v2), (2, 2));
}

#[test]
fn new_blood_changes_only_printed_text_not_granted_abilities() {
    cr!("612.2", "612.1");
    ruling!(
        "New Blood",
        "New Blood's effect changes only the text printed on the creature. It can't change words found in abilities it's been granted. For example, if you change Rat to Vampire on Hungry Lynx, your Cats will have protection from Vampires; but if instead you change Rat to Vampire on Phantom Nishoba while you control a Hungry Lynx, Phantom Nishoba will have protection from Rats."
    );
    supported("Hungry Lynx");
    supported("Phantom Nishoba");
    // Rat -> Vampire on Hungry Lynx: P0's Cats have protection from Vampires.
    let mut t = TestGame::new(2);
    let lynx = t.battlefield(P1, "Hungry Lynx");
    t.battlefield(P0, "Vampire Interloper");
    let cat = vanilla(&mut t, P0, "Test Cat", "Creature — Cat", (1, 1));
    let rat = vanilla(&mut t, P1, "Test Rat", "Creature — Rat", (1, 1));
    let vamp = vanilla(&mut t, P1, "Test Vampire", "Creature — Vampire", (1, 1));
    choose_words(&mut t, P0, &["Rat", "Vampire"]);
    cast_at(&mut t, P0, "New Blood", &[obj(lynx)]);
    t.resolve_all();
    assert_eq!(ctl(&t, lynx), P0);
    assert!(t.g.protected_from(cat, vamp));
    assert!(!t.g.protected_from(cat, rat));
    // Rat -> Vampire on Phantom Nishoba while P0 controls a Hungry Lynx: the granted
    // ability isn't changed — protection from Rats.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hungry Lynx");
    t.battlefield(P0, "Vampire Interloper");
    let nish = t.battlefield(P1, "Phantom Nishoba");
    t.g.add_counters(obj(nish), "+1/+1", 7, None);
    let rat = vanilla(&mut t, P1, "Test Rat", "Creature — Rat", (1, 1));
    let vamp = vanilla(&mut t, P1, "Test Vampire", "Creature — Vampire", (1, 1));
    choose_words(&mut t, P0, &["Rat", "Vampire"]);
    cast_at(&mut t, P0, "New Blood", &[obj(nish)]);
    t.resolve_all();
    assert_eq!(ctl(&t, nish), P0);
    assert!(t.g.protected_from(nish, rat));
    assert!(!t.g.protected_from(nish, vamp));
}

#[test]
fn leveler_steps_have_no_duration() {
    cr!("611.2a", "613.4b", "400.7");
    ruling!(
        "Ascendant Spirit",
        "None of Ascendant Spirit's abilities has a duration. If one of them resolves, it will remain in effect until Ascendant Spirit leaves the battlefield or a subsequent effect changes its characteristics."
    );
    ruling!(
        "Evolved Sleeper",
        "None of these abilities has a duration. If one of them resolves, it will remain in effect until the game ends, Evolved Sleeper leaves the battlefield, or some subsequent effect changes its characteristics, whichever comes first."
    );
    ruling!(
        "Figure of Destiny",
        "None of these abilities has a duration. If one of them resolves, it will remain in effect until the game ends, Figure of Destiny leaves the battlefield, or some subsequent effect changes its characteristics, whichever comes first."
    );
    ruling!(
        "Figure of Fable",
        "None of these abilities have durations. If one of them resolves, it will remain in effect until the game ends, Figure of Fable leaves the battlefield, or some subsequent effect changes its characteristics, whichever comes first."
    );
    supported("Turn to Frog");
    for (name, types, pt) in [
        ("Ascendant Spirit", vec!["Spirit", "Warrior"], (2, 3)),
        ("Evolved Sleeper", vec!["Cleric", "Human"], (2, 2)),
        ("Figure of Destiny", vec!["Kithkin", "Spirit"], (2, 2)),
        ("Figure of Fable", vec!["Kithkin", "Scout"], (2, 3)),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        t.lands(P0, "Snow-Covered Swamp", 12);
        let c = t.battlefield(P0, name);
        act(&mut t, P0, c, 0, &[]);
        two_turns(&mut t);
        assert_eq!(t.pt(c), pt, "{name}");
        assert_eq!(creature_types(&t, c), types, "{name}");
        // A subsequent effect changes it (until end of turn here).
        cast_at(&mut t, P1, "Turn to Frog", &[obj(c)]);
        t.resolve_all();
        assert_eq!(t.pt(c), (1, 1), "{name}");
        pass_turn(&mut t);
        assert_eq!(t.pt(c), pt, "{name}");
        // Leaving the battlefield ends it.
        let again = flicker(&mut t, c);
        assert_eq!(t.pt(again), (1, 1), "{name}");
    }
}

#[test]
fn oko_keeps_a_temporary_creature_an_elk_and_overwrites_earlier_setting_effects() {
    cr!("611.2a", "613.4b", "613.7", "205.1a");
    ruling!(
        "Oko, Thief of Crowns",
        "Oko's second ability may target a permanent that is only temporarily an artifact or a creature, such as Oko, the Trickster. If this happens, the effect causes that permanent to remain a green Elk creature even after the temporary effect expires."
    );
    ruling!(
        "Oko, Thief of Crowns",
        "Oko's second ability overwrites all previous effects that set the creature's base power and toughness to specific values. Any power- or toughness-setting effects that start to apply after Oko's second ability resolves will overwrite this effect."
    );
    supported("Mutavault");
    // An animated Mutavault: still a green Elk creature after its own effect ends.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let vault = t.battlefield(P1, "Mutavault");
    lots(&mut t, P1);
    t.activate(P1, vault, 1, &[]).expect("animate");
    t.resolve_all();
    assert!(is_creature(&t, vault));
    let oko = walker(&mut t, P0, "Oko, Thief of Crowns", 3);
    act(&mut t, P0, oko, 1, &[obj(vault)]);
    two_turns(&mut t);
    // A creature only: Oko's effect overwrote its other card types.
    let o = t.obj_now(vault);
    assert!(o.is(CardType::Creature) && !o.is(CardType::Land));
    assert!(o.chars.has_subtype("Elk"));
    assert!(o.chars.colors.contains(Color::Green));
    assert_eq!(t.pt(vault), (3, 3));
    // A Figure of Destiny that's 4/4 (an earlier setting effect) becomes 3/3; a later
    // setting effect (Turn to Frog, 1/1) overwrites Oko's; counters still apply.
    let mut t = TestGame::new(2);
    let fig = t.battlefield(P1, "Figure of Destiny");
    act(&mut t, P1, fig, 0, &[]);
    act(&mut t, P1, fig, 1, &[]);
    assert_eq!(t.pt(fig), (4, 4));
    let oko = walker(&mut t, P0, "Oko, Thief of Crowns", 3);
    t.set_step(P0, Step::PrecombatMain);
    act(&mut t, P0, oko, 1, &[obj(fig)]);
    assert_eq!(t.pt(fig), (3, 3));
    t.g.add_counters(obj(fig), "+1/+1", 1, None);
    t.g.recompute();
    assert_eq!(t.pt(fig), (4, 4));
    cast_at(&mut t, P0, "Turn to Frog", &[obj(fig)]);
    t.resolve_all();
    assert_eq!(t.pt(fig), (2, 2));
}

#[test]
fn gornog_a_blocker_turned_coward_stays_blocking() {
    cr!("506.4", "509.1b");
    ruling!(
        "Gornog, the Red Reaper",
        "Once a creature has blocked a Warrior, turning that creature into a Coward won't remove it from combat or cause the Warrior to become unblocked. (The Coward might suddenly have some regrets, though.)"
    );
    supported("Gornog, the Red Reaper");
    let mut t = TestGame::new(2);
    let gornog = t.battlefield(P0, "Gornog, the Red Reaper");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    // The trigger makes the Giant a Coward; the Bears block.
    t.answer_targets(P0, &[obj(giant)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(gornog, pl(P1))]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(bears, gornog)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.resolve_all();
    assert!(crate::r_s10_common::blocking(&t, bears));
    // Now the Bears become a Coward too.
    crate::r_s26_common::modify_until_eot(
        &mut t,
        bears,
        vec![Modification::AddSubtypes(vec!["Coward".into()])],
    );
    assert!(t.obj_now(bears).chars.has_subtype("Coward"));
    assert!(crate::r_s10_common::blocking(&t, bears));
    // Gornog (2/3, +2/+0 for two Cowards) kills the Bears and isn't unblocked.
    assert_eq!(t.pt(gornog), (4, 3));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(bears));
}

#[test]
fn gornogs_bonus_follows_the_number_of_cowards() {
    cr!("611.3a", "613.4c");
    ruling!(
        "Gornog, the Red Reaper",
        "The bonus granted by Gornog's last ability changes as the number of Cowards your opponents control changes."
    );
    let mut t = TestGame::new(2);
    let gornog = t.battlefield(P0, "Gornog, the Red Reaper");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(gornog, pl(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(t.pt(gornog), (3, 3));
    crate::r_s26_common::modify_until_eot(
        &mut t,
        bears,
        vec![Modification::AddSubtypes(vec!["Coward".into()])],
    );
    assert_eq!(t.pt(gornog), (4, 3));
    kill(&mut t, giant);
    assert_eq!(t.pt(gornog), (3, 3));
}

#[test]
fn effects_that_animate_or_change_a_permanent_last_indefinitely() {
    cr!("611.2a", "611.2b", "514.2");
    ruling!(
        "Siege of Towers",
        "The Mountain will remain a creature for the rest of the game. It doesn't have a creature type."
    );
    ruling!(
        "Ride the Shoopuf",
        "Ride the Shoopuf's last ability doesn't have a duration. Once it resolves, it will remain in effect until the game ends, Ride the Shoopuf's leaves the battlefield, or some subsequent effect changes its characteristics, whichever comes first."
    );
    ruling!(
        "Soulstone Sanctuary",
        "Soulstone Sanctuary's last ability doesn't have a duration. Once it resolves, it will remain in effect until the game ends, Soulstone Sanctuary leaves the battlefield, or some subsequent effect changes its characteristics, whichever comes first."
    );
    ruling!("Moonlace", "Moonlace's effect doesn't wear off.");
    ruling!(
        "Lizard, Connors's Curse",
        "The effect from Lizard's triggered ability doesn't end at the end of the turn."
    );
    ruling!(
        "The Irencrag",
        "The Irencrag's last ability lasts indefinitely."
    );
    ruling!(
        "Keiga, the Tide Star",
        "Keiga's effect lasts indefinitely. It doesn't wear off during the cleanup step."
    );
    for card in [
        "Siege of Towers",
        "Ride the Shoopuf",
        "Soulstone Sanctuary",
        "Moonlace",
        "Lizard, Connors's Curse",
        "The Irencrag",
        "Keiga, the Tide Star",
    ] {
        supported(card);
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        // The permanent the effect changes, and a check of the change.
        let (id, check): (ObjectId, Box<dyn Fn(&TestGame, ObjectId)>) = match card {
            "Siege of Towers" => {
                let m = t.battlefield(P1, "Mountain");
                cast_at(&mut t, P0, card, &[obj(m)]);
                (
                    m,
                    Box::new(|t, m| {
                        assert!(is_creature(t, m));
                        assert_eq!(t.pt(m), (3, 1));
                        assert!(creature_types(t, m).is_empty());
                    }),
                )
            }
            "Ride the Shoopuf" | "Soulstone Sanctuary" => {
                let s = t.battlefield(P0, card);
                let i = if card == "Ride the Shoopuf" { 0 } else { 1 };
                act(&mut t, P0, s, i, &[]);
                let pt = if card == "Ride the Shoopuf" {
                    (7, 7)
                } else {
                    (3, 3)
                };
                (
                    s,
                    Box::new(move |t, s| {
                        assert!(is_creature(t, s));
                        assert_eq!(t.pt(s), pt);
                    }),
                )
            }
            "Moonlace" => {
                let b = t.battlefield(P1, "Grizzly Bears");
                cast_at(&mut t, P0, card, &[obj(b)]);
                (
                    b,
                    Box::new(|t, b| assert!(t.obj_now(b).chars.colors.is_colorless())),
                )
            }
            "Lizard, Connors's Curse" => {
                let a = t.battlefield(P1, "Serra Angel");
                t.answer_targets(P0, &[obj(a)]);
                t.enter(P0, card);
                t.g.flush_events();
                t.settle();
                (
                    a,
                    Box::new(|t, a| {
                        assert_eq!(t.pt(a), (4, 4));
                        assert!(!has_kw(t, a, KeywordKind::Flying));
                        assert!(t.obj_now(a).chars.has_subtype("Lizard"));
                        assert_eq!(
                            t.obj_now(a).chars.colors,
                            mtg_engine::types::ColorSet::single(Color::Green)
                        );
                    }),
                )
            }
            "The Irencrag" => {
                let i = t.battlefield(P0, card);
                t.answer_yes(P0, true);
                t.enter(P0, "Drana, the Last Bloodchief");
                t.g.flush_events();
                t.settle();
                (
                    i,
                    Box::new(|t, i| {
                        assert_eq!(t.obj_now(i).chars.name, "Everflame, Heroes' Legacy");
                        assert!(t.obj_now(i).chars.has_subtype("Equipment"));
                        assert!(!has_ability(t, i, "{C}"));
                    }),
                )
            }
            _ => {
                let k = t.battlefield(P0, card);
                let b = t.battlefield(P1, "Grizzly Bears");
                t.answer_targets(P0, &[obj(b)]);
                kill(&mut t, k);
                (b, Box::new(|t, b| assert_eq!(ctl(t, b), P0)))
            }
        };
        t.resolve_all();
        check(&t, id);
        two_turns(&mut t);
        check(&t, id);
    }
}

#[test]
fn losing_all_abilities_lasts_indefinitely() {
    cr!("611.2a", "613.1f", "122.1b");
    ruling!(
        "Abigale, Eloquent First-Year",
        "The effect of Abigale's last ability that causes the target creature to lose all abilities lasts indefinitely. It doesn't expire during the cleanup step or when Abigale leaves the battlefield."
    );
    ruling!(
        "Retched Wretch",
        "The effect of Retched Wretch's last ability that causes it to lose all abilities lasts indefinitely. It doesn't expire during the cleanup step."
    );
    supported("Abigale, Eloquent First-Year");
    supported("Retched Wretch");
    // Abigale: Serra Angel loses vigilance (and its printed flying), then gets flying,
    // first strike and lifelink counters.
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    t.answer_targets(P0, &[obj(angel)]);
    let ab = t.enter(P0, "Abigale, Eloquent First-Year");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    kill(&mut t, ab);
    two_turns(&mut t);
    assert!(!has_kw(&t, angel, KeywordKind::Vigilance));
    for k in [
        KeywordKind::Flying,
        KeywordKind::FirstStrike,
        KeywordKind::Lifelink,
    ] {
        assert!(has_kw(&t, angel, k));
    }
    // Retched Wretch dies with a -1/-1 counter, returns without abilities; turns later it
    // dies with a -1/-1 counter again and stays dead.
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Retched Wretch");
    t.g.add_counters(obj(w), "-1/-1", 1, None);
    kill(&mut t, w);
    t.resolve_all();
    let w = t.named_on_battlefield("Retched Wretch")[0];
    assert!(t.obj_now(w).chars.abilities.is_empty());
    two_turns(&mut t);
    assert!(t.obj_now(w).chars.abilities.is_empty());
    t.g.add_counters(obj(w), "-1/-1", 1, None);
    kill(&mut t, w);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Retched Wretch"));
}

#[test]
fn phyrexian_scriptures_artifact_outlasts_the_saga() {
    cr!("611.2a", "714.4");
    ruling!(
        "Phyrexian Scriptures",
        "The effect of Phyrexian Scriptures's first chapter ability lasts indefinitely. It doesn't expire when Phyrexian Scriptures leaves the battlefield."
    );
    let mut t = TestGame::new(2);
    let s = blank_saga(&mut t, P0, "Phyrexian Scriptures");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    add_lore(&mut t, s, 1);
    t.resolve_all();
    assert!(t.obj_now(bears).is(CardType::Artifact));
    exile(&mut t, s);
    two_turns(&mut t);
    assert!(t.obj_now(bears).is(CardType::Artifact));
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn shyft_keeps_its_color_until_the_next_change() {
    cr!("611.2a", "105.3", "613.1e");
    ruling!(
        "Shyft",
        "The ability’s effect has no duration. Shyft will remain the chosen color or colors until the game ends, it leaves the battlefield, or some other effect (such as the next time its ability resolves) changes its colors."
    );
    let mut t = TestGame::new(2);
    let shyft = t.battlefield(P0, "Shyft");
    t.set_step(P1, Step::End);
    t.answer_yes(P0, true);
    choose_words(&mut t, P0, &["black and green"]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let c = t.obj_now(shyft).chars.colors;
    assert!(c.contains(Color::Black) && c.contains(Color::Green) && !c.contains(Color::Blue));
    // Declined next time: unchanged.
    t.answer_yes(P0, false);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.obj_now(shyft).chars.colors, c);
    // Chosen again: red.
    t.answer_yes(P0, true);
    choose_words(&mut t, P0, &["red"]);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(
        t.obj_now(shyft).chars.colors,
        mtg_engine::types::ColorSet::single(Color::Red)
    );
    // A new object is blue again.
    let again = flicker(&mut t, shyft);
    assert_eq!(
        t.obj_now(again).chars.colors,
        mtg_engine::types::ColorSet::single(Color::Blue)
    );
}

#[test]
fn dream_coat_color_change_doesnt_wear_off() {
    cr!("611.2a", "105.3");
    ruling!(
        "Dream Coat",
        "The color change does not wear off at end of turn."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let coat = cast_at(&mut t, P0, "Dream Coat", &[obj(bears)]);
    t.resolve_all();
    let coat = t.g.current(coat);
    choose_words(&mut t, P0, &["white"]);
    act(&mut t, P0, coat, 0, &[]);
    two_turns(&mut t);
    assert_eq!(
        t.obj_now(bears).chars.colors,
        mtg_engine::types::ColorSet::single(Color::White)
    );
}

#[test]
fn gaeas_liege_makes_a_land_only_a_forest_but_still_basic() {
    cr!("305.7", "205.4a");
    ruling!(
        "Gaea's Liege",
        "The activated ability doesn't affect whether the land is basic or not. It overwrites any other land types. Being a Forest gives the affected land the ability \"{T}: Add {G}.\""
    );
    supported("Gaea's Liege");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let liege = t.battlefield(P0, "Gaea's Liege");
    let island = t.battlefield(P1, "Island");
    act(&mut t, P0, liege, 0, &[obj(island)]);
    let o = t.obj_now(island);
    assert!(o.chars.supertypes.contains(Supertype::Basic));
    assert!(o.chars.has_subtype("Forest") && !o.chars.has_subtype("Island"));
    assert!(has_ability(&t, island, "{G}"));
    assert!(!has_ability(&t, island, "{U}"));
}

#[test]
fn kyoshis_island_keeps_its_land_types_and_abilities() {
    cr!("611.2a", "305.7", "714.2b");
    ruling!(
        "The Legend of Kyoshi // Avatar Kyoshi",
        "The effect making the land an Island doesn't have a duration and will last indefinitely while that land is on the battlefield. The land retains any land types and abilities it already had. An Island has the ability \"{T}: Add {U}.\""
    );
    supported("The Legend of Kyoshi // Avatar Kyoshi");
    let mut t = TestGame::new(2);
    let s = blank_saga(&mut t, P0, "The Legend of Kyoshi // Avatar Kyoshi");
    t.g.objects[s.0 as usize].counters.insert("lore".into(), 1);
    let forest = t.battlefield(P0, "Forest");
    for _ in 0..3 {
        t.hand(P0, "Island");
    }
    t.answer_targets(P0, &[obj(forest)]);
    add_lore(&mut t, s, 1);
    t.resolve_all();
    two_turns(&mut t);
    let o = t.obj_now(forest);
    assert!(o.chars.has_subtype("Forest") && o.chars.has_subtype("Island"));
    assert!(has_ability(&t, forest, "{G}") && has_ability(&t, forest, "{U}"));
}

#[test]
fn ascent_of_the_worthy_returns_a_flying_angel_warrior() {
    cr!("714.2b", "122.1b", "205.1b");
    ruling!(
        "Ascent of the Worthy",
        "The creature card put onto the battlefield by the chapter III ability will enter the battlefield as a flying Angel Warrior (in addition to whatever other creature types it has)."
    );
    supported("Ascent of the Worthy");
    let mut t = TestGame::new(2);
    let s = blank_saga(&mut t, P0, "Ascent of the Worthy");
    t.g.objects[s.0 as usize].counters.insert("lore".into(), 2);
    let card = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(card)]);
    add_lore(&mut t, s, 1);
    t.resolve_all();
    let b = t.g.current(card);
    assert!(t.on_battlefield(b));
    assert_eq!(creature_types(&t, b), vec!["Angel", "Bear", "Warrior"]);
    assert!(has_kw(&t, b, KeywordKind::Flying));
    assert_eq!(t.counters(b, "flying"), 1);
}

#[test]
fn ascent_of_the_worthy_redirected_combat_damage_is_still_combat_damage() {
    cr!("614.1a", "510.2", "120.3");
    ruling!(
        "Ascent of the Worthy",
        "The chapter I/II ability doesn’t change whether the damage is combat damage or not. If combat damage would be dealt to a creature you control, the damage that’s dealt instead to the chosen creature is still combat damage."
    );
    let mut t = TestGame::new(2);
    let s = blank_saga(&mut t, P0, "Ascent of the Worthy");
    let chosen = t.battlefield(P0, "Hill Giant");
    let blocker = t.battlefield(P0, "Grizzly Bears");
    let attacker = t.battlefield(P1, "Savannah Lions");
    t.answer_choose(P0, &[obj(chosen)]);
    add_lore(&mut t, s, 1);
    t.resolve_all();
    t.advance_to(P1, Step::BeginningOfCombat);
    t.attack(&[(attacker, pl(P0))], &[(blocker, attacker)]);
    let hits: Vec<(Entity, bool)> =
        t.g.turn_events
            .iter()
            .filter_map(|e| match e {
                Event::Damage {
                    source,
                    target,
                    combat,
                    ..
                } if *source == attacker => Some((*target, *combat)),
                _ => None,
            })
            .collect();
    assert_eq!(hits, vec![(obj(chosen), true)]);
    assert_eq!(crate::r_p009_common::dmg(&t, chosen), 2);
    assert_eq!(crate::r_p009_common::dmg(&t, blocker), 0);
}

#[test]
fn stigma_lasher_no_life_gain_lasts_without_the_lasher() {
    cr!("611.2a", "119.7");
    ruling!(
        "Stigma Lasher",
        "Once the last ability resolves, it will be in effect for the rest of the game. It doesn’t matter if Stigma Lasher remains on the battlefield or not."
    );
    supported("Stigma Lasher");
    let mut t = TestGame::new(2);
    let lasher = t.battlefield(P0, "Stigma Lasher");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(lasher, pl(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    kill(&mut t, lasher);
    two_turns(&mut t);
    crate::r_p009_common::gain(&mut t, P1, 5);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn screaming_nemesis_life_gain_spells_still_do_the_rest() {
    cr!("119.7", "608.2c");
    ruling!(
        "Screaming Nemesis",
        "Spells and abilities that cause a player affected by Screaming Nemesis's last ability to gain life still resolve. That player won't gain life, but any other effects of that spell or ability will happen."
    );
    supported("Screaming Nemesis");
    supported("Revitalize");
    let mut t = TestGame::new(2);
    let nem = t.battlefield(P0, "Screaming Nemesis");
    t.answer_targets(P0, &[pl(P1)]);
    crate::r_s06_common::damage(&mut t, nem, 2, nem);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    t.library_top(P1, "Island");
    let hand = t.hand_size(P1);
    t.set_step(P1, Step::PrecombatMain);
    cast_at(&mut t, P1, "Revitalize", &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Revitalize was put into the hand and cast; the draw still happened.
    assert_eq!(t.hand_size(P1), hand + 1);
}

#[test]
fn kheru_lich_lord_exile_abilities_belong_to_the_ability_not_the_creature() {
    cr!("603.7", "614.1", "613.1f");
    ruling!(
        "Kheru Lich Lord",
        "Kheru Lich Lord grants flying, trample, and haste to the creature that’s returned to the battlefield. However, neither of the “exile” abilities is granted to that creature. If that creature loses all its abilities, it will still be exiled at the beginning of your next end step, and if it would leave the battlefield, it is still exiled instead."
    );
    ruling!(
        "Kheru Lich Lord",
        "The creature card returned to the battlefield is chosen at random as the ability resolves. If any player responds to the ability, that player won’t yet know what card will be returned."
    );
    let setup = |t: &mut TestGame| -> ObjectId {
        t.battlefield(P0, "Kheru Lich Lord");
        let card = t.graveyard(P0, "Grizzly Bears");
        t.lands(P0, "Swamp", 3);
        t.set_step(P1, Step::End);
        t.answer_yes(P0, true);
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        // On the stack: nothing chosen yet.
        assert_eq!(t.zone(card), Zone::Graveyard(P0));
        t.resolve_all();
        let b = t.g.current(card);
        assert!(t.on_battlefield(b));
        b
    };
    let mut t = TestGame::new(2);
    let b = setup(&mut t);
    for k in [
        KeywordKind::Flying,
        KeywordKind::Trample,
        KeywordKind::Haste,
    ] {
        assert!(has_kw(&t, b, k));
    }
    assert!(t
        .obj_now(b)
        .chars
        .abilities
        .iter()
        .all(|a| !a.text.contains("xile")));
    // It loses all its abilities (Turn to Frog): still exiled at the end step.
    cast_at(&mut t, P1, "Turn to Frog", &[obj(b)]);
    t.resolve_all();
    assert!(!has_kw(&t, b, KeywordKind::Flying));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    // ... and dying is replaced by exile, also without abilities.
    let mut t = TestGame::new(2);
    let b = setup(&mut t);
    cast_at(&mut t, P1, "Turn to Frog", &[obj(b)]);
    t.resolve_all();
    kill(&mut t, b);
    assert!(t.in_exile("Grizzly Bears"));
    assert!(!t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn kheru_lich_lord_countered_delayed_trigger_leaves_the_replacement() {
    cr!("603.7", "614.1", "701.6a");
    ruling!(
        "Kheru Lich Lord",
        "The ability that exiles the card at the beginning of your next end step is a delayed triggered ability. If the delayed triggered ability is countered, the creature will stay on the battlefield and the ability won’t trigger again. However, the replacement effect will still exile the creature when it eventually leaves the battlefield."
    );
    supported("Stifle");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kheru Lich Lord");
    let card = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    t.set_step(P1, Step::End);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let b = t.g.current(card);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let top = *t.g.stack.last().unwrap();
    lots(&mut t, P1);
    let st = t.hand(P1, "Stifle");
    t.cast(P1, st).target(top).go();
    t.resolve_all();
    assert!(t.on_battlefield(b));
    two_turns(&mut t);
    assert!(t.on_battlefield(b));
    kill(&mut t, b);
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn souvenir_snatcher_keeps_an_artifact_that_becomes_a_creature() {
    cr!("611.2a", "611.2c");
    ruling!(
        "Souvenir Snatcher",
        "Once Souvenir Snatcher has snatched a noncreature artifact, you’ll continue to control that artifact if it becomes an artifact creature."
    );
    supported("Souvenir Snatcher");
    supported("March of the Machines");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let host = t.battlefield(P0, "Hill Giant");
    let ring = t.battlefield(P1, "Sol Ring");
    lots(&mut t, P0);
    let c = t.hand(P0, "Souvenir Snatcher");
    t.cast(P0, c)
        .method(CastMethod::Keyword(KeywordKind::Mutate))
        .target(host)
        .go();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer_targets(P0, &[obj(ring)]);
    t.resolve();
    t.settle();
    t.resolve_all();
    assert_eq!(ctl(&t, ring), P0);
    t.battlefield(P1, "March of the Machines");
    assert!(is_creature(&t, ring));
    assert_eq!(ctl(&t, ring), P0);
}

#[test]
fn risky_moves_control_change_outlasts_the_turn_and_the_enchantment() {
    cr!("611.2a", "611.2b", "705.2");
    ruling!(
        "Risky Move",
        "The change of control of creatures does not end at end of turn or even when this card leaves the battlefield."
    );
    let mut t = TestGame::new(2);
    let rm = t.battlefield(P0, "Risky Move");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // P1 gains control in their upkeep, chooses the Bears and P0, and loses the flip.
    t.answer_choose(P1, &[obj(bears)]);
    t.answer_choose(P1, &[pl(P0)]);
    t.g.dice.loaded_coins.push_back(false);
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Draw);
    t.resolve_all();
    assert_eq!(ctl(&t, rm), P1);
    assert_eq!(ctl(&t, bears), P0);
    // P0's upkeep: Risky Move comes back to P0, whose own trigger's flip is won: P0
    // keeps the Bears.
    t.g.dice.loaded_coins.push_back(true);
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::Draw);
    t.resolve_all();
    assert_eq!(ctl(&t, bears), P0);
    exile(&mut t, rm);
    pass_turn(&mut t);
    assert_eq!(ctl(&t, bears), P0);
}

#[test]
fn the_tarrasque_has_haste_and_ward_when_cast_from_another_zone() {
    cr!("601.2", "702.21a");
    ruling!(
        "The Tarrasque",
        "The Tarrasque will have haste and ward {10} even if it was cast from a zone other than your hand. May we suggest the command zone?"
    );
    supported("The Tarrasque");
    supported("Future Sight");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Future Sight");
    let card = t.library_top(P0, "The Tarrasque");
    lots(&mut t, P0);
    t.cast(P0, card).go();
    t.resolve_all();
    let tq = t.named_on_battlefield("The Tarrasque")[0];
    assert!(has_kw(&t, tq, KeywordKind::Haste));
    assert!(has_kw(&t, tq, KeywordKind::Ward));
    // Put onto the battlefield without being cast: neither.
    let mut t = TestGame::new(2);
    let tq = t.battlefield(P0, "The Tarrasque");
    assert!(!has_kw(&t, tq, KeywordKind::Haste));
    assert!(!has_kw(&t, tq, KeywordKind::Ward));
}
