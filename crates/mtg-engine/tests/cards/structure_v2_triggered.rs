//! Structure-coverage tests, stream 2 (item `structure-tests-2`): triggered-ability
//! structures that alone kept their cards from being verified (see
//! `docs/CARD_COVERAGE.md`). Each test checks that a real card's ability does what its text
//! says: the right objects and players affected and not others, amounts, durations ending,
//! conditions true and false, optional parts accepted and declined, and the targets the
//! engine offers. Combat triggers are in `structure_v2_triggered_combat.rs`.

use super::structure_activated_2::{act, next_turn, shuffled, target_choices};
use super::structure_activated_6::entity_choices;
use super::structure_mana_2::pool;
use super::structure_seed_common::*;
use super::structure_triggered_5::{etb, obj};
use super::structure_triggered_7::{bolt, cast_card, kill, tokens};
use super::structure_triggered_9::waiting;
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{Color, ColorSet};
use mtg_engine::*;

#[test]
fn eladamris_vineyard_adds_gg_in_each_players_first_main_phase() {
    cr!("603.2", "106.4");
    // "At the beginning of each player's first main phase, that player adds {G}{G}."
    supported("Eladamri's Vineyard");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Eladamri's Vineyard");
    t.advance_to(P1, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(pool(&t, P1, ManaType::G), 2, "the active player adds it");
    assert_eq!(pool(&t, P0, ManaType::G), 0);
    // Not in the second main phase.
    t.advance_to(P1, Step::PostcombatMain);
    assert_eq!(waiting(&t), 0, "no trigger in the second main phase");
    assert_eq!(pool(&t, P1, ManaType::G), 0, "the mana emptied and none was added");
    // And its controller's own first main phase.
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(pool(&t, P0, ManaType::G), 2);
}

#[test]
fn knucklebone_witch_may_grow_when_a_goblin_you_control_dies() {
    cr!("603.6c", "603.10a", "603.5");
    // "Whenever a Goblin you control is put into a graveyard from the battlefield, you
    // may put a +1/+1 counter on this creature."
    supported("Knucklebone Witch");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let w = t.battlefield(P0, "Knucklebone Witch");
        let g = t.battlefield(P0, "Raging Goblin");
        let theirs = t.battlefield(P1, "Raging Goblin");
        let bears = t.battlefield(P0, "Grizzly Bears");
        kill(&mut t, theirs);
        kill(&mut t, bears);
        assert_eq!(t.counters(w, "+1/+1"), 0, "not an opponent's Goblin or a non-Goblin");
        t.answer_yes(P0, yes);
        kill(&mut t, g);
        assert_eq!(t.counters(w, "+1/+1"), yes as u32, "accepted: {yes}");
    }
}

#[test]
fn vile_entomber_puts_any_card_from_your_library_into_your_graveyard() {
    cr!("603.6a", "701.23a");
    // "When this creature enters, search your library for a card, put that card into
    // your graveyard, then shuffle."
    supported("Vile Entomber");
    let mut t = TestGame::new(2);
    let giant = t.library_top(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(giant)]);
    etb(&mut t, P0, "Vile Entomber");
    assert_eq!(t.zone(giant), Zone::Graveyard(P0));
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(t.library_size(P0), 30);
    assert_eq!(t.hand_size(P0), 0);
    assert!(shuffled(&t, P0));
    assert!(!shuffled(&t, P1));
}

#[test]
fn servo_schematic_makes_a_servo_on_entering_and_on_dying() {
    cr!("603.6a", "603.6c", "111.1");
    // "When this artifact enters or is put into a graveyard from the battlefield, create a
    // 1/1 colorless Servo artifact creature token."
    supported("Servo Schematic");
    let mut t = TestGame::new(2);
    let s = etb(&mut t, P0, "Servo Schematic");
    let servos = tokens(&t, P0, "Servo");
    assert_eq!(servos.len(), 1);
    let o = t.obj_now(servos[0]);
    assert!(o.is(mtg_engine::types::CardType::Artifact) && o.is_creature());
    assert!(o.chars.colors.is_colorless());
    assert_eq!(t.pt(servos[0]), (1, 1));
    kill(&mut t, s);
    assert_eq!(tokens(&t, P0, "Servo").len(), 2);
    assert_eq!(tokens(&t, P1, "Servo").len(), 0);
    // Leaving to another zone isn't "put into a graveyard".
    let s = t.battlefield(P0, "Servo Schematic");
    t.g.move_object(s, Zone::Exile, MoveCause::Effect, None);
    t.resolve_all();
    assert_eq!(tokens(&t, P0, "Servo").len(), 2);
}

