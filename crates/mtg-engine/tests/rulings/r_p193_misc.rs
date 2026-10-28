//! Rulings batch P193 — other "this turn" trackers (Alena, Hobgoblin Bandit Lord, Moraug,
//! Neriv, Whipkeeper, Final Punishment, Impact Resonance, Niko Aris, Dargo, Sengir), spells
//! whose targets become illegal, and returning Auras with Brought Back.

use crate::r_s01_common::*;
use crate::r_s02_common::{destroy, target_candidates};
use crate::r_s03_common::respond;
use crate::r_s05_common::move_to;
use crate::r_s06_common::give_control;
use crate::r_s20_common::tap_for_mana;
use crate::r_s24_common::pool;
use crate::r_s29_common::{cast_and_resolve, divide, replacement_choosers, twice_first};
use crate::r_s30_common::damage_events;
use mtg_engine::ability::StepKind;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `battlefield` puts objects there this turn; makes one a permanent from an earlier turn.
fn from_earlier_turn(t: &mut TestGame, id: ObjectId) {
    t.g.objects[id.0 as usize].entered_turn = 0;
}

fn old(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.battlefield(p, name);
    from_earlier_turn(t, id);
    id
}

// ---------------------------------------------------------------------------------------
// Alena, Kessig Trapper: "{T}: Add an amount of {R} equal to the greatest power among
// creatures you control that entered this turn."
// ---------------------------------------------------------------------------------------

#[test]
fn alena_counts_creatures_that_entered_as_noncreatures_or_under_another_player() {
    cr!("605.1a", "605.3a");
    ruling!(
        "Alena, Kessig Trapper",
        "A creature you control that entered the battlefield as a noncreature permanent and/or under another player's control will be counted for Alena's mana ability."
    );
    ruling!(
        "Alena, Kessig Trapper",
        "Alena's activated ability is a mana ability. It doesn't use the stack and can't be responded to."
    );
    ruling!(
        "Alena, Kessig Trapper",
        "Creatures that are no longer on the battlefield won't be considered for Alena's mana ability, even if they entered that turn."
    );
    supported("Alena, Kessig Trapper");
    supported("Living Lands");
    // A Forest that entered this turn and became a 1/1 afterwards.
    let mut t = TestGame::new(2);
    let alena = old(&mut t, P0, "Alena, Kessig Trapper");
    old(&mut t, P0, "Craw Wurm");
    t.enter(P0, "Forest");
    old(&mut t, P0, "Living Lands");
    assert!(tap_for_mana(&mut t, P0, alena, "{R}"));
    assert_eq!(t.stack_len(), 0);
    assert_eq!(pool(&t, P0, ManaType::R), 1);
    // A creature that entered under P1's control, now P0's; and one that left again.
    let mut t = TestGame::new(2);
    let alena = old(&mut t, P0, "Alena, Kessig Trapper");
    let bears = t.enter(P1, "Grizzly Bears");
    give_control(&mut t, bears, P0);
    let wurm = t.enter(P0, "Craw Wurm");
    destroy(&mut t, wurm);
    assert!(tap_for_mana(&mut t, P0, alena, "{R}"));
    assert_eq!(pool(&t, P0, ManaType::R), 2);
}

// ---------------------------------------------------------------------------------------
// Hobgoblin Bandit Lord
// ---------------------------------------------------------------------------------------

#[test]
fn hobgoblin_bandit_lord_counts_goblins_that_entered_even_if_gone() {
    cr!("120.8", "608.2h");
    ruling!(
        "Hobgoblin Bandit Lord",
        "Hobgoblin Bandit Lord's activated ability only counts the number of Goblins that entered the battlefield this turn. It doesn't matter if those Goblins are still on the battlefield as it resolves."
    );
    ruling!(
        "Hobgoblin Bandit Lord",
        "If no Goblins have entered the battlefield this turn, no damage will be dealt. Abilities that trigger whenever damage is dealt won't trigger."
    );
    supported("Hobgoblin Bandit Lord");
    supported("Goblin Piker");
    for goblins in [0, 2] {
        let mut t = TestGame::new(2);
        let lord = old(&mut t, P0, "Hobgoblin Bandit Lord");
        for _ in 0..goblins {
            let g = t.enter(P0, "Goblin Piker");
            destroy(&mut t, g);
        }
        // An opponent's Goblin doesn't count.
        t.enter(P1, "Goblin Piker");
        t.lands(P0, "Mountain", 1);
        t.activate(P0, lord, 0, &[Entity::Player(P1)]).unwrap();
        t.resolve_all();
        assert_eq!(t.life(P1), 20 - goblins);
        assert_eq!(
            damage_events(&t).len(),
            goblins as usize / 2,
            "goblins: {goblins}"
        );
    }
}

