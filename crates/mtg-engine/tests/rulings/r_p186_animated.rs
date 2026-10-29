//! Rulings batch P186 — permanents that become creatures and artifact creatures: "until
//! end of combat" animation and delayed effects on a permanent that stops being a
//! creature (CR 611.2, 514.2), summoning sickness of a land that becomes a creature (CR
//! 302.6), layer 7b and timestamps (CR 613.4b, 613.7), attack requirements with costs
//! (CR 508.1d), costs that make marked damage lethal (CR 704.5g), discard replacement
//! effects (CR 614.1, 701.9), mana abilities (CR 605), "up to X" choices, "can't cast"
//! restrictions, choices made on resolution (CR 608.2d), and effects versus costs.

use crate::r_p185_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, can_attack, can_cast};
use crate::r_s04_common::{add_mana, is_creature, next_upkeep};
use crate::r_s06_common::damage;
use crate::r_s09_common::{legal_attack, to_combat};
use crate::r_s20_common::{entered_this_turn, tap_for_mana};
use mtg_engine::ability::{Modification, Value};
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Activates Jade Statue ({2}) for P0, paid from P0's mana pool, and resolves it.
fn animate_statue(t: &mut TestGame, statue: ObjectId) {
    add_mana(t, P0, ManaType::C, 2);
    t.activate(P0, statue, 0, &[]).expect("Jade Statue activation");
    t.resolve_all();
}

#[test]
fn jade_statue_animates_only_during_combat() {
    cr!("602.5d", "506.1");
    ruling!(
        "Jade Statue",
        "It can only make itself into a creature during combat phases and therefore it is difficult to get Auras placed on it."
    );
    supported("Jade Statue");
    let mut t = TestGame::new(2);
    let statue = t.battlefield(P0, "Jade Statue");
    add_mana(&mut t, P0, ManaType::C, 2);
    assert!(!can_activate(&mut t, P0, statue));
    to_combat(&mut t, P0);
    assert!(can_activate(&mut t, P0, statue));
    t.activate(P0, statue, 0, &[]).unwrap();
    t.resolve_all();
    assert!(is_creature(&t, statue));
    assert_eq!(t.pt(statue), (3, 6));
}

#[test]
fn jade_statue_delayed_effect_still_applies_after_it_stops_being_a_creature() {
    cr!("611.2a", "502.3");
    ruling!(
        "Jade Statue",
        "A delayed triggered ability that refers to Jade Statue still affects it even if it's no longer a creature."
    );
    supported("Kashi-Tribe Warriors");
    let mut t = TestGame::new(2);
    let statue = t.battlefield(P0, "Jade Statue");
    let warriors = t.battlefield(P1, "Kashi-Tribe Warriors");
    to_combat(&mut t, P1);
    attack_with(&mut t, &[(warriors, Entity::Player(P0))]);
    animate_statue(&mut t, statue);
    block_and_finish(&mut t, P0, &[(statue, warriors)]);
    t.resolve_all();
    assert!(crate::r_s24_common::tapped(&t, statue));
    t.advance_to(P1, Step::End);
    assert!(!is_creature(&t, statue));
    t.advance_to(P0, Step::Upkeep);
    assert!(crate::r_s24_common::tapped(&t, statue), "doesn't untap");
    t.advance_to(P0, Step::End);
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    assert!(!crate::r_s24_common::tapped(&t, statue), "untaps a turn later");
}

#[test]
fn jade_statue_animated_by_another_effect_adds_golem() {
    cr!("613.4b", "613.7a", "205.1b");
    ruling!(
        "Jade Statue",
        "If Jade Statue is animated by some other effect, you get an artifact creature with whatever power, toughness, and creature types (if any) are specified by that effect. If you subsequently use Jade Statue's ability during combat, it will become 3/6 and gain the creature type Golem in addition to those creature types until end of combat."
    );
    let mut t = TestGame::new(2);
    let statue = t.battlefield(P0, "Jade Statue");
    crate::r_s26_common::modify_until_eot(
        &mut t,
        statue,
        vec![
            Modification::AddTypes(vec![CardType::Creature]),
            Modification::AddSubtypes(vec!["Elemental".into()]),
            Modification::SetPT(Some(Value::c(1)), Some(Value::c(1))),
        ],
    );
    assert_eq!(t.pt(statue), (1, 1));
    to_combat(&mut t, P0);
    animate_statue(&mut t, statue);
    assert_eq!(t.pt(statue), (3, 6));
    // A "Golem artifact creature" effect replaces its creature types (CR 205.1b; the
    // ruling's "in addition to those creature types" predates that rule).
    let o = t.obj_now(statue);
    assert!(o.chars.has_subtype("Golem") && !o.chars.has_subtype("Elemental"));
    assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
    // After combat it's the other effect's 1/1 again.
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.pt(statue), (1, 1));
    assert!(!t.obj_now(statue).chars.has_subtype("Golem"));
    assert!(t.obj_now(statue).chars.has_subtype("Elemental"));
}

