//! Rulings batch P048 — D&D monsters and mechanics: Berserker's Frenzy (ignored rolls,
//! CR 706.6; choosing blocks for another player, CR 509.1), Ranger Class (CR 401.5, 716),
//! Radiant Solar, Dissonant Wave (countering abilities, CR 113, 605, 614, 603.7), Green
//! Dragon (CR 603.7, 702.19b), Explosive Crystal (divided damage, CR 601.2d, 608.2b),
//! Zariel's emblem (additional combat phases, CR 500.8) and Sculpted Sunburst.

use crate::r_p048_common::*;
use crate::r_s09_common::legal_attack;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

const ADVENTURE: CastMethod = CastMethod::Half(1);

/// P0 casts Berserker's Frenzy (with the dice loaded with `rolls`) in its beginning of
/// combat step, choosing `chosen` for the 1—14 row; it resolves.
fn frenzy(t: &mut TestGame, rolls: &[u32], chosen: &[ObjectId]) {
    supported("Berserker's Frenzy");
    t.g.dice.loaded.extend(rolls.iter().copied());
    t.answer_choose(P0, &chosen.iter().map(|c| obj(*c)).collect::<Vec<_>>());
    cast_new(t, P0, "Berserker's Frenzy", &[]);
    t.resolve_all();
}

/// The d20 results reported since event `from`.
fn rolls_since(t: &TestGame, from: usize) -> Vec<u32> {
    t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::DieRolled { result, .. } => Some(*result),
            _ => None,
        })
        .collect()
}

#[test]
fn berserkers_frenzys_ignored_roll_never_happens() {
    cr!("706.6");
    ruling!(
        "Berserker's Frenzy",
        "An ignored roll effectively never happens. No abilities will trigger because of an ignored roll."
    );
    ruling!(
        "Berserker's Frenzy",
        "If you roll doubles, ignore one of the rolls and use the result of the other."
    );
    // As Luck Would Have It: "Whenever you roll a die, put a number of luck counters on
    // this enchantment equal to the result."
    supported("As Luck Would Have It");
    for (rolls, kept) in [([18, 5], 18), ([7, 7], 7)] {
        let mut t = TestGame::new(2);
        let luck = t.battlefield(P0, "As Luck Would Have It");
        let bears = t.battlefield(P1, "Grizzly Bears");
        to_combat(&mut t, P0);
        let from = t.g.turn_events.len();
        frenzy(&mut t, &rolls, &[bears]);
        assert_eq!(rolls_since(&t, from), vec![kept]);
        assert_eq!(t.counters(luck, "luck"), kept);
    }
}

/// P0 attacks P1 with `attackers` after Berserker's Frenzy rolled 15—20 (P0 chooses how
/// P1's creatures block); stops before blockers are declared.
fn frenzied_attack(t: &mut TestGame, attackers: &[ObjectId]) {
    to_combat(t, P0);
    frenzy(t, &[17, 3], &[]);
    attack_with(
        t,
        &attackers
            .iter()
            .map(|a| (*a, Entity::Player(P1)))
            .collect::<Vec<_>>(),
    );
    t.resolve_all();
}

/// Who was asked to declare blockers since decision `from`.
fn block_deciders(t: &TestGame, from: usize) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::DeclareBlockers { .. }))
        .map(|(p, _)| *p)
        .collect()
}

/// Runs the game until the declare blockers step has begun and blockers are declared.
fn to_blocks(t: &mut TestGame) {
    let ok = t.g.run_until(10_000, |g| {
        g.turn.step == Step::DeclareBlockers && g.turn.stage == Stage::Priority
    });
    assert!(ok);
}