#[test]
fn charmed_clothier_attaches_a_royal_role_to_another_creature_you_control() {
    cr!("603.6a", "111.10m", "303.7");
    // "When this creature enters, create a Royal Role token attached to another target
    // creature you control."
    supported("Charmed Clothier");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(bears)]);
    let c = etb(&mut t, P0, "Charmed Clothier");
    let ch = target_choices(&t, P0).pop().unwrap();
    assert!(ch.contains(&obj(bears)));
    assert!(!ch.contains(&obj(c)) && !ch.contains(&obj(theirs)), "another, you control");
    let roles = subtype_ids(&t, P0, "Role");
    assert_eq!(roles.len(), 1);
    assert_eq!(t.obj_now(roles[0]).attached_to, Some(obj(bears)));
    assert_eq!(t.pt(bears), (3, 3));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Ward));
    assert_eq!(t.pt(c), (3, 3), "not itself");
}

#[test]
fn hekma_sentinels_grows_once_per_cycle_or_discard() {
    cr!("702.29a", "701.9a", "603.2c");
    // "Whenever you cycle or discard a card, this creature gets +1/+1 until end of turn."
    supported("Hekma Sentinels");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hekma Sentinels");
    let s = t.hand(P0, "River Serpent");
    t.lands(P0, "Island", 1);
    act(&mut t, P0, s, "cycling", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(h), (3, 4), "cycling is a discard too, but it triggers once");
    assert_eq!(t.hand_size(P0), 1, "cycling drew a card");
    // The opponent's discards don't count; yours do, once each.
    t.hand(P0, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    cast_card(&mut t, P1, "Mind Rot", "Swamp", 3, &[Entity::Player(P1)]);
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.pt(h), (3, 4));
    cast_card(&mut t, P1, "Mind Rot", "Swamp", 3, &[Entity::Player(P0)]);
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.pt(h), (5, 6), "two discards");
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.pt(h), (2, 3));
}

#[test]
fn rampaging_spiketail_gives_a_creature_you_control_two_power_and_indestructible() {
    cr!("603.6a", "702.12b", "611.2a");
    // "When this creature enters, target creature you control gets +2/+0 and gains
    // indestructible until end of turn."
    supported("Rampaging Spiketail");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(bears)]);
    let s = etb(&mut t, P0, "Rampaging Spiketail");
    let ch = target_choices(&t, P0).pop().unwrap();
    assert!(ch.contains(&obj(bears)) && ch.contains(&obj(s)));
    assert!(!ch.contains(&obj(theirs)), "creatures you control");
    assert_eq!(t.pt(bears), (4, 2));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Indestructible));
    t.g.destroy(bears, None);
    t.resolve_all();
    assert!(t.on_battlefield(bears), "indestructible");
    assert!(!t.obj_now(s).has_keyword(KeywordKind::Indestructible));
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Indestructible));
}

