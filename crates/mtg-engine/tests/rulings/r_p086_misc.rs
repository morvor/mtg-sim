//! Rulings batch P086 — the rest: Burning Earth, Pestilent Haze, Ivory Guardians, Damping
//! Matrix, Goatnap, Homing Lightning, Guildscorn Ward, Death's Caress, Break the Spell,
//! Draconian Gate-Bot and Down for Repairs, Hour of Glory, Light of Judgment, Ringwraiths,
//! Aim for the Head, Slayer of the Wicked, Viridian Revel, Magnetic Mine and Carry Away.

use crate::r_p086_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn burning_earth_tapping_a_land_for_mana() {
    cr!("106.12", "605.1a");
    ruling!(
        "Burning Earth",
        "To tap a land for mana is to activate one of that land’s mana abilities that has {T} in its activation cost and produces mana as it resolves."
    );
    supported("Burning Earth");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Burning Earth");
    // Tapping a basic land for mana: no damage.
    let mountain = t.battlefield(P0, "Mountain");
    assert!(crate::r_s20_common::tap_for_mana(
        &mut t, P0, mountain, "Add"
    ));
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // Mishra's Factory's "{T}: Target Assembly-Worker creature gets +1/+1 until end of
    // turn." taps it, but not for mana: no damage.
    let factory = t.battlefield(P0, "Mishra's Factory");
    let worker = t.battlefield(P0, "Mishra's Factory");
    add_mana(&mut t, P0, mtg_engine::mana::ManaType::C, 1);
    activate(&mut t, P0, worker, "becomes", &[]);
    t.resolve_all();
    activate(&mut t, P0, factory, "gets +1/+1", &[obj(worker)]);
    t.resolve_all();
    assert!(is_tapped(&t, factory));
    assert_eq!(t.life(P0), 20);
    // Tapped for mana: 1 damage.
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P0, worker, "Add"));
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
}

#[test]
fn pestilent_haze_doesnt_affect_creatures_entering_later() {
    cr!("611.2c", "613.4c");
    ruling!(
        "Pestilent Haze",
        "Pestilent Haze's first mode affects only creatures on the battlefield at the time it resolves. Creatures that enter the battlefield later in the turn won't get -2/-2."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    lands_for_cost(&mut t, P0, "Pestilent Haze");
    let card = t.hand(P0, "Pestilent Haze");
    t.cast(P0, card).modes(&[0]).go();
    t.resolve_all();
    assert_eq!(t.pt(wurm), (4, 2));
    let late = t.enter(P1, "Grizzly Bears");
    t.settle();
    assert_eq!(t.pt(late), (2, 2));
}

#[test]
fn ivory_guardians_bonus_cases() {
    cr!("613.4c", "201.2", "611.3a");
    ruling!(
        "Ivory Guardians",
        "Each ability applies a maximum of once, even if an opponent controls multiple nontoken red permanents or multiple opponents each control a nontoken red permanent."
    );
    ruling!(
        "Ivory Guardians",
        "The abilities from multiple Ivory Guardians are cumulative. For example, if you control two different Ivory Guardians and an opponent controls a nontoken red permanent, each of your Ivory Guardians will be 5/5."
    );
    ruling!(
        "Ivory Guardians",
        "The ability of an Ivory Guardians will affect itself as well as all other creatures named Ivory Guardians (not just the ones you control)."
    );
    supported("Ivory Guardians");
    // One Guardians, several red permanents among several opponents: 4/4.
    let mut t = TestGame::new(3);
    let g = t.battlefield(P0, "Ivory Guardians");
    t.battlefield(P1, "Raging Goblin");
    t.battlefield(P1, "Raging Goblin");
    t.battlefield(P2, "Raging Goblin");
    t.g.recompute();
    assert_eq!(t.pt(g), (4, 4));
    // Red tokens don't count (Dragon Fodder: two 1/1 red Goblin tokens).
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Ivory Guardians");
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Dragon Fodder", &[]);
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tokens(&t, P1).len(), 2);
    assert_eq!(t.pt(g), (3, 3));
    // Two Guardians: each 5/5.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ivory Guardians");
    let b = t.battlefield(P0, "Ivory Guardians");
    t.battlefield(P1, "Raging Goblin");
    t.g.recompute();
    assert_eq!(t.pt(a), (5, 5));
    assert_eq!(t.pt(b), (5, 5));
    // P0's Guardians also pumps P1's (P1's ability doesn't apply: P0 has no red permanent).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ivory Guardians");
    let theirs = t.battlefield(P1, "Ivory Guardians");
    t.battlefield(P1, "Raging Goblin");
    t.g.recompute();
    assert_eq!(t.pt(theirs), (4, 4));
}

