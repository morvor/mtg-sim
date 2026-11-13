//! Rulings batch P187 — Phyrexian creatures: untap restrictions tied to a tapped source
//! (CR 502.3, 611.2b), block requirements (CR 509.1c), changing targets (CR 115.7),
//! opening-hand reveals and casting during resolution (CR 103.6, 608.2g), state-based
//! actions before triggers (CR 704.3), maximum hand size (CR 402.2, 514.1), infect and
//! poison (CR 702.90), creature type choices (CR 205.3m), and targets chosen as abilities
//! are put on the stack (CR 603.3d, 608.2b).

use crate::r_s01_common::{attack_with, supported, tokens, triggers_on_stack};
use crate::r_s02_common::{can_activate, destroy, target_candidates};
use crate::r_s04_common::next_upkeep;
use crate::r_s06_common::damage;
use crate::r_s09_common::to_combat;
use crate::r_s21_common::legal_blocks;
use crate::r_s25_common::{lands_for_cost, targets_of};
use crate::r_s28_common::{cast_card, player_counters};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::opening_hand::opening_hand_actions;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn o(id: ObjectId) -> Entity {
    Entity::Object(id)
}

// ---------------------------------------------------------------------------------------
// Phyrexian Gremlins
// ---------------------------------------------------------------------------------------

/// P0's Phyrexian Gremlins taps `target` with its ability.
fn gremlins_tap(t: &mut TestGame, gremlins: ObjectId, target: ObjectId) {
    t.activate(P0, gremlins, 0, &[o(target)]).expect("gremlins");
    t.resolve_all();
    assert!(t.obj_now(target).tapped);
}

#[test]
fn gremlins_keep_an_artifact_tapped_only_during_untap_steps() {
    cr!("502.3", "611.2b");
    ruling!(
        "Phyrexian Gremlins",
        "Only prevents the artifact from untapping during untap step. Ones that untap during upkeep are not inhibited."
    );
    supported("Phyrexian Gremlins");
    supported("Voltaic Key");
    let mut t = TestGame::new(2);
    let gremlins = t.battlefield(P0, "Phyrexian Gremlins");
    let stone = t.battlefield(P1, "Mind Stone");
    gremlins_tap(&mut t, gremlins, stone);
    // P1's untap step: it stays tapped.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(stone).tapped);
    assert!(t.obj_now(gremlins).tapped);
    // In P1's upkeep, Voltaic Key untaps it.
    let key = t.battlefield(P1, "Voltaic Key");
    t.lands(P1, "Wastes", 1);
    t.activate(P1, key, 0, &[o(stone)]).expect("key");
    t.resolve_all();
    assert!(!t.obj_now(stone).tapped);
}

#[test]
fn gremlins_effect_continues_after_the_target_stops_being_an_artifact() {
    cr!("611.2b", "611.2c");
    ruling!(
        "Phyrexian Gremlins",
        "The effect does not end if the target stops being valid. For example, if it stops being an artifact."
    );
    supported("Frogify");
    // Ornithopter becomes a Frog creature (no longer an artifact).
    let mut t = TestGame::new(2);
    let gremlins = t.battlefield(P0, "Phyrexian Gremlins");
    let thopter = t.battlefield(P1, "Ornithopter");
    gremlins_tap(&mut t, gremlins, thopter);
    crate::r_s06_common::attach_new(&mut t, P0, "Frogify", thopter);
    assert!(!t.obj_now(thopter).is(CardType::Artifact));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(thopter).tapped);
}

