//! Paper-scope statics, permissions and characteristics (cards4-paper-combat-statics):
//! Guardian Beast, Serra Bestiary, Autumn Willow, Fastbond's land plays, "can't be
//! enchanted"; Wood Elemental, Dracoplasm, Angry Mob, Aurochs, Liu Bei, Call to Arms,
//! Shapeshifter.

use crate::basic_effects_common::oracle_card;
use crate::combat2_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The candidates `p` was offered for the last target choice they made (for the spell
/// `_spell` just cast).
fn offered_targets(t: &TestGame, p: PlayerId, _spell: ObjectId) -> Vec<Entity> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(q, d)| match d {
            Decision::ChooseTargets { candidates, .. } if q == p => Some(candidates),
            _ => None,
        })
        .expect("no target choice")
}

// ---------------------------------------------------------------------------------------
// Guardian Beast
// ---------------------------------------------------------------------------------------

#[test]
fn guardian_beast_protects_noncreature_artifacts_while_untapped() {
    cr!("303.4i", "702.5a", "702.12b");
    supported("Guardian Beast");
    let mut t = TestGame::new(2);
    let beast = t.battlefield(P0, "Guardian Beast");
    let ring = t.battlefield(P0, "Sol Ring");
    let aura = t.hand(P1, "Animate Artifact");
    let shatter = t.hand(P1, "Shatter");
    t.lands(P1, "Island", 4);
    t.lands(P1, "Mountain", 2);
    t.set_step(P1, Step::PrecombatMain);
    // Can't be enchanted: the Aura spell can target it (its enchant ability defines its
    // target, CR 702.5a), but it can't be attached, so it's put into the graveyard as it
    // resolves (CR 303.4i).
    t.cast(P1, aura).target(ring).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Animate Artifact"));
    assert!(t.g.attachments_of(Entity::Object(ring)).is_empty());
    assert!(!t.obj_now(ring).is_creature());
    // Indestructible.
    t.cast(P1, shatter).target(ring).go();
    t.resolve();
    assert!(t.on_battlefield(ring));
    assert!(t.on_battlefield(beast));
}

#[test]
fn guardian_beast_stops_other_players_gaining_control_but_not_sacrifices() {
    cr!("613.1b");
    interp!("control-gain-prohibition-before-controller");
    ruling!("Guardian Beast", "This does not stop sacrifices.");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Guardian Beast");
    let ring = t.battlefield(P0, "Sol Ring");
    let mastery = t.hand(P1, "Metallic Mastery");
    t.lands(P1, "Mountain", 3);
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, mastery).target(ring).go();
    t.resolve();
    assert_eq!(controller(&t, ring), P0);
    // Its controller can still sacrifice it.
    let atog = t.battlefield(P0, "Atog");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Object(ring)]);
    t.activate(P0, atog, 0, &[]).unwrap();
    t.resolve();
    assert!(t.in_graveyard(P0, "Sol Ring"));
}

#[test]
fn guardian_beast_does_nothing_while_tapped_and_leaves_auras_attached() {
    cr!("303.4c", "613.1b");
    interp!("control-gain-prohibition-before-controller");
    let mut t = TestGame::new(2);
    let beast = t.battlefield(P0, "Guardian Beast");
    let ring = t.battlefield(P0, "Sol Ring");
    let icy = t.battlefield(P1, "Icy Manipulator");
    let steal = t.hand(P1, "Steal Artifact");
    t.lands(P1, "Island", 5);
    t.set_step(P1, Step::PrecombatMain);
    t.activate(P1, icy, 0, &[Entity::Object(beast)]).unwrap();
    t.resolve();
    assert!(t.obj_now(beast).tapped);
    // While it's tapped, the artifact can be enchanted and stolen.
    t.cast(P1, steal).target(ring).go();
    t.resolve();
    let steal_now = t.g.current(steal);
    assert_eq!(t.obj_now(steal_now).attached_to, Some(Entity::Object(ring)));
    assert_eq!(controller(&t, ring), P1);
    // Untapped again: the Aura stays attached ("This effect doesn't remove Auras already
    // attached") and the player who gained control keeps it.
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(beast).tapped);
    assert!(t.on_battlefield(steal_now));
    assert_eq!(t.obj_now(steal_now).attached_to, Some(Entity::Object(ring)));
    assert_eq!(controller(&t, ring), P1);
}

#[test]
fn a_permanent_that_cant_be_enchanted_loses_its_auras() {
    cr!("303.4c");
    // A plain "can't be enchanted" (as Tetravus' tokens have) makes an attached Aura
    // illegal, unlike Guardian Beast's.
    let ward = oracle_card(
        "Anti-Aura Test",
        "Enchantment",
        "Artifacts you control can't be enchanted.",
    );
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Sol Ring");
    let aura = t.hand(P1, "Animate Artifact");
    t.lands(P1, "Island", 4);
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, aura).target(ring).go();
    t.resolve();
    assert!(t.on_battlefield(t.g.current(aura)));
    t.custom(P0, ward, Zone::Battlefield);
    t.settle();
    assert!(t.in_graveyard(P1, "Animate Artifact"));
}

