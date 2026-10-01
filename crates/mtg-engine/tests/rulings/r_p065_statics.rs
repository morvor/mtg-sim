//! Rulings batch P065 — static abilities and continuous effects: characteristic-defining
//! abilities in every zone (CR 604.3), abilities that matter only as damage is dealt or
//! as state-based actions are checked, layer-7 interactions (613.4), replacement effects
//! on life gain and token creation (614), the legend rule (704.5j), and regeneration
//! (701.19).

use crate::r_s01_common::{supported, tokens};
use crate::r_s02_common::{can_activate, can_play_land, create_token, destroy};
use crate::r_s06_common::{activate_containing, damage, has_kw};
use crate::r_s09_common::{declare, to_combat};
use crate::r_s10_common::attacking;
use crate::r_s13_common::{commander, commander_game};
use crate::r_s26_common::modify_until_eot;
use crate::r_s29_common::{cast_and_resolve, put_counters};
use mtg_engine::ability::{Modification, Value};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Characteristic-defining abilities in every zone
// ---------------------------------------------------------------------------------------

#[test]
fn aeon_chronicler_pt_applies_in_the_hand() {
    cr!("604.3", "604.3a");
    ruling!(
        "Aeon Chronicler",
        "The ability that defines Aeon Chronicler's power and toughness applies in all zones, not just the battlefield."
    );
    supported("Aeon Chronicler");
    let mut t = TestGame::new(2);
    let a = t.hand(P0, "Aeon Chronicler");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Forest");
    t.g.recompute();
    assert_eq!(t.pt(a), (3, 3), "three cards in hand, itself included");
    let g = t.graveyard(P0, "Aeon Chronicler");
    t.g.recompute();
    assert_eq!(t.pt(g), (3, 3), "in the graveyard too");
}

#[test]
fn benalish_commander_pt_applies_in_the_graveyard() {
    cr!("604.3", "604.3a");
    ruling!(
        "Benalish Commander",
        "The ability that defines Benalish Commander's power and toughness applies in all zones, not just the battlefield."
    );
    supported("Benalish Commander");
    let mut t = TestGame::new(2);
    create_token(&mut t, P0, "Soldier");
    create_token(&mut t, P0, "Soldier");
    let g = t.graveyard(P0, "Benalish Commander");
    let h = t.hand(P0, "Benalish Commander");
    t.g.recompute();
    assert_eq!(t.pt(g), (2, 2));
    assert_eq!(t.pt(h), (2, 2));
}

#[test]
fn kolaghan_forerunners_power_counts_in_every_zone_and_itself_on_the_battlefield() {
    cr!("604.3", "604.3a");
    ruling!(
        "Kolaghan Forerunners",
        "The ability that defines Kolaghan Forerunners’s power functions in all zones, not just the battlefield. If Kolaghan Forerunners is on the battlefield, its ability will count itself."
    );
    supported("Kolaghan Forerunners");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Savannah Lions");
    let h = t.hand(P0, "Kolaghan Forerunners");
    t.g.recompute();
    assert_eq!(t.pt(h), (2, 3));
    let b = t.battlefield(P0, "Kolaghan Forerunners");
    assert_eq!(t.pt(b), (3, 3));
}

// ---------------------------------------------------------------------------------------
// Abilities that matter as damage is dealt or as SBAs are checked
// ---------------------------------------------------------------------------------------

#[test]
fn cliffrunner_behemoth_lifelink_applies_as_its_white_permanent_dies_at_the_same_time() {
    cr!("702.15b", "510.2");
    ruling!(
        "Cliffrunner Behemoth",
        "Whether Cliffrunner Behemoth has lifelink matters only when it deals damage"
    );
    supported("Cliffrunner Behemoth");
    let mut t = TestGame::new(2);
    let cb = t.battlefield(P0, "Cliffrunner Behemoth");
    let lions = t.battlefield(P0, "Savannah Lions");
    let giant = t.battlefield(P1, "Hill Giant");
    to_combat(&mut t, P0);
    crate::r_s03_common::to_blockers(
        &mut t,
        &[(cb, Entity::Player(P1)), (lions, Entity::Player(P1))],
        &[(giant, lions)],
    );
    assert!(has_kw(&t, cb, KeywordKind::Lifelink));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(!t.on_battlefield(lions));
    assert!(!has_kw(&t, cb, KeywordKind::Lifelink));
    assert_eq!(t.life(P0), 25, "lifelink applied as the damage was dealt");
    assert_eq!(t.life(P1), 15);
}

