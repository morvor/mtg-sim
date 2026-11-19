//! Rulings batch P173 — the other rulings of the batch: sacrifice costs with colored
//! creatures (CR 601.2h, 105.2), casting and playing cards from exile and from the top of
//! the library with normal timing (CR 601.3, 305.2, 116.2a), choices on resolution (CR
//! 608.2c), returns of copies (CR 111.7, 707.2), cost reductions (CR 601.2f), abilities
//! that work only in a zone (CR 113.6), chapter abilities and affected sets (CR 611.2c),
//! emblems (CR 114.1), simultaneous entering (CR 603.6a), mana abilities with {T} (CR
//! 605.1a, 602.2b), "exile until" ending mid-resolution (CR 610.3c), searching (CR
//! 701.23b), targets among the creatures tapped for the cost (CR 601.2c, 601.2h), and
//! granted abilities on the stack (CR 113.7a).

use crate::r_p196_common::pump;
use crate::r_p197_common::{add_lore, blank_saga, walker};
use crate::r_s01_common::{
    asked_since, give_mana_for, stack_library, supported, tokens, triggers_on_stack,
};
use crate::r_s02_common::{can_activate, can_cast, can_play_land, destroy};
use crate::r_s04_common::add_mana;
use crate::r_s06_common::activate_containing;
use crate::r_s18_common::ADVENTURE;
use crate::r_s20_common::tap_for_mana;
use crate::r_s24_common::{enter_together, tapped};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn objs(ids: &[ObjectId]) -> Vec<Entity> {
    ids.iter().map(|i| obj(*i)).collect()
}

/// Whether the permanent has an ability whose text contains `needle`.
fn has_ability(t: &mut TestGame, id: ObjectId, needle: &str) -> bool {
    t.g.recompute();
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains(needle))
}

// --- The Heralds -------------------------------------------------------------------------------

#[test]
fn heralds_multicolored_creatures_pay_one_part_and_the_herald_can_be_sacrificed() {
    cr!("601.2h", "105.2", "602.2b");
    ruling!(
        "Behemoth's Herald",
        "You may sacrifice multicolored creatures as part of the cost of the search ability. If you sacrifice a multicolored creature this way, specify which part of the cost that creature is satisfying."
    );
    supported("Behemoth's Herald");
    // Behemoth's Herald (green) needs a red, a green and a white creature. A red-white
    // creature can't be both the red and the white one.
    let setup = |helpers: &[&str]| {
        let mut t = TestGame::new(2);
        let h = t.battlefield(P0, "Behemoth's Herald");
        for n in helpers {
            t.battlefield(P0, n);
        }
        t.battlefield(P0, "Forest");
        t.lands(P0, "Wastes", 2);
        stack_library(&mut t, P0, &["Godsire"]);
        (t, h)
    };
    let (mut t, h) = setup(&["Boros Swiftblade"]);
    assert!(!can_activate(&mut t, P0, h));
    // With a green-white one too: the Swiftblade is the red one, the Watchwolf the white
    // one, the Herald itself the green one.
    let (mut t, h) = setup(&["Boros Swiftblade", "Watchwolf"]);
    assert!(can_activate(&mut t, P0, h));
    activate_containing(&mut t, P0, h, "Search your library").expect("activates");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Godsire").len(), 1, "{}", t.dump_log());
    for n in ["Behemoth's Herald", "Boros Swiftblade", "Watchwolf"] {
        assert!(t.in_graveyard(P0, n), "{n}");
    }

    ruling!(
        "Angel's Herald",
        "You may sacrifice the Herald itself to help pay for the cost of its ability."
    );
    supported("Angel's Herald");
    // Angel's Herald (white) needs a green, a white and a blue creature.
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Angel's Herald");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Merfolk of the Pearl Trident");
    t.battlefield(P0, "Plains");
    t.lands(P0, "Wastes", 2);
    stack_library(&mut t, P0, &["Empyrial Archangel"]);
    activate_containing(&mut t, P0, h, "Search your library").expect("activates");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Empyrial Archangel").len(), 1);
    assert!(t.in_graveyard(P0, "Angel's Herald"));
}

// --- Normal timing for cards played from exile or the library ---------------------------------

