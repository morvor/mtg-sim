//! Rulings batch S20 — attacking: attacking and blocked creatures (CR 506.4, 508.4,
//! 509.1h), untapping attackers, "attacks and isn't blocked", attacking alone, defending
//! players of creatures attacking planeswalkers, and triggers on becoming tapped.

use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, destroy};
use crate::r_s09_common::legal_attack;
use crate::r_s20_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

/// From the declare attackers step (attackers declared), the defending player `dp`
/// declares `blocks` and the game stops in the declare blockers step with the active
/// player holding priority (triggers on the stack).
fn declare_blocks(t: &mut TestGame, dp: PlayerId, blocks: &[(ObjectId, ObjectId)]) {
    let ap = t.g.turn.active;
    t.resolve_all();
    t.answer(dp, DecisionKind::Blockers, Answer::Blockers(blocks.to_vec()));
    let turn = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| {
        (g.turn.step == Step::DeclareBlockers
            && g.turn.stage == Stage::Priority
            && g.turn.priority == Some(ap))
            || g.turn.number != turn
    });
    assert!(ok && t.g.turn.number == turn, "blockers not declared");
    t.settle();
}

fn p1() -> Entity {
    Entity::Player(P1)
}

// ---------------------------------------------------------------------------
// Untapping an attacking creature
// ---------------------------------------------------------------------------

#[test]
fn an_attacker_untapped_by_taranika_is_still_attacking() {
    cr!("506.4b");
    ruling!(
        "Taranika, Akroan Veteran",
        "Untapping an attacking creature doesn't remove it from combat."
    );
    supported("Taranika, Akroan Veteran");
    let mut t = TestGame::new(2);
    let taranika = t.battlefield(P0, "Taranika, Akroan Veteran");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // "Whenever Taranika attacks, untap another target creature you control. Until end
    // of turn, that creature has base power and toughness 4/4 and gains indestructible."
    t.answer_targets(P0, &[Entity::Object(bears)]);
    attack_with(&mut t, &[(taranika, p1()), (bears, p1())]);
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
    assert!(t.g.is_attacking(bears));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 20 - 3 - 4);
}

#[test]
fn an_attacker_untapped_by_dauntless_aven_is_still_attacking() {
    cr!("506.4b");
    ruling!(
        "Dauntless Aven",
        "Untapping an attacking creature doesn’t remove it from combat."
    );
    supported("Dauntless Aven");
    let mut t = TestGame::new(2);
    let aven = t.battlefield(P0, "Dauntless Aven");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // "Whenever this creature attacks, untap target creature you control."
    t.answer_targets(P0, &[Entity::Object(bears)]);
    attack_with(&mut t, &[(aven, p1()), (bears, p1())]);
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
    assert!(t.g.is_attacking(bears));
    // Untapped, it can even block in the opponent's turn later; this combat it deals its
    // damage as an attacker.
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 20 - 2 - 2);
}

#[test]
fn an_attacker_untapped_by_aim_high_is_still_attacking() {
    cr!("506.4b");
    ruling!(
        "Aim High",
        "Untapping an attacking creature doesn’t cause it to be removed from combat."
    );
    supported("Aim High");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, p1())]);
    assert!(t.obj_now(bears).tapped);
    // "Untap target creature. It gets +2/+2 and gains reach until end of turn."
    t.lands(P0, "Forest", 2);
    let aim = t.hand(P0, "Aim High");
    t.cast(P0, aim).target(bears).go();
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
    assert!(t.g.is_attacking(bears));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 16);
}

// ---------------------------------------------------------------------------
// "Attacks and isn't blocked"
// ---------------------------------------------------------------------------

