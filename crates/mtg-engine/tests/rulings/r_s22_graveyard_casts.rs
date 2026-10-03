//! Rulings batch S22 — casting a spell from a graveyard without paying its mana cost as an
//! ability resolves (CR 608.2g, 118.9): Sorcerous Squall ("... from that player's
//! graveyard"), Deluxe Dragster ("target instant or sorcery card from that player's
//! graveyard") and Sword of Once and Future ("... from your graveyard"). X is 0 (CR
//! 107.3b); additional costs are paid, alternative costs can't be (CR 118.9a); the spell
//! cast this way is exiled rather than put into a graveyard (CR 614.1a).

use crate::r_s01_common::*;
use crate::r_s04_common::add_mana;
use crate::r_s22_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Casts Sorcerous Squall ("Target opponent mills nine cards, then you may cast an instant
/// or sorcery spell from that player's graveyard without paying its mana cost. If that
/// spell would be put into a graveyard, exile it instead.") targeting P1, paying with
/// mana in P0's pool; P0 chooses `card`, `answers` queued for casting it.
fn run_squall(t: &mut TestGame, card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    add_mana(t, P0, ManaType::U, 9);
    let squall = t.hand(P0, "Sorcerous Squall");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[Entity::Object(card)]);
    answers(t);
    t.cast(P0, squall).go();
    t.resolve_all();
    t.clear_answers();
}

fn squall(t: &mut TestGame, name: &str) -> ObjectId {
    t.graveyard(P1, name)
}

#[test]
fn sorcerous_squall_casts_a_spell_with_x_as_zero() {
    cr!("107.3b", "118.9", "608.2g");
    ruling!(
        "Sorcerous Squall",
        "If the spell has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    supported("Sorcerous Squall");
    // Blaze ("Blaze deals X damage to any target.") is milled from P1's library and cast
    // with X = 0 at P1's Hill Giant.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let blaze = t.library_top(P1, "Blaze");
    t.lands(P0, "Mountain", 6);
    let lib = t.library_size(P1);
    add_mana(&mut t, P0, ManaType::U, 9);
    let squall = t.hand(P0, "Sorcerous Squall");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    // The card is chosen in the graveyard after it's milled.
    choose_named_when_offered(&mut t, P0, "Blaze");
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.cast(P0, squall).go();
    t.resolve_all();
    assert_eq!(t.library_size(P1), lib.saturating_sub(9));
    let blaze_now = t.g.current(blaze);
    assert!(t.in_exile("Blaze"), "Blaze was cast and exiled");
    assert_ne!(blaze_now, blaze);
    assert!(t.on_battlefield(giant));
    assert_eq!(crate::r_s07_common::damage_on(&t, giant), 0, "X was 0");
    assert_eq!(tapped_lands(&t, P0), 0);
}

#[test]
fn sorcerous_squall_casts_without_paying_but_additional_costs_are_paid() {
    cr!("118.9a", "118.9b", "118.8a", "601.2b", "608.2g", "614.1a");
    ruling!(
        "Sorcerous Squall",
        "you can't pay any alternative costs. You can, however, pay additional costs. If the spell has any mandatory additional costs, those must be paid to cast the card."
    );
    ruling!(
        "Sorcerous Squall",
        "If you cast an instant or sorcery spell, you do so as part of the resolution of Sorcerous Squall. You can't wait to cast one later in the turn. Timing restrictions based on a spell's type are ignored."
    );
    supported("Sorcerous Squall");
    check_free_cast_costs(&FreeCaster {
        instants_only: false,
        place: squall,
        run: run_squall,
    });
    // The spell is exiled rather than put into P1's graveyard.
    let mut t = TestGame::new(2);
    let bolt = squall(&mut t, "Burst Lightning");
    run_squall(&mut t, bolt, &|t| {
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
        t.answer_targets(P0, &[Entity::Player(P1)]);
    });
    assert_eq!(t.life(P1), 18);
    assert!(t.in_exile("Burst Lightning"));
    assert!(!t.in_graveyard(P1, "Burst Lightning"));
    // Declining leaves the card in the graveyard: it can't be cast later.
    let mut t = TestGame::new(2);
    let bolt = squall(&mut t, "Burst Lightning");
    add_mana(&mut t, P0, ManaType::U, 9);
    let sq = t.hand(P0, "Sorcerous Squall");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[]);
    t.cast(P0, sq).go();
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Graveyard(P1));
    assert_eq!(t.life(P1), 20);
}

