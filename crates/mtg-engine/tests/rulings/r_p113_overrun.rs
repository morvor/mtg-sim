//! Rulings batch P113 — overrun-style pumps ("creatures you control get +N/+N and gain
//! trample until end of turn"): the affected set is locked in as the effect resolves (CR
//! 611.2c), values are determined once (CR 608.2h), attacking creatures, and combinations
//! of evasion abilities.

use crate::r_p113_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::destroy;
use crate::r_s07_common::{damage_on, has_kw};
use crate::r_s21_common::legal_blocks;
use crate::r_s25_common::cast_new;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::*;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// Makes `id` a 2/2 creature in addition to its other types until end of turn.
fn animate(t: &mut TestGame, id: ObjectId) {
    modify_until_eot(
        t,
        id,
        vec![
            Modification::AddTypes(vec![CardType::Creature]),
            Modification::SetPT(Some(Value::c(2)), Some(Value::c(2))),
        ],
    );
}

#[test]
fn mass_pumps_affect_only_creatures_you_control_as_they_resolve() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Overrun",
        "Overrun affects only creatures you control at the time it resolves. It won't affect creatures that come under your control later in the turn."
    );
    ruling!(
        "Overcome",
        "Overcome affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won't get +2/+2 or gain trample."
    );
    ruling!(
        "Full Steam Ahead",
        "Full Steam Ahead affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn and noncreature permanents that become creatures later in the turn won’t get +2/+2 or gain any of the listed abilities."
    );
    ruling!(
        "Stampede Driver",
        "Stampede Driver's ability affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won't get +1/+1 or gain trample."
    );
    ruling!(
        "Vivien of the Arkbow",
        "Vivien’s last ability affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won’t get +4/+4 or gain trample."
    );
    ruling!(
        "Balmor, Battlemage Captain",
        "Balmor's last ability affects only creatures you control at the time it resolves, including Balmor itself. Creatures you begin to control later in the turn or noncreature permanents that become creatures later won't get the bonus."
    );
    // (source, bonus to power)
    for (src, bonus) in [
        ("Overrun", 3),
        ("Overcome", 2),
        ("Full Steam Ahead", 2),
        ("Stampede Driver", 1),
        ("Vivien of the Arkbow", 4),
        ("Balmor, Battlemage Captain", 1),
    ] {
        supported(src);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let factory = t.battlefield(P0, "Darksteel Ingot");
        let stolen = t.battlefield(P1, "Hill Giant");
        let mut own = None;
        match src {
            "Stampede Driver" => {
                let d = t.battlefield(P0, src);
                own = Some((d, 1));
                t.lands(P0, "Forest", 2);
                t.hand(P0, "Shock");
                t.activate(P0, d, 0, &[]).expect("Stampede Driver");
            }
            "Vivien of the Arkbow" => {
                let v = t.battlefield(P0, src);
                t.g.objects[v.0 as usize]
                    .counters
                    .insert(mtg_engine::types::counters::LOYALTY.into(), 9);
                t.activate(P0, v, 2, &[]).expect("-9");
            }
            "Balmor, Battlemage Captain" => {
                let b = t.battlefield(P0, src);
                own = Some((b, 1));
                cast_new(&mut t, P0, "Opt", &[]);
            }
            _ => {
                cast_modes(&mut t, P0, src, &[], &[]);
            }
        }
        t.resolve_all();
        assert_eq!(t.pt(bears).0, 2 + bonus, "{src}");
        if let Some((o, p)) = own {
            assert_eq!(t.pt(o).0, p + bonus, "{src} itself");
        }
        // Later: a new creature, a stolen one, and an animated noncreature permanent.
        let later = t.battlefield(P0, "Grizzly Bears");
        crate::r_s06_common::give_control(&mut t, stolen, P0);
        animate(&mut t, factory);
        assert_eq!(t.pt(later), (2, 2), "{src}");
        assert_eq!(t.pt(stolen), (3, 3), "{src}");
        assert_eq!(t.pt(factory), (2, 2), "{src}");
        assert!(!has_kw(&t, later, KeywordKind::Trample), "{src}");
        assert!(!has_kw(&t, factory, KeywordKind::Trample), "{src}");
        assert!(has_kw(&t, bears, KeywordKind::Trample), "{src}");
    }
}

