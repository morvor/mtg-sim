//! Rulings batch P096 — Lich: "As this enchantment enters, you lose life equal to your life
//! total. You don't lose the game for having 0 or less life. If you would gain life, draw
//! that many cards instead. Whenever you're dealt damage, sacrifice that many nontoken
//! permanents. If you can't, you lose the game. When this enchantment is put into a
//! graveyard from the battlefield, you lose the game."

use crate::r_p096_common::*;
use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s25_common::cast_new;
use crate::r_s29_common::cast_and_resolve;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0's Lich entering the battlefield for real: P0 goes to 0 life.
fn lich(t: &mut TestGame) -> ObjectId {
    supported("Lich");
    let lich = t.enter(P0, "Lich");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.life(P0), 0);
    assert!(!t.has_lost(P0));
    lich
}

#[test]
fn an_opponent_stealing_lich_makes_you_lose_but_doesnt_change_their_life() {
    cr!("704.5a", "613.1b", "614.12");
    ruling!(
        "Lich",
        "If an opponent steals control of Lich and no other effect prevents you from losing with a life total of zero, you will lose the game due to a zero life total as a State-Based Action before you can take any actions. The last sentence doesn't apply in this case since the Lich didn't leave the battlefield."
    );
    ruling!(
        "Lich",
        "If an opponent steals control of Lich, their life total does not change. The life total changes for a player only when it enters under that player's control."
    );
    // Three players, so the game goes on after P0 loses. P1 casts Confiscate ("Enchant
    // permanent. You control enchanted permanent.") on P0's Lich.
    let mut t = TestGame::new(3);
    let l = lich(&mut t);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let from = t.g.turn_events.len();
    cast_new(&mut t, P1, "Confiscate", &[Entity::Object(l)]);
    t.resolve();
    assert!(t.has_lost(P0));
    // The Lich didn't leave the battlefield (until P0 left the game), and its last
    // ability never triggered.
    assert!(t.g.turn_events[from..].iter().all(|e| !matches!(
        e,
        Event::ZoneChange { old, to: mtg_engine::object::Zone::Graveyard(_), .. } if *old == l
    )));
    assert_eq!(triggers_on_stack(&t, "you lose the game"), 0);
    assert_eq!((t.life(P1), t.life(P2)), (20, 20));
    assert!(!t.has_lost(P1));

    // P0 controlling Platinum Angel ("You can't lose the game and your opponents can't win
    // the game.") is another effect preventing the loss: P0 stays in the game at 0 life.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Platinum Angel");
    let l = lich(&mut t);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    cast_new(&mut t, P1, "Confiscate", &[Entity::Object(l)]);
    t.resolve_all();
    assert_eq!(t.obj_now(l).controller, P1);
    assert!(!t.has_lost(P0));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn lich_damage_sacrifices_the_permanents_simultaneously() {
    cr!("701.21a", "603.2", "704.5m");
    ruling!(
        "Lich",
        "If you take more than one damage at a time, sacrifice the permanents for that damage simultaneously. This allows you to sacrifice both a creature and any Aura that is on it all at once."
    );
    // P0 controls Grizzly Bears enchanted with their own Holy Strength, and lands. Shock
    // deals 2 damage to P0: P0 chooses the Bears and the Aura in one choice, and both are
    // sacrificed at once (when the first leaves, the other is already gone).
    let mut t = TestGame::new(2);
    let l = lich(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Holy Strength");
    t.g.attach(aura, Entity::Object(bears));
    t.lands(P0, "Swamp", 3);
    t.g.recompute();
    let seen: crate::r_s01_common::Seen<(bool, bool)> = Default::default();
    let s = seen.clone();
    t.g.observer = Some(mtg_engine::game::EventObserver::new(move |g, ev| {
        if matches!(ev, Event::Sacrificed { .. }) {
            s.lock().unwrap().push((g.is_live(bears), g.is_live(aura)));
        }
    }));
    t.answer_choose(P0, &[Entity::Object(bears), Entity::Object(aura)]);
    let from = t.asked().len();
    let events = t.g.turn_events.len();
    cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    t.resolve_all();
    let sacrificed: Vec<ObjectId> = t.g.turn_events[events..]
        .iter()
        .filter_map(|e| match e {
            Event::Sacrificed { obj, .. } => Some(*obj),
            _ => None,
        })
        .collect();
    assert_eq!(sacrificed, vec![bears, aura]);
    assert_eq!(*seen.lock().unwrap(), vec![(false, false); 2]);
    let choices: Vec<_> = t.asked()[from..]
        .iter()
        .filter(|(p, d)| {
            *p == P0 && matches!(d, mtg_engine::decision::Decision::ChooseEntities { .. })
        })
        .cloned()
        .collect();
    assert_eq!(choices.len(), 1);
    assert!(t.on_battlefield(l));
    assert!(!t.has_lost(P0));
    assert_eq!(t.life(P0), -2);
}

#[test]
fn lich_at_0_life_leaving_loses_by_state_based_action_first() {
    cr!("704.3", "704.5a", "603.3");
    ruling!(
        "Lich",
        "Note that usually you will have 0 or less life at the time Lich leaves the battlefield causing you to lose as a State-Based Action before the last ability can even go on the stack."
    );
    // Three players: P1 destroys the Lich with Disenchant. P0 is at 0 life with no Lich:
    // state-based actions make P0 lose before the Lich's trigger is put on the stack.
    let mut t = TestGame::new(3);
    let l = lich(&mut t);
    let from = t.g.turn_events.len();
    cast_new(&mut t, P1, "Disenchant", &[Entity::Object(l)]);
    t.resolve();
    assert!(t.has_lost(P0));
    assert!(t.g.turn_events[from..]
        .iter()
        .all(|e| !matches!(e, Event::AbilityTriggeredOnStack { .. })));
}

#[test]
fn lich_last_ability_loses_whatever_the_life_total_unless_you_cant_lose() {
    cr!("104.3b", "603.6c", "104.3e", "101.2");
    ruling!(
        "Lich",
        "The last ability will cause you to lose the game even if you somehow manage to have a life total greater than 0 at the time the Lich is put into the graveyard or if some other effect would prevent you from losing for having 0 life. If, on the other hand, some effect such as that from Platinum Angel says that you can't lose the game then even the last ability of the Lich cannot cause you to do so. The ability will just resolve and the game will continue as normal."
    );
    ruling!(
        "Lich",
        "or if some other effect would prevent you from losing for having 0 life. If, on the other hand, some effect such as that from Platinum Angel says that you can't lose the game then even the last ability of the Lich cannot cause you to do so. The ability will just resolve and the game will continue as normal."
    );
    // (Also in the 2010-08-15 version of the ruling above, which says "at the time the Lich
    // leaves the battlefield": the Oracle wording before the September 2010 Update Bulletin
    // (2010-09-22); Wizards reworded it on 2011-01-01. See
    // docs/rulings-exemptions/rulings-P096.tsv.)
    // A Lich put onto the battlefield without its enters replacement: P0 is at 20 life when
    // it's destroyed, and still loses to its trigger.
    let mut t = TestGame::new(3);
    let l = t.battlefield(P0, "Lich");
    cast_new(&mut t, P1, "Disenchant", &[Entity::Object(l)]);
    t.resolve();
    assert!(!t.has_lost(P0));
    assert_eq!(triggers_on_stack(&t, "you lose the game"), 1);
    t.resolve_all();
    assert!(t.has_lost(P0));
    assert_eq!(t.life(P0), 20);
    // Two Liches at 0 life: with one destroyed, the other still keeps P0 from losing for
    // having 0 life, and the destroyed one's trigger makes P0 lose.
    let mut t = TestGame::new(3);
    let l = lich(&mut t);
    lich(&mut t);
    cast_new(&mut t, P1, "Disenchant", &[Entity::Object(l)]);
    t.resolve();
    assert!(!t.has_lost(P0));
    assert_eq!(triggers_on_stack(&t, "you lose the game"), 1);
    t.resolve_all();
    assert!(t.has_lost(P0));
    // Platinum Angel: the trigger resolves and the game goes on.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Platinum Angel");
    let l = lich(&mut t);
    cast_new(&mut t, P1, "Disenchant", &[Entity::Object(l)]);
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "you lose the game"), 1);
    t.resolve_all();
    assert_eq!(triggers_on_stack(&t, "you lose the game"), 0);
    assert!(!t.has_lost(P0));
}

