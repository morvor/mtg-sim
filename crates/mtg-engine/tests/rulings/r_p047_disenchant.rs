//! Rulings batch P047 — artifact and enchantment removal: targets chosen and checked
//! (CR 601.2c, 608.2b), types changing while on the stack, "if you cast this during
//! your main phase", cast triggers resolving first, one life-gain event, and spells
//! cast while an ability resolves (CR 608.2g).

use crate::r_p047_common::*;
use crate::r_s01_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

#[test]
fn return_to_dust_second_target_only_in_main_phase() {
    cr!("601.2c", "608.2c", "505.1");
    ruling!(
        "Return to Dust",
        "Return to Dust can always target a second artifact or enchantment; it just won't exile it if it's not your main phase when you cast Return to Dust."
    );
    for main in [true, false] {
        let mut t = TestGame::new(2);
        if !main {
            t.set_step(P0, Step::BeginningOfCombat);
        }
        let a = t.battlefield(P1, "Ornithopter");
        let e = t.battlefield(P1, "Glorious Anthem");
        spell(&mut t, P0, "Return to Dust", &[obj(a), obj(e)]);
        assert_eq!(t.zone(a), Zone::Exile, "main={main}");
        let want = if main { Zone::Exile } else { Zone::Battlefield };
        assert_eq!(t.zone(e), want, "main={main}");
    }
}

#[test]
fn silverback_elder_trigger_resolves_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5", "701.6a");
    ruling!(
        "Silverback Elder",
        "Silverback Elder’s triggered ability resolves before the spell that caused it to trigger. The ability will resolve even if that spell is countered."
    );
    supported("Silverback Elder");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Silverback Elder");
    // "You gain 4 life."
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    let bears = cast_x(&mut t, P0, "Grizzly Bears", None, &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // The trigger is on top of the creature spell; counter the spell first.
    let cs = cast_x(&mut t, P1, "Counterspell", None, &[obj(bears)]);
    t.resolve();
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    assert!(!t.g.is_live(cs));
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    // Without a response: the life is gained while the Bears are still on the stack.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Silverback Elder");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    let bears = cast_x(&mut t, P0, "Grizzly Bears", None, &[]);
    t.resolve();
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.zone(bears), Zone::Stack);
}

