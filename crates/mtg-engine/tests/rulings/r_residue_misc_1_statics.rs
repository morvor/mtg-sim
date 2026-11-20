//! Item `rulings-residue-misc-1` — copies, continuous effects and layers, combat and turn
//! structure, and choices: Moritte of the Frost, Thelonite Druid, Balduvian Conjurer,
//! Liege of the Tangle, Xathrid Gorgon, Essence Flare, Cursed Rack, General Jarkeld, World
//! at War, Scheming Fence, Thundermane Dragon, Three Wishes, Mirror of Fate, Call to the
//! Void, Yorion, Bitter Feud, Blessing of Frost, Frenetic Sliver, Siren's Ruse, Sengir
//! Nosferatu, Soul Exchange and Tarmogoyf.

use crate::r_residue_misc_1_common::*;
use crate::r_s25_common::legendary;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::facedown::can_look_at;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Moritte of the Frost: "You may have Moritte enter as a copy of a permanent you control,
// except it's legendary and snow in addition to its other types and, if it's a creature,
// it enters with two additional +1/+1 counters on it and has changeling."
// ---------------------------------------------------------------------------------------

const MORITTE: &str = "Moritte of the Frost";

/// Moritte enters for P0 as a copy of `of`.
fn moritte_copies(t: &mut TestGame, of: ObjectId) -> ObjectId {
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(of)]);
    let m = t.enter(P0, MORITTE);
    t.settle();
    m
}

#[test]
fn moritte_enters_with_two_counters_plus_those_the_copy_brings() {
    cr!("707.9b", "614.1c", "122.6");
    ruling!(
        "Moritte of the Frost",
        "If Moritte copies a creature, it enters with two +1/+1 counters on it, plus any counters that will be put on it by abilities it copied or abilities of other objects."
    );
    supported(MORITTE);
    // Servant of the Scale ("This creature enters with a +1/+1 counter on it."), and
    // Hardened Scales (one more whenever +1/+1 counters would be put on it).
    let mut t = TestGame::new(2);
    let servant = t.battlefield(P0, "Servant of the Scale");
    let m = moritte_copies(&mut t, servant);
    assert_eq!(p1(&t, m), 3);
    let mut t = TestGame::new(2);
    let servant = t.battlefield(P0, "Servant of the Scale");
    t.battlefield(P0, "Hardened Scales");
    let m = moritte_copies(&mut t, servant);
    assert!(p1(&t, m) >= 4, "{}", p1(&t, m));
}

