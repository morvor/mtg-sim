//! Rulings batch P139 — sacrifices chosen and performed as the spell or ability resolves:
//! who chooses and when (Strategic Betrayal, Archfiend of Depravity, Sheoldred, Dystopia,
//! Cataclysm, Limited Resources, Tectonic Hellion, Argothian Wurm), targets that became
//! illegal (Retribution, Barrin's Spite), "sacrifice unless" choices (Archfiend of Spite,
//! Killing Wave, Fade Away, Mana Vortex), effects that still do the rest when nothing can
//! be sacrificed, and last known information of the sacrificed creature.

use crate::r_p137_common::{choosers_since, is_choose, obj};
use crate::r_s01_common::{give_mana_for, supported, watch};
use crate::r_s02_common::destroy;
use crate::r_s04_common::add_mana;
use crate::r_s06_common::{damage, give_control};
use crate::r_s25_common::cast_new;
use crate::r_s30_common::set_life;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

fn count_of(t: &TestGame, p: PlayerId, ty: CardType) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is(ty))
        .count()
}

fn permanents_of(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents().filter(|o| o.controller == p).count()
}

/// The candidates of the first object choice `p` was asked since `from`.
fn first_choice(t: &TestGame, p: PlayerId, from: usize) -> Option<Vec<Entity>> {
    t.asked()[from..].iter().find_map(|(q, d)| match d {
        Decision::ChooseEntities { candidates, .. } if *q == p => Some(candidates.clone()),
        _ => None,
    })
}

// ---------------------------------------------------------------------------------------
// Who chooses, and when
// ---------------------------------------------------------------------------------------

#[test]
fn strategic_betrayal_opponent_chooses_on_resolution_or_just_exiles_graveyard() {
    cr!("608.2c", "701.21a");
    ruling!(
        "Strategic Betrayal",
        "The opponent chooses which creature to exile as Strategic Betrayal resolves. If they don’t control any creatures at that time, they simply exile their graveyard."
    );
    supported("Strategic Betrayal");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.graveyard(P1, "Island");
    t.answer_choose(P1, &[obj(b)]);
    let from = t.asked().len();
    cast_new(&mut t, P0, "Strategic Betrayal", &[Entity::Player(P1)]);
    assert!(first_choice(&t, P1, from).is_none(), "nothing chosen on casting");
    t.resolve_all();
    assert_eq!(first_choice(&t, P1, from).map(|c| c.len()), Some(2));
    assert!(t.on_battlefield(a) && !t.on_battlefield(b));
    assert_eq!(t.graveyard_size(P1), 0);
    // No creatures: the graveyard is still exiled.
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Island");
    t.graveyard(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Strategic Betrayal", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 0);
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn archfiend_of_depravity_spares_up_to_two_chosen_without_targeting() {
    cr!("608.2c", "115.1", "702.18a");
    ruling!(
        "Archfiend of Depravity",
        "The opponent chooses which creatures to spare, if any, as the ability resolves. This choice doesn't target the creatures."
    );
    ruling!(
        "Archfiend of Depravity",
        "The opponent may choose to spare one or no creatures. If the player doesn't choose any creatures, they will sacrifice all creatures they control."
    );
    supported("Archfiend of Depravity");
    // Two spared, one with shroud.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Archfiend of Depravity");
    let shroud = t.battlefield(P1, "Gladecover Scout");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    // One creature at a time.
    t.answer_choose(P1, &[obj(shroud)]);
    t.answer_choose(P1, &[obj(a)]);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(shroud) && t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
    // One spared.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Archfiend of Depravity");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let c = t.battlefield(P1, "Savannah Lions");
    t.answer_choose(P1, &[obj(c)]);
    t.answer_choose(P1, &[]);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(c) && !t.on_battlefield(a) && !t.on_battlefield(b));
    // None spared: all sacrificed.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Archfiend of Depravity");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P1, &[]);
    t.answer_choose(P1, &[]);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    assert_eq!(count_of(&t, P1, CardType::Creature), 0);
}

#[test]
fn upkeep_edicts_choose_as_the_trigger_resolves() {
    cr!("608.2c", "603.3");
    ruling!(
        "Sheoldred, Whispering One",
        "The opponent whose upkeep it is chooses a creature to sacrifice when Sheoldred's second triggered ability resolves."
    );
    supported("Sheoldred, Whispering One");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sheoldred, Whispering One");
    let a = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // A creature that arrives with the trigger on the stack can be chosen.
    let late = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P1, &[obj(late)]);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(choosers_since(&t, from), vec![P1]);
    assert!(t.on_battlefield(a) && !t.on_battlefield(late));

    ruling!(
        "Dystopia",
        "The permanent is chosen and sacrificed during resolution."
    );
    supported("Dystopia");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dystopia");
    let lions = t.battlefield(P1, "Savannah Lions");
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P1, &[obj(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(lions) && !t.on_battlefield(bears));
}

#[test]
fn cataclysm_and_limited_resources_choose_on_resolution() {
    cr!("608.2c", "101.4");
    ruling!(
        "Cataclysm",
        "The permanents to save are chosen on resolution. You do not choose them when casting the spell."
    );
    supported("Cataclysm");
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Cataclysm");
    let a = t.battlefield(P1, "Grizzly Bears");
    let card = t.hand(P0, "Cataclysm");
    let from = t.asked().len();
    t.cast(P0, card).go();
    assert!(first_choice(&t, P0, from).is_none() && first_choice(&t, P1, from).is_none());
    // A creature that arrives in response can be kept.
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P1, &[obj(b)]);
    t.resolve_all();
    assert!(t.on_battlefield(b) && !t.on_battlefield(a));

    ruling!(
        "Limited Resources",
        "The sacrificed lands are chosen during resolution. The current player chooses first."
    );
    supported("Limited Resources");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P0, "Forest", 6);
    t.lands(P1, "Island", 6);
    let from = t.asked().len();
    t.enter(P0, "Limited Resources");
    t.resolve_all();
    let mut order = choosers_since(&t, from);
    order.dedup();
    assert_eq!(order, vec![P1, P0]);
    assert_eq!(count_of(&t, P0, CardType::Land), 5);
    assert_eq!(count_of(&t, P1, CardType::Land), 5);
}

