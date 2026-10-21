//! Rulings batch P163 — sweepers that trigger on attacking: they trigger as attackers are
//! declared and resolve in the declare attackers step, before blockers (CR 508.1m, 508.2b,
//! 509.1); "defending player" is the player the creature attacks or the controller of the
//! planeswalker it attacks (CR 508.5, 508.5a, 802.2a, 805.10e); intervening "if" clauses
//! (CR 603.4); card types (CR 205.2a); noncombat damage (CR 120.2b); spells that target a
//! player affect that player's hexproof creatures (CR 115.10a, 702.11b).

use crate::r_p163_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether blockers have been asked for since decision `from`.
fn blockers_asked_since(t: &TestGame, from: usize) -> bool {
    t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::DeclareBlockers { .. }))
}

/// Resolves the attack triggers and asserts they resolved in the declare attackers step,
/// before any blockers were declared.
fn resolve_before_blocks(t: &mut TestGame) {
    let from = n_asked(t);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert!(!blockers_asked_since(t, from));
}

#[test]
fn novablast_wurm_destroys_all_other_creatures_before_blocks() {
    cr!("508.1m", "508.2b", "603.3");
    ruling!(
        "Novablast Wurm",
        "Novablast Wurm’s ability triggers and resolves during the declare attackers step, before blockers can be declared."
    );
    ruling!(
        "Novablast Wurm",
        "Novablast Wurm’s ability destroys all players’ creatures (except Novablast Wurm), including yours."
    );
    supported("Novablast Wurm");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Novablast Wurm");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(w, Entity::Player(P1))]);
    assert_eq!(stacked_triggers(&t), 1);
    assert!(t.on_battlefield(giant));
    resolve_before_blocks(&mut t);
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(giant));
    assert!(t.on_battlefield(w));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 13);
}

#[test]
fn two_novablast_wurms_destroy_each_other() {
    cr!("508.1m", "603.3b");
    ruling!(
        "Novablast Wurm",
        "If two Novablast Wurms attack, they’ll destroy each other. This happens before they can deal combat damage."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Novablast Wurm");
    let b = t.battlefield(P0, "Novablast Wurm");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    assert_eq!(stacked_triggers(&t), 2);
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

/// Puts the real cards into `p`'s graveyard.
fn graveyard(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    names.iter().map(|n| t.graveyard(p, n)).collect()
}

#[test]
fn gouged_zealot_checks_delirium_on_trigger_and_resolution() {
    cr!("603.4", "207.2c");
    ruling!(
        "Gouged Zealot",
        "Gouged Zealot's triggered ability checks whether there are four or more card types among cards in your graveyard to see if it triggers. If you don't, the ability won't trigger at all. The ability will check again as it tries to resolve. If you don't at that time, the ability won't do anything. The specific cards in your graveyard may change between the two checks."
    );
    supported("Gouged Zealot");
    let four = ["Grizzly Bears", "Forest", "Lightning Bolt", "Divination"];
    // Three card types: no trigger.
    let mut t = TestGame::new(2);
    let z = t.battlefield(P0, "Gouged Zealot");
    graveyard(&mut t, P0, &four[..3]);
    t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(z, Entity::Player(P1))]);
    assert_eq!(stacked_triggers(&t), 0);
    // Four, then three as it resolves: it does nothing.
    let mut t = TestGame::new(2);
    let z = t.battlefield(P0, "Gouged Zealot");
    let gy = graveyard(&mut t, P0, &four);
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(z, Entity::Player(P1))]);
    assert_eq!(stacked_triggers(&t), 1);
    move_to(&mut t, gy[2], Zone::Exile);
    t.resolve_all();
    assert_eq!(marked(&t, giant), 0);
    // Four, then a different four (the instant replaced by an artifact): it deals damage.
    let mut t = TestGame::new(2);
    let z = t.battlefield(P0, "Gouged Zealot");
    let gy = graveyard(&mut t, P0, &four);
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(z, Entity::Player(P1))]);
    assert_eq!(stacked_triggers(&t), 1);
    move_to(&mut t, gy[2], Zone::Exile);
    t.graveyard(P0, "Ornithopter");
    t.resolve_all();
    assert_eq!(marked(&t, giant), 1);
}

