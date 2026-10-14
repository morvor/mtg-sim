//! CR 702.146 Disturb (with the back faces' "If this would be put into a graveyard from
//! anywhere, exile it instead").

use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

const DISTURB: CastMethod = CastMethod::Keyword(KeywordKind::Disturb);

fn can_cast(t: &mut TestGame, p: PlayerId, card: ObjectId, method: CastMethod) -> bool {
    t.g.turn.priority = Some(p);
    let card = t.g.current(card);
    t.g.legal_actions(p).iter().any(|a| {
        matches!(a, decision::Action::Cast { card: c, method: m } if *c == card && *m == method)
    })
}

#[test]
fn disturb_casts_the_card_transformed_from_the_graveyard() {
    cr!("702.146a", "702.146b");
    ruling!(
        "Lunarch Veteran // Luminous Phantom",
        "A spell cast this way enters the battlefield with its back face up."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let card = t.graveyard(P0, "Lunarch Veteran // Luminous Phantom");
    assert!(can_cast(&mut t, P0, card, DISTURB));
    // Not normally from the graveyard.
    assert!(!can_cast(&mut t, P0, card, CastMethod::Normal));
    let spell = t.cast(P0, card).method(DISTURB).go();
    // On the stack it has its back face's characteristics.
    let s = t.g.obj(spell);
    assert_eq!(s.chars.name.as_str(), "Luminous Phantom");
    assert_eq!(s.face, FaceState::Back);
    // The disturb cost {1}{W} was paid.
    let untapped =
        t.g.battlefield
            .iter()
            .filter(|id| !t.g.obj(**id).tapped)
            .count();
    assert_eq!(untapped, 0);
    t.resolve_all();
    let phantom = t.named_on_battlefield("Luminous Phantom");
    assert_eq!(phantom.len(), 1);
    assert_eq!(t.g.obj(phantom[0]).face, FaceState::Back);
}

#[test]
fn disturb_only_from_the_graveyard() {
    cr!("702.146a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let card = t.hand(P0, "Lunarch Veteran // Luminous Phantom");
    assert!(!can_cast(&mut t, P0, card, DISTURB));
    assert!(can_cast(&mut t, P0, card, CastMethod::Normal));
}

#[test]
fn a_countered_disturb_spell_is_exiled() {
    cr!("702.146a", "614.1a");
    ruling!(
        "Lunarch Veteran // Luminous Phantom",
        "if the spell is countered after you cast it using the disturb ability, it will be put into exile"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    t.lands(P1, "Island", 3);
    let card = t.graveyard(P0, "Lunarch Veteran // Luminous Phantom");
    let spell = t.cast(P0, card).method(DISTURB).go();
    let cancel = t.hand(P1, "Cancel");
    t.cast(P1, cancel).target(spell).go();
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Exile);
    assert!(!t.in_graveyard(P0, "Lunarch Veteran // Luminous Phantom"));
}
