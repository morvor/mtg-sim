//! Rulings batch P205 — devour (CR 702.82): "As this object enters, you may sacrifice any
//! number of creatures. This permanent enters with N +1/+1 counters on it for each
//! creature sacrificed this way."

use crate::r_s01_common::*;
use crate::r_s02_common::create_token;
use crate::r_s04_common::next_upkeep;
use crate::r_s05_common::run_from;
use crate::r_s20_common::to_beginning_of_combat;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// The candidates offered by each devour choice since decision `from`.
fn devour_offers(t: &TestGame, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if prompt.contains("devour") => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

/// The cards `cards` (in P0's hand) enter the battlefield at the same time.
fn enter_together(t: &mut TestGame, cards: &[ObjectId]) {
    let targets: Vec<Entity> = cards.iter().map(|c| Entity::Object(*c)).collect();
    run_from(
        t,
        P0,
        None,
        Effect::Move {
            what: Sel::AllTargets,
            to: Destination::battlefield(),
        },
        &targets,
    );
}

fn objs(ids: &[ObjectId]) -> Vec<Entity> {
    ids.iter().map(|o| Entity::Object(*o)).collect()
}

#[test]
fn voracious_dragon_counts_goblins_devoured_as_they_last_existed() {
    cr!("702.82a", "702.82b", "608.2h");
    ruling!(
        "Voracious Dragon",
        "\"The number of Goblins it devoured\" means \"The number of Goblins sacrificed as a result of its devour ability as it entered.\" For each creature that Voracious Dragon devoured, this ability checks its creature type as it last existed on the battlefield to see if it was a Goblin at that time."
    );
    ruling!(
        "Voracious Dragon",
        "Voracious Dragon can devour any type of creatures, not just Goblins."
    );
    supported("Voracious Dragon");
    let mut t = TestGame::new(2);
    // A Goblin token (it ceases to exist once sacrificed) and a Grizzly Bears.
    let goblin = create_token(&mut t, P0, "Goblin");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // A Goblin that isn't devoured doesn't count.
    t.battlefield(P0, "Raging Goblin");
    t.answer_choose(P0, &objs(&[goblin, bears]));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let dragon = t.enter(P0, "Voracious Dragon");
    t.resolve_all();
    assert_eq!(t.counters(dragon, counters::PLUS1), 2);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn caprichrome_sacrifices_all_devoured_artifacts_at_once() {
    cr!("702.82a", "603.10a");
    ruling!(
        "Caprichrome",
        "All artifacts devoured this way are sacrificed at the same time."
    );
    supported("Caprichrome");
    supported("Magnetic Mine");
    // Magnetic Mine: "Whenever another artifact is put into a graveyard from the
    // battlefield, this artifact deals 2 damage to that artifact's controller." Each Mine
    // sees the other leave at the same time.
    let mut t = TestGame::new(2);
    let m1 = t.battlefield(P0, "Magnetic Mine");
    let m2 = t.battlefield(P0, "Magnetic Mine");
    t.answer_choose(P0, &objs(&[m1, m2]));
    let capri = t.enter(P0, "Caprichrome");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "deals 2 damage"), 2);
    t.resolve_all();
    assert_eq!(t.counters(capri, counters::PLUS1), 2);
    assert_eq!(t.life(P0), 16);
}