#[test]
fn gouged_zealot_counts_card_types_not_supertypes_or_subtypes() {
    cr!("205.2a", "205.4a");
    ruling!(
        "Gouged Zealot",
        "The card types that can appear in a graveyard are artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Legendary, basic, and snow are supertypes, not card types; Cyclops and Berserker are subtypes, not card types."
    );
    let cases: &[(&[&str], bool)] = &[
        // Kindred + sorcery (All Is Dust), land, creature: four.
        (&["All Is Dust", "Forest", "Grizzly Bears"], true),
        // Battle, land, creature, instant: four.
        (
            &["Invasion of Ravnica", "Forest", "Grizzly Bears", "Lightning Bolt"],
            true,
        ),
        // Snow basic land, legendary creature, Cyclops Berserker creature, instant: three.
        (
            &[
                "Snow-Covered Forest",
                "Isamaru, Hound of Konda",
                "Gouged Zealot",
                "Lightning Bolt",
            ],
            false,
        ),
    ];
    for (gy, triggers) in cases {
        let mut t = TestGame::new(2);
        let z = t.battlefield(P0, "Gouged Zealot");
        graveyard(&mut t, P0, gy);
        attack_with(&mut t, &[(z, Entity::Player(P1))]);
        assert_eq!(stacked_triggers(&t) == 1, *triggers, "{gy:?}");
    }
}

#[test]
fn swathcutter_giant_damages_the_planeswalkers_controller() {
    cr!("508.5", "508.5a", "802.2a");
    ruling!(
        "Swathcutter Giant",
        "If a creature attacks a planeswalker, that planeswalker’s controller is the defending player."
    );
    supported("Swathcutter Giant");
    let mut t = TestGame::new(3);
    let g = t.battlefield(P0, "Swathcutter Giant");
    let pw = t.battlefield(P2, "Ajani Goldmane");
    let p1c = t.battlefield(P1, "Hill Giant");
    let p2c = t.battlefield(P2, "Hill Giant");
    attack_with(&mut t, &[(g, Entity::Object(pw))]);
    resolve_before_blocks(&mut t);
    assert_eq!(marked(&t, p2c), 1);
    assert_eq!(marked(&t, p1c), 0);
}

#[test]
fn swathcutter_giant_in_two_headed_giant_hits_only_the_attacked_player() {
    cr!("805.10e", "810.7", "508.5a");
    ruling!(
        "Swathcutter Giant",
        "In a Two-Headed Giant game, Swathcutter Giant deals 1 damage to each creature controlled by the player it’s attacking. The other player’s creatures are unaffected."
    );
    let mut t = two_headed_giant();
    let g = t.battlefield(P0, "Swathcutter Giant");
    let p2c = t.battlefield(P2, "Hill Giant");
    let p3c = t.battlefield(P3, "Hill Giant");
    attack_with(&mut t, &[(g, Entity::Player(P2))]);
    t.resolve_all();
    assert_eq!(marked(&t, p2c), 1);
    assert_eq!(marked(&t, p3c), 0);
}

/// A Lord of Shatterskull Pass at level 6 for `p`.
fn lord(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let l = t.battlefield(p, "Lord of Shatterskull Pass");
    put_counters(t, l, counters::LEVEL, 6);
    assert_eq!(t.pt(l), (6, 6));
    l
}

