//! Rulings batch P049 — the other "drain" cards: counters removed by choice (Heartless
//! Act), lands with the Swamp subtype (Dread Presence, Corrupt), noncreature spells
//! (Kambal), cost reductions (Accursed Witch; Tezzeret, Master of the Bridge's affinity),
//! a life total that becomes 10 (Sorin Markov), casting from a graveyard (Liliana),
//! protection from permanents with corruption counters (Geyadrone Dihada), layers 7b–7d
//! (Tezzeret, Agent of Bolas), "repeat this process" (Zimone and Dina), control of
//! creatures and their Auras (The Trickster-God's Heist), X in a graveyard, Foods that
//! aren't tokens, Chainer's Torment doubled, and Avatar Aang's bending triggers.

use crate::r_p049_common::*;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::decision::{Action, Answer};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether `p` could cast `card` by any method now.
fn can_cast_any(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let card = t.g.current(card);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Cast { card: c, .. } if *c == card))
}

#[test]
fn heartless_act_removes_counters_of_any_kinds_from_anyones_creature() {
    cr!("700.2", "122.1", "608.2d");
    ruling!(
        "Heartless Act",
        "If you choose the second mode for Heartless Act, you choose which counters to remove from the creature, no matter who controls the creature. You may choose counters of different kinds."
    );
    supported("Heartless Act");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    put_counters(&mut t, bears, counters::PLUS1, 2);
    put_counters(&mut t, bears, "lifelink", 2);
    // P0 removes three: two of one kind and one of the other.
    t.answer(P0, DecisionKind::Number, Answer::Number(3));
    for i in [0usize, 1, 0] {
        t.answer(P0, DecisionKind::Option, Answer::Index(i));
    }
    lands_for_cost(&mut t, P0, "Heartless Act");
    let act = t.hand(P0, "Heartless Act");
    t.cast(P0, act).modes(&[1]).target(bears).go();
    t.resolve_all();
    let (plus, life) = (
        t.counters(bears, counters::PLUS1),
        t.counters(bears, "lifelink"),
    );
    assert_eq!(plus + life, 1, "{plus} +1/+1 and {life} lifelink");
    assert!(plus <= 1 && life <= 1);
    assert!(t.on_battlefield(bears));
}

#[test]
fn a_swamp_is_a_land_with_the_subtype_not_any_black_mana_land() {
    cr!("205.3i", "305.6");
    ruling!(
        "Dread Presence",
        "A Swamp is a land with the subtype Swamp, not any land with a mana ability that produces black mana."
    );
    supported("Dread Presence");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dread Presence");
    // Vault of Whispers makes {B} but isn't a Swamp; Watery Grave is an Island Swamp.
    t.enter(P0, "Vault of Whispers");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "choose one"), 0);
    t.enter(P0, "Watery Grave");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "choose one"), 1);
    ruling!(
        "Corrupt",
        "Corrupt counts any land you control with the subtype Swamp, not just ones named Swamp."
    );
    supported("Corrupt");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Swamp");
    t.battlefield(P0, "Watery Grave");
    t.battlefield(P0, "Vault of Whispers");
    // (The lands for Corrupt's cost add a Swamp: three Swamps, not the Vault.)
    cast_new(&mut t, P0, "Corrupt", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (23, 17));
}