#[test]
fn angelic_overseer_survives_destroying_the_humans_at_the_same_time() {
    cr!("702.12b", "608.2h");
    ruling!(
        "Angelic Overseer",
        "If you control a Human, and an effect tries to destroy each Human you control and Angelic Overseer simultaneously, Angelic Overseer won’t be destroyed."
    );
    supported("Angelic Overseer");
    let mut t = TestGame::new(2);
    let o = t.battlefield(P0, "Angelic Overseer");
    let h = t.battlefield(P0, "Elite Vanguard");
    cast_and_resolve(&mut t, P0, "Wrath of God", &[]);
    assert!(!t.on_battlefield(h));
    assert!(t.on_battlefield(o));
    assert!(!has_kw(&t, o, KeywordKind::Indestructible));
}

#[test]
fn angelic_overseer_dies_once_its_lethal_damage_meets_no_human() {
    cr!("704.5g", "702.12b", "120.6");
    ruling!(
        "Angelic Overseer",
        "If you control a Human, and lethal damage is dealt to Angelic Overseer, that damage will remain marked on it that turn."
    );
    let mut t = TestGame::new(2);
    let o = t.battlefield(P0, "Angelic Overseer");
    let h = t.battlefield(P0, "Elite Vanguard");
    let src = t.battlefield(P1, "Hill Giant");
    damage(&mut t, src, 5, o);
    assert!(t.on_battlefield(o));
    assert_eq!(t.obj_now(o).damage, 5);
    destroy(&mut t, h);
    assert!(!t.on_battlefield(o));
}

#[test]
fn anya_damage_stays_marked_but_deathtouch_is_checked_only_once() {
    cr!("704.5g", "704.5h", "702.2b", "120.6");
    ruling!(
        "Anya, Merciless Angel",
        "Damage dealt to Anya is tracked even if Anya has indestructible."
    );
    supported("Anya, Merciless Angel");
    // Lethal damage while indestructible; losing indestructible later destroys it.
    let mut t = TestGame::new(2);
    t.g.players[1].life = 9;
    let anya = t.battlefield(P0, "Anya, Merciless Angel");
    assert_eq!(t.pt(anya), (7, 7));
    assert!(has_kw(&t, anya, KeywordKind::Indestructible));
    let src = t.battlefield(P1, "Hill Giant");
    damage(&mut t, src, 5, anya);
    assert!(t.on_battlefield(anya));
    t.g.players[1].life = 20;
    t.g.recompute();
    t.settle();
    assert!(!t.on_battlefield(anya), "4/4 with 5 damage");
    // Deathtouch damage while indestructible: checked only the first time.
    let mut t = TestGame::new(2);
    t.g.players[1].life = 9;
    let anya = t.battlefield(P0, "Anya, Merciless Angel");
    let rats = t.battlefield(P1, "Typhoid Rats");
    damage(&mut t, rats, 1, anya);
    assert!(t.on_battlefield(anya));
    t.g.players[1].life = 20;
    t.g.recompute();
    t.settle();
    assert!(t.on_battlefield(anya), "deathtouch was checked only the first time");
    assert_eq!(t.pt(anya), (4, 4));
}

#[test]
fn anya_gains_indestructible_before_sbas_when_opponents_are_dealt_damage_at_once() {
    cr!("510.2", "704.3");
    ruling!(
        "Anya, Merciless Angel",
        "If damage is dealt to Anya and to your opponents at the same time, Anya may gain indestructible and/or extra toughness before state-based actions are checked."
    );
    let mut t = TestGame::new(2);
    t.g.players[1].life = 12;
    let anya = t.battlefield(P0, "Anya, Merciless Angel");
    let giant = t.battlefield(P0, "Hill Giant");
    let angel = t.battlefield(P1, "Serra Angel");
    to_combat(&mut t, P0);
    crate::r_s03_common::to_blockers(
        &mut t,
        &[(anya, Entity::Player(P1)), (giant, Entity::Player(P1))],
        &[(angel, anya)],
    );
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 9);
    assert!(t.on_battlefield(anya), "7/7 indestructible with 4 damage");
    assert_eq!(t.pt(anya), (7, 7));
    assert!(!t.on_battlefield(angel));
}