#[test]
fn attacks_and_isnt_blocked_triggers_after_blocks_even_for_a_creature_put_onto_the_battlefield_attacking(
) {
    cr!("509.1h", "508.4", "603.2");
    ruling!(
        "Eternal of Harsh Truths",
        "It will trigger even if that creature was put onto the battlefield attacking rather than having been declared as an attacker in the declare attackers step."
    );
    supported("Eternal of Harsh Truths");
    supported("Raph & Mikey, Troublemakers");
    let mut t = TestGame::new(2);
    // Raph & Mikey: "Whenever Raph & Mikey attack, reveal cards from the top of your
    // library until you reveal a creature card. Put that card onto the battlefield tapped
    // and attacking ..."
    stack_library(&mut t, P0, &["Eternal of Harsh Truths"]);
    let rm = t.battlefield(P0, "Raph & Mikey, Troublemakers");
    let declared = t.battlefield(P0, "Eternal of Harsh Truths");
    attack_with(&mut t, &[(rm, p1()), (declared, p1())]);
    t.resolve_all();
    let entered = controlled_named(&t, P0, "Eternal of Harsh Truths")
        .into_iter()
        .find(|e| *e != declared)
        .expect("an Eternal put onto the battlefield attacking");
    assert!(t.g.is_attacking(entered));
    // Nothing has triggered yet in the declare attackers step.
    assert_eq!(triggers_on_stack(&t, "isn't blocked"), 0);
    // P1 blocks Raph & Mikey only: both Eternals' abilities trigger in the declare
    // blockers step.
    let wall = t.battlefield(P1, "Wall of Stone");
    declare_blocks(&mut t, P1, &[(wall, rm)]);
    assert_eq!(t.g.turn.step, Step::DeclareBlockers);
    assert_eq!(triggers_on_stack(&t, "isn't blocked"), 2);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn a_creature_returned_attacking_doesnt_trigger_attack_abilities_or_check_attack_costs() {
    cr!("508.4", "508.4c", "508.3a");
    ruling!(
        "Yore-Tiller Nephilim",
        "Putting an attacking creature onto the battlefield doesn't trigger \"When this creature attacks\" abilities. It also won't check attacking restrictions, costs, or requirements."
    );
    supported("Yore-Tiller Nephilim");
    let mut t = TestGame::new(2);
    // "Whenever this creature attacks, return target creature card from your graveyard
    // to the battlefield tapped and attacking."
    let nephilim = t.battlefield(P0, "Yore-Tiller Nephilim");
    // Hollow Dogs: "Whenever this creature attacks, it gets +2/+0 until end of turn."
    let dogs = t.graveyard(P0, "Hollow Dogs");
    // Propaganda: attacking P1 costs {2} per creature; P0 has exactly {2}.
    t.battlefield(P1, "Propaganda");
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[Entity::Object(dogs)]);
    attack_with(&mut t, &[(nephilim, p1())]);
    t.resolve_all();
    let dogs = t.g.current(dogs);
    assert!(t.on_battlefield(dogs));
    assert!(t.obj_now(dogs).tapped);
    assert!(t.g.is_attacking(dogs));
    // The Dogs' attack trigger didn't trigger, and no cost was paid for them.
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.pt(dogs), (3, 3));
    assert_eq!(tapped_lands(&t, P0), 2);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 20 - 2 - 3);

    // A creature with defender, which can't attack, is returned attacking all the same.
    let mut t = TestGame::new(2);
    let nephilim = t.battlefield(P0, "Yore-Tiller Nephilim");
    let wall = t.graveyard(P0, "Wall of Stone");
    t.answer_targets(P0, &[Entity::Object(wall)]);
    attack_with(&mut t, &[(nephilim, p1())]);
    t.resolve_all();
    assert!(t.g.is_attacking(t.g.current(wall)));
}

// ---------------------------------------------------------------------------
// Attacking and blocked creatures (CR 506.4, 509.1h)
// ---------------------------------------------------------------------------

