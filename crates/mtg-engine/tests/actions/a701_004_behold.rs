//! CR 701.4: behold.

use crate::a701_common::*;
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::reveal::is_revealed;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn behold_by_revealing_a_card_from_hand() {
    cr!("701.4", "701.4a");
    supported("Kinsbaile Aspirant");
    let mut t = TestGame::new(2);
    let plains = t.lands(P0, "Plains", 1);
    // "As an additional cost to cast this spell, behold a Kithkin or pay {2}."
    let kithkin = t.hand(P0, "Goldmeadow Harrier");
    let aspirant = t.hand(P0, "Kinsbaile Aspirant");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer_choose(P0, &[Entity::Object(kithkin)]);
    let s = t.cast(P0, aspirant).go();
    // Only {W} was paid; the Kithkin card was revealed and stays in hand.
    assert!(t.obj(plains[0]).tapped);
    assert!(t.g.is_live(kithkin));
    assert_eq!(t.obj(kithkin).zone, Zone::Hand(P0));
    assert!(is_revealed(&t.g, kithkin));
    t.resolve();
    assert!(!t.g.is_live(s) || t.zone(s) == Zone::Battlefield);
    assert_eq!(t.named_on_battlefield("Kinsbaile Aspirant").len(), 1);
}

#[test]
fn behold_by_choosing_a_permanent_you_control() {
    cr!("701.4a");
    supported("Kinsbaile Aspirant");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let kithkin = t.battlefield(P0, "Goldmeadow Harrier");
    // An opponent's Kithkin can't be beheld.
    let theirs = t.battlefield(P1, "Goldmeadow Harrier");
    let aspirant = t.hand(P0, "Kinsbaile Aspirant");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer_choose(P0, &[Entity::Object(theirs)]);
    t.cast(P0, aspirant).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Kinsbaile Aspirant").len(), 1);
    // The Kithkin permanent was chosen: it's still there, untapped.
    assert!(t.on_battlefield(kithkin));
    assert!(!t.obj(kithkin).tapped);
    let prompts: Vec<Vec<Entity>> = t
        .asked()
        .iter()
        .filter_map(|(p, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities {
                prompt, candidates, ..
            } if *p == P0 && prompt == "Behold" => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(prompts, vec![vec![Entity::Object(kithkin)]]);
}

#[test]
fn without_a_quality_object_you_cant_behold() {
    cr!("701.4a");
    supported("Kinsbaile Aspirant");
    // No Kithkin: the {2} must be paid instead.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    t.battlefield(P0, "Grizzly Bears");
    let aspirant = t.hand(P0, "Kinsbaile Aspirant");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    assert!(t.cast(P0, aspirant).try_go().is_err());
    let mut t = TestGame::new(2);
    let plains = t.lands(P0, "Plains", 3);
    let aspirant = t.hand(P0, "Kinsbaile Aspirant");
    t.cast(P0, aspirant).go();
    assert!(plains.iter().all(|p| t.obj(*p).tapped));
}

#[test]
fn what_was_beheld_is_decided_when_it_was_beheld() {
    cr!("701.4b");
    ruling!(
        "Osseous Exhale",
        "No matter what happens to that card or permanent after that, it was still beheld"
    );
    supported("Osseous Exhale");
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Shivan Dragon");
    let attacker = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, mtg_engine::turn::Step::BeginningOfCombat);
    attack_with(&mut t, &[(attacker, Entity::Player(P0))]);
    t.lands(P0, "Plains", 2);
    let exhale = t.hand(P0, "Osseous Exhale");
    // "As an additional cost to cast this spell, you may behold a Dragon." P0 chooses the
    // Shivan Dragon.
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Object(dragon)]);
    t.cast(P0, exhale).target(attacker).go();
    // The Dragon leaves before the spell resolves: a Dragon was still beheld.
    t.g.destroy(dragon, None);
    t.resolve();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P0), 22);
}

#[test]
fn if_nothing_was_beheld_the_rider_doesnt_happen() {
    cr!("701.4b");
    supported("Osseous Exhale");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shivan Dragon");
    let attacker = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, mtg_engine::turn::Step::BeginningOfCombat);
    attack_with(&mut t, &[(attacker, Entity::Player(P0))]);
    t.lands(P0, "Plains", 2);
    let exhale = t.hand(P0, "Osseous Exhale");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.cast(P0, exhale).target(attacker).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn a_land_that_enters_tapped_unless_you_behold() {
    cr!("701.4a");
    // (Its "Empower Jace 2" ability isn't supported; its entering ability is.)
    assert!(card("Theorist's Sanctum")
        .unsupported_text()
        .iter()
        .all(|u| !u.contains("behold")));
    let mut t = TestGame::new(2);
    t.hand(P0, "Jace Beleren");
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    let land = t.hand(P0, "Theorist's Sanctum");
    t.play_land(P0, land).unwrap();
    let land = t.g.current(land);
    assert!(!t.obj(land).tapped);
    // Without a Jace, it enters tapped.
    let mut t = TestGame::new(2);
    let land = t.hand(P0, "Theorist's Sanctum");
    t.play_land(P0, land).unwrap();
    let land = t.g.current(land);
    assert!(t.obj(land).tapped);
    let _ = Effect::Noop;
}
