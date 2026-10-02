//! Rulings batch P130: the remaining rulings of the "refund" group — triggers that resolve
//! before the spell that caused them, loyalty abilities, effects that affect only what's
//! there as they resolve, intervening "if" clauses, and costs.

use crate::r_p130_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn treasures(t: &TestGame, p: PlayerId) -> usize {
    crate::r_p108_common::tokens_with(t, p, "Treasure")
}

fn counter_spell(t: &mut TestGame, spell: ObjectId) {
    assert!(t.g.counter(spell, None));
    t.settle();
}

// --- Triggers that resolve before the spell -------------------------------------------------

#[test]
fn goldspan_dragons_target_trigger_resolves_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5", "701.6a");
    ruling!(
        "Goldspan Dragon",
        "An ability that triggers when a creature becomes the target of a spell resolves before the spell that caused it to trigger."
    );
    supported("Goldspan Dragon");
    let mut t = TestGame::new(2);
    let gold = t.battlefield(P0, "Goldspan Dragon");
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[obj(gold)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert!(is_trigger(&t, top(&t)));
    counter_spell(&mut t, bolt);
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 1);
}

#[test]
fn goldspan_dragon_targeted_several_times_by_one_spell_triggers_once() {
    cr!("603.2c", "115.3");
    ruling!(
        "Goldspan Dragon",
        "If a spell targets Goldspan Dragon more than once, the triggered ability will trigger only once."
    );
    let mut t = TestGame::new(2);
    let gold = t.battlefield(P0, "Goldspan Dragon");
    cast_new(
        &mut t,
        P0,
        "Seeds of Strength",
        &[obj(gold), obj(gold), obj(gold)],
    );
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 1);
    assert_eq!(t.pt(gold), (7, 7));
}

