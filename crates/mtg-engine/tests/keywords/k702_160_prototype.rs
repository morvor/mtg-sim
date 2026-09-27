//! CR 702.160 Prototype (see also `tests/cr/r718_prototype_cards.rs`).

use crate::common_k702_153_167::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::{CardType, Color};
use mtg_engine::*;

const PROTOTYPED: CastMethod = CastMethod::Keyword(KeywordKind::Prototype);

#[test]
fn a_prototyped_spell_uses_the_alternative_power_toughness_and_mana_cost() {
    cr!("702.160", "702.160a");
    ruling!(
        "Goring Warplow",
        "Regardless of how it was cast, a prototype card always has the same name, abilities, types, and so on."
    );
    assert_supported("Goring Warplow");
    // Goring Warplow: {6} 5/4 deathtouch; "Prototype {1}{B} — 1/1".
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let card = t.hand(P0, "Goring Warplow");
    // Two lands can't pay {6}, but can pay the prototype's {1}{B}.
    assert!(t.cast(P0, card).try_go().is_err());
    let spell = t.cast(P0, card).method(PROTOTYPED).go();
    let s = t.g.obj(spell);
    assert_eq!(s.chars.mana_cost.as_ref().unwrap().to_string(), "{1}{B}");
    assert_eq!(t.g.mana_value_of(spell), 2);
    assert!(s.chars.colors.contains(Color::Black));
    t.resolve();
    let w = named(&t, P0, "Goring Warplow")[0];
    assert_eq!(t.pt(w), (1, 1));
    // Same name, abilities and types.
    assert!(has_kw(&t, w, KeywordKind::Deathtouch));
    assert!(t.obj(w).is(CardType::Artifact) && t.obj(w).is(CardType::Creature));
    // Leaving the battlefield, it resumes its normal characteristics.
    t.g.move_object(
        w,
        Zone::Hand(P0),
        mtg_engine::events::MoveCause::Effect,
        None,
    )
    .unwrap();
    let back = t.g.current(w);
    assert_eq!(t.g.mana_value_of(back), 6);
    assert!(t.g.obj(back).chars.colors.is_colorless());
    // Cast normally, it's a colorless 5/4.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 6);
    let card = t.hand(P0, "Goring Warplow");
    t.cast(P0, card).go();
    t.resolve();
    let w = named(&t, P0, "Goring Warplow")[0];
    assert_eq!(t.pt(w), (5, 4));
    assert!(t.obj(w).chars.colors.is_colorless());
}

/// Casts `card` with an effect that has P0 cast it without paying its mana cost
/// (CR 608.2g), answering `prototyped` to how it's cast; returns the spell.
fn cast_free_by_effect(t: &mut TestGame, card: ObjectId, prototyped: bool) -> ObjectId {
    use mtg_engine::ability::*;
    t.answer(
        P0,
        DecisionKind::Option,
        Answer::Index(usize::from(prototyped)),
    );
    run_effect(
        t,
        None,
        P0,
        Effect::CastCard {
            who: PlayerRef::You,
            what: Sel::Target(0),
            free: true,
            optional: false,
        },
        &[Entity::Object(card)],
    );
    *t.g.stack.last().expect("the spell")
}

