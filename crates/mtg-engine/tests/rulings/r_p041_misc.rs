//! Rulings batch P041 — the other rulings on cards with damage prevention: Guardian
//! Archon's chosen opponent, Kiora's extra land, combat damage steps and trample,
//! counter limits (Rasputin Dreamweaver), mana spent (Kurbis), modal spells with illegal
//! targets, state triggers (CR 603.8), costs, Sevinne, Nefarious Lich and Khârn.

use crate::r_p041_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Moves the permanent to exile and back onto the battlefield under its owner's control
/// (a new object, CR 400.7); returns the new object.
fn flicker(t: &mut TestGame, id: ObjectId) -> ObjectId {
    let id = t.g.current(id);
    let owner = t.obj_now(id).owner;
    let e = t
        .g
        .move_object(id, Zone::Exile, mtg_engine::events::MoveCause::Effect, None)
        .expect("exiled");
    let new = t
        .g
        .move_object_ev(mtg_engine::replacement::MoveEv {
            obj: e,
            to: Zone::Battlefield,
            pos: mtg_engine::ability::LibraryPosition::Top,
            cause: mtg_engine::events::MoveCause::Effect,
            by: Some(owner),
            etb: mtg_engine::replacement::EtbInfo {
                controller: Some(owner),
                ..Default::default()
            },
            source: None,
        })
        .expect("returned");
    t.g.flush_events();
    t.settle();
    new
}

/// Whether a source `p` controls can deal damage to `to` now: 1 damage from a new
/// Grizzly Bears of `p`'s (protection from a player prevents it, CR 702.16e, 702.16k).
fn damaged_by(t: &mut TestGame, p: PlayerId, to: Entity) -> bool {
    let src = t.battlefield(p, "Grizzly Bears");
    let before = damage_events(t).len();
    damage(t, src, to, 1);
    damage_events(t).len() > before
}

// --- Guardian Archon --------------------------------------------------------------------------

#[test]
fn guardian_archon_returning_is_a_new_object_with_a_new_choice() {
    cr!("400.7", "602.5b", "702.16k");
    ruling!(
        "Guardian Archon",
        "If Guardian Archon leaves the battlefield and returns, it’s considered a new object. You may choose a new opponent and you may activate its last ability again."
    );
    ruling!(
        "Guardian Archon",
        "If you control more than one Guardian Archon, you may choose a different opponent for each one."
    );
    supported("Guardian Archon");
    let mut t = TestGame::new(3);
    t.answer_choose(P0, &[pl(P1)]);
    let archon = t.enter(P0, "Guardian Archon");
    t.g.flush_events();
    t.settle();
    activate_resolve(&mut t, P0, archon, 0, &[obj(archon)]);
    assert!(!damaged_by(&mut t, P1, pl(P0)), "protection from P1");
    assert!(damaged_by(&mut t, P2, pl(P0)));

    assert!(t.activate(P0, archon, 0, &[obj(archon)]).is_err(), "only once");
    t.clear_answers();
    // It returns: a new object, a new opponent, a new activation.
    t.answer_choose(P0, &[pl(P2)]);
    let archon = flicker(&mut t, archon);
    // A second Archon choosing P1.
    t.answer_choose(P0, &[pl(P1)]);
    let other = t.enter(P0, "Guardian Archon");
    t.g.flush_events();
    t.settle();
    activate_resolve(&mut t, P0, archon, 0, &[obj(archon)]);
    assert!(!damaged_by(&mut t, P2, pl(P0)), "protection from P2");
    assert!(damaged_by(&mut t, P1, obj(other)), "not from P1 yet");
    activate_resolve(&mut t, P0, other, 0, &[obj(other)]);
    assert!(!damaged_by(&mut t, P1, obj(other)), "the other Archon: P1");
}

#[test]
fn guardian_archon_in_a_two_player_game_chooses_the_opponent() {
    cr!("102.2");
    ruling!(
        "Guardian Archon",
        "In a two-player game, the chosen player won’t be much of a secret."
    );
    let mut t = TestGame::new(2);
    let archon = t.enter(P0, "Guardian Archon");
    t.g.flush_events();
    t.settle();
    activate_resolve(&mut t, P0, archon, 0, &[obj(archon)]);
    assert!(!damaged_by(&mut t, P1, pl(P0)));
    assert!(!damaged_by(&mut t, P1, obj(archon)));
}

// --- Kiora, the Crashing Wave -----------------------------------------------------------------

