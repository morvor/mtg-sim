//! Rulings batch P068 — cards cast while an effect resolves (CR 608.2g): the card is cast
//! then or not at all, type-based timing permissions don't apply (CR 307.1, 302.1) but
//! other casting restrictions do (CR 601.2, 101.2), and the spell resolves before the
//! spell or ability under it (CR 405.5, 608.1).

use crate::r_p068_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, stack_library, supported};
use mtg_engine::mana::ManaType::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Asserts that P0 can't cast `card` in its own main phase with an empty stack and plenty
/// of mana: the permission was only for the resolution.
fn not_castable_later(t: &mut TestGame, card: ObjectId) {
    assert!(exiled(t, card), "{}", t.dump_log());
    t.g.stack.clear();
    t.set_step(P0, Step::PostcombatMain);
    for ty in [W, U, B, R, G] {
        mana(t, P0, ty, 2);
    }
    mana(t, P0, C, 6);
    assert!(!castable(t, P0, card), "{}", t.dump_log());
}

/// Whether `log` shows `first` before `then`.
fn before(log: &str, first: &str, then: &str) -> bool {
    match (log.find(first), log.find(then)) {
        (Some(a), Some(b)) => a < b,
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Jeleva
// ---------------------------------------------------------------------------

/// Casts Jeleva from P0's hand with four mana and resolves it and its trigger.
fn cast_jeleva(t: &mut TestGame) -> ObjectId {
    let j = t.hand(P0, "Jeleva, Nephalia's Scourge");
    pool(t, P0, &[U, B, R, C]);
    t.cast(P0, j).go();
    t.resolve_all();
    let j = t.g.current(j);
    t.g.obj_mut(j).summoning_sick = false;
    j
}

#[test]
fn jeleva_casts_only_while_its_attack_trigger_resolves() {
    cr!("608.2g", "603.2", "307.1");
    ruling!(
        "Jeleva, Nephalia's Scourge",
        "If you cast an instant or sorcery card using Jeleva's last ability, you do so while that ability is resolving. If you choose not to (or you can't), you won't get a chance to cast it later unless the ability triggers again."
    );
    supported("Jeleva, Nephalia's Scourge");
    let mut t = TestGame::new(2);
    let div = t.library_top(P0, "Divination");
    let j = cast_jeleva(&mut t);
    assert!(exiled(&t, div));
    // The first attack: P0 casts nothing.
    t.answer_choose(P0, &[]);
    attack_with(&mut t, &[(j, Entity::Player(P1))]);
    t.resolve_all();
    assert!(exiled(&t, div));
    block_and_finish(&mut t, P1, &[]);
    t.set_step(P0, Step::PostcombatMain);
    mana(&mut t, P0, U, 3);
    assert!(!castable(&mut t, P0, div), "{}", t.dump_log());
    // The ability triggers again (another combat): Divination is cast, during combat.
    t.g.obj_mut(j).tapped = false;
    let hand = t.hand_size(P0);
    t.answer_choose_moved(P0, &[o(div)]);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(j, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Divination"), "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), hand + 2);
}

// ---------------------------------------------------------------------------
// Dance with Calamity, Etali, Primal Storm, Hazoret's Undying Fury, Ashiok
// ---------------------------------------------------------------------------

#[test]
fn dance_with_calamity_casts_during_its_resolution_ignoring_type_timing() {
    cr!("608.2g", "302.1");
    ruling!(
        "Dance with Calamity",
        "If you cast any of the exiled cards, you do so as part of the resolution of the spell. You can't wait to cast them later in the turn. Timing restrictions based on a card's type are ignored."
    );
    supported("Dance with Calamity");
    let mut t = TestGame::new(2);
    t.g.players[0].library.clear();
    let bears = stack_library(&mut t, P0, &["Grizzly Bears", "Grizzly Bears"]);
    let dance = t.hand(P0, "Dance with Calamity");
    mana(&mut t, P0, R, 8);
    // Exile two cards, stop; cast the first, not the second.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.cast(P0, dance).go();
    t.resolve();
    // A creature spell cast while Dance with Calamity was resolving.
    let log = t.dump_log();
    assert!(
        before(&log, "P0 casts Grizzly Bears", "moves Stack -> Graveyard"),
        "{log}"
    );
    t.resolve_all();
    let cast: Vec<_> = bears.iter().filter(|b| t.on_battlefield(**b)).collect();
    let left: Vec<_> = bears.iter().filter(|b| exiled(&t, **b)).copied().collect();
    assert_eq!((cast.len(), left.len()), (1, 1), "{}", t.dump_log());
    not_castable_later(&mut t, left[0]);
}

#[test]
fn etali_primal_storm_spells_are_cast_and_resolve_before_blockers() {
    cr!("608.2g", "302.1", "509.1");
    ruling!(
        "Etali, Primal Storm",
        "If you cast any of the exiled cards, you do so as part of the resolution of the triggered ability. You can't wait to cast them later in the turn. Timing permissions based on a card's type are ignored, and the spells resolve before blockers are declared."
    );
    supported("Etali, Primal Storm");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Etali, Primal Storm");
    let bears = t.library_top(P0, "Grizzly Bears");
    let giant = t.library_top(P1, "Hill Giant");
    attack_with(&mut t, &[(e, Entity::Player(P1))]);
    t.answer_choose_moved(P0, &[o(bears)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert!(t.on_battlefield(t.g.current(bears)), "{}", t.dump_log());
    assert!(exiled(&t, giant));
    block_and_finish(&mut t, P1, &[]);
    not_castable_later(&mut t, giant);
}

#[test]
fn hazorets_undying_fury_casts_during_its_resolution_only() {
    cr!("608.2g", "302.1");
    ruling!(
        "Hazoret's Undying Fury",
        "If you cast any of those cards, you do so as part of the resolution of Hazoret's Undying Fury. You can't wait to cast them later in the turn."
    );
    supported("Hazoret's Undying Fury");
    let mut t = TestGame::new(2);
    t.g.players[0].library.clear();
    let cards = stack_library(
        &mut t,
        P0,
        &["Grizzly Bears", "Hill Giant", "Hill Giant", "Hill Giant"],
    );
    let h = t.hand(P0, "Hazoret's Undying Fury");
    mana(&mut t, P0, R, 6);
    t.answer_choose_moved(P0, &[o(cards[0])]);
    t.cast(P0, h).go();
    t.resolve();
    let log = t.dump_log();
    assert!(before(&log, "P0 casts Grizzly Bears", "moves Stack -> Graveyard"), "{log}");
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(cards[0])));
    not_castable_later(&mut t, cards[1]);
}

#[test]
fn ashiok_casts_during_its_ultimate_only_ignoring_type_timing() {
    cr!("608.2g", "302.1");
    ruling!(
        "Ashiok, Nightmare Muse",
        "If you wish to cast any exiled cards, you do so as part of the resolution of Ashiok's last ability. You can't wait to cast them later. Timing permissions based on a card's type are ignored."
    );
    supported("Ashiok, Nightmare Muse");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ashiok, Nightmare Muse");
    t.g.obj_mut(a).counters.insert("loyalty".into(), 7);
    let bears = t.exile(P1, "Grizzly Bears");
    let div = t.exile(P1, "Divination");
    t.answer_choose(P0, &[o(bears)]);
    t.answer_yes(P0, true);
    t.activate(P0, a, 2, &[]).unwrap();
    t.resolve();
    // The creature spell was cast while the ability was resolving.
    assert_eq!(t.zone(bears), Zone::Stack, "{}", t.dump_log());
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(bears)), "{}", t.dump_log());
    not_castable_later(&mut t, div);
}

