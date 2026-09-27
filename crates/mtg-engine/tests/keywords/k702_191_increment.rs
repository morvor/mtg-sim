//! CR 702.191 Increment (`src/kw/increment.rs`).

use crate::common_k702_178_195::*;
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn plus1(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, counters::PLUS1)
}

fn modify(t: &mut TestGame, id: ObjectId, mods: Vec<Modification>) {
    run(
        t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods,
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(id)],
    );
}

#[test]
fn increment_cards_compile() {
    assert_supported(&[
        "Cuboid Colony",
        "Hungry Graffalon",
        "Pensive Professor",
        "Topiary Lecturer",
    ]);
}

#[test]
fn a_spell_with_more_mana_spent_than_power_or_toughness_adds_a_counter() {
    cr!("702.191a");
    ruling!(
        "Hungry Graffalon",
        "If the amount of mana you spent isn't greater than either of that creature's stats, increment won't trigger at all."
    );
    // Cuboid Colony: a 1/1 with increment.
    let mut t = TestGame::new(2);
    let colony = t.battlefield(P0, "Cuboid Colony");
    t.lands(P0, "Forest", 6);
    // One mana isn't greater than 1: no trigger.
    let elves = t.hand(P0, "Llanowar Elves");
    t.cast(P0, elves).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(plus1(&t, colony), 0);
    // Two mana: a counter; it resolves before the spell.
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(plus1(&t, colony), 1);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    // Now a 2/2: two mana isn't greater than either.
    let bears2 = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears2).go();
    t.resolve_all();
    assert_eq!(plus1(&t, colony), 1);
    // An opponent's spell doesn't trigger it.
    let mut t = TestGame::new(2);
    let colony = t.battlefield(P0, "Cuboid Colony");
    t.lands(P1, "Forest", 2);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let bears = t.hand(P1, "Grizzly Bears");
    t.cast(P1, bears).go();
    t.resolve_all();
    assert_eq!(plus1(&t, colony), 0);
}

#[test]
fn greater_than_power_or_toughness_is_enough() {
    cr!("702.191a");
    // Pensive Professor is a 0/2: one mana is greater than its power.
    let mut t = TestGame::new(2);
    let prof = t.battlefield(P0, "Pensive Professor");
    t.lands(P0, "Forest", 1);
    let elves = t.hand(P0, "Llanowar Elves");
    t.cast(P0, elves).go();
    t.resolve_all();
    assert_eq!(plus1(&t, prof), 1);
}

#[test]
fn the_comparison_is_made_again_on_resolution() {
    cr!("702.191a");
    ruling!(
        "Hungry Graffalon",
        "If the amount of mana spent to cast that spell isn't greater than either of that creature's stats anymore, the ability will do nothing."
    );
    ruling!(
        "Hungry Graffalon",
        "it's possible that the stat that is less than the mana spent changes from power to toughness or vice versa"
    );
    // Hungry Graffalon is a 3/4. Four mana: greater than its power.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Hungry Graffalon");
    t.lands(P0, "Forest", 4);
    let juggernaut = t.hand(P0, "Juggernaut");
    t.cast(P0, juggernaut).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // In response it becomes 5/4: four isn't greater than either.
    modify(&mut t, g, vec![Modification::ModifyPT(Value::c(2), Value::c(0))]);
    t.resolve_all();
    assert_eq!(plus1(&t, g), 0);
    // A 3/4 with four mana spent that becomes 5/2: greater than its toughness now.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Hungry Graffalon");
    t.lands(P0, "Forest", 4);
    let spell = t.hand(P0, "Juggernaut");
    t.cast(P0, spell).go();
    t.settle();
    modify(&mut t, g, vec![Modification::ModifyPT(Value::c(2), Value::c(-2))]);
    t.resolve_all();
    assert_eq!(plus1(&t, g), 1);
}

#[test]
fn only_while_it_is_a_creature() {
    cr!("702.191a");
    ruling!(
        "Hungry Graffalon",
        "if the permanent is no longer a creature when its increment ability resolves, the ability won't do anything"
    );
    let mut t = TestGame::new(2);
    let colony = t.battlefield(P0, "Cuboid Colony");
    t.lands(P0, "Forest", 4);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    modify(
        &mut t,
        colony,
        vec![Modification::RemoveTypes(vec![CardType::Creature])],
    );
    t.resolve_all();
    assert_eq!(plus1(&t, colony), 0);
    // Not a creature: it doesn't trigger at all.
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn it_resolves_even_if_the_spell_is_countered() {
    cr!("702.191a");
    ruling!(
        "Hungry Graffalon",
        "It resolves even if that spell is countered or otherwise leaves the stack."
    );
    let mut t = TestGame::new(2);
    let colony = t.battlefield(P0, "Cuboid Colony");
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, bears).go();
    t.settle();
    run(
        &mut t,
        P1,
        None,
        Effect::CounterSpell {
            what: Sel::Target(0),
        },
        &[Entity::Object(spell)],
    );
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert_eq!(plus1(&t, colony), 1);
}

#[test]
fn mana_actually_spent_counts() {
    cr!("702.191a");
    // Cast without paying its mana cost, no mana was spent.
    let mut t = TestGame::new(2);
    let colony = t.battlefield(P0, "Cuboid Colony");
    let wurm = t.hand(P0, "Craw Wurm");
    run(
        &mut t,
        P0,
        None,
        Effect::CastCard {
            who: PlayerRef::You,
            what: Sel::Target(0),
            free: true,
            optional: false,
        },
        &[Entity::Object(wurm)],
    );
    t.resolve_all();
    assert_eq!(plus1(&t, colony), 0);
    assert_eq!(t.zone(wurm), Zone::Battlefield);
}

#[test]
fn multiple_instances_trigger_separately() {
    cr!("702.191b");
    let def = custom_card(
        "Twice Incremented",
        "{2}",
        "Creature — Construct",
        Some((1, 1)),
        "Increment\nIncrement",
    );
    let mut t = TestGame::new(2);
    let c = put(&mut t, P0, def, Zone::Battlefield);
    t.lands(P0, "Forest", 3);
    let spell = t.hand(P0, "Grizzly Bears");
    t.cast(P0, spell).go();
    t.settle();
    assert_eq!(t.stack_len(), 3);
    t.resolve_all();
    // The second one checks again: 2 isn't greater than 2 after the first counter.
    assert_eq!(plus1(&t, c), 1);
    // Four mana on a 2/2: both put a counter (4 > 3 after the first).
    let mut t2 = TestGame::new(2);
    let def = custom_card(
        "Twice Incremented",
        "{2}",
        "Creature — Construct",
        Some((1, 1)),
        "Increment\nIncrement",
    );
    let c2 = put(&mut t2, P0, def, Zone::Battlefield);
    t2.lands(P0, "Forest", 4);
    let spell = t2.hand(P0, "Juggernaut");
    t2.cast(P0, spell).go();
    t2.resolve_all();
    assert_eq!(plus1(&t2, c2), 2);
}
