//! Item `rulings-residue-misc-2`, pass A — triggers, timing and game flow (CR 117, 500–514,
//! 603, 608): the open rulings of batches P125, P126, P133, P136, P137, P141–P146 and P150
//! about when abilities trigger and resolve, what they look at, and what the game does
//! next.

use crate::r_residue_misc_2a_common::*;
use mtg_engine::decision::{Action, Answer, Decision, SpecialAction};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// Puts `name` (a real card, from nowhere) onto the battlefield under `p`'s control
/// together with the others, as one event.
fn enter_together(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    let moves = names
        .iter()
        .map(|n| {
            let id =
                t.g.create_card_object(mtg_engine::card::card(n), p, Zone::Nowhere);
            mtg_engine::replacement::MoveEv {
                obj: id,
                to: Zone::Battlefield,
                pos: mtg_engine::ability::LibraryPosition::Top,
                cause: mtg_engine::events::MoveCause::Effect,
                by: Some(p),
                etb: mtg_engine::replacement::EtbInfo {
                    controller: Some(p),
                    ..Default::default()
                },
                source: None,
            }
        })
        .collect();
    let ids = t.g.move_objects(moves).into_iter().flatten().collect();
    t.g.flush_events();
    ids
}

// --- P125 ----------------------------------------------------------------------------------------

#[test]
fn everybody_lives_players_can_still_concede() {
    cr!("104.3a", "104.3b");
    ruling!(
        "Everybody Lives!",
        "Players can always concede from the game, even if they can't otherwise lose the game."
    );
    supported("Everybody Lives!");
    let mut t = TestGame::new(3);
    cast_new(&mut t, P0, "Everybody Lives!", &[]);
    t.resolve_all();
    t.g.players[1].life = 0;
    t.settle();
    assert!(!t.has_lost(P1));
    crate::r_p053_common::concede(&mut t, P1);
    assert!(t.has_lost(P1));
}

#[test]
fn boromir_counters_a_free_spell_that_still_counts_as_cast() {
    cr!("601.2i", "603.2", "701.6a");
    ruling!(
        "Boromir, Warden of the Tower",
        "Players may cast spells that they know Boromir, Warden of the Tower will counter. Any abilities that trigger when spells are cast will still trigger and resolve if appropriate, and any effects that count how many spells are cast will still count those spells if appropriate."
    );
    supported("Boromir, Warden of the Tower");
    supported("Beast Whisperer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Boromir, Warden of the Tower");
    t.battlefield(P1, "Beast Whisperer");
    t.set_step(P1, Step::PrecombatMain);
    let thopter = t.hand(P1, "Ornithopter");
    let hand = t.hand_size(P1);
    let cast_before = t.g.history.spells_cast.len();
    t.cast(P1, thopter).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Ornithopter"), "countered");
    assert_eq!(
        t.hand_size(P1),
        hand,
        "the cast spell left and Beast Whisperer drew"
    );
    assert_eq!(t.g.history.spells_cast.len(), cast_before + 1);
}

#[test]
fn legion_s_initiative_returns_at_the_next_combat_without_attacks() {
    cr!("506.1", "603.7");
    ruling!(
        "Legion's Initiative",
        "The beginning of the next combat will happen even if the player whose turn it is doesn't intend to attack with any creatures."
    );
    supported("Legion's Initiative");
    let mut t = TestGame::new(2);
    let li = t.battlefield(P0, "Legion's Initiative");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Plains", 1);
    t.activate(P0, li, 0, &[]).expect("activated");
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert!(t
        .obj_now(bears[0])
        .has_keyword(mtg_engine::keywords::KeywordKind::Haste));
}

#[test]
fn lae_zel_s_acrobatics_exiles_before_the_roll() {
    cr!("608.2c", "706.2");
    ruling!(
        "Lae'zel's Acrobatics",
        "The creatures you control are exiled before you roll a die. Notably, this means that if any of those creatures have abilities that replace a die roll or trigger due to a die roll, those won't occur."
    );
    supported("Lae'zel's Acrobatics");
    supported("Brazen Dwarf");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Brazen Dwarf");
    t.g.dice.loaded.push_back(5);
    cast_new(&mut t, P0, "Lae'zel's Acrobatics", &[]);
    t.resolve_all();
    assert!(t.in_exile("Brazen Dwarf"));
    assert_eq!(t.life(P1), 20, "{}", t.dump_log());
    // Baseline: a roll while the Dwarf is on the battlefield deals 1.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Brazen Dwarf");
    let rift = t.battlefield(P0, "Underdark Rift");
    let bears = t.battlefield(P1, "Grizzly Bears");
    add(&mut t, P0, ManaType::C, 5);
    t.g.dice.loaded.push_back(1);
    t.activate(P0, rift, 1, &[obj(bears)])
        .expect("Underdark Rift");
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn call_the_spirit_dragons_needs_five_different_dragons() {
    cr!("104.2b", "122.1");
    ruling!(
        "Call the Spirit Dragons",
        "The last ability will only cause you to win the game if you put a +1/+1 counter on each of five different Dragons with it."
    );
    supported("Call the Spirit Dragons");
    supported("Niv-Mizzet Reborn");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Call the Spirit Dragons");
    let niv = t.battlefield(P0, "Niv-Mizzet Reborn");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(niv, "+1/+1"), 5);
    assert!(!t.has_lost(P1));
}