// ---------------------------------------------------------------------------
// Mind's Dilation, Rashmi and Ragavan: before the spell that triggered them
// ---------------------------------------------------------------------------

/// P0 controls Mind's Dilation; P1, the active player in `step`, casts its first spell of
/// the turn (Lightning Bolt at itself) with `top` on top of its library; P0 answers `yes`.
/// Returns the top card and the Bolt.
fn dilation(t: &mut TestGame, step: Step, top: &str, yes: bool) -> (ObjectId, ObjectId) {
    supported("Mind's Dilation");
    t.battlefield(P0, "Mind's Dilation");
    let card = t.library_top(P1, top);
    t.set_step(P1, step);
    let bolt = t.hand(P1, "Lightning Bolt");
    mana(t, P1, R, 1);
    t.answer_yes(P0, yes);
    let bolt = t.cast(P1, bolt).target(P1).go();
    t.settle();
    t.resolve();
    (card, bolt)
}

#[test]
fn minds_dilation_spell_resolves_first_ignoring_type_timing_but_not_other_restrictions() {
    cr!("608.2g", "302.1", "405.5", "601.3");
    ruling!(
        "Mind's Dilation",
        "If you cast the card this way, it will resolve before the spell that caused Mind's Dilation's ability to trigger."
    );
    ruling!(
        "Mind's Dilation",
        "If you cast the exiled card, you do so as part of the resolution of Mind's Dilation's ability. You can't wait to cast it later in the turn. Timing permissions based on the card's type are ignored, but other restrictions (such as \"Cast [this card] only during combat\") are not."
    );
    ruling!(
        "Mind's Dilation",
        "If you don't cast the card, perhaps because you choose not to or because it's a land card, it remains exiled."
    );
    // A creature card, during P1's main phase: cast, and it resolves before the Bolt.
    let mut t = TestGame::new(2);
    let (bears, bolt) = dilation(&mut t, Step::PrecombatMain, "Grizzly Bears", true);
    assert_eq!(t.zone(bears), Zone::Stack, "{}", t.dump_log());
    t.resolve();
    assert!(t.on_battlefield(t.g.current(bears)));
    assert_eq!(t.obj_now(t.g.current(bears)).controller, P0);
    assert_eq!(t.zone(bolt), Zone::Stack);
    assert_eq!(t.life(P1), 20);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Declined: it remains exiled; P0 can't cast it later.
    let mut t = TestGame::new(2);
    let (bears, _) = dilation(&mut t, Step::PrecombatMain, "Grizzly Bears", false);
    t.resolve_all();
    not_castable_later(&mut t, bears);
    // A land card: it remains exiled.
    let mut t = TestGame::new(2);
    let (forest, _) = dilation(&mut t, Step::PrecombatMain, "Forest", true);
    t.resolve_all();
    assert!(exiled(&t, forest));
    // Panic ("Cast this spell only during combat before blockers are declared."), in a
    // main phase: it can't be cast and remains exiled.
    let mut t = TestGame::new(2);
    let victim = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[o(victim)]);
    let (panic, _) = dilation(&mut t, Step::PrecombatMain, "Panic", true);
    t.resolve_all();
    assert!(exiled(&t, panic), "{}", t.dump_log());
    // In P1's beginning of combat step: it can.
    let mut t = TestGame::new(2);
    let victim = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[o(victim)]);
    dilation(&mut t, Step::BeginningOfCombat, "Panic", true);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Panic"), "{}", t.dump_log());
}

