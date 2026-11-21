//! Rulings batch P124 — the rest of the batch's cards: the commander "Will" spells (CR
//! 700.2, 601.2b, 903.3), modes in listed order (CR 700.2g), keyword-copying abilities
//! and their variants (Odric, Bleeding Effect, Kathril), Linvala's full-party abilities
//! (CR 603.4, 800.4m, 602.5), and other abilities that work as written.

use crate::r_p124_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters::PLUS1;
use mtg_engine::*;

/// Makes `id` its owner's commander (the designation of its card, CR 903.3).
fn make_commander(t: &mut TestGame, id: ObjectId) {
    t.g.objects[id.0 as usize].is_commander = true;
    let owner = t.obj(id).owner;
    let name = t.obj(id).card.as_ref().unwrap().name.clone();
    t.g.players[owner.idx()].commander_names.push(name);
}

/// The hexproof keywords the creature (followed across zone changes) has now: for each,
/// whether it's "hexproof from [quality]" (has a filter).
fn hexproofs(t: &mut TestGame, id: ObjectId) -> Vec<bool> {
    t.g.recompute();
    t.obj_now(id)
        .chars
        .keywords_of(KeywordKind::Hexproof)
        .map(|k| k.filter.is_some())
        .collect()
}

#[test]
fn the_wills_any_commander_lets_you_choose_both_modes() {
    cr!("700.2", "601.2b", "903.3");
    ruling!(
        "Akroma's Will",
        "Even though the card is named after a specific character, controlling any commander will satisfy its condition."
    );
    supported("Akroma's Will");
    // Grizzly Bears is P0's commander. Akroma's Will: "• Creatures you control gain
    // flying, vigilance, and double strike until end of turn. • Creatures you control
    // gain lifelink, indestructible, and protection from each color until end of turn."
    for commander in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        if commander {
            make_commander(&mut t, bears);
        }
        lands_for_cost(&mut t, P0, "Akroma's Will");
        let will = t.hand(P0, "Akroma's Will");
        // Without a commander, the answer naming both modes isn't valid: one mode only.
        let spell = t.cast(P0, will).modes(&[0, 1]).go();
        let modes = t.obj(spell).stack.as_ref().unwrap().chosen.len();
        assert_eq!(modes, if commander { 2 } else { 1 }, "commander: {commander}");
        t.resolve_all();
        let both = has_kw(&mut t, bears, KeywordKind::Flying)
            && has_kw(&mut t, bears, KeywordKind::Lifelink);
        assert_eq!(both, commander);
    }
}

#[test]
fn the_wills_modes_are_locked_in_once_chosen() {
    cr!("601.2b", "601.2g", "601.2h", "605.3a");
    ruling!(
        "Akroma's Will",
        "Once you've chosen both modes for the spell, it doesn't matter whether you continue to control a commander. This is true even if you somehow no longer control a commander as you finish casting the spell."
    );
    ruling!(
        "Akroma's Will",
        "Once you've announced that you're casting a spell, players can't take any actions until you've finished doing so."
    );
    // P0's commander is Blood Pet ("Sacrifice this creature: Add {B}."): P0 chooses both
    // modes, then sacrifices it for mana to pay for the spell. No other player is asked
    // anything while P0 casts it.
    supported("Blood Pet");
    let mut t = TestGame::new(2);
    let pet = t.battlefield(P0, "Blood Pet");
    make_commander(&mut t, pet);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    let will = t.hand(P0, "Akroma's Will");
    let from = t.asked().len();
    t.cast(P0, will).modes(&[0, 1]).go();
    assert!(!t.on_battlefield(pet), "Blood Pet paid for it");
    assert!(t.asked()[from..].iter().all(|(p, _)| *p == P0));
    t.resolve_all();
    assert!(has_kw(&mut t, bears, KeywordKind::Flying));
    assert!(has_kw(&mut t, bears, KeywordKind::Lifelink));
}

#[test]
fn modes_happen_in_the_listed_order() {
    cr!("608.2c", "613.4b");
    ruling!(
        "Kamahl's Will",
        "If you choose both modes, the effects happen in the listed order. The lands will become 1/1 creatures in time to deal damage to the target of the second mode."
    );
    ruling!(
        "Storvald, Frost Giant Jarl",
        "If you choose both modes, the effects will happen in the listed order."
    );
    supported("Kamahl's Will");
    supported("Storvald, Frost Giant Jarl");
    // Kamahl's Will with a 2/2 commander: two Forests become 1/1s, then each creature P0
    // controls deals damage equal to its power to a 4/4: 2 + 1 + 1 kills it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    make_commander(&mut t, bears);
    let forests = t.lands(P0, "Forest", 2);
    let target = t.battlefield(P1, "Grizzly Bears");
    modify_until_eot(
        &mut t,
        target,
        vec![mtg_engine::ability::Modification::ModifyPT(
            mtg_engine::ability::Value::Const(2),
            mtg_engine::ability::Value::Const(2),
        )],
    );
    lands_for_cost(&mut t, P0, "Kamahl's Will");
    let will = t.hand(P0, "Kamahl's Will");
    t.answer_targets(P0, &[obj(forests[0]), obj(forests[1])]);
    t.answer_targets(P0, &[obj(target)]);
    t.cast(P0, will).modes(&[0, 1]).go();
    t.resolve_all();
    assert!(!t.on_battlefield(target));
    // Storvald entering, both modes on the same Hill Giant: 7/7, then 1/1.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0, 1]));
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_targets(P0, &[obj(giant)]);
    t.enter(P0, "Storvald, Frost Giant Jarl");
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.pt(giant), (1, 1));
}

