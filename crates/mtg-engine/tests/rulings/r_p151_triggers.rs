//! Rulings batch P151 — permanents whose triggered abilities sacrifice them: state
//! triggers (CR 603.8), "if you do" and reflexive riders (CR 603.12), triggers for several
//! simultaneous events, and delayed sacrifices (CR 603.7).

use crate::r_p151_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// The source of the triggered ability on top of the stack.
fn top_trigger_source(t: &TestGame) -> ObjectId {
    let top = *t.g.stack.last().expect("a trigger on the stack");
    match &t.g.obj(top).stack.as_ref().unwrap().kind {
        mtg_engine::object::StackKind::Triggered { source, .. } => *source,
        k => panic!("not a trigger: {k:?}"),
    }
}

#[test]
fn grave_peril_destroys_one_creature_and_the_last_to_enter() {
    cr!("603.2");
    ruling!(
        "Grave Peril",
        "If multiple nonblack creatures enter at the same time, Grave Peril will destroy only one of them. Grave Peril's ability will trigger multiple times, but only the first one to resolve will do anything."
    );
    supported("Grave Peril");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grave Peril");
    let both = crate::r_p116_common::enter_together(&mut t, P1, &["Grizzly Bears", "Hill Giant"]);
    t.settle();
    assert_eq!(abilities_on_stack(&t), 2);
    t.resolve_all();
    let alive = both.iter().filter(|c| t.on_battlefield(**c)).count();
    assert_eq!(alive, 1);
    assert!(t.in_graveyard(P0, "Grave Peril"));

    ruling!(
        "Grave Peril",
        "If a nonblack creature enters and Grave Peril triggers, then another nonblack creature enters, Grave Peril will trigger again and destroy only the last creature to enter."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grave Peril");
    let first = t.enter(P1, "Grizzly Bears");
    t.settle();
    let last = t.enter(P1, "Hill Giant");
    t.settle();
    assert_eq!(abilities_on_stack(&t), 2);
    t.resolve_all();
    assert!(t.on_battlefield(first));
    assert!(!t.on_battlefield(last));
}

#[test]
fn blood_spatter_analysis_returns_a_card_only_when_its_own_ability_sacrifices_it() {
    cr!("603.12");
    ruling!(
        "Blood Spatter Analysis",
        "Blood Spatter Analysis's reflexive triggered ability triggers and returns a creature card to your hand only if you sacrifice it while resolving its second ability. It won't trigger if you sacrifice it for any other reason."
    );
    supported("Blood Spatter Analysis");
    // Its own ability sacrifices it: the reflexive ability returns the Giant.
    let mut t = TestGame::new(2);
    let bsa = t.battlefield(P0, "Blood Spatter Analysis");
    put_counters(&mut t, bsa, "bloodstain", 4);
    let giant = t.graveyard(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(giant)]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Blood Spatter Analysis"));
    assert!(t.in_hand(P0, "Hill Giant"));

    // Sacrificed for another reason (to Ashnod's Altar's... a sacrifice effect): nothing.
    let mut t = TestGame::new(2);
    let bsa = t.battlefield(P0, "Blood Spatter Analysis");
    put_counters(&mut t, bsa, "bloodstain", 4);
    t.graveyard(P0, "Hill Giant");
    t.g.sacrifice(bsa, P0);
    t.g.flush_events();
    t.settle();
    assert_eq!(abilities_on_stack(&t), 0);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn evolved_spinoderm_isnt_sacrificed_until_its_upkeep_ability_resolves() {
    cr!("603.8");
    ruling!(
        "Evolved Spinoderm",
        "If Evolved Spinoderm somehow loses all of its oil counters due to an effect other than its upkeep triggered ability, this won't cause its controller to sacrifice it until the next time its upkeep triggered ability resolves."
    );
    supported("Evolved Spinoderm");
    let mut t = TestGame::new(2);
    let spino = t.battlefield(P0, "Evolved Spinoderm");
    put_counters(&mut t, spino, "oil", 1);
    remove_counters(&mut t, spino, "oil", 1);
    t.resolve_all();
    assert!(t.on_battlefield(spino));
    next_upkeep(&mut t);
    assert!(t.on_battlefield(spino));
    t.resolve_all();
    assert!(!t.on_battlefield(spino));
}

#[test]
fn millennium_calendar_retriggers_when_its_ability_is_countered() {
    cr!("603.8");
    ruling!(
        "The Millennium Calendar",
        "If The Millenium Calendar's last ability is countered or otherwise removed from the stack, it will just trigger again immediately if The Millenium Calendar still has 1,000 or more time counters on it."
    );
    supported("The Millennium Calendar");
    let mut t = TestGame::new(2);
    let cal = t.battlefield(P0, "The Millennium Calendar");
    put_counters(&mut t, cal, "time", 1000);
    assert_eq!(abilities_on_stack(&t), 1);
    let ability = *t.g.stack.last().unwrap();
    counter_spell(&mut t, ability);
    assert_eq!(abilities_on_stack(&t), 1, "triggered again");
    t.resolve_all();
    assert!(!t.on_battlefield(cal));
    assert!(t.has_lost(P1));
}

#[test]
fn porphyry_nodes_triggers_when_every_creature_blinks() {
    cr!("603.8");
    ruling!(
        "Porphyry Nodes",
        "If all creatures on the battlefield leave and immediately return (perhaps because Momentary Blink exiled and returned the only creature), the last ability of Porphyry Nodes triggers. You'll sacrifice it even though a creature's on the battlefield by the time the ability resolves."
    );
    supported("Porphyry Nodes");
    let mut t = TestGame::new(2);
    let nodes = t.battlefield(P0, "Porphyry Nodes");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Momentary Blink", &[obj(bears)]);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(abilities_on_stack(&t), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(nodes));
}

#[test]
fn dragon_whelp_activated_in_the_end_step_is_sacrificed_next_end_step() {
    cr!("603.7c");
    ruling!(
        "Dragon Whelp",
        "If the fourth (or more) time Dragon Whelp's ability is activated during the same turn happens to be during that turn's end step, the delayed triggered ability won't trigger until the beginning of the next turn's end step. You'll have to sacrifice Dragon Whelp at that time."
    );
    ruling!(
        "Dragon Whelp",
        "If the fourth (or subsequent) time Dragon Whelp's ability is activated during the same turn is during that turn's end step, the delayed triggered ability won't trigger until the beginning of the next turn's end step. You'll have to sacrifice Dragon Whelp at that time."
    );
    supported("Dragon Whelp");
    let mut t = TestGame::new(2);
    let whelp = t.battlefield(P0, "Dragon Whelp");
    t.lands(P0, "Mountain", 4);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    for _ in 0..4 {
        t.activate(P0, whelp, 0, &[]).expect("firebreathing");
        t.resolve_all();
    }
    assert_eq!(t.pt(whelp).0, 6);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(whelp), "not sacrificed this turn");
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(whelp));
}

