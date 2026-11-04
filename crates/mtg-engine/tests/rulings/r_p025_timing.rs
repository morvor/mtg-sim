//! Rulings batch P025 — the timing of abilities and effects that make copies: triggers
//! that resolve before the spell that caused them (CR 603.3), triggers that wait for the
//! spell to finish resolving (CR 603.3), "the first time each turn" (CR 614.1), choices
//! made on resolution (CR 608.2d), and the order replacement effects apply to a permanent
//! entering (CR 614.12, 616.1).

use crate::r_p025_common::*;
use mtg_engine::events::MoveCause;
use mtg_engine::object::{CastMethod, ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn tokens_of(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.battlefield
        .iter()
        .copied()
        .filter(|o| {
            let o = t.g.obj(*o);
            o.kind == ObjKind::Token && o.controller == p
        })
        .collect()
}

fn raise_the_alarm(t: &mut TestGame, p: PlayerId) {
    t.lands(p, "Plains", 2);
    let c = t.hand(p, "Raise the Alarm");
    t.cast(p, c).go();
    t.resolve_all();
}

// --- Triggers that resolve first ---------------------------------------------------------------

#[test]
fn muddle_and_saheeli_resolve_before_the_spell_even_if_its_countered() {
    cr!("603.3", "603.3b", "701.6a");
    ruling!(
        "Muddle, the Ever-Changing",
        "Muddle's ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    ruling!(
        "Saheeli, Sublime Artificer",
        "Saheeli's first ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    supported("Muddle, the Ever-Changing");
    supported("Saheeli, Sublime Artificer");
    let mut t = TestGame::new(2);
    let muddle = t.battlefield(P0, "Muddle, the Ever-Changing");
    t.battlefield(P0, "Saheeli, Sublime Artificer");
    let giant = t.battlefield(P0, "Hill Giant");
    lands_for(&mut t, P0, "{U}");
    let opt = t.hand(P0, "Opt");
    t.answer_targets(P0, &[obj(giant)]);
    let before = t.g.battlefield.clone();
    t.cast(P0, opt).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t).len(), 2);
    // P1 counters Opt.
    lands_for(&mut t, P1, "{U}{U}");
    let cs = t.hand(P1, "Counterspell");
    t.g.turn.priority = Some(P1);
    t.cast(P1, cs).target(obj(opt)).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Opt"));
    t.resolve_all();
    assert_eq!(name_of(&t, muddle), "Hill Giant");
    let servos = new_tokens(&t, P0, &before);
    assert_eq!(servos.len(), 1);
    assert_eq!(name_of(&t, servos[0]), "Servo Token");
}

#[test]
fn riku_copies_an_instant_with_no_targets() {
    cr!("603.2", "707.10");
    ruling!(
        "Riku of Two Reflections",
        "Riku's first ability triggers whenever you cast any instant or sorcery spell, regardless of whether that spell has targets."
    );
    supported("Riku of Two Reflections");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Riku of Two Reflections");
    // Opt: "Scry 1. Draw a card."
    lands_for(&mut t, P0, "{U}{U}{R}");
    let opt = t.hand(P0, "Opt");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.cast(P0, opt).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t).len(), 1);
    t.resolve_all();
    // The copy and Opt each drew a card.
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
}

// --- Sublime Epiphany ------------------------------------------------------------------------

#[test]
fn sublime_epiphany_counters_only_abilities_already_on_the_stack() {
    cr!("701.6b", "113.7a", "602.1");
    ruling!(
        "Sublime Epiphany",
        "Sublime Epiphany's second mode counters only activated and triggered abilities already on the stack. It doesn't prevent abilities from being activated or from triggering later in the turn, and it has no effect on static abilities."
    );
    supported("Sublime Epiphany");
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P1, "Shivan Dragon");
    lands_for(&mut t, P1, "{R}{R}");
    // P1 activates Shivan Dragon's firebreathing; P0 counters it.
    activate_containing(&mut t, P1, dragon, "+1/+0").expect("activation");
    let ability = *t.g.stack.last().unwrap();
    lands_for(&mut t, P0, "{4}{U}{U}");
    let ep = t.hand(P0, "Sublime Epiphany");
    t.g.turn.priority = Some(P0);
    t.cast(P0, ep).modes(&[1]).target(obj(ability)).go();
    t.resolve_all();
    assert_eq!(t.pt(dragon), (5, 5));
    // It can be activated again, and that resolves.
    activate_containing(&mut t, P1, dragon, "+1/+0").expect("activation");
    t.resolve_all();
    assert_eq!(t.pt(dragon), (6, 5));
}