#[test]
fn lord_of_shatterskull_pass_affects_only_the_attacked_player() {
    cr!("508.5", "508.5a", "802.2a", "711.2a");
    ruling!(
        "Lord of Shatterskull Pass",
        "In a multiplayer game, the only player affected by the last ability is the one Lord of Shatterskull Pass is attacking (or the one who controls the planeswalker Lord of Shatterskull Pass is attacking)."
    );
    ruling!(
        "Lord of Shatterskull Pass",
        "The last ability triggers and resolves during the declare attackers step, before blockers are declared."
    );
    supported("Lord of Shatterskull Pass");
    // Attacking a player.
    let mut t = TestGame::new(3);
    let l = lord(&mut t, P0);
    let p1c = t.battlefield(P1, "Craw Wurm");
    let p2c = t.battlefield(P2, "Craw Wurm");
    attack_with(&mut t, &[(l, Entity::Player(P1))]);
    resolve_before_blocks(&mut t);
    assert!(!t.on_battlefield(p1c));
    assert_eq!(marked(&t, p2c), 0);
    // Attacking a planeswalker.
    let mut t = TestGame::new(3);
    let l = lord(&mut t, P0);
    let pw = t.battlefield(P2, "Ajani Goldmane");
    let p1c = t.battlefield(P1, "Craw Wurm");
    let p2c = t.battlefield(P2, "Craw Wurm");
    attack_with(&mut t, &[(l, Entity::Object(pw))]);
    resolve_before_blocks(&mut t);
    assert_eq!(marked(&t, p1c), 0);
    assert!(!t.on_battlefield(p2c));
}

#[test]
fn scalding_salamander_resolves_before_blocks() {
    cr!("508.1m", "508.2b", "603.5");
    ruling!(
        "Scalding Salamander",
        "The ability triggers on the declaration of attackers and will resolve before blockers are declared."
    );
    supported("Scalding Salamander");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Scalding Salamander");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let drake = t.battlefield(P1, "Wind Drake");
    attack_with(&mut t, &[(s, Entity::Player(P1))]);
    yes(&mut t, P0);
    resolve_before_blocks(&mut t);
    assert_eq!(marked(&t, bears), 1);
    assert_eq!(marked(&t, drake), 0);
}

#[test]
fn silumgar_affects_the_player_each_dragon_attacks() {
    cr!("508.5", "508.5a", "802.2a", "603.2");
    ruling!(
        "Silumgar, the Drifting Death",
        "The creatures affected by Silumgar’s triggered ability are determined relative to the Dragon that attacked. For example, if Silumgar attacked one opponent and two other Dragons attacked another opponent, creatures controlled by the first opponent would get -1/-1 until end of turn and creatures controlled by the second opponent would get -2/-2 until end of turn."
    );
    supported("Silumgar, the Drifting Death");
    let mut t = TestGame::new(3);
    let s = t.battlefield(P0, "Silumgar, the Drifting Death");
    let d1 = t.battlefield(P0, "Shivan Dragon");
    let d2 = t.battlefield(P0, "Shivan Dragon");
    let p1c = t.battlefield(P1, "Craw Wurm");
    let p2c = t.battlefield(P2, "Craw Wurm");
    attack_with(
        &mut t,
        &[
            (s, Entity::Player(P1)),
            (d1, Entity::Player(P2)),
            (d2, Entity::Player(P2)),
        ],
    );
    assert_eq!(stacked_triggers(&t), 3);
    t.resolve_all();
    assert_eq!(t.pt(p1c), (5, 3));
    assert_eq!(t.pt(p2c), (4, 2));
}

