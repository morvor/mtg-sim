//! Rulings batch P179 — "toughness matters": life gain, tokens and counters equal to a
//! creature's toughness (as it last existed on the battlefield, or on resolution),
//! "creature with toughness 4 or greater" triggers, and friends.

use crate::r_p179_common::*;
use mtg_engine::ability::Modification;
use mtg_engine::ability::Value;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Gives the permanent +p/+t until end of turn (as a resolving spell would).
fn pump(t: &mut TestGame, id: ObjectId, p: i32, tt: i32) {
    crate::r_s26_common::modify_until_eot(
        t,
        id,
        vec![Modification::ModifyPT(Value::c(p), Value::c(tt))],
    );
}

/// A Grizzly Bears (2/2) of P0's with +0/+3 until end of turn: 2/5 on the battlefield,
/// 2/2 in the graveyard.
fn pumped_bears(t: &mut TestGame) -> ObjectId {
    let b = t.battlefield(P0, "Grizzly Bears");
    pump(t, b, 0, 3);
    assert_eq!(t.pt(b), (2, 5));
    b
}

#[test]
fn sacrificed_creature_toughness_is_last_known() {
    cr!("608.2h");
    ruling!(
        "Severed Strands",
        "The amount of life gained is equal to the sacrificed creature's toughness as it last existed on the battlefield."
    );
    ruling!(
        "Disciple of Griselbrand",
        "The amount of life you gain is equal to the toughness of the creature as it last existed on the battlefield, not its toughness in the graveyard."
    );
    ruling!(
        "Bushmeat Poacher",
        "Use the sacrificed creature’s toughness as it last existed on the battlefield to determine how much life you gain."
    );
    ruling!(
        "Arguel's Blood Fast // Temple of Aclazotz",
        "For the last ability of Temple of Aclazotz, use the creature's toughness as it last existed on the battlefield to determine how much life you gain."
    );
    ruling!(
        "Scourge of Skola Vale",
        "Use the toughness of the sacrificed creature as it last existed on the battlefield to determine how many +1/+1 counters to put on Scourge of Skola Vale."
    );
    ruling!(
        "Feed the Pack",
        "Check the sacrificed creature’s toughness as it last existed on the battlefield to determine how many Wolves you put onto the battlefield."
    );
    // Severed Strands: an additional cost.
    let mut t = TestGame::new(2);
    let bears = pumped_bears(&mut t);
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.answer_choose(P0, &[obj(bears)]);
    cast_resolve(&mut t, P0, "Severed Strands", &[obj(ogre)]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.life(P0), 25);
    // Activated abilities that sacrifice a creature as a cost: (card, ability text).
    for (name, text) in [
        ("Disciple of Griselbrand", "You gain life"),
        ("Bushmeat Poacher", "You gain life"),
        ("Ayli, Eternal Pilgrim", "You gain life"),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, name);
        let bears = pumped_bears(&mut t);
        t.lands(P0, "Wastes", 1);
        t.answer_choose(P0, &[obj(bears)]);
        activate_containing(&mut t, P0, src, text).unwrap();
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Grizzly Bears"), "{name}");
        assert_eq!(t.life(P0), 25, "{name}");
    }
    // Temple of Aclazotz.
    let mut t = TestGame::new(2);
    let temple = battlefield_back(&mut t, P0, "Arguel's Blood Fast // Temple of Aclazotz");
    assert_eq!(t.obj_now(temple).chars.name.as_str(), "Temple of Aclazotz");
    let bears = pumped_bears(&mut t);
    t.answer_choose(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, temple, "Sacrifice a creature").unwrap();
    // The creature is sacrificed as the ability is activated, before anyone can respond.
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
    // Scourge of Skola Vale: counters.
    supported("Scourge of Skola Vale");
    let mut t = TestGame::new(2);
    let scourge = t.battlefield(P0, "Scourge of Skola Vale");
    crate::r_s29_common::put_counters(&mut t, scourge, "+1/+1", 2);
    let bears = pumped_bears(&mut t);
    t.answer_choose(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, scourge, "Sacrifice another creature").unwrap();
    t.resolve_all();
    assert_eq!(t.counters(scourge, "+1/+1"), 7);
    // Feed the Pack: Wolves at the beginning of the end step.
    supported("Feed the Pack");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Feed the Pack");
    let bears = pumped_bears(&mut t);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(tokens(&t, P0).len(), 5);
}

