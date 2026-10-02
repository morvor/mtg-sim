//! Rulings batch P207 — enchant (CR 303.4, 702.5): Auras that grant activated or
//! triggered abilities, Auras with leaves-the-battlefield returns, intervening "if"
//! clauses about the enchanted creature, and Auras whose enchant restriction depends on
//! control.

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s04_common::*;
use crate::r_s05_common::*;
use crate::r_s06_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn cast_on(t: &mut TestGame, p: PlayerId, name: &str, target: ObjectId) {
    let c = in_hand_with_mana(t, p, name);
    t.g.turn.priority = Some(p);
    t.cast(p, c).target(target).go();
    t.resolve_all();
}

// ---------------------------------------------------------------------------------------
// Granted activated abilities
// ---------------------------------------------------------------------------------------

#[test]
fn trollhides_regeneration_shield_saves_only_from_destruction_this_turn() {
    cr!("701.19a", "701.19b", "701.19c", "614.8");
    ruling!(
        "Trollhide",
        "Activating the ability granted to the enchanted creature causes a “regeneration shield” to be created for it. The next time that creature would be destroyed that turn, the regeneration shield is used up instead. This works only if the creature is dealt lethal damage, dealt damage from a source with deathtouch, or affected by a spell or ability that says to “destroy” it. Other effects that cause the creature to be put into the graveyard (such as reducing its toughness to 0 or sacrificing it) don’t destroy it, so regeneration won’t save it. If it hasn’t been used, the regeneration shield goes away as the turn ends."
    );
    supported("Trollhide");
    supported("Grasp of Darkness");
    let setup = || {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        attach_new(&mut t, P0, "Trollhide", bears);
        assert_eq!(t.pt(bears), (4, 4));
        t.lands(P0, "Forest", 2);
        activate_containing(&mut t, P0, bears, "Regenerate").unwrap();
        t.resolve_all();
        (t, bears)
    };
    // "Destroy": regenerated (tapped).
    let (mut t, bears) = setup();
    cast_on(&mut t, P1, "Murder", bears);
    assert!(t.on_battlefield(bears));
    assert!(t.obj_now(bears).tapped);
    // Lethal damage: regenerated, damage removed.
    let (mut t, bears) = setup();
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, giant, 4, bears);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
    // Toughness 0 (Grasp of Darkness, -4/-4): not destroyed, so not regenerated.
    let (mut t, bears) = setup();
    cast_on(&mut t, P1, "Grasp of Darkness", bears);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // The shield goes away as the turn ends.
    let (mut t, bears) = setup();
    t.advance_to(P1, Step::Upkeep);
    destroy(&mut t, bears);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn compulsory_rests_ability_belongs_to_the_creatures_controller() {
    cr!("113.6", "602.2", "109.5");
    ruling!(
        "Compulsory Rest",
        "Compulsory Rest grants the activated ability to the enchanted creature. That creature’s controller (not the controller of Compulsory Rest) may activate the ability and gain 2 life."
    );
    supported("Compulsory Rest");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Compulsory Rest", bears);
    t.lands(P0, "Wastes", 2);
    t.lands(P1, "Wastes", 2);
    assert!(!can_activate(&mut t, P0, bears));
    activate_containing(&mut t, P1, bears, "gain 2 life").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 22);
    assert_eq!(t.life(P0), 20);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn detainment_spell_stops_mana_abilities() {
    cr!("602.5", "605.1a");
    ruling!(
        "Detainment Spell",
        "Detainment Spell prevents mana abilities of the enchanted creature from being activated."
    );
    supported("Detainment Spell");
    supported("Llanowar Elves");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    attach_new(&mut t, P1, "Detainment Spell", elves);
    assert!(t.activate(P0, elves, 0, &[]).is_err());
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    // Without it, the Elves' mana ability can be activated.
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    assert!(t.activate(P0, elves, 0, &[]).is_ok());
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
}

