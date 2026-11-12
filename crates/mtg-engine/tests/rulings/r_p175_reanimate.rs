//! Rulings batch P175 — temporary reanimation: creatures returned "tapped and attacking"
//! (Thunderkin Awakener), targets chosen before costs are paid (Apprentice Necromancer),
//! and delayed triggers that exile, return or sacrifice the creature at the next end step
//! only if it's still there (Corpse Dance, Shallow Grave, Cauldron Dance, Wake the Dead).

use crate::r_p175_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The game's battlefield object for a card that was put there (followed across zones).
fn now(t: &TestGame, id: ObjectId) -> ObjectId {
    t.g.current(id)
}

/// P0's Thunderkin Awakener, with +0/+`extra` until end of turn, attacks P1 (targeting
/// `target` if given); the game stops with the trigger on the stack.
fn thunderkin_attacks(t: &mut TestGame, extra: i32, target: Option<ObjectId>) -> ObjectId {
    let tk = t.battlefield(P0, "Thunderkin Awakener");
    if extra != 0 {
        pump(t, tk, 0, extra);
    }
    if let Some(c) = target {
        t.answer_targets(P0, &[obj(c)]);
    }
    attack_with(t, &[(tk, pl(P1))]);
    tk
}

/// The number of times anything "attacked" (AttackersDeclared entries) this turn.
fn declared_attackers(t: &TestGame) -> Vec<ObjectId> {
    t.g.turn_events
        .iter()
        .flat_map(|e| match e {
            Event::AttackersDeclared { attackers, .. } => {
                attackers.iter().map(|a| a.0).collect::<Vec<_>>()
            }
            _ => vec![],
        })
        .collect()
}

#[test]
fn thunderkin_awakener_returns_an_attacking_creature_that_never_attacked() {
    cr!("508.4", "508.3a");
    ruling!(
        "Thunderkin Awakener",
        "Although the returned creature is an attacking creature, it was never declared as an attacking creature. This means that abilities that trigger whenever a creature attacks (such as that of Thunderkin Awakener itself) won’t trigger when it enters the battlefield attacking."
    );
    supported("Thunderkin Awakener");
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Thunderkin Awakener");
    thunderkin_attacks(&mut t, 2, Some(card));
    t.resolve_all();
    let back = now(&t, card);
    assert!(t.on_battlefield(back));
    assert!(attacking(&t, back) && tapped(&t, back));
    // Its own "whenever this creature attacks" didn't trigger.
    assert_eq!(t.stack_len(), 0);
    assert!(!declared_attackers(&t).contains(&back));
}

