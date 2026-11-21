//! Rulings batch P202 — awaken (CR 702.113): "If you cast this spell for [cost], also put N
//! +1/+1 counters on target land you control and it becomes a 0/0 Elemental creature with
//! haste. It's still a land."

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::add_mana;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const AWAKEN: CastMethod = CastMethod::Keyword(KeywordKind::Awaken);

#[test]
fn planar_outburst_for_its_mana_cost_only_destroys_creatures() {
    cr!("702.113a", "702.113b");
    ruling!(
        "Planar Outburst",
        "You can cast a spell with awaken for its mana cost and get only its first effect. If you cast a spell for its awaken cost, you'll get both effects."
    );
    supported("Planar Outburst");
    // Mana cost: only "Destroy all nonland creatures."
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let c = t.hand(P0, "Planar Outburst");
    t.cast(P0, c).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t
        .g
        .permanents()
        .all(|o| !o.is(CardType::Creature)));

    // Awaken cost: both effects; the awakened land survives (it's a land creature, and it
    // became a creature after the destruction anyway).
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 8);
    let land = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let c = t.hand(P0, "Planar Outburst");
    t.cast(P0, c).method(AWAKEN).target(land).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.obj_now(land).is(CardType::Creature));
    assert_eq!(t.pt(land), (4, 4));
}

#[test]
fn planar_outbursts_awakened_land_is_colorless() {
    cr!("702.113a", "105.2c");
    ruling!(
        "Planar Outburst",
        "Awaken doesn't give the land you control a color. As most lands are colorless, in most cases the resulting land creature will also be colorless."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 8);
    let land = t.battlefield(P0, "Mountain");
    let c = t.hand(P0, "Planar Outburst");
    t.cast(P0, c).method(AWAKEN).target(land).go();
    t.resolve_all();
    let o = t.obj_now(land);
    assert!(o.is(CardType::Creature) && o.is(CardType::Land));
    assert!(o.chars.colors.is_colorless());
}

#[test]
fn planar_outburst_doesnt_resolve_if_its_only_target_land_becomes_illegal() {
    cr!("702.113a", "608.2b");
    ruling!(
        "Planar Outburst",
        "If the non-awaken part of the spell doesn't require a target and you cast the spell for its awaken cost, then the spell won't resolve if the target land you control becomes illegal before the spell resolves"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 8);
    let land = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let c = t.hand(P0, "Planar Outburst");
    t.cast(P0, c).method(AWAKEN).target(land).go();
    destroy(&mut t, land);
    t.resolve_all();
    // No creatures were destroyed.
    assert!(t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Planar Outburst"));
}

#[test]
fn an_awaken_spell_with_two_targets_still_affects_the_legal_one() {
    cr!("702.113a", "608.2b");
    ruling!(
        "Planar Outburst",
        "If a spell with awaken has multiple targets (including the land you control), and some but not all of those targets become illegal by the time the spell tries to resolve, the spell won't affect the illegal targets in any way."
    );
    supported("Earthen Arms");
    // Earthen Arms: "Put two +1/+1 counters on target permanent." Awaken 4—{6}{G}.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 7);
    let land = t.battlefield(P0, "Plains");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let c = t.hand(P0, "Earthen Arms");
    t.cast(P0, c).method(AWAKEN).target(bears).target(land).go();
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.counters(land, counters::PLUS1), 4);
    assert_eq!(t.pt(land), (4, 4));

    // The other way round: the land becomes illegal, the permanent still gets counters.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 7);
    let land = t.battlefield(P0, "Plains");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let c = t.hand(P0, "Earthen Arms");
    t.cast(P0, c).method(AWAKEN).target(bears).target(land).go();
    destroy(&mut t, land);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn the_awaken_target_is_needed_only_when_cast_for_the_awaken_cost() {
    cr!("702.113a", "601.2c");
    ruling!(
        "Planar Outburst",
        "If the non-awaken part of the spell requires a target, you must choose a legal target. You can't cast the spell if you can't choose a legal target for each instance of the word “target” (though you only need a legal target for the awaken ability if you're casting the spell for its awaken cost)."
    );
    // With no land to target, Planar Outburst can be cast for its mana cost only.
    let mut t = TestGame::new(2);
    add_mana(&mut t, P0, ManaType::W, 8);
    let c = t.hand(P0, "Planar Outburst");
    assert!(can_cast(&mut t, P0, c, CastMethod::Normal));
    assert!(!can_cast(&mut t, P0, c, AWAKEN));
    t.battlefield(P0, "Plains");
    assert!(can_cast(&mut t, P0, c, AWAKEN));

    // Earthen Arms needs a target permanent either way.
    let mut t = TestGame::new(2);
    add_mana(&mut t, P0, ManaType::G, 7);
    let c = t.hand(P0, "Earthen Arms");
    assert!(!can_cast(&mut t, P0, c, CastMethod::Normal));
    assert!(!can_cast(&mut t, P0, c, AWAKEN));
    t.battlefield(P1, "Grizzly Bears");
    assert!(can_cast(&mut t, P0, c, CastMethod::Normal));
    assert!(!can_cast(&mut t, P0, c, AWAKEN));
}

#[test]
fn earthen_arms_can_target_the_same_land_twice() {
    cr!("702.113a", "115.3");
    ruling!(
        "Earthen Arms",
        "If you cast Earthen Arms for its awaken cost, a land you control can be both targets."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 7);
    let land = t.battlefield(P0, "Plains");
    let c = t.hand(P0, "Earthen Arms");
    t.cast(P0, c).method(AWAKEN).target(land).target(land).go();
    t.resolve_all();
    assert_eq!(t.counters(land, counters::PLUS1), 6);
    assert_eq!(t.pt(land), (6, 6));
}

#[test]
fn rising_miasma_shrinks_creatures_before_the_land_becomes_one() {
    cr!("702.113a", "608.2h", "611.2c");
    ruling!(
        "Rising Miasma",
        "If you cast Rising Miasma for its awaken cost, creatures will get -2/-2 before the land you targeted is turned into a creature."
    );
    supported("Rising Miasma");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 7);
    let land = t.battlefield(P0, "Forest");
    let giant = t.battlefield(P1, "Hill Giant");
    let c = t.hand(P0, "Rising Miasma");
    t.cast(P0, c).method(AWAKEN).target(land).go();
    t.resolve_all();
    assert_eq!(t.pt(giant), (1, 1));
    // The land wasn't a creature as the -2/-2 applied: it's a plain 3/3.
    assert_eq!(t.pt(land), (3, 3));

    // A land that already was a creature does get -2/-2.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 14);
    let land = t.battlefield(P0, "Forest");
    let c = t.hand(P0, "Rising Miasma");
    t.cast(P0, c).method(AWAKEN).target(land).go();
    t.resolve_all();
    assert_eq!(t.pt(land), (3, 3));
    let c = t.hand(P0, "Rising Miasma");
    t.cast(P0, c).method(AWAKEN).target(land).go();
    t.resolve_all();
    // 3 + 3 counters, -2/-2.
    assert_eq!(t.pt(land), (4, 4));
}

