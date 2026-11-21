//! Rulings batch P039 — historic (CR 700.6): an object with two historic qualities (a
//! legendary artifact) is historic once, for each way the cards use the word: casting a
//! historic spell, counting historic cards, targeting or discarding a historic card, a
//! historic permanent, and casting historic spells as though they had flash.

use crate::r_p039_common::*;
use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s02_common::{can_cast, target_candidates};
use crate::r_p039_creatureland::activate_text;
use mtg_engine::keywords::KeywordKind as Keyword;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// A legendary artifact: historic twice over.
const LEGENDARY_ARTIFACT: &str = "Weatherlight";

#[test]
fn surtr_triggers_once_for_a_legendary_artifact_spell() {
    cr!("700.6", "603.2");
    ruling!(
        "Surtr, Fiery Jötun",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn't make an object more historic than another or provide an additional bonus—an object either is historic or it isn't."
    );
    supported("Surtr, Fiery Jötun");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Surtr, Fiery Jötun");
    t.answer_targets(P0, &[pl(P1)]);
    cast_named(&mut t, P0, LEGENDARY_ARTIFACT, &[]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "3 damage"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // A nonhistoric spell: none.
    cast_named(&mut t, P0, "Grizzly Bears", &[]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "3 damage"), 0);
}

#[test]
fn havi_counts_each_historic_card_once() {
    cr!("700.6");
    ruling!(
        "Havi, the All-Father",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn't make an object more historic than another or provide an additional bonus—an object either is historic or it isn't."
    );
    supported("Havi, the All-Father");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Havi, the All-Father");
    for _ in 0..3 {
        t.graveyard(P0, LEGENDARY_ARTIFACT);
    }
    t.graveyard(P0, "Grizzly Bears");
    t.g.recompute();
    assert!(!t.obj_now(h).has_keyword(Keyword::Indestructible));
    t.graveyard(P0, "Ornithopter");
    t.g.recompute();
    assert!(t.obj_now(h).has_keyword(Keyword::Indestructible));
}

#[test]
fn rona_and_lost_to_legend_target_historic_objects() {
    cr!("700.6", "115.1");
    ruling!(
        "Rona, Disciple of Gix",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn't make an object more historic than another or provide an additional bonus—an object either is historic or it isn't."
    );
    ruling!(
        "Lost to Legend",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn't make an object more historic than another or provide an additional bonus—an object either is historic or it isn't."
    );
    // Rona: "When Rona enters, you may exile target historic card from your graveyard."
    let mut t = TestGame::new(2);
    let w = t.graveyard(P0, LEGENDARY_ARTIFACT);
    let saga = t.graveyard(P0, "The Raven's Warning");
    let b = t.graveyard(P0, "Grizzly Bears");
    let id = new_card(&mut t, P0, "Rona, Disciple of Gix");
    let from = mark(&t);
    t.answer_targets(P0, &[o(w)]);
    enter_obj(&mut t, P0, id);
    let c = target_candidates(&t, P0, from);
    assert_eq!(c.len(), 1);
    assert!(c[0].contains(&o(w)) && c[0].contains(&o(saga)) && !c[0].contains(&o(b)));
    t.resolve_all();
    assert_eq!(t.zone(w), Zone::Exile);
    // Lost to Legend: "Put target nonland historic permanent into its owner's library
    // fourth from the top."
    let mut t = TestGame::new(2);
    let w = t.battlefield(P1, LEGENDARY_ARTIFACT);
    let orn = t.battlefield(P1, "Ornithopter");
    let b = t.battlefield(P1, "Grizzly Bears");
    let from = mark(&t);
    cast_named(&mut t, P0, "Lost to Legend", &[o(w)]);
    let c = target_candidates(&t, P0, from);
    assert!(c[0].contains(&o(w)) && c[0].contains(&o(orn)) && !c[0].contains(&o(b)));
    t.resolve_all();
    assert_eq!(t.zone(w), Zone::Library(P1));
}

#[test]
fn raff_capashen_flashes_a_legendary_artifact() {
    cr!("700.6", "702.8a");
    ruling!(
        "Raff Capashen, Ship's Mage",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn't make an object more historic than another or provide an additional bonus—an object either is historic or it isn't."
    );
    supported("Raff Capashen, Ship's Mage");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raff Capashen, Ship's Mage");
    let w = t.hand(P0, LEGENDARY_ARTIFACT);
    let b = t.hand(P0, "Grizzly Bears");
    t.set_step(P1, Step::Upkeep);
    mana_all(&mut t, P0, 4);
    assert!(can_cast(&mut t, P0, w, CastMethod::Normal));
    assert!(!can_cast(&mut t, P0, b, CastMethod::Normal));
}

#[test]
fn sanctum_spirit_discards_a_legendary_artifact() {
    cr!("700.6", "118.3");
    ruling!(
        "Sanctum Spirit",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn't make an object more historic than another or provide an additional bonus—an object either is historic or it isn't."
    );
    supported("Sanctum Spirit");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sanctum Spirit");
    let w = t.hand(P0, LEGENDARY_ARTIFACT);
    let b = t.hand(P0, "Grizzly Bears");
    let from = mark(&t);
    t.answer_choose(P0, &[o(w)]);
    activate_text(&mut t, P0, s, "indestructible").unwrap();
    t.resolve_all();
    let cands: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities { candidates, .. } => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .collect();
    assert!(cands.iter().all(|c| !c.contains(&o(b))), "{cands:?}");
    assert_eq!(t.zone(w), Zone::Graveyard(P0));
    assert_eq!(t.zone(b), Zone::Hand(P0));
    assert!(t.obj_now(s).has_keyword(Keyword::Indestructible));
}