#[test]
fn lich_leaving_the_battlefield_except_to_a_graveyard_doesnt_trigger_its_last_ability() {
    cr!("603.2", "603.2g", "614.6", "704.5a");
    ruling!(
        "Lich",
        "Note that usually you will have 0 or less life at the time Lich leaves the battlefield causing you to lose as a State-Based Action before the last ability can even go on the stack."
    );
    // The last ability has said "When Lich is put into a graveyard from the battlefield"
    // since the September 2010 Update Bulletin (2010-09-22; Oracle update announced for
    // 2010-09-24); before it, "When Lich leaves the battlefield"
    // (docs/rulings-exemptions/rulings-P096.tsv). Its owner leaving the game is covered by
    // an_opponent_stealing_lich_makes_you_lose_but_doesnt_change_their_life.
    // A Lich at 20 life (put onto the battlefield without its enters replacement) exiled
    // by Erase: no trigger, P0 stays in the game.
    let mut t = TestGame::new(3);
    let l = t.battlefield(P0, "Lich");
    cast_new(&mut t, P1, "Erase", &[Entity::Object(l)]);
    t.resolve();
    assert!(t.in_exile("Lich"));
    assert_eq!(triggers_on_stack(&t, "you lose the game"), 0);
    t.resolve_all();
    assert!(!t.has_lost(P0));
    assert_eq!(t.life(P0), 20);
    // Destroyed while P1 controls Rest in Peace ("If a card or token would be put into a
    // graveyard from anywhere, exile it instead."): the replaced event triggers nothing.
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Rest in Peace");
    let l = t.battlefield(P0, "Lich");
    cast_new(&mut t, P1, "Disenchant", &[Entity::Object(l)]);
    t.resolve();
    assert!(t.in_exile("Lich"));
    assert_eq!(triggers_on_stack(&t, "you lose the game"), 0);
    t.resolve_all();
    assert!(!t.has_lost(P0));
    // Two Liches at 0 life: one returned to its owner's hand doesn't trigger, and the
    // other still keeps P0 from losing for having 0 life.
    let mut t = TestGame::new(3);
    let l = lich(&mut t);
    let l2 = lich(&mut t);
    cast_new(&mut t, P1, "Boomerang", &[Entity::Object(l)]);
    t.resolve();
    assert!(t.in_hand(P0, "Lich"));
    assert_eq!(triggers_on_stack(&t, "you lose the game"), 0);
    assert!(!t.has_lost(P0));
    // The last Lich exiled at 0 life: P0 loses as a state-based action.
    cast_new(&mut t, P1, "Erase", &[Entity::Object(l2)]);
    t.resolve();
    assert!(t.has_lost(P0));
}