#[test]
fn sublime_epiphany_can_counter_a_loyalty_ability_that_adds_mana() {
    cr!("605.1a", "606.3", "701.6b");
    ruling!(
        "Sublime Epiphany",
        "Sublime Epiphany's second mode can't counter an activated or triggered mana ability. An activated mana ability is one that could add mana as it resolves, doesn't have a target, and isn't a loyalty ability."
    );
    supported("Sublime Epiphany");
    supported("Chandra, Torch of Defiance");
    let mut t = TestGame::new(2);
    // A mana ability doesn't use the stack: nothing to counter.
    let elves = t.battlefield(P1, "Llanowar Elves");
    activate_containing(&mut t, P1, elves, "Add").expect("mana ability");
    assert_eq!(t.stack_len(), 0);
    // Chandra's "+1: Add {R}{R}." is a loyalty ability, not a mana ability: it uses the
    // stack and can be countered.
    t.set_step(P1, Step::PrecombatMain);
    let chandra = t.battlefield(P1, "Chandra, Torch of Defiance");
    activate_containing(&mut t, P1, chandra, "Add {R}{R}").expect("activation");
    assert_eq!(t.stack_len(), 1);
    let ability = *t.g.stack.last().unwrap();
    lands_for(&mut t, P0, "{4}{U}{U}");
    let ep = t.hand(P0, "Sublime Epiphany");
    t.g.turn.priority = Some(P0);
    t.cast(P0, ep).modes(&[1]).target(obj(ability)).go();
    t.resolve_all();
    let pool = &t.g.player(P1).mana_pool;
    assert_eq!(pool.count(mtg_engine::mana::ManaType::R), 0);
}

// --- Faerie Artisans, Moonlit Meditation ---------------------------------------------------------

#[test]
fn each_faerie_artisans_exiles_only_its_own_tokens() {
    cr!("607.2a", "603.2");
    ruling!(
        "Faerie Artisans",
        "If you control multiple Faerie Artisans, each one’s ability only exiles tokens created with the ability of that specific Faerie Artisans."
    );
    supported("Faerie Artisans");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Faerie Artisans");
    t.battlefield(P0, "Faerie Artisans");
    t.enter(P1, "Grizzly Bears");
    t.settle();
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0).len(), 2);
    t.enter(P1, "Hill Giant");
    t.settle();
    t.resolve_all();
    let toks = tokens_of(&t, P0);
    assert_eq!(toks.len(), 2);
    assert!(toks.iter().all(|x| name_of(&t, *x) == "Hill Giant"));
}

#[test]
fn moonlit_meditation_counts_tokens_created_before_you_controlled_it() {
    cr!("614.1a", "111.1");
    ruling!(
        "Moonlit Meditation",
        "If you create one or more tokens, and then Moonlit Meditation comes under your control that same turn, the replacement effect won’t apply to any tokens you create for the rest of the turn. You’ll have to wait until the next turn."
    );
    supported("Moonlit Meditation");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    raise_the_alarm(&mut t, P0);
    t.lands(P0, "Island", 3);
    t.answer_targets(P0, &[obj(giant)]);
    let m = t.hand(P0, "Moonlit Meditation");
    t.cast(P0, m).go();
    t.resolve_all();
    t.answer_yes(P0, true);
    raise_the_alarm(&mut t, P0);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(t.named_on_battlefield("Soldier Token").len(), 4);
    // On P0's next turn it applies.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    raise_the_alarm(&mut t, P0);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 3);
    assert_eq!(t.named_on_battlefield("Soldier Token").len(), 4);
}

// --- Choices made on resolution --------------------------------------------------------------

#[test]
fn see_double_both_modes_dont_care_about_the_graveyard_afterwards() {
    cr!("700.2", "608.2b");
    ruling!(
        "See Double",
        "Once you’ve cast See Double and legally chosen both modes, it doesn’t matter what happens to the number of cards in opponents’ graveyards."
    );
    supported("See Double");
    let mut t = TestGame::new(2);
    let gy: Vec<_> = (0..8).map(|_| t.graveyard(P1, "Forest")).collect();
    let bears = t.battlefield(P0, "Grizzly Bears");
    // P1's Lightning Bolt targeting P0 is on the stack.
    lands_for(&mut t, P1, "{R}");
    let bolt = t.hand(P1, "Lightning Bolt");
    t.g.turn.priority = Some(P1);
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    lands_for(&mut t, P0, "{2}{U}{U}");
    let sd = t.hand(P0, "See Double");
    t.g.turn.priority = Some(P0);
    let before = t.g.battlefield.clone();
    t.cast(P0, sd)
        .modes(&[0, 1])
        .targets(&[obj(bolt), obj(bears)])
        .go();
    // In response, P1's graveyard is exiled.
    for c in gy {
        t.g.move_object(c, Zone::Exile, MoveCause::Effect, Some(P1));
    }
    assert_eq!(t.graveyard_size(P1), 0);
    t.answer_yes(P0, false);
    t.resolve_all();
    // The Bolt and its copy each dealt 3 to P0, and the Bears were copied.
    assert_eq!(t.life(P0), 14);
    assert_eq!(new_tokens(&t, P0, &before).len(), 1);
}