#[test]
fn temple_of_aclazotz_sacrifice_is_a_cost() {
    cr!("602.2b", "601.2h");
    ruling!(
        "Arguel's Blood Fast // Temple of Aclazotz",
        "Once you've announced the last ability of Temple of Aclazotz, it's too late for anyone to interrupt you by trying to remove the creature you sacrifice."
    );
    let mut t = TestGame::new(2);
    let temple = battlefield_back(&mut t, P0, "Arguel's Blood Fast // Temple of Aclazotz");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, temple, "Sacrifice a creature").unwrap();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.g.stack.len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn bushmeat_poacher_draws_one_card() {
    cr!("608.2c");
    ruling!(
        "Bushmeat Poacher",
        "You draw only one card, no matter how much life you gain."
    );
    let mut t = TestGame::new(2);
    let poacher = t.battlefield(P0, "Bushmeat Poacher");
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.lands(P0, "Wastes", 1);
    t.answer_choose(P0, &[obj(wurm)]);
    let hand = t.hand_size(P0);
    activate_containing(&mut t, P0, poacher, "You gain life").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn feed_the_pack_chooses_on_resolution() {
    cr!("603.5", "608.2c");
    ruling!(
        "Feed the Pack",
        "You choose whether to sacrifice a nontoken creature and which creature to sacrifice when Feed the Pack’s ability resolves."
    );
    let mut t = TestGame::new(2);
    let feed = t.battlefield(P0, "Feed the Pack");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let token = create_token(&mut t, P0, "Spirit");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_from(&t, feed), 1);
    // A creature that enters while the trigger is on the stack can be the one
    // sacrificed: the choice is made on resolution.
    let wurm = t.enter(P0, "Craw Wurm");
    t.settle();
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(wurm)]);
    t.resolve_all();
    let cands: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities { candidates, .. } => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .collect();
    // The token isn't offered (nontoken); the Bears and the Wurm are.
    assert_eq!(cands.len(), 1);
    assert!(cands[0].contains(&obj(bears)) && !cands[0].contains(&obj(token)));
    assert!(t.in_graveyard(P0, "Craw Wurm"));
    assert_eq!(tokens(&t, P0).len(), 5);
}