#[test]
fn tajuru_beastmaster_rallies_when_it_or_another_ally_enters() {
    cr!("603.6a", "611.2c");
    // "Rally — Whenever this creature or another Ally you control enters, creatures you
    // control get +1/+1 until end of turn."
    supported("Tajuru Beastmaster");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let b = etb(&mut t, P0, "Tajuru Beastmaster");
    assert_eq!(t.pt(bears), (3, 3), "itself entering");
    assert_eq!(t.pt(b), (6, 6));
    assert_eq!(t.pt(theirs), (2, 2), "your creatures only");
    t.enter(P1, "Expedition Envoy");
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3), "not an opponent's Ally");
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3), "not a non-Ally");
    let e = etb(&mut t, P0, "Expedition Envoy");
    assert_eq!(t.pt(bears), (4, 4), "another Ally");
    assert_eq!(t.pt(e), (3, 2), "creatures you control at resolution");
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn scavenging_harpy_exiles_a_card_from_an_opponents_graveyard() {
    cr!("603.6a", "115.1");
    // "When this creature enters, exile target card from an opponent's graveyard."
    supported("Scavenging Harpy");
    let mut t = TestGame::new(2);
    let theirs = t.graveyard(P1, "Hill Giant");
    let other = t.graveyard(P1, "Forest");
    let mine = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(theirs)]);
    etb(&mut t, P0, "Scavenging Harpy");
    let ch = target_choices(&t, P0).pop().unwrap();
    assert!(ch.contains(&obj(theirs)) && ch.contains(&obj(other)), "any card");
    assert!(!ch.contains(&obj(mine)), "not your graveyard");
    assert!(t.in_exile("Hill Giant"));
    assert!(t.in_graveyard(P1, "Forest") && t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn ghitu_chronicler_returns_an_instant_or_sorcery_only_if_kicked() {
    cr!("702.33d", "603.4");
    // "Kicker {3}{R} / When this creature enters, if it was kicked, return target instant
    // or sorcery card from your graveyard to your hand."
    supported("Ghitu Chronicler");
    for kicked in [true, false] {
        let mut t = TestGame::new(2);
        let bolt_card = t.graveyard(P0, "Lightning Bolt");
        let creature = t.graveyard(P0, "Grizzly Bears");
        let theirs = t.graveyard(P1, "Shock");
        t.lands(P0, "Mountain", 6);
        let c = t.hand(P0, "Ghitu Chronicler");
        t.answer_targets(P0, &[obj(bolt_card)]);
        t.cast(P0, c).kicked(kicked).go();
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "Lightning Bolt"), kicked, "kicked: {kicked}");
        if kicked {
            let ch = target_choices(&t, P0).pop().unwrap();
            assert!(!ch.contains(&obj(creature)) && !ch.contains(&obj(theirs)));
        } else {
            assert!(target_choices(&t, P0).is_empty(), "no trigger when not kicked");
        }
        assert!(t.in_graveyard(P0, "Grizzly Bears") && t.in_graveyard(P1, "Shock"));
    }
}

#[test]
fn cleric_of_lifes_bond_gains_life_when_another_cleric_enters() {
    cr!("603.6a", "119.3");
    // "Whenever another Cleric you control enters, you gain 1 life. / Whenever you gain
    // life for the first time each turn, put a +1/+1 counter on this creature."
    supported("Cleric of Life's Bond");
    let mut t = TestGame::new(2);
    let c = etb(&mut t, P0, "Cleric of Life's Bond");
    assert_eq!(t.life(P0), 20, "not itself");
    t.enter(P1, "Battlefield Medic");
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "not an opponent's Cleric or a non-Cleric");
    etb(&mut t, P0, "Battlefield Medic");
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.counters(c, "+1/+1"), 1, "the first life gain this turn");
    etb(&mut t, P0, "Battlefield Medic");
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.counters(c, "+1/+1"), 1, "only the first time");
}

#[test]
fn shocking_sharpshooter_pings_an_opponent_when_another_creature_you_control_enters() {
    cr!("603.6a", "115.1", "120.3a");
    // "Whenever another creature you control enters, this creature deals 1 damage to
    // target opponent."
    supported("Shocking Sharpshooter");
    let mut t = TestGame::new(3);
    t.answer_targets(P0, &[Entity::Player(P2)]);
    etb(&mut t, P0, "Shocking Sharpshooter");
    assert!(target_choices(&t, P0).is_empty(), "not itself");
    t.enter(P1, "Grizzly Bears");
    t.enter(P0, "Sol Ring");
    t.resolve_all();
    assert!(target_choices(&t, P0).is_empty(), "not an opponent's creature or a noncreature");
    etb(&mut t, P0, "Grizzly Bears");
    let ch = target_choices(&t, P0).pop().unwrap();
    assert!(ch.contains(&Entity::Player(P1)) && ch.contains(&Entity::Player(P2)));
    assert!(!ch.contains(&Entity::Player(P0)), "opponents only");
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (20, 20, 19));
}