// ---------------------------------------------------------------------------------------
// Moraug, Fury of Akoum: "Each creature you control gets +1/+0 for each time it has
// attacked this turn."
// ---------------------------------------------------------------------------------------

#[test]
fn moraug_counts_attacks_made_before_it_arrived_and_applies_to_that_combats_damage() {
    cr!("508.1", "611.3a");
    ruling!(
        "Moraug, Fury of Akoum",
        "Moraug's first ability checks each turn how many times each creature has attacked. It counts times that creature attacked before Moraug was on the battlefield, and it stops applying if Moraug leaves the battlefield."
    );
    ruling!(
        "Moraug, Fury of Akoum",
        "The first time a creature attacks in a turn, the +1/+0 bonus will apply to the combat damage it deals during that combat."
    );
    supported("Moraug, Fury of Akoum");
    let mut t = TestGame::new(2);
    let moraug = old(&mut t, P0, "Moraug, Fury of Akoum");
    let bears = old(&mut t, P0, "Grizzly Bears");
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 17);
    destroy(&mut t, moraug);
    t.g.recompute();
    assert_eq!(t.pt(bears), (2, 2));
    // Attacked before Moraug arrived.
    let mut t = TestGame::new(2);
    let bears = old(&mut t, P0, "Grizzly Bears");
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
    t.battlefield(P0, "Moraug, Fury of Akoum");
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 2));
}

#[test]
fn moraug_doesnt_count_a_creature_that_entered_attacking() {
    cr!("508.4", "508.1a");
    ruling!(
        "Moraug, Fury of Akoum",
        "A creature that enters the battlefield attacking hasn't attacked that combat. Moraug's ability won't count that combat. Unless that creature has haste, it won't be able to attack in additional combats this turn."
    );
    supported("Redoubled Stormsinger");
    let mut t = TestGame::new(2);
    old(&mut t, P0, "Moraug, Fury of Akoum");
    let singer = old(&mut t, P0, "Redoubled Stormsinger");
    let soldier = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    crate::r_s03_common::to_blockers(&mut t, &[(singer, Entity::Player(P1))], &[]);
    let copy = tokens(&t, P0)
        .into_iter()
        .find(|id| *id != t.g.current(soldier))
        .unwrap();
    assert!(crate::r_s10_common::attacking(&t, copy));
    t.g.recompute();
    assert_eq!(t.pt(copy), (1, 1));
    assert_eq!(t.pt(singer), (4, 3));
    // In a later combat this turn, it can't attack (summoning sick, no haste).
    crate::r_s21_common::go_to(&mut t, Step::EndOfCombat);
    t.g.turn
        .schedule
        .insert(0, Step::BeginningOfCombat);
    t.g.turn.schedule.insert(1, Step::DeclareAttackers);
    crate::r_s21_common::go_to(&mut t, Step::BeginningOfCombat);
    t.g.untap(copy);
    t.g.untap(singer);
    assert!(!crate::r_s02_common::can_attack(&mut t, copy));
    assert!(crate::r_s02_common::can_attack(&mut t, singer));
}

// ---------------------------------------------------------------------------------------
// Neriv, Heart of the Storm: "If a creature you control that entered this turn would deal
// damage, it deals twice that much damage instead."
// ---------------------------------------------------------------------------------------

fn prevent_first(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    match d {
        Decision::ChooseReplacement { options } => options
            .iter()
            .position(|o| !o.contains("twice"))
            .map(Answer::Index),
        _ => None,
    }
}

