//! Rulings batch P174 — effects that tap permanents: targets that are already tapped
//! (CR 115.1, 701.26a), partially illegal targets (CR 608.2b), mandatory targets
//! (CR 603.3d), defending players in multiplayer (CR 506.2, 802), Two-Headed Giant
//! (CR 810.9), tapping attackers after they're declared (CR 506.4, 508.1f), "enter
//! tapped" effects and phasing (CR 702.26d, 614.1c), emptying mana pools, and paying
//! costs without interruption (CR 601.2, 602.2).

use crate::r_s01_common::{asked_since, attack_with, block_and_finish, give_mana_for, supported};
use crate::r_s06_common::activate_containing;
use crate::r_s20_common::tap_for_mana;
use crate::r_s24_common::tapped;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn tap(t: &mut TestGame, id: ObjectId) {
    t.g.objects[id.0 as usize].tapped = true;
}

/// The candidates of the last target choice asked since `from`.
fn target_candidates(t: &TestGame, from: usize) -> Vec<Entity> {
    asked_since(t, from)
        .into_iter()
        .rev()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates),
            _ => None,
        })
        .expect("a target choice")
}

// --- Targets that are already tapped (cheap cases) ----------------------------------------

#[test]
fn tap_effects_can_target_tapped_permanents() {
    cr!("115.1", "701.26a");
    ruling!("Onslaught", "Can target an already tapped creature.");
    ruling!("Mishra's Helix", "You can target already tapped lands.");
    ruling!(
        "Repel the Darkness",
        "You may target zero, one, or two creatures. The creatures don't need to be untapped."
    );
    ruling!(
        "Song-Mad Treachery",
        "Song-Mad Treachery can target any creature, even one that's untapped or one you already control."
    );
    ruling!(
        "Wolfhunter's Quiver",
        "You could activate the first ability targeting a Werewolf creature if you wanted to."
    );
    // Onslaught: "Whenever you cast a creature spell, tap target creature."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Onslaught");
    let b = t.battlefield(P1, "Grizzly Bears");
    tap(&mut t, b);
    give_mana_for(&mut t, P0, "Llanowar Elves");
    let elves = t.hand(P0, "Llanowar Elves");
    t.answer_targets(P0, &[obj(b)]);
    t.cast(P0, elves).go();
    t.settle();
    assert_eq!(t.stack_len(), 2, "the trigger targets the tapped Bears");
    t.resolve_all();
    // Mishra's Helix: X = 2 targeting a tapped and an untapped land.
    let mut t = TestGame::new(2);
    let helix = t.battlefield(P0, "Mishra's Helix");
    t.lands(P0, "Wastes", 2);
    let l1 = t.battlefield(P1, "Forest");
    let l2 = t.battlefield(P1, "Forest");
    tap(&mut t, l1);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_targets(P0, &[obj(l1), obj(l2)]);
    activate_containing(&mut t, P0, helix, "Tap X target lands").expect("activates");
    t.resolve_all();
    assert!(tapped(&t, l1) && tapped(&t, l2));
    // Repel the Darkness: two tapped creatures; you still draw.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    tap(&mut t, a);
    tap(&mut t, b);
    give_mana_for(&mut t, P0, "Repel the Darkness");
    let s = t.hand(P0, "Repel the Darkness");
    t.cast(P0, s).targets(&[obj(a), obj(b)]).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    // Song-Mad Treachery on P0's own untapped creature.
    let mut t = TestGame::new(2);
    let mine = t.battlefield_sick(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Song-Mad Treachery // Song-Mad Ruins");
    let s = t.hand(P0, "Song-Mad Treachery // Song-Mad Ruins");
    assert!(t.cast(P0, s).target(mine).try_go().is_ok());
    t.resolve_all();
    assert!(!tapped(&t, mine));
    assert!(crate::r_s06_common::has_kw(
        &t,
        mine,
        mtg_engine::keywords::KeywordKind::Haste
    ));
    // Wolfhunter's Quiver: the 1-damage ability targets a Werewolf creature.
    supported("Wolfhunter's Quiver");
    let mut t = TestGame::new(2);
    let q = t.battlefield(P0, "Wolfhunter's Quiver");
    let c = t.battlefield(P0, "Grizzly Bears");
    assert!(t.g.attach(q, obj(c)));
    let wolf = t.battlefield(P1, "Village Ironsmith // Ironfang");
    t.answer_targets(P0, &[obj(wolf)]);
    activate_containing(&mut t, P0, c, "1 damage").expect("activates");
    t.resolve_all();
    assert!(!t.on_battlefield(wolf), "the 1/1 Werewolf was dealt 1 damage");
}

#[test]
fn chandras_revolution_taps_an_already_tapped_land_that_then_stays_tapped() {
    cr!("115.1", "502.3");
    ruling!(
        "Chandra's Revolution",
        "Chandra’s Revolution can target a land that’s already tapped. That land won’t untap during its controller’s next untap step."
    );
    supported("Chandra's Revolution");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let land = t.battlefield(P1, "Forest");
    tap(&mut t, land);
    give_mana_for(&mut t, P0, "Chandra's Revolution");
    let s = t.hand(P0, "Chandra's Revolution");
    t.cast(P0, s).targets(&[obj(bears), obj(land)]).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    t.advance_to(P1, Step::Upkeep);
    assert!(tapped(&t, land));
}

// --- Targets that become illegal -----------------------------------------------------------

#[test]
fn downpour_taps_the_remaining_legal_targets() {
    cr!("608.2b");
    ruling!(
        "Downpour",
        "If Downpour targets more than one creature, and some but not all of the creatures are illegal targets by the time Downpour resolves, the remaining legal targets will be tapped."
    );
    supported("Downpour");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let c = t.battlefield(P1, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Downpour");
    let s = t.hand(P0, "Downpour");
    t.cast(P0, s).targets(&[obj(a), obj(b), obj(c)]).go();
    crate::r_s02_common::destroy(&mut t, a);
    t.resolve_all();
    assert!(tapped(&t, b) && tapped(&t, c));
}

#[test]
fn pressure_point_draws_only_if_its_target_is_legal() {
    cr!("608.2b", "701.26a");
    ruling!(
        "Pressure Point",
        "If the target creature is an illegal target by the time Pressure Point tries to resolve, the spell doesn't resolve. You don't draw a card. If the target is legal but doesn't become tapped (most likely because it's already tapped), you do draw a card."
    );
    supported("Pressure Point");
    for (gone, already_tapped, draws) in [(true, false, 0), (false, true, 1), (false, false, 1)] {
        let mut t = TestGame::new(2);
        let b = t.battlefield(P1, "Grizzly Bears");
        if already_tapped {
            tap(&mut t, b);
        }
        give_mana_for(&mut t, P0, "Pressure Point");
        let s = t.hand(P0, "Pressure Point");
        t.cast(P0, s).target(b).go();
        if gone {
            crate::r_s02_common::destroy(&mut t, b);
        }
        t.resolve_all();
        assert_eq!(t.hand_size(P0), draws, "{gone} {already_tapped}");
    }
}

#[test]
fn repel_the_darkness_with_zero_one_or_two_targets() {
    cr!("608.2b", "115.1d");
    ruling!(
        "Repel the Darkness",
        "If you target one creature and that target is illegal as Repel the Darkness resolves, the spell doesn't resolve. You don't draw a card."
    );
    ruling!(
        "Repel the Darkness",
        "If you target two creatures and they're both illegal as Repel the Darkness resolves, the spell doesn't resolve; you don't draw a card. If just one target is illegal, the spell does resolve"
    );
    ruling!(
        "Repel the Darkness",
        "If you target zero creatures, all that happens is you draw a card."
    );
    // (targets, how many of them leave, cards drawn)
    for (n, gone, draws) in [(1, 1, 0), (2, 2, 0), (2, 1, 1), (0, 0, 1)] {
        let mut t = TestGame::new(2);
        let bears: Vec<ObjectId> = (0..n).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
        give_mana_for(&mut t, P0, "Repel the Darkness");
        let s = t.hand(P0, "Repel the Darkness");
        let targets: Vec<Entity> = bears.iter().map(|b| obj(*b)).collect();
        if n == 0 {
            t.answer_targets(P0, &[]);
        }
        t.cast(P0, s).targets(&targets).go();
        for b in &bears[..gone] {
            crate::r_s02_common::destroy(&mut t, *b);
        }
        t.resolve_all();
        assert_eq!(t.hand_size(P0), draws, "{n} {gone}");
        for b in &bears[gone..] {
            assert!(tapped(&t, *b), "{n} {gone}");
        }
    }
}

#[test]
fn glaring_aegis_doesnt_enter_if_its_target_is_illegal() {
    cr!("608.3b", "303.4a");
    ruling!(
        "Glaring Aegis",
        "If the target creature is an illegal target by the time Glaring Aegis tries to resolve, the spell doesn't resolve. It won't enter the battlefield, so its ability won't trigger."
    );
    supported("Glaring Aegis");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Glaring Aegis");
    let s = t.hand(P0, "Glaring Aegis");
    t.cast(P0, s).target(mine).go();
    crate::r_s02_common::destroy(&mut t, mine);
    t.answer_targets(P0, &[obj(theirs)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Glaring Aegis"));
    assert!(!tapped(&t, theirs));
}

// --- Mandatory targets --------------------------------------------------------------------

#[test]
fn mandatory_tap_triggers_target_your_own_creatures_if_they_must() {
    cr!("603.3d", "115.1");
    ruling!(
        "Onslaught",
        "The ability is not optional. If you have the only targetable creatures, you have to tap one of your own."
    );
    ruling!(
        "Harrier Griffin",
        "If your opponents control no creatures, you must target Harrier Griffin or another of your own creatures."
    );
    // Onslaught with only P0's creatures around.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Onslaught");
    let mine = t.battlefield(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Llanowar Elves");
    let elves = t.hand(P0, "Llanowar Elves");
    t.cast(P0, elves).go();
    t.resolve_all();
    assert!(tapped(&t, mine));
    // Harrier Griffin at P0's upkeep, P1 has no creatures: it taps itself.
    supported("Harrier Griffin");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Harrier Griffin");
    t.set_step(P0, Step::Untap);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(tapped(&t, g));
}

#[test]
fn tempest_caller_taps_hexproof_creatures() {
    cr!("115.1", "702.11b");
    ruling!(
        "Tempest Caller",
        "Tempest Caller’s ability targets only the player. Creatures with hexproof that player controls will be tapped as that ability resolves."
    );
    supported("Tempest Caller");
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P1, "Gladecover Scout");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Tempest Caller");
    t.resolve_all();
    assert!(tapped(&t, scout) && tapped(&t, bears));
}

// --- Defending players --------------------------------------------------------------------

#[test]
fn attack_triggers_target_only_the_defending_players_creatures() {
    cr!("506.2", "508.5", "802.2");
    ruling!(
        "Fiend Binder",
        "In a multiplayer game, Fiend Binder’s triggered ability can only target a creature controlled by the player it’s attacking"
    );
    ruling!(
        "Master of Diversion",
        "In multiplayer formats, including Commander, only a creature controlled by the defending player Master of Diversion is attacking can be chosen as the target"
    );
    ruling!(
        "Master of Diversion",
        "The ability will resolve and the creature will become tapped before blocking creatures are declared."
    );
    for name in ["Fiend Binder", "Master of Diversion"] {
        supported(name);
        let mut t = TestGame::with_config(
            3,
            GameConfig {
                attack_multiple_players: true,
                ..Default::default()
            },
        );
        let m = t.battlefield(P0, name);
        let other = t.battlefield(P0, "Grizzly Bears");
        let p1s = t.battlefield(P1, "Grizzly Bears");
        let p2s = t.battlefield(P2, "Grizzly Bears");
        let from = t.asked().len();
        t.answer_targets(P0, &[obj(p1s)]);
        attack_with(&mut t, &[(m, Entity::Player(P1)), (other, Entity::Player(P2))]);
        let cands = target_candidates(&t, from);
        assert!(cands.contains(&obj(p1s)), "{name}");
        assert!(!cands.contains(&obj(p2s)), "{name}");
        t.resolve_all();
        // Tapped before blockers are declared: it can't block.
        assert!(tapped(&t, p1s), "{name}");
        block_and_finish(&mut t, P1, &[(p1s, m)]);
        assert_eq!(t.life(P1), 20 - t.pt(m).0, "{name}");
    }
}

#[test]
fn legion_guildmage_makes_the_opposing_team_lose_six_in_two_headed_giant() {
    cr!("810.9", "810.9a");
    ruling!(
        "Legion Guildmage",
        "In a Two-Headed Giant game, Legion Guildmage's first ability causes the opposing team to lose 6 life."
    );
    supported("Legion Guildmage");
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    let g = t.battlefield(P0, "Legion Guildmage");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 5);
    let start = t.life(P2);
    activate_containing(&mut t, P0, g, "each opponent").expect("activates");
    t.resolve_all();
    assert_eq!(t.life(P2), start - 6);
    assert_eq!(t.life(P3), start - 6);
    assert_eq!(t.life(P0), start);
}

#[test]
fn haazda_snare_squad_targets_on_trigger_and_pays_on_resolution() {
    cr!("603.3d", "608.2d", "118.12");
    ruling!(
        "Haazda Snare Squad",
        "You choose the target creature when the ability triggers and goes on the stack. You choose whether to pay {W} when that ability resolves. You may pay {W} only once. The creature will be tapped before blockers are chosen."
    );
    supported("Haazda Snare Squad");
    for pay in [true, false] {
        let mut t = TestGame::new(2);
        let s = t.battlefield(P0, "Haazda Snare Squad");
        t.lands(P0, "Plains", 2);
        let b = t.battlefield(P1, "Grizzly Bears");
        let from = t.asked().len();
        t.answer_targets(P0, &[obj(b)]);
        attack_with(&mut t, &[(s, Entity::Player(P1))]);
        assert!(asked_since(&t, from)
            .iter()
            .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
        assert!(crate::r_s01_common::tapped_lands(&t, P0) == 0, "not paid yet");
        t.answer_yes(P0, pay);
        t.resolve_all();
        assert_eq!(tapped(&t, b), pay);
        assert_eq!(
            crate::r_s01_common::tapped_lands(&t, P0),
            pay as usize,
            "{{W}} paid once"
        );
    }
}

// --- Tapping attackers and Mirri ----------------------------------------------------------

#[test]
fn tapping_an_attacker_doesnt_remove_it_from_combat() {
    cr!("506.4", "508.1f");
    ruling!(
        "Pacification Array",
        "If you want to stop someone from attacking with a creature by using Pacification Array’s ability, you must do so before attackers are declared."
    );
    ruling!(
        "Mirri, Weatherlight Duelist",
        "Tapping Mirri after creatures have attacked will not remove creatures from combat."
    );
    supported("Pacification Array");
    // P1's vigilant Angel attacks; P0 taps it with Pacification Array afterwards.
    let mut t = TestGame::new(2);
    let array = t.battlefield(P0, "Pacification Array");
    t.lands(P0, "Wastes", 2);
    let angel = t.battlefield(P1, "Serra Angel");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(angel, Entity::Player(P0))]);
    t.answer_targets(P0, &[obj(angel)]);
    activate_containing(&mut t, P0, array, "Tap target").expect("activates");
    t.resolve_all();
    assert!(tapped(&t, angel) && t.g.is_attacking(angel));
    block_and_finish(&mut t, P0, &[]);
    assert_eq!(t.life(P0), 16);
    // Mirri tapped after two creatures attacked her controller: both still attack.
    supported("Mirri, Weatherlight Duelist");
    let mut t = TestGame::new(2);
    let mirri = t.battlefield(P1, "Mirri, Weatherlight Duelist");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    tap(&mut t, mirri);
    t.g.recompute();
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn tapped_mirri_restricts_attacks_on_its_controller_only() {
    cr!("508.1c");
    ruling!(
        "Mirri, Weatherlight Duelist",
        "While Mirri is tapped, there's no restriction on how many creatures can attack planeswalkers you control or battles you protect."
    );
    let mut t = TestGame::new(2);
    let mirri = t.battlefield(P1, "Mirri, Weatherlight Duelist");
    tap(&mut t, mirri);
    let pw = t.battlefield(P1, "Ajani Goldmane");
    if t.counters(pw, counters::LOYALTY) == 0 {
        t.g.add_counters(obj(pw), counters::LOYALTY, 4, None);
    }
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(a, obj(pw)), (b, obj(pw))]);
    assert!(t.g.is_attacking(a) && t.g.is_attacking(b));
    // Attacking the player: no more than one (and with Mirri untapped, no limit).
    for mirri_tapped in [true, false] {
        let mut t = TestGame::new(2);
        let mirri = t.battlefield(P1, "Mirri, Weatherlight Duelist");
        if mirri_tapped {
            tap(&mut t, mirri);
        }
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Grizzly Bears");
        t.set_step(P0, Step::BeginningOfCombat);
        t.g.recompute();
        let options = mtg_engine::combat::attack_options(&t.g);
        let legal = |decl: &[(ObjectId, Entity)]| {
            mtg_engine::combat::attack_declaration_legal(&t.g, &options, decl)
        };
        let both = [(a, Entity::Player(P1)), (b, Entity::Player(P1))];
        assert_eq!(legal(&both), !mirri_tapped, "tapped: {mirri_tapped}");
        assert!(legal(&both[..1]));
        let decl: &[(ObjectId, Entity)] = if mirri_tapped { &both[..1] } else { &both };
        attack_with(&mut t, decl);
        assert_eq!(t.g.attackers().len(), decl.len(), "tapped: {mirri_tapped}");
    }
}

#[test]
fn vigilance_gained_after_attacking_doesnt_untap() {
    cr!("702.20b", "508.1f");
    ruling!(
        "Veteran Warleader",
        "Vigilance matters only as attackers are declared. Giving a creature vigilance after that point won't untap it, even if it's attacking."
    );
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Veteran Warleader");
    t.battlefield(P0, "Expedition Envoy");
    attack_with(&mut t, &[(w, Entity::Player(P1))]);
    assert!(tapped(&t, w));
    // "Vigilance" is the second option.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    activate_containing(&mut t, P0, w, "Tap another untapped Ally").unwrap();
    t.resolve_all();
    assert!(crate::r_s06_common::has_kw(
        &t,
        w,
        mtg_engine::keywords::KeywordKind::Vigilance
    ));
    assert!(tapped(&t, w));
}

#[test]
fn frondland_felidar_granted_ability_after_attacking() {
    cr!("702.20b", "508.1f");
    ruling!(
        "Frondland Felidar",
        "You can activate the ability granted by Frondland Felidar after attacking but before blockers are chosen."
    );
    supported("Frondland Felidar");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Frondland Felidar");
    t.lands(P0, "Wastes", 1);
    let b = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(f, Entity::Player(P1))]);
    assert!(!tapped(&t, f), "vigilance");
    t.answer_targets(P0, &[obj(b)]);
    activate_containing(&mut t, P0, f, "Tap target creature").expect("activates");
    t.resolve_all();
    assert!(tapped(&t, b) && tapped(&t, f));
    assert!(t.g.is_attacking(f));
}