#[test]
fn march_of_swirling_mist_mana_value_and_minimum_cost() {
    cr!("202.3e", "601.2f", "118.7");
    ruling!(
        "March of Swirling Mist",
        "The mana value of March of Swirling Mist while it's on the stack is the value chosen for X plus 1, no matter how much mana you actually paid to cast it."
    );
    ruling!(
        "March of Swirling Mist",
        "You may exile more cards than necessary for March of Swirling Mist's first ability, but you can't reduce the mana it costs to less than {U} this way."
    );
    supported("March of Swirling Mist");
    // X = 2, two blue cards exiled ({4} less): it costs {U}, and its mana value is 3.
    let mut t = TestGame::new(2);
    let a = t.hand(P0, "Island");
    let b = t.hand(P0, "Counterspell");
    let c = t.hand(P0, "Unsummon");
    let _ = a;
    t.lands(P0, "Island", 1);
    let march = t.hand(P0, "March of Swirling Mist");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(2));
    t.answer_choose(P0, &[obj(b), obj(c)]);
    t.answer_targets(P0, &[]);
    let from = t.asked().len();
    let spell = t.cast(P0, march).x(2).try_go();
    let spell = spell.unwrap_or_else(|e| panic!("{e:?} {:?}", asked_since(&t, from)));
    assert_eq!(crate::r_s26_common::mv(&mut t, spell), 3);
    assert!(t.in_exile("Counterspell") && t.in_exile("Unsummon"));
    // With no land, not even {U} can be paid.
    let mut t = TestGame::new(2);
    let b = t.hand(P0, "Counterspell");
    let c = t.hand(P0, "Unsummon");
    let march = t.hand(P0, "March of Swirling Mist");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(2));
    t.answer_choose(P0, &[obj(b), obj(c)]);
    t.answer_targets(P0, &[]);
    assert!(t.cast(P0, march).x(1).try_go().is_err());
}

#[test]
fn linvala_s_lock_lasts_after_it_leaves() {
    cr!("611.2a", "611.2c");
    ruling!(
        "Linvala, Shield of Sea Gate",
        "The nonland permanent remains affected until your next turn even if the number of creatures in your party changes or Linvala leaves the battlefield."
    );
    supported("Linvala, Shield of Sea Gate");
    let mut t = TestGame::new(2);
    let linvala = t.battlefield(P0, "Linvala, Shield of Sea Gate");
    let party: Vec<ObjectId> = (0..3)
        .map(|_| t.battlefield(P0, "Changeling Outcast"))
        .collect();
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    destroy(&mut t, linvala);
    for c in party {
        destroy(&mut t, c);
    }
    t.advance_to(P1, Step::BeginningOfCombat);
    assert!(!crate::r_s09_common::legal_attack(
        &mut t,
        &[(giant, pl(P0))]
    ));
}

