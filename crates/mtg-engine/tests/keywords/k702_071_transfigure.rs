//! CR 702.71 Transfigure.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_027_037::activate_named;
use crate::common_k702_052_066::on_top;
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const TRANSFIGURE: &str = "Transfigure";

/// The cards offered by the most recent library search.
fn search_candidates(t: &TestGame) -> Vec<Entity> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if prompt.starts_with("Search") => Some(candidates),
            _ => None,
        })
        .unwrap_or_default()
}

#[test]
fn transfigure_sacrifices_it_to_put_a_creature_with_the_same_mana_value_onto_the_battlefield() {
    cr!("702.71", "702.71a");
    ruling!(
        "Fleshwrither",
        "it can fetch only a creature"
    );
    assert_supported("Fleshwrither");
    let mut t = TestGame::new(2);
    // Mana value 4: Hill Giant (creature), Control Magic (enchantment); Serra Angel is 5.
    let giant = on_top(&mut t, P0, "Hill Giant");
    let magic = on_top(&mut t, P0, "Control Magic");
    let angel = on_top(&mut t, P0, "Serra Angel");
    let writher = t.battlefield(P0, "Fleshwrither");
    t.lands(P0, "Swamp", 3);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    activate_named(&mut t, P0, writher, TRANSFIGURE, 0).unwrap();
    // Sacrificing it is part of the cost.
    assert!(t.in_graveyard(P0, "Fleshwrither"));
    t.resolve();
    assert_eq!(search_candidates(&t), vec![Entity::Object(giant)]);
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).controller, P0);
    assert_eq!(t.zone(magic), Zone::Library(P0));
    assert_eq!(t.zone(angel), Zone::Library(P0));
}

#[test]
fn transfigure_is_activated_only_as_a_sorcery_from_the_battlefield() {
    cr!("702.71a");
    ruling!(
        "Fleshwrither",
        "it can be activated only if the permanent with Transfigure is on the battlefield"
    );
    let mut t = TestGame::new(2);
    on_top(&mut t, P0, "Hill Giant");
    t.lands(P0, "Swamp", 3);
    let writher = t.battlefield(P0, "Fleshwrither");
    // Not from the hand.
    let in_hand = t.hand(P0, "Fleshwrither");
    assert!(activate_named(&mut t, P0, in_hand, TRANSFIGURE, 0).is_err());
    // Not during an opponent's turn, nor in combat, nor with a spell on the stack.
    t.set_step(P1, Step::PrecombatMain);
    assert!(activate_named(&mut t, P0, writher, TRANSFIGURE, 0).is_err());
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(activate_named(&mut t, P0, writher, TRANSFIGURE, 0).is_err());
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P1).go();
    assert!(activate_named(&mut t, P0, writher, TRANSFIGURE, 0).is_err());
    t.resolve_all();
    assert!(t.on_battlefield(writher));
    activate_named(&mut t, P0, writher, TRANSFIGURE, 0).unwrap();
    t.resolve();
    assert!(!t.named_on_battlefield("Hill Giant").is_empty());
}

#[test]
fn transfigure_shuffles_even_if_no_creature_is_found() {
    cr!("702.71a");
    let mut t = TestGame::new(2);
    let magic = on_top(&mut t, P0, "Control Magic");
    let writher = t.battlefield(P0, "Fleshwrither");
    t.lands(P0, "Swamp", 3);
    activate_named(&mut t, P0, writher, TRANSFIGURE, 0).unwrap();
    t.resolve();
    assert!(search_candidates(&t).is_empty());
    assert!(t.in_graveyard(P0, "Fleshwrither"));
    assert_eq!(t.zone(magic), Zone::Library(P0));
    // "Then shuffle your library."
    assert!(t
        .g
        .turn_events
        .iter()
        .any(|e| matches!(e, Event::Shuffled { player } if *player == P0)));
}