#[test]
fn gremlins_untapping_with_your_own_artifact_doesnt_free_the_artifact() {
    cr!("502.3", "611.2b");
    ruling!(
        "Phyrexian Gremlins",
        "The target artifact can’t untap if the Gremlins and the artifact are tapped during untap, even if it is your artifact, and you plan on untapping the Gremlins."
    );
    let mut t = TestGame::new(2);
    let gremlins = t.battlefield(P0, "Phyrexian Gremlins");
    let stone = t.battlefield(P0, "Mind Stone");
    gremlins_tap(&mut t, gremlins, stone);
    // P0 chooses to untap the Gremlins in their untap step.
    t.answer_yes(P0, true);
    next_upkeep(&mut t, P0);
    assert!(!t.obj_now(gremlins).tapped);
    assert!(t.obj_now(stone).tapped);
    // The effect has ended: the next untap step untaps it.
    next_upkeep(&mut t, P0);
    assert!(!t.obj_now(stone).tapped);
}

// ---------------------------------------------------------------------------------------
// Tangle Angler
// ---------------------------------------------------------------------------------------

/// P0 activates Tangle Angler's "{G}: Target creature blocks this creature this turn if
/// able." targeting `target`.
fn angle(t: &mut TestGame, angler: ObjectId, target: ObjectId) {
    t.lands(P0, "Forest", 1);
    t.activate(P0, angler, 0, &[o(target)]).expect("angler");
    t.resolve_all();
}

#[test]
fn tangle_angler_requirement_does_nothing_if_the_creature_cant_block_it() {
    cr!("509.1c");
    ruling!(
        "Tangle Angler",
        "the other creature is free to block whichever creature its controller chooses, or block no creatures at all."
    );
    supported("Tangle Angler");
    // Tangle Angler gains flying (Jump), so Grizzly Bears can't block it.
    let mut t = TestGame::new(2);
    let angler = t.battlefield(P0, "Tangle Angler");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    angle(&mut t, angler, bears);
    t.answer_targets(P0, &[o(angler)]);
    cast_card(&mut t, P0, "Jump");
    t.resolve_all();
    attack_with(
        &mut t,
        &[(angler, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    assert!(!legal_blocks(&mut t, P1, &[(bears, angler)]));
    assert!(legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(bears, giant)]));
}

#[test]
fn tangle_angler_can_target_tapped_creatures_but_they_needn_t_block() {
    cr!("509.1c", "115.1");
    ruling!(
        "Tangle Angler",
        "Such creatures can be targeted by Tangle Angler’s activated ability, but the requirement to block does nothing."
    );
    let mut t = TestGame::new(2);
    let angler = t.battlefield(P0, "Tangle Angler");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Hill Giant");
    t.g.tap(bears);
    angle(&mut t, angler, bears);
    attack_with(&mut t, &[(angler, Entity::Player(P1))]);
    assert!(legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(other, angler)]));
}

