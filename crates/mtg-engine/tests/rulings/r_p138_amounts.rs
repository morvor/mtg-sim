//! Rulings batch P138 — what edicts and sweepers count and when: last known information
//! of the sacrificed creature (CR 608.2h), amounts determined on resolution
//! (CR 608.2h), choices made on resolution (CR 608.2d), copies (CR 707.10), and what the
//! sacrificed cards are afterwards (CR 400.7, 614.1).

use crate::r_p138_common::*;
use crate::r_s13_common::{commander, commander_game};
use crate::r_s17_common::put_onto_battlefield;
use mtg_engine::decision::Decision;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn consume(t: &mut TestGame) {
    lots(t, P0);
    let c = t.hand(P0, "Consecrate // Consume");
    t.answer_targets(P0, &[pl(P1)]);
    t.cast(P0, c).method(CastMethod::Half(1)).go();
    t.resolve_all();
}

#[test]
fn life_gained_uses_the_sacrificed_creatures_last_known_power_and_toughness() {
    cr!("608.2h");
    ruling!(
        "Devour Flesh",
        "The amount of life gained is equal to the creature’s toughness as it last existed on the battlefield."
    );
    ruling!(
        "Consecrate // Consume",
        "The amount of life gained while Consume resolves is equal to the power of the creature as it last existed on the battlefield."
    );
    supported("Devour Flesh");
    supported("Consecrate // Consume");
    // Devour Flesh: Grizzly Bears with P1's Glorious Anthem is 3/3: P1 gains 3.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Glorious Anthem");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast(&mut t, P0, "Devour Flesh", &[&[pl(P1)]]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 23);
    // Consume: Hill Giant with P1's Glorious Anthem is 4/4: P0 gains 4.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Glorious Anthem");
    let giant = t.battlefield(P1, "Hill Giant");
    consume(&mut t);
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.life(P0), 24);
}