#[test]
fn jade_statue_is_a_creature_for_end_of_combat_triggers() {
    cr!("511.1", "514.2");
    ruling!(
        "Jade Statue",
        "If Jade Statue's ability has been activated, abilities that trigger \"at end of combat\" will see Jade Statue as an artifact creature."
    );
    let mut t = TestGame::new(2);
    let statue = t.battlefield(P0, "Jade Statue");
    let watcher = custom_card(
        "Combat Watcher",
        "Enchantment",
        "{1}",
        None,
        "At end of combat, put a +1/+1 counter on each artifact creature you control.",
    );
    t.custom(P0, watcher, Zone::Battlefield);
    to_combat(&mut t, P0);
    animate_statue(&mut t, statue);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.counters(statue, counters::PLUS1), 1);
}

/// Seven cards in `p`'s graveyard (threshold).
fn threshold(t: &mut TestGame, p: PlayerId) {
    crate::r_p189_common::fill_graveyard(t, p, "Grizzly Bears", 7);
}

/// Activates Nantuko Monastery's threshold ability ({G}{W}) and resolves it.
fn animate_monastery(t: &mut TestGame, m: ObjectId) {
    add_mana(t, P0, ManaType::G, 1);
    add_mana(t, P0, ManaType::W, 1);
    crate::r_s06_common::activate_containing(t, P0, m, "{G}{W}").unwrap().unwrap();
    t.resolve_all();
}

#[test]
fn nantuko_monastery_summoning_sickness_and_abilities() {
    cr!("302.6", "611.3a", "613.1d");
    ruling!(
        "Nantuko Monastery",
        "A noncreature permanent that becomes a creature can attack, and its {T} abilities can be activated, only if its controller has continuously controlled that permanent since the beginning of their most recent turn. It doesn't matter how long the permanent has been a creature. Notably, if you turn Nantuko Monastery into a creature on the turn it entered the battlefield, you won't be able to tap it for mana."
    );
    ruling!(
        "Nantuko Monastery",
        "After it becomes a creature, Nantuko Monastery will still have all its abilities."
    );
    supported("Nantuko Monastery");
    // Played this turn.
    let mut t = TestGame::new(2);
    threshold(&mut t, P0);
    let m = entered_this_turn(&mut t, P0, "Nantuko Monastery");
    assert!(tap_for_mana(&mut t, P0, m, "{C}"), "it can tap for mana as a land");
    let mut t = TestGame::new(2);
    threshold(&mut t, P0);
    let m = entered_this_turn(&mut t, P0, "Nantuko Monastery");
    animate_monastery(&mut t, m);
    assert!(is_creature(&t, m));
    assert!(!tap_for_mana(&mut t, P0, m, "{C}"));
    to_combat(&mut t, P0);
    assert!(!can_attack(&mut t, m));
    // Controlled since the turn began: it attacks and taps for mana, first strike too.
    let mut t = TestGame::new(2);
    threshold(&mut t, P0);
    let m = t.battlefield(P0, "Nantuko Monastery");
    animate_monastery(&mut t, m);
    assert_eq!(t.pt(m), (4, 4));
    assert!(t.obj_now(m).is(CardType::Land));
    assert!(crate::r_s06_common::has_kw(&t, m, mtg_engine::keywords::KeywordKind::FirstStrike));
    assert!(tap_for_mana(&mut t, P0, m, "{C}"));
    let mut t = TestGame::new(2);
    threshold(&mut t, P0);
    let m = t.battlefield(P0, "Nantuko Monastery");
    animate_monastery(&mut t, m);
    to_combat(&mut t, P0);
    assert!(can_attack(&mut t, m));
}