#[test]
fn kiora_extra_land_follows_normal_timing_and_survives_kiora_leaving() {
    cr!("305.2", "305.2a", "113.7a");
    ruling!(
        "Kiora, the Crashing Wave",
        "If Kiora leaves the battlefield after her second ability is activated but before it resolves, you’ll still be able to play an additional land after the ability resolves."
    );
    ruling!(
        "Kiora, the Crashing Wave",
        "Kiora’s second ability allows you to play an additional land during your main phase. Doing so follows the normal timing rules for playing lands. In particular, you won’t play a land as that ability resolves."
    );
    let mut t = TestGame::new(2);
    let kiora = t.battlefield(P0, "Kiora, the Crashing Wave");
    let drawn = t.library_top(P0, "Forest");
    let island = t.hand(P0, "Island");
    activate(&mut t, P0, kiora, 1, &[]);
    exile_now(&mut t, kiora);
    t.resolve_all();
    assert_eq!(t.zone(drawn), Zone::Hand(P0), "the card is drawn first");
    let lands = t.g.permanents().filter(|o| o.chars.is_land()).count();
    assert_eq!(lands, 0, "no land is played as it resolves");
    assert!(t.play_land(P0, island).is_ok());
    let drawn = t.g.current(drawn);
    assert!(t.play_land(P0, drawn).is_ok(), "the additional land");
    let third = t.hand(P0, "Plains");
    assert!(t.play_land(P0, third).is_err());
}

#[test]
fn kiora_plus_one_can_target_any_permanent_an_opponent_controls() {
    cr!("115.1");
    ruling!(
        "Kiora, the Crashing Wave",
        "Kiora’s first ability can target any permanent an opponent controls, not just one that can deal or be dealt damage."
    );
    let mut t = TestGame::new(2);
    let kiora = t.battlefield(P0, "Kiora, the Crashing Wave");
    let forest = t.battlefield(P1, "Forest");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let from = t.asked().len();
    activate_resolve(&mut t, P0, kiora, 0, &[obj(forest)]);
    let offered = crate::r_p016_common::target_choices(&t, P0, from, "");
    assert!(offered[0].contains(&obj(forest)) && offered[0].contains(&obj(anthem)));
    assert_eq!(t.counters(kiora, "loyalty"), 3);
}

// --- Combat -----------------------------------------------------------------------------------

#[test]
fn phytohydra_grows_from_first_strike_damage_before_dealing_its_own() {
    cr!("510.4", "614.1a");
    ruling!(
        "Phytohydra",
        "If Phytohydra blocks or is blocked by a creature with first strike or double strike, Phytohydra will get the counters during the first-strike combat damage step before it deals its own combat damage during the second combat damage step."
    );
    let mut t = TestGame::new(2);
    let hydra = t.battlefield(P0, "Phytohydra");
    let knight = t.battlefield(P1, "White Knight");
    fight_it_out(&mut t, &[hydra], &[(knight, hydra)]);
    assert_eq!(t.counters(hydra, "+1/+1"), 2);
    assert!(!t.on_battlefield(knight), "the 3/3 Hydra dealt 3 damage");
}

#[test]
fn seraph_of_the_sword_still_needs_lethal_damage_assigned_from_a_trampler() {
    cr!("702.19c", "510.1c");
    ruling!(
        "Seraph of the Sword",
        "If Seraph of the Sword blocks a creature with trample, that creature’s controller must assign 3 damage to Seraph of the Sword"
    );
    supported("Seraph of the Sword");
    supported("Colossal Dreadmaw");
    let mut t = TestGame::new(2);
    let dreadmaw = t.battlefield(P0, "Colossal Dreadmaw");
    let seraph = t.battlefield(P1, "Seraph of the Sword");
    // P0 tries to assign only 1 to the Seraph: not allowed.
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![1, 5]));
    let from = t.asked().len();
    fight_it_out(&mut t, &[dreadmaw], &[(seraph, dreadmaw)]);
    let lethal: Vec<Vec<u32>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::AssignCombatDamage { lethal, .. } => Some(lethal.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(lethal.first().map(|l| l[0]), Some(3));
    assert!(t.life(P1) >= 17, "at most 3 tramples over: {}", t.life(P1));
    assert!(t.on_battlefield(seraph));
    assert_eq!(damage_marked(&t, seraph), 0);
}