#[test]
fn rashmi_and_ragavan_spell_is_cast_during_the_trigger_and_resolves_first() {
    cr!("608.2g", "302.1", "405.5");
    ruling!(
        "Rashmi and Ragavan",
        "If you cast the spell without paying its mana cost, you do so during the resolution of the triggered ability. Timing restrictions of that spell based on card type are ignored. It will resolve before the spell that caused the ability to trigger."
    );
    supported("Rashmi and Ragavan");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rashmi and Ragavan");
    // Two artifacts and the Treasure: three; Grizzly Bears has mana value 2.
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Ornithopter");
    let bears = t.library_top(P1, "Grizzly Bears");
    let bolt = t.hand(P0, "Lightning Bolt");
    mana(&mut t, P0, R, 1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    let bolt = t.cast(P0, bolt).target(P1).go();
    t.settle();
    t.resolve();
    assert_eq!(t.zone(bears), Zone::Stack, "{}", t.dump_log());
    t.resolve();
    assert!(t.on_battlefield(t.g.current(bears)));
    assert_eq!(t.zone(bolt), Zone::Stack);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

// ---------------------------------------------------------------------------
// Bring to Light, Solstice Revelations, Chaos Wand: other restrictions apply
// ---------------------------------------------------------------------------

/// P0 casts Bring to Light with five colors and finds `card`, answering `yes`.
fn bring_to_light(t: &mut TestGame, card: &str, yes: bool) -> ObjectId {
    supported("Bring to Light");
    let c = t.library_top(P0, card);
    let btl = t.hand(P0, "Bring to Light");
    pool(t, P0, &[W, U, B, R, G]);
    t.answer_choose(P0, &[o(c)]);
    t.answer_yes(P0, yes);
    t.cast(P0, btl).go();
    t.resolve();
    c
}

#[test]
fn bring_to_light_casts_during_its_resolution_with_other_restrictions() {
    cr!("608.2g", "302.1", "601.3", "608.2n");
    ruling!(
        "Bring to Light",
        "If you cast the exiled card, you do so as part of the resolution of Bring to Light. You can't wait to cast it later in the turn. Timing restrictions based on the card's type are ignored, but other restrictions (such as \"Cast [this card] only during combat\") are not."
    );
    ruling!("Bring to Light", "If you don't cast the card, it remains exiled.");
    ruling!(
        "Bring to Light",
        "If you cast an instant or sorcery card this way, it goes to your graveyard as normal. It doesn't return to exile."
    );
    // A creature card, cast while Bring to Light is resolving.
    let mut t = TestGame::new(2);
    let bears = bring_to_light(&mut t, "Grizzly Bears", true);
    assert_eq!(t.zone(bears), Zone::Stack, "{}", t.dump_log());
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(bears)));
    // Declined: it remains exiled; it can't be cast later.
    let mut t = TestGame::new(2);
    let bears = bring_to_light(&mut t, "Grizzly Bears", false);
    not_castable_later(&mut t, bears);
    // Panic in a main phase: it can't be cast; it remains exiled.
    let mut t = TestGame::new(2);
    let victim = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[o(victim)]);
    let panic = bring_to_light(&mut t, "Panic", true);
    t.resolve_all();
    assert!(exiled(&t, panic), "{}", t.dump_log());
    // An instant cast this way goes to the graveyard.
    let mut t = TestGame::new(2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let bolt = bring_to_light(&mut t, "Lightning Bolt", true);
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0), "{}", t.dump_log());
    assert_eq!(t.life(P1), 17);
}

