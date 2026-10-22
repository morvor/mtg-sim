//! Keywords granted to spells as they're cast from a zone
//! (`src/oracle/patterns/k702_038_051.rs`, `casting::as_spell_filter`; CR 601.2, 610.5):
//! "Spells you cast from exile have convoke." (Hoarding Broodlord), "Noncreature spells you
//! cast from exile have convoke." (Party Thrasher).

use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Foretells `card` from `p`'s hand and returns it in exile.
fn foretell(t: &mut TestGame, p: PlayerId, card: ObjectId) -> ObjectId {
    t.g.players[p.idx()].mana_pool.add_type(ManaType::C, 2);
    t.g.turn.priority = Some(p);
    t.g.take_action(p, Action::Special(SpecialAction::Foretell { card }));
    t.g.flush_events();
    let new = t.g.current(card);
    assert_eq!(t.obj(new).zone, Zone::Exile);
    new
}

/// Whether `p` may cast `card` (with no mana: only by convoking) for its foretell cost.
fn castable_by_convoke(granter: &str) -> bool {
    let mut t = TestGame::new(2);
    if !granter.is_empty() {
        t.battlefield(P0, granter);
    }
    let a = t.battlefield(P1, "Grizzly Bears");
    // Poison the Cup: "Destroy target creature. ..." Foretell {1}{B}.
    let cup = t.hand(P0, "Poison the Cup");
    let foretold = foretell(&mut t, P0, cup);
    t.advance_to(P1, Step::Upkeep);
    let z1 = t.battlefield(P0, "Walking Corpse");
    let z2 = t.battlefield(P0, "Walking Corpse");
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(vec![Entity::Object(z1), Entity::Object(z2)]),
    );
    let ok = t
        .cast(P0, foretold)
        .method(CastMethod::Keyword(KeywordKind::Foretell))
        .target(a)
        .try_go()
        .is_ok();
    if ok {
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Grizzly Bears"));
        assert!(t.obj_now(z1).tapped && t.obj_now(z2).tapped);
    }
    ok
}

#[test]
fn spells_cast_from_exile_have_convoke() {
    cr!("702.51a", "610.5");
    for n in ["Hoarding Broodlord", "Party Thrasher"] {
        let c = card(n);
        assert!(
            c.unsupported_text()
                .iter()
                .all(|u| !u.contains("cast from exile have convoke")),
            "{n}"
        );
    }
    assert!(!castable_by_convoke(""));
    assert!(castable_by_convoke("Hoarding Broodlord"));
    assert!(castable_by_convoke("Party Thrasher"));
}

#[test]
fn spells_cast_from_hand_dont_get_convoke() {
    cr!("702.51a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hoarding Broodlord");
    t.battlefield(P0, "Walking Corpse");
    t.battlefield(P0, "Walking Corpse");
    t.battlefield(P0, "Walking Corpse");
    let a = t.battlefield(P1, "Hill Giant");
    let cup = t.hand(P0, "Poison the Cup");
    t.set_step(P0, Step::PrecombatMain);
    assert!(t.cast(P0, cup).target(a).try_go().is_err());
}


#[test]
fn molten_disaster_has_split_second_only_if_kicked() {
    cr!("702.61a", "702.33d");
    let c = card("Molten Disaster");
    assert!(c.unsupported_text().is_empty(), "{:?}", c.unsupported_text());
    for kicked in [false, true] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Mountain", 4);
        t.lands(P1, "Mountain", 1);
        let bolt = t.hand(P1, "Lightning Bolt");
        let d = t.hand(P0, "Molten Disaster");
        t.set_step(P0, Step::PrecombatMain);
        t.cast(P0, d).x(1).kicked(kicked).go();
        let responded = t.cast(P1, bolt).target(P0).try_go().is_ok();
        assert_eq!(responded, !kicked);
    }
}
