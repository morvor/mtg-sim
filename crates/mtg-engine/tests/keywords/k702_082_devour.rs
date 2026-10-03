//! CR 702.82 Devour.

use crate::common_k702_011_017::{assert_supported, give_mana_for};
use crate::common_k702_052_066::{remove_counters, run_effect};
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// The candidates offered by the most recent devour choice.
fn devour_candidates(t: &TestGame) -> Vec<Entity> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if prompt.contains("devour") => Some(candidates),
            _ => None,
        })
        .unwrap_or_default()
}

/// Casts the creature card `name` for `p`, devouring `devour`, and resolves everything.
fn cast_devouring(t: &mut TestGame, p: PlayerId, name: &str, devour: &[ObjectId]) -> ObjectId {
    give_mana_for(t, p, name);
    let card = t.hand(p, name);
    let es: Vec<Entity> = devour.iter().map(|o| Entity::Object(*o)).collect();
    t.answer_choose(p, &es);
    t.cast(p, card).go();
    t.resolve_all();
    t.named_on_battlefield(name)
        .into_iter()
        .find(|o| t.g.obj(*o).controller == p)
        .expect("the devouring creature")
}

#[test]
fn devour_sacrifices_creatures_as_it_enters_for_counters() {
    cr!("702.82", "702.82a");
    assert_supported("Thorn-Thrash Viashino");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    // Devour 2.
    let v = cast_devouring(&mut t, P0, "Thorn-Thrash Viashino", &[a, b]);
    // Only creatures its controller controls could be devoured.
    let cands = devour_candidates(&t);
    assert!(cands.contains(&Entity::Object(a)) && cands.contains(&Entity::Object(b)));
    assert!(!cands.contains(&Entity::Object(theirs)));
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    assert_eq!(t.counters(v, counters::PLUS1), 4);
    assert_eq!(t.pt(v), (6, 6));
}

#[test]
fn devouring_nothing_is_allowed() {
    cr!("702.82a");
    ruling!(
        "Thorn-Thrash Viashino",
        "You may choose not to sacrifice any creatures for the Devour ability."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let v = cast_devouring(&mut t, P0, "Thorn-Thrash Viashino", &[]);
    assert!(t.on_battlefield(a));
    assert_eq!(t.counters(v, counters::PLUS1), 0);
}

#[test]
fn creatures_are_devoured_as_the_spell_resolves() {
    cr!("702.82a");
    ruling!(
        "Predator Dragon",
        "If you cast this as a spell, you choose how many and which creatures to devour as part of the resolution of that spell."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Thorn-Thrash Viashino");
    let card = t.hand(P0, "Thorn-Thrash Viashino");
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.cast(P0, card).go();
    // Nothing is sacrificed while the spell is on the stack.
    assert!(t.on_battlefield(a));
    t.resolve();
    assert!(!t.on_battlefield(a));
    let v = t.named_on_battlefield("Thorn-Thrash Viashino")[0];
    assert_eq!(t.counters(v, counters::PLUS1), 2);
}

#[test]
fn a_creature_entering_at_the_same_time_cant_be_devoured() {
    cr!("702.82a", "614.12a");
    ruling!(
        "Thorn-Thrash Viashino",
        "You may sacrifice only creatures that are already on the battlefield."
    );
    ruling!(
        "Predator Dragon",
        "the creature with devour can't devour that other creature. The creature with devour also can't devour itself."
    );
    ruling!(
        "Skullmulcher",
        "Because devour applies as Skullmulcher enters the battlefield, it can't devour creatures that enter the battlefield at the same time as it."
    );
    let mut t = TestGame::new(2);
    let old = t.battlefield(P0, "Grizzly Bears");
    let mulcher = t.hand(P0, "Gorger Wurm");
    let buddy = t.hand(P0, "Llanowar Elves");
    t.answer_choose(P0, &[Entity::Object(old)]);
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::battlefield(),
        },
        &[Entity::Object(mulcher), Entity::Object(buddy)],
    );
    let cands = devour_candidates(&t);
    assert_eq!(cands, vec![Entity::Object(old)]);
    let wurm = t.named_on_battlefield("Gorger Wurm")[0];
    assert_eq!(t.counters(wurm, counters::PLUS1), 1);
    assert!(t.on_battlefield(buddy));
}