#[test]
fn chimeric_coils_with_x_0_dies() {
    cr!("704.5f");
    ruling!(
        "Chimeric Coils",
        "If X is 0, Chimeric Coils becomes a 0/0 creature and is put into its owner's graveyard."
    );
    supported("Chimeric Coils");
    let mut t = TestGame::new(2);
    let coils = t.battlefield(P0, "Chimeric Coils");
    t.lands(P0, "Wastes", 1);
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    t.activate(P0, coils, 0, &[]).expect("Chimeric Coils");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Chimeric Coils"));
}

#[test]
fn dress_down_as_a_creature_and_abilities_granted_later() {
    cr!("613.7");
    ruling!(
        "Dress Down",
        "If an effect causes Dress Down to become a creature, it will lose its own abilities along with all other creatures. This includes the triggered ability that causes it to be sacrificed at the beginning of the end step."
    );
    supported("Dress Down");
    let mut t = TestGame::new(2);
    let dress = t.battlefield(P0, "Dress Down");
    t.battlefield(P0, "Opalescence");
    let drake = t.battlefield(P1, "Wind Drake");
    t.g.recompute();
    assert!(t.obj_now(dress).is(CardType::Creature));
    assert!(!t.obj_now(drake).has_keyword(KeywordKind::Flying));
    end_step(&mut t, P0);
    assert!(t.on_battlefield(dress), "lost its sacrifice trigger");

    ruling!(
        "Dress Down",
        "If an effect grants a creature an ability after Dress Down has entered the battlefield, it won't lose that ability. For example, if a land becomes a creature while Dress Down is on the battlefield, it will still gain any abilities given to it by the effect that animated it. It will, however, lose any abilities it already had."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dress Down");
    let colonnade = t.battlefield(P0, "Celestial Colonnade");
    t.lands(P0, "Wastes", 5);
    // "{3}{W}{U}: Until end of turn, Celestial Colonnade becomes a 4/4 white and blue
    // Elemental creature with flying and vigilance. It's still a land."
    mana(&mut t, P0, &[mtg_engine::mana::ManaType::W, mtg_engine::mana::ManaType::U]);
    act(&mut t, P0, colonnade, "Elemental", &[]).expect("animate");
    t.resolve_all();
    let o = t.obj_now(colonnade);
    assert!(o.is(CardType::Creature));
    assert!(o.has_keyword(KeywordKind::Flying) && o.has_keyword(KeywordKind::Vigilance));
    assert!(
        !o.chars.abilities.iter().any(|a| a.is_mana_ability()),
        "its mana abilities are lost"
    );
}

/// Adds one mana of each type in `types` to `p`'s pool.
fn mana(t: &mut TestGame, p: PlayerId, types: &[mtg_engine::mana::ManaType]) {
    for ty in types {
        t.g.players[p.idx()].mana_pool.add_type(*ty, 1);
    }
}

#[test]
fn promise_of_tomorrow_cases() {
    ruling!(
        "Promise of Tomorrow",
        "If a creature you control dies, but it isn't a creature card in the graveyard (perhaps because it was a noncreature that became a creature), Promise of Tomorrow will still exile it, and it may return it to the battlefield later."
    );
    supported("Promise of Tomorrow");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Promise of Tomorrow");
    let coils = t.battlefield(P0, "Chimeric Coils");
    t.lands(P0, "Wastes", 3);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, coils, 0, &[]).expect("Chimeric Coils");
    t.resolve_all();
    destroy(&mut t, coils);
    t.resolve_all();
    assert!(t.in_exile("Chimeric Coils"));
    end_step(&mut t, P0);
    assert_eq!(t.named_on_battlefield("Chimeric Coils").len(), 1);
    assert!(t.in_graveyard(P0, "Promise of Tomorrow"));

    ruling!(
        "Promise of Tomorrow",
        "If you control more than one Promise of Tomorrow, each creature you control that dies will cause the triggered ability of each of them to trigger. Those abilities can be put on the stack in any order, and the first one to resolve is the one that exiles the creature card. That creature card will be exiled by that Promise of Tomorrow, and only that Promise of Tomorrow can return it."
    );
    for keep_exiler in [true, false] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Promise of Tomorrow");
        let b = t.battlefield(P0, "Promise of Tomorrow");
        let bears = t.battlefield(P0, "Grizzly Bears");
        destroy(&mut t, bears);
        assert_eq!(abilities_on_stack(&t), 2);
        let exiler = top_trigger_source(&t);
        let other = if exiler == a { b } else { a };
        t.resolve_all();
        assert!(t.in_exile("Grizzly Bears"));
        exile(&mut t, if keep_exiler { other } else { exiler });
        end_step(&mut t, P0);
        assert_eq!(
            t.named_on_battlefield("Grizzly Bears").len(),
            keep_exiler as usize,
            "only the Promise that exiled it returns it"
        );
    }
}

