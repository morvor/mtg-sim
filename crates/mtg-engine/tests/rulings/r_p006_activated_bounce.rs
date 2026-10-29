//! Rulings batch P006 — activated abilities that return their source to hand (they can be
//! activated only while it's on the battlefield, CR 602.1, 113.6; only by its controller,
//! CR 602.2; repeatedly in response to themselves, CR 602.2b), choices made on resolution
//! (CR 608.2d), Aetherplasm putting a creature onto the battlefield blocking (CR 509.1,
//! 509.4), Zurgo and Ojutai's combat damage trigger (CR 510.4), Jadzi's reveal-and-cast
//! trigger (CR 608.2g, 305.4, 712.12), Cryptic Coat and face-down instants (CR 701.58g).

use crate::r_p006_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, supported, watch};
use crate::r_s02_common::{can_activate, can_play_land, destroy};
use crate::r_s06_common::activate_containing;
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn return_abilities_work_only_on_the_battlefield() {
    cr!("602.1", "113.6");
    ruling!(
        "Skywing Aven",
        "Skywing Aven’s last ability can be activated only while it’s on the battlefield."
    );
    ruling!(
        "Fae of Wishes // Granted",
        "The activated ability of Fae of Wishes can be activated only while Fae of Wishes is on the battlefield."
    );
    for name in ["Skywing Aven", "Fae of Wishes // Granted"] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 2);
        t.hand(P0, "Grizzly Bears");
        t.hand(P0, "Grizzly Bears");
        let in_hand = t.hand(P0, name);
        assert!(!can_activate(&mut t, P0, in_hand), "{name} from hand");
        let live = t.battlefield(P0, name);
        assert!(can_activate(&mut t, P0, live), "{name} on the battlefield");
    }
}

#[test]
fn skywing_aven_can_respond_to_itself() {
    cr!("117.1b", "608.2b");
    ruling!(
        "Skywing Aven",
        "You can activate Skywing Aven’s ability, then activate it again before that ability resolves. You may do this as many times as you’d like before resolving any of those abilities."
    );
    let mut t = TestGame::new(2);
    let aven = t.battlefield(P0, "Skywing Aven");
    for _ in 0..3 {
        t.hand(P0, "Grizzly Bears");
    }
    for _ in 0..3 {
        t.activate(P0, aven, 0, &[]).unwrap();
    }
    assert_eq!(t.stack_len(), 3);
    assert_eq!(t.hand_size(P0), 0, "three cards discarded");
    t.resolve();
    assert!(in_hand(&t, aven));
    t.resolve_all();
    assert!(in_hand(&t, aven), "the others do nothing");
}

#[test]
fn lantern_spirit_only_its_controller_activates() {
    cr!("602.2");
    ruling!(
        "Lantern Spirit",
        "Only Lantern Spirit’s controller may activate its ability."
    );
    let mut t = TestGame::new(2);
    let spirit = t.battlefield(P0, "Lantern Spirit");
    t.lands(P0, "Island", 1);
    t.lands(P1, "Island", 1);
    assert!(!can_activate(&mut t, P1, spirit));
    assert!(t.activate(P1, spirit, 0, &[]).is_err());
    assert!(can_activate(&mut t, P0, spirit));
}

#[test]
fn rootha_needs_a_target() {
    cr!("602.2b", "601.2c", "115.1");
    ruling!(
        "Rootha, Mercurial Artist",
        "Rootha, Mercurial Artist's ability must have a target to be activated. You can't activate it without a target in order to return Rootha to your hand."
    );
    let mut t = TestGame::new(2);
    let rootha = t.battlefield(P0, "Rootha, Mercurial Artist");
    t.lands(P0, "Wastes", 2);
    assert!(!can_activate(&mut t, P0, rootha), "no spell");
    assert!(t.activate(P0, rootha, 0, &[]).is_err());
    assert!(t.on_battlefield(rootha));
    // An opponent's spell isn't a legal target either.
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(pl(P0)).go();
    assert!(!can_activate(&mut t, P0, rootha), "only opponents' spells");
    // Its own spell is.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let bolt = t.cast(P0, bolt).target(pl(P1)).go();
    assert!(can_activate(&mut t, P0, rootha));
    t.activate(P0, rootha, 0, &[obj(bolt)]).unwrap();
    assert!(in_hand(&t, rootha));
}

