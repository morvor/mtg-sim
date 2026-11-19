//! Rulings residue of batches P020, P021 and P023 — more interactions of creatures that
//! copy: what enters at the same time (CR 603.6a, 614.12), abilities whose source left
//! before they resolved (CR 610.3c, 603.7), copies and the legend rule (CR 704.5j),
//! face-down copies (CR 708.2, 707.2), type-changing copy exceptions (CR 707.9b, 306.5),
//! linked abilities of copies (CR 607.5, 707.7), and tokens doubled by replacement effects
//! (CR 614.1a, 603.7c).

use crate::r_residue_copy_creature::*;
use crate::r_s01_common::{custom_card, supported, triggers_on_stack};
use crate::r_s18_common::lands_for;
use crate::r_s26_common::new_tokens;
use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// --- Entering at the same time -----------------------------------------------------------

#[test]
fn copiers_cant_copy_what_enters_at_the_same_time() {
    cr!("614.12", "614.12a", "707.5");
    ruling!(
        "Altered Ego",
        "If Altered Ego somehow enters at the same time as another creature, Altered Ego can't become a copy of that creature. You may choose only a creature that's already on the battlefield."
    );
    ruling!(
        "Cursed Mirror",
        "If Cursed Mirror somehow enters the battlefield at the same time as a creature, it can't become a copy of that creature. You may choose only a creature that's already on the battlefield."
    );
    ruling!(
        "Hulking Metamorph",
        "If Hulking Metamorph somehow enters the battlefield at the same time as another permanent, Hulking Metamorph can't become a copy of that permanent. You may choose only a permanent that's already on the battlefield."
    );
    for card in ["Altered Ego", "Cursed Mirror", "Hulking Metamorph"] {
        supported(card);
        let mut t = TestGame::new(2);
        let old = t.battlefield(P0, "Hill Giant");
        let from = t.asked().len();
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(old)]);
        let entered =
            crate::r_s24_common::enter_together(&mut t, &[(P0, card), (P0, "Grizzly Bears")]);
        t.clear_answers();
        let offered = crate::r_p017_common::choice_candidates(&t, from);
        assert!(!offered.is_empty(), "{card}: no choice");
        for c in &offered {
            assert!(!c.contains(&obj(entered[1])), "{card} could copy the Bears");
        }
        assert!(offered[0].contains(&obj(old)), "{card}: {offered:?}");
        assert_eq!(name_of(&t, entered[0]), "Hill Giant", "{card}");
    }
}

#[test]
fn saheeli_rai_artifacts_enter_together() {
    cr!("603.6a", "614.12", "707.5");
    ruling!(
        "Saheeli Rai",
        "As Saheeli's last ability resolves, all three artifacts enter the battlefield at the same time. Triggered abilities of any of those artifacts will see all three entering the battlefield. Replacement effects, such as that of Sculpting Steel, won't see the others on the battlefield as they are being applied."
    );
    // "−7: Search your library for up to three artifact cards with different names, put
    // them onto the battlefield, then shuffle." Sculpting Steel ("You may have this
    // artifact enter as a copy of any artifact on the battlefield."), Glint Hawk Idol
    // ("Whenever another artifact you control enters, you may have this artifact become a
    // 2/2 Bird artifact creature with flying until end of turn.") and Ornithopter. A
    // Bonesplitter was already there.
    supported("Saheeli Rai");
    supported("Sculpting Steel");
    supported("Glint Hawk Idol");
    let mut t = TestGame::new(2);
    let saheeli = t.battlefield(P0, "Saheeli Rai");
    t.g.add_counters(obj(saheeli), counters::LOYALTY, 7, None);
    let splitter = t.battlefield(P0, "Bonesplitter");
    let steel = t.library_top(P0, "Sculpting Steel");
    let idol = t.library_top(P0, "Glint Hawk Idol");
    let thopter = t.library_top(P0, "Ornithopter");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[obj(steel), obj(idol), obj(thopter)]);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(splitter)]);
    crate::r_s06_common::activate_containing(&mut t, P0, saheeli, "Search your library")
        .expect("Saheeli's −7");
    t.resolve();
    t.settle();
    // Sculpting Steel could copy only the Bonesplitter: not the Idol or the Ornithopter.
    let offered = crate::r_p017_common::choice_candidates(&t, from);
    let steel_choice = offered
        .iter()
        .find(|c| c.contains(&obj(splitter)))
        .expect("Sculpting Steel's choice");
    for e in steel_choice {
        assert_eq!(name_of(&t, e.object().unwrap()), "Bonesplitter");
    }
    assert_eq!(t.named_on_battlefield("Bonesplitter").len(), 2);
    // The Idol's trigger saw the other two enter.
    assert_eq!(triggers_on_stack(&t, "Bird artifact creature"), 2);
}

