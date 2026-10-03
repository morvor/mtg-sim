//! CR 702.129 Eternalize.

use crate::common_k702_125_139::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn eternalize_creates_a_black_4_4_zombie_token_copy_with_no_mana_cost() {
    cr!("702.129", "702.129a");
    assert_supported_card("Adorned Pouncer");
    let mut t = TestGame::new(2);
    // Adorned Pouncer: {1}{W} 1/1 Cat with double strike; eternalize {3}{W}{W}.
    t.lands(P0, "Plains", 5);
    let pouncer = t.graveyard(P0, "Adorned Pouncer");
    assert!(can_activate_now(&mut t, P0, pouncer));
    t.activate(P0, pouncer, 0, &[]).unwrap();
    assert!(t.in_exile("Adorned Pouncer"));
    t.resolve_all();
    let tokens = tokens_of(&t, P0);
    assert_eq!(tokens.len(), 1);
    let tok = t.obj_now(tokens[0]);
    assert_eq!(tok.chars.name, "Adorned Pouncer");
    assert_eq!(tok.chars.colors, ColorSet::single(Color::Black));
    assert!(tok.chars.mana_cost.is_none());
    assert!(tok.chars.has_subtype("Zombie") && tok.chars.has_subtype("Cat"));
    assert!(tok.chars.has_keyword(KeywordKind::DoubleStrike));
    assert_eq!(t.pt(tokens[0]), (4, 4));
}

#[test]
fn eternalize_is_a_sorcery_speed_graveyard_ability_and_its_token_isnt_embalmed() {
    cr!("702.129a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    let pc = t.graveyard(P0, "Proven Combatant");
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_activate_now(&mut t, P0, pc));
    t.set_step(P0, Step::PrecombatMain);
    assert!(can_activate_now(&mut t, P0, pc));
    t.activate(P0, pc, 0, &[]).unwrap();
    t.resolve_all();
    let tok = tokens_of(&t, P0)[0];
    assert_eq!(t.pt(tok), (4, 4));
    assert!(!mtg_engine::kw::embalm::is_embalmed(&t.g, tok));
}

#[test]
fn the_tokens_power_is_4_as_its_enters_ability_resolves() {
    cr!("702.129a");
    assert_supported_card("Sunscourge Champion");
    // Sunscourge Champion: 2/3, "When this creature enters, you gain life equal to its
    // power."; eternalize—{2}{W}{W}, discard a card.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    let champ = t.graveyard(P0, "Sunscourge Champion");
    let fodder = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    t.activate(P0, champ, 0, &[]).unwrap();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0).len(), 1);
    assert_eq!(t.life(P0), 24);
}