#[test]
fn multiple_liches_sacrifice_for_each_but_draw_only_once() {
    cr!("603.2", "614.1a", "616.1");
    ruling!(
        "Lich",
        "If you have multiple Lich cards on the battlefield, you must sacrifice a permanent for each damage done to you for each Lich. This is because the sacrifice is a triggered ability. But you only draw one card for each life gained regardless of how many Liches you have. This is because the draw is a replacement effect and not a triggered one."
    );
    // (The ruling's last sentence, "You lose if any one of the Liches leaves the
    // battlefield.", was written for the Oracle text "When Lich leaves play, you lose the
    // game" ("leaves the battlefield" from Magic 2010), which the September 2010 Update
    // Bulletin (2010-09-22) changed to "When Lich is put into a graveyard from the
    // battlefield, you lose the game": see
    // lich_leaving_the_battlefield_except_to_a_graveyard_doesnt_trigger_its_last_ability and
    // docs/rulings-exemptions/rulings-P096.tsv. "Any one of the Liches" still holds for a
    // Lich put into a graveyard: lich_last_ability_loses_whatever_the_life_total_unless_you_cant_lose.)
    let mut t = TestGame::new(2);
    lich(&mut t);
    lich(&mut t);
    let swamps = t.lands(P0, "Swamp", 6);
    // Shock: 2 damage, two triggers: 4 permanents (Swamps) sacrificed.
    let e = |i: usize| Entity::Object(swamps[i]);
    t.answer_choose(P0, &[e(0), e(1)]);
    t.answer_choose(P0, &[e(2), e(3)]);
    cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "sacrifice that many"), 2);
    t.resolve_all();
    assert_eq!(
        t.g.permanents().filter(|o| o.controller == P0).count(),
        2 + 6 - 4
    );
    assert!(swamps[..4].iter().all(|s| !t.on_battlefield(*s)));
    // Gaining 4 life: one replacement applies, 4 cards drawn (not 8), no life gained.
    let hand = t.hand_size(P0);
    cast_and_resolve(&mut t, P0, "Sacred Nectar", &[]);
    assert_eq!(t.hand_size(P0), hand + 4);
    assert_eq!(t.life(P0), -2);
}

#[test]
fn lich_at_0_life_cant_pay_life_but_can_pay_0() {
    cr!("119.4", "601.2h");
    ruling!(
        "Lich",
        "You can't pay life, just like any player at less than one life can't pay life. You can pay zero life if you want."
    );
    // Greed: "{B}, Pay 2 life: Draw a card."
    let mut t = TestGame::new(2);
    lich(&mut t);
    let greed = t.battlefield(P0, "Greed");
    t.lands(P0, "Swamp", 1);
    assert!(t.activate(P0, greed, 0, &[]).is_err());
    // Toxic Deluge ("As an additional cost to cast this spell, pay X life.") with X = 0.
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(0));
    let deluge = cast_new(&mut t, P0, "Toxic Deluge", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Toxic Deluge"));
    let _ = deluge;
    assert_eq!(t.life(P0), 0);
    assert!(!t.has_lost(P0));
}