#[test]
fn brightling_chooses_on_resolution() {
    cr!("608.2d", "602.2b");
    ruling!(
        "Brightling",
        "If you activate Brightling's last ability, you don't choose whether Brightling gets +1/-1 or -1/+1 until that ability resolves."
    );
    let is_choice = |d: &Decision| {
        matches!(
            d,
            Decision::ChooseOption { .. } | Decision::ChooseModes { .. }
        )
    };
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Brightling");
    t.lands(P0, "Wastes", 1);
    let from = mark(&t);
    activate_containing(&mut t, P0, b, "+1/-1").unwrap();
    assert_eq!(
        t.asked()[from..]
            .iter()
            .filter(|(_, d)| is_choice(d))
            .count(),
        0,
        "nothing chosen on activation"
    );
    let from = mark(&t);
    t.answer(P0, DecisionKind::Any, Answer::Index(1));
    t.resolve();
    assert_eq!(
        t.asked()[from..]
            .iter()
            .filter(|(_, d)| is_choice(d))
            .count(),
        1
    );
    assert_eq!(t.pt(b), (2, 4));
}

#[test]
fn metathran_aerostat_can_respond_to_itself() {
    cr!("117.1b", "608.2c");
    ruling!(
        "Metathran Aerostat",
        "You can activate the ability in response to itself and get more than one creature card onto the battlefield before this returns to its owner’s hand."
    );
    supported("Metathran Aerostat");
    let mut t = TestGame::new(2);
    let aerostat = t.battlefield(P0, "Metathran Aerostat");
    t.lands(P0, "Island", 6);
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Grizzly Bears");
    for _ in 0..2 {
        t.answer(P0, DecisionKind::X, Answer::Number(2));
        t.activate(P0, aerostat, 0, &[]).unwrap();
    }
    assert_eq!(t.stack_len(), 2);
    t.answer_choose(P0, &[obj(b)]);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.on_battlefield(b));
    assert!(in_hand(&t, aerostat));
    t.answer_choose(P0, &[obj(a)]);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(
        t.on_battlefield(a),
        "the second activation still puts one in"
    );
}

#[test]
fn palinchron_untaps_zero_to_seven_lands() {
    cr!("608.2d");
    ruling!("Palinchron", "You can untap 0 to 7 lands.");
    for n in [0usize, 4, 7] {
        let mut t = TestGame::new(2);
        let lands = t.lands(P0, "Island", 9);
        for l in &lands {
            t.g.tap(*l);
        }
        let pal = t.enter(P0, "Palinchron");
        t.settle();
        let from = mark(&t);
        let pick: Vec<Entity> = lands[..n].iter().map(|l| obj(*l)).collect();
        t.answer_choose(P0, &pick);
        t.resolve_all();
        let c = choices_since(&t, P0, from);
        assert_eq!((c[0].1, c[0].2), (0, 7), "up to seven");
        let untapped = lands.iter().filter(|l| !t.obj_now(**l).tapped).count();
        assert_eq!(untapped, n);
        assert!(t.on_battlefield(pal));
    }
}