#[test]
fn abilities_can_refer_to_the_creatures_it_devoured() {
    cr!("702.82b");
    assert_supported("Skullmulcher");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    // "When this creature enters, draw a card for each creature it devoured."
    let m = cast_devouring(&mut t, P0, "Skullmulcher", &[a, b]);
    assert_eq!(t.counters(m, counters::PLUS1), 2);
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn it_devoured_counts_the_devoured_permanents_with_a_quality() {
    cr!("702.82b");
    assert_supported("Voracious Dragon");
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P0, "Raging Goblin");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // "When this creature enters, it deals damage to any target equal to twice the number
    // of Goblins it devoured."
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let d = cast_devouring(&mut t, P0, "Voracious Dragon", &[goblin, bears]);
    assert_eq!(t.counters(d, counters::PLUS1), 2);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn it_devoured_a_creature_doesnt_depend_on_its_counters() {
    cr!("702.82b");
    ruling!(
        "Hellkite Hatchling",
        "It retains those abilities even if its +1/+1 counters are somehow removed."
    );
    assert_supported("Hellkite Hatchling");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let h = cast_devouring(&mut t, P0, "Hellkite Hatchling", &[a]);
    assert!(t.obj_now(h).chars.has_keyword(KeywordKind::Flying));
    assert!(t.obj_now(h).chars.has_keyword(KeywordKind::Trample));
    remove_counters(&mut t, h, counters::PLUS1, 1);
    assert_eq!(t.counters(h, counters::PLUS1), 0);
    assert!(t.obj_now(h).chars.has_keyword(KeywordKind::Flying));
    // One that devoured nothing doesn't have them.
    let other = t.enter(P1, "Hellkite Hatchling");
    assert!(!t.obj_now(other).chars.has_keyword(KeywordKind::Flying));
}

#[test]
fn devour_quality_sacrifices_permanents_with_that_quality() {
    cr!("702.82c");
    ruling!(
        "Caprichrome",
        "If you cast this as a spell, you choose how many and which artifacts to devour as part of the resolution of the spell."
    );
    ruling!(
        "Caprichrome",
        "It allows you to sacrifice artifacts rather than creatures, but otherwise functions identically to devour."
    );
    assert_supported("Caprichrome");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let rock = t.battlefield(P0, "Mind Stone");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Devour artifact 1.
    let c = cast_devouring(&mut t, P0, "Caprichrome", &[thopter, rock]);
    let cands = devour_candidates(&t);
    assert!(cands.contains(&Entity::Object(rock)));
    assert!(!cands.contains(&Entity::Object(bears)));
    assert_eq!(t.counters(c, counters::PLUS1), 2);
    assert!(t.on_battlefield(bears));
}

#[test]
fn devour_food_sacrifices_foods() {
    cr!("702.82c");
    ruling!(
        "Feasting Hobbit",
        "Devour Food is a variant of the devour ability. It allows you to sacrifice Foods rather than creatures, but otherwise functions identically to devour."
    );
    assert_supported("Feasting Hobbit");
    let mut t = TestGame::new(2);
    run_effect(
        &mut t,
        None,
        P0,
        Effect::CreateToken {
            spec: mtg_engine::tokens::predefined("Food").expect("Food"),
            count: Value::c(2),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
        },
        &[],
    );
    let foods: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|o| o.chars.has_subtype("Food"))
        .map(|o| o.id)
        .collect();
    assert_eq!(foods.len(), 2);
    // Devour Food 3: six +1/+1 counters.
    let h = cast_devouring(&mut t, P0, "Feasting Hobbit", &foods);
    assert_eq!(t.counters(h, counters::PLUS1), 6);
}

