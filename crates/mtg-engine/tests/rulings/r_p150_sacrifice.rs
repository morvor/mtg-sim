//! Rulings batch P150 — sacrificing lands and creatures: as costs (Merciless Resolve,
//! Jarad, Tomb of Urami, Exalted Dragon), as replacement effects (Heart of Yavimaya,
//! Kjeldoran Outpost), on resolution (Nahiri's Lithoforming, Hew the Entwood, Reprocess),
//! in triggered abilities (Lotus Field, Thalia and The Gitrog Monster, Morkrut Necropod,
//! Baloth Prime, The Gitrog Monster), Old-Growth Troll's return, "can't play lands"
//! (Aggressive Mining, Territorial Dispute), and combat-damage triggers (Merfolk Spy,
//! Lavacore Elemental, Ruinous Minotaur).

use crate::r_s01_common::*;
use crate::r_s02_common::{can_play_land, destroy};
use crate::r_s05_common::move_to;
use crate::r_s21_common::{castable, go_to};
use crate::r_s25_common::cast_new;
use crate::r_s29_common::cast_and_resolve;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn lands_of(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is(CardType::Land))
        .count()
}

/// Deals `n` damage from `source` to `to` as a resolving effect of `source` would.
fn deal(t: &mut TestGame, source: ObjectId, n: i32, to: Entity) {
    let ctl = t.obj_now(source).controller;
    let mut ctx = mtg_engine::eval::Ctx::new(Some(source), ctl);
    ctx.targets = vec![vec![to]];
    t.g.exec(
        &Effect::DealDamage {
            source: Sel::This,
            amount: Value::Const(n),
            to: Sel::Target(0),
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.settle();
}

// ---------------------------------------------------------------------------------------
// Merfolk Spy, Lavacore Elemental, Ruinous Minotaur: damage triggers
// ---------------------------------------------------------------------------------------

#[test]
fn merfolk_spy_reveals_once_per_damage_event_and_only_while_resolving() {
    cr!("701.20a", "603.2c", "510.2");
    ruling!(
        "Merfolk Spy",
        "The second ability triggers just once each time Merfolk Spy deals combat damage to a player, regardless of how much damage it deals."
    );
    ruling!(
        "Merfolk Spy",
        "After the second ability resolves, the revealed card stops being revealed in its owner’s hand."
    );
    ruling!(
        "Merfolk Spy",
        "The next time Merfolk Spy deals combat damage to the same player, the card that’s revealed at random may be the same card or may be a different card."
    );
    supported("Merfolk Spy");
    let mut t = TestGame::new(2);
    let spy = t.battlefield(P0, "Merfolk Spy");
    let card = t.hand(P1, "Grizzly Bears");
    crate::r_s29_common::put_counters(&mut t, spy, "+1/+1", 2);
    t.set_step(P0, Step::PrecombatMain);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(spy, Entity::Player(P1))]),
    );
    go_to(&mut t, Step::CombatDamage);
    assert_eq!(t.life(P1), 17);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(crate::r_s31_common::revealed_cards(&t), vec![card]);
    assert!(!mtg_engine::reveal::is_revealed(&t.g, card));
    // Again (the same card, the only one): it can be revealed again.
    deal_combat_again(&mut t, spy);
    assert_eq!(crate::r_s31_common::revealed_cards(&t), vec![card, card]);
}

/// A second combat damage event from `spy` to P1 (the trigger resolved).
fn deal_combat_again(t: &mut TestGame, spy: ObjectId) {
    t.g.emit(mtg_engine::events::Event::Damage {
        source: spy,
        target: Entity::Player(P1),
        amount: 1,
        combat: true,
    });
    t.g.flush_events();
    t.resolve_all();
}

#[test]
fn lavacore_elemental_counts_its_own_and_each_creatures_combat_damage() {
    cr!("510.2", "603.2c");
    ruling!(
        "Lavacore Elemental",
        "If Lavacore Elemental deals combat damage to a player, it triggers its own ability."
    );
    ruling!(
        "Lavacore Elemental",
        "If multiple creatures you control deal combat damage to a player, Lavacore Elemental’s ability triggers that many times."
    );
    supported("Lavacore Elemental");
    let mut t = TestGame::new(2);
    let lava = t.battlefield(P0, "Lavacore Elemental");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let before = t.counters(lava, "time");
    t.attack(
        &[
            (lava, Entity::Player(P1)),
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
        ],
        &[],
    );
    t.resolve_all();
    assert_eq!(t.counters(lava, "time"), before + 3);
}