/// P0, with four Mountains, casts Solstice Revelations in `step` of `active`'s turn and
/// finds `card`, answering `yes`.
fn solstice(t: &mut TestGame, active: PlayerId, step: Step, card: &str, yes: bool) -> ObjectId {
    supported("Solstice Revelations");
    t.lands(P0, "Mountain", 4);
    let c = t.library_top(P0, card);
    let sr = t.hand(P0, "Solstice Revelations");
    t.set_step(active, step);
    pool(t, P0, &[R, C, C]);
    t.answer_yes(P0, yes);
    t.g.turn.priority = Some(P0);
    t.cast(P0, sr).go();
    t.resolve();
    c
}

#[test]
fn solstice_revelations_casts_during_its_resolution_with_other_restrictions() {
    cr!("608.2g", "302.1", "601.3");
    ruling!(
        "Solstice Revelations",
        "If you wish to cast the exiled card without paying its mana cost, you must cast it while Solstice Revelations is resolving. You can't cast it that way later in the turn. A spell cast this way may be cast at a time you normally wouldn't be able to cast a spell of that type, but other restrictions (such as \"Cast this spell only during combat\") are enforced."
    );
    // A creature card in P1's turn: cast.
    let mut t = TestGame::new(2);
    let bears = solstice(&mut t, P1, Step::PrecombatMain, "Grizzly Bears", true);
    assert_eq!(t.zone(bears), Zone::Stack, "{}", t.dump_log());
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(bears)));
    // Declined: it's put into P0's hand (and no longer free).
    let mut t = TestGame::new(2);
    let bears = solstice(&mut t, P1, Step::PrecombatMain, "Grizzly Bears", false);
    assert_eq!(t.zone(bears), Zone::Hand(P0));
    t.set_step(P0, Step::PrecombatMain);
    assert!(!castable(&mut t, P0, bears), "no free cast later");
    // Panic in a main phase: can't be cast; it's put into the hand.
    let mut t = TestGame::new(2);
    let victim = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[o(victim)]);
    let panic = solstice(&mut t, P0, Step::PrecombatMain, "Panic", true);
    t.resolve_all();
    assert_eq!(t.zone(panic), Zone::Hand(P0), "{}", t.dump_log());
    // In the beginning of combat step: cast.
    let mut t = TestGame::new(2);
    let victim = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[o(victim)]);
    solstice(&mut t, P0, Step::BeginningOfCombat, "Panic", true);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Panic"), "{}", t.dump_log());
}