#[test]
fn veil_of_summer_can_be_countered() {
    cr!("608.1", "701.6a");
    ruling!(
        "Veil of Summer",
        "Veil of Summer has no effect until it resolves. It can be countered."
    );
    supported("Veil of Summer");
    let mut t = TestGame::new(2);
    let veil = cast_new(&mut t, P0, "Veil of Summer", &[]);
    cast_new(&mut t, P1, "Counterspell", &[obj(veil)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Veil of Summer"));
    assert!(t.in_graveyard(P1, "Counterspell"));
}

#[test]
fn energy_arc_untapped_targets_and_no_targets() {
    cr!("601.2c", "615.1");
    ruling!(
        "Energy Arc",
        "Energy Arc can target untapped creatures. The second part of the effect will still affect them."
    );
    ruling!("Energy Arc", "The number of target creatures may be zero.");
    supported("Energy Arc");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    lands_for_cost(&mut t, P0, "Energy Arc");
    let arc = t.hand(P0, "Energy Arc");
    t.cast(P0, arc).go();
    t.resolve_all();
    attack_with(&mut t, &[(bears, pl(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 20);
    // No targets.
    let mut t = TestGame::new(2);
    t.answer_targets(P0, &[]);
    lands_for_cost(&mut t, P0, "Energy Arc");
    let arc = t.hand(P0, "Energy Arc");
    t.cast(P0, arc).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Energy Arc"));
}

#[test]
fn mandate_of_peace_with_the_postcombat_main_phase_skipped_goes_to_the_end_step() {
    cr!("500.1", "505.1a", "614.10");
    ruling!(
        "Mandate of Peace",
        "In the rare case that the postcombat main phase is skipped, the game skips to the end step of the ending phase."
    );
    supported("Fatespinner");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fatespinner");
    // P1 chooses "main phase" for Fatespinner: it skips both main phases this turn.
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.g.turn.number = 2;
    t.set_step(P1, Step::Untap);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    t.advance_to(P1, Step::BeginningOfCombat);
    assert_eq!(
        t.g.turn.main_phases, 0,
        "the precombat main phase was skipped"
    );
    cast_new(&mut t, P1, "Mandate of Peace", &[]);
    t.resolve_all();
    let ok = t.g.run_until(1000, |g| {
        !g.turn.step.is_combat() && g.turn.stage == mtg_engine::turn::Stage::Priority
    });
    assert!(ok);
    assert_eq!(t.g.turn.step, Step::End, "{}", t.dump_log());
}

#[test]
fn skyship_plunderer_gives_one_more_of_each_kind_to_any_target() {
    cr!("122.1", "115.1");
    ruling!(
        "Skyship Plunderer",
        "Skyship Plunderer's triggered ability can target any permanent or player, regardless of which player was dealt combat damage."
    );
    ruling!(
        "Skyship Plunderer",
        "Skyship Plunderer's triggered ability gives only one counter of each kind. It doesn't double the number of each kind of counter."
    );
    supported("Skyship Plunderer");
    let mut t = TestGame::new(2);
    let plunderer = t.battlefield(P0, "Skyship Plunderer");
    let mine = t.battlefield(P0, "Grizzly Bears");
    plus1(&mut t, mine, 2);
    t.g.add_counters(obj(mine), "charge", 1, None);
    attack_with(&mut t, &[(plunderer, pl(P1))]);
    t.answer_targets(P0, &[obj(mine)]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.counters(mine, "+1/+1"), 3);
    assert_eq!(t.counters(mine, "charge"), 2);
}

#[test]
fn maulfist_revolutionary_triggers_entering_and_dying() {
    cr!("603.6a", "603.10a");
    ruling!(
        "Maulfist Revolutionary",
        "The triggered ability triggers both when Maulfist Revolutionary enters the battlefield and when it dies. You don’t have to choose only one."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    plus1(&mut t, bears, 1);
    t.answer_targets(P0, &[obj(bears)]);
    let m = t.enter(P0, "Maulfist Revolutionary");
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 2);
    t.answer_targets(P0, &[obj(bears)]);
    destroy(&mut t, m);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 3);
}

// --- P126 ----------------------------------------------------------------------------------------

#[test]
fn storm_world_does_nothing_to_a_player_with_four_or_more_cards() {
    cr!("608.2c", "107.1b");
    ruling!(
        "Storm World",
        "If the player whose turn it is has four or more cards in their hand as Storm World's ability resolves, the ability just won't do anything that turn."
    );
    supported("Storm World");
    for (cards, life) in [(5, 20), (2, 18)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Storm World");
        for _ in 0..cards {
            t.hand(P1, "Island");
        }
        t.set_step(P1, Step::Untap);
        t.advance_to(P1, Step::Upkeep);
        t.resolve_all();
        assert_eq!(t.life(P1), life, "{cards} cards");
    }
}

/// P0 has cast Yawgmoth's Will (it resolved) this turn.
fn yawgmoth(t: &mut TestGame) {
    supported("Yawgmoth's Will");
    cast_new(t, P0, "Yawgmoth's Will", &[]);
    t.resolve_all();
}

#[test]
fn yawgmoth_s_will_and_buyback_the_controller_chooses() {
    cr!("616.1", "702.27a");
    ruling!(
        "Yawgmoth's Will",
        "If you cast a Buyback spell, then there will be two effects trying to replace where the card goes. You get to choose if the Buyback returns the card to your hand or the card gets exiled."
    );
    supported("Whispers of the Muse");
    for to_hand in [true, false] {
        let mut t = TestGame::new(2);
        yawgmoth(&mut t);
        let from = t.asked().len();
        t.lands(P0, "Island", 6);
        let w = t.hand(P0, "Whispers of the Muse");
        t.answer(
            P0,
            DecisionKind::Replacement,
            Answer::Index(if to_hand { 0 } else { 1 }),
        );
        t.cast(P0, w).kicked(true).go();
        t.resolve_all();
        let asked = asked_since(&t, from)
            .into_iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseReplacement { .. }))
            .count();
        assert_eq!(asked, 1, "{}", t.dump_log());
        assert!(t.in_hand(P0, "Whispers of the Muse") || t.in_exile("Whispers of the Muse"));
        assert_ne!(
            t.in_hand(P0, "Whispers of the Muse"),
            t.in_exile("Whispers of the Muse")
        );
    }
}

#[test]
fn yawgmoth_s_will_cast_from_the_graveyard_isnt_cast_from_your_hand() {
    cr!("603.4", "601.2");
    ruling!(
        "Yawgmoth's Will",
        "If you play a card using Yawgmoth’s Will and something triggers only when “cast from your hand”, that something will not trigger."
    );
    supported("Coal Stoker");
    let mut t = TestGame::new(2);
    yawgmoth(&mut t);
    let stoker = t.graveyard(P0, "Coal Stoker");
    t.lands(P0, "Mountain", 4);
    t.cast(P0, stoker).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Coal Stoker").len(), 1);
    assert!(t.g.player(P0).mana_pool.is_empty(), "no {{R}}{{R}}{{R}}");
    // From the hand it does.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let stoker = t.hand(P0, "Coal Stoker");
    t.cast(P0, stoker).go();
    t.resolve_all();
    assert_eq!(t.g.player(P0).mana_pool.total(), 3);
}

#[test]
fn yawgmoth_s_will_doesnt_let_you_suspend_from_the_graveyard() {
    cr!("702.62a", "116.2f");
    ruling!(
        "Yawgmoth's Will",
        "You cannot Suspend a card from your graveyard, only your hand."
    );
    let mut t = TestGame::new(2);
    yawgmoth(&mut t);
    t.lands(P0, "Mountain", 1);
    let gy = t.graveyard(P0, "Rift Bolt");
    let hand = t.hand(P0, "Rift Bolt");
    t.g.turn.priority = Some(P0);
    let actions = t.g.legal_actions(P0);
    assert!(!actions.contains(&Action::Special(SpecialAction::Suspend { card: gy })));
    assert!(actions.contains(&Action::Special(SpecialAction::Suspend { card: hand })));
}

#[test]
fn conduit_of_worlds_counts_every_spell_cast_this_turn() {
    cr!("608.2c", "601.2i");
    ruling!(
        "Conduit of Worlds",
        "The second ability of Conduit of Worlds will count spells you cast earlier in the turn even if Conduit of Worlds wasn't on the battlefield or under your control at that time. It will also count a spell you cast that subsequently was countered or failed to resolve. Notably, it will also include Conduit of Worlds itself on the turn you cast it."
    );
    supported("Conduit of Worlds");
    // The turn it's cast: Conduit itself counts.
    for case in ["itself", "countered", "none"] {
        let mut t = TestGame::new(2);
        let conduit = if case == "itself" {
            let c = cast_new(&mut t, P0, "Conduit of Worlds", &[]);
            t.resolve_all();
            t.g.current(c)
        } else {
            t.battlefield(P0, "Conduit of Worlds")
        };
        if case == "countered" {
            let shock = cast_new(&mut t, P0, "Shock", &[pl(P1)]);
            cast_new(&mut t, P1, "Counterspell", &[obj(shock)]);
            t.resolve_all();
            assert_eq!(t.life(P1), 20);
        }
        let bears = t.graveyard(P0, "Grizzly Bears");
        t.lands(P0, "Forest", 2);
        t.answer_yes(P0, true);
        t.activate(P0, conduit, 0, &[obj(bears)]).expect("Conduit");
        t.resolve_all();
        assert_eq!(
            t.named_on_battlefield("Grizzly Bears").len(),
            usize::from(case == "none"),
            "{case}"
        );
    }
}

// --- P133 ----------------------------------------------------------------------------------------

#[test]
fn confounding_conundrum_two_lands_together_trigger_for_each_and_any_land_returns() {
    cr!("603.2", "603.10a");
    ruling!(
        "Confounding Conundrum",
        "If an opponent has had no lands enter under their control and then two or more lands enter at the same time, Confounding Conundrum's second ability triggers for each of those lands."
    );
    ruling!(
        "Confounding Conundrum",
        "The player may return any land they control to its owner's hand, not just one of those that entered this turn."
    );
    supported("Confounding Conundrum");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Confounding Conundrum");
    let old = t.battlefield(P1, "Wastes");
    let old2 = t.battlefield(P1, "Wastes");
    enter_together(&mut t, P1, &["Forest", "Island"]);
    t.settle();
    assert_eq!(
        crate::r_s01_common::triggers_on_stack(&t, "return a land"),
        2
    );
    t.answer_choose(P1, &[obj(old)]);
    t.answer_choose(P1, &[obj(old2)]);
    t.resolve_all();
    assert!(!t.on_battlefield(old) && !t.on_battlefield(old2));
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    assert_eq!(t.named_on_battlefield("Island").len(), 1);
}

#[test]
fn noetic_scales_counts_the_hand_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Noetic Scales",
        "The number of cards in hand is counted on resolution of the ability."
    );
    supported("Noetic Scales");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Noetic Scales");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.hand(P1, "Island");
    t.set_step(P1, Step::Untap);
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // In response, P1's hand grows to three.
    t.g.draw_cards(P1, 2);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

// --- P136 ----------------------------------------------------------------------------------------

#[test]
fn dimensional_breach_s_returned_creature_cant_attack_that_turn() {
    cr!("302.6", "603.7c");
    ruling!(
        "Dimensional Breach",
        "Remember that the returned creature just came under your control, so it can’t attack until you start a turn with it under your control."
    );
    supported("Dimensional Breach");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, P0, ManaType::W, 7);
    let b = t.hand(P0, "Dimensional Breach");
    t.cast(P0, b).go();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1, "{}", t.dump_log());
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, back[0]));
    let _ = bears;
}