#[test]
fn damping_matrix_stops_activated_abilities_of_artifacts_and_creatures_only() {
    cr!("602.1", "602.5", "605.1a", "702.6a");
    ruling!(
        "Damping Matrix",
        "Activated abilities contain a colon. They’re generally written “[Cost]: [Effect].” Some keyword abilities, such as equip, are activated abilities and will have colons in their reminder text."
    );
    ruling!(
        "Damping Matrix",
        "Damping Matrix’s ability affects only artifacts and creatures on the battlefield. Activated abilities that work in other zones (such as bloodrush or unearth) can still be activated. Triggered abilities (starting with “when,” “whenever,” or “at”) are unaffected."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Damping Matrix");
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let scimitar = t.battlefield(P0, "Leonin Scimitar");
    t.lands(P0, "Wastes", 3);
    // A creature's ability and equip.
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, sorcerer));
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, scimitar));
    // A mana ability can.
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P0, elves, "Add"));
    // Unearth from the graveyard can.
    let zombie = t.graveyard(P0, "Dregscape Zombie");
    t.lands(P0, "Swamp", 1);
    assert!(crate::r_s02_common::can_activate(&mut t, P0, zombie));
    activate(&mut t, P0, zombie, "Unearth", &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Dregscape Zombie").len(), 1);
    // A triggered ability of a creature still triggers (Ripjaw Raptor's enrage).
    let raptor = t.battlefield(P0, "Ripjaw Raptor");
    cast_new(&mut t, P1, "Shock", &[obj(raptor)]);
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn goatnap_changelings_are_goats_any_creature_can_be_targeted() {
    cr!("702.73a", "115.1");
    ruling!(
        "Goatnap",
        "Goat is a creature type. Keep in mind that creatures with the changeling ability are Goats. (Whether they're \"goated\" or not is too subjective for this ruling to address.)"
    );
    ruling!(
        "Goatnap",
        "Goat is a creature type. You won't see this creature type printed on creatures in this set, but keep in mind that creatures with the changeling ability are Goats."
    );
    ruling!(
        "Goatnap",
        "Goatnap can target any creature, even one that's untapped or one you already control."
    );
    supported("Goatnap");
    let mut t = TestGame::new(2);
    let changeling = t.battlefield(P1, "Woodland Changeling");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let offered = spell_targets(&mut t, P0, "Goatnap");
    assert!(offered.contains(&obj(changeling)) && offered.contains(&obj(mine)));
    cast_new(&mut t, P0, "Goatnap", &[obj(changeling)]);
    t.resolve_all();
    assert_eq!(controller(&t, changeling), P0);
    assert_eq!(t.pt(changeling), (5, 2));
    assert!(crate::r_s06_common::has_kw(
        &t,
        changeling,
        KeywordKind::Haste
    ));
    // A creature P0 already controls (not a Goat): no bonus.
    cast_new(&mut t, P0, "Goatnap", &[obj(mine)]);
    t.resolve_all();
    assert_eq!(controller(&t, mine), P0);
    assert_eq!(t.pt(mine), (2, 2));
}

#[test]
fn homing_lightning_damages_a_same_named_hexproof_creature() {
    cr!("115.1", "702.11b", "201.2");
    ruling!(
        "Homing Lightning",
        "Homing Lightning has only one target. Other creatures with that name are not targeted. For example, a creature with hexproof will be dealt damage if it has the same name as the target creature."
    );
    supported("Homing Lightning");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Gladecover Scout");
    let theirs = t.battlefield(P1, "Gladecover Scout");
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(!spell_targets(&mut t, P0, "Homing Lightning").contains(&obj(theirs)));
    let spell = cast_new(&mut t, P0, "Homing Lightning", &[obj(mine)]);
    assert_eq!(crate::r_s25_common::targets_of(&t, spell), vec![obj(mine)]);
    t.resolve_all();
    assert!(!t.on_battlefield(mine));
    assert!(!t.on_battlefield(theirs));
    assert!(t.on_battlefield(bears));
}

#[test]
fn guildscorn_ward_hybrid_is_multicolored_whatever_mana_paid() {
    cr!("105.2", "202.2c", "702.16");
    ruling!(
        "Guildscorn Ward",
        "Hybrid spells and permanents in the Return to Ravnica block are multicolored, even if you cast them with one color of mana."
    );
    // Boros Reckoner ({R/W}{R/W}{R/W}) cast with red mana only.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Guildscorn Ward", obj(bears));
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 3);
    let card = t.hand(P1, "Boros Reckoner");
    t.cast(P1, card).go();
    t.resolve_all();
    let reckoner = t.g.current(card);
    assert!(t.on_battlefield(reckoner));
    t.set_step(P0, Step::PrecombatMain);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(reckoner, bears)]));
}