#[test]
fn kambal_triggers_on_noncreature_spells_only() {
    cr!("112.1", "305.9", "300.1"); // CR 305.9: lands aren't spells.
    ruling!(
        "Kambal, Consul of Allocation",
        "A noncreature spell is a spell that isn't a creature at all. An artifact creature spell isn't a noncreature spell. Lands are never spells."
    );
    supported("Kambal, Consul of Allocation");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kambal, Consul of Allocation");
    t.set_step(P1, Step::PrecombatMain);
    // An artifact creature spell: no trigger.
    cast_new(&mut t, P1, "Ornithopter", &[]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    // A noncreature artifact spell: P1 loses 2.
    cast_new(&mut t, P1, "Sol Ring", &[]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
    // Playing a land: no trigger.
    let land = t.hand(P1, "Swamp");
    t.play_land(P1, land).unwrap();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
}

#[test]
fn accursed_witch_reduces_opponents_spells_that_target_it_but_not_abilities() {
    cr!("601.2f", "113.3"); // Spells, including Auras and spells with other targets.
    ruling!(
        "Accursed Witch // Infectious Curse",
        "Accursed Witch's first ability affects all spells cast by your opponents that target it, including Aura spells and spells that have additional targets. It doesn't affect abilities."
    );
    supported("Accursed Witch // Infectious Curse");
    let case = |name: &str, extra: &[Entity]| -> usize {
        supported(name);
        let mut t = TestGame::new(2);
        let witch = t.battlefield(P0, "Accursed Witch // Infectious Curse");
        t.set_step(P1, Step::PrecombatMain);
        let spell = t.hand(P1, name);
        let mut targets = vec![obj(witch)];
        targets.extend_from_slice(extra);
        cost_to_cast(&mut t, P1, spell, &targets).unwrap_or_else(|| panic!("{name}"))
    };
    // Lightning Blast ({3}{R}): {2}{R}. Pacifism ({1}{W}), an Aura: {W}. Arc Trail
    // ({1}{R}), with another target: {R}.
    assert_eq!(case("Lightning Blast", &[]), 3);
    assert_eq!(case("Pacifism", &[]), 1);
    assert_eq!(case("Arc Trail", &[Entity::Player(P0)]), 1);
    // Rod of Ruin's ability ({3}, {T}: 1 damage to any target) still costs {3}.
    supported("Rod of Ruin");
    let mut t = TestGame::new(2);
    let witch = t.battlefield(P0, "Accursed Witch // Infectious Curse");
    t.set_step(P1, Step::PrecombatMain);
    let rod = t.battlefield(P1, "Rod of Ruin");
    fill_pool(&mut t, P1, 10);
    let before = pool(&t, P1);
    t.activate(P1, rod, 0, &[obj(witch)]).unwrap();
    assert_eq!(before - pool(&t, P1), 3);
}

#[test]
fn tezzerets_affinity_reduces_generic_mana_only_and_not_the_mana_value() {
    cr!("702.41a", "702.41b", "202.3", "601.2f");
    ruling!(
        "Tezzeret, Master of the Bridge",
        "Affinity can reduce only generic mana in a spell's cost. It can't reduce color requirements."
    );
    ruling!(
        "Tezzeret, Master of the Bridge",
        "Affinity doesn't change the spell's mana value. It changes only how much mana you pay to cast the spell."
    );
    ruling!(
        "Tezzeret, Master of the Bridge",
        "Affinity for artifacts means \"This spell costs {1} less for each artifact you control.\""
    );
    supported("Tezzeret, Master of the Bridge");
    let setup = || {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Tezzeret, Master of the Bridge");
        t.battlefield(P0, "Sol Ring");
        t.battlefield(P0, "Sol Ring");
        t
    };
    // Grizzly Bears ({1}{G}) with two artifacts: {G}; its mana value is still 2.
    let mut t = setup();
    let bears = t.hand(P0, "Grizzly Bears");
    assert_eq!(cost_to_cast(&mut t, P0, bears, &[]), Some(1));
    let spell = stack(&t)[0];
    assert_eq!(t.g.mana_value_of(spell), 2);
    // Hill Giant ({3}{R}): {1}{R}.
    let mut t = setup();
    let giant = t.hand(P0, "Hill Giant");
    assert_eq!(cost_to_cast(&mut t, P0, giant, &[]), Some(2));
    ruling!(
        "Tezzeret, Master of the Bridge",
        "If a spell has multiple instances of affinity, each applies."
    );
    // Frogmite ({4}, affinity for artifacts) gets a second instance from Tezzeret: free.
    supported("Frogmite");
    let mut t = setup();
    let frog = t.hand(P0, "Frogmite");
    assert_eq!(cost_to_cast(&mut t, P0, frog, &[]), Some(0));
}

#[test]
fn tezzerets_5_5_artifact_keeps_earlier_and_later_power_toughness_changes() {
    cr!("613.4b", "613.4c", "613.4d", "613.1");
    ruling!(
        "Tezzeret, Agent of Bolas",
        "Effects that modify that creature's power or toughness, such as the effects of Giant Growth, will apply to it no matter when they started to take effect. The same is true for counters that change the creature's power or toughness (such as -1/-1 counters) and effects that switch its power and toughness."
    );
    supported("Tezzeret, Agent of Bolas");
    let mut t = TestGame::new(2);
    let tez = t.battlefield(P0, "Tezzeret, Agent of Bolas");
    let ring = t.battlefield(P0, "Sol Ring");
    // A -1/-1 counter put on it before it's a creature.
    put_counters(&mut t, ring, counters::MINUS1, 1);
    t.answer_targets(P0, &[obj(ring)]);
    activate_containing(&mut t, P0, tez, "5/5").unwrap();
    t.resolve_all();
    assert!(t.obj_now(ring).is(CardType::Creature));
    assert_eq!(t.pt(ring), (4, 4));
    // A later +3/+0, then a switch.
    modify_until_eot(
        &mut t,
        ring,
        vec![Modification::ModifyPT(Value::c(3), Value::c(0))],
    );
    assert_eq!(t.pt(ring), (7, 4));
    modify_until_eot(&mut t, ring, vec![Modification::SwitchPT]);
    assert_eq!(t.pt(ring), (4, 7));
}

#[test]
fn sorin_markovs_life_total_of_10_is_a_gain_or_loss_of_life() {
    cr!("119.5", "119.3");
    ruling!(
        "Sorin Markov",
        "For a player’s life total to become 10, what actually happens is that the player gains or loses the appropriate amount of life."
    );
    supported("Sorin Markov");
    supported("Ajani's Pridemate");
    // At 17: P1 loses 7.
    let mut t = TestGame::new(2);
    let sorin = t.battlefield(P0, "Sorin Markov");
    set_life(&mut t, P1, 17);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, sorin, "becomes 10").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 10);
    assert_eq!(life_lost(&t, P1), 7);
    // At 4: P1 gains 6, and P1's Ajani's Pridemate sees the life gain.
    let mut t = TestGame::new(2);
    let sorin = t.battlefield(P0, "Sorin Markov");
    let pridemate = t.battlefield(P1, "Ajani's Pridemate");
    set_life(&mut t, P1, 4);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, sorin, "becomes 10").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 10);
    assert_eq!(t.g.history.life_gained.get(&P1).copied(), Some(6));
    assert_eq!(t.counters(pridemate, counters::PLUS1), 1);
}

