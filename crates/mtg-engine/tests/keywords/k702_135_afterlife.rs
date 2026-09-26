//! CR 702.135 Afterlife.

use crate::common_k702_125_139::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn spirits(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    tokens_of(t, p)
        .into_iter()
        .filter(|id| t.obj_now(*id).chars.has_subtype("Spirit"))
        .collect()
}

#[test]
fn afterlife_creates_flying_spirits_when_it_dies() {
    cr!("702.135", "702.135a");
    assert_supported_card("Ministrant of Obligation");
    let mut t = TestGame::new(2);
    // Ministrant of Obligation: 2/1, afterlife 2.
    let m = t.battlefield(P0, "Ministrant of Obligation");
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    t.g.turn.priority = Some(P1);
    t.cast(P1, bolt).target(Entity::Object(m)).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Ministrant of Obligation"));
    let s = spirits(&t, P0);
    assert_eq!(s.len(), 2);
    for id in s {
        let o = t.obj_now(id);
        assert_eq!(t.pt(id), (1, 1));
        assert!(o.chars.is(CardType::Creature));
        assert!(o.chars.colors.contains(Color::White) && o.chars.colors.contains(Color::Black));
        assert_eq!(o.chars.colors.count(), 2);
        assert!(o.chars.has_keyword(KeywordKind::Flying));
    }
}

#[test]
fn afterlife_triggers_on_any_trip_from_the_battlefield_to_a_graveyard_only() {
    cr!("702.135a");
    // Sacrificed: it's put into a graveyard from the battlefield.
    let mut t = TestGame::new(2);
    let o = t.battlefield(P0, "Imperious Oligarch");
    t.g.sacrifice(o, P0);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(spirits(&t, P0).len(), 1);
    // Exiled or bounced: no trigger.
    let mut t = TestGame::new(2);
    let o = t.battlefield(P0, "Imperious Oligarch");
    t.g.move_object(o, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    t.g.flush_events();
    t.resolve_all();
    assert!(spirits(&t, P0).is_empty());
    // Discarded from hand: not from the battlefield.
    let mut t = TestGame::new(2);
    let o = t.hand(P0, "Imperious Oligarch");
    t.g.move_object(o, Zone::Graveyard(P0), mtg_engine::events::MoveCause::Discard, None);
    t.g.flush_events();
    t.resolve_all();
    assert!(spirits(&t, P0).is_empty());
}

#[test]
fn each_instance_of_afterlife_triggers_separately() {
    cr!("702.135b");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Ministrant of Obligation");
    gain(&mut t, P0, m, Keyword::with_n(KeywordKind::Afterlife, 1));
    assert_eq!(keyword_count(&t, m, KeywordKind::Afterlife), 2);
    t.g.sacrifice(m, P0);
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Afterlife 2"), 1);
    assert_eq!(triggers_on_stack(&t, "Afterlife 1"), 1);
    t.resolve_all();
    assert_eq!(spirits(&t, P0).len(), 3);
}

#[test]
fn afterlife_granted_until_end_of_turn_by_a_spell() {
    cr!("702.135a");
    assert_supported_card("Afterlife Insurance");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ai = t.hand(P0, "Afterlife Insurance");
    t.lands(P0, "Plains", 2);
    t.cast(P0, ai).go();
    t.resolve_all();
    assert!(has(&t, bears, KeywordKind::Afterlife));
    t.g.sacrifice(bears, P0);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(spirits(&t, P0).len(), 1);
}

#[test]
fn spirit_tokens_arent_created_in_time_to_block() {
    cr!("702.135a");
    ruling!(
        "Ministrant of Obligation",
        "Because blockers are chosen all at once, you can’t block with a creature with afterlife, wait for it to die, then block with the resulting Spirit tokens."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let m = t.battlefield(P1, "Ministrant of Obligation");
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(m, giant)]);
    // The Spirits were created after blockers were declared: they aren't blocking.
    let s = spirits(&t, P1);
    assert_eq!(s.len(), 2);
    for id in s {
        assert!(!is_blocking_now(&t, id));
    }
    let _ = Step::EndOfCombat;
}

fn is_blocking_now(t: &TestGame, id: ObjectId) -> bool {
    t.g.is_blocking(t.g.current(id))
}
