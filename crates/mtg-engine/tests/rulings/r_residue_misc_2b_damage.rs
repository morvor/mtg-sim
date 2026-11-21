//! Item `rulings-residue-misc-2`, pass B — damage: replacement and prevention order (CR
//! 616.1), shields that must exist before the damage (615.3), regeneration of
//! unpreventable damage (701.19), divided and evenly divided damage (601.2d, 608.2c), new
//! targets that keep the number and division (115.7), and "blocked" bonuses (509.3c).

use crate::r_residue_misc_2b_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- Eye for an Eye -----------------------------------------------------------------------------

#[test]
fn eye_for_an_eye_must_resolve_before_the_damage() {
    cr!("615.3", "608.2");
    ruling!(
        "Eye for an Eye",
        "Eye for an Eye must resolve before damage would be dealt to you in order to affect that damage."
    );
    // Eye for an Eye cast first, the Bolt in response (on top): the Bolt resolves first,
    // its damage isn't affected.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let eye = t.hand(P0, "Eye for an Eye");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    // P0 chooses the Bolt card as the source (it's a spell by then).
    let eye_spell = t.cast(P0, eye).go();
    let _ = eye_spell;
    let b = t.cast(P1, bolt).target(pl(P0)).go();
    t.answer_choose(P0, &[obj(b)]);
    t.resolve();
    assert_eq!(t.life(P0), 17);
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "{}", t.dump_log());
}

// --- Collective Inferno --------------------------------------------------------------------------

#[test]
fn collective_inferno_affected_player_orders_doubling_and_prevention() {
    cr!("616.1", "616.1e", "615.1a");
    ruling!(
        "Collective Inferno",
        "If another effect (or effects) modifies how much damage a source you control of the chosen type would deal (for example, by preventing some of that damage), the player being dealt damage or the controller of the permanent being dealt damage chooses the order in which any such effects, including the effect of Collective Inferno's last ability, apply. If all of the damage is prevented before the effect of Collective Inferno's last ability would apply, its effect no longer applies."
    );
    supported("Collective Inferno");
    supported("Healing Salve");
    // P0's Goblin source deals 2 damage to P1, who has a Healing Salve shield of 3.
    // P1 applies the prevention first: all 2 prevented, the doubling no longer applies.
    // Doubling first: 4, 3 prevented, 1 dealt.
    for (prevent_first, life) in [(true, 20), (false, 19)] {
        let mut t = TestGame::new(2);
        choose_creature_type(&mut t, P0, "Goblin");
        t.enter(P0, "Collective Inferno");
        let gob = t.battlefield(P0, "Raging Goblin");
        assert!(t.obj_now(gob).chars.has_subtype("Goblin"));
        t.lands(P1, "Plains", 1);
        let salve = t.hand(P1, "Healing Salve");
        t.cast(P1, salve).modes(&[1]).target(pl(P1)).go();
        t.resolve_all();
        let from = t.asked().len();
        t.answer(
            P1,
            DecisionKind::Replacement,
            Answer::Index(if prevent_first { 1 } else { 0 }),
        );
        damage_to(&mut t, gob, 2, pl(P1));
        let asked: Vec<(PlayerId, Decision)> = asked_since(&t, from);
        let chooser = asked.iter().find_map(|(p, d)| match d {
            Decision::ChooseReplacement { .. } => Some(*p),
            _ => None,
        });
        assert_eq!(chooser, Some(P1), "{asked:?}");
        assert_eq!(
            t.life(P1),
            life,
            "prevent first: {prevent_first}; {asked:?}"
        );
    }
}

// --- Lava Burst ----------------------------------------------------------------------------------

#[test]
fn lava_burst_damage_cant_be_prevented_but_the_creature_can_regenerate() {
    cr!("701.19a", "614.8");
    ruling!(
        "Lava Burst",
        "A creature dealt damage by Lava Burst may be regenerated."
    );
    supported("Lava Burst");
    let mut t = TestGame::new(2);
    let sk = t.battlefield(P1, "Drudge Skeletons");
    t.lands(P1, "Swamp", 1);
    activate_containing(&mut t, P1, sk, "Regenerate").expect("regenerate");
    t.resolve_all();
    t.lands(P0, "Mountain", 4);
    let lb = t.hand(P0, "Lava Burst");
    t.cast(P0, lb).x(3).target(obj(sk)).go();
    t.resolve_all();
    assert!(t.on_battlefield(sk), "{}", t.dump_log());
    assert!(t.obj_now(sk).tapped);
    assert_eq!(dmg(&t, sk), 0);
}