#[test]
fn underdark_rift_with_a_short_library_puts_it_on_the_bottom() {
    cr!("401.7", "706.2");
    ruling!(
        "Underdark Rift",
        "If that permanent's owner has fewer than X cards in their library, put the permanent on the bottom of its owner's library."
    );
    supported("Underdark Rift");
    let mut t = TestGame::new(2);
    let rift = t.battlefield(P0, "Underdark Rift");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.players[1].library.truncate(2);
    add(&mut t, P0, ManaType::C, 5);
    t.g.dice.loaded.push_back(7);
    t.activate(P0, rift, 1, &[obj(bears)])
        .expect("Underdark Rift");
    t.resolve_all();
    let lib = &t.g.player(P1).library;
    assert_eq!(lib.len(), 3);
    assert_eq!(t.g.obj(lib[0]).chars.name, "Grizzly Bears", "on the bottom");
}

// --- P137 ----------------------------------------------------------------------------------------

#[test]
fn nature_s_wrath_one_permanent_can_trigger_both_abilities() {
    cr!("603.2", "305.6");
    ruling!(
        "Nature's Wrath",
        "A single permanent may cause both the second and third abilities to trigger. For example, Underground Sea is both an Island and a Swamp, and Skeleton Ship is both blue and black."
    );
    supported("Nature's Wrath");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nature's Wrath");
    t.set_step(P1, Step::PrecombatMain);
    let sea = t.hand(P1, "Underground Sea");
    t.play_land(P1, sea).expect("land");
    t.settle();
    assert_eq!(crate::r_s01_common::triggers_on_stack(&t, "sacrifices"), 2);
}

