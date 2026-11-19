//! The ways of casting a card from outside the hand that `permissions::attach` offers with
//! the permissions that allow them (CR 601.3; see `permissions.rs`). A way the card's own
//! permission allows (retrace's, flashback's, an adventurer card's on an adventure) is
//! offered with another permission too only if that permission changes how the card is cast,
//! and not when the way's cost comes with the card's own permission to cast it from where it
//! is: the player uses one permission or the other (Muldrotha, the Gravetide ruling) and pays
//! that one's costs, never both. Written for survivors of the November 2026 mutation run of
//! `permissions.rs` (docs/MUTATION.md).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

const RETRACE: CastMethod = CastMethod::Keyword(KeywordKind::Retrace);

/// Engine bug found triaging `permissions.rs:514:17` and `515:17` (in `attach`, the `||`
/// before or after `perm.terms.cost_increase_each.is_some()` as `&&`); this test FAILS until
/// the fix is made (in `permissions::attach`, a way the card's own permission allows is
/// skipped, as one with an alternative cost of its own already is, when the card isn't in the
/// player's hand and the way has an additional cost of its own: retrace's discard comes with
/// retrace's permission, CR 702.81a). During P0's turn Six gives Risen Executioner retrace in
/// P0's graveyard, and Risen Executioner's own permission lets P0 cast it from there for {1}
/// more for each other creature card there. P0 announces which permission they're using:
/// retrace's (a land card discarded) or Risen Executioner's ({2} more here). The engine also
/// offers retrace's way with Risen Executioner's permission, which pays both, and asks which
/// permission retrace's way uses. The mutants drop that way (the only way they change with
/// today's cards), so once it's gone they're equivalent.
#[test]
fn risen_executioner_cast_with_retrace_doesnt_cost_more_for_other_creature_cards() {
    cr!("702.81a", "601.2f");
    ruling!(
        "Muldrotha, the Gravetide",
        "If multiple effects allow you to play a card from your graveyard, you must announce which permission you're using as you begin to play the card."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Six");
    t.lands(P0, "Swamp", 4);
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
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Risen Executioner").len(), 1);
}