#[test]
fn moritte_copying_a_nonlegendary_permanent_isnt_a_legend_rule_problem() {
    cr!("704.5j", "707.9b");
    ruling!(
        "Moritte of the Frost",
        "If Moritte copies a nonlegendary permanent, you'll control two permanents with same name, but only one of them (Moritte) will be legendary. You won't put any of them into their owner's graveyard."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let m = moritte_copies(&mut t, bears);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
    assert!(legendary(&t, m));
    assert!(!legendary(&t, bears));
    assert!(t.obj_now(m).chars.supertypes.contains(Supertype::Snow));
}

#[test]
fn moritte_cant_copy_a_permanent_entering_at_the_same_time() {
    cr!("707.5", "614.12");
    ruling!(
        "Moritte of the Frost",
        "If Moritte somehow enters the battlefield at the same time as another permanent, it can't become a copy of that permanent. You may choose only a permanent that's already on the battlefield."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    let from = t.asked().len();
    let entered = enter_together(&mut t, &[(P0, MORITTE), (P0, "Hill Giant")]);
    let giant = entered[1];
    let offered: Vec<Entity> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(offered.contains(&obj(bears)));
    assert!(!offered.contains(&obj(giant)));
    assert!(!offered.contains(&obj(t.g.current(giant))));
}

#[test]
fn moritte_copying_a_token_copies_its_original_characteristics_but_isnt_a_token() {
    cr!("707.2", "111.1", "707.9b");
    ruling!(
        "Moritte of the Frost",
        "If the chosen permanent is a token, Moritte copies the original characteristics of that token as stated by the effect that created the token. Moritte doesn't become a token in this case."
    );
    let mut t = TestGame::new(2);
    let soldier = create_token(&mut t, P0, "Soldier");
    grant_kw(&mut t, soldier, KeywordKind::Flying);
    let m = moritte_copies(&mut t, soldier);
    assert_eq!(name_now(&t, m), "Soldier");
    assert!(!t.obj_now(m).is_token());
    assert!(!has(&mut t, m, KeywordKind::Flying));
    assert_eq!(pt_now(&mut t, m), (3, 3));
}

#[test]
fn moritte_copying_a_copy_uses_its_copiable_values_and_x_is_zero() {
    cr!("707.3", "707.9b", "107.3m");
    ruling!(
        "Moritte of the Frost",
        "If the chosen permanent is copying something else, Moritte will use the copiable values of the chosen permanent. In most cases, it will be a copy of whatever the chosen permanent is copying. If it's copying a permanent or card with {X} in its mana cost, X is 0."
    );
    // A Clone copying Grizzly Bears: Moritte becomes Grizzly Bears.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    let clone = t.enter(P0, "Clone");
    let m = moritte_copies(&mut t, clone);
    assert_eq!(name_now(&t, m), "Grizzly Bears");
    assert_eq!(pt_now(&mut t, m), (4, 4));
    // Endless One cast with X = 3 ("enters with X +1/+1 counters"): copied, X is 0.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 3);
    let one = t.hand(P0, "Endless One");
    t.cast(P0, one).x(3).go();
    t.resolve_all();
    assert_eq!(p1(&t, one), 3);
    let one = t.g.current(one);
    let m = moritte_copies(&mut t, one);
    assert_eq!(name_now(&t, m), "Endless One");
    assert_eq!(p1(&t, m), 2);
}

#[test]
fn moritte_copies_only_what_is_printed() {
    cr!("707.2", "613.2", "707.9b");
    ruling!(
        "Moritte of the Frost",
        "Moritte copies exactly what was printed on the original permanent (unless that permanent is copying something else or is a token; see below). It doesn't copy whether that permanent is tapped or untapped, whether it has any counters on it, whether it has any Auras and/or Equipment attached to it, or any non-copy effects that changed its power, toughness, types, color, and so on."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    crate::r_p108_common::put_counters(&mut t, bears, counters::PLUS1, 1);
    attach_new(&mut t, P0, "Holy Strength", bears);
    animate(&mut t, bears, 7, 7);
    let m = moritte_copies(&mut t, bears);
    assert!(!tapped(&t, m));
    assert_eq!(p1(&t, m), 2);
    assert_eq!(pt_now(&mut t, m), (4, 4));
    assert!(t.g.attachments_of(obj(t.g.current(m))).is_empty());
    assert!(t.obj_now(m).chars.colors.contains(Color::Green));
}

// ---------------------------------------------------------------------------------------
// Thelonite Druid: "{1}{G}, {T}, Sacrifice a creature: Forests you control become 2/3
// creatures until end of turn. They're still lands." Balduvian Conjurer: "{T}: Target
// snow land becomes a 2/2 creature until end of turn. It's still a land."
// ---------------------------------------------------------------------------------------

/// P0's Thelonite Druid sacrifices `fodder` (itself by default) and its ability resolves.
fn druid_animates(t: &mut TestGame, druid: ObjectId, fodder: ObjectId) {
    t.lands(P0, "Forest", 2);
    let tapped_forests: Vec<ObjectId> =
        t.g.permanents()
            .filter(|o| o.chars.name == "Forest")
            .map(|o| o.id)
            .collect();
    t.answer_choose(P0, &[obj(fodder)]);
    t.activate(P0, druid, 0, &[]).unwrap();
    t.resolve_all();
    let _ = tapped_forests;
}

#[test]
fn thelonite_druid_can_sacrifice_itself() {
    cr!("602.2b", "118.3");
    ruling!("Thelonite Druid", "Can sacrifice the Druid to itself.");
    supported("Thelonite Druid");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let druid = t.battlefield(P0, "Thelonite Druid");
    druid_animates(&mut t, druid, druid);
    assert!(t.in_graveyard(P0, "Thelonite Druid"));
    assert_eq!(pt_now(&mut t, forest), (2, 3));
    assert!(t.obj_now(forest).is(CardType::Land));
}

#[test]
fn thelonite_druid_affects_only_the_forests_there_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Thelonite Druid",
        "Only affects Forests that are on the battlefield when the effect resolves."
    );
    let mut t = TestGame::new(2);
    let druid = t.battlefield(P0, "Thelonite Druid");
    let bears = t.battlefield(P0, "Grizzly Bears");
    druid_animates(&mut t, druid, bears);
    let late = t.battlefield(P0, "Forest");
    assert!(!t.obj_now(late).is(CardType::Creature));
}

#[test]
fn thelonite_druid_animation_doesnt_end_when_the_land_stops_being_a_forest() {
    cr!("611.2c", "305.7");
    ruling!(
        "Thelonite Druid",
        "The land animation ability does not wear off if the land stops being a Forest. It continues until the end of the turn."
    );
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let druid = t.battlefield(P0, "Thelonite Druid");
    druid_animates(&mut t, druid, druid);
    // Spreading Seas: "Enchanted land is an Island."
    attach_new(&mut t, P1, "Spreading Seas", forest);
    t.g.recompute();
    assert!(t.obj_now(forest).chars.has_subtype("Island"));
    assert!(!t.obj_now(forest).chars.has_subtype("Forest"));
    assert_eq!(pt_now(&mut t, forest), (2, 3));
}

#[test]
fn the_most_recent_land_animation_wins() {
    cr!("613.4b", "613.7a");
    ruling!(
        "Thelonite Druid",
        "The most recent land animating ability takes precedence."
    );
    supported("Balduvian Conjurer");
    // Thelonite Druid makes a Snow-Covered Forest 2/3; Balduvian Conjurer then makes it
    // 2/2: the later one applies.
    let mut t = TestGame::new(2);
    let snow = t.battlefield(P0, "Snow-Covered Forest");
    let druid = t.battlefield(P0, "Thelonite Druid");
    let conj = t.battlefield(P0, "Balduvian Conjurer");
    druid_animates(&mut t, druid, druid);
    assert_eq!(pt_now(&mut t, snow), (2, 3));
    t.activate(P0, conj, 0, &[obj(snow)]).unwrap();
    t.resolve_all();
    assert_eq!(pt_now(&mut t, snow), (2, 2));
}

#[test]
fn balduvian_conjurer_animation_survives_losing_snow() {
    cr!("611.2c", "608.2b");
    ruling!(
        "Balduvian Conjurer",
        "If an affected land later loses the supertype snow, Balduvian Conjurer’s effect doesn’t end. The land will remain a creature until end of turn."
    );
    let mut t = TestGame::new(2);
    let snow = t.battlefield(P0, "Snow-Covered Island");
    let conj = t.battlefield(P0, "Balduvian Conjurer");
    t.activate(P0, conj, 0, &[obj(snow)]).unwrap();
    t.resolve_all();
    run_from(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveSupertypes(vec![Supertype::Snow])],
            duration: Duration::EndOfTurn,
        },
        &[obj(snow)],
    );
    assert!(!t.obj_now(snow).chars.supertypes.contains(Supertype::Snow));
    assert_eq!(pt_now(&mut t, snow), (2, 2));
    assert!(t.obj_now(snow).is(CardType::Creature));
}