// --- Pacification Array and paying costs --------------------------------------------------

#[test]
fn pacification_array_cant_undo_an_activated_tap_ability() {
    cr!("602.2b", "608.2");
    ruling!(
        "Pacification Array",
        "Once a player has activated an ability with {T} in its cost, Pacification Array can’t be used to undo or counter that ability."
    );
    let mut t = TestGame::new(2);
    let array = t.battlefield(P0, "Pacification Array");
    t.lands(P0, "Wastes", 2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let icy = t.battlefield(P1, "Icy Manipulator");
    t.lands(P1, "Wastes", 1);
    t.answer_targets(P1, &[obj(mine)]);
    activate_containing(&mut t, P1, icy, "Tap target").expect("P1 activates");
    t.answer_targets(P0, &[obj(icy)]);
    activate_containing(&mut t, P0, array, "Tap target").expect("P0 responds");
    t.resolve_all();
    assert!(tapped(&t, mine), "Icy's ability still resolved");
}

#[test]
fn players_may_respond_to_pacification_array_with_the_target() {
    cr!("116.1", "602.2");
    ruling!(
        "Pacification Array",
        "Players may respond to Pacification Array’s ability by activating an ability of the target permanent"
    );
    supported("Prodigal Pyromancer");
    let mut t = TestGame::new(2);
    let array = t.battlefield(P0, "Pacification Array");
    t.lands(P0, "Wastes", 2);
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    t.answer_targets(P0, &[obj(pyro)]);
    activate_containing(&mut t, P0, array, "Tap target").expect("activates");
    t.answer_targets(P1, &[Entity::Player(P0)]);
    activate_containing(&mut t, P1, pyro, "1 damage").expect("P1 responds");
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert!(tapped(&t, pyro));
}

#[test]
fn no_player_acts_while_a_spell_or_ability_is_being_paid_for() {
    cr!("601.2", "602.2", "117.1");
    ruling!(
        "Icy Manipulator",
        "Once a player announces that they're casting a spell or activating an ability, no player may take other actions until the spell or ability has been paid for."
    );
    ruling!(
        "Rishadan Port",
        "Once you announce that you're casting a spell or activating an ability, no player may take other actions until the spell or ability has been paid for."
    );
    supported("Rishadan Port");
    // P1 holds Icy Manipulator and Rishadan Port with mana for both; while P0 casts a
    // spell (paying with lands) and activates an ability, P1 gets no decision.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Icy Manipulator");
    t.battlefield(P1, "Rishadan Port");
    t.lands(P1, "Wastes", 2);
    let lands = t.lands(P0, "Forest", 2);
    let from = t.asked().len();
    let s = t.hand(P0, "Grizzly Bears");
    t.cast(P0, s).go();
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, pyro, "1 damage").expect("activates");
    assert!(asked_since(&t, from).iter().all(|(p, _)| *p == P0));
    assert!(lands.iter().all(|l| tapped(&t, *l)));
}