#[test]
fn ruinous_minotaur_triggers_on_noncombat_damage_to_an_opponent() {
    cr!("120.1", "603.2");
    ruling!(
        "Ruinous Minotaur",
        "Ruinous Minotaur’s ability triggers when it deals any kind of damage to an opponent, not just combat damage."
    );
    supported("Ruinous Minotaur");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Ruinous Minotaur");
    t.lands(P0, "Mountain", 2);
    deal(&mut t, m, 1, Entity::Player(P1));
    t.resolve_all();
    assert_eq!(lands_of(&t, P0), 1);
}

// ---------------------------------------------------------------------------------------
// "Sacrifice a ..." in triggered abilities
// ---------------------------------------------------------------------------------------

#[test]
fn sacrifice_triggers_that_must_sacrifice_the_source_itself() {
    cr!("701.21a", "603.3");
    ruling!(
        "Accursed Centaur",
        "If Accursed Centaur is the only creature on the battlefield, you must sacrifice it."
    );
    ruling!(
        "Lotus Field",
        "If Lotus Field enters the battlefield while you control fewer than two other lands, you must sacrifice each land you control, including Lotus Field."
    );
    ruling!(
        "Mold Demon",
        "The sacrifice is done as a triggered ability just after it enters."
    );
    supported("Accursed Centaur");
    supported("Lotus Field");
    supported("Mold Demon");
    let mut t = TestGame::new(2);
    let c = t.enter(P0, "Accursed Centaur");
    t.resolve_all();
    assert_eq!(t.zone(c), Zone::Graveyard(P0));
    // Lotus Field with one other land.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forest");
    let field = t.enter(P0, "Lotus Field");
    t.resolve_all();
    assert_eq!(lands_of(&t, P0), 0);
    assert_eq!(t.zone(field), Zone::Graveyard(P0));
    // Mold Demon: on the battlefield with the trigger on the stack.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let demon = t.enter(P0, "Mold Demon");
    t.settle();
    assert!(t.on_battlefield(demon));
    assert_eq!(t.stack_len(), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(demon));
    assert_eq!(lands_of(&t, P0), 0);
}

#[test]
fn single_sacrifices_on_resolution_take_exactly_one() {
    cr!("701.21a", "608.2c");
    ruling!(
        "Excavating Anurid",
        "While resolving Excavating Anurid’s triggered ability, you can’t sacrifice multiple lands to draw multiple cards."
    );
    ruling!(
        "The First Eruption",
        "While resolving The First Eruption's final chapter ability, you must sacrifice one Mountain if able. You can't sacrifice multiple Mountains to deal more damage."
    );
    ruling!(
        "Thalia and The Gitrog Monster",
        "As the last ability resolves, sacrificing a creature or land isn’t optional. If you control a creature or land (and if you control Thalia and The Gitrog Monster, you probably do), you must sacrifice one."
    );
    ruling!(
        "Harvester Troll",
        "Players can respond to the enters-the-battlefield ability (notably, while Harvester Troll is still 2/3), but once that ability starts resolving, it’s too late for any player to respond. You don’t choose which creature or land you’re sacrificing, if any, until the ability resolves."
    );
    supported("Excavating Anurid");
    supported("The First Eruption");
    supported("Thalia and The Gitrog Monster");
    supported("Harvester Troll");
    // Excavating Anurid: an answer naming two lands sacrifices one.
    let mut t = TestGame::new(2);
    let lands = t.lands(P0, "Forest", 2);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lands[0]), obj(lands[1])]);
    t.enter(P0, "Excavating Anurid");
    t.resolve_all();
    assert_eq!(lands_of(&t, P0), 1);
    assert_eq!(t.hand_size(P0), 1);
    // The First Eruption III: one Mountain, 3 damage.
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "The First Eruption");
    let ms = t.lands(P0, "Mountain", 2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_choose(P0, &[obj(ms[0]), obj(ms[1])]);
    crate::r_s29_common::put_counters(&mut t, saga, "lore", 3);
    t.resolve_all();
    assert_eq!(lands_of(&t, P0), 1);
    let threes = crate::r_s30_common::damage_events(&t)
        .iter()
        .filter(|d| d.1 == obj(wurm) && d.2 == 3)
        .count();
    assert_eq!(threes, 1);
    // Thalia and The Gitrog Monster: one creature or land is sacrificed, then a card drawn.
    let mut t = TestGame::new(2);
    let tg = t.battlefield(P0, "Thalia and The Gitrog Monster");
    t.lands(P0, "Forest", 1);
    t.answer_choose(P0, &[]);
    t.attack(&[(tg, Entity::Player(P1))], &[]);
    t.resolve_all();
    let left = lands_of(&t, P0) + t.on_battlefield(tg) as usize;
    assert_eq!(left, 1);
    assert_eq!(t.hand_size(P0), 1);
    // Harvester Troll: nothing is sacrificed until the trigger resolves.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let troll = t.enter(P0, "Harvester Troll");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert!(t.on_battlefield(forest));
    assert_eq!(t.pt(troll), (2, 3));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(forest)]);
    t.resolve_all();
    assert!(!t.on_battlefield(forest));
    assert_eq!(t.pt(troll), (4, 5));
}