#[test]
fn casting_it_prototyped_isnt_an_alternative_cost() {
    cr!("702.160a", "118.9");
    ruling!(
        "Goring Warplow",
        "Casting a prototyped spell isn't the same as casting it for an alternative cost, and an alternative cost may be applied to a spell cast this way."
    );
    ruling!(
        "Frogmyr Enforcer",
        "Casting a prototyped spell isn't the same as casting it for an alternative cost, and an alternative cost may be applied to a spell cast this way. For example, if an effect allows you to cast an artifact card without paying its mana cost, you could either cast Frogmyr Enforcer normally, or as a prototyped spell."
    );
    // An effect casts Blitz Automaton without paying its mana cost: prototyped...
    let mut t = TestGame::new(2);
    let card = t.exile(P0, "Blitz Automaton");
    let spell = cast_free_by_effect(&mut t, card, true);
    assert_eq!(t.g.obj(spell).chars.name, "Blitz Automaton");
    assert_eq!(t.g.mana_value_of(spell), 3);
    assert!(t.g.obj(spell).chars.colors.contains(Color::Red));
    t.resolve();
    let b = named(&t, P0, "Blitz Automaton")[0];
    assert_eq!(t.pt(b), (3, 2));
    // ... or normally.
    let mut t = TestGame::new(2);
    let card = t.exile(P0, "Blitz Automaton");
    let spell = cast_free_by_effect(&mut t, card, false);
    assert_eq!(t.g.mana_value_of(spell), 7);
    t.resolve();
    let b = named(&t, P0, "Blitz Automaton")[0];
    assert_eq!(t.pt(b), (6, 4));
    // An effect that lets P0 cast it later without paying its mana cost: either way, for
    // free.
    for method in [CastMethod::Free, PROTOTYPED] {
        let mut t = TestGame::new(2);
        let card = t.exile(P0, "Frogmyr Enforcer");
        run_effect(
            &mut t,
            None,
            P0,
            mtg_engine::ability::Effect::GrantPlayPermission {
                who: mtg_engine::ability::PlayerRef::You,
                what: mtg_engine::ability::Sel::Target(0),
                duration: mtg_engine::ability::Duration::EndOfTurn,
                free: true,
            },
            &[Entity::Object(card)],
        );
        let spell = t.cast(P0, card).method(method.clone()).go();
        let (p, _) = if method == PROTOTYPED { (2, 2) } else { (4, 4) };
        t.resolve();
        let f = named(&t, P0, "Frogmyr Enforcer")[0];
        assert_eq!(t.pt(f).0, p, "{method:?}");
        let _ = spell;
    }
}

#[test]
fn the_prototype_ability_functions_in_any_zone_it_could_be_cast_from() {
    cr!("702.160a", "601.3");
    ruling!(
        "Goring Warplow",
        "The prototype ability functions in any zone that the spell could be cast from."
    );
    ruling!(
        "Frogmyr Enforcer",
        "The prototype ability functions in any zone that the spell could be cast from. For example, if an effect allows you to cast artifact spells from your graveyard, you could cast a prototyped Frogmyr Enforcer from your graveyard."
    );
    // Magus of the Future: "You may play lands and cast spells from the top of your
    // library." Blitz Automaton on top is cast prototyped with {2}{R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Magus of the Future");
    let card = t.library_top(P0, "Blitz Automaton");
    t.lands(P0, "Mountain", 3);
    assert!(t.cast(P0, card).try_go().is_err(), "{{7}} can't be paid");
    let card = t.g.current(card);
    t.cast(P0, card).method(PROTOTYPED).go();
    t.resolve();
    let b = named(&t, P0, "Blitz Automaton")[0];
    assert_eq!(t.pt(b), (3, 2));
    // Without a permission, not from the library.
    let mut t = TestGame::new(2);
    let card = t.library_top(P0, "Frogmyr Enforcer");
    t.lands(P0, "Mountain", 4);
    assert!(t.cast(P0, card).method(PROTOTYPED).try_go().is_err());
}

#[test]
fn affinity_applies_to_the_prototyped_spell() {
    cr!("702.160a", "702.41a");
    ruling!(
        "Frogmyr Enforcer",
        "When casting Frogmyr Enforcer prototyped, its affinity for artifacts ability will still apply."
    );
    assert_supported("Frogmyr Enforcer");
    // Prototype {3}{R} with two artifacts: {1}{R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Ornithopter");
    t.lands(P0, "Mountain", 2);
    let card = t.hand(P0, "Frogmyr Enforcer");
    t.cast(P0, card).method(PROTOTYPED).go();
    t.resolve();
    let f = named(&t, P0, "Frogmyr Enforcer")[0];
    assert_eq!(t.pt(f), (2, 2));
}

#[test]
fn a_prototype_card_is_colorless_outside_the_stack_and_battlefield() {
    cr!("702.160a", "718.4");
    ruling!(
        "Frogmyr Enforcer",
        "A prototype card is a colorless card in every zone except the stack or the battlefield, as well as while on the stack or the battlefield if not cast as a prototyped spell."
    );
    let mut t = TestGame::new(2);
    let yard = t.graveyard(P0, "Frogmyr Enforcer");
    assert!(t.obj(yard).chars.colors.is_colorless());
    assert_eq!(t.g.mana_value_of(yard), 7);
    // Cast normally, it's colorless on the stack and the battlefield.
    t.lands(P0, "Mountain", 7);
    let card = t.hand(P0, "Frogmyr Enforcer");
    let spell = t.cast(P0, card).go();
    assert!(t.g.obj(spell).chars.colors.is_colorless());
    t.resolve();
    let f = named(&t, P0, "Frogmyr Enforcer")[0];
    assert!(t.obj(f).chars.colors.is_colorless());
    assert_eq!(t.pt(f), (4, 4));
}