#[test]
fn wingspan_stride_isnt_returned_after_its_creature_leaves() {
    cr!("704.5m", "608.2b");
    ruling!(
        "Wingspan Stride",
        "If the creature this Aura is attached to leaves the battlefield or stops being a creature before the activated ability resolves, Wingspan Stride will be put into its owner’s graveyard as a state-based action before that ability resolves, and it won’t be returned to its owner’s hand."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = crate::r_s06_common::attach_new(&mut t, P0, "Wingspan Stride", obj(bears));
    t.lands(P0, "Island", 3);
    t.activate(P0, aura, 0, &[]).unwrap();
    destroy(&mut t, bears);
    assert!(in_graveyard(&t, aura), "SBA before the ability resolves");
    t.resolve_all();
    assert!(in_graveyard(&t, aura));
}

#[test]
fn conviction_must_be_returned_before_combat_damage() {
    cr!("510.2", "510.3");
    ruling!(
        "Conviction",
        "Players don't have priority to cast spells and activate abilities between combat damage being assigned and being dealt."
    );
    // At the first priority of the combat damage step, damage has already been dealt.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = crate::r_s06_common::attach_new(&mut t, P0, "Conviction", obj(bears));
    let giant = t.battlefield(P1, "Hill Giant");
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::Priority { .. }),
        |g| {
            (g.turn.step == Step::CombatDamage).then(|| {
                g.battlefield
                    .iter()
                    .map(|id| g.obj(*id).damage)
                    .sum::<u32>()
            })
        },
    );
    attack_with(&mut t, &[(bears, pl(P1))]);
    block_and_finish(&mut t, P1, &[(giant, bears)]);
    let first = seen.lock().unwrap().iter().flatten().next().copied();
    assert_eq!(
        first,
        Some(3),
        "the Bears already damaged (and the Giant dead)"
    );
    assert!(!t.on_battlefield(giant));
    assert!(t.on_battlefield(bears), "a 3/5 survives");
    assert!(t.on_battlefield(aura));
    // Returned during the declare blockers step: the Bears are a 2/2 when damage is dealt.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = crate::r_s06_common::attach_new(&mut t, P0, "Conviction", obj(bears));
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(bears, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(giant, bears)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.lands(P0, "Plains", 1);
    t.activate(P0, aura, 0, &[]).unwrap();
    t.resolve_all();
    assert!(in_hand(&t, aura));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(in_graveyard(&t, bears));
}

#[test]
fn cycle_of_life_targets_only_a_creature_cast_this_turn() {
    cr!("601.2c", "115.1");
    ruling!(
        "Cycle of Life",
        "This does not work on creatures which are put onto the battlefield by an effect instead of by being cast."
    );
    let mut t = TestGame::new(2);
    let cycle = t.battlefield(P0, "Cycle of Life");
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    let giant = t.enter(P0, "Hill Giant");
    let from = mark(&t);
    t.answer_targets(P0, &[obj(giant)]);
    let r = t.activate(P0, cycle, 0, &[]);
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    let bears_now = t.g.current(bears);
    assert!(!cands.is_empty());
    assert!(cands[0].contains(&obj(bears_now)), "the cast Bears");
    assert!(
        !cands[0].contains(&obj(giant)),
        "not the Giant put onto the battlefield"
    );
    if r.is_ok() {
        assert!(!targets_of(&t, top(&t)).contains(&obj(giant)));
    }
}

#[test]
fn anthroplasm_survives_its_own_ability() {
    cr!("704.3", "608.2");
    ruling!(
        "Anthroplasm",
        "While Anthroplasm does become 0/0 for a moment while its ability resolves, state-based actions are not checked until the ability has finished resolving."
    );
    let mut t = TestGame::new(2);
    let a = t.enter(P0, "Anthroplasm");
    old(&mut t, a);
    assert_eq!(t.counters(a, counters::PLUS1), 2);
    t.lands(P0, "Wastes", 3);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.activate(P0, a, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(a));
    assert_eq!(t.counters(a, counters::PLUS1), 3);
    assert_eq!(t.pt(a), (3, 3));
}