// ---------------------------------------------------------------------------------------
// Liege of the Tangle: "... put an awakening counter on each of them. Each of those lands
// is an 8/8 green Elemental creature for as long as it has an awakening counter on it."
// Xathrid Gorgon: "{2}{B}, {T}: Put a petrification counter on target creature. It gains
// defender and becomes a colorless artifact in addition to its other types. Its activated
// abilities can't be activated."
// ---------------------------------------------------------------------------------------

#[test]
fn liege_of_the_tangle_a_moved_awakening_counter_doesnt_animate_the_new_land() {
    cr!("611.2b", "122.5");
    ruling!(
        "Liege of the Tangle",
        "If an awakening counter is moved from an affected land to another land, this doesn't cause the recipient to be affected by Liege of the Tangle's ability."
    );
    supported("Liege of the Tangle");
    let mut t = TestGame::new(2);
    let liege = t.battlefield(P0, "Liege of the Tangle");
    let a = t.battlefield(P0, "Forest");
    let b = t.battlefield(P0, "Forest");
    t.answer_targets(P0, &[obj(a)]);
    t.attack(&[(liege, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(pt_now(&mut t, a), (8, 8));
    run_from(
        &mut t,
        P0,
        None,
        Effect::MoveCounters {
            from: Sel::Target(0),
            to: Sel::Target(1),
            kind: Some("awakening".into()),
            n: Some(Value::Const(1)),
        },
        &[obj(a), obj(b)],
    );
    assert_eq!(t.counters(b, "awakening"), 1);
    assert!(!t.obj_now(b).is(CardType::Creature));
    assert!(!t.obj_now(a).is(CardType::Creature));
}

#[test]
fn xathrid_gorgon_effects_dont_depend_on_the_counter() {
    cr!("611.2c", "122.1");
    ruling!(
        "Xathrid Gorgon",
        "The effects of Xathrid Gorgon’s ability don’t depend on the petrification counter. If that counter is removed, the creature remains a colorless artifact with defender, and so on."
    );
    supported("Xathrid Gorgon");
    let mut t = TestGame::new(2);
    let gorgon = t.battlefield(P0, "Xathrid Gorgon");
    let dragon = t.battlefield(P1, "Shivan Dragon");
    t.lands(P0, "Swamp", 3);
    t.activate(P0, gorgon, 0, &[obj(dragon)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(dragon, "petrification"), 1);
    t.g.remove_counters_by(Entity::Object(dragon), "petrification", 1, None);
    t.g.recompute();
    assert_eq!(t.counters(dragon, "petrification"), 0);
    let o = t.obj_now(dragon);
    assert!(o.is(CardType::Artifact));
    assert!(o.chars.colors.is_colorless());
    assert!(o.has_keyword(KeywordKind::Defender));
    t.lands(P1, "Mountain", 1);
    assert!(!can_activate(&mut t, P1, dragon));
}

#[test]
fn essence_flare_counters_stay_when_the_aura_leaves() {
    cr!("122.1", "610.1");
    ruling!(
        "Essence Flare",
        "The -0/-1 counters stay on the creature even if Essence Flare is removed."
    );
    supported("Essence Flare");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let flare = attach_new(&mut t, P0, "Essence Flare", giant);
    assert_eq!(pt_now(&mut t, giant), (5, 3));
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(giant, "-0/-1"), 1);
    destroy(&mut t, flare);
    assert_eq!(t.counters(giant, "-0/-1"), 1);
    assert_eq!(pt_now(&mut t, giant), (3, 2));
}

// ---------------------------------------------------------------------------------------
// Cursed Rack: "As this artifact enters, choose an opponent. / The chosen player's
// maximum hand size is four."
// ---------------------------------------------------------------------------------------

#[test]
fn cursed_rack_and_another_hand_size_effect_apply_in_timestamp_order() {
    cr!("402.2", "613.11", "613.7a");
    ruling!(
        "Cursed Rack",
        "If multiple effects modify your hand size, apply them in timestamp order. For example, if you put Null Profusion (an enchantment that says your maximum hand size is two) onto the battlefield and then a player puts Cursed Rack onto the battlefield choosing you, your maximum hand size will be four. However, if those permanents entered in the opposite order, your maximum hand size would be two."
    );
    supported("Cursed Rack");
    supported("Null Profusion");
    for profusion_first in [true, false] {
        let mut t = TestGame::new(2);
        if profusion_first {
            t.enter(P0, "Null Profusion");
        }
        t.answer_choose(P1, &[Entity::Player(P0)]);
        t.enter(P1, "Cursed Rack");
        if !profusion_first {
            t.enter(P0, "Null Profusion");
        }
        t.g.recompute();
        let expected = if profusion_first { 4 } else { 2 };
        assert_eq!(t.g.player(P0).max_hand_size, Some(expected));
    }
}

// ---------------------------------------------------------------------------------------
// General Jarkeld: "{T}: Choose two target blocked attacking creatures. If each of those
// creatures could be blocked by all creatures that the other is blocked by, each creature
// that's blocking exactly one of those attacking creatures stops blocking it and is
// blocking the other attacking creature. Activate only during the declare blockers step."
// ---------------------------------------------------------------------------------------

#[test]
fn general_jarkeld_switch_keeps_block_triggers_and_triggers_nothing_new() {
    cr!("509.1h", "509.3c", "603.2");
    ruling!(
        "General Jarkeld",
        "Does not undo any abilities which triggered due to the declaration of blockers. A creature which is switched away still has any triggered abilities resolve on it."
    );
    ruling!(
        "General Jarkeld",
        "The new blocker does not trigger any abilities which trigger on creatures becoming blockers, because the creatures were already blockers and the simple change of who is blocking does not trigger such abilities."
    );
    supported("Devoted Retainer");
    // Devoted Retainer (1/1, bushido 1) and Grizzly Bears attack; Hill Giant blocks the
    // Retainer and Wall of Stone the Bears. The bushido trigger is on the stack when the
    // blockers are switched.
    let mut t = TestGame::new(2);
    let retainer = t.battlefield(P0, "Devoted Retainer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let jarkeld = t.battlefield(P1, "General Jarkeld");
    let giant = t.battlefield(P1, "Hill Giant");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![
            (retainer, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
        ]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(giant, retainer), (wall, bears)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.activate(P1, jarkeld, 0, &[obj(retainer), obj(bears)])
        .unwrap();
    t.resolve();
    // Switched; no new trigger for the Retainer becoming blocked by the Wall.
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(pt_now(&mut t, retainer), (2, 2));
    t.advance_to(P0, Step::EndOfCombat);
    t.settle();
    // The Giant blocked the Bears; the Wall the Retainer.
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.on_battlefield(retainer));
    assert_eq!(t.obj_now(wall).damage, 2);
}

// ---------------------------------------------------------------------------------------
// World at War: "After the second main phase this turn, there's an additional combat
// phase followed by an additional main phase. At the beginning of that combat, untap all
// creatures that attacked this turn."
// ---------------------------------------------------------------------------------------

/// Casts World at War in P0's precombat main phase (`n` times) and attacks P1 with
/// `attacker` in the first combat; stops at the postcombat main phase.
fn world_at_war(n: usize) -> (TestGame, ObjectId) {
    supported("World at War");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    for _ in 0..n {
        cast_new(&mut t, P0, "World at War", &[]);
        t.resolve_all();
    }
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    (t, bears)
}

#[test]
fn world_at_war_untaps_attackers_as_the_new_combat_begins_whoever_controls_them() {
    cr!("500.8", "505.1a", "603.7");
    ruling!(
        "World at War",
        "Unlike other similar cards (such as Relentless Assault), World at War doesn’t untap the creatures that have attacked this turn when it resolves. Rather, those creatures untap when the new combat phase created by the spell begins. All creatures that attacked this turn untap, regardless of which combat phase they attacked in (if there have been more than one) or who controls them (if you’re playing a Two-Headed Giant game, for example)."
    );
    let (mut t, bears) = world_at_war(1);
    assert!(tapped(&t, bears));
    // P1 gains control of it: it still untaps.
    give_control(&mut t, bears, P1);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(!tapped(&t, bears));
}

#[test]
fn world_at_war_no_one_has_to_attack_in_the_new_combat() {
    cr!("508.1", "505.1a");
    ruling!(
        "World at War",
        "You don’t have to attack with any creatures during the new combat phase."
    );
    let (mut t, bears) = world_at_war(1);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![]));
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.g.turn.main_phases, 3);
    assert!(!is_attacking(&t, bears));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn world_at_war_effects_are_cumulative() {
    cr!("500.8", "505.1a");
    ruling!(
        "World at War",
        "Multiple World at War effects are cumulative, as long as they’re cast early enough. The first one modifies the turn structure to this: beginning phase, precombat main phase, combat phase, postcombat main phase, [new combat phase], [new postcombat main phase], ending phase. Each subsequent one inserts another combat phase and main phase into the turn after the original postcombat main phase and before the newest combat phase."
    );
    let (mut t, bears) = world_at_war(2);
    for (combat, main) in [(2, 3), (3, 4)] {
        t.advance_to(P0, Step::BeginningOfCombat);
        t.resolve_all();
        assert_eq!(t.g.turn.combat_phases, combat);
        assert!(!tapped(&t, bears));
        t.attack(&[(bears, Entity::Player(P1))], &[]);
        t.advance_to(P0, Step::PostcombatMain);
        assert_eq!(t.g.turn.main_phases, main);
    }
    assert_eq!(t.life(P1), 14);
    // No fourth combat this turn.
    t.advance_to(P0, Step::End);
    assert_eq!(t.g.turn.combat_phases, 3);
}

// ---------------------------------------------------------------------------------------
// Scheming Fence: "This creature has all activated abilities of the chosen permanent
// except for loyalty abilities."
// ---------------------------------------------------------------------------------------

#[test]
fn scheming_fence_an_equip_ability_does_nothing_for_it() {
    cr!("702.6a", "301.5c", "701.3b");
    ruling!(
        "Scheming Fence",
        "Some activated abilities won't do anything when activated by Scheming Fence. For example, if the chosen permanent is an Equipment with an equip ability, activating that equip ability will not cause Scheming Fence to become attached because it isn't one of the subtypes which can be attached to permanents (Aura, Equipment, and Fortification)."
    );
    supported("Scheming Fence");
    let mut t = TestGame::new(2);
    let splitter = t.battlefield(P1, "Bonesplitter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(splitter)]);
    let fence = t.enter(P0, "Scheming Fence");
    t.lands(P0, "Plains", 1);
    t.activate(P0, fence, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(attached_to(&t, fence), None);
    assert_eq!(pt_now(&mut t, bears), (2, 2));
    assert_eq!(attached_to(&t, splitter), None);
}

// ---------------------------------------------------------------------------------------
// Thundermane Dragon: "You may look at the top card of your library any time. / You may
// cast creature spells with power 4 or greater from the top of your library. ..."
// Three Wishes: "Exile the top three cards of your library face down. You may look at
// those cards for as long as they remain exiled. ..."
// ---------------------------------------------------------------------------------------

#[test]
fn thundermane_dragon_look_at_the_top_card_any_time() {
    cr!("401.5", "117.1");
    ruling!(
        "Thundermane Dragon",
        "Thundermane Dragon lets you look at the top card of your library whenever you want (with one restriction—see below), even if you don’t have priority. This action doesn’t use the stack. Knowing what that card is becomes part of the information you have access to, just like you can look at the cards in your hand."
    );
    supported("Thundermane Dragon");
    let mut t = TestGame::new(2);
    let top = t.library_top(P0, "Hill Giant");
    t.battlefield(P0, "Thundermane Dragon");
    t.g.recompute();
    // P1 has priority (P0 doesn't): P0 may look; P1 may not.
    t.set_step(P1, Step::PrecombatMain);
    assert!(can_look_at(&t.g, P0, top));
    assert!(!can_look_at(&t.g, P1, top));
}

#[test]
fn thundermane_dragon_the_new_top_card_waits_until_the_spell_is_cast() {
    cr!("401.5", "601.2i");
    ruling!(
        "Thundermane Dragon",
        "If the top card of your library changes while you’re casting a spell, playing a land, activating an ability, or taking a special action, you can’t look at the new top card until you finish doing so. This means that if you cast the top card of your library, you can’t look at the next one until you’re done paying for that spell."
    );
    supported("Verdeloth the Ancient");
    // Verdeloth the Ancient (4/4, kicker {X}) cast from the top: while choosing whether
    // to kick it, the Hill Giant under it can't be looked at.
    let mut t = TestGame::new(2);
    let next = t.library_top(P0, "Hill Giant");
    let v = t.library_top(P0, "Verdeloth the Ancient");
    t.battlefield(P0, "Thundermane Dragon");
    t.lands(P0, "Forest", 6);
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::OptionalCost { .. }),
        |g| g.library_top(P0).is_some_and(|top| can_look_at(g, P0, top)),
    );
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.cast(P0, v).go();
    assert_eq!(*seen.lock().unwrap(), vec![false]);
    assert_eq!(t.g.library_top(P0), Some(next));
    assert!(can_look_at(&t.g, P0, next));
}

#[test]
fn three_wishes_you_may_look_at_the_exiled_cards_any_time() {
    cr!("406.3", "708.5");
    ruling!(
        "Three Wishes",
        "You can look at the three exiled cards at any time."
    );
    supported("Three Wishes");
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Grizzly Bears", "Hill Giant", "Shock"]);
    cast_new(&mut t, P0, "Three Wishes", &[]);
    t.resolve_all();
    t.set_step(P1, Step::PrecombatMain);
    for c in cards {
        let c = t.g.current(c);
        assert_eq!(t.zone(c), Zone::Exile);
        assert!(t.g.obj(c).face_down);
        assert!(can_look_at(&t.g, P0, c));
        assert!(!can_look_at(&t.g, P1, c));
    }
}

// ---------------------------------------------------------------------------------------
// Mirror of Fate: "{T}, Sacrifice this artifact: Choose up to seven face-up exiled cards
// you own. Exile all the cards from your library, then put the chosen cards on top of
// your library."
// ---------------------------------------------------------------------------------------

#[test]
fn mirror_of_fate_exiles_the_library_face_up() {
    cr!("406.3");
    ruling!(
        "Mirror of Fate",
        "Exiled cards are face up by default; they’re face down only if the effect that exiled them said so. For example, the cards from your library that Mirror of Fate exiles are exiled face up."
    );
    supported("Mirror of Fate");
    let mut t = TestGame::new(2);
    let mirror = t.battlefield(P0, "Mirror of Fate");
    let lib = stack_library(&mut t, P0, &["Grizzly Bears", "Hill Giant"]);
    // (Nothing in exile to choose yet.)
    t.activate(P0, mirror, 0, &[]).unwrap();
    t.resolve_all();
    for c in &lib {
        let c = t.g.current(*c);
        assert_eq!(t.zone(c), Zone::Exile);
        assert!(!t.g.obj(c).face_down);
    }
    // A second Mirror can choose them.
    let mirror = t.battlefield(P0, "Mirror of Fate");
    let bears = t.g.current(lib[0]);
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, mirror, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Library(P0));
}

#[test]
fn mirror_of_fate_you_choose_the_order() {
    cr!("401.4", "608.2c");
    ruling!(
        "Mirror of Fate",
        "You choose the order that the chosen cards are put on top of your library. You don’t have to show anyone else that order."
    );
    let top_after = |order: Vec<usize>| {
        let mut t = TestGame::new(2);
        let mirror = t.battlefield(P0, "Mirror of Fate");
        let a = t.exile(P0, "Grizzly Bears");
        let b = t.exile(P0, "Hill Giant");
        t.answer_choose(P0, &[obj(a), obj(b)]);
        t.answer(P0, DecisionKind::Order, Answer::Indices(order));
        t.activate(P0, mirror, 0, &[]).unwrap();
        t.resolve_all();
        assert_eq!(t.library_size(P0), 2);
        assert!(t
            .asked()
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. })));
        let top = t.g.library_top(P0).unwrap();
        // Nobody else sees the library's cards.
        assert!(!can_look_at(&t.g, P1, top));
        t.g.obj(top).chars.name.to_string()
    };
    assert_ne!(top_after(vec![0, 1]), top_after(vec![1, 0]));
}

