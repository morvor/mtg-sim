//! Rulings batch S22 — Epic Experiment ("Exile the top X cards of your library. You may
//! cast instant and sorcery spells with mana value X or less from among them without
//! paying their mana costs. Then put all cards exiled this way that weren't cast into
//! your graveyard."): the spells are cast as it resolves (CR 608.2g), ignoring timing
//! restrictions based on their types but not others (CR 307.1, 601.3).

use crate::r_s01_common::*;
use crate::r_s04_common::add_mana;
use crate::r_s22_common::*;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn epic_experiment_ignores_type_timing_but_not_other_timing_restrictions() {
    cr!("608.2g", "307.1", "601.3", "107.3b", "118.9");
    ruling!(
        "Epic Experiment",
        "When casting an instant or sorcery card this way, ignore timing restrictions based on the card's type. Other timing restrictions, such as \"Cast [this card] only during combat,\" must be followed."
    );
    supported("Epic Experiment");
    supported("Mandate of Peace");
    // X = 3: Lava Spike (sorcery), Mandate of Peace ("Cast this spell only during
    // combat.") and Divination (sorcery, mana value 3) are exiled; Tormenting Voice
    // stays in the library.
    let mut t = TestGame::new(2);
    stack_library(
        &mut t,
        P0,
        &["Lava Spike", "Mandate of Peace", "Divination", "Tormenting Voice"],
    );
    add_mana(&mut t, P0, ManaType::U, 1);
    add_mana(&mut t, P0, ManaType::R, 1);
    add_mana(&mut t, P0, ManaType::C, 3);
    let epic = t.hand(P0, "Epic Experiment");
    choose_names_when_offered(&mut t, P0, &["Lava Spike", "Mandate of Peace", "Divination"]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let hand = t.hand_size(P0);
    t.cast(P0, epic).x(3).go();
    t.resolve_all();
    // The sorceries were cast although Epic Experiment was on the stack.
    assert_eq!(t.life(P1), 17, "Lava Spike was cast");
    assert!(t.in_graveyard(P0, "Divination"));
    assert_eq!(t.hand_size(P0), hand - 1 + 2, "Divination drew two cards");
    // Mandate of Peace couldn't be cast outside combat: it was put into the graveyard,
    // and its "Your opponents can't cast spells this turn." didn't happen.
    assert!(t.in_graveyard(P0, "Mandate of Peace"));
    assert!(!t.in_exile("Mandate of Peace"));
    assert!(!t.in_exile("Lava Spike"));
    let bolt = t.hand(P1, "Lightning Bolt");
    add_mana(&mut t, P1, ManaType::R, 1);
    assert!(!crate::r_s08_common::legal_cast_methods(&mut t, P1, bolt).is_empty());
}