// --- Abilities whose source left -----------------------------------------------------------

#[test]
fn exile_until_it_leaves_does_nothing_if_it_already_left() {
    cr!("610.3b", "603.2");
    ruling!(
        "Assimilation Aegis",
        "If Assimilation Aegis leaves the battlefield before its first ability resolves, the target creature won't be exiled at all."
    );
    ruling!(
        "Mardu Siegebreaker",
        "If Mardu Siegebreaker leaves the battlefield before its “enters” ability resolves, the target creature won’t be exiled."
    );
    for (card, whose) in [("Assimilation Aegis", P1), ("Mardu Siegebreaker", P0)] {
        supported(card);
        let mut t = TestGame::new(2);
        let giant = t.battlefield(whose, "Hill Giant");
        t.answer_targets(P0, &[obj(giant)]);
        let it = t.enter(P0, card);
        t.settle();
        assert_eq!(t.stack_len(), 1, "{card}");
        crate::r_s02_common::destroy(&mut t, it);
        t.resolve_all();
        assert!(t.on_battlefield(giant), "{card}");
        assert!(!t.in_exile("Hill Giant"), "{card}");
    }
}

#[test]
fn dual_nature_token_stays_if_dual_nature_left_first() {
    cr!("603.6a", "603.6c", "113.7a");
    ruling!(
        "Dual Nature",
        "If Dual Nature leaves the battlefield between when it triggers and when a token is put onto the battlefield, that token remains on the battlefield indefinitely. It missed the separate ability which would have exiled the token."
    );
    // "Whenever a nontoken creature enters, its controller creates a token that's a copy
    // of that creature. ... When this enchantment leaves the battlefield, exile all tokens
    // created with this enchantment."
    supported("Dual Nature");
    let mut t = TestGame::new(2);
    let dn = t.battlefield(P0, "Dual Nature");
    t.enter(P0, "Hill Giant");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    crate::r_s02_common::destroy(&mut t, dn);
    let before = t.g.battlefield.clone();
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_eq!(name_of(&t, toks[0]), "Hill Giant");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert!(t.on_battlefield(toks[0]));
}

// --- Copies and the legend rule ------------------------------------------------------------

#[test]
fn gogo_copying_a_legend_keeps_its_name() {
    cr!("704.5j", "707.9b", "201.2a");
    ruling!(
        "Gogo, Mysterious Mime",
        "If Gogo copies a legendary creature, Gogo will also be legendary, but because its name remains Gogo, Mysterious Mime, the \"legend rule\" won't cause either of them to be put into the graveyard."
    );
    supported("Isamaru, Hound of Konda");
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    let gogo = copy_with(&mut t, "Gogo, Mysterious Mime", isamaru)[0];
    assert!(t.obj_now(gogo).chars.is_legendary());
    assert!(has_subtype(&t, gogo, "Dog"));
    assert!(t.on_battlefield(gogo) && t.on_battlefield(isamaru));
}

