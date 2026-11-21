//! Rulings batch P095 — triggered abilities of the "life payment" cards: "you may pay
//! life" choices made as the ability resolves (CR 608.2c), reflexive triggers (CR 603.12),
//! leaves-the-battlefield triggers alongside enters triggers (CR 603.6, 603.10), the
//! attack triggers of Dreadhorde Invasion (CR 603.2), Herald of Torment's "you" as an
//! Aura (CR 109.5), delayed triggers that lose track of an object (CR 610.3), Meathook
//! Massacre II and tokens (CR 111.8), Lotho counting spells (CR 603.2), and Cursed
//! Recording's copy (CR 707.10).

use crate::r_p095_common::*;
use crate::r_s01_common::{supported, tokens, with_subtype};
use crate::r_s25_common::{spell_copies, targets_of};
use crate::r_s29_common::put_counters;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn yes_no_asked(t: &TestGame, from: usize, p: PlayerId) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, Decision::YesNo { .. }))
        .count()
}

// ---------------------------------------------------------------------------------------
// "You may pay N life" as the ability resolves
// ---------------------------------------------------------------------------------------

#[test]
fn arrogant_poet_pays_as_the_ability_resolves() {
    cr!("118.12", "603.5");
    ruling!(
        "Arrogant Poet",
        "You choose whether or not to pay 2 life as the ability resolves. Once you make that choice, no one can respond before Arrogant Poet gains flying."
    );
    supported("Arrogant Poet");
    let mut t = TestGame::new(2);
    let poet = t.battlefield(P0, "Arrogant Poet");
    crate::r_s01_common::attack_with(&mut t, &[(poet, Entity::Player(P1))]);
    assert_eq!(abilities_from(&t, poet), 1);
    // Not asked as it triggers.
    let from = t.asked().len();
    assert_eq!(yes_no_asked(&t, 0, P0), 0);
    t.answer_yes(P0, true);
    t.g.resolve_top();
    // Asked during the resolution, which also gave the flying: one step.
    assert_eq!(yes_no_asked(&t, from, P0), 1);
    assert_eq!(t.life(P0), 18);
    assert!(t.obj_now(poet).has_keyword(KeywordKind::Flying));
}

#[test]
fn sangrophage_may_pay_life_while_tapped() {
    cr!("118.12a", "701.26a");
    ruling!(
        "Sangrophage",
        "You choose whether or not to pay 2 life as the triggered ability resolves. You can pay 2 life even if Sangrophage is already tapped. (Doing so won't cause Sangrophage to become untapped, however.)"
    );
    supported("Sangrophage");
    let mut t = TestGame::new(2);
    let phage = t.battlefield(P0, "Sangrophage");
    to_upkeep(&mut t, P0);
    assert_eq!(abilities_from(&t, phage), 1);
    t.g.tap(phage);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(yes_no_asked(&t, from, P0), 1);
    assert_eq!(t.life(P0), 18);
    assert!(t.obj_now(phage).tapped);
}

#[test]
fn corpses_of_the_lost_returns_as_part_of_the_resolution() {
    cr!("118.12", "603.4");
    ruling!(
        "Corpses of the Lost",
        "You decide to pay 1 life as the last ability is resolving. If you decide to pay 1 life and Corpses of the Lost is still on the battlefield, it will immediately return to its owner's hand. No player may take any actions in between you deciding to pay life and it returning this way."
    );
    supported("Corpses of the Lost");
    // P0 descended this turn: Grizzly Bears was put into P0's graveyard.
    let mut t = TestGame::new(2);
    let corpses = t.battlefield(P0, "Corpses of the Lost");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.g.flush_events();
    to_end_step(&mut t, P0);
    assert_eq!(abilities_from(&t, corpses), 1);
    t.answer_yes(P0, true);
    t.g.resolve_top();
    assert_eq!(t.life(P0), 19);
    assert!(t.in_hand(P0, "Corpses of the Lost"));
    // Not on the battlefield any more as the ability resolves: it stays where it is.
    let mut t = TestGame::new(2);
    let corpses = t.battlefield(P0, "Corpses of the Lost");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.g.flush_events();
    to_end_step(&mut t, P0);
    assert_eq!(abilities_from(&t, corpses), 1);
    t.g.move_object(
        corpses,
        mtg_engine::object::Zone::Exile,
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.g.flush_events();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_exile("Corpses of the Lost"));
}

