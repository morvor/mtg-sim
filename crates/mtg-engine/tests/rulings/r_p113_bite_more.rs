//! Rulings batch P113 — more one-sided fights: Ram Through's trample excess (CR 120.4a),
//! Waltz of Rage's damage and delayed trigger (CR 603.7), Graceful Takedown's groups of
//! targets (CR 608.2b), Feral Encounter's delayed trigger at the next combat (CR 603.7),
//! and Garruk's Wolves dying alongside him (CR 704.3).

use crate::r_p113_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s05_common::run_from;
use crate::r_s06_common::attach_new;
use crate::r_s07_common::damage_on;
use mtg_engine::ability::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Gives `id` the keyword until end of turn (from its controller).
fn grant(t: &mut TestGame, id: ObjectId, kw: KeywordKind) {
    let c = t.obj_now(id).controller;
    run_from(
        t,
        c,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddKeyword(Keyword::new(kw))],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(t.g.current(id))],
    );
}

#[test]
fn ram_through_excess_only_with_trample_and_deathtouch_makes_one_lethal() {
    cr!("120.4a", "702.2c", "702.19c");
    ruling!(
        "Ram Through",
        "If the target creature you control has deathtouch, 1 damage from it is lethal."
    );
    // Without trample: no excess.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_modes(&mut t, P0, "Ram Through", &[], &[ent(giant), ent(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 20);
    // With trample: 2 to the 2/2, 1 to its controller.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    grant(&mut t, giant, KeywordKind::Trample);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_modes(&mut t, P0, "Ram Through", &[], &[ent(giant), ent(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 19);
    // Deathtouch and trample: 1 to the 6/4 Wurm is lethal, the other 2 to P1.
    let mut t = TestGame::new(2);
    let viper = t.battlefield(P0, "Ambush Viper");
    grant(&mut t, viper, KeywordKind::Trample);
    run_from(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::ModifyPT(Value::Const(1), Value::Const(0))],
            duration: Duration::EndOfTurn,
        },
        &[ent(viper)],
    );
    assert_eq!(t.pt(viper), (3, 1));
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_modes(&mut t, P0, "Ram Through", &[], &[ent(viper), ent(wurm)]);
    t.resolve_all();
    assert!(!t.on_battlefield(wurm), "1 deathtouch damage is lethal");
    assert_eq!(t.life(P1), 18);
}

