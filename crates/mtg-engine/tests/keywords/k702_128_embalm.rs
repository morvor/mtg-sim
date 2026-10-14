//! CR 702.128 Embalm.

use crate::common_k702_125_139::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn embalm_exiles_the_card_and_creates_a_white_zombie_token_copy() {
    cr!("702.128", "702.128a");
    assert_supported_card("Sacred Cat");
    let mut t = TestGame::new(2);
    // Sacred Cat: {W} 1/1 Cat with lifelink; embalm {W}.
    t.lands(P0, "Plains", 1);
    let cat = t.graveyard(P0, "Sacred Cat");
    assert!(can_activate_now(&mut t, P0, cat));
    t.activate(P0, cat, 0, &[]).unwrap();
    // Exiling the card is part of the cost.
    assert!(t.in_exile("Sacred Cat"));
    t.resolve_all();
    let tokens = tokens_of(&t, P0);
    assert_eq!(tokens.len(), 1);
    let tok = t.obj_now(tokens[0]);
    assert_eq!(tok.chars.name, "Sacred Cat");
    assert_eq!(tok.chars.colors, ColorSet::single(Color::White));
    assert!(tok.chars.mana_cost.is_none());
    assert_eq!(t.g.mana_value_of(tokens[0]), 0);
    assert!(tok.chars.has_subtype("Zombie") && tok.chars.has_subtype("Cat"));
    assert!(tok.chars.is(CardType::Creature));
    assert!(tok.chars.has_keyword(KeywordKind::Lifelink));
    assert_eq!(t.pt(tokens[0]), (1, 1));
}

#[test]
fn embalm_is_activated_only_from_the_graveyard_as_a_sorcery() {
    cr!("702.128a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let cat = t.hand(P0, "Sacred Cat");
    assert!(!can_activate_now(&mut t, P0, cat));
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let cat = t.graveyard(P0, "Sacred Cat");
    t.set_step(P0, Step::Upkeep);
    assert!(!can_activate_now(&mut t, P0, cat));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_activate_now(&mut t, P0, cat));
    t.set_step(P0, Step::PostcombatMain);
    assert!(can_activate_now(&mut t, P0, cat));
}

#[test]
fn the_token_has_the_cards_enters_abilities() {
    cr!("702.128a");
    ruling!(
        "Vizier of Many Faces",
        "If the card copied by the token had any \"when [this permanent] enters the battlefield\" abilities, then the token also has those abilities and will trigger them when it's created."
    );
    assert_supported_card("Angel of Sanctions");
    let mut t = TestGame::new(2);
    // Angel of Sanctions: "When this creature enters, you may exile target nonland
    // permanent an opponent controls until this creature leaves the battlefield."; embalm
    // {5}{W}.
    t.lands(P0, "Plains", 6);
    let giant = t.battlefield(P1, "Hill Giant");
    let angel = t.graveyard(P0, "Angel of Sanctions");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer_yes(P0, true);
    t.activate(P0, angel, 0, &[]).unwrap();
    t.resolve_all();
    let tokens = tokens_of(&t, P0);
    assert_eq!(tokens.len(), 1);
    assert_eq!(t.obj_now(tokens[0]).chars.name, "Angel of Sanctions");
    assert!(!t.on_battlefield(giant));
    assert!(t.in_exile("Hill Giant"));
    // Anointer Priest's token sees itself enter: "Whenever a creature token you control
    // enters, you gain 1 life."
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    let priest = t.graveyard(P0, "Anointer Priest");
    t.activate(P0, priest, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0).len(), 1);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn a_token_created_by_an_embalm_ability_is_embalmed() {
    cr!("702.128b");
    assert_supported_card("Vizier of Many Faces");
    ruling!(
        "Vizier of Many Faces",
        "The token is a Zombie in addition to its other types and is white instead of its other colors. It has no mana cost, and thus its mana value is 0."
    );
    // Vizier of Many Faces: "You may have this creature enter as a copy of any creature on
    // the battlefield, except if this creature was embalmed, the token has no mana cost,
    // it's white, and it's a Zombie in addition to its other types."; embalm {3}{U}{U}.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let giant = t.battlefield(P1, "Hill Giant");
    let vizier = t.graveyard(P0, "Vizier of Many Faces");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.activate(P0, vizier, 0, &[]).unwrap();
    t.resolve_all();
    let tokens = tokens_of(&t, P0);
    assert_eq!(tokens.len(), 1);
    let tok = t.obj_now(tokens[0]);
    assert_eq!(tok.chars.name, "Hill Giant");
    assert!(tok.chars.has_subtype("Zombie") && tok.chars.has_subtype("Giant"));
    assert_eq!(tok.chars.colors, ColorSet::single(Color::White));
    assert!(tok.chars.mana_cost.is_none());
    assert_eq!(t.pt(tokens[0]), (3, 3));
    // A Vizier cast from the hand wasn't embalmed: a plain copy.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let giant = t.battlefield(P1, "Hill Giant");
    let vizier = t.hand(P0, "Vizier of Many Faces");
    t.cast(P0, vizier).go();
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.resolve_all();
    let v = t.obj_now(vizier);
    assert_eq!(v.chars.name, "Hill Giant");
    assert!(!v.chars.has_subtype("Zombie"));
    assert_eq!(v.chars.colors, ColorSet::single(Color::Red));
    assert!(v.chars.mana_cost.is_some());
}

