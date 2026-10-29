//! cards5-referents-statics, family `notes`: information noted for an object (CR 607.2e),
//! kept on that object (a new object has none, CR 400.7).

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::kw::noted_info;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// The options of the last "Note a creature type" decision.
fn note_options(t: &TestGame) -> Vec<String> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(_, d)| match d {
            Decision::ChooseOption { prompt, options, .. } if prompt == "Note a creature type" => {
                Some(options)
            }
            _ => None,
        })
        .unwrap_or_default()
}

fn index_of(options: &[String], t: &str) -> usize {
    options.iter().position(|o| o == t).expect("an option")
}

#[test]
fn long_list_of_the_ents_notes_a_new_creature_type_each_chapter() {
    cr!("607.2e", "714.2b", "603.7b");
    supported("Long List of the Ents");
    let all: Vec<String> = subtype_lists().creature.clone();
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    // Chapter I: Elf.
    t.answer(P0, DecisionKind::Option, Answer::Index(index_of(&all, "Elf")));
    let saga = t.enter(P0, "Long List of the Ents");
    t.resolve_all();
    assert!(note_options(&t).contains(&"Elf".to_string()));
    t.lands(P0, "Forest", 2);
    let elves = t.hand(P0, "Llanowar Elves");
    t.cast(P0, elves).go();
    t.resolve_all();
    let elves = t.named_on_battlefield("Llanowar Elves")[0];
    assert_eq!(t.counters(elves, "+1/+1"), 1);
    // Chapter II: Elf has been noted and isn't an option any more.
    let left: Vec<String> = all.iter().filter(|x| *x != "Elf").cloned().collect();
    t.answer(P0, DecisionKind::Option, Answer::Index(index_of(&left, "Human")));
    t.g.add_counters(Entity::Object(saga), "lore", 1, None);
    t.resolve_all();
    let options = note_options(&t);
    assert!(!options.contains(&"Elf".to_string()));
    assert_eq!(options.len(), all.len() - 1);
}

#[test]
fn volo_draws_for_each_creature_type_noted_for_the_journal() {
    cr!("607.2e", "111.4", "201.4");
    supported("Volo, Itinerant Scholar");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let volo = t.enter(P0, "Volo, Itinerant Scholar");
    t.resolve_all();
    let journal = t.named_on_battlefield("Volo's Journal");
    assert_eq!(journal.len(), 1);
    let journal = journal[0];
    t.lands(P0, "Forest", 2);
    // Llanowar Elves (Elf Druid): note Elf; then Elvish Mystic: only Druid is left.
    let elves = t.hand(P0, "Llanowar Elves");
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, elves).go();
    t.resolve_all();
    let first = note_options(&t);
    assert_eq!(first.len(), 2);
    assert_eq!(noted_info::noted_types(&t.g, journal), vec!["Elf"]);
    let mystic = t.hand(P0, "Elvish Mystic");
    t.cast(P0, mystic).go();
    t.resolve_all();
    assert_eq!(noted_info::noted_types(&t.g, journal), vec!["Elf", "Druid"]);
    // {2}, {T}: two creature types noted: draw two.
    t.g.objects[volo.0 as usize].summoning_sick = false;
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    t.lands(P0, "Island", 2);
    let hand = t.hand_size(P0);
    t.activate(P0, volo, 0, &[Entity::Object(journal)])
        .expect("activates");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn bell_borca_power_is_the_greatest_mana_value_noted_this_turn() {
    cr!("607.2e", "604.3");
    ruling!(
        "Bell Borca, Spectral Sergeant",
        "Bell Borca notes mana values only if it's on the battlefield as a card is put into exile"
    );
    ruling!(
        "Bell Borca, Spectral Sergeant",
        "before any cards are noted for it each turn, Bell Borca's power is 0"
    );
    supported("Bell Borca, Spectral Sergeant");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    // Exiled before Bell Borca is on the battlefield: not noted.
    let wurm = t.graveyard(P1, "Craw Wurm");
    t.g.move_object(wurm, Zone::Exile, events::MoveCause::Effect, None);
    t.settle();
    let bell = t.battlefield(P0, "Bell Borca, Spectral Sergeant");
    t.g.recompute();
    assert_eq!(t.pt(bell).0, 0);
    for name in ["Hill Giant", "Lightning Bolt"] {
        let c = t.graveyard(P1, name);
        t.g.move_object(c, Zone::Exile, events::MoveCause::Effect, None);
        t.settle();
    }
    t.g.recompute();
    assert_eq!(t.pt(bell), (4, 5));
    // A new turn: nothing noted this turn until the upkeep exile (Grizzly Bears, 2).
    t.library_top(P0, "Grizzly Bears");
    t.advance_to(P1, Step::PrecombatMain);
    t.g.recompute();
    assert_eq!(t.pt(bell).0, 0);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    t.g.recompute();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.pt(bell).0, 2);
}

#[test]
fn long_list_of_the_ents_each_chapter_remembers_its_own_type() {
    cr!("607.2e");
    let all: Vec<String> = subtype_lists().creature.clone();
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    // Chapter I notes Elf; chapter II, the same turn, notes Human.
    t.answer(P0, DecisionKind::Option, Answer::Index(index_of(&all, "Elf")));
    let saga = t.enter(P0, "Long List of the Ents");
    t.resolve_all();
    let left: Vec<String> = all.iter().filter(|x| *x != "Elf").cloned().collect();
    t.answer(P0, DecisionKind::Option, Answer::Index(index_of(&left, "Human")));
    t.g.add_counters(Entity::Object(saga), "lore", 1, None);
    t.resolve_all();
    // The Elf trigger from chapter I still waits for an Elf spell.
    t.lands(P0, "Forest", 1);
    let elves = t.hand(P0, "Llanowar Elves");
    t.cast(P0, elves).go();
    t.resolve_all();
    let elves = t.named_on_battlefield("Llanowar Elves")[0];
    assert_eq!(t.counters(elves, "+1/+1"), 1);
    t.lands(P0, "Plains", 1);
    let v = t.hand(P0, "Elite Vanguard");
    t.cast(P0, v).go();
    t.resolve_all();
    let v = t.named_on_battlefield("Elite Vanguard")[0];
    assert_eq!(t.counters(v, "+1/+1"), 1);
}