// ---------------------------------------------------------------------------------------
// Call to the Void: "Each player secretly chooses a creature they control and a creature
// they don't control. Then those choices are revealed. Destroy each creature chosen this
// way."
// ---------------------------------------------------------------------------------------

#[test]
fn call_to_the_void_choices_are_made_before_anything_is_revealed_or_destroyed() {
    cr!("101.4", "608.2d");
    ruling!(
        "Call to the Void",
        "For each player to secretly choose the creatures, that player should write down their choices without showing them to anyone else. Each player then keeps their choices secret until all players simultaneously reveal their choices."
    );
    ruling!(
        "Call to the Void",
        "If a creature is chosen more than once, it is destroyed only once. For example, having a single shield counter on it would allow it to survive."
    );
    supported("Call to the Void");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let c = t.battlefield(P1, "Llanowar Elves");
    crate::r_p108_common::put_counters(&mut t, a, "shield", 1);
    // Both choose P0's Bears (shielded once); P0 also chooses the Giant, P1 its Elves.
    t.answer_choose(P0, &[obj(a)]);
    t.answer_choose(P0, &[obj(b)]);
    t.answer_choose(P1, &[obj(c)]);
    t.answer_choose(P1, &[obj(a)]);
    // When P1 chooses, nothing chosen has been destroyed yet.
    let seen = crate::r_s01_common::watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| g.battlefield.len(),
    );
    cast_new(&mut t, P0, "Call to the Void", &[]);
    let before = t.g.battlefield.len();
    t.resolve_all();
    let seen = seen.lock().unwrap().clone();
    assert!(!seen.is_empty());
    assert!(seen.iter().all(|n| *n == before), "{seen:?} {before}");
    assert!(t.on_battlefield(a));
    assert_eq!(t.counters(a, "shield"), 0);
    assert!(!t.on_battlefield(b));
    assert!(!t.on_battlefield(c));
}

