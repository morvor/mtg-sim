//! Rulings batch P074 — mana abilities granted "until this card is cast from exile"
//! (Rakish Revelers, Shattered Seraph): the grant outlasts the card leaving exile some
//! other way, and the granted ability can pay for casting the card itself.

use crate::r_s01_common::supported;
use crate::r_s02_common::can_cast;
use crate::r_s04_common::add_mana;
use crate::r_s05_common::move_to;
use crate::r_s06_common::activate_containing;
use mtg_engine::ability::AbilityKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::CardType;
use mtg_engine::*;

const CARDS: [&str; 2] = ["Rakish Revelers", "Shattered Seraph"];

/// The number of mana abilities the land has.
fn mana_abilities(t: &TestGame, land: ObjectId) -> usize {
    t.obj_now(land)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(&a.kind, AbilityKind::Activated(x) if x.is_mana_ability))
        .count()
}

/// P0 exiles `name` from hand with its ability, targeting a Wastes; returns (card, land).
fn exile_targeting_wastes(t: &mut TestGame, name: &str) -> (ObjectId, ObjectId) {
    let card = t.hand(P0, name);
    let wastes = t.battlefield(P0, "Wastes");
    add_mana(t, P0, ManaType::C, 2);
    t.answer_targets(P0, &[Entity::Object(wastes)]);
    activate_containing(t, P0, card, "from your hand").unwrap();
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Exile, "{name}");
    assert_eq!(mana_abilities(t, wastes), 2, "{name}");
    (card, wastes)
}

#[test]
fn the_land_keeps_the_ability_if_the_card_leaves_exile_uncast() {
    cr!("611.2b", "610.3");
    ruling!(
        "Rakish Revelers",
        "If Rakish Revelers is removed from exile without being cast, the land will continue to have this ability."
    );
    ruling!(
        "Shattered Seraph",
        "If Shattered Seraph is removed from exile without being cast, the land will continue to have this ability."
    );
    for name in CARDS {
        supported(name);
        let mut t = TestGame::new(2);
        let (card, wastes) = exile_targeting_wastes(&mut t, name);
        move_to(&mut t, card, Zone::Graveyard(P0));
        t.g.recompute();
        assert_eq!(mana_abilities(&t, wastes), 2, "{name}");
        // Positive control: casting it from exile ends the grant.
        let mut t = TestGame::new(2);
        let (card, wastes) = exile_targeting_wastes(&mut t, name);
        add_mana(&mut t, P0, ManaType::W, 1);
        add_mana(&mut t, P0, ManaType::U, 1);
        add_mana(&mut t, P0, ManaType::B, 1);
        add_mana(&mut t, P0, ManaType::R, 1);
        add_mana(&mut t, P0, ManaType::G, 1);
        add_mana(&mut t, P0, ManaType::C, 2);
        t.cast(P0, t.g.current(card)).go();
        t.resolve_all();
        assert_eq!(t.named_on_battlefield(name).len(), 1, "{name}");
        assert_eq!(mana_abilities(&t, wastes), 1, "{name}");
    }
}

#[test]
fn the_granted_ability_can_pay_for_casting_the_card_from_exile() {
    cr!("601.2g", "605.3a", "611.2b");
    ruling!(
        "Rakish Revelers",
        "You may use the mana ability Rakish Revelers grants to the land while casting Rakish Revelers from exile."
    );
    ruling!(
        "Shattered Seraph",
        "You may use the mana ability Shattered Seraph grants to the land while casting Shattered Seraph from exile."
    );
    // Rakish Revelers {2}{R}{G}{W}: Forest, Plains, two Wastes and the granted Wastes
    // (for {R}). Shattered Seraph {4}{W}{U}{B}: Plains, Swamp, four Wastes and the
    // granted Wastes (for {U}).
    for (name, lands) in [
        ("Rakish Revelers", &["Forest", "Plains", "Wastes", "Wastes"][..]),
        (
            "Shattered Seraph",
            &["Plains", "Swamp", "Wastes", "Wastes", "Wastes", "Wastes"][..],
        ),
    ] {
        let mut t = TestGame::new(2);
        let (card, _) = exile_targeting_wastes(&mut t, name);
        for l in lands {
            t.battlefield(P0, l);
        }
        assert!(can_cast(&mut t, P0, card, CastMethod::Normal), "{name}");
        let r = t.cast(P0, t.g.current(card)).try_go();
        assert!(r.is_ok(), "{name}: {r:?} {}", t.dump_log());
        t.resolve_all();
        assert_eq!(t.named_on_battlefield(name).len(), 1, "{name}");
        assert!(t.g.permanents().all(|o| !o.is(CardType::Land) || o.tapped));
    }
}