#[test]
fn splinter_twin_tokens_get_enters_abilities_and_enters_with_counters() {
    cr!("707.2", "603.6a", "614.1c", "111.1");
    ruling!(
        "Splinter Twin",
        "Any \"enters\" abilities of the enchanted creature will trigger when the token is put onto the battlefield. Any \"as [this creature] enters\" or \"[this creature] enters with\" abilities of the enchanted creature will also work."
    );
    supported("Splinter Twin");
    supported("Elvish Visionary");
    supported("Servant of the Scale");
    // Elvish Visionary: "When this creature enters, draw a card."
    let mut t = TestGame::new(2);
    let visionary = t.battlefield(P0, "Elvish Visionary");
    attach_new(&mut t, P0, "Splinter Twin", visionary);
    let hand = t.hand_size(P0);
    activate_containing(&mut t, P0, visionary, "Create a token").unwrap();
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert!(has_kw(&t, toks[0], KeywordKind::Haste));
    assert_eq!(t.hand_size(P0), hand + 1);
    // Servant of the Scale: "This creature enters with a +1/+1 counter on it."
    let mut t = TestGame::new(2);
    let servant = t.battlefield(P0, "Servant of the Scale");
    attach_new(&mut t, P0, "Splinter Twin", servant);
    activate_containing(&mut t, P0, servant, "Create a token").unwrap();
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.counters(toks[0], counters::PLUS1), 1);
}

#[test]
fn shattered_ego_puts_the_creature_into_its_library_then_goes_to_the_graveyard() {
    cr!("704.5m", "608.2");
    ruling!(
        "Shattered Ego",
        "As the last ability resolves, the enchanted creature is put into its owner's library. After it resolves, Shattered Ego is put into the graveyard from the battlefield."
    );
    supported("Shattered Ego");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let ego = attach_new(&mut t, P0, "Shattered Ego", giant);
    assert_eq!(t.pt(giant), (0, 3));
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 3);
    t.activate(P0, ego, 0, &[]).unwrap();
    t.g.resolve_top();
    // Right after the ability resolves: the Giant is third from the top of P1's library,
    // and the Aura is still on the battlefield until state-based actions are checked.
    let lib = &t.g.player(P1).library;
    let third = lib[lib.len() - 3];
    assert_eq!(t.g.obj(third).chars.name, "Hill Giant");
    assert!(t.on_battlefield(ego));
    t.settle();
    assert!(!t.on_battlefield(ego));
    assert!(t.in_graveyard(P0, "Shattered Ego"));
}

// ---------------------------------------------------------------------------------------
// Granted triggered abilities
// ---------------------------------------------------------------------------------------

#[test]
fn inevitable_end_triggers_in_the_creature_controllers_upkeep_who_chooses() {
    cr!("113.6", "603.3a", "701.21a");
    ruling!(
        "Inevitable End",
        "Because Inevitable End grants the ability to the enchanted creature, the triggered ability triggers at the beginning of the upkeep of the controller of the enchanted creature, and that player chooses a creature to sacrifice. They might not sacrifice the enchanted creature right away, but it’s certain that they’ll have to do so eventually."
    );
    supported("Inevitable End");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    attach_new(&mut t, P0, "Inevitable End", bears);
    // In P1's upkeep, P1 sacrifices the Hill Giant.
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "sacrifice a creature"), 1);
    t.answer_choose(P1, &[Entity::Object(giant)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.on_battlefield(bears));
    // Not in P0's upkeep.
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "sacrifice a creature"), 0);
    // Next time, only the Bears is left.
    next_upkeep(&mut t, P1);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

// ---------------------------------------------------------------------------------------
// Leaving the battlefield
// ---------------------------------------------------------------------------------------

