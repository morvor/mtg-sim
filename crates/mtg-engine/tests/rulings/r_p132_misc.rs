//! Rulings batch P132 — more regrowth (enchantment, historic, instant, permanent,
//! sorcery) and mass or targeted bounce: what can be targeted (CR 115), historic spells
//! (CR 700.6), random choices on resolution (CR 608.2), piles (CR 700.3), tokens leaving
//! the battlefield (CR 111.7, 704.5d), unattached Auras (CR 704.5m) and spells versus
//! permanents (CR 110.1, 112.1).

use crate::r_p006_common::targets_of;
use crate::r_p079_common::cast_card;
use crate::r_p132_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::target_candidates;
use crate::r_s06_common::attach_new;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn restoration_specialist_can_be_activated_with_no_targets() {
    cr!("601.2c", "602.2b");
    ruling!(
        "Restoration Specialist",
        "You can activate Restoration Specialist’s ability choosing no targets at all if you wish."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let rs = t.battlefield(P0, "Restoration Specialist");
    t.graveyard(P0, "Ornithopter");
    t.answer_targets(P0, &[]);
    t.answer_targets(P0, &[]);
    t.activate(P0, rs, 0, &[]).unwrap();
    assert_eq!(where_is(&t, rs), "graveyard");
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn elena_counts_historic_spells_but_not_lands() {
    cr!("700.6", "305.1", "601.2");
    ruling!(
        "Elena, Turk Recruit",
        "A historic spell is a spell that is legendary, is an artifact, or is a Saga."
    );
    ruling!(
        "Elena, Turk Recruit",
        "Lands are never cast, so abilities that trigger \"whenever you cast a historic spell\" won't trigger if you play a legendary land."
    );
    let mut t = TestGame::new(2);
    let elena = t.battlefield(P0, "Elena, Turk Recruit");
    let counters = |t: &TestGame| t.counters(elena, counters::PLUS1);
    for (name, historic) in [
        ("Ornithopter", true),
        ("Isamaru, Hound of Konda", true),
        ("History of Benalia", true),
        ("Grizzly Bears", false),
    ] {
        let before = counters(&t);
        cast_card(&mut t, P0, name, &[]).unwrap();
        t.resolve_all();
        assert_eq!(counters(&t), before + u32::from(historic), "{name}");
    }
    let land = t.hand(P0, "Mikokoro, Center of the Sea");
    let before = counters(&t);
    t.play_land(P0, land).unwrap();
    flush(&mut t);
    t.resolve_all();
    assert_eq!(counters(&t), before);
}

#[test]
fn surreal_memoir_picks_at_random_as_it_resolves() {
    cr!("608.2", "115.1");
    ruling!(
        "Surreal Memoir",
        "If you have only one instant card in your graveyard as Surreal Memoir resolves, that's the one you'll return to your hand."
    );
    ruling!(
        "Surreal Memoir",
        "Surreal Memoir isn't targeted. You don't choose an instant card at random from your graveyard until it resolves."
    );
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    let spell = cast_card(&mut t, P0, "Surreal Memoir", &[]).unwrap();
    assert!(targets_of(&t, spell).is_empty());
    // The graveyard had no instant as it was cast; one put there afterwards is returned.
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.resolve_all();
    assert_eq!(where_is(&t, bolt), "hand");
}

#[test]
fn split_the_spoils_empty_pile_and_opponent_chosen_on_resolution() {
    cr!("700.3d", "608.2");
    ruling!(
        "Split the Spoils",
        "A pile may have zero cards in it but, if your opponent chooses that pile, you put all the cards back into your graveyard."
    );
    ruling!(
        "Split the Spoils",
        "You decide which opponent chooses the pile while resolving Split the Spoils."
    );
    let mut t = TestGame::new(3);
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Ornithopter");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    let c = t.hand(P0, "Split the Spoils");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Split the Spoils");
    let from = t.asked().len();
    t.cast(P0, c).go();
    let player_choice = |d: &Decision| {
        matches!(d, Decision::ChooseEntities { candidates, .. }
            if candidates.contains(&pl(P1)) && candidates.contains(&pl(P2)))
    };
    assert!(!t.asked()[from..].iter().any(|(_, d)| player_choice(d)));
    // The first pile is empty (everything in the second); P2 is chosen and takes the
    // empty pile.
    t.answer_choose_moved(P0, &[]);
    t.answer_choose(P0, &[pl(P2)]);
    t.answer(P2, DecisionKind::Option, Answer::Index(0));
    let mid = t.asked().len();
    t.resolve_all();
    let asked = &t.asked()[mid..];
    assert!(asked.iter().any(|(p, d)| *p == P0 && player_choice(d)));
    assert!(asked
        .iter()
        .any(|(p, d)| *p == P2 && matches!(d, Decision::ChooseOption { .. })));
    assert_eq!(where_is(&t, a), "graveyard");
    assert_eq!(where_is(&t, b), "graveyard");
}

#[test]
fn creeping_renaissance_offers_the_permanent_types() {
    cr!("110.4", "205.2a");
    ruling!(
        "Creeping Renaissance",
        "The permanent types are artifact, creature, enchantment, land, and planeswalker."
    );
    supported("Creeping Renaissance");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let forest = t.graveyard(P0, "Forest");
    let from = t.asked().len();
    crate::r_s35_common::pick_options(&mut t, P0, &["creature"]);
    cast_card(&mut t, P0, "Creeping Renaissance", &[]).unwrap();
    t.resolve_all();
    let options: Vec<Vec<String>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .collect();
    let opts = &options[0];
    for ty in ["artifact", "creature", "enchantment", "land", "planeswalker"] {
        assert!(opts.iter().any(|o| o == ty), "{ty} in {opts:?}");
    }
    for ty in ["instant", "sorcery", "kindred"] {
        assert!(!opts.iter().any(|o| o == ty), "{ty} in {opts:?}");
    }
    assert_eq!(where_is(&t, bears), "hand");
    assert_eq!(where_is(&t, forest), "graveyard");
}

#[test]
fn a_spell_that_caused_the_trigger_is_in_the_graveyard_to_be_targeted() {
    cr!("608.2n", "603.3d");
    ruling!(
        "Archaeomancer",
        "If an instant or sorcery spell puts Archaeomancer onto the battlefield, its ability can target that card in your graveyard."
    );
    ruling!(
        "Living Lightning",
        "If an instant or sorcery spell you own causes Living Lightning to die, its ability can target that instant or sorcery card in your graveyard."
    );
    // Cloudshift blinks Archaeomancer.
    let mut t = TestGame::new(2);
    let arch = t.battlefield(P0, "Archaeomancer");
    let shift = t.hand(P0, "Cloudshift");
    t.lands(P0, "Plains", 1);
    t.cast(P0, shift).target(obj(arch)).go();
    t.answer_targets(P0, &[obj(shift)]);
    t.resolve_all();
    assert_eq!(where_is(&t, shift), "hand");
    // Shock kills Living Lightning (3/1).
    let mut t = TestGame::new(2);
    let ll = t.battlefield(P0, "Living Lightning");
    let shock = t.hand(P0, "Shock");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, shock).target(obj(ll)).go();
    t.answer_targets(P0, &[obj(shock)]);
    t.resolve_all();
    assert_eq!(where_is(&t, ll), "graveyard");
    assert_eq!(where_is(&t, shock), "hand");
}

