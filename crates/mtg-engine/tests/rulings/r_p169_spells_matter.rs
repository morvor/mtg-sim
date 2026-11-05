//! Rulings batch P169 — noncreature-spell payoffs: delayed copy triggers, "can't be
//! countered", animated enchantments, blinking, last known information, intervening "if"
//! clauses (CR 603.4), durations tied to control (CR 611.2b), "cast this turn" history,
//! Two-Headed Giant team life, and attackers that stay blocked (CR 509.1h).

use crate::r_p169_common::*;
use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Returns the object to its owner's hand.
fn bounce(t: &mut TestGame, id: ObjectId) {
    crate::r_p189_common::bounce(t, id);
}

/// Moves the object to exile.
fn exile(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.move_object(id, Zone::Exile, MoveCause::Effect, None);
    t.g.recompute();
    t.g.flush_events();
}

#[test]
fn mercurial_spelldancer_copies_a_countered_spell_and_the_copy_resolves_first() {
    cr!("603.7", "707.10", "603.3");
    ruling!(
        "Mercurial Spelldancer",
        "A copy is created even if the spell that caused the delayed triggered ability to trigger has been countered by the time that delayed triggered ability resolves. The copy resolves before the original spell."
    );
    supported("Mercurial Spelldancer");
    let setup = |t: &mut TestGame| {
        let sd = t.battlefield(P0, "Mercurial Spelldancer");
        put_counters(t, sd, counters::OIL, 2);
        t.answer_yes(P0, true);
        t.attack(&[(sd, Entity::Player(P1))], &[]);
        t.advance_to(P0, Step::PostcombatMain);
        assert_eq!(t.life(P1), 18);
        assert_eq!(t.counters(sd, counters::OIL), 0);
        mana(t, P0, ManaType::R, 1);
        let shock = t.hand(P0, "Shock");
        let spell = t.cast(P0, shock).target(Entity::Player(P1)).go();
        // Keep the copy's target.
        t.answer_yes(P0, false);
        t.settle();
        spell
    };
    // Countered while the delayed trigger waits: the copy is still created.
    let mut t = TestGame::new(2);
    let spell = setup(&mut t);
    counterspell(&mut t, spell);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Not countered: the copy is above the original and resolves first.
    let mut t = TestGame::new(2);
    let spell = setup(&mut t);
    // Resolve the triggers above Shock until the copy appears.
    while spell_copies(&t).is_empty() {
        assert!(t.g.stack.last() != Some(&spell), "no copy was made");
        t.resolve();
    }
    let copy = spell_copies(&t)[0];
    assert_eq!(t.g.stack.last(), Some(&copy));
    t.resolve();
    assert_eq!(t.life(P1), 16);
    assert!(t.g.stack.contains(&spell));
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn pearl_lake_ancient_can_be_targeted_by_a_counterspell() {
    cr!("101.2", "608.2c");
    ruling!(
        "Pearl Lake Ancient",
        "A spell or ability that counters spells can still target Pearl Lake Ancient. When one resolves, Pearl Lake Ancient won’t be countered, but any additional effects of that spell or ability will still happen."
    );
    supported("Pearl Lake Ancient");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, ManaType::U, 7);
    let card = t.hand(P0, "Pearl Lake Ancient");
    let spell = t.cast(P0, card).go();
    let hand = t.hand_size(P1);
    mana(&mut t, P1, ManaType::U, 3);
    let exclude = t.hand(P1, "Exclude");
    t.cast(P1, exclude).target(spell).go();
    t.resolve();
    assert!(t.g.stack.contains(&spell));
    // Exclude's card left the hand, and P1 drew one.
    assert_eq!(t.hand_size(P1), hand + 1);
    t.resolve_all();
    assert!(t.on_battlefield(card));
}