#[test]
fn angelic_chorus_ignores_phasing_control_changes_and_animation() {
    cr!("702.26d", "603.6a");
    ruling!(
        "Angelic Chorus",
        "Angelic Chorus does not trigger when creatures phase in or change controllers."
    );
    ruling!(
        "Angelic Chorus",
        "This does not trigger on a permanent being turned into a creature."
    );
    supported("Angelic Chorus");
    let mut t = TestGame::new(2);
    let chorus = t.battlefield(P0, "Angelic Chorus");
    let wurm = t.battlefield(P0, "Craw Wurm");
    mtg_engine::kw::phasing::phase_out(&mut t.g, vec![wurm]);
    t.settle();
    mtg_engine::kw::phasing::phase_in(&mut t.g, wurm);
    t.g.flush_events();
    t.settle();
    assert!(t.on_battlefield(wurm) && !t.obj_now(wurm).phased_out);
    let ogre = t.battlefield(P1, "Gray Ogre");
    give_control(&mut t, ogre, P0);
    let factory = t.battlefield(P0, "Mishra's Factory");
    t.lands(P0, "Wastes", 1);
    activate_containing(&mut t, P0, factory, "becomes").unwrap();
    t.resolve_all();
    assert!(t.obj_now(factory).is(mtg_engine::types::CardType::Creature));
    assert_eq!(triggers_from(&t, chorus), 0);
    assert_eq!(t.life(P0), 20);
    // Control: a creature entering.
    t.enter(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}

#[test]
fn arnyn_checks_power_and_toughness_as_the_creature_last_existed() {
    cr!("603.10a");
    ruling!(
        "Arnyn, Deathbloom Botanist",
        "Arnyn's last ability cares about the power or toughness of a creature that died as it last existed on the battlefield."
    );
    supported("Arnyn, Deathbloom Botanist");
    // (creature, +p/+t before it dies, triggers): a shrunk 2/2 (1/1) triggers; a pumped
    // 1/1 (3/3) doesn't, though it's 1/1 in the graveyard.
    for (name, p, tt, triggers) in [
        ("Grizzly Bears", -1, -1, true),
        ("Llanowar Elves", 2, 2, false),
        ("Grizzly Bears", 0, 0, false),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Arnyn, Deathbloom Botanist");
        let c = t.battlefield(P0, name);
        pump(&mut t, c, p, tt);
        destroy(&mut t, c);
        t.resolve_all();
        assert_eq!(t.life(P1), if triggers { 18 } else { 20 }, "{name} {p}");
    }
}

#[test]
fn bighorner_rancher_mana_ability_adds_greatest_power() {
    cr!("605.1a", "605.3a", "106.4");
    ruling!(
        "Bighorner Rancher",
        "Bighorner Rancher’s second ability is a mana ability. It doesn’t use the stack and can’t be responded to. If the greatest power among creatures you control is somehow 0 or less, no mana is added."
    );
    supported("Bighorner Rancher");
    use mtg_engine::mana::ManaType;
    let mut t = TestGame::new(2);
    let rancher = t.battlefield(P0, "Bighorner Rancher");
    t.battlefield(P0, "Hill Giant");
    assert!(crate::r_s20_common::tap_for_mana(
        &mut t, P0, rancher, "Add"
    ));
    assert!(t.g.stack.is_empty());
    assert_eq!(crate::r_s24_common::pool(&t, P0, ManaType::G), 3);
    // Greatest power 0: no mana.
    let mut t = TestGame::new(2);
    let rancher = t.battlefield(P0, "Bighorner Rancher");
    pump(&mut t, rancher, -2, 0);
    crate::r_s20_common::tap_for_mana(&mut t, P0, rancher, "Add");
    assert_eq!(crate::r_s24_common::pool(&t, P0, ManaType::G), 0);
    assert!(t.g.player(P0).mana_pool.mana.is_empty());
}

#[test]
fn bighorner_rancher_life_is_determined_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Bighorner Rancher",
        "The amount of life you gain from Bighorner Rancher’s last ability is determined when the ability resolves."
    );
    for respond in [false, true] {
        let mut t = TestGame::new(2);
        let rancher = t.battlefield(P0, "Bighorner Rancher");
        let giant = t.battlefield(P0, "Hill Giant");
        activate_containing(&mut t, P0, rancher, "You gain life").unwrap();
        assert!(t.in_graveyard(P0, "Bighorner Rancher"));
        if respond {
            destroy(&mut t, giant);
        }
        t.resolve_all();
        assert_eq!(t.life(P0), if respond { 20 } else { 23 });
    }
}

#[test]
fn purging_scythe_doesnt_target() {
    cr!("115.10", "702.11b");
    ruling!("Purging Scythe", "Does not target the creature it damages.");
    supported("Purging Scythe");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Purging Scythe");
    t.battlefield(P0, "Hill Giant");
    let scout = t.battlefield(P1, "Gladecover Scout");
    let from = t.asked().len();
    crate::r_s04_common::next_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(
        t.in_graveyard(P1, "Gladecover Scout"),
        "{:?}",
        t.zone(scout)
    );
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::ChooseTargets { .. })));
}

