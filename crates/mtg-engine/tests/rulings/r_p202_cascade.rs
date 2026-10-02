//! Rulings batch P202 — cascade (CR 702.85): "When you cast this spell, exile cards from the
//! top of your library until you exile a nonland card whose mana value is less than this
//! spell's mana value. You may cast that card without paying its mana cost ..."

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Names of the spells on the stack, bottom first.
fn spell_names(t: &TestGame) -> Vec<String> {
    t.g.stack
        .iter()
        .filter(|s| t.g.obj(**s).is_spell())
        .map(|s| t.g.obj(*s).chars.name.to_string())
        .collect()
}

/// The name of the topmost spell on the stack.
fn top_name(t: &TestGame) -> String {
    spell_names(t).pop().expect("no spell on the stack")
}

/// The text of the top triggered ability of the stack, or the top spell's name.
fn top_text(t: &TestGame) -> String {
    let top = *t.g.stack.last().expect("empty stack");
    match t.g.obj(top).stack.as_ref().map(|si| &si.kind) {
        Some(mtg_engine::object::StackKind::Triggered { ability, .. }) => ability.text.to_string(),
        _ => t.g.obj(top).chars.name.to_string(),
    }
}

/// Casts the real card `name` from P0's hand (with the mana for it).
fn cast_from_hand(t: &mut TestGame, name: &str) -> ObjectId {
    give_mana_for(t, P0, name);
    let c = t.hand(P0, name);
    t.cast(P0, c).go()
}

#[test]
fn apex_devastators_cascades_resolve_one_after_another() {
    cr!("702.85a", "702.85c");
    ruling!(
        "Apex Devastator",
        "Each instance of cascade triggers and resolves separately. The spell you cast due to the first cascade ability will go on the stack on top of the second, third, and fourth cascade abilities. That spell will resolve before you exile cards for the second cascade ability."
    );
    supported("Apex Devastator");
    let mut t = TestGame::new(2);
    stack_library(
        &mut t,
        P0,
        &[
            "Hill Giant",
            "Grizzly Bears",
            "Llanowar Elves",
            "Centaur Courser",
        ],
    );
    cast_from_hand(&mut t, "Apex Devastator");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Cascade"), 4);
    for (i, name) in ["Hill Giant", "Grizzly Bears", "Llanowar Elves", "Centaur Courser"]
        .iter()
        .enumerate()
    {
        t.resolve();
        assert_eq!(top_name(&t), *name);
        assert_eq!(triggers_on_stack(&t, "Cascade"), 3 - i);
        t.resolve();
        assert_eq!(t.named_on_battlefield(name).len(), 1);
    }
    assert_eq!(spell_names(&t), vec!["Apex Devastator"]);
}

#[test]
fn each_of_apex_devastators_cascades_uses_its_own_mana_value() {
    cr!("702.85a", "702.85c");
    ruling!(
        "Apex Devastator",
        "Each of Apex Devastator's four cascade abilities will look for a nonland card with mana value less than 10 (Apex Devastator's mana value). This doesn't change even if one or more of the spells you cast because of those cascade abilities has cascade itself."
    );
    let mut t = TestGame::new(2);
    // First cascade: Enlisted Wurm (6), which cascades into the Bears (2 < 6). Second
    // cascade: the Dreadmaw (6 < 10), which the Wurm's cascade would have skipped.
    stack_library(
        &mut t,
        P0,
        &["Enlisted Wurm", "Grizzly Bears", "Colossal Dreadmaw"],
    );
    cast_from_hand(&mut t, "Apex Devastator");
    t.resolve();
    assert_eq!(top_name(&t), "Enlisted Wurm");
    t.resolve(); // the Wurm's cascade
    assert_eq!(top_name(&t), "Grizzly Bears");
    t.resolve();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Enlisted Wurm").len(), 1);
    t.resolve(); // Apex's second cascade
    assert_eq!(top_name(&t), "Colossal Dreadmaw");
}

#[test]
fn call_forth_the_tempests_second_cascade_still_uses_eight() {
    cr!("702.85a", "702.85c");
    ruling!(
        "Call Forth the Tempest",
        "No matter what spell you cast with the first cascade ability (or with any cascade abilities that result from casting that spell), the second cascade ability will look for a card with mana value less than Call Forth the Tempest's mana value of 8."
    );
    let mut t = TestGame::new(2);
    stack_library(
        &mut t,
        P0,
        &["Enlisted Wurm", "Grizzly Bears", "Myr Battlesphere"],
    );
    cast_from_hand(&mut t, "Call Forth the Tempest");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Cascade"), 2);
    t.resolve();
    assert_eq!(top_name(&t), "Enlisted Wurm");
    t.resolve();
    assert_eq!(top_name(&t), "Grizzly Bears");
    t.resolve();
    t.resolve();
    t.resolve(); // the second cascade: Myr Battlesphere (7 < 8)
    assert_eq!(top_name(&t), "Myr Battlesphere");
}

