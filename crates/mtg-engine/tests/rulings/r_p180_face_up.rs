//! Rulings batch P180 — "when turned face up" and "whenever a permanent is turned face
//! up" triggers (CR 708.8, 702.37e, 702.168): Secret Plans, Sumala Sentry, Aphetto
//! Runecaster, Aven Farseer, Bonethorn Valesk, Pine Walker, Pyrotechnic Performer, Ire
//! Shaman, Concealed Weapon, Forum Familiar, Master of the Veil, Skirk Alarmist and
//! Karona's Zealot.

use crate::r_p180_common::*;
use crate::r_s06_common::{attach_new, damage};
use crate::r_s11_common::triggered_from;
use mtg_engine::decision::Decision;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn secret_plans_bonus_ends_as_it_turns_face_up_and_lethal_damage_destroys_it() {
    cr!("708.8", "704.5g");
    ruling!(
        "Secret Plans",
        "As soon as a creature turns face up, it loses the +0/+1 from Secret Plans."
    );
    supported("Secret Plans");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Secret Plans");
    let pinger = t.battlefield(P1, "Grizzly Bears");
    // Willbender: a 1/2 with morph {1}{U}.
    let w = morph(&mut t, P0, "Willbender");
    assert_eq!(t.pt(w), (2, 3));
    damage(&mut t, pinger, 2, w);
    assert!(t.on_battlefield(w));
    let hand = t.hand_size(P0);
    turn_up(&mut t, P0, w, &[("Island", 2)]);
    assert!(t.in_graveyard(P0, "Willbender"), "1/2 with 2 damage");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1, "Secret Plans still draws");
}

#[test]
fn sumala_sentry_puts_each_counter_on_whatever_is_still_there() {
    cr!("608.2c");
    ruling!(
        "Sumala Sentry",
        "If the permanent that was turned face up is no longer on the battlefield when Sumala Sentry's triggered ability resolves, you'll still put a +1/+1 counter on Sumala Sentry."
    );
    supported("Sumala Sentry");
    for gone in ["turned up", "Sentry"] {
        let mut t = TestGame::new(2);
        let sentry = t.battlefield(P0, "Sumala Sentry");
        let w = morph(&mut t, P0, "Willbender");
        turn_up(&mut t, P0, w, &[("Island", 2)]);
        assert_eq!(triggered_from(&t, sentry), 1);
        if gone == "Sentry" {
            destroy(&mut t, sentry);
        } else {
            destroy(&mut t, w);
        }
        t.resolve_all();
        if gone == "Sentry" {
            assert_eq!(plus(&t, w), 1);
        } else {
            assert_eq!(plus(&t, sentry), 1);
        }
    }
}

#[test]
fn turned_face_up_triggers_include_noncreature_permanents() {
    cr!("708.8", "702.168d");
    ruling!(
        "Aphetto Runecaster",
        "Triggers when any permanent is turned face up, not just a creature."
    );
    ruling!(
        "Aven Farseer",
        "Triggers when any permanent is turned face up, not just a creature."
    );
    ruling!(
        "Bonethorn Valesk",
        "Triggers when any permanent is turned face up, not just a creature."
    );
    for name in ["Aphetto Runecaster", "Aven Farseer", "Bonethorn Valesk"] {
        supported(name);
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, name);
        // Concealed Weapon: a disguised Equipment, not a creature once face up.
        let weapon = disguise(&mut t, P0, "Concealed Weapon");
        let hand = t.hand_size(P0);
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        turn_up(&mut t, P0, weapon, &[("Mountain", 1), ("Wastes", 2)]);
        assert!(!t.obj_now(weapon).is(mtg_engine::types::CardType::Creature));
        assert_eq!(triggered_from(&t, src), 1, "{name}");
        t.resolve_all();
        match name {
            "Aphetto Runecaster" => assert_eq!(t.hand_size(P0), hand + 1),
            "Aven Farseer" => assert_eq!(plus(&t, src), 1),
            _ => assert_eq!(t.life(P1), 19),
        }
    }
}

