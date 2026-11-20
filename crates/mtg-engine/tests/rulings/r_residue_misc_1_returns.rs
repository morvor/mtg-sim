//! Item `rulings-residue-misc-1` — delayed returns, leaves-the-battlefield triggers and
//! zone changes (CR 400.7, 603.6, 603.7, 603.10): Reincarnation, Lifeline, Shirei, Ivory
//! Gargoyle, Lynde, Cosmic Intervention, Storm Herald, Astral Slide and Astral Drift,
//! Rescue from the Underworld, Inheritance, Plague Reaver, Krovikan Fetish, Lat-Nam's
//! Legacy, Guan Yu, Hazezon Tamar and Archfiend's Vessel.

use crate::r_residue_misc_1_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Reincarnation: "Choose target creature. When that creature dies this turn, return a
// creature card from its owner's graveyard to the battlefield under the control of that
// creature's owner."
// ---------------------------------------------------------------------------------------

#[test]
fn reincarnation_only_sets_up_a_delayed_trigger_for_this_turn() {
    cr!("603.7a", "603.7c");
    ruling!(
        "Reincarnation",
        "All Reincarnation does when it resolves is set up a delayed triggered ability. If the targeted creature isn't put into a graveyard this turn, that ability never triggers."
    );
    supported("Reincarnation");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.graveyard(P1, "Hill Giant");
    cast_new(&mut t, P0, "Reincarnation", &[obj(bears)]);
    t.resolve_all();
    // Nothing happened as it resolved.
    assert!(t.on_battlefield(bears));
    assert_eq!(t.zone(giant), Zone::Graveyard(P1));
    // The Bears die on a later turn: the delayed ability never triggers.
    t.advance_to(P1, Step::PrecombatMain);
    destroy(&mut t, bears);
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(t.zone(giant), Zone::Graveyard(P1));
    // This turn, it triggers and resolves like any triggered ability (on the stack).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.graveyard(P1, "Hill Giant");
    cast_new(&mut t, P0, "Reincarnation", &[obj(bears)]);
    t.resolve_all();
    destroy(&mut t, bears);
    assert_eq!(t.stack_len(), 1);
    t.answer_choose(P0, &[obj(giant)]);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
}

#[test]
fn reincarnation_can_return_the_same_creature_as_a_new_object() {
    cr!("400.7", "603.7a");
    ruling!(
        "Reincarnation",
        "The creature that's returned to the battlefield may be the same one that was put into the graveyard (assuming it's still in the graveyard when the delayed triggered ability resolves). If so, it returns to the battlefield as a brand-new creature."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_p108_common::put_counters(&mut t, bears, counters::PLUS1, 2);
    cast_new(&mut t, P0, "Reincarnation", &[obj(bears)]);
    t.resolve_all();
    destroy(&mut t, bears);
    let card = t.g.current(bears);
    assert_eq!(t.zone(card), Zone::Graveyard(P1));
    t.answer_choose(P0, &[obj(card)]);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_ne!(back[0], bears);
    assert_eq!(t.g.obj(back[0]).controller, P1);
    // A brand-new object: none of the old one's counters.
    assert_eq!(t.g.obj(back[0]).counter(counters::PLUS1), 0);
    assert_eq!(pt_now(&mut t, back[0]), (2, 2));
}

// ---------------------------------------------------------------------------------------
// Lifeline: "Whenever a creature dies, if another creature is on the battlefield, return
// the first card to the battlefield under its owner's control at the beginning of the
// next end step."
// ---------------------------------------------------------------------------------------

/// P0's Lifeline, a Hill Giant that stays, and two Grizzly Bears of P1's that die together
/// (Akroma's Vengeance is too much: the Bears are destroyed one by one at once by an
/// effect that destroys both).
fn lifeline_two_die(t: &mut TestGame) -> (ObjectId, ObjectId, ObjectId) {
    let lifeline = t.battlefield(P0, "Lifeline");
    t.battlefield(P0, "Hill Giant");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    run_from(
        t,
        P0,
        None,
        Effect::Destroy {
            what: Sel::All(Filter::Named("Grizzly Bears".into())),
            no_regen: false,
        },
        &[],
    );
    (lifeline, a, b)
}

#[test]
fn lifeline_returns_several_creatures_one_at_a_time() {
    cr!("603.7a", "603.7b");
    ruling!(
        "Lifeline",
        "If multiple creatures are coming back, they come back one at a time, not all at once. This is because Lifeline triggered once for each creature and set up a separate “at end of turn” effect for each."
    );
    let mut t = TestGame::new(2);
    lifeline_two_die(&mut t);
    // Two triggers, two delayed abilities.
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(
        t.g.find_in_zone(Zone::Graveyard(P1), "Grizzly Bears").len(),
        2
    );
    t.advance_to(P0, Step::End);
    t.settle();
    // Two separate delayed triggers: one card each.
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
}

#[test]
fn lifeline_fails_if_the_card_left_the_graveyard() {
    cr!("400.7", "603.7a");
    ruling!(
        "Lifeline",
        "It fails to bring the creature back if the creature is not still in the graveyard at the end of turn."
    );
    let mut t = TestGame::new(2);
    let (_, a, b) = lifeline_two_die(&mut t);
    t.resolve_all();
    exile_card(&mut t, a);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.zone(a), Zone::Exile);
    assert!(t.on_battlefield(b));
}

