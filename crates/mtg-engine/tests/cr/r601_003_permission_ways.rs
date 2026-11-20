//! The ways of casting a card that `permissions::attach` offers with the permissions that
//! allow them (CR 601.3; see `permissions.rs`). A way the card's own permission allows
//! (retrace's, flashback's, an adventurer card's on an adventure) is offered with another
//! permission too only if that permission changes how the card is cast, and not when the way's
//! cost comes with the card's own permission to cast it from where it is: the player uses one
//! permission or the other (as the Muldrotha, the Gravetide ruling has them announce) and pays
//! that one's costs, never both. In the hand the permission to cast it is the rules', so an own
//! way's additional cost there comes with a timing permission ("as though it had flash if you
//! pay {2} more"), which an effect's permission combines with. Written for survivors of the
//! November 2026 mutation run of `permissions.rs` (docs/MUTATION.md).

use mtg_engine::ability::Duration;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const RETRACE: CastMethod = CastMethod::Keyword(KeywordKind::Retrace);

/// Regression test for the engine bug found triaging `permissions.rs:514:17` and `515:17` (in
/// `attach`, the `||` before or after `perm.terms.cost_increase_each.is_some()` as `&&`); it
/// failed before the fix (commit `c83fe93`: in `permissions::attach`, a way the card's own
/// permission allows is skipped, as one with an alternative cost of its own already is, when
/// the card isn't in the player's hand and the way has an additional cost of its own:
/// retrace's discard comes with retrace's permission, CR 702.81a). During P0's turn Six gives
/// Risen Executioner retrace in P0's graveyard, and Risen Executioner's own permission lets P0
/// cast it from there for {1} more for each other creature card there. P0 uses one permission
/// or the other: retrace's (a land card discarded) or Risen Executioner's ({2} more here).
/// Before the fix the engine also offered retrace's way with Risen Executioner's permission,
/// which paid both, and asked which permission retrace's way used. The mutants drop that way
/// (the only way they change with today's cards), so since the fix they're equivalent.
#[test]
fn risen_executioner_cast_with_retrace_doesnt_cost_more_for_other_creature_cards() {
    cr!("702.81a", "601.2f");
    interp!("permission-choice-always-asked");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Six");
    let swamps = t.lands(P0, "Swamp", 4);
    let risen = t.graveyard(P0, "Risen Executioner");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    let forest = t.hand(P0, "Forest");
    t.g.recompute();
    assert!(t.obj_now(risen).chars.has_keyword(KeywordKind::Retrace));
    let ways = t.g.cast_options(P0, risen);
    // Retrace's way, with retrace's own permission only.
    let retrace: Vec<_> = ways.iter().filter(|o| o.method == RETRACE).collect();
    assert_eq!(retrace.len(), 1, "{ways:?}");
    assert!(retrace[0].permission.is_none());
    // Risen Executioner's permission: the normal way, for {2} more.
    assert!(ways
        .iter()
        .any(|o| o.method == CastMethod::Normal && o.permission.is_some()));
    // Cast with retrace: no permission to announce; {2}{B}{B} and the Forest discarded.
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.cast(P0, risen).method(RETRACE).go();
    assert!(!t.asked()[from..].iter().any(|(_, d)| matches!(
        d,
        Decision::ChooseOption { prompt, .. } if prompt.starts_with("Choose the permission")
    )));
    assert_eq!(t.zone(forest), Zone::Graveyard(P0));
    assert!(swamps.iter().all(|s| t.obj_now(*s).tapped));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Risen Executioner").len(), 1);
}

/// The other side of that fix: in its owner's hand a card's own way with an additional cost
/// of its own is still offered with an effect's permission that changes the way. Rout's "You
/// may cast this spell as though it had flash if you pay {2} more to cast it" is a timing
/// permission (CR 601.3c), not the permission to cast it from the hand (the rules', CR 601.3),
/// so an effect's permission to cast it without paying its mana cost (an alternative cost,
/// CR 118.9) combines with it, and the {2} is added to that alternative cost (CR 118.9d,
/// 601.2f): during P1's turn P0 casts Rout for {2}. Were the own way skipped in the hand too
/// (`attach`'s skip without its `Zone::Hand` exclusion), the only way to cast it with flash
/// would cost {3}{W}{W} and {2}, which P0's two Plains can't pay.
#[test]
fn rout_cast_with_flash_and_a_permission_to_cast_it_free_costs_only_the_2_more() {
    cr!("601.3c", "118.9d", "601.2f");
    interp!("permission-choice-always-asked");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let plains = t.lands(P0, "Plains", 2);
    let rout = t.hand(P0, "Rout");
    t.set_step(P1, Step::End);
    t.g.turn.priority = Some(P0);
    mtg_engine::casting::grant_play_permission(
        &mut t.g,
        P0,
        vec![rout],
        Duration::EndOfTurn,
        true,
        None,
    );
    let flash = CastMethod::Alternative(t.obj(rout).chars.abilities[0].uid);
    // Rout's flash way, with the effect's permission: no mana cost, {2} more.
    let ways = t.g.cast_options(P0, rout);
    assert!(
        ways.iter().any(|o| o.method == flash
            && o.flash
            && o.alt_cost.is_some()
            && o.extra_cost.is_some()
            && o.permission.is_some()),
        "{ways:?}"
    );
    t.cast(P0, rout).method(flash).go();
    assert!(plains.iter().all(|p| t.obj_now(*p).tapped));
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}