// ---------------------------------------------------------------------------------------
// Replacement effects: Mondrak, Heron of Hope
// ---------------------------------------------------------------------------------------

#[test]
fn mondraks_extra_tokens_are_also_tapped_and_attacking() {
    cr!("614.1a", "111.1", "508.4");
    ruling!(
        "Mondrak, Glory Dominus",
        "Everything that is specified by the effect creating the original token or tokens will also be true about the additional token or tokens"
    );
    supported("Mondrak, Glory Dominus");
    supported("Hero of Bladehold");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mondrak, Glory Dominus");
    let hero = t.battlefield(P0, "Hero of Bladehold");
    declare(&mut t, P0, &[(hero, Entity::Player(P1))]);
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 4);
    for tk in toks {
        assert!(t.obj_now(tk).tapped);
        assert!(attacking(&t, tk));
    }
}

/// Puts `n` Herons of Hope onto P0's battlefield.
fn herons(t: &mut TestGame, n: usize) {
    for _ in 0..n {
        t.battlefield(P0, "Heron of Hope");
    }
}

#[test]
fn heron_of_hope_applies_once_per_lifelink_creature_dealing_combat_damage() {
    cr!("702.15e", "614.1a", "119.10");
    ruling!(
        "Heron of Hope",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event."
    );
    supported("Heron of Hope");
    // Two lifelink creatures: two events, +1 each.
    let mut t = TestGame::new(2);
    herons(&mut t, 1);
    let a = t.battlefield(P0, "Vampire Nighthawk");
    let b = t.battlefield(P0, "Vampire Nighthawk");
    to_combat(&mut t, P0);
    declare(&mut t, P0, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P0), 26);
    // One lifelink creature dealing damage to two blockers: one event.
    let mut t = TestGame::new(2);
    herons(&mut t, 1);
    let g = t.battlefield(P0, "Hill Giant");
    modify_until_eot(
        &mut t,
        g,
        vec![Modification::AddKeyword(Keyword::new(KeywordKind::Lifelink))],
    );
    let e1 = t.battlefield(P1, "Llanowar Elves");
    let e2 = t.battlefield(P1, "Llanowar Elves");
    to_combat(&mut t, P0);
    crate::r_s03_common::to_blockers(&mut t, &[(g, Entity::Player(P1))], &[(e1, g), (e2, g)]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P0), 24);
}

#[test]
fn two_herons_of_hope_add_two() {
    cr!("614.1a", "616.1");
    ruling!(
        "Heron of Hope",
        "If you control two Heron of Hopes and you would gain life, you gain that much life plus 2."
    );
    let mut t = TestGame::new(2);
    herons(&mut t, 2);
    t.g.gain_life(P0, 3);
    t.settle();
    assert_eq!(t.life(P0), 25);
    herons(&mut t, 1);
    t.g.gain_life(P0, 3);
    t.settle();
    assert_eq!(t.life(P0), 31);
}

#[test]
fn heron_of_hope_applies_once_to_life_gained_for_each_creature() {
    cr!("614.1a", "119.10");
    ruling!(
        "Heron of Hope",
        "If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and the ability of Heron of Hope applies only once."
    );
    let mut t = TestGame::new(2);
    herons(&mut t, 1);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    cast_and_resolve(&mut t, P0, "Congregate", &[Entity::Player(P0)]);
    assert_eq!(t.life(P0), 20 + 6 + 1);
}

#[test]
fn heron_of_hope_ignores_a_teammates_life_gain_in_two_headed_giant() {
    cr!("810.9", "614.1a");
    ruling!(
        "Heron of Hope",
        "In a Two-Headed Giant game, life gained by your teammate won't cause Heron of Hope's second ability to apply"
    );
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    let start = t.life(P0);
    herons(&mut t, 1);
    t.g.gain_life(P1, 3);
    t.settle();
    assert_eq!(t.life(P0), start + 3);
    assert_eq!(t.life(P1), start + 3);
    t.g.gain_life(P0, 3);
    t.settle();
    assert_eq!(t.life(P0), start + 3 + 4);
}