#[test]
fn lifeline_returns_the_creature_even_if_lifeline_left() {
    cr!("603.7a", "113.7a");
    ruling!(
        "Lifeline",
        "The creature comes back even if Lifeline leaves the battlefield after triggering, but before it resolves."
    );
    let mut t = TestGame::new(2);
    let (lifeline, _, _) = lifeline_two_die(&mut t);
    assert_eq!(t.stack_len(), 2);
    destroy(&mut t, lifeline);
    assert!(t.in_graveyard(P0, "Lifeline"));
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
}

// ---------------------------------------------------------------------------------------
// Shirei, Shizo's Caretaker: "Whenever a creature with power 1 or less is put into your
// graveyard from the battlefield, you may return that card to the battlefield at the
// beginning of the next end step if Shirei is still on the battlefield."
// ---------------------------------------------------------------------------------------

#[test]
fn shirei_cant_return_creatures_that_died_before_it_entered() {
    cr!("603.2", "603.6a");
    ruling!(
        "Shirei, Shizo's Caretaker",
        "Shirei's ability can't return any creatures that died before Shirei entered the battlefield."
    );
    supported("Shirei, Shizo's Caretaker");
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Llanowar Elves");
    destroy(&mut t, elf);
    enter(&mut t, P0, "Shirei, Shizo's Caretaker");
    t.answer_yes(P0, true);
    through_end_step(&mut t, P0);
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    assert!(t.named_on_battlefield("Llanowar Elves").is_empty());
}