/// P0 activates Chaos Wand at P1 in `step` of its turn; P1's library has `card` on top.
fn chaos_wand(t: &mut TestGame, step: Step, card: &str, yes: bool) -> ObjectId {
    supported("Chaos Wand");
    let w = t.battlefield(P0, "Chaos Wand");
    let c = t.library_top(P1, card);
    t.set_step(P0, step);
    mana(t, P0, C, 4);
    t.answer_yes(P0, yes);
    t.activate(P0, w, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    c
}

#[test]
fn chaos_wand_casts_during_its_resolution_with_other_restrictions() {
    cr!("608.2g", "307.1", "601.3");
    ruling!(
        "Chaos Wand",
        "If you wish to cast the exiled card, you must cast it while Chaos Wand's ability is resolving. You can't cast it later in the turn. A spell cast this way may be cast at a time you normally wouldn't be able to cast a spell of that type, but other restrictions (such as \"Cast this spell only during combat\") are enforced."
    );
    ruling!(
        "Chaos Wand",
        "If you don't cast the exiled instant or sorcery card, it's put on the bottom of that library with the other cards."
    );
    // A sorcery while the ability resolves (the stack isn't empty): cast.
    let mut t = TestGame::new(2);
    let div = chaos_wand(&mut t, Step::PrecombatMain, "Divination", true);
    assert_eq!(t.zone(div), Zone::Stack, "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.zone(div), Zone::Graveyard(P1));
    // Declined: on the bottom of P1's library.
    let mut t = TestGame::new(2);
    let div = chaos_wand(&mut t, Step::PrecombatMain, "Divination", false);
    assert_eq!(library_pos(&t, P1, div), Some(t.library_size(P1) - 1));
    // Panic in a main phase: can't be cast; on the bottom.
    let mut t = TestGame::new(2);
    let victim = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[o(victim)]);
    let panic = chaos_wand(&mut t, Step::PrecombatMain, "Panic", true);
    t.resolve_all();
    assert!(in_library(&t, panic, P1), "{}", t.dump_log());
    // In the beginning of combat step: cast.
    let mut t = TestGame::new(2);
    let victim = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[o(victim)]);
    chaos_wand(&mut t, Step::BeginningOfCombat, "Panic", true);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Panic"), "{}", t.dump_log());
}

// ---------------------------------------------------------------------------
// Synthesis Pod, Bre of Clan Stoutarm, Evolving Door
// ---------------------------------------------------------------------------

#[test]
fn synthesis_pod_casts_during_its_resolution_ignoring_type_timing() {
    cr!("608.2g", "302.1");
    ruling!(
        "Synthesis Pod",
        "If you wish to cast the card, you do so as part of the resolution of Synthesis Pod's ability. You can't wait to cast it later. Timing rules based on a card's type are ignored."
    );
    supported("Synthesis Pod");
    for yes in [true, false] {
        // In P1's turn: P0 casts Lightning Bolt (mana value 1) and exiles it to the Pod;
        // P1 reveals Grizzly Bears (mana value 2).
        let mut t = TestGame::new(2);
        let pod = t.battlefield(P0, "Synthesis Pod");
        let bears = t.library_top(P1, "Grizzly Bears");
        t.set_step(P1, Step::PrecombatMain);
        let bolt = t.hand(P0, "Lightning Bolt");
        mana(&mut t, P0, R, 1);
        t.g.turn.priority = Some(P0);
        let bolt = t.cast(P0, bolt).target(P1).go();
        pool(&mut t, P0, &[U, C]);
        t.answer_choose(P0, &[o(bolt)]);
        t.answer_yes(P0, yes);
        t.activate(P0, pod, 0, &[Entity::Player(P1)]).unwrap();
        t.resolve();
        if yes {
            assert_eq!(t.zone(bears), Zone::Stack, "{}", t.dump_log());
            t.resolve_all();
            assert_eq!(t.obj_now(t.g.current(bears)).controller, P0);
            assert!(t.on_battlefield(t.g.current(bears)));
        } else {
            not_castable_later(&mut t, bears);
        }
    }
}

