//! Rulings batch P009 — burn with several targets: a spell needs every required target to
//! be cast (CR 601.2c, 115.1), it resolves for the targets still legal (CR 608.2b), it
//! doesn't resolve if all are illegal (CR 608.2b), the same object can't be chosen twice
//! for one "target" word (CR 115.3), and effects that don't target (CR 115.10).

use crate::r_p009_common::*;
use crate::r_s01_common::supported;
use crate::r_s06_common::damage;
use crate::r_s25_common::{cast_new, lands_for_cost};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const AFTERMATH: CastMethod = CastMethod::Keyword(KeywordKind::Aftermath);

/// Tries to cast the real card `name` (from `zone_gy`: the graveyard, else the hand) with
/// `method` and the given target answers; returns whether it was cast.
fn try_cast(
    t: &mut TestGame,
    name: &str,
    method: CastMethod,
    from_gy: bool,
    targets: &[Entity],
) -> bool {
    lands_for_cost(t, P0, name);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Swamp", 2);
    let card = if from_gy {
        t.graveyard(P0, name)
    } else {
        t.hand(P0, name)
    };
    for e in targets {
        t.answer_targets(P0, &[*e]);
    }
    let r = t.cast(P0, card).method(method).try_go();
    t.clear_answers();
    r.is_ok()
}

#[test]
fn spells_cant_be_cast_without_every_required_target() {
    cr!("601.2c", "115.1");
    ruling!(
        "Bedeck // Bedazzle",
        "You can't cast Bedazzle without a target nonbasic land and a target opponent or planeswalker."
    );
    ruling!(
        "Insult // Injury",
        "You can't cast Injury unless you target both a creature and a player."
    );
    ruling!(
        "Angrath's Fury",
        "You can’t cast Angrath’s Fury unless you choose both a target creature and a target player or planeswalker."
    );
    ruling!(
        "Cunning Strike",
        "You can’t cast Cunning Strike unless you target both a creature and a player."
    );
    ruling!(
        "Domri's Ambush",
        "You can’t cast Domri’s Ambush unless you choose both a creature you control and a creature or planeswalker you don’t control as targets."
    );
    ruling!(
        "Hungry Flames",
        "You can’t cast Hungry Flames unless you target both a creature and a player."
    );
    ruling!(
        "Punish the Enemy",
        "You must be able to target a player and a creature in order to cast Punish the Enemy."
    );
    // (card, method, cast from the graveyard, P1 has a creature, P0 has a creature)
    let cases: Vec<(&str, CastMethod, bool, bool, bool)> = vec![
        ("Bedeck // Bedazzle", CastMethod::Half(1), false, true, true),
        ("Insult // Injury", AFTERMATH, true, false, false),
        ("Angrath's Fury", CastMethod::Normal, false, false, false),
        ("Cunning Strike", CastMethod::Normal, false, false, false),
        ("Domri's Ambush", CastMethod::Normal, false, true, false),
        ("Hungry Flames", CastMethod::Normal, false, false, false),
        ("Punish the Enemy", CastMethod::Normal, false, false, false),
    ];
    for (name, method, gy, p1_creature, p0_creature) in cases {
        supported(name);
        let mut t = TestGame::new(2);
        let mut targets = vec![];
        if p1_creature {
            targets.push(obj(t.battlefield(P1, "Grizzly Bears")));
        }
        if p0_creature {
            targets.push(obj(t.battlefield(P0, "Grizzly Bears")));
        }
        targets.push(pl(P1));
        assert!(
            !try_cast(&mut t, name, method, gy, &targets),
            "{name} was cast"
        );
        assert_eq!(t.stack_len(), 0, "{name}");
    }
    // With every target available, each can be cast.
    let cases: Vec<(&str, CastMethod, bool)> = vec![
        ("Bedeck // Bedazzle", CastMethod::Half(1), false),
        ("Insult // Injury", AFTERMATH, true),
        ("Angrath's Fury", CastMethod::Normal, false),
        ("Cunning Strike", CastMethod::Normal, false),
        ("Domri's Ambush", CastMethod::Normal, false),
        ("Hungry Flames", CastMethod::Normal, false),
        ("Punish the Enemy", CastMethod::Normal, false),
    ];
    for (name, method, gy) in cases {
        let mut t = TestGame::new(2);
        let bears = obj(t.battlefield(P1, "Grizzly Bears"));
        let mine = obj(t.battlefield(P0, "Grizzly Bears"));
        let field = obj(t.battlefield(P1, "Mutavault"));
        let targets = match name {
            "Bedeck // Bedazzle" => vec![field, pl(P1)],
            "Domri's Ambush" => vec![mine, bears],
            "Punish the Enemy" => vec![pl(P1), bears],
            _ => vec![bears, pl(P1)],
        };
        assert!(try_cast(&mut t, name, method, gy, &targets), "{name}");
    }
}