#[test]
fn overwhelming_stampede_checks_the_greatest_power_as_it_resolves() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Overwhelming Stampede",
        "You check the power of your creatures as Overwhelming Stampede resolves. For example, if you control a 2/1 creature, a 2/2 creature, a 2/4 creature, a 4/1 creature, a 5/5 creature, and a 5/6 creature at that time, each of your creatures gets +5/+5 and gains trample until end of turn."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    cast_modes(&mut t, P0, "Overwhelming Stampede", &[], &[]);
    // In response, the Giant gets +3/+3: the greatest power is 6 as the spell resolves.
    cast_new(&mut t, P0, "Giant Growth", &[ent(giant)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (8, 8));
    assert_eq!(t.pt(giant), (12, 12));
    // It's not updated later.
    destroy(&mut t, giant);
    assert_eq!(t.pt(bears), (8, 8));
}

#[test]
fn pathbreaker_ibex_negative_greatest_power_is_zero() {
    cr!("107.1b", "608.2h");
    ruling!(
        "Pathbreaker Ibex",
        "If the greatest power among creatures you control is negative as this creature’s ability resolves, X is considered to be 0."
    );
    supported("Pathbreaker Ibex");
    let mut t = TestGame::new(2);
    let ibex = t.battlefield(P0, "Pathbreaker Ibex");
    let bears = t.battlefield(P0, "Grizzly Bears");
    modify_until_eot(&mut t, ibex, vec![Modification::ModifyPT(Value::c(-5), Value::c(0))]);
    modify_until_eot(&mut t, bears, vec![Modification::ModifyPT(Value::c(-4), Value::c(0))]);
    attack_with(&mut t, &[(ibex, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(ibex), (-2, 3));
    assert_eq!(t.pt(bears), (-2, 2));
    assert!(has_kw(&t, bears, KeywordKind::Trample));
}

#[test]
fn volcanic_rush_affects_creatures_attacking_as_it_resolves_for_the_rest_of_the_turn() {
    cr!("506.4", "611.2c", "508.1");
    ruling!(
        "Volcanic Rush",
        "Only creatures that are attacking as Volcanic Rush resolves will receive the bonus. In other words, casting it before you declare attackers usually won’t do anything."
    );
    ruling!(
        "Volcanic Rush",
        "Even though Volcanic Rush affects only attacking creatures, the bonus will remain for the rest of the turn."
    );
    ruling!(
        "Volcanic Rush",
        "Unless that creature leaves combat, it continues to be an attacking creature through the end of combat step, even if the player it was attacking has left the game, or the planeswalker it was attacking has left the battlefield."
    );
    supported("Volcanic Rush");
    // Before attackers are declared: nothing.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    cast_modes(&mut t, P0, "Volcanic Rush", &[], &[]);
    t.resolve_all();
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(t.pt(bears), (2, 2));
    // Attacking a planeswalker that has left the battlefield: still attacking.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let home = t.battlefield(P0, "Hill Giant");
    let jace = t.battlefield(P1, "Jace Beleren");
    attack_with(&mut t, &[(bears, ent(jace))]);
    destroy(&mut t, jace);
    cast_modes(&mut t, P0, "Volcanic Rush", &[], &[]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    assert!(has_kw(&t, bears, KeywordKind::Trample));
    assert_eq!(t.pt(home), (3, 3), "not attacking");
    // The bonus lasts after combat.
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.pt(bears), (4, 2));
}

#[test]
fn howlpack_resurgence_wolf_and_werewolf_gets_the_bonus_once() {
    cr!("613.4c", "702.73a");
    ruling!(
        "Howlpack Resurgence",
        "A creature that is both a Werewolf and a Wolf will only get +1/+1 from Howlpack Resurgence’s last ability."
    );
    supported("Howlpack Resurgence");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Howlpack Resurgence");
    // Changeling Outcast (1/1, changeling) is every creature type.
    let outcast = t.battlefield(P0, "Changeling Outcast");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(outcast), (2, 2));
    assert!(has_kw(&t, outcast, KeywordKind::Trample));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn full_steam_ahead_menace_and_one_blocker_limit_cant_be_blocked() {
    cr!("702.111b", "509.1b", "509.1c");
    ruling!(
        "Full Steam Ahead",
        "A creature with menace and “This creature can’t be blocked by more than one creature” can’t be blocked at all."
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let plain = t.battlefield(P0, "Hill Giant");
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Craw Wurm");
    cast_modes(&mut t, P0, "Full Steam Ahead", &[], &[]);
    t.resolve_all();
    modify_until_eot(
        &mut t,
        mine,
        vec![Modification::AddKeyword(Keyword::new(KeywordKind::Menace))],
    );
    attack_with(
        &mut t,
        &[(mine, Entity::Player(P1)), (plain, Entity::Player(P1))],
    );
    assert!(!legal_blocks(&mut t, P1, &[(a, mine)]));
    assert!(!legal_blocks(&mut t, P1, &[(a, mine), (b, mine)]));
    // Without menace, one blocker is fine, but not two.
    assert!(legal_blocks(&mut t, P1, &[(a, plain)]));
    assert!(!legal_blocks(&mut t, P1, &[(a, plain), (b, plain)]));
}

#[test]
fn rumbleweed_cost_reduction_doesnt_change_its_mana_value() {
    cr!("202.3", "601.2f");
    ruling!(
        "Rumbleweed",
        "Rumbleweed’s first ability doesn’t change its mana value, which is always 11."
    );
    supported("Rumbleweed");
    let mut t = TestGame::new(2);
    for _ in 0..5 {
        t.graveyard(P0, "Forest");
    }
    t.lands(P0, "Forest", 6);
    let card = t.hand(P0, "Rumbleweed");
    assert_eq!(t.g.mana_value_of(card), 11);
    let spell = t.cast(P0, card).go();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 6);
    assert_eq!(t.g.mana_value_of(spell), 11);
    t.resolve_all();
    let perm = t.g.current(spell);
    assert_eq!(t.g.mana_value_of(perm), 11);
}

#[test]
fn garruk_destroy_and_draw_with_an_illegal_or_indestructible_target() {
    cr!("608.2b", "702.12b");
    ruling!(
        "Garruk, Cursed Huntsman",
        "If the target creature is an illegal target by the time Garruk's second ability tries to resolve, the ability won't resolve. You won't draw a card. If the target is legal but not destroyed (most likely because it has indestructible), you will still draw."
    );
    for (illegal, indestructible) in [(false, false), (true, false), (false, true)] {
        let mut t = TestGame::new(2);
        let g = t.battlefield(P0, "Garruk, Cursed Huntsman");
        let bears = t.battlefield(P1, "Grizzly Bears");
        if indestructible {
            modify_until_eot(
                &mut t,
                bears,
                vec![Modification::AddKeyword(Keyword::new(
                    KeywordKind::Indestructible,
                ))],
            );
        }
        let hand = t.hand_size(P0);
        t.answer_targets(P0, &[ent(bears)]);
        crate::r_s06_common::activate_containing(&mut t, P0, g, "Destroy target creature")
            .expect("-3");
        if illegal {
            spoil(&mut t, bears, Spoil::Hexproof);
        }
        t.resolve_all();
        assert_eq!(t.on_battlefield(bears), illegal || indestructible);
        let drew = if illegal { 0 } else { 1 };
        assert_eq!(t.hand_size(P0), hand + drew, "illegal:{illegal}");
    }
}

#[test]
fn dinosaur_stampede_in_two_headed_giant() {
    cr!("810.2", "611.2c");
    ruling!(
        "Dinosaur Stampede",
        "In a Two-Headed Giant game, attacking creatures your teammate controls get +2/+0, but only Dinosaurs you control gain trample."
    );
    supported("Dinosaur Stampede");
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    // P0 and P1 are teammates.
    let mine = t.battlefield(P0, "Grizzly Bears");
    let dino = t.battlefield(P0, "Ravenous Daggertooth");
    let mate = t.battlefield(P1, "Hill Giant");
    let mate_dino = t.battlefield(P1, "Ravenous Daggertooth");
    let foe = t.battlefield(PlayerId(2), "Craw Wurm");
    attack_with(
        &mut t,
        &[
            (mine, Entity::Player(PlayerId(2))),
            (mate, Entity::Player(PlayerId(2))),
            (mate_dino, Entity::Player(PlayerId(2))),
        ],
    );
    cast_modes(&mut t, P0, "Dinosaur Stampede", &[], &[]);
    t.resolve_all();
    assert_eq!(t.pt(mine), (4, 2));
    assert_eq!(t.pt(mate), (5, 3));
    assert_eq!(t.pt(mate_dino).0, 5);
    assert!(!has_kw(&t, mate_dino, KeywordKind::Trample), "a teammate's Dinosaur");
    assert_eq!(t.pt(foe), (6, 4));
    assert_eq!(t.pt(dino).0, 3, "not attacking");
    assert!(has_kw(&t, dino, KeywordKind::Trample), "my Dinosaur");
    assert!(!has_kw(&t, mate, KeywordKind::Trample));
    let _ = damage_on;
}