#[test]
fn mystic_retrieval_cant_target_itself() {
    cr!("115.5", "601.2c");
    ruling!(
        "Mystic Retrieval",
        "Mystic Retrieval can't target itself. It's on the stack when you choose its target."
    );
    let mut t = TestGame::new(2);
    let mr = t.graveyard(P0, "Mystic Retrieval");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 3);
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(bolt)]);
    let spell = t
        .cast(P0, mr)
        .method(mtg_engine::object::CastMethod::Keyword(
            mtg_engine::keywords::KeywordKind::Flashback,
        ))
        .go();
    let cands = target_candidates(&t, P0, from);
    assert!(cands.iter().all(|c| !c.contains(&obj(spell))
        && !c.contains(&obj(mr))));
    assert!(cands[0].contains(&obj(bolt)));
}

#[test]
fn scholar_of_the_ages_returns_at_most_two_cards() {
    cr!("115.1d", "601.2c");
    ruling!(
        "Scholar of the Ages",
        "The triggered ability of Scholar of the Ages can return at most two cards, not two instant cards plus two sorcery cards."
    );
    let mut t = TestGame::new(2);
    let cards: Vec<ObjectId> = ["Lightning Bolt", "Shock", "Divination", "Mind Rot"]
        .iter()
        .map(|n| t.graveyard(P0, n))
        .collect();
    t.answer_targets(P0, &[obj(cards[0]), obj(cards[3])]);
    t.enter(P0, "Scholar of the Ages");
    t.resolve_all();
    let back = cards.iter().filter(|c| where_is(&t, **c) == "hand").count();
    assert_eq!(back, 2);
    let maxes: Vec<u32> = t
        .asked()
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { max, .. } => Some(*max),
            _ => None,
        })
        .collect();
    assert!(maxes.iter().all(|m| *m <= 2), "{maxes:?}");
}