#[test]
fn bringer_of_the_last_gift_triggers_only_if_cast() {
    cr!("603.4", "601.2");
    ruling!(
        "Bringer of the Last Gift",
        "Bringer of the Last Gift's last ability triggers if you cast it from any zone. It doesn't trigger if you put Bringer of the Last Gift onto the battlefield without casting it."
    );
    // Put onto the battlefield: nothing.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.enter(P0, "Bringer of the Last Gift");
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // Cast from the graveyard (Yawgmoth's Will): it triggers.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    yawgmoth(&mut t);
    let bringer = t.graveyard(P0, "Bringer of the Last Gift");
    add(&mut t, P0, ManaType::B, 8);
    t.cast(P0, bringer).go();
    t.resolve_all();
    // (The Bears was sacrificed this way, so it isn't returned.)
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 0);
}

#[test]
fn desolation_each_one_sacrifices_a_land() {
    cr!("603.2", "701.21a");
    ruling!(
        "Desolation",
        "Each Desolation only causes up to one land per turn to be sacrificed. If you have more than one, each will do this."
    );
    supported("Desolation");
    for n in [1usize, 2] {
        let mut t = TestGame::new(2);
        for _ in 0..n {
            t.battlefield(P0, "Desolation");
        }
        t.set_step(P1, Step::PrecombatMain);
        let lands = t.lands(P1, "Wastes", 3);
        t.activate(P1, lands[0], 0, &[]).unwrap();
        t.advance_to(P1, Step::End);
        t.resolve_all();
        let left = t.g.permanents().filter(|o| o.controller == P1).count();
        assert_eq!(left, 3 - n, "{n} Desolations");
    }
}

#[test]
fn land_equilibrium_and_a_land_s_own_replacement_its_controller_chooses() {
    cr!("616.1", "616.2");
    ruling!(
        "Land Equilibrium",
        "If a land that would enter has its own \"If [this land] would enter\" replacement effect, the player who will control that land chooses which replacement effect to apply. After applying one, if the other one would still be applicable, it's then applied."
    );
    supported("Land Equilibrium");
    supported("Lake of the Dead");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Land Equilibrium");
    t.lands(P0, "Wastes", 1);
    t.set_step(P1, Step::PrecombatMain);
    let swamp = t.battlefield(P1, "Swamp");
    t.battlefield(P1, "Wastes");
    let lake = t.hand(P1, "Lake of the Dead");
    let from = t.asked().len();
    t.answer_choose(P1, &[obj(swamp)]);
    t.play_land(P1, lake).expect("land");
    t.resolve_all();
    let asked: Vec<PlayerId> = asked_since(&t, from)
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseReplacement { .. }))
        .map(|(p, _)| p)
        .collect();
    assert_eq!(asked, vec![P1], "{}", t.dump_log());
    // Both applied: the Swamp went for the Lake, and one more land for Land Equilibrium.
    assert!(!t.on_battlefield(swamp));
    assert_eq!(t.g.permanents().filter(|o| o.controller == P1).count(), 1);
}

// --- P141, P142 ------------------------------------------------------------------------------------

#[test]
fn tetravus_its_controller_orders_its_two_upkeep_triggers() {
    cr!("603.3b");
    ruling!(
        "Tetravus",
        "This card now has two upkeep-triggered abilities. Its controller chooses the order they are put on the stack, and thus the order in which they resolve each upkeep."
    );
    supported("Tetravus");
    let mut t = TestGame::new(2);
    t.enter(P0, "Tetravus");
    t.set_step(P1, Step::End);
    let from = t.asked().len();
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert!(asked_since(&t, from)
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. })));
}

#[test]
fn petra_sphinx_with_an_empty_library_does_nothing() {
    cr!("121.4", "704.5b");
    ruling!(
        "Petra Sphinx",
        "If the targeted player’s library is empty, that player still names a card, but nothing further happens. That player will not lose the game."
    );
    supported("Petra Sphinx");
    let mut t = TestGame::new(2);
    let sphinx = t.battlefield(P0, "Petra Sphinx");
    t.g.players[1].library.clear();
    let from = t.asked().len();
    t.activate(P0, sphinx, 0, &[pl(P1)]).expect("Petra Sphinx");
    t.resolve_all();
    assert!(asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::NameCard { .. })));
    assert!(!t.has_lost(P1));
}

#[test]
fn comet_rolling_a_3_with_nothing_to_return_just_removes_loyalty() {
    cr!("606.4", "706.2");
    ruling!(
        "Comet, Stellar Pup",
        "If you roll a 3 but there aren't any cards with mana value 2 or less in your graveyard, you'll just remove one loyalty counter from Comet."
    );
    supported("Comet, Stellar Pup");
    let mut t = TestGame::new(2);
    let comet = crate::r_p007_common::pw(&mut t, P0, "Comet, Stellar Pup", 5);
    t.graveyard(P0, "Craw Wurm");
    t.g.dice.loaded.push_back(3);
    crate::r_p007_common::loyalty(&mut t, P0, comet, 0, &[]);
    t.resolve_all();
    assert_eq!(t.counters(comet, "loyalty"), 4);
    assert!(t.in_graveyard(P0, "Craw Wurm"));
}

