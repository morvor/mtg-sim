//! Rulings batch P073 — lifelink and life-gain triggers: Heliod, Sun-Crowned (counters on
//! a noncreature enchantment, one trigger per life-gaining event, lethal damage dealt as
//! life is gained), Firesong and Sunspeaker (which spells "cause" life gain, how many
//! times it triggers), Vito, Vizkopa Guildmage, Judith, Lo and Li, Prepare // Fight,
//! Battlegrace Angel and Blacklance Paragon.

use crate::r_p073_common::*;
use crate::r_s01_common::{attack_with, custom_card, supported};
use crate::r_s06_common::activate_containing;
use mtg_engine::decision::Answer;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::Color;
use mtg_engine::*;

/// Grants lifelink to `id` until end of turn with Heliod's last ability (`heliod` needs
/// {1}{W} available).
fn heliod_lifelink(t: &mut TestGame, heliod: ObjectId, id: ObjectId) {
    t.lands(P0, "Plains", 2);
    t.answer_targets(P0, &[obj(id)]);
    activate_containing(t, P0, heliod, "lifelink").unwrap();
    t.resolve_all();
    assert!(kw(t, id, LIFELINK));
}

#[test]
fn heliod_counter_on_a_noncreature_enchantment_applies_once_its_a_creature() {
    cr!("122.1a", "613.4c", "700.5");
    ruling!(
        "Heliod, Sun-Crowned",
        "A noncreature enchantment with a +1/+1 counter on it will be unaffected by that counter until it becomes a creature, at which time it will get +1/+1 for that counter."
    );
    ruling!(
        "Heliod, Sun-Crowned",
        "Heliod can be the target of its own triggered ability."
    );
    supported("Heliod, Sun-Crowned");
    let mut t = TestGame::new(2);
    let heliod = t.battlefield(P0, "Heliod, Sun-Crowned");
    assert!(!t.obj_now(heliod).is(mtg_engine::types::CardType::Creature));
    t.answer_targets(P0, &[obj(heliod)]);
    t.g.gain_life(P0, 1);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(heliod, "+1/+1"), 1);
    assert!(!t.obj_now(heliod).is(mtg_engine::types::CardType::Creature));
    // Devotion to white reaches five: Heliod is a 6/6 creature.
    t.battlefield(P0, "Benalish Marshal");
    t.battlefield(P0, "Benalish Marshal");
    assert!(t.obj_now(heliod).is(mtg_engine::types::CardType::Creature));
    // 5/5, +1/+1 from the counter, +1/+1 from each Marshal.
    assert_eq!(t.pt(heliod), (8, 8));
}

#[test]
fn each_lifelink_creature_is_a_separate_life_gaining_event() {
    cr!("119.9", "510.2", "702.15b");
    ruling!(
        "Heliod, Sun-Crowned",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Heliod's ability will trigger twice."
    );
    ruling!(
        "Vito, Thorn of the Dusk Rose",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Vito's first ability will trigger twice and you may choose a different opponent for each trigger."
    );
    // Heliod: two lifelinkers unblocked trigger twice; a lifelink trampler blocked deals
    // damage to the blocker and the player at once and triggers once.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Heliod, Sun-Crowned");
    let a = t.battlefield(P0, "Child of Night");
    let b = t.battlefield(P0, "Child of Night");
    let wall = t.battlefield(P0, "Wall of Stone");
    for _ in 0..2 {
        t.answer_targets(P0, &[obj(wall)]);
    }
    t.attack(&[(a, pl(P1)), (b, pl(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.counters(wall, "+1/+1"), 2, "two triggers");
    let mut t = TestGame::new(2);
    let heliod = t.battlefield(P0, "Heliod, Sun-Crowned");
    let wurm = t.battlefield(P0, "Colossal Dreadmaw");
    let wall = t.battlefield(P0, "Wall of Stone");
    heliod_lifelink(&mut t, heliod, wurm);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(wall)]);
    t.answer_targets(P0, &[obj(wall)]);
    t.attack(&[(wurm, pl(P1))], &[(bears, wurm)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P0), 26);
    assert_eq!(t.counters(wall, "+1/+1"), 1, "one trigger");
    // Vito in a three-player game: two triggers, each aimed at a different opponent.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Vito, Thorn of the Dusk Rose");
    let a = t.battlefield(P0, "Child of Night");
    let b = t.battlefield(P0, "Child of Night");
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[pl(P2)]);
    t.attack(&[(a, pl(P1)), (b, pl(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    // 4 combat damage to P1, then 2 life lost by each.
    assert_eq!((t.life(P1), t.life(P2)), (14, 18));
}

#[test]
fn heliod_triggers_once_for_life_gained_for_each() {
    cr!("119.9", "603.2c");
    ruling!(
        "Heliod, Sun-Crowned",
        "Heliod's triggered ability triggers just once for each life-gaining event, whether it's 1 life from Daxos, Blessed by the Sun or 3 life from Cling to Dust. If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and Heliod's ability triggers only once."
    );
    supported("Festival of Trokin");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Heliod, Sun-Crowned");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Llanowar Elves");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(bears)]);
    cast_resolve(&mut t, P0, "Festival of Trokin", &[]);
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.counters(bears, "+1/+1"), 1);
}