// ---------------------------------------------------------------------------------------
// Serra Bestiary, Autumn Willow, Fastbond
// ---------------------------------------------------------------------------------------

#[test]
fn serra_bestiary_stops_attacks_blocks_and_tap_abilities() {
    cr!("508.1c", "602.5");
    ruling!(
        "Serra Bestiary",
        "Does prevent the use of activated mana abilities with {T} in the cost."
    );
    supported("Serra Bestiary");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let other = t.battlefield(P0, "Llanowar Elves");
    let bestiary = t.battlefield(P1, "Serra Bestiary");
    t.g.attach(bestiary, Entity::Object(elves));
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &[(elves, Entity::Player(P1))]));
    assert!(legal_attack(&mut t, &[(other, Entity::Player(P1))]));
    assert!(t.activate(P0, elves, 0, &[]).is_err());
    assert!(t.activate(P0, other, 0, &[]).is_ok());
}

#[test]
fn autumn_willow_can_be_targeted_by_the_chosen_player_until_end_of_turn() {
    cr!("702.18a", "609.4");
    supported("Autumn Willow");
    let mut t = TestGame::new(2);
    let willow = t.battlefield(P0, "Autumn Willow");
    t.lands(P0, "Forest", 1);
    let shock = t.hand(P1, "Shock");
    let own = t.hand(P0, "Shock");
    t.lands(P1, "Mountain", 1);
    t.lands(P0, "Mountain", 1);
    // Before: nobody can target it.
    t.cast(P1, shock).target(Entity::Player(P0)).go();
    assert!(!offered_targets(&t, P1, shock).contains(&Entity::Object(willow)));
    t.resolve();
    let shock2 = t.hand(P1, "Shock");
    t.lands(P1, "Mountain", 1);
    t.activate(P0, willow, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    // Its controller still can't (only the target player can).
    t.cast(P0, own).target(Entity::Player(P1)).go();
    assert!(!offered_targets(&t, P0, own).contains(&Entity::Object(willow)));
    t.resolve();
    t.cast(P1, shock2).target(willow).go();
    assert!(offered_targets(&t, P1, shock2).contains(&Entity::Object(willow)));
    t.resolve();
    assert_eq!(t.obj_now(willow).damage, 2);
    // Until end of turn only.
    t.advance_to(P1, Step::PrecombatMain);
    let shock3 = t.hand(P1, "Shock");
    t.cast(P1, shock3).target(Entity::Player(P0)).go();
    assert!(!offered_targets(&t, P1, shock3).contains(&Entity::Object(willow)));
}

#[test]
fn fastbond_lets_you_play_any_number_of_lands_for_1_damage_each_after_the_first() {
    cr!("305.2", "305.2a");
    ruling!(
        "Fastbond",
        "You take damage when you play a land using the \"play a land\" action."
    );
    ruling!(
        "Fastbond",
        "You do not take damage when you \"put a land onto the battlefield\" through the effect of a spell or ability."
    );
    supported("Fastbond");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fastbond");
    let lands: Vec<ObjectId> = (0..4).map(|_| t.hand(P0, "Forest")).collect();
    t.play_land(P0, lands[0]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "the first land");
    // A land put onto the battlefield isn't played.
    let growth = t.hand(P0, "Rampant Growth");
    t.lands(P0, "Forest", 2);
    let plains = t.library_top(P0, "Plains");
    t.answer_choose(P0, &[Entity::Object(plains)]);
    t.cast(P0, growth).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    for l in &lands[1..] {
        t.play_land(P0, *l).unwrap();
        t.resolve_all();
    }
    assert_eq!(t.life(P0), 17);
    assert!(lands.iter().all(|l| t.on_battlefield(t.g.current(*l))));
}

// ---------------------------------------------------------------------------------------
// Characteristics
// ---------------------------------------------------------------------------------------

#[test]
fn wood_elemental_is_as_big_as_the_forests_sacrificed_as_it_entered() {
    cr!("604.3", "614.12");
    ruling!(
        "Wood Elemental",
        "You have to do this sacrifice no matter how this card enters."
    );
    supported("Wood Elemental");
    let mut t = TestGame::new(2);
    let in_hand = t.hand(P0, "Wood Elemental");
    assert_eq!(t.pt(in_hand), (0, 0));
    let forests = t.lands(P0, "Forest", 3);
    t.answer_choose(
        P0,
        &[Entity::Object(forests[0]), Entity::Object(forests[1])],
    );
    let elemental = t.enter(P0, "Wood Elemental");
    t.settle();
    assert_eq!(t.pt(elemental), (2, 2));
    assert!(t.on_battlefield(forests[2]));
    assert_eq!(t.graveyard_size(P0), 2);
}

#[test]
fn dracoplasm_gets_the_total_power_and_toughness_of_the_sacrificed_creatures() {
    cr!("613.4b", "608.2h");
    supported("Dracoplasm");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let growth = t.hand(P0, "Giant Growth");
    t.lands(P0, "Forest", 1);
    t.cast(P0, growth).target(bears).go();
    t.resolve();
    t.answer_choose(P0, &[Entity::Object(bears), Entity::Object(giant)]);
    let draco = t.enter(P0, "Dracoplasm");
    t.settle();
    // 5/5 (as it last existed) plus 3/3.
    assert_eq!(t.pt(draco), (8, 8));
    t.lands(P0, "Mountain", 1);
    t.activate(P0, draco, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.pt(draco), (9, 8));
}

#[test]
fn angry_mob_counts_opposing_swamps_only_during_your_turn() {
    cr!("613.4b", "604.3a");
    interp!("conditional-pt-setting-not-cda");
    supported("Angry Mob");
    let mut t = TestGame::new(2);
    let mob = t.battlefield(P0, "Angry Mob");
    t.lands(P1, "Swamp", 3);
    t.lands(P0, "Swamp", 1);
    assert_eq!(t.pt(mob), (5, 5));
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(t.pt(mob), (2, 2));
}

#[test]
fn aurochs_counts_other_attacking_aurochs_by_creature_type() {
    cr!("205.3m");
    ruling!(
        "Aurochs",
        "The ability counts the number of attacking creatures with the creature type Aurochs, not just those named Aurochs."
    );
    supported("Aurochs");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Aurochs");
    let herd = t.battlefield(P0, "Aurochs Herd");
    let bears = t.battlefield(P0, "Grizzly Bears");
    declare(
        &mut t,
        P0,
        &[
            (a, Entity::Player(P1)),
            (herd, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
        ],
    );
    t.resolve_all();
    assert_eq!(t.pt(a), (3, 3));
}

#[test]
fn liu_bei_gets_bigger_with_guan_yu_or_zhang_fei() {
    cr!("201.2");
    supported("Liu Bei, Lord of Shu");
    let mut t = TestGame::new(2);
    let liu = t.battlefield(P0, "Liu Bei, Lord of Shu");
    assert_eq!(t.pt(liu), (2, 4));
    t.battlefield(P1, "Zhang Fei, Fierce Warrior");
    assert_eq!(t.pt(liu), (2, 4), "you must control it");
    let zhang = t.battlefield(P0, "Zhang Fei, Fierce Warrior");
    assert_eq!(t.pt(liu), (4, 6));
    t.g.destroy(zhang, None);
    t.g.recompute();
    assert_eq!(t.pt(liu), (2, 4));
    t.battlefield(P0, "Guan Yu, Sainted Warrior");
    assert_eq!(t.pt(liu), (4, 6));
}

#[test]
fn call_to_arms_needs_the_chosen_color_strictly_most_common() {
    cr!("603.8", "105.2");
    ruling!(
        "Call to Arms",
        "Multicolored ones are counted for each of their colors; colorless ones are ignored."
    );
    ruling!(
        "Call to Arms",
        "Call to Arms will be sacrificed when the third ability resolves"
    );
    supported("Call to Arms");
    let mut t = TestGame::new(2);
    let lions = t.battlefield(P0, "Savannah Lions");
    t.battlefield(P1, "Savannah Lions");
    t.battlefield(P1, "Glorious Anthem");
    t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Sol Ring");
    // White (option 0) and the only opponent.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let arms = t.enter(P0, "Call to Arms");
    t.settle();
    // White 2 (Lions, Anthem; the colorless Sol Ring doesn't count) against red 1.
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.pt(lions), (3, 2));
    // A red-and-white permanent counts for both: still more white.
    t.battlefield(P1, "Boros Guildmage");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.pt(lions), (3, 2));
    // A tie: no bonus, and the state trigger sacrifices it, even if white is the most
    // common again by the time it resolves.
    let goblin = t.battlefield(P1, "Raging Goblin");
    assert_eq!(t.pt(lions), (2, 1));
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.g.destroy(goblin, None);
    t.resolve_all();
    assert!(!t.on_battlefield(arms));
    assert!(t.in_graveyard(P0, "Call to Arms"));
}

#[test]
fn shapeshifter_is_the_last_chosen_number_over_seven_minus_it() {
    cr!("604.3", "607.2d", "208.2a");
    supported("Shapeshifter");
    let mut t = TestGame::new(2);
    let hand = t.hand(P0, "Shapeshifter");
    assert_eq!(t.pt(hand), (0, 7));
    t.answer(P0, DecisionKind::Number, Answer::Number(5));
    let s = t.enter(P0, "Shapeshifter");
    t.settle();
    assert_eq!(t.pt(s), (5, 2));
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Number, Answer::Number(1));
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.pt(s), (1, 6));
}
