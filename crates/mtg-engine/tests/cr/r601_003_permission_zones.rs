//! A static permission to play cards from a zone covers only the cards in the zone it names
//! (CR 601.3, 305.1): the top card of *your* library (Future Sight), *your* graveyard
//! (Crucible of Worlds) or only its top card (Bösium Strip), *your* hand (Omniscience).
//! Another permission that lists cards elsewhere among those the player may play (Coram,
//! the Undertaker; Sen Triplets; Xanathar, Guild Kingpin: `effect_permitted_cards`)
//! doesn't widen it: each card is checked against the zone of each permission
//! (`permissions::in_permission_zone`). Nor does a permission cover anything for a player
//! it isn't for: "your" is its controller (CR 109.5), so another player's Crucible of
//! Worlds doesn't let P0 play a land from P0's graveyard, even one a permission of P0's
//! lists (`permissions::allowing` checks the player and the zone together). Written for
//! survivors of the November 2026 mutation run of `permissions.rs` (docs/MUTATION.md).

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Kills `permissions.rs:197:32` (`ZoneKind::Graveyard if perm.top_only` with `false`),
/// `196:32` (`ZoneKind::Graveyard if perm.terms.any_owner` with `true`) and `180:5`
/// (`in_permission_zone` returning `true`). Crucible of Worlds lists every card in P0's
/// graveyard among the cards P0 may play, so Shock, under Lightning Bolt, is checked
/// against the Strip's permission: the mutants read it as one for P0's whole graveyard, for
/// any graveyard and for any zone, and P0 casts Shock.
#[test]
fn bosium_strip_casts_only_the_top_card_while_crucible_of_worlds_lists_the_graveyard() {
    cr!("601.3");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Crucible of Worlds");
    let strip = t.battlefield(P0, "Bösium Strip");
    t.lands(P0, "Mountain", 5);
    let shock = t.graveyard(P0, "Shock");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.activate(P0, strip, 0, &[]).unwrap();
    t.resolve_all();
    // Shock isn't the top card, and Crucible of Worlds's permission is for lands.
    assert!(t
        .cast(P0, shock)
        .target(Entity::Player(P1))
        .try_go()
        .is_err());
    t.clear_answers();
    assert_eq!(t.zone(shock), Zone::Graveyard(P0));
    // Lightning Bolt, the top card, may be cast.
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

/// Kills `permissions.rs:196:32` (`ZoneKind::Graveyard if perm.terms.any_owner` with
/// `true`) and `180:5`. Coram's permissions, for cards in graveyards that were put there
/// from libraries this turn, list the cards in every graveyard among those P0 may play, so
/// P1's Mountain (not put there from a library) is checked against Crucible of Worlds's
/// permission too: the mutants read "your graveyard" (P0's, its controller's, CR 109.5) as
/// any graveyard (or any zone), and P0 plays P1's Mountain.
#[test]
fn crucible_of_worlds_doesnt_play_a_land_from_another_players_graveyard() {
    cr!("109.5", "305.1");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Coram, the Undertaker");
    t.battlefield(P0, "Crucible of Worlds");
    let theirs = t.graveyard(P1, "Mountain");
    let mine = t.graveyard(P0, "Mountain");
    assert!(t.play_land(P0, theirs).is_err());
    assert_eq!(t.zone(theirs), Zone::Graveyard(P1));
    // A land from P0's own graveyard: the land play was still available.
    t.play_land(P0, mine).unwrap();
    assert!(t.on_battlefield(mine));
}

/// Kills `permissions.rs:201:27` (`ZoneKind::Hand if perm.terms.any_owner` with `true`)
/// and `180:5`. Sen Triplets's permission, for the target opponent's hand, lists the cards
/// in every other player's hand among those P0 may play (its qualities pick out P1's), so
/// P2's Ornithopter is checked against Omniscience's permission too: the mutants read
/// "your hand" as any hand (or any zone), and P0 casts P2's Ornithopter.
#[test]
fn omniscience_doesnt_cast_spells_from_another_players_hand() {
    cr!("601.3");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Sen Triplets");
    t.battlefield(P0, "Omniscience");
    let targeted = t.hand(P1, "Ornithopter");
    let other = t.hand(P2, "Ornithopter");
    t.advance_to(P0, Step::Upkeep);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    t.advance_to(P0, Step::PrecombatMain);
    assert!(t.cast(P0, other).try_go().is_err());
    assert_eq!(t.zone(other), Zone::Hand(P2));
    // P1's Ornithopter, with Sen Triplets's permission.
    t.cast(P0, targeted).go();
    t.resolve_all();
    assert!(t.on_battlefield(targeted));
    assert_eq!(t.obj_now(targeted).controller, P0);
}

/// Kills `permissions.rs:184:30` (`ZoneKind::Library if perm.top_only &&
/// perm.terms.any_owner` with `true`; `184:44`, the `&&` as `||`, is the same mutant, every
/// library permission being for the top card) and `180:5`. Xanathar's permission, for the
/// top card of the target opponent's library, lists the top card of every library among
/// the cards P0 may play (its qualities pick out P1's), so P2's top card is checked against
/// Future Sight's permission too: the mutants read "the top of your library" as the top of
/// any library (or any zone), and P0 casts P2's top card.
#[test]
fn future_sight_doesnt_cast_the_top_card_of_another_players_library() {
    cr!("601.3");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Xanathar, Guild Kingpin");
    t.battlefield(P0, "Future Sight");
    t.advance_to(P0, Step::Upkeep);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    t.advance_to(P0, Step::PrecombatMain);
    let targeted = t.library_top(P1, "Ornithopter");
    let other = t.library_top(P2, "Ornithopter");
    assert!(t.cast(P0, other).try_go().is_err());
    assert_eq!(t.zone(other), Zone::Library(P2));
    // P1's top card, with Xanathar's permission.
    t.cast(P0, targeted).go();
    t.resolve_all();
    assert!(t.on_battlefield(targeted));
}

/// Kills `permissions.rs:279:53` (in `permissions::allowing`,
/// `!g.player_rel_matches(perm.who, p, &ctx) || !in_permission_zone(g, p, card, perm)` with
/// `&&`: a static permission is skipped only if it's neither for the player nor for a zone
/// holding the card). For the player's own permissions the mutant ignores the zone, as
/// `180:5` does, and the first assertion of each test above kills it; this test kills its
/// other half. Lurrus of the Dream-Den lists P0's graveyard among the cards P0 may play (and
/// allows no land), so P0's Forest is checked against P1's Crucible of Worlds too: that
/// permission is P1's, but its zone, "your graveyard" read for P0, holds the Forest, and the
/// mutant lets P0 play it.
#[test]
fn another_players_crucible_of_worlds_doesnt_play_a_land_from_your_graveyard() {
    cr!("109.5", "305.1");
    ruling!(
        "Lurrus of the Dream-Den",
        "Lurrus doesn't let you play lands from your graveyard."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lurrus of the Dream-Den");
    t.battlefield(P1, "Crucible of Worlds");
    let forest = t.graveyard(P0, "Forest");
    assert!(t.play_land(P0, forest).is_err());
    assert_eq!(t.zone(forest), Zone::Graveyard(P0));
    // With a Crucible of Worlds of P0's own, it can be played: the land play was still
    // available.
    t.battlefield(P0, "Crucible of Worlds");
    t.play_land(P0, forest).unwrap();
    assert!(t.on_battlefield(forest));
}

/// Regression test for the engine bug found triaging `permissions.rs:196:32`; it failed
/// before the fix (commit `c83fe93`, in `permissions.rs`: `in_permission_zone`'s
/// `ZoneKind::Graveyard` arm by `as_though::in_graveyard_for`, and `effect_permitted_cards`'s
/// also listing `as_though::other_graveyard_cards`). Shaman's Trance lets P0 play lands and
/// cast spells from other players' graveyards "as though those cards were in your
/// graveyard": for that purpose they're in P0's graveyard (CR 609.4), so Crucible of
/// Worlds's permission, "You may play lands from your graveyard", covers them, as the Trance
/// already did for flashback's (`kw/flashback.rs`).
#[test]
fn shamans_trance_lets_crucible_of_worlds_play_a_land_from_another_players_graveyard() {
    cr!("609.4", "305.1");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Crucible of Worlds");
    t.lands(P0, "Mountain", 3);
    let forest = t.graveyard(P1, "Forest");
    assert!(
        t.play_land(P0, forest).is_err(),
        "P1's graveyard isn't P0's"
    );
    let trance = t.hand(P0, "Shaman's Trance");
    t.cast(P0, trance).go();
    t.resolve_all();
    t.play_land(P0, forest)
        .expect("as though it were in P0's graveyard");
    let f = t.g.current(forest);
    assert!(t.on_battlefield(f));
    assert_eq!(t.obj_now(f).controller, P0);
    assert_eq!(t.obj_now(f).owner, P1);
}