#[test]
fn heliod_counter_cant_save_a_creature_dealt_lethal_damage_as_life_is_gained() {
    cr!("704.5g", "603.3", "510.2");
    ruling!(
        "Heliod, Sun-Crowned",
        "If a creature is dealt lethal damage at the same time that you gain life, it can't receive a counter from Heliod's ability in time to save it."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Heliod, Sun-Crowned");
    let child = t.battlefield(P0, "Child of Night");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.attack(&[(child, pl(P1))], &[(bears, child)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert!(!t.on_battlefield(child), "died before the trigger could target it");
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(cands.iter().all(|c| !c.contains(&obj(child))));
}

#[test]
fn firesong_triggers_for_lifelink_damage_a_white_spell_causes() {
    cr!("702.15b", "119.10");
    ruling!(
        "Firesong and Sunspeaker",
        "If a spell's cost or effect instructs a source with lifelink you control to deal damage, that spell causes that life gain as well."
    );
    supported("Justice Strike");
    // Justice Strike (red and white): "Target creature deals damage to itself equal to
    // its power." Child of Night has lifelink: the spell causes that life gain.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Firesong and Sunspeaker");
    let child = t.battlefield(P0, "Child of Night");
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_p079_common::cast_card(&mut t, P0, "Justice Strike", &[obj(child)]).unwrap();
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(child));
    assert_eq!(t.life(P0), 22);
    assert!(!t.on_battlefield(bears), "Firesong's trigger dealt 3 damage");
}

#[test]
fn firesong_triggers_twice_for_a_lifelink_spell_that_also_gains_life() {
    cr!("119.9", "702.15b");
    ruling!(
        "Firesong and Sunspeaker",
        "If a red and white spell you control deals damage and also instructs you to gain life, Firesong and Sunspeaker's last ability triggers twice."
    );
    supported("Lightning Helix");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Firesong and Sunspeaker");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Stone");
    crate::r_p079_common::cast_card(&mut t, P0, "Lightning Helix", &[pl(P1)]).unwrap();
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(wall)]);
    t.resolve();
    assert_eq!(t.life(P0), 26, "lifelink and the instruction");
    assert_eq!(t.stack_len(), 2, "two triggers");
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.obj_now(wall).damage, 3);
}

#[test]
fn firesong_counts_each_instance_of_deals() {
    cr!("119.9", "120.2");
    ruling!(
        "Firesong and Sunspeaker",
        "If a red and white spell you control deals damage to multiple things using the word \"deals\" only once, Firesong and Sunspeaker's last ability triggers only once. Similarly, if a red and white spell's effect causes it to deal damage to one thing and then deal more damage with a second instance of the word \"deals,\" Firesong and Sunspeaker's last ability triggers twice, and so on."
    );
    // Deafening Clarion's first mode: one "deals" for each creature: one trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Firesong and Sunspeaker");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    let c = t.hand(P0, "Deafening Clarion");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Deafening Clarion");
    t.cast(P0, c).modes(&[0]).go();
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve();
    // 3 damage to each of three creatures (Firesong included): 9 life at once.
    assert_eq!(t.life(P0), 29);
    assert_eq!(t.stack_len(), 1, "one trigger");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // A red and white spell with two instances of "deals": two triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Firesong and Sunspeaker");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mut def = custom_card(
        "Twin Strike",
        "Instant",
        "{R}{W}",
        None,
        "Twin Strike deals 1 damage to target creature. Twin Strike deals 1 damage to target player.",
    );
    def.faces[0].chars.colors = [Color::Red, Color::White].into_iter().collect();
    let spell = t.custom(P0, def, Zone::Hand(P0));
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Mountain", 1);
    t.cast(P0, spell).targets(&[obj(bears)]).targets(&[pl(P1)]).go();
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.stack_len(), 2, "two triggers");
    t.resolve_all();
    assert_eq!(t.life(P1), 13);
}

