//! Rulings batch P128 — abilities that work while the card is in its owner's graveyard:
//! activated abilities that return it (activatable only there, CR 113.6k, 602.5b),
//! triggered abilities that trigger only if it's in the graveyard when the event happens
//! (CR 113.6k, 603.4), and conditions that look back over the whole turn.

use crate::r_p128_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn return_abilities_can_be_activated_only_from_the_graveyard() {
    cr!("113.6m", "602.2");
    ruling!(
        "Stitchwing Skaab",
        "Stitchwing Skaab's last ability can only be activated while it's in your graveyard."
    );
    ruling!(
        "Necrosavant",
        "The ability is activated while this card is in the graveyard."
    );
    ruling!(
        "Evershrike",
        "You can activate Evershrike's ability only while it's in your graveyard."
    );
    ruling!(
        "Kuldotha Phoenix",
        "You can activate Kuldotha Phoenix's ability only if it's in your graveyard."
    );
    type Setup = fn(&mut TestGame);
    // (card, the ability's conditions and costs, the step it's activated in)
    let cases: [(&str, Setup, Step); 4] = [
        (
            "Stitchwing Skaab",
            |t| {
                t.lands(P0, "Island", 2);
                t.hand(P0, "Forest");
                t.hand(P0, "Forest");
            },
            Step::PrecombatMain,
        ),
        (
            "Necrosavant",
            |t| {
                t.lands(P0, "Swamp", 5);
                t.battlefield(P0, "Grizzly Bears");
            },
            Step::Upkeep,
        ),
        (
            "Evershrike",
            |t| {
                t.lands(P0, "Plains", 2);
            },
            Step::PrecombatMain,
        ),
        (
            "Kuldotha Phoenix",
            |t| {
                t.lands(P0, "Wastes", 4);
                for _ in 0..3 {
                    t.battlefield(P0, "Ornithopter");
                }
            },
            Step::Upkeep,
        ),
    ];
    for (card, setup, step) in cases {
        supported(card);
        let mut t = TestGame::new(2);
        setup(&mut t);
        t.set_step(P0, step);
        let on_bf = t.battlefield(P0, card);
        assert!(!activatable(&mut t, P0, on_bf), "{card} on the battlefield");
        let c = t.graveyard(P0, card);
        assert!(activatable(&mut t, P0, c), "{card} in the graveyard");
        t.answer(P0, DecisionKind::X, Answer::Number(0));
        activate_containing(&mut t, P0, c, "Return ~").unwrap();
        t.resolve_all();
        if card == "Evershrike" {
            // With X = 0 no Aura is put onto it: it's exiled again (see
            // `evershrike_is_returned_then_exiled_without_an_aura`).
            assert!(is_in(&t, c, Zone::Exile), "{card}");
        } else {
            assert!(t.on_battlefield(c), "{card}");
        }
    }
}

#[test]
fn bloodsoaked_champions_own_attack_satisfies_raid() {
    cr!("113.6m", "602.1b", "508.1");
    ruling!(
        "Bloodsoaked Champion",
        "The activated ability can be activated only if Bloodsoaked Champion is in your graveyard. Notably, if it attacks and then dies later in the turn, you can use its ability to return it to the battlefield, as its attack satisfies its own activation instruction."
    );
    supported("Bloodsoaked Champion");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let bc = t.battlefield(P0, "Bloodsoaked Champion");
    let early = t.graveyard(P0, "Bloodsoaked Champion");
    assert!(!activatable(&mut t, P0, early), "no attack yet this turn");
    attack_with(&mut t, &[(bc, Entity::Player(P1))]);
    assert!(!activatable(&mut t, P0, bc), "on the battlefield");
    destroy(&mut t, bc);
    assert!(is_in(&t, bc, Zone::Graveyard(P0)));
    assert!(activatable(&mut t, P0, bc));
    activate_containing(&mut t, P0, bc, "Return ~").unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(bc));
}