#[test]
fn ox_of_agonas_discards_your_hand_then_draws_three() {
    cr!("603.6a", "701.9a", "121.1");
    // "When this creature enters, discard your hand, then draw three cards."
    supported("Ox of Agonas");
    let mut t = TestGame::new(2);
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Hill Giant");
    t.hand(P1, "Shock");
    etb(&mut t, P0, "Ox of Agonas");
    assert_eq!(t.zone(a), Zone::Graveyard(P0));
    assert_eq!(t.zone(b), Zone::Graveyard(P0));
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.library_size(P0), 27);
    assert_eq!(t.hand_size(P1), 1);
    // With an empty hand it still draws three.
    let mut t = TestGame::new(2);
    etb(&mut t, P0, "Ox of Agonas");
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn firemaw_kavu_deals_four_damage_when_it_leaves() {
    cr!("603.6c", "603.10a", "115.1");
    // "When this creature leaves the battlefield, it deals 4 damage to target creature."
    supported("Firemaw Kavu");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Firemaw Kavu");
    let giant = t.battlefield(P1, "Hill Giant");
    let big = t.battlefield(P1, "Craw Wurm");
    let land = t.battlefield(P1, "Forest");
    t.answer_targets(P0, &[obj(giant)]);
    t.g.move_object(k, Zone::Hand(P0), MoveCause::Effect, None);
    t.resolve_all();
    let ch = target_choices(&t, P0).pop().unwrap();
    assert!(ch.contains(&obj(big)) && !ch.contains(&obj(land)));
    assert!(t.in_graveyard(P1, "Hill Giant"), "4 damage");
    assert_eq!(t.obj_now(big).damage, 0);
    // Dying counts as leaving too.
    let k = t.battlefield(P0, "Firemaw Kavu");
    t.answer_targets(P0, &[obj(big)]);
    kill(&mut t, k);
    assert!(t.in_graveyard(P1, "Craw Wurm"), "4 damage to the 6/4");
}

#[test]
fn thundering_wurm_is_sacrificed_unless_you_discard_a_land() {
    cr!("603.6a", "118.12a", "701.9a");
    // "When this creature enters, sacrifice it unless you discard a land card."
    supported("Thundering Wurm");
    // Discard a land: it stays.
    let mut t = TestGame::new(2);
    let land = t.hand(P0, "Forest");
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(land)]);
    let w = etb(&mut t, P0, "Thundering Wurm");
    let ch = entity_choices(&t, P0).pop().unwrap();
    assert!(ch.contains(&obj(land)) && !ch.contains(&obj(bears)), "land cards only");
    assert!(t.on_battlefield(w));
    assert_eq!(t.zone(land), Zone::Graveyard(P0));
    assert_eq!(t.zone(bears), Zone::Hand(P0));
    // Decline: sacrificed.
    let mut t = TestGame::new(2);
    let land = t.hand(P0, "Forest");
    t.answer_yes(P0, false);
    etb(&mut t, P0, "Thundering Wurm");
    assert!(t.in_graveyard(P0, "Thundering Wurm"));
    assert_eq!(t.zone(land), Zone::Hand(P0));
    // No land card: sacrificed.
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    etb(&mut t, P0, "Thundering Wurm");
    assert!(t.in_graveyard(P0, "Thundering Wurm"));
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn penumbra_wurm_and_brood_weaver_leave_tokens_when_they_die() {
    cr!("603.6c", "111.1");
    // Penumbra Wurm: "When this creature dies, create a 6/6 black Wurm creature token
    // with trample." Brood Weaver: "When this creature dies, create a 1/2 green Spider
    // creature token with reach."
    for (name, sub, pt, color, kw) in [
        ("Penumbra Wurm", "Wurm", (6, 6), Color::Black, KeywordKind::Trample),
        ("Brood Weaver", "Spider", (1, 2), Color::Green, KeywordKind::Reach),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        t.g.move_object(c, Zone::Exile, MoveCause::Effect, None);
        t.resolve_all();
        assert!(tokens(&t, P0, sub).is_empty(), "{name}: exiled, it didn't die");
        let c = t.battlefield(P0, name);
        kill(&mut t, c);
        let tk = tokens(&t, P0, sub);
        assert_eq!(tk.len(), 1, "{name}");
        assert_eq!(t.pt(tk[0]), pt);
        assert_eq!(t.obj_now(tk[0]).chars.colors, ColorSet::single(color));
        assert!(t.obj_now(tk[0]).has_keyword(kw));
        assert!(tokens(&t, P1, sub).is_empty());
    }
}