/// P0 attacks P1's Jace Beleren with Grizzly Bears and P1 with Raph & Mikey (which puts
/// a Hill Giant onto the battlefield attacking); in the declare blockers step, after no
/// blocks, Jace is destroyed. The game stops in the end of combat step. Returns (Bears,
/// Raph & Mikey, Hill Giant).
fn attack_a_planeswalker_that_leaves(t: &mut TestGame) -> (ObjectId, ObjectId, ObjectId) {
    stack_library(t, P0, &["Hill Giant"]);
    let jace = t.battlefield(P1, "Jace Beleren");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rm = t.battlefield(P0, "Raph & Mikey, Troublemakers");
    attack_with(t, &[(bears, Entity::Object(jace)), (rm, p1())]);
    t.resolve_all();
    let giant = t.named_on_battlefield("Hill Giant")[0];
    declare_blocks(t, P1, &[]);
    destroy(t, jace);
    assert!(!t.on_battlefield(jace));
    t.advance_to(P0, Step::EndOfCombat);
    (bears, rm, giant)
}

#[test]
fn a_creature_attacking_a_planeswalker_that_left_is_still_an_attacking_creature() {
    cr!("506.4c", "508.4");
    ruling!(
        "Divine Verdict",
        "An “attacking creature” is one that has been declared as an attacker this combat, or one that was put onto the battlefield attacking this combat. Unless that creature leaves combat, it continues to be an attacking creature through the end of combat step, even if the player it was attacking has left the game, or the planeswalker it was attacking has left combat."
    );
    supported("Divine Verdict");
    supported("Flurry of Wings");
    let mut t = TestGame::new(2);
    let (bears, _rm, giant) = attack_a_planeswalker_that_leaves(&mut t);
    assert_eq!(t.g.turn.step, Step::EndOfCombat);
    // Flurry of Wings: "Create X 1/1 white Bird Soldier creature tokens with flying,
    // where X is the number of attacking creatures." All three count, including the
    // Giant put onto the battlefield attacking and the Bears whose planeswalker is gone.
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    let flurry = t.hand(P0, "Flurry of Wings");
    t.cast(P0, flurry).go();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Bird").len(), 3);
    // Divine Verdict ("Destroy target attacking or blocking creature") can target either.
    t.lands(P1, "Plains", 8);
    for victim in [bears, giant] {
        let verdict = t.hand(P1, "Divine Verdict");
        t.cast(P1, verdict).target(victim).go();
        t.resolve_all();
        assert!(!t.on_battlefield(victim));
    }
}

#[test]
fn trumpet_blast_pumps_creatures_attacking_a_planeswalker_that_left_combat() {
    cr!("506.4c", "508.4");
    ruling!(
        "Trumpet Blast",
        "An \"attacking creature\" is one that has been declared as an attacker this combat, or one that was put onto the battlefield attacking this combat. Unless that creature leaves combat, it continues to be an attacking creature through the end of combat step"
    );
    supported("Trumpet Blast");
    let mut t = TestGame::new(2);
    let (bears, rm, giant) = attack_a_planeswalker_that_leaves(&mut t);
    let before: Vec<i32> = [bears, rm, giant].iter().map(|c| t.pt(*c).0).collect();
    // "Attacking creatures get +2/+0 until end of turn."
    t.lands(P0, "Mountain", 3);
    let blast = t.hand(P0, "Trumpet Blast");
    t.cast(P0, blast).go();
    t.resolve_all();
    let after: Vec<i32> = [bears, rm, giant].iter().map(|c| t.pt(*c).0).collect();
    assert_eq!(after, before.iter().map(|p| p + 2).collect::<Vec<_>>());
}

#[test]
fn a_creature_whose_blocker_left_combat_is_still_a_blocked_creature() {
    cr!("509.1h", "506.4");
    ruling!(
        "Smite",
        "A “blocked creature” is an attacking creature that has been blocked by a creature this combat, or has become blocked as the result of a spell or ability this combat. Unless the attacking creature leaves combat, it continues to be a blocked creature through the end of combat step, even if the creature or creatures that blocked it are no longer on the battlefield or have otherwise left combat by then."
    );
    supported("Smite");
    supported("Fight to the Death");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let wall = t.battlefield(P1, "Wall of Stone");
    attack_with(&mut t, &[(bears, p1()), (giant, p1())]);
    declare_blocks(&mut t, P1, &[(elves, bears), (wall, giant)]);
    // The Elves leave the battlefield: the Bears remain blocked (they deal no damage).
    destroy(&mut t, elves);
    assert!(t.g.combat.as_ref().unwrap().is_blocked(bears));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    // Smite ("Destroy target blocked creature") can target the Bears.
    t.lands(P1, "Plains", 1);
    let smite = t.hand(P1, "Smite");
    t.cast(P1, smite).target(bears).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    // Fight to the Death destroys the blocked Giant and its blocker.
    t.lands(P1, "Mountain", 1);
    t.lands(P1, "Plains", 1);
    let fight = t.hand(P1, "Fight to the Death");
    t.cast(P1, fight).go();
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert!(!t.on_battlefield(wall));
}