#[test]
fn morkrut_necropod_sacrifices_after_attackers_and_blockers_are_chosen() {
    cr!("508.1", "509.1h", "506.4");
    ruling!(
        "Morkrut Necropod",
        "In the unusual situation that you control no other creatures or lands, the last ability won't do anything. Morkrut Necropod can attack or block with no penalty."
    );
    ruling!(
        "Morkrut Necropod",
        "When Morkrut Necropod attacks, its trigger doesn't resolve until after all attackers have been chosen. The same is true for when it blocks. This means that another creature, such as one enchanted with Invocation of Saint Traft, can also attack or block and then be sacrificed. Sacrificing a blocking creature this way doesn't cause the creature it blocked to become unblocked."
    );
    supported("Morkrut Necropod");
    // Alone: attacks with no penalty.
    let mut t = TestGame::new(2);
    let pod = t.battlefield(P0, "Morkrut Necropod");
    t.attack(&[(pod, Entity::Player(P1))], &[]);
    assert!(t.on_battlefield(pod));
    assert_eq!(t.life(P1), 13);
    // Blocking with the Bears too, then sacrificing them: the attacker stays blocked.
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Hill Giant");
    let pod = t.battlefield(P1, "Morkrut Necropod");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P1, &[obj(bears)]);
    t.attack(
        &[(attacker, Entity::Player(P1))],
        &[(bears, attacker), (pod, attacker)],
    );
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(pod));
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(attacker));
}

#[test]
fn baloth_prime_sees_lands_sacrificed_along_with_it() {
    cr!("603.10a", "701.21a");
    ruling!(
        "Baloth Prime",
        "If Baloth Prime leaves the battlefield at the same time as you sacrifice one or more lands (probably because you also sacrificed Baloth Prime), its second ability triggers for each of those lands."
    );
    supported("Baloth Prime");
    supported("Reprocess");
    let mut t = TestGame::new(2);
    let baloth = t.battlefield(P0, "Baloth Prime");
    let lands = t.lands(P0, "Forest", 2);
    t.answer_choose(P0, &[obj(baloth), obj(lands[0]), obj(lands[1])]);
    cast_and_resolve(&mut t, P0, "Reprocess", &[]);
    assert!(!t.on_battlefield(baloth));
    let beasts = crate::r_s25_common::creature_tokens(&t, P0);
    assert_eq!(beasts, 2);
}