// ---------------------------------------------------------------------------------------
// Yorion, Sky Nomad: "When Yorion enters, exile any number of other nonland permanents you
// own and control. Return those cards to the battlefield at the beginning of the next end
// step." Haphazard Bombardment: "When this enchantment enters, choose four nonenchantment
// permanents you don't control and put an aim counter on each of them."
// ---------------------------------------------------------------------------------------

#[test]
fn yorion_exiles_only_permanents_you_both_own_and_control() {
    cr!("608.2c", "108.3");
    ruling!(
        "Yorion, Sky Nomad",
        "You can't exile permanents you control but don't own, or permanents that you own but don't control."
    );
    ruling!(
        "Yorion, Sky Nomad",
        "You choose which permanents to exile as Yorion's triggered ability resolves. No player may take action between the time you choose the permanents and the time they're exiled."
    );
    supported("Yorion, Sky Nomad");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let borrowed = t.battlefield(P1, "Hill Giant");
    give_control(&mut t, borrowed, P0);
    let lent = t.battlefield(P0, "Llanowar Elves");
    give_control(&mut t, lent, P1);
    t.answer_choose(P0, &[obj(mine)]);
    enter(&mut t, P0, "Yorion, Sky Nomad");
    let from = t.asked().len();
    t.resolve();
    let offered: Vec<Entity> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(offered.contains(&obj(mine)));
    assert!(!offered.contains(&obj(borrowed)));
    assert!(!offered.contains(&obj(lent)));
    assert!(others_asked_since(&t, from, P0).is_empty());
    assert_eq!(t.zone(mine), Zone::Exile);
    assert!(t.on_battlefield(borrowed));
    assert!(t.on_battlefield(lent));
}