#[test]
fn forerunner_of_the_coalition_drains_when_another_pirate_enters() {
    cr!("603.6a", "119.3");
    // "Whenever another Pirate you control enters, each opponent loses 1 life."
    supported("Forerunner of the Coalition");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Forerunner of the Coalition");
    t.enter(P1, "Kitesail Corsair");
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (20, 20), "not an opponent's Pirate or a non-Pirate");
    etb(&mut t, P0, "Kitesail Corsair");
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (20, 19, 19));
}

#[test]
fn city_pigeon_leaves_a_food_when_it_leaves() {
    cr!("603.6c", "603.10a", "111.10b");
    // "When this creature leaves the battlefield, create a Food token."
    supported("City Pigeon");
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "City Pigeon");
    t.g.move_object(p, Zone::Hand(P0), MoveCause::Effect, None);
    t.resolve_all();
    assert_eq!(tokens(&t, P0, "Food").len(), 1, "bounced");
    let p = t.battlefield(P0, "City Pigeon");
    kill(&mut t, p);
    assert_eq!(tokens(&t, P0, "Food").len(), 2, "died");
    assert!(tokens(&t, P1, "Food").is_empty());
}

#[test]
fn filthy_cur_makes_you_lose_life_equal_to_damage_dealt_to_it() {
    cr!("603.2", "119.3");
    // "Whenever this creature is dealt damage, you lose that much life."
    supported("Filthy Cur");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Filthy Cur");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Mountain", 1);
    let s = t.hand(P1, "Shock");
    t.cast(P1, s).target(obj(c)).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Filthy Cur"));
    assert_eq!(t.life(P0), 18, "2 damage dealt to it");
    assert_eq!(t.life(P1), 20);
    // Damage to you or to another creature doesn't trigger it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Filthy Cur");
    let bears = t.battlefield(P0, "Grizzly Bears");
    bolt(&mut t, P1, obj(bears));
    assert_eq!(t.life(P0), 20);
    bolt(&mut t, P1, Entity::Player(P0));
    assert_eq!(t.life(P0), 17);
}

#[test]
fn frilled_deathspitter_hits_an_opponent_when_dealt_damage() {
    cr!("603.2", "115.1", "120.3a");
    // "Enrage — Whenever this creature is dealt damage, it deals 2 damage to target
    // opponent or planeswalker."
    supported("Frilled Deathspitter");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Frilled Deathspitter");
    let walker = t.battlefield(P1, "Ajani Goldmane");
    let mine = t.battlefield(P0, "Ajani Goldmane");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.lands(P1, "Mountain", 1);
    let s = t.hand(P1, "Shock");
    t.cast(P1, s).target(obj(d)).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Frilled Deathspitter"), "3/2: Shock kills it");
    let ch = target_choices(&t, P0).pop().unwrap();
    assert!(ch.contains(&Entity::Player(P1)) && ch.contains(&obj(walker)));
    assert!(ch.contains(&obj(mine)), "any planeswalker");
    assert!(!ch.contains(&Entity::Player(P0)), "an opponent");
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20);
}