#[test]
fn a_permission_to_cast_red_spells_permits_the_prototyped_spell() {
    cr!("702.160a", "601.3e");
    ruling!(
        "Frogmyr Enforcer",
        "When casting a prototyped spell, use only its prototype characteristics to determine whether it's legal to cast it."
    );
    // "You may cast red spells from among cards in exile": Frogmyr Enforcer is a red
    // spell only when cast prototyped.
    use mtg_engine::ability::*;
    let perm = mtg_engine::card::CardDef::custom(mtg_engine::object::Characteristics {
        name: "Red Exile Permission".into(),
        card_types: [CardType::Enchantment].into_iter().collect(),
        abilities: vec![AbilityDef::new(
            AbilityKind::Static(StaticAbility::new(StaticEffect::PlayPermission(
                PlayPermission {
                    who: PlayerRel::You,
                    zone: ZoneKind::Exile,
                    top_only: false,
                    what: Filter::Color(Color::Red),
                    lands: false,
                    spells: true,
                    cost: None,
                },
            ))),
            "You may cast red spells from among cards in exile.",
        )],
        rules_text: std::sync::Arc::from("You may cast red spells from among cards in exile."),
        ..Default::default()
    });
    let mut t = TestGame::new(2);
    t.custom(P0, perm, Zone::Battlefield);
    let card = t.exile(P0, "Frogmyr Enforcer");
    t.lands(P0, "Mountain", 7);
    assert!(t.cast(P0, card).try_go().is_err(), "a colorless spell");
    let card = t.g.current(card);
    t.cast(P0, card).method(PROTOTYPED).go();
    t.resolve();
    assert_eq!(named(&t, P0, "Frogmyr Enforcer").len(), 1);
}

#[test]
fn copies_made_by_skitterbeam_battalion_have_its_prototyped_characteristics() {
    cr!("702.160a", "718.3d");
    ruling!(
        "Skitterbeam Battalion",
        "The copies will have the same mana cost, mana value, color, power, and toughness as Skitterbeam Battalion, which vary depending on whether it was cast as a prototyped spell."
    );
    assert_supported("Skitterbeam Battalion");
    // Prototype {3}{R}{R} — 2/2; "When this creature enters, if you cast it, create two
    // tokens that are copies of it."
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    let card = t.hand(P0, "Skitterbeam Battalion");
    t.cast(P0, card).method(PROTOTYPED).go();
    t.resolve_all();
    let all = named(&t, P0, "Skitterbeam Battalion");
    assert_eq!(all.len(), 3);
    for id in all {
        assert_eq!(t.pt(id), (2, 2));
        assert_eq!(t.g.mana_value_of(id), 5);
        assert!(t.obj(id).chars.colors.contains(Color::Red));
    }
    // Cast normally: 4/4 colorless copies.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 9);
    let card = t.hand(P0, "Skitterbeam Battalion");
    t.cast(P0, card).go();
    t.resolve_all();
    let all = named(&t, P0, "Skitterbeam Battalion");
    assert_eq!(all.len(), 3);
    for id in all {
        assert_eq!(t.pt(id), (4, 4));
        assert_eq!(t.g.mana_value_of(id), 9);
        assert!(t.obj(id).chars.colors.is_colorless());
    }
}

#[test]
fn a_card_discovered_may_be_cast_prototyped() {
    cr!("702.160a", "701.57a", "118.9");
    ruling!(
        "Goring Warplow",
        "an alternative cost may be applied to a spell cast this way"
    );
    // Trumpeting Carnosaur: "When this creature enters, discover 5." Rootwire Amalgam ({5}
    // 5/5; prototype {1}{G} — 2/3) is found; casting it without paying its mana cost, P0
    // chooses to cast it prototyped.
    for (prototyped, pt) in [(false, (5, 5)), (true, (2, 3))] {
        let mut t = TestGame::new(2);
        t.library_top(P0, "Rootwire Amalgam");
        t.answer_yes(P0, true);
        t.answer(
            P0,
            DecisionKind::Option,
            Answer::Index(usize::from(prototyped)),
        );
        t.enter(P0, "Trumpeting Carnosaur");
        t.resolve_all();
        let r = named(&t, P0, "Rootwire Amalgam");
        assert_eq!(r.len(), 1, "prototyped: {prototyped}");
        assert_eq!(t.pt(r[0]), pt, "prototyped: {prototyped}");
    }
}