#[test]
fn loafing_giant_blocking_two_creatures_triggers_once() {
    cr!("509.3a", "603.2c");
    ruling!(
        "Loafing Giant",
        "If it somehow blocks multiple creatures in a single combat, the triggered ability only triggers once."
    );
    supported("Loafing Giant");
    supported("Brave the Sands");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Loafing Giant");
    t.battlefield(P1, "Brave the Sands");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Elite Vanguard");
    let library = t.library_size(P1);
    fight_it_out(&mut t, &[a, b], &[(giant, a), (giant, b)]);
    assert_eq!(t.library_size(P1), library - 1, "one card milled");
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
}

#[test]
fn sewers_of_estark_on_a_creature_not_in_combat_does_nothing() {
    cr!("608.2c");
    ruling!(
        "Sewers of Estark",
        "If the creature is neither an attacking creature nor a blocking creature at the time the spell resolves, it will have no effect."
    );
    supported("Sewers of Estark");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Sewers of Estark", &[obj(giant)]);
    fight_it_out(&mut t, &[giant], &[(bears, giant)]);
    assert!(!t.on_battlefield(bears), "the Giant could be blocked");
    assert_eq!(damage_marked(&t, giant), 2);
    assert_eq!(t.life(P1), 20);
}

// --- Rasputin Dreamweaver -------------------------------------------------------------------

#[test]
fn rasputin_cant_have_more_than_seven_dream_counters() {
    cr!("704.5r", "122.1");
    ruling!(
        "Rasputin Dreamweaver",
        "If a number of dream counters would be put on Rasputin Dreamweaver for any reason that would bring the total number of dream counters on it to eight or more, the total number of dream counters on it is brought to seven instead."
    );
    ruling!(
        "Rasputin Dreamweaver",
        "If a permanent with eight or more dream counters on it becomes a copy of Rasputin Dreamweaver (or just gains its last ability), all dream counters except seven are removed from it as a state-based action."
    );
    supported("Rasputin Dreamweaver");
    supported("Unstable Shapeshifter");
    let mut t = TestGame::new(2);
    let rasputin = t.battlefield(P0, "Rasputin Dreamweaver");
    t.g.add_counters(obj(rasputin), "dream", 6, None);
    t.settle();
    t.g.add_counters(obj(rasputin), "dream", 3, None);
    t.settle();
    assert_eq!(t.counters(rasputin, "dream"), 7);
    // Unstable Shapeshifter with nine dream counters becomes a copy of P1's Rasputin.
    let mut t = TestGame::new(2);
    let shifter = t.battlefield(P0, "Unstable Shapeshifter");
    t.g.add_counters(obj(shifter), "dream", 9, None);
    t.enter(P1, "Rasputin Dreamweaver");
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.obj_now(shifter).name(), "Rasputin Dreamweaver");
    assert_eq!(t.counters(shifter, "dream"), 7);
}

// --- Kurbis, Harvest Celebrant ----------------------------------------------------------------

#[test]
fn kurbis_enters_with_counters_equal_to_the_mana_spent() {
    cr!("601.2h", "614.1c");
    ruling!(
        "Kurbis, Harvest Celebrant",
        "Kurbis enters with a number of +1/+1 counters on it equal to the total mana spent to cast it, and that number usually won't be the same as X. If you cast Kurbis for {G}{G}, it will enter the battlefield with two +1/+1 counters on it."
    );
    for (x, n) in [(0, 2), (2, 4)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 2 + x as usize);
        let card = t.hand(P0, "Kurbis, Harvest Celebrant");
        t.cast(P0, card).x(x).go();
        t.resolve_all();
        let k = t.named_on_battlefield("Kurbis, Harvest Celebrant");
        assert_eq!(t.counters(k[0], "+1/+1"), n, "X = {x}");
    }
}

