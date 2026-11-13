//! Rulings batch P126 — returning cards from graveyards to the battlefield: targets and
//! costs (CR 601.2c, 601.2h, 608.2b), "sacrifice a creature. If you do" (CR 608.2c),
//! creatures put onto the battlefield attacking (CR 506.3, 508.4), Auras put onto the
//! battlefield without being cast (CR 303.4f), bestow cards that aren't cast
//! (CR 702.103), and even mana values (CR 202.3).

use crate::r_p076_common::mana;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::{can_activate, target_candidates};
use crate::r_s05_common::{enter, move_to};
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn victimize_mana(t: &mut TestGame) {
    mana(t, P0, ManaType::B, 1);
    mana(t, P0, ManaType::C, 2);
}

#[test]
fn victimize_needs_two_targets() {
    cr!("601.2c", "115.3");
    ruling!(
        "Victimize",
        "You must choose two targets. You can't cast Victimize targeting only one creature card."
    );
    supported("Victimize");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let a = t.graveyard(P0, "Hill Giant");
    let v = t.hand(P0, "Victimize");
    victimize_mana(&mut t);
    assert!(t.cast(P0, v).targets(&[obj(a)]).try_go().is_err());
    assert_eq!(t.zone(v), Zone::Hand(P0));
}

#[test]
fn victimize_sacrifices_on_resolution() {
    cr!("608.2c", "608.2b", "701.21a");
    ruling!(
        "Victimize",
        "As Victimize resolves, you must sacrifice a creature if able. You can't change your mind and choose not to sacrifice anything."
    );
    ruling!(
        "Victimize",
        "The creature you sacrifice isn't chosen until Victimize resolves. You can't return the creature you sacrifice because it will still be on the battlefield at the time targets are chosen."
    );
    ruling!(
        "Victimize",
        "If one of the targeted creature cards is an illegal target (for instance, because it has left your graveyard before Victimize resolves), you'll still sacrifice a creature and put the other card onto the battlefield. If both are illegal targets, Victimize won't resolve. You won't sacrifice a creature."
    );
    // (cards that leave the graveyard before it resolves)
    for leave in 0..3 {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let a = t.graveyard(P0, "Hill Giant");
        let b = t.graveyard(P0, "Serra Angel");
        let v = t.hand(P0, "Victimize");
        victimize_mana(&mut t);
        let from = t.asked().len();
        t.cast(P0, v).targets(&[obj(a), obj(b)]).go();
        // The creature on the battlefield wasn't a possible target.
        assert!(target_candidates(&t, P0, from)
            .iter()
            .all(|c| !c.contains(&obj(bears))));
        if leave >= 1 {
            move_to(&mut t, a, Zone::Exile);
        }
        if leave >= 2 {
            move_to(&mut t, b, Zone::Exile);
        }
        // Declining doesn't stop the sacrifice.
        t.answer_yes(P0, false);
        t.answer_choose(P0, &[obj(bears)]);
        t.resolve_all();
        assert_eq!(t.on_battlefield(bears), leave == 2, "leave {leave}");
        assert_eq!(t.on_battlefield(a), leave == 0, "leave {leave}");
        assert_eq!(t.on_battlefield(b), leave < 2, "leave {leave}");
        if leave < 2 {
            assert!(t.obj_now(b).tapped);
        }
    }
}

#[test]
fn gravewaker_only_from_the_battlefield() {
    cr!("602.1", "113.6");
    ruling!(
        "Gravewaker",
        "Gravewaker’s ability can be activated only while it’s on the battlefield. It can’t return itself from your graveyard."
    );
    supported("Gravewaker");
    let mut t = TestGame::new(2);
    let gw = t.graveyard(P0, "Gravewaker");
    mana(&mut t, P0, ManaType::B, 2);
    mana(&mut t, P0, ManaType::C, 5);
    assert!(!can_activate(&mut t, P0, gw));
}

#[test]
fn whisper_sacrifices_itself_but_cant_target_the_sacrificed() {
    cr!("601.2c", "601.2h", "602.2");
    ruling!(
        "Whisper, Blood Liturgist",
        "Neither sacrificed creature can be the target of Whisper’s ability."
    );
    ruling!(
        "Whisper, Blood Liturgist",
        "Whisper can be one of the creatures sacrificed to activate its ability."
    );
    supported("Whisper, Blood Liturgist");
    let mut t = TestGame::new(2);
    let whisper = t.battlefield(P0, "Whisper, Blood Liturgist");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(whisper), obj(bears)]);
    t.activate(P0, whisper, 0, &[obj(giant)]).unwrap();
    let cands = target_candidates(&t, P0, from);
    assert!(cands
        .iter()
        .all(|c| !c.contains(&obj(whisper)) && !c.contains(&obj(bears))));
    assert!(!t.on_battlefield(whisper));
    assert!(!t.on_battlefield(bears));
    t.resolve_all();
    assert!(t.on_battlefield(giant));
}

