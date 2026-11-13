//! Rulings batch P225 — permanents that transform during combat or with abilities on the
//! stack: a transforming permanent stays the same object (CR 712.18), so it stays tapped,
//! attacking, blocking or blocked (CR 506.4, 509.1h), and effects that already applied
//! keep applying (CR 611.2c); attack triggers of the new face don't trigger
//! retroactively (CR 603.2), and resolved abilities of the other face still do their thing.

use crate::r_s01_common::*;
use crate::r_s03_common::to_blockers;
use crate::r_s04_common::add_mana;
use crate::r_s06_common::{activate_containing, attach_new, attached_to};
use crate::r_s10_common::blocking;
use crate::r_s17_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const KRUIN: &str = "Kruin Outlaw // Terror of Kruin Pass";
const TANGLECLAW: &str = "Tangleclaw Werewolf // Fibrous Entangler";
const VOLTAIC: &str = "Voltaic Visionary // Volt-Charged Berserker";
const TORMENTOR: &str = "Elusive Tormentor // Insidious Mist";
const SMOLDERING: &str = "Smoldering Werewolf // Erupting Dreadwolf";
const GOSSIP: &str = "Town Gossipmonger // Incited Rabble";
const LANDING: &str = "Legion's Landing // Adanto, the First Fort";
const CONDUIT: &str = "Conduit of Storms // Conduit of Emrakul";
const BALLISTA: &str = "Ballista Watcher // Ballista Wielder";
const MISSIONARIES: &str = "Avacynian Missionaries // Lunarch Inquisitors";

#[test]
fn transforming_doesnt_change_blocks_already_declared() {
    cr!("509.1h", "506.4", "712.18");
    ruling!(
        "Kruin Outlaw // Terror of Kruin Pass",
        "any Werewolves you control that are blocked by a single creature will remain blocked"
    );
    ruling!(
        "Elusive Tormentor // Insidious Mist",
        "Once blockers have been chosen, transforming Elusive Tormentor into Insidious Mist won't cause it to become unblocked."
    );
    ruling!(
        "Voltaic Visionary // Volt-Charged Berserker",
        "it will not be removed from combat and is still blocking"
    );
    supported(KRUIN);
    supported(TORMENTOR);
    // Kruin Outlaw blocked by one creature; it becomes Terror of Kruin Pass (Werewolves
    // you control have menace).
    let mut t = TestGame::new(2);
    let kruin = t.battlefield(P0, KRUIN);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    to_blockers(&mut t, &[(kruin, Entity::Player(P1))], &[(bears, kruin)]);
    transform(&mut t, kruin);
    assert!(t.obj_now(kruin).has_keyword(KeywordKind::Menace));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Elusive Tormentor blocked; it becomes Insidious Mist ("can't be blocked").
    let mut t = TestGame::new(2);
    let tormentor = t.battlefield(P0, TORMENTOR);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    let discard = t.hand(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    to_blockers(
        &mut t,
        &[(tormentor, Entity::Player(P1))],
        &[(bears, tormentor)],
    );
    t.answer_choose(P0, &[Entity::Object(discard)]);
    activate_containing(&mut t, P0, tormentor, "Transform").unwrap();
    t.resolve_all();
    assert_eq!(name_of(&t, tormentor), "Insidious Mist");
    // (Insidious Mist is a 0/1, so P1's life total can't tell; check the block itself.)
    assert!(crate::r_p076_common::is_blocked(&t, tormentor));
    t.advance_to(P0, Step::CombatDamage);
    assert!(crate::r_p076_common::is_blocked(&t, tormentor));
    // (Voltaic Visionary's last ability doesn't compile; it isn't involved.) It blocks,
    // then becomes Volt-Charged Berserker ("can't block"), and is still blocking.
    let mut t = TestGame::new(2);
    let visionary = t.battlefield(P0, VOLTAIC);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    to_blockers(
        &mut t,
        &[(bears, Entity::Player(P0))],
        &[(visionary, bears)],
    );
    transform(&mut t, visionary);
    assert_eq!(name_of(&t, visionary), "Volt-Charged Berserker");
    assert!(blocking(&t, visionary));
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.life(P0), 20);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn transforming_doesnt_untap_a_permanent() {
    cr!("712.18", "702.20b");
    ruling!(
        "Tangleclaw Werewolf // Fibrous Entangler",
        "having vigilance won't cause it to become untapped"
    );
    ruling!(
        "Town Gossipmonger // Incited Rabble",
        "Note that transforming Town Gossipmonger won’t untap it."
    );
    ruling!(
        "Heirloom Mirror // Inherited Fiend",
        "Heirloom Mirror will still be tapped after it transforms into Inherited Fiend unless some other effect has untapped it."
    );
    supported(TANGLECLAW);
    supported(GOSSIP);
    // (Heirloom Mirror's ability, which taps it and then may transform it, doesn't
    // compile; a tapped Heirloom Mirror transformed by an effect stays tapped.)
    let mut t = TestGame::new(2);
    let mirror = t.battlefield(P0, "Heirloom Mirror // Inherited Fiend");
    t.g.tap(mirror);
    transform(&mut t, mirror);
    assert_eq!(name_of(&t, mirror), "Inherited Fiend");
    assert!(t.obj_now(mirror).tapped);
    let mut t = TestGame::new(2);
    let wolf = t.battlefield(P0, TANGLECLAW);
    attack_with(&mut t, &[(wolf, Entity::Player(P1))]);
    assert!(t.obj_now(wolf).tapped);
    transform(&mut t, wolf);
    assert!(t.obj_now(wolf).has_keyword(KeywordKind::Vigilance));
    assert!(t.obj_now(wolf).tapped);
    // Town Gossipmonger taps to transform; Incited Rabble is tapped and doesn't attack.
    let mut t = TestGame::new(2);
    let gossip = t.battlefield(P0, GOSSIP);
    let other = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(other)]);
    activate_containing(&mut t, P0, gossip, "Transform").unwrap();
    t.resolve_all();
    assert_eq!(name_of(&t, gossip), "Incited Rabble");
    assert!(t.obj_now(gossip).tapped);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!crate::r_s02_common::can_attack(&mut t, gossip));
}

