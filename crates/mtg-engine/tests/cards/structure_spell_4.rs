//! Structure-coverage tests (round 4): spell-ability structures shared by many cards that
//! no other test exercised (see `docs/STRUCTURE_COVERAGE.md`).

use super::structure_activated_2::{next_turn, target_choices};
use super::structure_seed_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::planechase;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{Color, ColorSet};
use mtg_engine::*;

#[test]
fn grotesque_mutation_pumps_and_grants_lifelink() {
    cr!("608.2", "611.2a", "702.15b");
    // "Target creature gets +3/+1 and gains lifelink until end of turn."
    supported("Grotesque Mutation");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Swamp", 2);
    let s = t.hand(P0, "Grotesque Mutation");
    t.cast(P0, s).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 3));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Lifelink));
    assert!(!t.obj_now(other).has_keyword(KeywordKind::Lifelink));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.life(P0), 25, "lifelink");
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Lifelink));
}

#[test]
fn bite_down_deals_damage_equal_to_power_one_way() {
    cr!("608.2", "115.1", "120.3e", "120.3c");
    // "Target creature you control deals damage equal to its power to target creature or
    // planeswalker you don't control."
    supported("Bite Down");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Craw Wurm");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let pw = t.battlefield(P1, "Garruk Wildspeaker");
    t.lands(P0, "Forest", 2);
    let s = t.hand(P0, "Bite Down");
    t.cast(P0, s)
        .targets(&[Entity::Object(giant), Entity::Object(theirs)])
        .go();
    let c = target_choices(&t, P0);
    assert!(c[0].contains(&Entity::Object(mine)) && !c[0].contains(&Entity::Object(elves)));
    assert!(c[1].contains(&Entity::Object(elves)) && c[1].contains(&Entity::Object(pw)));
    assert!(!c[1].contains(&Entity::Object(mine)), "a creature you control");
    t.resolve_all();
    assert_eq!(t.obj_now(theirs).damage, 3);
    assert_eq!(t.obj_now(giant).damage, 0, "it isn't a fight");
    assert_eq!(t.obj_now(elves).damage, 0);
    // A planeswalker loses loyalty.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let pw = t.battlefield(P1, "Garruk Wildspeaker");
    t.lands(P0, "Forest", 2);
    let s = t.hand(P0, "Bite Down");
    t.cast(P0, s)
        .targets(&[Entity::Object(giant), Entity::Object(pw)])
        .go();
    t.resolve_all();
    assert!(!t.on_battlefield(pw), "3 damage to a 3-loyalty planeswalker");
}

#[test]
fn make_obsolete_shrinks_creatures_your_opponents_control() {
    cr!("608.2", "611.2a", "611.2c");
    // "Creatures your opponents control get -1/-1 until end of turn."
    supported("Make Obsolete");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Swamp", 3);
    let s = t.hand(P0, "Make Obsolete");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (1, 1));
    assert!(!t.on_battlefield(elves), "a 1/1 dies");
    assert_eq!(t.pt(mine), (2, 2));
    // Locked in: a creature that enters later isn't affected (CR 611.2c).
    let late = t.battlefield(P1, "Hill Giant");
    assert_eq!(t.pt(late), (3, 3));
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn inspirit_untaps_and_pumps() {
    cr!("608.2", "701.26b", "611.2a");
    // "Untap target creature. It gets +2/+4 until end of turn."
    supported("Inspirit");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    t.g.tap(bears);
    t.g.tap(other);
    t.lands(P0, "Plains", 3);
    let s = t.hand(P0, "Inspirit");
    t.cast(P0, s).target(bears).go();
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
    assert!(t.obj_now(other).tapped);
    assert_eq!(t.pt(bears), (4, 6));
    assert_eq!(t.pt(other), (3, 3));
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn entropic_decay_mills_four() {
    cr!("608.2", "701.17a", "715.3d");
    // Gray Slaad // Entropic Decay: "Mill four cards." (an Adventure)
    let name = "Gray Slaad // Entropic Decay";
    supported(name);
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let card = t.hand(P0, name);
    let lib = t.library_size(P0);
    let spell = t.cast(P0, card).method(CastMethod::Half(1)).go();
    assert_eq!(t.obj_now(spell).chars.name.as_str(), "Entropic Decay");
    t.resolve_all();
    assert_eq!(t.library_size(P0), lib - 4);
    assert_eq!(t.graveyard_size(P0), 4);
    assert_eq!(t.library_size(P1), 30);
    assert!(t.in_exile("Gray Slaad"), "an Adventure goes to exile");
}

/// A Planechase game where P0 has a planar deck of two planes.
fn planechase() -> TestGame {
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Planechase,
            ..Default::default()
        },
    );
    for n in ["Goldmeadow", "Krosa"] {
        let id = t.command(P0, n);
        t.g.objects[id.0 as usize].face_down = true;
    }
    t.g.recompute();
    planechase::set_starting_plane(&mut t.g);
    t
}

