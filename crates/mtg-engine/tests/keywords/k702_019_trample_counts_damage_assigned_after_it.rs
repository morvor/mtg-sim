//! CR 702.19b–c, 510.1e: a trampler's lethal damage (and, with trample over planeswalkers,
//! the damage the attacked planeswalker needs) counts the damage the other creatures
//! assign in the same combat damage step, including those whose damage is assigned after
//! it; the total assignment is what's checked. Found by the combat damage property
//! (`mtg_engine::properties::damage`, docs/PROPERTIES.md).

use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

fn creature(name: &str, p: i32, t: i32, kws: &[KeywordKind]) -> CardDef {
    with_keywords(name, p, t, kws.iter().map(|k| Keyword::new(*k)).collect())
}

fn with_keywords(name: &str, p: i32, t: i32, kws: Vec<Keyword>) -> CardDef {
    CardDef::custom(Characteristics {
        name: name.into(),
        card_types: CardTypeSet::single(CardType::Creature),
        power: Some(p),
        toughness: Some(t),
        abilities: kws
            .into_iter()
            .map(|k| {
                let n = k.kind.name();
                AbilityDef::new(AbilityKind::Keyword(k), n)
            })
            .collect(),
        rules_text: Arc::from(""),
        ..Default::default()
    })
}

/// Two 3/3 tramplers in a band, blocked by a 0/4: each assigns `a` and `b` (to the
/// blocker, then the defending player). Returns the defending player's life and whether
/// the blocker died.
fn banded_tramplers(a: [i64; 2], b: [i64; 2]) -> (i32, bool) {
    let mut t = TestGame::new(2);
    let x = t.custom(
        P0,
        creature(
            "Banded Trampler",
            3,
            3,
            &[KeywordKind::Trample, KeywordKind::Banding],
        ),
        Zone::Battlefield,
    );
    let y = t.custom(
        P0,
        creature("Trampler", 3, 3, &[KeywordKind::Trample]),
        Zone::Battlefield,
    );
    let wall = t.custom(P1, creature("Wall", 0, 4, &[]), Zone::Battlefield);
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(x, Entity::Player(P1)), (y, Entity::Player(P1))]),
    );
    // The band (CR 702.22c); the Wall blocks it (CR 702.22h).
    t.answer_choose(P0, &[Entity::Object(y)]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(wall, x)]),
    );
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(a.to_vec()));
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(b.to_vec()));
    t.advance_to(P0, Step::EndOfCombat);
    // The first trampler's decision offers the minimum counting the second's 3 damage to
    // the 0/4 (4 - 3); the second, assigning last, only the lethal counting what's
    // already assigned to it.
    let offered: Vec<_> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::AssignCombatDamage {
                creature,
                lethal_counting_later,
                ..
            } => Some((creature, lethal_counting_later)),
            _ => None,
        })
        .collect();
    assert_eq!(offered[0], (x, Some(vec![1])));
    assert_eq!(offered[1].1, None);
    (t.life(P1), !t.on_battlefield(wall))
}

#[test]
fn tramplers_may_count_each_others_damage_to_a_shared_blocker() {
    cr!("702.19b", "510.1e", "702.22h");
    // 2 + 2 is lethal for the 0/4: each assigns its last point to the player.
    assert_eq!(banded_tramplers([2, 1], [2, 1]), (18, true));
}

#[test]
fn a_total_assignment_without_lethal_damage_is_illegal_and_assigned_again() {
    cr!("702.19b", "510.1e");
    interp!("trample-reassignment-counts-only-assigned-damage");
    // 1 + 1 isn't lethal: the assignment is illegal, the player assigns again (here the
    // defaults: 3 to the Wall, then 1 more and 2 to the player).
    assert_eq!(banded_tramplers([1, 2], [1, 2]), (18, true));
}

/// Two attackers with trample over planeswalkers attacking a planeswalker with loyalty 4:
/// 3 + 2 is at least its loyalty, so each may assign its last point to its controller
/// (CR 702.19c).
#[test]
fn trample_over_planeswalkers_counts_damage_assigned_after_it() {
    cr!("702.19c", "510.1e");
    let over = || {
        let mut k = Keyword::new(KeywordKind::Trample);
        k.text = Some(mtg_engine::kw::trample::OVER_PLANESWALKERS.into());
        vec![k]
    };
    let mut t = TestGame::new(2);
    let x = t.custom(P0, with_keywords("Big", 4, 4, over()), Zone::Battlefield);
    let y = t.custom(P0, with_keywords("Small", 3, 3, over()), Zone::Battlefield);
    let walker = t.custom(
        P1,
        CardDef::custom(Characteristics {
            name: "Walker".into(),
            card_types: CardTypeSet::single(CardType::Planeswalker),
            loyalty: Some(4),
            rules_text: Arc::from(""),
            ..Default::default()
        }),
        Zone::Battlefield,
    );
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![
            (x, Entity::Object(walker)),
            (y, Entity::Object(walker)),
        ]),
    );
    // Recipients: the planeswalker, then its controller.
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![3, 1]));
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![2, 1]));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
    assert!(!t.on_battlefield(walker));
}