#[test]
fn nantuko_monastery_stays_a_creature_below_threshold() {
    cr!("611.2a", "602.5b");
    ruling!(
        "Nantuko Monastery",
        "Once you activate the last ability, it doesn't matter what happens to the number of cards in your graveyard. Nantuko Monastery will remain a creature that turn even if that number falls below seven."
    );
    let mut t = TestGame::new(2);
    threshold(&mut t, P0);
    let m = t.battlefield(P0, "Nantuko Monastery");
    animate_monastery(&mut t, m);
    crate::r_p189_common::exile_graveyard(&mut t, P0);
    t.g.recompute();
    assert!(is_creature(&t, m));
    assert_eq!(t.pt(m), (4, 4));
}

#[test]
fn scarecrow_prevents_noncombat_damage_from_flyers() {
    cr!("615.1", "702.9a");
    ruling!(
        "Scarecrow",
        "Even prevents non-combat damage from flying creatures."
    );
    supported("Scarecrow");
    let mut t = TestGame::new(2);
    let scarecrow = t.battlefield(P0, "Scarecrow");
    assert!(t.obj_now(scarecrow).is(CardType::Creature));
    let flyer = custom_card(
        "Pinging Bird",
        "Creature — Bird",
        "{2}",
        Some((1, 1)),
        "Flying\n{T}: This creature deals 1 damage to any target.",
    );
    let bird = t.custom(P1, flyer, Zone::Battlefield);
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    add_mana(&mut t, P0, ManaType::C, 6);
    t.activate(P0, scarecrow, 0, &[]).unwrap();
    t.resolve_all();
    t.activate(P1, bird, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "prevented");
    t.activate(P1, sorcerer, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 19, "not a flyer");
}

/// Activates Chimeric Staff with X = `x` (paid from P0's mana pool) and resolves it.
fn staff_x(t: &mut TestGame, staff: ObjectId, x: i64) {
    add_mana(t, P0, ManaType::C, x as u32);
    t.answer(P0, DecisionKind::X, Answer::Number(x));
    t.activate(P0, staff, 0, &[]).unwrap();
    t.resolve_all();
}

#[test]
fn chimeric_staff_reactivation_resets_the_base_and_keeps_damage() {
    cr!("613.4b", "613.4c", "613.7a", "120.6");
    ruling!(
        "Chimeric Staff",
        "If Chimeric Staff is already an artifact creature when you activate its ability, Chimeric Staff stays an artifact creature but the ability resets its creature types."
    );
    supported("Chimeric Staff");
    let mut t = TestGame::new(2);
    let staff = t.battlefield(P0, "Chimeric Staff");
    staff_x(&mut t, staff, 3);
    assert_eq!(t.pt(staff), (3, 3));
    assert!(t.obj_now(staff).chars.has_subtype("Construct"));
    pump(&mut t, staff, -1, -1);
    assert_eq!(t.pt(staff), (2, 2));
    let src = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, src, 1, staff);
    staff_x(&mut t, staff, 4);
    assert_eq!(t.pt(staff), (3, 3), "4/4 applied before the -1/-1");
    assert_eq!(marked(&t, staff), 1, "damage stays");
    assert!(t.obj_now(staff).is(CardType::Artifact));
}