// --- Dwarven Catapult ------------------------------------------------------------------------------

#[test]
fn dwarven_catapult_divides_evenly_by_the_creatures_there_as_it_resolves() {
    cr!("608.2c", "107.1a");
    ruling!(
        "Dwarven Catapult",
        "Count the number of creatures on resolution of the spell and divide X by that number at that time. If the number of creatures changes before resolution, the damage done may be different than what you expected."
    );
    ruling!(
        "Dwarven Catapult",
        "For example, if X is 5 and the targeted opponent controls three creatures, Dwarven Catapult deals 1 damage to each of those creatures."
    );
    ruling!(
        "Dwarven Catapult",
        "If the targeted opponent controls more than X creatures, no damage will be dealt to any of them."
    );
    supported("Dwarven Catapult");
    // (x, creatures, killed in response, damage to each survivor)
    for (x, n, kill_one, each) in [(5, 3, false, 1), (5, 3, true, 2), (2, 3, false, 0)] {
        let mut t = TestGame::new(2);
        let cs: Vec<ObjectId> = (0..n).map(|_| t.battlefield(P1, "Craw Wurm")).collect();
        t.lands(P0, "Mountain", 1);
        t.lands(P0, "Wastes", x);
        let dc = t.hand(P0, "Dwarven Catapult");
        t.cast(P0, dc).x(x as i64).target(pl(P1)).go();
        if kill_one {
            kill(&mut t, cs[0]);
        }
        t.resolve_all();
        for c in &cs[usize::from(kill_one)..] {
            assert_eq!(
                dmg(&t, *c),
                each,
                "X={x}, {n} creatures, killed: {kill_one}"
            );
        }
    }
}

// --- Fireball, Impact Resonance, Winter Blast, Cinder Seer -----------------

#[test]
fn fireball_with_zero_targets_deals_no_damage() {
    cr!("601.2c", "115.1");
    ruling!(
        "Fireball",
        "You may cast Fireball with zero targets, regardless of the value chosen for X. If you do so, it will not be a targeted spell, and no damage will actually be dealt when it resolves."
    );
    supported("Fireball");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 4);
    let fb = t.hand(P0, "Fireball");
    t.answer_targets(P0, &[]);
    let spell = t.cast(P0, fb).x(3).go();
    let targets = t.g.obj(spell).stack.as_ref().unwrap().chosen[0]
        .targets
        .clone();
    assert!(targets.iter().all(|s| s.is_empty()), "{targets:?}");
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    assert_eq!(dmg(&t, bears), 0);
    assert!(t.in_graveyard(P0, "Fireball"));
}

#[test]
fn impact_resonance_x_and_division_are_locked_in_as_its_cast() {
    cr!("601.2d", "107.3");
    ruling!(
        "Impact Resonance",
        "You determine the value for X and announce how damage will be divided as you cast Impact Resonance. Each chosen target must receive at least 1 damage. Once the value of X is determined, it won’t change, even if a source deals a greater amount of damage before Impact Resonance resolves."
    );
    supported("Impact Resonance");
    let mut t = TestGame::new(2);
    let src = t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Craw Wurm");
    damage_to(&mut t, src, 3, pl(P1));
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 2);
    let ir = t.hand(P0, "Impact Resonance");
    divide(&mut t, P0, &[2, 1]);
    t.cast(P0, ir).targets(&[obj(a), obj(b)]).go();
    // A bigger hit before it resolves doesn't change X.
    damage_to(&mut t, src, 5, pl(P1));
    t.resolve_all();
    assert_eq!((dmg(&t, a), dmg(&t, b)), (2, 1), "{}", t.dump_log());
}

#[test]
fn winter_blast_can_tap_tapped_creatures_and_still_hits_flyers() {
    cr!("115.1", "701.26a");
    ruling!(
        "Winter Blast",
        "Winter Blast can target tapped creatures. If they have flying, the second part of the effect will still affect them."
    );
    supported("Winter Blast");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(angel);
    t.lands(P0, "Forest", 3);
    let wb = t.hand(P0, "Winter Blast");
    t.cast(P0, wb).x(2).targets(&[obj(angel), obj(bears)]).go();
    t.resolve_all();
    assert_eq!(dmg(&t, angel), 2, "{}", t.dump_log());
    assert_eq!(dmg(&t, bears), 0);
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn cinder_seer_can_reveal_nothing_and_deal_nothing() {
    cr!("701.20a", "120.8");
    ruling!(
        "Cinder Seer",
        "You can reveal zero cards and deal zero damage."
    );
    supported("Cinder Seer");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let cs = t.battlefield(P0, "Cinder Seer");
    t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 3);
    t.answer_choose(P0, &[]);
    t.activate(P0, cs, 0, &[pl(P1)]).expect("activated");
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P0), 1);
}

