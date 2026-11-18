//! Rulings batch P135 — "each player sacrifices / loses / discards" and mass land
//! destruction: the parts happen in sequence, each with choices in APNAP order and
//! simultaneous actions (CR 101.4, 608.2c-e), triggered abilities wait until the spell
//! has finished resolving (CR 603.3), look-back for permanents leaving at the same time
//! (CR 603.10a), and the other mass effects of the batch (Braids, Teferi's Realm,
//! Equipoise, Realm Razer, Chaotic Transformation, Fall of the Thran, Raiding Party,
//! Tectonic Rift, Clash of the Eikons).

use crate::r_p108_common::put_counters;
use crate::r_p135_common::*;
use crate::r_s25_common::targets_of;
use mtg_engine::ability::Modification;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// A land that gains its controller 1 life whenever another land is put into a graveyard
/// from the battlefield (a leaves-the-battlefield ability: it looks back, CR 603.10a).
fn witness_land(t: &mut TestGame, p: PlayerId) -> ObjectId {
    witness_land_typed(t, p, "Land")
}

/// A Witness Land with the given type line ("Land — Swamp").
fn witness_land_typed(t: &mut TestGame, p: PlayerId, type_line: &str) -> ObjectId {
    let def = custom_card(
        "Witness Land",
        type_line,
        "",
        None,
        "Whenever another land is put into a graveyard from the battlefield, you gain 1 life.",
    );
    t.custom(p, def, Zone::Battlefield)
}

/// The lands `p` controls.
fn lands_of(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is(CardType::Land))
        .map(|o| o.id)
        .collect()
}

/// The (player, prompt) of each choice of objects asked since decision `from`.
fn choices_since(t: &TestGame, from: usize) -> Vec<(PlayerId, String)> {
    t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { prompt, .. } => Some((*p, prompt.clone())),
            _ => None,
        })
        .collect()
}

/// The positions of the events of this turn that `f` matches.
fn event_positions(t: &TestGame, f: impl Fn(&Event) -> bool) -> Vec<usize> {
    t.g.turn_events
        .iter()
        .enumerate()
        .filter(|(_, e)| f(e))
        .map(|(i, _)| i)
        .collect()
}

/// Every ability put on the stack this turn was put there after `spell` finished
/// resolving (CR 603.3: triggered abilities wait for the next time a player would receive
/// priority).
fn triggers_waited_for(t: &TestGame, spell: ObjectId) {
    let done = event_positions(
        t,
        |e| matches!(e, Event::SpellResolved { spell: s } if *s == spell),
    );
    let put = event_positions(t, |e| matches!(e, Event::AbilityTriggeredOnStack { .. }));
    assert_eq!(done.len(), 1, "the spell resolved");
    assert!(!put.is_empty(), "abilities triggered");
    assert!(put.iter().all(|i| *i > done[0]), "{done:?} {put:?}");
}

/// The players asked to choose objects since decision `from`, in order (repeats
/// collapsed).
fn choosers_since(t: &TestGame, from: usize) -> Vec<PlayerId> {
    let mut v: Vec<PlayerId> = vec![];
    for (p, d) in &t.asked()[from..] {
        if matches!(d, Decision::ChooseEntities { .. }) && v.last() != Some(p) {
            v.push(*p);
        }
    }
    v
}

// ---------------------------------------------------------------------------------------
// Each player sacrifices lands: choices in turn order, then all at once.
// ---------------------------------------------------------------------------------------