#[test]
fn coffin_puppets_can_sacrifice_the_last_swamp() {
    cr!("602.1b", "602.2b", "601.2h");
    ruling!(
        "Coffin Puppets",
        "The ability works even if you sacrifice your last swamp to activate it. It only checks for a swamp on activation."
    );
    supported("Coffin Puppets");
    for (land, ok) in [("Forest", false), ("Swamp", true)] {
        let mut t = TestGame::new(2);
        let cp = t.graveyard(P0, "Coffin Puppets");
        t.lands(P0, land, 2);
        t.set_step(P0, Step::Upkeep);
        assert_eq!(activatable(&mut t, P0, cp), ok, "{land}");
        if ok {
            activate_containing(&mut t, P0, cp, "Return ~").unwrap();
            assert!(t.g.permanents().all(|o| !o.is(CardType::Land)));
            t.resolve_all();
            assert!(t.on_battlefield(cp));
        }
    }
}

#[test]
fn cult_conscript_needs_a_death_this_turn_from_any_zone() {
    cr!("602.1b", "700.4");
    ruling!(
        "Cult Conscript",
        "Similarly, Cult Conscript didn't need to be any specific zone at the time that creature died."
    );
    ruling!(
        "Cult Conscript",
        "The creature that died doesn't have to still be in its owner's graveyard to satisfy the activation restriction."
    );
    supported("Cult Conscript");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let early = t.graveyard(P0, "Cult Conscript");
    assert!(!activatable(&mut t, P0, early), "nothing died yet");
    // Grizzly Bears dies while this Cult Conscript is in P0's hand, then is exiled.
    let cc = t.hand(P0, "Cult Conscript");
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    exile_now(&mut t, bears);
    let cc = move_to(&mut t, cc, Zone::Graveyard(P0)).unwrap();
    assert!(activatable(&mut t, P0, cc));
    activate_containing(&mut t, P0, cc, "Return ~").unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(cc));
}

#[test]
fn sekki_can_sacrifice_any_spirits() {
    cr!("602.2b", "205.3m");
    ruling!(
        "Sekki, Seasons' Guide",
        "You can sacrifice any eight Spirits to pay for Sekki's activated ability, not just the Spirit creature tokens."
    );
    supported("Sekki, Seasons' Guide");
    let mut t = TestGame::new(2);
    let sk = t.graveyard(P0, "Sekki, Seasons' Guide");
    for _ in 0..7 {
        t.battlefield(P0, "Will-o'-the-Wisp");
    }
    assert!(!activatable(&mut t, P0, sk), "seven Spirits");
    t.battlefield(P0, "Will-o'-the-Wisp");
    assert!(activatable(&mut t, P0, sk));
    activate_containing(&mut t, P0, sk, "Return ~").unwrap();
    assert!(t.named_on_battlefield("Will-o'-the-Wisp").is_empty());
    t.resolve_all();
    assert!(t.on_battlefield(sk));
    assert_eq!(t.counters(sk, counters::PLUS1), 8);
}

#[test]
fn evershrike_puts_only_an_aura_that_can_enchant_it() {
    cr!("608.2", "303.4i");
    ruling!(
        "Evershrike",
        "You can't put an Aura onto the battlefield this way unless that Aura can legally be attached to Evershrike. For example, you can't put an Aura card with \"enchant player\" onto the battlefield this way."
    );
    supported("Evershrike");
    // (Aura in hand, X) -> it's put onto the battlefield attached to Evershrike
    for (aura, x, attached) in [
        ("Curse of the Pierced Heart", 2, false),
        ("Rancor", 1, true),
    ] {
        let mut t = TestGame::new(2);
        let es = t.graveyard(P0, "Evershrike");
        t.lands(P0, "Plains", 2);
        t.lands(P0, "Wastes", x as usize);
        let a = t.hand(P0, aura);
        t.answer(P0, DecisionKind::X, Answer::Number(x));
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(a)]);
        activate_containing(&mut t, P0, es, "Return ~").unwrap();
        t.resolve_all();
        assert_eq!(t.on_battlefield(a), attached, "{aura}");
        if attached {
            assert!(t.on_battlefield(es));
            assert_eq!(attached_to(&t, a), Some(obj(t.g.current(es))));
            assert_eq!(t.pt(es), (6, 4));
        } else {
            assert!(is_in(&t, a, Zone::Hand(P0)), "{aura}");
            // No Aura was put onto it: it's exiled.
            assert!(is_in(&t, es, Zone::Exile), "{aura}");
        }
    }
}