#[test]
fn the_gitrog_monster_triggers_once_for_lands_put_into_the_graveyard_together() {
    cr!("603.2c", "701.17a");
    ruling!(
        "The Gitrog Monster",
        "If multiple land cards are put into your graveyard at once, The Gitrog Monster's last ability triggers only once."
    );
    supported("The Gitrog Monster");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Gitrog Monster");
    stack_library(&mut t, P0, &["Forest", "Swamp", "Island"]);
    let hand = t.hand_size(P0);
    crate::r_s05_common::run_from(
        &mut t,
        P0,
        None,
        Effect::Mill {
            who: PlayerRef::You,
            n: Value::c(3),
        },
        &[],
    );
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

// ---------------------------------------------------------------------------------------
// Playing additional lands, and "can't play lands"
// ---------------------------------------------------------------------------------------

#[test]
fn additional_land_permissions_are_cumulative() {
    cr!("305.2", "305.2a");
    ruling!(
        "Thalia and The Gitrog Monster",
        "The effect of Thalia and The Gitrog Monster’s ability that allows you to play an additional land is cumulative with similar effects. For example, if you control Thalia and The Gitrog Monster and another permanent with that ability, you’ll be able to play three lands during each of your turns."
    );
    ruling!(
        "Nahiri's Lithoforming",
        "Nahiri's Lithoforming's permission to play lands is cumulative with other effects that allow you to play additional lands, such as that of Scale the Heights."
    );
    supported("Nahiri's Lithoforming");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Thalia and The Gitrog Monster");
    t.battlefield(P0, "The Gitrog Monster");
    let lands: Vec<_> = (0..4).map(|_| t.hand(P0, "Forest")).collect();
    for l in &lands[..3] {
        assert!(can_play_land(&mut t, P0, *l));
        t.play_land(P0, *l).unwrap();
    }
    assert!(!can_play_land(&mut t, P0, lands[3]));
    // Nahiri's Lithoforming (X = 1) and Explore.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let sac = t.battlefield(P0, "Forest");
    t.lands(P0, "Mountain", 3);
    let lith = t.hand(P0, "Nahiri's Lithoforming");
    t.answer_choose(P0, &[obj(sac)]);
    t.cast(P0, lith).x(1).go();
    t.resolve_all();
    cast_and_resolve(&mut t, P0, "Explore", &[]);
    let lands: Vec<_> = (0..4).map(|_| t.hand(P0, "Forest")).collect();
    for l in &lands[..3] {
        assert!(can_play_land(&mut t, P0, *l));
        t.play_land(P0, *l).unwrap();
    }
    assert!(!can_play_land(&mut t, P0, lands[3]));
}

#[test]
fn nahiris_lithoforming_sacrifices_on_resolution_and_lands_enter_tapped_regardless() {
    cr!("608.2c", "614.12", "614.1c");
    ruling!(
        "Nahiri's Lithoforming",
        "You don't sacrifice lands until Nahiri's Lithoforming is resolving. Once you do so, players can't take actions until the spell has finished resolving."
    );
    ruling!(
        "Nahiri's Lithoforming",
        "If an effect says that a land enters tapped unless a condition is met or a cost is paid, that land enters tapped even if that condition is met or that cost is paid if it's affected by Nahiri's Lithoforming."
    );
    supported("Clifftop Retreat");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let ms = t.lands(P0, "Mountain", 4);
    let lith = t.hand(P0, "Nahiri's Lithoforming");
    t.cast(P0, lith).x(2).go();
    // Only mana was paid: four Mountains still there.
    assert_eq!(lands_of(&t, P0), 4);
    t.answer_choose(P0, &[obj(ms[0]), obj(ms[1])]);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(lands_of(&t, P0), 2);
    assert_eq!(t.hand_size(P0), 2);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    // A Clifftop Retreat with a Mountain: tapped anyway.
    let retreat = t.hand(P0, "Clifftop Retreat");
    t.play_land(P0, retreat).unwrap();
    let r = t.g.current(retreat);
    assert!(t.obj_now(r).tapped);
}

#[test]
fn cant_play_lands_doesnt_stop_putting_lands_onto_the_battlefield() {
    cr!("305.1", "305.2b", "101.2");
    ruling!(
        "Aggressive Mining",
        "Aggressive Mining doesn’t stop lands from being put onto the battlefield by a spell or ability. It only stops the special action of playing a land."
    );
    ruling!(
        "Territorial Dispute",
        "Does not stop lands from being put onto the battlefield by a spell or ability. It only stops players from taking the action of playing a land."
    );
    supported("Aggressive Mining");
    supported("Growth Spiral");
    for (name, me) in [("Aggressive Mining", P0), ("Territorial Dispute", P1)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        t.battlefield(me, name);
        let forest = t.hand(P0, "Forest");
        assert!(!can_play_land(&mut t, P0, forest), "{name}");
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(forest)]);
        cast_and_resolve(&mut t, P0, "Growth Spiral", &[]);
        assert!(t.on_battlefield(t.g.current(forest)), "{name}");
    }
}

#[test]
fn aggressive_mining_once_on_each_players_turn() {
    cr!("602.1b");
    ruling!(
        "Aggressive Mining",
        "You can activate the ability once on each player’s turn, not just your own."
    );
    let mut t = TestGame::new(2);
    let am = t.battlefield(P0, "Aggressive Mining");
    t.lands(P0, "Forest", 3);
    t.set_step(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[]);
    t.activate(P0, am, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, am));
    t.advance_to(P1, Step::Upkeep);
    assert!(crate::r_s02_common::can_activate(&mut t, P0, am));
    t.answer_choose(P0, &[]);
    t.activate(P0, am, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(lands_of(&t, P0), 1);
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, am));
}