#[test]
fn destructive_force_lands_are_chosen_in_turn_order_and_sacrificed_together() {
    cr!("101.4", "603.10a", "701.21a");
    ruling!("Destructive Force", "First the player whose turn it is chooses five lands to sacrifice, then each other player in turn order does the same, then all chosen lands are sacrificed at the same time. After that, Destructive Force deals 5 damage to each creature.");
    supported("Destructive Force");
    let mut t = TestGame::new(2);
    // Each player sacrifices its Witness Land with the others: sacrificed all at once, each
    // sees the nine other lands leave (had one player's gone first, that player's Witness
    // Land would have missed the other player's five).
    let w0 = witness_land(&mut t, P0);
    let w1 = witness_land(&mut t, P1);
    t.lands(P1, "Forest", 4);
    let wurm = t.battlefield(P1, "Colossal Dreadmaw");
    lands_for_cost(&mut t, P0, "Destructive Force");
    let others: Vec<Entity> = lands_of(&t, P0)
        .into_iter()
        .filter(|l| *l != w0)
        .take(4)
        .map(ob)
        .collect();
    t.answer_choose(P0, &[vec![ob(w0)], others].concat());
    let card = t.hand(P0, "Destructive Force");
    let from = t.asked().len();
    let s = t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!(choosers_since(&t, from), vec![P0, P1]);
    assert!(!t.on_battlefield(w0) && !t.on_battlefield(w1));
    assert_eq!((t.life(P0), t.life(P1)), (29, 29));
    assert_eq!(controlled(&t, P0, CardType::Land), 3);
    assert_eq!(controlled(&t, P1, CardType::Land), 0);
    // After that, the damage.
    assert_eq!(damage_on(&t, wurm), 5);
    let sacrificed = event_positions(&t, |e| {
        matches!(
            e,
            Event::ZoneChange {
                from: Zone::Battlefield,
                ..
            }
        )
    });
    let damage = event_positions(&t, |e| matches!(e, Event::Damage { .. }));
    assert!(sacrificed.iter().max() < damage.iter().min());
    assert!(resolved(&t, s));
}

#[test]
fn ember_swallower_lands_are_chosen_in_turn_order_and_sacrificed_together() {
    cr!("101.4", "603.10a", "701.37a");
    ruling!("Ember Swallower", "When the last ability resolves, the active player will choose three lands they control, then each other player in turn order will do the same. Then all of those lands are sacrificed at the same time.");
    supported("Ember Swallower");
    let mut t = TestGame::new(2);
    let es = t.battlefield(P0, "Ember Swallower");
    let mountains = t.lands(P0, "Mountain", 7);
    // Each player's Witness Land is among the lands it sacrifices; each sees the five
    // other lands leave with it.
    let w0 = witness_land(&mut t, P0);
    let w1 = witness_land(&mut t, P1);
    t.lands(P1, "Forest", 2);
    t.answer_choose(P0, &[ob(w0), ob(mountains[0]), ob(mountains[1])]);
    t.activate(P0, es, 0, &[]).expect("monstrosity");
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(t.counters(es, counters::PLUS1), 3);
    assert_eq!(choosers_since(&t, from), vec![P0, P1]);
    assert!(!t.on_battlefield(w0) && !t.on_battlefield(w1));
    assert_eq!((t.life(P0), t.life(P1)), (25, 25));
    assert_eq!(controlled(&t, P0, CardType::Land), 5);
}

#[test]
fn sundering_titan_chooses_lands_untargeted_and_destroys_them_together() {
    cr!("608.2c", "603.10a", "115.1", "117.1", "117.1d");
    ruling!("Sundering Titan", "Sundering Titan's ability isn't targeted. When it resolves, Sundering Titan's controller must choose one land for each basic land type (Plains, Island, Swamp, Mountain, and Forest), and then they are destroyed simultaneously.");
    ruling!("Sundering Titan", "Players can't take actions in between the time you choose the lands and the time you destroy them. Notably, they can't activate mana abilities of those lands.");
    supported("Sundering Titan");
    let mut t = TestGame::new(2);
    // A hexproof Tundra (Plains and Island), P1's Witness Land that is a Swamp, a Mountain;
    // P0's two Forests.
    let tundra = t.battlefield(P1, "Tundra");
    spoil(&mut t, tundra, Spoil::Hexproof);
    let swamp = witness_land_typed(&mut t, P1, "Land — Swamp");
    let mountain = t.battlefield(P1, "Mountain");
    let forest = t.battlefield(P0, "Forest");
    let other_forest = t.battlefield(P0, "Forest");
    t.answer_choose(P0, &[ob(tundra)]);
    t.answer_choose(P0, &[ob(tundra)]);
    t.answer_choose(P0, &[ob(swamp)]);
    t.answer_choose(P0, &[ob(mountain)]);
    t.answer_choose(P0, &[ob(forest)]);
    t.enter(P0, "Sundering Titan");
    t.settle();
    let top = *t.g.stack.last().unwrap();
    assert!(targets_of(&t, top).is_empty(), "not targeted");
    let from = t.asked().len();
    t.resolve();
    // As it resolves, P0 makes the five choices, then the lands are destroyed: no player
    // gets priority, or is asked for a mana payment (the only times mana abilities can be
    // activated, CR 117.1d), in between; the only other decision is the order the
    // destroyed cards go into their owners' graveyards.
    let asked: Vec<(PlayerId, bool)> = t.asked()[from..]
        .iter()
        .map(|(p, d)| (*p, matches!(d, Decision::ChooseEntities { .. })))
        .collect();
    assert_eq!(asked[..5], [(P0, true); 5], "{asked:?}");
    assert!(
        t.asked()[from..][5..]
            .iter()
            .all(|(_, d)| matches!(d, Decision::Order { .. })),
        "{asked:?}"
    );
    for l in [tundra, swamp, mountain, forest] {
        assert!(!t.on_battlefield(l));
    }
    assert!(t.on_battlefield(other_forest));
    t.resolve_all();
    // Destroyed with the others, the Swamp saw the other three leave (one at a time, it
    // would have seen only the lands destroyed before it).
    assert_eq!(t.life(P1), 23);
}