#[test]
fn returning_conviction_can_make_marked_damage_lethal() {
    cr!("120.6", "704.5g");
    ruling!(
        "Conviction",
        "Because damage remains marked on a creature until it's removed as the turn ends, nonlethal damage dealt to the enchanted creature may become lethal if you return Conviction to its owner's hand during that turn."
    );
    supported("Conviction");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let conviction = attach_new(&mut t, P0, "Conviction", bears);
    assert_eq!(t.pt(bears), (3, 5));
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, giant, 3, bears);
    assert!(t.on_battlefield(bears));
    t.lands(P0, "Plains", 1);
    t.activate(P0, conviction, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Conviction"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn demonic_vigor_doesnt_return_a_token() {
    cr!("111.7", "704.5d", "603.10a");
    ruling!(
        "Demonic Vigor",
        "Demonic Vigor can enchant a token, but its last ability won’t return the token to your hand."
    );
    supported("Demonic Vigor");
    let mut t = TestGame::new(2);
    let token = create_token(&mut t, P0, "Soldier");
    let vigor = cast_aura_p0(&mut t, "Demonic Vigor", token);
    assert_eq!(attached_to(&t, vigor), Some(Entity::Object(token)));
    assert_eq!(t.pt(token), (2, 2));
    let hand = t.hand_size(P0);
    destroy(&mut t, token);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert!(tokens(&t, P0).is_empty());
    // A nontoken creature card is returned.
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Demonic Vigor", bears);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

fn cast_aura_p0(t: &mut TestGame, name: &str, target: ObjectId) -> ObjectId {
    let c = in_hand_with_mana(t, P0, name);
    t.cast(P0, c).target(target).go();
    t.resolve_all();
    t.g.current(c)
}

#[test]
fn fruit_of_the_first_tree_rewards_its_controller_on_any_creature() {
    cr!("109.5", "603.10a");
    ruling!(
        "Fruit of the First Tree",
        "Fruit of the First Tree can enchant any creature, but Fruit of the First Tree’s controller will gain life and draw cards."
    );
    supported("Fruit of the First Tree");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    attach_new(&mut t, P0, "Fruit of the First Tree", giant);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    destroy(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.hand_size(P0), h0 + 3);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P1), h1);
}

#[test]
fn fire_whip_goes_to_the_graveyard_if_you_lose_control_of_the_creature() {
    cr!("303.4d", "704.5m");
    ruling!(
        "Fire Whip",
        "Fire Whip is put into the graveyard if you lose control of the creature since the card text says it can only enchant a creature you control."
    );
    supported("Fire Whip");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let whip = attach_new(&mut t, P0, "Fire Whip", bears);
    give_control(&mut t, bears, P1);
    assert!(!t.on_battlefield(whip));
    assert!(t.in_graveyard(P0, "Fire Whip"));
    assert!(t.on_battlefield(bears));
}

#[test]
fn emblem_of_the_warmind_gives_haste_to_all_your_creatures() {
    cr!("702.10b", "303.4");
    ruling!(
        "Emblem of the Warmind",
        "Although this is an Aura, Emblem of the Warmind has no ability that specifically references the enchanted creature."
    );
    supported("Emblem of the Warmind");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Emblem of the Warmind", bears);
    let sick = t.battlefield_sick(P0, "Hill Giant");
    t.g.recompute();
    assert!(has_kw(&t, bears, KeywordKind::Haste));
    assert!(has_kw(&t, sick, KeywordKind::Haste));
    // P1's creatures don't get it.
    let theirs = t.battlefield(P1, "Hill Giant");
    t.g.recompute();
    assert!(!has_kw(&t, theirs, KeywordKind::Haste));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(sick, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 17);
}

// ---------------------------------------------------------------------------------------
// Intervening "if" clauses about the enchanted creature's power
// ---------------------------------------------------------------------------------------

/// Humble ("Until end of turn, target creature loses all abilities and has base power and
/// toughness 0/1"), cast by `p`.
fn humble(t: &mut TestGame, p: PlayerId, target: ObjectId) {
    supported("Humble");
    cast_on(t, p, "Humble", target);
}