#[test]
fn casting_a_zombie_from_the_graveyard_with_liliana_follows_the_normal_rules() {
    cr!("601.2", "307.1", "601.3"); // Costs and timing as normal.
    ruling!(
        "Liliana, Untouched by Death",
        "Casting a Zombie card from your graveyard once Liliana’s third ability has resolved follows the normal rules for casting that card. You must pay its costs, and you must follow all applicable timing rules."
    );
    supported("Liliana, Untouched by Death");
    let mut t = TestGame::new(2);
    let liliana = t.battlefield(P0, "Liliana, Untouched by Death");
    let corpse = t.graveyard(P0, "Walking Corpse");
    assert!(!can_cast_any(&mut t, P0, corpse));
    activate_containing(&mut t, P0, liliana, "cast Zombie spells").unwrap();
    t.resolve_all();
    // Without mana for {1}{B}, it can't be cast.
    assert!(!can_cast_any(&mut t, P0, corpse));
    t.lands(P0, "Swamp", 2);
    assert!(can_cast_any(&mut t, P0, corpse));
    // Not with a spell on the stack (a creature spell at sorcery timing)...
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    assert!(!can_cast_any(&mut t, P0, corpse));
    t.resolve_all();
    let _ = bolt;
    // ...and it costs its mana cost.
    t.g.turn.priority = Some(P0);
    let c = t.g.current(corpse);
    t.g.cast_spell(P0, c, CastMethod::Normal).unwrap();
    t.g.flush_events();
    assert_eq!(
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.is(CardType::Land) && o.tapped)
            .count(),
        2
    );
    t.resolve_all();
    assert!(!t.named_on_battlefield("Walking Corpse").is_empty());
}