#[test]
fn nascent_metamorph_copying_a_legend_you_control_meets_the_legend_rule() {
    cr!("704.5j", "707.2");
    ruling!(
        "Nascent Metamorph",
        "If Nascent Metamorph becomes a copy of a legendary card with the same name as a legendary permanent you control, you'll put one of them into its owner's graveyard."
    );
    // "Whenever this creature attacks or blocks, target opponent reveals cards from the
    // top of their library until they reveal a creature card. This creature becomes a
    // copy of that card until end of turn."
    supported("Nascent Metamorph");
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    let m = t.battlefield(P0, "Nascent Metamorph");
    crate::r_s01_common::stack_library(&mut t, P1, &["Isamaru, Hound of Konda"]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    // Keep the original.
    t.answer_choose(P0, &[obj(isamaru)]);
    attack_p1(&mut t, &[m]);
    assert!(t.on_battlefield(isamaru));
    assert!(!t.on_battlefield(m));
    assert!(t.in_graveyard(P0, "Nascent Metamorph"));
}

#[test]
fn nascent_metamorphs_linked_abilities_last_only_while_it_copies() {
    cr!("607.5", "707.7", "607.2a", "611.2a");
    ruling!(
        "Nascent Metamorph",
        "If Nascent Metamorph becomes a copy of an object with a set of linked abilities (for example, one ability that exiles a card and another that refers to the card \"exiled with\" the object), that link only lasts as long as Nascent Metamorph is copying that object. If it stops being a copy of that object and then becomes a copy again later, the link is lost."
    );
    // Kheru Mind-Eater: "Whenever Kheru Mind-Eater deals combat damage to a player, that
    // player exiles a card from their hand face down. / You may look at cards exiled with
    // Kheru Mind-Eater, and you may play lands and cast spells from among those cards."
    // The Metamorph becomes a copy of it as it attacks, deals combat damage, and P1
    // exiles Shock.
    supported("Kheru Mind-Eater");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Nascent Metamorph");
    t.lands(P0, "Mountain", 2);
    let shock = t.hand(P1, "Shock");
    crate::r_s01_common::stack_library(&mut t, P1, &["Kheru Mind-Eater"]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P1, &[obj(shock)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(m, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(name_of(&t, m), "Kheru Mind-Eater");
    let exiled = t.g.current(shock);
    assert_eq!(t.zone(exiled), Zone::Exile);
    t.advance_to(P0, Step::PostcombatMain);
    assert!(
        crate::r_s02_common::can_cast(&mut t, P0, exiled, CastMethod::Normal),
        "the linked ability works while it's a copy"
    );
    // P0's next turn: it's a Nascent Metamorph again, then becomes a copy of Kheru
    // Mind-Eater again as it attacks: the Shock exiled with the earlier copy isn't
    // exiled with this one.
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(name_of(&t, m), "Nascent Metamorph");
    crate::r_s01_common::stack_library(&mut t, P1, &["Kheru Mind-Eater"]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(m, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(name_of(&t, m), "Kheru Mind-Eater");
    assert!(
        !crate::r_s02_common::can_cast(&mut t, P0, exiled, CastMethod::Normal),
        "the link was lost"
    );
}

// --- Face-down copies ---------------------------------------------------------------------

fn turn_up(id: ObjectId) -> Action {
    Action::Special(SpecialAction::TurnFaceUp { obj: id })
}

#[test]
fn body_double_copying_vesuvan_shapeshifter_turned_face_down_can_morph_up() {
    cr!("707.2", "707.3", "708.2a", "702.37e", "614.1e");
    ruling!(
        "Body Double",
        "If Body Double copies a Vesuvan Shapeshifter which then copies another creature and then is somehow turned face down, the Body Double is now a face-down Vesuvan Shapeshifter and may be turned face up via its Morph ability."
    );
    // Body Double ("You may have this creature enter as a copy of any creature card in a
    // graveyard.") enters as a copy of a Vesuvan Shapeshifter card, which ("As ~ enters or
    // is turned face up, you may choose another creature on the battlefield. If you do,
    // until ~ is turned face down, it becomes a copy of that creature, except it has "At
    // the beginning of your upkeep, you may turn ~ face down."" Morph {1}{U}) has it enter
    // as a copy of Hill Giant.
    supported("Vesuvan Shapeshifter");
    let mut t = TestGame::new(2);
    let shifter = t.graveyard(P1, "Vesuvan Shapeshifter");
    let giant = t.battlefield(P1, "Hill Giant");
    let angel = t.battlefield(P1, "Serra Angel");
    t.answer_choose(P0, &[obj(shifter)]);
    t.answer_choose(P0, &[obj(giant)]);
    let bd = t.enter(P0, "Body Double");
    t.settle();
    let bd = t.g.current(bd);
    assert_eq!(name_of(&t, bd), "Hill Giant");
    // Turned face down at P0's upkeep: a face-down 2/2.
    t.answer_yes(P0, true);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.obj_now(bd).face_down);
    assert_eq!(t.pt(bd), (2, 2));
    // Its morph ({1}{U}) is a Vesuvan Shapeshifter's: turned face up, it chooses again.
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 2);
    t.g.turn.priority = Some(P0);
    let bd = t.g.current(bd);
    assert!(t.g.legal_actions(P0).contains(&turn_up(bd)));
    t.answer_choose(P0, &[obj(angel)]);
    t.g.perform_action(P0, turn_up(bd)).unwrap();
    t.resolve_all();
    assert!(!t.obj_now(bd).face_down);
    assert_eq!(name_of(&t, bd), "Serra Angel");
    assert_eq!(t.pt(bd), (4, 4));
}

// --- Gyrus, Waker of Corpses ----------------------------------------------------------------

#[test]
fn gyrus_put_onto_the_battlefield_has_no_counters_and_dies() {
    cr!("614.1c", "704.5f");
    ruling!(
        "Gyrus, Waker of Corpses",
        "If Gyrus enters the battlefield without being cast, then no mana was spent to cast it. It enters the battlefield without any +1/+1 counters. If no other effects are increasing its toughness at that time, it will be put into its owner's graveyard as a state-based action."
    );
    supported("Gyrus, Waker of Corpses");
    let mut t = TestGame::new(2);
    let g = t.enter(P0, "Gyrus, Waker of Corpses");
    assert_eq!(t.counters(t.g.current(g), counters::PLUS1), 0);
    t.settle();
    assert!(t.in_graveyard(P0, "Gyrus, Waker of Corpses"));
    // With Glorious Anthem (+1/+1) it survives as a 1/1 with no counters.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    let g = t.enter(P0, "Gyrus, Waker of Corpses");
    t.settle();
    let g = t.g.current(g);
    assert!(t.on_battlefield(g));
    assert_eq!(t.counters(g, counters::PLUS1), 0);
    assert_eq!(t.pt(g), (1, 1));
}

// --- Imposter Mech copying a creature planeswalker -----------------------------------------

#[test]
fn imposter_mech_copying_a_creature_planeswalker_is_neither() {
    cr!(
        "707.9b", "205.1a", "606.3", "306.5", "306.5b", "120.3c", "120.3e", "702.122a", "508.1b"
    );
    ruling!(
        "Imposter Mech",
        "If Imposter Mech enters the battlefield as a copy of a creature that was also a planeswalker, it loses both the planeswalker and creature card types. Its controller may activate any loyalty abilities it now has as normal, but it notably cannot be attacked and it does not lose loyalty counters when it is dealt damage."
    );
    // No card is printed a creature planeswalker: a custom one, "Creature Planeswalker —
    // Cat Ajani" with loyalty 3 and "+1: You gain 1 life." and "−1: You gain 3 life."
    supported("Imposter Mech");
    let mut def = custom_card(
        "Feline Walker",
        "Creature Planeswalker — Cat Ajani",
        "{2}{W}",
        Some((2, 2)),
        "+1: You gain 1 life.\n−1: You gain 3 life.",
    );
    def.faces[0].chars.loyalty = Some(3);
    let mut t = TestGame::new(2);
    let walker = t.custom(P1, def, Zone::Battlefield);
    let mech = crate::r_p023_common::enter_copying(&mut t, P0, "Imposter Mech", walker);
    t.clear_answers();
    let o = t.obj_now(mech);
    assert_eq!(o.chars.name, "Feline Walker");
    assert!(o.is(CardType::Artifact) && o.chars.has_subtype("Vehicle"));
    assert!(!o.is(CardType::Creature) && !o.is(CardType::Planeswalker));
    // Not a planeswalker, it entered with no loyalty counters (CR 306.5b).
    assert_eq!(t.counters(mech, counters::LOYALTY), 0);
    // Its loyalty abilities can be activated as normal: +1 (a loyalty counter, 1 life).
    t.set_step(P0, Step::PrecombatMain);
    let life = t.life(P0);
    crate::r_s06_common::activate_containing(&mut t, P0, mech, "+1").expect("loyalty ability");
    t.resolve_all();
    assert_eq!(t.life(P0), life + 1);
    assert_eq!(t.counters(mech, counters::LOYALTY), 1);
    // Crewed by a Hill Giant, it's an artifact creature (a 2/2, still no planeswalker):
    // damage dealt to it is marked on it and doesn't remove its loyalty counter.
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(giant)]);
    crate::r_s06_common::activate_containing(&mut t, P0, mech, "Crew").expect("crew");
    t.resolve_all();
    let o = t.obj_now(mech);
    assert!(o.is(CardType::Creature) && !o.is(CardType::Planeswalker));
    let src = t.battlefield(P1, "Prodigal Pyromancer");
    crate::r_s06_common::damage(&mut t, src, 1, mech);
    assert!(t.on_battlefield(mech));
    assert_eq!(t.obj_now(mech).damage, 1);
    assert_eq!(t.counters(mech, counters::LOYALTY), 1);
    // It can't be attacked: P1's attackers can attack only P0.
    // (A planeswalker P0 controls could be attacked.)
    let pw = t.battlefield(P0, "Gideon Jura");
    t.set_step(P1, Step::BeginningOfCombat);
    let targets = mtg_engine::combat::attack_targets(&t.g);
    assert!(targets.contains(&Entity::Player(P0)), "{targets:?}");
    assert!(targets.contains(&obj(pw)));
    assert!(!targets.contains(&obj(mech)));
}

// --- Inalla, Archmage Ritualist --------------------------------------------------------------

#[test]
fn inalla_entering_with_other_wizards_triggers_for_each() {
    cr!("603.6a", "603.2");
    ruling!(
        "Inalla, Archmage Ritualist",
        "If Inalla enters the battlefield at the same time as other Wizards you control, its ability will trigger for each of those other Wizards."
    );
    // "Eminence — Whenever another nontoken Wizard you control enters, if Inalla is in
    // the command zone or on the battlefield, you may pay {1}. If you do, create a token
    // that's a copy of that Wizard. ..." Two Prodigal Pyromancers (Human Wizards).
    supported("Inalla, Archmage Ritualist");
    let mut t = TestGame::new(2);
    crate::r_s24_common::enter_together(
        &mut t,
        &[
            (P0, "Inalla, Archmage Ritualist"),
            (P0, "Prodigal Pyromancer"),
            (P0, "Prodigal Pyromancer"),
        ],
    );
    assert_eq!(triggers_on_stack(&t, "Wizard"), 2, "{}", t.dump_log());
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    let before = t.g.battlefield.clone();
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 2, "{}", t.dump_log());
    for tok in toks {
        assert_eq!(name_of(&t, tok), "Prodigal Pyromancer");
    }
}

// --- Infinite Reflection -------------------------------------------------------------------

/// P0 casts Infinite Reflection ("Enchant creature / When this Aura enters attached to a
/// creature, each other nontoken creature you control becomes a copy of that creature. /
/// Nontoken creatures you control enter as a copy of enchanted creature.") on `what`.
fn infinite_reflection(t: &mut TestGame, what: ObjectId) -> ObjectId {
    supported("Infinite Reflection");
    lands_for(t, P0, "{5}{U}");
    let c = t.hand(P0, "Infinite Reflection");
    t.cast(P0, c).target(obj(what)).go();
    t.resolve_all();
    t.named_on_battlefield("Infinite Reflection")[0]
}

#[test]
fn infinite_reflection_on_a_token_and_after_it_leaves() {
    cr!("707.2", "111.1", "611.2a", "614.12");
    ruling!(
        "Infinite Reflection",
        "If Infinite Reflection enters attached to a token creature, the nontoken creatures that become copies of that creature are not themselves tokens."
    );
    ruling!(
        "Infinite Reflection",
        "If Infinite Reflection leaves the battlefield or becomes attached to a different creature, none of the copy effects end. Your creatures will still be copies of whatever creature each was a copy of."
    );
    // On a Wolf token: the Grizzly Bears becomes a Wolf, not a token; so does a Hill
    // Giant entering.
    let mut t = TestGame::new(2);
    let wolf = crate::r_p023_common::wolf_token(&mut t, P0);
    let bears = t.battlefield(P0, "Grizzly Bears");
    infinite_reflection(&mut t, wolf);
    assert_eq!(name_of(&t, bears), "Wolf");
    assert!(!t.obj_now(bears).is_token());
    let giant = t.enter(P0, "Hill Giant");
    t.settle();
    let giant = t.g.current(giant);
    assert_eq!(name_of(&t, giant), "Wolf");
    assert!(!t.obj_now(giant).is_token());
    // Moved to the Serra Angel, then gone: the Bears and the Giant are still Wolves.
    let angel = t.battlefield(P0, "Serra Angel");
    let aura = t.named_on_battlefield("Infinite Reflection")[0];
    assert!(t.g.attach(aura, obj(angel)));
    t.g.recompute();
    assert_eq!(name_of(&t, bears), "Wolf");
    assert_eq!(name_of(&t, giant), "Wolf");
    crate::r_s02_common::destroy(&mut t, aura);
    assert_eq!(name_of(&t, bears), "Wolf");
    assert_eq!(name_of(&t, giant), "Wolf");
    assert_eq!(name_of(&t, angel), "Serra Angel");
}

// --- Tokens doubled by Doubling Season -------------------------------------------------------

#[test]
fn doubled_token_copies_are_all_sacrificed_or_exiled() {
    cr!("614.1a", "603.7c", "111.1");
    ruling!(
        "Jaxis, the Troublemaker",
        "If Jaxis, the Troublemaker's ability creates multiple tokens due to a replacement effect (such as the one Doubling Season creates), you'll sacrifice each of them at the beginning of the next end step."
    );
    ruling!(
        "Mordor on the March",
        "If Mordor on the March creates multiple tokens due to a replacement effect (such as the one Doubling Season creates), each of those tokens will gain haste, and you'll exile each of them at the beginning of the next end step."
    );
    ruling!(
        "Fable of the Mirror-Breaker // Reflection of Kiki-Jiki",
        "If Reflection of Kiki-Jiki's ability creates multiple tokens due to a replacement effect (such as the one Doubling Season creates), you'll sacrifice each of them."
    );
    supported("Doubling Season");
    let at_end_step = |t: &mut TestGame, toks: &[ObjectId]| {
        assert_eq!(toks.len(), 2, "{}", t.dump_log());
        for tok in toks {
            assert!(
                has_kw(t, *tok, mtg_engine::keywords::KeywordKind::Haste),
                "haste"
            );
        }
        t.advance_to(P0, Step::End);
        t.resolve_all();
        for tok in toks {
            assert!(!t.on_battlefield(*tok), "{}", t.dump_log());
        }
    };
    // Jaxis: "{R}, {T}, Discard a card: Create a token that's a copy of another target
    // creature you control. It gains haste and "When this token dies, draw a card."
    // Sacrifice it at the beginning of the next end step. Activate only as a sorcery."
    supported("Jaxis, the Troublemaker");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doubling Season");
    t.set_step(P0, Step::PrecombatMain);
    let jaxis = t.battlefield(P0, "Jaxis, the Troublemaker");
    let giant = t.battlefield(P0, "Hill Giant");
    let island = t.hand(P0, "Island");
    lands_for(&mut t, P0, "{R}");
    t.answer_choose(P0, &[obj(island)]);
    let before = t.g.battlefield.clone();
    t.answer_targets(P0, &[obj(giant)]);
    crate::r_s06_common::activate_containing(&mut t, P0, jaxis, "Create a token").unwrap();
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    let hand = t.hand_size(P0);
    at_end_step(&mut t, &toks);
    // Both were sacrificed: each died and drew a card.
    assert_eq!(t.hand_size(P0), hand + 2, "{}", t.dump_log());
    // Mordor on the March: "Exile a creature card from your graveyard. Create a token
    // that's a copy of it. It gains haste until end of turn. Exile it at the beginning of
    // the next end step."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doubling Season");
    t.set_step(P0, Step::PrecombatMain);
    let card = t.graveyard(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(card)]);
    let toks = cast_for_tokens(&mut t, P0, "Mordor on the March", "{3}{B}{R}", &[]);
    at_end_step(&mut t, &toks);
    assert!(t.in_exile("Hill Giant"));
    // Reflection of Kiki-Jiki: "{1}, {T}: Create a token that's a copy of another target
    // nonlegendary creature you control, except it has haste. Sacrifice it at the
    // beginning of the next end step."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doubling Season");
    t.set_step(P0, Step::PrecombatMain);
    let kiki = crate::r_p026_common::reflection(&mut t);
    let giant = t.battlefield(P0, "Hill Giant");
    lands_for(&mut t, P0, "{1}");
    let before = t.g.battlefield.clone();
    t.answer_targets(P0, &[obj(giant)]);
    crate::r_s06_common::activate_containing(&mut t, P0, kiki, "Create a token").unwrap();
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    at_end_step(&mut t, &toks);
}