/// P0 with Cleric Class at level 2 (and mana spent on it), in the beginning of combat.
fn cleric_class_level_2(t: &mut TestGame) -> ObjectId {
    supported("Cleric Class");
    let class = t.battlefield(P0, "Cleric Class");
    t.lands(P0, "Plains", 4);
    crate::r_s19_common::gain_level(t, P0, class, 2).unwrap();
    t.resolve_all();
    assert_eq!(crate::r_s19_common::level(t, class), 2);
    class
}

/// +1/+1 counters on P0's creatures.
fn p0_counters(t: &TestGame) -> u32 {
    t.g.permanents()
        .filter(|o| o.controller == P0)
        .map(|o| o.counter("+1/+1"))
        .sum()
}

#[test]
fn cleric_class_counts_each_lifelink_creatures_damage_as_one_gain() {
    cr!("702.15b", "119.9", "510.2");
    ruling!(
        "Cleric Class",
        "Each creature with lifelink dealing combat damage causes a separate life gain event."
    );
    // Two Vampire Nighthawks unblocked: two gains of 2+1, two triggers.
    let mut t = TestGame::new(2);
    cleric_class_level_2(&mut t);
    let a = t.battlefield(P0, "Vampire Nighthawk");
    let b = t.battlefield(P0, "Vampire Nighthawk");
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
    assert_eq!(p0_counters(&t), 2);
    // One Child of Night (2/1 lifelink) blocked by two Llanowar Elves, dealing 1 to
    // each: one gain of 2+1, one trigger (its counter goes on the Bears).
    let mut t = TestGame::new(2);
    cleric_class_level_2(&mut t);
    let a = t.battlefield(P0, "Child of Night");
    t.battlefield(P0, "Grizzly Bears");
    let e1 = t.battlefield(P1, "Llanowar Elves");
    let e2 = t.battlefield(P1, "Llanowar Elves");
    t.answer(
        P0,
        DecisionKind::Damage,
        mtg_engine::decision::Answer::Numbers(vec![1, 1]),
    );
    t.attack(&[(a, Entity::Player(P1))], &[(e1, a), (e2, a)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    assert_eq!(t.life(P0), 23);
    assert_eq!(p0_counters(&t), 1);
}

#[test]
fn cleric_class_creature_dealt_lethal_damage_dies_before_the_counter() {
    cr!("704.5g", "603.3");
    ruling!(
        "Cleric Class",
        "If you gain life at the same time a creature is dealt lethal damage, it dies before Cleric Class's level 2 ability can put a +1/+1 counter on it."
    );
    let mut t = TestGame::new(2);
    cleric_class_level_2(&mut t);
    let child = t.battlefield(P0, "Child of Night");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.attack(&[(child, Entity::Player(P1))], &[(giant, child)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Child of Night"));
    assert_eq!(t.life(P0), 23);
    // The counter could only go on the surviving Bears.
    assert_eq!(t.counters(bears, "+1/+1"), 1);
}

#[test]
fn cleric_class_level_3_uses_toughness_on_the_battlefield() {
    cr!("716.2a", "608.2h");
    interp!("returned-card-values-current-on-the-battlefield");
    ruling!(
        "Cleric Class",
        "When Cleric Class's level 3 ability resolves, you gain life equal to the creature's toughness as it exists on battlefield, which may be different than its toughness in the graveyard."
    );
    let mut t = TestGame::new(2);
    let class = cleric_class_level_2(&mut t);
    // Scourge of Skola Vale is 0/0 in the graveyard and enters with two +1/+1 counters.
    let scourge = t.graveyard(P0, "Scourge of Skola Vale");
    t.lands(P0, "Plains", 5);
    t.answer_targets(P0, &[obj(scourge)]);
    crate::r_s19_common::gain_level(&mut t, P0, class, 3).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(scourge));
    // 2 life, plus 1 from the level 1 ability.
    assert_eq!(t.life(P0), 23);
}

#[test]
fn gate_smasher_falls_off_below_toughness_4() {
    cr!("704.5n");
    ruling!(
        "Gate Smasher",
        "If Gate Smasher is attached to a creature whose toughness falls below 4, Gate Smasher will become unattached as a state-based action."
    );
    supported("Gate Smasher");
    let mut t = TestGame::new(2);
    let smasher = t.battlefield(P0, "Gate Smasher");
    let flayer = t.battlefield(P0, "Sultai Flayer");
    t.lands(P0, "Wastes", 3);
    t.answer_targets(P0, &[obj(flayer)]);
    activate_containing(&mut t, P0, smasher, "Equip").unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(smasher).attached_to, Some(obj(flayer)));
    assert_eq!(t.pt(flayer), (6, 4));
    pump(&mut t, flayer, 0, -1);
    assert!(t.obj_now(smasher).attached_to.is_none());
    assert!(t.on_battlefield(smasher));
}

#[test]
fn toughness_4_death_triggers_see_the_dying_creature_itself() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Sultai Flayer",
        "If Sultai Flayer dies, and its toughness is still 4 or greater, its own ability will trigger."
    );
    ruling!(
        "Ulvenwald Observer",
        "If Ulvenwald Observer’s toughness is still 4 or greater at the time it dies, its ability triggers."
    );
    ruling!(
        "Ulvenwald Observer",
        "If Ulvenwald Observer dies at the same time as other creatures with toughness 4 or greater, its ability will trigger for each of those creatures."
    );
    supported("Sultai Flayer");
    supported("Ulvenwald Observer");
    // Sultai Flayer (3/4) dies: 4 life; shrunk to 3/3 first: nothing.
    for shrink in [false, true] {
        let mut t = TestGame::new(2);
        let f = t.battlefield(P0, "Sultai Flayer");
        if shrink {
            pump(&mut t, f, 0, -1);
        }
        destroy(&mut t, f);
        t.resolve_all();
        assert_eq!(t.life(P0), if shrink { 20 } else { 24 });
    }
    // Ulvenwald Observer alone: one card.
    let mut t = TestGame::new(2);
    let o = t.battlefield(P0, "Ulvenwald Observer");
    let hand = t.hand_size(P0);
    destroy(&mut t, o);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // With Craw Wurm (6/4) and Grizzly Bears (2/2), destroyed together: two cards.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ulvenwald Observer");
    t.battlefield(P0, "Craw Wurm");
    t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    cast_resolve(&mut t, P0, "Wrath of God", &[]);
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn glint_weaver_distribution_is_kept_when_a_target_becomes_illegal() {
    cr!("601.2d", "608.2b");
    ruling!(
        "Glint Weaver",
        "the original distribution of counters still applies and the counters that would have been put on the illegal targets are lost."
    );
    ruling!(
        "Glint Weaver",
        "You choose how many targets Glint Weaver's triggered ability has and how the counters are distributed as you put the ability onto the stack. Each target must receive at least one counter."
    );
    supported("Glint Weaver");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Gray Ogre");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(a), obj(b)]);
    crate::r_s29_common::divide(&mut t, P0, &[2, 1]);
    let weaver = t.enter(P0, "Glint Weaver");
    t.settle();
    assert_eq!(triggers_from(&t, weaver), 1);
    // The division was asked as the ability was put on the stack, each getting >= 1.
    let divides: Vec<(u32, u32)> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::Divide {
                total, min_each, ..
            } => Some((*total, *min_each)),
            _ => None,
        })
        .collect();
    assert_eq!(divides, vec![(3, 1)]);
    // Bears (to get 2) is destroyed in response: the Ogre still gets only 1.
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(t.counters(b, "+1/+1"), 1);
    assert_eq!(t.counters(weaver, "+1/+1"), 0);
    // Then life equal to the greatest toughness: Glint Weaver's 3.
    assert_eq!(t.life(P0), 23);
}

