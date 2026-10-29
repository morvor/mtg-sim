//! CR 122.6, 122.6a: who puts counters on a permanent or player, and whether an effect
//! puts them (CR 609.1) — counters put as a cost (CR 118, 606.4), as the result of damage
//! (CR 120.3b, 120.3d), or by a turn-based action (CR 714.3c) aren't put by an effect.

use super::r703_common::*;
use mtg_engine::ability::AbilityKind;
use mtg_engine::events::{CounterOrigin, Event};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Activates the activated ability of `source` whose text contains `needle`.
fn activate(t: &mut TestGame, p: PlayerId, source: ObjectId, needle: &str) {
    t.g.recompute();
    let uid = t
        .g
        .obj(source)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.contains(needle))
        .map(|a| a.uid)
        .expect("no such activated ability");
    t.g.turn.priority = Some(p);
    t.g.activate_ability(p, source, uid).expect("activation");
    t.g.flush_events();
}

/// The (player, origin) of each counters event of this turn putting `kind` counters on
/// `on`.
fn puts_on(t: &TestGame, on: ObjectId, kind: &str) -> Vec<(Option<PlayerId>, CounterOrigin)> {
    t.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::CountersAdded {
                target: Entity::Object(o),
                kind: k,
                by,
                origin,
                ..
            } if *o == on && k.as_str() == kind => Some((*by, *origin)),
            _ => None,
        })
        .collect()
}

#[test]
fn counters_put_as_a_cost_arent_put_by_an_effect() {
    cr!("122.6", "602.2b", "609.1");
    supported("Devoted Druid");
    supported("Vizier of Remedies");
    // Devoted Druid: "Put a -1/-1 counter on this creature: Untap this creature." Doubling
    // Season ("If an effect would put ...") doesn't double the cost's counter.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doubling Season");
    let druid = t.battlefield(P0, "Devoted Druid");
    t.g.objects[druid.0 as usize].tapped = true;
    activate(&mut t, P0, druid, "Untap");
    assert_eq!(t.counters(druid, counters::MINUS1), 1);
    assert_eq!(
        puts_on(&t, druid, counters::MINUS1),
        vec![(Some(P0), CounterOrigin::Cost)]
    );
    t.resolve_all();
    assert!(!t.obj(druid).tapped);
    // Vizier of Remedies ("If one or more -1/-1 counters would be put on a creature you
    // control, that many minus one are put on it instead") isn't limited to effects.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vizier of Remedies");
    let druid = t.battlefield(P0, "Devoted Druid");
    activate(&mut t, P0, druid, "Untap");
    assert_eq!(t.counters(druid, counters::MINUS1), 0);
}

#[test]
fn a_sagas_turn_based_lore_counter_isnt_put_by_an_effect() {
    cr!("609.1", "714.3a", "714.3c", "122.6");
    supported("History of Benalia");
    supported("Doubling Season");
    // Doubling Season doubles the lore counter a Saga enters with (its intrinsic
    // replacement effect, CR 714.3a), not the one its controller puts on it as their
    // precombat main phase begins (a turn-based action).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doubling Season");
    let saga = t.enter(P0, "History of Benalia");
    assert_eq!(t.counters(saga, counters::LORE), 2);
    t.resolve_all();
    t.set_step(P0, Step::Draw);
    to_step_start(&mut t, P0, Step::PrecombatMain);
    assert_eq!(t.counters(saga, counters::LORE), 3);
    // As it entered, by an effect; then as a turn-based action.
    assert_eq!(
        puts_on(&t, saga, counters::LORE),
        vec![
            (Some(P0), CounterOrigin::Effect),
            (Some(P0), CounterOrigin::Rule)
        ]
    );
    // "If you would put one or more counters" (Vorinclex) does apply: P0 puts it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vorinclex, Monstrous Raider");
    let saga = t.enter(P0, "History of Benalia");
    assert_eq!(t.counters(saga, counters::LORE), 2);
    t.resolve_all();
    t.set_step(P0, Step::Draw);
    to_step_start(&mut t, P0, Step::PrecombatMain);
    assert_eq!(t.counters(saga, counters::LORE), 4);
}

#[test]
fn the_controller_of_a_wither_source_puts_the_counters() {
    cr!("120.3d", "122.6", "702.80a");
    supported("Hapatra, Vizier of Poisons");
    supported("Boggart Ram-Gang");
    // Hapatra: "Whenever you put one or more -1/-1 counters on a creature, create a 1/1
    // green Snake creature token with deathtouch."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hapatra, Vizier of Poisons");
    let mine = t.battlefield(P0, "Boggart Ram-Gang");
    let theirs = t.battlefield(P1, "Boggart Ram-Gang");
    let their_bears = t.battlefield(P1, "Grizzly Bears");
    let my_bears = t.battlefield(P0, "Grizzly Bears");
    // P0's wither source damages P1's creature: P0 puts the counters, not by an effect.
    t.g.deal_damage(mine, Entity::Object(their_bears), 1, false);
    t.g.flush_events();
    assert_eq!(
        puts_on(&t, their_bears, counters::MINUS1),
        vec![(Some(P0), CounterOrigin::Damage)]
    );
    t.resolve_all();
    let snakes = |t: &TestGame| {
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.chars.has_subtype("Snake"))
            .count()
    };
    assert_eq!(snakes(&t), 1);
    // P1's wither source damages P0's creature: P1 puts them; Hapatra doesn't trigger.
    t.g.deal_damage(theirs, Entity::Object(my_bears), 1, false);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(my_bears, counters::MINUS1), 1);
    assert_eq!(snakes(&t), 1);
}