#[test]
fn runaway_steam_kins_trigger_resolves_even_if_the_spell_is_countered() {
    cr!("603.3", "701.6a");
    ruling!(
        "Runaway Steam-Kin",
        "Runaway Steam-Kin’s triggered ability resolves before the spell that caused it to trigger."
    );
    supported("Runaway Steam-Kin");
    let mut t = TestGame::new(2);
    let kin = t.battlefield(P0, "Runaway Steam-Kin");
    let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    counter_spell(&mut t, bolt);
    t.resolve_all();
    assert_eq!(t.counters(kin, counters::PLUS1), 1);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn runaway_steam_kins_intervening_if_is_checked_twice() {
    cr!("603.4");
    ruling!(
        "Runaway Steam-Kin",
        "If Runaway Steam-Kin has three or more +1/+1 counters on it after you’re done paying for a red spell, its ability doesn’t trigger at all."
    );
    // Three counters: no trigger.
    let mut t = TestGame::new(2);
    let kin = t.battlefield(P0, "Runaway Steam-Kin");
    t.g.add_counters(obj(kin), counters::PLUS1, 3, None);
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // Two counters as it triggers, three as it resolves: no counter.
    let mut t = TestGame::new(2);
    let kin = t.battlefield(P0, "Runaway Steam-Kin");
    t.g.add_counters(obj(kin), counters::PLUS1, 2, None);
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.g.add_counters(obj(kin), counters::PLUS1, 1, None);
    t.resolve_all();
    assert_eq!(t.counters(kin, counters::PLUS1), 3);
}

#[test]
fn casting_runaway_steam_kin_doesnt_trigger_its_own_ability() {
    cr!("603.6", "603.2");
    ruling!(
        "Runaway Steam-Kin",
        "Runaway Steam-Kin has to be on the battlefield for its ability to trigger."
    );
    let mut t = TestGame::new(2);
    let kin = cast_new(&mut t, P0, "Runaway Steam-Kin", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(kin, counters::PLUS1), 0);
}

#[test]
fn spawn_gang_commanders_cast_trigger_resolves_even_if_it_is_countered() {
    cr!("603.3", "601.2i");
    ruling!(
        "Spawn-Gang Commander",
        "Spawn-Gang Commander's triggered ability will resolve before Spawn-Gang Commander does."
    );
    supported("Spawn-Gang Commander");
    let mut t = TestGame::new(2);
    let spell = cast_new(&mut t, P0, "Spawn-Gang Commander", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    counter_spell(&mut t, spell);
    t.resolve_all();
    assert_eq!(crate::r_p108_common::tokens_with(&t, P0, "Spawn"), 3);
    assert!(t.in_graveyard(P0, "Spawn-Gang Commander"));
}

// --- Chandra ------------------------------------------------------------------------------

#[test]
fn chandras_exiled_land_cant_be_played_and_deals_damage() {
    cr!("305.9", "608.2c");
    ruling!(
        "Chandra, Torch of Defiance",
        "An effect that instructs you to \"cast\" a card doesn't allow you to play lands."
    );
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra, Torch of Defiance");
    let land = t.library_top(P0, "Mountain");
    t.activate(P0, chandra, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Mountain"));
    assert_eq!(t.life(P1), 18);
    assert!(!can_play_land(&mut t, P0, land));
}

#[test]
fn chandras_exiled_card_is_cast_during_the_resolution_or_not_at_all() {
    cr!("608.2g", "601.2");
    ruling!(
        "Chandra, Torch of Defiance",
        "If you cast the exiled card, you do so as part of the resolution of Chandra's ability."
    );
    // Declined: it stays in exile and can't be cast later; Chandra deals 2 damage.
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra, Torch of Defiance");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    t.answer_yes(P0, false);
    t.activate(P0, chandra, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.life(P1), 18);
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
    // Cast: the creature spell is on the stack as the ability finishes resolving (its
    // sorcery timing is ignored), and there's no damage.
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra, Torch of Defiance");
    t.library_top(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    t.answer_yes(P0, true);
    t.activate(P0, chandra, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.life(P1), 20);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

/// P0's Chandra, Torch of Defiance makes her emblem.
fn chandra_emblem(t: &mut TestGame) {
    let chandra = t.battlefield(P0, "Chandra, Torch of Defiance");
    loyalty(t, chandra, 7);
    t.activate(P0, chandra, 3, &[]).unwrap();
    t.resolve_all();
}

#[test]
fn chandras_emblem_trigger_resolves_before_the_spell() {
    cr!("603.3", "114.4");
    ruling!(
        "Chandra, Torch of Defiance",
        "Chandra's emblem's ability resolves before the spell that caused it to trigger."
    );
    let mut t = TestGame::new(2);
    chandra_emblem(&mut t);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn chandras_emblem_is_a_colorless_source() {
    cr!("114.4", "702.16b");
    ruling!(
        "Chandra, Torch of Defiance",
        "The emblem created by Chandra's last ability is colorless."
    );
    let mut t = TestGame::new(2);
    chandra_emblem(&mut t);
    // Kor Firewalker has protection from red: a colorless source can target and damage it.
    let walker = t.battlefield(P1, "Kor Firewalker");
    t.answer_targets(P0, &[obj(walker)]);
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.settle();
    t.resolve_all();
    assert!(!t.on_battlefield(walker));
}

#[test]
fn chandra_novice_pyromancer_pumps_only_elementals_there_on_resolution() {
    cr!("611.2c");
    ruling!(
        "Chandra, Novice Pyromancer",
        "Chandra's first ability affects only Elementals you control at the time it resolves."
    );
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra, Novice Pyromancer");
    let fire = t.battlefield(P0, "Fire Elemental");
    t.activate(P0, chandra, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(fire), (7, 4));
    let earth = t.battlefield(P0, "Earth Elemental");
    assert_eq!(t.pt(earth), (4, 5));
}

// --- Other planeswalkers ------------------------------------------------------------------

#[test]
fn tezzeret_the_seekers_first_ability_can_have_no_targets() {
    cr!("115.1d", "606.4");
    ruling!(
        "Tezzeret the Seeker",
        "The first ability can target zero, one, or two artifacts."
    );
    supported("Tezzeret the Seeker");
    let mut t = TestGame::new(2);
    let tez = t.battlefield(P0, "Tezzeret the Seeker");
    t.battlefield(P0, "Mind Stone");
    t.answer_targets(P0, &[]);
    t.activate(P0, tez, 0, &[]).unwrap();
    assert_eq!(t.counters(tez, counters::LOYALTY), 5);
    assert!(targets_of(&t, top(&t)).is_empty());
    t.resolve_all();
}

#[test]
fn tezzeret_the_seekers_x_is_chosen_on_activation_and_capped_by_loyalty() {
    cr!("606.4", "107.3", "601.2b");
    ruling!(
        "Tezzeret the Seeker",
        "For the second ability, you choose the value of X when you activate it."
    );
    let mut t = TestGame::new(2);
    let tez = t.battlefield(P0, "Tezzeret the Seeker");
    t.library_top(P0, "Mind Stone");
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    assert!(t.activate(P0, tez, 1, &[]).is_err());
    t.clear_answers();
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, tez, 1, &[]).unwrap();
    assert_eq!(t.counters(tez, counters::LOYALTY), 2);
    assert_eq!(crate::r_s25_common::x_of(&t, top(&t)), Some(2));
    // The library is searched only as it resolves.
    assert!(t.named_on_battlefield("Mind Stone").is_empty());
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Mind Stone").len(), 1);
}

#[test]
fn tezzeret_the_seekers_ultimate_affects_all_artifacts_and_adds_the_creature_type() {
    cr!("205.1b", "613.1d", "613.4b");
    ruling!(
        "Tezzeret the Seeker",
        "The third ability affects all artifacts you control, including artifacts that are already creatures."
    );
    ruling!(
        "Tezzeret the Seeker",
        "The third ability causes artifacts you control to become creatures in addition to their other card types."
    );
    let mut t = TestGame::new(2);
    let tez = t.battlefield(P0, "Tezzeret the Seeker");
    loyalty(&mut t, tez, 5);
    let thopter = t.battlefield(P0, "Ornithopter");
    let stone = t.battlefield(P0, "Mind Stone");
    t.activate(P0, tez, 2, &[]).unwrap();
    t.resolve_all();
    for a in [thopter, stone] {
        assert_eq!(t.pt(a), (5, 5));
        let o = t.obj_now(a);
        assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
    }
}

#[test]
fn garruk_wildspeaker_can_untap_lands_that_arent_tapped() {
    cr!("115.1", "701.26b");
    ruling!(
        "Garruk Wildspeaker",
        "The first ability can target any two lands. They don’t have to be tapped."
    );
    supported("Garruk Wildspeaker");
    let mut t = TestGame::new(2);
    let garruk = t.battlefield(P0, "Garruk Wildspeaker");
    let a = t.battlefield(P0, "Forest");
    let b = t.battlefield(P1, "Forest");
    t.activate(P0, garruk, 0, &[obj(a), obj(b)]).unwrap();
    assert_eq!(targets_of(&t, top(&t)), vec![obj(a), obj(b)]);
    t.resolve_all();
    assert!(!t.obj_now(a).tapped && !t.obj_now(b).tapped);
}

#[test]
fn garruk_wildspeakers_overrun_affects_only_creatures_there_on_resolution() {
    cr!("611.2c");
    ruling!(
        "Garruk Wildspeaker",
        "The third ability affects only creatures you control at the time it resolves."
    );
    let mut t = TestGame::new(2);
    let garruk = t.battlefield(P0, "Garruk Wildspeaker");
    loyalty(&mut t, garruk, 4);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, garruk, 2, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    let later = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(later), (3, 3));
}

#[test]
fn lord_windgrace_with_an_empty_hand_discards_nothing_and_draws_one() {
    cr!("701.9b", "121.1");
    ruling!(
        "Lord Windgrace",
        "If you have no cards in hand as Lord Windgrace's first ability resolves, you'll discard nothing then you'll draw one card."
    );
    supported("Lord Windgrace");
    let mut t = TestGame::new(2);
    let lw = t.battlefield(P0, "Lord Windgrace");
    t.library_top(P0, "Grizzly Bears");
    assert_eq!(t.hand_size(P0), 0);
    t.activate(P0, lw, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.graveyard_size(P0), 0);
}

// --- Effects and conditions -------------------------------------------------------------

#[test]
fn tivit_votes_add_up_with_another_extra_vote_ability() {
    cr!("701.38d");
    ruling!(
        "Tivit, Seller of Secrets",
        "Abilities that allow players to vote an additional time are cumulative."
    );
    supported("Tivit, Seller of Secrets");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ballot Broker");
    t.enter(P0, "Tivit, Seller of Secrets");
    t.settle();
    t.resolve_all();
    // P0 votes three times, P1 once: four tokens.
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 4);
}

#[test]
fn gone_fishing_returns_new_objects() {
    cr!("400.7", "506.4", "704.5m", "704.5n");
    ruling!(
        "Lake-town Mariners // Gone Fishing",
        "After each permanent returns to the battlefield, it will be a new object with no connection to the permanent that was exiled."
    );
    supported("Lake-town Mariners // Gone Fishing");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let splitter = t.battlefield(P0, "Bonesplitter");
    t.g.attach(splitter, obj(bears));
    t.g.add_counters(obj(bears), counters::PLUS1, 1, None);
    let paci = t.battlefield(P1, "Pacifism");
    t.g.attach(paci, obj(giant));
    t.g.recompute();
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Lake-town Mariners // Gone Fishing");
    t.cast(P0, card)
        .method(CastMethod::Half(1))
        .targets(&[obj(bears), obj(giant)])
        .go();
    t.resolve();
    t.settle();
    let b = t.g.current(bears);
    assert_ne!(b, bears);
    assert!(t.on_battlefield(b));
    assert!(!t.g.is_attacking(b));
    assert_eq!(t.counters(b, counters::PLUS1), 0);
    assert!(t.in_graveyard(P1, "Pacifism"));
    assert!(t.on_battlefield(splitter));
    assert!(t.obj_now(splitter).attached_to.is_none());
}

#[test]
fn lost_jitte_doesnt_remove_a_blocker_from_combat() {
    cr!("509.1h", "506.4");
    ruling!(
        "Lost Jitte",
        "Choosing a creature that’s already been declared as a blocker as the target of the second mode of the activated ability won’t cause that creature to stop blocking."
    );
    supported("Lost Jitte");
    let mut t = TestGame::new(2);
    let jitte = t.battlefield(P0, "Lost Jitte");
    t.g.add_counters(obj(jitte), counters::CHARGE, 1, None);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(bears, giant)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(t.g.is_blocking(bears));
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.activate(P0, jitte, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert!(t.g.is_blocking(bears));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(bears));
}

#[test]
fn chromatic_orrery_draws_at_most_five() {
    cr!("105.1", "105.2c");
    ruling!(
        "Chromatic Orrery",
        "Chromatic Orrery's last ability can have you draw at most five cards."
    );
    supported("Chromatic Orrery");
    for (perms, draws) in [
        (
            &[
                "Savannah Lions",
                "Wind Drake",
                "Walking Corpse",
                "Raging Goblin",
                "Grizzly Bears",
                "Fusion Elemental",
                "Ornithopter",
            ][..],
            5,
        ),
        (&["Ornithopter", "Mind Stone"][..], 0),
    ] {
        let mut t = TestGame::new(2);
        let orrery = t.battlefield(P0, "Chromatic Orrery");
        for p in perms {
            t.battlefield(P0, p);
        }
        t.lands(P0, "Wastes", 5);
        let hand = t.hand_size(P0);
        t.activate(P0, orrery, 1, &[]).unwrap();
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + draws);
    }
}

#[test]
fn cavern_hoard_dragon_counts_the_opponent_with_the_most_artifacts() {
    cr!("601.2f", "800.4");
    ruling!(
        "Cavern-Hoard Dragon",
        "Cavern-Hoard Dragon's first ability counts only the artifacts controlled by the opponent who controls the greatest number of artifacts among your opponents."
    );
    ruling!(
        "Cavern-Hoard Dragon",
        "Once a player has announced that they are casting Cavern-Hoard Dragon, no player may take actions"
    );
    supported("Cavern-Hoard Dragon");
    let mut t = TestGame::new(3);
    for _ in 0..2 {
        t.battlefield(P1, "Ornithopter");
    }
    for _ in 0..3 {
        t.battlefield(P2, "Ornithopter");
    }
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 3);
    let dragon = t.hand(P0, "Cavern-Hoard Dragon");
    assert!(t.cast(P0, dragon).try_go().is_err());
    t.lands(P0, "Wastes", 1);
    let from = t.asked().len();
    t.cast(P0, dragon).go();
    // No other player was asked anything while it was being cast.
    assert!(asked_since(&t, from).iter().all(|(p, _)| *p == P0));
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn cavern_hoard_dragon_costs_rr_against_seven_artifacts() {
    cr!("601.2f", "118.7");
    ruling!(
        "Cavern-Hoard Dragon",
        "If the greatest number of artifacts an opponent controls is seven or more, Cavern-Hoard Dragon costs {R}{R} to cast."
    );
    let mut t = TestGame::new(2);
    for _ in 0..8 {
        t.battlefield(P1, "Ornithopter");
    }
    t.lands(P0, "Mountain", 2);
    let dragon = t.hand(P0, "Cavern-Hoard Dragon");
    t.cast(P0, dragon).go();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn cavern_hoard_dragon_counts_artifacts_as_its_trigger_resolves() {
    cr!("608.2h");
    ruling!(
        "Cavern-Hoard Dragon",
        "Cavern-Hoard Dragon's last ability cares about the number of artifacts that player controls as the ability resolves."
    );
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Cavern-Hoard Dragon");
    let a = t.battlefield(P1, "Mind Stone");
    t.battlefield(P1, "Mind Stone");
    t.battlefield(P1, "Mind Stone");
    attack_with(&mut t, &[(dragon, Entity::Player(P1))]);
    crate::r_p130_common::to_combat_damage_triggers(&mut t);
    assert_eq!(t.life(P1), 14);
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 2);
}

#[test]
fn magda_needs_a_dwarf_to_become_tapped() {
    cr!("701.26a", "603.2e");
    ruling!(
        "Magda, Brazen Outlaw",
        "For the triggered ability to trigger, a Dwarf you control has to actually change from untapped to tapped."
    );
    ruling!(
        "Magda, Brazen Outlaw",
        "Magda’s triggered ability doesn’t allow you to tap any Dwarves."
    );
    supported("Magda, Brazen Outlaw");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Magda, Brazen Outlaw");
    let dwarf = t.battlefield(P0, "Dwarven Trader");
    // Nothing lets you tap the Dwarf.
    assert!(!can_activate(&mut t, P0, dwarf));
    assert!(t.g.tap(dwarf));
    t.settle();
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 1);
    // Already tapped: tapping it again doesn't happen and doesn't trigger.
    assert!(!t.g.tap(dwarf));
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(treasures(&t, P0), 1);
}

#[test]
fn glorious_sunrises_draw_mode_checks_on_resolution() {
    cr!("700.2b", "608.2c");
    ruling!(
        "Glorious Sunrise",
        "Glorious Sunrise's third mode may be chosen even if you don't control any creatures with power 3 or greater as the ability goes on the stack."
    );
    supported("Glorious Sunrise");
    for giant in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Glorious Sunrise");
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
        t.advance_to(P0, Step::BeginningOfCombat);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        if giant {
            t.battlefield(P0, "Hill Giant");
        }
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + usize::from(giant));
    }
}

