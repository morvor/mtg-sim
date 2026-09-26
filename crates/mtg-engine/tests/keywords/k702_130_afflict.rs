//! CR 702.130 Afflict.

use crate::common_k702_125_139::*;
use mtg_engine::game::GameConfig;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn afflict_makes_the_defending_player_lose_life_when_it_becomes_blocked() {
    cr!("702.130", "702.130a");
    ruling!(
        "Khenra Eternal",
        "Afflict causes the defending player to lose life; it's not damage or combat damage."
    );
    assert_supported_card("Khenra Eternal");
    let mut t = TestGame::new(2);
    // Khenra Eternal: 2/2, afflict 1.
    let khenra = t.battlefield(P0, "Khenra Eternal");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(khenra, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(bears, khenra)]);
    assert_eq!(triggers_on_stack(&t, "Afflict 1"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // Life loss, not damage: a "damage can't be dealt" shield wouldn't stop it, and the
    // player wasn't dealt damage.
    assert!(t.g.history.life_lost.get(&P1).is_some_and(|n| *n == 1));
}

#[test]
fn afflict_doesnt_trigger_if_it_isnt_blocked() {
    cr!("702.130a");
    let mut t = TestGame::new(2);
    let khenra = t.battlefield(P0, "Khenra Eternal");
    t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(khenra, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[]);
    assert_eq!(triggers_on_stack(&t, "Afflict 1"), 0);
    t.advance_to(P0, Step::EndOfCombat);
    // Only its combat damage.
    assert_eq!(t.life(P1), 18);
}

#[test]
fn afflict_triggers_once_however_many_creatures_block_it() {
    cr!("702.130a");
    ruling!(
        "Khenra Eternal",
        "If multiple creatures block a creature with afflict, afflict triggers only once."
    );
    let mut t = TestGame::new(2);
    let wildfire = t.battlefield(P0, "Wildfire Eternal");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(wildfire, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(b1, wildfire), (b2, wildfire)]);
    assert_eq!(triggers_on_stack(&t, "Afflict 4"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn each_instance_of_afflict_triggers_separately() {
    cr!("702.130b");
    let mut t = TestGame::new(2);
    // Lost Monarch of Ifnir: "Other Zombies you control have afflict 3." Khenra Eternal
    // (a Zombie) has afflict 1 and afflict 3.
    t.battlefield(P0, "Lost Monarch of Ifnir");
    let khenra = t.battlefield(P0, "Khenra Eternal");
    assert_eq!(keyword_count(&t, khenra, KeywordKind::Afflict), 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(khenra, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(bears, khenra)]);
    assert_eq!(triggers_on_stack(&t, "Afflict 1"), 1);
    assert_eq!(triggers_on_stack(&t, "Afflict 3"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn the_life_loss_happens_before_combat_damage_and_can_end_the_game() {
    cr!("702.130a");
    ruling!(
        "Khenra Eternal",
        "Afflict resolves before combat damage is dealt. If this loss of life brings a player to 0 life or less, that player loses the game immediately."
    );
    let mut t = TestGame::new(2);
    let wildfire = t.battlefield(P0, "Wildfire Eternal");
    t.g.players[1].life = 4;
    // A lifelink blocker won't deal combat damage in time.
    let blocker = t.battlefield(P1, "Grizzly Bears");
    gain(&mut t, P1, blocker, Keyword::new(KeywordKind::Lifelink));
    attack_with(&mut t, &[(wildfire, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(blocker, wildfire)]);
    t.resolve_all();
    t.settle();
    assert!(t.has_lost(P1));
}

#[test]
fn the_defending_player_is_the_controller_of_the_attacked_planeswalker() {
    cr!("702.130a");
    ruling!(
        "Khenra Eternal",
        "If a creature is attacking a planeswalker, that planeswalker's controller is the defending player."
    );
    let mut t = TestGame::with_config(
        3,
        GameConfig {
            ..Default::default()
        },
    );
    let khenra = t.battlefield(P0, "Khenra Eternal");
    let jace = t.battlefield(P2, "Jace Beleren");
    let bears = t.battlefield(P2, "Grizzly Bears");
    attack_with(&mut t, &[(khenra, Entity::Object(jace))]);
    declare_blocks(&mut t, P2, &[(bears, khenra)]);
    t.resolve_all();
    assert_eq!(t.life(P2), 19);
    assert_eq!(t.life(P1), 20);
}