#[test]
fn neriv_and_prevention_are_ordered_by_the_player_dealt_damage() {
    cr!("616.1", "615.1a", "614.1a");
    ruling!(
        "Neriv, Heart of the Storm",
        "the player being dealt damage or the controller of the permanent being dealt damage chooses the order in which any such effects, including the effect of Neriv’s last ability, apply. If all of the damage is prevented before the effect of Neriv’s last ability would apply, its effect no longer applies."
    );
    supported("Neriv, Heart of the Storm");
    supported("Healing Salve");
    // Grizzly Bears (entered this turn) deals 2 damage to P1, who has a "prevent the next 3
    // damage" shield.
    for doubling_first in [true, false] {
        let mut t = TestGame::new(2);
        old(&mut t, P0, "Neriv, Heart of the Storm");
        let bears = t.enter(P0, "Grizzly Bears");
        t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![1]));
        cast_and_resolve(&mut t, P1, "Healing Salve", &[Entity::Player(P1)]);
        respond(
            &mut t,
            P1,
            if doubling_first {
                twice_first
            } else {
                prevent_first
            },
        );
        let from = t.asked().len();
        t.g.deal_damage(bears, Entity::Player(P1), 2, false);
        t.g.flush_events();
        t.settle();
        assert_eq!(replacement_choosers(&t, from), vec![P1]);
        assert_eq!(t.life(P1), if doubling_first { 19 } else { 20 });
    }
}

#[test]
fn neriv_doubles_damage_after_it_is_assigned() {
    cr!("510.1c", "510.2", "614.1a");
    ruling!(
        "Neriv, Heart of the Storm",
        "If damage dealt is being divided or assigned among multiple permanents or players, that damage is divided or assigned before any effects modify how much damage would be dealt."
    );
    supported("War Mammoth");
    let mut t = TestGame::new(2);
    old(&mut t, P0, "Neriv, Heart of the Storm");
    let mammoth = t.enter(P0, "War Mammoth");
    t.g.objects[mammoth.0 as usize].summoning_sick = false;
    let bears = old(&mut t, P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![2, 1]));
    t.attack(&[(mammoth, Entity::Player(P1))], &[(bears, mammoth)]);
    assert_eq!(t.life(P1), 18);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(damage_events(&t)
        .iter()
        .any(|(s, to, n, _)| *s == mammoth && *to == Entity::Object(bears) && *n == 4));
}

// ---------------------------------------------------------------------------------------
// Damage already dealt this turn
// ---------------------------------------------------------------------------------------

#[test]
fn whipkeeper_counts_damage_removed_by_regeneration() {
    cr!("701.19a", "120.6");
    ruling!(
        "Whipkeeper",
        "If the creature regenerated during the turn, and thereby had all damage on it removed, the damage before it regenerated was still dealt that turn and will be counted."
    );
    supported("Whipkeeper");
    supported("Drudge Skeletons");
    let mut t = TestGame::new(2);
    let whip = old(&mut t, P0, "Whipkeeper");
    let skel = old(&mut t, P1, "Drudge Skeletons");
    t.lands(P1, "Swamp", 1);
    t.activate(P1, skel, 0, &[]).unwrap();
    t.resolve_all();
    let src = old(&mut t, P0, "Grizzly Bears");
    t.g.deal_damage(src, Entity::Object(skel), 1, false);
    t.g.flush_events();
    t.settle();
    assert!(t.on_battlefield(skel));
    assert_eq!(t.obj_now(skel).damage, 0);
    t.activate(P0, whip, 0, &[Entity::Object(skel)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Drudge Skeletons"));
}

#[test]
fn final_punishment_doesnt_count_prevented_damage() {
    cr!("615.1a", "120.3");
    ruling!(
        "Final Punishment",
        "Damage that is prevented does not count toward the total."
    );
    supported("Final Punishment");
    let mut t = TestGame::new(2);
    let src = old(&mut t, P0, "Grizzly Bears");
    t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![1]));
    cast_and_resolve(&mut t, P1, "Healing Salve", &[Entity::Player(P1)]);
    t.g.deal_damage(src, Entity::Player(P1), 5, false);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.life(P1), 18);
    cast_and_resolve(&mut t, P0, "Final Punishment", &[Entity::Player(P1)]);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn impact_resonance_keeps_its_division_when_some_targets_become_illegal() {
    cr!("601.2d", "608.2b");
    ruling!(
        "Impact Resonance",
        "If Impact Resonance has multiple targets, and some, but not all, of them become illegal before Impact Resonance resolves, Impact Resonance will still deal damage to the remaining legal targets according to the original damage division."
    );
    supported("Impact Resonance");
    let mut t = TestGame::new(2);
    let src = old(&mut t, P0, "Grizzly Bears");
    let a = old(&mut t, P1, "Centaur Courser");
    let b = old(&mut t, P1, "Craw Wurm");
    // The greatest damage dealt by a source this turn: 4.
    t.g.deal_damage(src, Entity::Player(P1), 4, false);
    t.g.flush_events();
    t.settle();
    divide(&mut t, P0, &[1, 3]);
    crate::r_s33_common::cast_one_slot(
        &mut t,
        P0,
        "Impact Resonance",
        &[Entity::Object(a), Entity::Object(b)],
    );
    move_to(&mut t, a, Zone::Hand(P1));
    t.resolve_all();
    assert_eq!(t.obj_now(b).damage, 3);
    assert!(t.on_battlefield(b));
}