#[test]
fn a_second_first_sliver_has_two_cascades() {
    cr!("702.85a", "702.85c");
    ruling!(
        "The First Sliver",
        "If a spell has multiple instances of cascade (for example, if you cast a second The First Sliver), each instance of cascade triggers and resolves separately. The spell you cast due to the first cascade ability will go on the stack on top of the second cascade ability. That spell will resolve before you exile cards for the second cascade ability."
    );
    supported("The First Sliver");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The First Sliver");
    stack_library(&mut t, P0, &["Hill Giant", "Grizzly Bears"]);
    cast_from_hand(&mut t, "The First Sliver");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Cascade"), 2);
    t.resolve();
    assert_eq!(top_name(&t), "Hill Giant");
    assert_eq!(triggers_on_stack(&t, "Cascade"), 1);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    t.resolve();
    assert_eq!(top_name(&t), "Grizzly Bears");
}

#[test]
fn a_sliver_cast_by_the_first_slivers_own_cascade_has_no_cascade() {
    cr!("702.85a", "611.3a");
    ruling!(
        "The First Sliver",
        "The First Sliver's last ability only applies while it's on the battlefield. If The First Sliver's own cascade ability lets you cast another Sliver card, that Sliver won't have cascade."
    );
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Muscle Sliver", "Grizzly Bears"]);
    cast_from_hand(&mut t, "The First Sliver");
    t.resolve();
    assert_eq!(top_name(&t), "Muscle Sliver");
    let top = *t.g.stack.last().unwrap();
    assert!(!t.obj(top).chars.has_keyword(KeywordKind::Cascade));
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Cascade"), 0);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Muscle Sliver").len(), 1);
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn sakashimas_protege_can_copy_the_permanent_its_cascade_cast() {
    cr!("702.85a", "707.9");
    ruling!(
        "Sakashima's Protege",
        "If you cast a permanent spell using the cascade ability of Sakashima's Protege, that permanent will be on the battlefield in time to be copied by Sakashima's Protege."
    );
    supported("Sakashima's Protege");
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Hill Giant"]);
    cast_from_hand(&mut t, "Sakashima's Protege");
    t.resolve();
    assert_eq!(top_name(&t), "Hill Giant");
    t.resolve();
    let giant = t.named_on_battlefield("Hill Giant")[0];
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 2);
}

#[test]
fn imoti_lost_while_casting_means_no_cascade() {
    cr!("702.85a", "601.2i", "611.3a");
    ruling!(
        "Imoti, Celebrant of Bounty",
        "In some unusual situations, you may lose control of Imoti during the process of casting a spell with mana value 6 or greater. In those situations, that spell won't have cascade."
    );
    supported("Imoti, Celebrant of Bounty");
    supported("Elder Deep-Fiend");
    let mut t = TestGame::new(2);
    let imoti = t.battlefield(P0, "Imoti, Celebrant of Bounty");
    stack_library(&mut t, P0, &["Grizzly Bears"]);
    // Elder Deep-Fiend (mana value 7) cast for its emerge cost, sacrificing Imoti.
    t.lands(P0, "Island", 5);
    let fiend = t.hand(P0, "Elder Deep-Fiend");
    t.answer_choose(P0, &[Entity::Object(imoti)]);
    t.cast(P0, fiend)
        .method(CastMethod::Keyword(KeywordKind::Emerge))
        .go();
    assert!(t.in_graveyard(P0, "Imoti, Celebrant of Bounty"));
    let top = *t.g.stack.last().unwrap();
    assert!(!t.obj(top).chars.has_keyword(KeywordKind::Cascade));
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Cascade"), 0);
}