#[test]
fn dwarven_armory_works_in_any_upkeep() {
    cr!("602.1b", "503.1");
    ruling!(
        "Dwarven Armory",
        "The ability can be activated during your opponent’s upkeep as well as your own."
    );
    supported("Dwarven Armory");
    let mut t = TestGame::new(2);
    let armory = t.battlefield(P0, "Dwarven Armory");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 3);
    t.set_step(P0, Step::PrecombatMain);
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, armory));
    t.advance_to(P1, Step::Upkeep);
    assert!(crate::r_s02_common::can_activate(&mut t, P0, armory));
    t.activate(P0, armory, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(bears, "+2/+2"), 1);
}

// ---------------------------------------------------------------------------------------
// Sacrifices as costs
// ---------------------------------------------------------------------------------------

#[test]
fn merciless_resolve_sacrifices_exactly_one_as_it_is_cast() {
    cr!("601.2h");
    ruling!(
        "Merciless Resolve",
        "You must sacrifice exactly one creature or land to cast Merciless Resolve. You can’t cast it without sacrificing a permanent, and you can’t sacrifice additional permanents."
    );
    ruling!(
        "Merciless Resolve",
        "Players can respond to this spell only after it’s been cast and all its costs have been paid. No one can try to destroy the creature or land you sacrificed to stop you from casting this spell or to make you sacrifice a different one."
    );
    supported("Merciless Resolve");
    let mut t = TestGame::new(2);
    crate::r_s04_common::add_mana(&mut t, P0, mtg_engine::mana::ManaType::B, 3);
    let mr = t.hand(P0, "Merciless Resolve");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let forest = t.battlefield(P0, "Forest");
    // Naming both: only one is sacrificed.
    t.answer_choose(P0, &[obj(bears), obj(forest)]);
    let spell = t.cast_with(P0, mr, &[]).unwrap();
    // Paid as it was cast: on the stack, exactly one gone.
    assert_eq!(t.zone(spell), Zone::Stack);
    let gone = [bears, forest]
        .iter()
        .filter(|id| !t.on_battlefield(**id))
        .count();
    assert_eq!(gone, 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    // No creature or land: can't be cast.
    let mut t = TestGame::new(2);
    let mr = t.hand(P0, "Merciless Resolve");
    crate::r_s04_common::add_mana(&mut t, P0, mtg_engine::mana::ManaType::B, 3);
    assert!(!castable(&mut t, P0, mr));
}

#[test]
fn tomb_of_urami_sacrifices_itself_too() {
    cr!("602.2b", "118.3");
    ruling!(
        "Tomb of Urami",
        "Activating Tomb of Urami’s second ability requires sacrificing all lands you control, including Tomb of Urami."
    );
    supported("Tomb of Urami");
    let mut t = TestGame::new(2);
    let tomb = t.battlefield(P0, "Tomb of Urami");
    t.lands(P0, "Swamp", 4);
    t.battlefield(P1, "Forest");
    let i = crate::r_s06_common::activate_containing(&mut t, P0, tomb, "Urami");
    assert!(i.is_ok());
    t.resolve_all();
    assert_eq!(lands_of(&t, P0), 0);
    assert_eq!(lands_of(&t, P1), 1);
    assert_eq!(crate::r_s25_common::creature_tokens(&t, P0), 1);
}

#[test]
fn jarad_sacrifice_costs_and_last_known_power() {
    cr!("118.3", "305.6", "608.2h");
    ruling!(
        "Jarad, Golgari Lich Lord",
        "To activate Jarad's last ability, you must sacrifice two lands. It doesn't matter what other land types the two lands have, as long as one is a Swamp and one is a Forest. Even if one of the lands you sacrifice is both a Swamp and a Forest, you have to sacrifice another land that is a Swamp or a Forest."
    );
    ruling!(
        "Jarad, Golgari Lich Lord",
        "You can sacrifice any land with the land type Swamp or Forest, not just ones named Swamp or Forest. For example, you could sacrifice two Overgrown Tombs."
    );
    ruling!(
        "Jarad, Golgari Lich Lord",
        "The life loss is based on the power of the sacrificed creature as it last existed on the battlefield."
    );
    ruling!(
        "Jarad, Golgari Lich Lord",
        "Jarad's first ability applies only when Jarad is on the battlefield."
    );
    supported("Jarad, Golgari Lich Lord");
    // In the graveyard: a 2/2 regardless of creature cards there; one Overgrown Tomb isn't
    // enough to return it, two are.
    let mut t = TestGame::new(2);
    let jarad = t.graveyard(P0, "Jarad, Golgari Lich Lord");
    t.graveyard(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(jarad), (2, 2));
    let tomb = t.battlefield(P0, "Overgrown Tomb");
    let r = crate::r_s06_common::activate_containing(&mut t, P0, jarad, "Return");
    assert!(r.is_err());
    assert!(t.on_battlefield(tomb));
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, jarad));
    t.battlefield(P0, "Overgrown Tomb");
    assert!(crate::r_s02_common::can_activate(&mut t, P0, jarad));
    crate::r_s06_common::activate_containing(&mut t, P0, jarad, "Return").unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Jarad, Golgari Lich Lord"));
    assert_eq!(lands_of(&t, P0), 0);
    // On the battlefield: +1/+1 per creature card; sacrificing a pumped creature.
    let mut t = TestGame::new(2);
    let jarad = t.battlefield(P0, "Jarad, Golgari Lich Lord");
    t.graveyard(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(jarad), (3, 3));
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s29_common::put_counters(&mut t, bears, "+1/+1", 3);
    t.lands(P0, "Overgrown Tomb", 3);
    t.answer_choose(P0, &[obj(bears)]);
    crate::r_s06_common::activate_containing(&mut t, P0, jarad, "loses life").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
}