#[test]
fn tectonic_hellion_decides_who_controls_the_most_lands_on_resolution() {
    cr!("608.2c", "508.1m");
    ruling!(
        "Tectonic Hellion",
        "The player or players who will sacrifice lands is determined only as Tectonic Hellion’s triggered ability begins to resolve."
    );
    supported("Tectonic Hellion");
    let mut t = TestGame::new(2);
    let hellion = t.battlefield(P0, "Tectonic Hellion");
    t.lands(P0, "Mountain", 2);
    t.lands(P1, "Island", 3);
    crate::r_s01_common::attack_with(&mut t, &[(hellion, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    // With the trigger on the stack, P0 gets two more lands: now P0 has the most.
    t.lands(P0, "Mountain", 2);
    t.resolve_all();
    assert_eq!(count_of(&t, P0, CardType::Land), 2);
    assert_eq!(count_of(&t, P1, CardType::Land), 3);
}

#[test]
fn argothian_wurm_each_player_in_turn_order_may_sacrifice_a_land() {
    cr!("101.4", "608.2c");
    ruling!(
        "Argothian Wurm",
        "When Argothian Wurm enters, first the active player gets the option to sacrifice a land. If they decline, the next player in turn order gets the option. If a player elects to sacrifice a land, Argothian Wurm is put on top of its owner's library, but then all remaining players still get the option. If all players decline, then nothing happens and Argothian Wurm stays on the battlefield."
    );
    supported("Argothian Wurm");
    let mut t = TestGame::new(3);
    for p in [P0, P1, P2] {
        t.lands(p, "Forest", 2);
    }
    t.answer_yes(P0, false);
    t.answer_yes(P1, true);
    t.answer_yes(P2, true);
    let wurm = t.enter(P0, "Argothian Wurm");
    let from = t.asked().len();
    t.resolve_all();
    let askers: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(askers, vec![P0, P1, P2]);
    assert_eq!(count_of(&t, P0, CardType::Land), 2);
    assert_eq!(count_of(&t, P1, CardType::Land), 1);
    assert_eq!(count_of(&t, P2, CardType::Land), 1);
    let now = t.g.current(wurm);
    assert_eq!(t.zone(now), Zone::Library(P0));
    assert_eq!(t.g.players[0].library.last().copied(), Some(now));
    // Everybody declines: it stays.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    t.lands(P1, "Forest", 1);
    t.answer_yes(P0, false);
    t.answer_yes(P1, false);
    let wurm = t.enter(P0, "Argothian Wurm");
    t.resolve_all();
    assert!(t.on_battlefield(wurm));
}

// ---------------------------------------------------------------------------------------
// Targets that became illegal
// ---------------------------------------------------------------------------------------

#[test]
fn choose_and_sacrifice_one_of_two_targets_when_only_one_is_still_legal() {
    cr!("608.2b", "701.21a");
    ruling!(
        "Retribution",
        "The player chooses which one to sacrifice as Retribution resolves. If only one of the targets is still legal at that time, then the player must choose that one to sacrifice."
    );
    supported("Retribution");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Retribution", &[obj(a), obj(b)]);
    destroy(&mut t, a);
    // P1 would rather keep the Hill Giant, but it's the only legal target left.
    t.answer_choose(P1, &[obj(a)]);
    t.resolve_all();
    assert!(!t.on_battlefield(b));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    ruling!(
        "Barrin's Spite",
        "The player chooses which one to sacrifice as the spell resolves. If only one of the targets is still legal at that time, then the player must choose that one to sacrifice."
    );
    supported("Barrin's Spite");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Barrin's Spite", &[obj(a), obj(b)]);
    destroy(&mut t, a);
    t.answer_choose(P1, &[obj(a)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(!t.in_hand(P1, "Hill Giant"));
    // Both legal: P1 chooses, the other returns to hand.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Barrin's Spite", &[obj(a), obj(b)]);
    t.answer_choose(P1, &[obj(a)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears") && t.in_hand(P1, "Hill Giant"));
}

#[test]
fn ashling_sacrifices_the_target_if_its_player_still_controls_it() {
    cr!("701.21a", "702.12b", "115.1");
    ruling!(
        "Ashling, the Extinguisher",
        "The target of the ability is chosen when Ashling's ability is put on the stack. When the ability resolves, the player dealt combat damage by Ashling sacrifices that creature if they still control it. Neither regeneration nor indestructible can save a creature from being sacrificed."
    );
    supported("Ashling, the Extinguisher");
    let mut t = TestGame::new(2);
    let ashling = t.battlefield(P0, "Ashling, the Extinguisher");
    let wall = t.battlefield(P1, "Darksteel Myr");
    t.answer_targets(P0, &[obj(wall)]);
    t.attack(&[(ashling, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert!(!t.on_battlefield(wall), "indestructible doesn't stop a sacrifice");
    // Control changed before it resolves: nothing is sacrificed.
    let mut t = TestGame::new(2);
    let ashling = t.battlefield(P0, "Ashling, the Extinguisher");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(ashling, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    give_control(&mut t, bears, P0);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P0);
}

#[test]
fn mercy_killing_sacrificed_by_its_controller_even_if_indestructible() {
    cr!("701.21a", "702.12b");
    ruling!(
        "Mercy Killing",
        "You target the creature that will be sacrificed when you cast Mercy Killing, then its controller (who may or may not be you) sacrifices it when Mercy Killing resolves. Because this is a sacrifice, neither regeneration nor indestructible will save the creature."
    );
    supported("Mercy Killing");
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P1, "Darksteel Myr");
    cast_new(&mut t, P0, "Mercy Killing", &[obj(myr)]);
    t.resolve_all();
    assert!(!t.on_battlefield(myr));
    let mine = t.battlefield(P0, "Hill Giant");
    cast_new(&mut t, P0, "Mercy Killing", &[obj(mine)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    let elves = |p| {
        t.g.permanents()
            .filter(|o| o.controller == p && o.is_token())
            .count()
    };
    assert_eq!(elves(P0), 3);
    assert_eq!(elves(P1), 0, "Darksteel Myr has power 0");
}

// ---------------------------------------------------------------------------------------
// Archfiend of Spite: sacrifice or lose life
// ---------------------------------------------------------------------------------------

/// P0's Archfiend of Spite is dealt `n` damage by P1's Prodigal Pyromancer; returns the
/// Archfiend and the source (the trigger is on the stack).
fn spite(t: &mut TestGame, n: i32) -> (ObjectId, ObjectId) {
    let fiend = t.battlefield(P0, "Archfiend of Spite");
    let src = t.battlefield(P1, "Prodigal Pyromancer");
    damage(t, src, n, fiend);
    (fiend, src)
}

#[test]
fn archfiend_of_spite_all_permanents_or_all_life() {
    cr!("603.2", "701.21a");
    ruling!(
        "Archfiend of Spite",
        "The source's controller must either sacrifice permanents or lose life. They can't choose to sacrifice some permanents and lose some life."
    );
    supported("Archfiend of Spite");
    // P1 sacrifices three permanents (the source among the candidates) and loses nothing.
    let mut t = TestGame::new(2);
    let lands = t.lands(P1, "Island", 3);
    let (_, src) = spite(&mut t, 3);
    assert_eq!(t.stack_len(), 1);
    t.answer_yes(P1, true);
    t.answer_choose(P1, &lands.iter().map(|l| obj(*l)).collect::<Vec<_>>());
    t.answer(P1, DecisionKind::OptionalCost, Answer::Bool(true));
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(count_of(&t, P1, CardType::Land), 0);
    assert!(t.on_battlefield(src));
    // P1 declines: loses 3 life, keeps everything.
    let mut t = TestGame::new(2);
    t.lands(P1, "Island", 3);
    spite(&mut t, 3);
    t.answer_yes(P1, false);
    t.answer(P1, DecisionKind::OptionalCost, Answer::Bool(false));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(permanents_of(&t, P1), 4);
    // Two permanents can't pay for three: all the life is lost, nothing is sacrificed.
    let mut t = TestGame::new(2);
    t.lands(P1, "Island", 1);
    spite(&mut t, 3);
    t.answer_yes(P1, true);
    t.answer(P1, DecisionKind::OptionalCost, Answer::Bool(true));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(permanents_of(&t, P1), 2);
}

#[test]
fn archfiend_of_spite_may_decline_even_when_life_cant_be_lost() {
    cr!("119.10", "701.21a");
    ruling!(
        "Archfiend of Spite",
        "The player may always choose not to sacrifice permanents, even if they will end up losing more life than they have or if an effect (such as that of Platinum Emperion) says that they can't lose life."
    );
    supported("Archfiend of Spite");
    // Platinum Emperion: P1 declines and loses nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Platinum Emperion");
    t.lands(P1, "Island", 3);
    spite(&mut t, 3);
    t.answer_yes(P1, false);
    t.answer(P1, DecisionKind::OptionalCost, Answer::Bool(false));
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(permanents_of(&t, P1), 5);
    // At 2 life, P1 declines and loses the game.
    let mut t = TestGame::new(2);
    set_life(&mut t, P1, 2);
    t.lands(P1, "Island", 3);
    spite(&mut t, 3);
    t.answer_yes(P1, false);
    t.answer(P1, DecisionKind::OptionalCost, Answer::Bool(false));
    t.resolve_all();
    assert!(t.has_lost(P1));
}

#[test]
fn archfiend_of_spite_uses_the_sources_controller_as_the_trigger_resolves() {
    cr!("603.2", "608.2h");
    ruling!(
        "Archfiend of Spite",
        "The player who loses life or permanents is the player who controls the source as Archfiend of Spite's triggered ability resolves. In some rare cases, this may not be the same player who controlled the source as it dealt damage. If the source is no longer in the zone it was in as it dealt damage, the source's last known information is used to determine who controlled it."
    );
    supported("Archfiend of Spite");
    // P2 takes control of the source with the trigger on the stack: P2 loses the life.
    let mut t = TestGame::new(3);
    let (_, src) = spite(&mut t, 3);
    assert_eq!(t.stack_len(), 1);
    give_control(&mut t, src, P2);
    t.answer_yes(P2, false);
    t.answer(P2, DecisionKind::OptionalCost, Answer::Bool(false));
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P2), 17);
    // The source left the battlefield: its last controller (P1) loses the life.
    let mut t = TestGame::new(3);
    let (_, src) = spite(&mut t, 3);
    destroy(&mut t, src);
    t.answer_yes(P1, false);
    t.answer(P1, DecisionKind::OptionalCost, Answer::Bool(false));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P2), 20);
}

// ---------------------------------------------------------------------------------------
// "Unless" payments
// ---------------------------------------------------------------------------------------

#[test]
fn killing_wave_cant_pay_more_life_than_you_have() {
    cr!("119.4", "118.3");
    ruling!("Killing Wave", "You can’t pay more life than you have.");
    supported("Killing Wave");
    let mut t = TestGame::new(2);
    set_life(&mut t, P1, 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Hill Giant");
    // P1 would pay, but can't pay 3 life with 2.
    t.answer_yes(P1, true);
    t.answer(P1, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    give_mana_for(&mut t, P0, "Killing Wave");
    add_mana(&mut t, P0, ManaType::C, 3);
    let wave = t.hand(P0, "Killing Wave");
    t.cast(P0, wave).x(3).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 2);
    assert!(t.on_battlefield(mine));
    assert_eq!(t.life(P0), 17);
}

#[test]
fn fade_away_pays_with_mana_abilities_during_resolution_and_can_mix() {
    cr!("608.2c", "605.3a", "118.3");
    ruling!(
        "Fade Away",
        "You can activate mana abilities during the resolution if you decide to pay mana."
    );
    ruling!(
        "Fade Away",
        "You can choose to pay some mana and sacrifice some permanents if you have more than one creature."
    );
    supported("Fade Away");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let island = t.battlefield(P1, "Island");
    let ring = t.battlefield(P1, "Sol Ring");
    // For the first creature P1 pays {1} by tapping the Island during resolution; for the
    // second P1 doesn't pay and sacrifices the Sol Ring.
    t.answer_yes(P1, true);
    t.answer(P1, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_yes(P1, false);
    t.answer(P1, DecisionKind::OptionalCost, Answer::Bool(false));
    t.answer_choose(P1, &[obj(ring)]);
    give_mana_for(&mut t, P0, "Fade Away");
    let spell = t.hand(P0, "Fade Away");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.obj_now(island).tapped, "tapped for mana while Fade Away resolved");
    assert!(!t.on_battlefield(ring));
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
}

#[test]
fn mana_vortex_cast_trigger_resolves_first_sacrifice_a_land_or_countered() {
    cr!("603.2", "118.12");
    ruling!(
        "Mana Vortex",
        "When you cast Mana Vortex, its first ability triggers and goes on the stack on top of the Mana Vortex spell. The triggered ability resolves first. When it resolves, you either sacrifice a land or you counter Mana Vortex."
    );
    supported("Mana Vortex");
    for pay in [true, false] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 2);
        add_mana(&mut t, P0, ManaType::U, 2);
        add_mana(&mut t, P0, ManaType::C, 1);
        let vortex = t.hand(P0, "Mana Vortex");
        t.cast(P0, vortex).go();
        t.settle();
        assert_eq!(t.stack_len(), 2);
        assert_eq!(
            t.g.stack.first().copied(),
            Some(t.g.current(vortex)),
            "the spell is below"
        );
        t.answer_yes(P0, pay);
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(pay));
        t.resolve();
        assert_eq!(t.stack_len(), if pay { 1 } else { 0 });
        t.resolve_all();
        assert_eq!(count_of(&t, P0, CardType::Land), if pay { 1 } else { 2 });
        assert_eq!(
            t.named_on_battlefield("Mana Vortex").len(),
            usize::from(pay)
        );
    }
}

#[test]
fn pendrell_flux_requires_the_colored_mana_of_the_cost() {
    cr!("118.3", "202.1");
    ruling!(
        "Pendrell Flux",
        "You have to even match the color requirements of the mana cost."
    );
    supported("Pendrell Flux");
    for (land, kept) in [("Island", false), ("Forest", true)] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let flux = t.battlefield(P0, "Pendrell Flux");
        t.g.attach(flux, Entity::Object(bears));
        t.lands(P1, "Island", 1);
        t.lands(P1, land, 1);
        t.answer_yes(P1, true);
        t.answer(P1, DecisionKind::OptionalCost, Answer::Bool(true));
        t.advance_to(P1, Step::Upkeep);
        t.resolve_all();
        assert_eq!(t.on_battlefield(bears), kept, "{land}");
    }
}

// ---------------------------------------------------------------------------------------
// The rest of the effect happens even if nothing can be sacrificed
// ---------------------------------------------------------------------------------------

#[test]
fn the_rest_happens_when_nothing_can_be_sacrificed() {
    cr!("608.2c", "701.21a");
    ruling!(
        "Structural Collapse",
        "The target player needn’t control an artifact or a land. Structural Collapse will still deal 2 damage to that player."
    );
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Structural Collapse", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);

    ruling!(
        "Undercity's Embrace",
        "You can still gain 4 life even if the target opponent can’t sacrifice a creature."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Craw Wurm");
    cast_new(&mut t, P0, "Undercity's Embrace", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    ruling!(
        "Undercity's Embrace",
        "You gain only 4 life, even if you control more than one creature with power 4 or greater."
    );
    t.battlefield(P0, "Craw Wurm");
    cast_new(&mut t, P0, "Undercity's Embrace", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 28);

    ruling!(
        "Foul-Tongue Invocation",
        "You may cast Foul-Tongue Invocation targeting a player who controls no creatures. If you qualify for the “Dragon bonus,” you’ll just gain 4 life."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shivan Dragon");
    cast_new(&mut t, P0, "Foul-Tongue Invocation", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);

    ruling!(
        "Priest of Forgotten Gods",
        "You may target players who can't sacrifice a creature. Those players still lose 2 life."
    );
    let mut t = TestGame::new(3);
    let priest = t.battlefield(P0, "Priest of Forgotten Gods");
    let s1 = t.battlefield(P0, "Grizzly Bears");
    let s2 = t.battlefield(P0, "Savannah Lions");
    let bears = t.battlefield(P2, "Grizzly Bears");
    t.answer_choose(P0, &[obj(s1), obj(s2)]);
    t.answer_targets(P0, &[Entity::Player(P1), Entity::Player(P2)]);
    t.activate(P0, priest, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (18, 18));
    assert!(!t.on_battlefield(bears));

    ruling!(
        "Abzan Advantage",
        "You can target any player with Abzan Advantage, even if that player doesn't control an enchantment."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Abzan Advantage", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);

    ruling!(
        "Clip Wings",
        "You can cast Clip Wings even if some (or all) of your opponents don’t control a creature with flying. Those who do must each choose a flying creature to sacrifice, and those who don’t are unaffected."
    );
    let mut t = TestGame::new(3);
    let bird = t.battlefield(P1, "Birds of Paradise");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P2, "Grizzly Bears");
    cast_new(&mut t, P0, "Clip Wings", &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(bird));
    assert!(t.on_battlefield(b1) && t.on_battlefield(b2));

    ruling!(
        "Liliana's Triumph",
        "You can cast Liliana's Triumph even if some or all of your opponents will be unable to sacrifice a creature. If you control a Liliana planeswalker, they'll still discard a card, even if they couldn't sacrifice a creature."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Liliana of the Veil");
    t.hand(P1, "Island");
    cast_new(&mut t, P0, "Liliana's Triumph", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);

    ruling!(
        "Sorin, Solemn Visitor",
        "When the emblem’s ability resolves, if the player controls no creatures, the player won’t sacrifice anything."
    );
    let mut t = TestGame::new(2);
    let sorin = t.battlefield(P0, "Sorin, Solemn Visitor");
    t.g.objects[sorin.0 as usize]
        .counters
        .insert("loyalty".into(), 6);
    t.activate(P0, sorin, 2, &[]).unwrap();
    t.resolve_all();
    let forest = t.battlefield(P1, "Forest");
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(forest));
    assert_eq!(permanents_of(&t, P1), 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(bears), "the emblem works when there is a creature");
}

#[test]
fn priest_of_forgotten_gods_may_have_no_targets() {
    cr!("115.1d", "601.2c");
    ruling!(
        "Priest of Forgotten Gods",
        "You can activate the ability of Priest of Forgotten Gods without choosing any targets if you wish. You'll still add {B}{B} and draw a card."
    );
    supported("Priest of Forgotten Gods");
    let mut t = TestGame::new(2);
    let priest = t.battlefield(P0, "Priest of Forgotten Gods");
    let s1 = t.battlefield(P0, "Grizzly Bears");
    let s2 = t.battlefield(P0, "Savannah Lions");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(s1), obj(s2)]);
    let hand = t.hand_size(P0);
    t.activate(P0, priest, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.g.players[0].mana_pool.count(ManaType::B), 2);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn hellish_rebuke_does_each_part_it_can() {
    cr!("608.2c", "119.10");
    ruling!(
        "Hellish Rebuke",
        "When the triggered ability that Hellish Rebuke grants permanents resolves, if that permanent's controller can't sacrifice it, they will still lose 2 life. Similarly, if they can't lose life, they will still sacrifice the permanent."
    );
    supported("Hellish Rebuke");
    // The permanent is gone before the trigger resolves: P1 still loses 2 life.
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    cast_new(&mut t, P0, "Hellish Rebuke", &[]);
    t.resolve_all();
    crate::r_s30_common::set_life(&mut t, P0, 20);
    deal_to_player(&mut t, pyro, P0);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, pyro);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // P1 can't lose life: the permanent is still sacrificed.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Platinum Emperion");
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    cast_new(&mut t, P0, "Hellish Rebuke", &[]);
    t.resolve_all();
    deal_to_player(&mut t, pyro, P0);
    t.resolve_all();
    assert!(!t.on_battlefield(pyro));
    assert_eq!(t.life(P1), 20);
}

/// `source` deals 1 damage to player `to` (as its own resolving effect would) and the
/// resulting triggers go on the stack.
fn deal_to_player(t: &mut TestGame, source: ObjectId, to: PlayerId) {
    use mtg_engine::ability::{Effect, Sel, Value};
    let mut ctx = mtg_engine::eval::Ctx::new(Some(source), t.obj_now(source).controller);
    ctx.targets = vec![vec![Entity::Player(to)]];
    t.g.exec(
        &Effect::DealDamage {
            source: Sel::This,
            amount: Value::Const(1),
            to: Sel::Target(0),
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.settle();
}

// ---------------------------------------------------------------------------------------
// What the chooser may choose
// ---------------------------------------------------------------------------------------

#[test]
fn edicts_the_player_chooses_which_but_not_whether() {
    cr!("701.21a", "608.2d");
    ruling!(
        "Invoke Despair",
        "The target opponent doesn't choose whether to sacrifice a creature or not, although they do choose which creature they control to sacrifice."
    );
    supported("Invoke Despair");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    // P1 tries to choose nothing: one creature is sacrificed anyway.
    t.answer_choose(P1, &[]);
    let from = t.asked().len();
    cast_new(&mut t, P0, "Invoke Despair", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(first_choice(&t, P1, from).map(|c| c.len()), Some(2));
    assert_eq!(
        usize::from(t.on_battlefield(a)) + usize::from(t.on_battlefield(b)),
        1
    );
    ruling!(
        "Invoke Despair",
        "When you repeat the process, you don't choose a new player. The same player sacrifices each of the three permanents, if able."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Glorious Anthem");
    t.battlefield(P1, "Liliana of the Veil");
    t.battlefield(P2, "Hill Giant");
    t.battlefield(P2, "Intangible Virtue");
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Invoke Despair", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(permanents_of(&t, P1), 0);
    assert_eq!(permanents_of(&t, P2), 2);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P0), hand, "P1 sacrificed all three: no cards drawn");

    ruling!(
        "Dead Drop",
        "The target player chooses which creatures they will sacrifice as Dead Drop resolves. If the player controls only one creature at that time, they will sacrifice it."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Dead Drop", &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let c = t.battlefield(P1, "Savannah Lions");
    t.answer_choose(P1, &[obj(a), obj(c)]);
    cast_new(&mut t, P0, "Dead Drop", &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(!t.on_battlefield(a) && t.on_battlefield(b) && !t.on_battlefield(c));

    ruling!(
        "Mire in Misery",
        "Your opponents choose what to sacrifice from among the creatures and enchantments they control. This means that you don’t decide which type of permanent a player sacrifices, and a player can’t choose to sacrifice an enchantment if they control none."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    t.answer_choose(P1, &[obj(anthem)]);
    let from = t.asked().len();
    cast_new(&mut t, P0, "Mire in Misery", &[]);
    t.resolve_all();
    assert_eq!(first_choice(&t, P1, from).map(|c| c.len()), Some(2));
    assert!(t.on_battlefield(bears) && !t.on_battlefield(anthem));
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Mire in Misery", &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn natural_balance_needn_t_find_every_basic_land() {
    cr!("701.23d");
    ruling!(
        "Natural Balance",
        "You are not required to find all the basic lands you are entitled to."
    );
    supported("Natural Balance");
    let mut t = TestGame::new(2);
    t.lands(P1, "Forest", 2);
    let f = t.library_top(P1, "Forest");
    t.library_top(P1, "Forest");
    t.library_top(P1, "Forest");
    t.answer_choose(P1, &[obj(f)]);
    cast_new(&mut t, P0, "Natural Balance", &[]);
    t.resolve_all();
    assert_eq!(count_of(&t, P1, CardType::Land), 3);
}

#[test]
fn rakdos_the_defiler_sacrifices_half_the_non_demons_rounded_up() {
    cr!("701.21a", "107.1a");
    ruling!(
        "Rakdos the Defiler",
        "To calculate the number of permanents that must be sacrificed, count the number of permanents the affected player controls that don't have the subtype Demon, then divide that number in half and round up. For example, a player who controls eleven non-Demon permanents will need to sacrifice six of them."
    );
    supported("Rakdos the Defiler");
    let mut t = TestGame::new(2);
    let rakdos = t.battlefield(P0, "Rakdos the Defiler");
    t.battlefield(P0, "Archfiend of Spite");
    t.lands(P0, "Swamp", 3);
    t.battlefield(P1, "Archfiend of Spite");
    t.lands(P1, "Island", 11);
    t.attack(&[(rakdos, Entity::Player(P1))], &[]);
    t.resolve_all();
    // P0: three non-Demons (the Swamps) → two sacrificed; the Demons stay.
    assert_eq!(count_of(&t, P0, CardType::Land), 1);
    assert!(t.on_battlefield(rakdos));
    assert_eq!(t.named_on_battlefield("Archfiend of Spite").len(), 2);
    // P1: eleven non-Demons → six sacrificed.
    assert_eq!(count_of(&t, P1, CardType::Land), 5);
}

#[test]
fn din_of_the_fireherd_counts_the_new_black_and_red_token() {
    cr!("608.2c", "105.2");
    ruling!(
        "Din of the Fireherd",
        "The parts of the spell happen in order. When determining how many black creatures and how many red creatures you control, keep in mind that you just put a creature onto the battlefield that’s black and red."
    );
    supported("Din of the Fireherd");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.lands(P1, "Island", 2);
    add_mana(&mut t, P0, ManaType::B, 3);
    add_mana(&mut t, P0, ManaType::C, 5);
    let din = t.hand(P0, "Din of the Fireherd");
    t.cast_with(P0, din, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(count_of(&t, P1, CardType::Creature), 1);
    assert_eq!(count_of(&t, P1, CardType::Land), 1);
}

#[test]
fn global_ruin_sacrifices_lands_without_basic_land_types() {
    cr!("305.6", "701.21a");
    ruling!(
        "Global Ruin",
        "You sacrifice all lands other than the chosen ones. Since you can only choose lands with basic land types, any non-basic lands that don’t have a basic land type (even if they have some other land type, as in the case of Cloudpost) are necessarily going to be sacrificed."
    );
    supported("Global Ruin");
    let mut t = TestGame::new(2);
    let post = t.battlefield(P1, "Cloudpost");
    let island = t.battlefield(P1, "Island");
    let from = t.asked().len();
    cast_new(&mut t, P0, "Global Ruin", &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(post));
    assert!(t.on_battlefield(island));
    assert!(t.asked()[from..].iter().all(|(_, d)| match d {
        Decision::ChooseEntities { candidates, .. } => !candidates.contains(&obj(post)),
        _ => true,
    }));
}

#[test]
fn crackling_doom_sacrifice_doesnt_depend_on_the_damage() {
    cr!("615.1", "608.2c");
    ruling!(
        "Crackling Doom",
        "The sacrifice is not dependent on the damage being dealt. It doesn't matter if that damage is prevented or redirected."
    );
    supported("Crackling Doom");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Solitary Confinement");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Crackling Doom", &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "the damage was prevented");
    assert!(!t.on_battlefield(giant) && t.on_battlefield(bears));
}

#[test]
fn sacrificed_creatures_toughness_is_its_last_known_toughness() {
    cr!("608.2h", "701.21a");
    ruling!(
        "Tribute to Hunger",
        "Use the sacrificed creature's toughness as it last existed on the battlefield to determine how much life to gain."
    );
    supported("Tribute to Hunger");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s29_common::put_counters(&mut t, bears, "+1/+1", 3);
    t.battlefield(P1, "Glorious Anthem");
    cast_new(&mut t, P0, "Tribute to Hunger", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
    ruling!(
        "Consuming Vapors",
        "The sacrificed creature's last known existence on the battlefield is checked to determine its toughness."
    );
    supported("Consuming Vapors");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s29_common::put_counters(&mut t, bears, "+1/+1", 2);
    t.battlefield(P1, "Glorious Anthem");
    cast_new(&mut t, P0, "Consuming Vapors", &[Entity::Player(P1)]);
    t.resolve();
    assert_eq!(t.life(P0), 25);
}