#[test]
fn storm_the_citadel_affects_only_creatures_there_on_resolution() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Storm the Citadel",
        "Storm the Citadel affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won’t get +2/+2 or gain the triggered ability."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    spell(&mut t, P0, "Storm the Citadel", &[]);
    let later = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.pt(later), (3, 3));
    // Only the Bears have the damage trigger: the Giant's combat damage destroys nothing.
    let art = t.battlefield(P1, "Ornithopter");
    attack_with(&mut t, &[(later, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert!(t.on_battlefield(art));
}

#[test]
fn heliod_s_intervention_gains_twice_x_once() {
    cr!("119.9", "119.10", "603.2");
    ruling!(
        "Heliod's Intervention",
        "The second mode of Heliod's Intervention causes the player to gain one amount of life equal to twice X. It doesn't cause the player to gain X life then gain X life again."
    );
    supported("Heliod's Intervention");
    let mut t = TestGame::new(2);
    let pm = t.battlefield(P0, "Ajani's Pridemate");
    let h = t.hand(P0, "Heliod's Intervention");
    t.lands(P0, "Plains", 5);
    t.cast(P0, h).modes(&[1]).x(3).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
    let gains = t
        .g
        .turn_events
        .iter()
        .filter(|e| matches!(e, Event::LifeGained { player, .. } if *player == P0))
        .count();
    assert_eq!(gains, 1);
    assert_eq!(t.counters(pm, "+1/+1"), 1);
}

#[test]
fn disenchant_isnt_modal_a_target_changing_type_is_still_destroyed() {
    cr!("608.2b", "613.1d");
    ruling!(
        "Disenchant",
        "This is not modal. If the target changes from an artifact to an enchantment or vice versa, this still destroys it."
    );
    ruling!(
        "Seal of Primordium",
        "This is not modal. If the target changes from an artifact to an enchantment or vice versa, this still destroys it."
    );
    for name in ["Disenchant", "Seal of Primordium"] {
        let mut t = TestGame::new(2);
        let thopter = t.battlefield(P1, "Ornithopter");
        if name == "Disenchant" {
            cast_x(&mut t, P0, name, None, &[obj(thopter)]);
        } else {
            let seal = t.battlefield(P0, name);
            t.activate(P0, seal, 0, &[obj(thopter)]).unwrap();
        }
        // In response, One with the Stars makes it an enchantment and nothing else.
        let ows = t.battlefield(P1, "One with the Stars");
        t.g.attach(ows, obj(thopter));
        t.g.recompute();
        assert!(!t.g.obj(thopter).is(CardType::Artifact));
        assert!(t.g.obj(thopter).is(CardType::Enchantment));
        t.resolve_all();
        assert_eq!(t.zone(thopter), Zone::Graveyard(P1), "{name}");
    }
}

#[test]
fn stomp_and_howl_needs_both_targets() {
    cr!("601.2c", "115.3");
    ruling!(
        "Stomp and Howl",
        "This spell can’t be cast unless there are both an artifact and an enchantment on the battlefield to target. If a permanent is both an artifact and an enchantment, it can be both targets."
    );
    supported("Stomp and Howl");
    // Only an artifact: it can't be cast.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Ornithopter");
    let s = t.hand(P0, "Stomp and Howl");
    give_mana_for(&mut t, P0, "Stomp and Howl");
    t.answer_targets(P0, &[obj(a)]);
    assert!(t.cast(P0, s).try_go().is_err());
    // An enchantment artifact is both targets.
    let mut t = TestGame::new(2);
    let bow = t.battlefield(P1, "Bow of Nylea");
    spell(&mut t, P0, "Stomp and Howl", &[obj(bow), obj(bow)]);
    assert_eq!(t.zone(bow), Zone::Graveyard(P1));
}

#[test]
fn broken_bond_needs_a_target() {
    cr!("601.2c", "115.1");
    ruling!(
        "Broken Bond",
        "You can’t cast Broken Bond unless you choose an artifact or enchantment as a target."
    );
    let mut t = TestGame::new(2);
    t.hand(P0, "Forest");
    let s = t.hand(P0, "Broken Bond");
    give_mana_for(&mut t, P0, "Broken Bond");
    assert!(t.cast(P0, s).try_go().is_err());
    // With a target: it destroys it and puts the land onto the battlefield.
    let a = t.battlefield(P1, "Ornithopter");
    t.answer_yes(P0, true);
    t.cast(P0, s).target(a).go();
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Graveyard(P1));
}

#[test]
fn cathartic_parting_shuffles_with_zero_targets() {
    cr!("601.2c", "115.1d", "701.24a");
    ruling!(
        "Cathartic Parting",
        "You may target zero cards in your graveyard and still choose to shuffle your library."
    );
    supported("Cathartic Parting");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Ornithopter");
    t.answer_targets(P0, &[obj(a)]);
    // Zero cards from the graveyard.
    t.answer_targets(P0, &[]);
    t.answer_yes(P0, true);
    spell(&mut t, P0, "Cathartic Parting", &[]);
    assert!(matches!(t.zone(a), Zone::Library(_)));
    let shuffled = t
        .g
        .turn_events
        .iter()
        .filter(|e| matches!(e, Event::Shuffled { player } if *player == P0))
        .count();
    assert_eq!(shuffled, 1);
}

#[test]
fn garth_one_eye_casts_the_copy_while_resolving_paying_its_costs() {
    cr!("608.2g", "707.13", "601.2h");
    ruling!(
        "Garth One-Eye",
        "You must pay the costs for the chosen card, but you cast the copy as the ability resolves, which may not be a time that you could normally cast that card."
    );
    // On the opponent's turn, Shivan Dragon (a creature) is cast and its cost paid.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let garth = t.battlefield(P0, "Garth One-Eye");
    t.lands(P0, "Mountain", 6);
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.answer_yes(P0, true);
    t.activate(P0, garth, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Shivan Dragon").len(), 1);
    assert_eq!(tapped_lands(&t, P0), 6);
    // Without the mana, it can't be cast.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let garth = t.battlefield(P0, "Garth One-Eye");
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.answer_yes(P0, true);
    t.activate(P0, garth, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.named_on_battlefield("Shivan Dragon").is_empty());
}

#[test]
fn destructive_revelry_damage_needs_a_legal_target_not_destruction() {
    cr!("608.2b", "702.12b");
    ruling!(
        "Destructive Revelry",
        "You must target an artifact or enchantment to cast Destructive Revelry. If that artifact or enchantment is an illegal target when Destructive Revelry tries to resolve, it won't resolve and none of its effects will happen. No damage will be dealt. However, if Destructive Revelry resolves and the artifact or enchantment isn't destroyed (perhaps because it has indestructible), damage will be dealt."
    );
    supported("Destructive Revelry");
    // No target: it can't be cast.
    let mut t = TestGame::new(2);
    let s = t.hand(P0, "Destructive Revelry");
    give_mana_for(&mut t, P0, "Destructive Revelry");
    assert!(t.cast(P0, s).try_go().is_err());
    // Illegal target on resolution: no damage.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Ornithopter");
    cast_x(&mut t, P0, "Destructive Revelry", None, &[obj(a)]);
    spell(&mut t, P1, "Unsummon", &[obj(a)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // Indestructible: not destroyed, damage still dealt.
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P1, "Darksteel Myr");
    spell(&mut t, P0, "Destructive Revelry", &[obj(myr)]);
    assert!(t.on_battlefield(myr));
    assert_eq!(t.life(P1), 18);
}