#[test]
fn geyadrone_dihadas_protection_covers_targeting_and_damage_from_corrupted_permanents() {
    cr!("702.16b", "702.16e", "306.5");
    ruling!(
        "Geyadrone Dihada",
        "Geyadrone Dihada can't be targeted, dealt damage, or enchanted by any permanent with a corruption counter on it."
    );
    ruling!(
        "Geyadrone Dihada",
        "Creatures with corruption counters can still attack Dihada, but the damage they would deal will be prevented."
    );
    supported("Geyadrone Dihada");
    let mut t = TestGame::new(2);
    let dihada = t.battlefield(P0, "Geyadrone Dihada");
    assert_eq!(t.counters(dihada, counters::LOYALTY), 4);
    t.set_step(P1, Step::PrecombatMain);
    let corrupted = t.battlefield(P1, "Prodigal Sorcerer");
    put_counters(&mut t, corrupted, "corruption", 1);
    let clean = t.battlefield(P1, "Prodigal Sorcerer");
    // The corrupted Sorcerer's ability can't target Dihada; the other's can.
    let from = t.asked().len();
    let _ = t.activate(P1, corrupted, 0, &[obj(dihada)]);
    t.clear_answers();
    let offered = target_candidates(&t, P1, from);
    assert!(offered.iter().all(|c| !c.contains(&obj(dihada))));
    t.resolve_all();
    assert_eq!(t.counters(dihada, counters::LOYALTY), 4);
    let from = t.asked().len();
    t.activate(P1, clean, 0, &[obj(dihada)]).unwrap();
    let offered = target_candidates(&t, P1, from);
    assert!(offered[0].contains(&obj(dihada)));
    t.resolve_all();
    assert_eq!(t.counters(dihada, counters::LOYALTY), 3);
    // A corrupted creature attacks Dihada: its damage is prevented.
    let bears = t.battlefield(P1, "Grizzly Bears");
    put_counters(&mut t, bears, "corruption", 1);
    t.attack(&[(bears, obj(dihada))], &[]);
    assert_eq!(t.counters(dihada, counters::LOYALTY), 3);
}

#[test]
fn zimone_and_dina_repeat_the_effect_but_not_the_cost() {
    cr!("608.2c", "602.2b"); // The cost is paid once; the effect is repeated.
    ruling!(
        "Zimone and Dina",
        "For the activated ability, repeating the process means the entire effect, but not the cost. You don’t have to (and, in fact, can’t) sacrifice another creature."
    );
    supported("Zimone and Dina");
    let mut t = TestGame::new(2);
    let zd = t.battlefield(P0, "Zimone and Dina");
    t.lands(P0, "Forest", 8);
    let fodder = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    let s1 = t.hand(P0, "Swamp");
    let s2 = t.hand(P0, "Swamp");
    t.answer_choose(P0, &[obj(fodder)]);
    // Twice: draw, then put a land onto the battlefield tapped (yes, which).
    for s in [s1, s2] {
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(s)]);
    }
    // The second card drawn this turn: Zimone's trigger targets P1.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, zd, "Draw a card").unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.on_battlefield(other));
    assert!(t.on_battlefield(s1) && t.on_battlefield(s2));
    assert!(tapped(&t, s1) && tapped(&t, s2));
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.is(CardType::Land))
            .count(),
        10
    );
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
}