#[test]
fn severed_strands_target_legality() {
    cr!("608.2b", "702.12b");
    ruling!(
        "Severed Strands",
        "If the target creature is a legal target but can't be destroyed, most likely because it has indestructible, you still gain life."
    );
    ruling!(
        "Severed Strands",
        "If the target creature is an illegal target by the time Severed Strands tries to resolve, the spell doesn't resolve. You don't gain life."
    );
    supported("Severed Strands");
    // An indestructible target: life is still gained.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let myr = t.battlefield(P1, "Darksteel Myr");
    t.answer_choose(P0, &[obj(bears)]);
    cast_resolve(&mut t, P0, "Severed Strands", &[obj(myr)]);
    assert!(t.on_battlefield(myr));
    assert_eq!(t.life(P0), 22);
    // The target leaves before it resolves: no life (the creature stays sacrificed).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.answer_choose(P0, &[obj(bears)]);
    cast_targeting(&mut t, P0, "Severed Strands", &[obj(ogre)]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    destroy(&mut t, ogre);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn kin_tree_invocation_x_once_and_a_0_0_token() {
    cr!("608.2h", "704.5f");
    ruling!(
        "Kin-Tree Invocation",
        "If you control no creatures as Kin-Tree Invocation resolves, you’ll put a 0/0 token onto the battlefield."
    );
    ruling!(
        "Kin-Tree Invocation",
        "The value of X is determined only once, as Kin-Tree Invocation resolves."
    );
    supported("Kin-Tree Invocation");
    supported("Impact Tremors");
    supported("Grave Pact");
    // No creatures: the 0/0 token enters (Impact Tremors deals 1 to P1) and dies (Grave
    // Pact makes P1 sacrifice their Gray Ogre).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Impact Tremors");
    t.battlefield(P0, "Grave Pact");
    t.battlefield(P1, "Gray Ogre");
    cast_resolve(&mut t, P0, "Kin-Tree Invocation", &[]);
    assert!(tokens(&t, P0).is_empty());
    assert_eq!(t.life(P1), 19);
    assert!(t.in_graveyard(P1, "Gray Ogre"));
    // X fixed: Craw Wurm (toughness 4) gives a 4/4 that stays 4/4 when the Wurm dies.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    cast_resolve(&mut t, P0, "Kin-Tree Invocation", &[]);
    let token = tokens(&t, P0)[0];
    assert_eq!(t.pt(token), (4, 4));
    destroy(&mut t, wurm);
    assert_eq!(t.pt(token), (4, 4));
}

#[test]
fn arguels_blood_fast_checks_life_at_upkeep_and_on_resolution() {
    cr!("603.4", "503.1a");
    ruling!(
        "Arguel's Blood Fast // Temple of Aclazotz",
        "If your life total is 6 or greater as your upkeep begins, the second ability of Arguel's Blood Fast won't trigger."
    );
    ruling!(
        "Arguel's Blood Fast // Temple of Aclazotz",
        "If your life total is 6 or greater as the second ability of Arguel's Blood Fast tries to resolve, it won't do anything."
    );
    supported("Arguel's Blood Fast // Temple of Aclazotz");
    // (life at upkeep, gain in response, triggers, transforms)
    for (life, gain, triggers, transforms) in [
        (6, 0, false, false),
        (5, 0, true, true),
        (5, 1, true, false),
    ] {
        let mut t = TestGame::new(2);
        let fast = t.battlefield(P0, "Arguel's Blood Fast // Temple of Aclazotz");
        t.advance_to(P1, Step::Upkeep);
        set_life(&mut t, P0, life);
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        assert_eq!(triggers_from(&t, fast) == 1, triggers, "{life} {gain}");
        set_life(&mut t, P0, life + gain);
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(
            t.obj_now(fast).chars.name.as_str() == "Temple of Aclazotz",
            transforms,
            "{life} {gain}"
        );
    }
}

/// P0 has Ayli, Grizzly Bears to sacrifice and mana for the exile ability; P1 has a Gray
/// Ogre to exile. Returns (Ayli, Bears, Ogre).
fn ayli_setup(t: &mut TestGame) -> (ObjectId, ObjectId, ObjectId) {
    supported("Ayli, Eternal Pilgrim");
    let ayli = t.battlefield(P0, "Ayli, Eternal Pilgrim");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 1);
    (ayli, bears, ogre)
}

#[test]
fn ayli_checks_the_team_life_total_in_two_headed_giant() {
    cr!("810.9a", "810.4");
    ruling!(
        "Ayli, Eternal Pilgrim",
        "In a Two-Headed Giant game, Ayli's last ability checks your team's life total."
    );
    // The team starts at 30: it needs 40.
    for (life, ok) in [(40, true), (39, false)] {
        let mut t = two_headed_giant();
        assert_eq!(t.life(P0), 30);
        let (ayli, _, _) = ayli_setup(&mut t);
        set_life(&mut t, P0, life);
        set_life(&mut t, P1, life);
        let act = crate::r_s27_common::can_activate_containing(&mut t, P0, ayli, "Exile target");
        assert_eq!(act, ok, "{life}");
    }
}

#[test]
fn ayli_exile_resolves_whatever_happens_to_your_life() {
    cr!("602.5");
    ruling!(
        "Ayli, Eternal Pilgrim",
        "Once you've activated Ayli's last ability, it doesn't matter what happens to your life total."
    );
    let mut t = TestGame::new(2);
    let (ayli, bears, ogre) = ayli_setup(&mut t);
    set_life(&mut t, P0, 30);
    t.answer_targets(P0, &[obj(ogre)]);
    t.answer_choose(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, ayli, "Exile target").unwrap();
    set_life(&mut t, P0, 5);
    t.resolve_all();
    assert_eq!(t.zone(ogre), Zone::Exile);
    // Control: below 30 it can't be activated at all.
    let mut t = TestGame::new(2);
    let (ayli, _, _) = ayli_setup(&mut t);
    set_life(&mut t, P0, 29);
    assert!(!crate::r_s27_common::can_activate_containing(
        &mut t,
        P0,
        ayli,
        "Exile target"
    ));
}

#[test]
fn abzan_beastmaster_checks_toughness_on_resolution() {
    cr!("608.2c", "603.4");
    ruling!(
        "Abzan Beastmaster",
        "The ability checks the creatures on the battlefield as it resolves to determine if you will draw a card."
    );
    supported("Abzan Beastmaster");
    // (P0's Grizzly Bears pumped in response, P1's Hill Giant pumped in response, draws)
    for (mine, theirs, draws) in [(0, 0, false), (3, 0, true), (1, 0, true), (2, 3, false)] {
        let mut t = TestGame::new(2);
        let bm = t.battlefield(P0, "Abzan Beastmaster");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        t.advance_to(P1, Step::Upkeep);
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        // It triggers even though P0 doesn't control the toughest creature now.
        assert_eq!(triggers_from(&t, bm), 1);
        pump(&mut t, bears, 0, mine);
        pump(&mut t, giant, 0, theirs);
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0) > hand, draws, "{mine} {theirs}");
    }
}