// ---------------------------------------------------------------------------------------
// Doubling, Myojin
// ---------------------------------------------------------------------------------------

#[test]
fn two_zopandrels_each_double_power_and_toughness() {
    cr!("701.10a", "701.10b", "613.4c");
    ruling!(
        "Zopandrel, Hunger Dominus",
        "If you somehow control more than one Zopandrel, each one applies independently."
    );
    supported("Zopandrel, Hunger Dominus");
    supported("Mirror Gallery");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirror Gallery");
    t.battlefield(P0, "Zopandrel, Hunger Dominus");
    t.battlefield(P0, "Zopandrel, Hunger Dominus");
    t.settle();
    assert_eq!(t.named_on_battlefield("Zopandrel, Hunger Dominus").len(), 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(bears), (8, 8));
}

#[test]
fn myojin_of_nights_reach_cast_from_the_command_zone_gets_no_divinity_counter() {
    cr!("903.8", "601.2a");
    ruling!(
        "Myojin of Night's Reach",
        "In a Commander game where this card is your commander, casting it from the command zone does not count as casting it from your hand."
    );
    supported("Myojin of Night's Reach");
    let mut t = commander_game();
    let m = commander(&mut t, P0, "Myojin of Night's Reach");
    t.lands(P0, "Swamp", 8);
    t.cast(P0, m).go();
    t.resolve_all();
    let on = t.named_on_battlefield("Myojin of Night's Reach");
    assert_eq!(on.len(), 1);
    assert_eq!(t.counters(on[0], "divinity"), 0);
    assert!(!has_kw(&t, on[0], KeywordKind::Indestructible));
    // Cast from the hand, it gets the counter.
    let mut t = TestGame::new(2);
    let m = t.hand(P0, "Myojin of Night's Reach");
    t.lands(P0, "Swamp", 8);
    t.cast(P0, m).go();
    t.resolve_all();
    let on = t.named_on_battlefield("Myojin of Night's Reach");
    assert_eq!(t.counters(on[0], "divinity"), 1);
}

#[test]
fn myojin_of_towering_might_distributes_at_least_one_counter_to_each_of_up_to_eight_targets() {
    cr!("601.2d", "115.1");
    ruling!(
        "Myojin of Towering Might",
        "You choose how to distribute the counters as you activate Myojin of Towering Might's last ability. Each target must get at least one counter."
    );
    supported("Myojin of Towering Might");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Myojin of Towering Might");
    put_counters(&mut t, m, "indestructible", 1);
    let mut ids = vec![];
    for _ in 0..9 {
        ids.push(t.battlefield(P0, "Llanowar Elves"));
    }
    let a = ids[0];
    let b = ids[1];
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![5, 3]));
    let from = t.asked().len();
    activate_containing(&mut t, P0, m, "Distribute").unwrap();
    // The division is chosen as it's activated, with at least one each and at most 8
    // targets.
    let asked = t.asked()[from..].to_vec();
    let max = asked
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { max, .. } => Some(*max),
            _ => None,
        })
        .unwrap();
    assert_eq!(max, 8);
    assert!(asked.iter().any(|(_, d)| matches!(d, Decision::Divide { .. })));
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 5);
    assert_eq!(t.counters(b, counters::PLUS1), 3);
    assert!(has_kw(&t, a, KeywordKind::Trample));
    assert!(has_kw(&t, b, KeywordKind::Trample));
    assert!(!has_kw(&t, ids[2], KeywordKind::Trample));
}

// ---------------------------------------------------------------------------------------
// Layer 7: Shambling Vent, Sanguine Statuette
// ---------------------------------------------------------------------------------------