#[test]
fn devour_x_squares_the_number_of_creatures_devoured() {
    cr!("702.82a");
    ruling!(
        "Thromok the Insatiable",
        "Devouring three creatures will produce nine +1/+1 counters"
    );
    assert_supported("Thromok the Insatiable");
    let mut t = TestGame::new(2);
    let v: Vec<ObjectId> = (0..3)
        .map(|_| t.battlefield(P0, "Grizzly Bears"))
        .collect();
    let th = cast_devouring(&mut t, P0, "Thromok the Insatiable", &v);
    assert_eq!(t.counters(th, counters::PLUS1), 9);
}

#[test]
fn devouring_no_artifacts_is_allowed() {
    cr!("702.82c");
    ruling!(
        "Caprichrome",
        "You may choose not to sacrifice any artifacts for the devour artifact ability."
    );
    ruling!(
        "Feasting Hobbit",
        "You may choose not to sacrifice any Foods for the devour Food ability."
    );
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let c = cast_devouring(&mut t, P0, "Caprichrome", &[]);
    assert!(t.on_battlefield(thopter));
    assert_eq!(t.counters(c, counters::PLUS1), 0);
}

#[test]
fn devour_food_can_sacrifice_a_food_that_isnt_a_token() {
    cr!("702.82c");
    ruling!(
        "Feasting Hobbit",
        "If an effect refers to a Food, it means any Food artifact, not just a Food artifact token."
    );
    let mut t = TestGame::new(2);
    // Tough Cookie: an Artifact Creature — Food Golem.
    let cookie = t.battlefield(P0, "Tough Cookie");
    let h = cast_devouring(&mut t, P0, "Feasting Hobbit", &[cookie]);
    assert!(devour_candidates(&t).contains(&Entity::Object(cookie)));
    assert!(t.in_graveyard(P0, "Tough Cookie"));
    assert_eq!(t.counters(h, counters::PLUS1), 3);
}

#[test]
fn permanents_entering_at_the_same_time_cant_devour_each_other_or_the_same_objects() {
    cr!("702.82a", "702.82c");
    ruling!(
        "Caprichrome",
        "they can't devour the same objects. They can't devour each other, themselves, or any other objects entering the battlefield at the same time."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ornithopter");
    let b = t.battlefield(P0, "Ornithopter");
    let c1 = t.hand(P0, "Caprichrome");
    let c2 = t.hand(P0, "Caprichrome");
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.answer_choose(P0, &[Entity::Object(b)]);
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::battlefield(),
        },
        &[Entity::Object(c1), Entity::Object(c2)],
    );
    // The second one could devour only the Ornithopter the first one didn't.
    assert_eq!(devour_candidates(&t), vec![Entity::Object(b)]);
    let both = t.named_on_battlefield("Caprichrome");
    assert_eq!(both.len(), 2);
    for c in both {
        assert_eq!(t.counters(c, counters::PLUS1), 1);
    }
}

/// `what` becomes a copy of `of` (as an effect would).
fn become_copy(t: &mut TestGame, what: ObjectId, of: ObjectId) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(what)], vec![Entity::Object(of)]];
    t.g.exec(
        &Effect::BecomeCopy {
            what: Sel::Target(0),
            of: Sel::Target(1),
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
}

#[test]
fn a_copy_checks_what_it_devoured_itself() {
    cr!("702.82b");
    ruling!(
        "Hellkite Hatchling",
        "the second ability checks to see whether that creature — not the original Hellkite Hatchling — devoured a creature as it entered"
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let hatchling = cast_devouring(&mut t, P0, "Hellkite Hatchling", &[a]);
    assert!(t.obj_now(hatchling).chars.has_keyword(KeywordKind::Flying));
    // A creature that devoured nothing becomes a copy: no flying.
    let bears = t.battlefield(P0, "Grizzly Bears");
    become_copy(&mut t, bears, hatchling);
    assert_eq!(t.obj_now(bears).chars.name, "Hellkite Hatchling");
    assert!(!t.obj_now(bears).chars.has_keyword(KeywordKind::Flying));
    // One that devoured a creature does.
    let b = t.battlefield(P0, "Grizzly Bears");
    let wurm = cast_devouring(&mut t, P0, "Gorger Wurm", &[b]);
    become_copy(&mut t, wurm, hatchling);
    assert!(t.obj_now(wurm).chars.has_keyword(KeywordKind::Flying));
}