#[test]
fn cryptic_coat_has_no_equip_ability() {
    cr!("301.5", "702.6a", "701.58a");
    ruling!(
        "Cryptic Coat",
        "Cryptic Coat has no equip ability. Without assistance from other cards, there's no way to attach it to a creature other than with its first triggered ability."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let coat = t.battlefield(P0, "Cryptic Coat");
    let abilities: Vec<String> = t
        .obj_now(coat)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .map(|a| a.text.to_string())
        .collect();
    assert_eq!(abilities.len(), 1, "{abilities:?}");
    assert!(abilities[0].contains("Return"));
    assert_eq!(t.obj_now(coat).attached_to, None);
    let _ = bears;
    // Its enters ability cloaks and attaches.
    let mut t = TestGame::new(2);
    let coat = t.enter(P0, "Cryptic Coat");
    t.resolve_all();
    let host = t.obj_now(coat).attached_to.and_then(|e| e.object());
    assert!(host.is_some_and(|h| t.obj_now(h).face_down));
}

#[test]
fn a_face_down_instant_stays_face_down_when_turned_up() {
    cr!("701.58g");
    ruling!(
        "Arno Dorian",
        "If something tries to turn a face-down instant or sorcery card on the battlefield face up, reveal that card to show all players it’s an instant or sorcery card. The permanent remains on the battlefield face down. Abilities that trigger when a permanent turns face up won’t trigger, because even though you revealed the card, it never turned face up."
    );
    supported("Break Open");
    supported("Trail of Mystery");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Trail of Mystery");
    t.library_top(P0, "Lightning Bolt");
    let coat = t.enter(P0, "Cryptic Coat");
    t.resolve_all();
    let cloaked = t
        .obj_now(coat)
        .attached_to
        .and_then(|e| e.object())
        .unwrap();
    assert!(t.obj_now(cloaked).face_down);
    t.lands(P1, "Mountain", 4);
    let br = t.hand(P1, "Break Open");
    t.cast(P1, br).target(obj(cloaked)).go();
    // P1 pays the ward {2} of the cloaked permanent.
    t.answer_yes(P1, true);
    t.resolve_all();
    assert!(t.on_battlefield(cloaked));
    assert!(t.obj_now(cloaked).face_down, "remains face down");
    assert!(crate::r_s11_common::revealed_this_turn(&t).contains(&t.g.current(cloaked)));
    assert!(!crate::r_s11_common::turned_face_up_this_turn(&t));
    t.settle();
    assert_eq!(t.stack_len(), 0, "Trail of Mystery doesn't trigger");
}

// --- Aetherplasm --------------------------------------------------------------------

/// P0 attacks with `attacker`; P1 blocks it with Aetherplasm, whose trigger is on the
/// stack (P1 holds `hand` in hand).
fn aetherplasm_blocks(attacker: &str, hand: &str) -> (TestGame, ObjectId, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, attacker);
    let plasm = t.battlefield(P1, "Aetherplasm");
    let card = t.hand(P1, hand);
    attack_with(&mut t, &[(a, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(plasm, a)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.settle();
    assert_eq!(triggers_from(&t, plasm), 1);
    (t, a, plasm, card)
}

fn blockers_of(t: &TestGame, attacker: ObjectId) -> Vec<ObjectId> {
    let c = t.g.combat.as_ref().expect("combat");
    c.blockers
        .iter()
        .filter(|b| b.blocking.contains(&t.g.current(attacker)))
        .map(|b| b.id)
        .collect()
}

#[test]
fn aetherplasm_swaps_in_a_blocker_before_damage() {
    cr!("509.4", "510.1");
    ruling!(
        "Aetherplasm",
        "Aetherplasm will return to your hand before any combat damage is assigned."
    );
    ruling!(
        "Aetherplasm",
        "Although the creature that enters the battlefield is blocking, it was never declared as a blocking creature for the purposes of abilities that trigger whenever a creature blocks"
    );
    supported("Aetherplasm");
    // The creature put in is Wall of Junk: "When this creature blocks" doesn't trigger.
    let (mut t, giant, plasm, wall) = aetherplasm_blocks("Hill Giant", "Wall of Junk");
    t.answer_yes(P1, true);
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(wall)]);
    t.resolve_all();
    assert!(in_hand(&t, plasm));
    assert!(t.on_battlefield(wall));
    let wall = t.g.current(wall);
    assert_eq!(blockers_of(&t, giant), vec![wall]);
    assert_eq!(triggers_from(&t, wall), 0, "it never blocked");
    t.advance_to(P0, Step::EndOfCombat);
    t.settle();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.obj_now(wall).damage, 3);
    assert_eq!(triggers_from(&t, wall), 0, "no return at end of combat");
    t.resolve_all();
    assert!(t.on_battlefield(wall));
}

