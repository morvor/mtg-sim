//! Rulings batch P115 — the "pair commander" cards (partner, Background, Doctor's
//! companion) and Wicker Picker's sticker kicker: tokens entering attacking, cascade
//! ordering, graveyard-leaving triggers, token replacement, casting from a graveyard and
//! from exile, tapping artifacts for a cost, and Background-granted triggers.

use crate::r_p115_common::*;
use crate::r_s02_common::can_play_land;
use crate::r_s04_common::add_mana;
use crate::r_s06_common::activate_containing;
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn attack_target(t: &TestGame, a: ObjectId) -> Option<Entity> {
    t.g.combat.as_ref().and_then(|c| c.attack_target(a))
}

#[test]
fn gut_you_choose_what_the_skeleton_attacks() {
    cr!("508.4");
    ruling!(
        "Gut, True Soul Zealot",
        "You choose the player or planeswalker the Skeleton is attacking."
    );
    supported("Gut, True Soul Zealot");
    for choose_jace in [true, false] {
        let mut t = TestGame::new(2);
        let gut = t.battlefield(P0, "Gut, True Soul Zealot");
        let thopter = t.battlefield(P0, "Ornithopter");
        let jace = t.battlefield(P1, "Jace Beleren");
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(thopter)]);
        t.answer_choose(P0, &[if choose_jace { obj(jace) } else { pl(P1) }]);
        attack_with(&mut t, &[(gut, pl(P1))]);
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Ornithopter"));
        let skeletons = with_subtype(&t, P0, "Skeleton");
        assert_eq!(skeletons.len(), 1);
        assert!(t.obj_now(skeletons[0]).tapped);
        assert_eq!(
            attack_target(&t, skeletons[0]),
            Some(if choose_jace { obj(jace) } else { pl(P1) })
        );
        assert_eq!(attack_target(&t, gut), Some(pl(P1)));
    }
}

#[test]
fn the_first_doctor_targets_before_cascade_reveals() {
    cr!("603.3d", "702.85a", "603.3b");
    ruling!(
        "The First Doctor",
        "You choose the target for The First Doctor's last ability before revealing any cards for cascade."
    );
    supported("The First Doctor");
    let mut t = TestGame::new(2);
    let doctor = t.battlefield(P0, "The First Doctor");
    t.library_top(P0, "Grizzly Bears");
    // The number of cards in exile when P0 chooses the Doctor's target.
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseTargets { .. }),
        |g| g.exile.len(),
    );
    t.answer_targets(P0, &[obj(doctor)]);
    cast_at(&mut t, P0, "Bloodbraid Elf", &[]);
    t.settle();
    assert_eq!(seen.lock().unwrap().clone(), vec![0]);
    t.resolve_all();
    assert_eq!(t.counters(doctor, "+1/+1"), 1);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn tormod_makes_one_zombie_per_event() {
    cr!("603.2c");
    ruling!(
        "Tormod, the Desecrator",
        "You create one Zombie token each time Tormod, the Desecrator's ability resolves, no matter how many cards left your graveyard."
    );
    supported("Tormod, the Desecrator");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tormod, the Desecrator");
    for name in ["Grizzly Bears", "Hill Giant", "Shock"] {
        t.graveyard(P0, name);
    }
    let crypt = t.battlefield(P0, "Tormod's Crypt");
    t.answer_targets(P0, &[pl(P0)]);
    t.activate(P0, crypt, 0, &[]).expect("Crypt");
    t.resolve_all();
    // Everything, the sacrificed Crypt included, was exiled at once.
    assert_eq!(t.graveyard_size(P0), 0);
    assert!(t.in_exile("Tormod's Crypt"));
    let zombies = with_subtype(&t, P0, "Zombie")
        .into_iter()
        .filter(|z| t.g.obj(*z).is_token())
        .collect::<Vec<_>>();
    assert_eq!(zombies.len(), 1);
    assert!(t.obj_now(zombies[0]).tapped);
}

