//! Rulings batch P066 — cards that let you cast (or play) cards from exile: timing
//! permissions (CR 601.3), cards left in exile, spells cast during a resolution (CR
//! 608.2g), "cast" vs "play" (CR 305.9), triggers resolving before the spell that caused
//! them, and face-up vs face-down exile (CR 406.3).

use crate::r_p066_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, stack_library, supported};
use crate::r_s02_common::destroy;
use crate::r_s21_common::castable;
use crate::r_s25_common::cast_new;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn can_play_land(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    crate::r_s02_common::can_play_land(t, p, t.g.current(card))
}

fn in_exile(t: &TestGame, id: ObjectId) -> bool {
    t.zone(id) == Zone::Exile
}

// ---------------------------------------------------------------------------
// Permissions don't change timing
// ---------------------------------------------------------------------------

#[test]
fn abstruse_appropriation_doesnt_change_when_you_can_cast_the_card() {
    cr!("601.3", "307.1");
    ruling!(
        "Abstruse Appropriation",
        "Abstruse Appropriation doesn't change when you can cast the exiled card. For example, if you exile a creature card without flash, you can cast it only during your main phase when the stack is empty."
    );
    supported("Abstruse Appropriation");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Abstruse Appropriation", &[o(bears)]);
    t.resolve_all();
    assert!(in_exile(&t, bears));
    // Colorless mana may be spent as though it were green.
    mana(&mut t, P0, ManaType::C, 2);
    assert!(castable(&mut t, P0, bears), "main phase, empty stack");
    // Not with something on the stack.
    let bolt = t.hand(P1, "Lightning Bolt");
    mana(&mut t, P1, ManaType::R, 1);
    t.g.turn.priority = Some(P1);
    t.cast(P1, bolt).target(P0).go();
    assert!(!castable(&mut t, P0, bears), "stack not empty");
    t.resolve_all();
    // Not in combat, nor in the opponent's turn.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!castable(&mut t, P0, bears), "combat");
    t.set_step(P1, Step::PrecombatMain);
    assert!(!castable(&mut t, P0, bears), "opponent's turn");
    t.set_step(P0, Step::PostcombatMain);
    assert!(castable(&mut t, P0, bears));
    t.cast(P0, t.g.current(bears)).go();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    assert!(t.on_battlefield(bears));
}

