//! Rulings batch P221 — raid ("if you attacked this turn", an ability word, CR 207.2c):
//! a conditional enter-as-a-copy replacement (Protean Raider, CR 707.9, 614.12), raid
//! triggers, and playing a card exiled by a raid ability.

use crate::r_s01_common::*;
use crate::r_s02_common::{can_play_land, destroy};
use crate::r_s03_common::{choice_candidates, respond};
use crate::r_s14_common::cast_from_hand;
use crate::r_s24_common::{enter_together, pool};
use crate::r_s26_common::dress_up;
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 attacks P1 with a Grizzly Bears (unblocked) and the game moves on to P0's
/// postcombat main phase.
fn attack_then_main_2(t: &mut TestGame) -> ObjectId {
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(t, &[(bears, Entity::Player(P1))]);
    block_and_finish(t, P1, &[]);
    t.advance_to(P0, Step::PostcombatMain);
    bears
}

const COPY: &str = "Choose an object to copy";

/// Protean Raider enters for P0 (after `answer_choose` picks what it copies).
fn raider_copying(t: &mut TestGame, what: ObjectId) -> ObjectId {
    t.answer_choose(P0, &[Entity::Object(what)]);
    let r = t.enter(P0, "Protean Raider");
    t.resolve_all();
    r
}

#[test]
fn protean_raider_copies_only_if_you_attacked_and_gets_the_copied_etb_abilities() {
    cr!("707.9", "614.12", "603.6a", "614.1c");
    ruling!(
        "Protean Raider",
        "Any enters-the-battlefield abilities of the copied creature will trigger when Protean Raider enters the battlefield. Any \"as [this creature] enters the battlefield\" or \"[this creature] enters the battlefield with\" abilities of the chosen creature will also work."
    );
    supported("Protean Raider");
    // Without an attack this turn, it just enters as itself (no choice offered).
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P1, "Hill Giant");
    let from = t.asked().len();
    let r = t.enter(P0, "Protean Raider");
    assert!(choice_candidates(&t, from, COPY).is_empty());
    assert_eq!(t.obj_now(r).chars.name, "Protean Raider");
    // After attacking: a copy of Elvish Visionary ("When this creature enters, draw a
    // card.") draws a card.
    let mut t = TestGame::new(2);
    attack_then_main_2(&mut t);
    let visionary = t.battlefield(P1, "Elvish Visionary");
    let hand = t.hand_size(P0);
    let r = raider_copying(&mut t, visionary);
    assert_eq!(t.obj_now(r).chars.name, "Elvish Visionary");
    assert_eq!(t.hand_size(P0), hand + 1);
    // A copy of Phantom Centaur ("This creature enters with three +1/+1 counters on it.").
    let mut t = TestGame::new(2);
    attack_then_main_2(&mut t);
    let centaur = t.enter(P1, "Phantom Centaur");
    let r = raider_copying(&mut t, centaur);
    assert_eq!(t.obj_now(r).chars.name, "Phantom Centaur");
    assert_eq!(t.counters(r, counters::PLUS1), 3);
}

#[test]
fn protean_raider_cant_copy_a_creature_entering_with_it() {
    cr!("707.9", "614.12");
    ruling!(
        "Protean Raider",
        "If Protean Raider somehow enters the battlefield at the same time as another creature, Protean Raider can't become a copy of that creature. You may choose only a creature that's already on the battlefield."
    );
    let mut t = TestGame::new(2);
    attack_then_main_2(&mut t);
    let giant = t.battlefield(P1, "Hill Giant");
    let from = t.asked().len();
    let entered = enter_together(&mut t, &[(P0, "Protean Raider"), (P0, "Craw Wurm")]);
    let wurm = entered[1];
    let c = choice_candidates(&t, from, COPY);
    assert_eq!(c.len(), 1);
    assert!(c[0].contains(&Entity::Object(giant)));
    assert!(!c[0].iter().any(|e| matches!(e, Entity::Object(o) if t.g.current(*o) == t.g.current(wurm))));
}