#[test]
fn deaths_caress_checks_human_even_if_not_destroyed_and_not_with_an_illegal_target() {
    cr!("608.2b", "608.2h");
    ruling!(
        "Death's Caress",
        "If Death’s Caress resolves but the target creature isn’t destroyed, perhaps because it has indestructible or it regenerated, you still check if it’s a Human. If so, you’ll gain life equal to its toughness."
    );
    ruling!(
        "Death's Caress",
        "If the target creature isn’t on the battlefield or is otherwise an illegal target when Death’s Caress tries to resolve, Death’s Caress doesn’t resolve and none of its effects will happen. You won’t gain any life."
    );
    // A Human given a regeneration shield (Regenerate): not destroyed, still life.
    let mut t = TestGame::new(2);
    let human = t.battlefield(P1, "Holy Justiciar");
    cast_new(&mut t, P0, "Death's Caress", &[obj(human)]);
    cast_new(&mut t, P1, "Regenerate", &[obj(human)]);
    t.resolve_all();
    assert!(t.on_battlefield(human));
    assert_eq!(t.life(P0), 21);
    // Illegal target: nothing.
    let mut t = TestGame::new(2);
    let human = t.battlefield(P1, "Holy Justiciar");
    cast_new(&mut t, P0, "Death's Caress", &[obj(human)]);
    cast_new(&mut t, P1, "Unsummon", &[obj(human)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn break_the_spell_draws_by_destruction_not_by_graveyard() {
    cr!("701.8a", "614.6", "702.12b");
    ruling!(
        "Break the Spell",
        "If a permanent you controlled or a token is destroyed with Break the Spell but is put into a zone other than a graveyard as a result, you will draw a card."
    );
    ruling!(
        "Break the Spell",
        "If the enchantment has indestructible, it won't be destroyed. You won't draw a card."
    );
    supported("Break the Spell");
    // P0's own enchantment, destroyed with Rest in Peace out: exiled instead, still draws.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Rest in Peace");
    let ward = t.battlefield(P0, "Burning Earth");
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Break the Spell", &[obj(ward)]);
    t.resolve_all();
    assert!(t.in_exile("Burning Earth"));
    assert_eq!(t.hand_size(P0), hand + 1);
    // An opponent's token (a Shard), exiled instead too: still draws; an opponent's
    // nontoken enchantment doesn't.
    for token in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Rest in Peace");
        let target = if token {
            create_token(&mut t, P1, "Shard")
        } else {
            t.battlefield(P1, "Burning Earth")
        };
        let hand = t.hand_size(P0);
        cast_new(&mut t, P0, "Break the Spell", &[obj(target)]);
        t.resolve_all();
        assert!(!t.on_battlefield(target), "token={token}");
        assert_eq!(t.hand_size(P0), hand + usize::from(token), "token={token}");
    }
    // An indestructible enchantment (Avacyn): not destroyed, no card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Avacyn, Angel of Hope");
    let ward = t.battlefield(P0, "Burning Earth");
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Break the Spell", &[obj(ward)]);
    t.resolve_all();
    assert!(t.on_battlefield(ward));
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn a_copy_of_an_attraction_goes_to_the_graveyard() {
    cr!("717.6", "707.2");
    ruling!(
        "Draconian Gate-Bot",
        "If an Attraction that is not normally an Attraction (probably because it's a copy of an Attraction) is destroyed, it will go to its owner's graveyard as normal, not their junkyard."
    );
    ruling!(
        "Down for Repairs",
        "If an Attraction that is not normally an Attraction (probably because it’s a copy of an Attraction) is destroyed, it will go to its owner’s graveyard as normal, not their junkyard."
    );
    for name in ["Draconian Gate-Bot", "Down for Repairs"] {
        let mut t = TestGame::new(2);
        let stand = t.battlefield(P1, "Balloon Stand");
        // Sculpting Steel entering as a copy of Balloon Stand.
        t.answer_yes(P1, true);
        t.answer_choose(P1, &[obj(stand)]);
        let steel = t.enter(P1, "Sculpting Steel");
        t.settle();
        assert!(t.obj_now(steel).chars.has_subtype("Attraction"), "{name}");
        if name == "Draconian Gate-Bot" {
            t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
            t.answer_targets(P0, &[obj(steel)]);
            t.enter(P0, name);
        } else {
            t.hand(P1, "Grizzly Bears");
            lands_for_cost(&mut t, P0, name);
            let card = t.hand(P0, name);
            t.cast_with(P0, card, &[Entity::Player(P1), obj(steel)])
                .expect("cast");
        }
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Sculpting Steel"), "{name}");
        // The real Attraction goes to the junkyard.
        destroy(&mut t, stand);
        assert!(!t.in_graveyard(P1, "Balloon Stand"), "{name}");
        assert_eq!(
            mtg_engine::attraction_cards::junkyard(&t.g, P1).len(),
            1,
            "{name}"
        );
    }
}

#[test]
fn removal_with_an_illegal_target_does_none_of_its_other_effects() {
    cr!("608.2b");
    ruling!(
        "Hour of Glory",
        "If the target creature is an illegal target by the time Hour of Glory resolves, the entire spell doesn’t resolve. No player reveals their hand or exiles cards from it."
    );
    ruling!(
        "Light of Judgment",
        "If the target creature is an illegal target when Light of Judgment tries to resolve, it won't resolve and none of its effects will happen. You won't destroy any Equipment."
    );
    ruling!(
        "Ringwraiths",
        "If the target is not legal as Ringwraiths's first ability tries to resolve, the ability is removed from the stack. The creature's controller won't lose life."
    );
    for name in ["Hour of Glory", "Light of Judgment", "Ringwraiths"] {
        supported(name);
    }
    // Hour of Glory at a God, which is bounced in response.
    let mut t = TestGame::new(2);
    let kefnet = t.battlefield(P1, "Kefnet the Mindful");
    t.hand(P1, "Kefnet the Mindful");
    cast_new(&mut t, P0, "Hour of Glory", &[obj(kefnet)]);
    cast_new(&mut t, P1, "Unsummon", &[obj(kefnet)]);
    t.resolve_all();
    assert_eq!(
        t.g.find_in_zone(Zone::Hand(P1), "Kefnet the Mindful").len(),
        2
    );
    // Light of Judgment at an equipped creature, which gains hexproof (Blossoming Defense)
    // in response: it's still there, equipped, and nothing happens to either.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let scimitar = attach_new(&mut t, P1, "Leonin Scimitar", obj(bears));
    cast_new(&mut t, P0, "Light of Judgment", &[obj(bears)]);
    cast_new(&mut t, P1, "Blossoming Defense", &[obj(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(damage_marked(&t, bears), 0);
    assert!(t.on_battlefield(scimitar));
    assert_eq!(attached_to(&t, scimitar), Some(obj(bears)));
    // Ringwraiths at a legendary creature, which is bounced in response.
    let mut t = TestGame::new(2);
    let avacyn = t.battlefield(P1, "Avacyn, Angel of Hope");
    t.answer_targets(P0, &[obj(avacyn)]);
    t.enter(P0, "Ringwraiths");
    t.settle();
    cast_new(&mut t, P1, "Unsummon", &[obj(avacyn)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn light_of_judgment_can_target_a_creature_without_equipment() {
    cr!("115.1", "608.2c");
    ruling!(
        "Light of Judgment",
        "Light of Judgment can target a creature that has no Equipment attached to it."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    assert!(spell_targets(&mut t, P0, "Light of Judgment").contains(&obj(wurm)));
    cast_new(&mut t, P0, "Light of Judgment", &[obj(wurm)]);
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
}

#[test]
fn aim_for_the_head_with_one_card_in_hand() {
    cr!("609.3", "700.2a");
    ruling!(
        "Aim for the Head",
        "If the target opponent only has one card in hand, they will exile that card."
    );
    supported("Aim for the Head");
    let mut t = TestGame::new(2);
    t.hand(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Aim for the Head");
    let card = t.hand(P0, "Aim for the Head");
    t.cast(P0, card).modes(&[1]).target(P1).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn slayer_of_the_wicked_must_target_your_own_only_zombie_but_may_decline() {
    cr!("603.3d", "603.5");
    ruling!(
        "Slayer of the Wicked",
        "If you control the only Vampire, Werewolf, or Zombie, you must target it with Slayer of the Wicked’s ability. You choose whether or not to destroy the target when the ability resolves."
    );
    let mut t = TestGame::new(2);
    let zombie = t.battlefield(P0, "Walking Corpse");
    t.answer_yes(P0, false);
    t.enter(P0, "Slayer of the Wicked");
    t.settle();
    let top = crate::r_s04_common::top_of_stack(&t);
    assert_eq!(crate::r_s25_common::targets_of(&t, top), vec![obj(zombie)]);
    t.resolve_all();
    assert!(t.on_battlefield(zombie));
}

#[test]
fn viridian_revel_checks_whose_graveyard_not_who_controlled_it() {
    cr!("400.3", "603.10a");
    ruling!(
        "Viridian Revel",
        "It doesn’t matter who controlled the artifact while it was on the battlefield, only whose graveyard it was put into."
    );
    supported("Viridian Revel");
    // P1's artifact P0 controls: it goes to P1's graveyard, P0 may draw.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Viridian Revel");
    let theirs = t.battlefield(P1, "Ornithopter");
    give_control(&mut t, theirs, P0);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    destroy(&mut t, theirs);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // P0's own artifact P1 controls: it goes to P0's graveyard, no trigger.
    let mine = t.battlefield(P0, "Ornithopter");
    give_control(&mut t, mine, P1);
    destroy(&mut t, mine);
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn magnetic_mine_triggers_for_another_artifact_dying_with_it() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Magnetic Mine",
        "Magnetic Mine will not trigger when it is put into a graveyard from the battlefield itself, although it will trigger if another artifact is put into the graveyard from the battlefield at the same time."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Magnetic Mine");
    t.battlefield(P1, "Ornithopter");
    cast_new(&mut t, P0, "Shatterstorm", &[]);
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn carry_away_unattaches_then_the_new_controller_can_equip() {
    cr!("301.5c", "702.6a", "603.2");
    ruling!(
        "Carry Away",
        "Normally, gaining control of an Equipment doesn't change what it's attached to. However, Carry Away has a triggered ability that unattaches the Equipment from whatever it's attached to. The Equipment's new controller can activate its equip ability to move it onto a creature they control."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let scimitar = attach_new(&mut t, P1, "Leonin Scimitar", obj(bears));
    let mine = t.battlefield(P0, "Craw Wurm");
    t.answer_targets(P0, &[obj(scimitar)]);
    cast_new(&mut t, P0, "Carry Away", &[]);
    t.resolve();
    // The Aura is on: P0 controls the Equipment, still attached until the trigger
    // resolves.
    assert_eq!(controller(&t, scimitar), P0);
    assert_eq!(attached_to(&t, scimitar), Some(obj(bears)));
    t.resolve_all();
    assert_eq!(attached_to(&t, scimitar), None);
    t.lands(P0, "Wastes", 1);
    activate(&mut t, P0, scimitar, "Equip", &[obj(mine)]);
    t.resolve_all();
    assert_eq!(attached_to(&t, scimitar), Some(obj(mine)));
}