#[test]
fn sothera_triggers_for_creatures_dying_as_it_leaves() {
    cr!("603.10a");
    ruling!(
        "Sothera, the Supervoid",
        "If Sothera leaves the battlefield at the same time as one or more creatures you control die, its first ability will trigger for each of those creatures."
    );
    supported("Sothera, the Supervoid");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sothera, the Supervoid");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    cast_new(&mut t, P0, "Akroma's Vengeance", &[]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Sothera, the Supervoid"));
    assert_eq!(abilities_on_stack(&t), 2);
}

#[test]
fn charitable_levy_trigger_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3");
    ruling!(
        "Charitable Levy",
        "Charitable Levy's triggered ability resolves before the spell that causes it to trigger. The ability will resolve even if that spell is countered or otherwise leaves the stack."
    );
    supported("Charitable Levy");
    let mut t = TestGame::new(2);
    let levy = t.battlefield(P0, "Charitable Levy");
    put_counters(&mut t, levy, "collection", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    let shock = cast_new(&mut t, P0, "Shock", &[obj(bears)]);
    t.settle();
    assert_eq!(abilities_on_stack(&t), 1);
    counter_spell(&mut t, shock);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(!t.on_battlefield(levy));
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.on_battlefield(bears));
}

#[test]
fn quest_for_the_holy_relic_trigger_resolves_before_the_creature_spell() {
    cr!("603.3");
    ruling!(
        "Quest for the Holy Relic",
        "If you cast a creature spell, Quest for the Holy Relic’s triggered ability triggers and goes on the stack on top of it. The triggered ability will resolve before the spell does."
    );
    supported("Quest for the Holy Relic");
    let mut t = TestGame::new(2);
    let quest = t.battlefield(P0, "Quest for the Holy Relic");
    let bears = cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.counters(quest, "quest"), 1);
    assert_eq!(t.zone(bears), mtg_engine::object::Zone::Stack);
}

#[test]
fn quest_for_the_gemblades_triggers_too_late_to_save_the_creature() {
    cr!("510.2");
    ruling!(
        "Quest for the Gemblades",
        "If there are no counters on Quest for the Gemblades yet, and simultaneously a creature you control deals combat damage to a creature and a creature you control is dealt lethal combat damage, the first ability will trigger too late for you to be able to activate the second ability and save your creature."
    );
    supported("Quest for the Gemblades");
    let mut t = TestGame::new(2);
    let quest = t.battlefield(P0, "Quest for the Gemblades");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(blocker, bears)]));
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(abilities_on_stack(&t), 1);
    // By the time the trigger is on the stack, the Bears have already died.
    assert!(!t.on_battlefield(bears));
    t.resolve_all();
    assert_eq!(t.counters(quest, "quest"), 1);
}