#[test]
fn blightwing_bandit_doesnt_change_when_you_can_cast_the_card() {
    cr!("601.3", "307.1");
    ruling!(
        "Blightwing Bandit",
        "Blightwing Bandit doesn't change when you can cast a spell. For example, if you exile a creature card without flash, you can cast it only during your main phase when the stack is empty."
    );
    supported("Blightwing Bandit");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blightwing Bandit");
    let bears = t.library_top(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    mana(&mut t, P0, ManaType::R, 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert!(in_exile(&t, bears));
    mana(&mut t, P0, ManaType::C, 2);
    // During P1's turn: no.
    assert!(!castable(&mut t, P0, bears));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!castable(&mut t, P0, bears));
    t.set_step(P0, Step::PostcombatMain);
    assert!(castable(&mut t, P0, bears));
}

// ---------------------------------------------------------------------------
// Cards not cast remain in exile
// ---------------------------------------------------------------------------

#[test]
fn etali_cards_not_cast_stay_exiled_and_cant_be_cast_later() {
    cr!("608.2g", "400.7");
    ruling!(
        "Etali, Primal Storm",
        "Any cards not cast, including land cards, remain in exile. They can't be cast on later turns, even if Etali attacks again."
    );
    supported("Etali, Primal Storm");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Etali, Primal Storm");
    let forest = t.library_top(P0, "Forest");
    let bears = t.library_top(P1, "Grizzly Bears");
    attack_with(&mut t, &[(e, Entity::Player(P1))]);
    t.answer_choose_moved(P0, &[]);
    t.resolve_all();
    assert!(in_exile(&t, forest));
    assert!(in_exile(&t, bears));
    block_and_finish(&mut t, P1, &[]);
    // Not this turn after the ability resolved, nor later turns.
    t.advance_to(P0, Step::PostcombatMain);
    mana(&mut t, P0, ManaType::G, 2);
    assert!(!castable(&mut t, P0, bears));
    assert!(!can_play_land(&mut t, P0, forest));
    t.advance_to(P0, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    mana(&mut t, P0, ManaType::G, 2);
    assert!(in_exile(&t, bears));
    assert!(!castable(&mut t, P0, bears));
    // Etali attacking again exiles new cards; the old ones still can't be cast.
    let e = t.g.current(e);
    t.library_top(P1, "Hill Giant");
    attack_with(&mut t, &[(e, Entity::Player(P1))]);
    let from = t.asked().len();
    t.resolve_all();
    let offered = crate::r_s03_common::choice_candidates(&t, from, "");
    assert!(offered.iter().all(|c| !c.contains(&o(t.g.current(bears)))));
}

#[test]
fn villainous_wealth_cards_not_cast_remain_exiled() {
    cr!("608.2g");
    ruling!(
        "Villainous Wealth",
        "Any cards you don't cast this way will remain in exile."
    );
    supported("Villainous Wealth");
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P1, &["Grizzly Bears", "Hill Giant", "Forest"]);
    for ty in [ManaType::B, ManaType::G, ManaType::U] {
        mana(&mut t, P0, ty, 1);
    }
    mana(&mut t, P0, ManaType::C, 3);
    let w = t.hand(P0, "Villainous Wealth");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose_moved(P0, &[o(lib[0])]);
    t.cast(P0, w).x(3).go();
    t.resolve_all();
    assert_eq!(t.obj_now(lib[0]).controller, P0);
    assert!(t.on_battlefield(lib[0]));
    assert!(in_exile(&t, lib[1]));
    assert!(in_exile(&t, lib[2]));
    t.advance_to(P0, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(in_exile(&t, lib[1]));
    mana(&mut t, P0, ManaType::R, 4);
    assert!(!castable(&mut t, P0, lib[1]));
}

#[test]
fn fallen_shinobi_cards_not_played_remain_exiled() {
    cr!("611.2a");
    ruling!("Fallen Shinobi", "Any cards you don't cast will remain in exile.");
    supported("Fallen Shinobi");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Fallen Shinobi");
    let lib = stack_library(&mut t, P1, &["Grizzly Bears", "Forest"]);
    attack_with(&mut t, &[(s, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert!(in_exile(&t, lib[0]));
    assert!(in_exile(&t, lib[1]));
    t.advance_to(P0, Step::PostcombatMain);
    assert!(castable(&mut t, P0, lib[0]), "this turn it may be cast");
    assert!(can_play_land(&mut t, P0, lib[1]));
    t.advance_to(P0, Step::PrecombatMain);
    assert!(in_exile(&t, lib[0]));
    assert!(in_exile(&t, lib[1]));
    mana(&mut t, P0, ManaType::G, 2);
    assert!(!castable(&mut t, P0, lib[0]));
    assert!(!can_play_land(&mut t, P0, lib[1]));
}

#[test]
fn izzet_chemister_cards_not_cast_remain_exiled() {
    cr!("608.2g");
    ruling!("Izzet Chemister", "Any of the cards you don't cast will remain exiled.");
    supported("Izzet Chemister");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Izzet Chemister");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let div = t.graveyard(P0, "Divination");
    for card in [bolt, div] {
        mana(&mut t, P0, ManaType::R, 1);
        t.activate(P0, c, 0, &[o(card)]).unwrap();
        t.resolve_all();
        t.g.untap(c);
    }
    assert!(in_exile(&t, bolt) && in_exile(&t, div));
    mana(&mut t, P0, ManaType::R, 2);
    let hand = t.hand_size(P0);
    // Cast only Divination.
    t.answer_choose_moved(P0, &[o(t.g.current(div))]);
    t.activate(P0, c, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert!(in_exile(&t, bolt));
    assert_eq!(t.life(P1), 20);
}

// ---------------------------------------------------------------------------
// "Cast" isn't "play"; permissions that outlast their source
// ---------------------------------------------------------------------------

#[test]
fn robber_of_the_rich_cant_play_an_exiled_land() {
    cr!("305.9", "601.3");
    ruling!(
        "Robber of the Rich",
        "An effect that instructs you to “cast” a card doesn't allow you to play lands."
    );
    supported("Robber of the Rich");
    for top in ["Forest", "Grizzly Bears"] {
        let mut t = TestGame::new(2);
        let r = t.battlefield(P0, "Robber of the Rich");
        for _ in 0..3 {
            t.hand(P1, "Island");
        }
        let card = t.library_top(P1, top);
        attack_with(&mut t, &[(r, Entity::Player(P1))]);
        t.resolve_all();
        assert!(in_exile(&t, card), "{top}");
        block_and_finish(&mut t, P1, &[]);
        t.advance_to(P0, Step::PostcombatMain);
        if top == "Forest" {
            assert!(!can_play_land(&mut t, P0, card));
        } else {
            mana(&mut t, P0, ManaType::C, 2);
            assert!(castable(&mut t, P0, card));
        }
    }
}

#[test]
fn fiend_of_the_shadows_card_stays_playable_after_fiend_leaves_or_changes_control() {
    cr!("610.3", "611.2a");
    ruling!(
        "Fiend of the Shadows",
        "As long as the card remains exiled, you may still play it, even if Fiend of the Shadows leaves the battlefield or changes controllers."
    );
    supported("Fiend of the Shadows");
    for leaves in [true, false] {
        let mut t = TestGame::new(2);
        let f = t.battlefield(P0, "Fiend of the Shadows");
        let bears = t.hand(P1, "Grizzly Bears");
        attack_with(&mut t, &[(f, Entity::Player(P1))]);
        t.answer_choose(P1, &[o(bears)]);
        block_and_finish(&mut t, P1, &[]);
        t.resolve_all();
        assert!(in_exile(&t, bears));
        t.advance_to(P0, Step::PostcombatMain);
        if leaves {
            destroy(&mut t, f);
        } else {
            crate::r_s06_common::give_control(&mut t, f, P1);
        }
        mana(&mut t, P0, ManaType::G, 2);
        assert!(castable(&mut t, P0, bears), "leaves: {leaves}");
        // Still on a later turn, and P1 (now controlling the Fiend) can't.
        t.advance_to(P0, Step::PrecombatMain);
        mana(&mut t, P0, ManaType::G, 2);
        assert!(castable(&mut t, P0, bears), "leaves: {leaves}");
        t.set_step(P1, Step::PrecombatMain);
        mana(&mut t, P1, ManaType::G, 2);
        assert!(!castable(&mut t, P1, bears));
    }
}

#[test]
fn codie_stops_only_permanent_spells() {
    cr!("101.2", "305.9");
    ruling!(
        "Codie, Vociferous Codex",
        "A permanent spell is an artifact, creature, enchantment, or planeswalker spell. You may still play lands and cast instant and sorcery spells. Other effects can still put permanents onto the battlefield."
    );
    supported("Codie, Vociferous Codex");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Codie, Vociferous Codex");
    t.lands(P0, "Plains", 6);
    for name in ["Grizzly Bears", "Bonesplitter", "Glorious Anthem", "Domri Rade"] {
        let c = t.hand(P0, name);
        mana(&mut t, P0, ManaType::R, 3);
        mana(&mut t, P0, ManaType::G, 3);
        assert!(!castable(&mut t, P0, c), "{name}");
    }
    let bolt = t.hand(P0, "Lightning Bolt");
    let div = t.hand(P0, "Divination");
    mana(&mut t, P0, ManaType::U, 3);
    assert!(castable(&mut t, P0, bolt));
    assert!(castable(&mut t, P0, div));
    let forest = t.hand(P0, "Forest");
    assert!(can_play_land(&mut t, P0, forest));
    // An instant that creates tokens; a land put onto the battlefield.
    cast_new(&mut t, P0, "Raise the Alarm", &[]);
    t.resolve_all();
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Soldier").len(), 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(forest)]);
    cast_new(&mut t, P0, "Swell of Growth", &[o(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(forest)));
}

#[test]
fn sweet_gum_recluse_can_target_a_permanent_that_entered_as_a_noncreature() {
    cr!("115.1a", "603.6a");
    ruling!(
        "Sweet-Gum Recluse",
        "A permanent that wasn't a creature as it entered the battlefield this turn but is currently a creature can be chosen as a target of Sweet-Gum Recluse's last ability."
    );
    supported("Sweet-Gum Recluse");
    let mut t = TestGame::new(2);
    // A land that entered this turn, then became a creature (Tawnos's Tinkering).
    let land = t.enter(P0, "Forest");
    let old = t.battlefield(P0, "Hill Giant");
    t.g.objects[old.0 as usize].entered_turn = 0;
    t.lands(P0, "Forest", 4);
    cast_new(&mut t, P0, "Tawnos's Tinkering", &[o(land)]);
    t.resolve_all();
    assert!(t.obj_now(land).is(mtg_engine::types::CardType::Creature));
    t.answer_targets(P0, &[o(land)]);
    let from = t.asked().len();
    let r = t.enter(P0, "Sweet-Gum Recluse");
    t.resolve_all();
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(offered[0].contains(&o(land)));
    assert!(offered[0].contains(&o(r)), "itself entered this turn");
    assert!(!offered[0].contains(&o(old)), "an older creature");
    assert_eq!(t.counters(land, "+1/+1"), 5);
}

// ---------------------------------------------------------------------------
// Triggers resolving before the spell that caused them
// ---------------------------------------------------------------------------

#[test]
fn abigale_becomes_prepared_even_if_the_creature_spell_is_countered() {
    cr!("603.3", "608.1", "722.3a");
    ruling!(
        "Abigale, Poet Laureate // Heroic Stanza",
        "Abigale's ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    supported("Abigale, Poet Laureate // Heroic Stanza");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Abigale, Poet Laureate // Heroic Stanza");
    let spell = cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_ne!(*t.g.stack.last().unwrap(), spell, "the trigger is above the spell");
    t.g.counter(spell, None);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.obj_now(a).prepared.is_some());
}

#[test]
fn jodahs_ability_and_its_spell_resolve_before_the_original_even_if_countered() {
    cr!("603.3", "608.2g", "405.5");
    ruling!(
        "Jodah, the Unifier",
        "Both Jodah, the Unifier’s triggered ability and the spell cast with that ability resolve before the spell that caused the ability to trigger, even if the original spell was countered in response."
    );
    supported("Jodah, the Unifier");
    for countered in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Jodah, the Unifier");
        let lib = stack_library(&mut t, P0, &["Isamaru, Hound of Konda"]);
        let original = t.hand(P0, "Sun Quan, Lord of Wu");
        mana(&mut t, P0, ManaType::U, 2);
        mana(&mut t, P0, ManaType::C, 4);
        t.answer_yes(P0, true);
        let spell = t.cast(P0, original).go();
        t.settle();
        assert_eq!(t.stack_len(), 2);
        if countered {
            t.g.counter(spell, None);
        }
        t.resolve();
        // Isamaru's spell is on top, above the original spell (if it's still there).
        let isamaru = t.g.current(lib[0]);
        assert_eq!(t.g.stack.last(), Some(&isamaru), "countered: {countered}");
        assert_eq!(t.stack_len(), if countered { 1 } else { 2 });
        t.resolve();
        assert!(t.on_battlefield(isamaru));
        assert_eq!(
            t.named_on_battlefield("Sun Quan, Lord of Wu").is_empty(),
            true,
            "the original resolved first"
        );
        t.resolve_all();
        assert_eq!(
            t.named_on_battlefield("Sun Quan, Lord of Wu").len(),
            if countered { 0 } else { 1 }
        );
    }
}

#[test]
fn enigma_sphinx_removed_from_the_graveyard_isnt_put_into_the_library() {
    cr!("603.6c", "400.7", "608.2b");
    ruling!(
        "Enigma Sphinx",
        "After Enigma Sphinx's middle ability triggers, if Enigma Sphinx is removed from your graveyard in response, it won't be put into your library."
    );
    supported("Enigma Sphinx");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Enigma Sphinx");
    let lib = t.library_size(P0);
    destroy(&mut t, s);
    assert_eq!(t.stack_len(), 1);
    crate::r_s05_common::move_to(&mut t, s, Zone::Exile);
    t.resolve_all();
    assert!(in_exile(&t, s));
    assert_eq!(t.library_size(P0), lib);
    // Otherwise, it's put third from the top.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Enigma Sphinx");
    destroy(&mut t, s);
    t.resolve_all();
    let s = t.g.current(s);
    let library = &t.g.player(P0).library;
    assert_eq!(library[library.len() - 3], s);
}

#[test]
fn bre_checks_life_gained_as_the_end_step_starts() {
    cr!("603.4", "513.1");
    ruling!(
        "Bre of Clan Stoutarm",
        "Bre's last ability will check as your end step starts to see if you gained life this turn. If you didn't, the ability won't trigger at all. Gaining life during your end step won't cause the ability to trigger."
    );
    supported("Bre of Clan Stoutarm");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bre of Clan Stoutarm");
    stack_library(&mut t, P0, &["Grizzly Bears"]);
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0, "it triggered without life gained");
    t.g.gain_life(P0, 3);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0, "gaining life in the end step triggered it");
    t.advance_to(P1, Step::Upkeep);
    assert!(t.g.find_in_zone(Zone::Exile, "Grizzly Bears").is_empty());
    // Control: with life gained before the end step, it triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bre of Clan Stoutarm");
    stack_library(&mut t, P0, &["Grizzly Bears"]);
    t.set_step(P0, Step::PostcombatMain);
    t.g.gain_life(P0, 3);
    t.g.flush_events();
    t.settle();
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn primordial_mist_exiling_a_face_down_card_doesnt_turn_it_face_up() {
    cr!("708.8", "702.37e");
    ruling!(
        "Primordial Mist",
        "Activating Primordial Mist's last ability doesn't trigger abilities that trigger \"when [something] is turned face up.\""
    );
    supported("Primordial Mist");
    supported("Fathom Seer");
    let mut t = TestGame::new(2);
    let mist = t.battlefield(P0, "Primordial Mist");
    // Fathom Seer: "When this creature is turned face up, draw two cards."
    let m = crate::r_s11_common::manifest_card(&mut t, P0, "Fathom Seer");
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[o(m)]);
    t.activate(P0, mist, 0, &[]).unwrap();
    t.resolve_all();
    let card = t.g.current(m);
    assert!(in_exile(&t, card));
    assert!(!t.obj(card).face_down);
    assert_eq!(t.hand_size(P0), hand);
    assert!(!crate::r_s11_common::turned_face_up_this_turn(&t));
}

#[test]
fn snow_mana_spent_as_any_color_makes_the_spell_a_legal_action() {
    cr!("609.4b", "601.2f");
    // Draugr Necromancer: "... you may spend mana from snow sources as though it were mana
    // of any color to cast those spells." With only blue snow mana, P1's exiled Grizzly
    // Bears ({1}{G}) is among P0's legal actions, and can be cast. (Like Abstruse
    // Appropriation's colorless mana above, this checks the permission's spending terms
    // as the list of legal actions is made.)
    supported("Draugr Necromancer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Draugr Necromancer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    assert!(in_exile(&t, bears));
    assert_eq!(t.counters(bears, "ice"), 1);
    let islands = t.lands(P0, "Snow-Covered Island", 2);
    assert!(castable(&mut t, P0, bears));
    t.cast(P0, t.g.current(bears)).go();
    assert!(islands.iter().all(|i| tapped(&t, *i)));
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // Non-snow blue mana can't pay the {G}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Draugr Necromancer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    t.lands(P0, "Island", 2);
    assert!(!castable(&mut t, P0, bears));
}