#[test]
fn ram_through_damage_is_split_before_replacement_effects() {
    cr!("120.4a", "614.1a", "120.3");
    ruling!(
        "Ram Through",
        "Once you’ve determined how much damage is excess, the creature you control simultaneously deals damage to the creature and to its controller. This damage may be modified by replacement or prevention effects."
    );
    supported("Furnace of Rath");
    // Furnace of Rath doubles both parts: Hill Giant (3, trample) at Grizzly Bears: 2
    // and 1 become 4 to the Bears and 2 to P1 (not 3 + ... from a doubled 6).
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Furnace of Rath");
    let giant = t.battlefield(P0, "Hill Giant");
    grant(&mut t, giant, KeywordKind::Trample);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_modes(&mut t, P0, "Ram Through", &[], &[ent(giant), ent(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn waltz_of_rage_damage_comes_from_the_creature_with_its_power_as_it_resolves() {
    cr!("120.3f", "608.2h", "702.15b");
    ruling!(
        "Waltz of Rage",
        "The creature is the source of the damage, not Waltz of Rage."
    );
    ruling!(
        "Waltz of Rage",
        "Use the power of the target creature as Waltz of Rage resolves to determine how much damage that creature deals to each other creature."
    );
    // Vampire Nighthawk (2/3 lifelink, deathtouch): its lifelink and deathtouch apply.
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let angel = t.battlefield(P1, "Serra Angel");
    cast_modes(&mut t, P0, "Waltz of Rage", &[], &[ent(hawk)]);
    t.resolve_all();
    assert!(!t.on_battlefield(wurm) && !t.on_battlefield(angel));
    assert_eq!(t.life(P0), 24, "lifelink: 2 damage to each of two creatures");
    assert!(t.on_battlefield(hawk));
    // The power as it resolves: Hill Giant pumped by +3/+3 in response deals 6.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let brontodon = t.battlefield(P1, "Ancient Brontodon");
    cast_modes(&mut t, P0, "Waltz of Rage", &[], &[ent(giant)]);
    run_from(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::ModifyPT(Value::Const(3), Value::Const(3))],
            duration: Duration::EndOfTurn,
        },
        &[ent(giant)],
    );
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
    assert_eq!(damage_on(&t, brontodon), 6);
}

#[test]
fn waltz_of_rage_with_an_illegal_target_creates_no_delayed_trigger() {
    cr!("608.2b", "603.7a");
    ruling!(
        "Waltz of Rage",
        "If the target creature is an illegal target when Waltz of Rage tries to resolve, it won't resolve and none of its effects will happen."
    );
    let run = |illegal: bool| {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let giant = t.battlefield(P0, "Hill Giant");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let mine = t.battlefield(P0, "Llanowar Elves");
        let top = t.library_top(P0, "Forest");
        cast_modes(&mut t, P0, "Waltz of Rage", &[], &[ent(giant)]);
        if illegal {
            spoil(&mut t, giant, Spoil::Hexproof);
        }
        t.resolve_all();
        // Another creature P0 controls dies later this turn.
        if t.on_battlefield(mine) {
            crate::r_s02_common::destroy(&mut t, mine);
            t.resolve_all();
        }
        (t, bears, top)
    };
    let (t, bears, top) = run(false);
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.zone(top), Zone::Exile, "Llanowar Elves died: the top card is exiled");
    let (t, bears, top) = run(true);
    assert!(t.on_battlefield(bears));
    assert_eq!(damage_on(&t, bears), 0);
    assert_eq!(t.zone(top), Zone::Library(P0), "no delayed trigger");
}

#[test]
fn waltz_of_rage_exiled_land_follows_timing_rules() {
    cr!("305.1", "603.7b", "116.2a");
    ruling!(
        "Waltz of Rage",
        "You pay all costs and follow all timing rules for cards played with the permission granted by Waltz of Rage's delayed triggered ability."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    // Grizzly Bears (P0's) dies to the Giant's 3 damage: the delayed trigger exiles the
    // Forest.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let top = t.library_top(P0, "Forest");
    cast_modes(&mut t, P0, "Waltz of Rage", &[], &[ent(giant)]);
    t.settle();
    // Resolve the spell only: the death trigger goes on the stack.
    t.resolve();
    t.settle();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.stack_len(), 1, "the delayed trigger");
    t.resolve();
    t.settle();
    let land = t.g.current(top);
    assert_eq!(t.zone(land), Zone::Exile);
    // Not during combat.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(t.play_land(P0, land).is_err());
    // In a main phase with an empty stack, it can be played.
    t.set_step(P0, Step::PostcombatMain);
    t.play_land(P0, land).expect("play the exiled land");
    assert!(t.on_battlefield(land));
}

/// P0's enchanted Grizzly Bears (Holy Strength: 3/4) and enchanted Hill Giant (Holy
/// Strength: 4/5), and Serra Angel (4/4), target P1's Ancient Brontodon (9/9) with
/// Graceful Takedown. Returns (game, bears, giant, angel, brontodon, giant's aura).
fn takedown() -> (TestGame, ObjectId, ObjectId, ObjectId, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Holy Strength", bears);
    let giant = t.battlefield(P0, "Hill Giant");
    let aura = attach_new(&mut t, P0, "Holy Strength", giant);
    let angel = t.battlefield(P0, "Serra Angel");
    let bronto = t.battlefield(P1, "Ancient Brontodon");
    supported("Graceful Takedown");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Graceful Takedown");
    let card = t.hand(P0, "Graceful Takedown");
    t.answer_targets(P0, &[ent(bears), ent(giant)]);
    t.answer_targets(P0, &[ent(angel)]);
    t.answer_targets(P0, &[ent(bronto)]);
    t.cast(P0, card).go();
    (t, bears, giant, angel, bronto, aura)
}

#[test]
fn graceful_takedown_with_some_targets_illegal() {
    cr!("608.2b", "115.3", "601.2c");
    ruling!(
        "Graceful Takedown",
        "If one of the target creatures you control is an illegal target as Graceful Takedown resolves (perhaps because it's no longer enchanted or is no longer on the battlefield), the remaining legal target creatures you control will still deal damage equal to their power."
    );
    ruling!(
        "Graceful Takedown",
        "If the last target creature is an illegal target as Graceful Takedown resolves, or if all of the target creatures you control are illegal targets, no creature deals or is dealt damage."
    );
    // All legal: 3 + 4 + 4.
    let (mut t, .., bronto, _) = takedown();
    t.resolve_all();
    assert!(!t.on_battlefield(bronto), "11 damage to a 9/9");
    // The Giant is no longer enchanted: 3 + 4.
    let (mut t, .., bronto, aura) = takedown();
    crate::r_s02_common::destroy(&mut t, aura);
    t.resolve_all();
    assert_eq!(damage_on(&t, bronto), 7);
    // The last target is illegal: nothing.
    let (mut t, bears, giant, angel, bronto, _) = takedown();
    spoil(&mut t, bronto, Spoil::Hexproof);
    t.resolve_all();
    assert_eq!(damage_on(&t, bronto), 0);
    for c in [bears, giant, angel] {
        assert_eq!(damage_on(&t, c), 0);
    }
    // All of P0's creatures are illegal: nothing.
    let (mut t, bears, giant, angel, bronto, _) = takedown();
    for c in [bears, giant, angel] {
        spoil(&mut t, c, Spoil::Hexproof);
    }
    t.resolve_all();
    assert_eq!(damage_on(&t, bronto), 0);
}

#[test]
fn friendly_rivalry_second_creature_must_be_legendary() {
    cr!("601.2c", "115.3");
    supported("Friendly Rivalry");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    let giant = t.battlefield(P0, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Friendly Rivalry");
    let card = t.hand(P0, "Friendly Rivalry");
    let from = t.asked().len();
    t.answer_targets(P0, &[ent(bears)]);
    t.answer_targets(P0, &[ent(isamaru)]);
    t.answer_targets(P0, &[ent(wurm)]);
    t.cast(P0, card).go();
    // The second target's candidates: legendary creatures P0 controls other than the
    // first target.
    let second = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseTargets { candidates, .. } => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .nth(1)
        .expect("second target");
    assert_eq!(second, vec![ent(isamaru)]);
    let _ = giant;
    t.resolve_all();
    assert!(!t.on_battlefield(wurm), "2 + 2 damage to a 6/4");
}

#[test]
fn feral_encounter_delayed_trigger_needs_a_later_combat_this_turn() {
    cr!("603.7a", "603.7b", "506.1");
    ruling!(
        "Feral Encounter",
        "If you cast Feral Encounter and no combat phases occur later in that turn, the delayed triggered ability won't trigger."
    );
    let run = |step: Step| {
        let mut t = TestGame::new(2);
        t.set_step(P0, step);
        let giant = t.battlefield(P0, "Hill Giant");
        let bears = t.battlefield(P1, "Grizzly Bears");
        cast_modes(&mut t, P0, "Feral Encounter", &[], &[]);
        t.resolve_all();
        t.answer_targets(P0, &[ent(giant)]);
        t.answer_targets(P0, &[ent(bears)]);
        // On to P0's next turn's beginning of combat.
        t.advance_to(P1, Step::Upkeep);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.resolve_all();
        (t, bears)
    };
    let (t, bears) = run(Step::PostcombatMain);
    assert!(t.on_battlefield(bears), "no combat later that turn");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_modes(&mut t, P0, "Feral Encounter", &[], &[]);
    t.resolve_all();
    t.answer_targets(P0, &[ent(giant)]);
    t.answer_targets(P0, &[ent(bears)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(!t.on_battlefield(bears), "the Giant deals 3 at the next combat");
}

#[test]
fn garruk_dies_before_his_wolf_trigger_can_save_him() {
    cr!("704.3", "704.5i", "603.10a");
    ruling!(
        "Garruk, Cursed Huntsman",
        "If lethal damage is dealt to one of Garruk's Wolf tokens at the same time that Garruk's loyalty is brought to 0 or less, Garruk is put into your graveyard before the Wolf's triggered ability can save him."
    );
    supported("Garruk, Cursed Huntsman");
    // P1 attacks Garruk with Craw Wurm (6) and P0 with Hill Giant; a Wolf blocks the
    // Giant. `lethal`: Garruk has 6 loyalty or fewer.
    let run = |loyalty: i32| {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let garruk = t.battlefield(P0, "Garruk, Cursed Huntsman");
        t.activate(P0, garruk, 0, &[]).expect("0: wolves");
        t.resolve_all();
        let wolves: Vec<ObjectId> = t
            .g
            .permanents()
            .filter(|o| o.controller == P0 && o.chars.has_subtype("Wolf"))
            .map(|o| o.id)
            .collect();
        assert_eq!(wolves.len(), 2);
        let now = t.counters(garruk, counters::LOYALTY) as i32;
        assert!(loyalty >= now);
        let g_now = t.g.current(garruk);
        t.g.add_counters(
            ent(g_now),
            counters::LOYALTY,
            (loyalty - now) as u32,
            None,
        );
        let wurm = t.battlefield(P1, "Craw Wurm");
        let giant = t.battlefield(P1, "Hill Giant");
        t.set_step(P1, Step::BeginningOfCombat);
        let g_now = t.g.current(garruk);
        attack_with(&mut t, &[(wurm, ent(g_now)), (giant, Entity::Player(P0))]);
        block_and_finish(&mut t, P0, &[(wolves[0], giant)]);
        t.resolve_all();
        (t, garruk, wolves)
    };
    // Garruk survives the Wurm (7 loyalty): the Wolf's trigger adds a counter.
    let (t, garruk, wolves) = run(7);
    assert!(!t.on_battlefield(wolves[0]));
    assert_eq!(t.counters(garruk, counters::LOYALTY), 2);
    // Garruk at 6 dies at the same time as the Wolf.
    let (t, garruk, wolves) = run(6);
    assert!(!t.on_battlefield(wolves[0]));
    assert_eq!(t.zone(garruk), Zone::Graveyard(P0));
}

/// "~ deals 3 damage to target player and 1 damage to each creature that player
/// controls." (Radiating Lightning): a damage list whose second recipient follows a
/// target phrase, compiled with the same fix as Cuombajj Witches.
#[test]
fn radiating_lightning_damages_the_player_and_their_creatures() {
    cr!("120.3", "115.1a");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Llanowar Elves");
    let theirs = t.battlefield(P1, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_modes(&mut t, P0, "Radiating Lightning", &[], &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(!t.on_battlefield(theirs));
    assert_eq!(damage_on(&t, bears), 1);
    assert!(t.on_battlefield(mine));
    assert_eq!(t.life(P0), 20);
}