#[test]
fn pine_walker_doesnt_trigger_for_a_noncreature_turned_face_up() {
    cr!("708.8");
    ruling!(
        "Pine Walker",
        "If a face-down creature other than Pine Walker is turned face up and the resulting permanent isn't a creature, Pine Walker's last ability won't trigger."
    );
    supported("Pine Walker");
    let mut t = TestGame::new(2);
    let pine = t.battlefield(P0, "Pine Walker");
    let weapon = disguise(&mut t, P0, "Concealed Weapon");
    t.g.tap(weapon);
    turn_up(&mut t, P0, weapon, &[("Mountain", 1), ("Wastes", 2)]);
    assert_eq!(triggered_from(&t, pine), 0);
    t.resolve_all();
    assert!(t.obj_now(weapon).tapped);
    // A creature turned face up: it's untapped.
    let w = morph(&mut t, P0, "Willbender");
    t.g.tap(w);
    turn_up(&mut t, P0, w, &[("Island", 2)]);
    assert_eq!(triggered_from(&t, pine), 1);
    t.resolve_all();
    assert!(!t.obj_now(w).tapped);
}

#[test]
fn pyrotechnic_performer_uses_the_power_the_creature_last_had() {
    cr!("608.2h", "702.37b");
    ruling!(
        "Pyrotechnic Performer",
        "If a creature that is turned face up leaves the battlefield before Pyrotechnic Performer's triggered ability resolves, use that creature's power as it last existed on the battlefield"
    );
    supported("Pyrotechnic Performer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pyrotechnic Performer");
    // Ire Shaman: a 2/1 with megamorph {R}: a 3/2 once turned face up.
    let shaman = morph(&mut t, P0, "Ire Shaman");
    turn_up(&mut t, P0, shaman, &[("Mountain", 1)]);
    assert_eq!(t.pt(shaman), (3, 2));
    destroy(&mut t, shaman);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn ire_shaman_turned_face_up_while_blocked_stays_blocked() {
    cr!("509.1h", "702.111b");
    ruling!(
        "Ire Shaman",
        "If a face-down Ire Shaman attacks and is blocked by one creature, turning it face up won't cause it to become unblocked."
    );
    supported("Ire Shaman");
    let mut t = TestGame::new(2);
    let shaman = morph(&mut t, P0, "Ire Shaman");
    t.g.objects[shaman.0 as usize].summoning_sick = false;
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s03_common::to_blockers(&mut t, &[(shaman, Entity::Player(P1))], &[(bears, shaman)]);
    turn_up(&mut t, P0, shaman, &[("Mountain", 1)]);
    assert!(t.obj_now(shaman).has_keyword(mtg_engine::keywords::KeywordKind::Menace));
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "3 damage to the blocker");
}

#[test]
fn ire_shamans_exiled_card_is_face_up_and_played_with_normal_timing() {
    cr!("307.1", "305.2");
    ruling!(
        "Ire Shaman",
        "Playing the card exiled with Ire Shaman's last ability follows the normal rules for playing the card."
    );
    ruling!(
        "Ire Shaman",
        "The card exiled with Ire Shaman's last ability is exiled face up."
    );
    ruling!(
        "Ire Shaman",
        "Under normal circumstances, you can play a land card exiled with Ire Shaman only if you haven't played a land yet that turn."
    );
    // A creature card: only at sorcery speed.
    let mut t = TestGame::new(2);
    let shaman = morph(&mut t, P0, "Ire Shaman");
    t.set_step(P0, Step::BeginningOfCombat);
    let bears = t.library_top(P0, "Grizzly Bears");
    turn_up(&mut t, P0, shaman, &[("Mountain", 1)]);
    t.resolve_all();
    let bears = t.g.current(bears);
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(!t.obj(bears).face_down, "exiled face up");
    t.lands(P0, "Forest", 2);
    assert!(!crate::r_s02_common::can_cast(&mut t, P0, bears, CastMethod::Normal));
    t.set_step(P0, Step::PostcombatMain);
    assert!(crate::r_s02_common::can_cast(&mut t, P0, bears, CastMethod::Normal));
    // A land card: only if no land was played this turn.
    for played in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let shaman = morph(&mut t, P0, "Ire Shaman");
        let forest = t.library_top(P0, "Forest");
        if played {
            let island = t.hand(P0, "Island");
            t.play_land(P0, island).unwrap();
        }
        turn_up(&mut t, P0, shaman, &[("Mountain", 1)]);
        t.resolve_all();
        let forest = t.g.current(forest);
        assert_eq!(t.zone(forest), Zone::Exile);
        assert_eq!(
            crate::r_s02_common::can_play_land(&mut t, P0, forest),
            !played
        );
    }
}