#[test]
fn shirei_triggers_for_a_creature_you_own_that_dies_under_another_players_control() {
    cr!("603.2", "603.10a", "404.2");
    ruling!(
        "Shirei, Shizo's Caretaker",
        "Shirei's ability triggers if a creature you own with power 1 or less dies under another player's control."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shirei, Shizo's Caretaker");
    let elf = t.battlefield(P0, "Llanowar Elves");
    give_control(&mut t, elf, P1);
    destroy(&mut t, elf);
    // It's put into its owner's (P0's) graveyard.
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    assert_eq!(t.stack_len(), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    through_end_step(&mut t, P0);
    let back = t.named_on_battlefield("Llanowar Elves");
    assert_eq!(back.len(), 1);
    assert_eq!(t.g.obj(back[0]).controller, P0);
}

// ---------------------------------------------------------------------------------------
// Ivory Gargoyle: "When this creature dies, return it to the battlefield under its owner's
// control at the beginning of the next end step and you skip your next draw step. /
// {4}{W}: Exile this creature."
// ---------------------------------------------------------------------------------------

#[test]
fn ivory_gargoyle_its_last_controller_skips_the_draw_step_whatever_returns() {
    cr!("603.3a", "603.10a", "614.10");
    ruling!(
        "Ivory Gargoyle",
        "The player who controlled Ivory Gargoyle when it was put into a graveyard will skip their next draw step even if Ivory Gargoyle is returned to the battlefield under a different player’s control (because it’s owned by someone else) or isn’t returned to the battlefield at all (because it left the graveyard before the end of the turn, for example)."
    );
    supported("Ivory Gargoyle");
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        // P1's Gargoyle, controlled by P0, dies on P0's turn.
        let g = t.battlefield(P1, "Ivory Gargoyle");
        give_control(&mut t, g, P0);
        destroy(&mut t, g);
        t.resolve_all();
        assert_eq!(t.zone(g), Zone::Graveyard(P1));
        if leaves {
            exile_card(&mut t, g);
        }
        through_end_step(&mut t, P0);
        let back = t.named_on_battlefield("Ivory Gargoyle");
        if leaves {
            assert!(back.is_empty());
        } else {
            assert_eq!(back.len(), 1);
            assert_eq!(t.g.obj(back[0]).controller, P1);
        }
        // P1 (its owner) draws on their turn; P0, who controlled it, skips the next draw.
        let p1_hand = t.hand_size(P1);
        t.advance_to(P1, Step::PrecombatMain);
        assert_eq!(t.hand_size(P1), p1_hand + 1, "leaves: {leaves}");
        let p0_hand = t.hand_size(P0);
        t.advance_to(P0, Step::PrecombatMain);
        assert_eq!(t.hand_size(P0), p0_hand, "leaves: {leaves}");
    }
}

#[test]
fn ivory_gargoyle_exiles_itself_only_from_the_battlefield() {
    cr!("602.2", "113.6");
    ruling!(
        "Ivory Gargoyle",
        "The second ability can be activated only if Ivory Gargoyle is on the battlefield."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    let gy = t.graveyard(P0, "Ivory Gargoyle");
    let hand = t.hand(P0, "Ivory Gargoyle");
    assert!(!can_activate(&mut t, P0, gy));
    assert!(!can_activate(&mut t, P0, hand));
    let bf = t.battlefield(P0, "Ivory Gargoyle");
    assert!(can_activate(&mut t, P0, bf));
}

// ---------------------------------------------------------------------------------------
// Lynde, Cheerful Tormentor: "Whenever a Curse is put into your graveyard from the
// battlefield, return it to the battlefield attached to you at the beginning of the next
// end step. / At the beginning of your upkeep, you may attach a Curse attached to you to
// one of your opponents. If you do, draw two cards."
// ---------------------------------------------------------------------------------------

#[test]
fn lynde_a_curse_falls_off_a_player_who_leaves_the_game_and_returns() {
    cr!("800.4a", "704.5m", "303.4d");
    ruling!(
        "Lynde, Cheerful Tormentor",
        "In a multiplayer game, if a player enchanted by one of your Curses leaves the game, perhaps because they have been tormented too much by Lynde, that Curse will be put into your graveyard and cause Lynde's first triggered ability to trigger."
    );
    supported("Lynde, Cheerful Tormentor");
    supported("Curse of the Pierced Heart");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Lynde, Cheerful Tormentor");
    let curse = attach_new(&mut t, P0, "Curse of the Pierced Heart", Entity::Player(P1));
    t.g.lose_game(P1);
    t.settle();
    assert!(t.in_graveyard(P0, "Curse of the Pierced Heart"));
    assert_eq!(triggers_on_stack(&t, "Curse"), 1);
    t.resolve_all();
    through_end_step(&mut t, P0);
    assert!(t.on_battlefield(curse));
    assert_eq!(attached_to(&t, curse), Some(Entity::Player(P0)));
}

#[test]
fn lynde_attaches_a_curse_to_an_opponent_with_hexproof() {
    cr!("115.10", "702.11b", "303.4f");
    ruling!(
        "Lynde, Cheerful Tormentor",
        "Lynde's last ability doesn't target, so you can use it to attach a Curse to a player who has hexproof."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lynde, Cheerful Tormentor");
    t.battlefield(P1, "Leyline of Sanctity");
    let curse = attach_new(&mut t, P0, "Curse of the Pierced Heart", Entity::Player(P0));
    t.set_step(P1, Step::End);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(attached_to(&t, curse), Some(Entity::Player(P1)));
    assert_eq!(t.hand_size(P0), hand + 2);
}

// ---------------------------------------------------------------------------------------
// Cosmic Intervention: "If a permanent you control would be put into a graveyard from the
// battlefield this turn, exile it instead. Return it to the battlefield under its owner's
// control at the beginning of the next end step."
// ---------------------------------------------------------------------------------------

/// P0 casts Cosmic Intervention, then P0's Holy Strength (on P0's Grizzly Bears) is
/// destroyed; `setup` adds the creatures there are at the end step, before it returns.
/// Returns the Holy Strength and the candidates P0 was offered for it to enchant.
fn holy_strength_returns(
    setup: impl FnOnce(&mut TestGame) -> Option<ObjectId>,
) -> (TestGame, ObjectId, Option<ObjectId>) {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = attach_new(&mut t, P0, "Holy Strength", bears);
    cast_new(&mut t, P0, "Cosmic Intervention", &[]);
    t.resolve_all();
    destroy(&mut t, aura);
    assert_eq!(t.zone(aura), Zone::Exile);
    // The Bears leave too (exiled, as P0's), so the Aura has a new home to find.
    move_to(&mut t, bears, Zone::Hand(P0));
    let choice = setup(&mut t);
    if let Some(c) = choice {
        t.answer_choose(P0, &[obj(c)]);
    }
    through_end_step(&mut t, P0);
    (t, aura, choice)
}

#[test]
fn cosmic_intervention_a_returning_aura_ignores_hexproof_but_not_protection() {
    cr!("303.4f", "702.16c", "702.11b");
    ruling!(
        "Cosmic Intervention",
        "If you return an Aura this way, you choose what the Aura will enchant just before it enters the battlefield. An Aura returning to the battlefield this way doesn’t target the player or permanent it will enchant, so permanents or players with hexproof may be chosen; however, the chosen recipient must be able to legally be enchanted by the Aura, so a player or permanent with protection from one of the Aura’s qualities can’t be chosen this way. If there’s nothing legal for the Aura to enchant, it stays in exile."
    );
    supported("Cosmic Intervention");
    // An opponent's hexproof creature may be chosen.
    let (t, aura, scout) = holy_strength_returns(|t| Some(t.battlefield(P1, "Gladecover Scout")));
    assert!(t.on_battlefield(aura));
    assert_eq!(attached_to(&t, aura), scout.map(obj));
    // A creature with protection from white (Black Knight; Holy Strength is white) can't be: with only
    // that one, the Aura stays in exile.
    let (t, aura, _) = holy_strength_returns(|t| {
        t.battlefield(P1, "Black Knight");
        None
    });
    assert_eq!(t.zone(aura), Zone::Exile);
}

// ---------------------------------------------------------------------------------------
// Storm Herald: "When this creature enters, return any number of Aura cards from your
// graveyard to the battlefield attached to creatures you control. Exile those Auras at
// the beginning of your next end step. If those Auras would leave the battlefield, exile
// them instead of putting them anywhere else."
// ---------------------------------------------------------------------------------------

#[test]
fn storm_herald_returns_only_auras_that_can_enchant_a_creature_you_control() {
    cr!("303.4a", "303.4f");
    ruling!(
        "Storm Herald",
        "If an Aura can't be attached to a creature you control, you can't return it to the battlefield."
    );
    supported("Storm Herald");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let growth = t.graveyard(P0, "Wild Growth");
    let strength = t.graveyard(P0, "Holy Strength");
    t.answer_choose(P0, &[obj(growth), obj(strength)]);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_choose(P0, &[obj(bears)]);
    enter(&mut t, P0, "Storm Herald");
    t.resolve_all();
    // Wild Growth (enchant land) stays in the graveyard.
    assert_eq!(t.zone(growth), Zone::Graveyard(P0));
    assert!(t.on_battlefield(strength));
    assert_eq!(attached_to(&t, strength), Some(obj(bears)));
}

/// P0's Storm Herald returns Holy Strength (and leaves Pacifism in the graveyard) onto
/// P0's Grizzly Bears.
fn herald_returns_holy_strength(t: &mut TestGame) -> (ObjectId, ObjectId, ObjectId) {
    let bears = t.battlefield(P0, "Grizzly Bears");
    let strength = t.graveyard(P0, "Holy Strength");
    let pacifism = t.graveyard(P0, "Pacifism");
    t.answer_choose(P0, &[obj(strength)]);
    t.answer_choose(P0, &[obj(bears)]);
    enter(t, P0, "Storm Herald");
    t.resolve_all();
    assert!(t.on_battlefield(strength));
    let strength = t.g.current(strength);
    (bears, strength, pacifism)
}

#[test]
fn storm_herald_exiles_only_the_returned_auras() {
    cr!("603.7a", "610.3");
    ruling!(
        "Storm Herald",
        "Only Auras that were returned to the battlefield will be exiled at the beginning of your next end step. Any that you choose to leave in your graveyard will remain."
    );
    let mut t = TestGame::new(2);
    let (_, strength, pacifism) = herald_returns_holy_strength(&mut t);
    through_end_step(&mut t, P0);
    assert_eq!(t.zone(strength), Zone::Exile);
    assert_eq!(t.zone(pacifism), Zone::Graveyard(P0));
}

#[test]
fn storm_herald_an_aura_another_effect_exiles_and_returns_is_free_of_it() {
    cr!("400.7", "610.3c", "614.1a");
    ruling!(
        "Storm Herald",
        "If an effect exiles an Aura returned this way, that effect exiles it, not Storm Herald's effect. If that effect returns the Aura to the battlefield, it won't be affected by Storm Herald's effect anymore."
    );
    let mut t = TestGame::new(2);
    let (bears, strength, _) = herald_returns_holy_strength(&mut t);
    // P1's Oblivion Ring exiles it (Oblivion Ring's effect, until O-Ring leaves).
    t.answer_targets(P1, &[obj(strength)]);
    let ring = enter(&mut t, P1, "Oblivion Ring");
    t.resolve_all();
    assert_eq!(t.zone(strength), Zone::Exile);
    // O-Ring leaves: the Aura returns (onto the Bears), a new object.
    t.answer_choose(P0, &[obj(bears)]);
    destroy(&mut t, ring);
    t.resolve_all();
    let back = t.g.current(strength);
    assert!(t.on_battlefield(back), "{}", t.dump_log());
    // Storm Herald's end-step exile and replacement no longer apply to it.
    through_end_step(&mut t, P0);
    assert!(t.on_battlefield(back));
    destroy(&mut t, back);
    assert_eq!(t.zone(back), Zone::Graveyard(P0));
}

// ---------------------------------------------------------------------------------------
// Astral Slide: "Whenever a player cycles a card, you may exile target creature. If you
// do, return that card to the battlefield under its owner's control at the beginning of
// the next end step." Astral Drift: "Whenever you cycle this card or cycle another card
// while this enchantment is on the battlefield, ..." (the same).
// ---------------------------------------------------------------------------------------

#[test]
fn astral_slide_a_face_down_creature_returns_face_up_without_turning_face_up() {
    cr!("708.9", "708.8", "400.7");
    ruling!(
        "Astral Slide",
        "If a face-down card is exiled, it turns face-up and remains face-up when it returns to the battlefield. This does not trigger any abilities that would trigger on it being turned face-up."
    );
    supported("Astral Slide");
    supported("Skinthinner");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Astral Slide");
    t.battlefield(P0, "Grizzly Bears");
    let thinner = t.battlefield(P1, "Skinthinner");
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, thinner));
    t.g.recompute();
    t.lands(P0, "Swamp", 1);
    let moor = t.hand(P0, "Barren Moor");
    t.answer_targets(P0, &[obj(thinner)]);
    t.answer_yes(P0, true);
    crate::r_s04_common::cycle(&mut t, P0, moor, 0).unwrap();
    t.resolve_all();
    let exiled = t.g.current(thinner);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert!(!t.g.obj(exiled).face_down);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve();
    let back = t.g.current(thinner);
    assert!(t.on_battlefield(back));
    assert!(!t.g.obj(back).face_down);
    // Skinthinner's "When this creature is turned face up" didn't trigger.
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(t.named_on_battlefield("Grizzly Bears")[0]));
}