#[test]
fn tangle_angler_can_force_several_creatures_to_block_it() {
    cr!("509.1c");
    ruling!(
        "Tangle Angler",
        "You may activate Tangle Angler’s second ability multiple times in a turn to force multiple creatures to block it."
    );
    let mut t = TestGame::new(2);
    let angler = t.battlefield(P0, "Tangle Angler");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    angle(&mut t, angler, a);
    angle(&mut t, angler, b);
    attack_with(&mut t, &[(angler, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(!legal_blocks(&mut t, P1, &[(a, angler)]));
    assert!(!legal_blocks(&mut t, P1, &[(b, angler)]));
    assert!(legal_blocks(&mut t, P1, &[(a, angler), (b, angler)]));
}

// ---------------------------------------------------------------------------------------
// Spellskite
// ---------------------------------------------------------------------------------------

#[test]
fn spellskite_chooses_which_target_to_change_as_it_resolves() {
    cr!("115.7b", "608.2b");
    ruling!(
        "Spellskite",
        "you choose which one target you're changing to Spellskite as Spellskite's ability resolves."
    );
    supported("Spellskite");
    supported("Symbiosis");
    // "Two target creatures each get +2/+2 until end of turn."
    for which in [0usize, 1] {
        let mut t = TestGame::new(2);
        let skite = t.battlefield(P0, "Spellskite");
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P1, "Hill Giant");
        t.set_step(P1, Step::PrecombatMain);
        lands_for_cost(&mut t, P1, "Symbiosis");
        let card = t.hand(P1, "Symbiosis");
        let s = t.cast(P1, card).targets(&[o(a), o(b)]).go();
        t.lands(P0, "Island", 1);
        let from = t.asked().len();
        t.activate(P0, skite, 0, &[o(s)]).unwrap();
        // Nothing about which target is asked while activating.
        assert!(!t.asked()[from..]
            .iter()
            .any(|(_, d)| matches!(d, Decision::ChooseOption { prompt, .. }
                if !prompt.starts_with("How will you pay"))));
        t.answer(P0, DecisionKind::Option, Answer::Index(which));
        t.resolve();
        let now = targets_of(&t, s);
        let expect = if which == 0 {
            vec![o(skite), o(b)]
        } else {
            vec![o(a), o(skite)]
        };
        assert_eq!(now, expect);
    }
}

#[test]
fn spellskite_can_be_activated_when_it_cant_become_a_target() {
    cr!("115.7b", "115.7");
    ruling!(
        "Spellskite",
        "You can activate Spellskite's ability even if Spellskite isn't a legal target for the target spell or ability—or even if that spell or ability has no targets."
    );
    supported("Duress");
    // Duress targets an opponent: Spellskite can't be its target.
    let mut t = TestGame::new(2);
    let skite = t.battlefield(P0, "Spellskite");
    t.hand(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    lands_for_cost(&mut t, P1, "Duress");
    let card = t.hand(P1, "Duress");
    let s = t.cast(P1, card).target(Entity::Player(P0)).go();
    t.lands(P0, "Island", 1);
    t.activate(P0, skite, 0, &[o(s)]).expect("activate");
    t.resolve();
    assert_eq!(targets_of(&t, s), vec![Entity::Player(P0)]);
    // A spell with no targets.
    let mut t = TestGame::new(2);
    let skite = t.battlefield(P0, "Spellskite");
    t.set_step(P1, Step::PrecombatMain);
    lands_for_cost(&mut t, P1, "Divination");
    let card = t.hand(P1, "Divination");
    let s = t.cast(P1, card).go();
    t.lands(P0, "Island", 1);
    t.activate(P0, skite, 0, &[o(s)]).expect("activate");
    t.resolve();
    assert!(targets_of(&t, s).is_empty());
    t.resolve();
    assert_eq!(t.hand_size(P1), 2);
}

// ---------------------------------------------------------------------------------------
// Chancellors
// ---------------------------------------------------------------------------------------

#[test]
fn two_revealed_chancellors_of_the_spires_mill_fourteen() {
    cr!("103.6", "603.7");
    ruling!(
        "Chancellor of the Spires",
        "If you reveal more than one Chancellor of the Spires from your opening hand, each opponent will put the top seven cards of their library into their graveyard that many times"
    );
    supported("Chancellor of the Spires");
    let mut t = TestGame::new(2);
    t.hand(P0, "Chancellor of the Spires");
    t.hand(P0, "Chancellor of the Spires");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    opening_hand_actions(&mut t.g);
    let lib = t.library_size(P1);
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.library_size(P1), lib - 14);
}

/// P0's Chancellor of the Spires enters and its trigger targets `target` in an opponent's
/// graveyard; P0 chooses to cast it.
fn chancellor_casts(t: &mut TestGame, target: ObjectId, spell_targets: &[Entity]) {
    t.answer_targets(P0, &[o(target)]);
    t.answer_yes(P0, true);
    for e in spell_targets {
        t.answer_targets(P0, &[*e]);
    }
    t.enter(P0, "Chancellor of the Spires");
    t.settle();
    t.resolve();
}

#[test]
fn chancellor_of_the_spires_ignores_timing_but_not_other_restrictions() {
    cr!("608.2g", "601.2", "307.5");
    ruling!(
        "Chancellor of the Spires",
        "Timing restrictions based on the card's type are ignored. Other restrictions, such as \"Cast [this card] only during combat,\" are not."
    );
    supported("Panic");
    // A sorcery, during an opponent's turn with the ability resolving.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let div = t.graveyard(P1, "Divination");
    let hand = t.hand_size(P0);
    chancellor_casts(&mut t, div, &[]);
    assert_eq!(t.zone(t.g.current(div)), Zone::Stack);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // Panic: "Cast this spell only during combat before blockers are declared." Not now.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let panic = t.graveyard(P1, "Panic");
    chancellor_casts(&mut t, panic, &[o(bears)]);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(panic)), Zone::Graveyard(P1));
}