#[test]
fn domestication_can_take_a_big_creature_and_checks_its_power_only_at_end_step() {
    cr!("603.4", "303.4", "613.4b");
    ruling!(
        "Domestication",
        "Domestication can target, and can enchant, a creature with power 4 or greater. The enchanted creature's power is checked only when the triggered ability triggers and resolves."
    );
    ruling!(
        "Domestication",
        "Domestication's triggered ability has an \"intervening 'if' clause.\" That means (1) the ability triggers only if the enchanted creature's power is 4 or greater as your end step begins, and (2) the ability does nothing if the enchanted creature's power is 3 or less by the time it resolves."
    );
    supported("Domestication");
    supported("Craw Wurm");
    // Craw Wurm (6/4): P0 gains control of it; at P0's end step Domestication is
    // sacrificed and P1 gets it back.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let aura = cast_aura_p0(&mut t, "Domestication", wurm);
    assert_eq!(attached_to(&t, aura), Some(Entity::Object(wurm)));
    assert_eq!(t.obj_now(wurm).controller, P0);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "sacrifice"), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(aura));
    assert_eq!(t.obj_now(wurm).controller, P1);
    // Its power is 3 or less by the time the ability resolves: nothing happens.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let aura = cast_aura_p0(&mut t, "Domestication", wurm);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "sacrifice"), 1);
    humble(&mut t, P0, wurm);
    assert!(t.on_battlefield(aura));
    assert_eq!(t.obj_now(wurm).controller, P0);
    // Its power is 3 or less as the end step begins: it doesn't trigger.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let aura = cast_aura_p0(&mut t, "Domestication", wurm);
    humble(&mut t, P0, wurm);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "sacrifice"), 0);
    assert!(t.on_battlefield(aura));
}

#[test]
fn arachnus_web_isnt_destroyed_if_the_power_drops_before_it_resolves() {
    cr!("603.4");
    ruling!(
        "Arachnus Web",
        "If Arachnus Web's last ability triggers, but the enchanted creature's power is reduced to 3 or less before the ability resolves, the ability will have no effect. Arachnus Web won't be destroyed."
    );
    supported("Arachnus Web");
    supported("Craw Wurm");
    // It triggers at the beginning of each end step (P0's here, though P1 controls the
    // enchanted creature).
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let web = attach_new(&mut t, P0, "Arachnus Web", wurm);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "destroy"), 1);
    humble(&mut t, P0, wurm);
    assert!(t.on_battlefield(web));
    // Without the Humble, it's destroyed.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let web = attach_new(&mut t, P0, "Arachnus Web", wurm);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(web));
    assert!(t.in_graveyard(P0, "Arachnus Web"));
}

#[test]
fn historians_wisdom_checks_the_greatest_power_on_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Historian's Wisdom",
        "Historian's Wisdom has an intervening \"if\" clause in its triggered ability. This means that you will draw a card only if the enchanted permanent is a creature with the greatest power both at the time the ability triggers and at the time that it resolves."
    );
    supported("Historian's Wisdom");
    // Grizzly Bears (4/3 with it) against Hill Giant (3/3): a card is drawn.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    let hand = t.hand_size(P0);
    let w = in_hand_with_mana(&mut t, P0, "Historian's Wisdom");
    t.cast(P0, w).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (4, 3));
    assert_eq!(triggers_on_stack(&t, "greatest power"), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // P1 responds with Giant Growth on the Hill Giant (6/6): no card.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant2 = t.battlefield(P1, "Hill Giant");
    let hand = t.hand_size(P0);
    let w = in_hand_with_mana(&mut t, P0, "Historian's Wisdom");
    t.cast(P0, w).target(bears).go();
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "greatest power"), 1);
    cast_on(&mut t, P1, "Giant Growth", giant2);
    assert_eq!(t.hand_size(P0), hand);
    // Against Craw Wurm (6/4), it doesn't trigger at all.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Craw Wurm");
    let w = in_hand_with_mana(&mut t, P0, "Historian's Wisdom");
    t.cast(P0, w).target(bears).go();
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "greatest power"), 0);
}