// --- P143 ------------------------------------------------------------------------------------------

#[test]
fn kagha_s_permission_still_needs_its_costs_paid() {
    cr!("601.2f", "601.2h");
    ruling!(
        "Kagha, Shadow Archdruid",
        "You must pay all costs when casting a spell this way."
    );
    supported("Kagha, Shadow Archdruid");
    let mut t = TestGame::new(2);
    let kagha = t.battlefield(P0, "Kagha, Shadow Archdruid");
    stack_library(&mut t, P0, &["Grizzly Bears", "Island"]);
    attack_with(&mut t, &[(kagha, pl(P1))]);
    t.resolve_all();
    let bears = t.g.find_in_zone(Zone::Graveyard(P0), "Grizzly Bears");
    assert_eq!(bears.len(), 1);
    t.advance_to(P0, Step::PostcombatMain);
    assert!(t.cast(P0, bears[0]).try_go().is_err(), "no mana");
    t.lands(P0, "Forest", 2);
    t.cast(P0, bears[0]).go();
}

#[test]
fn depala_triggers_on_becoming_tapped_only() {
    cr!("603.2", "701.26a");
    ruling!(
        "Depala, Pilot Exemplar",
        "Depala, Pilot Exemplar's last ability is a triggered ability, not an activated ability. It doesn't allow you to tap Depala whenever you want"
    );
    ruling!(
        "Depala, Pilot Exemplar",
        "For the ability to trigger, Depala has to actually change from untapped to tapped. If an effect attempts to tap Depala, but it was already tapped at the time, this ability won't trigger."
    );
    supported("Depala, Pilot Exemplar");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Depala, Pilot Exemplar");
    assert!(!can_activate(&mut t, P0, d));
    t.g.tap(d);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    // Tapping it again while tapped: no trigger.
    assert!(!t.g.tap(d));
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn uba_mask_cards_not_played_stay_exiled() {
    cr!("614.1a", "611.2a");
    ruling!(
        "Uba Mask",
        "Any cards you don’t play just remain exiled when the turn ends."
    );
    supported("Uba Mask");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Uba Mask");
    let top = t.library_top(P0, "Grizzly Bears");
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    assert!(t.in_exile("Grizzly Bears"));
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(t.in_exile("Grizzly Bears"));
    t.lands(P0, "Forest", 2);
    let exiled = t.g.current(top);
    assert!(t.cast(P0, exiled).try_go().is_err(), "no longer this turn");
}

// --- P145 -------------------------------------------------------------------------------------------

#[test]
fn oblivious_bookworm_checks_face_down_permanents_as_it_resolves() {
    cr!("608.2c", "603.4");
    ruling!(
        "Oblivious Bookworm",
        "Once Oblivious Bookworm's ability begins to resolve, it's too late to put any permanents onto the battlefield face down under your control or turn any permanents face up to prevent yourself from having to discard a card if you choose to draw a card."
    );
    supported("Oblivious Bookworm");
    for face_down in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Oblivious Bookworm");
        if face_down {
            t.lands(P0, "Wastes", 3);
            let tracer = t.hand(P0, "Echo Tracer");
            t.cast(P0, tracer)
                .method(mtg_engine::object::CastMethod::FaceDown(
                    mtg_engine::keywords::KeywordKind::Morph,
                ))
                .go();
            t.resolve_all();
        }
        t.hand(P0, "Island");
        let hand = t.hand_size(P0);
        t.answer_yes(P0, true);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), if face_down { hand + 1 } else { hand });
    }
}

#[test]
fn bazaar_of_baghdad_discards_without_a_window() {
    cr!("608.2", "117.3");
    ruling!(
        "Bazaar of Baghdad",
        "You can’t cast spells or activate mana abilities before discarding the extra cards."
    );
    supported("Bazaar of Baghdad");
    let mut t = TestGame::new(2);
    let bazaar = t.battlefield(P0, "Bazaar of Baghdad");
    t.hand(P0, "Island");
    t.activate(P0, bazaar, 0, &[]).expect("Bazaar");
    let from = t.asked().len();
    t.g.resolve_top();
    t.g.flush_events();
    assert!(asked_since(&t, from)
        .iter()
        .all(|(_, d)| !matches!(d, Decision::Priority { .. })));
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.graveyard_size(P0), 3);
}

// --- P146 -------------------------------------------------------------------------------------------