#[test]
fn berserkers_frenzy_blocks_must_be_legal_and_costs_can_be_declined() {
    cr!("509.1", "509.1a", "509.1b", "509.1d", "702.9b", "706.3a");
    ruling!(
        "Berserker's Frenzy",
        "When choosing blocks for creatures you don't control, you must still make legal choices. For example, you can't choose to have a creature without flying or reach block a creature with flying."
    );
    ruling!(
        "Berserker's Frenzy",
        "If there's a cost associated with having a creature block and you choose for that creature to block, its controller can choose to pay that cost or not. If that player doesn't pay that cost, you must propose a new set of blocking creatures."
    );
    // P0 chooses P1's blocks: the Gray Ogre can't block the Serra Angel (flying); the
    // illegal declaration is undone.
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ogre = t.battlefield(P1, "Gray Ogre");
    frenzied_attack(&mut t, &[angel, bears]);
    assert!(!legal_blocks(&mut t, P1, &[(ogre, angel)]));
    assert!(legal_blocks(&mut t, P1, &[(ogre, bears)]));
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(ogre, angel)]),
    );
    let from = t.asked().len();
    to_blocks(&mut t);
    assert!(block_deciders(&t, from).contains(&P0));
    assert!(!block_deciders(&t, from).contains(&P1));
    assert!(blocks_now(&t).is_empty());
    // Archangel of Tithes ("As long as this creature is attacking, creatures can't block
    // unless their controller pays {1} for each of those creatures."): P0 has the Ogre
    // block the Bears; P1 may decline to pay, and P0 proposes again.
    supported("Archangel of Tithes");
    for pays in [false, true] {
        let mut t = TestGame::new(2);
        let archangel = t.battlefield(P0, "Archangel of Tithes");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let ogre = t.battlefield(P1, "Gray Ogre");
        t.lands(P1, "Wastes", 1);
        frenzied_attack(&mut t, &[archangel, bears]);
        t.answer(
            P0,
            DecisionKind::Blockers,
            Answer::Blockers(vec![(ogre, bears)]),
        );
        t.answer_yes(P1, pays);
        t.answer(P0, DecisionKind::Blockers, Answer::Blockers(vec![]));
        let from = t.asked().len();
        to_blocks(&mut t);
        assert!(t.asked()[from..]
            .iter()
            .any(|(p, d)| *p == P1 && matches!(d, Decision::YesNo { .. })));
        let proposals = block_deciders(&t, from);
        if pays {
            assert_eq!(proposals, vec![P0]);
            assert_eq!(blocks_now(&t), vec![(ogre, bears)]);
        } else {
            assert_eq!(proposals, vec![P0, P0]);
            assert!(blocks_now(&t).is_empty());
        }
    }
}

/// P0's Ranger Class at level `level`.
fn ranger_class(t: &mut TestGame, level: u32) -> ObjectId {
    supported("Ranger Class");
    let class = t.battlefield(P0, "Ranger Class");
    t.g.objects[class.0 as usize].class_level = level;
    t.g.recompute();
    class
}

#[test]
fn ranger_class_level_3_lets_you_look_at_the_top_card_any_time() {
    cr!("401.5", "716.2");
    ruling!(
        "Ranger Class",
        "If Ranger Class is level 3, you can look at the top card of your library whenever you want"
    );
    for level in [2, 3] {
        let mut t = TestGame::new(2);
        ranger_class(&mut t, level);
        let top = t.library_top(P0, "Grizzly Bears");
        // In P1's turn, with P1 holding priority and a spell on the stack.
        t.set_step(P1, Step::PrecombatMain);
        cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
        t.g.recompute();
        assert_eq!(
            mtg_engine::zones::can_see_in_library(&t.g, P0, top),
            level == 3
        );
        assert!(!mtg_engine::zones::can_see_in_library(&t.g, P1, top));
    }
}

#[test]
fn ranger_class_doesnt_change_when_creatures_can_be_cast() {
    cr!("601.3", "302.1", "702.8a");
    ruling!(
        "Ranger Class",
        "Ranger Class doesn't change when you can cast creature spells. Normally, this means during your main phase when the stack is empty, although flash may change this."
    );
    // Grizzly Bears on top: castable in P0's main phase, not in P1's turn. Ambush Viper
    // (flash) on top: castable in P1's turn.
    for (creature, main, theirs) in [("Grizzly Bears", true, false), ("Ambush Viper", true, true)] {
        let mut t = TestGame::new(2);
        ranger_class(&mut t, 3);
        lands_for_cost(&mut t, P0, creature);
        let top = t.library_top(P0, creature);
        assert_eq!(
            castable(&mut t, P0, top),
            main,
            "{creature} in P0's main phase"
        );
        t.set_step(P1, Step::PrecombatMain);
        assert_eq!(castable(&mut t, P0, top), theirs, "{creature} in P1's turn");
    }
}