#[test]
fn amber_gristle_discards_on_resolution_even_an_empty_hand() {
    cr!("608.2c", "608.2d", "117.3d");
    ruling!(
        "Amber Gristle O'Maul",
        "You decide whether to discard your hand as Amber Gristle O'Maul's ability resolves. No player may respond after you have decided to discard your hand but before you have drawn cards."
    );
    ruling!(
        "Amber Gristle O'Maul",
        "You may choose to discard your hand even if it is empty. Especially if it is empty."
    );
    supported("Amber Gristle O'Maul");
    for in_hand in [0usize, 2] {
        let mut t = TestGame::new(3);
        let amber = t.battlefield(P0, "Amber Gristle O'Maul");
        for _ in 0..in_hand {
            t.hand(P0, "Grizzly Bears");
        }
        // When P0 decides, and P1's priority: when, the stack's size and P0's hand size.
        let decided = watch(
            &mut t,
            P0,
            |d| matches!(d, Decision::YesNo { .. }),
            |_| next_seq(),
        );
        let prio = watch(
            &mut t,
            P1,
            |d| matches!(d, Decision::Priority { .. }),
            |g| (next_seq(), g.stack.len(), g.player(P0).hand.len()),
        );
        t.answer_yes(P0, true);
        attack_with(&mut t, &[(amber, pl(P1))]);
        assert_eq!(t.stack_len(), 1);
        assert!(
            decided.lock().unwrap().is_empty(),
            "decided only as it resolves"
        );
        // Through the real priority loop: both pass, it resolves, both pass again.
        pass_until_step_ends(&mut t);
        let decided = decided.lock().unwrap().clone();
        assert_eq!(decided.len(), 1, "{in_hand}");
        let prio = prio.lock().unwrap().clone();
        // P1 could respond to the trigger; the next time P1 had priority, after P0
        // decided, P0 had discarded and drawn one (one player attacked).
        assert!(
            prio.iter()
                .any(|&(s, stack, _)| s < decided[0] && stack == 1),
            "{prio:?}"
        );
        let after: Vec<_> = prio.iter().filter(|(s, ..)| *s > decided[0]).collect();
        assert!(!after.is_empty(), "{prio:?}");
        assert!(
            after
                .iter()
                .all(|&&(_, stack, hand)| stack == 0 && hand == 1),
            "{in_hand}: {prio:?}"
        );
        assert!(!t.in_hand(P0, "Grizzly Bears"));
        assert_eq!(t.graveyard_size(P0), in_hand);
    }
}

#[test]
fn donatello_applies_to_tokens_you_create_whoever_controls_the_effect() {
    cr!("614.1a", "111.2");
    ruling!(
        "Donatello, the Brains",
        "You don't need to control the spell or ability that creates the tokens, but you do have to be the one creating the tokens for Donatello's first ability to apply."
    );
    supported("Donatello, the Brains");
    // P1's Forbidden Orchard makes P0 create a Spirit: P0 also gets a Mutagen.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Donatello, the Brains");
    let orchard = t.battlefield(P1, "Forbidden Orchard");
    t.answer_targets(P1, &[pl(P0)]);
    t.activate(P1, orchard, 0, &[]).expect("tap for mana");
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Spirit"), 1);
    assert_eq!(tokens_of(&t, P0, "Mutagen"), 1);
    assert!(tokens(&t, P1).is_empty());
    // P0's Hunted Horror makes P1 create the Centaurs: no Mutagen.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Donatello, the Brains");
    t.answer_targets(P0, &[pl(P1)]);
    t.enter(P0, "Hunted Horror");
    t.resolve_all();
    assert_eq!(tokens_of(&t, P1, "Centaur"), 2);
    assert_eq!(tokens(&t, P1).len(), 2);
    assert!(tokens(&t, P0).is_empty());
}