#[test]
fn haphazard_bombardment_aim_counters_ignore_hexproof() {
    cr!("115.10", "702.11b");
    ruling!(
        "Haphazard Bombardment",
        "The nonenchantment permanents that receive aim counters aren’t targeted. Permanents with hexproof can be given an aim counter this way."
    );
    supported("Haphazard Bombardment");
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P1, "Gladecover Scout");
    let others: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
    let mut chosen = vec![obj(scout)];
    chosen.extend(others.iter().map(|o| obj(*o)));
    t.answer_choose(P0, &chosen);
    enter(&mut t, P0, "Haphazard Bombardment");
    t.resolve_all();
    assert_eq!(t.counters(scout, "aim"), 1);
}

// ---------------------------------------------------------------------------------------
// Bitter Feud: "If a source controlled by one of the chosen players would deal damage to
// the other chosen player or a permanent that player controls, that source deals double
// that damage to that player or permanent instead."
// ---------------------------------------------------------------------------------------

#[test]
fn bitter_feud_doubles_the_damage_without_changing_its_source() {
    cr!("614.1a", "702.15b", "120.3");
    ruling!(
        "Bitter Feud",
        "The source of the damage doesn’t change. A spell that deals damage will specify the source of the damage. This is usually the spell itself. An ability that deals damage will specify the source of the damage, although it will never be the ability itself. Usually the source of the ability is also the source of the damage."
    );
    supported("Bitter Feud");
    // P0's lifelinking Child of Night deals 2 damage to P1's Hill Giant: 4 is dealt, by
    // the Child, so P0 gains 4.
    let mut t = TestGame::new(2);
    t.answer_choose(P0, &[Entity::Player(P0), Entity::Player(P1)]);
    t.enter(P0, "Bitter Feud");
    let child = t.battlefield(P0, "Child of Night");
    let giant = t.battlefield(P1, "Hill Giant");
    crate::r_s06_common::damage(&mut t, child, 2, giant);
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P0), 24);
}

// ---------------------------------------------------------------------------------------
// Blessing of Frost: "Distribute X +1/+1 counters among any number of creatures you
// control, where X is the amount of {S} spent to cast this spell. Then draw a card for
// each creature you control with power 4 or greater."
// ---------------------------------------------------------------------------------------

