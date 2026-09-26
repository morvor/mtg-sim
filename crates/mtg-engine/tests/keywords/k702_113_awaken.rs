//! CR 702.113 Awaken.

use crate::common_k702_111_124::*;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const AWAKEN: CastMethod = CastMethod::Keyword(KeywordKind::Awaken);

fn target_decisions(t: &TestGame) -> usize {
    asked_of(t, P0, |d| matches!(d, Decision::ChooseTargets { .. }))
}

#[test]
fn awaken_animates_target_land_you_control() {
    cr!("702.113", "702.113a");
    ruling!(
        "Coastal Discovery",
        "You can cast a spell with awaken for its mana cost and get only its first effect. If you cast a spell for its awaken cost, you’ll get both effects."
    );
    assert_supported_card("Coastal Discovery");
    let mut t = TestGame::new(2);
    // Coastal Discovery: {3}{U} sorcery, "Draw two cards.", awaken 4—{5}{U}.
    let lands = t.lands(P0, "Island", 6);
    let land = t.battlefield(P0, "Forest");
    let c = t.hand(P0, "Coastal Discovery");
    let hand = t.hand_size(P0);
    let spell = t.cast(P0, c).method(AWAKEN).target(land).go();
    assert!(lands.iter().all(|l| t.obj(*l).tapped));
    assert!(!t.obj(land).tapped);
    // Its mana value is still that of its mana cost.
    assert_eq!(t.g.mana_value_of(spell), 4);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
    let o = t.obj_now(land);
    assert!(o.is(CardType::Land) && o.is(CardType::Creature));
    assert!(o.chars.has_subtype("Elemental") && o.chars.has_subtype("Forest"));
    assert!(o.chars.has_keyword(KeywordKind::Haste));
    assert_eq!(t.counters(land, counters::PLUS1), 4);
    assert_eq!(t.pt(land), (4, 4));
    // The effect doesn't end.
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    assert!(t.obj_now(land).is(CardType::Creature));
}

#[test]
fn the_awakened_land_keeps_its_types_and_gets_no_color() {
    cr!("702.113a");
    ruling!(
        "Coastal Discovery",
        "Awaken doesn’t give the land you control a color."
    );
    ruling!(
        "Coastal Discovery",
        "The land will retain any other types, subtypes, or supertypes it previously had. It will also retain any mana abilities it had as a result of those subtypes."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    let forest = t.battlefield(P0, "Forest");
    let c = t.hand(P0, "Coastal Discovery");
    t.cast(P0, c).method(AWAKEN).target(forest).go();
    t.resolve_all();
    let o = t.obj_now(forest);
    assert!(o.chars.colors.is_colorless());
    assert!(o.chars.supertypes.contains(Supertype::Basic));
    // It can still be tapped for {G}.
    let pool = t.g.player(P0).mana_pool.total();
    t.activate(P0, forest, 0, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.total(), pool + 1);
}

#[test]
fn cast_for_its_mana_cost_it_has_no_awaken_target() {
    cr!("702.113b");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let c = t.hand(P0, "Coastal Discovery");
    let hand = t.hand_size(P0);
    t.cast(P0, c).go();
    // No target was chosen.
    assert_eq!(target_decisions(&t), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t
        .g
        .permanents()
        .all(|o| !o.chars.is_land() || !o.is(CardType::Creature)));
}

#[test]
fn if_the_land_becomes_an_illegal_target_the_spell_doesnt_resolve() {
    cr!("702.113a", "702.113b");
    ruling!(
        "Coastal Discovery",
        "If the non-awaken part of the spell doesn’t require a target and you cast the spell for its awaken cost, then the spell won’t resolve if the target land you control becomes illegal before the spell resolves"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    let land = t.battlefield(P0, "Forest");
    let c = t.hand(P0, "Coastal Discovery");
    let hand = t.hand_size(P0);
    t.cast(P0, c).method(AWAKEN).target(land).go();
    // In response, the land is destroyed.
    t.g.destroy(land, None);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forest"));
    // Coastal Discovery didn't resolve: no cards were drawn.
    assert_eq!(t.hand_size(P0), hand - 1);
    assert!(t.in_graveyard(P0, "Coastal Discovery"));
}

#[test]
fn each_other_target_is_still_affected() {
    cr!("702.113a");
    ruling!(
        "Ruinous Path",
        "If a spell with awaken has multiple targets (including the land you control), and some but not all of those targets become illegal by the time the spell tries to resolve, the spell won’t affect the illegal targets in any way."
    );
    assert_supported_card("Ruinous Path");
    let mut t = TestGame::new(2);
    // Ruinous Path: "Destroy target creature or planeswalker.", awaken 4—{5}{B}{B}.
    t.lands(P0, "Swamp", 7);
    let land = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let c = t.hand(P0, "Ruinous Path");
    t.cast(P0, c)
        .method(AWAKEN)
        .target(bears)
        .target(land)
        .go();
    // The creature leaves the battlefield in response.
    let owner = t.g.obj(bears).owner;
    t.g.move_object(
        bears,
        mtg_engine::object::Zone::Hand(owner),
        events::MoveCause::Effect,
        None,
    );
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert_eq!(t.pt(land), (4, 4));
}

#[test]
fn the_non_awaken_target_is_still_required() {
    cr!("702.113a");
    ruling!(
        "Ruinous Path",
        "If the non-awaken part of the spell requires a target, you must choose a legal target."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 7);
    let c = t.hand(P0, "Ruinous Path");
    // No creature or planeswalker to destroy.
    assert!(!castable(&mut t, P0, c, AWAKEN));
    assert!(!castable(&mut t, P0, c, CastMethod::Normal));
    t.battlefield(P1, "Grizzly Bears");
    assert!(castable(&mut t, P0, c, AWAKEN));
}