#[test]
fn bre_casts_during_its_end_step_trigger_or_puts_the_card_into_hand() {
    cr!("608.2g", "302.1");
    ruling!(
        "Bre of Clan Stoutarm",
        "If you choose to cast the exiled card, you do so while Bre's last ability is resolving and still on the stack. You can't wait to cast it later in the turn. Timing restrictions based on the card's type are ignored."
    );
    supported("Bre of Clan Stoutarm");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Bre of Clan Stoutarm");
        let bears = t.library_top(P0, "Grizzly Bears");
        t.g.gain_life(P0, 3);
        t.answer_yes(P0, yes);
        t.advance_to(P0, Step::End);
        t.settle();
        t.resolve();
        if yes {
            // A creature spell in the end step.
            assert_eq!(t.zone(bears), Zone::Stack, "{}", t.dump_log());
            t.resolve_all();
            assert!(t.on_battlefield(t.g.current(bears)));
        } else {
            assert_eq!(t.zone(bears), Zone::Hand(P0), "{}", t.dump_log());
        }
    }
}

/// P0 activates Evolving Door sacrificing `sac`; its library has Grizzly Bears (green),
/// Watchwolf (green and white) and Child of Alara (all five colors). Returns the library
/// cards and the index of the first decision asked during the activation.
fn evolving_door(t: &mut TestGame, sac: &str) -> (Vec<ObjectId>, usize) {
    supported("Evolving Door");
    let door = t.battlefield(P0, "Evolving Door");
    let creature = t.battlefield(P0, sac);
    let lib = stack_library(t, P0, &["Grizzly Bears", "Watchwolf", "Child of Alara"]);
    mana(t, P0, C, 1);
    let from = t.asked().len();
    t.answer_choose(P0, &[o(creature)]);
    t.activate(P0, door, 0, &[]).unwrap();
    (lib, from)
}

#[test]
fn evolving_door_casts_during_its_resolution_only() {
    cr!("608.2g");
    ruling!(
        "Evolving Door",
        "If you choose to cast the spell, you must cast it as Evolving Door's ability resolves. You cannot wait and cast it later."
    );
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let lib = {
            t.battlefield(P0, "Forest");
            t.battlefield(P0, "Plains");
            let (lib, _) = evolving_door(&mut t, "Llanowar Elves");
            lib
        };
        t.answer_choose(P0, &[o(lib[1])]);
        t.answer_yes(P0, yes);
        t.resolve();
        if yes {
            // Cast for its mana cost {G}{W}.
            assert_eq!(t.zone(lib[1]), Zone::Stack, "{}", t.dump_log());
            t.resolve_all();
            assert!(t.on_battlefield(t.g.current(lib[1])));
            assert_eq!(tapped_lands(&t, P0), 2);
        } else {
            not_castable_later(&mut t, lib[1]);
        }
    }
}

#[test]
fn evolving_door_colorless_finds_one_color_five_colors_finds_nothing() {
    cr!("105.2", "105.2c", "701.23");
    ruling!(
        "Evolving Door",
        "If you sacrifice a colorless creature, you must search your library for a creature card with exactly one color. If you sacrifice a creature that is all five colors, you won't be able to find any creatures because there are no creature cards in Magic with six colors."
    );
    // Ornithopter (colorless): only Grizzly Bears can be found.
    let mut t = TestGame::new(2);
    let (lib, from) = evolving_door(&mut t, "Ornithopter");
    t.answer_choose(P0, &[o(lib[0])]);
    t.answer_yes(P0, false);
    t.resolve();
    assert!(was_offered(&t, from, lib[0]), "{}", t.dump_log());
    assert!(!was_offered(&t, from, lib[1]));
    assert!(!was_offered(&t, from, lib[2]));
    assert!(exiled(&t, lib[0]));
    // Child of Alara (five colors): nothing.
    let mut t = TestGame::new(2);
    let (lib, from) = evolving_door(&mut t, "Child of Alara");
    t.resolve_all();
    for c in &lib {
        assert!(!was_offered(&t, from, *c), "{}", t.dump_log());
        assert!(in_library(&t, *c, P0));
    }
}