#[test]
fn chancellor_of_the_spires_leaves_a_card_it_cant_cast_in_the_graveyard() {
    cr!("608.2g", "601.2c");
    ruling!(
        "Chancellor of the Spires",
        "If you are unable to cast the card, perhaps because there are no legal targets available, the card remains in its owner's graveyard."
    );
    supported("Shatter");
    // Shatter ("Destroy target artifact.") with no artifacts on the battlefield.
    let mut t = TestGame::new(2);
    let shatter = t.graveyard(P1, "Shatter");
    chancellor_casts(&mut t, shatter, &[]);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(shatter)), Zone::Graveyard(P1));
}

#[test]
fn chancellor_of_the_forge_counts_itself_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Chancellor of the Forge",
        "The number of creature tokens you put onto the battlefield with the last ability is determined when the ability resolves. If Chancellor of the Forge is still on the battlefield at that time, it will be counted."
    );
    supported("Chancellor of the Forge");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.enter(P0, "Chancellor of the Forge");
    t.settle();
    // In response, another creature arrives: X = 3.
    t.battlefield(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 3);
    // If the Chancellor is gone by then, it isn't counted.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let c = t.enter(P0, "Chancellor of the Forge");
    t.settle();
    destroy(&mut t, c);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}

// ---------------------------------------------------------------------------------------
// State-based actions and damage
// ---------------------------------------------------------------------------------------

#[test]
fn perilous_myrs_controller_loses_before_its_trigger_resolves() {
    cr!("704.3", "704.5a", "104.3b");
    ruling!(
        "Perilous Myr",
        "If your life total is brought to 0 or less at the same time that Perilous Myr is dealt lethal damage, you lose the game before its triggered ability resolves."
    );
    supported("Perilous Myr");
    supported("Earthquake");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Perilous Myr");
    t.g.player_mut(P0).life = 2;
    t.g.player_mut(P1).life = 4;
    // Its trigger could deal the 2 damage P1 has left.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    t.lands(P1, "Wastes", 2);
    let quake = t.hand(P1, "Earthquake");
    t.cast(P1, quake).x(2).go();
    t.resolve();
    t.settle();
    assert!(t.has_lost(P0));
    assert!(!t.has_lost(P1));
    assert_eq!(t.life(P1), 2);
}

