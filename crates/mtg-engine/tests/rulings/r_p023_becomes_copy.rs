//! Rulings batch P023 — permanents that become a copy of a creature: becoming a copy of a
//! token copies the original characteristics the effect that created it gave it and
//! doesn't make the permanent a token (CR 707.2, 111.4); becoming a copy of something
//! that's copying something else copies what it copies (CR 707.3).

use crate::r_p023_common::*;
use crate::r_s01_common::supported;
use crate::r_s18_common::lands_for;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Casts Fleeting Reflection: P0's Grizzly Bears becomes a copy of `what`.
fn fleeting_reflection(t: &mut TestGame, what: ObjectId) -> ObjectId {
    supported("Fleeting Reflection");
    let bears = t.battlefield(P0, "Grizzly Bears");
    lands_for(t, P0, "{1}{U}");
    let spell = t.hand(P0, "Fleeting Reflection");
    t.cast(P0, spell).target(bears).target(what).go();
    t.resolve_all();
    bears
}

#[test]
fn fleeting_reflection_copying_a_token() {
    cr!("707.2", "111.4", "611.2a");
    ruling!(
        "Fleeting Reflection",
        "If the copied creature is a token, the first target creature copies the original characteristics of that token as stated by the effect that created that token."
    );
    let mut t = TestGame::new(2);
    let wolf = dressed_wolf(&mut t, P1);
    let bears = fleeting_reflection(&mut t, wolf);
    assert_wolf(&t, bears, false, true);
}

#[test]
fn fleeting_reflection_copying_a_clone() {
    cr!("707.3", "611.2a");
    ruling!(
        "Fleeting Reflection",
        "If the copied creature is copying something else, then the first target creature becomes a copy of whatever that creature copied."
    );
    let mut t = TestGame::new(2);
    let clone = cloned_angel(&mut t, P1);
    let bears = fleeting_reflection(&mut t, clone);
    assert_angel(&t, bears, true);
    assert!(!t.obj_now(bears).is_token());
}

/// P0 casts Giant Growth on `what` with Muddle, the Ever-Changing on the battlefield;
/// Muddle's trigger targets `what`.
fn muddle_copies(t: &mut TestGame, what: ObjectId) -> ObjectId {
    supported("Muddle, the Ever-Changing");
    let muddle = t.battlefield(P0, "Muddle, the Ever-Changing");
    lands_for(t, P0, "{G}");
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(what).go();
    t.answer_targets(P0, &[Entity::Object(what)]);
    t.resolve_all();
    assert!(t.obj_now(muddle).has_keyword(KeywordKind::Myriad));
    assert!(!t.obj_now(muddle).is_token());
    muddle
}

#[test]
fn muddle_copying_a_token() {
    cr!("707.2", "111.4", "707.9a");
    ruling!(
        "Muddle, the Ever-Changing",
        "If the copied creature is a token, Muddle copies the original characteristics of that token as stated by the effect that created that token, with the stated exception."
    );
    let mut t = TestGame::new(2);
    let wolf = dressed_wolf(&mut t, P0);
    let muddle = muddle_copies(&mut t, wolf);
    assert_wolf(&t, muddle, false, true);
}

#[test]
fn muddle_copying_a_clone() {
    cr!("707.3", "707.9a");
    ruling!(
        "Muddle, the Ever-Changing",
        "If the copied creature is copying something else, then Muddle becomes a copy of whatever that creature copied, with the stated exception."
    );
    let mut t = TestGame::new(2);
    let clone = cloned_angel(&mut t, P0);
    let muddle = muddle_copies(&mut t, clone);
    assert_angel(&t, muddle, true);
}

/// P0's Silent Hallcreeper deals combat damage to P1, choosing its third mode (it
/// becomes a copy of `what`).
fn hallcreeper_copies(t: &mut TestGame, what: ObjectId) -> ObjectId {
    supported("Silent Hallcreeper");
    let creeper = t.battlefield(P0, "Silent Hallcreeper");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    t.answer_targets(P0, &[Entity::Object(what)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(creeper, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert!(!t.obj_now(creeper).is_token());
    creeper
}

#[test]
fn silent_hallcreeper_copying_a_token() {
    cr!("707.2", "111.4");
    ruling!(
        "Silent Hallcreeper",
        "If the copied creature is a token, Silent Hallcreeper copies the original characteristics of that token as stated by the effect that created that token."
    );
    let mut t = TestGame::new(2);
    let wolf = dressed_wolf(&mut t, P0);
    let creeper = hallcreeper_copies(&mut t, wolf);
    assert_wolf(&t, creeper, false, true);
}

#[test]
fn silent_hallcreeper_copying_a_clone() {
    cr!("707.3");
    ruling!(
        "Silent Hallcreeper",
        "If the copied creature is copying something else, then Silent Hallcreeper becomes a copy of whatever that creature copied."
    );
    let mut t = TestGame::new(2);
    let clone = cloned_angel(&mut t, P0);
    let creeper = hallcreeper_copies(&mut t, clone);
    assert_angel(&t, creeper, true);
}