/// Whether `p` could cast `card` now.
fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    crate::r_s21_common::castable(t, p, card)
}

#[test]
fn ranger_classs_level_2_ability_triggers_once_per_combat() {
    cr!("508.3d", "603.2c");
    ruling!(
        "Ranger Class",
        "The triggered ability Ranger Class gets at level 2 can trigger only once per combat, no matter how many creatures you attack with."
    );
    let mut t = TestGame::new(2);
    ranger_class(&mut t, 2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    to_combat(&mut t, P0);
    t.answer_targets(P0, &[obj(a)]);
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "Whenever you attack"), 1);
    t.resolve_all();
    assert_eq!(t.counters(a, "+1/+1") + t.counters(b, "+1/+1"), 1);
}

#[test]
fn a_token_copy_of_radiant_solar_triggers_only_its_own_ability() {
    cr!("707.2", "603.6a", "111.1");
    ruling!(
        "Radiant Solar",
        "A token copy of Radiant Solar entering the battlefield will cause its own ability to trigger, but not that of any other Radiant Solars."
    );
    supported("Cackling Counterpart");
    let mut t = TestGame::new(2);
    let solar = t.battlefield(P0, "Radiant Solar");
    cast_new(&mut t, P0, "Cackling Counterpart", &[obj(solar)]);
    t.resolve();
    t.settle();
    assert_eq!(named_by(&t, P0, "Radiant Solar").len(), 2);
    assert_eq!(triggers_on_stack(&t, "venture into the dungeon"), 1);
    assert!(
        crate::r_s25_common::abilities_from(&t, solar).is_empty(),
        "the token's own ability, not the original's"
    );
}

/// The abilities and spells on the stack Dissonant Wave (P0's, cast from hand) could
/// target now, or None if P0 couldn't cast it.
fn wave_targets(t: &mut TestGame) -> Option<Vec<Entity>> {
    supported("Emerald Dragon // Dissonant Wave");
    t.lands(P0, "Forest", 3);
    let card = t.hand(P0, "Emerald Dragon // Dissonant Wave");
    let saved = t.g.turn.priority;
    t.g.turn.priority = Some(P0);
    t.g.recompute();
    let ok = t.g.legal_actions(P0).iter().any(|a| {
        matches!(a, mtg_engine::decision::Action::Cast { card: c, method }
            if *c == card && *method == ADVENTURE)
    });
    t.g.turn.priority = saved;
    if !ok {
        return None;
    }
    let from = t.asked().len();
    let spell = t.cast(P0, card).method(ADVENTURE).try_go();
    let cands = t.asked()[from..].iter().find_map(|(_, d)| match d {
        Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
        _ => None,
    });
    let _ = spell;
    cands
}

/// P0 casts Dissonant Wave targeting the ability `ab` on the stack, and it resolves.
fn wave(t: &mut TestGame, ab: ObjectId) {
    supported("Emerald Dragon // Dissonant Wave");
    t.lands(P0, "Forest", 3);
    let card = t.hand(P0, "Emerald Dragon // Dissonant Wave");
    t.answer_targets(P0, &[obj(ab)]);
    t.cast(P0, card).method(ADVENTURE).go();
    t.resolve();
}

#[test]
fn dissonant_wave_counters_activated_abilities_including_equip_and_cycling() {
    cr!("602.1", "702.6a", "702.29a");
    ruling!(
        "Emerald Dragon // Dissonant Wave",
        "Activated abilities are written in the form “Cost: Effect.” Some keyword abilities, such as equip, are activated abilities"
    );
    ruling!(
        "Emerald Dragon // Dissonant Wave",
        "Activated and triggered abilities from noncreature sources include abilities of noncreature cards that can be activated or triggered from other zones, such as cycling abilities."
    );
    // P1 equips Bonesplitter in its main phase; P0 counters the equip ability.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let eq = t.battlefield(P1, "Bonesplitter");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Wastes", 1);
    let ab = activate_with(&mut t, P1, eq, "Equip", &[obj(bears)]).expect("on the stack");
    wave(&mut t, ab);
    t.resolve_all();
    assert_eq!(crate::r_s06_common::attached_to(&t, eq), None);
    assert_eq!(t.pt(bears), (2, 2));
    // P1 cycles Molten Frame (an instant card in its hand); P0 counters it: no card.
    let mut t = TestGame::new(2);
    let frame = t.hand(P1, "Molten Frame");
    t.lands(P1, "Wastes", 2);
    let hand = t.hand_size(P1);
    let ab = activate_with(&mut t, P1, frame, "Cycling", &[]).expect("on the stack");
    assert!(t.in_graveyard(P1, "Molten Frame"));
    wave(&mut t, ab);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand - 1);
}