#[test]
fn juggernaut_and_xantcha_dont_have_to_attack_when_unable_or_taxed() {
    cr!("508.1d", "508.1h");
    ruling!(
        "Juggernaut",
        "If Juggernaut can't attack for any reason (such as being tapped or having come under your control that turn), then it doesn't attack. If there's a cost associated with having it attack, you're not forced to pay that cost, so it doesn't have to attack in that case either."
    );
    ruling!(
        "Xantcha, Sleeper Agent",
        "If Xantcha can't attack for any reason (such as being tapped or having come under that player's control that turn), then it doesn't attack. If there's a cost associated with having it attack, the player isn't forced to pay that cost, so it doesn't have to attack in that case either."
    );
    supported("Juggernaut");
    supported("Xantcha, Sleeper Agent");
    // Juggernaut: able and untaxed, it must attack; tapped or taxed, it needn't.
    let mut t = TestGame::new(2);
    let j = t.battlefield(P0, "Juggernaut");
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &[(j, Entity::Player(P1))]));
    t.g.tap(j);
    assert!(legal_attack(&mut t, &[]));
    let mut t2 = TestGame::new(2);
    t2.battlefield(P0, "Juggernaut");
    t2.battlefield(P1, "Ghostly Prison");
    to_combat(&mut t2, P0);
    assert!(legal_attack(&mut t2, &[]), "Ghostly Prison's cost isn't forced");
    // Xantcha (owned by P1, controlled by P0) can attack only P2.
    let mut t = TestGame::new(3);
    let x = t.battlefield(P1, "Xantcha, Sleeper Agent");
    crate::r_s06_common::give_control(&mut t, x, P0);
    let xi = t.g.current(x).0 as usize;
    t.g.objects[xi].summoning_sick = false;
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &[]));
    assert!(!legal_attack(&mut t, &[(x, Entity::Player(P1))]), "not its owner");
    assert!(legal_attack(&mut t, &[(x, Entity::Player(P2))]));
    t.battlefield(P2, "Ghostly Prison");
    assert!(legal_attack(&mut t, &[]));
    // It came under P0's control this turn: it can't attack.
    let mut t = TestGame::new(3);
    let x = t.battlefield(P1, "Xantcha, Sleeper Agent");
    crate::r_s06_common::give_control(&mut t, x, P0);
    to_combat(&mut t, P0);
    assert!(legal_attack(&mut t, &[]));
}

#[test]
fn triskelion_removing_a_counter_can_make_damage_lethal() {
    cr!("704.5g", "602.2b", "608.1");
    ruling!(
        "Triskelion",
        "If Triskelion has damage marked on it, activating its ability may cause that damage to become lethal."
    );
    supported("Triskelion");
    let mut t = TestGame::new(2);
    let tri = t.battlefield(P0, "Triskelion");
    crate::r_s29_common::put_counters(&mut t, tri, counters::PLUS1, 2);
    let src = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, src, 2, tri);
    assert!(t.on_battlefield(tri));
    t.activate(P0, tri, 0, &[Entity::Player(P1)]).unwrap();
    t.settle();
    assert!(!t.on_battlefield(tri), "destroyed before the ability resolves");
    assert!(!can_activate(&mut t, P0, tri));
    t.resolve_all();
    assert_eq!(t.life(P1), 19, "the activation still resolves");
}

#[test]
fn triskelion_in_combat() {
    cr!("510.1", "510.2", "704.3");
    ruling!(
        "Triskelion",
        "If you activate the ability of an attacking or blocking Triskelion during the declare blockers step, it will deal less combat damage as a result. If you wait until the combat damage step, but Triskelion is dealt lethal damage, it’ll be destroyed before you get a chance to activate its ability."
    );
    let mut t = TestGame::new(2);
    let tri = t.battlefield(P0, "Triskelion");
    crate::r_s29_common::put_counters(&mut t, tri, counters::PLUS1, 3);
    let blocker = t.battlefield(P1, "Hill Giant");
    let pinged = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(tri, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(blocker, tri)]));
    t.advance_to(P0, Step::DeclareBlockers);
    t.activate(P0, tri, 0, &[Entity::Object(pinged)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(tri), (3, 3));
    // P0's priority in the combat damage step: the Triskelion is already gone.
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, mtg_engine::decision::Decision::Priority { .. }),
        |g| {
            (
                g.turn.step,
                g.find_in_zone(Zone::Battlefield, "Triskelion").len(),
                g.find_in_zone(Zone::Battlefield, "Hill Giant").len(),
            )
        },
    );
    t.advance_to(P0, Step::EndOfCombat);
    let in_damage_step: Vec<_> = seen
        .lock()
        .unwrap()
        .iter()
        .filter(|(s, _, _)| *s == Step::CombatDamage)
        .map(|(_, a, b)| (*a, *b))
        .collect();
    assert!(!in_damage_step.is_empty());
    assert!(in_damage_step.iter().all(|x| *x == (0, 0)), "{in_damage_step:?}");
}

