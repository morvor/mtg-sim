//! Characteristic-changing grammar, the "other" family: life paid as a permanent entered
//! (CR 607.2g, 614.12a), {X} defined by an activated ability's text (CR 107.3c),
//! conditional characteristic-defining power and toughness (CR 604.3), counting an
//! object's types, and "you lose all but 1 life".

use crate::basic_effects_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn minion_of_the_wastes_is_as_big_as_the_life_paid_as_it_entered() {
    cr!("119.4", "614.12a", "604.3");
    ruling!("Minion of the Wastes", "The life is paid as Minion of the Wastes enters");
    assert_supported("Minion of the Wastes");
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Number, Answer::Number(7));
    let minion = t.enter(P0, "Minion of the Wastes");
    assert_eq!(t.life(P0), 13);
    assert_eq!(t.pt(minion), (7, 7));
    // No more than your life total.
    let mut t = TestGame::new(2);
    t.g.players[0].life = 3;
    t.answer(P0, DecisionKind::Number, Answer::Number(3));
    let minion = t.enter(P0, "Minion of the Wastes");
    assert!(t.asked().iter().any(|(p, d)| *p == P0
        && matches!(d, decision::Decision::ChooseNumber { max: 3, .. })));
    assert_eq!(t.life(P0), 0);
    assert_eq!(t.pt(minion), (3, 3));
}

#[test]
fn nameless_race_can_pay_only_up_to_the_white_count() {
    cr!("119.4", "614.12a");
    ruling!("Nameless Race", "The life payment is made just as the card is being put onto the battlefield");
    assert_supported("Nameless Race");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Savannah Lions");
    t.graveyard(P1, "Savannah Lions");
    t.graveyard(P0, "Savannah Lions");
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    let race = t.enter(P0, "Nameless Race");
    // At most 2: one white permanent and one white card in an opponent's graveyard.
    assert!(t.asked().iter().any(|(p, d)| *p == P0
        && matches!(d, decision::Decision::ChooseNumber { max: 2, .. })));
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.pt(race), (2, 2));
}

#[test]
fn phyrexian_processor_makes_minions_the_size_of_the_life_paid() {
    cr!("607.2g", "107.3c");
    ruling!("Phyrexian Processor", "You can pay 0 life if you want");
    assert_supported("Phyrexian Processor");
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Number, Answer::Number(4));
    let processor = t.enter(P0, "Phyrexian Processor");
    assert_eq!(t.life(P0), 16);
    t.lands(P0, "Wastes", 4);
    t.activate(P0, processor, 0, &[]).unwrap();
    t.resolve();
    let minion: Vec<ObjectId> = t
        .g
        .battlefield
        .iter()
        .copied()
        .filter(|o| t.obj_now(*o).chars.has_subtype("Minion"))
        .collect();
    assert_eq!(minion.len(), 1, "{}", t.dump_log());
    assert_eq!(t.pt(minion[0]), (4, 4));
}

#[test]
fn soul_foundrys_x_is_the_mana_value_of_the_exiled_card() {
    cr!("107.3c", "602.2b");
    ruling!("Prototype Portal", "You don't choose the value of {X}");
    assert_supported("Soul Foundry");
    assert_supported("Prototype Portal");
    let mut t = TestGame::new(2);
    let wurm = t.hand(P0, "Craw Wurm");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    let foundry = t.enter(P0, "Soul Foundry");
    t.resolve_all();
    assert!(t.in_exile("Craw Wurm"));
    // X is 6: with five lands it can't be activated, with six it can (and any X the
    // player tries to announce is 6).
    t.lands(P0, "Forest", 5);
    assert!(t.activate(P0, foundry, 0, &[]).is_err());
    t.lands(P0, "Forest", 1);
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    t.activate(P0, foundry, 0, &[]).unwrap();
    t.resolve();
    let tokens = t.named_on_battlefield("Craw Wurm");
    assert_eq!(tokens.len(), 1);
    assert!(t.obj_now(tokens[0]).is_token());
}

#[test]
fn gaeas_liege_counts_forests_of_the_defending_player_while_attacking() {
    cr!("604.3", "613.4a");
    ruling!("Gaea's Liege", "use the \"not attacking\" power and toughness");
    assert_supported("Gaea's Liege");
    let mut t = TestGame::new(2);
    let liege = t.battlefield(P0, "Gaea's Liege");
    t.lands(P0, "Forest", 3);
    t.lands(P1, "Forest", 5);
    t.g.recompute();
    assert_eq!(t.pt(liege), (3, 3));
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(liege, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    assert_eq!(t.pt(liege), (5, 5));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 15);
}