#[test]
fn devout_harpist_targets_only_auras_attached_to_creatures() {
    cr!("115.1", "303.4");
    ruling!(
        "Devout Harpist",
        "It can only target “Enchantment - Aura” cards, which includes cards changed into Auras, such as Licids."
    );
    let mut t = TestGame::new(2);
    let harpist = t.battlefield(P0, "Devout Harpist");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let forest = t.battlefield(P1, "Forest");
    let on_creature = attach_new(&mut t, P1, "Pacifism", obj(bears));
    let on_land = attach_new(&mut t, P1, "Wild Growth", obj(forest));
    let not_aura = t.battlefield(P1, "Glorious Anthem");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(on_creature)]);
    t.activate(P0, harpist, 0, &[]).unwrap();
    let cands = target_candidates(&t, P0, from);
    assert!(cands[0].contains(&obj(on_creature)));
    assert!(!cands[0].contains(&obj(on_land)));
    assert!(!cands[0].contains(&obj(not_aura)));
    t.resolve_all();
    assert_eq!(where_is(&t, on_creature), "graveyard");
}

#[test]
fn street_sweeper_may_target_a_land_with_no_auras() {
    cr!("603.3d", "115.1");
    ruling!(
        "Street Sweeper",
        "Street Sweeper’s ability is mandatory, but you can choose a land with no Auras attached to it as the target."
    );
    let mut t = TestGame::new(2);
    let sweeper = t.battlefield(P0, "Street Sweeper");
    let forest = t.battlefield(P1, "Forest");
    t.answer_targets(P0, &[obj(forest)]);
    attack_with(&mut t, &[(sweeper, pl(P1))]);
    assert_eq!(t.stack_len(), 1);
    assert_eq!(targets_of(&t, *t.g.stack.last().unwrap()), vec![obj(forest)]);
    t.resolve_all();
    assert!(t.on_battlefield(forest));
}

#[test]
fn inundate_leaves_a_hybrid_blue_creature() {
    cr!("105.2", "202.2");
    ruling!(
        "Inundate",
        "A hybrid creature that’s blue and another color is *not* a nonblue creature."
    );
    let mut t = TestGame::new(2);
    let liege = t.battlefield(P1, "Thistledown Liege");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_card(&mut t, P0, "Inundate", &[]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(liege));
    assert_eq!(where_is(&t, bears), "hand");
}

#[test]
fn blood_clock_payment_options() {
    cr!("119.4", "118.3", "608.2c");
    ruling!(
        "Blood Clock",
        "A player who controls no permanents doesn’t need to pay 2 life (although that player can choose to do so)."
    );
    ruling!(
        "Blood Clock",
        "A player with less than 2 life can’t pay 2 life and must return a permanent."
    );
    // No permanents: declining to pay does nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blood Clock");
    t.set_step(P0, Step::End);
    t.answer_yes(P1, false);
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // At 1 life: can't pay, must return a permanent.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blood Clock");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.players[1].life = 1;
    t.set_step(P0, Step::End);
    t.answer_yes(P1, true);
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 1);
    assert_eq!(where_is(&t, bears), "hand");
}