/// P0 controls `enchantment` and cycles `cycler` during P0's end step (after the "at the
/// beginning of the end step" triggers), exiling P1's Grizzly Bears. Returns the game and
/// the Bears.
fn cycle_in_end_step(enchantment: &str, cycler: &str) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    if enchantment != cycler {
        t.battlefield(P0, enchantment);
    }
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    t.lands(P0, "Swamp", 1);
    t.set_step(P0, Step::End);
    let card = t.hand(P0, cycler);
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_yes(P0, true);
    crate::r_s04_common::cycle(&mut t, P0, card, 0).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    (t, bears)
}

/// The Bears stay exiled through this turn and come back at the next turn's end step.
fn returns_next_end_step(t: &mut TestGame, bears: ObjectId) {
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.zone(bears), Zone::Exile);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn astral_slide_cycling_in_the_end_step_returns_the_creature_next_turn() {
    cr!("603.7a", "603.7c", "513.1");
    ruling!(
        "Astral Slide",
        "If a player cycles a card during the end step, the creature that is exiled won't come back until the end of the next turn."
    );
    let (mut t, bears) = cycle_in_end_step("Astral Slide", "Barren Moor");
    returns_next_end_step(&mut t, bears);
}

#[test]
fn astral_drift_cycling_in_the_end_step_returns_the_creature_next_turn() {
    cr!("603.7a", "603.7c", "513.1");
    ruling!(
        "Astral Drift",
        "If you cycle a card during a player's end step, the exiled creature will remain exiled until the next turn's end step."
    );
    for cycler in ["Barren Moor", "Astral Drift"] {
        let (mut t, bears) = cycle_in_end_step("Astral Drift", cycler);
        returns_next_end_step(&mut t, bears);
    }
}