#[test]
fn evershrike_is_returned_then_exiled_without_an_aura() {
    cr!("400.7j", "608.2", "603.2");
    ruling!(
        "Evershrike",
        "You can activate Evershrike's ability with X = 0. If you do, you'll return Evershrike to the battlefield from your graveyard, then exile it. (There are no Aura cards with mana value 0.)"
    );
    ruling!(
        "Evershrike",
        "You can activate Evershrike's ability even if you have no Auras in your hand, or if you have an Aura in your hand that you don't want to put on it or can't put on it. In either case, you'll return Evershrike to the battlefield from your graveyard, then exile it. No player has the opportunity to cast spells or activate abilities while Evershrike is on the battlefield."
    );
    supported("Evershrike");
    supported("Soul Warden");
    // (Aura card in hand, X, whether P0 chooses to put an Aura onto it) -> it stays
    let cases: [(Option<&str>, i64, bool, bool); 5] = [
        // X = 0: there are no Aura cards with mana value 0.
        (Some("Rancor"), 0, true, false),
        // No Aura card in hand.
        (None, 2, true, false),
        // An Aura P0 doesn't want to put onto it.
        (Some("Rancor"), 1, false, false),
        // An Aura that can't enchant it.
        (Some("Curse of the Pierced Heart"), 2, true, false),
        // (Control: an Aura is put onto it.)
        (Some("Rancor"), 1, true, true),
    ];
    for (aura, x, put, stays) in cases {
        let what = format!("{aura:?} X={x} put={put}");
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Soul Warden");
        let es = t.graveyard(P0, "Evershrike");
        t.lands(P0, "Plains", 2);
        t.lands(P0, "Wastes", x as usize);
        let a = aura.map(|n| t.hand(P0, n));
        t.answer(P0, DecisionKind::X, Answer::Number(x));
        t.answer_yes(P0, put);
        if let Some(a) = a {
            t.answer_choose(P0, &[obj(a)]);
        }
        activate_containing(&mut t, P0, es, "Return ~").unwrap();
        assert_eq!(t.stack_len(), 1, "{what}");
        t.resolve();
        // It entered the battlefield (Soul Warden's ability triggered) and, without an
        // Aura, was exiled while the ability was still resolving: by the time anyone
        // could act, it's gone.
        t.settle();
        assert_eq!(triggers_on_stack(&t, "gain 1 life"), 1, "{what}: it entered");
        assert_eq!(t.on_battlefield(es), stays, "{what}");
        if !stays {
            assert!(is_in(&t, es, Zone::Exile), "{what}");
        }
        t.resolve_all();
        assert_eq!(t.life(P0), 21, "{what}");
    }
}

#[test]
fn deathpact_angels_cleric_token() {
    cr!("707.2", "111.3");
    ruling!(
        "Deathpact Angel",
        "The activated ability is part of the Cleric token's copiable values. Any copy of that creature token will also have that ability."
    );
    ruling!(
        "Deathpact Angel",
        "The card named Deathpact Angel that you return to the battlefield doesn't necessarily have to be the same one that created the token you sacrificed."
    );
    supported("Deathpact Angel");
    let mut t = TestGame::new(2);
    let first = t.battlefield(P0, "Deathpact Angel");
    destroy(&mut t, first);
    t.resolve_all();
    let cleric = crate::r_s01_common::with_subtype(&t, P0, "Cleric")[0];
    // A copy of the token has the ability too.
    let copy = crate::r_s17_common::token_copy(&mut t, P0, cleric)[0];
    assert!(t
        .obj(copy)
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains("named")));
    // The Angel that made the token is exiled; another one is in the graveyard.
    exile_now(&mut t, first);
    let second = t.graveyard(P0, "Deathpact Angel");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Wastes", 3);
    t.g.objects[copy.0 as usize].summoning_sick = false;
    t.answer_choose(P0, &[obj(second)]);
    activate_containing(&mut t, P0, copy, "named").unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(second));
    assert!(is_in(&t, first, Zone::Exile));
}