#[test]
fn mass_sacrifice_of_a_creature_land_happens_once() {
    cr!("701.21a", "608.2c");
    ruling!("Death Cloud", "If you control a permanent that’s a creature and a land, and you choose to sacrifice it as you sacrifice creatures, it won’t be on the battlefield when you sacrifice lands. You can’t sacrifice it again.");
    ruling!(
        "Rite of Ruin",
        "Permanents sacrificed for one card type won’t be on the battlefield to be chosen for subsequent card types."
    );
    supported("Death Cloud");
    supported("Rite of Ruin");
    // P1's choices since decision `from` among candidates that include `any`.
    fn offered(t: &TestGame, from: usize, any: Entity) -> Vec<Vec<Entity>> {
        t.asked()[from..]
            .iter()
            .filter_map(|(p, d)| match d {
                Decision::ChooseEntities { candidates, .. }
                    if *p == P1 && candidates.contains(&any) =>
                {
                    Some(candidates.clone())
                }
                _ => None,
            })
            .collect()
    }
    // Death Cloud, X = 1: P1's Dryad Arbor is its only creature; then one of its two
    // Forests, chosen among the Forests alone.
    let mut t = TestGame::new(2);
    let arbor = t.battlefield(P1, "Dryad Arbor");
    let forests = t.lands(P1, "Forest", 2);
    lands_for_cost(&mut t, P0, "Death Cloud");
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Death Cloud");
    let from = t.asked().len();
    t.cast(P0, card).x(1).go();
    t.resolve_all();
    assert!(!t.on_battlefield(arbor));
    assert_eq!(forests.iter().filter(|f| t.on_battlefield(**f)).count(), 1);
    assert_eq!(t.life(P1), 19);
    let lands = offered(&t, from, ob(forests[0]));
    assert!(!lands.is_empty());
    assert!(lands.iter().all(|c| !c.contains(&ob(arbor))), "{lands:?}");
    // Rite of Ruin (artifacts, creatures, lands): P1 sacrifices Ornithopter as its
    // artifact; then two creatures, chosen among the three others.
    let mut t = TestGame::new(2);
    let orni = t.battlefield(P1, "Ornithopter");
    let ring = t.battlefield(P1, "Sol Ring");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let creatures = [
        bears,
        t.battlefield(P1, "Hill Giant"),
        t.battlefield(P1, "Wall of Wood"),
    ];
    t.answer_choose(P1, &[ob(orni)]);
    lands_for_cost(&mut t, P0, "Rite of Ruin");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let card = t.hand(P0, "Rite of Ruin");
    let from = t.asked().len();
    t.cast(P0, card).go();
    t.resolve_all();
    assert!(!t.on_battlefield(orni));
    assert!(t.on_battlefield(ring));
    assert_eq!(
        creatures.iter().filter(|c| t.on_battlefield(**c)).count(),
        1
    );
    let chosen = offered(&t, from, ob(bears));
    assert!(!chosen.is_empty());
    assert!(chosen.iter().all(|c| !c.contains(&ob(orni))), "{chosen:?}");
}

// ---------------------------------------------------------------------------------------
// Smallpox, Pox: in sequence, choices in turn order, triggers wait.
// ---------------------------------------------------------------------------------------