#[test]
fn balefire_dragon_damage_isnt_combat_damage() {
    cr!("120.2b", "510.2", "603.2");
    ruling!(
        "Balefire Dragon",
        "The damage dealt by Balefire Dragon’s triggered ability isn’t combat damage."
    );
    supported("Balefire Dragon");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Balefire Dragon");
    let wurm = t.battlefield(P1, "Craw Wurm");
    attack_with(&mut t, &[(d, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert!(!t.on_battlefield(wurm));
    let dealt = damage_by(&t, d);
    assert!(dealt.contains(&(Entity::Player(P1), 6, true)), "{dealt:?}");
    assert!(dealt.contains(&(Entity::Object(wurm), 6, false)), "{dealt:?}");
}

#[test]
fn player_targeting_sweepers_damage_hexproof_creatures() {
    cr!("115.10a", "702.11b");
    ruling!(
        "Aggravate",
        "Aggravate targets only the player. Creatures with hexproof that player controls will be dealt damage by Aggravate if it resolves."
    );
    ruling!(
        "Radiating Lightning",
        "Radiating Lightning targets only the player. Creatures that player controls with hexproof will be dealt damage."
    );
    ruling!(
        "Chandra, Flame's Fury",
        "Chandra's last ability targets the player, not their creatures. A creature with hexproof may be dealt damage this way."
    );
    supported("Aggravate");
    supported("Radiating Lightning");
    supported("Chandra, Flame's Fury");
    for (name, dealt) in [("Aggravate", 1), ("Radiating Lightning", 1)] {
        let mut t = TestGame::new(2);
        let troll = t.battlefield(P1, "Troll Ascetic");
        let life = t.life(P1);
        cast_resolve(&mut t, P0, name, &[Entity::Player(P1)]);
        assert_eq!(marked(&t, troll), dealt, "{name}");
        if name == "Radiating Lightning" {
            assert_eq!(t.life(P1), life - 3);
        }
    }
    let mut t = TestGame::new(2);
    let ch = t.battlefield(P0, "Chandra, Flame's Fury");
    add_loyalty(&mut t, ch, 4);
    let troll = t.battlefield(P1, "Troll Ascetic");
    activate_resolve(&mut t, P0, ch, 2, &[Entity::Player(P1)]);
    assert!(!t.on_battlefield(troll));
    assert_eq!(t.life(P1), 10);
}

#[test]
fn aggravated_creature_attacks_whoever_its_controller_chooses() {
    cr!("508.1d", "506.2a");
    ruling!(
        "Aggravate",
        "The controller of a creature dealt damage by Aggravate still decides which player or planeswalker the creature attacks."
    );
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    // P0 casts Aggravate on P1 during P1's turn.
    cast_resolve(&mut t, P0, "Aggravate", &[Entity::Player(P1)]);
    assert_eq!(marked(&t, bears), 1);
    // P1 declares no attackers: the requirement makes the Bears attack anyway.
    attack_with(&mut t, &[]);
    let c = t.g.combat.as_ref().unwrap();
    assert!(c.attack_target(bears).is_some());
    // Another game: P1 chooses to attack P2 with it.
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    cast_resolve(&mut t, P0, "Aggravate", &[Entity::Player(P1)]);
    attack_with(&mut t, &[(bears, Entity::Player(P2))]);
    let c = t.g.combat.as_ref().unwrap();
    assert_eq!(c.attack_target(bears), Some(Entity::Player(P2)));
}

#[test]
fn chandra_flames_fury_illegal_targets() {
    cr!("608.2b", "115.10a");
    ruling!(
        "Chandra, Flame's Fury",
        "If the target creature is an illegal target by the time Chandra's second ability resolves, the ability doesn't resolve. No player is dealt damage. Similarly, if the target player is an illegal target by the time Chandra's last ability resolves, no creatures are dealt damage."
    );
    // −2 with the creature gone: its controller isn't dealt damage.
    let mut t = TestGame::new(2);
    let ch = t.battlefield(P0, "Chandra, Flame's Fury");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, ch, 1, &[obj(bears)]).unwrap();
    move_to(&mut t, bears, Zone::Hand(P1));
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // −8 with the player gaining hexproof: no creatures are dealt damage.
    let mut t = TestGame::new(2);
    let ch = t.battlefield(P0, "Chandra, Flame's Fury");
    add_loyalty(&mut t, ch, 4);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.activate(P0, ch, 2, &[Entity::Player(P1)]).unwrap();
    t.battlefield(P1, "Leyline of Sanctity");
    t.resolve_all();
    assert_eq!(marked(&t, wurm), 0);
    assert_eq!(t.life(P1), 20);
}