#[test]
fn thunderkin_awakener_toughness_is_checked_on_attack_and_resolution_with_lki() {
    cr!("608.2b", "115.1", "603.3d", "608.2h");
    ruling!(
        "Thunderkin Awakener",
        "If Thunderkin Awakener leaves the battlefield after it has attacked but before its triggered ability resolves, use its toughness as it last existed on the battlefield to determine whether the card is still a legal target."
    );
    ruling!(
        "Thunderkin Awakener",
        "The target card must have lesser toughness both as Thunderkin Awakener attacks and as its triggered ability resolves. Other abilities that trigger whenever Thunderkin Awakener attacks can’t raise its toughness in time to allow the ability to target cards with greater toughness."
    );
    // Thunderkin (1/2, given +0/+1: toughness 3) dies with its trigger on the stack: its
    // last toughness (3) still makes Terrain Elemental (toughness 2) a legal target.
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Terrain Elemental");
    let tk = thunderkin_attacks(&mut t, 1, Some(card));
    assert_eq!(t.pt(tk).1, 3);
    kill(&mut t, tk);
    t.resolve_all();
    assert!(
        t.on_battlefield(now(&t, card)),
        "returned using last known toughness"
    );
    // Its toughness drops to 2 before resolution: the target is illegal.
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Terrain Elemental");
    let tk = thunderkin_attacks(&mut t, 1, Some(card));
    pump(&mut t, tk, 0, -1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Terrain Elemental"));
    // Toughness 2 as it attacks: Terrain Elemental can't be targeted, and raising the
    // toughness with the trigger on the stack is too late.
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Terrain Elemental");
    let from = t.asked().len();
    let tk = thunderkin_attacks(&mut t, 0, Some(card));
    assert!(target_choices(&t, P0, from, "")
        .iter()
        .all(|c| !c.contains(&obj(card))));
    pump(&mut t, tk, 0, 5);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Terrain Elemental"));
}

#[test]
fn thunderkin_awakener_creature_that_left_isnt_sacrificed() {
    cr!("603.7c", "400.7");
    ruling!(
        "Thunderkin Awakener",
        "If the new creature leaves the battlefield before the end step, that card remains in its current zone. It won’t be sacrificed."
    );
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Terrain Elemental");
    thunderkin_attacks(&mut t, 2, Some(card));
    t.resolve_all();
    let back = now(&t, card);
    assert!(t.on_battlefield(back));
    bounce(&mut t, back);
    end_step(&mut t);
    assert!(t.in_hand(P0, "Terrain Elemental"));
    // Control: left alone, it's sacrificed.
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Terrain Elemental");
    thunderkin_attacks(&mut t, 2, Some(card));
    t.resolve_all();
    end_step(&mut t);
    assert!(t.in_graveyard(P0, "Terrain Elemental"));
}

#[test]
fn thunderkin_awakener_returned_creature_can_attack_a_planeswalker() {
    cr!("508.4");
    ruling!(
        "Thunderkin Awakener",
        "You choose which player or planeswalker the returned creature is attacking. It doesn’t have to be attacking the same player or planeswalker that Thunderkin Awakener is attacking."
    );
    let mut t = TestGame::new(2);
    let pw = t.battlefield(P1, "Ajani Goldmane");
    let card = t.graveyard(P0, "Terrain Elemental");
    thunderkin_attacks(&mut t, 2, Some(card));
    t.answer_choose(P0, &[obj(pw)]);
    t.resolve_all();
    let back = now(&t, card);
    let target =
        t.g.combat
            .as_ref()
            .unwrap()
            .attackers
            .iter()
            .find(|a| a.id == back)
            .map(|a| a.target);
    assert_eq!(target, Some(Some(obj(pw))));
}

#[test]
fn apprentice_necromancer_cant_target_itself() {
    cr!("602.2b", "601.2c", "601.2h");
    ruling!(
        "Apprentice Necromancer",
        "Apprentice Necromancer can’t be the target of its own ability. This is because targets are chosen for activated abilities before costs (such as “Sacrifice Apprentice Necromancer”) are paid."
    );
    supported("Apprentice Necromancer");
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Apprentice Necromancer");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let from = t.asked().len();
    act(&mut t, P0, n, 0, &[obj(bears)]);
    let choices = target_choices(&t, P0, from, "");
    assert_eq!(
        choices,
        vec![vec![obj(bears)]],
        "only the Bears, not itself"
    );
    assert!(t.in_graveyard(P0, "Apprentice Necromancer"));
    assert!(t.on_battlefield(now(&t, bears)));
    // With no other creature card in the graveyard, it can't be activated.
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Apprentice Necromancer");
    lots(&mut t, P0);
    assert!(t.activate(P0, n, 0, &[]).is_err());
    assert!(t.on_battlefield(n));
}

#[test]
fn corpse_dance_and_shallow_grave_exile_only_a_creature_still_there() {
    cr!("603.7c", "400.7");
    ruling!(
        "Corpse Dance",
        "Corpse Dance only exiles the card if the card is still on the battlefield at the beginning of the end step."
    );
    ruling!(
        "Shallow Grave",
        "Only exiles the creature if the creature is still on the battlefield at the beginning of the end step."
    );
    supported("Corpse Dance");
    supported("Shallow Grave");
    for spell in ["Corpse Dance", "Shallow Grave"] {
        for leave in [false, true] {
            let mut t = TestGame::new(2);
            let bears = t.graveyard(P0, "Grizzly Bears");
            cast_at(&mut t, P0, spell, &[]);
            t.resolve_all();
            assert!(t.on_battlefield(now(&t, bears)), "{spell}");
            if leave {
                let b = now(&t, bears);
                bounce(&mut t, b);
            }
            end_step(&mut t);
            if leave {
                assert!(t.in_hand(P0, "Grizzly Bears"), "{spell}: stays in hand");
            } else {
                assert!(t.in_exile("Grizzly Bears"), "{spell}: exiled");
            }
        }
    }
}

/// P0 casts Cauldron Dance in P0's combat targeting `target`, putting `from_hand` (if any)
/// onto the battlefield; the spell is left on the stack.
fn cauldron_dance(t: &mut TestGame, target: ObjectId, from_hand: Option<ObjectId>) -> ObjectId {
    t.set_step(P0, Step::BeginningOfCombat);
    if let Some(h) = from_hand {
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(h)]);
    }
    cast_at(t, P0, "Cauldron Dance", &[obj(target)])
}

#[test]
fn cauldron_dance_rulings() {
    cr!("601.2c", "608.2b", "603.7c");
    ruling!(
        "Cauldron Dance",
        "You must have a creature in your graveyard to cast this spell since it requires a valid target."
    );
    ruling!(
        "Cauldron Dance",
        "If the targeted creature card in the graveyard is removed before resolution, this spell won’t resolve since its only target will be invalid."
    );
    ruling!(
        "Cauldron Dance",
        "You can cast this spell when you have no creature card in your hand. If this happens, simply ignore that part of the spell. You still put the graveyard creature onto the battlefield and still return it to your hand at the beginning of the end step."
    );
    supported("Cauldron Dance");
    // No creature card in the graveyard: it can't be cast.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    lots(&mut t, P0);
    let spell = t.hand(P0, "Cauldron Dance");
    assert!(t.cast(P0, spell).try_go().is_err());
    // Its target leaves the graveyard: it doesn't resolve, and the creature card in hand
    // isn't put onto the battlefield.
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let giant = t.hand(P0, "Hill Giant");
    cauldron_dance(&mut t, bears, Some(giant));
    exile(&mut t, bears);
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    // No creature card in hand: the graveyard creature still returns, then goes to hand
    // at the next end step.
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    cauldron_dance(&mut t, bears, None);
    t.resolve_all();
    assert!(t.on_battlefield(now(&t, bears)));
    end_step(&mut t);
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn wake_the_dead_sacrifice_trigger_fires_only_once() {
    cr!("603.7c", "701.21a", "603.7b");
    ruling!(
        "Wake the Dead",
        "The delayed triggered ability that makes you sacrifice the creatures will trigger only once. If you don’t sacrifice one or more of those creatures at that time (perhaps because another player gained control of them), the ability won’t try to make you sacrifice them on future turns."
    );
    supported("Wake the Dead");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    // P1's combat: P0 casts Wake the Dead with X = 1.
    t.set_step(P1, Step::BeginningOfCombat);
    lots(&mut t, P0);
    let spell = t.hand(P0, "Wake the Dead");
    t.cast(P0, spell).x(1).target(bears).go();
    t.resolve_all();
    let back = now(&t, bears);
    assert!(t.on_battlefield(back));
    // P1 gains control of it until end of turn: P0 can't sacrifice it at the end step.
    give_control_until_eot(&mut t, back, P1);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(back));
    // Back under P0's control, it isn't sacrificed at later end steps.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(ctl(&t, back), P0);
    assert!(t.on_battlefield(back));
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(back));
}