#[test]
fn myth_realized_reactivated_overrides_setting_effects_but_not_modifications() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Myth Realized",
        "Activating the ability that turns Myth Realized into a creature while it's already a creature will override any effects that set its power or toughness to a specific value. Effects that modify power or toughness without directly setting them to a specific value will continue to apply."
    );
    supported("Myth Realized");
    supported("Diminish");
    let mut t = TestGame::new(2);
    let myth = t.battlefield(P0, "Myth Realized");
    put_counters(&mut t, myth, counters::LORE, 2);
    mana(&mut t, P0, ManaType::W, 1);
    t.activate(P0, myth, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(myth), (2, 2));
    // Diminish (base 1/1) and Giant Growth (+3/+3).
    cast_new(&mut t, P0, "Diminish", &[obj(myth)]);
    t.resolve_all();
    cast_new(&mut t, P0, "Giant Growth", &[obj(myth)]);
    t.resolve_all();
    // Diminish (the later base-setting effect) wins over the activation; each spell also
    // added a lore counter.
    let lore = t.counters(myth, counters::LORE) as i32;
    assert_eq!(lore, 4);
    assert_eq!(t.pt(myth), (4, 4));
    mana(&mut t, P0, ManaType::W, 1);
    t.activate(P0, myth, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(myth), (lore + 3, lore + 3));
}

#[test]
fn myth_realized_cant_attack_the_turn_it_enters() {
    cr!("302.6");
    ruling!(
        "Myth Realized",
        "Myth Realized can't attack the turn it enters the battlefield."
    );
    let attack_ok = |sick: bool| {
        let mut t = TestGame::new(2);
        let myth = if sick {
            t.enter(P0, "Myth Realized")
        } else {
            t.battlefield(P0, "Myth Realized")
        };
        put_counters(&mut t, myth, counters::LORE, 2);
        mana(&mut t, P0, ManaType::W, 1);
        t.activate(P0, myth, 1, &[]).unwrap();
        t.resolve_all();
        t.set_step(P0, Step::BeginningOfCombat);
        can_attack(&mut t, myth)
    };
    assert!(!attack_ok(true));
    assert!(attack_ok(false));
}

#[test]
fn ojutai_exemplars_returns_as_a_new_object() {
    cr!("400.7", "506.4", "704.5m");
    ruling!(
        "Ojutai Exemplars",
        "After the ability returns Ojutai Exemplars to the battlefield, it will be a new object with no connection to the Ojutai Exemplars that left the battlefield. It won't be in combat or have any additional abilities it may have had when it left the battlefield. Any +1/+1 counters on it or Auras attached to it are removed."
    );
    supported("Ojutai Exemplars");
    let mut t = TestGame::new(2);
    let ex = t.battlefield(P0, "Ojutai Exemplars");
    put_counters(&mut t, ex, counters::PLUS1, 1);
    let aura = t.battlefield(P0, "Holy Strength");
    assert!(t.g.attach(aura, obj(ex)));
    attack_with(&mut t, &[(ex, Entity::Player(P1))]);
    // Additional abilities until end of turn: trample from Selesnya Charm, and first
    // strike and lifelink from its own trigger (second mode).
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    cast_new(&mut t, P0, "Selesnya Charm", &[obj(ex)]);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.resolve_all();
    assert!(has_kw(&mut t, ex, KeywordKind::Trample));
    assert!(has_kw(&mut t, ex, KeywordKind::FirstStrike));
    // The third mode: exile it and return it.
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    cast_opt(&mut t, P0);
    t.resolve_all();
    let new = t.g.current(ex);
    assert_ne!(new, ex);
    assert!(t.on_battlefield(new));
    assert!(t.obj_now(new).tapped);
    assert!(!is_blocked(&t, new));
    assert!(t
        .g
        .combat
        .as_ref()
        .unwrap()
        .attackers
        .iter()
        .all(|a| a.id != new));
    assert_eq!(t.counters(new, counters::PLUS1), 0);
    assert!(!has_kw(&mut t, new, KeywordKind::Trample));
    assert!(!has_kw(&mut t, new, KeywordKind::FirstStrike));
    assert!(!has_kw(&mut t, new, KeywordKind::Lifelink));
    assert_eq!(t.pt(new), (4, 4));
    assert!(t.in_graveyard(P0, "Holy Strength"));
}

#[test]
fn ojutai_exemplars_that_left_in_response_doesnt_return() {
    cr!("400.7", "608.2b");
    ruling!(
        "Ojutai Exemplars",
        "If Ojutai Exemplars leaves the battlefield in response to its ability with the third mode chosen, that ability won't return Ojutai Exemplars to the battlefield. It will stay wherever it is."
    );
    let mut t = TestGame::new(2);
    let ex = t.battlefield(P0, "Ojutai Exemplars");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    cast_opt(&mut t, P0);
    t.settle();
    assert_eq!(stack_triggers_from(&t, ex).len(), 1);
    bounce(&mut t, ex);
    t.resolve_all();
    assert!(t.in_hand(P0, "Ojutai Exemplars"));
    assert!(t.named_on_battlefield("Ojutai Exemplars").is_empty());
}