/// Silas Renn deals combat damage to P1, choosing the artifact card `card` in P0's
/// graveyard; returns that card.
fn silas(t: &mut TestGame, card: &str) -> (ObjectId, ObjectId) {
    let silas = t.battlefield(P0, "Silas Renn, Seeker Adept");
    let c = t.graveyard(P0, card);
    t.answer_targets(P0, &[obj(c)]);
    t.attack(&[(silas, pl(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    (silas, c)
}

#[test]
fn silas_renns_permission_outlasts_it_and_allows_alternative_costs() {
    cr!("601.2b", "601.2f", "118.9");
    ruling!(
        "Silas Renn, Seeker Adept",
        "You may cast that card this turn even if Silas Renn leaves the battlefield."
    );
    ruling!(
        "Silas Renn, Seeker Adept",
        "You pay the costs for the target artifact card if you cast it. You may pay alternative costs rather than the card's mana cost."
    );
    supported("Silas Renn, Seeker Adept");
    // Silas dies; Mind Stone is cast from the graveyard for {2}.
    let mut t = TestGame::new(2);
    let (silas_id, stone) = silas(&mut t, "Mind Stone");
    t.set_step(P0, Step::PostcombatMain);
    kill(&mut t, silas_id);
    t.g.turn.priority = Some(P0);
    assert!(
        t.g.cast_spell(P0, stone, CastMethod::Normal).is_err(),
        "unpaid"
    );
    add_mana(&mut t, P0, ManaType::C, 2);
    t.g.turn.priority = Some(P0);
    t.g.cast_spell(P0, stone, CastMethod::Normal)
        .expect("cast from the graveyard");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Mind Stone").len(), 1);
    // Mistvein Borderpost by its alternative cost: {1} and returning a basic land.
    let mut t = TestGame::new(2);
    let (_, post) = silas(&mut t, "Mistvein Borderpost");
    t.set_step(P0, Step::PostcombatMain);
    let island = t.battlefield(P0, "Island");
    add_mana(&mut t, P0, ManaType::C, 1);
    t.answer_choose(P0, &[obj(island)]);
    t.g.turn.priority = Some(P0);
    let alt =
        t.g.cast_options(P0, post)
            .into_iter()
            .map(|o| o.method)
            .find(|m| matches!(m, CastMethod::Alternative(_)))
            .expect("the alternative cost is offered from the graveyard");
    t.g.cast_spell(P0, post, alt).expect("alternative cost");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Mistvein Borderpost").len(), 1);
    assert!(t.in_hand(P0, "Island"));
}

#[test]
fn glacian_can_tap_summoning_sick_artifact_creatures() {
    cr!("302.6");
    ruling!(
        "Glacian, Powerstone Engineer",
        "You may tap any untapped artifacts you control, including artifact creatures that haven't been under your control continuously since the beginning of your most recent turn."
    );
    supported("Glacian, Powerstone Engineer");
    let mut t = TestGame::new(2);
    let glacian = t.battlefield(P0, "Glacian, Powerstone Engineer");
    let thopter = t.battlefield_sick(P0, "Ornithopter");
    let top = t.library_top(P0, "Hill Giant");
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.answer_choose(P0, &[obj(thopter)]);
    t.answer_choose(P0, &[obj(top)]);
    t.activate(P0, glacian, 0, &[]).expect("activate");
    t.resolve_all();
    assert!(t.obj_now(thopter).tapped);
    assert!(t.in_hand(P0, "Hill Giant"));
}

#[test]
fn yasmin_khans_card_follows_normal_costs_and_timing() {
    cr!("305.1", "601.2f", "601.3");
    ruling!(
        "Yasmin Khan",
        "You must pay all costs and follow all normal timing rules when playing a card using Yasmin Khan's first ability. For example, you may play a land only during your main phase while the stack is empty."
    );
    supported("Yasmin Khan");
    let mut t = TestGame::new(2);
    let yasmin = t.battlefield(P0, "Yasmin Khan");
    t.library_top(P0, "Grizzly Bears");
    let forest = t.library_top(P0, "Forest");
    t.activate(P0, yasmin, 0, &[]).expect("activate");
    t.resolve_all();
    let forest = t.g.current(forest);
    assert_eq!(t.zone(forest), Zone::Exile);
    // In combat, or with a spell on the stack: no land play.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play_land(&mut t, P0, forest));
    t.set_step(P0, Step::PostcombatMain);
    cast_at(&mut t, P0, "Shock", &[pl(P1)]);
    assert!(!can_play_land(&mut t, P0, forest));
    t.resolve_all();
    assert!(can_play_land(&mut t, P0, forest));
    t.play_land(P0, forest).expect("play the land");
    // A spell exiled this way still costs its mana.
    t.g.objects[yasmin.0 as usize].tapped = false;
    let bears = t.library_top(P0, "Grizzly Bears");
    t.activate(P0, yasmin, 0, &[]).expect("activate");
    t.resolve_all();
    let bears = t.g.current(bears);
    assert_eq!(t.zone(bears), Zone::Exile);
    t.g.turn.priority = Some(P0);
    t.g.players[P0.idx()].mana_pool = Default::default();
    for l in t.g.battlefield.clone() {
        t.g.objects[l.0 as usize].tapped = true;
    }
    assert!(t.g.cast_spell(P0, bears, CastMethod::Normal).is_err());
    add_mana(&mut t, P0, ManaType::G, 2);
    t.g.cast_spell(P0, bears, CastMethod::Normal)
        .expect("cast for {1}{G}");
}

#[test]
fn dragon_cultist_needs_control_only_when_the_damage_was_dealt() {
    cr!("603.4");
    ruling!(
        "Dragon Cultist",
        "You need to control the source that dealt damage only at the time that the damage was dealt. If it dies or changes control after that point, this ability will still trigger at the beginning of your end step as long as you control a commander creature with the ability."
    );
    supported("Dragon Cultist");
    for after in ["dies", "changes control", "nothing"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Dragon Cultist");
        let cmdr = t.battlefield(P0, "Grizzly Bears");
        t.g.objects[cmdr.0 as usize].is_commander = true;
        t.g.dirty = true;
        let wurm = t.battlefield(P0, "Craw Wurm");
        t.attack(&[(wurm, pl(P1))], &[]);
        assert_eq!(t.life(P1), 14);
        match after {
            "dies" => kill(&mut t, wurm),
            "changes control" => give_control(&mut t, wurm, P1),
            _ => {}
        }
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(tokens_of(&t, P0, "Dragon"), 1, "{after}");
    }
    // Without 5 damage from one source: no Dragon.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dragon Cultist");
    let cmdr = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[cmdr.0 as usize].is_commander = true;
    t.g.dirty = true;
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Dragon"), 0);
}

#[test]
fn jaheiras_creature_tokens_need_haste_noncreature_tokens_dont() {
    cr!("302.6", "605.1a");
    ruling!(
        "Jaheira, Friend of the Forest",
        "You won't be able to activate the mana abilities of creature tokens you control on the turn they're created unless they have haste. Conversely, you will be able to activate the mana abilities of noncreature tokens you control on the turn they're created (provided they're untapped)."
    );
    supported("Jaheira, Friend of the Forest");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jaheira, Friend of the Forest");
    // Real token creation: Raise the Alarm's two Soldiers, Strike It Rich's Treasure.
    cast_at(&mut t, P0, "Raise the Alarm", &[]);
    t.resolve_all();
    cast_at(&mut t, P0, "Strike It Rich", &[]);
    t.resolve_all();
    t.g.players[P0.idx()].mana_pool = Default::default();
    let soldier = with_subtype(&t, P0, "Soldier")[0];
    let treasure = with_subtype(&t, P0, "Treasure")[0];
    assert!(activate_containing(&mut t, P0, soldier, "Add {G}").is_err());
    assert!(!t.obj_now(soldier).tapped);
    activate_containing(&mut t, P0, treasure, "Add {G}").expect("Treasure taps");
    assert!(t.obj_now(treasure).tapped);
    assert!(t.on_battlefield(treasure));
}

#[test]
fn wicker_pickers_sticker_kicker_cant_dodge_a_cast_prohibition() {
    cr!("101.2", "601.3", "702.33h");
    ruling!(
        "Wicker Picker",
        "Conversely, you can't use sticker kicker to change a spell's name to get around effects that prohibit that spell from being cast"
    );
    supported("Wicker Picker");
    supported("Meddling Mage");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wicker Picker");
    t.answer(P1, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
    t.enter(P1, "Meddling Mage");
    t.settle();
    let bears = t.hand(P0, "Grizzly Bears");
    lots(&mut t, P0);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.g.turn.priority = Some(P0);
    assert!(t.g.cast_spell(P0, bears, CastMethod::Normal).is_err());
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // Another creature spell can be cast with sticker kicker.
    let giant = t.hand(P0, "Hill Giant");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.g.cast_spell(P0, giant, CastMethod::Normal)
        .expect("Hill Giant");
}