fn custom_events(t: &TestGame, name: &str) -> usize {
    t.g.turn_events
        .iter()
        .chain(t.g.events.iter())
        .filter(|e| matches!(e, Event::Custom { name: n, .. } if n.as_str() == name))
        .count()
}

fn face_up_planes(t: &TestGame) -> Vec<String> {
    t.g.command
        .iter()
        .filter(|id| !t.obj(**id).face_down)
        .map(|id| t.obj(*id).chars.name.to_string())
        .collect()
}

#[test]
fn will_of_the_planeswalkers_votes_planeswalk_or_chaos() {
    cr!("701.38a", "701.31b", "311.7");
    // Path of the Enigma: "Target player draws four cards. Will of the Planeswalkers —
    // Starting with you, each player votes for planeswalk or chaos. If planeswalk gets
    // more votes, planeswalk. If chaos gets more votes or the vote is tied, chaos ensues."
    supported("Path of the Enigma");
    // (P0's vote, P1's vote) as option indices (0 planeswalk, 1 chaos) -> planeswalked?
    // Both tie orders: a result that only counted one player's vote would fail one of them.
    for (v0, v1, walks) in [(0, 0, true), (1, 1, false), (0, 1, false), (1, 0, false)] {
        let mut t = planechase();
        assert_eq!(face_up_planes(&t), vec!["Goldmeadow"]);
        t.lands(P0, "Island", 5);
        let s = t.hand(P0, "Path of the Enigma");
        t.cast(P0, s).target(Entity::Player(P1)).go();
        t.answer(P0, DecisionKind::Option, Answer::Index(v0));
        t.answer(P1, DecisionKind::Option, Answer::Index(v1));
        t.resolve();
        // Each player votes once, starting with the caster, between the two words.
        let votes: Vec<_> = t
            .asked()
            .into_iter()
            .filter_map(|(p, d)| match d {
                Decision::ChooseOption { options, .. } => Some((p, options)),
                _ => None,
            })
            .collect();
        let words = vec!["planeswalk".to_string(), "chaos".to_string()];
        assert_eq!(votes, vec![(P0, words.clone()), (P1, words)]);
        assert_eq!(t.hand_size(P1), 4, "the target draws four");
        assert_eq!(t.hand_size(P0), 0);
        let walked = custom_events(&t, planechase::PLANESWALKED);
        let chaos = custom_events(&t, planechase::CHAOS_ENSUES);
        assert_eq!(walked, usize::from(walks), "votes {v0} {v1}");
        assert_eq!(chaos, usize::from(!walks), "votes {v0} {v1}");
        if walks {
            assert_eq!(face_up_planes(&t), vec!["Krosa"]);
        } else {
            assert_eq!(face_up_planes(&t), vec!["Goldmeadow"]);
            // Goldmeadow's chaos ability: "create a 0/1 white Goat creature token".
            t.resolve_all();
            assert_eq!(count_subtype(&t, P0, "Goat"), 1);
        }
    }
}