/// P1 controls Creeping Dread; in P1's upkeep each player discards their only card in
/// hand: P1 `theirs`, P0 Dodecapod. Returns the game and P0's Dodecapod card.
fn dodecapod_discarded_by(controller: PlayerId) -> (TestGame, ObjectId) {
    supported("Dodecapod");
    supported("Creeping Dread");
    let mut t = TestGame::new(2);
    t.battlefield(controller, "Creeping Dread");
    t.battlefield(P0, "Drake Haven");
    let pod = t.hand(P0, "Dodecapod");
    t.hand(P1, "Ornithopter");
    t.lands(P0, "Wastes", 1);
    t.answer_yes(P0, true);
    next_upkeep(&mut t, controller);
    t.resolve_all();
    (t, pod)
}

#[test]
fn dodecapod_discarded_by_an_opponents_ability_enters_and_is_still_discarded() {
    cr!("614.1a", "701.9a", "603.2");
    ruling!(
        "Dodecapod",
        "If a spell or ability an opponent controls causes you to choose a card to discard and you discard Dodecapod, you'll put it onto the battlefield."
    );
    ruling!(
        "Dodecapod",
        "If the spell or ability that caused you to discard Dodecapod refers to \"the discarded card\" or similar, it refers to the Dodecapod on the battlefield."
    );
    ruling!(
        "Dodecapod",
        "If you discard Dodecapod and put it onto the battlefield, you've still discarded it. Abilities that trigger whenever you discard a card will trigger."
    );
    supported("Drake Haven");
    let (t, pod) = dodecapod_discarded_by(P1);
    let on = t.named_on_battlefield("Dodecapod");
    assert_eq!(on.len(), 1);
    assert_eq!(t.g.current(pod), on[0]);
    assert_eq!(t.counters(on[0], counters::PLUS1), 2);
    // Creeping Dread compares the card P0 discarded (an artifact creature) with P1's
    // Ornithopter.
    assert_eq!(t.life(P0), 17);
    // Drake Haven triggered for the discard.
    assert_eq!(with_subtype(&t, P0, "Drake").len(), 1);
    // P0's own Creeping Dread: it goes to the graveyard.
    let (t, _) = dodecapod_discarded_by(P0);
    assert!(t.named_on_battlefield("Dodecapod").is_empty());
    assert!(t.in_graveyard(P0, "Dodecapod"));
}

