//! CR 702.102 Fuse.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_027_037::{can_cast, untapped_lands};
use crate::common_k702_052_066::destroy;
use mtg_engine::casting::PlayGrant;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, FaceState};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const FUSED: CastMethod = CastMethod::Keyword(KeywordKind::Fuse);

#[test]
fn a_split_card_with_fuse_can_be_cast_fused_from_hand() {
    cr!("702.102", "702.102a", "702.102b", "702.102c");
    ruling!(
        "Far // Away",
        "To cast a fused split spell, pay both of its mana costs. While the spell is on the stack, its mana value is the total amount of mana in both costs."
    );
    ruling!(
        "Far // Away",
        "If such a card is cast as a fused split spell, the resulting spell is multicolored."
    );
    assert_supported("Far // Away");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 2);
    let c = t.hand(P0, "Far // Away");
    // Four lands can't pay both costs ({1}{U} and {2}{B}).
    assert!(!can_cast(&mut t, P0, c, FUSED));
    assert!(can_cast(&mut t, P0, c, CastMethod::Half(0)));
    t.lands(P0, "Wastes", 1);
    assert!(can_cast(&mut t, P0, c, FUSED));
    // The choice is made before it's put onto the stack: one spell, both halves.
    let spell = t
        .cast(P0, c)
        .method(FUSED)
        .targets(&[Entity::Object(bears)])
        .target(P1)
        .go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.stack_len(), 1);
    let o = t.obj(spell);
    assert_eq!(o.face, FaceState::Fused);
    assert_eq!(o.chars.name, "Far // Away");
    assert!(o.chars.colors.contains(Color::Blue) && o.chars.colors.contains(Color::Black));
    assert_eq!(t.g.mana_value_of(spell), 5);
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn a_fused_split_spell_follows_the_left_half_then_the_right_half() {
    cr!("702.102d");
    ruling!(
        "Far // Away",
        "When a fused split spell resolves, follow the instructions of the left half first, then the instructions on the right half."
    );
    let mut t = TestGame::new(2);
    // Far returns the Bears; then Away makes P1 sacrifice a creature: only the Elves are
    // left to sacrifice.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 3);
    let c = t.hand(P0, "Far // Away");
    t.cast(P0, c)
        .method(FUSED)
        .targets(&[Entity::Object(bears)])
        .target(P1)
        .go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
}

#[test]
fn both_halves_can_target_the_same_player() {
    cr!("702.102d");
    ruling!(
        "Toil // Trouble",
        "If you cast Toil // Trouble as a fused split card and choose to target the same player twice, the two drawn cards are counted when determining how much damage is dealt."
    );
    assert_supported("Toil // Trouble");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 4);
    for _ in 0..3 {
        t.hand(P1, "Grizzly Bears");
    }
    let c = t.hand(P0, "Toil // Trouble");
    t.cast(P0, c).method(FUSED).target(P1).target(P1).go();
    t.resolve_all();
    // Toil: P1 draws two (5 cards) and loses 2; Trouble: 5 damage.
    assert_eq!(t.hand_size(P1), 5);
    assert_eq!(t.life(P1), 13);
}

#[test]
fn a_fused_split_spell_with_one_legal_target_still_resolves() {
    cr!("702.102d");
    ruling!(
        "Toil // Trouble",
        "If at least one target is still legal at that time, the spell resolves, but an illegal target can't perform any actions or have any actions performed on it."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 3);
    let c = t.hand(P0, "Far // Away");
    t.cast(P0, c)
        .method(FUSED)
        .targets(&[Entity::Object(bears)])
        .target(P1)
        .go();
    // The Bears leave the battlefield before it resolves: Far's target is illegal.
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Away still makes P1 sacrifice the Elves.
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
}

#[test]
fn a_split_card_cast_from_elsewhere_cant_be_fused() {
    cr!("702.102a");
    ruling!(
        "Far // Away",
        "If you're casting a split card with fuse from any zone other than your hand, you can't cast both halves."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Wastes", 2);
    t.battlefield(P1, "Grizzly Bears");
    let c = t.exile(P0, "Far // Away");
    // An effect lets P0 cast it from exile.
    t.g.play_grants.push(PlayGrant {
        player: P0,
        object: c,
        duration: ability::Duration::EndOfTurn,
        free: false,
        source: None,
        turn: 1,
    });
    assert!(can_cast(&mut t, P0, c, CastMethod::Half(0)));
    assert!(can_cast(&mut t, P0, c, CastMethod::Half(1)));
    assert!(!can_cast(&mut t, P0, c, FUSED));
}

#[test]
fn a_split_card_without_fuse_cant_be_fused() {
    cr!("702.102a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 10);
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Mountain", 2);
    // Fire // Ice has no fuse.
    let c = t.hand(P0, "Fire // Ice");
    assert!(can_cast(&mut t, P0, c, CastMethod::Half(0)));
    assert!(!can_cast(&mut t, P0, c, FUSED));
}

#[test]
fn a_fused_split_spell_can_be_cast_without_paying_its_mana_costs() {
    cr!("702.102a", "702.102c");
    ruling!(
        "Far // Away",
        "If you cast a split card with fuse from your hand without paying its mana cost, you can choose to use its fuse ability and cast both halves without paying their mana costs."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Llanowar Elves");
    let c = t.hand(P0, "Far // Away");
    assert!(!can_cast(&mut t, P0, c, FUSED));
    // An effect lets P0 cast it from their hand without paying its mana cost.
    t.g.play_grants.push(PlayGrant {
        player: P0,
        object: c,
        duration: ability::Duration::EndOfTurn,
        free: true,
        source: None,
        turn: 1,
    });
    assert!(can_cast(&mut t, P0, c, FUSED));
    t.cast(P0, c)
        .method(FUSED)
        .targets(&[Entity::Object(bears)])
        .target(P1)
        .go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
}