#[test]
fn seek_the_beast_cards_are_played_with_normal_timing_and_costs() {
    cr!("305.2", "116.2a", "601.3", "715.3d");
    ruling!(
        "Questing Druid",
        "You pay all costs and follow all normal timing rules for cards played from exile with Seek the Beast. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
    );
    supported("Questing Druid // Seek the Beast");
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Forest", "Grizzly Bears"]);
    add_mana(&mut t, P0, ManaType::R, 2);
    let c = t.hand(P0, "Questing Druid // Seek the Beast");
    t.cast(P0, c).method(ADVENTURE).go();
    t.resolve_all();
    let (forest, bears) = (t.g.current(cards[0]), t.g.current(cards[1]));
    assert_eq!(t.zone(forest), Zone::Exile);
    // In combat: neither.
    t.set_step(P0, Step::BeginningOfCombat);
    t.lands(P0, "Forest", 2);
    assert!(!can_play_land(&mut t, P0, forest));
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
    // In the main phase with a spell on the stack: neither.
    t.set_step(P0, Step::PostcombatMain);
    add_mana(&mut t, P0, ManaType::R, 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    assert!(!can_play_land(&mut t, P0, forest));
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
    t.resolve_all();
    // Main phase, empty stack: both, paying the Bears' cost.
    assert!(can_play_land(&mut t, P0, forest));
    assert!(can_cast(&mut t, P0, bears, CastMethod::Normal));
    t.cast(P0, bears).go();
    assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 0);
}

#[test]
fn eladamri_and_elven_chorus_cast_creatures_from_the_top_with_normal_timing_only() {
    cr!("305.9", "601.3", "307.1", "116.2a");
    ruling!(
        "Eladamri, Korvecdal",
        "Because you never \"cast\" a land card, Eladamri, Korvecdal doesn't allow you to play a land creature (such as Dryad Arbor) from the top of your library."
    );
    supported("Eladamri, Korvecdal");
    supported("Dryad Arbor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Eladamri, Korvecdal");
    t.lands(P0, "Forest", 2);
    let arbor = t.library_top(P0, "Dryad Arbor");
    assert!(!can_play_land(&mut t, P0, arbor));
    assert!(!can_cast(&mut t, P0, arbor, CastMethod::Normal));
    t.g.players[0].library.retain(|c| *c != arbor);
    let bears = t.library_top(P0, "Grizzly Bears");
    assert!(can_cast(&mut t, P0, bears, CastMethod::Normal));

    ruling!(
        "Elven Chorus",
        "Elven Chorus doesn't change when you can cast creature spells. Normally, this means during your main phase when the stack is empty, although flash may change this."
    );
    supported("Elven Chorus");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elven Chorus");
    t.lands(P0, "Forest", 4);
    let bears = t.library_top(P0, "Grizzly Bears");
    assert!(can_cast(&mut t, P0, bears, CastMethod::Normal));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
    t.set_step(P1, Step::PrecombatMain);
    t.g.turn.priority = Some(P0);
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
    // A creature with flash can be cast then.
    t.g.players[0].library.retain(|c| *c != bears);
    let flash = t.library_top(P0, "Ambush Viper");
    assert!(can_cast(&mut t, P0, flash, CastMethod::Normal));
}

