//! Rulings batch P094 — effects that care about or modify life loss and damage to
//! opponents: Bloodletter of Aclazotz doubling, "whenever an opponent loses life" gains
//! (and losing the game first), Far Fortune's max-speed "+1 damage", and Vnwxt's
//! draw replacement.

use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s16_common::set_speed;
use crate::r_s20_common::tap_for_mana;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[test]
fn bloodletter_doubles_the_loss_not_the_damage() {
    cr!("120.3a", "702.15b", "614.1a");
    ruling!(
        "Bloodletter of Aclazotz",
        "Bloodletter of Aclazotz's last ability doesn't change the amount of damage dealt to opponents. For example, if a 1/1 creature with lifelink deals combat damage to an opponent on your turn, they would lose 2 life, but you'd still gain only 1 life."
    );
    supported("Bloodletter of Aclazotz");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bloodletter of Aclazotz");
    // Vampire Nighthawk: 2/3 flying, deathtouch, lifelink.
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    let from = t.g.turn_events.len();
    t.attack(&[(hawk, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P0), 22);
    let dealt: Vec<u32> = t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::Damage { amount, .. } => Some(*amount),
            _ => None,
        })
        .collect();
    assert_eq!(dealt, vec![2]);
}

#[test]
fn bloodthirsty_conqueror_damage_and_paying_life_are_life_loss() {
    cr!("120.3a", "119.4", "603.2");
    ruling!(
        "Bloodthirsty Conqueror",
        "Damage dealt to an opponent causes that opponent to lose life. An opponent paying life also causes loss of life."
    );
    supported("Bloodthirsty Conqueror");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bloodthirsty Conqueror");
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.deal_damage(giant, Entity::Player(P1), 3, false);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert!(t.g.pay_life(P1, 2));
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.life(P0), 25);
}

#[test]
fn losing_life_simultaneously_you_lose_before_the_gain_trigger_resolves() {
    cr!("704.5a", "704.3", "117.5");
    ruling!(
        "Bloodthirsty Conqueror",
        "If you and an opponent both lose life simultaneously, and this causes your life total to become 0 or less, you'll lose the game before Bloodthirsty Conqueror's triggered ability can resolve."
    );
    ruling!(
        "Exquisite Blood",
        "If you and an opponent both lose life simultaneously, and this causes your life total to become 0 or less, you'll lose the game before Exquisite Blood's triggered ability can resolve."
    );
    supported("Cryptolith Fragment // Aurora of Emrakul");
    for name in ["Bloodthirsty Conqueror", "Exquisite Blood"] {
        supported(name);
        // Three players so the game goes on: P0 at 1 life taps Cryptolith Fragment ("Each
        // player loses 1 life").
        let mut t = TestGame::new(3);
        t.battlefield(P0, name);
        let frag = t.battlefield(P0, "Cryptolith Fragment // Aurora of Emrakul");
        t.g.players[0].life = 1;
        assert!(tap_for_mana(&mut t, P0, frag, "Add"));
        assert_eq!(t.life(P0), 0);
        t.settle();
        assert!(t.has_lost(P0), "{name}");
        t.resolve_all();
        assert_eq!(t.life(P0), 0, "{name}");
        // Control: at 2 life the triggers do resolve.
        let mut t = TestGame::new(3);
        t.battlefield(P0, name);
        let frag = t.battlefield(P0, "Cryptolith Fragment // Aurora of Emrakul");
        t.g.players[0].life = 2;
        assert!(tap_for_mana(&mut t, P0, frag, "Add"));
        t.settle();
        assert_eq!(triggers_on_stack(&t, "gain that much life"), 2, "{name}");
        t.resolve_all();
        assert_eq!(t.life(P0), 3, "{name}");
    }
}

/// P0 controls Far Fortune at max speed.
fn far_fortune(t: &mut TestGame) -> ObjectId {
    supported("Far Fortune, End Boss");
    let ff = t.battlefield(P0, "Far Fortune, End Boss");
    set_speed(t, P0, 4);
    ff
}

