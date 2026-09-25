//! CR 609: effects — what they apply to, doing as much as possible, and "as though"
//! effects.

use crate::r609_common::*;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn effects_apply_only_to_permanents_unless_they_say_otherwise() {
    // CR 609.2 example: an effect that turns all lands into creatures doesn't affect land
    // cards in graveyards; an effect making spells cost more applies to spells on the stack.
    cr!("609.2");
    let mut t = TestGame::new(2);
    t.custom(
        P1,
        permanent(
            "Living Lands",
            &[CardType::Enchantment],
            vec![continuous(
                Filter::Type(CardType::Land),
                vec![
                    Modification::AddTypes(vec![CardType::Creature]),
                    set_pt(1, 1),
                ],
            )],
        ),
        Zone::Battlefield,
    );
    let on_bf = t.battlefield(P1, "Forest");
    let in_gy = t.graveyard(P0, "Forest");
    t.recompute();
    assert!(t.obj_now(on_bf).is_creature());
    assert!(!t.obj_now(in_gy).is_creature());
    // "Spells cost {1} more to cast."
    t.custom(
        P1,
        permanent(
            "Tax",
            &[CardType::Enchantment],
            vec![static_ab(StaticEffect::CostModifier(CostModifier {
                applies_to: CostTarget::Spells(Filter::Any),
                who: PlayerRel::Any,
                change: CostChange::IncreaseGeneric(Value::c(1)),
            }))],
        ),
        Zone::Battlefield,
    );
    t.recompute();
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).target(P1).try_go().is_err());
    t.lands(P0, "Mountain", 1);
    assert!(t.cast(P0, bolt).target(P1).try_go().is_ok());
}

#[test]
fn impossible_effects_do_as_much_as_possible() {
    // CR 609.3 examples: "Discard two cards" with one card in hand discards that card;
    // an effect moving more cards out of a library than it has moves as many as possible.
    cr!("609.3");
    let mut t = TestGame::new(2);
    t.g.players[1].hand.clear();
    t.hand(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    let rot = t.hand(P0, "Mind Rot");
    t.cast(P0, rot).target(P1).go();
    t.resolve();
    assert_eq!(t.hand_size(P1), 0);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let lib = t.library_size(P1);
    resolve_effect(
        &mut t,
        P0,
        vec![],
        &[],
        Effect::Mill {
            who: PlayerRef::EachOpponent,
            n: Value::c(lib as i32 + 10),
        },
    );
    assert_eq!(t.library_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), lib + 1);
}

#[test]
fn two_as_though_effects_can_both_apply() {
    // CR 609.4, 609.4a example: Vedalken Orrery ("You may cast spells as though they had
    // flash") and Shaman's Trance ("...cast spells from other players' graveyards this turn
    // as though those cards were in your graveyard"): a sorcery with flashback in another
    // player's graveyard can be cast at instant speed.
    cr!("609.4", "609.4a");
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        permanent(
            "Vedalken Orrery",
            &[CardType::Artifact],
            vec![static_ab(StaticEffect::FlashPermission {
                who: PlayerRel::You,
                what: Filter::Any,
            })],
        ),
        Zone::Battlefield,
    );
    let firebolt = t.graveyard(P1, "Firebolt");
    t.lands(P0, "Mountain", 5);
    t.set_step(P1, Step::Upkeep);
    let flashback = CastMethod::Keyword(KeywordKind::Flashback);
    assert!(t
        .cast(P0, firebolt)
        .target(P1)
        .method(flashback.clone())
        .try_go()
        .is_err());
    t.clear_answers();
    // Shaman's Trance (this turn).
    resolve_effect(
        &mut t,
        P0,
        vec![],
        &[],
        Effect::AddPlayerEffect {
            who: PlayerRef::You,
            effect: PlayerModification::Custom(
                mtg_engine::as_though::OTHER_GRAVEYARDS_AS_YOURS.into(),
            ),
            duration: Duration::EndOfTurn,
        },
    );
    t.cast(P0, firebolt).target(P1).method(flashback).go();
    t.resolve();
    assert_eq!(t.life(P1), 18);
    // Flashback exiles it.
    assert!(t.in_exile("Firebolt"));
}

#[test]
fn spending_mana_as_though_any_color_changes_only_the_payment() {
    // CR 609.4b: "Players may spend mana as though it were mana of any color" affects only
    // how the cost is paid: the cost and the mana actually spent don't change.
    cr!("609.4b");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).target(P1).try_go().is_err());
    t.clear_answers();
    t.custom(
        P1,
        permanent(
            "Mycosynth Lattice",
            &[CardType::Artifact],
            vec![static_ab(StaticEffect::PlayerEffect {
                affected: PlayerFilter::Any,
                effect: PlayerModification::Custom(
                    mtg_engine::as_though::SPEND_AS_ANY_COLOR.into(),
                ),
            })],
        ),
        Zone::Battlefield,
    );
    t.recompute();
    let spell = t.cast(P0, bolt).target(P1).go();
    let o = t.obj_now(spell);
    assert_eq!(t.g.mana_value_of(spell), 1);
    assert_eq!(o.chars.mana_cost.as_ref().unwrap().to_string(), "{R}");
    assert_eq!(o.stack.as_ref().unwrap().cast.mana_spent, vec![ManaType::G]);
    t.resolve();
    assert_eq!(t.life(P1), 17);
}