#[test]
fn the_remaining_legal_target_is_still_affected() {
    cr!("608.2b");
    ruling!(
        "Bedeck // Bedazzle",
        "If one of the targets is illegal when Bedazzle tries to resolve, the other is still affected as appropriate."
    );
    ruling!(
        "Insult // Injury",
        "If one target is illegal as Injury resolves, the spell deals damage to the remaining legal target."
    );
    ruling!(
        "Hungry Flames",
        "If one target is illegal as Hungry Flames resolves, the spell deals damage to the remaining legal target."
    );
    ruling!(
        "Cunning Strike",
        "If one target (but not both) is illegal as Cunning Strike resolves, it deals damage to the remaining legal target and you draw a card. If both targets are illegal, the spell won’t resolve and none of its effects will happen. You won’t draw a card in that case."
    );
    // Bedazzle: the land is gone; P1 is still dealt 2 damage.
    let mut t = TestGame::new(2);
    let field = t.battlefield(P1, "Mutavault");
    assert!(try_cast(
        &mut t,
        "Bedeck // Bedazzle",
        CastMethod::Half(1),
        false,
        &[obj(field), pl(P1)]
    ));
    kill(&mut t, field);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // ... and with P1 an illegal target (hexproof from Leyline of Sanctity), the land is
    // still destroyed and P1 isn't dealt damage.
    let mut t = TestGame::new(2);
    let field = t.battlefield(P1, "Mutavault");
    assert!(try_cast(
        &mut t,
        "Bedeck // Bedazzle",
        CastMethod::Half(1),
        false,
        &[obj(field), pl(P1)]
    ));
    add(&mut t, P1, "Leyline of Sanctity");
    t.resolve_all();
    assert!(!t.on_battlefield(field));
    assert_eq!(t.life(P1), 20);

    // Injury and Hungry Flames: the creature is gone; the player is still dealt damage.
    for (name, method, gy, life) in [
        ("Insult // Injury", AFTERMATH, true, 18),
        ("Hungry Flames", CastMethod::Normal, false, 18),
    ] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        assert!(try_cast(&mut t, name, method, gy, &[obj(bears), pl(P1)]));
        kill(&mut t, bears);
        t.resolve_all();
        assert_eq!(t.life(P1), life, "{name}");
    }

    // Cunning Strike: one illegal target -> damage to the other and a card drawn.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(try_cast(
        &mut t,
        "Cunning Strike",
        CastMethod::Normal,
        false,
        &[obj(bears), pl(P1)]
    ));
    let hand = t.hand_size(P0);
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.hand_size(P0), hand + 1);
    // Both targets illegal (the creature is gone; P1 is a planeswalker target that left
    // too): it doesn't resolve and nothing is drawn.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let pw = t.battlefield(P1, "Chandra Nalaar");
    assert!(try_cast(
        &mut t,
        "Cunning Strike",
        CastMethod::Normal,
        false,
        &[obj(bears), obj(pw)]
    ));
    let hand = t.hand_size(P0);
    kill(&mut t, bears);
    kill(&mut t, pw);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn a_creature_target_illegal_means_no_damage_to_its_controller() {
    cr!("608.2b");
    ruling!(
        "Chandra's Outrage",
        "If the target creature is an illegal target by the time Chandra's Outrage tries to resolve, the spell doesn't resolve. No player is dealt damage."
    );
    ruling!(
        "Consign to the Pit",
        "If the target creature is an illegal target by the time Consign to the Pit tries to resolve, the spell doesn’t resolve. No player is dealt damage. If the target is legal but not destroyed (most likely because it has indestructible), its controller is dealt damage."
    );
    ruling!(
        "Unlicensed Disintegration",
        "If the target creature is an illegal target by the time Unlicensed Disintegration tries to resolve, the spell won't resolve. It won't deal damage to any player. If the target is legal but not destroyed (most likely because it has indestructible), its controller is dealt 3 damage."
    );
    ruling!(
        "Smash to Smithereens",
        "Smash to Smithereens targets only the artifact, not any player. If that artifact becomes an illegal target by the time Smash to Smithereens tries to resolve, Smash to Smithereens won’t resolve and none of its effects will happen. No damage will be dealt."
    );
    for (name, victim, legal_life) in [
        ("Chandra's Outrage", "Grizzly Bears", 18),
        ("Consign to the Pit", "Darksteel Myr", 18),
        ("Unlicensed Disintegration", "Darksteel Myr", 17),
        ("Smash to Smithereens", "Darksteel Myr", 17),
    ] {
        supported(name);
        // Illegal target: no damage.
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Ornithopter");
        let v = t.battlefield(P1, "Grizzly Bears");
        let v = if victim == "Darksteel Myr" {
            t.battlefield(P1, "Ornithopter")
        } else {
            v
        };
        cast_new(&mut t, P0, name, &[obj(v)]);
        let v2 = t.g.current(v);
        t.g.exile_object(v2, None);
        t.g.flush_events();
        t.resolve_all();
        assert_eq!(t.life(P1), 20, "{name}");
        // Legal target (indestructible for the destroy spells): its controller is dealt
        // damage.
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Ornithopter");
        let v = t.battlefield(P1, victim);
        cast_new(&mut t, P0, name, &[obj(v)]);
        t.resolve_all();
        assert_eq!(t.life(P1), legal_life, "{name}");
        if victim == "Darksteel Myr" {
            assert!(t.on_battlefield(v), "{name}");
        }
    }
    // Smash to Smithereens doesn't target the player: a player with hexproof is dealt the
    // damage. (Its only target is the artifact.)
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P1, "Darksteel Myr");
    t.battlefield(P1, "Leyline of Sanctity");
    cast_new(&mut t, P0, "Smash to Smithereens", &[obj(myr)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn unlicensed_disintegration_checks_artifacts_after_destroying() {
    cr!("608.2c");
    ruling!(
        "Unlicensed Disintegration",
        "Whether or not you control an artifact is checked only after the creature is destroyed while Unlicensed Disintegration is resolving."
    );
    supported("Fairgrounds Warden");
    // Destroying your only artifact (an artifact creature) means no damage.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    cast_new(&mut t, P0, "Unlicensed Disintegration", &[obj(thopter)]);
    t.resolve_all();
    assert!(!t.on_battlefield(thopter));
    assert_eq!(t.life(P0), 20);
    // Destroying a Fairgrounds Warden that exiled your artifact creature returns it first:
    // 3 damage to the Warden's controller.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    t.answer_targets(P1, &[obj(thopter)]);
    let warden = t.enter(P1, "Fairgrounds Warden");
    t.resolve_all();
    assert!(!t.on_battlefield(thopter), "exiled by the Warden");
    cast_new(&mut t, P0, "Unlicensed Disintegration", &[obj(warden)]);
    t.resolve_all();
    assert!(!t.named_on_battlefield("Ornithopter").is_empty());
    assert_eq!(t.life(P1), 17);
}

#[test]
fn impractical_joke_needs_no_target_but_an_illegal_one_stops_it() {
    cr!("608.2b", "614.16", "702.16e");
    ruling!(
        "Impractical Joke",
        "You don't have to choose a target for Impractical Joke. However, if you do and the target is illegal as Impractical Joke tries to resolve, it won't resolve and none of its effects will happen. Damage can still be prevented this turn."
    );
    supported("Impractical Joke");
    // No target: damage can't be prevented this turn (protection doesn't prevent it).
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P1, "White Knight");
    let black = t.battlefield(P0, "Walking Corpse");
    t.answer_targets(P0, &[]);
    cast_new(&mut t, P0, "Impractical Joke", &[]);
    t.resolve_all();
    damage(&mut t, black, 1, knight);
    assert_eq!(dmg(&t, knight), 1);
    // Illegal target: it doesn't resolve, and protection still prevents damage.
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P1, "White Knight");
    let black = t.battlefield(P0, "Walking Corpse");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Impractical Joke", &[obj(bears)]);
    kill(&mut t, bears);
    t.resolve_all();
    damage(&mut t, black, 1, knight);
    assert_eq!(dmg(&t, knight), 0);
}