// --- Boltbender ------------------------------------------------------------------------------------

#[test]
fn boltbender_keeps_the_number_of_targets_and_the_division() {
    cr!("115.7d", "601.2d");
    ruling!(
        "Boltbender",
        "If a spell or ability has a variable number of targets, you can’t change how many targets it has."
    );
    ruling!(
        "Boltbender",
        "If a spell or ability has damage divided, the division can’t be changed, although the targets receiving that damage still can. The same is true of spells and abilities that distribute counters."
    );
    supported("Forked Bolt");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    t.lands(P1, "Mountain", 1);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Hill Giant");
    let card = t.hand(P0, "Boltbender");
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, card)
        .method(CastMethod::FaceDown(
            mtg_engine::keywords::KeywordKind::Disguise,
        ))
        .go();
    t.resolve();
    let bb = t.g.current(card);
    t.advance_to(P1, Step::PrecombatMain);
    // P1's Forked Bolt: 1 to the Elves, 1 to P0's Craw Wurm.
    let fbolt = t.hand(P1, "Forked Bolt");
    divide(&mut t, P1, &[1, 1]);
    let spell = t.cast(P1, fbolt).targets(&[obj(elves), obj(wurm)]).go();
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(spell)]);
    t.answer_targets(P0, &[obj(a)]);
    t.answer_targets(P0, &[obj(b)]);
    t.g.turn.priority = Some(P0);
    t.g.take_action(
        P0,
        mtg_engine::decision::Action::Special(mtg_engine::decision::SpecialAction::TurnFaceUp {
            obj: bb,
        }),
    );
    t.settle();
    t.resolve();
    // Each of the two targets may be changed, one for one: still two targets.
    let new_targets: Vec<(u32, u32)> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { min, max, text, .. } if text.starts_with("New target") => {
                Some((min, max))
            }
            _ => None,
        })
        .collect();
    assert_eq!(new_targets, vec![(0, 1), (0, 1)]);
    let chosen = t.g.obj(spell).stack.as_ref().unwrap().chosen[0].clone();
    assert_eq!(chosen.divided.concat(), vec![1, 1]);
    t.resolve_all();
    assert_eq!((dmg(&t, a), dmg(&t, b)), (1, 1), "{}", t.dump_log());
    assert!(t.on_battlefield(elves));
}

// --- Spined Sliver, Dreams of the Dead -----------------------------------------------------------

#[test]
fn two_spined_slivers_give_two_bonuses_per_blocker() {
    cr!("509.3c", "113.2c");
    ruling!(
        "Spined Sliver",
        "If two Spined Slivers are on the battlefield then each blocked sliver gets +2/+2 for each creature blocking it."
    );
    supported("Spined Sliver");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let s1 = t.battlefield(P0, "Spined Sliver");
    t.battlefield(P0, "Spined Sliver");
    let a = t.battlefield(P1, "Wall of Stone");
    let b = t.battlefield(P1, "Wall of Stone");
    attack_with(&mut t, &[(s1, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(a, s1), (b, s1)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.resolve_all();
    // 2/2 base: +1/+1 per blocker from each Spined Sliver, twice: 6/6.
    assert_eq!(t.pt(s1), (6, 6), "{}", t.dump_log());
}

#[test]
fn dreams_of_the_dead_exile_is_a_replacement_effect() {
    cr!("614.1a", "614.6", "700.4");
    ruling!("Dreams of the Dead", "This is a replacement effect.");
    supported("Dreams of the Dead");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let dd = t.battlefield(P0, "Dreams of the Dead");
    t.battlefield(P1, "Blood Artist");
    let sk = t.graveyard(P0, "Drudge Skeletons");
    t.lands(P0, "Island", 2);
    t.activate(P0, dd, 0, &[obj(sk)]).expect("Dreams");
    t.resolve_all();
    let sk = t.named_on_battlefield("Drudge Skeletons")[0];
    destroy(&mut t, sk);
    // Exiled instead of dying: Blood Artist doesn't trigger.
    assert!(t.in_exile("Drudge Skeletons"));
    assert!(triggers_of(&t, "Blood Artist").is_empty());
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}