// ---------------------------------------------------------------------------------------
// Niko Aris, Dargo, Sengir, Children of Korlis, Stonehorn Dignitary, Canopy Stalker,
// Blazing Effigy
// ---------------------------------------------------------------------------------------

#[test]
fn niko_aris_makes_shards_only_when_cast_with_x() {
    cr!("107.3m", "601.2b");
    ruling!(
        "Niko Aris",
        "If Niko Aris enters the battlefield without being cast, they were cast for a cost other than their mana cost, or if X was chosen to be 0, the enters-the-battlefield ability won't create any Shard tokens."
    );
    supported("Niko Aris");
    let mut t = TestGame::new(2);
    t.enter(P0, "Niko Aris");
    t.resolve_all();
    assert!(crate::r_s05_common::tokens_with_subtype(&t, P0, "Shard").is_empty());
    for x in [0, 2] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Plains", 1);
        t.lands(P0, "Island", 2 + x as usize);
        let niko = t.hand(P0, "Niko Aris");
        t.cast(P0, niko).x(x).go();
        t.resolve_all();
        assert_eq!(
            crate::r_s05_common::tokens_with_subtype(&t, P0, "Shard").len(),
            x as usize
        );
    }
}

#[test]
fn dargo_cant_cost_less_than_r_but_any_number_can_be_sacrificed() {
    cr!("601.2b", "601.2f", "118.7");
    ruling!(
        "Dargo, the Shipwrecker",
        "Dargo's first ability can't reduce its cost below {R}. You can sacrifice any number of artifacts and creatures, even if they won't reduce Dargo's cost any further."
    );
    supported("Dargo, the Shipwrecker");
    let mut t = TestGame::new(2);
    let fodder: Vec<Entity> = (0..4)
        .map(|_| Entity::Object(old(&mut t, P0, "Grizzly Bears")))
        .collect();
    t.lands(P0, "Mountain", 1);
    // Sacrifice four ({8} of reduction for a {6}{R} spell).
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(4));
    for f in &fodder {
        t.answer_choose(P0, &[*f]);
    }
    let dargo = t.hand(P0, "Dargo, the Shipwrecker");
    t.cast(P0, dargo).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Dargo, the Shipwrecker").len(), 1);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 0);
}

#[test]
fn sengir_dies_with_creatures_dealt_lethal_damage_at_the_same_time() {
    cr!("704.3", "704.5g", "704.5a");
    ruling!(
        "Sengir, the Dark Baron",
        "If Sengir, the Dark Baron is dealt lethal damage at the same time as other creatures, they die at the same time. The second ability won't resolve in time to save Sengir, the Dark Baron."
    );
    ruling!(
        "Sengir, the Dark Baron",
        "If you and another player lose the game at the same time, the third ability won't resolve in time to save you."
    );
    supported("Sengir, the Dark Baron");
    let mut t = TestGame::new(2);
    let sengir = old(&mut t, P0, "Sengir, the Dark Baron");
    let bears = old(&mut t, P1, "Grizzly Bears");
    let src = old(&mut t, P1, "Craw Wurm");
    t.g.deal_damage_batch(
        vec![(src, Entity::Object(sengir), 4), (src, Entity::Object(bears), 2)],
        false,
    );
    t.g.flush_events();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Sengir, the Dark Baron"));
    // Three players: P0 and P1 reach 0 life together.
    let mut t = TestGame::new(3);
    old(&mut t, P0, "Sengir, the Dark Baron");
    t.g.players[0].life = 0;
    t.g.players[1].life = 0;
    t.settle();
    t.resolve_all();
    assert!(t.has_lost(P0) && t.has_lost(P1));
}

#[test]
fn children_of_korlis_cant_save_you_at_zero_life() {
    cr!("704.5a", "117.5");
    ruling!(
        "Children of Korlis",
        "If your life total becomes 0 or less, you'll lose the game before you can activate the ability of Children of Korlis."
    );
    supported("Children of Korlis");
    let mut t = TestGame::new(3);
    old(&mut t, P0, "Children of Korlis");
    t.g.lose_life(P0, 20);
    t.g.flush_events();
    t.settle();
    assert!(t.has_lost(P0));
}