// ---------------------------------------------------------------------------
// Defending players (CR 506.2, 508.1b)
// ---------------------------------------------------------------------------

#[test]
fn the_controller_of_an_attacked_planeswalker_is_the_defending_player_for_lurking_green_dragon() {
    cr!("506.2", "508.1c", "508.5");
    ruling!(
        "Lurking Green Dragon",
        "If a creature attacks a planeswalker, that planeswalker's controller is the defending player."
    );
    supported("Lurking Green Dragon");
    let mut t = TestGame::new(3);
    // "This creature can't attack unless defending player controls a creature with
    // flying." P1 controls a flier; P2 doesn't.
    let dragon = t.battlefield(P0, "Lurking Green Dragon");
    t.battlefield(P1, "Air Elemental");
    let jace1 = t.battlefield(P1, "Jace Beleren");
    let jace2 = t.battlefield(P2, "Jace Beleren");
    to_beginning_of_combat(&mut t, P0);
    assert!(legal_attack(&mut t, &[(dragon, Entity::Object(jace1))]));
    assert!(legal_attack(&mut t, &[(dragon, p1())]));
    assert!(!legal_attack(&mut t, &[(dragon, Entity::Object(jace2))]));
    assert!(!legal_attack(&mut t, &[(dragon, Entity::Player(P2))]));
}

#[test]
fn storm_the_citadel_destroys_a_permanent_of_the_attacked_planeswalkers_controller() {
    cr!("508.5", "508.5a");
    ruling!(
        "Storm the Citadel",
        "If a creature is attacking a planeswalker, the controller of the planeswalker is the defending player."
    );
    supported("Storm the Citadel");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let jace = t.battlefield(P2, "Jace Beleren");
    let anthem1 = t.battlefield(P1, "Glorious Anthem");
    let anthem2 = t.battlefield(P2, "Glorious Anthem");
    // "Until end of turn, creatures you control get +2/+2 and gain 'Whenever this
    // creature deals combat damage to a player or planeswalker, destroy target artifact
    // or enchantment defending player controls.'"
    t.lands(P0, "Forest", 5);
    let storm = t.hand(P0, "Storm the Citadel");
    t.cast(P0, storm).go();
    t.resolve_all();
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(anthem2)]);
    attack_with(&mut t, &[(bears, Entity::Object(jace))]);
    block_and_finish(&mut t, P2, &[]);
    t.resolve_all();
    // Only P2's enchantment could be targeted.
    let candidates = crate::r_s02_common::target_candidates(&t, P0, from);
    assert_eq!(candidates, vec![vec![Entity::Object(anthem2)]]);
    assert!(!t.on_battlefield(anthem2));
    assert!(t.on_battlefield(anthem1));
}

// ---------------------------------------------------------------------------
// Evasion is checked as blockers are declared (CR 509.1b)
// ---------------------------------------------------------------------------

