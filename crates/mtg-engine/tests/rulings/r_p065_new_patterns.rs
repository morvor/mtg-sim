//! Rulings batch P065 — abilities the compiler learned for this batch: "spells and
//! abilities can't be countered" (Spider-Punk, CR 701.6), "as long as it attacked a battle
//! this turn" (War Historian, CR 508.1b), "if that spell was kicked" in a cast trigger
//! (Bloodstone Goblin, CR 603.4, 702.33d), and "as long as you own a card in exile that has
//! an Adventure" (Howling Galefang, CR 715.2a).

use crate::r_s01_common::supported;
use crate::r_s02_common::{can_attack, destroy};
use crate::r_s05_common::move_to;
use crate::r_s06_common::has_kw;
use crate::r_s09_common::{declare, to_combat};
use crate::r_s14_common::triggers_from;
use crate::r_s18_common::ADVENTURE;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn spider_punk_spells_can_still_be_targeted_by_counterspells_but_arent_countered() {
    cr!("701.6a", "701.6b", "608.2b");
    ruling!(
        "Spider-Punk",
        "While Spider-Punk is on the battlefield, spells and abilities can still be the target of another spell or ability that would normally counter them, but they won't be countered as that spell or ability resolves."
    );
    supported("Spider-Punk");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Spider-Punk");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    let spell = t.cast(P0, bolt).target(Entity::Player(P1)).go();
    let cs = t.hand(P1, "Counterspell");
    t.lands(P1, "Island", 2);
    // The Bolt is a legal target for Counterspell.
    t.cast(P1, cs).target(Entity::Object(spell)).go();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert!(t.in_graveyard(P1, "Counterspell"), "Counterspell resolved");
    assert_eq!(t.stack_len(), 1, "the Bolt wasn't countered");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Without Spider-Punk, it would have been countered.
    let mut t = TestGame::new(2);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    let spell = t.cast(P0, bolt).target(Entity::Player(P1)).go();
    let cs = t.hand(P1, "Counterspell");
    t.lands(P1, "Island", 2);
    t.cast(P1, cs).target(Entity::Object(spell)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn spider_punk_abilities_cant_be_countered_either() {
    cr!("701.6a", "701.6b");
    ruling!(
        "Spider-Punk",
        "While Spider-Punk is on the battlefield, spells and abilities can still be the target of another spell or ability"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spider-Punk");
    let shaman = t.enter(P1, "Nantuko Shaman");
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_from(&t, shaman), 1);
    let ability = *t.g.stack.last().unwrap();
    assert!(!t.g.counter(ability, None));
    let hand = t.hand_size(P1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 1);
}

/// Invasion of Segovia, a Siege P0 controls and P1 protects.
fn segovia(t: &mut TestGame) -> ObjectId {
    t.answer_choose(P0, &[Entity::Player(P1)]);
    let id = t.enter(P0, "Invasion of Segovia // Caetus, Sea Tyrant of Segovia");
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(mtg_engine::battle::protector(&t.g, id), Some(P1));
    id
}

#[test]
fn war_historian_is_indestructible_all_turn_after_attacking_a_battle() {
    cr!("508.1b", "702.12b", "611.3a");
    ruling!(
        "War Historian",
        "War Historian's last ability starts to apply as soon as it's declared as an attacker that's attacking a battle, and it applies for the entire turn. It doesn't matter what happens to the battle after that point."
    );
    supported("War Historian");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "War Historian");
    let battle = segovia(&mut t);
    assert!(!has_kw(&t, w, KeywordKind::Indestructible));
    to_combat(&mut t, P0);
    declare(&mut t, P0, &[(w, Entity::Object(battle))]);
    t.g.recompute();
    assert!(has_kw(&t, w, KeywordKind::Indestructible));
    // The battle leaves; War Historian keeps indestructible for the turn.
    move_to(&mut t, battle, Zone::Exile);
    t.g.recompute();
    assert!(has_kw(&t, w, KeywordKind::Indestructible));
    t.advance_to(P0, Step::End);
    destroy(&mut t, w);
    assert!(t.on_battlefield(w));
    // Not in the next turn.
    t.advance_to(P1, Step::Upkeep);
    assert!(!has_kw(&t, w, KeywordKind::Indestructible));
    // Attacking a player isn't attacking a battle.
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "War Historian");
    segovia(&mut t);
    let attacks = declare(&mut t, P0, &[(w, Entity::Player(P1))]);
    assert_eq!(attacks, vec![(w, Entity::Player(P1))]);
    t.g.recompute();
    assert!(!has_kw(&t, w, KeywordKind::Indestructible));
}

