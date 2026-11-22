//! Referents of older Oracle wordings (cards4-paper-effects, family `referents`): "that
//! Sliver", "the creature", "each of those creatures with flying".

use crate::combat2_common::supported;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn spined_sliver_pumps_the_blocked_sliver_per_blocker() {
    cr!("509.3c");
    supported("Spined Sliver");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let spined = t.battlefield(P0, "Spined Sliver");
    // Another Sliver attacks; Spined Sliver stays home.
    let metallic = t.battlefield(P0, "Metallic Sliver");
    let a = t.battlefield(P1, "Llanowar Elves");
    let b = t.battlefield(P1, "Fyndhorn Elves");
    t.attack(
        &[(metallic, Entity::Player(P1))],
        &[(a, metallic), (b, metallic)],
    );
    // +1/+1 for each of the two blockers: 3/3, survives the 2 damage.
    assert!(t.on_battlefield(metallic));
    assert_eq!(t.pt(metallic), (3, 3));
    assert_eq!(t.pt(spined), (2, 2));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(metallic), (1, 1));
}

#[test]
fn dreams_of_the_dead_exiles_the_returned_creature_when_it_would_leave() {
    cr!("614.1a", "702.24a");
    ruling!(
        "Dreams of the Dead",
        "If a creature returned to the battlefield with Dreams of the Dead would leave the battlefield for any reason, it’s exiled instead"
    );
    supported("Dreams of the Dead");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let d = t.battlefield(P0, "Dreams of the Dead");
    t.lands(P0, "Island", 2);
    let lions = t.graveyard(P0, "Savannah Lions");
    t.activate(P0, d, 0, &[Entity::Object(lions)]).unwrap();
    t.resolve();
    let now = t.named_on_battlefield("Savannah Lions");
    assert_eq!(now.len(), 1);
    let lions = now[0];
    assert!(t
        .obj_now(lions)
        .chars
        .has_keyword(mtg_engine::keywords::KeywordKind::CumulativeUpkeep));
    t.g.destroy(lions, None);
    t.g.flush_events();
    t.settle();
    assert!(t.in_exile("Savannah Lions"));
    assert!(!t.in_graveyard(P0, "Savannah Lions"));
}

#[test]
fn winter_blast_damages_only_the_tapped_targets_with_flying() {
    cr!("608.2b");
    ruling!(
        "Winter Blast",
        "Checks if the creatures are Flying on resolution and not on announcement. It’s not a targeting restriction."
    );
    supported("Winter Blast");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let wb = t.hand(P0, "Winter Blast");
    t.lands(P0, "Forest", 3);
    let drake = t.battlefield(P1, "Wind Drake");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Hill Giant");
    t.cast(P0, wb)
        .x(2)
        .targets(&[Entity::Object(drake), Entity::Object(bears)])
        .go();
    t.resolve();
    // Both targets tapped; only the flyer took 2 damage (and died).
    assert!(t.obj_now(bears).tapped);
    assert_eq!(t.obj_now(bears).damage, 0);
    assert!(t.in_graveyard(P1, "Wind Drake"));
    // Not a target: untouched.
    assert!(!t.obj_now(other).tapped);
}