#[test]
fn consume_with_negative_power_changes_no_life() {
    cr!("107.1b", "119.1");
    ruling!(
        "Consecrate // Consume",
        "If the creature's power is negative, you don't lose or gain life."
    );
    // Ornithopter with Weakness is -2/1.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    let weakness = t.battlefield(P1, "Weakness");
    assert!(t.g.attach(weakness, obj(thopter)));
    t.g.recompute();
    assert_eq!(t.pt(thopter), (-2, 1));
    consume(&mut t);
    assert!(!t.on_battlefield(thopter));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn skull_storm_copies_each_determine_the_life_loss_on_resolution() {
    cr!("608.2h", "707.10");
    ruling!(
        "Skull Storm",
        "The amount of life to lose is determined only as Skull Storm resolves. For example, if an opponent controls no creatures and has 30 life when you cast Skull Storm and make two copies, the first to resolve causes that opponent to lose 15 life (so they have 15 life left), the second causes that opponent to lose 8 life (7 life left), and the original spell causes that player to lose 4 life, leaving the player at 3 life."
    );
    let mut t = commander_game();
    let cmdr = commander(&mut t, P0, "Grizzly Bears");
    let key = mtg_engine::kw::partner::commander_key(&t.g, cmdr);
    t.g.players[0].commander_casts.insert(key, 2);
    t.g.players[1].life = 30;
    cast(&mut t, P0, "Skull Storm", &[]);
    t.settle();
    t.resolve();
    assert_eq!(t.stack_len(), 3, "the spell and two copies");
    let mut lives = vec![];
    while t.stack_len() > 0 {
        t.resolve();
        lives.push(t.life(P1));
    }
    assert_eq!(lives, vec![15, 7, 3]);
}

#[test]
fn szats_will_both_modes_exile_sacrificed_cards_but_not_tokens() {
    cr!("700.2", "608.2c", "111.7");
    ruling!(
        "Szat's Will",
        "If both modes are chosen, and an opponent sacrifices a creature token to the first mode, that token will be in the graveyard as the instructions in the second mode are followed. As it's not a creature card, it won't be exiled, and it won't be considered for determining the value of X. The token will cease to exist after Szat's Will resolves."
    );
    ruling!(
        "Szat's Will",
        "If both modes are chosen, they happen in the stated order. Any creature cards that end up in an opponent's graveyard because they were sacrificed for the first mode will be exiled as the instructions for the second mode are followed."
    );
    for token in [true, false] {
        let mut t = commander_game();
        // P0 controls its commander (Grizzly Bears) on the battlefield.
        let cmdr = commander(&mut t, P0, "Grizzly Bears");
        let cmdr = move_to(&mut t, cmdr, Zone::Battlefield);
        assert!(t.on_battlefield(cmdr));
        t.graveyard(P1, "Grizzly Bears");
        let sacrificed = if token {
            let alarm = t.hand(P1, "Raise the Alarm");
            lots(&mut t, P1);
            t.cast(P1, alarm).go();
            t.resolve_all();
            None
        } else {
            Some(t.battlefield(P1, "Hill Giant"))
        };
        lots(&mut t, P0);
        lots(&mut t, P0);
        let s = t.hand(P0, "Szat's Will");
        t.cast(P0, s).modes(&[0, 1]).go();
        t.resolve_all();
        let thrulls = crate::r_s01_common::tokens(&t, P0).len();
        if let Some(giant) = sacrificed {
            // The Hill Giant (power 3) was sacrificed, then exiled with the Bears.
            assert!(t.in_exile("Hill Giant"));
            assert_eq!(t.zone(giant), Zone::Exile);
            assert_eq!(thrulls, 3);
        } else {
            // The Soldier token didn't count: X is the Bears' 2.
            assert_eq!(thrulls, 2);
            assert_eq!(crate::r_s01_common::tokens(&t, P1).len(), 1);
        }
        assert!(t.in_exile("Grizzly Bears"));
        assert_eq!(t.graveyard_size(P1), 0);
    }
}

#[test]
fn dark_intimations_each_copy_adds_a_loyalty_counter() {
    cr!("603.2", "122.6");
    ruling!(
        "Dark Intimations",
        "If two or more Dark Intimations are in your graveyard when you cast a Bolas planeswalker spell, you'll exile each of them and that planeswalker will enter the battlefield with that many additional loyalty counters."
    );
    supported("Nicol Bolas, Planeswalker");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Dark Intimations");
    t.graveyard(P0, "Dark Intimations");
    cast(&mut t, P0, "Nicol Bolas, Planeswalker", &[]);
    t.resolve_all();
    let bolas = t.named_on_battlefield("Nicol Bolas, Planeswalker");
    assert_eq!(bolas.len(), 1);
    assert_eq!(t.counters(bolas[0], counters::LOYALTY), 7);
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn ob_nixilis_opponent_searching_their_own_library() {
    cr!("701.23a", "603.2");
    ruling!(
        "Ob Nixilis, Unshackled",
        "If the opponent controls no creatures when the first triggered ability resolves, that player still loses 10 life."
    );
    ruling!(
        "Ob Nixilis, Unshackled",
        "Ob Nixilis's first triggered ability won't trigger if you search an opponent's library or if an opponent searches another player's library."
    );
    supported("Ob Nixilis, Unshackled");
    // P1 searches their own library (no creatures): loses 10.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ob Nixilis, Unshackled");
    t.library_top(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    cast(&mut t, P1, "Diabolic Tutor", &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 10);
    // P0 searches P1's library; P1 searches P0's library: no trigger.
    for caster in [P0, P1] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Ob Nixilis, Unshackled");
        let other = if caster == P0 { P1 } else { P0 };
        t.library_top(other, "Grizzly Bears");
        t.set_step(caster, Step::PrecombatMain);
        cast(&mut t, caster, "Bribery", &[&[pl(other)]]);
        t.resolve_all();
        assert_eq!(t.life(P1), 20, "{caster:?} cast Bribery");
    }
}

#[test]
fn killing_wave_a_player_who_cant_sacrifice_may_decline_to_pay() {
    cr!("118.12", "701.21a");
    ruling!(
        "Killing Wave",
        "If you can’t sacrifice a creature (perhaps because of Sigarda, Host of Herons), you can choose not to pay life and nothing will happen."
    );
    supported("Killing Wave");
    let mut t = TestGame::new(2);
    let sigarda = t.battlefield(P1, "Sigarda, Host of Herons");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P1, false);
    t.answer_yes(P1, false);
    lots(&mut t, P0);
    let wave = t.hand(P0, "Killing Wave");
    t.cast(P0, wave).x(2).go();
    t.resolve_all();
    assert!(t.on_battlefield(sigarda) && t.on_battlefield(bears));
    assert_eq!(t.life(P1), 20);
}

/// The same for a sacrifice of the player's choice planned with the other players'
/// choices ("for each opponent, ... unless they pay 2 life"): with Sigarda, P1 has nothing
/// it can be made to sacrifice.
#[test]
fn a_planned_sacrifice_of_a_players_choice_respects_sigarda() {
    cr!("701.21a", "101.4");
    let def = crate::r_s01_common::custom_card(
        "Probe Edict",
        "Sorcery",
        "{B}",
        None,
        "For each opponent, that player sacrifices a creature of their choice unless they pay 2 life.",
    );
    for sigarda in [false, true] {
        let mut t = TestGame::new(3);
        let angel = sigarda.then(|| t.battlefield(P1, "Sigarda, Host of Herons"));
        let b1 = t.battlefield(P1, "Grizzly Bears");
        let b2 = t.battlefield(P2, "Grizzly Bears");
        t.answer_yes(P1, false);
        t.answer_yes(P2, false);
        let c = t.custom(P0, def.clone(), Zone::Hand(P0));
        lots(&mut t, P0);
        t.cast(P0, c).go();
        t.resolve_all();
        assert_eq!(t.on_battlefield(b1), sigarda, "Sigarda {sigarda}");
        assert!(angel.is_none_or(|a| t.on_battlefield(a)));
        assert!(!t.on_battlefield(b2), "Sigarda {sigarda}");
    }
}

#[test]
fn living_death_returns_only_the_cards_its_first_instruction_exiled() {
    cr!("608.2c", "614.1a", "400.7");
    ruling!(
        "Living Death",
        "Only cards exiled by Living Death's first instruction are put onto the battlefield. If a replacement effect (such as that of Leyline of the Void) causes any of the sacrificed creatures to be exiled instead of put into a graveyard, those cards aren't returned to the battlefield."
    );
    supported("Living Death");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of the Void");
    let giant = t.graveyard(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast(&mut t, P0, "Living Death", &[]);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(creatures(&t, P1).is_empty());
}

#[test]
fn sorin_pumps_only_creatures_controlled_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Sorin, Solemn Visitor",
        "Only creatures you control as the first ability resolves will get the bonuses. Creatures you come to control after that happens will not."
    );
    supported("Sorin, Solemn Visitor");
    let mut t = TestGame::new(2);
    let sorin = t.battlefield(P0, "Sorin, Solemn Visitor");
    let early = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, sorin, 0, &[]).expect("+1");
    t.resolve_all();
    let late = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(early), (3, 2));
    assert_eq!(t.pt(late), (2, 2));
}

