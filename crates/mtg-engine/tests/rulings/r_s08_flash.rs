//! Rulings batch S08 — flash (CR 702.8): "You may cast this spell any time you could cast
//! an instant." The rulings of the flash cards of this batch: a creature spell with flash
//! is still a creature spell, P/T effects from flash Auras (layer 7), countering delayed
//! and keyword triggered abilities, and "as though they had flash" for sorceries.

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s06_common::*;
use crate::r_s08_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_creature_with_flash_is_a_creature_spell_whenever_its_cast() {
    cr!("702.8a", "302.1", "304.1");
    ruling!(
        "Defender of Law",
        "This card is always a creature spell and is never an instant spell, no matter when you cast it."
    );
    supported("Defender of Law");
    // P0 casts Defender of Law (flash) in P1's turn.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::DeclareAttackers);
    t.lands(P0, "Plains", 3);
    let defender = t.hand(P0, "Defender of Law");
    let spell = t.cast(P0, defender).go();
    let o = t.obj(spell);
    assert!(o.is(CardType::Creature));
    assert!(!o.is(CardType::Instant));
    // Essence Scatter ("Counter target creature spell") can target it; Negate ("Counter
    // target noncreature spell") can't.
    t.lands(P1, "Island", 4);
    assert!(spell_targets(&mut t, P1, "Essence Scatter").contains(&Entity::Object(spell)));
    assert!(!spell_targets(&mut t, P1, "Negate").contains(&Entity::Object(spell)));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Defender of Law").len(), 1);
}

#[test]
fn a_flash_auras_base_pt_doesnt_overwrite_earlier_modifications_or_switches() {
    cr!("613.4b", "613.4c", "613.4d");
    ruling!(
        "Utter Insignificance",
        "will apply to the creature no matter when they started to take effect. The same is true for counters that change its power and/or toughness and effects that switch its power and toughness."
    );
    supported("Utter Insignificance");
    supported("Wing It");
    supported("Twisted Image");
    // Hill Giant (3/3) with a +1/+1 counter, Wing It (+2/+2), Bonesplitter (+2/+0) and
    // Twisted Image's switch, all before Utter Insignificance ("Enchanted creature loses
    // all abilities and has base power and toughness 1/1.").
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.add_counters(Entity::Object(giant), "+1/+1", 1, None);
    attach_new(&mut t, P1, "Bonesplitter", giant);
    t.lands(P1, "Plains", 2);
    t.lands(P1, "Island", 1);
    let wing = t.hand(P1, "Wing It");
    t.cast(P1, wing).target(giant).go();
    t.resolve_all();
    let twist = t.hand(P1, "Twisted Image");
    t.cast(P1, twist).target(giant).go();
    t.resolve_all();
    // 3/3 + 1/1 + 2/2 + 2/0 = 8/6, switched: 6/8.
    assert_eq!(t.pt(giant), (6, 8));
    t.lands(P0, "Island", 2);
    let utter = t.hand(P0, "Utter Insignificance");
    t.cast(P0, utter).target(giant).go();
    t.resolve_all();
    // 1/1 + 1/1 + 2/2 + 2/0 = 6/4, switched: 4/6.
    assert_eq!(t.pt(giant), (4, 6));
}

/// P0 Sneak Attacks Grizzly Bears into play: "Sacrifice the creature at the beginning of
/// the next end step." Returns the Bears.
fn sneak_in_bears(t: &mut TestGame) -> ObjectId {
    let sneak = t.battlefield(P0, "Sneak Attack");
    t.lands(P0, "Mountain", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, sneak, 0, &[]).unwrap();
    t.resolve();
    t.named_on_battlefield("Grizzly Bears")[0]
}

/// P1 flashes in Ertai Resurrected and counters the stack object `what` with its enters
/// ability ("Counter target spell, activated ability, or triggered ability. Its
/// controller draws a card.").
fn ertai_counters(t: &mut TestGame, what: ObjectId) {
    t.lands(P1, "Island", 2);
    t.lands(P1, "Swamp", 2);
    let ertai = t.hand(P1, "Ertai Resurrected");
    t.cast(P1, ertai).go();
    // The enters trigger: the first mode, targeting `what`.
    t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer_targets(P1, &[Entity::Object(what)]);
    t.resolve();
    assert!(on_stack(&t, "Counter target spell") == 1, "{:?}", stack_items(t));
    t.resolve();
}