#[test]
fn pillar_of_flame_exiles_a_creature_it_damaged_that_would_die() {
    cr!("608.2", "120.3e", "614.1a");
    // "Pillar of Flame deals 2 damage to any target. If a creature dealt damage this way
    // would die this turn, exile it instead."
    supported("Pillar of Flame");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let s = t.hand(P0, "Pillar of Flame");
    t.cast(P0, s).target(bears).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.in_exile("Grizzly Bears"));
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
    // A creature that survives the damage but dies later this turn is exiled too.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let other = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let s = t.hand(P0, "Pillar of Flame");
    t.cast(P0, s).target(giant).go();
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    t.g.destroy(giant, None);
    t.g.destroy(other, None);
    t.settle();
    assert!(t.in_exile("Hill Giant"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "not dealt damage this way");
    // Any target: a player.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let s = t.hand(P0, "Pillar of Flame");
    t.cast(P0, s).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn destroy_target_creature_and_its_controller_loses_life() {
    cr!("608.2", "701.8a", "119.3");
    // Bitter Downfall: "Destroy target creature. Its controller loses 2 life."
    supported("Bitter Downfall");
    for owner in [P1, P0] {
        let mut t = TestGame::new(2);
        let victim = t.battlefield(owner, "Hill Giant");
        let other = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Swamp", 4);
        let s = t.hand(P0, "Bitter Downfall");
        t.cast(P0, s).target(victim).go();
        t.resolve_all();
        assert!(!t.on_battlefield(victim));
        assert!(t.on_battlefield(other));
        assert_eq!(t.life(owner), 18, "its controller");
        let other_player = if owner == P0 { P1 } else { P0 };
        assert_eq!(t.life(other_player), 20);
    }
}

#[test]
fn venoms_hunger_destroys_and_gains_life() {
    cr!("608.2", "701.8a", "119.3");
    // "Destroy target creature. You gain 2 life."
    supported("Venom's Hunger");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let other = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 5);
    let s = t.hand(P0, "Venom's Hunger");
    t.cast(P0, s).target(giant).go();
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert!(t.on_battlefield(other));
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn bestial_incursion_makes_a_trampling_beast() {
    cr!("608.2", "111.1", "701.7a");
    // "Create a 4/4 green Beast creature token with trample."
    supported("Bestial Incursion");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    let s = t.hand(P0, "Bestial Incursion");
    t.cast(P0, s).go();
    t.resolve_all();
    let b = subtype_ids(&t, P0, "Beast");
    assert_eq!(b.len(), 1);
    let o = t.obj_now(b[0]);
    assert!(o.is_token());
    assert_eq!(t.pt(b[0]), (4, 4));
    assert_eq!(o.chars.colors, ColorSet::single(Color::Green));
    assert!(o.has_keyword(KeywordKind::Trample));
    assert_eq!(count_subtype(&t, P1, "Beast"), 0);
}

#[test]
fn savage_smash_pumps_then_fights() {
    cr!("608.2", "701.14a", "611.2a");
    // "Target creature you control gets +2/+2 until end of turn. It fights target
    // creature you don't control."
    supported("Savage Smash");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let other = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Taiga", 3);
    let s = t.hand(P0, "Savage Smash");
    t.cast(P0, s)
        .targets(&[Entity::Object(bears), Entity::Object(giant)])
        .go();
    let c = target_choices(&t, P0);
    assert!(c[0].contains(&Entity::Object(bears)) && !c[0].contains(&Entity::Object(giant)));
    assert!(c[1].contains(&Entity::Object(giant)) && !c[1].contains(&Entity::Object(bears)));
    t.resolve_all();
    assert!(!t.on_battlefield(giant), "4 damage to a 3/3");
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.obj_now(bears).damage, 3, "it survives the fight because of the pump");
    assert_eq!(t.obj_now(other).damage, 0);
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn minor_misstep_counters_only_a_cheap_spell() {
    cr!("608.2", "701.6a", "115.1", "202.3");
    // "Counter target spell with mana value 1 or less."
    supported("Minor Misstep");
    let mut t = TestGame::new(2);
    t.lands(P1, "Plateau", 3);
    t.lands(P0, "Island", 1);
    let helix = t.hand(P1, "Lightning Helix");
    let bolt = t.hand(P1, "Lightning Bolt");
    let helix = t.cast_with(P1, helix, &[Entity::Player(P0)]).unwrap();
    let bolt = t.cast_with(P1, bolt, &[Entity::Player(P0)]).unwrap();
    let m = t.hand(P0, "Minor Misstep");
    t.cast_with(P0, m, &[Entity::Object(bolt)]).unwrap();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(bolt)));
    assert!(!c.contains(&Entity::Object(helix)), "mana value 2");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert_eq!(t.life(P0), 17, "only the Helix resolved");
    assert_eq!(t.life(P1), 23);
}

#[test]
fn justice_strike_makes_a_creature_damage_itself() {
    cr!("608.2", "120.3e", "704.5g", "702.15b", "702.2b");
    // "Target creature deals damage to itself equal to its power."
    supported("Justice Strike");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.lands(P0, "Plateau", 2);
    let s = t.hand(P0, "Justice Strike");
    t.cast(P0, s).target(giant).go();
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.obj_now(wall).damage, 0);
    // The creature is the source of the damage: its lifelink and deathtouch apply.
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P1, "Vampire Nighthawk");
    t.lands(P0, "Plateau", 2);
    let s = t.hand(P0, "Justice Strike");
    t.cast(P0, s).target(hawk).go();
    t.resolve_all();
    assert!(!t.on_battlefield(hawk), "2 deathtouch damage to a 2/3");
    assert_eq!(t.life(P1), 22, "its controller gains life from its lifelink");
    assert_eq!(t.life(P0), 20);
    // A 0-power creature deals no damage.
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P1, "Wall of Stone");
    t.lands(P0, "Plateau", 2);
    let s = t.hand(P0, "Justice Strike");
    t.cast(P0, s).target(wall).go();
    t.resolve_all();
    assert_eq!(t.obj_now(wall).damage, 0);
}

#[test]
fn resounding_silence_exiles_an_attacking_creature() {
    cr!("608.2", "701.13a", "115.1");
    // "Exile target attacking creature."
    supported("Resounding Silence");
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Hill Giant");
    let idle = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Plains", 4);
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(attacker, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    let s = t.hand(P1, "Resounding Silence");
    t.cast_with(P1, s, &[Entity::Object(attacker)]).unwrap();
    let c = target_choices(&t, P1).pop().unwrap();
    assert!(c.contains(&Entity::Object(attacker)));
    assert!(!c.contains(&Entity::Object(idle)), "not attacking");
    t.resolve_all();
    assert_eq!(t.zone(attacker), Zone::Exile);
    assert!(t.in_exile("Hill Giant"));
    assert!(t.on_battlefield(idle));
}