#[test]
fn intimidate_matters_only_as_blockers_are_declared() {
    cr!("702.13b", "509.1h");
    ruling!(
        "Hideous Visage",
        "If an attacking creature has intimidate, what colors it is matters only as the defending player declares blockers. Once it’s blocked, changing its colors won’t change that."
    );
    supported("Hideous Visage");
    supported("Cerulean Wisps");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let merfolk = t.battlefield(P1, "Coral Merfolk");
    // "Creatures you control gain intimidate until end of turn."
    t.lands(P0, "Swamp", 3);
    let visage = t.hand(P0, "Hideous Visage");
    t.cast(P0, visage).go();
    t.resolve_all();
    attack_with(&mut t, &[(bears, p1())]);
    // The green Bears can be blocked by the green Elves but not the blue Merfolk.
    t.g.recompute();
    assert!(t.g.can_block(elves, bears));
    assert!(!t.g.can_block(merfolk, bears));
    declare_blocks(&mut t, P1, &[(elves, bears)]);
    assert!(t.g.combat.as_ref().unwrap().is_blocked(bears));
    // The Bears become blue: they no longer share a color with the Elves, but they stay
    // blocked, and the Elves still block them.
    t.lands(P0, "Island", 1);
    let wisps = t.hand(P0, "Cerulean Wisps");
    t.cast(P0, wisps).target(bears).go();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).chars.colors, ColorSet::single(Color::Blue));
    assert!(t.g.combat.as_ref().unwrap().is_blocked(bears));
    assert_eq!(
        t.g.combat.as_ref().unwrap().blockers_of(bears),
        vec![elves]
    );
    t.advance_to(P0, Step::EndOfCombat);
    assert!(!t.on_battlefield(elves));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn a_creature_that_must_block_blocks_the_attacker_its_controller_chooses() {
    cr!("509.1a", "509.1c");
    ruling!(
        "Culling Mark",
        "The controller of the creature chooses which attacking creature that creature blocks."
    );
    supported("Culling Mark");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let spider = t.battlefield(P1, "Giant Spider");
    // "Target creature blocks this turn if able."
    t.lands(P0, "Forest", 3);
    let mark = t.hand(P0, "Culling Mark");
    t.cast(P0, mark).target(spider).go();
    t.resolve_all();
    attack_with(&mut t, &[(bears, p1()), (giant, p1())]);
    let options = mtg_engine::combat::block_options(&t.g, &[P1]);
    // Not blocking is illegal; blocking either attacker is legal.
    assert!(!mtg_engine::combat::block_declaration_legal(
        &t.g,
        &options,
        &[]
    ));
    for attacker in [bears, giant] {
        assert!(mtg_engine::combat::block_declaration_legal(
            &t.g,
            &options,
            &[(spider, attacker)]
        ));
    }
    // P1 chooses the Giant.
    declare_blocks(&mut t, P1, &[(spider, giant)]);
    assert_eq!(t.g.combat.as_ref().unwrap().blocking(spider), vec![giant]);
}

// ---------------------------------------------------------------------------
// Attack triggers and combat phases
// ---------------------------------------------------------------------------

#[test]
fn a_creature_that_attacks_twice_in_a_turn_gets_its_attack_bonus_each_time() {
    cr!("508.3a", "500.8");
    ruling!(
        "Hollow Dogs",
        "If it attacks more than once per turn, it gets the bonus each time."
    );
    supported("Hollow Dogs");
    supported("Relentless Assault");
    let mut t = TestGame::new(2);
    // "Whenever this creature attacks, it gets +2/+0 until end of turn."
    let dogs = t.battlefield(P0, "Hollow Dogs");
    attack_with(&mut t, &[(dogs, p1())]);
    t.resolve_all();
    assert_eq!(t.pt(dogs), (5, 3));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 15);
    // Relentless Assault: untap it; an additional combat phase follows this main phase.
    t.advance_to(P0, Step::PostcombatMain);
    t.lands(P0, "Mountain", 4);
    let assault = t.hand(P0, "Relentless Assault");
    t.cast(P0, assault).go();
    t.resolve_all();
    assert!(!t.obj_now(dogs).tapped);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(dogs, p1())]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(t.pt(dogs), (7, 3));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 8);
}