#[test]
fn displacement_wave_sees_token_mana_values() {
    cr!("202.3a", "707.2", "202.3");
    ruling!(
        "Displacement Wave",
        "A token has mana value 0 unless it's a copy of something else, in which case it has the mana cost of whatever it's copying."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let copy = crate::r_s17_common::token_copy(&mut t, P1, giant)[0];
    let soldier = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    let c = t.hand(P0, "Displacement Wave");
    t.lands(P0, "Island", 4);
    t.cast(P0, c).x(2).go();
    t.resolve_all();
    assert!(!t.g.is_live(soldier) || !t.on_battlefield(soldier));
    assert!(t.on_battlefield(copy));
    assert!(t.on_battlefield(giant));
}

#[test]
fn perplexing_test_returned_tokens_cease_to_exist() {
    cr!("111.7", "704.5d");
    ruling!(
        "Perplexing Test",
        "A token that is returned to its owner's hand ceases to exist as a state-based action the next time a player would receive priority."
    );
    let mut t = TestGame::new(2);
    let soldier = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let c = t.hand(P0, "Perplexing Test");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Perplexing Test");
    t.cast(P0, c).modes(&[0]).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    assert!(t.on_battlefield(bears));
    assert!(!t.on_battlefield(soldier));
}

#[test]
fn ghostly_wings_goes_to_the_graveyard_after_returning_its_creature() {
    cr!("704.5m");
    ruling!(
        "Ghostly Wings",
        "After the last ability of Ghostly Wings resolves, it will be an Aura on the battlefield that isn’t attached to a creature and will immediately be put into its owner’s graveyard."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wings = attach_new(&mut t, P0, "Ghostly Wings", obj(bears));
    t.hand(P0, "Forest");
    t.activate(P0, wings, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(where_is(&t, bears), "hand");
    assert_eq!(where_is(&t, wings), "graveyard");
}

#[test]
fn steel_sabotage_distinguishes_artifact_spells_from_artifacts() {
    cr!("110.1", "112.1", "700.2a");
    ruling!(
        "Steel Sabotage",
        "An artifact spell exists only on the stack. An artifact exists only on the battlefield."
    );
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    t.lands(P0, "Island", 2);
    // No artifact spell: only the bounce mode.
    let c = t.hand(P0, "Steel Sabotage");
    let from = t.asked().len();
    t.cast(P0, c).modes(&[1]).target(obj(thopter)).go();
    let available: Vec<Vec<usize>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseModes { available, .. } => Some(available.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(available, vec![vec![1]]);
    t.resolve_all();
    assert_eq!(where_is(&t, thopter), "hand");
    // An artifact spell on the stack: countered with the first mode; the bounce mode
    // can't target it.
    t.set_step(P1, Step::PrecombatMain);
    let spell = cast_card(&mut t, P1, "Ornithopter", &[]).unwrap();
    let c = t.hand(P0, "Steel Sabotage");
    t.cast(P0, c).modes(&[0]).target(obj(spell)).go();
    t.resolve_all();
    assert_eq!(where_is(&t, spell), "graveyard");
}

/// P0's The Great Synthesis reaches chapter III; its ability is on the stack.
fn synthesis_at_chapter_three(t: &mut TestGame) -> ObjectId {
    let jin = t.battlefield(P0, "Jin-Gitaxias // The Great Synthesis");
    crate::r_s17_common::transform(t, jin);
    let saga = t.g.current(jin);
    t.g.objects[saga.0 as usize]
        .counters
        .insert(counters::LORE.into(), 2);
    crate::r_s19_common::add_lore(t, saga, 1);
    assert_eq!(t.stack_len(), 1);
    saga
}

#[test]
fn great_synthesis_casts_during_resolution_then_returns_before_they_resolve() {
    cr!("714.2b", "608.2g", "601.2");
    ruling!(
        "Jin-Gitaxias // The Great Synthesis",
        "All the spells you cast due to chapter III are cast during the resolution of that ability. You can cast them in any order, and timing restrictions based on the cards’ types are ignored. Once you’re done casting spells, before any of them resolve, The Great Synthesis will be exiled and return as Jin-Gitaxias."
    );
    let mut t = TestGame::new(2);
    // P1's turn: sorcery timing would forbid casting these normally.
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.hand(P0, "Grizzly Bears");
    let div = t.hand(P0, "Divination");
    let saga = synthesis_at_chapter_three(&mut t);
    t.answer_choose(P0, &[obj(div), obj(bears)]);
    t.resolve();
    let names: Vec<String> = t
        .g
        .stack
        .iter()
        .map(|s| t.g.obj(*s).chars.name.to_string())
        .collect();
    assert!(names.contains(&"Grizzly Bears".to_string()), "{names:?}");
    assert!(names.contains(&"Divination".to_string()), "{names:?}");
    let jin = t.named_on_battlefield("Jin-Gitaxias");
    assert_eq!(jin.len(), 1, "back as Jin-Gitaxias before the spells resolve");
    assert_ne!(t.g.current(saga), saga);
}

#[test]
fn great_synthesis_spells_can_target_earlier_spells_but_not_their_permanents() {
    cr!("714.2b", "115.1", "303.4a");
    ruling!(
        "Jin-Gitaxias // The Great Synthesis",
        "A spell you cast this way can be the target of a later spell you cast this way. However, permanent spells cast this way won’t resolve until you’re done casting spells, so the permanents they become can’t be the target of spells cast this way."
    );
    let mut t = TestGame::new(2);
    let strike = t.hand(P0, "Lightning Strike");
    let twincast = t.hand(P0, "Twincast");
    let bears = t.hand(P0, "Grizzly Bears");
    let pacifism = t.hand(P0, "Pacifism");
    synthesis_at_chapter_three(&mut t);
    t.answer_choose(
        P0,
        &[obj(strike), obj(twincast), obj(bears), obj(pacifism)],
    );
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve();
    t.resolve_all();
    assert_eq!(t.life(P1), 14, "Lightning Strike and its copy");
    assert_eq!(where_is(&t, bears), "battlefield");
    assert_eq!(where_is(&t, pacifism), "hand", "no creature to enchant while casting");
}