// --- Myrkul, Lord of Bones -------------------------------------------------------------------

#[test]
fn myrkul_survives_lethal_damage_dealt_as_its_controllers_life_halves() {
    cr!("704.3", "704.5g", "702.12b", "119.3");
    ruling!(
        "Myrkul, Lord of Bones",
        "If Myrkul, Lord of Bones is dealt lethal damage at the same time that its controller's life total is reduced to less than or equal to half their starting life total, Myrkul will have indestructible at the time state-based actions are performed and survive."
    );
    // "As long as your life total is less than or equal to half your starting life total,
    // Myrkul has indestructible." P0 at 15 life; P1's Earthquake with X = 5 deals 5 damage
    // to Myrkul (a 7/5) and to P0 at once: P0 is at 10.
    supported("Myrkul, Lord of Bones");
    supported("Earthquake");
    let mut t = TestGame::new(2);
    t.g.players[P0.idx()].life = 15;
    let myrkul = t.battlefield(P0, "Myrkul, Lord of Bones");
    t.lands(P1, "Mountain", 6);
    let eq = t.hand(P1, "Earthquake");
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, eq).x(5).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 10);
    assert_eq!(t.obj_now(myrkul).damage, 5);
    assert!(t.on_battlefield(myrkul));
}

// --- Dermotaxi ---------------------------------------------------------------------------------