#[test]
fn verdant_suns_avatar_uses_toughness_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Verdant Sun's Avatar",
        "The entering creature's toughness is determined as the ability of Verdant Sun's Avatar resolves. If that creature has left the battlefield, use its toughness as it last existed on the battlefield. If the creature's toughness was less than 0, your life total won't change."
    );
    supported("Verdant Sun's Avatar");
    // (+toughness in response, destroyed in response, life gained)
    for (dt, kill, gained) in [(0, false, 2), (3, false, 5), (3, true, 5), (-5, false, 0)] {
        let mut t = TestGame::new(2);
        let avatar = t.battlefield(P0, "Verdant Sun's Avatar");
        let bears = t.enter(P0, "Grizzly Bears");
        t.settle();
        assert_eq!(triggers_from(&t, avatar), 1);
        pump(&mut t, bears, 0, dt);
        if kill {
            destroy(&mut t, bears);
        }
        t.resolve_all();
        assert_eq!(t.life(P0), 20 + gained, "{dt} {kill}");
    }
}

#[test]
fn bioplasm_uses_the_exiled_cards_characteristic_defining_ability() {
    cr!("604.3", "208.2a");
    ruling!(
        "Bioplasm",
        "Abilities that define a creature's power (and toughness) apply in every zone."
    );
    supported("Bioplasm");
    supported("Nightmare");
    let mut t = TestGame::new(2);
    let bio = t.battlefield(P0, "Bioplasm");
    t.lands(P0, "Swamp", 3);
    t.library_top(P0, "Nightmare");
    t.attack(&[(bio, Entity::Player(P1))], &[]);
    assert!(t.in_exile("Nightmare"));
    // Nightmare is 3/3 in exile (three Swamps): Bioplasm dealt 7.
    assert_eq!(t.life(P1), 13);
    assert_eq!(t.pt(bio), (7, 7));
}