#[test]
fn part_the_waterveil_goes_to_the_graveyard_if_it_doesnt_resolve() {
    cr!("702.113a", "608.2b", "608.2n");
    ruling!(
        "Part the Waterveil",
        "Exiling Part the Waterveil is part of its effect. If Part the Waterveil doesn’t resolve, including if it was cast for its awaken cost and the land you control became an illegal target, it will be put into its owner’s graveyard."
    );
    supported("Part the Waterveil");
    // Resolving: it exiles itself and an extra turn is taken.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 9);
    let land = t.battlefield(P0, "Forest");
    let c = t.hand(P0, "Part the Waterveil");
    t.cast(P0, c).method(AWAKEN).target(land).go();
    t.resolve_all();
    assert!(t.in_exile("Part the Waterveil"));
    assert_eq!(t.g.extra_turns, vec![P0]);
    assert_eq!(t.pt(land), (6, 6));

    // The land becomes an illegal target: no extra turn, and it's put into the graveyard.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 9);
    let land = t.battlefield(P0, "Forest");
    let c = t.hand(P0, "Part the Waterveil");
    t.cast(P0, c).method(AWAKEN).target(land).go();
    destroy(&mut t, land);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Part the Waterveil"));
    assert!(!t.in_exile("Part the Waterveil"));
    assert!(t.g.extra_turns.is_empty());
}

#[test]
fn ondu_risings_delayed_trigger_covers_every_attacker_including_the_awakened_land() {
    cr!("702.113a", "603.7b", "702.15b");
    ruling!(
        "Ondu Rising",
        "Ondu Rising creates a delayed triggered ability. Any creature that attacks that turn will cause that ability to trigger, including creatures controlled by a teammate and creatures you didn’t control or that didn’t exist as Ondu Rising resolved. Notably, this includes the land creature created if Ondu Rising is cast for its awaken cost."
    );
    supported("Ondu Rising");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    let land = t.battlefield(P0, "Forest");
    let c = t.hand(P0, "Ondu Rising");
    t.cast(P0, c).method(AWAKEN).target(land).go();
    t.resolve_all();
    // A creature that didn't exist as it resolved.
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(
        &mut t,
        &[(land, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(t.obj_now(land).chars.has_keyword(KeywordKind::Lifelink));
    assert!(t.obj_now(bears).chars.has_keyword(KeywordKind::Lifelink));
    t.advance_to(P0, mtg_engine::turn::Step::EndOfCombat);
    assert_eq!(t.life(P1), 20 - 4 - 2);
    assert_eq!(t.life(P0), 20 + 4 + 2);
}