#[test]
fn imoti_leaving_after_the_spell_was_cast_doesnt_stop_its_cascade() {
    cr!("702.85a", "601.2i", "603.2");
    ruling!(
        "Imoti, Celebrant of Bounty",
        "Once you announce that you're casting a spell, players can't take actions until you've finished casting it. Causing Imoti, Celebrant of Bounty to leave the battlefield after a spell with mana value 6 or greater has been cast won't stop that spell's cascade ability from resolving."
    );
    let mut t = TestGame::new(2);
    let imoti = t.battlefield(P0, "Imoti, Celebrant of Bounty");
    stack_library(&mut t, P0, &["Grizzly Bears"]);
    cast_from_hand(&mut t, "Colossal Dreadmaw");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Cascade"), 1);
    destroy(&mut t, imoti);
    t.resolve();
    assert_eq!(top_name(&t), "Grizzly Bears");
}

#[test]
fn quandrix_cascade_cant_cast_a_face_with_a_greater_mana_value() {
    cr!("702.85a");
    ruling!(
        "Quandrix, the Proof",
        "Not only do you stop exiling cards if you exile a nonland card with lesser mana value than the spell with cascade, but the resulting spell you cast must also have lesser mana value."
    );
    supported("Quandrix, the Proof");
    supported("Selfless Glyphweaver");
    // Selfless Glyphweaver ({2}{W}, 3) // Deadly Vanity ({5}{B}{B}, 7): only the
    // Glyphweaver can be cast (7 isn't less than Quandrix's 6).
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Selfless Glyphweaver"]);
    let from = t.asked().len();
    cast_from_hand(&mut t, "Quandrix, the Proof");
    t.resolve();
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseOption { options, .. } if options.iter().any(|o| o.contains("Deadly Vanity")))));
    assert_eq!(top_name(&t), "Selfless Glyphweaver");
}

#[test]
fn quandrix_cascade_into_a_split_card_casts_one_half() {
    cr!("702.85a", "709.3");
    ruling!(
        "Quandrix, the Proof",
        "The mana value of a split card is determined by the combined mana cost of its two halves. If cascade allows you to cast a split card, you may cast either half (as long as the resulting spell would have lesser mana value than the spell with cascade) but not both halves."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    stack_library(&mut t, P0, &["Fire // Ice"]);
    cast_from_hand(&mut t, "Quandrix, the Proof");
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve();
    assert_eq!(top_name(&t), "Ice");
    assert_eq!(spell_names(&t), vec!["Quandrix, the Proof", "Ice"]);
    t.resolve();
    assert!(t.obj_now(bears).tapped);
    assert!(t.in_graveyard(P0, "Fire // Ice"));
}

#[test]
fn noise_marine_counts_the_spell_its_cascade_cast() {
    cr!("702.85a", "608.2h");
    ruling!(
        "Noise Marine",
        "The spell you cast due to cascade resolves before Noise Marine, so when its enters-the-battlefield ability resolves it will count that spell as a spell you've cast this turn."
    );
    supported("Noise Marine");
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Grizzly Bears"]);
    cast_from_hand(&mut t, "Noise Marine");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn volcanic_torrent_counts_every_spell_cast_this_turn_as_it_resolves() {
    cr!("702.85a", "608.2h");
    ruling!(
        "Volcanic Torrent",
        "The value of X is calculated as Volcanic Torrent resolves. It will include Volcanic Torrent, spells you cast in response to Volcanic Torrent, and spells you cast prior to casting Volcanic Torrent, even if those spells haven't resolved yet."
    );
    supported("Volcanic Torrent");
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    // Enlisted Wurm cascades into Volcanic Torrent, which cascades into the Bears.
    stack_library(&mut t, P0, &["Volcanic Torrent", "Grizzly Bears"]);
    cast_from_hand(&mut t, "Enlisted Wurm");
    t.resolve();
    assert_eq!(top_name(&t), "Volcanic Torrent");
    t.resolve();
    assert_eq!(top_name(&t), "Grizzly Bears");
    t.resolve();
    // Volcanic Torrent resolves while the Wurm is still on the stack: X = 3.
    t.resolve();
    assert_eq!(spell_names(&t), vec!["Enlisted Wurm"]);
    assert_eq!(t.obj_now(maw).damage, 3);
}

#[test]
fn forceful_denial_cant_target_itself() {
    cr!("702.85a", "115.5");
    ruling!(
        "Forceful Denial",
        "You can't cast Forceful Denial targeting itself just to cascade."
    );
    supported("Forceful Denial");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let c = t.hand(P0, "Forceful Denial");
    assert!(!can_cast(&mut t, P0, c, CastMethod::Normal));
    // With another spell on the stack, it can be cast targeting that spell.
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    assert!(can_cast(&mut t, P0, c, CastMethod::Normal));
}

#[test]
fn aurora_phoenix_returns_only_if_already_in_the_graveyard() {
    cr!("603.2", "603.10");
    ruling!(
        "Aurora Phoenix",
        "Aurora Phoenix's last ability triggers only if it's in your graveyard immediately after you finish casting the spell with cascade."
    );
    supported("Aurora Phoenix");
    let mut t = TestGame::new(2);
    let phoenix = t.graveyard(P0, "Aurora Phoenix");
    stack_library(&mut t, P0, &["Grizzly Bears"]);
    cast_from_hand(&mut t, "Bloodbraid Elf");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "spell with cascade"), 1);
    t.resolve_all();
    assert!(t.in_hand(P0, "Aurora Phoenix"));
    let _ = phoenix;

    // Put into the graveyard after the spell was cast: no trigger.
    let mut t = TestGame::new(2);
    let phoenix = t.hand(P0, "Aurora Phoenix");
    stack_library(&mut t, P0, &["Grizzly Bears"]);
    cast_from_hand(&mut t, "Bloodbraid Elf");
    t.g.move_object(
        phoenix,
        mtg_engine::object::Zone::Graveyard(P0),
        events::MoveCause::Effect,
        None,
    );
    t.settle();
    assert_eq!(triggers_on_stack(&t, "spell with cascade"), 0);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Aurora Phoenix"));
}