#[test]
fn slimefoot_and_squee_target_is_optional_but_an_illegal_one_fizzles() {
    cr!("608.2b", "601.2c", "113.6m");
    ruling!(
        "Slimefoot and Squee",
        "You don't have to choose a target to activate the last ability. However, if you do, and that target is illegal at the time the ability tries to resolve, it won't resolve and none of its effects will happen. Slimefoot and Squee won't be returned to the battlefield."
    );
    supported("Slimefoot and Squee");
    for (target, exile_it) in [(false, false), (true, false), (true, true)] {
        let mut t = TestGame::new(2);
        let ss = t.graveyard(P0, "Slimefoot and Squee");
        let bears = t.graveyard(P0, "Grizzly Bears");
        create_token(&mut t, P0, "Saproling");
        for l in ["Swamp", "Mountain", "Forest", "Wastes"] {
            t.lands(P0, l, 1);
        }
        let ts: Vec<Entity> = if target { vec![obj(bears)] } else { vec![] };
        t.answer_targets(P0, &ts);
        activate_containing(&mut t, P0, ss, "Return ~").unwrap();
        if exile_it {
            exile_now(&mut t, bears);
        }
        t.resolve_all();
        assert_eq!(t.on_battlefield(ss), !exile_it, "{target} {exile_it}");
        assert_eq!(t.on_battlefield(bears), target && !exile_it);
    }
}

#[test]
fn graveyard_triggers_need_the_card_in_the_graveyard_as_the_event_happens() {
    cr!("113.6m", "603.2", "603.10");
    ruling!(
        "Prized Amalgam",
        "Prized Amalgam's ability triggers only if it's in your graveyard immediately after an appropriate creature enters the battlefield. A Prized Amalgam that's already on the battlefield won't be returned at the beginning of the next end step if it's put into your graveyard later."
    );
    ruling!(
        "Sword of the Meek",
        "Sword of the Meek's last ability triggers only if it's in your graveyard immediately after the 1/1 creature enters the battlefield."
    );
    supported("Prized Amalgam");
    supported("Sword of the Meek");
    for in_graveyard in [false, true] {
        // Prized Amalgam: Gravecrawler is cast from the graveyard and enters.
        let mut t = TestGame::new(2);
        let pa = if in_graveyard {
            t.graveyard(P0, "Prized Amalgam")
        } else {
            t.battlefield(P0, "Prized Amalgam")
        };
        t.battlefield(P0, "Walking Corpse");
        let gc = t.graveyard(P0, "Gravecrawler");
        t.lands(P0, "Swamp", 1);
        cast_any(&mut t, P0, gc);
        t.resolve_all();
        if !in_graveyard {
            destroy(&mut t, pa);
        }
        end_step(&mut t, P0);
        assert_eq!(
            t.on_battlefield(pa),
            in_graveyard,
            "Amalgam: {in_graveyard}"
        );
        // Sword of the Meek: a 1/1 enters.
        let mut t = TestGame::new(2);
        let sw = if in_graveyard {
            t.graveyard(P0, "Sword of the Meek")
        } else {
            t.battlefield(P0, "Sword of the Meek")
        };
        t.answer_yes(P0, true);
        let elves = t.enter(P0, "Llanowar Elves");
        t.settle();
        if !in_graveyard {
            destroy(&mut t, sw);
        }
        t.resolve_all();
        assert_eq!(t.on_battlefield(sw), in_graveyard, "Sword: {in_graveyard}");
        if in_graveyard {
            assert_eq!(attached_to(&t, sw), Some(obj(elves)));
        }
    }
}

#[test]
fn wolfbat_triggers_only_on_the_second_draw() {
    cr!("603.2", "113.6m", "121.2");
    ruling!(
        "Wolfbat",
        "The triggered ability can trigger only once each turn. It doesn't matter whether Wolfbat was in your graveyard when the first card was drawn. If it's not in your graveyard when the second card is drawn, the ability can't trigger at all that turn. It won't trigger when the third or fourth card is drawn."
    );
    supported("Wolfbat");
    // (cards drawn while it's in hand, cards drawn after it's put into the graveyard)
    for (before, after, returns) in [(1, 1, true), (2, 2, false)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Swamp", 1);
        let wb = t.hand(P0, "Wolfbat");
        t.g.draw_cards(P0, before);
        t.g.flush_events();
        t.settle();
        let wb = move_to(&mut t, wb, Zone::Graveyard(P0)).unwrap();
        for _ in 0..after {
            t.answer_yes(P0, true);
            t.g.draw_cards(P0, 1);
            t.g.flush_events();
            t.settle();
        }
        t.resolve_all();
        assert_eq!(t.on_battlefield(wb), returns, "{before}+{after}");
        if returns {
            assert_eq!(finality(&t, wb), 1);
        }
    }
}

