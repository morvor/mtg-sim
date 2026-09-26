//! CR 701.60: suspect.

use crate::a701_028_071_common::*;
use mtg_engine::ability::*;
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::kwa::suspect_detain::{is_suspected, SUSPECTED};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn suspected(t: &TestGame, id: ObjectId) -> bool {
    is_suspected(&t.g, t.g.current(id))
}

fn suspect(t: &mut TestGame, id: ObjectId) {
    run(t, P0, None, ka(KeywordAction::Suspect, Sel::Target(0), 1), &[Entity::Object(id)]);
}

#[test]
fn a_suspected_creature_stays_suspected_until_it_leaves_or_an_effect_ends_it() {
    cr!("701.60a");
    ruling!(
        "Person of Interest",
        "It stays suspected until it leaves the battlefield or another effect causes it to no longer be suspected."
    );
    supported("Person of Interest");
    supported("Absolving Lammasu");
    // "When this creature enters, suspect it. Create a 2/2 white and blue Detective
    // creature token."
    let mut t = TestGame::new(2);
    let poi = t.enter(P0, "Person of Interest");
    t.resolve_all();
    assert!(suspected(&t, poi));
    assert_eq!(custom_events(&t, SUSPECTED), vec![(Some(P0), Some(poi), 0)]);
    // Absolving Lammasu: "When this creature enters, all suspected creatures are no longer
    // suspected."
    let other = t.battlefield(P1, "Grizzly Bears");
    suspect(&mut t, other);
    assert!(suspected(&t, other));
    t.enter(P1, "Absolving Lammasu");
    t.resolve_all();
    assert!(!suspected(&t, poi) && !suspected(&t, other));
    assert!(!t.obj(poi).has_keyword(KeywordKind::Menace));
    // Leaving the battlefield ends it: the new object isn't suspected.
    suspect(&mut t, poi);
    assert!(suspected(&t, poi));
    let x = t.g.move_object(poi, Zone::Exile, MoveCause::Effect, None).unwrap();
    let back = t
        .g
        .move_object(x, Zone::Battlefield, MoveCause::Effect, Some(P0))
        .unwrap();
    t.g.recompute();
    assert!(!suspected(&t, back));
    assert!(!t.obj(back).has_keyword(KeywordKind::Menace));
}

#[test]
fn suspected_is_a_designation_not_an_ability_or_copiable_value() {
    cr!("701.60b");
    ruling!(
        "Person of Interest",
        "Being suspected isn't a copiable value. If a permanent becomes a copy of a suspected creature, it won't be suspected."
    );
    ruling!(
        "Person of Interest",
        "If a suspected creature loses all abilities, it will lose menace and \"This creature can't block\", but it won't stop being suspected."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    suspect(&mut t, bears);
    // A copy of it isn't suspected.
    let giant = t.battlefield(P0, "Hill Giant");
    run(
        &mut t,
        P0,
        None,
        Effect::BecomeCopy {
            what: Sel::Target(0),
            of: Sel::All(Filter::Objects(vec![bears])),
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(giant)],
    );
    assert_eq!(t.obj(giant).chars.name.as_str(), "Grizzly Bears");
    assert!(!suspected(&t, giant));
    assert!(!t.obj(giant).has_keyword(KeywordKind::Menace));
    assert!(t.g.can_block_at_all(giant));
    // Losing all abilities: no menace, and it can block, but it's still suspected.
    run(
        &mut t,
        P1,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveAllAbilities],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(bears)],
    );
    assert!(suspected(&t, bears));
    assert!(!t.obj(bears).has_keyword(KeywordKind::Menace));
    assert!(t.g.can_block_at_all(bears));
    // Only permanents can be suspected.
    let card = t.hand(P0, "Grizzly Bears");
    suspect(&mut t, card);
    assert!(!t.obj(card).suspected);
}

#[test]
fn a_suspected_creature_has_menace_and_cant_block() {
    cr!("701.60c");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    assert!(t.g.can_block_at_all(bears));
    suspect(&mut t, bears);
    assert!(t.obj(bears).has_keyword(KeywordKind::Menace));
    assert!(!t.g.can_block_at_all(bears));
    // It can still attack; as an attacker it can't be blocked except by two or more.
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[(theirs, bears)]);
    assert_eq!(t.life(P1), 18);
    assert!(t.on_battlefield(bears));
}

#[test]
fn a_suspected_permanent_cant_become_suspected_again() {
    cr!("701.60d");
    ruling!(
        "Person of Interest",
        "If a creature is already suspected, suspecting it again won't have any effect."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    suspect(&mut t, bears);
    run(
        &mut t,
        P1,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveAllAbilities],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(bears)],
    );
    assert!(!t.obj(bears).has_keyword(KeywordKind::Menace));
    // Suspecting it again doesn't give it menace and "can't block" anew.
    suspect(&mut t, bears);
    assert!(!t.obj(bears).has_keyword(KeywordKind::Menace));
    assert!(t.g.can_block_at_all(bears));
    assert_eq!(custom_events(&t, SUSPECTED).len(), 1);
}

#[test]
fn suspecting_the_token_just_created() {
    cr!("701.60a", "701.60c");
    // Case of the Stashed Skeleton: "When this Case enters, create a 2/1 black Skeleton
    // creature token and suspect it."
    let mut t = TestGame::new(2);
    let case = t.enter(P0, "Case of the Stashed Skeleton");
    t.resolve_all();
    let skeletons: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Skeleton"))
        .map(|o| o.id)
        .collect();
    assert_eq!(skeletons.len(), 1);
    assert!(suspected(&t, skeletons[0]));
    assert!(t.obj(skeletons[0]).has_keyword(KeywordKind::Menace));
    assert!(!t.g.can_block_at_all(skeletons[0]));
    assert!(!suspected(&t, case));
}