#[test]
fn the_embalmed_tokens_exceptions_are_copiable() {
    cr!("702.128a");
    ruling!(
        "Vizier of Many Faces",
        "These are copiable values of the token that other effects may copy."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let cat = t.graveyard(P0, "Sacred Cat");
    t.activate(P0, cat, 0, &[]).unwrap();
    t.resolve_all();
    let tok = tokens_of(&t, P0)[0];
    // A clone of the embalmed token is a white Zombie Cat with no mana cost too.
    t.answer_choose(P0, &[Entity::Object(tok)]);
    let clone = t.enter(P0, "Clone");
    let c = t.obj_now(clone);
    assert_eq!(c.chars.name, "Sacred Cat");
    assert!(c.chars.has_subtype("Zombie"));
    assert_eq!(c.chars.colors, ColorSet::single(Color::White));
    assert!(c.chars.mana_cost.is_none());
}

#[test]
fn a_granted_embalm_ability_costs_the_cards_mana_cost() {
    cr!("702.128a");
    assert_supported_card("Cursecloth Wrappings");
    let mut t = TestGame::new(2);
    // Cursecloth Wrappings: "Zombies you control get +1/+1. {T}: Target creature card in
    // your graveyard gains embalm until end of turn. The embalm cost is equal to its mana
    // cost."
    let wraps = t.battlefield(P0, "Cursecloth Wrappings");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.activate(P0, wraps, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert!(has(&t, bears, KeywordKind::Embalm));
    t.lands(P0, "Forest", 2);
    assert!(can_activate_now(&mut t, P0, bears));
    t.activate(P0, bears, 0, &[]).unwrap();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    let tok = tokens_of(&t, P0)[0];
    assert_eq!(t.obj_now(tok).chars.name, "Grizzly Bears");
    assert!(t.obj_now(tok).chars.has_subtype("Zombie"));
    // A white Zombie Bear, and a Zombie you control: 3/3.
    assert_eq!(t.pt(tok), (3, 3));
}

#[test]
fn whenever_you_activate_an_embalm_or_eternalize_ability() {
    cr!("702.128a", "702.129a");
    assert_line_supported("Vizier of the Anointed", "whenever you activate");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vizier of the Anointed");
    t.lands(P0, "Plains", 1);
    let cat = t.graveyard(P0, "Sacred Cat");
    let hand = t.hand_size(P0);
    t.activate(P0, cat, 0, &[]).unwrap();
    t.settle();
    // The draw trigger resolves first.
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0).len(), 1);
    // An eternalize ability triggers it too.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vizier of the Anointed");
    t.lands(P0, "Island", 6);
    let pc = t.graveyard(P0, "Proven Combatant");
    let hand = t.hand_size(P0);
    t.activate(P0, pc, 0, &[]).unwrap();
    t.settle();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Other activated abilities don't trigger it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vizier of the Anointed");
    let wraps = t.battlefield(P0, "Cursecloth Wrappings");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.activate(P0, wraps, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}