#[test]
fn phenomenon_investigators_without_a_choice_has_neither_ability() {
    cr!("700.2a", "607.2a");
    ruling!(
        "Phenomenon Investigators",
        "If you somehow control Phenomenon Investigators and no choice was made for it"
    );
    supported("Phenomenon Investigators");
    // Entered for real, choosing Believe: a Horror when a nontoken creature dies.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.enter(P0, "Phenomenon Investigators");
    t.settle();
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    // Put onto the battlefield with no choice: no Horror, no end step trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Phenomenon Investigators");
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn curse_of_chaos_triggers_once_per_attacking_player() {
    cr!("603.2c", "508.1");
    ruling!(
        "Curse of Chaos",
        "Curse of Chaos’s ability will trigger only once per attacking player per combat phase, no matter how many creatures that player attacks with."
    );
    supported("Curse of Chaos");
    let mut t = TestGame::new(2);
    crate::r_s06_common::attach_new(&mut t, P1, "Curse of Chaos", pl(P0));
    t.set_step(P1, Step::BeginningOfCombat);
    let atk: Vec<_> = (0..3)
        .map(|_| (t.battlefield(P1, "Grizzly Bears"), pl(P0)))
        .collect();
    attack_with(&mut t, &atk);
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn conspiracy_theorist_pays_once() {
    cr!("608.2c", "118.12");
    ruling!(
        "Conspiracy Theorist",
        "While resolving Conspiracy Theorist's ability, you can't pay {1} and discard a card multiple times to draw more cards."
    );
    supported("Conspiracy Theorist");
    let mut t = TestGame::new(2);
    let ct = t.battlefield(P0, "Conspiracy Theorist");
    t.lands(P0, "Mountain", 3);
    for _ in 0..3 {
        t.hand(P0, "Island");
    }
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(ct, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.graveyard_size(P0), 1);
}

#[test]
fn yenna_s_token_copies_copiable_values_only() {
    cr!("707.2", "707.3");
    ruling!(
        "Yenna, Redtooth Regent",
        "Except for the listed exceptions, the token copies exactly what was printed on the original enchantment and nothing else"
    );
    ruling!(
        "Yenna, Redtooth Regent",
        "If the copied enchantment is copying something else, then the token enters the battlefield as whatever that creature copied, with the stated exceptions."
    );
    supported("Yenna, Redtooth Regent");
    supported("Copy Enchantment");
    // A tapped Boon Satyr with a +1/+1 counter: the token is an untapped 4/2.
    let mut t = TestGame::new(2);
    let yenna = t.battlefield(P0, "Yenna, Redtooth Regent");
    let satyr = t.battlefield(P0, "Boon Satyr");
    plus1(&mut t, satyr, 1);
    t.g.tap(satyr);
    t.lands(P0, "Wastes", 2);
    t.activate(P0, yenna, 0, &[obj(satyr)]).expect("Yenna");
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.pt(toks[0]), (4, 2));
    assert!(!t.obj_now(toks[0]).tapped);
    // Copy Enchantment copying P1's Glorious Anthem: the token is a Glorious Anthem.
    let mut t = TestGame::new(2);
    let yenna = t.battlefield(P0, "Yenna, Redtooth Regent");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    t.answer_yes(P0, true);
    let copy = crate::r_p023_common::enter_copying(&mut t, P0, "Copy Enchantment", anthem);
    t.lands(P0, "Wastes", 2);
    t.activate(P0, yenna, 0, &[obj(copy)]).expect("Yenna");
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Glorious Anthem");
}

#[test]
fn doors_of_durin_sees_the_creature_it_put_onto_the_battlefield() {
    cr!("608.2c", "608.2h");
    ruling!(
        "Doors of Durin",
        "If the creature put onto the battlefield this way is a Dwarf and/or Elf, Doors of Durin will see it when checking whether you control a Dwarf or Elf."
    );
    supported("Doors of Durin");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doors of Durin");
    let bears = t.battlefield(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Llanowar Elves", "Island", "Island"]);
    t.answer(P0, DecisionKind::Scry, Answer::Default);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(bears, pl(P1))]);
    t.resolve_all();
    let elves = t.named_on_battlefield("Llanowar Elves");
    assert_eq!(elves.len(), 1, "{}", t.dump_log());
    assert!(t
        .obj_now(elves[0])
        .has_keyword(mtg_engine::keywords::KeywordKind::Hexproof));
    assert!(!t
        .obj_now(elves[0])
        .has_keyword(mtg_engine::keywords::KeywordKind::Trample));
}

#[test]
fn isolated_watchtower_needs_only_one_opponent_with_two_more_lands() {
    cr!("602.5b");
    ruling!(
        "Isolated Watchtower",
        "You can activate Isolated Watchtower’s ability if you control more lands than an opponent, as long as at least one opponent also controls two or more lands than you."
    );
    supported("Isolated Watchtower");
    for (p2_lands, ok) in [(5, true), (4, false)] {
        let mut t = TestGame::new(3);
        let tower = t.battlefield(P0, "Isolated Watchtower");
        t.lands(P0, "Wastes", 2);
        t.lands(P1, "Wastes", 1);
        t.lands(P2, "Wastes", p2_lands);
        assert_eq!(can_activate(&mut t, P0, tower), ok, "{p2_lands}");
    }
}

#[test]
fn victor_does_nothing_past_the_third_resolution() {
    cr!("608.2c");
    ruling!(
        "Victor, Valgavoth's Seneschal",
        "Victor's ability has no effect beyond the third time it resolves in a turn."
    );
    supported("Victor, Valgavoth's Seneschal");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Victor, Valgavoth's Seneschal");
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    for _ in 0..4 {
        t.hand(P1, "Island");
    }
    for _ in 0..4 {
        t.enter(P0, "Glorious Anthem");
        t.resolve_all();
    }
    // One discard (the second), one creature (the third); the fourth did nothing.
    assert_eq!(t.hand_size(P1), 3);
    let back =
        t.g.permanents()
            .filter(|o| {
                o.controller == P0
                    && o.is(CardType::Creature)
                    && o.chars.name != "Victor, Valgavoth's Seneschal"
            })
            .count();
    assert_eq!(back, 1, "{}", t.dump_log());
}

// --- P150 -------------------------------------------------------------------------------------------

