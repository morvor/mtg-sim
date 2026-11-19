//! A permission to play cards from a zone puts only the cards in that zone among the cards
//! a player may play (`permissions::effect_permitted_cards`, read through
//! `Game::permitted_cards`): the top card of *your* library (Future Sight), the top card of
//! *your* graveyard (Bösium Strip), *your* hand (Omniscience). The list matters by itself for
//! a way of casting that doesn't ask the permissions again: mutate's way (`kw/mutate.rs`) is
//! offered for any card the list holds, and `Game::cast_options` keeps only the ways some
//! permission allows (`permissions::attach`) just for a card some permission's zone holds
//! (`permissions::may_be_permitted`). A card listed from outside the zone would be cast for
//! its mutate cost with no permission at all, which CR 601.3 doesn't allow. Written for
//! survivors of the November 2026 mutation run of `permissions.rs` (docs/MUTATION.md).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const MUTATE: CastMethod = CastMethod::Keyword(KeywordKind::Mutate);

/// How many ways P0 is offered to cast `card` for its mutate cost (`Game::cast_options`, the
/// ways the legal actions and casting choose among).
fn mutate_ways(t: &mut TestGame, card: ObjectId) -> usize {
    t.g.recompute();
    t.g.cast_options(P0, card)
        .iter()
        .filter(|o| o.method == MUTATE)
        .count()
}

/// Kills `permissions.rs:407:34` (in `effect_permitted_cards`, `ZoneKind::Library if
/// perm.top_only && perm.terms.any_owner` with `true`) and `407:48` (the `&&` as `||`: the
/// same mutant, every library permission being for the top card). Both list the top card of
/// every library for Future Sight's permission, so P1's Sea-Dasher Octopus is among the
/// cards P0 may play although no permission's zone holds it: `cast_options` returns its ways
/// unchecked, and P0 casts it for its mutate cost onto P1's Bears. (The test pairing Future
/// Sight with Xanathar, Guild Kingpin in `r601_003_permission_zones.rs` doesn't kill them:
/// Xanathar's zone, the top of any library, holds the card listed, so its ways are checked.)
#[test]
fn future_sight_doesnt_offer_mutate_for_the_top_card_of_another_players_library() {
    cr!("601.3", "702.140a", "118.9");
    ruling!(
        "Future Sight",
        "If the card has alternative or additional costs, you may pay those."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Future Sight");
    t.lands(P0, "Island", 2);
    let my_bears = t.battlefield(P0, "Grizzly Bears");
    let their_bears = t.battlefield(P1, "Grizzly Bears");
    let mine = t.library_top(P0, "Sea-Dasher Octopus");
    let theirs = t.library_top(P1, "Sea-Dasher Octopus");
    // P1's top card: no permission of P0's covers it, so it can't be cast for its mutate
    // cost onto P1's Bears (a creature with the same owner, the one it would target).
    assert_eq!(mutate_ways(&mut t, theirs), 0);
    assert!(t
        .cast(P0, theirs)
        .method(MUTATE)
        .target(their_bears)
        .try_go()
        .is_err());
    t.clear_answers();
    assert_eq!(t.zone(theirs), Zone::Library(P1));
    // P0's own top card can, with Future Sight's permission, for its mutate cost.
    let spell = t.cast(P0, mine).method(MUTATE).target(my_bears).go();
    assert_eq!(t.zone(spell), Zone::Stack);
    assert_eq!(t.obj_now(spell).controller, P0);
}

/// Kills `permissions.rs:424:36` (in `effect_permitted_cards`, `ZoneKind::Graveyard if
/// perm.top_only` with `false`) and `419:36` (`ZoneKind::Graveyard if perm.terms.any_owner`
/// with `true`). Bösium Strip's permission, for the top card of P0's graveyard, then lists
/// P0's whole graveyard (or every graveyard), so the Sea-Dasher Octopus under Lightning Bolt
/// is among the cards P0 may play although no permission's zone holds it: `cast_options`
/// returns its ways unchecked, and P0 casts it for its mutate cost. (The test pairing the
/// Strip with Crucible of Worlds in `r601_003_permission_zones.rs` doesn't kill them:
/// Crucible's zone, P0's graveyard, holds every card listed, so their ways are checked.)
#[test]
fn bosium_strip_doesnt_offer_mutate_for_a_card_under_the_top_of_your_graveyard() {
    cr!("601.3", "702.140a", "118.9");
    ruling!(
        "Muldrotha, the Gravetide",
        "You must pay the costs to cast a spell this way. If it has an alternative cost, you may cast it for that cost instead."
    );
    let mut t = TestGame::new(2);
    let strip = t.battlefield(P0, "Bösium Strip");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 5);
    let octopus = t.graveyard(P0, "Sea-Dasher Octopus");
    t.graveyard(P0, "Lightning Bolt");
    t.activate(P0, strip, 0, &[]).unwrap();
    t.resolve_all();
    // The Octopus isn't the top card: the Strip's permission doesn't cover it.
    assert_eq!(mutate_ways(&mut t, octopus), 0);
    assert!(t
        .cast(P0, octopus)
        .method(MUTATE)
        .target(bears)
        .try_go()
        .is_err());
    t.clear_answers();
    assert_eq!(t.zone(octopus), Zone::Graveyard(P0));
    // With Muldrotha's permission (a creature spell from P0's graveyard), it can be cast
    // for its mutate cost: two Islands are left after the Strip's {3}.
    t.battlefield(P0, "Muldrotha, the Gravetide");
    let spell = t.cast(P0, octopus).method(MUTATE).target(bears).go();
    assert_eq!(t.zone(spell), Zone::Stack);
    assert_eq!(t.obj_now(spell).controller, P0);
}

/// Kills `permissions.rs:429:31` (in `effect_permitted_cards`, `ZoneKind::Hand if
/// perm.terms.any_owner` with `true`). Omniscience's permission, for P0's hand, then lists
/// the cards in every other player's hand, so P1's Sea-Dasher Octopus is among the cards P0
/// may play although no permission's zone holds it: `cast_options` returns its ways
/// unchecked, and P0 casts it for its mutate cost onto P1's Bears. (The test pairing
/// Omniscience with Sen Triplets in `r601_003_permission_zones.rs` doesn't kill it: Sen
/// Triplets's zone, any hand, holds the card listed, so its ways are checked.)
#[test]
fn omniscience_doesnt_offer_mutate_for_a_card_in_another_players_hand() {
    cr!("601.3", "702.140a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    t.lands(P0, "Island", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let theirs = t.hand(P1, "Sea-Dasher Octopus");
    assert_eq!(mutate_ways(&mut t, theirs), 0);
    assert!(t
        .cast(P0, theirs)
        .method(MUTATE)
        .target(bears)
        .try_go()
        .is_err());
    t.clear_answers();
    assert_eq!(t.zone(theirs), Zone::Hand(P1));
    // With Sen Triplets's permission for P1's hand, P0 casts it for its mutate cost onto
    // P1's Bears: a mutating creature spell targets a creature with the same owner.
    t.battlefield(P0, "Sen Triplets");
    t.advance_to(P0, Step::Upkeep);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    t.advance_to(P0, Step::PrecombatMain);
    let spell = t.cast(P0, theirs).method(MUTATE).target(bears).go();
    assert_eq!(t.zone(spell), Zone::Stack);
    assert_eq!(t.obj_now(spell).controller, P0);
}
