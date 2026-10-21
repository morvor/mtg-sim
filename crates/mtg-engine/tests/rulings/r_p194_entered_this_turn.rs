//! Rulings batch P194 — permanents that entered this turn (characteristics checked as
//! the ability resolves, any controller), token copies of tokens that entered this turn,
//! Neriv's doubled damage, Stonehorn Dignitary during combat, and Storybook Ride's
//! "you may play those cards this turn".

use crate::r_p194_common::*;
use crate::r_s01_common::{attack_with, supported, with_subtype};
use mtg_engine::events::Event;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts `name` onto the battlefield under `p`'s control through a real zone change (so it
/// "entered this turn"), then settles.
fn enters(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.enter(p, name);
    t.g.flush_events();
    t.settle();
    id
}

/// Puts `name` onto the battlefield as a permanent that has been there since an earlier
/// turn.
fn already_there(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.battlefield(p, name);
    t.g.objects[id.0 as usize].entered_turn = 0;
    id
}

/// P0 casts Raise the Alarm (two 1/1 Soldier tokens) and it resolves.
fn raise_the_alarm(t: &mut TestGame) -> Vec<ObjectId> {
    let rta = t.hand(P0, "Raise the Alarm");
    mana(t, P0, ManaType::W, 2);
    t.cast(P0, rta).go();
    t.resolve_all();
    with_subtype(t, P0, "Soldier")
}

#[test]
fn oakhollow_village_checks_creature_types_as_it_resolves() {
    cr!("608.2h", "613.1d");
    ruling!(
        "Oakhollow Village",
        "Oakhollow Village’s last ability cares about permanents’ characteristics at the time the ability resolves, not their characteristics at the time they entered the battlefield. For example, say a Bear creature enters the battlefield and, later in the turn, gains the Frog creature type. When Oakhollow Village’s last ability resolves that turn, you’ll put a +1/+1 counter on that creature."
    );
    supported("Oakhollow Village");
    supported("Maskwood Nexus");
    let mut t = TestGame::new(2);
    let village = t.battlefield(P0, "Oakhollow Village");
    let entered = enters(&mut t, P0, "Grizzly Bears");
    let old = already_there(&mut t, P0, "Grizzly Bears");
    let theirs = enters(&mut t, P1, "Grizzly Bears");
    // Now every creature P0 controls is a Frog.
    t.battlefield(P0, "Maskwood Nexus");
    assert!(t.obj_now(entered).chars.has_subtype("Frog"));
    mana(&mut t, P0, ManaType::G, 1);
    t.activate(P0, village, 2, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(entered, "+1/+1"), 1);
    assert_eq!(t.counters(old, "+1/+1"), 0);
    assert_eq!(t.counters(theirs, "+1/+1"), 0);
}

#[test]
fn oran_rief_and_novijen_affect_every_controller_as_they_resolve() {
    cr!("608.2h", "105.3");
    ruling!(
        "Oran-Rief, the Vastwood",
        "Oran-Rief's last ability affects all permanents that are green creatures that entered this turn, not just the ones you control."
    );
    ruling!(
        "Oran-Rief, the Vastwood",
        "Oran-Rief's last ability cares about permanents' characteristics at the time the ability resolves, not their characteristics at the time they entered. For example, if a blue creature enters, then is turned green by a spell or ability, then Oran-Rief's second ability resolves, you'll put a +1/+1 counter on that creature."
    );
    ruling!(
        "Novijen, Heart of Progress",
        "The second ability puts a counter on each applicable creature regardless of who controls it."
    );
    supported("Oran-Rief, the Vastwood");
    supported("Novijen, Heart of Progress");
    supported("Viridescent Wisps");
    let mut t = TestGame::new(2);
    let oran = t.battlefield(P0, "Oran-Rief, the Vastwood");
    let theirs = enters(&mut t, P1, "Grizzly Bears");
    let blue = enters(&mut t, P0, "Air Elemental");
    let other_blue = enters(&mut t, P1, "Air Elemental");
    let old = already_there(&mut t, P1, "Grizzly Bears");
    let w = t.hand(P0, "Viridescent Wisps");
    mana(&mut t, P0, ManaType::G, 1);
    t.cast(P0, w).target(blue).go();
    t.resolve_all();
    t.activate(P0, oran, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(theirs, "+1/+1"), 1);
    assert_eq!(t.counters(blue, "+1/+1"), 1);
    assert_eq!(t.counters(other_blue, "+1/+1"), 0);
    assert_eq!(t.counters(old, "+1/+1"), 0);

    let mut t = TestGame::new(2);
    let nov = t.battlefield(P0, "Novijen, Heart of Progress");
    let theirs = enters(&mut t, P1, "Grizzly Bears");
    let mine = enters(&mut t, P0, "Grizzly Bears");
    let old = already_there(&mut t, P1, "Grizzly Bears");
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::U, 1);
    t.activate(P0, nov, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(theirs, "+1/+1"), 1);
    assert_eq!(t.counters(mine, "+1/+1"), 1);
    assert_eq!(t.counters(old, "+1/+1"), 0);
}