#[test]
fn dissonant_wave_counters_triggered_abilities_of_noncreature_sources_only() {
    cr!("603.1", "113.3c", "702.108a");
    ruling!(
        "Emerald Dragon // Dissonant Wave",
        "Triggered abilities use the word “when,” “whenever,” or “at.”"
    );
    // P1's Witch Hunt: "At the beginning of your upkeep, this enchantment deals 4 damage
    // to you." P0 counters it in P1's upkeep.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Witch Hunt");
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    let ab = *t.g.stack.last().expect("the upkeep trigger");
    wave(&mut t, ab);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // A prowess trigger (Monastery Swiftspear, a creature) can't be targeted.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Monastery Swiftspear");
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert!(wave_targets(&mut t).is_none(), "nothing to target");
}

#[test]
fn replacement_effects_and_mana_abilities_cant_be_targeted() {
    cr!("605.3b", "614.1c", "605.1a");
    ruling!(
        "Emerald Dragon // Dissonant Wave",
        "Abilities that create replacement effects, such as a permanent entering the battlefield tapped or with counters on it, can't be targeted."
    );
    ruling!(
        "Emerald Dragon // Dissonant Wave",
        "Mana abilities can't be targeted."
    );
    // P1 plays a Guildgate (enters tapped) and taps a Sol Ring for mana (an artifact's
    // mana ability): nothing goes on the stack, and Dissonant Wave has nothing to target.
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P1, "Sol Ring");
    t.set_step(P1, Step::PrecombatMain);
    let gate = t.hand(P1, "Azorius Guildgate");
    t.play_land(P1, gate).unwrap();
    t.settle();
    assert!(t.obj_now(gate).tapped);
    activate_containing(&mut t, P1, ring, "Add").unwrap();
    assert_eq!(t.stack_len(), 0);
    assert!(wave_targets(&mut t).is_none());
}

#[test]
fn a_countered_next_end_step_trigger_doesnt_trigger_again() {
    cr!("603.7b", "701.6a");
    ruling!(
        "Emerald Dragon // Dissonant Wave",
        "If you counter a delayed triggered ability that triggers at the beginning of the “next” occurrence of a specified step or phase, that ability won't trigger again the following time that phase or step occurs."
    );
    // P1's Through the Breach puts Hill Giant onto the battlefield ("Sacrifice that
    // creature at the beginning of the next end step."); P0 counters the delayed trigger.
    supported("Through the Breach");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.hand(P1, "Hill Giant");
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(true));
    let giant_card = t.g.player(P1).hand[0];
    t.answer_choose(P1, &[obj(giant_card)]);
    cast_new(&mut t, P1, "Through the Breach", &[]);
    t.resolve_all();
    let giant = named_by(&t, P1, "Hill Giant")[0];
    t.advance_to(P1, Step::End);
    t.settle();
    let ab = *t.g.stack.last().expect("the delayed trigger");
    wave(&mut t, ab);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    // The next end steps: no trigger.
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.advance_to(P1, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(giant));
}

