//! Rulings batch P069 — which exiled cards a permission covers: linked abilities refer only
//! to the cards their own object exiled (CR 607.1, 607.2a, 406.6), a new object has no
//! memory of the old one's cards (CR 400.7); a cast card leaves exile once (CR 601.2a); and
//! whether the exiled cards are face up (CR 406.3).

use crate::r_p067_common::{activate_x, move_to, was_offered};
use crate::r_p069_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, stack_library, supported, watch};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::facedown;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0's creature `attacker` deals combat damage to P1 (unblocked); its triggers resolve.
fn connect(t: &mut TestGame, attacker: ObjectId) {
    attack_with(t, &[(attacker, Entity::Player(P1))]);
    block_and_finish(t, P1, &[]);
    t.resolve_all();
}

/// The candidates of every target choice asked of `p` since `from`.
fn target_candidates_since(t: &TestGame, p: PlayerId, from: usize) -> Vec<Entity> {
    t.asked()[from..]
        .iter()
        .filter(|(q, _)| *q == p)
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .flatten()
        .collect()
}

#[test]
fn kaho_casts_only_cards_its_triggered_ability_exiled() {
    cr!("607.2a", "607.1");
    ruling!(
        "Kaho, Minamo Historian",
        "Kaho's activated ability refers only to cards exiled by its triggered ability. If another effect exiled the cards, you won't be able to use Kaho to cast those cards."
    );
    supported("Kaho, Minamo Historian");
    let mut t = TestGame::new(2);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let shock = t.exile(P0, "Shock");
    t.answer_choose_moved(P0, &[o(bolt)]);
    let k = t.enter(P0, "Kaho, Minamo Historian");
    t.resolve_all();
    assert!(exiled(&t, bolt), "{}", t.dump_log());
    t.g.obj_mut(k).summoning_sick = false;
    t.clear_answers();
    let from = t.asked().len();
    mana(&mut t, P0, ManaType::C, 1);
    t.answer_choose_moved(P0, &[o(bolt)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_x(&mut t, P0, k, 0, 1, &[]);
    t.resolve_all();
    assert!(was_offered(&t, from, bolt), "{}", t.dump_log());
    assert!(!was_offered(&t, from, shock));
    assert_eq!(t.life(P1), 17);
    assert!(exiled(&t, shock));
}

#[test]
fn nightveil_specter_cards_belong_to_that_specific_specter() {
    cr!("607.2a");
    ruling!(
        "Nightveil Specter",
        "Nightveil Specter’s last ability applies to cards exiled with that specific Nightveil Specter, not any other creature named Nightveil Specter. You should keep cards exiled by different Nightveil Specters separate."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Nightveil Specter");
    t.battlefield(P0, "Nightveil Specter");
    let bears = t.library_top(P1, "Grizzly Bears");
    connect(&mut t, a);
    assert!(exiled(&t, bears));
    t.advance_to(P0, Step::PostcombatMain);
    mana(&mut t, P0, ManaType::G, 2);
    assert!(castable(&mut t, P0, bears));
    // The Specter that exiled it leaves: the other Nightveil Specter doesn't let P0 cast it.
    t.g.destroy(a, None);
    t.settle();
    assert_eq!(t.named_on_battlefield("Nightveil Specter").len(), 1);
    assert!(!castable(&mut t, P0, bears));
}

#[test]
fn muse_vessel_plays_only_cards_its_first_ability_exiled() {
    cr!("607.2a");
    ruling!(
        "Muse Vessel",
        "Only cards exiled as a result of the first ability can be played as a result of the second ability."
    );
    supported("Muse Vessel");
    let mut t = TestGame::new(2);
    let mv = t.battlefield(P0, "Muse Vessel");
    let bears = t.hand(P1, "Grizzly Bears");
    let giant = t.exile(P1, "Hill Giant");
    mana(&mut t, P0, ManaType::C, 3);
    t.activate(P0, mv, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(exiled(&t, bears));
    let from = t.asked().len();
    mana(&mut t, P0, ManaType::C, 1);
    t.answer_choose_moved(P0, &[o(bears)]);
    t.activate(P0, mv, 1, &[]).unwrap();
    t.resolve_all();
    assert!(was_offered(&t, from, bears), "{}", t.dump_log());
    assert!(!was_offered(&t, from, giant));
    mana(&mut t, P0, ManaType::G, 2);
    assert!(castable(&mut t, P0, bears));
    mana(&mut t, P0, ManaType::R, 4);
    assert!(!castable(&mut t, P0, giant));
}

#[test]
fn quintorius_targets_only_cards_its_trigger_exiled_not_a_sacrificed_spirit() {
    cr!("607.2a", "614.1a", "602.2b", "601.2h");
    ruling!(
        "Quintorius, Loremaster",
        "Quintorius's last two abilities are linked. The last ability can target only cards exiled by the triggered ability. If a Spirit you sacrifice to activate the last ability ends up being exiled because of a replacement effect, that card can't be targeted by the last ability."
    );
    supported("Quintorius, Loremaster");
    let mut t = TestGame::new(2);
    let q = t.battlefield(P0, "Quintorius, Loremaster");
    let div = t.graveyard(P0, "Divination");
    let shock = t.exile(P0, "Shock");
    t.battlefield(P0, "Rest in Peace");
    let sailors = [
        t.battlefield(P0, "Spectral Sailor"),
        t.battlefield(P0, "Spectral Sailor"),
    ];
    t.answer_targets(P0, &[o(div)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(exiled(&t, div), "{}", t.dump_log());
    // Twice, sacrificing a Spectral Sailor (a Spirit card) each time; Rest in Peace exiles
    // it instead.
    for s in sailors {
        let from = t.asked().len();
        t.g.obj_mut(q).tapped = false;
        mana(&mut t, P0, ManaType::R, 1);
        mana(&mut t, P0, ManaType::W, 1);
        mana(&mut t, P0, ManaType::C, 1);
        t.answer_choose(P0, &[o(s)]);
        t.activate(P0, q, 0, &[o(t.g.current(div))]).unwrap();
        assert!(exiled(&t, s), "{}", t.dump_log());
        let offered = target_candidates_since(&t, P0, from);
        assert!(offered.contains(&o(t.g.current(div))), "{offered:?}");
        assert!(!offered.contains(&o(shock)));
        for x in sailors {
            assert!(!offered.contains(&o(t.g.current(x))), "{offered:?}");
        }
        t.resolve_all();
    }
}

#[test]
fn izzet_chemister_casts_only_its_own_cards_and_they_are_lost_if_it_leaves() {
    cr!("607.2a");
    ruling!(
        "Izzet Chemister",
        "The last ability of Izzet Chemister only lets you cast cards exiled by that card. If Izzet Chemister leaves the battlefield before you activate that ability, the exiled cards are lost forever."
    );
    supported("Izzet Chemister");
    let setup = || {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, "Izzet Chemister");
        let bolt = t.graveyard(P0, "Lightning Bolt");
        mana(&mut t, P0, ManaType::R, 1);
        t.activate(P0, c, 0, &[o(bolt)]).unwrap();
        t.resolve_all();
        assert!(exiled(&t, bolt));
        t.g.obj_mut(c).tapped = false;
        (t, c, bolt)
    };
    let (mut t, c, bolt) = setup();
    let shock = t.exile(P0, "Shock");
    let from = t.asked().len();
    mana(&mut t, P0, ManaType::R, 2);
    t.answer_choose_moved(P0, &[o(bolt)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, c, 1, &[]).unwrap();
    t.resolve_all();
    assert!(was_offered(&t, from, bolt), "{}", t.dump_log());
    assert!(!was_offered(&t, from, shock));
    assert_eq!(t.life(P1), 17);
    // It leaves before the last ability is activated: another Izzet Chemister can't cast
    // the Bolt, which stays exiled.
    let (mut t, c, bolt) = setup();
    t.g.destroy(c, None);
    t.settle();
    let c2 = t.battlefield(P0, "Izzet Chemister");
    let from = t.asked().len();
    mana(&mut t, P0, ManaType::R, 2);
    t.activate(P0, c2, 1, &[]).unwrap();
    t.resolve_all();
    assert!(!was_offered(&t, from, bolt));
    assert!(exiled(&t, bolt));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn kheru_mind_eater_casts_only_its_own_cards_and_they_are_lost_if_it_leaves() {
    cr!("607.2a");
    ruling!(
        "Kheru Mind-Eater",
        "The last ability of Kheru Mind-Eater only lets you cast cards exiled by that card. If Kheru Mind-Eater leaves the battlefield, the exiled cards are lost forever."
    );
    supported("Kheru Mind-Eater");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kheru Mind-Eater");
    let bears = t.hand(P1, "Grizzly Bears");
    let giant = t.exile(P1, "Hill Giant");
    connect(&mut t, k);
    assert!(exiled(&t, bears), "{}", t.dump_log());
    assert!(t.obj_now(bears).face_down);
    t.advance_to(P0, Step::PostcombatMain);
    mana(&mut t, P0, ManaType::G, 2);
    assert!(castable(&mut t, P0, bears));
    mana(&mut t, P0, ManaType::R, 4);
    assert!(!castable(&mut t, P0, giant));
    t.g.destroy(k, None);
    t.settle();
    t.battlefield(P0, "Kheru Mind-Eater");
    assert!(!castable(&mut t, P0, bears));
    assert!(exiled(&t, bears));
}

#[test]
fn share_the_spoils_cards_cant_be_played_after_it_leaves_and_a_new_one_has_no_memory() {
    cr!("400.7", "607.2a");
    ruling!(
        "Share the Spoils",
        "Once Share the Spoils leaves the battlefield, players may no longer play the exiled cards. If Share the Spoils is destroyed and somehow returns to the battlefield, it's a new object with no memory of the previously exiled cards."
    );
    let mut t = TestGame::new(2);
    let mine = t.library_top(P0, "Grizzly Bears");
    let theirs = t.library_top(P1, "Hill Giant");
    let s = t.enter(P0, "Share the Spoils");
    t.resolve_all();
    assert!(exiled(&t, mine) && exiled(&t, theirs), "{}", t.dump_log());
    // Mana as though it were mana of any color.
    mana(&mut t, P0, ManaType::R, 4);
    assert!(castable(&mut t, P0, mine) && castable(&mut t, P0, theirs));
    t.g.destroy(s, None);
    t.settle();
    assert!(!castable(&mut t, P0, mine) && !castable(&mut t, P0, theirs));
    // It returns: it exiles the new top cards, and only those can be played.
    let new_mine = t.library_top(P0, "Serra Angel");
    t.library_top(P1, "Forest");
    let back = t.g.current(s);
    t.g.move_object(
        back,
        Zone::Battlefield,
        mtg_engine::events::MoveCause::Effect,
        None,
    )
    .unwrap();
    t.resolve_all();
    assert!(exiled(&t, new_mine), "{}", t.dump_log());
    mana(&mut t, P0, ManaType::R, 5);
    assert!(castable(&mut t, P0, new_mine));
    assert!(!castable(&mut t, P0, mine) && !castable(&mut t, P0, theirs));
}

#[test]
fn idol_of_endurance_cards_return_when_it_leaves_but_cast_ones_stay() {
    cr!("610.3", "400.7");
    ruling!(
        "Idol of Endurance",
        "The exiled cards return to your graveyard immediately after Idol of Endurance leaves the battlefield. Any cards that had left exile before then (most likely because they were cast with Idol of Endurance's second ability) remain in their current zone."
    );
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let viper = t.graveyard(P0, "Ambush Viper");
    let idol = t.enter(P0, "Idol of Endurance");
    t.resolve_all();
    mana(&mut t, P0, ManaType::W, 2);
    t.activate(P0, idol, 0, &[]).unwrap();
    t.resolve_all();
    cast_free(&mut t, P0, bears, &[]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(exiled(&t, viper));
    t.g.destroy(idol, None);
    t.settle();
    assert_eq!(t.zone(viper), Zone::Graveyard(P0));
    assert!(t.on_battlefield(bears));
}

#[test]
fn gonti_canny_acquisitor_card_can_be_played_only_once() {
    cr!("601.2a", "400.7");
    ruling!(
        "Gonti, Canny Acquisitor",
        "Playing an exiled card causes it to leave exile. You can’t play it multiple times."
    );
    supported("Gonti, Canny Acquisitor");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Gonti, Canny Acquisitor");
    let bolt = t.library_top(P1, "Lightning Bolt");
    connect(&mut t, g);
    assert!(exiled(&t, bolt), "{}", t.dump_log());
    // Mana of any type; Gonti makes it cost {1} less, which doesn't affect {R}.
    mana(&mut t, P0, ManaType::B, 1);
    cast_now(&mut t, P0, bolt, &[Entity::Player(P1)]);
    assert_eq!(t.zone(bolt), Zone::Stack);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 5 - 3);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    mana(&mut t, P0, ManaType::B, 1);
    assert!(!castable(&mut t, P0, bolt));
    // Put back into exile, it's a new object the permission doesn't cover.
    move_to(&mut t, bolt, Zone::Exile);
    assert!(!castable(&mut t, P0, bolt));
}

/// P0 casts Psychic Theft on P1, exiling P1's Lightning Bolt.
fn psychic_theft(t: &mut TestGame) -> ObjectId {
    supported("Psychic Theft");
    let bolt = t.hand(P1, "Lightning Bolt");
    let pt = t.hand(P0, "Psychic Theft");
    mana(t, P0, ManaType::U, 2);
    t.answer_choose_moved(P0, &[o(bolt)]);
    t.cast(P0, pt).target(P1).go();
    t.resolve_all();
    assert!(exiled(t, bolt), "{}", t.dump_log());
    bolt
}

#[test]
fn psychic_theft_card_is_cast_once_and_the_effect_ends() {
    cr!("601.2a", "603.7", "400.7");
    ruling!(
        "Psychic Theft",
        "The card may only be cast once. Once it is announced and put on the stack, the card is no longer exiled and the effect of Psychic Theft ends."
    );
    let mut t = TestGame::new(2);
    let bolt = psychic_theft(&mut t);
    mana(&mut t, P0, ManaType::R, 1);
    cast_now(&mut t, P0, bolt, &[Entity::Player(P1)]);
    assert_eq!(t.zone(bolt), Zone::Stack);
    assert!(!t.in_exile("Lightning Bolt"));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    mana(&mut t, P0, ManaType::R, 1);
    assert!(!castable(&mut t, P0, bolt));
    // The end step trigger doesn't return it to P1's hand.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert!(!t.in_hand(P1, "Lightning Bolt"));
    // Put back into exile, it's a new object the permission doesn't cover.
    move_to(&mut t, bolt, Zone::Exile);
    mana(&mut t, P0, ManaType::R, 1);
    assert!(!castable(&mut t, P0, bolt));
}

#[test]
fn psychic_theft_card_is_revealed_while_exiled() {
    cr!("406.3");
    ruling!(
        "Psychic Theft",
        "The card is revealed to all players while it is exiled."
    );
    let mut t = TestGame::new(2);
    let bolt = psychic_theft(&mut t);
    let b = t.g.current(bolt);
    assert!(!t.obj_now(bolt).face_down);
    for p in [P0, P1] {
        assert!(facedown::can_look_at(&t.g, p, b));
    }
}

#[test]
fn hostage_taker_spell_stays_yours_if_it_leaves() {
    cr!("400.7", "601.2a", "608.3");
    ruling!(
        "Hostage Taker",
        "Once you begin to cast the exiled card, it's considered a new object. You'll control that spell and the permanent that spell becomes even if Hostage Taker leaves the battlefield."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[o(bears)]);
    let ht = t.enter(P0, "Hostage Taker");
    t.resolve_all();
    assert!(exiled(&t, bears));
    mana(&mut t, P0, ManaType::U, 2);
    cast_now(&mut t, P0, bears, &[]);
    t.g.destroy(ht, None);
    t.settle();
    assert_eq!(t.zone(bears), Zone::Stack);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P0);
}

#[test]
fn kheru_mind_eater_spell_is_unaffected_by_losing_control_of_it() {
    cr!("400.7", "601.2a");
    ruling!(
        "Kheru Mind-Eater",
        "Once you begin to cast an exiled card, losing control of Kheru Mind-Eater won’t affect the spell."
    );
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kheru Mind-Eater");
    let bears = t.hand(P1, "Grizzly Bears");
    connect(&mut t, k);
    t.advance_to(P0, Step::PostcombatMain);
    mana(&mut t, P0, ManaType::G, 2);
    cast_now(&mut t, P0, bears, &[]);
    // P1 takes Kheru Mind-Eater with Ray of Command in response.
    let ray = t.hand(P1, "Ray of Command");
    mana(&mut t, P1, ManaType::U, 4);
    t.g.turn.priority = Some(P1);
    t.cast(P1, ray).target(k).go();
    t.resolve();
    assert_eq!(t.obj_now(k).controller, P1);
    t.resolve_all();
    assert!(t.on_battlefield(bears), "{}", t.dump_log());
    assert_eq!(t.obj_now(bears).controller, P0);
}

#[test]
fn robber_grenzo_and_nightveil_exile_face_up() {
    cr!("406.3");
    ruling!(
        "Robber of the Rich",
        "The card Robber of the Rich exiles is exiled face up."
    );
    ruling!(
        "Grenzo, Havoc Raiser",
        "The card exiled by the second mode is exiled face up."
    );
    ruling!(
        "Nightveil Specter",
        "The card is exiled face up. All players may look at it."
    );
    supported("Grenzo, Havoc Raiser");
    for name in ["Robber of the Rich", "Grenzo, Havoc Raiser", "Nightveil Specter"] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        t.hand(P1, "Island");
        let bears = t.library_top(P1, "Grizzly Bears");
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
        connect(&mut t, c);
        assert!(exiled(&t, bears), "{name}: {}", t.dump_log());
        assert!(!t.obj_now(bears).face_down, "{name}");
        let b = t.g.current(bears);
        for p in [P0, P1] {
            assert!(facedown::can_look_at(&t.g, p, b), "{name}");
        }
    }
}

#[test]
fn grima_exiles_the_cards_face_up() {
    cr!("406.3");
    ruling!(
        "Gríma, Saruman's Footman",
        "The cards are exiled face up. All players will be able to see them."
    );
    supported("Gríma, Saruman's Footman");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Gríma, Saruman's Footman");
    let cards = stack_library(&mut t, P1, &["Hill Giant", "Forest", "Divination", "Island"]);
    // While P0 decides whether to cast Divination, the exiled cards are face up.
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::YesNo { .. } | Decision::ChooseEntities { .. }),
        |g| {
            g.exile
                .iter()
                .map(|id| (g.obj(*id).chars.name.to_string(), g.obj(*id).face_down))
                .collect::<Vec<_>>()
        },
    );
    connect(&mut t, g);
    let seen = seen.lock().unwrap();
    let first = seen.first().expect("asked whether to cast Divination");
    assert_eq!(first.len(), 3, "{first:?}");
    assert!(first.iter().all(|(_, down)| !down), "{first:?}");
    drop(seen);
    assert_eq!(t.zone(cards[3]), Zone::Library(P1));
}

#[test]
fn expensive_taste_and_siphon_insight_only_you_look_at_the_cards() {
    cr!("406.3", "406.3a");
    ruling!(
        "Decadent Dragon // Expensive Taste",
        "Only you get to look at the cards exiled with Expensive Taste. Their owner doesn't get to know what they are while they're in exile until you play them."
    );
    ruling!(
        "Siphon Insight",
        "Only you get to look at the exiled card. Its owner doesn't get to know what it is while it is in exile unless you tell them."
    );
    supported("Decadent Dragon // Expensive Taste");
    supported("Siphon Insight");
    // Expensive Taste: the top two cards of P1's library.
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P1, &["Grizzly Bears", "Lightning Bolt"]);
    let dd = t.hand(P0, "Decadent Dragon // Expensive Taste");
    mana(&mut t, P0, ManaType::B, 3);
    t.cast(P0, dd).method(CastMethod::Half(1)).target(P1).go();
    t.resolve_all();
    let mut exiled_ids = vec![];
    for c in &cards {
        assert!(exiled(&t, *c), "{}", t.dump_log());
        assert!(t.obj_now(*c).face_down);
        exiled_ids.push(t.g.current(*c));
    }
    // Siphon Insight: one of the top two.
    let mut t2 = TestGame::new(2);
    let top = stack_library(&mut t2, P1, &["Serra Angel", "Hill Giant"]);
    let si = t2.hand(P0, "Siphon Insight");
    mana(&mut t2, P0, ManaType::U, 1);
    mana(&mut t2, P0, ManaType::B, 1);
    t2.answer_choose_moved(P0, &[o(top[0])]);
    t2.cast(P0, si).target(P1).go();
    t2.resolve_all();
    assert!(exiled(&t2, top[0]), "{}", t2.dump_log());
    assert!(t2.obj_now(top[0]).face_down);
    for (g, id) in exiled_ids
        .iter()
        .map(|id| (&t.g, *id))
        .chain([(&t2.g, t2.g.current(top[0]))])
    {
        assert!(facedown::can_look_at(g, P0, id));
        assert!(!facedown::can_look_at(g, P1, id));
    }
    // Once P0 plays it, it's a spell everyone sees.
    mana(&mut t, P0, ManaType::R, 1);
    let s = cast_now(&mut t, P0, cards[1], &[Entity::Player(P1)]);
    assert!(!t.obj(s).face_down);
    assert!(facedown::can_look_at(&t.g, P1, s));
}