#[test]
fn aetherplasm_returned_alone_leaves_the_attacker_blocked() {
    cr!("509.1h", "506.4");
    ruling!(
        "Aetherplasm",
        "If you want, you can return Aetherplasm to your hand and not put anything onto the battlefield. The attacking creature will still be considered blocked"
    );
    let (mut t, giant, plasm, _) = aetherplasm_blocks("Hill Giant", "Grizzly Bears");
    t.answer_yes(P1, true);
    t.answer_yes(P1, false);
    t.answer_choose(P1, &[]);
    t.resolve_all();
    assert!(in_hand(&t, plasm));
    assert!(t.g.combat.as_ref().unwrap().is_blocked(t.g.current(giant)));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn aetherplasm_puts_in_a_creature_that_couldnt_block() {
    cr!("509.4b");
    ruling!(
        "Aetherplasm",
        "The creature you put onto the battlefield from your hand is blocking the attacking creature, even if the block couldn't legally be declared"
    );
    let (mut t, giant, plasm, craven) = aetherplasm_blocks("Hill Giant", "Craven Giant");
    t.answer_yes(P1, true);
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(craven)]);
    t.resolve_all();
    assert!(in_hand(&t, plasm));
    let craven = t.g.current(craven);
    assert_eq!(
        blockers_of(&t, giant),
        vec![craven],
        "Craven Giant can't block"
    );
    t.advance_to(P0, Step::EndOfCombat);
    assert!(in_graveyard(&t, craven));
    assert!(in_graveyard(&t, giant), "4 damage from Craven Giant");
}

#[test]
fn aetherplasm_can_put_itself_back_but_deathgazer_sees_the_new_blocker() {
    cr!("400.7", "509.4", "509.3d");
    interp!("put-onto-battlefield-blocking-retriggers-becomes-blocked-by");
    // Aetherplasm's ruling "The creature you put onto the battlefield may be the
    // Aetherplasm you just returned to your hand. This allows it to dodge abilities like
    // Deathgazer's." is superseded by CR 509.3d (exempt, rulings-P006.tsv): the first
    // sentence holds, the dodge doesn't.
    supported("Deathgazer");
    let (mut t, gazer, plasm, _) = aetherplasm_blocks("Deathgazer", "Island");
    assert_eq!(
        triggers_from(&t, gazer),
        1,
        "Deathgazer: becomes blocked by it"
    );
    // Aetherplasm's trigger is on top (APNAP): it returns and puts itself back.
    t.answer_yes(P1, true);
    t.answer_yes(P1, true);
    t.answer_choose_moved(P1, &[obj(plasm)]);
    t.resolve();
    assert!(t.on_battlefield(plasm));
    let plasm2 = t.g.current(plasm);
    assert_ne!(plasm2, plasm, "a new object");
    assert_eq!(blockers_of(&t, gazer), vec![plasm2]);
    // CR 509.3d: a creature put onto the battlefield blocking it triggers "becomes blocked
    // by a creature" again, so (unlike the 2005 ruling's claim) the dodge doesn't work.
    assert_eq!(triggers_from(&t, gazer), 2);
    t.lands(P1, "Forest", 1);
    let gg = t.hand(P1, "Giant Growth");
    t.cast(P1, gg).target(obj(plasm2)).go();
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(in_graveyard(&t, plasm2));
}

// --- Zurgo and Ojutai ----------------------------------------------------------------