#[test]
fn odric_and_bleeding_effect_grant_the_specific_variants() {
    cr!("702.11e", "702.11b", "613.1f");
    ruling!(
        "Odric, Lunarch Marshal",
        "If one of those creatures has one or more variants of the listed keywords (for example, hexproof from white), creatures you control gain those specific variants."
    );
    ruling!(
        "Bleeding Effect",
        "If one of the creature cards in your graveyard has one or more variants of the listed keywords (for example, hexproof from white), creatures you control gain those specific variants."
    );
    supported("Odric, Lunarch Marshal");
    supported("Bleeding Effect");
    // Knight of Grace: "First strike. Hexproof from black."
    for card in ["Odric, Lunarch Marshal", "Bleeding Effect"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, card);
        if card == "Bleeding Effect" {
            t.graveyard(P0, "Knight of Grace");
        } else {
            t.battlefield(P0, "Knight of Grace");
        }
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.advance_to(P0, Step::BeginningOfCombat);
        t.resolve_all();
        assert!(has_kw(&mut t, bears, KeywordKind::FirstStrike), "{card}");
        assert_eq!(hexproofs(&mut t, bears), vec![true], "{card}: hexproof from black");
    }
}

#[test]
fn odric_triggers_each_combat_and_counts_on_resolution() {
    cr!("603.2", "608.2h", "507.1");
    ruling!(
        "Odric, Lunarch Marshal",
        "Odric's ability triggers at the beginning of each combat, not just combat on your turn, whether or not any creatures you control have any of the listed abilities. If a creature gains one of the listed abilities before Odric's triggered ability resolves"
    );
    // On P1's turn, with no keywords around, it triggers; P0 gives the Bears flying in
    // response, and all P0's creatures gain flying.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Odric, Lunarch Marshal");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    t.advance_to(P1, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    modify_until_eot(
        &mut t,
        bears,
        vec![mtg_engine::ability::Modification::AddKeyword(
            mtg_engine::keywords::Keyword::new(KeywordKind::Flying),
        )],
    );
    t.resolve_all();
    assert!(has_kw(&mut t, giant, KeywordKind::Flying));
}

#[test]
fn kathril_resolves_all_at_once() {
    cr!("608.2c", "122.1b");
    ruling!(
        "Kathril, Aspect Warper",
        "Once Kathril's ability begins resolving, no player may take other actions until it's done and you've put +1/+1 counters on Kathril."
    );
    supported("Kathril, Aspect Warper");
    // Serra Angel (flying, vigilance) in the graveyard: a flying and a vigilance counter
    // on the Bears, then two +1/+1 counters on Kathril, all in one resolution.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Serra Angel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_choose(P0, &[obj(bears)]);
    let kathril = t.enter(P0, "Kathril, Aspect Warper");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    let counters_now = (t.counters(kathril, PLUS1), t.stack_len());
    assert_eq!(counters_now, (2, 0));
    assert_eq!((t.counters(bears, "flying"), t.counters(bears, "vigilance")), (1, 1));
    assert!(has_kw(&mut t, bears, KeywordKind::Flying));
    assert!(has_kw(&mut t, bears, KeywordKind::Vigilance));
}

/// P0 with Linvala, Shield of Sea Gate (an Angel Wizard) and three changelings: a full
/// party. Returns the changelings.
fn linvala_party(t: &mut TestGame) -> Vec<ObjectId> {
    supported("Linvala, Shield of Sea Gate");
    t.battlefield(P0, "Linvala, Shield of Sea Gate");
    (0..3)
        .map(|_| t.battlefield(P0, "Woodland Changeling"))
        .collect()
}