#[test]
fn vault_21_counts_x_as_0_in_hand() {
    cr!("202.3e", "714.2b");
    ruling!(
        "Vault 21: House Gambit",
        "If a card in your hand has {X} in its mana cost, X is 0."
    );
    supported("Vault 21: House Gambit");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Vault 21: House Gambit");
    t.g.add_counters(obj(saga), "lore", 2, None);
    let fireball = t.hand(P0, "Fireball");
    let shock = t.hand(P0, "Shock");
    t.answer_choose(P0, &[obj(fireball), obj(shock)]);
    t.g.add_counters(obj(saga), "lore", 1, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2, "{}", t.dump_log());
}

// --- Central Elevator // Promising Stairs ------------------------------------------------------------

/// A Room put onto `p`'s battlefield with the given doors unlocked.
fn room(t: &mut TestGame, p: PlayerId, name: &str, doors: &[usize]) -> ObjectId {
    supported(name);
    let id = t.battlefield(p, name);
    for d in doors {
        assert!(
            mtg_engine::rooms::unlock(&mut t.g, id, *d, p),
            "{name} door {d}"
        );
    }
    t.g.recompute();
    id
}

#[test]
fn promising_stairs_counts_both_names_of_a_fully_unlocked_room() {
    cr!("709.5c", "709.5", "104.2b");
    ruling!(
        "Central Elevator // Promising Stairs",
        "Promising Stairs's ability counts names, not permanents. A Room with two unlocked doors has two names."
    );
    for last_full in [true, false] {
        let mut t = TestGame::new(2);
        room(&mut t, P0, "Central Elevator // Promising Stairs", &[0, 1]);
        room(&mut t, P0, "Cramped Vents // Access Maze", &[0, 1]);
        room(&mut t, P0, "Walk-In Closet // Forgotten Cellar", &[0, 1]);
        let doors: &[usize] = if last_full { &[0, 1] } else { &[1] };
        room(&mut t, P0, "Painter's Studio // Defaced Gallery", doors);
        // (The unlock triggers, such as Central Elevator's search, are dropped.)
        t.g.stack.clear();
        t.g.pending_triggers.clear();
        t.set_step(P1, Step::End);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        assert_eq!(t.has_lost(P1), last_full, "eight names: {last_full}");
    }
}

#[test]
fn central_elevator_checks_only_the_names_of_unlocked_doors() {
    cr!("201.2c", "709.5", "701.23a");
    ruling!(
        "Central Elevator // Promising Stairs",
        "Two or more objects (cards, permanents, spells, etc.) have the same name if they have at least one name in common, even if one or more of those objects have additional names. Central Elevator's ability lets you search your library for any Room card that doesn't share a name with a Room you control on the battlefield, checking only the names of unlocked doors. If a Room on the battlefield has no unlocked doors, it doesn't have either of its names."
    );
    let mut t = TestGame::new(2);
    // On the battlefield: Cramped Vents // Access Maze with its left door unlocked, and
    // Walk-In Closet // Forgotten Cellar with no door unlocked.
    room(&mut t, P0, "Cramped Vents // Access Maze", &[0]);
    room(&mut t, P0, "Walk-In Closet // Forgotten Cellar", &[]);
    let vents = t.library_top(P0, "Cramped Vents // Access Maze");
    let closet = t.library_top(P0, "Walk-In Closet // Forgotten Cellar");
    let studio = t.library_top(P0, "Painter's Studio // Defaced Gallery");
    let elevator = room(&mut t, P0, "Central Elevator // Promising Stairs", &[]);
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(closet)]);
    assert!(mtg_engine::rooms::unlock(&mut t.g, elevator, 0, P0));
    t.g.flush_events();
    t.resolve_all();
    let offered: Vec<Vec<Entity>> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(offered.len(), 1, "{}", t.dump_log());
    assert!(!offered[0].contains(&obj(vents)), "shares Cramped Vents");
    assert!(
        offered[0].contains(&obj(closet)),
        "a locked Room has no names"
    );
    assert!(offered[0].contains(&obj(studio)));
    assert!(t.in_hand(P0, "Walk-In Closet // Forgotten Cellar"));
}

// --- Boing! -------------------------------------------------------------------------------------------

#[test]
fn boing_with_a_result_of_0_doesnt_scry() {
    cr!("706.2", "706.6", "701.22a");
    ruling!(
        "Boing!",
        "If the result is 0 or less because of modifications to the result, you don't scry at all."
    );
    supported("Boing!");
    supported("Xenosquirrels");
    for decrease in [false, true] {
        let mut t = TestGame::new(2);
        let squirrels = t.enter(P0, "Xenosquirrels");
        t.settle();
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.g.dice.loaded.push_back(1);
        t.answer_yes(P0, decrease);
        // "increase or decrease": decrease.
        t.answer(P0, DecisionKind::Option, Answer::Index(1));
        let from = t.asked().len();
        cast_new(&mut t, P0, "Boing!", &[obj(bears)]);
        t.resolve_all();
        assert!(t.in_hand(P1, "Grizzly Bears"));
        let scried = asked_since(&t, from)
            .iter()
            .any(|(_, d)| matches!(d, Decision::Scry { .. }));
        assert_eq!(scried, !decrease, "{:?}", asked_since(&t, from));
        assert_eq!(t.counters(squirrels, "+1/+1"), if decrease { 1 } else { 2 });
    }
}