#[test]
fn aurora_phoenix_and_the_cascade_trigger_go_on_the_stack_in_either_order() {
    cr!("603.3b", "702.85a");
    ruling!(
        "Aurora Phoenix",
        "The cascade ability of the spell you cast and Aurora Phoenix's return ability can be put on the stack in either order."
    );
    let mut tops = Vec::new();
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        t.graveyard(P0, "Aurora Phoenix");
        stack_library(&mut t, P0, &["Grizzly Bears"]);
        t.answer(P0, DecisionKind::Order, Answer::Indices(order));
        cast_from_hand(&mut t, "Bloodbraid Elf");
        t.settle();
        assert_eq!(t.stack_len(), 3);
        let phoenix_on_top = top_text(&t).contains("spell with cascade");
        tops.push(phoenix_on_top);
        t.resolve();
        if phoenix_on_top {
            // The Phoenix returns before any card is exiled for cascade.
            assert!(t.in_hand(P0, "Aurora Phoenix"));
            assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
            assert_eq!(triggers_on_stack(&t, "Cascade"), 1);
        } else {
            // The cascaded spell is cast and resolves before the Phoenix returns.
            assert_eq!(top_name(&t), "Grizzly Bears");
            t.resolve();
            assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
            assert!(t.in_graveyard(P0, "Aurora Phoenix"));
        }
        // Either way, all of this happens before Bloodbraid Elf resolves.
        while spell_names(&t) != vec!["Bloodbraid Elf"] || t.stack_len() > 1 {
            assert!(t.named_on_battlefield("Bloodbraid Elf").is_empty());
            t.resolve();
        }
        assert!(t.in_hand(P0, "Aurora Phoenix"));
        assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
        t.resolve();
        assert_eq!(t.named_on_battlefield("Bloodbraid Elf").len(), 1);
    }
    assert_ne!(tops[0], tops[1]);
}

#[test]
fn call_forth_the_tempest_counts_mana_value_of_other_spells_cast_this_turn() {
    cr!("702.85a", "601.2i", "707.10", "202.3");
    ruling!(
        "Call Forth the Tempest",
        "Count the mana values of all other spells you've cast this turn before Call Forth the Tempest resolved, including any spells that you've cast due to Call Forth the Tempest's cascade abilities."
    );
    supported("Call Forth the Tempest");
    let mut t = TestGame::new(2);
    // Earlier this turn: Lightning Bolt (1) and Twincast (2), whose copy of the Bolt
    // wasn't cast and doesn't count.
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Island", 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, bolt).target(Entity::Player(P1)).go();
    let twin = t.hand(P0, "Twincast");
    t.cast(P0, twin).target(spell).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    // The cascades cast Grizzly Bears (2) and Hill Giant (4).
    stack_library(&mut t, P0, &["Grizzly Bears", "Hill Giant"]);
    let colossus = t.battlefield(P1, "Darksteel Colossus");
    cast_from_hand(&mut t, "Call Forth the Tempest");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    // 1 + 2 + 2 + 4: not Call Forth the Tempest itself (8), nor the copy.
    assert_eq!(t.obj_now(colossus).damage, 9);
    let giant = t.named_on_battlefield("Hill Giant")[0];
    assert_eq!(t.obj_now(giant).damage, 0);
}
