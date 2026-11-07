//! Rulings batch P074 — Ninja Teen's level 3 permission with a creature card that has
//! more than one sneak ability.

use crate::r_s01_common::supported;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

const SNEAK: CastMethod = CastMethod::Keyword(KeywordKind::Sneak);

#[test]
fn ninja_teen_either_sneak_ability_can_be_used_from_the_graveyard() {
    cr!("702.190a", "601.3", "601.2b");
    ruling!(
        "Ninja Teen",
        "If a creature card in your graveyard has multiple sneak abilities, you can cast it from your graveyard using any of those sneak abilities"
    );
    supported("Ninja Teen");
    supported("Splinter, Hamato Yoshi");
    // Splinter, Hamato Yoshi has its own "Sneak {B}"; Ninja Teen gives it sneak {3}{B}.
    let mut paid = vec![];
    for way in ["first", "second"] {
        let mut t = TestGame::new(2);
        let teen = t.battlefield(P0, "Ninja Teen");
        t.lands(P0, "Swamp", 3);
        t.activate(P0, teen, 0, &[]).unwrap();
        t.resolve();
        t.activate(P0, teen, 1, &[]).unwrap();
        t.resolve();
        assert_eq!(t.obj_now(teen).class_level, 3);
        let fresh = t.lands(P0, "Swamp", 4);
        let splinter = t.graveyard(P0, "Splinter, Hamato Yoshi");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.answer(
            P0,
            DecisionKind::Attackers,
            Answer::Attackers(vec![(bears, Entity::Player(P1))]),
        );
        t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
        t.set_step(P0, Step::BeginningOfCombat);
        t.advance_to(P0, Step::DeclareBlockers);
        t.settle();
        let from = t.asked().len();
        let pick = if way == "second" { 1 } else { 0 };
        t.answer(P0, DecisionKind::Option, Answer::Index(pick));
        let r = t.cast(P0, splinter).method(SNEAK).try_go();
        assert!(r.is_ok(), "{way}: {r:?}");
        let offered = t.asked()[from..]
            .iter()
            .find_map(|(_, d)| match d {
                Decision::ChooseCastingMethod { options, .. } => Some(options.clone()),
                _ => None,
            })
            .expect("a choice between the two sneak abilities");
        assert_eq!(offered.len(), 2, "{offered:?}");
        let tapped = fresh.iter().filter(|l| t.obj_now(**l).tapped).count();
        paid.push(tapped);
        t.resolve_all();
        let s = t.named_on_battlefield("Splinter, Hamato Yoshi");
        assert_eq!(s.len(), 1);
        assert!(t.obj_now(s[0]).is(CardType::Creature));
        assert!(t.in_hand(P0, "Grizzly Bears"));
    }
    // One way paid {B}, the other {3}{B}.
    paid.sort();
    assert_eq!(paid, vec![1, 4]);
}