#[test]
fn a_new_faces_attack_trigger_doesnt_trigger_after_attackers_were_declared() {
    cr!("603.2", "508.3a");
    ruling!(
        "Smoldering Werewolf // Erupting Dreadwolf",
        "Erupting Dreadwolf's triggered ability won't trigger that combat"
    );
    supported(SMOLDERING);
    let mut t = TestGame::new(2);
    let wolf = t.battlefield(P0, SMOLDERING);
    attack_with(&mut t, &[(wolf, Entity::Player(P1))]);
    transform(&mut t, wolf);
    assert_eq!(name_of(&t, wolf), "Erupting Dreadwolf");
    assert_eq!(t.stack_len(), 0);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 14);
}

#[test]
fn legions_landing_transforms_even_if_attackers_left_combat() {
    cr!("603.2", "608.2b");
    ruling!(
        "Legion's Landing // Adanto, the First Fort",
        "Legion's Landing will transform even if some of those creatures leave the battlefield or are removed from combat"
    );
    supported(LANDING);
    let mut t = TestGame::new(2);
    let landing = t.battlefield(P0, LANDING);
    let a: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    attack_with(
        &mut t,
        &a.iter()
            .map(|c| (*c, Entity::Player(P1)))
            .collect::<Vec<_>>(),
    );
    assert_eq!(t.stack_len(), 1);
    crate::r_s02_common::destroy(&mut t, a[0]);
    crate::r_s02_common::destroy(&mut t, a[1]);
    t.resolve_all();
    assert_eq!(name_of(&t, landing), "Adanto, the First Fort");
}

#[test]
fn conduit_of_storms_attack_trigger_adds_red_after_it_transforms() {
    cr!("603.7", "712.18", "113.7a");
    ruling!(
        "Conduit of Storms // Conduit of Emrakul",
        "you’ll add {R}, not {C}{C} and not {C}{C}{R}"
    );
    supported(CONDUIT);
    let mut t = TestGame::new(2);
    let conduit = t.battlefield(P0, CONDUIT);
    attack_with(&mut t, &[(conduit, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    transform(&mut t, conduit);
    assert_eq!(name_of(&t, conduit), "Conduit of Emrakul");
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    let pool = &t.g.player(P0).mana_pool;
    assert_eq!(pool.count(ManaType::R), 1);
    assert_eq!(pool.count(ManaType::C), 0);
}

#[test]
fn ballista_faces_abilities_are_independent() {
    cr!("611.2c", "712.18", "702.145c");
    ruling!(
        "Ballista Watcher // Ballista Wielder",
        "The activated abilities of Ballista Watcher and Ballista Wielder are completely independent of one another."
    );
    supported(BALLISTA);
    // Ballista Watcher pings a creature, then it becomes night: the creature can block.
    let mut t = TestGame::new(2);
    let ballista = t.battlefield(P0, BALLISTA);
    let giant = t.battlefield(P1, "Hill Giant");
    add_mana(&mut t, P0, ManaType::R, 3);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    activate_containing(&mut t, P0, ballista, "1 damage").unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 1);
    t.g.day = Some(false);
    t.g.recompute();
    t.settle();
    assert_eq!(name_of(&t, ballista), "Ballista Wielder");
    assert!(t.g.can_block_at_all(giant));
    // Ballista Wielder pings a creature, then it becomes day: the creature still can't
    // block this turn.
    let mut t = TestGame::new(2);
    let ballista = t.battlefield(P0, BALLISTA);
    t.g.day = Some(false);
    t.g.recompute();
    t.settle();
    assert_eq!(name_of(&t, ballista), "Ballista Wielder");
    let giant = t.battlefield(P1, "Hill Giant");
    add_mana(&mut t, P0, ManaType::R, 3);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    activate_containing(&mut t, P0, ballista, "1 damage").unwrap();
    t.resolve_all();
    assert!(!t.g.can_block_at_all(giant));
    t.g.day = Some(true);
    t.g.recompute();
    t.settle();
    assert_eq!(name_of(&t, ballista), "Ballista Watcher");
    assert!(!t.g.can_block_at_all(giant));
}

#[test]
fn equipment_stays_attached_when_avacynian_missionaries_transforms() {
    cr!("712.18");
    ruling!(
        "Avacynian Missionaries // Lunarch Inquisitors",
        "The Equipment attached to Avacynian Missionaries remains attached after it transforms into Lunarch Inquisitors."
    );
    supported(MISSIONARIES);
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, MISSIONARIES);
    let gear = attach_new(&mut t, P0, "Bonesplitter", m);
    t.advance_to(P0, Step::End);
    t.settle();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(name_of(&t, m), "Lunarch Inquisitors");
    assert_eq!(attached_to(&t, gear), Some(Entity::Object(t.g.current(m))));
    assert_eq!(t.pt(m), (6, 4));
}