#[test]
fn a_countered_next_end_step_trigger_doesnt_trigger_again() {
    cr!("603.7b", "701.6a");
    ruling!(
        "Ertai Resurrected",
        "If you counter a delayed triggered ability that triggers at the beginning of the “next” occurrence of a specified step or phase, that ability won’t trigger again the following time that phase or step occurs."
    );
    supported("Ertai Resurrected");
    supported("Sneak Attack");
    let mut t = TestGame::new(2);
    let bears = sneak_in_bears(&mut t);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(on_stack(&t, "delayed trigger"), 1, "{:?}", stack_items(&t));
    let delayed = top_of_stack(&t);
    let hand = t.hand_size(P0);
    ertai_counters(&mut t, delayed);
    assert_eq!(on_stack(&t, "delayed trigger"), 0);
    assert_eq!(t.hand_size(P0), hand + 1);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // The following end steps: it doesn't trigger again.
    for p in [P1, P0] {
        t.advance_to(p, Step::End);
        t.settle();
        assert_eq!(on_stack(&t, "delayed trigger"), 0);
        t.resolve_all();
        assert!(t.on_battlefield(bears));
    }
}

#[test]
fn keyword_triggered_abilities_like_prowess_can_be_countered() {
    cr!("603.1", "702.108a", "701.6a");
    ruling!(
        "Ertai Resurrected",
        "Some keyword abilities, such as prowess and fabricate, are triggered abilities and will have “when,” “whenever,” or “at” in their reminder text."
    );
    supported("Monastery Swiftspear");
    // Monastery Swiftspear (1/2, prowess): P0 casts Lightning Bolt, prowess triggers, and
    // P1 counters the prowess trigger.
    let mut t = TestGame::new(2);
    let spear = t.battlefield(P0, "Monastery Swiftspear");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    let prowess = top_of_stack(&t);
    ertai_counters(&mut t, prowess);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.pt(spear), (1, 2));
}

#[test]
fn casting_sorceries_as_though_they_had_flash_doesnt_change_sorcery_speed_activations() {
    cr!("702.8a", "307.5", "702.6a", "602.5d");
    ruling!(
        "Gandalf, Friend of the Shire",
        "The second ability applies only to casting spells. It does not, for example, change when you may activate abilities that can be activated \"only as a sorcery.\""
    );
    supported("Gandalf, Friend of the Shire");
    // Gandalf: "You may cast sorcery spells as though they had flash."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gandalf, Friend of the Shire");
    t.battlefield(P0, "Grizzly Bears");
    let splitter = t.battlefield(P0, "Bonesplitter");
    t.lands(P0, "Island", 3);
    let divination = t.hand(P0, "Divination");
    // In P1's turn: the sorcery can be cast, but Bonesplitter can't be equipped.
    t.set_step(P1, Step::PrecombatMain);
    assert!(can_cast(&mut t, P0, divination, CastMethod::Normal));
    assert!(!can_activate(&mut t, P0, splitter));
    // In P0's main phase with a spell on the stack: the same.
    t.set_step(P0, Step::PrecombatMain);
    assert!(can_activate(&mut t, P0, splitter));
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P1).go();
    assert!(can_cast(&mut t, P0, divination, CastMethod::Normal));
    assert!(!can_activate(&mut t, P0, splitter));
}

#[test]
fn with_teferi_creature_cards_can_be_suspended_any_time_you_could_cast_an_instant() {
    cr!("702.8a", "116.2f", "702.62a");
    ruling!(
        "Teferi, Mage of Zhalfir",
        "While you control Teferi, you may exile creature cards in your hand with suspend any time you could cast an instant."
    );
    supported("Teferi, Mage of Zhalfir");
    supported("Teferi, Druid of Argoth");
    supported("Durkwood Baloth");
    use mtg_engine::decision::{Action, SpecialAction};
    // Teferi: "Creature cards you own that aren't on the battlefield have flash."
    // Durkwood Baloth: {4}{G}{G} creature, "Suspend 5—{G}".
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P0, "Forest", 6);
    let baloth = t.hand(P0, "Durkwood Baloth");
    let suspend = Action::Special(SpecialAction::Suspend { card: baloth });
    assert!(!actions_of(&mut t, P0).contains(&suspend));
    assert!(!can_cast(&mut t, P0, baloth, CastMethod::Normal));
    t.battlefield(P0, "Teferi, Mage of Zhalfir");
    assert!(t.obj(baloth).has_keyword(mtg_engine::keywords::KeywordKind::Flash));
    assert!(actions_of(&mut t, P0).contains(&suspend));
    assert!(can_cast(&mut t, P0, baloth, CastMethod::Normal));
    t.g.take_action(P0, suspend);
    t.g.flush_events();
    assert!(t.in_exile("Durkwood Baloth"));
    assert_eq!(t.counters(baloth, "time"), 5);
    // Cards P1 owns don't get flash, nor do P0's noncreature cards.
    let theirs = t.hand(P1, "Durkwood Baloth");
    let sorcery = t.hand(P0, "Divination");
    t.g.recompute();
    assert!(!t.obj(theirs).has_keyword(mtg_engine::keywords::KeywordKind::Flash));
    assert!(!t.obj(sorcery).has_keyword(mtg_engine::keywords::KeywordKind::Flash));
}