#[test]
fn spiteful_bully_damages_itself_when_its_your_only_creature() {
    cr!("603.3d", "115.1");
    ruling!(
        "Spiteful Bully",
        "It can damage itself. In fact, it has to if it is the only creature you control."
    );
    supported("Spiteful Bully");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spiteful Bully");
    t.battlefield(P1, "Grizzly Bears");
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Spiteful Bully"));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn skinrender_must_target_itself_when_alone() {
    cr!("603.3d", "115.1");
    ruling!(
        "Skinrender",
        "This ability is mandatory. If there are no other creatures on the battlefield, you must target Skinrender itself."
    );
    supported("Skinrender");
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    let skin = t.enter(P0, "Skinrender");
    t.settle();
    assert_eq!(target_candidates(&t, P0, from), vec![vec![o(skin)]]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Skinrender"));
}

#[test]
fn nested_ghoul_triggers_on_damage_from_an_infect_source() {
    cr!("702.90b", "120.3d");
    ruling!(
        "Nested Ghoul",
        "Nested Ghoul’s ability will trigger if a source with infect deals damage to it."
    );
    supported("Nested Ghoul");
    supported("Plague Stinger");
    let mut t = TestGame::new(2);
    let ghoul = t.battlefield(P0, "Nested Ghoul");
    let stinger = t.battlefield(P1, "Plague Stinger");
    damage(&mut t, stinger, 1, ghoul);
    assert_eq!(t.counters(ghoul, "-1/-1"), 1);
    t.resolve_all();
    let zombies: Vec<_> = tokens(&t, P0)
        .into_iter()
        .filter(|id| t.obj_now(*id).chars.has_subtype("Zombie"))
        .collect();
    assert_eq!(zombies.len(), 1);
}

#[test]
fn phyrexian_negator_sacrificing_itself_doesnt_reduce_the_sacrifices() {
    cr!("701.21a", "603.10a");
    ruling!(
        "Phyrexian Negator",
        "Sacrificing this card does not prevent you from having to make the other sacrifices."
    );
    supported("Phyrexian Negator");
    // Dealt 3 damage: P0 sacrifices the Negator and two other permanents.
    let mut t = TestGame::new(2);
    let negator = t.battlefield(P0, "Phyrexian Negator");
    let lands = t.lands(P0, "Swamp", 3);
    let src = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, src, 3, negator);
    t.answer_choose(P0, &[o(negator), o(lands[0]), o(lands[1])]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Phyrexian Negator"));
    assert_eq!(t.graveyard_size(P0), 3);
    // Dealt 5 (lethal) damage: it dies, and five other permanents are still sacrificed.
    let mut t = TestGame::new(2);
    let negator = t.battlefield(P0, "Phyrexian Negator");
    t.lands(P0, "Swamp", 6);
    let src = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, src, 5, negator);
    assert!(t.in_graveyard(P0, "Phyrexian Negator"));
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 6);
    assert_eq!(t.g.permanents().filter(|p| p.controller == P0).count(), 1);
}

// ---------------------------------------------------------------------------------------
// Jin-Gitaxias, Core Augur
// ---------------------------------------------------------------------------------------

#[test]
fn jin_gitaxias_doesnt_reduce_its_controllers_hand_size() {
    cr!("402.2", "514.1", "513.1a");
    ruling!(
        "Jin-Gitaxias, Core Augur",
        "Jin-Gitaxias doesn't affect the maximum hand size of its controller. Because the cleanup step is after the end step, its controller may have to discard some of the cards that were just drawn."
    );
    supported("Jin-Gitaxias, Core Augur");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jin-Gitaxias, Core Augur");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 9);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.hand_size(P0), 7);
}

#[test]
fn jin_gitaxias_makes_an_opponents_hand_size_zero() {
    cr!("402.2", "514.1");
    ruling!(
        "Jin-Gitaxias, Core Augur",
        "Unless another spell or ability is affecting your opponent's maximum hand size, Jin-Gitaxias's ability will result in your opponent having a maximum hand size of zero. They will discard each card from their hand during their cleanup step."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jin-Gitaxias, Core Augur");
    for _ in 0..3 {
        t.hand(P1, "Grizzly Bears");
    }
    t.g.recompute();
    assert_eq!(t.g.player(P1).max_hand_size, Some(0));
    assert_eq!(t.g.player(P0).max_hand_size, Some(7));
    // P1's turn: draws one, then discards everything in cleanup.
    t.advance_to(P1, Step::End);
    assert_eq!(t.hand_size(P1), 4);
    next_upkeep(&mut t, P0);
    assert_eq!(t.hand_size(P1), 0);
}

// ---------------------------------------------------------------------------------------
// Activated-ability restrictions, choices and targets
// ---------------------------------------------------------------------------------------

#[test]
fn phyrexian_revoker_stops_a_named_cards_cycling_from_hand() {
    cr!("602.5", "201.4");
    ruling!(
        "Phyrexian Revoker",
        "Phyrexian Revoker's ability affects sources with the chosen name no matter what zone they are in. For example, a cycling ability of a card with the chosen name can't be activated from hand."
    );
    supported("Phyrexian Revoker");
    supported("Street Wraith");
    let mut t = TestGame::new(2);
    let wraith = t.hand(P1, "Street Wraith");
    t.set_step(P1, Step::PrecombatMain);
    assert!(can_activate(&mut t, P1, wraith));
    t.answer(P0, DecisionKind::Name, Answer::Text("Street Wraith".into()));
    t.enter(P0, "Phyrexian Revoker");
    t.settle();
    assert!(!can_activate(&mut t, P1, wraith));
}

#[test]
fn plague_engineers_type_is_chosen_as_it_enters_and_applies_at_once() {
    cr!("614.12", "614.1c", "704.5f");
    ruling!(
        "Plague Engineer",
        "The choice of creature type is made as Plague Engineer enters the battlefield. Players can’t respond to this choice. The -1/-1 effect starts applying immediately."
    );
    supported("Plague Engineer");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P1, "Llanowar Elves");
    let i = subtype_lists()
        .creature
        .iter()
        .position(|s| s == "Elf")
        .unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
    t.enter(P0, "Plague Engineer");
    assert_eq!(t.stack_len(), 0);
    t.g.recompute();
    assert_eq!(t.pt(elves), (0, 0));
    t.settle();
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
}