#[test]
fn burden_of_guilt_only_its_controller_activates() {
    cr!("602.2");
    ruling!(
        "Burden of Guilt",
        "Only Burden of Guilt’s controller can activate its ability."
    );
    supported("Burden of Guilt");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P1, "Grizzly Bears");
    let aura = t.battlefield(P0, "Burden of Guilt");
    assert!(t.g.attach(aura, obj(b)));
    t.lands(P1, "Wastes", 1);
    assert!(activate_containing(&mut t, P1, aura, "Tap enchanted").is_err());
    assert!(!tapped(&t, b));
    t.lands(P0, "Wastes", 1);
    assert!(activate_containing(&mut t, P0, aura, "Tap enchanted").is_ok());
    t.resolve_all();
    assert!(tapped(&t, b));
}

// --- Enter tapped -------------------------------------------------------------------------

#[test]
fn opponents_permanents_enter_tapped_however_they_enter() {
    cr!("614.1c", "614.12", "802.2");
    ruling!("Frozen Aether", "It affects all opponents.");
    ruling!("Kismet", "It affects all opponents.");
    ruling!(
        "Frozen Aether",
        "It applies to cards and tokens that are put onto the battlefield by an effect, as well as ones that are played from the player's hand."
    );
    for name in ["Frozen Aether", "Kismet"] {
        supported(name);
        let mut t = TestGame::new(3);
        t.battlefield(P0, name);
        let mine = t.enter(P0, "Grizzly Bears");
        let p1 = t.enter(P1, "Grizzly Bears");
        let p2 = t.enter(P2, "Ornithopter");
        assert!(!tapped(&t, mine), "{name}");
        assert!(tapped(&t, p1) && tapped(&t, p2), "{name}");
        // Played from hand: a land.
        let f = t.hand(P1, "Forest");
        t.set_step(P1, Step::PrecombatMain);
        t.play_land(P1, f).unwrap();
        let land = t.named_on_battlefield("Forest")[0];
        assert!(tapped(&t, land), "{name}");
    }
}