#[test]
fn linvala_full_party_is_checked_as_it_triggers_and_resolves() {
    cr!("603.4", "700.8");
    ruling!(
        "Linvala, Shield of Sea Gate",
        "Linvala’s middle ability won’t trigger at all if you don’t have a full party as your combat phase begins. It won’t affect the target if you don’t have a full party as it resolves."
    );
    ruling!(
        "Linvala, Shield of Sea Gate",
        "No abilities of the affected permanent can be activated, including mana abilities."
    );
    // Not a full party: no trigger.
    let mut t = TestGame::new(2);
    let party = linvala_party(&mut t);
    exile_now(&mut t, party[0]);
    t.battlefield(P1, "Llanowar Elves");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // A full party as it triggers, but not as it resolves: no effect.
    let mut t = TestGame::new(2);
    let party = linvala_party(&mut t);
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.answer_targets(P0, &[obj(elves)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    exile_now(&mut t, party[0]);
    t.resolve_all();
    assert!(t.activate(P1, elves, 0, &[]).is_ok());
    // A full party throughout: not even the Elves' mana ability can be activated.
    let mut t = TestGame::new(2);
    linvala_party(&mut t);
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.answer_targets(P0, &[obj(elves)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(t.activate(P1, elves, 0, &[]).is_err());
}

#[test]
fn linvala_effect_lasts_until_your_next_turn_would_have_begun() {
    cr!("800.4m", "800.4a");
    ruling!(
        "Linvala, Shield of Sea Gate",
        "In a multiplayer game, if you leave the game after Linvala’s middle ability resolves but before your next turn begins, its effect lasts until your next turn would have begun."
    );
    let mut t = TestGame::new(3);
    linvala_party(&mut t);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    crate::r_p053_common::concede(&mut t, P0);
    assert!(t.has_lost(P0));
    t.advance_to(P1, Step::BeginningOfCombat);
    assert!(!crate::r_s02_common::can_attack(&mut t, bears), "P1's turn");
    t.advance_to(P2, Step::BeginningOfCombat);
    assert!(!can_block(&mut t, bears), "P2's turn");
    // P0's turn would have begun after P2's: P1's next turn, it can attack.
    t.advance_to(P1, Step::BeginningOfCombat);
    assert!(crate::r_s02_common::can_attack(&mut t, bears));
}

#[test]
fn cards_that_work_as_written() {
    cr!("614.1c", "122.1");
    ruling!(
        "Dragon's Disciple",
        "If you choose to reveal a Dragon card from your hand and also control a Dragon, Dragon's Disciple still only gets a single +1/+1 counter."
    );
    ruling!(
        "Call the Spirit Dragons",
        "If you control a Dragon that is more than one color, you may put more than one +1/+1 counter on it with the last ability of Call the Spirit Dragons."
    );
    ruling!(
        "Eldrazi Monument",
        "If you control any creatures as the triggered ability resolves, you must sacrifice one. You can’t choose to sacrifice Eldrazi Monument instead."
    );
    ruling!(
        "Invasion of Pyrulea // Gargantuan Slabhorn",
        "If you reveal a land or double-faced card for Invasion of Pyrulea’s ability, that revealed card will be the one you draw."
    );
    // Dragon's Disciple: P0 controls Shivan Dragon and reveals another from hand.
    supported("Dragon's Disciple");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shivan Dragon");
    let dragon = t.hand(P0, "Shivan Dragon");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(dragon)]);
    let disciple = t.enter(P0, "Dragon's Disciple");
    t.settle();
    assert_eq!(t.counters(disciple, PLUS1), 1);
    // Call the Spirit Dragons with only Dragonlord Ojutai (white and blue): two counters.
    supported("Call the Spirit Dragons");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Call the Spirit Dragons");
    let ojutai = t.battlefield(P0, "Dragonlord Ojutai");
    to_next_upkeep(&mut t);
    t.resolve_all();
    assert_eq!(t.counters(ojutai, PLUS1), 2);
    // Eldrazi Monument: with a creature, the creature is sacrificed (even when P0 tries
    // to choose the Monument); with none, the Monument.
    supported("Eldrazi Monument");
    for creature in [true, false] {
        let mut t = TestGame::new(2);
        let monument = t.battlefield(P0, "Eldrazi Monument");
        let bears = creature.then(|| t.battlefield(P0, "Grizzly Bears"));
        t.answer_choose(P0, &[obj(monument)]);
        to_next_upkeep(&mut t);
        t.resolve_all();
        assert_eq!(t.on_battlefield(monument), creature, "creature: {creature}");
        if let Some(b) = bears {
            assert!(!t.on_battlefield(b));
        }
    }
    // Invasion of Pyrulea: a land on top after scrying is the card drawn.
    supported("Invasion of Pyrulea // Gargantuan Slabhorn");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Forest");
    let hand = t.hand_size(P0);
    t.enter(P0, "Invasion of Pyrulea // Gargantuan Slabhorn");
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.in_hand(P0, "Forest"));
}

#[test]
fn onakke_oathkeeper_doesnt_tax_attacks_on_a_teammates_planeswalkers() {
    cr!("810.7", "805.10b", "508.1h");
    ruling!(
        "Onakke Oathkeeper",
        "In a Two-Headed Giant game, creatures can attack planeswalkers your teammate controls without requiring a mana payment."
    );
    supported("Onakke Oathkeeper");
    // P0 and P1 are teammates; P2 attacks P1's Jace Beleren with no mana.
    let mut t = two_headed_giant();
    t.battlefield(P0, "Onakke Oathkeeper");
    let jace = t.battlefield(P1, "Jace Beleren");
    let giant = t.battlefield(P2, "Hill Giant");
    t.set_step(P2, Step::BeginningOfCombat);
    t.attack(&[(giant, obj(jace))], &[]);
    assert!(!t.on_battlefield(jace));
}