#[test]
fn shambling_vent_overwrites_an_earlier_pt_setting_but_keeps_counters() {
    cr!("613.4b", "613.4c", "613.7a");
    ruling!(
        "Shambling Vent",
        "if Shambling Vent has been made a 0/0 creature with three +1/+1 counters on it, activating its last ability will turn it into a 5/6 creature that's still a land."
    );
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Shambling Vent");
    put_counters(&mut t, v, counters::PLUS1, 3);
    modify_until_eot(
        &mut t,
        v,
        vec![
            Modification::AddTypes(vec![CardType::Creature]),
            Modification::SetPT(Some(Value::c(0)), Some(Value::c(0))),
        ],
    );
    assert_eq!(t.pt(v), (3, 3));
    t.lands(P0, "Scrubland", 3);
    activate_containing(&mut t, P0, v, "becomes").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(v), (5, 6));
    assert!(t.obj_now(v).is(CardType::Land));
}

/// Sacrifices a new Blood token of P0's, answering yes to the Statuette's trigger.
fn sacrifice_blood(t: &mut TestGame) {
    let blood = create_token(t, P0, "Blood");
    t.answer_yes(P0, true);
    t.g.sacrifice(blood, P0);
    t.g.flush_events();
    t.resolve_all();
}

#[test]
fn sanguine_statuette_cant_undo_a_toughness_reduction() {
    cr!("613.4b", "613.4c", "613.7a");
    ruling!(
        "Sanguine Statuette",
        "sacrificing another Blood token won't allow you to counteract that effect."
    );
    supported("Sanguine Statuette");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sanguine Statuette");
    sacrifice_blood(&mut t);
    assert_eq!(t.pt(s), (3, 3));
    assert!(has_kw(&t, s, KeywordKind::Haste));
    modify_until_eot(
        &mut t,
        s,
        vec![Modification::ModifyPT(Value::c(-2), Value::c(-2))],
    );
    assert_eq!(t.pt(s), (1, 1));
    sacrifice_blood(&mut t);
    assert_eq!(t.pt(s), (1, 1));
}

#[test]
fn sanguine_statuette_resets_a_pt_setting_effect_but_keeps_modifications() {
    cr!("613.4b", "613.4c", "613.7a");
    ruling!(
        "Sanguine Statuette",
        "If another effect sets Sanguine Statuette's power and/or toughness to another number, sacrificing a Blood token will set it back to 3/3."
    );
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sanguine Statuette");
    sacrifice_blood(&mut t);
    modify_until_eot(
        &mut t,
        s,
        vec![Modification::ModifyPT(Value::c(1), Value::c(1))],
    );
    modify_until_eot(
        &mut t,
        s,
        vec![Modification::SetPT(Some(Value::c(0)), Some(Value::c(1)))],
    );
    assert_eq!(t.pt(s), (1, 2));
    sacrifice_blood(&mut t);
    assert_eq!(t.pt(s), (4, 4));
}

#[test]
fn sanguine_sipper_counts_a_sticker_on_itself_or_another_permanent() {
    cr!("123.1", "123.3");
    ruling!(
        "Sanguine Sipper",
        "The stickered permanent could be Sanguine Sipper itself or another permanent you control."
    );
    supported("Sanguine Sipper");
    use mtg_engine::stickers::{put_sticker, StickerKind};
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sanguine Sipper");
    assert!(!has_kw(&t, s, KeywordKind::Lifelink));
    assert!(put_sticker(&mut t.g, P0, s, StickerKind::Art));
    t.g.recompute();
    assert!(has_kw(&t, s, KeywordKind::Lifelink));
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sanguine Sipper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(put_sticker(&mut t.g, P0, bears, StickerKind::Art));
    t.g.recompute();
    assert!(has_kw(&t, s, KeywordKind::Lifelink));
}

// ---------------------------------------------------------------------------------------
// Regeneration
// ---------------------------------------------------------------------------------------

#[test]
fn skithiryx_regenerated_is_healed_tapped_and_removed_from_combat() {
    cr!("701.19a", "701.19b", "506.4");
    ruling!(
        "Skithiryx, the Blight Dragon",
        "If the regeneration shield is used, Skithiryx isn't destroyed, all damage marked on Skithiryx is healed, it becomes tapped, and it's removed from combat"
    );
    supported("Skithiryx, the Blight Dragon");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Skithiryx, the Blight Dragon");
    t.lands(P0, "Swamp", 2);
    let src = t.battlefield(P1, "Hill Giant");
    declare(&mut t, P0, &[(s, Entity::Player(P1))]);
    assert!(attacking(&t, s));
    activate_containing(&mut t, P0, s, "Regenerate").unwrap();
    t.resolve_all();
    damage(&mut t, src, 2, s);
    assert_eq!(t.obj_now(s).damage, 2);
    destroy(&mut t, s);
    assert!(t.on_battlefield(s));
    assert_eq!(t.obj_now(s).damage, 0);
    assert!(t.obj_now(s).tapped);
    assert!(!attacking(&t, s));
}