#[test]
fn protean_raider_copying_a_token_isnt_a_token() {
    cr!("707.9", "707.2", "111.1");
    ruling!(
        "Protean Raider",
        "If the chosen creature is a token, Protean Raider copies the original characteristics of that token as stated by the effect that created the token. Protean Raider is not a token in this case."
    );
    let mut t = TestGame::new(2);
    attack_then_main_2(&mut t);
    let raise = t.hand(P1, "Raise the Alarm");
    give_mana_for(&mut t, P1, "Raise the Alarm");
    t.cast(P1, raise).go();
    t.resolve_all();
    let soldier = t.named_on_battlefield("Soldier Token")[0];
    dress_up(&mut t, soldier);
    let r = raider_copying(&mut t, soldier);
    let o = t.obj_now(r);
    assert_eq!(o.chars.name, "Soldier Token");
    assert!(o.chars.subtypes.iter().any(|s| s == "Soldier"));
    assert!(!o.is_token());
    assert_eq!(t.pt(r), (1, 1));
}

#[test]
fn protean_raider_copying_a_copy_becomes_what_it_copied() {
    cr!("707.9", "707.3", "706.2");
    ruling!(
        "Protean Raider",
        "If the chosen creature is copying something else (for example, if the chosen creature is another Protean Raider), then Protean Raider enters the battlefield as whatever the chosen creature copied."
    );
    let mut t = TestGame::new(2);
    attack_then_main_2(&mut t);
    let giant = t.battlefield(P1, "Hill Giant");
    // A first Protean Raider copying Hill Giant.
    let first = raider_copying(&mut t, giant);
    assert_eq!(t.obj_now(first).chars.name, "Hill Giant");
    let second = raider_copying(&mut t, first);
    assert_eq!(t.obj_now(second).chars.name, "Hill Giant");
    assert_eq!(t.pt(second), (3, 3));
}

#[test]
fn protean_raider_copies_only_the_printed_values() {
    cr!("707.2", "707.9", "613.2");
    ruling!(
        "Protean Raider",
        "Protean Raider copies exactly what was printed on the original creature (unless that creature is copying something else or is a token; see below). It doesn't copy whether that creature is tapped or untapped, whether it has any counters on it or Auras attached to it, or any non-copy effects that have changed its power, toughness, types, color, or so on."
    );
    let mut t = TestGame::new(2);
    attack_then_main_2(&mut t);
    let giant = t.battlefield(P1, "Hill Giant");
    // Tapped, with two +1/+1 counters, +3/+3 and green and blue until end of turn, and
    // enchanted by Holy Strength.
    dress_up(&mut t, giant);
    let strength = t.battlefield(P1, "Holy Strength");
    assert!(t.g.attach(strength, Entity::Object(giant)));
    let r = raider_copying(&mut t, giant);
    let o = t.obj_now(r);
    assert_eq!(o.chars.name, "Hill Giant");
    assert!(!o.tapped);
    assert_eq!(t.counters(r, counters::PLUS1), 0);
    assert_eq!(t.pt(r), (3, 3));
    assert!(o.chars.colors.contains(Color::Red));
    assert!(!o.chars.colors.contains(Color::Green));
}

#[test]
fn mardu_warshriekers_raid_ability_uses_the_stack() {
    cr!("605.1b", "605.5a", "603.3", "106.4");
    ruling!(
        "Mardu Warshrieker",
        "Mardu Warshrieker's raid ability isn't a mana ability even though it adds mana. It uses the stack and can be responded to."
    );
    supported("Mardu Warshrieker");
    let mut t = TestGame::new(2);
    attack_then_main_2(&mut t);
    cast_from_hand(&mut t, P0, "Mardu Warshrieker", &[]);
    t.resolve();
    // The creature resolved; its trigger is on the stack and no mana was added yet.
    assert_eq!(t.named_on_battlefield("Mardu Warshrieker").len(), 1);
    assert_eq!(triggers_on_stack(&t, "add"), 1);
    assert_eq!(pool(&t, P0, ManaType::R), 0);
    t.resolve();
    assert_eq!(pool(&t, P0, ManaType::R), 1);
    assert_eq!(pool(&t, P0, ManaType::W), 1);
    assert_eq!(pool(&t, P0, ManaType::B), 1);
}

