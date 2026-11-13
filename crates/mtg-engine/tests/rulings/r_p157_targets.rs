//! Rulings batch P157 — targets of abilities that give an opponent something: illegal
//! targets on resolution (CR 608.2b), divided damage (CR 601.2d), triggered abilities
//! with no legal targets (CR 603.3d), activation restrictions from targets (CR 602.2b),
//! choices made on resolution, and the player choosing for their own graveyard.

use crate::r_p003_common::divide;
use crate::r_p157_common::*;
use crate::r_p180_common::plus;
use crate::r_s01_common::{attack_with, watch};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p` gets hexproof now (Leyline of Sanctity enters under their control).
fn hexproof(t: &mut TestGame, p: PlayerId) {
    t.battlefield(p, "Leyline of Sanctity");
    t.g.recompute();
}

/// Moves a card from its zone to exile (as an effect would).
fn exile_card(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.move_object(id, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    t.g.recompute();
    t.g.flush_events();
}

/// P0's next upkeep (P1's turn is passed), triggers on the stack.
fn p0_upkeep(t: &mut TestGame) {
    to_upkeep(t, P0);
}

#[test]
fn evolutionary_escalation_targets_on_the_stack_and_one_illegal() {
    cr!("603.3d", "608.2b", "117.5", "503.1a");
    ruling!(
        "Evolutionary Escalation",
        "You choose two targets for Evolutionary Escalation’s ability as it’s put onto the stack. No player may cast spells or activate abilities during your turn before this happens."
    );
    ruling!(
        "Evolutionary Escalation",
        "If one target becomes illegal before Evolutionary Escalation’s ability resolves, you’ll put three +1/+1 counters on the target that’s still legal."
    );
    supported("Evolutionary Escalation");
    // The game runs through P0's next upkeep: the first time P0 gets priority in it, the
    // ability is already on the stack, its targets chosen (CR 117.5).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Evolutionary Escalation");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Savannah Lions");
    t.answer_targets(P0, &[obj(mine)]);
    t.answer_targets(P0, &[obj(theirs)]);
    pass_turn(&mut t);
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::Priority { .. }),
        |g| {
            let targets = g
                .stack
                .last()
                .and_then(|s| g.obj(*s).stack.as_deref())
                .map_or(0, |si| {
                    si.chosen
                        .iter()
                        .flat_map(|c| c.targets.iter())
                        .map(|slot| slot.len())
                        .sum::<usize>()
                });
            (g.turn.active, g.turn.step, g.stack.len(), targets)
        },
    );
    t.advance_to(P0, Step::Draw);
    let first = seen
        .lock()
        .unwrap()
        .iter()
        .find(|(p, step, ..)| *p == P0 && *step == Step::Upkeep)
        .copied();
    assert_eq!(first, Some((P0, Step::Upkeep, 1, 2)), "the trigger first, with both targets");
    assert_eq!((plus(&t, mine), plus(&t, theirs)), (3, 3));
    // The opponent's creature leaves: P0's still gets three counters.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Evolutionary Escalation");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Savannah Lions");
    t.answer_targets(P0, &[obj(mine)]);
    t.answer_targets(P0, &[obj(theirs)]);
    p0_upkeep(&mut t);
    let trig = *t.g.stack.last().expect("on the stack");
    assert_eq!(stack_targets(&t, trig), vec![obj(mine), obj(theirs)]);
    kill(&mut t, theirs);
    t.resolve_all();
    assert_eq!(plus(&t, mine), 3);
}

#[test]
fn evolutionary_escalation_without_two_legal_targets_is_removed() {
    cr!("603.3d");
    ruling!(
        "Evolutionary Escalation",
        "If there aren’t two legal targets for Evolutionary Escalation’s ability as it’s put onto the stack, it’s taken off the stack and no creature gets +1/+1 counters."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Evolutionary Escalation");
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(mine)]);
    p0_upkeep(&mut t);
    assert_eq!(triggers_on_stack(&t, "+1/+1 counters"), 0);
    t.resolve_all();
    assert_eq!(plus(&t, mine), 0);
}

#[test]
fn fiery_justice_division_chosen_as_cast_at_least_one_each() {
    cr!("601.2c", "601.2d");
    ruling!(
        "Fiery Justice",
        "You choose how many targets Fiery Justice has and how the damage is divided as you cast it. Each target for its damage-dealing effect must receive at least 1 damage."
    );
    supported("Fiery Justice");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.answer_targets(P0, &[pl(P1)]);
    divide(&mut t, P0, &[2, 3]);
    let m = mark(&t);
    cast_at(&mut t, P0, "Fiery Justice", &[]);
    let div: Vec<_> = t.asked()[m..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Divide {
                total,
                recipients,
                min_each,
                ..
            } => Some((*total, recipients.len(), *min_each)),
            _ => None,
        })
        .collect();
    assert_eq!(div, vec![(5, 2, 1)], "divided as it's cast, at least 1 each");
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert_eq!(t.obj_now(b).damage, 3);
    assert_eq!(t.life(P1), 25);
}

#[test]
fn fiery_justice_illegal_targets() {
    cr!("608.2b", "601.2d");
    ruling!(
        "Fiery Justice",
        "If some (but not all) of the targets become illegal, the original division of damage still applies, but no damage is dealt to illegal targets. If all targets become illegal, including the target opponent for the life-gaining effect, Fiery Justice doesn't resolve. If each target for the damage-dealing effect becomes illegal while the target for the life-gaining effect remains legal, that player gains 5 life even though no damage is dealt."
    );
    let setup = |t: &mut TestGame| {
        let a = t.battlefield(P1, "Hill Giant");
        let b = t.battlefield(P1, "Hill Giant");
        t.answer_targets(P0, &[obj(a), obj(b)]);
        t.answer_targets(P0, &[pl(P1)]);
        divide(t, P0, &[1, 4]);
        cast_at(t, P0, "Fiery Justice", &[]);
        (a, b)
    };
    // Some targets illegal: the remaining one still gets its 1.
    let mut t = TestGame::new(2);
    let (a, b) = setup(&mut t);
    kill(&mut t, b);
    t.resolve_all();
    assert_eq!(t.obj_now(a).damage, 1);
    assert_eq!(t.life(P1), 25);
    // Every target illegal: it doesn't resolve.
    let mut t = TestGame::new(2);
    let (a, b) = setup(&mut t);
    kill(&mut t, a);
    kill(&mut t, b);
    hexproof(&mut t, P1);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Fiery Justice"));
    // Only the damage targets illegal: P1 still gains 5.
    let mut t = TestGame::new(2);
    let (a, b) = setup(&mut t);
    kill(&mut t, a);
    kill(&mut t, b);
    t.resolve_all();
    assert_eq!(t.life(P1), 25);
}

#[test]
fn fiery_justice_the_life_gaining_opponent_can_be_dealt_damage_too() {
    cr!("608.2c", "704.3", "704.5a");
    ruling!(
        "Fiery Justice",
        "The target opponent for the life-gaining effect may also be a target for the damage-dealing effect. If this happens and the damage brings that player's life total to 0 or less, the life-gaining effect will raise their life total above 0 again before the player would lose the game."
    );
    let mut t = TestGame::new(2);
    set_life(&mut t, P1, 3);
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[pl(P1)]);
    divide(&mut t, P0, &[5]);
    cast_at(&mut t, P0, "Fiery Justice", &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 3);
    assert!(!t.has_lost(P1));
}

#[test]
fn avacyns_judgment_keeps_the_original_division() {
    cr!("608.2b", "601.2d");
    ruling!(
        "Avacyn's Judgment",
        "If Avacyn's Judgment has multiple targets, and some but not all of them are illegal targets when Avacyn's Judgment resolves, Avacyn's Judgment will still deal damage to the remaining legal targets according to the original damage division."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    divide(&mut t, P0, &[1, 1]);
    cast_at(&mut t, P0, "Avacyn's Judgment", &[]);
    kill(&mut t, b);
    t.resolve_all();
    assert_eq!(t.obj_now(a).damage, 1);
}

#[test]
fn acorn_catapult_squirrel_even_if_the_damage_is_prevented() {
    cr!("608.2c", "615.1");
    ruling!(
        "Acorn Catapult",
        "If the ability resolves, but the damage is prevented or redirected, the targeted player or the controller of the targeted creature puts the Squirrel token onto the battlefield, regardless of how much damage was dealt or what the damage was ultimately dealt to."
    );
    supported("Acorn Catapult");
    // Prevented (Mending Hands on P1's Bears).
    let mut t = TestGame::new(2);
    let cat = t.battlefield(P0, "Acorn Catapult");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_at(&mut t, P0, "Mending Hands", &[obj(bears)]);
    t.resolve_all();
    act(&mut t, P0, cat, 0, &[obj(bears)]);
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(with_subtype(&t, P1, "Squirrel").len(), 1);
    assert!(with_subtype(&t, P0, "Squirrel").is_empty());
    // Redirected: P1 casts Harm's Way choosing the Catapult and redirects to P0.
    let mut t = TestGame::new(2);
    let cat = t.battlefield(P0, "Acorn Catapult");
    act_only(&mut t, P0, cat, 0, &[pl(P1)]);
    t.answer_choose(P1, &[obj(cat)]);
    t.answer_targets(P1, &[pl(P0)]);
    cast_at(&mut t, P1, "Harm's Way", &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 19);
    assert_eq!(with_subtype(&t, P1, "Squirrel").len(), 1);
    assert!(with_subtype(&t, P0, "Squirrel").is_empty());
}

#[test]
fn acorn_catapult_illegal_target_no_squirrel() {
    cr!("608.2b");
    ruling!(
        "Acorn Catapult",
        "If the permanent or player is an illegal target when the ability tries to resolve, it won't resolve and none of its effects will happen. No Squirrel will be created."
    );
    let mut t = TestGame::new(2);
    let cat = t.battlefield(P0, "Acorn Catapult");
    let bears = t.battlefield(P1, "Grizzly Bears");
    act_only(&mut t, P0, cat, 0, &[obj(bears)]);
    kill(&mut t, bears);
    t.resolve_all();
    assert!(with_subtype(&t, P1, "Squirrel").is_empty());
    // A player target that gets hexproof.
    let mut t = TestGame::new(2);
    let cat = t.battlefield(P0, "Acorn Catapult");
    act_only(&mut t, P0, cat, 0, &[pl(P1)]);
    hexproof(&mut t, P1);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(with_subtype(&t, P1, "Squirrel").is_empty());
}

#[test]
fn tataru_taru_and_ox_drover_illegal_opponent_no_card() {
    cr!("608.2b");
    ruling!(
        "Tataru Taru",
        "If the target opponent is an illegal target as Tataru Taru's first ability tries to resolve, it won't resolve and none of its effects will happen. You won't draw a card."
    );
    ruling!(
        "Ox Drover",
        "If the target opponent is an illegal target by the time Ox Drover's last ability tries to resolve, it won't resolve. They won't create an Ox token, and you won't draw a card."
    );
    supported("Tataru Taru");
    supported("Ox Drover");
    for (name, ox) in [("Tataru Taru", false), ("Ox Drover", true)] {
        // Legal: P0 draws (and P1 gets an Ox / may draw).
        let mut t = TestGame::new(2);
        let hand = t.hand_size(P0);
        t.answer_targets(P0, &[pl(P1)]);
        t.answer_yes(P1, false);
        t.enter(P0, name);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 1, "{name}");
        if ox {
            assert_eq!(with_subtype(&t, P1, "Ox").len(), 1);
        }
        // Illegal: nothing happens.
        let mut t = TestGame::new(2);
        let hand = t.hand_size(P0);
        t.answer_targets(P0, &[pl(P1)]);
        t.enter(P0, name);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        hexproof(&mut t, P1);
        let theirs = t.hand_size(P1);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand, "{name}: no card");
        assert_eq!(t.hand_size(P1), theirs);
        assert!(with_subtype(&t, P1, "Ox").is_empty());
    }
}

#[test]
fn skullwinder_illegal_target_returns_nothing() {
    cr!("608.2b");
    ruling!(
        "Skullwinder",
        "Skullwinder's enters-the-battlefield ability targets only the card in your graveyard. If that card is an illegal target as the ability tries to resolve, the ability won't resolve and none of its effects will happen. No cards will be returned to any player's hand."
    );
    supported("Skullwinder");
    let mut t = TestGame::new(2);
    let mine = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(mine)]);
    t.enter(P0, "Skullwinder");
    t.settle();
    // Only one target: the card in P0's graveyard.
    let trig = *t.g.stack.last().unwrap();
    assert_eq!(stack_targets(&t, trig), vec![obj(mine)]);
    exile_card(&mut t, mine);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"), "nothing returned to P1");
    assert!(!t.in_hand(P1, "Hill Giant"));
}

#[test]
fn skullwinder_opponent_chosen_after_the_card_returns_and_they_choose_theirs() {
    cr!("608.2c", "603.3d");
    ruling!(
        "Skullwinder",
        "You choose the target card as you put Skullwinder's triggered ability on the stack, but you don't choose an opponent until after you have returned the card to your hand."
    );
    ruling!(
        "Skullwinder",
        "The chosen opponent gets to choose which card to return from their graveyard to their hand."
    );
    let mut t = TestGame::new(2);
    let mine = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    let theirs = t.graveyard(P1, "Savannah Lions");
    t.answer_targets(P0, &[obj(mine)]);
    // When P0 is asked for the opponent, the Bears are already in P0's hand.
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseEntities { candidates, .. }
            if candidates.iter().any(|e| matches!(e, Entity::Player(_)))),
        |g| {
            g.player(P0)
                .hand
                .iter()
                .any(|id| g.obj(*id).chars.name == "Grizzly Bears")
        },
    );
    let m = mark(&t);
    t.answer_choose(P1, &[obj(theirs)]);
    t.enter(P0, "Skullwinder");
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // P1 chose the Lions over the Giant.
    assert!(!choices(&t, P1, m, "").is_empty(), "P1 chooses");
    assert!(t.in_hand(P1, "Savannah Lions"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    let seen = seen.lock().unwrap();
    assert!(!seen.is_empty(), "P0 chose the opponent");
    assert!(seen.iter().all(|in_hand| *in_hand), "{seen:?}");
}

#[test]
fn skullwinder_may_choose_an_opponent_with_an_empty_graveyard() {
    cr!("608.2c", "608.2d");
    ruling!(
        "Skullwinder",
        "You choose the opponent as the ability is resolving. You may choose an opponent with no cards in their graveyard."
    );
    ruling!(
        "Skullwinder",
        "You may choose an opponent with no cards in their graveyard. In that case, they will not get to return anything."
    );
    let mut t = TestGame::new(3);
    let mine = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(mine)]);
    t.answer_choose(P0, &[pl(P2)]);
    let m = mark(&t);
    t.enter(P0, "Skullwinder");
    t.resolve_all();
    assert!(
        choices(&t, P0, m, "")
            .iter()
            .any(|c| c.contains(&pl(P1)) && c.contains(&pl(P2))),
        "P2 was a choice"
    );
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.hand_size(P2), 0);
}

#[test]
fn pulsemage_advocate_needs_one_opponent_with_three_cards_and_your_creature_card() {
    cr!("602.2b", "601.2c", "115.1");
    ruling!(
        "Pulsemage Advocate",
        "You can’t activate this ability unless a single opponent has at least three cards in their graveyard to target and you have a creature card in your graveyard to target."
    );
    supported("Pulsemage Advocate");
    // Two cards each in two opponents' graveyards: not activatable.
    let mut t = TestGame::new(3);
    let adv = t.battlefield(P0, "Pulsemage Advocate");
    t.graveyard(P0, "Grizzly Bears");
    for p in [P1, P2] {
        t.graveyard(p, "Island");
        t.graveyard(p, "Island");
    }
    assert!(t.activate(P0, adv, 0, &[]).is_err());
    assert!(t.on_battlefield(adv) && !t.obj_now(adv).tapped);
    // Three in one opponent's graveyard but no creature card in P0's: not activatable.
    let mut t = TestGame::new(2);
    let adv = t.battlefield(P0, "Pulsemage Advocate");
    t.graveyard(P0, "Island");
    let cards: Vec<_> = (0..3).map(|_| t.graveyard(P1, "Island")).collect();
    assert!(t.activate(P0, adv, 0, &[]).is_err());
    // Both: activatable, and it resolves.
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &cards.iter().map(|c| obj(*c)).collect::<Vec<_>>());
    t.answer_targets(P0, &[obj(bears)]);
    t.g.turn.priority = Some(P0);
    t.activate(P0, adv, 0, &[]).expect("activatable");
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 3);
    assert!(named_here(&t, "Grizzly Bears"));
}

#[test]
fn dawnbreak_reclaimer_must_choose_a_creature_card_if_any_opponent_has_one() {
    cr!("608.2c", "608.2d");
    ruling!(
        "Dawnbreak Reclaimer",
        "If any opponent has a creature card in their graveyard as Dawnbreak Reclaimer’s ability resolves, then you must choose one of those cards. You can’t choose a different opponent with no creature cards in their graveyard to avoid returning one of those cards."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Dawnbreak Reclaimer");
    let giant = t.graveyard(P1, "Hill Giant");
    t.graveyard(P2, "Island");
    t.graveyard(P0, "Grizzly Bears");
    let m = mark(&t);
    // P0 tries to choose nothing (or P2) — the Giant must be chosen.
    t.answer_choose(P0, &[]);
    t.answer_yes(P0, true);
    end_step(&mut t);
    let asked = choices(&t, P0, m, "");
    assert!(!asked.is_empty());
    assert!(
        asked.iter().any(|c| c == &vec![obj(giant)]),
        "the Giant is the only choice: {asked:?}"
    );
    assert!(
        named_here(&t, "Hill Giant"),
        "the Giant was chosen and returned"
    );
    assert_eq!(ctl(&t, t.named_on_battlefield("Hill Giant")[0]), P1);
}

#[test]
fn faiths_shield_illegal_target_no_protection_for_anyone() {
    cr!("608.2b", "702.16a");
    ruling!(
        "Faith's Shield",
        "Faith’s Shield targets a permanent regardless of how much life you have. If that permanent is an illegal target when Faith’s Shield tries to resolve, Faith’s Shield won’t resolve and none of its effects will happen. Neither you nor any of your permanents will gain protection from a color."
    );
    supported("Faith's Shield");
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 5);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    let m = mark(&t);
    cast_at(&mut t, P0, "Faith's Shield", &[obj(bears)]);
    kill(&mut t, bears);
    t.resolve_all();
    assert!(
        t.asked()[m..]
            .iter()
            .all(|(_, d)| !matches!(d, Decision::ChooseOption { .. })),
        "no color is chosen"
    );
    // Nothing got protection from red: a red source's damage to P0 and the Giant is dealt.
    let goblin = t.battlefield(P1, "Raging Goblin");
    damage(&mut t, goblin, pl(P0), 1);
    damage(&mut t, goblin, obj(giant), 1);
    assert_eq!(t.life(P0), 4);
    assert_eq!(t.obj_now(giant).damage, 1);
}

#[test]
fn faiths_shield_player_protection_from_a_color() {
    cr!("702.16b", "702.16c", "702.16e", "303.4a", "704.5m");
    ruling!(
        "Faith's Shield",
        "A player with protection from a color can’t be damaged, enchanted (including by Curses), or targeted by anything of that color."
    );
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 5);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer(
        P0,
        DecisionKind::Option,
        Answer::Index(3), // red: W, U, B, R, G
    );
    let m = mark(&t);
    cast_at(&mut t, P0, "Faith's Shield", &[obj(bears)]);
    t.resolve_all();
    let red = t.asked()[m..].iter().any(|(_, d)| {
        matches!(d, Decision::ChooseOption { options, .. }
            if options.get(3).is_some_and(|o| o.to_lowercase().contains("red")))
    });
    assert!(red, "red was chosen");
    // Damage from a red source to P0 is prevented.
    let goblin = t.battlefield(P1, "Raging Goblin");
    damage(&mut t, goblin, pl(P0), 1);
    assert_eq!(t.life(P0), 5, "protection from red (check the color option)");
    // A red spell can't target P0.
    t.set_step(P1, Step::PrecombatMain);
    let m = mark(&t);
    t.answer_targets(P1, &[pl(P1)]);
    cast_at(&mut t, P1, "Shock", &[]);
    let cands = target_choices(&t, P1, m, "");
    assert!(cands.iter().all(|c| !c.contains(&pl(P0))));
    t.resolve_all();
    // A red Curse can't enchant P0: it can't be cast targeting P0.
    let m = mark(&t);
    t.answer_targets(P1, &[pl(P1)]);
    lots(&mut t, P1);
    let curse = t.hand(P1, "Curse of the Pierced Heart");
    let _ = t.cast(P1, curse).try_go();
    let cands = target_choices(&t, P1, m, "");
    assert!(!cands.is_empty());
    assert!(cands.iter().all(|c| !c.contains(&pl(P0))));
    t.resolve_all();
    // A red Curse that somehow becomes attached to P0 is put into its owner's graveyard.
    let curse = t.battlefield(P1, "Curse of the Pierced Heart");
    t.g.attach(curse, pl(P0));
    t.settle();
    assert!(t.in_graveyard(P1, "Curse of the Pierced Heart"));
    assert!(!t.on_battlefield(curse));
}

#[test]
fn fatal_push_targets_any_creature_mana_value_checked_on_resolution() {
    cr!("608.2c", "115.1", "202.3");
    ruling!(
        "Fatal Push",
        "Fatal Push can target any creature, even one with mana value 5 or greater. The creature's mana value is checked only as Fatal Push resolves."
    );
    supported("Fatal Push");
    let mut t = TestGame::new(2);
    let big = t.battlefield(P1, "Craw Wurm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let m = mark(&t);
    cast_at(&mut t, P0, "Fatal Push", &[obj(big)]);
    let cands = target_choices(&t, P0, m, "");
    assert!(cands[0].contains(&obj(big)) && cands[0].contains(&obj(bears)));
    t.resolve_all();
    assert!(t.on_battlefield(big), "mana value 6: not destroyed");
    cast_at(&mut t, P0, "Fatal Push", &[obj(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn duelists_heritage_triggers_on_any_players_attack() {
    cr!("508.1m", "508.3a", "603.2");
    ruling!(
        "Duelist's Heritage",
        "The ability of Duelist's Heritage triggers whenever any player attacks with one or more creatures, not just when you do. You know which players and planeswalkers the creatures are attacking before you choose the target creature."
    );
    supported("Duelist's Heritage");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Duelist's Heritage");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lions = t.battlefield(P1, "Savannah Lions");
    t.set_step(P1, Step::BeginningOfCombat);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(lions)]);
    // When P0 chooses the target, the attacks are declared.
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseTargets { .. }),
        |g| g.combat.as_ref().map_or(0, |c| c.attackers.len()),
    );
    attack_with(&mut t, &[(bears, pl(P0)), (lions, pl(P2))]);
    t.resolve_all();
    assert_eq!(seen.lock().unwrap().first().copied(), Some(2));
    assert!(t
        .obj_now(lions)
        .has_keyword(mtg_engine::keywords::KeywordKind::DoubleStrike));
    assert!(!t
        .obj_now(bears)
        .has_keyword(mtg_engine::keywords::KeywordKind::DoubleStrike));
}

#[test]
fn soldevi_golem_untaps_the_target_even_if_already_untapped() {
    cr!("608.2c", "701.26b");
    ruling!(
        "Soldevi Golem",
        "The ability to untap a creature can be used even if the Golem is already untapped."
    );
    supported("Soldevi Golem");
    let mut t = TestGame::new(2);
    let golem = t.battlefield(P0, "Soldevi Golem");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // P1's Bears become tapped in P1's end step.
    t.set_step(P1, Step::End);
    tap(&mut t, bears);
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "untap"), 1);
    assert!(!tapped(&t, golem));
    t.resolve_all();
    assert!(!tapped(&t, bears), "the Bears were untapped");
    assert!(!tapped(&t, golem));
}

#[test]
fn gift_of_growth_kicked_untaps_then_plus_four() {
    cr!("608.2c", "702.33d");
    ruling!(
        "Gift of Growth",
        "If Gift of Growth was kicked, the target creature is untapped before getting +4/+4."
    );
    supported("Gift of Growth");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    tap(&mut t, bears);
    lots(&mut t, P0);
    let c = t.hand(P0, "Gift of Growth");
    t.cast(P0, c).kicked(true).target(bears).go();
    t.resolve_all();
    assert!(!tapped(&t, bears));
    assert_eq!(t.pt(bears), (6, 6));
}

#[test]
fn gideons_triumph_targets_the_player_hexproof_creature_can_be_sacrificed() {
    cr!("115.1", "701.21a");
    ruling!(
        "Gideon's Triumph",
        "Gideon’s Triumph targets the player, not the creature or creatures to be sacrificed. A creature with hexproof may be sacrificed this way."
    );
    ruling!(
        "Gideon's Triumph",
        "Gideon’s Triumph can be cast before combat damage has been dealt. In this case, it can be cast before or after blockers have been chosen."
    );
    supported("Gideon's Triumph");
    // Before blockers.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::BeginningOfCombat);
    let hex = t.battlefield(P1, "Gladecover Scout");
    attack_with(&mut t, &[(hex, pl(P0))]);
    cast_at(&mut t, P0, "Gideon's Triumph", &[pl(P1)]);
    t.resolve_all();
    assert!(!t.on_battlefield(hex));
    // After blockers, before damage: a blocker.
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Hill Giant");
    let blocker = t.battlefield(P1, "Gladecover Scout");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(attacker, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(blocker, attacker)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.settle();
    cast_at(&mut t, P0, "Gideon's Triumph", &[pl(P1)]);
    t.resolve_all();
    assert!(!t.on_battlefield(blocker));
}

#[test]
fn gideons_triumph_attackers_removed_from_combat_and_put_onto_the_battlefield_attacking() {
    cr!("506.4", "508.4", "701.21a");
    ruling!(
        "Gideon's Triumph",
        "A creature that attacked and was removed from combat (such as by Spires of Orazca) still attacked, so it can be sacrificed."
    );
    ruling!(
        "Gideon's Triumph",
        "A creature that was put onto the battlefield attacking didn’t attack, so it can’t be sacrificed."
    );
    // Removed from combat: still attacked.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::BeginningOfCombat);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(bears, pl(P0))]);
    mtg_engine::combat::remove_from_combat(&mut t.g, bears);
    t.g.recompute();
    cast_at(&mut t, P0, "Gideon's Triumph", &[pl(P1)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    // Hero of Bladehold's Soldiers entered attacking: they can't be sacrificed, the
    // Hero is.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::BeginningOfCombat);
    let hero = t.battlefield(P1, "Hero of Bladehold");
    attack_with(&mut t, &[(hero, pl(P0))]);
    t.resolve_all();
    let soldiers = with_subtype(&t, P1, "Soldier");
    assert_eq!(soldiers.len(), 2);
    let m = mark(&t);
    t.answer_choose(P1, &[obj(soldiers[0])]);
    cast_at(&mut t, P0, "Gideon's Triumph", &[pl(P1)]);
    t.resolve_all();
    assert!(!t.on_battlefield(hero));
    assert_eq!(with_subtype(&t, P1, "Soldier").len(), 2);
    for c in choices(&t, P1, m, "") {
        assert!(c.iter().all(|e| !soldiers.iter().any(|s| *e == obj(*s))));
    }
}

#[test]
fn gideons_triumph_after_combat_damage_survivors() {
    cr!("510.2", "701.21a");
    ruling!(
        "Gideon's Triumph",
        "Gideon’s Triumph can be cast after combat damage has been dealt. In this case, the target player chooses one or two creatures that attacked or blocked this turn and have survived combat damage."
    );
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Stone");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lions = t.battlefield(P1, "Savannah Lions");
    let idle = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, pl(P0)), (lions, pl(P0))]);
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(wall, lions)]),
    );
    t.advance_to(P1, Step::EndOfCombat);
    t.settle();
    assert!(t.on_battlefield(lions) && t.on_battlefield(bears));
    let m = mark(&t);
    t.answer_choose(P1, &[obj(lions)]);
    cast_at(&mut t, P0, "Gideon's Triumph", &[pl(P1)]);
    t.resolve_all();
    let asked = choices(&t, P1, m, "");
    assert!(asked
        .iter()
        .any(|c| c.contains(&obj(lions)) && c.contains(&obj(bears)) && !c.contains(&obj(idle))));
    assert!(!t.on_battlefield(lions));
    assert!(t.on_battlefield(bears) && t.on_battlefield(idle));
}