#[test]
fn caldera_hellion_damages_itself_too() {
    cr!("702.82a", "704.5g");
    ruling!(
        "Caldera Hellion",
        "Caldera Hellion will deal 3 damage to itself (as well as to each other creature) when its \"enters\" ability resolves. This damage will be lethal if Caldera Hellion hasn't devoured any creatures and its toughness hasn't been increased by any other means."
    );
    supported("Caldera Hellion");
    // Devouring nothing: it dies with the rest.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[]);
    let hellion = t.enter(P0, "Caldera Hellion");
    t.resolve_all();
    assert!(!t.on_battlefield(hellion));
    assert!(t.in_graveyard(P0, "Caldera Hellion"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // Devouring one creature: a 4/4 with 3 damage survives.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let hellion = t.enter(P0, "Caldera Hellion");
    t.resolve_all();
    assert!(t.on_battlefield(hellion));
    assert_eq!(t.obj(hellion).damage, 3);
}

#[test]
fn fell_beast_and_ribtruss_roaster_cant_devour_creatures_entering_with_them() {
    cr!("702.82a", "614.12");
    ruling!(
        "Fell Beast of Mordor",
        "Fell Beast of Mordor can't devour creatures entering the battlefield at the same time as it."
    );
    ruling!(
        "Ribtruss Roaster",
        "Ribtruss Roaster can't devour creatures entering at the same time as it."
    );
    for name in ["Fell Beast of Mordor", "Ribtruss Roaster"] {
        supported(name);
        let mut t = TestGame::new(2);
        let old = t.battlefield(P0, "Grizzly Bears");
        let devourer = t.hand(P0, name);
        let fresh = t.hand(P0, "Grizzly Bears");
        t.answer_choose(P0, &[Entity::Object(old)]);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        let from = t.asked().len();
        enter_together(&mut t, &[devourer, fresh]);
        assert_eq!(
            devour_offers(&t, from),
            vec![vec![Entity::Object(old)]],
            "{name}"
        );
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1, "{name}");
        let d = t.named_on_battlefield(name)[0];
        assert_eq!(t.counters(d, counters::PLUS1), 1, "{name}");
    }
}

#[test]
fn bloodspore_thrinax_doesnt_affect_or_devour_creatures_entering_with_it() {
    cr!("702.82a", "614.12", "614.1c");
    ruling!(
        "Bloodspore Thrinax",
        "If Bloodspore Thrinax enters the battlefield at the same time as other creatures you control, those creatures won't get additional +1/+1 counters from Bloodspore Thrinax's last ability. Those creatures also can't be devoured by Bloodspore Thrinax."
    );
    supported("Bloodspore Thrinax");
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P0, "Raging Goblin");
    let thrinax = t.hand(P0, "Bloodspore Thrinax");
    let fresh = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(goblin)]);
    let from = t.asked().len();
    enter_together(&mut t, &[thrinax, fresh]);
    assert_eq!(devour_offers(&t, from), vec![vec![Entity::Object(goblin)]]);
    let thrinax = t.named_on_battlefield("Bloodspore Thrinax")[0];
    assert_eq!(t.counters(thrinax, counters::PLUS1), 1);
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
    // A creature entering afterward gets the additional counter.
    let giant = t.enter(P0, "Hill Giant");
    assert_eq!(t.counters(giant, counters::PLUS1), 1);
}

#[test]
fn feasting_hobbit_devours_food_as_it_resolves() {
    cr!("702.82a", "702.82c", "608.3");
    ruling!(
        "Feasting Hobbit",
        "If you cast this spell, you choose how many and which Foods to devour as part of the resolution of the spell. (It can't be countered at that point.)"
    );
    supported("Feasting Hobbit");
    let mut t = TestGame::new(2);
    let food = create_token(&mut t, P0, "Food");
    let card = t.hand(P0, "Feasting Hobbit");
    give_mana_for(&mut t, P0, "Feasting Hobbit");
    t.answer_choose(P0, &[Entity::Object(food)]);
    let from = t.asked().len();
    let spell = t.cast(P0, card).go();
    t.settle();
    // Nothing chosen while casting.
    assert!(devour_offers(&t, from).is_empty());
    assert!(t.on_battlefield(food));
    t.resolve();
    assert_eq!(devour_offers(&t, from), vec![vec![Entity::Object(food)]]);
    let hobbit = t.g.current(spell);
    assert!(t.on_battlefield(hobbit));
    assert_eq!(t.counters(hobbit, counters::PLUS1), 3);
    assert!(!t.g.is_live(food));
}

#[test]
fn preyseizer_dragon_counts_all_its_counters_as_the_ability_resolves() {
    cr!("608.2h", "702.82a");
    ruling!(
        "Preyseizer Dragon",
        "Preyseizer Dragon's ability counts the number of +1/+1 counters on it when the ability resolves. It counts any +1/+1 counters, not just ones put on it due to devour."
    );
    supported("Preyseizer Dragon");
    let mut t = TestGame::new(2);
    // Devours one creature: two counters.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let dragon = t.enter(P0, "Preyseizer Dragon");
    t.settle();
    t.g.objects[dragon.0 as usize].summoning_sick = false;
    assert_eq!(t.counters(dragon, counters::PLUS1), 2);
    to_beginning_of_combat(&mut t, P0);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    attack_with(&mut t, &[(dragon, Entity::Player(P1))]);
    // Another counter (from another source) before the trigger resolves.
    t.g.add_counters(Entity::Object(dragon), counters::PLUS1, 1, None);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn mycoloth_counts_all_its_counters() {
    cr!("608.2h", "702.82a");
    ruling!(
        "Mycoloth",
        "The number of Saproling tokens created by the triggered ability is based on the number of +1/+1 counters on Mycoloth, not on the number of creatures Mycoloth devoured. It doesn't matter where the +1/+1 counters came from."
    );
    supported("Mycoloth");
    let mut t = TestGame::new(2);
    // Devoured one creature (two counters), then got a third counter elsewhere.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let myco = t.enter(P0, "Mycoloth");
    t.settle();
    t.g.add_counters(Entity::Object(myco), counters::PLUS1, 1, None);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Saproling").len(), 3);
}
