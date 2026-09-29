//! Rulings batch S31 — what counts as entering the battlefield (CR 603.6a, 614.12): a land
//! becoming a creature doesn't enter, becoming the monarch when you already are doesn't
//! trigger, a modal enters trigger keeps its mode, Blood Moon and Magus of the Moon take
//! away a land's enters abilities before they apply, Kismet's permanents enter tapped
//! (they don't become tapped), and Soul Warden sees creatures entering along with it.

use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s04_common::add_mana;
use crate::r_s06_common::activate_containing;
use crate::r_s07_common::chosen_modes;
use crate::r_s14_common::triggers_from;
use crate::r_s29_common::choose_modes;
use crate::r_s31_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 activates the land's "becomes a creature" ability with mana from their pool, and it
/// resolves.
fn animate(t: &mut TestGame, land: ObjectId, cost: &[(ManaType, u32)]) {
    for (ty, n) in cost {
        add_mana(t, P0, *ty, *n);
    }
    activate_containing(t, P0, land, "becomes").expect("activate");
    t.resolve_all();
}

#[test]
fn a_land_becoming_a_creature_doesnt_enter_the_battlefield() {
    cr!("603.6a", "305.1", "205.1b");
    ruling!(
        "Creeping Tar Pit",
        "When a land becomes a creature, that doesn't count as having a creature enter. The permanent was already on the battlefield; it only changed its types. Abilities that trigger whenever a creature enters won't trigger."
    );
    ruling!(
        "Lumbering Falls",
        "When a land becomes a creature, that doesn't count as having a creature enter the battlefield. The permanent was already on the battlefield; it only changed its types. Abilities that trigger whenever a creature enters the battlefield won't trigger."
    );
    supported("Creeping Tar Pit");
    supported("Lumbering Falls");
    supported("Soul Warden");
    // Soul Warden: "Whenever another creature enters, you gain 1 life."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul Warden");
    let pit = t.battlefield(P0, "Creeping Tar Pit");
    let falls = t.battlefield(P0, "Lumbering Falls");
    animate(&mut t, pit, &[(ManaType::C, 1), (ManaType::U, 1), (ManaType::B, 1)]);
    animate(&mut t, falls, &[(ManaType::C, 2), (ManaType::G, 1), (ManaType::U, 1)]);
    assert!(t.obj_now(pit).is(CardType::Creature));
    assert!(t.obj_now(falls).is(CardType::Creature));
    assert_eq!(t.pt(pit), (3, 2));
    assert_eq!(t.pt(falls), (3, 3));
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.stack_len(), 0);
    // A creature that does enter makes it trigger.
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn becoming_the_monarch_while_already_the_monarch_doesnt_trigger() {
    cr!("725.2", "603.2");
    ruling!(
        "Custodi Lich",
        "Abilities that trigger whenever you “become the monarch” trigger only if you aren't already the monarch. For example, if you are already the monarch as Custodi Lich enters the battlefield, its last ability won't trigger."
    );
    ruling!(
        "Palace Sentinels",
        "Abilities that trigger whenever you \"become the monarch\" trigger only if you aren't already the monarch. For example, if you are already the monarch as Custodi Lich enters the battlefield, its last ability won't trigger."
    );
    supported("Custodi Lich");
    supported("Palace Sentinels");
    // Custodi Lich: "When this creature enters, you become the monarch. Whenever you become
    // the monarch, target player sacrifices a creature of their choice."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    mtg_engine::designations::become_monarch(&mut t.g, P0);
    t.g.flush_events();
    t.settle();
    let lich = t.enter(P0, "Custodi Lich");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.g.monarch, Some(P0));
    assert!(t.on_battlefield(bears));
    // Palace Sentinels ("When this creature enters, you become the monarch.") entering
    // while P0 is still the monarch doesn't make the Lich's ability trigger either.
    t.enter(P0, "Palace Sentinels");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(triggers_from(&t, lich), 0);
    // Once P1 has taken the monarchy, P0 becoming the monarch again does trigger it.
    mtg_engine::designations::become_monarch(&mut t.g, P1);
    t.g.flush_events();
    t.settle();
    t.enter(P0, "Palace Sentinels");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.g.monarch, Some(P0));
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn a_modal_enters_trigger_keeps_its_mode_if_the_creature_leaves() {
    cr!("700.2b", "603.3c", "608.2b");
    ruling!(
        "Ainok Guide",
        "You choose which mode you're using as you put the ability on the stack, after the creature has entered the battlefield. Once you've chosen a mode, you can't change that mode even if the creature leaves the battlefield in response to that ability."
    );
    supported("Ainok Guide");
    // "When this creature enters, choose one — • Put a +1/+1 counter on this creature.
    // • Search your library for a basic land card, reveal it, then shuffle and put that
    // card on top."
    let mut t = TestGame::new(2);
    t.library_top(P0, "Forest");
    let from = t.asked().len();
    choose_modes(&mut t, P0, &[0]);
    let guide = t.enter(P0, "Ainok Guide");
    t.settle();
    // The mode was chosen as the trigger was put on the stack, after Ainok Guide entered.
    let asked: Vec<Decision> = t.asked()[from..].iter().map(|(_, d)| d.clone()).collect();
    assert!(asked
        .iter()
        .any(|d| matches!(d, Decision::ChooseModes { .. })));
    let trig = t.g.stack.last().copied().expect("trigger");
    assert_eq!(chosen_modes(&t, trig), vec![0]);
    // Ainok Guide leaves in response: the mode stays "put a +1/+1 counter"; nothing is
    // searched for.
    destroy(&mut t, guide);
    assert_eq!(chosen_modes(&t, trig), vec![0]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Ainok Guide"));
    assert!(!t
        .g
        .turn_events
        .iter()
        .any(|e| matches!(e, Event::Searched { .. })));
}

#[test]
fn blood_moon_removes_a_nonbasic_lands_enters_trigger_before_it_triggers() {
    cr!("614.12", "603.6a", "613.1d", "613.1f");
    ruling!(
        "Blood Moon",
        "If a nonbasic land has an ability that triggers \"when\" it enters the battlefield, it will lose that ability before it can trigger."
    );
    supported("Blood Moon");
    supported("Karoo");
    // Karoo: "This land enters tapped. When this land enters, sacrifice it unless you
    // return an untapped Plains you control to its owner's hand."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Blood Moon");
    let plains = t.battlefield(P0, "Plains");
    let karoo = t.enter(P0, "Karoo");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert!(t.on_battlefield(karoo));
    assert!(t.on_battlefield(plains));
    assert!(t.obj_now(karoo).chars.has_subtype("Mountain"));
}

#[test]
fn magus_of_the_moon_removes_enters_tapped_and_as_enters_abilities() {
    cr!("614.12", "613.1d", "613.1f", "305.7");
    ruling!(
        "Magus of the Moon",
        "If a nonbasic land has an ability that causes it to enter the battlefield tapped, it will lose that ability before it can apply. The same is also true of any other abilities that modify how a land enters the battlefield or apply \"as\" a land enters the battlefield, such as that of Vesuva or Cavern of Souls."
    );
    supported("Magus of the Moon");
    supported("Vesuva");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Magus of the Moon");
    let summit = t.battlefield(P1, "Dragonskull Summit");
    // Karoo ("This land enters tapped.") enters untapped.
    let karoo = t.enter(P0, "Karoo");
    t.settle();
    assert!(!entered_tapped(&t, karoo));
    // Vesuva ("You may have this land enter tapped as a copy of any land on the
    // battlefield.") can't become a copy: it enters untapped as a Mountain named Vesuva.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(summit)]);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let vesuva = t.enter(P0, "Vesuva");
    t.settle();
    assert_eq!(t.obj_now(vesuva).chars.name.as_str(), "Vesuva");
    assert!(!entered_tapped(&t, vesuva));
    assert!(t.obj_now(vesuva).chars.has_subtype("Mountain"));
    // Without Magus of the Moon, the same choices make it enter tapped as a copy.
    let mut t = TestGame::new(2);
    let summit = t.battlefield(P1, "Dragonskull Summit");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(summit)]);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let vesuva = t.enter(P0, "Vesuva");
    t.settle();
    assert_eq!(t.obj_now(vesuva).chars.name.as_str(), "Dragonskull Summit");
    assert!(entered_tapped(&t, vesuva));
}