#[test]
fn diligent_zookeeper_counts_creature_types_up_to_ten() {
    cr!("613.4c", "702.73a");
    assert_supported("Diligent Zookeeper");
    let mut t = TestGame::new(2);
    let keeper = t.battlefield(P0, "Diligent Zookeeper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let krasis = t.battlefield(P0, "Unruly Krasis");
    let changeling = t.battlefield(P0, "Changeling Outcast");
    let human = t.battlefield(P0, "Elite Vanguard");
    assert_eq!(t.pt(bears), (3, 3));
    // Shark Octopus Lizard.
    assert_eq!(t.pt(krasis), (7, 7));
    // Changeling: every creature type, Human included, so it isn't affected.
    assert_eq!(t.pt(changeling), (1, 1));
    assert_eq!(t.pt(human), (2, 1));
    assert_eq!(t.pt(keeper), (4, 4));
}

#[test]
fn embiggen_counts_supertypes_card_types_and_subtypes_as_it_resolves() {
    cr!("205.2a", "205.3a", "205.4a", "608.2h");
    ruling!("Embiggen", "The bonus is calculated as Embiggen resolves");
    ruling!("Embiggen", "Token is not a type");
    assert_supported("Embiggen");
    let mut t = TestGame::new(2);
    // Legendary Artifact Creature — Phyrexian Mite: 1 + 2 + 2 = 5.
    let skrelv = t.battlefield(P0, "Skrelv, Defector Mite");
    let spell = t.hand(P0, "Embiggen");
    t.lands(P0, "Forest", 1);
    t.cast(P0, spell).target(skrelv).go();
    t.resolve();
    assert_eq!(t.pt(skrelv), (6, 6));
}

#[test]
fn soulgorger_orgg_gives_back_the_life_lost_when_it_entered() {
    cr!("119.3", "607.2e");
    assert_supported("Soulgorger Orgg");
    let mut t = TestGame::new(2);
    let orgg = t.enter(P0, "Soulgorger Orgg");
    t.resolve_all();
    assert_eq!(t.life(P0), 1);
    t.g.players[0].life = 4;
    t.g.destroy(orgg, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}

#[test]
fn brine_hag_shrinks_the_creatures_that_dealt_damage_to_it() {
    cr!("613.4b", "603.10a");
    assert_supported("Brine Hag");
    let mut t = TestGame::new(2);
    let hag = t.battlefield(P1, "Brine Hag");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(wurm, Entity::Player(P1))]),
    );
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(hag, wurm)]));
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(!t.on_battlefield(hag));
    // The Wurm became 0/2 with the Hag's 2 damage marked on it: lethal damage.
    assert!(t.in_graveyard(P0, "Craw Wurm"), "{}", t.dump_log());
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn xathrid_gorgon_petrifies_a_creature() {
    cr!("602.5a", "205.1b", "105.3");
    assert_supported("Xathrid Gorgon");
    let mut t = TestGame::new(2);
    let gorgon = t.battlefield(P0, "Xathrid Gorgon");
    let elf = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Swamp", 3);
    t.activate(P0, gorgon, 0, &[Entity::Object(elf)]).unwrap();
    t.resolve();
    let c = &t.obj_now(elf).chars;
    assert!(c.card_types.contains(types::CardType::Artifact) && c.is_creature());
    assert_eq!(c.colors, types::ColorSet::NONE);
    assert!(c.has_keyword(keywords::KeywordKind::Defender));
    assert_eq!(t.counters(elf, "petrification"), 1);
    // Its mana ability can't be activated either.
    t.g.turn.priority = Some(P1);
    assert!(t.activate(P1, elf, 0, &[]).is_err());
}

#[test]
fn cycle_of_life_targets_only_a_creature_you_cast_this_turn() {
    cr!("601.2c", "613.4b");
    assert_supported("Cycle of Life");
    let mut t = TestGame::new(2);
    let cycle = t.battlefield(P0, "Cycle of Life");
    let old = t.battlefield(P0, "Grizzly Bears");
    let bear = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    t.cast(P0, bear).go();
    t.resolve();
    let new = *t
        .named_on_battlefield("Grizzly Bears")
        .iter()
        .find(|b| **b != old)
        .expect("the cast Bears");
    t.answer_targets(P0, &[Entity::Object(old)]);
    t.activate(P0, cycle, 0, &[Entity::Object(new)]).unwrap();
    t.resolve();
    assert_eq!(t.pt(new), (0, 1));
    assert_eq!(t.pt(old), (2, 2));
    assert!(t.in_hand(P0, "Cycle of Life"));
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.pt(new), (3, 3));
}