#[test]
fn green_dragons_poison_breath_outlasts_it_and_isnt_deathtouch() {
    cr!("603.7a", "702.19b", "702.2c", "510.1c");
    ruling!(
        "Green Dragon",
        "Green Dragon's Poison Breath ability continues to function until end of turn even if Green Dragon is destroyed or otherwise leaves the battlefield."
    );
    ruling!(
        "Green Dragon",
        "Green Dragon's Poison Breath ability is different than deathtouch. Notably, creatures still need to deal damage to a blocking creature equal to its toughness"
    );
    supported("Green Dragon");
    let mut t = TestGame::new(2);
    let dragon = t.enter(P0, "Green Dragon");
    t.resolve_all();
    destroy(&mut t, dragon);
    assert!(!t.on_battlefield(dragon));
    let wurm = t.battlefield(P1, "Craw Wurm");
    // Lightning Bolt deals 3 damage to the Craw Wurm (6/4).
    cast_new(&mut t, P0, "Lightning Bolt", &[obj(wurm)]);
    t.resolve_all();
    assert!(!t.on_battlefield(wurm), "destroyed by Poison Breath");
    // P0's Colossal Dreadmaw (6/6 trample) is blocked by P1's Hill Giant (3/3): lethal
    // damage to the Giant is still 3, not 1.
    let mut t = TestGame::new(2);
    t.enter(P0, "Green Dragon");
    t.resolve_all();
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let giant = t.battlefield(P1, "Hill Giant");
    to_combat(&mut t, P0);
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![1, 5]));
    t.attack(&[(maw, Entity::Player(P1))], &[(giant, maw)]);
    let lethal = t.asked()[from..].iter().find_map(|(_, d)| match d {
        Decision::AssignCombatDamage { lethal, .. } => Some(lethal.clone()),
        _ => None,
    });
    assert_eq!(lethal.as_ref().and_then(|l| l.first()).copied(), Some(3));
    assert_eq!(t.life(P1), 17, "only 3 tramples over");
    assert!(!t.on_battlefield(giant));
}

#[test]
fn explosive_crystals_division_is_chosen_on_casting_and_kept() {
    cr!("601.2d", "608.2b");
    ruling!(
        "Amethyst Dragon // Explosive Crystal",
        "You choose how many targets Explosive Crystal has and how the damage is divided as you put the spell on the stack. Each target must receive at least 1 damage."
    );
    ruling!(
        "Amethyst Dragon // Explosive Crystal",
        "If some of the targets are illegal as Explosive Crystal tries to resolve, the original division of damage still applies and the damage that would have been dealt to the illegal targets is lost."
    );
    supported("Amethyst Dragon // Explosive Crystal");
    // 3 to the Bears and 1 to P1, divided as it's cast; the Bears leave in response: P1
    // is still dealt 1 (not the Bears' 3, nor all 4).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 5);
    let card = t.hand(P0, "Amethyst Dragon // Explosive Crystal");
    t.answer_targets(P0, &[obj(bears), Entity::Player(P1)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![3, 1]));
    let from = t.asked().len();
    t.cast(P0, card).method(ADVENTURE).go();
    // The division was asked as it was cast, at least 1 each.
    let min = t.asked()[from..].iter().find_map(|(_, d)| match d {
        Decision::Divide { min_each, .. } => Some(*min_each),
        _ => None,
    });
    assert_eq!(min, Some(1));
    bounce(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P1), 19, "the Bears' 3 damage is lost");
    // A division giving a target nothing isn't accepted: both targets are dealt damage.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 5);
    let card = t.hand(P0, "Amethyst Dragon // Explosive Crystal");
    t.answer_targets(P0, &[obj(bears), Entity::Player(P1)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![4, 0]));
    t.cast(P0, card).method(ADVENTURE).go();
    t.resolve_all();
    assert!(t.life(P1) < 20);
    assert!(!t.on_battlefield(bears) || t.obj_now(bears).damage > 0);
}

/// P0 gets `n` of Zariel's emblems (each from a Zariel at 6 loyalty using its −6).
fn zariel_emblems(t: &mut TestGame, n: usize) {
    supported("Zariel, Archduke of Avernus");
    for _ in 0..n {
        let z = t.battlefield(P0, "Zariel, Archduke of Avernus");
        t.g.objects[z.0 as usize]
            .counters
            .insert("loyalty".into(), 6);
        activate_with(t, P0, z, "emblem", &[]);
        t.resolve_all();
    }
}

/// Runs the game from the current point until the beginning of combat step or the
/// postcombat main phase of P0's turn begins (whichever comes first), returning which.
fn next_combat_or_main(t: &mut TestGame) -> Step {
    let ok = t.g.run_until(10_000, |g| {
        matches!(g.turn.step, Step::BeginningOfCombat | Step::PostcombatMain)
            && g.turn.stage == Stage::Priority
    });
    assert!(ok);
    t.g.turn.step
}