#[test]
fn permanents_entering_tapped_dont_become_tapped() {
    cr!("614.1c", "614.12", "603.2");
    ruling!(
        "Kismet",
        "The appropriate permanents enter tapped. They do not enter untapped and then immediately tap, therefore they do not trigger any effects due to tapping."
    );
    supported("Kismet");
    supported("Stonybrook Schoolmaster");
    // Kismet: "Artifacts, creatures, and lands your opponents control enter tapped."
    // Stonybrook Schoolmaster: "Whenever this creature becomes tapped, you may create a
    // 1/1 blue Merfolk Wizard creature token."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kismet");
    let school = t.enter(P1, "Stonybrook Schoolmaster");
    t.settle();
    assert!(entered_tapped(&t, school));
    assert_eq!(t.stack_len(), 0);
    assert!(!t
        .g
        .turn_events
        .iter()
        .any(|e| matches!(e, Event::Tapped { .. })));
    // Becoming tapped later does trigger it.
    t.g.untap(school);
    t.g.tap(school);
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_from(&t, school), 1);
}

#[test]
fn soul_warden_triggers_for_each_creature_entering_with_it() {
    cr!("603.6a", "603.2c");
    ruling!(
        "Soul Warden",
        "If this creature enters at the same time as one or more other creatures, its ability will trigger for each of those other creatures."
    );
    supported("Soul Warden");
    supported("Essence Warden");
    // "Whenever another creature enters, you gain 1 life."
    let mut t = TestGame::new(2);
    let cards = wave(&mut t, &["Soul Warden", "Grizzly Bears", "Grizzly Bears"]);
    assert!(cards.iter().all(|c| t.on_battlefield(*c)));
    // Two other creatures: two life (not three: it doesn't see itself).
    assert_eq!(t.life(P0), 22);
}