/// Deluxe Dragster crewed by Grizzly Bears attacks P1 unblocked; P0 targets `card` in
/// P1's graveyard with its trigger, `answers` queued for casting it.
fn run_dragster(t: &mut TestGame, card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    let dragster = t.named_on_battlefield("Deluxe Dragster")[0];
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(crate::r_s04_common::crew(t, P0, dragster, &[bears]));
    t.answer_targets(P0, &[Entity::Object(card)]);
    answers(t);
    attack_p1_unblocked(t, dragster);
    t.clear_answers();
}

fn dragster(t: &mut TestGame, name: &str) -> ObjectId {
    t.battlefield(P0, "Deluxe Dragster");
    t.battlefield(P0, "Grizzly Bears");
    t.graveyard(P1, name)
}

#[test]
fn deluxe_dragster_casts_without_paying_but_additional_costs_are_paid() {
    cr!("118.9a", "118.9b", "118.8a", "601.2b", "608.2g", "115.1");
    ruling!(
        "Deluxe Dragster",
        "If you cast a card \"without paying its mana cost,\" you can't pay any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, those must be paid to cast the card."
    );
    ruling!(
        "Deluxe Dragster",
        "If you cast the instant or sorcery card, you do so as part of the resolution of the triggered ability."
    );
    supported("Deluxe Dragster");
    check_free_cast_costs(&FreeCaster {
        instants_only: false,
        place: dragster,
        run: run_dragster,
    });
    // The target is a card in the damaged player's graveyard: a card in P0's own
    // graveyard can't be chosen, so the ability is removed from the stack.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Deluxe Dragster");
    t.battlefield(P0, "Grizzly Bears");
    let own = t.graveyard(P0, "Lava Spike");
    run_dragster(&mut t, own, &|t| {
        t.answer_targets(P0, &[Entity::Player(P1)]);
    });
    assert_eq!(t.zone(own), Zone::Graveyard(P0));
    assert_eq!(t.life(P1), 20 - 4, "only Deluxe Dragster's combat damage");
}

#[test]
fn deluxe_dragster_casts_a_spell_with_x_as_zero() {
    cr!("107.3b", "118.9");
    ruling!(
        "Deluxe Dragster",
        "If an exiled card has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    supported("Deluxe Dragster");
    let mut t = TestGame::new(2);
    let blaze = dragster(&mut t, "Blaze");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 6);
    run_dragster(&mut t, blaze, &|t| {
        t.answer(P0, DecisionKind::X, Answer::Number(5));
        t.answer_targets(P0, &[Entity::Object(giant)]);
    });
    assert!(t.in_exile("Blaze"));
    assert_eq!(crate::r_s07_common::damage_on(&t, giant), 0);
    assert_eq!(tapped_lands(&t, P0), 0);
    // Deluxe Dragster dealt its 4 combat damage.
    assert_eq!(t.life(P1), 16);
}

#[test]
fn sword_of_once_and_future_can_cast_a_card_just_surveilled_into_the_graveyard() {
    cr!("701.25a", "118.9", "608.2g");
    ruling!(
        "Sword of Once and Future",
        "The card you cast may be one you just put into the graveyard with surveil or one already in the graveyard."
    );
    supported("Sword of Once and Future");
    // "Whenever equipped creature deals combat damage to a player, surveil 2. Then you may
    // cast an instant or sorcery spell with mana value 2 or less from your graveyard
    // without paying its mana cost. If that spell would be put into your graveyard, exile
    // it instead."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s06_common::attach_new(&mut t, P0, "Sword of Once and Future", bears);
    let lib = stack_library(&mut t, P0, &["Lava Spike", "Forest"]);
    // Surveil: both cards go to the graveyard.
    t.answer(P0, DecisionKind::Surveil, Answer::Split(vec![], lib.clone()));
    choose_named_when_offered(&mut t, P0, "Lava Spike");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    // Grizzly Bears (4/4 equipped) deals 4; Lava Spike deals 3.
    attack_p1_unblocked(&mut t, bears);
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(t.in_exile("Lava Spike"), "the surveilled card was cast and exiled");
    assert_eq!(t.life(P1), 20 - 4 - 3);
}