#[test]
fn phasing_in_isnt_entering_tapped() {
    cr!("702.26d", "614.1c");
    ruling!("Frozen Aether", "Does not affect cards that phase in.");
    ruling!("Kismet", "Does not affect cards that phase in.");
    for name in ["Frozen Aether", "Kismet"] {
        let mut t = TestGame::new(2);
        let b = t.battlefield(P1, "Grizzly Bears");
        mtg_engine::keyword_impls::phase_out(&mut t.g, vec![b]);
        t.battlefield(P0, name);
        mtg_engine::keyword_impls::phase_in(&mut t.g, b);
        t.g.recompute();
        assert!(!tapped(&t, b), "{name}");
    }
}

#[test]
fn misers_cage_affects_each_opponent() {
    cr!("503.1a", "603.4");
    ruling!("Misers' Cage", "It affects all opponents.");
    supported("Misers' Cage");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Misers' Cage");
    for p in [P1, P2] {
        for _ in 0..6 {
            t.hand(p, "Grizzly Bears");
        }
    }
    t.advance_to(P2, Step::Draw);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P2), 18);
    assert_eq!(t.life(P0), 20);
}

// --- Mana Short ---------------------------------------------------------------------------

#[test]
fn mana_short_after_spells_were_paid_for() {
    cr!("106.4", "601.2h", "608.1");
    ruling!(
        "Mana Short",
        "If you play Mana Short in response to a spell, it will have no effect on that spell since the mana has already been paid."
    );
    ruling!(
        "Mana Short",
        "Since this is an instant, your opponent may use instants in response to the casting of this spell."
    );
    ruling!("Mana Short", "It even taps lands that do not produce mana.");
    supported("Mana Short");
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Mana Short");
    let ms = t.hand(P0, "Mana Short");
    // P1 has cast Lightning Bolt at P0; P0 responds with Mana Short.
    let m1 = t.battlefield(P1, "Mountain");
    let m2 = t.battlefield(P1, "Mountain");
    let m3 = t.battlefield(P1, "Mountain");
    let wilds = t.battlefield(P1, "Evolving Wilds");
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.cast(P0, ms).target(P1).go();
    // P1 responds with another instant and floats mana.
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(P0).go();
    assert!(tap_for_mana(&mut t, P1, m3, "Add"));
    t.resolve_all();
    assert_eq!(t.life(P0), 15, "both burn spells resolved");
    assert!([m1, m2, m3, wilds].iter().all(|l| tapped(&t, *l)));
    assert_eq!(t.g.player(P1).mana_pool.total(), 0);
}