#[test]
fn vengevine_counts_the_whole_turn_but_must_be_in_the_graveyard() {
    cr!("603.4", "113.6m", "601.2i");
    ruling!(
        "Vengevine",
        "Vengevine's triggered ability checks the spells you cast over the course of the entire turn, not just the ones you cast while Vengevine is in your graveyard."
    );
    ruling!(
        "Vengevine",
        "Vengevine's triggered ability works only if Vengevine is already in your graveyard as you finish casting your second creature spell in a turn and if it remains there until the triggered ability resolves."
    );
    supported("Vengevine");
    // When Vengevine is put into the graveyard: 0 = after the first creature spell, 1 =
    // while the second is on the stack; and whether it's exiled in response.
    for (when, exiled, returns) in [(0, false, true), (1, false, false), (0, true, false)] {
        let mut t = TestGame::new(2);
        let vv = t.hand(P0, "Vengevine");
        cast_new(&mut t, P0, "Grizzly Bears", &[]);
        t.resolve_all();
        let mut vv = if when == 0 {
            move_to(&mut t, vv, Zone::Graveyard(P0)).unwrap()
        } else {
            vv
        };
        t.answer_yes(P0, true);
        cast_new(&mut t, P0, "Hill Giant", &[]);
        t.settle();
        if when == 1 {
            vv = move_to(&mut t, vv, Zone::Graveyard(P0)).unwrap();
        }
        if exiled {
            assert_eq!(t.stack_len(), 2, "the trigger is on the stack");
            exile_now(&mut t, vv);
        }
        t.resolve_all();
        assert_eq!(t.on_battlefield(vv), returns, "{when} {exiled}");
    }
}

#[test]
fn return_triggers_with_a_payment_can_be_responded_to() {
    cr!("603.3", "608.2", "608.2g");
    ruling!(
        "Forsaken Miner",
        "You choose whether to pay {B} as the triggered ability is resolving. Once the ability starts resolving, it's too late for anyone to respond by removing Forsaken Miner from your graveyard to stop you from bringing it back."
    );
    ruling!(
        "Akoum Firebird",
        "Players can respond to the triggered ability, but once it starts resolving and you decide whether to pay {4}{R}{R}, it's too late for anyone to respond."
    );
    supported("Forsaken Miner");
    supported("Akoum Firebird");
    for card in ["Forsaken Miner", "Akoum Firebird"] {
        for respond in [false, true] {
            let mut t = TestGame::new(2);
            let c = t.graveyard(P0, card);
            t.answer_yes(P0, true);
            if card == "Forsaken Miner" {
                t.lands(P0, "Swamp", 1);
                // Committing a crime: Shock targeting P1.
                cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
            } else {
                t.lands(P0, "Mountain", 6);
                let m = t.hand(P0, "Mountain");
                t.play_land(P0, m).unwrap();
            }
            t.settle();
            assert!(triggers_on_stack(&t, "return ~") == 1, "{card}");
            if respond {
                exile_now(&mut t, c);
            }
            // While it resolves, the payment and the return happen with nobody getting
            // priority in between: the card is still in the graveyard when P0 decides.
            let seen = watch(
                &mut t,
                P0,
                |d| matches!(d, Decision::YesNo { .. }),
                |g| g.stack.len(),
            );
            t.resolve_all();
            assert_eq!(t.on_battlefield(c), !respond, "{card} respond: {respond}");
            if !respond {
                assert!(!seen.lock().unwrap().is_empty(), "{card}: asked to pay");
            }
        }
    }
}