#[test]
fn exalted_dragon_needs_a_land_sacrificed_to_attack() {
    cr!("508.1g", "508.1h");
    ruling!(
        "Exalted Dragon",
        "You have to sacrifice a land at the time you declare this creature as an attacker if you want it to attack. This is additive with any other attack costs."
    );
    supported("Exalted Dragon");
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Exalted Dragon");
    t.battlefield(P1, "Ghostly Prison");
    // A land to sacrifice but no mana for Ghostly Prison: can't attack.
    let land = t.battlefield(P0, "Forest");
    t.g.tap(land);
    t.attack(&[(dragon, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20);
    assert!(t.on_battlefield(land));
    // With {2} more: pays both.
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Exalted Dragon");
    t.battlefield(P1, "Ghostly Prison");
    t.lands(P0, "Wastes", 3);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.attack(&[(dragon, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 15);
    assert_eq!(lands_of(&t, P0), 2);
}

#[test]
fn lake_of_the_dead_sacrifice_ability_is_tapping_for_mana() {
    cr!("605.1a", "106.12");
    ruling!(
        "Lake of the Dead",
        "The tap and sacrifice counts as tapping the land for mana."
    );
    supported("Lake of the Dead");
    supported("Heartbeat of Spring");
    let mut t = TestGame::new(2);
    let lake = t.battlefield(P0, "Lake of the Dead");
    let swamp = t.battlefield(P0, "Swamp");
    t.battlefield(P0, "Heartbeat of Spring");
    t.answer_choose(P0, &[obj(swamp)]);
    crate::r_s06_common::activate_containing(&mut t, P0, lake, "Sacrifice").unwrap();
    assert_eq!(t.stack_len(), 0);
    // {B}{B}{B}{B}, plus one more from Heartbeat of Spring ("Whenever a player taps a
    // land for mana, that player adds one mana of any type that land produced.").
    assert_eq!(
        crate::r_s24_common::pool(&t, P0, mtg_engine::mana::ManaType::B),
        5
    );
}

#[test]
fn goblin_trenches_token_names_and_types() {
    cr!("111.4");
    ruling!(
        "Goblin Trenches",
        "The tokens are named \"Goblin Soldier Token\" and have the creature types \"Goblin\" and \"Soldier.\""
    );
    supported("Goblin Trenches");
    let mut t = TestGame::new(2);
    let gt = t.battlefield(P0, "Goblin Trenches");
    let land = t.battlefield(P0, "Plains");
    t.lands(P0, "Wastes", 2);
    t.answer_choose(P0, &[obj(land)]);
    t.activate(P0, gt, 0, &[]).unwrap();
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 2);
    for tok in toks {
        let o = t.obj_now(tok);
        assert_eq!(o.chars.name.as_str(), "Goblin Soldier Token");
        assert!(o.chars.has_subtype("Goblin") && o.chars.has_subtype("Soldier"));
    }
}

#[test]
fn reprocess_and_highway_robbery_choices() {
    cr!("701.21a", "608.2d");
    ruling!(
        "Reprocess",
        "You can sacrifice zero permanents if you want to."
    );
    ruling!(
        "Highway Robbery",
        "You can’t choose to discard a card if you have no cards in hand. Similarly, you can’t choose to sacrifice a land if you control no lands."
    );
    supported("Highway Robbery");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[]);
    cast_and_resolve(&mut t, P0, "Reprocess", &[]);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.hand_size(P0), 0);
    // Highway Robbery with an empty hand and no lands other than those tapped for it:
    // only the land option, and with no lands at all, nothing.
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    crate::r_s04_common::add_mana(&mut t, P0, mtg_engine::mana::ManaType::R, 2);
    let hr = t.hand(P0, "Highway Robbery");
    t.cast_with(P0, hr, &[]).unwrap();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0, "nothing given up, nothing drawn");
    let offered = crate::r_s32_common::options_offered(&t, from);
    assert!(offered.iter().all(|o| o.len() <= 1), "{offered:?}");
}