#[test]
fn plague_engineer_offers_only_creature_types() {
    cr!("205.3m", "614.12");
    ruling!(
        "Plague Engineer",
        "You must choose an existing creature type, such as Sliver or Warrior. Card types such as artifact, and supertypes such as legendary or snow, can’t be chosen."
    );
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    t.enter(P0, "Plague Engineer");
    let options: Vec<String> = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .expect("a creature type choice");
    assert!(options.iter().any(|s| s == "Sliver"));
    assert!(options.iter().any(|s| s == "Warrior"));
    for bad in [
        "Artifact",
        "Legendary",
        "Snow",
        "Creature",
        "Forest",
        "Equipment",
    ] {
        assert!(!options.iter().any(|s| s == bad), "{bad}");
    }
    // An out-of-range answer falls back to a real creature type.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(100_000));
    let pe = t.enter(P0, "Plague Engineer");
    let chosen = t.obj_now(pe).choices.creature_type.clone();
    assert!(chosen.is_some_and(|c| subtype_lists().creature.iter().any(|s| *s == c)));
}

#[test]
fn tsabo_tavoc_can_destroy_only_legendary_creatures() {
    cr!("115.1", "205.4a");
    ruling!(
        "Tsabo Tavoc",
        "The destroy ability only works on legendary creatures, not on other legendary permanents."
    );
    supported("Tsabo Tavoc");
    let mut t = TestGame::new(2);
    let tsabo = t.battlefield(P0, "Tsabo Tavoc");
    let isamaru = t.battlefield(P1, "Isamaru, Hound of Konda");
    let mox = t.battlefield(P1, "Mox Opal");
    t.lands(P0, "Swamp", 2);
    let from = t.asked().len();
    t.activate(P0, tsabo, 0, &[o(isamaru)]).expect("tsabo");
    let cands = target_candidates(&t, P0, from);
    assert!(cands[0].contains(&o(isamaru)));
    assert!(!cands[0].contains(&o(mox)));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Isamaru, Hound of Konda"));
    assert!(t.on_battlefield(mox));
}