#[test]
fn silversmote_ghoul_counts_all_life_gained_this_turn() {
    cr!("113.6m", "603.4");
    ruling!(
        "Silversmote Ghoul",
        "Silversmote Ghoul's first ability looks at how much life you've gained in the turn, even if it wasn't on the battlefield or in your graveyard when you gained life. It doesn't care if you also lost life, even if you lost more life than you gained."
    );
    supported("Silversmote Ghoul");
    for (gained, returns) in [(3, true), (2, false)] {
        let mut t = TestGame::new(2);
        let sg = t.hand(P0, "Silversmote Ghoul");
        t.g.gain_life(P0, gained);
        t.g.lose_life(P0, 5);
        t.g.flush_events();
        let sg = move_to(&mut t, sg, Zone::Graveyard(P0)).unwrap();
        end_step(&mut t, P0);
        assert_eq!(t.on_battlefield(sg), returns, "{gained}");
        if returns {
            assert!(t.obj_now(sg).tapped);
        }
    }
}

#[test]
fn sandmans_characteristic_defining_ability_works_in_the_graveyard() {
    cr!("604.3", "113.6a");
    ruling!(
        "Sandman, Shifting Scoundrel",
        "Sandman's first ability works in all zones, not only the battlefield."
    );
    supported("Sandman, Shifting Scoundrel");
    let mut t = TestGame::new(2);
    let s = t.graveyard(P0, "Sandman, Shifting Scoundrel");
    t.lands(P0, "Forest", 3);
    t.g.recompute();
    assert_eq!(t.pt(s), (3, 3));
    t.lands(P0, "Forest", 1);
    t.g.recompute();
    assert_eq!(t.pt(s), (4, 4));
}

#[test]
fn nethergoyfs_characteristic_defining_ability_works_in_the_graveyard_and_hand() {
    cr!("604.3", "113.6a");
    ruling!(
        "Nethergoyf",
        "The ability that defines Nethergoyf's power and toughness works in all zones, not just the battlefield."
    );
    supported("Nethergoyf");
    let mut t = TestGame::new(2);
    let ng = t.graveyard(P0, "Nethergoyf");
    t.g.recompute();
    // Only itself (a creature card) in the graveyard.
    assert_eq!(t.pt(ng), (1, 2));
    t.graveyard(P0, "Forest");
    t.graveyard(P0, "Lightning Bolt");
    t.g.recompute();
    assert_eq!(t.pt(ng), (3, 4), "creature, land, instant");
    // In the hand too.
    let in_hand = t.hand(P0, "Nethergoyf");
    t.g.recompute();
    assert_eq!(t.pt(in_hand), (3, 4));
}

#[test]
fn sephiroth_sets_base_power_and_toughness_over_earlier_effects() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Sephiroth, Fallen Hero",
        "Sephiroth's first ability will overwrite any previous effects that set the creature's power and toughness to specific numbers. Effects that otherwise modify its power and toughness will still apply no matter when they took effect. The same is true for +1/+1 counters."
    );
    supported("Sephiroth, Fallen Hero");
    supported("Turn to Frog");
    let mut t = TestGame::new(2);
    let seph = t.battlefield(P0, "Sephiroth, Fallen Hero");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, bears, counters::PLUS1, 1);
    cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    t.resolve_all();
    cast_new(&mut t, P0, "Turn to Frog", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5), "base 1/1, +3/+3, a +1/+1 counter");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(seph, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(bears, "cell"), 1);
    assert_eq!(t.pt(bears), (11, 9), "base 7/5, +3/+3, a +1/+1 counter");
}

#[test]
fn nether_shadows_upkeep_trigger_cannot_loop() {
    cr!("603.4", "503.1a", "404.1");
    ruling!(
        "Nether Shadow",
        "Since it enters due to triggering at the beginning of upkeep, it is not possible to get an infinite loop with four Nether Shadows."
    );
    supported("Nether Shadow");
    let mut t = TestGame::new(2);
    // Four Nether Shadows, the first at the bottom of the graveyard.
    let shadows: Vec<_> = (0..4).map(|_| t.graveyard(P0, "Nether Shadow")).collect();
    for _ in 0..4 {
        t.answer_yes(P0, true);
    }
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    // Only the bottom one had three creature cards above it as the upkeep began.
    assert!(t.on_battlefield(shadows[0]));
    for s in &shadows[1..] {
        assert!(is_in(&t, *s, Zone::Graveyard(P0)));
    }
    t.advance_to(P0, Step::Draw);
    assert_eq!(t.named_on_battlefield("Nether Shadow").len(), 1);
}

