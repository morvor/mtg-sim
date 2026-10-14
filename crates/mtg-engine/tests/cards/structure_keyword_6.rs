//! Structure-coverage tests (round 6): keyword structures shared by many cards that no
//! other test exercised (see `docs/STRUCTURE_COVERAGE.md`).

use super::structure_seed_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn plainswalk_is_unblockable_while_the_defender_controls_a_plains() {
    cr!("702.14c", "509.1b");
    // Zodiac Rooster: "Plainswalk"
    supported("Zodiac Rooster");
    // (defender's land, P0's land): a nonbasic Plains (Scrubland) counts too.
    for (defender_land, my_land) in [
        (Some("Plains"), None),
        (Some("Scrubland"), None),
        (None, Some("Plains")),
        (None, None),
    ] {
        let defender_plains = defender_land.is_some();
        let mut t = TestGame::new(2);
        let r = t.battlefield(P0, "Zodiac Rooster");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let wall = t.battlefield(P1, "Wall of Stone");
        t.battlefield(P1, "Island");
        if let Some(l) = defender_land {
            t.battlefield(P1, l);
        }
        if let Some(l) = my_land {
            t.battlefield(P0, l);
        }
        t.set_step(P0, Step::BeginningOfCombat);
        declare(&mut t, &[(r, Entity::Player(P1)), (bears, Entity::Player(P1))]);
        go_to(&mut t, Step::DeclareAttackers);
        assert_eq!(
            t.g.can_block(wall, r),
            !defender_plains,
            "{defender_land:?} / {my_land:?}"
        );
        assert!(t.g.can_block(wall, bears), "no landwalk");
    }
}

#[test]
fn miracle_with_x_casts_for_the_miracle_cost() {
    cr!("702.94a", "107.3a");
    // Entreat the Angels: "Create X 4/4 white Angel creature tokens with flying." /
    // "Miracle {X}{W}{W}"
    supported("Entreat the Angels");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Entreat the Angels");
    t.lands(P0, "Plains", 4);
    t.answer_yes(P0, true); // reveal
    t.answer_yes(P0, true); // cast
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(2));
    t.g.draw_cards(P0, 1);
    t.settle();
    t.resolve();
    let spell = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(spell).chars.name, "Entreat the Angels");
    assert_eq!(
        t.g.obj(spell).stack.as_ref().unwrap().cast.method,
        CastMethod::Keyword(KeywordKind::Miracle)
    );
    assert_eq!(untapped(&t), 0, "{{2}}{{W}}{{W}} paid, not {{4}}{{W}}{{W}}{{W}}");
    t.resolve_all();
    let angels = subtype_ids(&t, P0, "Angel");
    assert_eq!(angels.len(), 2, "X = 2");
    for a in angels {
        assert_eq!(t.pt(a), (4, 4));
        assert!(t.obj_now(a).has_keyword(KeywordKind::Flying));
    }
    assert!(t.in_graveyard(P0, "Entreat the Angels"));
    // Without enough mana for the miracle cost, it stays in hand.
    let mut t = TestGame::new(2);
    t.library_top(P0, "Entreat the Angels");
    t.lands(P0, "Plains", 1);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert_eq!(t.g.find_in_zone(Zone::Hand(P0), "Entreat the Angels").len(), 1);
    assert_eq!(count_subtype(&t, P0, "Angel"), 0);
}

/// The number of untapped Plains P0 controls.
fn untapped(t: &TestGame) -> usize {
    t.named_on_battlefield("Plains")
        .into_iter()
        .filter(|id| !t.obj_now(*id).tapped)
        .count()
}

#[test]
fn ward_with_mana_and_life_needs_both_paid() {
    cr!("702.21a", "118.3");
    // Gisa, the Hellraiser: "Ward—{2}, Pay 2 life."
    supported("Gisa, the Hellraiser");
    // (swamps, life, pays): the spell resolves only if both parts are paid.
    for (swamps, life, resolves) in [(4, 20, true), (2, 20, false), (4, 1, false)] {
        let mut t = TestGame::new(2);
        let gisa = t.battlefield(P0, "Gisa, the Hellraiser");
        t.g.player_mut(P1).life = life;
        t.lands(P1, "Swamp", swamps);
        let s = t.hand(P1, "Go for the Throat");
        t.cast(P1, s).target(gisa).go();
        t.answer_yes(P1, true);
        t.resolve_all();
        let case = format!("{swamps} Swamps, {life} life");
        assert_eq!(!t.on_battlefield(gisa), resolves, "{case}");
        assert!(t.in_graveyard(P1, "Go for the Throat"), "{case}");
        if resolves {
            assert_eq!(t.life(P1), life - 2, "{case}");
            assert!(
                t.named_on_battlefield("Swamp").iter().all(|s| t.obj_now(*s).tapped),
                "{case}: {{1}}{{B}} and {{2}} paid"
            );
        } else {
            assert_eq!(t.life(P1), life, "{case}: nothing paid");
        }
    }
}