#[test]
fn summoning_materia_looks_at_the_top_card_without_priority() {
    cr!("401.5", "116.1", "117.1");
    ruling!(
        "Summoning Materia",
        "Summoning Materia lets you look at the top card of your library whenever you want (with one restriction—see below), even if you don't have priority. This action doesn't use the stack. Knowing what that card is becomes part of the information you have access to, just like you can look at the cards in your hand."
    );
    supported("Summoning Materia");
    let sees_top = |t: &TestGame| {
        t.g.library_top(P0)
            .is_some_and(|c| mtg_engine::zones::can_see_in_library(&t.g, P0, c))
    };
    let mut t = TestGame::new(2);
    t.library_top(P0, "Grizzly Bears");
    t.g.recompute();
    assert!(!sees_top(&t));
    t.battlefield(P0, "Summoning Materia");
    t.g.recompute();
    mtg_engine::zones::update_revealed_tops(&mut t.g);
    assert!(sees_top(&t));
    // On P1's turn with P1 holding priority, and with something on the stack.
    t.set_step(P1, Step::Upkeep);
    add_mana(&mut t, P1, ManaType::R, 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    assert_eq!(t.g.turn.priority, Some(P1));
    assert!(sees_top(&t));
    assert_eq!(t.stack_len(), 1, "looking used no stack object");
}

// --- Choices made on resolution ----------------------------------------------------------------

#[test]
fn scent_of_jasmine_reveals_on_resolution() {
    cr!("608.2c", "701.20a");
    ruling!(
        "Scent of Jasmine",
        "You reveal cards, and thereby the amount of life gained, on resolution."
    );
    supported("Scent of Jasmine");
    let mut t = TestGame::new(2);
    add_mana(&mut t, P0, ManaType::W, 1);
    let s = t.hand(P0, "Scent of Jasmine");
    let from = t.asked().len();
    t.cast(P0, s).go();
    assert!(
        asked_since(&t, from)
            .iter()
            .all(|(_, d)| !matches!(d, Decision::ChooseEntities { .. })),
        "nothing revealed as it's cast"
    );
    // White cards that come to the hand afterwards are revealed.
    let a = t.hand(P0, "Savannah Lions");
    let b = t.hand(P0, "Savannah Lions");
    t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &objs(&[a, b]));
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
}

#[test]
fn arachnus_spinner_may_fail_to_find_and_still_shuffles() {
    cr!("701.23b", "701.24a");
    ruling!(
        "Arachnus Spinner",
        "If you choose to search your library this way, you don't have to find a card named Arachnus Web, even if one is there. You'll still shuffle your library after searching it."
    );
    supported("Arachnus Spinner");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Arachnus Spinner");
    stack_library(&mut t, P0, &["Arachnus Web"]);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_choose(P0, &[obj(s)]); // the Spinner taps itself for the cost
    t.answer_yes(P0, true); // search the library
    t.answer_choose(P0, &[]); // and find nothing
    activate_containing(&mut t, P0, s, "Tap an untapped Spider").expect("activates");
    let shuffles = |t: &TestGame| {
        t.g.turn_events
            .iter()
            .chain(t.g.events.iter())
            .filter(|e| matches!(e, mtg_engine::events::Event::Shuffled { .. }))
            .count()
    };
    let before = shuffles(&t);
    t.resolve_all();
    assert!(t.named_on_battlefield("Arachnus Web").is_empty(), "{}", t.dump_log());
    assert_eq!(t.library_size(P0), 31);
    assert_eq!(shuffles(&t), before + 1);
}

// --- Copies returning ---------------------------------------------------------------------------

#[test]
fn enduring_vitality_copies_return_only_if_nontoken_and_only_as_enchantments() {
    cr!("111.7", "707.2", "603.6d", "205.1a");
    ruling!(
        "Enduring Vitality",
        "A token that's a copy of Enduring Vitality won't return to the battlefield when its last ability resolves."
    );
    supported("Enduring Vitality");
    supported("Cackling Counterpart");
    let mut t = TestGame::new(2);
    let ev = t.battlefield(P0, "Enduring Vitality");
    add_mana(&mut t, P0, ManaType::U, 3);
    let cc = t.hand(P0, "Cackling Counterpart");
    t.cast(P0, cc).target(ev).go();
    t.resolve_all();
    let token = tokens(&t, P0)[0];
    destroy(&mut t, token);
    assert_eq!(triggers_on_stack(&t, "return it to the battlefield"), 1);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    assert_eq!(t.named_on_battlefield("Enduring Vitality").len(), 1);

    ruling!(
        "Enduring Vitality",
        "If a nontoken permanent that's a copy of Enduring Vitality dies while it's a creature, it will return to the battlefield as an enchantment when its last ability resolves. It won't have any card types other than enchantment."
    );
    supported("Clone");
    let mut t = TestGame::new(2);
    let ev = t.battlefield(P0, "Enduring Vitality");
    give_mana_for(&mut t, P0, "Clone");
    let c = t.hand(P0, "Clone");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(ev)]);
    t.cast(P0, c).go();
    t.resolve_all();
    let clone = t.g.current(c);
    assert_eq!(t.obj_now(clone).chars.name.as_str(), "Enduring Vitality");
    destroy(&mut t, clone);
    // It returns, copying Enduring Vitality again: an enchantment only.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(ev)]);
    t.resolve_all();
    let back = t.g.current(c);
    assert!(t.on_battlefield(back), "{}", t.dump_log());
    let o = t.obj_now(back);
    assert_eq!(o.chars.card_types, CardTypeSet::single(CardType::Enchantment));
    assert_eq!(o.chars.name.as_str(), "Enduring Vitality");
}