#[test]
fn skeleton_crew_returning_doesnt_trigger_itself() {
    cr!("113.6", "603.10", "603.10a");
    ruling!(
        "Skeleton Crew",
        "Resolving Skeleton Crew's last ability won't cause its second ability to trigger."
    );
    supported("Skeleton Crew");
    for other in [false, true] {
        let mut t = TestGame::new(2);
        let sc = t.graveyard(P0, "Skeleton Crew");
        if other {
            t.battlefield(P0, "Skeleton Crew");
        }
        t.lands(P0, "Swamp", 6);
        activate_containing(&mut t, P0, sc, "Return ~").unwrap();
        t.resolve_all();
        assert!(t.on_battlefield(sc));
        let pirates =
            t.g.permanents()
                .filter(|o| o.is_token() && o.chars.has_subtype("Pirate"))
                .count();
        assert_eq!(pirates, usize::from(other), "another Crew: {other}");
    }
}

#[test]
fn squee_chooses_what_its_goblin_token_attacks() {
    cr!("508.4", "506.2");
    ruling!(
        "Squee, Dubious Monarch",
        "Squee's controller chooses which player or planeswalker the Goblin token is attacking. It doesn't need to be the same player or planeswalker that Squee is attacking."
    );
    supported("Squee, Dubious Monarch");
    let mut t = TestGame::new(3);
    let sq = t.battlefield(P0, "Squee, Dubious Monarch");
    t.answer_choose(P0, &[Entity::Player(P2)]);
    attack_with(&mut t, &[(sq, Entity::Player(P1))]);
    t.resolve_all();
    let goblin = crate::r_s01_common::with_subtype(&t, P0, "Goblin")
        .into_iter()
        .find(|g| t.obj(*g).is_token())
        .expect("a Goblin token");
    let c = t.g.combat.as_ref().unwrap();
    let target = c
        .attackers
        .iter()
        .find(|a| a.id == goblin)
        .map(|a| a.target);
    assert_eq!(target, Some(Some(Entity::Player(P2))));
    let target = c.attackers.iter().find(|a| a.id == sq).map(|a| a.target);
    assert_eq!(target, Some(Some(Entity::Player(P1))));
}

#[test]
fn syrix_phoenix_deals_damage_equal_to_its_own_power() {
    cr!("608.2h", "603.4");
    ruling!(
        "Syrix, Carrier of the Flame",
        "The damage dealt by the target Phoenix as Syrix's first triggered ability resolves is equal to its own power, not the power of the creature card that left the graveyard this turn."
    );
    supported("Syrix, Carrier of the Flame");
    let mut t = TestGame::new(2);
    let sx = t.battlefield(P0, "Syrix, Carrier of the Flame");
    let wurm = t.graveyard(P0, "Craw Wurm");
    exile_now(&mut t, wurm);
    t.answer_targets(P0, &[obj(sx)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    end_step(&mut t, P0);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn dennick_stops_only_targeting_cards_in_graveyards() {
    cr!("115.1", "115.10");
    ruling!(
        "Dennick, Pious Apprentice // Dennick, Pious Apparition",
        "The second ability of Dennick, Pious Apprentice affects only spells and abilities that actually target cards in graveyards."
    );
    supported("Dennick, Pious Apprentice // Dennick, Pious Apparition");
    supported("Tormod's Crypt");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dennick, Pious Apprentice // Dennick, Pious Apparition");
    let bears = t.graveyard(P1, "Grizzly Bears");
    // Rotten Reunion ("Exile up to one target card from a graveyard") can't target it.
    t.answer_targets(P0, &[obj(bears)]);
    cast_new(&mut t, P0, "Rotten Reunion", &[]);
    t.resolve_all();
    assert!(is_in(&t, bears, Zone::Graveyard(P1)));
    assert_eq!(crate::r_p108_common::tokens_with(&t, P0, "Zombie"), 1);
    t.clear_answers();
    // Tormod's Crypt targets the player and exiles the whole graveyard.
    let crypt = t.battlefield(P0, "Tormod's Crypt");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, crypt, "Exile target player").unwrap();
    t.resolve_all();
    assert!(is_in(&t, bears, Zone::Exile));
}