#[test]
fn choices_are_made_on_resolution_not_on_casting() {
    cr!("608.2d", "601.2c");
    ruling!(
        "Urborg Justice",
        "The creatures are chosen on resolution, not on announcement."
    );
    ruling!(
        "Gix's Command",
        "Since none of the modes have targets, you don't have to choose which creature is getting the counters or which cards you're returning until Gix's Command resolves."
    );
    supported("Urborg Justice");
    supported("Gix's Command");
    // Urborg Justice: a creature of P0's died this turn.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    kill(&mut t, mine);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let from = t.asked().len();
    cast(&mut t, P0, "Urborg Justice", &[&[pl(P1)]]);
    assert!(choosers_since(&t, from).is_empty());
    t.answer_choose(P1, &[obj(b)]);
    t.resolve_all();
    assert_eq!(choosers_since(&t, from), vec![P1]);
    assert!(t.on_battlefield(a) && !t.on_battlefield(b));

    // Gix's Command: counters on up to one creature, and up to two creature cards back.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let dead = t.graveyard(P0, "Hill Giant");
    lots(&mut t, P0);
    lots(&mut t, P0);
    let c = t.hand(P0, "Gix's Command");
    let from = t.asked().len();
    t.cast(P0, c).modes(&[0, 2]).go();
    let at_cast = asked_since(&t, from);
    assert!(at_cast.iter().all(|(_, d)| !matches!(
        d,
        Decision::ChooseTargets { .. } | Decision::ChooseEntities { .. }
    )));
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_choose(P0, &[obj(dead)]);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    assert_eq!(t.zone(dead), Zone::Hand(P0));
}

#[test]
fn malfegor_discards_the_whole_hand_even_with_nothing_to_sacrifice() {
    cr!("701.9a", "608.2c");
    ruling!(
        "Malfegor",
        "Malfegor’s ability is not optional. You must discard your entire hand no matter how many creatures your opponents control."
    );
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.hand(P0, "Grizzly Bears");
    }
    t.enter(P0, "Malfegor");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.graveyard_size(P0), 3);
}

#[test]
fn ashen_reaper_counts_tokens_put_into_a_graveyard() {
    cr!("111.7", "700.4");
    ruling!(
        "Invasion of Azgol // Ashen Reaper",
        "The ability will count any permanent that was put into a graveyard, including token permanents."
    );
    let name = "Invasion of Azgol // Ashen Reaper";
    supported(name);
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, name);
    let reaper = put_onto_battlefield(&mut t, P0, card, true).expect("entered");
    assert_eq!(t.obj(reaper).chars.name, "Ashen Reaper");
    let alarm = t.hand(P1, "Raise the Alarm");
    lots(&mut t, P1);
    t.g.turn.priority = Some(P1);
    t.g.cast_spell(P1, alarm, CastMethod::Normal).expect("cast");
    t.resolve_all();
    let soldier = crate::r_s01_common::tokens(&t, P1)[0];
    kill(&mut t, soldier);
    to_end_step(&mut t);
    t.resolve_all();
    assert_eq!(t.counters(reaper, counters::PLUS1), 1);
}

#[test]
fn kefka_counts_card_types_not_supertypes_or_subtypes() {
    cr!("205.2a", "205.4a", "300.1");
    ruling!(
        "Kefka, Court Mage // Kefka, Ruler of Ruin",
        "The card types in Magic include artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Legendary, basic, and snow are supertypes, not card types; Hero and Saga are subtypes, not card types."
    );
    let name = "Kefka, Court Mage // Kefka, Ruler of Ruin";
    supported(name);
    // P0 discards a basic snow land (one type: land); P1 a kindred instant (two).
    let mut t = TestGame::new(2);
    t.hand(P0, "Snow-Covered Forest");
    t.hand(P1, "Crib Swap");
    t.enter(P0, name);
    t.settle();
    let before = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before - 1 + 3);
}