#[test]
fn season_of_weaving_chooses_what_to_copy_as_it_resolves() {
    cr!("608.2d", "700.2h");
    ruling!(
        "Season of Weaving",
        "The choice made for the second mode of Season of Weaving isn't made until the spell is resolving."
    );
    supported("Season of Weaving");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    let season = t.hand(P0, "Season of Weaving");
    t.cast(P0, season).modes(&[1]).go();
    // A creature that enters in response can be chosen.
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(giant)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 2);
}

#[test]
fn wall_of_stolen_identity_taps_the_copied_creature_with_a_reflexive_trigger() {
    cr!("603.12", "707.9a");
    ruling!(
        "Wall of Stolen Identity",
        "Once Wall of Stolen Identity has entered the battlefield and stolen a creature’s identity, players can take actions before the creature is tapped."
    );
    supported("Wall of Stolen Identity");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let wall = enter_copying(&mut t, P0, "Wall of Stolen Identity", giant);
    assert_eq!(name_of(&t, wall), "Hill Giant");
    // The tap is a triggered ability waiting on the stack.
    assert!(!t.obj_now(giant).tapped);
    assert_eq!(triggers_on_stack(&t).len(), 1);
    t.resolve_all();
    assert!(t.obj_now(giant).tapped);
}

#[test]
fn zndrsplts_judgment_triggers_wait_for_the_foes() {
    cr!("603.3", "608.2e");
    ruling!(
        "Zndrsplt's Judgment",
        "Players can't take actions in between the time that friends perform their actions and the time that foes perform theirs. If any abilities trigger from friends performing actions, those triggers won't be put onto the stack until the foes perform their actions and the spell finishes resolving."
    );
    supported("Zndrsplt's Judgment");
    let mut t = TestGame::new(2);
    // Soul Warden: "Whenever another creature enters, you gain 1 life."
    t.battlefield(P0, "Soul Warden");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P1, "Savannah Lions");
    lands_for(&mut t, P0, "{4}{U}");
    let j = t.hand(P0, "Zndrsplt's Judgment");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_choose(P1, &[obj(lions)]);
    t.cast(P0, j).go();
    t.resolve();
    // The spell has finished: the Lions are already in P1's hand, and the Soul Warden
    // trigger is on the stack.
    assert!(t.in_hand(P1, "Savannah Lions"));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
    t.settle();
    assert_eq!(triggers_on_stack(&t).len(), 1);
    let life = t.life(P0);
    t.resolve_all();
    assert_eq!(t.life(P0), life + 1);
}

// --- Entering as a copy and other replacement effects ---------------------------------------

/// Casts `p`'s Mystic Reflection choosing `what`.
fn mystic_reflection(t: &mut TestGame, p: PlayerId, what: ObjectId) {
    lands_for(t, p, "{1}{U}");
    let mr = t.hand(p, "Mystic Reflection");
    t.cast(p, mr).target(obj(what)).go();
    t.resolve_all();
}

#[test]
fn mystic_reflection_uses_the_copiable_values_as_the_permanents_enter() {
    cr!("707.2", "614.1c", "608.2h");
    ruling!(
        "Mystic Reflection",
        "In most cases, the copiable values of the target creature won’t change between Mystic Reflection resolving and the next time one or more creatures or planeswalkers enter the battlefield. In some unusual cases, they might, and if they do, use the copiable values of the creature at the time the permanents enter the battlefield."
    );
    supported("Mystic Reflection");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    mystic_reflection(&mut t, P0, bears);
    // The Bears becomes a copy of a Hill Giant (Cytoshape) after Mystic Reflection
    // resolves.
    let giant = t.battlefield(P1, "Hill Giant");
    lands_for(&mut t, P0, "{1}{G}{U}");
    let cyto = t.hand(P0, "Cytoshape");
    t.answer_choose(P0, &[obj(giant)]);
    t.cast(P0, cyto).target(obj(bears)).go();
    t.resolve_all();
    assert_eq!(name_of(&t, bears), "Hill Giant");
    let e = t.enter(P0, "Savannah Lions");
    t.settle();
    assert_eq!(name_of(&t, e), "Hill Giant");
}