/// P0's land that gains 1 life whenever a creature dies, and P0's creature that gains 1
/// life whenever a player discards a card or a land is put into a graveyard.
fn smallpox_watchers(t: &mut TestGame) {
    let land = custom_card(
        "Mourning Field",
        "Land",
        "",
        None,
        "Whenever a creature dies, you gain 1 life.",
    );
    t.custom(P0, land, Zone::Battlefield);
    let creature = custom_card(
        "Pox Watcher",
        "Creature — Spirit",
        "{1}",
        Some((1, 1)),
        "Whenever a player discards a card, you gain 1 life.\nWhenever a land is put into a graveyard from the battlefield, you gain 1 life.",
    );
    t.custom(P0, creature, Zone::Battlefield);
}

#[test]
fn smallpox_happens_in_order_and_triggers_wait() {
    cr!("608.2c", "608.2e", "603.3", "101.4", "603.10a");
    ruling!("Smallpox", "As Smallpox resolves, first each player loses 1 life at the same time. Next, each player chooses a card in hand without revealing it, then all of the cards are discarded at the same time. Then starting with the player whose turn it is and proceeding in turn order, each player chooses a creature, knowing the choices made before them. All of the creatures are sacrificed at the same time, then finally repeat this process for lands.");
    ruling!("Smallpox", "Any abilities that trigger while Smallpox is resolving will wait to be put onto the stack until Smallpox is done resolving.");
    ruling!("Smallpox", "Any abilities of the lands sacrificed may modify or trigger on the creatures being sacrificed and on the cards being discarded. Abilities of the creatures won't modify or trigger on the lands being sacrificed, but may modify or trigger on the cards being discarded.");
    supported("Smallpox");
    let mut t = TestGame::new(2);
    smallpox_watchers(&mut t);
    t.hand(P0, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Forest");
    let from = t.asked().len();
    let s = cast_modes(&mut t, P0, "Smallpox", &[], &[]);
    t.resolve();
    // Everything happened; the triggered abilities are only now on the stack.
    assert_eq!(t.graveyard_size(P1), 3, "card, creature, land");
    assert!(t.stack_len() > 0);
    triggers_waited_for(&t, s);
    assert_eq!(t.life(P0), 19);
    // Each part's choices: P0 first, then P1.
    let discard = "Choose cards to discard".to_string();
    let sacrifice = "Choose permanents to sacrifice".to_string();
    assert_eq!(
        choices_since(&t, from),
        vec![
            (P0, discard.clone()),
            (P1, discard),
            (P0, sacrifice.clone()),
            (P1, sacrifice.clone()),
            (P0, sacrifice.clone()),
            (P1, sacrifice),
        ]
    );
    // Life loss, then the discards, then the creatures, then the lands.
    let g = &t.g;
    let left = |ty: CardType| {
        move |e: &Event| {
            matches!(e, Event::ZoneChange { old, from: Zone::Battlefield, .. }
                if g.obj(*old).is(ty))
        }
    };
    let lost = event_positions(&t, |e| matches!(e, Event::LifeLost { .. }));
    let discarded = event_positions(&t, |e| matches!(e, Event::Discarded { .. }));
    let creatures = event_positions(&t, left(CardType::Creature));
    let lands = event_positions(&t, left(CardType::Land));
    for (a, b) in [
        (&lost, &discarded),
        (&discarded, &creatures),
        (&creatures, &lands),
    ] {
        assert_eq!(a.len(), 2);
        assert!(
            a.iter().max() < b.iter().min(),
            "{lost:?} {discarded:?} {creatures:?} {lands:?}"
        );
    }
    t.resolve_all();
    // Pox Watcher saw both discards (+2); Mourning Field saw both creatures die (+2);
    // Pox Watcher was gone when the lands were sacrificed.
    assert_eq!(t.life(P0), 23);
}

#[test]
fn pox_parts_are_processed_and_counted_one_at_a_time() {
    cr!("608.2c", "608.2e", "101.4", "107.1a");
    ruling!("Pox", "Each part of Pox's effect is processed separately. For each part, first the player whose turn it is makes all necessary choices (such as which cards to discard), then each other player in turn order chooses, then the actions happen at the same time. Then Pox's effect moves to the next stage.");
    ruling!("Pox", "The number you need to lose, discard, or sacrifice isn't calculated until it's time to perform that part of the effect. For example, if your opponent casts Pox and you discard Dodecapod as part of Pox's discard effect, Dodecapod will be put onto the battlefield instead of into your graveyard. Then, when determining how many creatures you need to sacrifice, the Dodecapod is taken into account.");
    supported("Pox");
    supported("Dodecapod");
    let mut t = TestGame::new(2);
    // P0 (three Swamps to cast it): three Islands in hand, three Bears.
    for _ in 0..3 {
        t.hand(P0, "Island");
        t.battlefield(P0, "Grizzly Bears");
    }
    // P1: Dodecapod and two Islands in hand, three Bears, three Forests.
    let dodecapod = t.hand(P1, "Dodecapod");
    t.hand(P1, "Island");
    t.hand(P1, "Island");
    for _ in 0..3 {
        t.battlefield(P1, "Grizzly Bears");
    }
    t.lands(P1, "Forest", 3);
    t.answer_choose(P1, &[ob(dodecapod)]);
    let from = t.asked().len();
    cast_modes(&mut t, P0, "Pox", &[], &[]);
    t.resolve_all();
    assert_eq!(
        (t.life(P0), t.life(P1)),
        (13, 13),
        "a third of 20, rounded up"
    );
    // P1 discarded the Dodecapod, which entered: four creatures, so two are sacrificed.
    assert_eq!(controlled(&t, P1, CardType::Creature), 2);
    assert_eq!(controlled(&t, P0, CardType::Creature), 2);
    assert_eq!(
        (
            controlled(&t, P0, CardType::Land),
            controlled(&t, P1, CardType::Land)
        ),
        (2, 2)
    );
    // Each part: P0 chooses, then P1, before the next part.
    let players: Vec<PlayerId> = choices_since(&t, from).into_iter().map(|c| c.0).collect();
    assert_eq!(players, vec![P0, P1, P0, P1, P0, P1]);
}

#[test]
fn rite_of_ruin_and_casualties_of_war_triggers_wait_until_they_finish() {
    cr!("603.3", "603.3b", "101.4", "608.2c", "603.10a");
    ruling!("Rite of Ruin", "If any abilities trigger while Rite of Ruin is resolving, those abilities will wait until Rite of Ruin finishes resolving before going on the stack. Starting with the active player, each player puts their abilities on the stack in any order.");
    ruling!("Casualties of War", "Each target permanent is destroyed in the order specified. Abilities that trigger when one is destroyed won't be put onto the stack until Casualties of War has finished resolving.");
    supported("Doomed Dissenter");
    // Rite of Ruin: both players' Doomed Dissenters die; P0's trigger goes on the stack
    // first.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Doomed Dissenter");
    let theirs = t.battlefield(P1, "Doomed Dissenter");
    lands_for_cost(&mut t, P0, "Rite of Ruin");
    let s = cast_modes(&mut t, P0, "Rite of Ruin", &[], &[]);
    t.resolve();
    assert!(!t.on_battlefield(mine) && !t.on_battlefield(theirs));
    triggers_waited_for(&t, s);
    assert_eq!(t.stack_len(), 2);
    let bottom = t.g.stack[0];
    assert_eq!(t.g.obj(bottom).controller, P0);
    t.resolve_all();
    // Casualties of War: the artifact, then the creature, then the land are destroyed,
    // one after another. The artifact gains its controller 1 life whenever a land is put
    // into a graveyard from the battlefield: destroyed before the land, it doesn't see it
    // (destroyed together with it, it would, CR 603.10a).
    let mut t = TestGame::new(2);
    let relic = custom_card(
        "Relic Watcher",
        "Artifact",
        "{1}",
        None,
        "Whenever a land is put into a graveyard from the battlefield, you gain 1 life.",
    );
    let a = t.custom(P1, relic, Zone::Battlefield);
    let d = t.battlefield(P1, "Doomed Dissenter");
    let l = t.battlefield(P1, "Mutavault");
    let s = cast_modes(
        &mut t,
        P0,
        "Casualties of War",
        &[0, 1, 3],
        &[ob(a), ob(d), ob(l)],
    );
    t.resolve();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(d) && !t.on_battlefield(l));
    triggers_waited_for(&t, s);
    let moved = |id: ObjectId| {
        event_positions(
            &t,
            |e| matches!(e, Event::ZoneChange { old, .. } if *old == id),
        )
    };
    let (ma, md, ml) = (moved(a), moved(d), moved(l));
    assert_eq!((ma.len(), md.len(), ml.len()), (1, 1, 1));
    assert!(ma[0] < md[0] && md[0] < ml[0], "in the order specified");
    assert_eq!(
        t.stack_len(),
        1,
        "only the Dissenter's trigger, after the spell"
    );
    t.resolve_all();
    assert_eq!(
        t.life(P1),
        20,
        "the artifact was gone when the land was destroyed"
    );
    assert_eq!(crate::r_s25_common::creature_tokens(&t, P1), 1);
}

