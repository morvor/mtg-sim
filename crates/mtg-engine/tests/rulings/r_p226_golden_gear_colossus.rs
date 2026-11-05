//! Rulings batch P226 — The Golden-Gear Colossus: "Whenever The Golden-Gear Colossus
//! enters or attacks, transform up to one other target double-faced artifact you
//! control." (CR 701.27a, 701.27d).

use crate::r_s01_common::*;
use crate::r_s17_common::{enter_transformed, face, name_of};
use mtg_engine::decision::Decision;
use mtg_engine::object::FaceState;
use mtg_engine::testing::*;
use mtg_engine::*;

const TETZIN: &str = "Tetzin, Gnome Champion // The Golden-Gear Colossus";

/// The Golden-Gear Colossus enters (put onto the battlefield transformed) with `artifact`
/// on P0's battlefield, targeting it. Returns the artifact after the trigger resolved.
fn colossus_targets(artifact: &str) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, artifact);
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(a)]);
    let colossus = enter_transformed(&mut t, P0, TETZIN);
    assert_eq!(name_of(&t, colossus), "The Golden-Gear Colossus");
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .expect("no target asked");
    // A double-faced artifact, modal or not, can be targeted.
    assert_eq!(offered, vec![Entity::Object(a)]);
    t.resolve_all();
    (t, a)
}

#[test]
fn golden_gear_colossus_can_target_pestilent_cauldron_but_it_doesnt_transform() {
    cr!("701.27a", "701.27d");
    // The ruling's reason ("only transforming double-faced cards and transforming tokens
    // can transform") predates the Marvel's Spider-Man CR update effective 2025-09-19,
    // which let modal double-faced cards transform (CR 701.27a, 712.3); its outcome for
    // Pestilent Cauldron holds because Cauldron's other face, Restorative Burst, is a
    // sorcery (CR 701.27d).
    ruling!(
        "Tetzin, Gnome Champion // The Golden-Gear Colossus",
        "Choosing an artifact represented by a modal double-faced card (Pestilent Cauldron from Strixhaven, for example) won't cause that permanent to transform."
    );
    supported(TETZIN);
    let (t, cauldron) = colossus_targets("Pestilent Cauldron // Restorative Burst");
    assert!(t.on_battlefield(cauldron));
    assert_eq!(name_of(&t, cauldron), "Pestilent Cauldron");
    assert_eq!(face(&t, cauldron), FaceState::Front);
    // A nonmodal double-faced artifact transforms.
    let (t, map) = colossus_targets("Treasure Map // Treasure Cove");
    assert_eq!(name_of(&t, map), "Treasure Cove");
    assert_eq!(face(&t, map), FaceState::Back);
}