#[test]
fn jeweled_spirit_chooses_on_resolution() {
    cr!("608.2d", "702.16a");
    ruling!(
        "Jeweled Spirit",
        "You choose the color (or artifacts) on resolution."
    );
    supported("Jeweled Spirit");
    let mut t = TestGame::new(2);
    let spirit = t.battlefield(P0, "Jeweled Spirit");
    let lands = t.lands(P0, "Plains", 2);
    t.answer_choose(P0, &[obj(lands[0]), obj(lands[1])]);
    t.activate(P0, spirit, 0, &[]).unwrap();
    let from = t.asked().len();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(crate::r_s32_common::options_offered(&t, from).len() >= 1);
}

// ---------------------------------------------------------------------------------------
// Heart of Yavimaya / Kjeldoran Outpost: "If this land would enter, sacrifice a Forest
// instead. If you do, put this land onto the battlefield. If you don't, put it into its
// owner's graveyard."
// ---------------------------------------------------------------------------------------

#[test]
fn sacrifice_on_entering_lands_however_they_enter() {
    cr!("614.1c", "614.12");
    ruling!(
        "Heart of Yavimaya",
        "You have to sacrifice a forest before this card is put onto the battlefield, and no matter how it is put onto the battlefield."
    );
    ruling!(
        "Kjeldoran Outpost",
        "You have to sacrifice a plains before this card is put onto the battlefield. You have to do this no matter how it is put onto the battlefield."
    );
    supported("Heart of Yavimaya");
    supported("Kjeldoran Outpost");
    for (name, basic) in [
        ("Heart of Yavimaya", "Forest"),
        ("Kjeldoran Outpost", "Plains"),
    ] {
        // Put onto the battlefield by an effect: without the land, it goes to the graveyard.
        let mut t = TestGame::new(2);
        let card = t.hand(P0, name);
        move_to(&mut t, card, Zone::Battlefield);
        assert!(t.in_graveyard(P0, name), "{name}: {}", t.dump_log());
        // With one: sacrificed.
        let mut t = TestGame::new(2);
        let b = t.battlefield(P0, basic);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(b)]);
        let card = t.hand(P0, name);
        move_to(&mut t, card, Zone::Battlefield);
        assert_eq!(t.named_on_battlefield(name).len(), 1, "{name}");
        assert!(t.in_graveyard(P0, basic), "{name}");
        // Played: same.
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let card = t.hand(P0, name);
        t.play_land(P0, card).unwrap();
        t.settle();
        assert!(t.in_graveyard(P0, name), "{name}");
    }
}

// ---------------------------------------------------------------------------------------
// Hew the Entwood
// ---------------------------------------------------------------------------------------

#[test]
fn hew_the_entwood_triggers_wait_and_go_on_the_stack_in_apnap_order() {
    cr!("603.3b", "101.4", "608.2c");
    ruling!(
        "Hew the Entwood",
        "Any abilities that trigger during the resolution of Hew the Entwood will wait to be put on the stack until everything is put on the battlefield and resolution is complete."
    );
    supported("Hew the Entwood");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ankh of Mishra");
    t.battlefield(P1, "Ankh of Mishra");
    let sac = t.lands(P0, "Forest", 2);
    let top = stack_library(&mut t, P0, &["Swamp", "Island"]);
    t.answer_choose(P0, &[obj(sac[0]), obj(sac[1])]);
    t.answer_choose(P0, &[obj(top[0]), obj(top[1])]);
    cast_new(&mut t, P0, "Hew the Entwood", &[]);
    t.settle();
    t.resolve();
    // Both lands entered (tapped); four Ankh triggers (each Ankh, each land) are waiting,
    // P1's on top.
    let entered: Vec<ObjectId> = top.iter().map(|c| t.g.current(*c)).collect();
    assert!(entered
        .iter()
        .all(|c| t.on_battlefield(*c) && t.obj_now(*c).tapped));
    assert_eq!(t.stack_len(), 4);
    let ctl: Vec<PlayerId> = t.g.stack.iter().map(|s| t.g.obj(*s).controller).collect();
    assert_eq!(ctl, vec![P0, P0, P1, P1]);
    t.resolve_all();
    assert_eq!(t.life(P0), 12);
}