#[test]
fn zimone_and_dina_check_the_land_count_after_the_first_pass_and_repeat_at_most_once() {
    cr!("608.2c");
    ruling!(
        "Zimone and Dina",
        "As this is your second time through the process, the ability will finish resolving."
    );
    supported("Zimone and Dina");
    let lands = |t: &TestGame| {
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.is(CardType::Land))
            .count()
    };
    // Seven lands: the land the first pass puts onto the battlefield is the eighth, so the
    // process is repeated, and only once (nine lands are controlled after the second
    // pass, but that pass is the last).
    let mut t = TestGame::new(2);
    let zd = t.battlefield(P0, "Zimone and Dina");
    t.lands(P0, "Forest", 7);
    let fodder = t.battlefield(P0, "Grizzly Bears");
    let s1 = t.hand(P0, "Swamp");
    let s2 = t.hand(P0, "Swamp");
    t.answer_choose(P0, &[obj(fodder)]);
    for s in [s1, s2] {
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(s)]);
    }
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, zd, "Draw a card").unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(s1) && t.on_battlefield(s2));
    assert_eq!(lands(&t), 9);
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
    // Seven lands and the land kept in hand: seven lands as the condition is checked, so
    // the process isn't repeated (one card drawn, so Zimone's trigger doesn't happen).
    let mut t = TestGame::new(2);
    let zd = t.battlefield(P0, "Zimone and Dina");
    t.lands(P0, "Forest", 7);
    let fodder = t.battlefield(P0, "Grizzly Bears");
    let s1 = t.hand(P0, "Swamp");
    t.answer_choose(P0, &[obj(fodder)]);
    t.answer_yes(P0, false);
    activate_containing(&mut t, P0, zd, "Draw a card").unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Swamp"));
    assert_eq!(t.zone(s1), mtg_engine::object::Zone::Hand(P0));
    assert_eq!(lands(&t), 7);
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
}

#[test]
fn gaining_control_of_a_creature_doesnt_give_control_of_its_auras() {
    cr!("301.5", "303.4", "714.2"); // CR 303.4: an Aura's controller is separate.
    ruling!(
        "The Trickster-God's Heist",
        "Gaining control of a creature doesn't cause a player to gain control of any Auras or Equipment attached to it."
    );
    supported("The Trickster-God's Heist");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = attach_new(&mut t, P0, "Holy Strength", bears);
    let giant = t.battlefield(P1, "Hill Giant");
    // Chapter I: you may exchange control of two target creatures.
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_yes(P0, true);
    t.enter(P0, "The Trickster-God's Heist");
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P1);
    assert_eq!(t.obj_now(giant).controller, P0);
    assert_eq!(t.obj_now(aura).controller, P0);
    assert_eq!(attached_to(&t, aura), Some(obj(bears)));
    assert_eq!(t.pt(bears), (3, 4));
}