#[test]
fn concealed_weapon_turns_up_without_a_creature_and_stays_unattached() {
    cr!("608.2b", "603.3d", "702.168d");
    ruling!(
        "Concealed Weapon",
        "You may turn Concealed Weapon face up even if you don't control any creatures."
    );
    ruling!(
        "Concealed Weapon",
        "If the target creature becomes an illegal target, Concealed Weapon remains on the battlefield unattached."
    );
    supported("Concealed Weapon");
    let mut t = TestGame::new(2);
    let weapon = disguise(&mut t, P0, "Concealed Weapon");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    assert!(can_turn_face_up(&mut t, P0, weapon));
    assert!(turn_face_up(&mut t, P0, weapon));
    t.settle();
    assert_eq!(t.stack_len(), 0, "no creature to target");
    assert!(t.on_battlefield(weapon));
    assert_eq!(t.obj_now(weapon).attached_to, None);
    // Its target leaves: it stays unattached.
    let mut t = TestGame::new(2);
    let weapon = disguise(&mut t, P0, "Concealed Weapon");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    turn_up(&mut t, P0, weapon, &[("Mountain", 1), ("Wastes", 2)]);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.on_battlefield(weapon));
    assert_eq!(t.obj_now(weapon).attached_to, None);
}

#[test]
fn forum_familiar_with_an_illegal_target_gets_no_counter() {
    cr!("608.2b");
    ruling!(
        "Forum Familiar",
        "If the target of Forum Familiar's last ability is illegal when the ability tries to resolve, it won't resolve"
    );
    supported("Forum Familiar");
    for leave in [false, true] {
        let mut t = TestGame::new(2);
        let fam = disguise(&mut t, P0, "Forum Familiar");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        turn_up(&mut t, P0, fam, &[("Plains", 1), ("Wastes", 1)]);
        if leave {
            destroy(&mut t, bears);
        }
        t.resolve_all();
        assert_eq!(plus(&t, fam), u32::from(!leave), "{leave}");
        if !leave {
            assert!(t.in_hand(P0, "Grizzly Bears"));
        }
    }
}

#[test]
fn master_of_the_veil_can_turn_itself_face_down() {
    cr!("702.37a", "708.2a");
    ruling!(
        "Master of the Veil",
        "It can target itself and turn itself face down."
    );
    supported("Master of the Veil");
    let mut t = TestGame::new(2);
    let master = morph(&mut t, P0, "Master of the Veil");
    t.answer_targets(P0, &[obj(master)]);
    t.answer_yes(P0, true);
    turn_up(&mut t, P0, master, &[("Island", 1), ("Wastes", 2)]);
    assert!(!t.obj_now(master).face_down);
    t.resolve_all();
    assert!(t.obj_now(master).face_down);
    assert_eq!(t.pt(master), (2, 2));
}

#[test]
fn skirk_alarmist_used_in_the_end_step_sacrifices_at_the_next_turns_end() {
    cr!("603.7b");
    ruling!(
        "Skirk Alarmist",
        "If this ability is used during the end step, the creature is not sacrificed until the end of the next turn."
    );
    supported("Skirk Alarmist");
    let mut t = TestGame::new(2);
    let alarmist = t.battlefield(P0, "Skirk Alarmist");
    let w = morph(&mut t, P0, "Willbender");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    t.answer_targets(P0, &[obj(w)]);
    crate::r_s06_common::activate_containing(&mut t, P0, alarmist, "Turn target").unwrap();
    t.resolve_all();
    assert!(!t.obj_now(w).face_down);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(w), "not sacrificed this turn");
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Willbender"));
}

#[test]
fn karonas_zealot_stops_redirecting_once_the_creature_is_gone() {
    cr!("614.9");
    ruling!(
        "Karona's Zealot",
        "If the target creature leaves the battlefield or stops being a creature before damage would be dealt, that damage is not redirected."
    );
    supported("Karona's Zealot");
    let mut t = TestGame::new(2);
    let src = t.battlefield(P1, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let zealot = morph(&mut t, P0, "Karona's Zealot");
    t.answer_targets(P0, &[obj(elves)]);
    turn_up(&mut t, P0, zealot, &[("Plains", 2), ("Wastes", 3)]);
    t.resolve_all();
    damage(&mut t, src, 3, zealot);
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    assert_eq!(damage_on(&t, zealot), 0);
    damage(&mut t, src, 3, zealot);
    assert_eq!(damage_on(&t, zealot), 3);
    assert!(t.on_battlefield(zealot));
}