#[test]
fn far_fortune_extra_damage_is_dealt_by_the_original_source() {
    cr!("614.1a", "702.179f", "702.15b");
    ruling!(
        "Far Fortune, End Boss",
        "The additional 1 damage from Far Fortune’s last ability is dealt by the same source as the original source of damage. The damage isn’t dealt by Far Fortune unless Far Fortune is the original source of damage."
    );
    let mut t = TestGame::new(2);
    let ff = far_fortune(&mut t);
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.g.turn_events.len();
    t.g.deal_damage(hawk, Entity::Player(P1), 2, false);
    t.g.flush_events();
    t.resolve_all();
    // 3 damage, all from Nighthawk: its lifelink gains 3.
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 23);
    let sources: Vec<(ObjectId, u32)> = t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::Damage { source, amount, .. } => Some((*source, *amount)),
            _ => None,
        })
        .collect();
    assert_eq!(sources, vec![(hawk, 3)]);
    assert!(!sources.iter().any(|(s, _)| *s == ff));
    // A permanent an opponent controls gets +1 too; P0's own creature doesn't.
    let ours = t.battlefield(P0, "Hill Giant");
    t.g.deal_damage(hawk, obj(bears), 1, false);
    t.g.deal_damage(hawk, obj(ours), 1, false);
    t.g.flush_events();
    assert_eq!(t.obj_now(ours).damage, 1);
    t.settle();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn far_fortune_divides_the_original_amount_before_adding_one() {
    cr!("510.1c", "702.19c", "614.1a");
    ruling!(
        "Far Fortune, End Boss",
        "If damage dealt by a source you control is being divided or assigned among multiple permanents an opponent controls or among an opponent and one or more permanents they control, divide the original amount before adding 1. For example, if you attack with a 5/5 creature with trample and your opponent blocks with a 2/2 creature, you can assign 2 damage to the blocker and 3 damage to the defending player. These amounts are then modified to 3 and 4, respectively."
    );
    let mut t = TestGame::new(2);
    far_fortune(&mut t);
    // Mounted Dreadknight: 5/4 trample. ("Whenever you attack" has Far Fortune deal
    // damage to P1 too; only the Dreadknight's damage is looked at.)
    let wurm = t.battlefield(P0, "Mounted Dreadknight");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![2, 3]));
    let from = t.g.turn_events.len();
    t.attack(&[(wurm, Entity::Player(P1))], &[(bear, wurm)]);
    let combat: Vec<(Entity, u32)> = t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::Damage {
                source,
                target,
                amount,
                ..
            } if *source == wurm => Some((*target, *amount)),
            _ => None,
        })
        .collect();
    assert!(combat.contains(&(obj(bear), 3)), "{combat:?}");
    assert!(combat.contains(&(Entity::Player(P1), 4)), "{combat:?}");
    assert_eq!(combat.len(), 2);
}

#[test]
fn far_fortune_and_prevention_the_damaged_player_chooses_the_order() {
    cr!("616.1", "120.8", "615.7");
    ruling!(
        "Far Fortune, End Boss",
        "If another effect modifies how much damage your sources would deal, including preventing some of it, the player being dealt damage or the controller of the permanent being dealt damage chooses an order in which to apply those effects. If all of the damage is prevented, Far Fortune’s last ability no longer applies."
    );
    supported("Healing Salve");
    let mut lives = vec![];
    for first in 0..2 {
        let mut t = TestGame::new(2);
        far_fortune(&mut t);
        let giant = t.battlefield(P0, "Hill Giant");
        // P1 casts Healing Salve: prevent the next 3 damage to P1.
        t.lands(P1, "Plains", 1);
        let salve = t.hand(P1, "Healing Salve");
        t.g.turn.priority = Some(P1);
        t.cast(P1, salve).modes(&[1]).target(Entity::Player(P1)).go();
        t.resolve_all();
        t.answer(P1, DecisionKind::Replacement, Answer::Index(first));
        let from = t.asked().len();
        t.g.deal_damage(giant, Entity::Player(P1), 3, false);
        t.g.flush_events();
        let chosen = t.asked()[from..]
            .iter()
            .find_map(|(p, d)| match d {
                Decision::ChooseReplacement { options } if *p == P1 => {
                    Some(options[first].clone())
                }
                _ => None,
            })
            .expect("P1 chooses the order");
        if !chosen.contains("Far Fortune") {
            // All 3 prevented: Far Fortune no longer applies.
            assert_eq!(t.life(P1), 20, "{chosen}");
        } else {
            // 4 damage, 3 prevented.
            assert_eq!(t.life(P1), 19, "{chosen}");
        }
        lives.push(t.life(P1));
    }
    lives.sort();
    assert_eq!(lives, vec![19, 20]);
}

#[test]
fn vnwxt_the_drawing_player_orders_draw_replacements() {
    cr!("616.1", "616.1e", "614.5", "121.6");
    ruling!(
        "Vnwxt, Verbose Host",
        "If two or more replacement effects would apply to a card-drawing event, the player who’s drawing the card chooses what order to apply them."
    );
    supported("Vnwxt, Verbose Host");
    supported("Underrealm Lich");
    let mut seen = vec![];
    for first in 0..2 {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Vnwxt, Verbose Host");
        t.battlefield(P0, "Underrealm Lich");
        set_speed(&mut t, P0, 4);
        t.answer(P0, DecisionKind::Replacement, Answer::Index(first));
        let from = t.asked().len();
        let (hand, gy) = (t.hand_size(P0), t.graveyard_size(P0));
        t.g.draw_cards(P0, 1);
        t.g.flush_events();
        let chosen = t.asked()[from..]
            .iter()
            .find_map(|(p, d)| match d {
                Decision::ChooseReplacement { options } if *p == P0 => {
                    Some(options[first].clone())
                }
                _ => None,
            })
            .expect("P0 chooses the order");
        let got = (t.hand_size(P0) - hand, t.graveyard_size(P0) - gy);
        if chosen.contains("look at the top three") {
            // Lich first: the draw is gone; Vnwxt has nothing to replace.
            assert_eq!(got, (1, 2), "{chosen}");
        } else {
            // Vnwxt first: two draws, each replaced by Lich.
            assert_eq!(got, (2, 4), "{chosen}");
        }
        seen.push(got);
    }
    assert_ne!(seen[0], seen[1]);
}