#[test]
fn lifelink_spell_damaging_you_leaves_your_life_unchanged() {
    cr!("702.15b", "119.9");
    ruling!(
        "Firesong and Sunspeaker",
        "If a source you control with lifelink deals damage to you, you gain and lose that much life simultaneously. Your life total doesn't change."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Firesong and Sunspeaker");
    crate::r_p079_common::cast_card(&mut t, P0, "Lightning Bolt", &[pl(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn firesong_triggers_for_an_opponents_white_spell() {
    cr!("603.2");
    ruling!(
        "Firesong and Sunspeaker",
        "If a white instant or sorcery spell you don't control causes you to gain life, Firesong and Sunspeaker's last ability triggers."
    );
    supported("Soothing Balm");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Firesong and Sunspeaker");
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_p079_common::cast_card(&mut t, P1, "Soothing Balm", &[pl(P0)]).unwrap();
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
    assert!(!t.on_battlefield(bears));
}

#[test]
fn deathtouch_and_lifelink_spells_must_deal_the_damage_themselves() {
    cr!("702.2b", "702.15b", "120.1a");
    ruling!(
        "Judith, Carnage Connoisseur",
        "An instant or sorcery spell with deathtouch must actually deal damage to a creature for it to be destroyed. If the spell instructs another object to deal damage (for example, Hard-Hitting Question), the spell doesn't deal any damage itself and its instance of deathtouch doesn't apply. Dealing 0 damage isn't dealing damage."
    );
    ruling!(
        "Lo and Li, Twin Tutors",
        "An instant or sorcery spell with lifelink causes its controller to gain life only if it's the source of any damage that's dealt. An instant or sorcery spell with lifelink that causes another source to deal damage won't cause its controller to gain life."
    );
    supported("Hard-Hitting Question");
    supported("Lo and Li, Twin Tutors");
    // Judith's deathtouch on Hard-Hitting Question: the Elves deal the damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Judith, Carnage Connoisseur");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    cast_resolve(&mut t, P0, "Hard-Hitting Question", &[obj(elves), obj(wurm)]);
    assert!(t.on_battlefield(wurm));
    assert_eq!(t.obj_now(wurm).damage, 1);
    assert_eq!(t.life(P0), 20);
    // Positive control: Shock with deathtouch destroys it and gains life.
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    cast_resolve(&mut t, P0, "Shock", &[obj(wurm)]);
    assert!(!t.on_battlefield(wurm));
    assert_eq!(t.life(P0), 22);
    // 0 damage isn't damage: a deathtouch Fireball for X=0 doesn't destroy.
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.lands(P0, "Mountain", 1);
    let fb = t.hand(P0, "Fireball");
    t.cast(P0, fb).x(0).target(obj(wurm)).go();
    t.resolve_all();
    assert!(t.on_battlefield(wurm));
    // Lo and Li: a Lesson spell with lifelink that deals damage itself gains life; one
    // that has a creature deal the damage doesn't.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lo and Li, Twin Tutors");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Both halves of "Noble creatures you control and Lesson spells you control" apply
    // (the Lesson half used to be dropped: regression).
    let noble = t.battlefield(P0, "Arwen, Mortal Queen");
    assert!(kw(&t, noble, LIFELINK) && !kw(&t, elves, LIFELINK));
    let c = t.hand(P0, "Start from Scratch");
    t.lands(P0, "Mountain", 3);
    t.cast(P0, c).modes(&[0]).target(pl(P1)).go();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
    let def = custom_card(
        "Sparring Lesson",
        "Sorcery — Lesson",
        "{G}",
        None,
        "Target creature you control deals damage equal to its power to target creature you don't control.",
    );
    let lesson = t.custom(P0, def, Zone::Hand(P0));
    t.lands(P0, "Forest", 1);
    t.cast(P0, lesson).target(obj(elves)).target(obj(bears)).go();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).damage, 1);
    assert_eq!(t.life(P0), 21, "the Elves dealt the damage");
}

#[test]
fn prepare_lifelink_applies_to_fight_damage() {
    cr!("702.15b", "701.14a", "702.127a");
    ruling!(
        "Prepare // Fight",
        "Any damage dealt by a creature you control with lifelink causes you to gain that much life, not just combat damage."
    );
    supported("Prepare // Fight");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.tap(bears);
    let card = t.hand(P0, "Prepare // Fight");
    t.lands(P0, "Plains", 2);
    t.cast(P0, card)
        .method(CastMethod::Half(0))
        .target(obj(bears))
        .go();
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
    assert_eq!(t.pt(bears), (4, 4));
    let card = t.g.current(card);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    t.lands(P0, "Forest", 4);
    t.cast(P0, card)
        .method(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Aftermath))
        .target(obj(bears))
        .target(obj(giant))
        .go();
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.life(P0), 24);
}

#[test]
fn vito_with_lifelink_doesnt_gain_from_its_life_loss_trigger() {
    cr!("120.3", "119.3");
    ruling!(
        "Vito, Thorn of the Dusk Rose",
        "Because Vito's first ability doesn't deal damage, you won't gain life when it resolves if Vito has lifelink."
    );
    let mut t = TestGame::new(2);
    let vito = t.battlefield(P0, "Vito, Thorn of the Dusk Rose");
    t.lands(P0, "Swamp", 5);
    activate_containing(&mut t, P0, vito, "lifelink").unwrap();
    t.resolve_all();
    assert!(kw(&t, vito, LIFELINK));
    t.answer_targets(P0, &[pl(P1)]);
    t.g.gain_life(P0, 3);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (23, 17));
}