#[test]
fn zariels_emblem_untaps_any_creature_and_adds_a_combat_right_after() {
    cr!("500.8");
    ruling!(
        "Zariel, Archduke of Avernus",
        "The ability of the emblem can target any creature you control, not just one that's tapped."
    );
    ruling!(
        "Zariel, Archduke of Avernus",
        "There is no main phase between the combat phases. You move from the first combat phase directly into the second combat phase."
    );
    let mut t = TestGame::new(2);
    zariel_emblems(&mut t, 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let idle = t.battlefield(P0, "Hill Giant");
    // The untapped Hill Giant (it didn't attack) is a legal target.
    t.answer_targets(P0, &[obj(idle)]);
    to_combat(&mut t, P0);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::EndOfCombat);
    t.settle();
    let ab = *t.g.stack.last().expect("the emblem's trigger");
    assert_eq!(crate::r_s25_common::targets_of(&t, ab), vec![obj(idle)]);
    t.resolve_all();
    assert_eq!(next_combat_or_main(&mut t), Step::BeginningOfCombat);
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn zariels_emblem_with_an_illegal_target_gives_no_combat() {
    cr!("608.2b", "500.8");
    ruling!(
        "Zariel, Archduke of Avernus",
        "If the target of the emblem's ability becomes an illegal target by the time the ability tries to resolve, the ability won't resolve and none of its effects will happen. You won't get an additional combat phase."
    );
    let mut t = TestGame::new(2);
    zariel_emblems(&mut t, 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    to_combat(&mut t, P0);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::EndOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    bounce(&mut t, bears);
    t.resolve_all();
    assert_eq!(next_combat_or_main(&mut t), Step::PostcombatMain);
}

#[test]
fn several_zariel_emblems_give_several_combats_with_no_untap_between_them() {
    cr!("500.8", "603.2");
    ruling!(
        "Zariel, Archduke of Avernus",
        "If you have more than one of Zariel's emblem, the abilities will each trigger at the end of the first combat phase on your turn. You can choose the same creature as the target of each ability. Those targets will untap before any additional combats. No creatures will untap between the first additional combat and the second additional combat"
    );
    let mut t = TestGame::new(2);
    zariel_emblems(&mut t, 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(bears)]);
    to_combat(&mut t, P0);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::EndOfCombat);
    t.settle();
    assert_eq!(
        t.stack_len(),
        2,
        "both trigger at the end of the first combat"
    );
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
    // First additional combat: the Bears attack again.
    assert_eq!(next_combat_or_main(&mut t), Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    // Second additional combat: the Bears are still tapped and can't attack.
    assert_eq!(next_combat_or_main(&mut t), Step::BeginningOfCombat);
    assert!(t.obj_now(bears).tapped);
    assert!(!legal_attack(&mut t, &[(bears, Entity::Player(P1))]));
    assert_eq!(t.life(P1), 16);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(next_combat_or_main(&mut t), Step::PostcombatMain);
}

#[test]
fn sculpted_sunburst_chooses_on_resolution_without_targeting() {
    cr!("608.2c", "115.10", "702.11b");
    ruling!(
        "Sculpted Sunburst",
        "Sculpted Sunburst doesn't target any of the chosen creatures. All of the choices are made as Sculpted Sunburst resolves."
    );
    supported("Sculpted Sunburst");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let scout = t.battlefield(P1, "Gladecover Scout");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let from = t.asked().len();
    cast_new(&mut t, P0, "Sculpted Sunburst", &[]);
    assert!(t.asked()[from..].iter().all(|(_, d)| !matches!(
        d,
        Decision::ChooseTargets { .. } | Decision::ChooseEntities { .. }
    )));
    // P0 chooses the Hill Giant; P1 chooses its hexproof Gladecover Scout; the rest are
    // exiled.
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_choose(P1, &[obj(scout)]);
    let from = t.asked().len();
    t.resolve_all();
    let choosers: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(choosers, vec![P0, P1]);
    assert!(t.on_battlefield(giant) && t.on_battlefield(scout));
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(elves));
}