#[test]
fn eldrazi_confluence_is_countered_only_if_all_its_targets_are_illegal() {
    cr!("608.2b", "700.2d");
    ruling!(
        "Eldrazi Confluence",
        "If all targets for the chosen modes become illegal before a Confluence resolves, the spell will be countered and none of its effects will happen."
    );
    supported("Eldrazi Confluence");
    for kill in [1usize, 2] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, "Craw Wurm");
        let b = t.battlefield(P1, "Craw Wurm");
        t.lands(P0, "Wastes", 4);
        let card = t.hand(P0, "Eldrazi Confluence");
        t.cast(P0, card)
            .modes(&[0, 0, 2])
            .target(a)
            .target(b)
            .go();
        destroy(&mut t, a);
        if kill == 2 {
            destroy(&mut t, b);
        }
        t.resolve_all();
        let scions = crate::r_p108_common::tokens_with(&t, P0, "Scion");
        if kill == 2 {
            assert_eq!(scions, 0);
        } else {
            assert_eq!(scions, 1);
            assert_eq!(t.pt(b), (9, 1));
        }
    }
}

#[test]
fn ruby_keeps_her_bonus_if_the_big_creature_leaves() {
    cr!("603.2", "608.2c");
    ruling!(
        "Ruby, Daring Tracker",
        "If you controlled a creature with power 4 or greater when you declared Ruby as an attacker"
    );
    supported("Ruby, Daring Tracker");
    let mut t = TestGame::new(2);
    let ruby = t.battlefield(P0, "Ruby, Daring Tracker");
    let wurm = t.battlefield(P0, "Craw Wurm");
    attack_with(&mut t, &[(ruby, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, wurm);
    t.resolve_all();
    assert_eq!(t.pt(ruby), (3, 4));
}

#[test]
fn khalni_gem_returns_what_lands_you_have() {
    cr!("609.3");
    ruling!(
        "Khalni Gem",
        "If you don’t control any lands by the time Khalni Gem’s triggered ability resolves, nothing happens."
    );
    supported("Khalni Gem");
    for lands in [0usize, 1] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", lands);
        let gem = t.enter(P0, "Khalni Gem");
        t.settle();
        t.resolve_all();
        assert!(t.on_battlefield(gem));
        assert_eq!(t.hand_size(P0), lands);
    }
}

#[test]
fn wall_of_roots_can_pay_mana_and_be_sacrificed_for_the_same_cost() {
    cr!("601.2g", "601.2h", "704.3");
    ruling!(
        "Wall of Roots",
        "If you must sacrifice a creature to pay a casting or activation cost that also includes mana"
    );
    supported("Wall of Roots");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Roots");
    t.g.add_counters(obj(wall), "-0/-1", 4, None);
    t.g.recompute();
    assert_eq!(t.pt(wall), (0, 1));
    let cauldron = t.battlefield(P0, "Bubbling Cauldron");
    t.answer_choose(P0, &[obj(wall)]);
    t.activate(P0, cauldron, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Wall of Roots"));
    assert_eq!(t.life(P0), 24);
}

#[test]
fn herd_heirloom_grant_stays_if_power_drops() {
    cr!("611.2c");
    ruling!(
        "Herd Heirloom",
        "Once Herd Heirloom’s last ability has resolved, lowering the affected creature’s power below 4 won’t cause it to lose those abilities."
    );
    supported("Herd Heirloom");
    let mut t = TestGame::new(2);
    let heirloom = t.battlefield(P0, "Herd Heirloom");
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.activate(P0, heirloom, 1, &[obj(wurm)]).unwrap();
    t.resolve_all();
    t.g.add_counters(obj(wurm), counters::MINUS1, 3, None);
    t.g.recompute();
    assert_eq!(t.pt(wurm), (3, 1));
    let o = t.obj_now(wurm);
    assert!(o.chars.has_keyword(mtg_engine::keywords::KeywordKind::Trample));
    assert!(o.chars.abilities.iter().any(|a| a.text.contains("draw a card")));
}

#[test]
fn pentad_prism_stays_without_charge_counters() {
    cr!("122.1", "704.1");
    ruling!(
        "Pentad Prism",
        "Once Pentad Prism has run out of charge counters, it remains on the battlefield."
    );
    supported("Pentad Prism");
    let mut t = TestGame::new(2);
    let prism = t.battlefield(P0, "Pentad Prism");
    t.g.add_counters(obj(prism), counters::CHARGE, 1, None);
    t.activate(P0, prism, 0, &[]).unwrap();
    t.settle();
    assert_eq!(t.counters(prism, counters::CHARGE), 0);
    assert!(t.on_battlefield(prism));
    assert_eq!(pool_total(&t, P0), 1);
}

#[test]
fn rain_of_filth_grants_only_to_lands_there_on_resolution_even_tapped_ones() {
    cr!("611.2c", "602.1");
    ruling!(
        "Rain of Filth",
        "Only grants the ability to lands you control when this ability resolves."
    );
    ruling!("Rain of Filth", "The granted ability can be used while the land is tapped.");
    supported("Rain of Filth");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    cast_new(&mut t, P0, "Rain of Filth", &[]);
    t.resolve_all();
    let swamp = t.named_on_battlefield("Swamp")[0];
    let later = t.hand(P0, "Plains");
    t.play_land(P0, later).unwrap();
    let granted = |t: &TestGame, l: ObjectId| {
        t.obj_now(l)
            .chars
            .abilities
            .iter()
            .any(|a| a.text.to_lowercase().contains("sacrifice ~: add {b}"))
    };
    assert!(granted(&t, forest) && granted(&t, swamp));
    assert!(!granted(&t, later));
    // The Swamp paid for the spell and is tapped: it can still be sacrificed for {B}.
    assert!(t.obj_now(swamp).tapped);
    t.activate(P0, swamp, 1, &[]).unwrap();
    assert!(!t.on_battlefield(swamp));
    assert_eq!(pool(&t, P0)[2], 1);
}

#[test]
fn mishra_triggers_on_any_artifact_discard() {
    cr!("701.9a", "603.2");
    ruling!(
        "Mishra, Excavation Prodigy",
        "The last ability will trigger if you discard artifact cards for any reason"
    );
    supported("Mishra, Excavation Prodigy");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mishra, Excavation Prodigy");
    let card = t.hand(P0, "Ornithopter");
    t.g.discard(P0, card, None);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(pool(&t, P0)[3], 2);
}

#[test]
fn a_mana_ability_that_costs_putting_a_counter_pays_for_a_spell() {
    cr!("605.3a", "601.2g");
    // Wall of Roots ("Put a -0/-1 counter on this creature: Add {G}. Activate only once each
    // turn.") is a mana source for an automatic payment.
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Roots");
    let elves = t.hand(P0, "Llanowar Elves");
    t.cast(P0, elves).go();
    assert_eq!(t.pt(wall), (0, 4));
    // Only once each turn.
    let more = t.hand(P0, "Llanowar Elves");
    assert!(t.cast(P0, more).try_go().is_err());
}

#[test]
fn knight_of_the_white_orchid_has_an_intervening_if_clause() {
    cr!("603.4");
    ruling!(
        "Knight of the White Orchid",
        "Knight of the White Orchid's triggered ability has an “intervening ‘if' clause.”"
    );
    supported("Knight of the White Orchid");
    // As many lands as the opponent: no trigger.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    t.lands(P1, "Forest", 1);
    t.library_top(P0, "Plains");
    t.enter(P0, "Knight of the White Orchid");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // It triggers, but P0 has caught up by the time it resolves: nothing happens.
    let mut t = TestGame::new(2);
    t.lands(P1, "Forest", 2);
    t.library_top(P0, "Plains");
    t.enter(P0, "Knight of the White Orchid");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.lands(P0, "Plains", 2);
    let library = t.library_size(P0);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.library_size(P0), library);
    assert_eq!(t.named_on_battlefield("Plains").len(), 2);
}

#[test]
fn knight_of_the_white_orchid_can_find_a_nonbasic_plains() {
    cr!("205.3i", "701.23a");
    ruling!(
        "Knight of the White Orchid",
        "The Plains you search for doesn't have to be basic."
    );
    let mut t = TestGame::new(2);
    t.lands(P1, "Forest", 1);
    let foundry = t.library_top(P0, "Sacred Foundry");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(foundry)]);
    t.enter(P0, "Knight of the White Orchid");
    t.settle();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Sacred Foundry").len(), 1);
}

#[test]
fn eldrazi_confluence_repeated_modes_happen_in_the_chosen_order() {
    cr!("700.2d", "608.2c");
    ruling!(
        "Eldrazi Confluence",
        "If the same mode is chosen more than once, you choose their relative order as you cast the spell."
    );
    for a_first in [true, false] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P1, "Hill Giant");
        t.lands(P0, "Wastes", 4);
        let card = t.hand(P0, "Eldrazi Confluence");
        let (first, second) = if a_first { (a, b) } else { (b, a) };
        t.cast(P0, card)
            .modes(&[1, 1, 2])
            .target(first)
            .target(second)
            .go();
        t.resolve_all();
        // Each was exiled and returned (tapped) in turn: the first one chosen came back
        // first, with the earlier timestamp.
        let (f, s) = (t.obj_now(first), t.obj_now(second));
        assert!(f.tapped && s.tapped);
        assert!(f.id != first && s.id != second);
        assert!(f.timestamp < s.timestamp);
    }
}