#[test]
fn predators_rapport_uses_power_and_toughness_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Predator's Rapport",
        "Use the creature’s power and toughness when Predator’s Rapport resolves to determine how much life you gain."
    );
    supported("Predator's Rapport");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_targeting(&mut t, P0, "Predator's Rapport", &[obj(bears)]);
    pump(&mut t, bears, 1, 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 27);
}

#[test]
fn fruit_of_the_first_tree_uses_last_known_toughness() {
    cr!("603.10a", "107.1b");
    ruling!(
        "Fruit of the First Tree",
        "Use the creature’s toughness when it left the battlefield to determine the value of X. If that number is 0 or less, you won’t gain life or draw cards."
    );
    supported("Fruit of the First Tree");
    for (dt, x) in [(3, 5), (-2, 0), (-4, 0)] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        crate::r_s06_common::attach_new(&mut t, P0, "Fruit of the First Tree", bears);
        let hand = t.hand_size(P0);
        pump(&mut t, bears, 0, dt);
        if t.on_battlefield(bears) {
            destroy(&mut t, bears);
        }
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Grizzly Bears"));
        assert_eq!(t.life(P0), 20 + x as i32, "{dt}");
        assert_eq!(t.hand_size(P0), hand + x, "{dt}");
    }
}

#[test]
fn wolverine_riders_uses_the_elfs_toughness_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Wolverine Riders",
        "Use the toughness of the creature as the last ability resolves to determine how much life you gain. If the Elf is no longer on the battlefield at that time, use its toughness from when it was last on the battlefield."
    );
    supported("Wolverine Riders");
    for (dt, kill, gained) in [(0, false, 1), (3, false, 4), (3, true, 4)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Wolverine Riders");
        let elf = t.enter(P0, "Llanowar Elves");
        t.settle();
        pump(&mut t, elf, 0, dt);
        if kill {
            destroy(&mut t, elf);
        }
        t.resolve_all();
        assert_eq!(t.life(P0), 20 + gained, "{dt} {kill}");
    }
}

#[test]
fn arwen_counts_her_toughness_as_the_creature_enters() {
    cr!("614.12", "614.1c");
    ruling!(
        "Arwen, Weaver of Hope",
        "To determine how many additional +1/+1 counters a creature enters the battlefield with, use Arwen, Weaver of Hope's toughness as that creature is entering the battlefield."
    );
    ruling!(
        "Arwen, Weaver of Hope",
        "If Arwen, Weaver of Hope and another creature you control enter the battlefield at the same time, Arwen's ability won't cause that creature to enter with additional +1/+1 counters on it."
    );
    supported("Arwen, Weaver of Hope");
    let mut t = TestGame::new(2);
    let arwen = t.battlefield(P0, "Arwen, Weaver of Hope");
    pump(&mut t, arwen, 0, 2);
    let bears = t.enter(P0, "Grizzly Bears");
    assert_eq!(t.counters(bears, "+1/+1"), 3);
    // Entering together with Arwen: no counters.
    let mut t = TestGame::new(2);
    let v = enter_together(
        &mut t,
        &[(P0, "Arwen, Weaver of Hope"), (P0, "Grizzly Bears")],
    );
    assert!(t.on_battlefield(v[0]));
    assert_eq!(t.counters(v[1], "+1/+1"), 0);
}