#[test]
fn chandra_pyrogenius_ultimate_doesnt_target_the_creatures() {
    cr!("115.10", "702.11b");
    ruling!(
        "Chandra, Pyrogenius",
        "The last ability of Chandra, Pyrogenius doesn't target the creatures. A creature with hexproof the target player (or controller of the target planeswalker) controls will be dealt damage this way."
    );
    supported("Chandra, Pyrogenius");
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra, Pyrogenius");
    t.g.add_counters(obj(chandra), counters::LOYALTY, 10, None);
    let scout = t.battlefield(P1, "Gladecover Scout");
    t.activate(P0, chandra, 2, &[pl(P1)]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(scout));
    assert_eq!(t.life(P1), 14);
}

#[test]
fn volcanic_salvo_cant_target_the_same_creature_twice() {
    cr!("115.3", "601.2c");
    ruling!(
        "Volcanic Salvo",
        "You can't target the same creature or planeswalker twice with Volcanic Salvo to deal 12 damage to it."
    );
    supported("Volcanic Salvo");
    let mut t = TestGame::new(2);
    let colossus = t.battlefield(P1, "Darksteel Colossus");
    let salvo = t.hand(P0, "Volcanic Salvo");
    t.lands(P0, "Mountain", 12);
    let r = t
        .cast(P0, salvo)
        .targets(&[obj(colossus), obj(colossus)])
        .try_go();
    t.clear_answers();
    if r.is_ok() {
        t.resolve_all();
    }
    assert!(dmg(&t, colossus) <= 6);
    // The legal way: one target, 6 damage.
    let mut t = TestGame::new(2);
    let colossus = t.battlefield(P1, "Darksteel Colossus");
    let salvo = t.hand(P0, "Volcanic Salvo");
    t.lands(P0, "Mountain", 12);
    t.cast(P0, salvo).targets(&[obj(colossus)]).go();
    t.resolve_all();
    assert_eq!(dmg(&t, colossus), 6);
}