#[test]
fn whenever_an_opponent_puts_counters_names_the_putter() {
    cr!("122.6");
    // "Whenever an opponent puts one or more counters on a creature, you gain that much
    // life."
    let watcher = oracle_card(
        "Counter Watcher",
        "Enchantment",
        "{0}",
        None,
        "Whenever an opponent puts one or more counters on a creature, you gain that much life.",
    );
    let mut t = TestGame::new(2);
    t.custom(P0, watcher, Zone::Battlefield);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    // P1's ability puts two counters on P0's creature: P0 gains 2.
    t.g.add_counters(Entity::Object(mine), counters::PLUS1, 2, Some(theirs));
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // P0's ability puts counters on P1's creature: no.
    t.g.add_counters(Entity::Object(theirs), counters::PLUS1, 3, Some(mine));
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn counters_on_a_card_in_exile_arent_counters_put_on_a_creature() {
    cr!("122.6", "702.62a");
    supported("Mikey & Leo, Chaos & Order");
    supported("Lord Jyscal Guado");
    // Suspending a creature card exiles it with time counters on it: those aren't counters
    // put on a creature (only on a permanent, or on an object as it enters).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mikey & Leo, Chaos & Order");
    t.battlefield(P0, "Lord Jyscal Guado");
    let card = t.hand(P0, "Veiling Oddity");
    t.lands(P0, "Island", 2);
    let hand = t.hand_size(P0);
    t.g.turn.priority = Some(P0);
    t.g.perform_action(
        P0,
        mtg_engine::decision::Action::Special(mtg_engine::decision::SpecialAction::Suspend {
            card,
        }),
    )
    .expect("suspend Veiling Oddity");
    t.settle();
    assert_eq!(t.counters(t.g.current(card), counters::TIME), 4);
    assert_eq!(t.stack_len(), 0, "Mikey & Leo doesn't trigger");
    assert_eq!(t.hand_size(P0), hand - 1);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0, "Lord Jyscal Guado doesn't trigger");
    // A counter on a creature P0 controls: both do.
    let mut t = TestGame::new(2);
    let mikey = t.battlefield(P0, "Mikey & Leo, Chaos & Order");
    t.battlefield(P0, "Lord Jyscal Guado");
    let hand = t.hand_size(P0);
    t.g.add_counters(Entity::Object(mikey), counters::TIME, 1, Some(mikey));
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn if_you_would_put_counters_on_your_creature_or_yourself() {
    cr!("122.6", "606.4", "614.1a");
    supported("Lae'zel, Vlaakith's Champion");
    supported("Ajani, Caller of the Pride");
    // Lae'zel: "If you would put one or more counters on a creature or planeswalker you
    // control or on yourself, put that many plus one of each of those kinds of counters
    // on that permanent or player instead."
    let mut t = TestGame::new(2);
    let laezel = t.battlefield(P0, "Lae'zel, Vlaakith's Champion");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    // P0 puts one on P0's creature: two.
    t.g.add_counters(Entity::Object(mine), counters::PLUS1, 1, Some(laezel));
    assert_eq!(t.counters(mine, counters::PLUS1), 2);
    // P1 puts one on it: P1 isn't "you".
    t.g.add_counters(Entity::Object(mine), counters::PLUS1, 1, Some(theirs));
    assert_eq!(t.counters(mine, counters::PLUS1), 3);
    // P0 puts one on P1's creature: not a creature P0 controls.
    t.g.add_counters(Entity::Object(theirs), counters::PLUS1, 1, Some(laezel));
    assert_eq!(t.counters(theirs, counters::PLUS1), 1);
    // P0 puts counters on P0 (yourself), not on P1.
    t.g.add_counters(Entity::Player(P0), "energy", 2, Some(laezel));
    assert_eq!(t.g.player(P0).counter("energy"), 3);
    t.g.add_counters(Entity::Player(P1), counters::POISON, 1, Some(laezel));
    assert_eq!(t.g.player(P1).counter(counters::POISON), 1);
    // A planeswalker entering under P0's control (4 loyalty) gets one more, and so does a
    // loyalty cost's counter: not only counters put by an effect.
    let ajani = t.enter(P0, "Ajani, Caller of the Pride");
    assert_eq!(t.counters(ajani, counters::LOYALTY), 5);
    activate(&mut t, P0, ajani, "Put a +1/+1 counter on up to one target creature");
    assert_eq!(t.counters(ajani, counters::LOYALTY), 7);
}