#[test]
fn dragons_rage_channeler_damage_becomes_lethal_when_delirium_is_lost() {
    cr!("704.5g", "120.6");
    ruling!(
        "Dragon's Rage Channeler",
        "Because damage remains marked on a creature until the cleanup step or an effect removes that damage, nonlethal damage dealt to Dragon's Rage Channeler may become lethal if the number of card types found in your graveyard drops below four."
    );
    supported("Dragon's Rage Channeler");
    let mut t = TestGame::new(2);
    let drc = t.battlefield(P0, "Dragon's Rage Channeler");
    let forest = t.graveyard(P0, "Forest");
    for c in ["Opt", "Grizzly Bears", "Divination"] {
        t.graveyard(P0, c);
    }
    assert_eq!(t.pt(drc), (3, 3));
    let shock = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(shock, obj(drc), 2, false);
    t.g.flush_events();
    t.settle();
    assert!(t.on_battlefield(drc));
    exile(&mut t, forest);
    t.settle();
    assert!(t.in_graveyard(P0, "Dragon's Rage Channeler"));
}

#[test]
fn scroll_of_the_masters_counts_as_its_ability_resolves() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Scroll of the Masters",
        "Count the number of lore counters on Scroll of the Masters as the activated ability resolves to determine how big a bonus the target creature gets. If Scroll of the Masters isn’t on the battlefield at that time, use the number of lore counters on it when it left the battlefield."
    );
    ruling!(
        "Scroll of the Masters",
        "Once the activated ability has resolved, changing the number of lore counters on Scroll of the Masters won’t affect the bonus that activation granted."
    );
    supported("Scroll of the Masters");
    // A third counter arrives in response, then the Scroll is destroyed: +3/+3.
    let mut t = TestGame::new(2);
    let scroll = t.battlefield(P0, "Scroll of the Masters");
    put_counters(&mut t, scroll, counters::LORE, 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::C, 3);
    t.activate(P0, scroll, 0, &[obj(bears)]).unwrap();
    cast_opt(&mut t, P0);
    t.settle();
    // Resolve the lore trigger and Opt, leaving the activation.
    t.resolve();
    t.resolve();
    assert_eq!(t.counters(scroll, counters::LORE), 3);
    destroy(&mut t, scroll);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    // Counters added after it resolved don't change the bonus.
    let mut t = TestGame::new(2);
    let scroll = t.battlefield(P0, "Scroll of the Masters");
    put_counters(&mut t, scroll, counters::LORE, 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::C, 3);
    t.activate(P0, scroll, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    cast_opt(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(scroll, counters::LORE), 3);
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn dragonfly_swarm_checks_for_a_lesson_twice() {
    cr!("603.4");
    ruling!(
        "Dragonfly Swarm",
        "Dragonfly Swarm's triggered ability checks if there is a Lesson card in your graveyard when it is put on the stack and again when it goes to resolve. If there is no Lesson card in your graveyard when Dragonfly Swarm dies, the ability will not be put on the stack at all."
    );
    supported("Dragonfly Swarm");
    supported("Environmental Sciences");
    // No Lesson: no trigger.
    let mut t = TestGame::new(2);
    let swarm = t.battlefield(P0, "Dragonfly Swarm");
    destroy(&mut t, swarm);
    assert_eq!(triggers_of(&t, swarm), 0);
    // A Lesson that leaves before it resolves: no card.
    let mut t = TestGame::new(2);
    let swarm = t.battlefield(P0, "Dragonfly Swarm");
    let lesson = t.graveyard(P0, "Environmental Sciences");
    destroy(&mut t, swarm);
    assert_eq!(stack_triggers(&t).len(), 1);
    exile(&mut t, lesson);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    // Positive control: the Lesson stays.
    let mut t = TestGame::new(2);
    let swarm = t.battlefield(P0, "Dragonfly Swarm");
    t.graveyard(P0, "Environmental Sciences");
    destroy(&mut t, swarm);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn hurkyl_counts_spells_cast_before_it_entered() {
    cr!("603.4");
    ruling!(
        "Hurkyl, Master Wizard",
        "Hurkyl need not have been on the battlefield at the time the noncreature spells were cast for the ability to trigger or for the spells to be counted."
    );
    supported("Hurkyl, Master Wizard");
    let mut t = TestGame::new(2);
    cast_opt(&mut t, P0);
    t.resolve_all();
    t.battlefield(P0, "Hurkyl, Master Wizard");
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.answer_choose(P0, &[obj(bolt)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_hand(P0, "Lightning Bolt"));
}

#[test]
fn thancred_waters_gone_before_its_ability_resolves_grants_nothing() {
    cr!("611.2b");
    ruling!(
        "Thancred Waters",
        "If Thancred Waters leaves the battlefield or you lose control of it before its triggered ability resolves, the target legendary permanent won't gain indestructible at all."
    );
    supported("Thancred Waters");
    let run = |act: fn(&mut TestGame, ObjectId)| {
        let mut t = TestGame::new(2);
        let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
        t.answer_targets(P0, &[obj(isamaru)]);
        let th = t.enter(P0, "Thancred Waters");
        t.settle();
        act(&mut t, th);
        t.resolve_all();
        has_kw(&mut t, isamaru, KeywordKind::Indestructible)
    };
    assert!(!run(|t, th| bounce(t, th)));
    assert!(!run(|t, th| crate::r_p117_common::gain_control(t, P1, th)));
    assert!(run(|_, _| {}));
}

/// Ty Lee enters under P0's control targeting P1's Grizzly Bears; `respond` runs while its
/// ability is on the stack, `after` once it resolved. Returns whether the Bears is tapped
/// after P1's next untap step.
fn ty_lee(respond: fn(&mut TestGame, ObjectId), after: fn(&mut TestGame, ObjectId)) -> bool {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let ty = t.enter(P0, "Ty Lee, Chi Blocker");
    t.settle();
    respond(&mut t, ty);
    t.resolve_all();
    assert!(t.obj_now(bears).tapped, "the Bears was tapped");
    after(&mut t, ty);
    t.advance_to(P1, Step::Upkeep);
    t.obj_now(bears).tapped
}

#[test]
fn ty_lee_effect_lasts_only_while_you_control_her() {
    cr!("611.2b", "502.3");
    ruling!(
        "Ty Lee, Chi Blocker",
        "If Ty Lee, Chi Blocker ceases to be under your control before its ability resolves, you will still tap the targeted creature, but it will untap normally during its controller's next untap step."
    );
    ruling!(
        "Ty Lee, Chi Blocker",
        "If Ty Lee, Chi Blocker leaves the battlefield, you no longer control her, and her effect ends."
    );
    ruling!(
        "Ty Lee, Chi Blocker",
        "If another player gains control of Ty Lee, Chi Blocker, her effect ends. Regaining control of Ty Lee, Chi Blocker won't cause the effect to resume."
    );
    supported("Ty Lee, Chi Blocker");
    fn none(_: &mut TestGame, _: ObjectId) {}
    // Positive control: she stays.
    assert!(ty_lee(none, none));
    // Control lost before the ability resolves.
    assert!(!ty_lee(|t, ty| crate::r_p117_common::gain_control(t, P1, ty), none));
    // She leaves afterwards.
    assert!(!ty_lee(none, bounce));
    // Another player gains control, then P0 regains it.
    assert!(!ty_lee(none, |t, ty| {
        crate::r_p117_common::gain_control(t, P1, ty);
        crate::r_p117_common::gain_control(t, P0, ty);
        assert_eq!(t.obj_now(ty).controller, P0);
    }));
}

#[test]
fn lilah_doesnt_exile_a_countered_spell() {
    cr!("701.6a", "608.2b");
    ruling!(
        "Lilah, Undefeated Slickshot",
        "If a spell is countered or it never resolves, Lilah, Undefeated Slickshot will not exile it."
    );
    supported("Lilah, Undefeated Slickshot");
    let helix = |t: &mut TestGame| {
        t.battlefield(P0, "Lilah, Undefeated Slickshot");
        mana(t, P0, ManaType::R, 1);
        mana(t, P0, ManaType::W, 1);
        let card = t.hand(P0, "Lightning Helix");
        let spell = t.cast(P0, card).target(Entity::Player(P1)).go();
        (card, spell)
    };
    let mut t = TestGame::new(2);
    let (_, spell) = helix(&mut t);
    counterspell(&mut t, spell);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Lightning Helix"));
    // It never resolves: its only target becomes illegal.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lilah, Undefeated Slickshot");
    let bears = t.battlefield(P1, "Grizzly Bears");
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::W, 1);
    let card = t.hand(P0, "Lightning Helix");
    t.cast(P0, card).target(bears).go();
    exile(&mut t, bears);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Lightning Helix"));
    // Positive control: it resolves and is exiled.
    let mut t = TestGame::new(2);
    helix(&mut t);
    t.resolve_all();
    assert!(t.in_exile("Lightning Helix"));
}

#[test]
fn niblis_of_frost_two_dont_untap_effects_share_one_untap_step() {
    cr!("502.3", "611.2a");
    ruling!(
        "Niblis of Frost",
        "If multiple effects say that a creature doesn’t untap during its controller’s next untap step, those effects all apply during one untap step. For example, if you cast two instants in one turn and target the same creature twice with Niblis of Frost’s ability, that creature will only spend one untap step without untapping."
    );
    supported("Niblis of Frost");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Niblis of Frost");
    let bears = t.battlefield(P1, "Grizzly Bears");
    for _ in 0..2 {
        cast_opt(&mut t, P0);
        t.answer_targets(P0, &[obj(bears)]);
        t.resolve_all();
    }
    assert!(t.obj_now(bears).tapped);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(bears).tapped);
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn the_mechanist_sets_base_pt_in_timestamp_order() {
    cr!("613.4b", "613.7");
    ruling!(
        "The Mechanist, Aerial Artisan",
        "If the artifact token was already a creature, its base power and toughness will become 3/1. This overwrites any previous effects that set the creature's base power and toughness to specific values. Any power- or toughness-setting effects that start to apply after The Mechanist, Aerial Artisan's ability resolves will overwrite this effect."
    );
    supported("The Mechanist, Aerial Artisan");
    let servo = |t: &mut TestGame| -> ObjectId {
        t.enter(P0, "Cogworker's Puzzleknot");
        t.resolve_all();
        let s = tokens(t, P0)[0];
        assert!(t.obj_now(s).is(CardType::Artifact) && t.obj_now(s).is(CardType::Creature));
        s
    };
    // Diminish before: The Mechanist overwrites it.
    let mut t = TestGame::new(2);
    let mech = t.battlefield(P0, "The Mechanist, Aerial Artisan");
    let s = servo(&mut t);
    cast_new(&mut t, P0, "Diminish", &[obj(s)]);
    t.resolve_all();
    t.activate(P0, mech, 0, &[obj(s)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(s), (3, 1));
    assert!(has_kw(&mut t, s, KeywordKind::Flying));
    // Diminish after: it overwrites The Mechanist.
    let mut t = TestGame::new(2);
    let mech = t.battlefield(P0, "The Mechanist, Aerial Artisan");
    let s = servo(&mut t);
    t.activate(P0, mech, 0, &[obj(s)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(s), (3, 1));
    cast_new(&mut t, P0, "Diminish", &[obj(s)]);
    t.resolve_all();
    assert_eq!(t.pt(s), (1, 1));
}

#[test]
fn discard_your_hand_then_draw_three_with_an_empty_hand() {
    cr!("609.3");
    ruling!(
        "Bedlam Reveler",
        "If you have no cards in hand when Bedlam Reveler’s enters ability resolves, you just draw three cards."
    );
    ruling!(
        "Fugitive Codebreaker",
        "If you have no cards in hand when Fugitive Codebreaker's last ability resolves, you won't discard any cards, but you'll still draw three cards."
    );
    supported("Bedlam Reveler");
    supported("Fugitive Codebreaker");
    let mut t = TestGame::new(2);
    t.enter(P0, "Bedlam Reveler");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.graveyard_size(P0), 0);
    // Fugitive Codebreaker cast face down, then turned face up with an empty hand.
    let mut t = TestGame::new(2);
    mana(&mut t, P0, ManaType::C, 3);
    let card = t.hand(P0, "Fugitive Codebreaker");
    t.cast(P0, card)
        .method(CastMethod::FaceDown(KeywordKind::Disguise))
        .go();
    t.resolve_all();
    let fc = t.g.current(card);
    assert!(t.obj_now(fc).face_down);
    assert_eq!(t.hand_size(P0), 0);
    mana(&mut t, P0, ManaType::R, 6);
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, Action::Special(SpecialAction::TurnFaceUp { obj: fc }))
        .unwrap();
    t.resolve_all();
    assert!(!t.obj_now(fc).face_down);
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn eshki_dragonclaw_checks_as_combat_begins() {
    cr!("603.4", "603.2b");
    ruling!(
        "Eshki Dragonclaw",
        "If you haven’t cast both a creature spell and a noncreature spell this turn by the time your beginning of combat step begins, Eshki’s last ability won’t trigger at all. You won’t be able to cast a creature or noncreature spell during your beginning of combat step in time to have the ability trigger."
    );
    supported("Eshki Dragonclaw");
    let mut t = TestGame::new(2);
    let eshki = t.battlefield(P0, "Eshki Dragonclaw");
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    go_to(&mut t, Step::BeginningOfCombat);
    assert_eq!(triggers_of(&t, eshki), 0);
    // The noncreature spell in the beginning of combat step comes too late.
    cast_opt(&mut t, P0);
    t.resolve_all();
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(triggers_of(&t, eshki), 0);
    assert_eq!(t.counters(eshki, counters::PLUS1), 0);
    // Positive control: both before combat.
    let mut t = TestGame::new(2);
    let eshki = t.battlefield(P0, "Eshki Dragonclaw");
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    cast_opt(&mut t, P0);
    t.resolve_all();
    go_to(&mut t, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.counters(eshki, counters::PLUS1), 2);
}

#[test]
fn mage_ring_bully_isnt_forced_to_attack_when_tapped_or_for_a_cost() {
    cr!("508.1d");
    ruling!(
        "Mage-Ring Bully",
        "If, during its controller's declare attackers step, Mage-Ring Bully is tapped or is affected by a spell or ability that says it can't attack, then Mage-Ring Bully doesn't attack. If there's a cost associated with having it attack, its controller isn't forced to pay that cost. If they don't, Mage-Ring Bully doesn't have to attack."
    );
    supported("Mage-Ring Bully");
    use crate::r_s09_common::legal_attack;
    // Positive control: untapped, it must attack.
    let mut t = TestGame::new(2);
    let bully = t.battlefield(P0, "Mage-Ring Bully");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &[(bully, Entity::Player(P1))]));
    // Tapped.
    let mut t = TestGame::new(2);
    let bully = t.battlefield(P0, "Mage-Ring Bully");
    t.g.tap(bully);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(legal_attack(&mut t, &[]));
    // Can't attack (Pacifism).
    let mut t = TestGame::new(2);
    let bully = t.battlefield(P0, "Mage-Ring Bully");
    let pac = t.battlefield(P1, "Pacifism");
    assert!(t.g.attach(pac, obj(bully)));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(legal_attack(&mut t, &[]));
    // An attack cost (Ghostly Prison): not forced to pay.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mage-Ring Bully");
    t.battlefield(P1, "Ghostly Prison");
    t.lands(P0, "Wastes", 2);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(legal_attack(&mut t, &[]));
}

#[test]
fn each_opponent_in_two_headed_giant_is_the_whole_team() {
    cr!("810.9", "102.3");
    ruling!(
        "Blisterspit Gremlin",
        "In a Two-Headed Giant game, Blisterspit Gremlin's first ability causes the opposing team to lose 2 life."
    );
    ruling!(
        "Firebrand Archer",
        "In a Two-Headed Giant game, Firebrand Archer’s ability causes it to deal a total of 2 damage to the opposing team."
    );
    supported("Blisterspit Gremlin");
    let mut t = two_headed_giant();
    let gremlin = t.battlefield(P0, "Blisterspit Gremlin");
    let them = t.life(P2);
    mana(&mut t, P0, ManaType::C, 1);
    t.activate(P0, gremlin, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!((t.life(P2), t.life(P3)), (them - 2, them - 2));
    let mut t = two_headed_giant();
    t.battlefield(P0, "Firebrand Archer");
    let them = t.life(P2);
    cast_opt(&mut t, P0);
    t.resolve_all();
    assert_eq!((t.life(P2), t.life(P3)), (them - 2, them - 2));
}

#[test]
fn a_noncreature_spell_cast_this_turn_counts_whatever_happened_to_it() {
    cr!("601.2i");
    ruling!(
        "Seeker of Insight",
        "It doesn’t matter whether the noncreature spell has resolved or was countered, as long as it was cast. That spell may even still be on the stack when you activate Seeker of Insight’s ability."
    );
    ruling!(
        "Silburlind Snapper",
        "It doesn’t matter whether the noncreature spell resolved or was countered, as long as it was cast."
    );
    supported("Seeker of Insight");
    supported("Silburlind Snapper");
    let mut t = TestGame::new(2);
    let seeker = t.battlefield(P0, "Seeker of Insight");
    assert!(!can_activate(&mut t, P0, seeker));
    let spell = cast_opt(&mut t, P0);
    // Still on the stack.
    assert!(can_activate(&mut t, P0, seeker));
    counterspell(&mut t, spell);
    assert!(can_activate(&mut t, P0, seeker));
    // Snapper: the countered spell lets it attack.
    let mut t = TestGame::new(2);
    let snapper = t.battlefield(P0, "Silburlind Snapper");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, snapper));
    t.set_step(P0, Step::PrecombatMain);
    let spell = cast_opt(&mut t, P0);
    counterspell(&mut t, spell);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, snapper));
    // A resolved one too.
    let mut t = TestGame::new(2);
    let snapper = t.battlefield(P0, "Silburlind Snapper");
    cast_opt(&mut t, P0);
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, snapper));
}