#[test]
fn each_trigger_chooses_its_own_target() {
    cr!("603.3d", "115.1");
    ruling!(
        "Furious Assault",
        "You can choose a different target player or planeswalker each time this triggers."
    );
    ruling!(
        "Scalding Tongs",
        "You target one opposing player each time the ability triggers."
    );
    supported("Furious Assault");
    supported("Scalding Tongs");
    // Furious Assault: two creature spells, two triggers, two different targets.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Furious Assault");
    t.answer_targets(P0, &[pl(P1)]);
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    t.answer_targets(P0, &[pl(P2)]);
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (19, 19));
    // Scalding Tongs: each upkeep trigger targets one opponent.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Scalding Tongs");
    t.answer_targets(P0, &[pl(P2)]);
    t.set_step(P2, mtg_engine::turn::Step::End);
    t.advance_to(P0, mtg_engine::turn::Step::Upkeep);
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (20, 19));
}

#[test]
fn clan_defiance_modes_share_one_x() {
    cr!("700.2", "601.2b", "107.3a");
    ruling!(
        "Clan Defiance",
        "The value of X is the same for each mode you choose."
    );
    ruling!(
        "Clan Defiance",
        "You can choose just one mode, any two of the modes, or all three. You make this choice as you cast Clan Defiance."
    );
    supported("Clan Defiance");
    let mut t = TestGame::new(2);
    let flyer = t.battlefield(P1, "Serra Angel");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 4);
    t.lands(P0, "Forest", 1);
    let cd = t.hand(P0, "Clan Defiance");
    t.cast(P0, cd)
        .modes(&[0, 1, 2])
        .x(3)
        .targets(&[obj(flyer)])
        .targets(&[obj(wurm)])
        .targets(&[pl(P1)])
        .go();
    t.resolve_all();
    assert_eq!(dmg(&t, flyer), 3);
    assert_eq!(dmg(&t, wurm), 3);
    assert_eq!(t.life(P1), 17);
    // One mode only.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Forest", 1);
    let cd = t.hand(P0, "Clan Defiance");
    t.cast(P0, cd).modes(&[2]).x(2).targets(&[pl(P1)]).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn fires_of_victory_deals_damage_kicked_or_not() {
    cr!("702.33d", "608.2c");
    ruling!(
        "Fires of Victory",
        "You will draw a card only if Fires of Victory was kicked, but it will deal damage to the target creature or planeswalker whether it was kicked or not."
    );
    supported("Fires of Victory");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let fires = t.hand(P0, "Fires of Victory");
    t.cast(P0, fires).kicked(false).target(wurm).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(dmg(&t, wurm), 2);
    // Kicked: draw first, then damage equal to the cards in hand.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 4);
    t.lands(P0, "Island", 1);
    let fires = t.hand(P0, "Fires of Victory");
    t.cast(P0, fires).kicked(true).target(wurm).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(dmg(&t, wurm), 3);
}

#[test]
fn brokers_charm_first_mode_needs_an_opposing_target() {
    cr!("700.2a", "601.2b");
    ruling!(
        "Brokers Charm",
        "You can't choose the first mode for Brokers Charm unless your opponent has a creature or planeswalker to target."
    );
    supported("Brokers Charm");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Brokers Charm");
    let charm = t.hand(P0, "Brokers Charm");
    let r = t
        .cast(P0, charm)
        .modes(&[0])
        .targets(&[obj(mine)])
        .targets(&[obj(mine)])
        .try_go();
    t.clear_answers();
    let first_mode_chosen = r.is_ok_and(|s| {
        t.g.obj(s)
            .stack
            .as_deref()
            .is_some_and(|si| si.chosen.iter().any(|m| m.mode == Some(0)))
    });
    assert!(!first_mode_chosen);
    // With an opposing creature, it can be chosen.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Craw Wurm");
    lands_for_cost(&mut t, P0, "Brokers Charm");
    let charm = t.hand(P0, "Brokers Charm");
    t.cast(P0, charm)
        .modes(&[0])
        .targets(&[obj(mine)])
        .targets(&[obj(theirs)])
        .go();
    t.resolve_all();
    assert_eq!(dmg(&t, theirs), 3);
}