#[test]
fn elenda_and_azor_x_is_chosen_as_the_attack_trigger_resolves() {
    cr!("107.3", "118.12");
    ruling!(
        "Elenda and Azor",
        "You choose the value of X as the first triggered ability resolves."
    );
    supported("Elenda and Azor");
    let mut t = TestGame::new(2);
    let ea = t.battlefield(P0, "Elenda and Azor");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 2);
    crate::r_s01_common::attack_with(&mut t, &[(ea, Entity::Player(P1))]);
    assert_eq!(abilities_from(&t, ea), 1);
    let x_asked = |t: &TestGame| {
        t.asked()
            .iter()
            .filter(|(p, d)| *p == P0 && matches!(d, Decision::ChooseX { .. }))
            .count()
    };
    assert_eq!(x_asked(&t), 0);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    answer_x(&mut t, P0, 2);
    t.resolve_all();
    assert_eq!(x_asked(&t), 1);
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 5);
}

// ---------------------------------------------------------------------------------------
// Reflexive triggers
// ---------------------------------------------------------------------------------------

#[test]
fn pay_life_reflexive_triggers_choose_targets_when_put_on_the_stack() {
    cr!("603.12", "603.3d");
    ruling!(
        "Ambulatory Edifice",
        "You don't choose a target for Ambulatory Edifice's last ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when you pay 2 life this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    ruling!(
        "Zoraline, Cosmos Caller",
        "You don’t choose a target for Zoraline’s last ability at the time it triggers. Rather, a second “reflexive” ability triggers when you pay {W}{B} and 2 life this way. You choose a target for this ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    supported("Ambulatory Edifice");
    supported("Zoraline, Cosmos Caller");
    // Ambulatory Edifice enters; the trigger has no target. Paying 2 life puts the
    // reflexive trigger on the stack with its target; a response (giving the target a
    // +1/+1 counter) happens before it resolves.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let edifice = t.enter(P0, "Ambulatory Edifice");
    t.settle();
    let first = crate::r_s25_common::abilities_from(&t, edifice);
    assert_eq!(first.len(), 1);
    assert!(targets_of(&t, first[0]).is_empty());
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve();
    assert_eq!(t.life(P0), 18);
    let reflexive = crate::r_s25_common::abilities_from(&t, edifice);
    assert_eq!(reflexive.len(), 1);
    assert_eq!(targets_of(&t, reflexive[0]), vec![Entity::Object(bears)]);
    put_counters(&mut t, bears, "+1/+1", 1);
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
    // Zoraline: pay {W}{B} and 2 life; the reflexive trigger targets Grizzly Bears in the
    // graveyard as it is put on the stack.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 1);
    let card = t.graveyard(P0, "Grizzly Bears");
    let zoraline = t.enter(P0, "Zoraline, Cosmos Caller");
    t.settle();
    let first = crate::r_s25_common::abilities_from(&t, zoraline);
    assert_eq!(first.len(), 1);
    assert!(targets_of(&t, first[0]).is_empty());
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(card)]);
    t.resolve();
    assert_eq!(t.life(P0), 18);
    let reflexive = crate::r_s25_common::abilities_from(&t, zoraline);
    assert_eq!(reflexive.len(), 1);
    assert_eq!(targets_of(&t, reflexive[0]), vec![Entity::Object(card)]);
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert_eq!(t.counters(bears[0], "finality"), 1);
}

// ---------------------------------------------------------------------------------------
// Enters and leaves
// ---------------------------------------------------------------------------------------