// ---------------------------------------------------------------------------------------
// The legend rule and Sliver Gravemother
// ---------------------------------------------------------------------------------------

fn legions(t: &TestGame) -> usize {
    t.named_on_battlefield("Sliver Legion").len()
}

#[test]
fn legend_rule_keeps_one_of_two_same_named_legendary_slivers() {
    cr!("704.5j");
    ruling!(
        "Sliver Gravemother",
        "The \"legend rule\" is the rule that states that if a player controls two or more legendary permanents with the same name"
    );
    supported("Sliver Legion");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sliver Legion");
    t.battlefield(P0, "Sliver Legion");
    t.settle();
    assert_eq!(legions(&t), 1);
    assert_eq!(t.graveyard_size(P0), 1);
}

#[test]
fn sliver_gravemother_lets_you_keep_same_named_legendary_slivers() {
    cr!("704.5j");
    ruling!(
        "Sliver Gravemother",
        "While the \"legend rule\" doesn't apply to Slivers you control, you can control any number of legendary Slivers with the same name"
    );
    supported("Sliver Gravemother");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sliver Gravemother");
    t.battlefield(P0, "Sliver Legion");
    t.battlefield(P0, "Sliver Legion");
    t.battlefield(P0, "Sliver Legion");
    t.settle();
    assert_eq!(legions(&t), 3);
    // Not for another player's Slivers.
    t.battlefield(P1, "Sliver Legion");
    t.battlefield(P1, "Sliver Legion");
    t.settle();
    assert_eq!(legions(&t), 4);
}

#[test]
fn legend_rule_applies_again_when_sliver_gravemother_leaves() {
    cr!("704.5j", "704.3");
    ruling!(
        "Sliver Gravemother",
        "you'll immediately have to comply with the rule and put all but one of those Slivers into the graveyard."
    );
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Sliver Gravemother");
    t.battlefield(P0, "Sliver Legion");
    t.battlefield(P0, "Sliver Legion");
    t.battlefield(P0, "Sliver Legion");
    t.settle();
    assert_eq!(legions(&t), 3);
    destroy(&mut t, g);
    assert_eq!(legions(&t), 1);
}

// ---------------------------------------------------------------------------------------
// Perennial Behemoth
// ---------------------------------------------------------------------------------------

#[test]
fn perennial_behemoth_doesnt_let_you_activate_lands_in_the_graveyard() {
    cr!("305.1", "113.6");
    ruling!(
        "Perennial Behemoth",
        "Perennial Behemoth doesn't allow you to activate abilities (such as cycling) of land cards in your graveyard."
    );
    supported("Perennial Behemoth");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Perennial Behemoth");
    t.lands(P0, "Plains", 2);
    let steppe = t.graveyard(P0, "Secluded Steppe");
    assert!(!can_activate(&mut t, P0, steppe));
    assert!(can_play_land(&mut t, P0, steppe));
}

#[test]
fn perennial_behemoth_doesnt_change_when_lands_can_be_played() {
    cr!("305.1", "305.2", "116.2a");
    ruling!(
        "Perennial Behemoth",
        "Perennial Behemoth doesn't change the times when you can play those land cards."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Perennial Behemoth");
    let a = t.graveyard(P0, "Forest");
    let b = t.graveyard(P0, "Island");
    // Not in combat, not with a spell on the stack, not in the opponent's turn.
    to_combat(&mut t, P0);
    assert!(!can_play_land(&mut t, P0, a));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_play_land(&mut t, P0, a));
    t.set_step(P0, Step::PrecombatMain);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert!(!can_play_land(&mut t, P0, a));
    t.resolve_all();
    assert!(can_play_land(&mut t, P0, a));
    t.play_land(P0, a).unwrap();
    // Only one land per turn.
    assert!(!can_play_land(&mut t, P0, b));
}