#[test]
fn mindless_automaton_dies_before_the_card_is_drawn() {
    cr!("704.5g", "602.2b", "608.1");
    ruling!(
        "Mindless Automaton",
        "If removing two +1/+1 counters from Mindless Automaton causes the amount of damage already marked on Mindless Automaton to be equal to or greater than its toughness, it will be put into its owner's graveyard as a state-based action before the ability can be activated again and before the card is drawn."
    );
    supported("Mindless Automaton");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Mindless Automaton");
    crate::r_s29_common::put_counters(&mut t, a, counters::PLUS1, 4);
    let src = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, src, 2, a);
    let hand = t.hand_size(P0);
    crate::r_s06_common::activate_containing(&mut t, P0, a, "Remove two").unwrap().unwrap();
    t.settle();
    assert!(!t.on_battlefield(a));
    assert_eq!(t.hand_size(P0), hand, "not drawn yet");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn transmogrifying_licid_adds_the_artifact_type() {
    cr!("205.1b", "613.1d");
    ruling!(
        "Transmogrifying Licid",
        "The enchanted creature gains the artifact type in addition to any other card types it may already have."
    );
    supported("Transmogrifying Licid");
    let mut t = TestGame::new(2);
    let licid = t.battlefield(P0, "Transmogrifying Licid");
    let bears = t.battlefield(P0, "Grizzly Bears");
    add_mana(&mut t, P0, ManaType::C, 1);
    t.answer_yes(P0, false);
    t.activate(P0, licid, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    let o = t.obj_now(bears);
    assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn metalworker_is_a_mana_ability_and_may_reveal_nothing() {
    cr!("605.1a", "605.3a", "701.20a");
    ruling!(
        "Metalworker",
        "This is a mana ability. It does not go on the stack and can’t be responded to."
    );
    ruling!("Metalworker", "You can reveal zero cards to gain zero mana.");
    supported("Metalworker");
    let mut t = TestGame::new(2);
    let mw = t.battlefield(P0, "Metalworker");
    let a = t.hand(P0, "Ornithopter");
    let b = t.hand(P0, "Juggernaut");
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    assert!(tap_for_mana(&mut t, P0, mw, "Reveal"));
    assert_eq!(t.stack_len(), 0);
    assert_eq!(crate::r_s24_common::pool(&t, P0, ManaType::C), 4);
    let mut t = TestGame::new(2);
    let mw = t.battlefield(P0, "Metalworker");
    t.hand(P0, "Ornithopter");
    t.answer_choose(P0, &[]);
    tap_for_mana(&mut t, P0, mw, "Reveal");
    assert!(t.obj_now(mw).tapped);
    assert_eq!(crate::r_s24_common::pool(&t, P0, ManaType::C), 0);
}

#[test]
fn clockwork_steed_may_get_fewer_than_x_counters() {
    cr!("107.1c", "608.2d");
    ruling!(
        "Clockwork Steed",
        "When Clockwork Steed's last ability resolves, you can choose to put fewer than X +1/+0 counters on it."
    );
    supported("Clockwork Steed");
    let mut t = TestGame::new(2);
    let steed = t.battlefield(P0, "Clockwork Steed");
    t.set_step(P0, Step::Upkeep);
    add_mana(&mut t, P0, ManaType::C, 3);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer(P0, DecisionKind::Number, Answer::Number(1));
    crate::r_s06_common::activate_containing(&mut t, P0, steed, "up to X").unwrap().unwrap();
    t.resolve_all();
    assert_eq!(t.counters(steed, "+1/+0"), 1);
    assert_eq!(t.pt(steed), (1, 3));
}

#[test]
fn steel_golem_and_grid_monitor_stop_only_their_controller_casting_creatures() {
    cr!("101.2", "601.3");
    ruling!("Steel Golem", "Yes, this only affects you.");
    ruling!(
        "Steel Golem",
        "You can put creatures onto the battlefield by means other than casting them."
    );
    ruling!(
        "Grid Monitor",
        "You can still put creature cards onto the battlefield or create creature tokens."
    );
    supported("Steel Golem");
    supported("Grid Monitor");
    for name in ["Steel Golem", "Grid Monitor"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        let mine = t.hand(P0, "Grizzly Bears");
        t.lands(P0, "Forest", 2);
        assert!(!can_cast(&mut t, P0, mine, CastMethod::Normal), "{name}");
        let theirs = t.hand(P1, "Grizzly Bears");
        t.lands(P1, "Forest", 2);
        t.set_step(P1, Step::PrecombatMain);
        assert!(can_cast(&mut t, P1, theirs, CastMethod::Normal), "{name}");
        let e = crate::r_s05_common::enter(&mut t, P0, "Hill Giant");
        assert!(t.on_battlefield(e));
        crate::r_s02_common::create_token(&mut t, P0, "Soldier");
        assert_eq!(creatures(&t, P0).len(), 3, "{name}");
    }
}

#[test]
fn copper_gnomes_chooses_the_artifact_on_resolution() {
    cr!("608.2d", "602.2");
    ruling!("Copper Gnomes", "You choose the artifact card on resolution.");
    supported("Copper Gnomes");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Copper Gnomes");
    add_mana(&mut t, P0, ManaType::C, 4);
    t.activate(P0, g, 0, &[]).unwrap();
    // The artifact card arrives in hand after activation.
    let card = t.hand(P0, "Juggernaut");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(card)]);
    t.resolve_all();
    assert!(t.on_battlefield(card));
}

#[test]
fn chimeric_idol_taps_lands_as_its_effect() {
    cr!("118.3", "608.2");
    ruling!(
        "Chimeric Idol",
        "Your lands are tapped as part of the effect, not part of the cost."
    );
    supported("Chimeric Idol");
    let mut t = TestGame::new(2);
    let idol = t.battlefield(P0, "Chimeric Idol");
    t.lands(P0, "Forest", 2);
    t.activate(P0, idol, 0, &[]).unwrap();
    assert_eq!(tapped_lands(&t, P0), 0, "not tapped to activate it");
    t.resolve_all();
    assert_eq!(tapped_lands(&t, P0), 2);
    assert_eq!(t.pt(idol), (3, 3));
    // With every land already tapped, it can still be activated.
    let mut t = TestGame::new(2);
    let idol = t.battlefield(P0, "Chimeric Idol");
    let l = t.battlefield(P0, "Forest");
    t.g.tap(l);
    assert!(can_activate(&mut t, P0, idol));
}