#[test]
fn kurbis_copies_and_mana_spent() {
    cr!("707.10", "601.2h", "614.1c");
    ruling!(
        "Kurbis, Harvest Celebrant",
        "If a resolving spell enters the battlefield as a copy of Kurbis, it will enter the battlefield with a number of +1/+1 counters equal to the mana spent to cast the spell."
    );
    ruling!(
        "Kurbis, Harvest Celebrant",
        "If you copy Kurbis while it is on the stack (perhaps using an effect like that of Double Major), the copy wasn't cast, so no mana was spent to cast it. That Kurbis will enter with zero +1/+1 counters on it."
    );
    supported("Double Major");
    // Clone ({3}{U}) copying P1's Kurbis: four counters.
    let mut t = TestGame::new(2);
    let kurbis = t.battlefield(P1, "Kurbis, Harvest Celebrant");
    t.g.add_counters(obj(kurbis), "+1/+1", 2, None);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(kurbis)]);
    let clone = cast_resolve(&mut t, P0, "Clone", &[]);
    let clone = t.g.current(clone);
    assert_eq!(t.obj_now(clone).name(), "Kurbis, Harvest Celebrant");
    assert_eq!(t.counters(clone, "+1/+1"), 4);
    // Double Major copies a Kurbis spell cast for {G}{G}: the copy gets none and dies.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let card = t.hand(P0, "Kurbis, Harvest Celebrant");
    let spell = t.cast(P0, card).x(0).go();
    cast_new(&mut t, P0, "Double Major", &[obj(spell)]);
    t.resolve_all();
    let k = t.named_on_battlefield("Kurbis, Harvest Celebrant");
    assert_eq!(k.len(), 1, "the copy died as a 0/0");
    assert_eq!(t.counters(k[0], "+1/+1"), 2);
    assert!(t.g.turn_events.iter().any(|e| matches!(
        e,
        mtg_engine::events::Event::ZoneChange { from: Zone::Battlefield, .. }
    )));
}

// --- Commands --------------------------------------------------------------------------------

#[test]
fn a_command_with_some_or_all_targets_illegal() {
    cr!("608.2b", "700.2");
    ruling!(
        "Kolaghan's Command",
        "If all targets for the chosen modes become illegal before the Command resolves, the spell won't resolve and none of its effects will happen. If at least one target is still legal, the spell will resolve but will have no effect on any illegal targets."
    );
    supported("Kolaghan's Command");
    // Modes: target player discards a card; 2 damage to any target (P1's Bears).
    for all_illegal in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.hand(P1, "Hill Giant");
        lands_for_cost(&mut t, P0, "Kolaghan's Command");
        let card = t.hand(P0, "Kolaghan's Command");
        t.cast(P0, card)
            .modes(&[1, 3])
            .target(pl(P1))
            .target(obj(bears))
            .go();
        exile_now(&mut t, bears);
        if all_illegal {
            t.battlefield(P1, "Leyline of Sanctity");
            t.g.recompute();
        }
        t.resolve_all();
        assert_eq!(t.hand_size(P1), if all_illegal { 1 } else { 0 });
        assert_eq!(t.life(P1), 20);
    }
}

// --- State triggers ---------------------------------------------------------------------------

#[test]
fn a_countered_state_trigger_triggers_again() {
    cr!("603.8");
    ruling!(
        "Immortal Coil",
        "If the third ability is countered, but your graveyard is still empty, the ability will immediately trigger again."
    );
    ruling!(
        "Nine Lives",
        "If you counter the first triggered ability or otherwise remove it from the stack, it will just trigger again immediately if Nine Lives still has nine or more incarnation counters on it."
    );
    supported("Immortal Coil");
    supported("Stifle");
    for coil in [true, false] {
        let mut t = TestGame::new(2);
        if coil {
            t.battlefield(P0, "Immortal Coil");
        } else {
            let lives = t.battlefield(P0, "Nine Lives");
            t.g.add_counters(obj(lives), "incarnation", 9, None);
        }
        t.g.flush_events();
        t.settle();
        assert_eq!(t.stack_len(), 1, "Coil: {coil}");
        let trigger = t.g.stack[0];
        cast_new(&mut t, P1, "Stifle", &[obj(trigger)]);
        t.resolve();
        assert_eq!(t.stack_len(), 1, "it triggered again (Coil: {coil})");
        assert_ne!(t.g.stack[0], trigger);
    }
}

// --- Costs --------------------------------------------------------------------------------------

#[test]
fn wandering_mage_puts_the_counter_as_a_cost() {
    cr!("602.2b", "601.2h", "704.5f");
    ruling!(
        "Wandering Mage",
        "If you activate Wandering Mage’s third ability, you put the -1/-1 counter on a creature you control as a cost. That means it happens when you activate the ability, not when it resolves. If paying the cost causes the only creature you control to have 0 toughness, it’s put into the graveyard before you can pay the cost again."
    );
    supported("Wandering Mage");
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, "Wandering Mage");
    t.g.add_counters(obj(mage), "-1/-1", 2, None);
    t.lands(P0, "Swamp", 2);
    activate(&mut t, P0, mage, 2, &[pl(P0)]);
    assert_eq!(t.stack_len(), 1, "the ability is on the stack");
    t.settle();
    assert!(!t.on_battlefield(mage), "0 toughness");
    t.resolve_all();
    // The ability still resolved: the next 2 damage to P0 is prevented.
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, giant, pl(P0), 3);
    assert_eq!(t.life(P0), 19);
}