#[test]
fn ocelot_pride_copies_tokens_without_targeting_them() {
    cr!("702.131b", "702.18a", "707.2");
    ruling!(
        "Ocelot Pride",
        "Ocelot Pride's last ability doesn't target any of the tokens."
    );
    supported("Ocelot Pride");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ocelot Pride");
    let soldiers = raise_the_alarm(&mut t);
    // A Soldier with shroud.
    let greaves = t.battlefield(P0, "Lightning Greaves");
    assert!(t.g.attach(greaves, obj(soldiers[0])));
    t.lands(P0, "Plains", 6);
    gain(&mut t, P0, 1);
    t.g.recompute();
    end_step(&mut t);
    assert!(t.g.player(P0).has_citys_blessing);
    // The Cat token, plus a copy of each of the three tokens that entered this turn.
    assert_eq!(with_subtype(&t, P0, "Cat").len(), 1 + 1 + 1);
    assert_eq!(with_subtype(&t, P0, "Soldier").len(), 4);
}

#[test]
fn renewed_solidarity_copies_only_the_tokens_copiable_values() {
    cr!("707.2", "111.1");
    ruling!(
        "Renewed Solidarity",
        "When this enchantment’s last ability resolves, for each applicable token you control that entered this turn, create a token that copies the original characteristics of that first token as stated by the effect that created it (unless that first token is copying something else; see below). It doesn’t copy whether that token is tapped or untapped, whether it has any counters on it or Auras or Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, and so on."
    );
    supported("Renewed Solidarity");
    supported("Giant Growth");
    let mut t = TestGame::new(2);
    crate::r_p125_common::choose_creature_type(&mut t, P0, "Soldier");
    enters(&mut t, P0, "Renewed Solidarity");
    let soldiers = raise_the_alarm(&mut t);
    let s = soldiers[0];
    kill(&mut t, soldiers[1]);
    t.g.add_counters(obj(s), "+1/+1", 1, None);
    t.g.tap(s);
    let gg = t.hand(P0, "Giant Growth");
    mana(&mut t, P0, ManaType::G, 1);
    t.cast(P0, gg).target(s).go();
    t.resolve_all();
    let greaves = t.battlefield(P0, "Lightning Greaves");
    assert!(t.g.attach(greaves, obj(s)));
    // 1/1 + counter + Giant Growth + Renewed Solidarity's +1/+0.
    assert_eq!(t.pt(s), (6, 5));
    end_step(&mut t);
    let copies: Vec<ObjectId> = with_subtype(&t, P0, "Soldier")
        .into_iter()
        .filter(|c| *c != t.g.current(s))
        .collect();
    assert_eq!(copies.len(), 1);
    let c = copies[0];
    assert!(!t.obj_now(c).tapped);
    assert_eq!(t.counters(c, "+1/+1"), 0);
    assert!(t.g.attachments_of(obj(c)).is_empty());
    assert_eq!(t.pt(c), (2, 1));
}

#[test]
fn redoubled_stormsinger_tokens_each_choose_what_they_attack() {
    cr!("508.4", "707.2");
    ruling!(
        "Redoubled Stormsinger",
        "You declare which player, planeswalker, or battle each token is attacking as you put it onto the battlefield. It doesn't have to be the same player, planeswalker, or battle as Redoubled Stormsinger or any of the other tokens created by Redoubled Stormsinger's last ability."
    );
    supported("Redoubled Stormsinger");
    let mut t = TestGame::new(3);
    let storm = t.battlefield(P0, "Redoubled Stormsinger");
    let soldiers = raise_the_alarm(&mut t);
    t.answer_choose(P0, &[pl(P2)]);
    t.answer_choose(P0, &[pl(P1)]);
    attack_with(&mut t, &[(storm, pl(P1))]);
    t.resolve_all();
    let copies: Vec<ObjectId> = with_subtype(&t, P0, "Soldier")
        .into_iter()
        .filter(|c| !soldiers.contains(c))
        .collect();
    assert_eq!(copies.len(), 2);
    let combat = t.g.combat.as_ref().unwrap();
    let mut targets: Vec<Option<Entity>> =
        copies.iter().map(|c| combat.attack_target(*c)).collect();
    targets.sort();
    assert_eq!(targets, vec![Some(pl(P1)), Some(pl(P2))]);
    assert!(copies.iter().all(|c| t.obj_now(*c).tapped));
}

