//! Rulings batch P076 — cards newly compiled for this batch: counters put on two objects by
//! one instruction (Ajani, the Greathearted; Trygon Prime), "other than enchanted
//! creature" (Due Diligence), creatures assigning combat damage as though unblocked
//! (Predatory Focus), spells cast this turn (Murmuration), returning a card and attaching
//! the source to it (Pre-War Formalwear), divided counters with X fixed as the spell is
//! cast (Undercity Upheaval), and a scheme's "that player" condition with a shared life
//! total (You Cannot Hide from Me).

use crate::r_p076_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s06_common::has_kw;
use crate::r_s21_common::legal_blocks;
use crate::r_s25_common::cast_new;
use mtg_engine::ability::Modification;
use mtg_engine::card::{card, CardDef};
use mtg_engine::game::GameConfig;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Trygon Prime

#[test]
fn trygon_prime_that_creature_is_the_other_target() {
    cr!("115.1", "608.2b");
    ruling!(
        "Trygon Prime",
        "\"That creature\" refers to the other target attacking creature. It doesn't refer to Trygon Prime, even if you didn't choose a target."
    );
    supported("Trygon Prime");
    // With a target: the target gets a counter and can't be blocked; Trygon Prime can be.
    let mut t = TestGame::new(2);
    let trygon = t.battlefield(P0, "Trygon Prime");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    attack_with(
        &mut t,
        &[(trygon, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    t.resolve_all();
    assert_eq!(t.pt(trygon), (5, 5));
    assert_eq!(t.pt(bears), (3, 3));
    assert!(!legal_blocks(&mut t, P1, &[(giant, bears)]));
    assert!(legal_blocks(&mut t, P1, &[(giant, trygon)]));
    // Without a target: Trygon Prime still gets its counter and can still be blocked.
    let mut t = TestGame::new(2);
    let trygon = t.battlefield(P0, "Trygon Prime");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[]);
    attack_with(&mut t, &[(trygon, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(trygon), (5, 5));
    assert!(legal_blocks(&mut t, P1, &[(giant, trygon)]));
}

// ---------------------------------------------------------------------------------------
// Ajani, the Greathearted

/// P0's Ajani, the Greathearted activates its −2 ("Put a +1/+1 counter on each creature
/// you control and a loyalty counter on each other planeswalker you control.").
fn ajani_minus_two(t: &mut TestGame, ajani: ObjectId) {
    t.activate(P0, ajani, 1, &[]).unwrap();
    t.resolve_all();
}

/// Makes the permanent a 2/2 creature in addition to its other types until end of turn.
fn animate(t: &mut TestGame, id: ObjectId) {
    crate::r_s26_common::modify_until_eot(
        t,
        id,
        vec![
            Modification::AddTypes(vec![CardType::Creature]),
            Modification::SetPT(Some(mtg_engine::ability::Value::c(2)), Some(mtg_engine::ability::Value::c(2))),
        ],
    );
}

#[test]
fn ajani_the_greathearted_gets_a_counter_if_it_is_a_creature() {
    cr!("606.3", "122.1");
    ruling!(
        "Ajani, the Greathearted",
        "If Ajani is somehow a creature as his last ability resolves, he’ll get a +1/+1 counter."
    );
    supported("Ajani, the Greathearted");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let ajani = t.battlefield(P0, "Ajani, the Greathearted");
    let bears = t.battlefield(P0, "Grizzly Bears");
    animate(&mut t, ajani);
    ajani_minus_two(&mut t, ajani);
    assert_eq!(t.counters(ajani, "+1/+1"), 1);
    assert_eq!(t.pt(ajani), (3, 3));
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    // Not a loyalty counter: it isn't "other".
    assert_eq!(t.counters(ajani, "loyalty"), 3);
}

#[test]
fn ajani_the_greathearted_planeswalker_creature_gets_both_counters() {
    cr!("606.3", "122.1");
    ruling!(
        "Ajani, the Greathearted",
        "If a planeswalker you control is also a creature (most likely because it’s Gideon), that planeswalker receives both a +1/+1 counter and a loyalty counter as Ajani’s last ability resolves."
    );
    supported("Ajani, the Greathearted");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let ajani = t.battlefield(P0, "Ajani, the Greathearted");
    let basri = t.battlefield(P0, "Basri, Devoted Paladin");
    let loyalty = t.counters(basri, "loyalty");
    animate(&mut t, basri);
    ajani_minus_two(&mut t, ajani);
    assert_eq!(t.counters(basri, "+1/+1"), 1);
    assert_eq!(t.counters(basri, "loyalty"), loyalty + 1);
}

// ---------------------------------------------------------------------------------------
// Due Diligence

#[test]
fn due_diligence_trigger_fizzles_if_the_aura_moved_onto_its_target() {
    cr!("608.2b", "603.3d");
    ruling!(
        "Due Diligence",
        "In the unusual case where Due Diligence's second ability triggers and then Due Diligence becomes attached to the target of its own triggered ability before that ability resolves, when the triggered ability tries to resolve, it won't resolve and none of its effects will happen."
    );
    supported("Due Diligence");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    // Due Diligence on the Bears; its trigger targets the Giant.
    let aura = cast_new(&mut t, P0, "Due Diligence", &[Entity::Object(a)]);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.resolve(); // the Aura spell
    t.settle();
    assert_eq!(t.stack_len(), 1, "the enters trigger");
    let aura = t.g.current(aura);
    assert_eq!(t.pt(a), (4, 4));
    // Before it resolves, the Aura moves onto the Giant: the target is now illegal.
    assert!(t.g.attach(aura, Entity::Object(b)));
    t.g.recompute();
    t.resolve_all();
    // Only the Aura's static bonus: no +2/+2 from the trigger.
    assert_eq!(t.pt(b), (5, 5));
    assert_eq!(t.pt(a), (2, 2));
    // Control: when the Aura stays put, the target gets +2/+2 and vigilance.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    cast_new(&mut t, P0, "Due Diligence", &[Entity::Object(a)]);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.resolve_all();
    assert_eq!(t.pt(b), (5, 5));
    assert!(has_kw(&t, b, KeywordKind::Vigilance));
    assert_eq!(t.pt(a), (4, 4));
}

#[test]
fn due_diligence_cannot_target_the_enchanted_creature() {
    cr!("115.1", "303.4");
    supported("Due Diligence");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let a = t.battlefield(P0, "Grizzly Bears");
    // The only creature is the enchanted one: the trigger has no legal target.
    cast_new(&mut t, P0, "Due Diligence", &[Entity::Object(a)]);
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.pt(a), (4, 4));
}

// ---------------------------------------------------------------------------------------
// Predatory Focus

/// P0 casts Predatory Focus in their precombat main phase, choosing to use it or not.
fn predatory_focus(t: &mut TestGame, use_it: bool) {
    t.set_step(P0, Step::PrecombatMain);
    supported("Predatory Focus");
    t.answer_yes(P0, use_it);
    cast_new(t, P0, "Predatory Focus", &[]);
    t.resolve_all();
}

#[test]
fn predatory_focus_all_damage_to_the_player_if_chosen() {
    cr!("510.1b", "510.1c", "608.2c");
    ruling!(
        "Predatory Focus",
        "When Predatory Focus resolves, you choose whether to use its effect or not. If you choose to use it, all your creatures will deal their combat damage to the planeswalker or defending player this turn whether or not they become blocked. You can't have any of them deal combat damage to creatures that block them. If you choose not to use its effect, nothing happens."
    );
    // Used: both blocked attackers deal their damage to P1, none to the blockers.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    predatory_focus(&mut t, true);
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(
        &mut t,
        &[(giant, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    block_and_finish(&mut t, P1, &[(b1, giant), (b2, bears)]);
    assert_eq!(t.life(P1), 15);
    assert!(t.on_battlefield(b1) && t.on_battlefield(b2));
    assert_eq!(crate::r_s07_common::damage_on(&t, b1), 0);
    // Not used: nothing happens; the blocked attackers deal damage to the blockers.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    predatory_focus(&mut t, false);
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(b1, giant)]);
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(b1));
}

#[test]
fn predatory_focus_damage_goes_to_the_attacked_planeswalker() {
    cr!("510.1b", "506.4");
    ruling!(
        "Predatory Focus",
        "If a creature is attacking a planeswalker, assigning its damage as though it weren't blocked means the damage is assigned to the planeswalker, not to the defending player."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let pw = t.battlefield(P1, "Basri, Devoted Paladin");
    crate::r_s29_common::put_counters(&mut t, pw, "loyalty", 2);
    let loyalty = t.counters(pw, "loyalty");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    predatory_focus(&mut t, true);
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(giant, Entity::Object(pw))]);
    block_and_finish(&mut t, P1, &[(b1, giant)]);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.counters(pw, "loyalty"), loyalty - 3);
    assert_eq!(crate::r_s07_common::damage_on(&t, b1), 0);
}

#[test]
fn predatory_focus_applies_to_creatures_that_enter_later() {
    cr!("611.2c", "510.1b");
    ruling!(
        "Predatory Focus",
        "Will even apply to creatures that entered the battlefield after it resolved (which obviously must have Haste in order to attack)."
    );
    let mut t = TestGame::new(2);
    let b1 = t.battlefield(P1, "Grizzly Bears");
    predatory_focus(&mut t, true);
    // Raging Goblin (1/1 haste) enters after Predatory Focus resolved.
    let goblin = t.battlefield(P0, "Raging Goblin");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(goblin, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(b1, goblin)]);
    assert_eq!(t.life(P1), 19);
    assert_eq!(crate::r_s07_common::damage_on(&t, b1), 0);
}