#[test]
fn zurgo_and_ojutai_triggers_in_each_damage_step() {
    cr!("510.4", "702.7b", "702.4b");
    ruling!(
        "Zurgo and Ojutai",
        "Because creatures with first strike deal combat damage before creatures without first strike, it’s also possible to have the ability trigger twice in the same combat"
    );
    supported("Zurgo and Ojutai");
    // A first striker and a regular Dragon: twice.
    let mut t = TestGame::new(2);
    let zurgo = t.battlefield(P0, "Zurgo and Ojutai");
    let wing = t.battlefield(P0, "Stormwing Dragon");
    old(&mut t, zurgo);
    attack_with(&mut t, &[(zurgo, pl(P1)), (wing, pl(P1))]);
    t.advance_to(P0, Step::FirstStrikeDamage);
    t.settle();
    assert_eq!(triggers_from(&t, zurgo), 1);
    t.resolve_all();
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(triggers_from(&t, zurgo), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 3 - 4);
    // A double striker alone: twice.
    let mut t = TestGame::new(2);
    let zurgo = t.battlefield(P0, "Zurgo and Ojutai");
    let engine = t.battlefield(P0, "Phyrexian Dragon Engine");
    attack_with(&mut t, &[(engine, pl(P1))]);
    t.advance_to(P0, Step::FirstStrikeDamage);
    t.settle();
    assert_eq!(triggers_from(&t, zurgo), 1);
    t.answer_yes(P0, false); // keep the Dragon Engine
    t.resolve_all();
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(triggers_from(&t, zurgo), 1);
}

#[test]
fn zurgo_and_ojutai_chooses_a_dragon_on_resolution() {
    cr!("608.2c", "115.1");
    ruling!(
        "Zurgo and Ojutai",
        "You choose whether to return one of the Dragons that dealt combat damage to the player or battle as the triggered ability is resolving. This doesn’t target any of those Dragons."
    );
    let mut t = TestGame::new(2);
    let zurgo = t.battlefield(P0, "Zurgo and Ojutai");
    old(&mut t, zurgo);
    let wing = t.battlefield(P0, "Stormwing Dragon");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(zurgo, pl(P1)), (bears, pl(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(triggers_from(&t, zurgo), 1);
    assert!(targets_of(&t, top(&t)).is_empty());
    let from = mark(&t);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Entities, Answer::Default);
    t.answer_choose(P0, &[obj(zurgo)]);
    t.resolve_all();
    let returned_choice = choices_since(&t, P0, from)
        .into_iter()
        .find(|(c, _, _)| c.contains(&obj(zurgo)));
    let (cands, _, _) = returned_choice.expect("asked which Dragon on resolution");
    assert!(!cands.contains(&obj(bears)), "only Dragons");
    assert!(
        !cands.contains(&obj(wing)),
        "only Dragons that dealt damage"
    );
    assert!(in_hand(&t, zurgo));
}

// --- Jadzi ---------------------------------------------------------------------------

const JADZI: &str = "Jadzi, Oracle of Arcavios // Journey to the Oracle";

/// P0 controls Jadzi and casts Opt (an instant) during the turn of `active`; the top card
/// of P0's library is `top_card`. Returns after Jadzi's trigger is on the stack.
fn jadzi_reveals(active: PlayerId, top_card: &str) -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    t.set_step(active, Step::PrecombatMain);
    let jadzi = t.battlefield(P0, JADZI);
    let top_ = t.library_top(P0, top_card);
    t.lands(P0, "Island", 2);
    let opt = t.hand(P0, "Opt");
    t.g.turn.priority = Some(P0);
    t.cast(P0, opt).go();
    t.settle();
    assert_eq!(triggers_from(&t, jadzi), 1);
    (t, jadzi, top_)
}

#[test]
fn jadzi_may_decline_and_the_card_stays_on_top() {
    cr!("608.2g", "608.2d");
    ruling!(
        "Jadzi, Oracle of Arcavios // Journey to the Oracle",
        "You may choose not to cast a nonland card revealed by Jadzi’s triggered ability. If you don’t cast it, it stays on top of your library."
    );
    supported(JADZI);
    let (mut t, _, card) = jadzi_reveals(P0, "Hill Giant");
    t.answer_yes(P0, false);
    t.resolve();
    assert_eq!(t.g.player(P0).library.last().copied(), Some(card));
    assert_eq!(t.zone(card), Zone::Library(P0));
}

