//! Rulings batch S22 — snow mana (CR 107.4h): {S} in a cost is paid with mana from a snow
//! source, and effects that reduce generic costs don't reduce it; "the amount of {S}
//! spent to cast this spell" counts mana from snow sources spent on it, and such a spell
//! can be cast without any.

use crate::r_s01_common::*;
use crate::r_s22_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::CardDef;
use mtg_engine::decision::Decision;
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

/// Casts Graven Lore ("Scry X, where X is the amount of {S} spent to cast this spell,
/// then draw three cards.") paying with `snow` Snow-Covered Islands and Islands for the
/// rest; returns the number of cards P0 was asked to scry and the cards drawn.
fn graven_lore(snow: usize) -> (usize, usize) {
    let mut t = TestGame::new(2);
    t.lands(P0, "Snow-Covered Island", snow);
    t.lands(P0, "Island", 5 - snow);
    let lore = t.hand(P0, "Graven Lore");
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    t.cast(P0, lore).go();
    assert_eq!(tapped_lands(&t, P0), 5);
    t.resolve_all();
    let scried = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Scry { cards, .. } => Some(cards.len()),
            _ => None,
        })
        .sum();
    (scried, t.hand_size(P0) + 1 - hand)
}

#[test]
fn a_spell_counting_snow_mana_can_be_cast_without_any() {
    cr!("107.4h", "601.2h");
    ruling!(
        "Snow-Covered Island",
        "Some cards have additional effects for each {S} spent to cast them. You can cast these spells even if you don't spend any snow mana to cast them; their additional effects simply won't do anything."
    );
    ruling!(
        "Icebind Pillar",
        "Some cards have additional effects for each {S} spent to cast them. You can cast these spells even if you don’t spend any snow mana to cast them; their additional effects simply won’t do anything."
    );
    supported("Graven Lore");
    // No snow mana: it's cast and resolves; it scries 0 and draws three cards.
    assert_eq!(graven_lore(0), (0, 3));
    // Two mana from snow sources: scry 2.
    assert_eq!(graven_lore(2), (2, 3));
    assert_eq!(graven_lore(5), (5, 3));
}

#[test]
fn a_copy_of_a_spell_had_no_snow_mana_spent_on_it() {
    cr!("107.4h", "707.10");
    supported("Graven Lore");
    supported("Twincast");
    let mut t = TestGame::new(2);
    t.lands(P0, "Snow-Covered Island", 5);
    t.lands(P0, "Island", 2);
    let lore = t.hand(P0, "Graven Lore");
    let twincast = t.hand(P0, "Twincast");
    let spell = t.cast(P0, lore).go();
    let from = t.asked().len();
    t.cast(P0, twincast).target(Entity::Object(spell)).go();
    // Twincast resolves; its copy of Graven Lore resolves: scry 0.
    t.resolve();
    t.resolve();
    let scries: Vec<usize> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Scry { cards, .. } => Some(cards.len()),
            _ => None,
        })
        .collect();
    assert!(scries.iter().all(|n| *n == 0), "{scries:?}");
    // The original: scry 5.
    let from = t.asked().len();
    t.resolve();
    let scried: usize = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Scry { cards, .. } => Some(cards.len()),
            _ => None,
        })
        .sum();
    assert_eq!(scried, 5);
}

/// "Activated abilities of creatures you control cost {1} less to activate."
fn ability_discount() -> CardDef {
    let s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::Abilities(Filter::and(vec![
            Filter::creature(),
            Filter::ControlledBy(PlayerRel::You),
        ])),
        who: PlayerRel::You,
        change: CostChange::ReduceGeneric(Value::c(1)),
    }));
    let text = "Activated abilities of creatures you control cost {1} less to activate.";
    CardDef::custom(Characteristics {
        name: SmolStr::new("Discount Engine"),
        card_types: TypeLine::parse("Enchantment").card_types,
        abilities: vec![AbilityDef::new(AbilityKind::Static(s), text)],
        rules_text: Arc::from(text),
        ..Default::default()
    })
}

#[test]
fn a_generic_cost_reduction_doesnt_reduce_snow_costs() {
    cr!("107.4h", "118.7a", "601.2f", "602.2b");
    ruling!(
        "Snow-Covered Island",
        "The Kaldheim set doesn't have any cards with mana costs that include {S}, but some previous sets do. If an effect says such a spell costs {1} less to cast, that reduction doesn't apply to any {S} costs. This is also true for activated abilities that include {S} in their activation costs and effects that reduce those costs."
    );
    ruling!(
        "Icebind Pillar",
        "The Kaldheim set doesn’t have any cards with mana costs that include {S}, but some previous sets do. If an effect says such a spell costs {1} less to cast, that reduction doesn’t apply to any {S} costs."
    );
    supported("Icehide Golem");
    supported("Foundry Inspector");
    supported("Icehide Troll");
    // Foundry Inspector: "Artifact spells you cast cost {1} less to cast." Icehide Golem
    // (mana cost {S}) still costs {S}: it can't be cast with mana from non-snow sources.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Foundry Inspector");
    t.lands(P0, "Island", 3);
    let golem = t.hand(P0, "Icehide Golem");
    assert!(t.cast(P0, golem).try_go().is_err());
    assert_eq!(t.zone(golem), Zone::Hand(P0));
    // With a snow source, it costs one mana from it.
    let snow = t.battlefield(P0, "Snow-Covered Island");
    t.cast(P0, golem).go();
    assert!(t.obj_now(snow).tapped);
    assert_eq!(tapped_lands(&t, P0), 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Icehide Golem").len(), 1);
    // A generic reduction of its activated ability's cost ({S}{S}) doesn't reduce it
    // either: one snow source isn't enough.
    let mut t = TestGame::new(2);
    t.custom(P0, ability_discount(), Zone::Battlefield);
    let troll = t.battlefield(P0, "Icehide Troll");
    t.battlefield(P0, "Snow-Covered Forest");
    t.lands(P0, "Forest", 3);
    assert!(t.activate(P0, troll, 0, &[]).is_err());
    assert_eq!(tapped_lands(&t, P0), 0);
    t.battlefield(P0, "Snow-Covered Forest");
    t.activate(P0, troll, 0, &[]).expect("{S}{S} paid");
    assert_eq!(tapped_named(&t, P0, "Snow-Covered Forest"), 2);
    assert_eq!(tapped_named(&t, P0, "Forest"), 0);
    t.resolve_all();
    assert_eq!(t.pt(troll), (4, 3));
}