// ---------------------------------------------------------------------------------------
// Murmuration

#[test]
fn murmuration_counts_spells_cast_before_it_and_countered_spells() {
    cr!("601.2i", "700.14");
    ruling!(
        "Murmuration",
        "Murmuration's last ability counts spells you cast earlier in the turn even if you didn't control Murmuration as you cast them and even if those spells were countered or otherwise didn't resolve."
    );
    supported("Murmuration");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // A spell that resolves, then one that's countered — before Murmuration is around.
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    let bears = cast_new(&mut t, P0, "Grizzly Bears", &[]);
    crate::r_s25_common::lands_for_cost(&mut t, P1, "Counterspell");
    let cs = t.hand(P1, "Counterspell");
    let _ = t.cast_with(P1, cs, &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.battlefield(P0, "Murmuration");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let crows: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|o| o.controller == P0 && o.chars.name == "Storm Crow")
        .map(|o| o.id)
        .collect();
    assert_eq!(crows.len(), 2);
    // Birds: 1/2 +1/+1 from Murmuration.
    assert_eq!(t.pt(crows[0]), (2, 3));
}

// ---------------------------------------------------------------------------------------
// Pre-War Formalwear

#[test]
fn pre_war_formalwear_stays_unattached_if_it_cant_be_attached() {
    cr!("701.3b", "702.16d");
    ruling!(
        "Pre-War Formalwear",
        "If Pre-War Formalwear can't be attached to the permanent returned to the battlefield by its first ability (perhaps because it's not currently a creature), Pre-War Formalwear will remain on the battlefield unattached."
    );
    supported("Pre-War Formalwear");
    // Tel-Jilad Chosen has protection from artifacts: the Equipment can't be attached to
    // it, so it stays on the battlefield unattached.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let chosen = t.graveyard(P0, "Tel-Jilad Chosen");
    t.answer_targets(P0, &[Entity::Object(chosen)]);
    let fw = cast_new(&mut t, P0, "Pre-War Formalwear", &[]);
    t.resolve_all();
    let fw = t.g.current(fw);
    assert_eq!(t.zone(fw), Zone::Battlefield);
    assert_eq!(t.obj(fw).attached_to, None);
    assert!(!t.named_on_battlefield("Tel-Jilad Chosen").is_empty());
    // Control: a creature it can be attached to gets it (+2/+2 and vigilance).
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    cast_new(&mut t, P0, "Pre-War Formalwear", &[]);
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.pt(bears), (4, 4));
    assert!(has_kw(&t, bears, KeywordKind::Vigilance));
}