#[test]
fn dermotaxi_doesnt_crew() {
    cr!("702.122a", "702.122b", "707.2");
    ruling!(
        "Dermotaxi",
        "Dermotaxi does not have a crew ability. Activating the ability to have it become a copy of the exiled card doesn't count as crewing a Vehicle. If a creature you tap to activate the ability has a triggered ability that triggers whenever it crews a Vehicle (such as that of Gearshift Ace), that ability won't trigger."
    );
    // "Imprint — As this Vehicle enters, exile a creature card from a graveyard. / Tap two
    // untapped creatures you control: Until end of turn, this Vehicle becomes a copy of
    // the exiled card, except it's a Vehicle artifact in addition to its other types."
    // Gearshift Ace: "Whenever this creature crews a Vehicle, that Vehicle gains first
    // strike until end of turn."
    supported("Dermotaxi");
    supported("Gearshift Ace");
    let mut t = TestGame::new(2);
    let card = t.graveyard(P1, "Hill Giant");
    t.answer_choose(P0, &[obj(card)]);
    let taxi = t.enter(P0, "Dermotaxi");
    t.settle();
    t.clear_answers();
    let taxi = t.g.current(taxi);
    assert!(!has_kw(&t, taxi, mtg_engine::keywords::KeywordKind::Crew));
    let ace = t.battlefield(P0, "Gearshift Ace");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(ace), obj(bears)]);
    crate::r_s06_common::activate_containing(&mut t, P0, taxi, "Tap two untapped").unwrap();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "crews"), 0);
    t.resolve_all();
    assert!(t.obj_now(ace).tapped && t.obj_now(bears).tapped);
    assert_eq!(name_of(&t, taxi), "Hill Giant");
    assert!(!has_kw(
        &t,
        taxi,
        mtg_engine::keywords::KeywordKind::FirstStrike
    ));
}