#[test]
fn viridian_betrayers_has_infect_only_against_a_poisoned_player() {
    cr!("702.90b", "702.90c", "510.2");
    ruling!(
        "Viridian Betrayers",
        "If the player has no poison counters, damage dealt to that player by Viridian Betrayers will result in loss of life, even if it's dealt at the same time as damage from another creature with infect."
    );
    supported("Viridian Betrayers");
    let mut t = TestGame::new(2);
    let betrayers = t.battlefield(P0, "Viridian Betrayers");
    let stinger = t.battlefield(P0, "Plague Stinger");
    t.attack(
        &[
            (betrayers, Entity::Player(P1)),
            (stinger, Entity::Player(P1)),
        ],
        &[],
    );
    assert_eq!(t.life(P1), 17);
    assert_eq!(player_counters(&t, P1, "poison"), 1);
    // Now P1 is poisoned: the Betrayers have infect.
    t.g.recompute();
    assert!(t.obj_now(betrayers).has_keyword(KeywordKind::Infect));
    next_upkeep(&mut t, P0);
    t.set_step(P0, Step::BeginningOfCombat);
    t.g.combat = None;
    t.attack(&[(betrayers, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 17);
    assert_eq!(player_counters(&t, P1, "poison"), 4);
}

#[test]
fn skittering_skirges_trigger_resolves_before_the_creature_spell() {
    cr!("603.3", "117.3b");
    ruling!(
        "Skittering Skirge",
        "The triggered ability triggers when the creature spell is cast, and it is put on the stack before any responses can be cast or activated. This ability will resolve before that creature enters."
    );
    supported("Skittering Skirge");
    let mut t = TestGame::new(2);
    let skirge = t.battlefield(P0, "Skittering Skirge");
    lands_for_cost(&mut t, P0, "Grizzly Bears");
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(triggers_on_stack(&t, "sacrifice"), 1);
    t.resolve();
    assert!(!t.on_battlefield(skirge));
    assert_eq!(t.zone(t.g.current(bears)), Zone::Stack);
    t.resolve();
    assert!(t.on_battlefield(t.g.current(bears)));
}

#[test]
fn wurmcoil_engines_tokens_enter_at_the_same_time() {
    cr!("603.2c", "111.1");
    ruling!(
        "Wurmcoil Engine",
        "The two creature tokens enter at the same time."
    );
    supported("Wurmcoil Engine");
    supported("Woodland Champion");
    // "Whenever one or more tokens you control enter, put that many +1/+1 counters on this
    // creature." triggers once for both.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Wurmcoil Engine");
    let champion = t.battlefield(P0, "Woodland Champion");
    destroy(&mut t, wurm);
    t.resolve();
    assert_eq!(tokens(&t, P0).len(), 2);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "tokens you control enter"), 1);
    t.resolve_all();
    assert_eq!(t.counters(champion, "+1/+1"), 2);
}

#[test]
fn sleeper_agent_damages_its_new_controller() {
    cr!("109.5", "603.3a");
    ruling!(
        "Sleeper Agent",
        "When it enters under your control, you give control of it to an opponent. After that it damages them each turn because the \"you\" on the card means its controller."
    );
    supported("Sleeper Agent");
    let mut t = TestGame::new(2);
    let agent = t.enter(P0, "Sleeper Agent");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.obj_now(agent).controller, P1);
    next_upkeep(&mut t, P1);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn urabrask_gives_itself_haste() {
    cr!("302.6", "702.10b");
    ruling!(
        "Urabrask the Hidden",
        "Urabrask the Hidden gives itself haste while it's on the battlefield."
    );
    supported("Urabrask the Hidden");
    let mut t = TestGame::new(2);
    let urabrask = t.battlefield_sick(P0, "Urabrask the Hidden");
    t.g.recompute();
    assert!(t.obj_now(urabrask).has_keyword(KeywordKind::Haste));
    to_combat(&mut t, P0);
    assert!(crate::r_s02_common::can_attack(&mut t, urabrask));
}

#[test]
fn dementia_bat_cant_make_a_player_discard_a_spell_already_cast() {
    cr!("601.2a", "701.9a");
    ruling!(
        "Dementia Bat",
        "You can’t force an opponent to discard a card they cast by activating Dementia Bat’s ability in response. The spell is on the stack by then."
    );
    supported("Dementia Bat");
    let mut t = TestGame::new(2);
    let bat = t.battlefield(P0, "Dementia Bat");
    t.set_step(P1, Step::PrecombatMain);
    lands_for_cost(&mut t, P1, "Hill Giant");
    let giant = t.hand(P1, "Hill Giant");
    t.hand(P1, "Grizzly Bears");
    t.cast(P1, giant).go();
    assert_eq!(t.hand_size(P1), 1);
    t.lands(P0, "Swamp", 5);
    t.activate(P0, bat, 0, &[Entity::Player(P1)]).expect("bat");
    t.resolve();
    assert_eq!(t.hand_size(P1), 0);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(giant)));
}