#[test]
fn vizkopa_guildmage_delayed_triggers_stack_up() {
    cr!("603.7a", "603.7c");
    ruling!(
        "Vizkopa Guildmage",
        "Each time Vizkopa Guildmage’s second ability resolves, a delayed triggered ability is created. Whenever you gain life that turn, each of those abilities will trigger."
    );
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, "Vizkopa Guildmage");
    t.lands(P0, "Plains", 3);
    t.lands(P0, "Swamp", 3);
    for _ in 0..2 {
        activate_containing(&mut t, P0, mage, "each opponent loses").unwrap();
        t.resolve_all();
    }
    t.g.gain_life(P0, 2);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Only that turn.
    t.advance_to(P1, Step::Upkeep);
    t.g.gain_life(P0, 2);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn battlegrace_angel_extra_lifelink_is_redundant() {
    cr!("702.15f", "702.83a");
    ruling!(
        "Battlegrace Angel",
        "If a creature has multiple instances of lifelink, they are redundant. You’ll still only gain life equal to the damage dealt."
    );
    supported("Battlegrace Angel");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Battlegrace Angel");
    t.battlefield(P0, "Battlegrace Angel");
    let child = t.battlefield(P0, "Child of Night");
    attack_with(&mut t, &[(child, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.obj_now(child).chars.keywords_of(LIFELINK).count(), 3);
    assert_eq!(t.pt(child), (4, 3), "two exalted");
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!((t.life(P0), t.life(P1)), (24, 16));
}

#[test]
fn blacklance_paragon_can_target_itself() {
    cr!("115.1", "603.3d");
    ruling!(
        "Blacklance Paragon",
        "Blacklance Paragon can be the target of its own triggered ability."
    );
    supported("Blacklance Paragon");
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    let p = crate::r_s05_common::enter(&mut t, P0, "Blacklance Paragon");
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert_eq!(cands, vec![vec![obj(p)]]);
    t.resolve_all();
    assert!(kw(&t, p, LIFELINK) && kw(&t, p, mtg_engine::keywords::KeywordKind::Deathtouch));
}
