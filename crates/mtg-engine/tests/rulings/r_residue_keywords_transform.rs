//! Item `rulings-residue-keywords` — transform and Treasure interactions: transforming
//! permanents stay the same object (CR 712.18, 400.7), checks made as an ability resolves
//! (608.2h), loyalty abilities once per turn per permanent (606.3), copies chosen as a
//! permanent transforms (712.14), choices made by opponents in turn order (101.4), triggers
//! resolving before the spell that caused them (603.3), destroy-or-not (701.8), illegal
//! targets (608.2b), comparisons on resolution, and tripled combat damage with trample.

use crate::r_residue_keywords_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{counters, CardType};
use mtg_engine::*;

use crate::r_s06_common::activate_containing;

/// Whether the permanent (followed across zone changes) shows its back face.
fn transformed(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).face == mtg_engine::object::FaceState::Back
}

#[test]
fn chandra_checks_her_damage_only_as_her_ability_resolves() {
    cr!("608.2h", "400.7", "602.1");
    ruling!(
        "Chandra, Fire of Kaladesh // Chandra, Roaring Flame",
        "The last sentence of Chandra, Fire of Kaladesh's activated ability isn't a separate ability. The check happens only as that activated ability resolves. You must activate the ability in order to exile Chandra and return her to the battlefield transformed, even if Chandra has already dealt 3 or more damage during the turn."
    );
    supported("Chandra, Fire of Kaladesh // Chandra, Roaring Flame");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Chandra, Fire of Kaladesh // Chandra, Roaring Flame");
    // Chandra (2/2, +1/+0 this turn) deals 3 combat damage: nothing happens.
    crate::r_p214_common::pump(&mut t, c, 1, 0);
    attack_with(&mut t, &[(c, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 17);
    t.set_step(P0, Step::PostcombatMain);
    t.resolve_all();
    assert!(t.on_battlefield(c));
    assert!(!transformed(&t, c));
    // Her ability (untapped, "somehow"): as it resolves she has dealt 4: she's exiled and
    // returns transformed.
    t.g.objects[c.0 as usize].tapped = false;
    t.activate(P0, c, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    let back = t.g.current(c);
    assert_ne!(back, c);
    assert!(t.on_battlefield(back) && transformed(&t, back));
    assert!(t.obj_now(back).is(CardType::Planeswalker));
}

#[test]
fn gwen_stacy_exiled_card_stays_playable_after_she_transforms() {
    cr!("712.18", "611.2b", "400.7");
    ruling!(
        "Gwen Stacy // Ghost-Spider",
        "Transforming Gwen Stacy doesn't cause her to leave the battlefield, which means that the card you exiled when she entered can still be played after she transforms."
    );
    supported("Gwen Stacy // Ghost-Spider");
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    let g = t.enter(P0, "Gwen Stacy // Ghost-Spider");
    t.resolve_all();
    assert_eq!(t.zone(forest), Zone::Exile);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    t.activate(P0, g, 0, &[]).unwrap();
    t.resolve_all();
    assert!(transformed(&t, g));
    assert_eq!(t.g.current(g), g);
    let f = t.g.current(forest);
    t.play_land(P0, f).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(f), Zone::Battlefield);
    // Ghost-Spider's "whenever you play a land from exile" saw it.
    assert_eq!(p1(&t, g), 1);
}

#[test]
fn innocent_traveler_opponents_choose_in_turn_order_and_sacrifice_together() {
    cr!("101.4", "701.21a");
    ruling!(
        "Innocent Traveler // Malicious Invader",
        "When Innocent Traveler's ability resolves, the next opponent in turn order chooses whether they want to sacrifice a creature at all and which creature to sacrifice if so, then each other opponent in turn order does the same, knowing the choices made before them. Then all the chosen creatures are sacrificed at the same time. If no creatures are sacrificed this way, Innocent Traveler is transformed."
    );
    supported("Innocent Traveler // Malicious Invader");
    for sacrifice in [true, false] {
        let mut t = TestGame::new(3);
        let tr = t.battlefield(P0, "Innocent Traveler // Malicious Invader");
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P2, "Grizzly Bears");
        if sacrifice {
            // P2 knows P1's choice when choosing: P1's Bears is still on the battlefield.
            let seen = crate::r_s01_common::watch(
                &mut t,
                P2,
                |d| matches!(d, Decision::YesNo { .. } | Decision::ChooseEntities { .. }),
                |g| {
                    g.permanents()
                        .filter(|o| o.chars.name == "Grizzly Bears")
                        .count()
                },
            );
            t.answer_yes(P1, true);
            t.answer_choose(P1, &[obj(a)]);
            t.answer_yes(P2, true);
            t.answer_choose(P2, &[obj(b)]);
            let from = t.asked().len();
            t.advance_to(P2, Step::End);
            t.advance_to(P0, Step::Upkeep);
            t.resolve_all();
            let askers: Vec<PlayerId> = asked_since(&t, from)
                .iter()
                .filter(|(_, d)| {
                    matches!(d, Decision::YesNo { .. } | Decision::ChooseEntities { .. })
                })
                .map(|(p, _)| *p)
                .collect();
            let first_p2 = askers.iter().position(|p| *p == P2).unwrap();
            assert!(askers[..first_p2].contains(&P1), "{askers:?}");
            assert!(seen.lock().unwrap().iter().all(|n| *n == 2));
            assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
            assert!(!transformed(&t, tr));
        } else {
            t.answer_yes(P1, false);
            t.answer_yes(P2, false);
            t.advance_to(P2, Step::End);
            t.advance_to(P0, Step::Upkeep);
            t.resolve_all();
            assert!(t.on_battlefield(a) && t.on_battlefield(b));
            assert!(transformed(&t, tr));
        }
    }
}

#[test]
fn voltaic_visionary_transforms_before_the_spell_resolves() {
    cr!("603.3", "405.5", "712.18");
    ruling!(
        "Voltaic Visionary // Volt-Charged Berserker",
        "When you play a spell exiled this way, Voltaic Visionary will transform before that spell resolves."
    );
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Voltaic Visionary // Volt-Charged Berserker");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.activate(P0, v, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    t.lands(P0, "Forest", 2);
    let spell = t.cast(P0, t.g.current(bears)).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    // The transform trigger resolved first; the Bears spell is still on the stack.
    assert!(transformed(&t, v));
    assert_eq!(t.zone(spell), Zone::Stack);
    t.resolve_all();
}

#[test]
fn garruk_keeps_his_loyalty_and_cant_activate_again_after_transforming() {
    cr!("606.3", "712.18", "306.5d");
    ruling!(
        "Garruk Relentless // Garruk, the Veil-Cursed",
        "You can't activate a loyalty ability of Garruk Relentless and later that turn after he transforms activate a loyalty ability of Garruk, the Veil-Cursed."
    );
    ruling!(
        "Garruk Relentless // Garruk, the Veil-Cursed",
        "You don't add or remove loyalty counters from Garruk Relentless when he transforms into Garruk, the Veil-Cursed. In most cases, he'll have one or two loyalty counters on him."
    );
    supported("Garruk Relentless // Garruk, the Veil-Cursed");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Garruk Relentless // Garruk, the Veil-Cursed");
    assert_eq!(t.counters(g, counters::LOYALTY), 3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, g, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    // The Bears dealt him 2: one loyalty counter left, and he transformed.
    assert!(transformed(&t, g));
    assert_eq!(t.counters(g, counters::LOYALTY), 1);
    assert!(!can_activate(&mut t, P0, g));
    // Next turn he can.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(can_activate(&mut t, P0, g));
}

#[test]
fn olag_chooses_its_copy_as_it_transforms() {
    cr!("712.20", "707.9b", "607.2a");
    ruling!(
        "Ludevic, Necrogenius // Olag, Ludevic's Hubris",
        "You choose which exiled creature card Olag is copying as it transforms. It doesn't transform and then become a copy."
    );
    supported("Ludevic, Necrogenius // Olag, Ludevic's Hubris");
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Ludevic, Necrogenius // Olag, Ludevic's Hubris");
    let giant = t.graveyard(P0, "Hill Giant");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Wastes", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[obj(giant), obj(bears)]);
    t.activate(P0, l, 0, &[]).unwrap();
    // The copy is chosen as it transforms (the ability resolving).
    t.answer_choose_moved(P0, &[obj(giant)]);
    let from = t.asked().len();
    t.resolve_all();
    assert!(was_asked(&t, from, |d| matches!(
        d,
        Decision::ChooseEntities { .. } | Decision::ChooseOption { .. }
    )));
    assert!(transformed(&t, l));
    assert_eq!(name_now(&t, l), "Olag, Ludevic's Hubris");
    // A copy of Hill Giant (a Giant), 4/4, legendary blue and black Zombie, with two
    // +1/+1 counters.
    t.g.recompute();
    let o = t.obj_now(l);
    assert!(o.chars.has_subtype("Giant") && o.chars.has_subtype("Zombie"));
    assert_eq!(p1(&t, l), 2);
    assert_eq!(pt_now(&mut t, l), (6, 6));
}

#[test]
fn hauken_s_insight_plays_cards_jacob_exiled() {
    cr!("607.2a", "712.18", "601.3");
    ruling!(
        "Jacob Hauken, Inspector // Hauken's Insight",
        "You may play cards this way that were exiled with Jacob Hauken, Inspector before it transformed into Hauken's Insight."
    );
    supported("Jacob Hauken, Inspector // Hauken's Insight");
    let mut t = TestGame::new(2);
    let j = t.battlefield(P0, "Jacob Hauken, Inspector // Hauken's Insight");
    let giant = t.hand(P0, "Hill Giant");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 4);
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_yes(P0, true);
    t.activate(P0, j, 0, &[]).unwrap();
    t.resolve_all();
    assert!(transformed(&t, j));
    let ex = t.g.current(giant);
    assert_eq!(t.zone(ex), Zone::Exile);
    // Cast without paying its mana cost (P0 has no untapped land).
    assert!(
        can_cast(&mut t, P0, ex, CastMethod::Free) || can_cast(&mut t, P0, ex, CastMethod::Normal)
    );
    let m = if can_cast(&mut t, P0, ex, CastMethod::Free) {
        CastMethod::Free
    } else {
        CastMethod::Normal
    };
    t.cast(P0, ex).method(m).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn rashmi_and_ragavan_counts_the_new_treasure_and_resolves_after_payment() {
    cr!("603.3", "601.2h", "608.2h", "111.10a");
    ruling!(
        "Rashmi and Ragavan",
        "Check the number of artifacts you control as the ability resolves, including the Treasure token you just created, to see if you can cast the exiled card without paying its mana cost."
    );
    ruling!(
        "Rashmi and Ragavan",
        "Rashmi and Ragavan’s ability resolves before the spell that caused it to trigger, but not before you have to pay for the original spell. You won’t be able to use the Treasure token you create to pay for it."
    );
    supported("Rashmi and Ragavan");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rashmi and Ragavan");
    let thopter = t.library_top(P1, "Ornithopter");
    // One Forest: not enough for Grizzly Bears, even though a Treasure would be created.
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
    assert_eq!(tokens_of(&t, P0, "Treasure"), 0);
    t.lands(P0, "Forest", 1);
    let spell = t.cast(P0, bears).go();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    // The Treasure (P0's only artifact) makes Ornithopter (mana value 0) castable for
    // free; the Bears spell is still waiting.
    assert_eq!(tokens_of(&t, P0, "Treasure"), 1);
    assert_eq!(t.zone(spell), Zone::Stack);
    let o = t.g.current(thopter);
    assert_eq!(t.zone(o), Zone::Stack, "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 1);
}

#[test]
fn blood_money_makes_treasure_only_for_creatures_it_destroyed() {
    cr!("701.8a", "701.8c", "702.12b");
    ruling!(
        "Blood Money",
        "If a creature isn't destroyed this way for some reason, such as having indestructible or being regenerated, Blood Money's controller will not create a Treasure token for it."
    );
    supported("Blood Money");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Darksteel Myr");
    let boa = t.battlefield(P1, "River Boa");
    t.lands(P1, "Forest", 1);
    t.activate(P1, boa, 0, &[]).unwrap();
    t.resolve_all();
    cast_new(&mut t, P0, "Blood Money", &[]);
    t.resolve_all();
    assert!(t.on_battlefield(boa));
    assert_eq!(t.named_on_battlefield("Darksteel Myr").len(), 1);
    assert_eq!(tokens_of(&t, P0, "Treasure"), 1);
}

#[test]
fn discerning_financier_draws_nothing_if_the_treasure_target_is_illegal() {
    cr!("608.2b");
    ruling!(
        "Discerning Financier",
        "If the target Treasure is an illegal target as Discerning Financier's last ability tries to resolve, it won't resolve. You won't draw a card."
    );
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Discerning Financier");
    let tr = create_token(&mut t, P0, "Treasure");
    t.lands(P0, "Plains", 3);
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.activate(P0, f, 0, &[obj(tr)]).unwrap();
    // In response, the Treasure is sacrificed.
    move_to(&mut t, tr, Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

/// P0's Beza entering, with the ability on the stack.
fn beza(t: &mut TestGame) {
    supported("Beza, the Bounding Spring");
    t.enter(P0, "Beza, the Bounding Spring");
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn beza_compares_as_it_resolves() {
    cr!("608.2h", "603.4");
    ruling!(
        "Beza, the Bounding Spring",
        "You compare the number of lands you control with the number of lands each opponent controls as Beza's ability resolves. At that time, if at least one opponent controls more lands than you, you'll create a Treasure token. Similarly, you compare life totals, creatures controlled, and cards in hand with each opponent as Beza's ability resolves. If at least one opponent has more life, creatures, or cards in hand than you, you'll get the appropriate bonus. It doesn't matter if you qualified for any of the previous bonuses or not."
    );
    let mut t = TestGame::new(3);
    // As Beza enters, P1 has more lands; before it resolves P1 loses one, and P2 gains
    // life above P0's.
    let l = t.lands(P1, "Forest", 1);
    beza(&mut t);
    move_to(&mut t, l[0], Zone::Hand(P1));
    t.g.players[2].life = 25;
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Treasure"), 0);
    assert_eq!(t.life(P0), 24);
}

#[test]
fn beza_treasure_entering_as_a_creature_counts_for_the_fish() {
    cr!("608.2c", "117.5", "704.3");
    ruling!(
        "Beza, the Bounding Spring",
        "No player may take any actions while Beza's ability is resolving. For example, you can't use the Treasure token to try and change the outcome of any of the other comparisons. However, in some unusual cases, that Treasure token might enter as a creature, in which case it might impact whether you get Fish or not."
    );
    for march in [false, true] {
        let mut t = TestGame::new(2);
        t.lands(P1, "Forest", 1);
        t.battlefield(P1, "Grizzly Bears");
        t.battlefield(P1, "Grizzly Bears");
        if march {
            // "Each noncreature artifact is an artifact creature with power and toughness
            // each equal to its mana value."
            t.battlefield(P0, "March of the Machines");
        }
        beza(&mut t);
        t.resolve_all();
        // P0: Beza (+ the Treasure creature) against P1's two Bears.
        assert_eq!(tokens_of(&t, P0, "Fish"), if march { 0 } else { 2 });
    }
}

#[test]
fn jeska_triples_only_the_damage_trample_assigns_to_the_player() {
    cr!("510.1c", "702.19b", "614.1a", "510.2");
    ruling!(
        "Jeska, Thrice Reborn",
        "For a creature affected by Jeska's second ability, if that creature has trample, only the damage dealt to the defending player is tripled (as long as that player is one of your opponents). For example, say you activate that ability targeting a 5/5 creature with trample, attack one of your opponents with that creature, and that creature is blocked by a 3/3 creature. You could assign 3 damage to the blocking creature and 2 damage to the defending player. If you did, your attacker would deal 3 damage to the blocking creature and 6 damage (2 damage tripled) to the defending player. All of the damage remains combat damage."
    );
    supported("Jeska, Thrice Reborn");
    let mut t = TestGame::new(2);
    let jeska = t.battlefield(P0, "Jeska, Thrice Reborn");
    t.g.add_counters(obj(jeska), counters::LOYALTY, 3, None);
    // A 5/5 trampler (Craw Wurm, 6/4, gets -1/+1) blocked by a 3/3.
    let wurm = t.battlefield(P0, "Craw Wurm");
    crate::r_p214_common::pump(&mut t, wurm, -1, 1);
    grant_kw(&mut t, wurm, mtg_engine::keywords::KeywordKind::Trample);
    let giant = t.battlefield(P1, "Hill Giant");
    t.activate(P0, jeska, 0, &[obj(wurm)]).unwrap();
    t.resolve_all();
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![3, 2]));
    attack_with(&mut t, &[(wurm, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(giant, wurm)]);
    assert_eq!(t.life(P1), 14);
    assert!(!t.on_battlefield(giant));
    let combat: u32 =
        t.g.turn_events
            .iter()
            .filter_map(|e| match e {
                mtg_engine::events::Event::Damage {
                    target: Entity::Object(o),
                    amount,
                    combat: true,
                    ..
                } if *o == giant => Some(*amount),
                _ => None,
            })
            .sum();
    assert_eq!(combat, 3);
    assert_eq!(dealt(&t, Entity::Player(P1)), 6);
}