#[test]
fn blind_zealot_targets_on_the_stack_and_sacrifices_on_resolution() {
    cr!("603.3d", "608.2b");
    ruling!(
        "Blind Zealot",
        "You’ll choose the target creature when the ability goes on the stack. You’ll choose whether or not to sacrifice Blind Zealot when the ability resolves."
    );
    supported("Blind Zealot");
    for sac in [true, false] {
        let mut t = TestGame::new(2);
        let zealot = t.battlefield(P0, "Blind Zealot");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.g.tap(bears);
        attack_with(&mut t, &[(zealot, Entity::Player(P1))]);
        t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
        t.answer_targets(P0, &[o(bears)]);
        let from = t.asked().len();
        t.advance_to(P0, Step::CombatDamage);
        t.g.flush_events();
        t.settle();
        assert_eq!(t.life(P1), 18);
        let trig = *t.g.stack.last().expect("trigger");
        assert_eq!(targets_of(&t, trig), vec![o(bears)]);
        assert!(!t.asked()[from..]
            .iter()
            .any(|(_, d)| matches!(d, Decision::YesNo { .. })));
        t.answer_yes(P0, sac);
        t.resolve();
        assert_eq!(t.on_battlefield(zealot), !sac);
        assert_eq!(t.on_battlefield(bears), !sac);
    }
}

#[test]
fn vedalken_anatomist_does_nothing_if_its_target_is_illegal() {
    cr!("608.2b");
    ruling!(
        "Vedalken Anatomist",
        "If the creature is an illegal target when the ability tries to resolve, it won’t resolve. None of its effects happen."
    );
    supported("Vedalken Anatomist");
    supported("Blossoming Defense");
    let mut t = TestGame::new(2);
    let anatomist = t.battlefield(P0, "Vedalken Anatomist");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 3);
    t.activate(P0, anatomist, 0, &[o(bears)])
        .expect("anatomist");
    // In response, the Bears gain hexproof.
    t.answer_targets(P1, &[o(bears)]);
    cast_card(&mut t, P1, "Blossoming Defense");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.counters(bears, "-1/-1"), 0);
    assert!(!t.obj_now(bears).tapped);
    assert_eq!(t.pt(bears), (4, 4));
}

// ---------------------------------------------------------------------------------------
// Gift of Tusks; Vault 87: Forced Evolution
// ---------------------------------------------------------------------------------------

#[test]
fn gift_of_tusks_doesnt_stop_an_ability_that_already_triggered() {
    cr!("113.7a", "603.2");
    ruling!(
        "Gift of Tusks",
        "Gift of Tusks doesn’t counter abilities that have already triggered or been activated."
    );
    supported("Elvish Visionary");
    // "When this creature enters, draw a card."
    let mut t = TestGame::new(2);
    let elf = t.enter(P0, "Elvish Visionary");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.answer_targets(P1, &[o(elf)]);
    cast_card(&mut t, P1, "Gift of Tusks");
    t.resolve();
    assert!(t.obj_now(elf).chars.abilities.is_empty());
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn vault_87_leaving_before_chapter_one_resolves_takes_no_control() {
    cr!("611.2b", "714.2b");
    ruling!(
        "Vault 87: Forced Evolution",
        "If Vault 87 leaves the battlefield before its first chapter ability resolves, you won't gain control of the target creature at all."
    );
    supported("Vault 87: Forced Evolution");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[o(bears)]);
    let vault = t.enter(P0, "Vault 87: Forced Evolution");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, vault);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P1);
    // Had it stayed, P0 would control the Bears.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[o(bears)]);
    t.enter(P0, "Vault 87: Forced Evolution");
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
}
