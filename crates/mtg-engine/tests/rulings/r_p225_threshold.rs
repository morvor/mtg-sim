//! Rulings batch P225 — threshold (an ability word, CR 207.2c): abilities granted
//! "as long as there are seven or more cards in your graveyard" trigger only if the
//! ability exists when the event happens (CR 603.2, 603.10a: leaves-the-battlefield
//! abilities look back in time), and once triggered they resolve even if the source loses
//! the ability (CR 113.7a); an intervening "if" is checked again on resolution (CR 603.4).

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use mtg_engine::ability::AbilityKind;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts `n` Grizzly Bears into `p`'s graveyard.
fn fill_graveyard(t: &mut TestGame, p: PlayerId, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.graveyard(p, "Grizzly Bears")).collect()
}

/// Exiles every card in `p`'s graveyard.
fn exile_graveyard(t: &mut TestGame, p: PlayerId) {
    let cards: Vec<ObjectId> = t.g.player(p).graveyard.clone();
    for c in cards {
        t.g.exile_object(c, None);
    }
    t.g.recompute();
    assert_eq!(t.graveyard_size(p), 0);
}

#[test]
fn threshold_etb_abilities_trigger_only_with_seven_cards_and_resolve_after_losing_it() {
    cr!("603.2", "603.6a", "113.7a");
    ruling!(
        "Centaur Chieftain",
        "at the moment Centaur Chieftain enters the battlefield, its threshold ability won"
    );
    ruling!(
        "Cephalid Sage",
        "at the moment Cephalid Sage enters the battlefield, its threshold ability won"
    );
    supported("Centaur Chieftain");
    supported("Cephalid Sage");
    // Six cards: neither enters ability triggers.
    for name in ["Centaur Chieftain", "Cephalid Sage"] {
        let mut t = TestGame::new(2);
        fill_graveyard(&mut t, P0, 6);
        t.enter(P0, name);
        t.settle();
        assert_eq!(t.stack_len(), 0, "{name} triggered with six cards");
    }
    // Centaur Chieftain with seven: it triggers; the graveyard is exiled in response, the
    // Chieftain loses the ability, and the ability still resolves.
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P0, 7);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let chief = t.enter(P0, "Centaur Chieftain");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    exile_graveyard(&mut t, P0);
    assert!(!t
        .obj_now(chief)
        .chars
        .abilities
        .iter()
        .any(|a| matches!(a.kind, AbilityKind::Triggered(_))));
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Trample));
    assert_eq!(t.pt(chief), (4, 4));
    // Cephalid Sage with seven: draw three, then discard two, after the graveyard is gone.
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P0, 7);
    for _ in 0..5 {
        t.library_top(P0, "Island");
    }
    t.enter(P0, "Cephalid Sage");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    exile_graveyard(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.graveyard_size(P0), 2);
}

#[test]
fn kioras_threshold_trigger_checks_the_graveyard_on_trigger_and_on_resolution() {
    cr!("603.4");
    ruling!(
        "Kiora, the Rising Tide",
        "Kiora's threshold ability checks your graveyard at the moment it would trigger"
    );
    supported("Kiora, the Rising Tide");
    let scions = |t: &TestGame| t.named_on_battlefield("Scion of the Deep").len();
    // Six cards: no trigger.
    let mut t = TestGame::new(2);
    let kiora = t.battlefield(P0, "Kiora, the Rising Tide");
    fill_graveyard(&mut t, P0, 6);
    attack_with(&mut t, &[(kiora, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 0);
    // Seven cards: it triggers, but the graveyard is exiled before it resolves.
    let mut t = TestGame::new(2);
    let kiora = t.battlefield(P0, "Kiora, the Rising Tide");
    fill_graveyard(&mut t, P0, 7);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(kiora, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    exile_graveyard(&mut t, P0);
    t.resolve_all();
    assert_eq!(scions(&t), 0);
    // Seven cards at both times: the Scion is created.
    let mut t = TestGame::new(2);
    let kiora = t.battlefield(P0, "Kiora, the Rising Tide");
    fill_graveyard(&mut t, P0, 7);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(kiora, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(scions(&t), 1);
}

#[test]
fn decaying_soil_looks_back_at_threshold_before_the_creatures_died() {
    cr!("603.10a", "603.2");
    ruling!(
        "Decaying Soil",
        "either all of them or none of them will cause the trigger to go off"
    );
    ruling!(
        "Decaying Soil",
        "So the one that causes Threshold to be met will not trigger the ability."
    );
    supported("Decaying Soil");
    let soil_triggers = |t: &TestGame| triggers_on_stack(t, "return that card to your hand");
    // Five cards, two creatures die at once (seven afterward): neither triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Decaying Soil");
    fill_graveyard(&mut t, P0, 5);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy_all(vec![a, b], None, false);
    t.settle();
    assert_eq!(t.graveyard_size(P0), 7);
    assert_eq!(soil_triggers(&t), 0);
    // Six cards, one creature dies (the seventh card): no trigger either.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Decaying Soil");
    fill_graveyard(&mut t, P0, 6);
    let a = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, a);
    assert_eq!(soil_triggers(&t), 0);
    // Seven cards, two creatures die at once: both trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Decaying Soil");
    fill_graveyard(&mut t, P0, 7);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy_all(vec![a, b], None, false);
    t.settle();
    assert_eq!(soil_triggers(&t), 2);
}

#[test]
fn threshold_dies_abilities_dont_count_the_dying_card() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Treacherous Werewolf",
        "The Threshold ability can’t count this card (soon to be in the graveyard)"
    );
    ruling!(
        "Reborn Hero",
        "you don't include Reborn Hero itself in the count"
    );
    supported("Treacherous Werewolf");
    supported("Reborn Hero");
    // Six cards: the dying card would be the seventh, but neither ability triggers.
    for name in ["Treacherous Werewolf", "Reborn Hero"] {
        let mut t = TestGame::new(2);
        fill_graveyard(&mut t, P0, 6);
        let c = t.battlefield(P0, name);
        destroy(&mut t, c);
        assert_eq!(t.graveyard_size(P0), 7);
        assert_eq!(t.stack_len(), 0, "{name} triggered");
    }
    // Seven cards: Treacherous Werewolf's controller loses 4 life.
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P0, 7);
    let w = t.battlefield(P0, "Treacherous Werewolf");
    assert_eq!(t.pt(w), (4, 4));
    destroy(&mut t, w);
    t.resolve_all();
    assert_eq!(t.life(P0), 16);
    // Seven cards: Reborn Hero returns for {W}{W}.
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P0, 7);
    t.lands(P0, "Plains", 2);
    let h = t.battlefield(P0, "Reborn Hero");
    destroy(&mut t, h);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Reborn Hero").len(), 1);
    assert_eq!(t.zone(h), Zone::Battlefield);
}
