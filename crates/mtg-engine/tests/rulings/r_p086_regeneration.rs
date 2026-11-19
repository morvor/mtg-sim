//! Rulings batch P086 — hate for regeneration: "can't be regenerated" removal (Winds of
//! Rath, Dead Ringers, Disintegrate, Carbonize, Incinerate, Fatal Blow, Detonate,
//! Annihilate, Soul Rend, Putrefy, Slay, Fatal Lore), The Abyss and Magus of the Abyss,
//! and Giant Albatross.

use crate::r_p086_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Gives the creature a regeneration shield with the real spell Regenerate (cast by its
/// controller) and resolves it.
fn shield(t: &mut TestGame, id: ObjectId) {
    let p = controller(t, id);
    cast_new(t, p, "Regenerate", &[obj(id)]);
    t.resolve_all();
}

#[test]
fn winds_of_rath_spares_creatures_with_any_aura_attached() {
    cr!("303.4b", "701.8a");
    ruling!(
        "Winds of Rath",
        "A creature is \"enchanted\" if it has any Auras attached to it."
    );
    supported("Winds of Rath");
    let mut t = TestGame::new(2);
    let pacified = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Pacifism", obj(pacified));
    let mine = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Guildscorn Ward", obj(mine));
    let bare = t.battlefield(P1, "Raging Goblin");
    cast_new(&mut t, P0, "Winds of Rath", &[]);
    t.resolve_all();
    assert!(t.on_battlefield(pacified));
    assert!(t.on_battlefield(mine));
    assert!(!t.on_battlefield(bare));
}

#[test]
fn dead_ringers_checks_colors_on_resolution() {
    cr!("608.2b", "105.2");
    ruling!("Dead Ringers", "Colors are checked on resolution.");
    supported("Dead Ringers");
    for wisps in [false, true] {
        let mut t = TestGame::new(2);
        let goblin = t.battlefield(P1, "Raging Goblin");
        let drake = t.battlefield(P1, "Cloud Sprite");
        cast_new(&mut t, P0, "Dead Ringers", &[obj(goblin), obj(drake)]);
        if wisps {
            // Cerulean Wisps: "Target creature becomes blue until end of turn."
            cast_new(&mut t, P1, "Cerulean Wisps", &[obj(goblin)]);
        }
        t.resolve_all();
        assert_eq!(t.on_battlefield(goblin), !wisps, "wisps={wisps}");
        assert_eq!(t.on_battlefield(drake), !wisps, "wisps={wisps}");
    }
}

#[test]
fn disintegrated_creatures_never_go_to_the_graveyard() {
    cr!("614.6", "700.4");
    ruling!(
        "Disintegrate",
        "Disintegrated creatures do not go to the graveyard at all before being exiled. They do not trigger abilities which trigger due to a creature going to the graveyard."
    );
    supported("Disintegrate");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let traveler = t.battlefield(P1, "Doomed Traveler");
    t.lands(P0, "Mountain", 2);
    let card = t.hand(P0, "Disintegrate");
    t.cast(P0, card).x(1).target(traveler).go();
    t.resolve_all();
    assert!(t.in_exile("Doomed Traveler"));
    assert_eq!(t.graveyard_size(P1), 0);
    assert!(crate::r_s01_common::tokens(&t, P1).is_empty());
}

/// P0 casts `name` ("... deals N damage to any target. If it's a creature, it can't be
/// regenerated this turn, and if it would die this turn, exile it instead.") at `target`
/// and it resolves (lands for X are the caller's).
fn burn_and_exile(t: &mut TestGame, name: &str, target: ObjectId, x: Option<i64>) {
    lands_for_cost(t, P0, name);
    let card = t.hand(P0, name);
    let mut c = t.cast(P0, card).target(target);
    if let Some(x) = x {
        c = c.x(x);
    }
    c.go();
    t.resolve_all();
}

#[test]
fn disintegrates_exile_effect_doesnt_depend_on_the_damage() {
    cr!("614.6", "615.1a", "701.19c");
    ruling!(
        "Disintegrate",
        "The “can’t regenerate” is an effect of Disintegrate and not an effect of the damage. It works even if the damage is prevented or redirected. If redirected, the damage does not take this effect with it."
    );
    // The damage is prevented (Healing Salve's second mode).
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Carnassid");
    t.lands(P1, "Plains", 1);
    let salve = t.hand(P1, "Healing Salve");
    t.cast(P1, salve).modes(&[1]).target(wurm).go();
    t.resolve_all();
    t.lands(P0, "Wastes", 2);
    burn_and_exile(&mut t, "Disintegrate", wurm, Some(2));
    assert_eq!(damage_marked(&t, wurm), 0);
    shield(&mut t, wurm);
    destroy(&mut t, wurm);
    assert!(t.in_exile("Carnassid"));
    // The damage is redirected to Palisade Giant ("All damage that would be dealt to you
    // and other permanents you control is dealt to this creature instead."): the Giant
    // doesn't get the effect, the target still does.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Carnassid");
    let giant = t.battlefield(P1, "Palisade Giant");
    t.lands(P0, "Wastes", 2);
    burn_and_exile(&mut t, "Disintegrate", wurm, Some(2));
    assert_eq!(damage_marked(&t, giant), 2);
    destroy(&mut t, giant);
    assert!(t.in_graveyard(P1, "Palisade Giant"));
    destroy(&mut t, wurm);
    assert!(t.in_exile("Carnassid"));
}

#[test]
fn carbonize_exiles_the_creature_whatever_makes_it_die_this_turn() {
    cr!("614.6", "701.19c");
    ruling!(
        "Carbonize",
        "The targeted creature can’t be regenerated and is exiled if it would die no matter why it would die this turn. It doesn’t have to be destroyed because of the damage Carbonize deals to it."
    );
    supported("Carbonize");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Carnassid");
    burn_and_exile(&mut t, "Carbonize", wurm, None);
    assert!(t.on_battlefield(wurm));
    shield(&mut t, wurm);
    destroy(&mut t, wurm);
    assert!(t.in_exile("Carnassid"));
    assert!(!t.in_graveyard(P1, "Carnassid"));
}

#[test]
fn incinerate_regeneration_can_be_activated_but_does_nothing() {
    cr!("701.19c", "602.1");
    ruling!(
        "Incinerate",
        "Regeneration abilities that would affect that creature can still be activated; they just won’t do anything. The creature can’t be regenerated for any reason, even if an effect other than Incinerate would deal lethal damage to it or otherwise destroy it."
    );
    supported("Incinerate");
    supported("Carnassid");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Carnassid");
    cast_new(&mut t, P0, "Incinerate", &[obj(wurm)]);
    t.resolve_all();
    assert_eq!(damage_marked(&t, wurm), 3);
    t.lands(P1, "Forest", 2);
    assert!(crate::r_s02_common::can_activate(&mut t, P1, wurm));
    activate(&mut t, P1, wurm, "Regenerate", &[]);
    t.resolve_all();
    destroy(&mut t, wurm);
    assert!(t.in_graveyard(P1, "Carnassid"));
}

#[test]
fn fatal_blow_needs_a_creature_dealt_damage_this_turn() {
    cr!("115.1", "120.6");
    ruling!(
        "Fatal Blow",
        "Fatal Blow can’t target a creature that hasn’t been dealt damage this turn."
    );
    ruling!(
        "Fatal Blow",
        "Fatal Blow cares whether the creature had been dealt damage during the turn, not whether there is still damage on it. If the creature had regenerated, causing all damage to clear, Fatal Blow can still target and destroy it."
    );
    supported("Fatal Blow");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(!spell_targets(&mut t, P0, "Fatal Blow").contains(&obj(bears)));
    // Dealt lethal damage, then regenerated: the damage is removed (CR 701.19a).
    shield(&mut t, bears);
    cast_new(&mut t, P0, "Shock", &[obj(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(damage_marked(&t, bears), 0);
    shield(&mut t, bears);
    cast_new(&mut t, P0, "Fatal Blow", &[obj(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn spells_with_an_illegal_target_do_nothing_else() {
    cr!("608.2b");
    ruling!(
        "Detonate",
        "If the artifact becomes an illegal target before resolution, then no damage is done."
    );
    ruling!(
        "Annihilate",
        "If the targeted creature is an illegal target when Annihilate tries to resolve, the entire spell doesn’t resolve. You won’t draw a card."
    );
    supported("Detonate");
    supported("Annihilate");
    // Detonate: X=4 on Thran Dynamo, which is destroyed in response.
    let mut t = TestGame::new(2);
    let dynamo = t.battlefield(P1, "Thran Dynamo");
    t.lands(P0, "Mountain", 5);
    let card = t.hand(P0, "Detonate");
    t.cast(P0, card).x(4).target(dynamo).go();
    destroy(&mut t, dynamo);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // Without the response, the damage is dealt.
    let mut t = TestGame::new(2);
    let dynamo = t.battlefield(P1, "Thran Dynamo");
    t.lands(P0, "Mountain", 5);
    let card = t.hand(P0, "Detonate");
    t.cast(P0, card).x(4).target(dynamo).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Annihilate.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Annihilate", &[obj(bears)]);
    let hand = t.hand_size(P0);
    cast_new(&mut t, P1, "Unsummon", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn soul_rend_checks_the_color_only_on_resolution() {
    cr!("608.2c", "105.2");
    ruling!(
        "Soul Rend",
        "Only checks the color of the creature on resolution."
    );
    supported("Soul Rend");
    // A creature that isn't white can be targeted; it's destroyed only if it's white then.
    let mut t = TestGame::new(2);
    let lions = t.battlefield(P1, "Savannah Lions");
    let goblin = t.battlefield(P1, "Raging Goblin");
    assert!(spell_targets(&mut t, P0, "Soul Rend").contains(&obj(goblin)));
    cast_new(&mut t, P0, "Soul Rend", &[obj(lions)]);
    cast_new(&mut t, P1, "Cerulean Wisps", &[obj(lions)]);
    t.resolve_all();
    assert!(t.on_battlefield(lions));
    // Made white in response (Purelace: "Target spell or permanent becomes white."), the
    // goblin is destroyed.
    cast_new(&mut t, P0, "Soul Rend", &[obj(goblin)]);
    cast_new(&mut t, P0, "Purelace", &[obj(goblin)]);
    t.resolve_all();
    assert!(!t.on_battlefield(goblin));
}

#[test]
fn putrefy_destroys_an_artifact_that_became_a_creature() {
    cr!("608.2b", "115.1");
    ruling!(
        "Putrefy",
        "This spell isn't modal. When it resolves, it will destroy the target if it's a creature or an artifact, even if it changed from one to the other between targeting and resolution."
    );
    // Targeted as a noncreature artifact; March of the Machines ("Each noncreature
    // artifact is an artifact creature ...") enters and makes it an artifact creature
    // before Putrefy resolves.
    let mut t = TestGame::new(2);
    let dynamo = t.battlefield(P1, "Thran Dynamo");
    cast_new(&mut t, P0, "Putrefy", &[obj(dynamo)]);
    t.battlefield(P1, "March of the Machines");
    assert!(t.obj_now(dynamo).is(mtg_engine::types::CardType::Creature));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Thran Dynamo"));
}

#[test]
fn slay_countered_draws_no_card() {
    cr!("701.6a", "608.2");
    ruling!("Slay", "You don’t draw a card if this spell is countered.");
    supported("Slay");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P1, "Llanowar Elves");
    let slay = cast_new(&mut t, P0, "Slay", &[obj(elves)]);
    let hand = t.hand_size(P0);
    cast_new(&mut t, P1, "Counterspell", &[obj(slay)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.on_battlefield(elves));
}

#[test]
fn fatal_lore_opponent_chooses_the_mode_you_choose_the_targets() {
    cr!("700.2a", "601.2c");
    ruling!(
        "Fatal Lore",
        "The opponent chooses which option they want (this is a mode choice). If they choose the “destroy” option, you choose the zero, one, or two targets after that."
    );
    supported("Fatal Lore");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_targets(P0, &[obj(a)]);
    let from = t.asked().len();
    lands_for_cost(&mut t, P0, "Fatal Lore");
    let card = t.hand(P0, "Fatal Lore");
    t.cast(P0, card).go();
    let asked = t.asked()[from..].to_vec();
    let modes_by: Vec<PlayerId> = asked
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseModes { .. }))
        .map(|(p, _)| *p)
        .collect();
    let targets_by: Vec<PlayerId> = asked
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(modes_by, vec![P1]);
    assert_eq!(targets_by, vec![P0]);
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

// --- The Abyss and Magus of the Abyss ---------------------------------------------------------

#[test]
fn the_abyss_controller_controls_the_ability_the_upkeep_player_chooses() {
    cr!("603.3d", "115.1");
    ruling!(
        "The Abyss",
        "The controller of The Abyss controls the ability, but the current player chooses what gets destroyed."
    );
    ruling!(
        "Magus of the Abyss",
        "Even though the player whose upkeep it is chooses the target creature, you control the ability. An opponent can't target a creature with hexproof they control."
    );
    for name in ["The Abyss", "Magus of the Abyss"] {
        supported(name);
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        let scout = t.battlefield(P1, "Gladecover Scout");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let from = t.asked().len();
        next_upkeep(&mut t, P1);
        let top = crate::r_s04_common::top_of_stack(&t);
        assert_eq!(t.g.obj(top).controller, P0, "{name}");
        let offered = target_candidates(&t, P1, from);
        assert_eq!(offered.len(), 1, "{name}");
        assert!(offered[0].contains(&obj(bears)), "{name}");
        // The hexproof creature can't be targeted: P0 controls the ability.
        assert!(!offered[0].contains(&obj(scout)), "{name}");
        t.resolve_all();
        assert!(!t.on_battlefield(bears), "{name}");
        assert!(t.on_battlefield(scout), "{name}");
    }
}

#[test]
fn magus_of_the_abyss_can_target_an_indestructible_creature() {
    cr!("702.12b", "115.1");
    ruling!(
        "Magus of the Abyss",
        "The triggered ability can target a creature with indestructible. It won't be destroyed."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Magus of the Abyss");
    let avacyn = t.battlefield(P1, "Avacyn, Angel of Hope");
    let from = t.asked().len();
    next_upkeep(&mut t, P1);
    assert!(target_candidates(&t, P1, from)[0].contains(&obj(avacyn)));
    t.resolve_all();
    assert!(t.on_battlefield(avacyn));
}

// --- Giant Albatross ----------------------------------------------------------------------------

/// P0's Giant Albatross dies with its trigger on the stack once; P0 pays {1}{U} and P1
/// doesn't pay life. Returns the number of its triggers that went on the stack.
fn albatross_dies(
    t: &mut TestGame,
    albatross: ObjectId,
    kill: impl FnOnce(&mut TestGame),
) -> usize {
    t.lands(P0, "Island", 2);
    t.answer_yes(P0, true);
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(false));
    kill(t);
    t.settle();
    assert!(t.in_graveyard(P0, "Giant Albatross"));
    let _ = albatross;
    let n = crate::r_s01_common::triggers_on_stack(t, "pay {1}{U}");
    t.resolve_all();
    n
}

#[test]
fn giant_albatross_triggers_once_and_seeks_every_creature_that_damaged_it() {
    cr!("603.6c", "603.10a", "120.6");
    ruling!(
        "Giant Albatross",
        "The ability is a triggered ability and is only put on the stack once just after this card goes to the graveyard."
    );
    ruling!(
        "Giant Albatross",
        "The ability works no matter how it goes to the graveyard from the battlefield. It seeks out all creatures that damaged it during the turn and causes them to be destroyed."
    );
    supported("Giant Albatross");
    let mut t = TestGame::new(2);
    let albatross = t.battlefield(P0, "Giant Albatross");
    let a = t.battlefield(P1, "Prodigal Sorcerer");
    let b = t.battlefield(P1, "Prodigal Sorcerer");
    let c = t.battlefield(P1, "Prodigal Sorcerer");
    cast_new(&mut t, P0, "Giant Growth", &[obj(albatross)]);
    t.resolve_all();
    activate(&mut t, P1, a, "damage", &[obj(albatross)]);
    t.resolve_all();
    activate(&mut t, P1, b, "damage", &[obj(albatross)]);
    t.resolve_all();
    assert!(t.on_battlefield(albatross));
    // Destroyed by a spell, not by the damage.
    let n = albatross_dies(&mut t, albatross, |t| destroy(t, albatross));
    assert_eq!(n, 1);
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
    assert!(t.on_battlefield(c));
}

#[test]
fn giant_albatross_counts_damage_redirected_to_it() {
    cr!("120.6", "614.1a");
    ruling!(
        "Giant Albatross",
        "The ability works even if the damage is redirected to the Albatross."
    );
    // Pariah: "All damage that would be dealt to you is dealt to enchanted creature
    // instead."
    let mut t = TestGame::new(2);
    let albatross = t.battlefield(P0, "Giant Albatross");
    attach_new(&mut t, P0, "Pariah", obj(albatross));
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    t.lands(P0, "Island", 2);
    t.answer_yes(P0, true);
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(false));
    activate(&mut t, P1, sorcerer, "damage", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(t.in_graveyard(P0, "Giant Albatross"));
    assert!(!t.on_battlefield(sorcerer));
}
