//! CR 702.195 Storied (`src/kw/storied.rs`).

use crate::common_k702_178_195::*;
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Ori, Keeper of Songs ({2}{W} legendary 3/3): "Storied. As long as you have an enduring
/// story, Ori gets +1/+0 and has vigilance."
const ORI: &str = "Ori, Keeper of Songs";

/// Puts a card onto the battlefield (a real zone change) and processes the events.
fn enter(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.enter(p, name);
    t.g.flush_events();
    id
}

fn story(t: &TestGame, p: PlayerId) -> bool {
    mtg_engine::kw::storied::has_enduring_story(&t.g, p)
}

fn destroy(t: &mut TestGame, id: ObjectId) {
    run(
        t,
        P1,
        None,
        Effect::Destroy {
            what: Sel::Target(0),
            no_regen: false,
        },
        &[Entity::Object(id)],
    );
}

#[test]
fn storied_cards_compile() {
    assert_supported(&[
        ORI,
        "Fíli the Pathfinder",
        "Óin the Brave",
        "Thorin Oakenshield",
        "Bombur, Gentle Dreamer",
        "Dáin, Lord of the Iron Hills",
    ]);
}

#[test]
fn three_artifacts_sagas_or_legendaries_give_an_enduring_story() {
    cr!("702.195a", "702.195b");
    ruling!(
        "Balin, Loremaster",
        "Storied isn't a triggered ability and doesn't use the stack."
    );
    let mut t = TestGame::new(2);
    let ori = enter(&mut t, P0, ORI);
    enter(&mut t, P0, "Mind Stone");
    t.settle();
    // Ori (legendary) and one artifact: two.
    assert!(!story(&t, P0));
    assert_eq!(t.pt(ori), (3, 3));
    // A third (a Saga): the player has an enduring story at once, with nothing put on
    // the stack or resolved.
    enter(&mut t, P0, "History of Benalia");
    assert!(story(&t, P0));
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.pt(ori), (4, 3));
    assert!(t
        .obj(ori)
        .has_keyword(mtg_engine::keywords::KeywordKind::Vigilance));
    assert!(!story(&t, P1));
}

#[test]
fn an_enduring_story_lasts_for_the_rest_of_the_game() {
    cr!("702.195a", "702.195b");
    ruling!(
        "Balin, Loremaster",
        "Once you have an enduring story, you have it for the rest of the game, even if you lose control of some or all of your storied permanents."
    );
    let mut t = TestGame::new(2);
    let ori = enter(&mut t, P0, ORI);
    let a = enter(&mut t, P0, "Mind Stone");
    let b = enter(&mut t, P0, "Mind Stone");
    assert!(story(&t, P0));
    destroy(&mut t, a);
    destroy(&mut t, b);
    t.settle();
    assert!(story(&t, P0));
    assert_eq!(t.pt(ori), (4, 3));
    destroy(&mut t, ori);
    t.settle();
    assert!(story(&t, P0));
}

#[test]
fn only_with_a_storied_permanent() {
    cr!("702.195a");
    ruling!(
        "Balin, Loremaster",
        "If you control three legendary, Saga, and/or artifact permanents but don't control a permanent with storied, you don't get an enduring story."
    );
    let mut t = TestGame::new(2);
    enter(&mut t, P0, "Mind Stone");
    enter(&mut t, P0, "Mind Stone");
    enter(&mut t, P0, "Mind Stone");
    t.settle();
    assert!(!story(&t, P0));
    // An opponent's storied permanent doesn't count either.
    enter(&mut t, P1, ORI);
    t.settle();
    assert!(!story(&t, P0));
    assert!(!story(&t, P1));
    // Once one enters under its controller's control, they get it.
    enter(&mut t, P0, "Óin the Brave");
    assert!(story(&t, P0));
}

#[test]
fn each_permanent_counts_once() {
    cr!("702.195a");
    ruling!(
        "Balin, Loremaster",
        "A single permanent can only count once toward the three legendary, Saga, and/or artifact permanents needed to get an enduring story"
    );
    let mut t = TestGame::new(2);
    // Ori and a legendary artifact (Mox Amber): two permanents, though three qualities.
    enter(&mut t, P0, ORI);
    enter(&mut t, P0, "Mox Amber");
    t.settle();
    assert!(!story(&t, P0));
    enter(&mut t, P0, "History of Benalia");
    assert!(story(&t, P0));
}

#[test]
fn the_story_comes_before_the_third_permanent_leaves() {
    cr!("702.195a");
    ruling!(
        "Balin, Loremaster",
        "you get an enduring story before it leaves the battlefield"
    );
    // A second Ori enters: three legendary/artifact permanents, then the legend rule puts
    // one into the graveyard.
    let mut t = TestGame::new(2);
    enter(&mut t, P0, ORI);
    enter(&mut t, P0, "Mind Stone");
    enter(&mut t, P0, ORI);
    t.settle();
    assert_eq!(named(&t, ORI).len(), 1);
    assert!(story(&t, P0));
}

#[test]
fn any_number_of_players_may_have_one() {
    cr!("702.195b");
    let mut t = TestGame::new(2);
    for p in [P0, P1] {
        enter(&mut t, p, ORI);
        enter(&mut t, p, "Mind Stone");
        enter(&mut t, p, "Mind Stone");
    }
    assert!(story(&t, P0) && story(&t, P1));
}

#[test]
fn continuous_effects_are_reapplied_before_triggers_are_checked() {
    cr!("702.195c");
    // Fíli the Pathfinder (2/2): "Storied. As long as you have an enduring story,
    // creatures you control get +1/+1."
    let watcher = custom_card(
        "Strength Watcher",
        "{2}",
        "Artifact",
        None,
        "Whenever a creature you control with power 3 or greater enters, you gain 1 life.",
    );
    let mut t = TestGame::new(2);
    put(&mut t, P0, watcher, Zone::Battlefield);
    enter(&mut t, P0, "Mind Stone");
    // Fíli is the third: it has +1/+1 as it enters, as far as the trigger is concerned.
    let fili = enter(&mut t, P0, "Fíli the Pathfinder");
    assert!(story(&t, P0));
    assert_eq!(t.pt(fili), (3, 3));
    t.resolve_all();
    // Fíli's own trigger creates a 2/2 Dwarf, a 3/3 as it enters: 2 life.
    assert_eq!(t.life(P0), 22);
}
