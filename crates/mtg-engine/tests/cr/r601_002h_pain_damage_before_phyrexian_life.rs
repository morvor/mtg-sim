//! CR 601.2g–h, 107.4f, 119.4: mana abilities are activated before the cost is paid, so
//! the damage a mana ability deals ("This land deals 1 damage to you.") lowers the life
//! left to pay Phyrexian mana with. The automatic payer must find the payment that keeps
//! enough life. Found by the mana payment property (`mtg_engine::properties::payment`,
//! docs/PROPERTIES.md).

use mtg_engine::mana::ManaCost;
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

#[test]
fn a_payment_with_phyrexian_life_avoids_pain_land_damage() {
    cr!("601.2g", "601.2h", "107.4f", "119.4");
    let mut t = TestGame::new(2);
    t.g.players[0].life = 2;
    t.battlefield(P0, "City of Brass");
    t.battlefield(P0, "Snow-Covered Swamp");
    let river = t.battlefield(P0, "Underground River");
    let cost = ManaCost::parse("{2/W}{U/P}{R/P}").unwrap();
    let card = t.custom(
        P0,
        CardDef::custom(Characteristics {
            name: "Payment Test".into(),
            colors: cost.colors(),
            mana_cost: Some(cost),
            card_types: CardTypeSet::single(CardType::Sorcery),
            rules_text: Arc::from(""),
            ..Default::default()
        }),
        Zone::Hand(P0),
    );
    // The only payment: City of Brass's {R} for {R/P}, 2 life for {U/P}, and {2/W} with the
    // Swamp's {B} and the River's painless {C} (its {U} or {B} would deal 1 damage and
    // leave too little life for {U/P}).
    let spell = t.cast(P0, card).go();
    assert_eq!(t.zone(spell), Zone::Stack);
    assert!(t.obj(river).tapped);
    assert_eq!(t.life(P0), 0);
}