#[test]
fn bloodstone_goblin_resolves_first_even_if_the_kicked_spell_is_countered() {
    cr!("603.3", "405.5", "702.33d");
    ruling!(
        "Bloodstone Goblin",
        "Bloodstone Goblin’s last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    supported("Bloodstone Goblin");
    // A kicked spell: the trigger is above it and resolves first.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Bloodstone Goblin");
    let burst = t.hand(P0, "Burst Lightning");
    t.lands(P0, "Mountain", 5);
    t.cast(P0, burst)
        .kicked(true)
        .target(Entity::Player(P1))
        .go();
    t.settle();
    assert_eq!(triggers_from(&t, g), 1);
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.pt(g), (3, 3));
    assert!(has_kw(&t, g, KeywordKind::Menace));
    assert_eq!(t.life(P1), 20, "the spell hasn't resolved yet");
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // The spell is countered first: the ability still resolves.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Bloodstone Goblin");
    let burst = t.hand(P0, "Burst Lightning");
    t.lands(P0, "Mountain", 5);
    let spell = t
        .cast(P0, burst)
        .kicked(true)
        .target(Entity::Player(P1))
        .go();
    t.settle();
    assert!(t.g.counter(spell, None));
    t.resolve_all();
    assert_eq!(t.pt(g), (3, 3));
    assert!(has_kw(&t, g, KeywordKind::Menace));
    assert_eq!(t.life(P1), 20);
    // An unkicked spell doesn't trigger it.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Bloodstone Goblin");
    let burst = t.hand(P0, "Burst Lightning");
    t.lands(P0, "Mountain", 5);
    t.cast(P0, burst)
        .kicked(false)
        .target(Entity::Player(P1))
        .go();
    t.settle();
    assert_eq!(triggers_from(&t, g), 0);
}

#[test]
fn howling_galefang_has_haste_with_any_adventurer_card_you_own_in_exile() {
    cr!("715.2a", "702.10b");
    ruling!(
        "Howling Galefang",
        "Howling Galefang will have haste as long as any card you own in exile has an Adventure. It doesn't matter if that card was cast as an Adventure or not."
    );
    supported("Howling Galefang");
    // Exiled without being cast as an Adventure.
    let mut t = TestGame::new(2);
    let h = t.battlefield_sick(P0, "Howling Galefang");
    assert!(!has_kw(&t, h, KeywordKind::Haste));
    t.exile(P1, "Bonecrusher Giant // Stomp");
    t.g.recompute();
    assert!(!has_kw(&t, h, KeywordKind::Haste), "an opponent's card");
    t.exile(P0, "Grizzly Bears");
    t.g.recompute();
    assert!(!has_kw(&t, h, KeywordKind::Haste), "no Adventure");
    let giant = t.exile(P0, "Bonecrusher Giant // Stomp");
    t.g.recompute();
    assert!(has_kw(&t, h, KeywordKind::Haste));
    to_combat(&mut t, P0);
    assert!(can_attack(&mut t, h));
    move_to(&mut t, giant, Zone::Graveyard(P0));
    t.g.recompute();
    assert!(!has_kw(&t, h, KeywordKind::Haste));
    // Cast as an Adventure, and exiled as it resolves.
    let mut t = TestGame::new(2);
    let h = t.battlefield_sick(P0, "Howling Galefang");
    let giant = t.hand(P0, "Bonecrusher Giant // Stomp");
    t.lands(P0, "Mountain", 2);
    t.cast(P0, giant)
        .method(ADVENTURE)
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert!(t.in_exile("Bonecrusher Giant"));
    assert!(has_kw(&t, h, KeywordKind::Haste));
}