// ---------------------------------------------------------------------------------------
// Upkeep effects.
// ---------------------------------------------------------------------------------------

#[test]
fn braids_resolves_before_your_own_upkeep_triggers() {
    cr!("101.4", "603.3b", "405.3");
    ruling!("Braids, Cabal Minion", "At the beginning of your upkeep, triggered abilities you control will resolve after triggered abilities your opponents control. If an opponent controls Braids and you control a triggered ability that puts a permanent onto the battlefield, you won't be able to sacrifice that permanent to satisfy Braids's ability.");
    supported("Braids, Cabal Minion");
    supported("Bitterblossom");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Braids, Cabal Minion");
    t.battlefield(P0, "Bitterblossom");
    let forest = t.battlefield(P0, "Forest");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.g.obj(t.g.stack[1]).controller, P1, "Braids's on top");
    t.resolve_all();
    assert!(!t.on_battlefield(forest), "P0 sacrificed its only land");
    assert_eq!(
        crate::r_s25_common::creature_tokens(&t, P0),
        1,
        "the Faerie stays"
    );
}

#[test]
fn teferis_realm_doesnt_phase_out_tokens() {
    cr!("702.26b", "111.1");
    ruling!(
        "Teferi's Realm",
        "Does not affect token permanents, only cards."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teferi's Realm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let token = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    t.answer(P1, DecisionKind::Option, Answer::Index(1)); // creature
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(t.obj_now(bears).phased_out);
    assert!(!t.obj_now(token).phased_out);
}

#[test]
fn equipoise_can_target_any_player_including_you() {
    cr!("115.1", "702.26b");
    ruling!("Equipoise", "In multiplayer games, you can choose a different target player each turn. You can even choose yourself.");
    supported("Equipoise");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Equipoise");
    t.lands(P0, "Plains", 1);
    let p1 = t.lands(P1, "Forest", 2);
    let p2 = t.lands(P2, "Island", 3);
    // First upkeep: P0 targets itself; nothing happens.
    t.answer_targets(P0, &[Entity::Player(P0)]);
    t.set_step(P2, Step::End);
    let from = t.asked().len();
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    let offered = target_candidates(&t, P0, from);
    for p in [P0, P1, P2] {
        assert!(offered[0].contains(&Entity::Player(p)));
    }
    t.resolve_all();
    assert!(t.g.permanents().all(|o| !o.phased_out));
    // Next turn: P2, who has two more lands than P0.
    t.answer_targets(P0, &[Entity::Player(P2)]);
    t.set_step(P2, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(p2.iter().filter(|l| t.obj_now(**l).phased_out).count(), 2);
    assert!(p1.iter().all(|l| !t.obj_now(*l).phased_out));
}

// ---------------------------------------------------------------------------------------
// Exile and return.
// ---------------------------------------------------------------------------------------

#[test]
fn realm_razer_leaving_first_exiles_the_lands_forever() {
    cr!("603.6c", "607.2a", "603.2");
    ruling!("Realm Razer", "If Realm Razer leaves the battlefield before its first ability has resolved, its second ability will trigger and do nothing. Then its first ability will resolve and exile all lands forever.");
    supported("Realm Razer");
    let mut t = TestGame::new(2);
    let lands = [t.battlefield(P0, "Forest"), t.battlefield(P1, "Island")];
    let rr = t.enter(P0, "Realm Razer");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, rr);
    assert_eq!(t.stack_len(), 2, "the leaves ability on top");
    t.resolve();
    assert!(lands.iter().all(|l| t.on_battlefield(*l)));
    t.resolve_all();
    assert!(lands.iter().all(|l| t.zone(*l) == Zone::Exile));
    assert_eq!(controlled(&t, P0, CardType::Land), 0);
    // Even another Realm Razer leaving doesn't return them.
    let rr2 = t.enter(P0, "Realm Razer");
    t.resolve_all();
    destroy(&mut t, rr2);
    t.resolve_all();
    assert!(lands.iter().all(|l| t.zone(*l) == Zone::Exile));
}

#[test]
fn chaotic_transformation_handles_each_exiled_permanent_of_a_player_in_turn() {
    cr!("608.2c", "608.2f", "608.2h", "701.20a");
    // Chaotic Transformation's ruling on this (2022-09-09: "If a player controls multiple
    // permanents that were exiled this way, that player first chooses which target
    // they're considering, ...") is left uncited: it has that player choose the order,
    // where CR 608.2f has "the player who controls the resolving spell or ability", and
    // the engine asks neither (docs/RULING_DISCREPANCIES.md, Inconsistent rulings; register
    // `per-object-actions-in-given-order`, provisional, for the user: ruling-conflicts-10).
    // What both agree on is pinned here: each permanent's process in turn, by its types
    // as it last existed on the battlefield.
    supported("Chaotic Transformation");
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P1, "Sol Ring");
    let bears = t.battlefield(P1, "Grizzly Bears");
    stack_library(&mut t, P1, &["Hill Giant", "Mind Stone"]);
    lands_for_cost(&mut t, P0, "Chaotic Transformation");
    let card = t.hand(P0, "Chaotic Transformation");
    // Artifact, creature, (no enchantment, planeswalker, land).
    t.answer_targets(P0, &[ob(ring)]);
    t.answer_targets(P0, &[ob(bears)]);
    t.cast(P0, card).go();
    t.resolve_all();
    assert!(t.in_exile("Sol Ring") && t.in_exile("Grizzly Bears"));
    assert_eq!(count_named(&t, "Hill Giant"), 1);
    assert_eq!(count_named(&t, "Mind Stone"), 1);
    // "As it last existed": an animated Mutavault (a land creature) finds a creature card.
    let mut t = TestGame::new(2);
    let mv = t.battlefield(P1, "Mutavault");
    animate(&mut t, mv);
    stack_library(&mut t, P1, &["Hill Giant"]);
    lands_for_cost(&mut t, P0, "Chaotic Transformation");
    let card = t.hand(P0, "Chaotic Transformation");
    t.answer_targets(P0, &[]);
    t.answer_targets(P0, &[ob(mv)]);
    t.cast(P0, card).go();
    t.resolve_all();
    assert!(t.in_exile("Mutavault"));
    assert_eq!(count_named(&t, "Hill Giant"), 1);
}