#[test]
fn lyse_hext_counts_spells_cast_before_it_entered() {
    cr!("611.3a");
    ruling!(
        "Lyse Hext",
        "Noncreature spells that were cast before Lyse Hext entered count. For example, if Lyse Hext enters on a turn where you've already cast two noncreature spells, she'll have double strike immediately."
    );
    supported("Lyse Hext");
    let mut t = TestGame::new(2);
    cast_opt(&mut t, P0);
    t.resolve_all();
    let lyse = t.battlefield(P0, "Lyse Hext");
    assert!(!has_kw(&mut t, lyse, KeywordKind::DoubleStrike));
    let mut t = TestGame::new(2);
    for _ in 0..2 {
        cast_opt(&mut t, P0);
        t.resolve_all();
    }
    let lyse = t.battlefield(P0, "Lyse Hext");
    assert!(has_kw(&mut t, lyse, KeywordKind::DoubleStrike));
}

#[test]
fn cant_be_blocked_after_blocks_doesnt_unblock() {
    cr!("509.1h");
    ruling!(
        "Elusive Spellfist",
        "Once Elusive Spellfist has been blocked, causing its ability to trigger won’t change or undo that block."
    );
    ruling!(
        "Sahagin",
        "Once Sahagin has been blocked, casting a noncreature spell in order to cause Sahagin's ability to trigger won't cause Sahagin to become unblocked."
    );
    ruling!(
        "Hraesvelgr of the First Brood",
        "Once a creature has been blocked, Hraesvelgr's last ability won't cause it to become unblocked."
    );
    for (card, four) in [("Elusive Spellfist", false), ("Sahagin", true)] {
        supported(card);
        let mut t = TestGame::new(2);
        let me = t.battlefield(P0, card);
        let wall = t.battlefield(P1, "Hill Giant");
        crate::r_p076_common::unblockable_after_blocked(&mut t, me, wall, |t| {
            if four {
                mana(t, P0, ManaType::U, 4);
                let c = t.hand(P0, "Fact or Fiction");
                t.cast(P0, c).go();
            } else {
                cast_opt(t, P0);
            }
            t.settle();
            assert_eq!(stack_triggers_from(t, me).len(), 1, "{card}");
        });
    }
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hraesvelgr of the First Brood");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Hill Giant");
    crate::r_p076_common::unblockable_after_blocked(&mut t, bears, wall, |t| {
        cast_opt(t, P0);
        t.answer_targets(P0, &[obj(bears)]);
    });
}