// ---------------------------------------------------------------------------------------
// Rescue from the Underworld: "As an additional cost to cast this spell, sacrifice a
// creature. / Choose target creature card in your graveyard. Return that card and the
// sacrificed card to the battlefield under your control at the beginning of your next
// upkeep. Exile Rescue from the Underworld."
// ---------------------------------------------------------------------------------------

#[test]
fn rescue_from_the_underworld_a_card_that_left_the_graveyard_doesnt_return() {
    cr!("400.7", "603.7a");
    ruling!(
        "Rescue from the Underworld",
        "If either the sacrificed creature or the target creature card leaves the graveyard before the delayed triggered ability resolves during your next upkeep, it won't return."
    );
    supported("Rescue from the Underworld");
    for exiled in [0, 1] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.graveyard(P0, "Hill Giant");
        t.answer_choose(P0, &[obj(bears)]);
        cast_new(&mut t, P0, "Rescue from the Underworld", &[obj(giant)]);
        t.resolve_all();
        exile_card(&mut t, if exiled == 0 { bears } else { giant });
        t.set_step(P1, Step::End);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), exiled);
        assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1 - exiled);
    }
}

#[test]
fn rescue_from_the_underworld_no_one_acts_between_announcing_and_sacrificing() {
    cr!("601.2", "601.2h");
    ruling!(
        "Rescue from the Underworld",
        "Once you announce you're casting Rescue from the Underworld, no player may attempt to stop you from casting the spell by removing the creature you want to sacrifice."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    // P1 could kill the Bears at instant speed, but gets no chance while P0 casts.
    t.lands(P1, "Mountain", 1);
    t.hand(P1, "Lightning Bolt");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(bears)]);
    let rescue = t.hand(P0, "Rescue from the Underworld");
    lands_for_cost(&mut t, P0, "Rescue from the Underworld");
    t.cast(P0, rescue).target(giant).go();
    assert!(others_asked_since(&t, from, P0).is_empty());
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.stack_len(), 1);
}