#[test]
fn thought_lash_cant_be_activated_with_an_empty_library() {
    cr!("602.2b", "118.3");
    ruling!(
        "Thought Lash",
        "If your library has no cards in it, you can’t activate the last ability."
    );
    supported("Thought Lash");
    let mut t = TestGame::new(2);
    let lash = t.battlefield(P0, "Thought Lash");
    assert!(t.activate(P0, lash, 0, &[]).is_ok());
    t.resolve_all();
    while let Some(c) = t.g.players[0].library.pop() {
        t.g.objects[c.0 as usize].zone = Zone::Nowhere;
    }
    assert!(t.activate(P0, lash, 0, &[]).is_err());
}

// --- Sevinne, Nefarious Lich, Khârn -------------------------------------------------------------

#[test]
fn sevinne_doesnt_trigger_after_an_earlier_graveyard_spell_that_turn() {
    cr!("603.2", "702.34a");
    ruling!(
        "Sevinne, the Chronoclasm",
        "If you cast an instant or sorcery spell from your graveyard before Sevinne enters the battlefield during the same turn, its last ability can't trigger that turn."
    );
    supported("Sevinne, the Chronoclasm");
    supported("Think Twice");
    for earlier in [false, true] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 6);
        if earlier {
            let tt = t.graveyard(P0, "Think Twice");
            t.cast(P0, tt)
                .method(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Flashback))
                .go();
            t.resolve_all();
        }
        t.battlefield(P0, "Sevinne, the Chronoclasm");
        let tt = t.graveyard(P0, "Think Twice");
        t.cast(P0, tt)
            .method(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Flashback))
            .go();
        t.settle();
        assert_eq!(t.stack_len(), if earlier { 1 } else { 2 }, "earlier: {earlier}");
    }
}

#[test]
fn nefarious_lich_and_a_draw_to_life_replacement() {
    cr!("614.5", "616.1f");
    ruling!(
        "Nefarious Lich",
        "you can gain life by using the Lich to turn the original life gain into card drawing, and then using the other replacement to turn a card draw into life gain. Nefarious Lich will not replace the life gain with a card draw again because it has already acted on this event."
    );
    supported("Nefarious Lich");
    supported("Words of Worship");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nefarious Lich");
    let words = t.battlefield(P0, "Words of Worship");
    t.lands(P0, "Plains", 2);
    activate_resolve(&mut t, P0, words, 0, &[]);
    let hand = t.hand_size(P0);
    t.g.gain_life(P0, 3);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.life(P0), 25, "one draw became 5 life");
    assert_eq!(t.hand_size(P0), hand + 2, "the other two draws happened");
}

#[test]
fn kharn_dealt_damage_several_times_by_one_spell() {
    cr!("101.4", "603.3b", "615.5");
    ruling!(
        "Khârn the Betrayer",
        "In some cases, a single spell or ability will deal damage to this creature multiple times during its resolution. In that case, a new player gains control of it each time, its ability triggers once each time, and the appropriate players control those triggers."
    );
    supported("Khârn the Betrayer");
    supported("Fiery Confluence");
    // Fiery Confluence deals 1 damage to each creature three times. Khârn goes from P0
    // to P1, from P1 to P2, and from P2 back to P0; each loses control and draws two.
    let mut t = TestGame::new(3);
    let kharn = t.battlefield(P0, "Khârn the Betrayer");
    t.answer_choose(P0, &[pl(P1)]);
    t.answer_choose(P1, &[pl(P2)]);
    t.answer_choose(P2, &[pl(P0)]);
    lands_for_cost(&mut t, P0, "Fiery Confluence");
    let card = t.hand(P0, "Fiery Confluence");
    let hands = [t.hand_size(P0), t.hand_size(P1), t.hand_size(P2)];
    t.cast(P0, card).modes(&[0, 0, 0]).go();
    t.resolve();
    assert_eq!(t.obj_now(kharn).controller, P0);
    assert_eq!(damage_marked(&t, kharn), 0);
    assert_eq!(t.stack_len(), 3, "three triggers");
    let controllers: Vec<PlayerId> = t.g.stack.iter().map(|s| t.g.obj(*s).controller).collect();
    assert_eq!(controllers, vec![P0, P1, P2], "APNAP: P0's first, P2's on top");
    t.resolve_all();
    assert_eq!(
        [t.hand_size(P0), t.hand_size(P1), t.hand_size(P2)],
        [hands[0] + 2 - 1, hands[1] + 2, hands[2] + 2]
    );
}