// --- "As this enters, you may have it become a copy" from any zone ---------------------------

#[test]
fn cursed_mirror_becomes_a_copy_however_it_enters() {
    cr!("614.1c", "614.12", "707.2");
    // Regression: Cursed Mirror ("As this artifact enters, you may have it become a copy
    // of any creature on the battlefield until end of turn, except it has haste.") put
    // onto the battlefield from a graveyard (as a reanimation spell would) or by an
    // effect, not cast, becomes the copy too, until end of turn.
    supported("Cursed Mirror");
    for from_graveyard in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let giant = t.battlefield(P1, "Hill Giant");
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(giant)]);
        let m = if from_graveyard {
            let c = t.graveyard(P0, "Cursed Mirror");
            t.g.move_object(
                c,
                Zone::Battlefield,
                mtg_engine::events::MoveCause::Effect,
                Some(P0),
            );
            t.g.current(c)
        } else {
            let c = t.enter(P0, "Cursed Mirror");
            t.g.current(c)
        };
        t.settle();
        assert_eq!(
            name_of(&t, m),
            "Hill Giant",
            "from graveyard: {from_graveyard}"
        );
        assert!(has_kw(&t, m, mtg_engine::keywords::KeywordKind::Haste));
        assert_eq!(t.pt(m), (3, 3));
        t.set_step(P1, Step::End);
        t.advance_to(P0, Step::Upkeep);
        assert_eq!(name_of(&t, m), "Cursed Mirror");
    }
}