#[test]
fn miraculous_recovery_counter_before_enters_triggers_resolve() {
    cr!("608.2c", "603.3", "117.3b");
    ruling!(
        "Miraculous Recovery",
        "No player may take actions between the time the creature returns to the battlefield and the time you put a +1/+1 counter on it."
    );
    ruling!(
        "Miraculous Recovery",
        "The creature returns to the battlefield without a +1/+1 counter on it. Any abilities that trigger when a creature enters the battlefield will trigger or not as appropriate before it receives the +1/+1 counter and resolve after it has received that counter."
    );
    supported("Miraculous Recovery");
    let mut t = TestGame::new(2);
    let warden = t.battlefield(P0, "Soul Warden");
    let _ = warden;
    let sentinel = t.graveyard(P0, "Wall of Omens");
    t.library_top(P0, "Island");
    let mr = t.hand(P0, "Miraculous Recovery");
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::C, 4);
    t.cast(P0, mr).target(sentinel).go();
    t.resolve();
    // Recovery is done: the counter is on it, and the enters triggers are waiting.
    assert!(t.on_battlefield(sentinel));
    assert_eq!(t.counters(sentinel, "+1/+1"), 1);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn yore_tiller_nephilim_returns_a_creature_attacking() {
    cr!("508.4", "506.3");
    ruling!(
        "Yore-Tiller Nephilim",
        "The creature you put onto the battlefield from your graveyard is attacking, even if the attack couldn't legally be declared (for example, if that creature has defender"
    );
    supported("Yore-Tiller Nephilim");
    let mut t = TestGame::new(2);
    let yt = t.battlefield(P0, "Yore-Tiller Nephilim");
    let wall = t.graveyard(P0, "Wall of Omens");
    t.answer_targets(P0, &[obj(wall)]);
    attack_with(&mut t, &[(yt, Entity::Player(P1))]);
    t.resolve_all();
    let combat = t.g.combat.as_ref().unwrap();
    let w = t.g.current(wall);
    assert!(combat.attackers.iter().any(|a| a.id == w));
    assert!(t.obj_now(wall).tapped);
}

#[test]
fn yore_tiller_nephilim_in_multiplayer() {
    cr!("508.4", "506.2");
    ruling!(
        "Yore-Tiller Nephilim",
        "In a multiplayer game in which you're allowed to attack only one player each combat, the new creature will be attacking the same player as Yore-Tiller Nephilim. In a multiplayer game in which you may attack multiple players, you choose the player the new creature is attacking."
    );
    let mut t = TestGame::new(3);
    let yt = t.battlefield(P0, "Yore-Tiller Nephilim");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    t.answer(P0, DecisionKind::Any, Answer::Default);
    attack_with(&mut t, &[(yt, Entity::Player(P2)), (bears, Entity::Player(P1))]);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    let combat = t.g.combat.as_ref().unwrap();
    let g = t.g.current(giant);
    let att = combat.attackers.iter().find(|a| a.id == g).expect("attacking");
    // P0 may attack several players: it chose P1, not the player Yore-Tiller attacks.
    assert_eq!(att.target, Some(Entity::Player(P1)));
}

#[test]
fn silent_sentinel_returns_auras_without_targeting() {
    cr!("303.4f", "303.4g", "115.1");
    ruling!(
        "Silent Sentinel",
        "If the enchantment card is an Aura, you choose a legal player or object for that Aura to enchant as it enters the battlefield. This doesn’t target the player or object, so it’s possible to enchant an opposing creature with hexproof this way, for example. If there’s no player or object the Aura can legally enchant, it stays in the graveyard."
    );
    supported("Silent Sentinel");
    let mut t = TestGame::new(2);
    let ss = t.battlefield(P0, "Silent Sentinel");
    let scout = t.battlefield(P1, "Gladecover Scout");
    let pac = t.graveyard(P0, "Pacifism");
    t.answer_targets(P0, &[obj(pac)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(scout)]);
    attack_with(&mut t, &[(ss, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.on_battlefield(pac));
    assert_eq!(t.obj_now(pac).attached_to, Some(obj(scout)));
    // Silent Sentinel is the only creature, and it leaves before the trigger resolves:
    // there's nothing to enchant, so Pacifism stays in the graveyard.
    let mut t = TestGame::new(2);
    let ss = t.battlefield(P0, "Silent Sentinel");
    let pac = t.graveyard(P0, "Pacifism");
    t.answer_targets(P0, &[obj(pac)]);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(ss, Entity::Player(P1))]);
    move_to(&mut t, ss, Zone::Exile);
    t.resolve_all();
    assert_eq!(t.zone(pac), Zone::Graveyard(P0));
}

#[test]
fn silent_sentinel_bestow_card_returns_as_a_creature() {
    cr!("702.103a", "303.4f");
    ruling!(
        "Silent Sentinel",
        "If you return an enchantment creature card with bestow to the battlefield, you can’t pay its bestow cost. It won’t be an Aura and can’t be attached to a creature."
    );
    let mut t = TestGame::new(2);
    let ss = t.battlefield(P0, "Silent Sentinel");
    let satyr = t.graveyard(P0, "Boon Satyr");
    t.answer_targets(P0, &[obj(satyr)]);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(ss, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.on_battlefield(satyr));
    let o = t.obj_now(satyr);
    assert!(o.is(CardType::Creature));
    assert!(!o.chars.has_subtype("Aura"));
    assert_eq!(o.attached_to, None);
}

#[test]
fn gyruda_mana_value_zero_is_even() {
    cr!("202.3");
    ruling!("Gyruda, Doom of Depths", "A card with mana value 0 has an even mana value.");
    supported("Gyruda, Doom of Depths");
    let mut t = TestGame::new(2);
    // P1's top four cards: three Islands above an Ornithopter (mana value 0).
    let orn = t.library_top(P1, "Ornithopter");
    for _ in 0..3 {
        t.library_top(P1, "Island");
    }
    t.answer_choose(P0, &[obj(orn)]);
    enter(&mut t, P0, "Gyruda, Doom of Depths");
    t.resolve_all();
    assert!(t.on_battlefield(orn));
    assert_eq!(t.obj_now(orn).controller, P0);
}