#[test]
fn a_card_with_x_in_its_mana_cost_has_mana_value_0_in_a_graveyard() {
    cr!("202.3e", "202.3"); // CR 202.3e: X is 0 except on the stack.
    ruling!(
        "Revival // Revenge",
        "If a card in a graveyard has {X} in its mana cost, X is considered to be 0."
    );
    supported("Hangarback Walker");
    supported("Lightning Serpent");
    supported("Revival // Revenge");
    // Lightning Serpent ({X}{R}, a 2/1 that enters with X +1/+0 counters): mana value 1
    // in a graveyard. Revival returns a creature card with mana value 3 or less.
    let mut t = TestGame::new(2);
    let serpent = t.graveyard(P0, "Lightning Serpent");
    assert_eq!(t.g.mana_value_of(serpent), 1);
    let walker = t.graveyard(P0, "Hangarback Walker");
    assert_eq!(t.g.mana_value_of(walker), 0);
    t.lands(P0, "Plains", 2);
    let rev = t.hand(P0, "Revival // Revenge");
    t.cast(P0, rev)
        .method(CastMethod::Half(0))
        .target(serpent)
        .go();
    t.resolve_all();
    assert!(t.on_battlefield(serpent));
    assert_eq!(t.counters(serpent, "+1/+0"), 0);
    assert_eq!(t.pt(serpent), (2, 1));
    ruling!(
        "Command the Dreadhorde",
        "If a card in a graveyard has {X} in its mana cost, X is considered to be 0."
    );
    supported("Command the Dreadhorde");
    // Command the Dreadhorde: damage equal to the total mana value: 1 for the Serpent, 2
    // for the Bears.
    let mut t = TestGame::new(2);
    let serpent = t.graveyard(P0, "Lightning Serpent");
    let bears = t.graveyard(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Command the Dreadhorde");
    let cmd = t.hand(P0, "Command the Dreadhorde");
    t.cast(P0, cmd).targets(&[obj(serpent), obj(bears)]).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert!(t.on_battlefield(serpent) && t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P0);
    ruling!(
        "Sorin, Vengeful Bloodlord",
        "If a card in a graveyard has {X} in its mana cost, X is considered to be 0."
    );
    supported("Sorin, Vengeful Bloodlord");
    // Sorin's −X: the Serpent has mana value 1, so X is 1 (not 0 and not more); it comes
    // back as a Vampire.
    let mut t = TestGame::new(2);
    let serpent = t.graveyard(P0, "Lightning Serpent");
    let sorin = t.battlefield(P0, "Sorin, Vengeful Bloodlord");
    let from = t.asked().len();
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    let _ = t.activate(P0, sorin, 1, &[obj(serpent)]);
    t.clear_answers();
    let offered = target_candidates(&t, P0, from);
    assert!(offered.iter().all(|c| !c.contains(&obj(serpent))), "{offered:?}");
    t.resolve_all();
    assert!(!t.on_battlefield(serpent));
    let mut t = TestGame::new(2);
    let serpent = t.graveyard(P0, "Lightning Serpent");
    let sorin = t.battlefield(P0, "Sorin, Vengeful Bloodlord");
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.activate(P0, sorin, 1, &[obj(serpent)]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(serpent));
    assert!(t.obj_now(serpent).chars.has_subtype("Vampire"));
    assert_eq!(t.counters(sorin, counters::LOYALTY), 3);
    ruling!(
        "Blood on the Snow",
        "If a card in a graveyard has {X} in its mana cost, X is considered to be 0."
    );
    supported("Blood on the Snow");
    // Blood on the Snow paid with no snow mana: X is 0. A card with mana value 0 or less
    // can come back: the Walker (which then dies as a 0/0), not the Serpent.
    let mut t = TestGame::new(2);
    let serpent = t.graveyard(P0, "Lightning Serpent");
    let walker = t.graveyard(P0, "Hangarback Walker");
    t.answer_choose_moved(P0, &[obj(walker)]);
    lands_for_cost(&mut t, P0, "Blood on the Snow");
    let blood = t.hand(P0, "Blood on the Snow");
    t.cast(P0, blood).modes(&[0]).go();
    t.resolve_all();
    assert_ne!(t.g.current(walker), walker, "the Walker was put onto the battlefield");
    assert_eq!(t.g.current(serpent), serpent, "the Serpent stayed in the graveyard");
    assert!(t.in_graveyard(P0, "Lightning Serpent"));
}

#[test]
fn a_food_is_any_food_artifact_and_pays_one_cost() {
    cr!("205.3g", "118.3", "601.2h"); // One permanent pays one cost.
    ruling!(
        "Pizza Face, Gastromancer",
        "If an effect refers to a Food, it means any Food artifact, not just a Food artifact token."
    );
    supported("Pizza Face, Gastromancer");
    supported("Maraleaf Rider");
    let mut t = TestGame::new(2);
    let pizza = t.battlefield(P0, "Pizza Face, Gastromancer");
    let rider = t.battlefield(P0, "Maraleaf Rider");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Maraleaf Rider: "Sacrifice a Food: Target creature blocks this creature this turn
    // if able." Pizza Face, a Food creature, pays.
    t.answer_choose(P0, &[obj(pizza)]);
    t.activate(P0, rider, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Pizza Face, Gastromancer"));
    // A Food token sacrificed to its own ability can't also pay the Rider's.
    let food = create_token(&mut t, P0, "Food");
    t.lands(P0, "Forest", 2);
    assert!(can_activate(&mut t, P0, rider));
    t.activate(P0, food, 0, &[]).unwrap();
    assert!(!t.on_battlefield(food));
    assert!(!can_activate(&mut t, P0, rider));
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}

#[test]
fn chainers_torment_doubled_makes_two_tokens_that_each_deal_x_damage() {
    cr!("714.2b", "614.1c", "608.2c");
    ruling!(
        "Chainer's Torment",
        "If an effect such as that of Anointed Procession causes the final chapter ability of Chainer's Torment to create two Nightmare Horror tokens, each will deal X damage to you."
    );
    supported("Chainer's Torment");
    supported("Anointed Procession");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Anointed Procession");
    let saga = t.battlefield(P0, "Chainer's Torment");
    put_counters(&mut t, saga, "lore", 2);
    t.resolve_all();
    set_life(&mut t, P0, 11);
    // After P0's next draw step the third lore counter is added: X is 6 (half of 11,
    // rounded up); two 6/6 tokens each deal 6 to P0.
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    let horrors: Vec<ObjectId> =
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.is_token() && o.chars.has_subtype("Horror"))
            .map(|o| o.id)
            .collect();
    assert_eq!(horrors.len(), 2);
    assert!(horrors.iter().all(|h| t.pt(*h) == (6, 6)));
    assert_eq!(t.life(P0), 11 - 12);
    assert!(t.has_lost(P0));
}

#[test]
fn avatar_aang_triggers_once_a_bend_is_complete_even_if_parts_were_impossible() {
    cr!("701.65b", "701.66b", "603.2");
    ruling!(
        "Avatar Aang // Aang, Master of Elements",
        "An ability that triggers \"whenever you airbend\" triggers after the process described by the airbend keyword action is complete, even if some or all of the actions were impossible."
    );
    ruling!(
        "Avatar Aang // Aang, Master of Elements",
        "An ability that triggers \"whenever you earthbend\" triggers immediately after the list of actions earthbend entails is complete, even if some or all of those actions were impossible, perhaps because an effect does not allow counters to be placed on creatures you control."
    );
    supported("Avatar Aang // Aang, Master of Elements");
    supported("Airbending Lesson");
    supported("Dai Li Indoctrination");
    supported("Solemnity");
    // Airbending Lesson exiles P1's Bears (then draws); Aang's trigger draws another.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Avatar Aang // Aang, Master of Elements");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Airbending Lesson", &[obj(bears)]);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.hand_size(P0), hand + 2);
    // Earthbend 2 with Solemnity: no counters can be put on the land, which becomes a
    // 0/0 creature and dies (and returns tapped); Aang still triggers and draws.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Avatar Aang // Aang, Master of Elements");
    t.battlefield(P0, "Solemnity");
    let forest = t.battlefield(P0, "Forest");
    let hand = t.hand_size(P0);
    lands_for_cost(&mut t, P0, "Dai Li Indoctrination");
    let dai = t.hand(P0, "Dai Li Indoctrination");
    t.cast(P0, dai).modes(&[1]).target(forest).go();
    t.resolve_all();
    assert_eq!(t.counters(forest, counters::PLUS1), 0);
    // The land died and came back as a new object.
    assert!(t.on_battlefield(forest));
    assert_ne!(t.g.current(forest), forest);
    assert!(tapped(&t, forest));
    assert_eq!(t.hand_size(P0), hand + 1);
}