#[test]
fn enters_or_dies_triggers_both_happen() {
    cr!("603.6a", "603.6c", "603.10a");
    ruling!(
        "Ichor Wellspring",
        "The ability will trigger when Ichor Wellspring is put into a graveyard from the battlefield, even if the ability that triggered when it entered hasn't resolved yet."
    );
    ruling!(
        "Hunting Moa",
        "You get a +1/+1 counter both when it enters and when it goes to a graveyard from the battlefield."
    );
    supported("Ichor Wellspring");
    supported("Hunting Moa");
    // Ichor Wellspring enters and is destroyed before its enters trigger resolves: two
    // cards.
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    let well = t.enter(P0, "Ichor Wellspring");
    t.settle();
    assert_eq!(abilities_from(&t, well), 1);
    t.g.destroy(well, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // Hunting Moa: a counter on Grizzly Bears as it enters and as it dies.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let moa = t.enter(P0, "Hunting Moa");
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.g.destroy(moa, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 2);
}

#[test]
fn servo_schematic_cant_be_sacrificed_at_will() {
    cr!("602.1", "603.6c");
    ruling!(
        "Servo Schematic",
        "Servo Schematic's ability doesn't allow you to sacrifice it whenever you'd like. You must find another way to get Servo Schematic into the graveyard."
    );
    supported("Servo Schematic");
    let mut t = TestGame::new(2);
    let schematic = t.battlefield(P0, "Servo Schematic");
    t.g.turn.priority = Some(P0);
    assert!(!t.g.legal_actions(P0).iter().any(
        |a| matches!(a, Action::Activate { source, .. } if *source == schematic)
    ));
    t.g.destroy(schematic, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Servo").len(), 1);
}

// ---------------------------------------------------------------------------------------
// Dreadhorde Invasion, Herald of Torment, Dawn of the Dead, Meathook Massacre II
// ---------------------------------------------------------------------------------------

#[test]
fn dreadhorde_invasion_checks_power_immediately_after_the_attack() {
    cr!("603.2", "508.3a");
    ruling!(
        "Dreadhorde Invasion",
        "The power of the Zombie token is checked only immediately after it attacks. Any other abilities that trigger when creatures you control attack won't have resolved yet."
    );
    supported("Dreadhorde Invasion");
    // A 5/5 Zombie token attacks with Signal Pest (battle cry: "each other attacking
    // creature gets +1/+0"): power 5 as it attacks, so no lifelink, though it's 6/5 later.
    for (counters, lifelink) in [(4, false), (5, true)] {
        let mut t = TestGame::new(2);
        let invasion = t.battlefield(P0, "Dreadhorde Invasion");
        let zombie = crate::r_s02_common::create_token(&mut t, P0, "Zombie");
        put_counters(&mut t, zombie, "+1/+1", counters);
        let pest = t.battlefield(P0, "Signal Pest");
        crate::r_s01_common::attack_with(
            &mut t,
            &[(zombie, Entity::Player(P1)), (pest, Entity::Player(P1))],
        );
        assert_eq!(abilities_from(&t, invasion), lifelink as usize);
        t.resolve_all();
        assert_eq!(t.pt(zombie).0, 2 + counters as i32);
        assert_eq!(
            t.obj_now(zombie).has_keyword(KeywordKind::Lifelink),
            lifelink
        );
    }
}

#[test]
fn herald_of_torment_as_an_aura_its_controller_loses_life() {
    cr!("109.5", "702.103b", "503.1a");
    ruling!(
        "Herald of Torment",
        "The triggered ability that causes you to lose life applies whether Herald of Torment is a creature or an Aura. The \"you\" in that ability refers to Herald of Torment's controller. If you control Herald of Torment as an Aura enchanting a creature another player controls, that ability will trigger at the beginning of your upkeep and cause you to lose 1 life."
    );
    supported("Herald of Torment");
    // Bestowed on P1's Grizzly Bears.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Wastes", 3);
    let herald = t.hand(P0, "Herald of Torment");
    t.cast(P0, herald)
        .method(CastMethod::Keyword(KeywordKind::Bestow))
        .target(bears)
        .go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    let herald = t.named_on_battlefield("Herald of Torment")[0];
    assert!(t.obj_now(herald).chars.has_subtype("Aura"));
    to_upkeep(&mut t, P1);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    to_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (19, 20));
    // As a creature, too.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Herald of Torment");
    to_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
}

#[test]
fn dawn_of_the_dead_exiles_the_creature_only_if_it_is_still_on_the_battlefield() {
    cr!("400.7", "603.7c");
    ruling!(
        "Dawn of the Dead",
        "The creature is only exiled if it is still on the battlefield at the beginning of the end step."
    );
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Dawn of the Dead");
        let bears = t.graveyard(P0, "Grizzly Bears");
        t.answer_targets(P0, &[Entity::Object(bears)]);
        to_upkeep(&mut t, P0);
        t.answer_yes(P0, true);
        t.resolve_all();
        let on = t.named_on_battlefield("Grizzly Bears");
        assert_eq!(on.len(), 1);
        if leaves {
            bounce(&mut t, on[0]);
        }
        to_end_step(&mut t, P0);
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "Grizzly Bears"), leaves);
        assert_eq!(t.in_exile("Grizzly Bears"), !leaves);
    }
}