#[test]
fn mystic_reflection_copies_dont_get_their_own_enters_replacements_but_get_external_ones() {
    cr!("614.12", "616.1", "707.2");
    ruling!(
        "Mystic Reflection",
        "Other enters-the-battlefield replacement abilities printed on the creatures entering the battlefield won’t be applied because those permanents will already be copies of the target creature at that point (and therefore they won’t have those abilities). External abilities may still affect how those permanents enter the battlefield."
    );
    supported("Mystic Reflection");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Imposing Sovereign: "Creatures your opponents control enter tapped."
    t.battlefield(P0, "Imposing Sovereign");
    mystic_reflection(&mut t, P0, bears);
    // P1's Kalonian Hydra ("enters with four +1/+1 counters") enters as a Grizzly Bears:
    // no counters, but tapped.
    let h = t.enter(P1, "Kalonian Hydra");
    t.settle();
    assert_eq!(name_of(&t, h), "Grizzly Bears");
    assert_eq!(t.counters(h, counters::PLUS1), 0);
    assert!(t.obj_now(h).tapped);
}

#[test]
fn control_changing_then_copy_then_other_enters_replacements() {
    cr!("614.12", "616.1", "707.2");
    ruling!(
        "Mystic Reflection",
        "Replacement effects that modify how a creature enters the battlefield are applied in the following order: first, control-changing effects (such as Gather Specimens), then copy effects (such as the abilities of Mystic Reflection and Clone), then all other effects."
    );
    ruling!(
        "Infinite Reflection",
        "Replacement effects that modify how a permanent enters are now applied in the following order: control-changing effects (such as Gather Specimens), then copy effects (such as the abilities of Infinite Reflection and Clone), then all other effects."
    );
    supported("Gather Specimens");
    // Gather Specimens: "If a creature would enter under an opponent's control this turn,
    // it enters under your control instead."
    let gather = |t: &mut TestGame| {
        lands_for(t, P0, "{3}{U}{U}{U}");
        let g = t.hand(P0, "Gather Specimens");
        t.cast(P0, g).go();
        t.resolve_all();
    };
    // Mystic Reflection: P1's Kalonian Hydra enters under P0's control as a Grizzly Bears,
    // tapped by P1's Imposing Sovereign (P0 is P1's opponent).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Imposing Sovereign");
    gather(&mut t);
    mystic_reflection(&mut t, P0, bears);
    let h = t.enter(P1, "Kalonian Hydra");
    t.settle();
    assert_eq!(t.obj_now(h).controller, P0);
    assert_eq!(name_of(&t, h), "Grizzly Bears");
    assert_eq!(t.counters(h, counters::PLUS1), 0);
    assert!(t.obj_now(h).tapped);
    // Infinite Reflection: "Nontoken creatures you control enter as a copy of enchanted
    // creature." P1's Hydra enters under P0's control, so it enters as a copy.
    supported("Infinite Reflection");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let aura = t.battlefield(P0, "Infinite Reflection");
    t.g.obj_mut(aura).attached_to = Some(Entity::Object(giant));
    t.g.recompute();
    t.battlefield(P1, "Imposing Sovereign");
    gather(&mut t);
    let h = t.enter(P1, "Kalonian Hydra");
    t.settle();
    assert_eq!(t.obj_now(h).controller, P0);
    assert_eq!(name_of(&t, h), "Hill Giant");
    assert_eq!(t.counters(h, counters::PLUS1), 0);
    assert!(t.obj_now(h).tapped);
}

#[test]
fn fractured_realm_doesnt_double_replacement_effects() {
    cr!("614.1c", "709.5", "603.2d");
    ruling!(
        "Mirror Room // Fractured Realm",
        "Replacement effects are unaffected by Fractured Realm's ability. For example, a 1/1 creature that enters under your control \"with a +1/+1 counter on it\" won't receive an additional +1/+1 counter."
    );
    supported("Mirror Room // Fractured Realm");
    let mut t = TestGame::new(2);
    lands_for(&mut t, P0, "{5}{U}{U}");
    let card = t.hand(P0, "Mirror Room // Fractured Realm");
    t.cast(P0, card).method(CastMethod::Half(1)).go();
    t.resolve_all();
    let h = t.enter(P0, "Kalonian Hydra");
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(h, counters::PLUS1), 4);
}