#[test]
fn blessing_of_frost_can_put_counters_on_a_creature_with_protection_from_green() {
    cr!("702.16b", "115.10");
    ruling!(
        "Blessing of Frost",
        "Because none of the creatures are targeted, you can choose to put +1/+1 counters on a creature with protection from green, for example."
    );
    supported("Blessing of Frost");
    supported("Mirran Crusader");
    let mut t = TestGame::new(2);
    let crusader = t.battlefield(P0, "Mirran Crusader");
    t.lands(P0, "Snow-Covered Forest", 4);
    t.answer_choose(P0, &[obj(crusader)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![4]));
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Blessing of Frost", &[]);
    t.resolve_all();
    assert_eq!(p1(&t, crusader), 4);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn blessing_of_frost_copy_distributes_nothing_but_draws() {
    cr!("707.10", "107.4h", "601.2h");
    ruling!(
        "Blessing of Frost",
        "If Blessing of Frost is copied, no mana was spent to cast the copy, so no +1/+1 counters will be distributed. You may still draw cards if your creatures are large enough."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Snow-Covered Forest", 4);
    let blessing = t.hand(P0, "Blessing of Frost");
    t.answer_choose(P0, &[obj(bears)]);
    t.cast(P0, blessing).go();
    let spell = t.g.current(blessing);
    t.lands(P0, "Island", 2);
    let twin = t.hand(P0, "Twincast");
    t.cast(P0, twin).target(spell).go();
    t.answer_yes(P0, false);
    t.answer_choose(P0, &[obj(bears)]);
    let hand = t.hand_size(P0);
    t.resolve();
    // The copy: no counters; one card (the Wurm).
    t.resolve();
    assert_eq!(p1(&t, bears), 0);
    assert_eq!(t.hand_size(P0), hand + 1);
    let _ = wurm;
}

// ---------------------------------------------------------------------------------------
// Frenetic Sliver: "All Slivers have '{0}: If this permanent is on the battlefield, flip a
// coin. If you win the flip, exile this permanent and return it to the battlefield under
// its owner's control at the beginning of the next end step. If you lose the flip,
// sacrifice it.'"
// ---------------------------------------------------------------------------------------

#[test]
fn frenetic_sliver_activated_repeatedly_flips_only_one_coin() {
    cr!("608.2c", "705.1");
    ruling!(
        "Frenetic Sliver",
        "For any given Sliver, this ability can be activated numerous times in response to itself, but the end result is that only one coin will be flipped for that Sliver."
    );
    supported("Frenetic Sliver");
    for heads in [true, false] {
        let mut t = TestGame::new(2);
        let s = t.battlefield(P0, "Frenetic Sliver");
        crate::r_p017_common::load_coins(&mut t, &[heads, heads, heads]);
        for _ in 0..3 {
            t.activate(P0, s, 0, &[]).unwrap();
        }
        t.resolve_all();
        assert_eq!(
            crate::r_p017_common::flip_results(&t).len(),
            1,
            "heads: {heads}"
        );
        if heads {
            assert_eq!(t.zone(s), Zone::Exile);
        } else {
            assert!(t.in_graveyard(P0, "Frenetic Sliver"));
        }
    }
}

// ---------------------------------------------------------------------------------------
// Siren's Ruse: "Exile target creature you control, then return that card to the
// battlefield under its owner's control. If a Pirate was exiled this way, draw a card."
// ---------------------------------------------------------------------------------------

#[test]
fn sirens_ruse_the_returned_creature_is_a_new_object() {
    cr!("400.7", "608.2b");
    ruling!(
        "Siren's Ruse",
        "The returned creature won’t be the target of any spells or abilities that targeted it before. Any spells that don’t target it, such as Star of Extinction, will still affect it."
    );
    supported("Siren's Ruse");
    // P1's Shock targets the Bears: Siren's Ruse in response, and the Shock fizzles.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(bears).go();
    cast_new(&mut t, P0, "Siren's Ruse", &[obj(bears)]);
    t.resolve_all();
    let back = t.g.current(bears);
    assert!(t.on_battlefield(back));
    assert_eq!(damage_on(&t, back), 0);
    // P1's Pyroclasm doesn't target: it still kills the returned Bears.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Pyroclasm", &[]);
    cast_new(&mut t, P0, "Siren's Ruse", &[obj(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn sirens_ruse_draws_for_a_pirate_as_it_was_exiled() {
    cr!("608.2h", "111.7");
    ruling!(
        "Siren's Ruse",
        "You’ll draw a card if the creature was a Pirate as it was exiled, even if it doesn’t return to the battlefield (most likely because it’s a token) or if it returns to the battlefield but isn’t a Pirate anymore (most likely because it’s copying something else)."
    );
    // A Pirate token: it doesn't return; the card is drawn.
    let mut t = TestGame::new(2);
    let pirate = create_token(&mut t, P0, "Pirate");
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Siren's Ruse", &[obj(pirate)]);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    assert_eq!(t.hand_size(P0), hand + 1);
    // A Clone copying a Pirate returns as a Clone that copies the Bears: still a card.
    let mut t = TestGame::new(2);
    let captain = t.battlefield(P1, "Fathom Fleet Captain");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(captain)]);
    let clone = t.enter(P0, "Clone");
    assert!(t.obj_now(clone).chars.has_subtype("Pirate"));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Siren's Ruse", &[obj(clone)]);
    t.resolve_all();
    let back = t.g.current(clone);
    assert_eq!(name_now(&t, back), "Grizzly Bears");
    assert_eq!(t.hand_size(P0), hand + 1);
}

// ---------------------------------------------------------------------------------------
// Sengir Nosferatu: "{1}{B}, Exile this creature: Create a 1/2 black Bat creature token
// with flying. It has '{1}{B}, Sacrifice this token: Return an exiled card named Sengir
// Nosferatu to the battlefield under its owner's control.'"
// ---------------------------------------------------------------------------------------

/// `p`'s `source` (a Sengir Nosferatu, or something copying one) exiles itself; the Bat.
fn bat_from(t: &mut TestGame, p: PlayerId, source: ObjectId) -> ObjectId {
    t.lands(p, "Swamp", 2);
    activate_containing(t, p, source, "Exile").unwrap();
    t.resolve_all();
    with_subtype(t, p, "Bat")[0]
}

#[test]
fn sengir_nosferatu_a_copy_in_exile_isnt_named_sengir_nosferatu() {
    cr!("707.2", "400.7", "201.2");
    ruling!(
        "Sengir Nosferatu",
        "If the Nosferatu is copied (by Clone, for example) and then the copy is exiled by the first activated ability, it won't be able to come back using the second activated ability, since it's no longer named Sengir Nosferatu in the Exile zone."
    );
    supported("Sengir Nosferatu");
    let mut t = TestGame::new(2);
    let nos = t.battlefield(P1, "Sengir Nosferatu");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(nos)]);
    let clone = t.enter(P0, "Clone");
    let bat = bat_from(&mut t, P0, clone);
    assert!(t.in_exile("Clone"));
    t.lands(P0, "Swamp", 2);
    activate_containing(&mut t, P0, bat, "Return").unwrap();
    t.resolve_all();
    assert!(t.in_exile("Clone"));
    assert!(t.named_on_battlefield("Clone").is_empty());
}

#[test]
fn sengir_nosferatu_the_bat_returns_any_exiled_nosferatu_to_its_owner() {
    cr!("110.2", "201.2");
    ruling!(
        "Sengir Nosferatu",
        "The Sengir Nosferatu that the Bat token's ability returns to the battlefield doesn't have to be the same one that created it, and it doesn't even have to be owned by the same player."
    );
    let mut t = TestGame::new(2);
    let nos = t.battlefield(P0, "Sengir Nosferatu");
    let bat = bat_from(&mut t, P0, nos);
    // P0's own Nosferatu leaves exile; P1's is there.
    move_to(&mut t, nos, Zone::Graveyard(P0));
    let theirs = t.exile(P1, "Sengir Nosferatu");
    t.answer_choose(P0, &[obj(theirs)]);
    t.lands(P0, "Swamp", 2);
    activate_containing(&mut t, P0, bat, "Return").unwrap();
    t.resolve_all();
    let back = t.g.current(theirs);
    assert!(t.on_battlefield(back));
    assert_eq!(t.g.obj(back).controller, P1);
}

// ---------------------------------------------------------------------------------------
// Soul Exchange: "As an additional cost to cast this spell, exile a creature you control.
// / Return target creature card from your graveyard to the battlefield. Put a +2/+2
// counter on that creature if the exiled creature was a Thrull."
// ---------------------------------------------------------------------------------------

#[test]
fn soul_exchange_counter_isnt_stopped_by_protection_from_black() {
    cr!("702.16b", "115.10");
    ruling!(
        "Soul Exchange",
        "Putting a counter on the creature put onto the battlefield is not a targeted effect and so Protection from Black will not prevent it."
    );
    ruling!(
        "Soul Exchange",
        "You exile exactly one creature. You can’t exile more than one in an attempt to get a larger effect."
    );
    supported("Soul Exchange");
    let mut t = TestGame::new(2);
    let thrull = t.battlefield(P0, "Basal Thrull");
    let other = t.battlefield(P0, "Grizzly Bears");
    let knight = t.graveyard(P0, "White Knight");
    t.answer_choose(P0, &[obj(thrull), obj(other)]);
    cast_new(&mut t, P0, "Soul Exchange", &[obj(knight)]);
    // Exactly one creature was exiled.
    let exiled = [thrull, other]
        .iter()
        .filter(|c| t.zone(**c) == Zone::Exile)
        .count();
    assert_eq!(exiled, 1);
    let thrull_exiled = t.zone(thrull) == Zone::Exile;
    t.resolve_all();
    let k = t.g.current(knight);
    assert!(t.on_battlefield(k));
    assert!(has(&mut t, k, KeywordKind::Protection));
    if thrull_exiled {
        assert_eq!(t.counters(k, "+2/+2"), 1);
        assert_eq!(pt_now(&mut t, k), (4, 4));
    }
}

// ---------------------------------------------------------------------------------------
// Tarmogoyf: "Tarmogoyf's power is equal to the number of card types among cards in all
// graveyards and its toughness is equal to that number plus 1."
// ---------------------------------------------------------------------------------------

#[test]
fn tarmogoyf_counts_card_types_battle_and_kindred_included_not_supertypes() {
    cr!("205.2a", "205.4a", "604.3");
    ruling!(
        "Tarmogoyf",
        "The card types that can appear on cards in a graveyard are artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Legendary, basic, and snow are supertypes, not card types."
    );
    supported("Tarmogoyf");
    let mut t = TestGame::new(2);
    let goyf = t.battlefield(P0, "Tarmogoyf");
    // Snow-Covered Forest (basic snow land): land only.
    t.graveyard(P0, "Snow-Covered Forest");
    assert_eq!(pt_now(&mut t, goyf), (1, 2));
    // Bitterblossom (kindred enchantment): kindred and enchantment.
    t.graveyard(P1, "Bitterblossom");
    assert_eq!(pt_now(&mut t, goyf), (3, 4));
    // A battle card.
    t.graveyard(P1, "Invasion of Tarkir // Defiant Thundermaw");
    assert_eq!(pt_now(&mut t, goyf), (4, 5));
    // A legendary creature card: creature only (legendary is a supertype).
    t.graveyard(P0, "Kaysa");
    assert_eq!(pt_now(&mut t, goyf), (5, 6));
}