#[test]
fn fall_of_the_thran_returns_a_lone_land_card() {
    cr!("714.2b", "608.2c");
    ruling!("Fall of the Thran", "If a player somehow has only one land card in their graveyard when either of Fall of the Thran’s last two chapter abilities resolves, that player returns that one card to the battlefield.");
    supported("Fall of the Thran");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Fall of the Thran");
    t.g.objects[saga.0 as usize]
        .counters
        .insert(counters::LORE.into(), 1);
    t.graveyard(P1, "Forest");
    put_counters(&mut t, saga, counters::LORE, 1);
    t.resolve_all();
    assert_eq!(count_named(&t, "Forest"), 1);
    assert_eq!(controlled(&t, P1, CardType::Land), 1);
}

// ---------------------------------------------------------------------------------------
// Raiding Party, Tectonic Rift, Clash of the Eikons.
// ---------------------------------------------------------------------------------------

#[test]
fn raiding_party_a_player_may_save_fewer_than_two_plains() {
    cr!("608.2d", "701.8a");
    ruling!(
        "Raiding Party",
        "A player can choose zero or one Plains if they want."
    );
    supported("Raiding Party");
    for keep in [0usize, 1] {
        let mut t = TestGame::new(2);
        let rp = t.battlefield(P0, "Raiding Party");
        let orc = t.battlefield(P0, "Ironclaw Orcs");
        let knight = t.battlefield(P1, "White Knight");
        let plains = t.lands(P1, "Plains", 3);
        t.answer_choose(P1, &[ob(knight)]);
        t.answer_choose(
            P1,
            &plains[..keep].iter().map(|p| ob(*p)).collect::<Vec<_>>(),
        );
        t.activate(P0, rp, 0, &[]).expect("sacrifice an Orc");
        assert!(!t.on_battlefield(orc));
        t.resolve_all();
        assert!(t.obj_now(knight).tapped, "keep {keep}");
        assert_eq!(
            plains.iter().filter(|p| t.on_battlefield(**p)).count(),
            keep
        );
    }
}