#[test]
fn before_attackers_are_declared_means_before_the_first_combats_declare_attackers_step() {
    cr!("506.8a", "506.8d", "506.8g");
    ruling!(
        "Apprentice Sorcerer",
        "If a turn has multiple combat phases, the ability can only be activated before the beginning of the declare attackers step of the first combat phase in that turn."
    );
    supported("Apprentice Sorcerer");
    let mut t = TestGame::new(2);
    // "{T}: This creature deals 1 damage to any target. Activate only during your turn,
    // before attackers are declared."
    let sorcerer = t.battlefield(P0, "Apprentice Sorcerer");
    assert!(can_activate(&mut t, P0, sorcerer));
    // Relentless Assault in the first main phase: an additional combat phase and main
    // phase follow it.
    t.lands(P0, "Mountain", 4);
    let assault = t.hand(P0, "Relentless Assault");
    t.cast(P0, assault).go();
    t.resolve_all();
    assert!(can_activate(&mut t, P0, sorcerer));
    // After the first combat, in the additional main phase, another combat phase is still
    // to come, but attackers were declared in this turn's first combat.
    t.advance_to(P0, Step::DeclareAttackers);
    assert!(!can_activate(&mut t, P0, sorcerer));
    let turn = t.g.turn.number;
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.g.turn.number, turn);
    assert!(t
        .g
        .turn
        .schedule
        .iter()
        .any(|s| *s == Step::BeginningOfCombat));
    assert!(!can_activate(&mut t, P0, sorcerer));
    // It can't be activated in the second combat's beginning of combat step either.
    t.advance_to(P0, Step::BeginningOfCombat);
    assert_eq!(t.g.turn.number, turn);
    assert!(!can_activate(&mut t, P0, sorcerer));
}

#[test]
fn exalted_doesnt_trigger_when_several_creatures_attack() {
    cr!("506.5", "702.83a", "702.83b");
    ruling!(
        "Goblin Champion",
        "A creature attacks alone if it’s the only creature declared as an attacker during the declare attackers step"
    );
    supported("Goblin Champion");
    let mut t = TestGame::new(2);
    let champion = t.battlefield(P0, "Goblin Champion");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(champion, p1()), (bears, p1())]);
    assert_eq!(t.stack_len(), 0);
    // The Bears are removed from combat: the Champion is attacking alone now, but it
    // didn't attack alone, and exalted doesn't trigger.
    mtg_engine::combat::remove_from_combat(&mut t.g, bears);
    t.settle();
    assert!(!t.g.is_attacking(bears));
    assert_eq!(t.stack_len(), 0);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 20);
    // Attacking alone, it gets +1/+1.
    let mut t = TestGame::new(2);
    let champion = t.battlefield(P0, "Goblin Champion");
    attack_with(&mut t, &[(champion, p1())]);
    t.resolve_all();
    assert_eq!(t.pt(champion), (1, 2));
}

// ---------------------------------------------------------------------------
// "Whenever this creature becomes tapped" (CR 603.2e)
// ---------------------------------------------------------------------------

#[test]
fn fallowsage_needs_another_way_to_become_tapped_such_as_attacking() {
    cr!("603.2e", "508.1f");
    ruling!(
        "Fallowsage",
        "This is a triggered ability, not an activated ability. It doesn't allow you to tap the creature whenever you want; rather, you need some other way of tapping it, such as by attacking with the creature."
    );
    supported("Fallowsage");
    let mut t = TestGame::new(2);
    // "Whenever this creature becomes tapped, you may draw a card."
    let sage = t.battlefield(P0, "Fallowsage");
    // There's nothing to activate.
    assert!(!can_activate(&mut t, P0, sage));
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(sage, p1())]);
    t.resolve_all();
    assert!(t.obj_now(sage).tapped);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn veteran_of_the_depths_needs_another_way_to_become_tapped_such_as_attacking() {
    cr!("603.2e", "508.1f");
    ruling!(
        "Veteran of the Depths",
        "This is a triggered ability, not an activated ability. It doesn’t allow you to tap the creature whenever you want; rather, you need some other way of tapping it, such as by attacking with the creature."
    );
    supported("Veteran of the Depths");
    let mut t = TestGame::new(2);
    // "Whenever this creature becomes tapped, you may put a +1/+1 counter on it."
    let vet = t.battlefield(P0, "Veteran of the Depths");
    assert!(!can_activate(&mut t, P0, vet));
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(vet, p1())]);
    t.resolve_all();
    assert_eq!(t.counters(vet, "+1/+1"), 1);
}