#[test]
fn two_stonehorn_dignitaries_skip_two_combat_phases() {
    cr!("614.10", "500.11");
    ruling!(
        "Stonehorn Dignitary",
        "If more than one Stonehorn Dignitary enters the battlefield and their abilities target the same opponent, that opponent will skip that many combat phases (over multiple turns if necessary). Skipping one combat phase won't satisfy all of the effects."
    );
    supported("Stonehorn Dignitary");
    let mut t = TestGame::new(2);
    for _ in 0..2 {
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.enter(P0, "Stonehorn Dignitary");
        t.resolve_all();
    }
    let skips = |t: &TestGame| {
        t.g.player(P1)
            .skips
            .iter()
            .filter(|s| **s == StepKind::Combat)
            .count()
    };
    assert_eq!(skips(&t), 2);
    t.advance_to(P1, Step::End);
    assert_eq!(skips(&t), 1);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P1, Step::End);
    assert_eq!(skips(&t), 0);
    let combats_of_p1 = t
        .g
        .turn_events
        .iter()
        .filter(|e| {
            matches!(e, mtg_engine::events::Event::StepBegan { step: Step::BeginningOfCombat, active } if *active == P1)
        })
        .count();
    assert_eq!(combats_of_p1, 0);
}

#[test]
fn canopy_stalker_is_unblocked_if_no_creature_can_block_it() {
    cr!("509.1c");
    ruling!(
        "Canopy Stalker",
        "If each creature the defending player controls can't block for any reason (such as being tapped), then Canopy Stalker isn't blocked."
    );
    supported("Canopy Stalker");
    let mut t = TestGame::new(2);
    let stalker = old(&mut t, P0, "Canopy Stalker");
    let bears = old(&mut t, P1, "Grizzly Bears");
    t.g.tap(bears);
    t.attack(&[(stalker, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn blazing_effigy_does_nothing_without_a_creature_to_target() {
    cr!("603.3d");
    ruling!(
        "Blazing Effigy",
        "If it dies and there are no creatures to target, then the ability does nothing."
    );
    supported("Blazing Effigy");
    let mut t = TestGame::new(2);
    let effigy = old(&mut t, P0, "Blazing Effigy");
    destroy(&mut t, effigy);
    assert_eq!(t.stack_len(), 0);
    // With a creature, it targets it.
    let mut t = TestGame::new(2);
    let effigy = old(&mut t, P0, "Blazing Effigy");
    let courser = old(&mut t, P1, "Centaur Courser");
    t.answer_targets(P0, &[Entity::Object(courser)]);
    destroy(&mut t, effigy);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Centaur Courser"));
}

// ---------------------------------------------------------------------------------------
// Illegal targets
// ---------------------------------------------------------------------------------------

#[test]
fn spells_with_an_illegal_only_target_do_nothing_at_all() {
    cr!("608.2b");
    ruling!(
        "Deadly Embrace",
        "If the target creature is an illegal target as Deadly Embrace tries to resolve, it won't resolve and none of its effects will happen. You won't draw cards."
    );
    ruling!(
        "Fists of Flame",
        "If the target creature is an illegal target by the time Fists of Flame tries to resolve, the spell doesn't resolve. You won't draw a card."
    );
    ruling!(
        "Nightmare's Thirst",
        "If the target creature is an illegal target when Nightmare’s Thirst tries to resolve, the spell doesn’t resolve. You won’t gain 1 life."
    );
    for (name, theirs) in [
        ("Deadly Embrace", true),
        ("Fists of Flame", false),
        ("Nightmare's Thirst", true),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let dead = old(&mut t, P1, "Grizzly Bears");
        destroy(&mut t, dead);
        let target = old(&mut t, if theirs { P1 } else { P0 }, "Centaur Courser");
        crate::r_s25_common::cast_new(&mut t, P0, name, &[Entity::Object(target)]);
        let owner = t.obj(target).owner;
        move_to(&mut t, target, Zone::Hand(owner));
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand, "{name}");
        assert_eq!(t.life(P0), 20, "{name}");
        assert!(t.in_graveyard(P0, name), "{name}");
    }
}

#[test]
fn burn_together_with_one_illegal_target() {
    cr!("608.2b", "608.2c");
    ruling!(
        "Callous Sell-Sword // Burn Together",
        "If the first target of Burn Together is an illegal target as the spell resolves but the last target is still legal, Burn Together will resolve, but no damage will be dealt and nothing will be sacrificed."
    );
    ruling!(
        "Callous Sell-Sword // Burn Together",
        "If the last target of Burn Together is an illegal target as the spell resolves but the first target is still legal, Burn Together will still resolve and you'll still sacrifice the first target."
    );
    for first_illegal in [true, false] {
        let mut t = TestGame::new(2);
        let mine = old(&mut t, P0, "Centaur Courser");
        let theirs = old(&mut t, P1, "Grizzly Bears");
        t.lands(P0, "Mountain", 1);
        let card = t.hand(P0, "Callous Sell-Sword // Burn Together");
        t.cast(P0, card)
            .method(crate::r_s18_common::ADVENTURE)
            .targets(&[Entity::Object(mine), Entity::Object(theirs)])
            .go();
        if first_illegal {
            give_control(&mut t, mine, P1);
        } else {
            move_to(&mut t, theirs, Zone::Hand(P1));
        }
        t.resolve_all();
        if first_illegal {
            assert!(t.on_battlefield(mine));
            assert!(t.on_battlefield(theirs));
            assert_eq!(t.obj_now(theirs).damage, 0);
        } else {
            assert!(t.in_graveyard(P0, "Centaur Courser"));
        }
        // It resolved: an Adventure that resolves is exiled (CR 715.4).
        assert_eq!(t.zone(t.g.current(card)), Zone::Exile);
    }
}

// ---------------------------------------------------------------------------------------
// Brought Back and Auras
// ---------------------------------------------------------------------------------------

#[test]
fn brought_back_auras_choose_a_legal_existing_object_to_enchant() {
    cr!("303.4f", "702.16c");
    ruling!(
        "Brought Back",
        "If you return an Aura this way, you choose what the Aura will enchant just before it enters, but you can't choose any permanent cards that will enter at the same time as that Aura."
    );
    supported("Brought Back");
    supported("Troll Ascetic");
    supported("Shivan Zombie");
    let mut t = TestGame::new(2);
    let bears = old(&mut t, P0, "Grizzly Bears");
    let pacifism = attach_and_destroy(&mut t, bears);
    destroy(&mut t, bears);
    let troll = old(&mut t, P1, "Troll Ascetic");
    let zombie = old(&mut t, P1, "Shivan Zombie");
    let bears_card = t.g.current(bears);
    let pac_card = t.g.current(pacifism);
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Object(troll)]);
    crate::r_s25_common::cast_new(
        &mut t,
        P0,
        "Brought Back",
        &[Entity::Object(pac_card), Entity::Object(bears_card)],
    );
    t.resolve_all();
    let offered = asked_since(&t, from)
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. }
                if candidates.contains(&Entity::Object(troll)) =>
            {
                Some(candidates)
            }
            _ => None,
        })
        .expect("asked what to enchant");
    // Hexproof is fine (no target); protection from white isn't; the returning Bears isn't.
    assert!(!offered.contains(&Entity::Object(zombie)));
    assert!(offered
        .iter()
        .all(|e| matches!(e, Entity::Object(o) if t.obj(*o).chars.name != "Grizzly Bears")));
    let aura = t.named_on_battlefield("Pacifism")[0];
    assert_eq!(crate::r_s06_common::attached_to(&t, aura), Some(Entity::Object(troll)));
    assert!(t.obj(aura).tapped);
    let _ = target_candidates;
}

/// Puts Pacifism onto `on`, then destroys it; returns the Pacifism.
fn attach_and_destroy(t: &mut TestGame, on: ObjectId) -> ObjectId {
    let pac = crate::r_s06_common::attach_new(t, P0, "Pacifism", on);
    destroy(t, pac);
    pac
}

#[test]
fn brought_back_leaves_an_aura_with_nothing_to_enchant_in_the_graveyard() {
    cr!("303.4g");
    ruling!(
        "Brought Back",
        "If there's nothing legal for the Aura to enchant, it stays in the graveyard."
    );
    let mut t = TestGame::new(2);
    let bears = old(&mut t, P0, "Grizzly Bears");
    let pacifism = attach_and_destroy(&mut t, bears);
    move_to(&mut t, bears, Zone::Exile);
    let pac_card = t.g.current(pacifism);
    crate::r_s25_common::cast_new(&mut t, P0, "Brought Back", &[Entity::Object(pac_card)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Pacifism"));
}