#[test]
fn raiding_party_color_is_locked_in_when_activated() {
    cr!("113.7a", "612.2");
    ruling!("Raiding Party", "The color of creature to be tapped is set when the effect is announced and even if you use Sleight of Mind after that, you can’t change what color creature needs to be tapped to prevent the destruction.");
    supported("Sleight of Mind");
    let mut t = TestGame::new(2);
    let rp = t.battlefield(P0, "Raiding Party");
    t.battlefield(P0, "Ironclaw Orcs");
    let knight = t.battlefield(P1, "White Knight");
    let plains = t.lands(P1, "Plains", 2);
    t.answer_choose(P1, &[ob(knight)]);
    t.answer_choose(P1, &plains.iter().map(|p| ob(*p)).collect::<Vec<_>>());
    t.activate(P0, rp, 0, &[]).expect("sacrifice an Orc");
    // In response, Sleight of Mind changes Raiding Party's "white" to "blue".
    lands_for_cost(&mut t, P1, "Sleight of Mind");
    let som = t.hand(P1, "Sleight of Mind");
    // (The options: the colors, then the colors other than the first choice.)
    let white = Color::ALL.iter().position(|c| *c == Color::White).unwrap();
    let blue = Color::ALL
        .iter()
        .filter(|c| **c != Color::White)
        .position(|c| *c == Color::Blue)
        .unwrap();
    t.answer(P1, DecisionKind::Option, Answer::Index(white));
    t.answer(P1, DecisionKind::Option, Answer::Index(blue));
    t.cast(P1, som).target(rp).go();
    t.resolve(); // Sleight of Mind
    assert!(
        format!("{:?}", t.obj_now(rp).chars.abilities).contains("Blue"),
        "the text now says blue"
    );
    t.resolve_all();
    assert!(t.obj_now(knight).tapped);
    assert!(plains.iter().all(|p| t.on_battlefield(*p)));
}