#[test]
fn jadzi_casts_ignoring_timing() {
    cr!("608.2g", "307.1", "608.2d");
    ruling!(
        "Jadzi, Oracle of Arcavios // Journey to the Oracle",
        "If you’re casting the card you reveal with Jadzi’s ability, you ignore normal timing restrictions."
    );
    // A sorcery during the opponent's turn, with Opt still on the stack.
    let (mut t, _, card) = jadzi_reveals(P1, "Divination");
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.zone(card), Zone::Stack, "Divination cast");
    let before = t.hand_size(P0);
    t.resolve_all();
    assert!(t.hand_size(P0) > before);
}

#[test]
fn jadzi_puts_a_land_onto_the_battlefield_without_playing_it() {
    cr!("305.4", "305.2");
    ruling!(
        "Jadzi, Oracle of Arcavios // Journey to the Oracle",
        "Putting a land onto the battlefield does not count as playing a land for the turn and doesn’t use the normal timing restrictions."
    );
    // During the opponent's turn.
    let (mut t, _, land) = jadzi_reveals(P1, "Forest");
    t.resolve();
    assert!(t.on_battlefield(land));
    // During its own turn: P0 can still play a land.
    let (mut t, _, land) = jadzi_reveals(P0, "Forest");
    t.resolve_all();
    assert!(t.on_battlefield(land));
    assert_eq!(t.g.player(P0).lands_played_this_turn, 0);
    let mountain = t.hand(P0, "Mountain");
    assert!(can_play_land(&mut t, P0, mountain));
}

#[test]
fn jadzi_with_a_modal_double_faced_card() {
    cr!("712.11", "712.14", "608.2g");
    ruling!(
        "Jadzi, Oracle of Arcavios // Journey to the Oracle",
        "If you reveal a modal double-faced card with Jadzi’s triggered ability, and both faces are nonland, you may cast either face by paying {1} rather than its mana cost. If only the back face is a land, you may cast the front face. If both faces are lands, you will put onto the battlefield front face up."
    );
    // Both faces lands: onto the battlefield front face up.
    let (mut t, _, card) = jadzi_reveals(P0, "Hengegate Pathway // Mistgate Pathway");
    t.resolve();
    assert!(t.on_battlefield(card));
    assert_eq!(t.obj_now(card).chars.name, "Hengegate Pathway");
    // Back face a land: the front face can be cast.
    let (mut t, _, card) = jadzi_reveals(P0, "Kazandu Mammoth // Kazandu Valley");
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.zone(card), Zone::Stack);
    t.resolve_all();
    assert_eq!(t.obj_now(card).chars.name, "Kazandu Mammoth");
    // Both faces nonland: either face can be cast; the back face here.
    let (mut t, _, card) = jadzi_reveals(P0, "Plargg, Dean of Chaos // Augusta, Dean of Order");
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.resolve();
    assert_eq!(t.zone(card), Zone::Stack);
    t.resolve_all();
    assert!(t.on_battlefield(card));
    assert_eq!(t.obj_now(card).chars.name, "Augusta, Dean of Order");
}

#[test]
fn jadzi_returns_only_from_the_battlefield() {
    cr!("602.1", "608.2b", "400.7");
    ruling!(
        "Jadzi, Oracle of Arcavios // Journey to the Oracle",
        "You may activate Jadzi’s activated ability only while Jadzi is on the battlefield. If Jadzi is no longer on the battlefield as the ability resolves, it won’t return her to your hand from any other zone."
    );
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Grizzly Bears");
    let in_hand_ = t.hand(P0, JADZI);
    assert!(!can_activate(&mut t, P0, in_hand_));
    let jadzi = t.battlefield(P0, JADZI);
    t.activate(P0, jadzi, 0, &[]).unwrap();
    destroy(&mut t, jadzi);
    t.resolve_all();
    assert!(in_graveyard(&t, jadzi));
}