// ---------------------------------------------------------------------------------------
// Inheritance: "Whenever a creature dies, you may pay {3}. If you do, draw a card."
// ---------------------------------------------------------------------------------------

#[test]
fn inheritance_sees_a_creature_dying_at_the_same_time_as_itself() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Inheritance",
        "If Inheritance and a creature are each put into a graveyard from the battlefield at the same time, Inheritance’s ability will trigger."
    );
    supported("Inheritance");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Inheritance");
    t.battlefield(P1, "Grizzly Bears");
    run_from(
        &mut t,
        P1,
        None,
        Effect::Destroy {
            what: Sel::All(Filter::Or(vec![
                Filter::Named("Inheritance".into()),
                Filter::Named("Grizzly Bears".into()),
            ])),
            no_regen: false,
        },
        &[],
    );
    assert!(t.in_graveyard(P0, "Inheritance"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.stack_len(), 1);
    t.lands(P0, "Wastes", 3);
    t.answer_yes(P0, true);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

// ---------------------------------------------------------------------------------------
// Plague Reaver: "Discard two cards, Sacrifice this creature: Choose target opponent.
// Return this creature to the battlefield under that player's control at the beginning of
// their next upkeep."
// ---------------------------------------------------------------------------------------

#[test]
fn plague_reaver_is_exiled_when_the_player_who_got_it_leaves() {
    cr!("800.4a");
    ruling!(
        "Plague Reaver",
        "If Plague Reaver is put onto the battlefield under the control of someone other than its owner, and that player leaves the game, Plague Reaver is exiled."
    );
    supported("Plague Reaver");
    let mut t = TestGame::new(3);
    let reaver = t.battlefield(P0, "Plague Reaver");
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.activate(P0, reaver, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Plague Reaver"));
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    let r = t.g.current(reaver);
    assert!(t.on_battlefield(r));
    assert_eq!(t.g.obj(r).controller, P1);
    t.g.lose_game(P1);
    t.settle();
    assert_eq!(t.zone(r), Zone::Exile);
}

// ---------------------------------------------------------------------------------------
// Krovikan Fetish: "Enchant creature / When this Aura enters, draw a card at the beginning
// of the next turn's upkeep. / Enchanted creature gets +1/+1."
// ---------------------------------------------------------------------------------------

#[test]
fn krovikan_fetish_moved_to_another_creature_draws_no_card() {
    cr!("303.4", "603.6a");
    ruling!(
        "Krovikan Fetish",
        "If it is moved to a new creature by something that moves Auras, the controller of the Fetish does not get to draw another card."
    );
    supported("Krovikan Fetish");
    supported("Enchantment Alteration");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let fetish = attach_new(&mut t, P0, "Krovikan Fetish", a);
    t.answer_choose(P0, &[obj(b)]);
    cast_new(&mut t, P0, "Enchantment Alteration", &[obj(fetish)]);
    t.resolve_all();
    assert_eq!(attached_to(&t, fetish), Some(obj(b)));
    assert_eq!(pt_now(&mut t, b), (4, 4));
    // No delayed draw was set up: P1 draws only their own card.
    let hand = t.hand_size(P0);
    t.advance_to(P1, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn krovikan_fetish_that_doesnt_resolve_draws_no_card() {
    cr!("608.2b", "608.3a");
    ruling!(
        "Krovikan Fetish",
        "You do not get a card if the target becomes illegal and Krovikan Fetish doesn't resolve. This is because it never successfully enters."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Krovikan Fetish", &[obj(bears)]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Krovikan Fetish"));
    let hand = t.hand_size(P0);
    t.advance_to(P1, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // With a legal target, the card is drawn at the next upkeep.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Krovikan Fetish", &[obj(bears)]);
    t.resolve_all();
    let hand = t.hand_size(P0);
    t.advance_to(P1, Step::Draw);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

// ---------------------------------------------------------------------------------------
// Lat-Nam's Legacy: "Shuffle a card from your hand into your library. If you do, draw two
// cards at the beginning of the next turn's upkeep."
// ---------------------------------------------------------------------------------------

#[test]
fn lat_nams_legacy_with_an_empty_hand_draws_nothing() {
    cr!("608.2c", "603.7a");
    ruling!(
        "Lat-Nam's Legacy",
        "Shuffling a card from your hand into your library is mandatory. The effect says “if you do” because performing that action might have been impossible. If you fail to shuffle a card into your library because your hand was empty, you won’t draw any cards."
    );
    supported("Lat-Nam's Legacy");
    for with_card in [true, false] {
        let mut t = TestGame::new(2);
        if with_card {
            let c = t.hand(P0, "Grizzly Bears");
            t.answer_choose(P0, &[obj(c)]);
        }
        cast_new(&mut t, P0, "Lat-Nam's Legacy", &[]);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), 0);
        let lib = t.library_size(P0);
        t.advance_to(P1, Step::Draw);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), if with_card { 2 } else { 0 });
        assert_eq!(t.library_size(P0), if with_card { lib - 2 } else { lib });
    }
}

// ---------------------------------------------------------------------------------------
// Guan Yu, Sainted Warrior: "When Guan Yu is put into your graveyard from the battlefield,
// you may shuffle Guan Yu into your library."
// ---------------------------------------------------------------------------------------

#[test]
fn guan_yu_exiled_instead_doesnt_trigger() {
    cr!("614.6", "603.6c");
    ruling!(
        "Guan Yu, Sainted Warrior",
        "If a replacement effect has Guan Yu move to a different zone instead of being put into the graveyard, the ability won't trigger at all."
    );
    supported("Guan Yu, Sainted Warrior");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Guan Yu, Sainted Warrior");
    t.battlefield(P1, "Leyline of the Void");
    let from = t.asked().len();
    destroy(&mut t, g);
    assert_eq!(t.zone(g), Zone::Exile);
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert!(!was_asked(&t, from, |d| matches!(
        d,
        Decision::YesNo { .. }
    )));
}

// ---------------------------------------------------------------------------------------
// Hazezon Tamar: "When Hazezon enters, create X 1/1 Sand Warrior creature tokens that are
// red, green, and white at the beginning of your next upkeep, where X is the number of
// lands you control at that time. / When Hazezon leaves the battlefield, exile all Sand
// Warriors."
// ---------------------------------------------------------------------------------------

#[test]
fn hazezon_leaving_in_response_to_the_upkeep_trigger_leaves_the_tokens_around() {
    cr!("603.7a", "603.6c", "608.2h");
    ruling!(
        "Hazezon Tamar",
        "Only Sand Warriors that are actually on the battlefield when Hazezon Tamar leaves the battlefield are affected when its second ability resolves. If Hazezon leaves the battlefield after the delayed triggered ability created by its first ability triggers but before that ability resolves, its second ability triggers and resolves first, then the Sand Warriors are put on the battlefield. Those Sand Warriors will remain on the battlefield indefinitely."
    );
    supported("Hazezon Tamar");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let h = enter(&mut t, P0, "Hazezon Tamar");
    t.resolve_all();
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // Hazezon leaves in response: its leaves trigger resolves first, exiling nothing.
    destroy(&mut t, h);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Sand"), 2);
    // They stay.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(tokens_of(&t, P0, "Sand"), 2);
}

#[test]
fn hazezon_exiles_every_sand_warrior_not_just_its_tokens() {
    cr!("205.3m", "702.73a");
    ruling!(
        "Hazezon Tamar",
        "When Hazezon Tamar's second ability resolves, all permanents with both the Sand and Warrior creature types are exiled, not just the Sand Warrior creature tokens it created. Permanents that have just one of those types are unaffected."
    );
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hazezon Tamar");
    // A changeling (every creature type: Sand and Warrior) of P1's; an Elf Warrior.
    let outcast = t.battlefield(P1, "Changeling Outcast");
    let warrior = t.battlefield(P1, "Elvish Warrior");
    destroy(&mut t, h);
    t.resolve_all();
    assert_eq!(t.zone(outcast), Zone::Exile);
    assert!(t.on_battlefield(warrior));
}

// ---------------------------------------------------------------------------------------
// Archfiend's Vessel: "When this creature enters, if it entered from your graveyard or you
// cast it from your graveyard, exile it. If you do, create a 5/5 black Demon creature
// token with flying."
// ---------------------------------------------------------------------------------------

#[test]
fn archfiends_vessel_that_left_makes_no_demon() {
    cr!("400.7", "608.2c");
    ruling!(
        "Archfiend's Vessel",
        "If Archfiend's Vessel leaves the battlefield while its triggered ability is on the stack, you can't exile it from the zone it's put into, so you won't create a Demon."
    );
    supported("Archfiend's Vessel");
    for leaves in [true, false] {
        let mut t = TestGame::new(2);
        let v = t.graveyard(P0, "Archfiend's Vessel");
        move_to(&mut t, v, Zone::Battlefield);
        assert_eq!(t.stack_len(), 1);
        if leaves {
            move_to(&mut t, v, Zone::Hand(P0));
        }
        t.resolve_all();
        assert_eq!(tokens_of(&t, P0, "Demon"), usize::from(!leaves));
        if leaves {
            assert!(t.in_hand(P0, "Archfiend's Vessel"));
        } else {
            assert!(t.in_exile("Archfiend's Vessel"));
        }
    }
}
