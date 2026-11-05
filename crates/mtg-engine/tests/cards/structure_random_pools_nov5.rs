//! Structure-coverage tests for ability structures that only the random card pools of the
//! property and API tests happened to exercise: the verified-refresh-nov4 sweep made more
//! cards verified, which changed those pools, and these structures lost their only exercise
//! (item `verified-refresh-bugs`). Each test checks a real card's ability does what its text
//! says.

use super::structure_seed_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn vigilant(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    t.obj_now(id).has_keyword(KeywordKind::Vigilance)
}

#[test]
fn veil_of_assimilation_pumps_a_creature_as_it_or_another_artifact_you_control_enters() {
    cr!("603.6a", "611.2a");
    // "Whenever this artifact or another artifact you control enters, target creature you
    // control gets +1/+1 and gains vigilance until end of turn."
    supported("Veil of Assimilation");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    let veil = t.hand(P0, "Veil of Assimilation");
    t.cast(P0, veil).go();
    // It enters: the trigger's one possible target is P0's Grizzly Bears.
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
    assert!(vigilant(&mut t, bears));
    assert_eq!(t.pt(theirs), (2, 2));
    // Another artifact P0 controls enters: it triggers again.
    let thopter = t.hand(P0, "Ornithopter");
    t.cast(P0, thopter).go();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    // An artifact an opponent controls doesn't trigger it.
    t.enter(P1, "Memnite");
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.pt(bears), (4, 4));
    // Until end of turn.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(bears), (2, 2));
    assert!(!vigilant(&mut t, bears));
}