// --- Cost reductions and abilities that work only from one zone --------------------------------

#[test]
fn embalmers_tools_reduces_embalm() {
    cr!("602.1", "702.128a", "601.2f");
    ruling!(
        "Embalmer's Tools",
        "Activated abilities contain a colon. They’re generally written “[Cost]: [Effect].” Some keyword abilities (such as embalm) are activated abilities and will have colons in their reminder text."
    );
    supported("Embalmer's Tools");
    supported("Anointer Priest");
    // Anointer Priest: embalm {3}{W}; with the Tools, {2}{W}.
    for tools in [false, true] {
        let mut t = TestGame::new(2);
        if tools {
            t.battlefield(P0, "Embalmer's Tools");
        }
        let p = t.graveyard(P0, "Anointer Priest");
        t.battlefield(P0, "Plains");
        t.lands(P0, "Wastes", 2);
        assert_eq!(can_activate(&mut t, P0, p), tools);
        if tools {
            activate_containing(&mut t, P0, p, "Embalm").expect("embalm");
            t.resolve_all();
            assert_eq!(tokens(&t, P0).len(), 1);
        }
    }
}

#[test]
fn deathless_ancient_returns_only_from_the_graveyard() {
    cr!("113.6m", "602.1");
    ruling!(
        "Deathless Ancient",
        "Deathless Ancient's last ability can be activated only while it's in your graveyard."
    );
    let mut t = TestGame::new(2);
    let da = t.battlefield(P0, "Deathless Ancient");
    for _ in 0..3 {
        t.battlefield(P0, "Vampire Noble");
    }
    assert!(!can_activate(&mut t, P0, da));
    assert!(activate_containing(&mut t, P0, da, "Tap three untapped Vampires").is_err());
}

// --- Affected sets ---------------------------------------------------------------------------

#[test]
fn song_of_freyalise_affects_only_creatures_there_as_each_chapter_resolves() {
    cr!("611.2c", "714.2b");
    ruling!(
        "Song of Freyalise",
        "Each of Song of Freyalise’s chapter abilities affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won’t gain abilities or get a +1/+1 counter."
    );
    let mut t = TestGame::new(2);
    let saga = blank_saga(&mut t, P0, "Song of Freyalise");
    let early = t.battlefield(P0, "Grizzly Bears");
    add_lore(&mut t, saga, 1);
    t.resolve_all();
    let late = t.battlefield(P0, "Grizzly Bears");
    assert!(has_ability(&mut t, early, "Add one mana of any color"));
    assert!(!has_ability(&mut t, late, "Add one mana of any color"));
    // Chapter III.
    add_lore(&mut t, saga, 2);
    t.resolve_all();
    let later = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.counters(early, counters::PLUS1), 1);
    assert_eq!(t.counters(late, counters::PLUS1), 1);
    assert_eq!(t.counters(later, counters::PLUS1), 0);
    assert!(crate::r_s06_common::has_kw(&t, late, KeywordKind::Indestructible));
    assert!(!crate::r_s06_common::has_kw(&t, later, KeywordKind::Indestructible));
}

// --- Emblems, simultaneous entering, triggers ---------------------------------------------------

#[test]
fn chandras_emblem_is_colorless_and_damages_protection_from_red() {
    cr!("114.4", "105.2c", "702.16b");
    ruling!(
        "Chandra, Spark Hunter",
        "Emblems are colorless. This means that a permanent with protection from red can be the target of the emblem’s triggered ability."
    );
    supported("Chandra, Spark Hunter");
    supported("Paladin en-Vec");
    let mut t = TestGame::new(2);
    let c = walker(&mut t, P0, "Chandra, Spark Hunter", 7);
    activate_containing(&mut t, P0, c, "emblem").expect("-7");
    t.resolve_all();
    let paladin = t.battlefield(P1, "Paladin en-Vec");
    t.answer_targets(P0, &[obj(paladin)]);
    t.enter(P0, "Ornithopter");
    t.resolve_all();
    assert!(!t.on_battlefield(paladin), "{}", t.dump_log());
}

