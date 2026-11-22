//! CR 702.116 Myriad.

use crate::common_k702_111_124::*;
use mtg_engine::game::GameConfig;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn attackers_at(t: &TestGame, target: Entity) -> Vec<ObjectId> {
    t.g.combat
        .as_ref()
        .map(|c| {
            c.attackers
                .iter()
                .filter(|a| a.target == Some(target))
                .map(|a| a.id)
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn myriad_creates_attacking_token_copies_for_each_other_opponent() {
    cr!("702.116", "702.116a");
    assert_supported_card("Warchief Giant");
    let mut t = TestGame::new(4);
    // Warchief Giant: 5/3 haste, myriad.
    let giant = t.battlefield(P0, "Warchief Giant");
    declare_attack(&mut t, &[(giant, Entity::Player(P1))]);
    assert_eq!(on_stack(&t, "Myriad"), 1);
    t.resolve_all();
    let tokens = tokens_of(&t, P0);
    assert_eq!(tokens.len(), 2);
    for tok in &tokens {
        let o = t.g.obj(*tok);
        assert_eq!(o.chars.name, "Warchief Giant");
        assert!(o.tapped);
        assert!(t.g.is_attacking(*tok));
    }
    assert_eq!(attackers_at(&t, Entity::Player(P2)).len(), 1);
    assert_eq!(attackers_at(&t, Entity::Player(P3)).len(), 1);
    assert_eq!(attackers_at(&t, Entity::Player(P1)), vec![giant]);
    // Combat damage to each player; the tokens are exiled at end of combat.
    t.answer(P1, DecisionKind::Blockers, decision::Answer::Blockers(vec![]));
    to_step(&mut t, Step::EndOfCombat);
    assert_eq!((t.life(P1), t.life(P2), t.life(P3)), (15, 15, 15));
    t.resolve_all();
    assert!(tokens.iter().all(|x| !t.g.is_live(*x)));
    assert!(tokens_of(&t, P0).is_empty());
    assert!(t.on_battlefield(giant));
}

#[test]
fn with_only_one_opponent_no_tokens_are_created() {
    cr!("702.116a");
    ruling!(
        "Warchief Giant",
        "If the defending player is your only opponent, no tokens are put onto the battlefield."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Warchief Giant");
    declare_attack(&mut t, &[(giant, Entity::Player(P1))]);
    t.resolve_all();
    assert!(tokens_of(&t, P0).is_empty());
}

#[test]
fn creating_each_token_is_optional() {
    cr!("702.116a");
    let mut t = TestGame::new(4);
    let giant = t.battlefield(P0, "Warchief Giant");
    declare_attack(&mut t, &[(giant, Entity::Player(P2))]);
    // Opponents other than P2, in APNAP order: P1 (no), P3 (yes).
    t.answer_yes(P0, false);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0).len(), 1);
    assert_eq!(attackers_at(&t, Entity::Player(P3)).len(), 1);
    assert!(attackers_at(&t, Entity::Player(P1)).is_empty());
}

#[test]
fn a_token_can_attack_a_planeswalker_that_player_controls() {
    cr!("702.116a");
    ruling!(
        "Warchief Giant",
        "You choose whether each token is attacking the player or a planeswalker they control as the token is created."
    );
    let mut t = TestGame::new(3);
    let giant = t.battlefield(P0, "Warchief Giant");
    let jace = t.battlefield(P2, "Jace Beleren");
    declare_attack(&mut t, &[(giant, Entity::Player(P1))]);
    t.answer_choose(P0, &[Entity::Object(jace)]);
    t.resolve_all();
    let tok = tokens_of(&t, P0);
    assert_eq!(tok.len(), 1);
    assert_eq!(attackers_at(&t, Entity::Object(jace)), tok);
}

#[test]
fn the_tokens_werent_declared_as_attackers() {
    cr!("702.116a");
    ruling!(
        "Warchief Giant",
        "Although the tokens enter the battlefield attacking, they were never declared as attackers. Abilities that trigger whenever a creature attacks won’t trigger, including the myriad ability of the tokens."
    );
    let mut t = TestGame::new(4);
    let giant = t.battlefield(P0, "Warchief Giant");
    declare_attack(&mut t, &[(giant, Entity::Player(P1))]);
    t.resolve();
    assert_eq!(tokens_of(&t, P0).len(), 2);
    t.settle();
    assert_eq!(on_stack(&t, "Myriad"), 0);
    assert_eq!(tokens_of(&t, P0).len(), 2);
}

#[test]
fn the_tokens_copy_only_the_copiable_values() {
    cr!("702.116a");
    ruling!(
        "Warchief Giant",
        "Each token copies exactly what was printed on the original creature and nothing else."
    );
    let mut t = TestGame::new(3);
    let giant = t.battlefield(P0, "Warchief Giant");
    t.g.add_counters(Entity::Object(giant), counters::PLUS1, 2, None);
    t.g.recompute();
    assert_eq!(t.pt(giant), (7, 5));
    declare_attack(&mut t, &[(giant, Entity::Player(P1))]);
    t.resolve_all();
    let tok = tokens_of(&t, P0)[0];
    assert_eq!(t.pt(tok), (5, 3));
    assert!(has(&t, tok, KeywordKind::Myriad));
    assert!(has(&t, tok, KeywordKind::Haste));
}

#[test]
fn teammates_arent_opponents() {
    cr!("702.116a");
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            teams: Some(vec![0, 1, 0, 1]),
            ..Default::default()
        },
    );
    let giant = t.battlefield(P0, "Warchief Giant");
    declare_attack(&mut t, &[(giant, Entity::Player(P1))]);
    t.resolve_all();
    // Only P3 is an opponent other than the defending player.
    assert_eq!(tokens_of(&t, P0).len(), 1);
    assert_eq!(attackers_at(&t, Entity::Player(P3)).len(), 1);
    assert!(attackers_at(&t, Entity::Player(P2)).is_empty());
}

#[test]
fn each_instance_of_myriad_triggers_separately() {
    cr!("702.116b");
    ruling!(
        "Blade of Selves",
        "If a creature has multiple instances of myriad, each triggers separately. You'll get two tokens per opponent other than the defending player."
    );
    let mut t = TestGame::new(3);
    let giant = t.battlefield(P0, "Warchief Giant");
    gain(&mut t, P0, giant, Keyword::new(KeywordKind::Myriad));
    declare_attack(&mut t, &[(giant, Entity::Player(P1))]);
    assert_eq!(on_stack(&t, "Myriad"), 2);
    t.resolve_all();
    assert_eq!(attackers_at(&t, Entity::Player(P2)).len(), 2);
    to_step(&mut t, Step::EndOfCombat);
    t.resolve_all();
    assert!(tokens_of(&t, P0).is_empty());
    assert_eq!(t.zone(giant), Zone::Battlefield);
}