#[test]
fn meathook_massacre_ii_tokens_that_die_dont_return() {
    cr!("111.8", "704.5d");
    ruling!(
        "Meathook Massacre II",
        "When a token creature dies, Meathook Massacre II's second or third ability (as appropriate) will trigger. The appropriate player has the option to pay 3 life, but regardless of their choice, the token won't return to the battlefield."
    );
    supported("Meathook Massacre II");
    let mut t = TestGame::new(2);
    let mm = t.battlefield(P0, "Meathook Massacre II");
    let mine = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    let theirs = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    t.g.destroy(mine, None);
    t.g.destroy(theirs, None);
    t.g.flush_events();
    t.settle();
    assert_eq!(abilities_from(&t, mm), 2);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_yes(P1, false);
    t.resolve_all();
    assert_eq!(yes_no_asked(&t, from, P0), 1);
    assert_eq!(yes_no_asked(&t, from, P1), 1);
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.life(P1), 20);
    assert!(tokens(&t, P0).is_empty());
    assert!(tokens(&t, P1).is_empty());
}

// ---------------------------------------------------------------------------------------
// Lotho, Corrupt Shirriff and Cursed Recording
// ---------------------------------------------------------------------------------------

#[test]
fn lotho_counts_spells_cast_before_it() {
    cr!("603.2", "601.2i");
    ruling!(
        "Lotho, Corrupt Shirriff",
        "Spells that were cast before Lotho, Corrupt Shirriff count. If Lotho, Corrupt Shirriff was the first spell you cast this turn, the next spell you cast this turn is your second spell."
    );
    supported("Lotho, Corrupt Shirriff");
    // Lotho the first spell: the next spell triggers it.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Lotho, Corrupt Shirriff", &[]);
    t.resolve_all();
    let lotho = t.named_on_battlefield("Lotho, Corrupt Shirriff")[0];
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.settle();
    assert_eq!(abilities_from(&t, lotho), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert_eq!(with_subtype(&t, P0, "Treasure").len(), 1);
    // Lotho the second spell: the third doesn't trigger it.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    cast_new(&mut t, P0, "Lotho, Corrupt Shirriff", &[]);
    t.resolve_all();
    let lotho = t.named_on_battlefield("Lotho, Corrupt Shirriff")[0];
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.settle();
    assert_eq!(abilities_from(&t, lotho), 0);
}

#[test]
fn cursed_recording_copy_keeps_or_changes_targets_and_isnt_cast() {
    cr!("707.10", "707.10c", "603.2");
    ruling!(
        "Cursed Recording",
        "The copy created by Cursed Recording's last ability will have the same targets as the spell it's copying unless you choose new ones. You may change any number of the targets, including all of them or none of them. The new targets must be legal."
    );
    ruling!(
        "Cursed Recording",
        "The copy made by Cursed Recording's last ability is created on the stack, so it's not \"cast.\""
    );
    supported("Cursed Recording");
    for change in [false, true] {
        let mut t = TestGame::new(2);
        let rec = t.battlefield(P0, "Cursed Recording");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.activate(P0, rec, 0, &[]).unwrap();
        t.resolve();
        cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
        // (The answers for the copy, queued after the Shock's target.)
        if change {
            crate::r_s25_common::change_copy_targets(
                &mut t,
                P0,
                &[Some(Entity::Object(bears))],
            );
        } else {
            crate::r_s25_common::keep_copy_targets(&mut t, P0);
        }
        t.settle();
        // The copy trigger resolves; the time counter trigger (for the cast) too.
        while spell_copies(&t).is_empty() && t.stack_len() > 1 {
            t.resolve();
        }
        let copies = spell_copies(&t);
        assert_eq!(copies.len(), 1);
        let expected = if change {
            Entity::Object(bears)
        } else {
            Entity::Player(P1)
        };
        assert_eq!(targets_of(&t, copies[0]), vec![expected]);
        t.resolve_all();
        // One time counter: for the Shock that was cast, not for its copy.
        assert_eq!(t.counters(rec, "time"), 1);
        assert_eq!(t.life(P1), if change { 18 } else { 16 });
        assert_eq!(t.in_graveyard(P1, "Grizzly Bears"), change);
    }
}