#[test]
fn high_perfect_morcant_triggers_for_each_elf_entering_with_it() {
    cr!("603.6a", "603.2");
    ruling!(
        "High Perfect Morcant",
        "If High Perfect Morcant enters at the same time as one or more other Elves you control, its first ability will trigger for each of those other Elves as well as itself."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    enter_together(
        &mut t,
        &[
            (P0, "High Perfect Morcant"),
            (P0, "Llanowar Elves"),
            (P0, "Llanowar Elves"),
        ],
    );
    assert_eq!(t.stack_len(), 3, "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.counters(wurm, counters::MINUS1), 3);
}

#[test]
fn saradoc_triggers_for_itself_whatever_its_power() {
    cr!("603.6a", "603.2");
    ruling!(
        "Saradoc, Master of Buckland",
        "If Saradoc, Master of Buckland's power is greater than 2 as it enters the battlefield, its first ability will still trigger."
    );
    supported("Saradoc, Master of Buckland");
    supported("Glorious Anthem");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    let s = t.enter(P0, "Saradoc, Master of Buckland");
    assert_eq!(t.pt(s).0, 3);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    // Another creature with power 3 doesn't trigger it.
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

// --- Mana abilities with {T} -----------------------------------------------------------------

#[test]
fn cryptolith_rite_two_mana_abilities_but_one_tap() {
    cr!("602.2b", "605.1a", "302.6");
    ruling!(
        "Cryptolith Rite",
        "If a creature has multiple mana abilities with a cost of {T}, such as if you control two Cryptolith Rites, you can only activate one of them at a time. Tapping the creature doesn’t produce multiple mana."
    );
    supported("Cryptolith Rite");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cryptolith Rite");
    t.battlefield(P0, "Cryptolith Rite");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    let n = t
        .obj_now(b)
        .chars
        .abilities
        .iter()
        .filter(|a| a.text.contains("Add one mana of any color"))
        .count();
    assert_eq!(n, 2);
    assert!(tap_for_mana(&mut t, P0, b, "Add one mana"));
    assert!(!tap_for_mana(&mut t, P0, b, "Add one mana"));
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
}

#[test]
fn heirloom_epic_creatures_used_for_mana_cant_also_pay_by_tapping() {
    cr!("601.2g", "601.2h", "602.2b");
    ruling!(
        "Heirloom Epic",
        "If a creature you control has a mana ability with {T} in the cost, activating that ability to pay the activation cost of Heirloom Epic’s ability will result in the creature being tapped before you pay the ability’s costs."
    );
    supported("Heirloom Epic");
    supported("Blood Pet");
    // {4}: two Wastes and Llanowar Elves can pay {3} at most (the Elves either add {G} or
    // are tapped for {1}); a third Wastes makes it.
    for (wastes, creature, ok) in [
        (2, "Llanowar Elves", false),
        (3, "Llanowar Elves", true),
        // Blood Pet: "Sacrifice this creature: Add {B}." Sacrificed, it can't be tapped.
        (2, "Blood Pet", false),
        (3, "Blood Pet", true),
    ] {
        let mut t = TestGame::new(2);
        let e = t.battlefield(P0, "Heirloom Epic");
        t.battlefield(P0, creature);
        t.lands(P0, "Wastes", wastes);
        let r = activate_containing(&mut t, P0, e, "Draw a card");
        assert_eq!(r.is_ok(), ok, "{wastes} Wastes and {creature}: {r:?}");
    }
}

#[test]
fn explosive_singularity_taps_any_number_but_costs_at_least_rr() {
    cr!("601.2b", "601.2f", "118.7");
    ruling!(
        "Explosive Singularity",
        "Explosive Singularity's additional cost can't reduce its cost below {R}{R}. You can tap any number of untapped creatures, however, even if they won't reduce the cost further."
    );
    supported("Explosive Singularity");
    let mut t = TestGame::new(2);
    let cs: Vec<ObjectId> = (0..10)
        .map(|_| t.battlefield(P0, "Grizzly Bears"))
        .collect();
    let ms = t.lands(P0, "Mountain", 3);
    let s = t.hand(P0, "Explosive Singularity");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(10));
    t.answer_choose(P0, &objs(&cs));
    t.cast(P0, s).target(P1).go();
    assert!(cs.iter().all(|c| tapped(&t, *c)));
    assert_eq!(ms.iter().filter(|m| tapped(&t, **m)).count(), 2, "{{R}}{{R}}");
    t.resolve_all();
    assert_eq!(t.life(P1), 10);
}

// --- Exile until ---------------------------------------------------------------------------

#[test]
fn collective_effort_exiled_creatures_return_before_the_counters() {
    cr!("610.3c", "608.2c", "702.120a");
    ruling!(
        "Collective Effort",
        "If you choose the first and third modes, and destroying the creature ends an \"exile until\" effect (such as that of Lunarch Inquisitors), the creature will return to the battlefield before you put +1/+1 counters on creatures. The same is true if you choose the second and third modes and destroying the enchantment ends such an effect."
    );
    supported("Collective Effort");
    supported("Banishing Light");
    supported("Banisher Priest");
    for (exiler, modes) in [("Banisher Priest", [0, 2]), ("Banishing Light", [1, 2])] {
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Grizzly Bears");
        let tapper = t.battlefield(P0, "Grizzly Bears");
        t.answer_targets(P1, &[obj(mine)]);
        let ex = t.enter(P1, exiler);
        t.resolve_all();
        assert_eq!(t.zone(mine), Zone::Exile, "{exiler}");
        if exiler == "Banisher Priest" {
            pump(&mut t, ex, 2, 0);
        }
        t.lands(P0, "Plains", 3);
        let c = t.hand(P0, "Collective Effort");
        t.answer_choose(P0, &[obj(tapper)]);
        t.cast(P0, c)
            .modes(&modes)
            .target(ex)
            .target(Entity::Player(P0))
            .go();
        assert!(tapped(&t, tapper));
        t.resolve_all();
        let back = t.g.current(mine);
        assert!(t.on_battlefield(back), "{exiler}: {}", t.dump_log());
        assert_eq!(t.counters(back, counters::PLUS1), 1, "{exiler}");
        assert_eq!(t.counters(tapper, counters::PLUS1), 1, "{exiler}");
    }
}

// --- Targets and costs --------------------------------------------------------------------------

#[test]
fn hand_of_justice_can_destroy_a_creature_it_tapped() {
    cr!("601.2c", "601.2h", "608.2b");
    ruling!("Hand of Justice", "It can destroy one of the three creatures it taps.");
    supported("Hand of Justice");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hand of Justice");
    let lions: Vec<ObjectId> = (0..3)
        .map(|_| t.battlefield(P0, "Savannah Lions"))
        .collect();
    t.answer_targets(P0, &[obj(lions[0])]);
    t.answer_choose(P0, &objs(&lions));
    activate_containing(&mut t, P0, h, "Destroy target creature").expect("activates");
    assert!(lions.iter().all(|l| tapped(&t, *l)));
    t.resolve_all();
    assert!(!t.on_battlefield(lions[0]));
    assert!(t.on_battlefield(lions[1]) && t.on_battlefield(lions[2]));
}

#[test]
fn frondland_felidar_granted_ability_resolves_without_the_felidar() {
    cr!("113.7a", "602.2");
    ruling!(
        "Frondland Felidar",
        "Once you’ve activated the ability granted by Frondland Felidar, causing the creature to lose that ability (such as by removing Frondland Felidar from the battlefield) won’t affect the ability on the stack."
    );
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Frondland Felidar");
    let angel = t.battlefield(P0, "Serra Angel");
    t.lands(P0, "Wastes", 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, angel, "Tap target creature").expect("granted");
    destroy(&mut t, f);
    assert!(!has_ability(&mut t, angel, "Tap target creature"));
    t.resolve_all();
    assert!(tapped(&t, bears));
}