#[test]
fn cursed_mirror_copy_has_the_copied_enters_with_abilities_however_it_enters() {
    cr!("614.1c", "614.12", "707.2", "603.6a");
    ruling!(
        "Cursed Mirror",
        "Any \"As [this creature] enters the battlefield\" or \"[This creature] enters the battlefield with\" abilities of the copied creature will also work."
    );
    // Regression: the effect Cursed Mirror's "as this enters" ability creates is part of
    // the permanent as it would exist on the battlefield, so the replacement effects
    // applied after it see a copy of Aven Riftwatcher ("Vanishing 3": it enters with three
    // time counters), whether it's cast, put onto the battlefield from a graveyard or by an
    // effect. Its copied "When this creature enters or leaves the battlefield, you gain 2
    // life." triggers too.
    supported("Cursed Mirror");
    for how in ["cast", "graveyard", "effect"] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let rift = t.battlefield(P0, crate::r_p020_common::RIFTWATCHER);
        let life = t.life(P0);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(rift)]);
        let m = match how {
            "cast" => {
                t.lands(P0, "Mountain", 3);
                let c = t.hand(P0, "Cursed Mirror");
                t.cast(P0, c).go();
                c
            }
            "graveyard" => {
                let c = t.graveyard(P0, "Cursed Mirror");
                t.g.move_object(
                    c,
                    Zone::Battlefield,
                    mtg_engine::events::MoveCause::Effect,
                    Some(P0),
                );
                c
            }
            _ => t.enter(P0, "Cursed Mirror"),
        };
        t.settle();
        t.resolve_all();
        let m = t.g.current(m);
        assert_eq!(name_of(&t, m), crate::r_p020_common::RIFTWATCHER, "{how}");
        assert_eq!(t.counters(m, counters::TIME), 3, "{how}: no vanishing counters");
        assert_eq!(t.life(P0), life + 2, "{how}: {}", t.dump_log());
    }
}
