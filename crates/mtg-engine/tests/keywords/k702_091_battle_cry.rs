//! CR 702.91 Battle cry.

use crate::common_k702_011_017::{assert_supported, attack_with, bf, custom_card};
use crate::common_k702_018_026::{declare_blocks, triggers_on_stack};
use mtg_engine::decision::Answer;
use mtg_engine::object::StackKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn battle_cry_pumps_each_other_attacking_creature_until_end_of_turn() {
    cr!("702.91", "702.91a");
    assert_supported("Goblin Wardriver");
    let mut t = TestGame::new(2);
    let wardriver = t.battlefield(P0, "Goblin Wardriver");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let home = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    attack_with(
        &mut t,
        &[
            (wardriver, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
            (elves, Entity::Player(P1)),
        ],
    );
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Battle Cry"), 1);
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 2));
    assert_eq!(t.pt(elves), (2, 1));
    // Not the creature with battle cry, nor creatures that aren't attacking.
    assert_eq!(t.pt(wardriver), (2, 2));
    assert_eq!(t.pt(home), (3, 3));
    assert_eq!(t.pt(theirs), (2, 2));
    declare_blocks(&mut t, P1, &[]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20 - 2 - 3 - 2);
    // Until end of turn.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn two_creatures_with_battle_cry_pump_each_other() {
    cr!("702.91a");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Goblin Wardriver");
    let b = t.battlefield(P0, "Accorder Paladin");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(
        &mut t,
        &[
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
        ],
    );
    t.resolve_all();
    assert_eq!(t.pt(a), (3, 2));
    assert_eq!(t.pt(b), (4, 1));
    assert_eq!(t.pt(bears), (4, 2));
}

#[test]
fn each_instance_of_battle_cry_triggers_separately() {
    cr!("702.91b");
    let def = custom_card(
        "Twice-Crying Goblin",
        "Creature — Goblin",
        Some((1, 1)),
        "Battle cry\nBattle cry",
    );
    let mut t = TestGame::new(2);
    let goblin = bf(&mut t, P0, def);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(
        &mut t,
        &[(goblin, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Battle Cry"), 2);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    assert_eq!(t.pt(goblin), (1, 1));
}

#[test]
fn creatures_put_onto_the_battlefield_attacking_before_it_resolves_get_the_bonus() {
    cr!("702.91a", "611.2c");
    ruling!(
        "Hero of Bladehold",
        "If the token-creating ability resolves first, the tokens each get +1/+0 until end of turn from the battle cry ability."
    );
    assert_supported("Hero of Bladehold");
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, "Hero of Bladehold");
    // Put battle cry on the stack first (bottom), then the token ability on top.
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![1, 0]));
    attack_with(&mut t, &[(hero, Entity::Player(P1))]);
    t.settle();
    let top = *t.g.stack.last().unwrap();
    let top_text = match &t.g.obj(top).stack.as_ref().unwrap().kind {
        StackKind::Triggered { ability, .. } => ability.text.clone(),
        _ => panic!("expected a trigger"),
    };
    assert_ne!(top_text, "Battle Cry");
    t.resolve_all();
    let tokens: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|o| o.is_token() && o.controller == P0)
        .map(|o| o.id)
        .collect();
    assert_eq!(tokens.len(), 2);
    for tok in tokens {
        assert!(t.g.is_attacking(tok));
        assert_eq!(t.pt(tok), (2, 1));
    }
    assert_eq!(t.pt(hero), (3, 4));
}

#[test]
fn creatures_put_onto_the_battlefield_attacking_after_it_resolves_dont_get_the_bonus() {
    cr!("702.91a");
    ruling!(
        "Hero of Bladehold",
        "Whenever Hero of Bladehold attacks, both abilities will trigger. You can put them onto the stack in any order."
    );
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, "Hero of Bladehold");
    // Battle cry on top: it resolves before the tokens exist.
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![0, 1]));
    attack_with(&mut t, &[(hero, Entity::Player(P1))]);
    t.settle();
    let top = *t.g.stack.last().unwrap();
    let top_text = match &t.g.obj(top).stack.as_ref().unwrap().kind {
        StackKind::Triggered { ability, .. } => ability.text.clone(),
        _ => panic!("expected a trigger"),
    };
    assert_eq!(top_text, "Battle Cry");
    t.resolve_all();
    let tokens: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|o| o.is_token() && o.controller == P0)
        .map(|o| o.id)
        .collect();
    assert_eq!(tokens.len(), 2);
    for tok in tokens {
        assert!(t.g.is_attacking(tok));
        assert_eq!(t.pt(tok), (1, 1));
    }
    // The Hero itself isn't "each other attacking creature".
    assert_eq!(t.pt(hero), (3, 4));
}