#[test]
fn neriv_doubles_damage_from_the_same_source() {
    cr!("614.1a", "120.1");
    ruling!(
        "Neriv, Heart of the Storm",
        "The damage is dealt by the same source as the original source of damage. The doubled damage isn’t dealt by Neriv unless it was the original source of damage."
    );
    supported("Neriv, Heart of the Storm");
    let mut t = TestGame::new(2);
    let neriv = t.battlefield(P0, "Neriv, Heart of the Storm");
    let pyro = enters(&mut t, P0, "Prodigal Pyromancer");
    t.g.objects[pyro.0 as usize].summoning_sick = false;
    // Not doubled: a creature that didn't enter this turn.
    let old = already_there(&mut t, P0, "Prodigal Pyromancer");
    t.activate(P0, pyro, 0, &[pl(P1)]).unwrap();
    t.resolve_all();
    t.activate(P0, old, 0, &[pl(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    let to_p1 = damage_events(&t);
    assert_eq!(to_p1, vec![(pyro, pl(P1), 2), (old, pl(P1), 1)]);
    assert!(to_p1.iter().all(|(s, _, _)| *s != neriv));
}

#[test]
fn stonehorn_dignitary_during_combat_skips_the_next_combat_phase() {
    cr!("500.11", "506.1");
    ruling!(
        "Stonehorn Dignitary",
        "Once a combat phase starts, it can't be skipped. A Stonehorn Dignitary entering the battlefield during combat won't affect that combat, even if it's the turn of the opponent chosen as the target of the triggered ability."
    );
    supported("Stonehorn Dignitary");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    enters(&mut t, P0, "Stonehorn Dignitary");
    t.resolve_all();
    attack_with(&mut t, &[(bears, pl(P0))]);
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.life(P0), 18);
    // P1's next turn has no combat phase.
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::PostcombatMain);
    let combat_began = t.g.turn_events.iter().any(|e| {
        matches!(
            e,
            Event::StepBegan {
                step: Step::BeginningOfCombat,
                ..
            }
        )
    });
    assert!(!combat_began);
    assert_eq!(t.g.turn.number, 3);
}

#[test]
fn storybook_ride_doesnt_change_when_the_cards_can_be_played() {
    cr!("117.1a", "305.2");
    ruling!(
        "Storybook Ride",
        "Storybook Ride doesn’t change when you can play the exiled cards. For example, if you exile a sorcery card, you can cast it only during your main phase when the stack is empty. If you exile a land card, you can play it only during your main phase and only if you have an available land play remaining."
    );
    supported("Storybook Ride");
    let visit = |top: &str| -> (TestGame, ObjectId) {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Storybook Ride");
        t.library_top(P0, top);
        t.g.dice.loaded.push_back(2);
        mtg_engine::variants::roll_to_visit(&mut t.g, P0);
        t.g.flush_events();
        t.resolve_all();
        let exiled = *t.g.exile.last().unwrap();
        assert_eq!(t.g.obj(exiled).chars.name.as_str(), top);
        (t, exiled)
    };
    // A sorcery: not with something on the stack, not outside a main phase.
    let (mut t, div) = visit("Divination");
    mana(&mut t, P0, ManaType::U, 3);
    let bolt = t.hand(P0, "Lightning Bolt");
    mana(&mut t, P0, ManaType::R, 1);
    t.cast(P0, bolt).target(pl(P1)).go();
    assert!(t.cast(P0, div).try_go().is_err());
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(t.cast(P0, div).try_go().is_err());
    t.set_step(P0, Step::PostcombatMain);
    t.cast(P0, div).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Divination"));
    // A land: only with a land play left.
    let (mut t, forest) = visit("Forest");
    t.g.players[0].lands_played_this_turn = 1;
    assert!(t.play_land(P0, forest).is_err());
    assert_eq!(t.zone(forest), Zone::Exile);
    t.g.players[0].lands_played_this_turn = 0;
    t.play_land(P0, forest).unwrap();
    assert!(t.on_battlefield(forest));
}