// ---------------------------------------------------------------------------------------
// Old-Growth Troll: "When Old-Growth Troll dies, if it was a creature, return it to the
// battlefield. It's an Aura enchantment with enchant Forest you control and ..."
// ---------------------------------------------------------------------------------------

#[test]
fn old_growth_troll_returns_under_its_controller_or_stays_in_the_graveyard() {
    cr!("603.6c", "303.4f", "303.4g", "110.2");
    ruling!(
        "Old-Growth Troll",
        "If there’s nothing for Old-Growth Troll to legally enchant, it remains in its owner’s graveyard."
    );
    ruling!(
        "Old-Growth Troll",
        "If you control but don’t own Old-Growth Troll, you (not its owner) will return it to the battlefield when it dies. You’ll choose which Forest you control it will enchant as it returns to the battlefield."
    );
    supported("Old-Growth Troll");
    // No Forest: stays in the graveyard.
    let mut t = TestGame::new(2);
    let troll = t.battlefield(P0, "Old-Growth Troll");
    t.battlefield(P1, "Forest");
    destroy(&mut t, troll);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Old-Growth Troll"));
    // P1 controls P0's Troll: it returns under P1's control, on P1's chosen Forest.
    let mut t = TestGame::new(2);
    let troll = t.battlefield(P0, "Old-Growth Troll");
    t.battlefield(P0, "Forest");
    t.battlefield(P1, "Forest");
    let f2 = t.battlefield(P1, "Forest");
    crate::r_s06_common::give_control(&mut t, troll, P1);
    t.answer_choose(P1, &[obj(f2)]);
    destroy(&mut t, troll);
    t.resolve_all();
    let aura = t.g.current(troll);
    assert!(t.on_battlefield(aura), "{}", t.dump_log());
    assert_eq!(t.obj_now(aura).controller, P1);
    assert_eq!(crate::r_s06_common::attached_to(&t, aura), Some(obj(f2)));
}

#[test]
fn old_growth_troll_copies() {
    cr!("707.2", "111.7", "603.6c");
    ruling!(
        "Old-Growth Troll",
        "If a token is a copy of Old-Growth Troll, it won’t return from its owner’s graveyard."
    );
    ruling!(
        "Old-Growth Troll",
        "If a nontoken permanent that's a copy of Old-Growth Troll dies while it is a creature, it will return to the battlefield as an Aura with both its own normal abilities and the two new abilities granted by old-Growth Troll."
    );
    supported("Clone");
    let mut t = TestGame::new(2);
    let troll = t.battlefield(P0, "Old-Growth Troll");
    t.battlefield(P0, "Forest");
    let copy = crate::r_s17_common::token_copy(&mut t, P0, troll)[0];
    destroy(&mut t, copy);
    t.resolve_all();
    assert!(t.named_on_battlefield("Old-Growth Troll").len() == 1);
    assert_eq!(
        t.g.permanents()
            .filter(|o| o.is(CardType::Enchantment))
            .count(),
        0
    );
    // A Clone copying it returns as an Aura: Clone's own card, an Aura enchantment.
    let mut t = TestGame::new(2);
    let troll = t.battlefield(P0, "Old-Growth Troll");
    let forest = t.battlefield(P0, "Forest");
    t.answer_choose(P0, &[obj(troll)]);
    let clone = t.enter(P0, "Clone");
    t.resolve_all();
    assert_eq!(t.obj_now(clone).chars.name.as_str(), "Old-Growth Troll");
    let forest_abilities = |t: &mut TestGame| {
        t.g.recompute();
        t.obj_now(forest)
            .chars
            .abilities
            .iter()
            .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
            .count()
    };
    let before = forest_abilities(&mut t);
    t.answer_choose(P0, &[]); // Clone doesn't copy anything as it returns
    t.answer_choose(P0, &[obj(forest)]);
    destroy(&mut t, clone);
    t.resolve_all();
    let back = t.g.current(clone);
    assert!(t.on_battlefield(back), "{}", t.dump_log());
    let o = t.obj_now(back);
    assert_eq!(o.chars.name.as_str(), "Clone");
    assert!(o.is(CardType::Enchantment) && o.chars.has_subtype("Aura"));
    assert!(!o.is(CardType::Creature));
    assert_eq!(
        crate::r_s06_common::attached_to(&t, back),
        Some(obj(forest))
    );
    // The two abilities the return grants: the enchanted Forest has both.
    assert_eq!(forest_abilities(&mut t), before + 2);
}