#[test]
fn tectonic_rift_affects_creatures_by_flying_as_blockers_are_declared() {
    cr!("611.2c", "509.1b");
    ruling!("Tectonic Rift", "Because it doesn’t modify the characteristics of creatures, Tectonic Rift can affect creatures that enter the battlefield or lose flying after the spell resolves, and it will stop affecting creatures that gain flying.");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    let angel = t.battlefield(P1, "Serra Angel");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_modes(&mut t, P0, "Tectonic Rift", &[], &[ob(land)]);
    t.resolve_all();
    let late = t.battlefield(P1, "Grizzly Bears");
    modify(
        &mut t,
        P0,
        angel,
        vec![Modification::RemoveKeyword(KeywordKind::Flying)],
    );
    modify(
        &mut t,
        P1,
        bears,
        vec![Modification::AddKeyword(Keyword::new(KeywordKind::Flying))],
    );
    t.g.recompute();
    assert!(!t.g.can_block_at_all(late), "entered later");
    assert!(!t.g.can_block_at_all(angel), "lost flying");
    assert!(t.g.can_block_at_all(bears), "gained flying");
}

#[test]
fn clash_of_the_eikons_lore_counter_triggers_the_next_chapter_only() {
    cr!("714.2b", "714.4", "702.8a");
    ruling!("Clash of the Eikons", "Putting a lore counter on a Saga will usually cause its next chapter ability to trigger. If it already has a number of lore counters on it that's greater than or equal to its final chapter number, however, no chapter abilities trigger.");
    supported("Clash of the Eikons");
    supported("Leyline of Anticipation");
    // History of Benalia with one lore counter: chapter II triggers (a Knight).
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "History of Benalia");
    t.g.objects[saga.0 as usize]
        .counters
        .insert(counters::LORE.into(), 1);
    cast_modes(&mut t, P0, "Clash of the Eikons", &[2], &[ob(saga)]);
    t.resolve_all();
    assert_eq!(t.counters(saga, counters::LORE), 2);
    assert_eq!(crate::r_s25_common::creature_tokens(&t, P0), 1);
    // With chapter III on the stack (three counters), another counter (cast at instant
    // speed thanks to Leyline of Anticipation) triggers nothing.
    t.battlefield(P0, "Leyline of Anticipation");
    put_counters(&mut t, saga, counters::LORE, 1);
    assert_eq!(t.stack_len(), 1, "chapter III");
    cast_modes(&mut t, P0, "Clash of the Eikons", &[2], &[ob(saga)]);
    t.resolve(); // Clash of the Eikons
    t.settle();
    assert_eq!(t.counters(saga, counters::LORE), 4);
    assert_eq!(t.stack_len(), 1, "still only chapter III");
    t.resolve_all();
    assert!(!t.on_battlefield(saga));
}