// ---------------------------------------------------------------------------------------
// Undercity Upheaval

/// P0 has `n` creature cards in their graveyard and casts Undercity Upheaval at
/// `targets`, dividing the counters as `division`.
fn upheaval(t: &mut TestGame, n: usize, targets: &[ObjectId], division: Vec<i64>) -> ObjectId {
    supported("Undercity Upheaval");
    t.set_step(P0, Step::PrecombatMain);
    for _ in 0..n {
        t.graveyard(P0, "Grizzly Bears");
    }
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(division));
    let ents: Vec<Entity> = targets.iter().map(|x| Entity::Object(*x)).collect();
    crate::r_s25_common::lands_for_cost(t, P0, "Undercity Upheaval");
    let card = t.hand(P0, "Undercity Upheaval");
    t.answer_targets(P0, &ents);
    t.cast(P0, card).go()
}

#[test]
fn undercity_upheaval_all_targets_illegal_no_vigilance() {
    cr!("608.2b", "601.2d");
    ruling!(
        "Undercity Upheaval",
        "If all of Undercity Upheaval's targets are illegal at the time the spell tries to resolve, it won't resolve and none of its effects will happen. Creatures you control won't gain vigilance."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Hill Giant");
    let other = t.battlefield(P0, "Hill Giant");
    upheaval(&mut t, 2, &[a], vec![2]);
    crate::r_s02_common::destroy(&mut t, a);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Undercity Upheaval"));
    assert!(!has_kw(&t, other, KeywordKind::Vigilance));
}

#[test]
fn undercity_upheaval_counters_for_illegal_targets_are_lost() {
    cr!("608.2b", "601.2d");
    ruling!(
        "Undercity Upheaval",
        "If some of the creatures are illegal targets as Undercity Upheaval tries to resolve, the original distribution of counters still applies and the counters that would have been put on the illegal targets are lost. They won't be put instead on a legal target."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Hill Giant");
    let b = t.battlefield(P0, "Grizzly Bears");
    // X = 3 creature cards in the graveyard as it's cast: 2 on the Giant, 1 on the Bears.
    upheaval(&mut t, 3, &[a, b], vec![2, 1]);
    crate::r_s02_common::destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(t.counters(b, "+1/+1"), 1);
    assert!(has_kw(&t, b, KeywordKind::Vigilance));
}

#[test]
fn undercity_upheaval_x_is_counted_as_it_is_cast() {
    cr!("601.2d", "107.3c");
    // The Giant destroyed in response goes to the graveyard, but X was 1 as the spell
    // was cast: only one counter.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Hill Giant");
    let b = t.battlefield(P0, "Grizzly Bears");
    upheaval(&mut t, 1, &[b], vec![1]);
    crate::r_s02_common::destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(t.counters(b, "+1/+1"), 1);
}

// ---------------------------------------------------------------------------------------
// You Cannot Hide from Me

/// Runs the game to the cleanup step of the opposing team's turn (the team takes its
/// turns together, with P1 as the active player).
fn to_team_cleanup(t: &mut TestGame) {
    assert!(t
        .g
        .run_until(10_000, |g| g.turn.active == P1 && g.turn.step == Step::Cleanup));
}

/// Cards to draw for every player.
fn libraries(t: &mut TestGame) {
    for p in [P0, P1, P2] {
        for _ in 0..5 {
            t.library_top(p, "Forest");
        }
    }
}

#[test]
fn you_cannot_hide_from_me_uses_the_shared_life_total() {
    cr!("904.13b", "603.4", "119.1");
    ruling!(
        "You Cannot Hide from Me",
        "In a game with shared life totals (such as a game of Commander Archenemy), effects that reference a player's life total use the team's shared life total."
    );
    let c = card("You Cannot Hide from Me");
    assert!(c.is_fully_supported(), "{:?}", c.unsupported_text());
    // Commander Archenemy: P0 is the archenemy; P1 and P2 share a life total of 60.
    let mut t = TestGame::with_config(3, GameConfig::archenemy_commander(vec![0, 1, 1]));
    let def: CardDef = (*c).clone();
    let scheme = t.custom(P0, def, Zone::Command);
    t.g.recompute();
    libraries(&mut t);
    // P2 loses 31 life: the team (P1 too) has 29, less than half of 60.
    t.g.lose_life(P2, 31);
    assert_eq!(t.life(P1), 29);
    to_team_cleanup(&mut t);
    assert!(
        t.obj(t.g.current(scheme)).face_down,
        "the scheme was abandoned at P1's end step"
    );
}

#[test]
fn you_cannot_hide_from_me_stays_while_life_is_at_least_half() {
    cr!("603.4", "119.1");
    let c = card("You Cannot Hide from Me");
    let mut t = TestGame::with_config(3, GameConfig::archenemy_commander(vec![0, 1, 1]));
    let scheme = t.custom(P0, (*c).clone(), Zone::Command);
    t.g.recompute();
    libraries(&mut t);
    // 30 is exactly half: not less than half.
    t.g.lose_life(P2, 30);
    to_team_cleanup(&mut t);
    assert!(!t.obj(t.g.current(scheme)).face_down);
}

// ---------------------------------------------------------------------------------------
// Angel's Trumpet

/// Cards to draw for P0 and P1.
fn two_libraries(t: &mut TestGame) {
    for p in [P0, P1] {
        for _ in 0..5 {
            t.library_top(p, "Forest");
        }
    }
}

#[test]
fn angels_trumpet_taps_creatures_that_cant_attack() {
    cr!("603.2", "702.3b");
    ruling!(
        "Angel's Trumpet",
        "Angel's Trumpet affects creatures with Defender and other creatures that can't attack for some reason."
    );
    supported("Angel's Trumpet");
    let mut t = TestGame::new(2);
    two_libraries(&mut t);
    t.battlefield(P0, "Angel's Trumpet");
    // P1's Wall of Stone has defender: it can't attack. P1's Grizzly Bears didn't.
    let wall = t.battlefield(P1, "Wall of Stone");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.obj_now(wall).tapped);
    assert!(t.obj_now(bears).tapped);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn angels_trumpet_ignores_already_tapped_creatures() {
    cr!("603.2", "701.26a");
    ruling!(
        "Angel's Trumpet",
        "It does not affect creatures which did not attack, but which are already tapped at the time the ability resolves."
    );
    supported("Angel's Trumpet");
    let mut t = TestGame::new(2);
    two_libraries(&mut t);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Angel's Trumpet");
    // P0's Hill Giant attacks (vigilance from Angel's Trumpet keeps it untapped); P0's
    // Grizzly Bears didn't attack but is already tapped; P0's Llanowar Elves didn't
    // attack and is untapped.
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.g.tap(bears);
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert!(!t.obj_now(giant).tapped);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.obj_now(giant).tapped, "it attacked");
    assert!(t.obj_now(elves).tapped);
    // Only the Elves were tapped this way: 1 damage.
    assert_eq!(t.life(P0), 19);
}