#[test]
fn raid_cares_only_that_you_attacked_with_a_creature_whatever_happened_since() {
    cr!("508.1", "603.4", "104.3a");
    ruling!(
        "Fire Nation Raider",
        "Raid abilities care only that you attacked with a creature. It doesn't matter how many creatures you attacked with, or which opponent or planeswalker controlled by an opponent those creatures attacked."
    );
    ruling!(
        "Fire Nation Raider",
        "Raid abilities evaluate the entire turn to see if you attacked with a creature. That creature doesn't have to still be on the battlefield. Similarly, the player, planeswalker, or battle it attacked doesn't have to still be in the game or on the battlefield, respectively."
    );
    ruling!(
        "Fire Nation Engineer",
        "Raid abilities care only that you attacked with a creature. It doesn't matter how many creatures you attacked with, or which opponent, planeswalker, or battle those creatures attacked."
    );
    supported("Fire Nation Raider");
    supported("Fire Nation Engineer");
    let clues = |t: &TestGame| t.named_on_battlefield("Clue Token").len();
    // No attack: no Clue.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    cast_from_hand(&mut t, P0, "Fire Nation Raider", &[]);
    t.resolve_all();
    assert_eq!(clues(&t), 0);
    // Three players: one creature attacked P2's planeswalker (not P1); the planeswalker
    // is gone, P2 has left the game, and the attacker is destroyed.
    let mut t = TestGame::new(3);
    let giant = t.battlefield(P0, "Hill Giant");
    let jace = t.battlefield(P2, "Jace Beleren");
    t.attack(&[(giant, Entity::Object(jace))], &[]);
    assert!(t.in_graveyard(P2, "Jace Beleren"));
    destroy(&mut t, giant);
    t.g.perform_action(P2, mtg_engine::decision::Action::Concede)
        .expect("concede");
    t.advance_to(P0, Step::PostcombatMain);
    cast_from_hand(&mut t, P0, "Fire Nation Raider", &[]);
    t.resolve_all();
    assert_eq!(clues(&t), 1);
    // Fire Nation Engineer ("At the beginning of your end step, if you attacked this
    // turn, put a +1/+1 counter on another target creature or Vehicle you control."):
    // two creatures attacked two different opponents; one counter.
    let mut t = TestGame::new(3);
    let engineer = t.battlefield(P0, "Fire Nation Engineer");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P2))], &[]);
    t.answer_targets(P0, &[Entity::Object(a)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 1);
    assert_eq!(t.counters(b, counters::PLUS1), 0);
    assert_eq!(t.counters(engineer, counters::PLUS1), 0);
    // Without an attack, no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fire Nation Engineer");
    let a = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 0);
}

/// Picks the Forest among the cards offered by an entity choice of Strongbox Raider.
fn pick_forest(g: &mtg_engine::game::Game, d: &Decision) -> Option<mtg_engine::decision::Answer> {
    match d {
        Decision::ChooseEntities { candidates, .. } => candidates
            .iter()
            .find(|e| matches!(e, Entity::Object(o) if g.obj(*o).chars.name == "Forest"))
            .map(|e| mtg_engine::decision::Answer::Entities(vec![*e])),
        _ => None,
    }
}

#[test]
fn a_land_exiled_by_strongbox_raider_follows_the_normal_rules_for_playing_lands() {
    cr!("305.1", "305.2", "116.2a", "505.6b");
    ruling!(
        "Strongbox Raider",
        "You pay all costs and follow all normal timing rules for cards played with the permission granted by Strongbox Raider's ability. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
    );
    supported("Strongbox Raider");
    let mut t = TestGame::new(2);
    attack_then_main_2(&mut t);
    stack_library(&mut t, P0, &["Forest", "Lightning Bolt"]);
    respond(&mut t, P0, pick_forest);
    cast_from_hand(&mut t, P0, "Strongbox Raider", &[]);
    t.resolve_all();
    let forest = t.g.find_in_zone(Zone::Exile, "Forest");
    assert_eq!(forest.len(), 1);
    let forest = forest[0];
    // Main phase, empty stack, no land played: playable.
    assert!(can_play_land(&mut t, P0, forest));
    // Not with something on the stack.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    assert!(!can_play_land(&mut t, P0, forest));
    t.resolve_all();
    assert!(can_play_land(&mut t, P0, forest));
    // Not after a land was played this turn.
    let plains = t.hand(P0, "Plains");
    t.play_land(P0, plains).expect("land");
    assert!(!can_play_land(&mut t, P0, forest));
    // Not during the opponent's turn; yes in P0's next main phase.
    t.advance_to(P1, Step::PrecombatMain);
    assert!(!can_play_land(&mut t, P0, forest));
    t.advance_to(P0, Step::PrecombatMain);
    assert!(can_play_land(&mut t, P0, forest));
    t.play_land(P0, forest).expect("play the exiled Forest");
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
}
