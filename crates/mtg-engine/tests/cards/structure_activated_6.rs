//! Structure-coverage tests (round 6): activated-ability structures shared by many cards
//! that no other test exercised (see `docs/STRUCTURE_COVERAGE.md`). Each test checks that
//! a real card's ability does what its text says.

use super::structure_activated_2::{act, next_turn, shuffled, target_choices, untapped, with_p1p1};
use super::structure_seed_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{CardType, Color, ColorSet};
use mtg_engine::*;

/// Declares `attackers` (controlled by the active player `ap`) attacking `defender` and
/// advances to the declare attackers step.
pub fn attack(t: &mut TestGame, ap: PlayerId, defender: PlayerId, attackers: &[ObjectId]) {
    t.set_step(ap, Step::BeginningOfCombat);
    let decl: Vec<_> = attackers.iter().map(|a| (*a, Entity::Player(defender))).collect();
    declare(t, &decl);
    go_to(t, Step::DeclareAttackers);
}

/// The candidates offered by each "choose entities" decision asked of `p` so far.
pub fn entity_choices(t: &TestGame, p: PlayerId) -> Vec<Vec<Entity>> {
    t.asked()
        .into_iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseEntities { candidates, .. } if q == p => Some(candidates),
            _ => None,
        })
        .collect()
}

#[test]
fn myr_matrix_creates_a_myr_token() {
    cr!("602.2", "111.1", "118.3");
    // "{5}: Create a 1/1 colorless Myr artifact creature token." (and "Myr creatures get
    // +1/+1.")
    supported("Myr Matrix");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Myr Matrix");
    t.lands(P0, "Wastes", 4);
    assert!(act(&mut t, P0, m, "create", &[]).is_err(), "{{5}} needs five mana");
    t.clear_answers();
    t.lands(P0, "Wastes", 1);
    act(&mut t, P0, m, "create", &[]).unwrap();
    assert_eq!(untapped(&t, P0, "Wastes"), 0);
    assert!(!t.obj_now(m).tapped, "no {{T}} in the cost");
    t.resolve_all();
    let myr = subtype_ids(&t, P0, "Myr");
    assert_eq!(myr.len(), 1);
    assert_eq!(count_subtype(&t, P1, "Myr"), 0);
    let o = t.obj_now(myr[0]);
    assert!(o.is_token());
    assert!(o.chars.is(CardType::Artifact) && o.chars.is(CardType::Creature));
    assert!(o.chars.colors.is_colorless());
    assert_eq!(t.pt(myr[0]), (2, 2), "a 1/1 Myr pumped by the Matrix");
}

#[test]
fn cult_guildmage_pings_an_opponent_or_planeswalker() {
    cr!("602.2", "115.1", "120.3a", "120.3c");
    // "{R}, {T}: This creature deals 1 damage to target opponent or planeswalker."
    supported("Cult Guildmage");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Cult Guildmage");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let garruk = t.battlefield(P1, "Garruk Wildspeaker");
    let mine = t.battlefield(P0, "Chandra, Torch of Defiance");
    t.lands(P0, "Mountain", 1);
    act(&mut t, P0, g, "damage", &[Entity::Player(P1)]).unwrap();
    assert!(t.obj_now(g).tapped);
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Player(P1)));
    assert!(!c.contains(&Entity::Player(P0)), "not an opponent");
    assert!(c.contains(&Entity::Object(garruk)) && c.contains(&Entity::Object(mine)));
    assert!(!c.contains(&Entity::Object(bears)));
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 20);
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Cult Guildmage");
    let garruk = t.battlefield(P1, "Garruk Wildspeaker");
    t.lands(P0, "Mountain", 1);
    act(&mut t, P0, g, "damage", &[Entity::Object(garruk)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(garruk, "loyalty"), 2);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn rubblebelt_maverick_exiles_itself_from_the_graveyard_for_a_counter() {
    cr!("602.2", "602.5d", "118.3", "122.1");
    // "{G}, Exile this card from your graveyard: Put a +1/+1 counter on target creature.
    // Activate only as a sorcery."
    supported("Rubblebelt Maverick");
    let mut t = TestGame::new(2);
    let on_bf = t.battlefield(P0, "Rubblebelt Maverick");
    let m = t.graveyard(P0, "Rubblebelt Maverick");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    assert!(
        act(&mut t, P0, on_bf, "counter", &[Entity::Object(bears)]).is_err(),
        "only from the graveyard"
    );
    t.clear_answers();
    t.set_step(P0, Step::Upkeep);
    assert!(
        act(&mut t, P0, m, "counter", &[Entity::Object(bears)]).is_err(),
        "only as a sorcery"
    );
    t.clear_answers();
    t.set_step(P0, Step::PrecombatMain);
    act(&mut t, P0, m, "counter", &[Entity::Object(bears)]).unwrap();
    assert!(t.in_exile("Rubblebelt Maverick"), "exiled as a cost");
    assert!(!t.in_graveyard(P0, "Rubblebelt Maverick"));
    assert_eq!(untapped(&t, P0, "Forest"), 0);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.counters(other, "+1/+1"), 0);
    assert_eq!(t.counters(on_bf, "+1/+1"), 0);
}

#[test]
fn loch_korrigan_pumps_itself_for_hybrid_mana() {
    cr!("602.2", "107.4e", "611.2a", "514.2");
    // "{U/B}: This creature gets +1/+1 until end of turn."
    supported("Loch Korrigan");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Loch Korrigan");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Mountain");
    assert!(act(&mut t, P0, k, "gets", &[]).is_err(), "{{R}} can't pay {{U/B}}");
    let i = t.battlefield(P0, "Island");
    let s = t.battlefield(P0, "Swamp");
    act(&mut t, P0, k, "gets", &[]).unwrap();
    t.resolve_all();
    act(&mut t, P0, k, "gets", &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(i).tapped && t.obj_now(s).tapped, "paid with {{U}} and {{B}}");
    assert_eq!(t.pt(k), (3, 3));
    assert_eq!(t.pt(bears), (2, 2));
    next_turn(&mut t);
    assert_eq!(t.pt(k), (1, 1));
}

#[test]
fn silkwing_scout_sacrifices_to_fetch_a_basic_land_tapped() {
    cr!("602.2", "701.21a", "701.23a", "205.4c");
    // "{G}, Sacrifice this creature: Search your library for a basic land card, put that
    // card onto the battlefield tapped, then shuffle."
    supported("Silkwing Scout");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Silkwing Scout");
    t.lands(P0, "Forest", 1);
    t.library_top(P0, "Taiga");
    let island = t.library_top(P0, "Island");
    t.library_top(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(island)]);
    act(&mut t, P0, s, "search", &[]).unwrap();
    assert!(!t.on_battlefield(s), "sacrificed as a cost");
    t.resolve_all();
    let c = entity_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(island)));
    assert_eq!(c.len(), 1, "only basic land cards: {c:?}");
    let v = t.named_on_battlefield("Island");
    assert_eq!(v.len(), 1);
    assert!(t.obj_now(v[0]).tapped, "enters tapped");
    assert_eq!(t.obj_now(v[0]).controller, P0);
    assert!(t.named_on_battlefield("Taiga").is_empty());
    assert!(shuffled(&t, P0));
}

#[test]
fn sneaking_guide_makes_a_small_creature_unblockable() {
    cr!("602.2", "115.1", "509.1b", "514.2", "608.2b");
    // "{2}, {T}: Target creature with power 2 or less can't be blocked this turn."
    supported("Sneaking Guide");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Sneaking Guide");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.lands(P0, "Wastes", 2);
    act(&mut t, P0, g, "blocked", &[Entity::Object(bears)]).unwrap();
    assert_eq!(untapped(&t, P0, "Wastes"), 0);
    assert!(t.obj_now(g).tapped);
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(bears)) && c.contains(&Entity::Object(g)));
    assert!(!c.contains(&Entity::Object(giant)), "power 3");
    t.resolve_all();
    attack(&mut t, P0, P1, &[bears, giant]);
    assert!(!t.g.can_block(wall, bears), "it can't be blocked");
    assert!(t.g.can_block(wall, giant));
    t.advance_to(P0, Step::PrecombatMain);
    attack(&mut t, P0, P1, &[bears]);
    assert!(t.g.can_block(wall, bears), "only this turn");
    // A target whose power rose above 2 before resolution is illegal (CR 608.2b).
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Sneaking Guide");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.lands(P0, "Wastes", 2);
    t.lands(P0, "Forest", 1);
    act(&mut t, P0, g, "blocked", &[Entity::Object(bears)]).unwrap();
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    attack(&mut t, P0, P1, &[bears]);
    assert!(t.g.can_block(wall, bears), "the ability did nothing");
}

#[test]
fn fencer_clique_returns_to_the_top_of_its_owners_library() {
    cr!("602.2", "400.7");
    // "{U}: Put this creature on top of its owner's library."
    supported("Fencer Clique");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P1, "Fencer Clique");
    t.library_top(P1, "Forest");
    t.library_top(P0, "Island");
    t.lands(P0, "Mountain", 3);
    let treason = t.hand(P0, "Act of Treason");
    t.cast(P0, treason).target(f).go();
    t.resolve_all();
    assert_eq!(t.obj_now(f).controller, P0);
    t.lands(P0, "Island", 1);
    let (l0, l1) = (t.library_size(P0), t.library_size(P1));
    act(&mut t, P0, f, "top", &[]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(f));
    assert_eq!(t.library_size(P1), l1 + 1, "its owner's library");
    assert_eq!(t.library_size(P0), l0, "not its controller's");
    let top = *t.g.player(P1).library.last().unwrap();
    assert_eq!(t.obj(top).chars.name, "Fencer Clique", "on top");
    assert_ne!(top, f, "a new object (CR 400.7)");
}

#[test]
fn deadapult_sacrifices_a_zombie_to_deal_two_damage() {
    cr!("602.2", "118.3", "701.21a", "115.4", "120.3a", "120.3e");
    // "{R}, Sacrifice a Zombie: This enchantment deals 2 damage to any target."
    supported("Deadapult");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Deadapult");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Walking Corpse");
    t.lands(P0, "Mountain", 2);
    assert!(
        act(&mut t, P0, d, "damage", &[Entity::Player(P1)]).is_err(),
        "no Zombie of yours to sacrifice"
    );
    t.clear_answers();
    let z = t.battlefield(P0, "Walking Corpse");
    t.answer_choose(P0, &[Entity::Object(z)]);
    act(&mut t, P0, d, "damage", &[Entity::Player(P1)]).unwrap();
    assert!(!t.on_battlefield(z), "the Zombie was sacrificed");
    assert!(t.on_battlefield(bears));
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20);
    // Any target: a creature.
    let z2 = t.battlefield(P0, "Walking Corpse");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(z2)]);
    act(&mut t, P0, d, "damage", &[Entity::Object(giant)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 2);
}

#[test]
fn explosive_apparatus_sacrifices_itself_to_deal_two_damage() {
    cr!("602.2", "118.3", "701.21a", "115.4", "120.3e", "120.3a");
    // "{3}, {T}, Sacrifice this artifact: It deals 2 damage to any target."
    supported("Explosive Apparatus");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Explosive Apparatus");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 3);
    act(&mut t, P0, a, "damage", &[Entity::Object(giant)]).unwrap();
    assert!(!t.on_battlefield(a));
    assert!(t.in_graveyard(P0, "Explosive Apparatus"));
    assert_eq!(untapped(&t, P0, "Wastes"), 0);
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 2);
    assert_eq!(t.obj_now(bears).damage, 0);
    // A player.
    let a = t.battlefield(P0, "Explosive Apparatus");
    t.lands(P0, "Wastes", 3);
    act(&mut t, P0, a, "damage", &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn stuffed_bear_becomes_a_green_bear_creature_until_end_of_turn() {
    cr!("602.2", "613.1d", "613.1e", "611.2a", "514.2");
    // "{2}: This artifact becomes a 4/4 green Bear artifact creature until end of turn."
    supported("Stuffed Bear");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Stuffed Bear");
    assert!(!t.obj_now(b).chars.is_creature());
    t.lands(P0, "Wastes", 2);
    act(&mut t, P0, b, "becomes", &[]).unwrap();
    t.resolve_all();
    let o = t.obj_now(b);
    assert!(o.chars.is_creature() && o.chars.is(CardType::Artifact));
    assert!(o.chars.has_subtype("Bear"));
    assert_eq!(o.chars.colors, ColorSet::single(Color::Green));
    assert_eq!(t.pt(b), (4, 4));
    next_turn(&mut t);
    let o = t.obj_now(b);
    assert!(!o.chars.is_creature() && o.chars.is(CardType::Artifact));
    assert!(!o.chars.has_subtype("Bear"));
    assert!(o.chars.colors.is_colorless());
}

#[test]
fn telimtors_darts_pings_a_player_or_planeswalker() {
    cr!("602.2", "115.1", "120.3a", "120.3c");
    // "{2}, {T}: This artifact deals 1 damage to target player or planeswalker."
    supported("Telim'Tor's Darts");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Telim'Tor's Darts");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let garruk = t.battlefield(P1, "Garruk Wildspeaker");
    t.lands(P0, "Wastes", 1);
    assert!(act(&mut t, P0, d, "damage", &[Entity::Player(P1)]).is_err());
    t.clear_answers();
    t.lands(P0, "Wastes", 1);
    act(&mut t, P0, d, "damage", &[Entity::Object(garruk)]).unwrap();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Player(P0)) && c.contains(&Entity::Player(P1)));
    assert!(c.contains(&Entity::Object(garruk)));
    assert!(!c.contains(&Entity::Object(bears)));
    t.resolve_all();
    assert_eq!(t.counters(garruk, "loyalty"), 2);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn sinister_starfish_surveils_one() {
    cr!("602.2", "701.25a");
    // "{T}: Surveil 1."
    supported("Sinister Starfish");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sinister Starfish");
    let under = t.library_top(P0, "Island");
    let top = t.library_top(P0, "Forest");
    t.answer(P0, DecisionKind::Surveil, Answer::Split(vec![], vec![top]));
    act(&mut t, P0, s, "surveil", &[]).unwrap();
    assert!(t.obj_now(s).tapped);
    t.resolve_all();
    assert_eq!(surveils(&t), vec![1]);
    assert!(t.in_graveyard(P0, "Forest"));
    assert_eq!(*t.g.player(P0).library.last().unwrap(), under);
    assert_eq!(t.hand_size(P0), 0);
    // Keeping it on top.
    t.g.untap(s);
    t.answer(P0, DecisionKind::Surveil, Answer::Split(vec![under], vec![]));
    act(&mut t, P0, s, "surveil", &[]).unwrap();
    t.resolve_all();
    assert_eq!(*t.g.player(P0).library.last().unwrap(), under);
    assert!(!t.in_graveyard(P0, "Island"));
}

#[test]
fn acolyte_of_xathrid_makes_target_player_lose_life() {
    cr!("602.2", "115.1", "119.3");
    // "{1}{B}, {T}: Target player loses 1 life."
    supported("Acolyte of Xathrid");
    for target in [P1, P0] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Acolyte of Xathrid");
        t.lands(P0, "Swamp", 2);
        act(&mut t, P0, a, "loses", &[Entity::Player(target)]).unwrap();
        assert!(t.obj_now(a).tapped);
        assert_eq!(untapped(&t, P0, "Swamp"), 0);
        t.resolve_all();
        assert_eq!(t.life(target), 19);
        assert_eq!(t.life(if target == P1 { P0 } else { P1 }), 20);
    }
}

#[test]
fn tower_of_calamities_deals_twelve_to_a_creature() {
    cr!("602.2", "115.1", "120.3e");
    // "{8}, {T}: This artifact deals 12 damage to target creature."
    supported("Tower of Calamities");
    let mut t = TestGame::new(2);
    let tw = t.battlefield(P0, "Tower of Calamities");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let garruk = t.battlefield(P1, "Garruk Wildspeaker");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 8);
    act(&mut t, P0, tw, "damage", &[Entity::Object(wurm)]).unwrap();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(wurm)) && c.contains(&Entity::Object(bears)));
    assert!(!c.contains(&Entity::Player(P1)) && !c.contains(&Entity::Object(garruk)));
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn unstable_obelisk_sacrifices_itself_to_destroy_a_permanent() {
    cr!("602.2", "118.3", "701.21a", "701.8a");
    // "{7}, {T}, Sacrifice this artifact: Destroy target permanent."
    supported("Unstable Obelisk");
    let mut t = TestGame::new(2);
    let o = t.battlefield(P0, "Unstable Obelisk");
    let land = t.battlefield(P1, "Forest");
    let ench = t.battlefield(P1, "Glorious Anthem");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 6);
    assert!(act(&mut t, P0, o, "destroy", &[Entity::Object(land)]).is_err());
    t.clear_answers();
    t.lands(P0, "Wastes", 1);
    act(&mut t, P0, o, "destroy", &[Entity::Object(land)]).unwrap();
    let c = target_choices(&t, P0).pop().unwrap();
    for x in [land, ench, bears] {
        assert!(c.contains(&Entity::Object(x)));
    }
    assert!(!c.contains(&Entity::Player(P1)));
    assert!(!t.on_battlefield(o), "sacrificed as a cost");
    t.resolve_all();
    assert!(!t.on_battlefield(land));
    assert!(t.in_graveyard(P1, "Forest"));
    assert!(t.on_battlefield(ench) && t.on_battlefield(bears));
}

#[test]
fn vindictive_warden_pings_each_opponent() {
    cr!("602.2", "120.3a");
    // "{3}: This creature deals 1 damage to each opponent."
    supported("Vindictive Warden");
    let mut t = TestGame::new(3);
    let w = t.battlefield(P0, "Vindictive Warden");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 3);
    act(&mut t, P0, w, "damage", &[]).unwrap();
    assert!(!t.obj_now(w).tapped);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(PlayerId(2)), 19);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.obj_now(bears).damage, 0);
}

#[test]
fn wall_of_tanglecord_gains_reach_to_block_a_flier() {
    cr!("602.2", "702.17b", "611.2a", "514.2");
    // "{G}: This creature gains reach until end of turn."
    supported("Wall of Tanglecord");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Wall of Tanglecord");
    let other = t.battlefield(P0, "Grizzly Bears");
    let drake = t.battlefield(P1, "Wind Drake");
    t.lands(P0, "Forest", 1);
    attack(&mut t, P1, P0, &[drake]);
    assert!(!t.g.can_block(w, drake), "no reach yet");
    act(&mut t, P0, w, "reach", &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(w).has_keyword(KeywordKind::Reach));
    assert!(!t.obj_now(other).has_keyword(KeywordKind::Reach));
    assert!(t.g.can_block(w, drake));
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(w).has_keyword(KeywordKind::Reach));
}

#[test]
fn goblin_legionnaire_sacrifices_to_prevent_the_next_two_damage() {
    cr!("602.2", "701.21a", "615.1a", "615.7");
    // "{W}, Sacrifice this creature: Prevent the next 2 damage that would be dealt to any
    // target this turn."
    supported("Goblin Legionnaire");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Goblin Legionnaire");
    let giant = t.battlefield(P0, "Hill Giant");
    let other = t.battlefield(P0, "Hill Giant");
    let src = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    act(&mut t, P0, g, "prevent", &[Entity::Object(giant)]).unwrap();
    assert!(!t.on_battlefield(g), "sacrificed as a cost");
    t.resolve_all();
    t.g.deal_damage(src, Entity::Object(giant), 1, false);
    t.settle();
    assert_eq!(t.obj_now(giant).damage, 0, "1 of the 2 prevented");
    t.g.deal_damage(src, Entity::Object(giant), 2, false);
    t.settle();
    assert_eq!(t.obj_now(giant).damage, 1, "the last 1 prevented");
    t.g.deal_damage(src, Entity::Object(other), 2, false);
    t.settle();
    assert_eq!(t.obj_now(other).damage, 2, "only the target");
    // "This turn": an unused shield is gone next turn.
    let g = t.battlefield(P0, "Goblin Legionnaire");
    t.lands(P0, "Plains", 1);
    act(&mut t, P0, g, "prevent", &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    next_turn(&mut t);
    t.g.deal_damage(src, Entity::Player(P0), 2, false);
    t.settle();
    assert_eq!(t.life(P0), 18);
}

#[test]
fn avacynian_priest_taps_a_non_human_creature() {
    cr!("602.2", "115.1", "701.26a");
    // "{1}, {T}: Tap target non-Human creature."
    supported("Avacynian Priest");
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Avacynian Priest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let human = t.battlefield(P1, "Elite Vanguard");
    let other = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Wastes", 1);
    act(&mut t, P0, p, "tap target", &[Entity::Object(bears)]).unwrap();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(bears)));
    assert!(!c.contains(&Entity::Object(human)), "a Human");
    assert!(!c.contains(&Entity::Object(p)), "a Human");
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert!(!t.obj_now(other).tapped && !t.obj_now(human).tapped);
}

#[test]
fn candy_trail_sacrifices_to_gain_life_and_draw() {
    cr!("602.2", "701.21a", "119.3", "121.1");
    // "{2}, {T}, Sacrifice this artifact: You gain 3 life and draw a card."
    supported("Candy Trail");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Candy Trail");
    t.lands(P0, "Wastes", 2);
    act(&mut t, P0, c, "gain", &[]).unwrap();
    assert!(!t.on_battlefield(c));
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P1), 0);
}

#[test]
fn barbarian_lunatic_sacrifices_itself_to_damage_a_creature() {
    cr!("602.2", "701.21a", "115.1", "120.3e");
    // "{2}{R}, Sacrifice this creature: It deals 2 damage to target creature."
    supported("Barbarian Lunatic");
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Barbarian Lunatic");
    let giant = t.battlefield(P1, "Hill Giant");
    let garruk = t.battlefield(P1, "Garruk Wildspeaker");
    t.lands(P0, "Mountain", 3);
    act(&mut t, P0, l, "damage", &[Entity::Object(giant)]).unwrap();
    assert!(!t.on_battlefield(l));
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(giant)));
    assert!(!c.contains(&Entity::Player(P1)) && !c.contains(&Entity::Object(garruk)));
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 2);
    assert_eq!(t.counters(garruk, "loyalty"), 3);
}

#[test]
fn blood_rites_sacrifices_a_creature_to_deal_two_damage() {
    cr!("602.2", "118.3", "701.21a", "115.4", "120.3a");
    // "{1}{R}, Sacrifice a creature: This enchantment deals 2 damage to any target."
    supported("Blood Rites");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Blood Rites");
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    assert!(
        act(&mut t, P0, b, "damage", &[Entity::Player(P1)]).is_err(),
        "no creature of yours to sacrifice"
    );
    t.clear_answers();
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.answer_choose(P0, &[Entity::Object(elves)]);
    act(&mut t, P0, b, "damage", &[Entity::Player(P1)]).unwrap();
    assert!(!t.on_battlefield(elves));
    assert!(t.on_battlefield(b), "the enchantment stays");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn advance_scout_gives_target_creature_first_strike() {
    cr!("602.2", "702.7b", "611.2a", "514.2");
    // "{W}: Target creature gains first strike until end of turn."
    supported("Advance Scout");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Advance Scout");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    act(&mut t, P0, s, "first strike", &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::FirstStrike));
    assert!(!t.obj_now(other).has_keyword(KeywordKind::FirstStrike));
    // It kills a 2/2 blocker before that blocker deals damage.
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[(blocker, bears)]);
    assert!(!t.on_battlefield(blocker));
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
    next_turn(&mut t);
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::FirstStrike));
}

#[test]
fn boros_reckoner_gains_first_strike_for_hybrid_mana() {
    cr!("602.2", "107.4e", "611.2a", "514.2");
    // "{R/W}: This creature gains first strike until end of turn."
    supported("Boros Reckoner");
    for land in ["Mountain", "Plains"] {
        let mut t = TestGame::new(2);
        let r = t.battlefield(P0, "Boros Reckoner");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P0, "Island");
        assert!(act(&mut t, P0, r, "first strike", &[]).is_err(), "{{U}} can't pay");
        let l = t.battlefield(P0, land);
        act(&mut t, P0, r, "first strike", &[]).unwrap();
        assert!(t.obj_now(l).tapped);
        t.resolve_all();
        assert!(t.obj_now(r).has_keyword(KeywordKind::FirstStrike), "{land}");
        assert!(!t.obj_now(bears).has_keyword(KeywordKind::FirstStrike));
        next_turn(&mut t);
        assert!(!t.obj_now(r).has_keyword(KeywordKind::FirstStrike));
    }
}

#[test]
fn cephalid_scout_sacrifices_a_land_to_draw() {
    cr!("602.2", "118.3", "701.21a", "121.1");
    // "{2}{U}, Sacrifice a land: Draw a card."
    supported("Cephalid Scout");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Cephalid Scout");
    t.lands(P0, "Island", 3);
    let forest = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(forest)]);
    act(&mut t, P0, s, "draw", &[]).unwrap();
    let c = entity_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(forest)));
    assert!(!c.contains(&Entity::Object(bears)) && !c.contains(&Entity::Object(s)));
    assert!(!t.on_battlefield(forest), "sacrificed");
    assert!(t.on_battlefield(s) && t.on_battlefield(bears));
    assert_eq!(t.named_on_battlefield("Island").len(), 3);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn chandra_minus_three_deals_four_to_a_creature() {
    cr!("606.4", "115.1", "120.3e");
    // "−3: Chandra deals 4 damage to target creature."
    supported("Chandra, Torch of Defiance");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Chandra, Torch of Defiance");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    let garruk = t.battlefield(P1, "Garruk Wildspeaker");
    act(&mut t, P0, c, "-3:", &[Entity::Object(wurm)]).unwrap();
    assert_eq!(t.counters(c, "loyalty"), 1, "the loyalty cost is paid");
    let ch = target_choices(&t, P0).pop().unwrap();
    assert!(ch.contains(&Entity::Object(giant)));
    assert!(!ch.contains(&Entity::Player(P1)) && !ch.contains(&Entity::Object(garruk)));
    t.resolve_all();
    assert!(!t.on_battlefield(wurm), "4 damage to a 6/4");
    assert_eq!(t.obj_now(giant).damage, 0);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn commanders_sphere_sacrifices_itself_to_draw() {
    cr!("602.2", "701.21a", "121.1");
    // "Sacrifice this artifact: Draw a card."
    supported("Commander's Sphere");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Commander's Sphere");
    t.g.tap(s);
    act(&mut t, P0, s, "draw", &[]).unwrap();
    assert!(!t.on_battlefield(s), "no {{T}} needed: a tapped Sphere is sacrificed");
    assert!(t.in_graveyard(P0, "Commander's Sphere"));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.hand_size(P1), 0);
}

#[test]
fn crossroads_consecrator_pumps_an_attacking_human() {
    cr!("602.2", "115.1", "611.2a", "514.2");
    // "{G}, {T}: Target attacking Human gets +1/+1 until end of turn."
    supported("Crossroads Consecrator");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Crossroads Consecrator");
    let human = t.battlefield(P0, "Elite Vanguard");
    let idle = t.battlefield(P0, "Elite Vanguard");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    attack(&mut t, P0, P1, &[human, bears]);
    act(&mut t, P0, c, "gets", &[Entity::Object(human)]).unwrap();
    let ch = target_choices(&t, P0).pop().unwrap();
    assert!(ch.contains(&Entity::Object(human)));
    assert!(!ch.contains(&Entity::Object(bears)), "not a Human");
    assert!(!ch.contains(&Entity::Object(idle)), "not attacking");
    assert!(!ch.contains(&Entity::Object(c)), "not attacking");
    t.resolve_all();
    assert_eq!(t.pt(human), (3, 2));
    assert_eq!(t.pt(idle), (2, 1));
    next_turn(&mut t);
    assert_eq!(t.pt(human), (2, 1));
}

#[test]
fn dawnray_archer_pings_an_attacking_or_blocking_creature() {
    cr!("602.2", "115.1", "120.3e");
    // "{W}, {T}: This creature deals 1 damage to target attacking or blocking creature."
    supported("Dawnray Archer");
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Llanowar Elves");
    let idle = t.battlefield(P0, "Llanowar Elves");
    let a = t.battlefield(P1, "Dawnray Archer");
    t.lands(P1, "Plains", 1);
    attack(&mut t, P0, P1, &[attacker]);
    act(&mut t, P1, a, "damage", &[Entity::Object(attacker)]).unwrap();
    let c = target_choices(&t, P1).pop().unwrap();
    assert!(c.contains(&Entity::Object(attacker)));
    assert!(!c.contains(&Entity::Object(idle)) && !c.contains(&Entity::Object(a)));
    t.resolve_all();
    assert!(!t.on_battlefield(attacker));
    assert!(t.on_battlefield(idle));
    // A blocker.
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Hill Giant");
    let a = t.battlefield(P1, "Dawnray Archer");
    let blocker = t.battlefield(P1, "Llanowar Elves");
    t.lands(P1, "Plains", 1);
    attack(&mut t, P0, P1, &[attacker]);
    block(&mut t, P1, &[(blocker, attacker)]);
    go_to(&mut t, Step::DeclareBlockers);
    act(&mut t, P1, a, "damage", &[Entity::Object(blocker)]).unwrap();
    assert!(target_choices(&t, P1).pop().unwrap().contains(&Entity::Object(blocker)));
    t.resolve_all();
    assert!(!t.on_battlefield(blocker));
}

#[test]
fn triskelion_removes_counters_to_ping() {
    cr!("602.2", "118.3", "122.1", "115.4", "120.3a", "120.3e");
    // "Remove a +1/+1 counter from this creature: It deals 1 damage to any target."
    supported("Triskelion");
    let mut t = TestGame::new(2);
    let tr = with_p1p1(&mut t, P0, "Triskelion", 2);
    let elves = t.battlefield(P1, "Llanowar Elves");
    act(&mut t, P0, tr, "damage", &[Entity::Player(P1)]).unwrap();
    assert_eq!(t.counters(tr, "+1/+1"), 1, "removed as a cost");
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    act(&mut t, P0, tr, "damage", &[Entity::Object(elves)]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(elves));
    assert_eq!(t.pt(tr), (1, 1));
    assert!(
        act(&mut t, P0, tr, "damage", &[Entity::Player(P1)]).is_err(),
        "no counter left"
    );
    assert_eq!(t.life(P1), 19);
    // "It" still deals the damage if Triskelion has left the battlefield.
    let mut t = TestGame::new(2);
    let tr = with_p1p1(&mut t, P0, "Triskelion", 1);
    act(&mut t, P0, tr, "damage", &[Entity::Player(P1)]).unwrap();
    assert!(t.g.destroy(tr, None));
    assert!(t.in_graveyard(P0, "Triskelion"));
    t.resolve_all();
    assert_eq!(t.life(P1), 19, "last known information");
}

#[test]
fn destructive_digger_sacrifices_an_artifact_or_land_to_draw() {
    cr!("602.2", "118.3", "701.21a", "121.1");
    // "{3}, {T}, Sacrifice an artifact or land: Draw a card."
    supported("Destructive Digger");
    for sac_artifact in [true, false] {
        let mut t = TestGame::new(2);
        let d = t.battlefield(P0, "Destructive Digger");
        t.lands(P0, "Wastes", 3);
        let art = t.battlefield(P0, "Ornithopter");
        let land = t.battlefield(P0, "Forest");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let theirs = t.battlefield(P1, "Ornithopter");
        let pick = if sac_artifact { art } else { land };
        t.answer_choose(P0, &[Entity::Object(pick)]);
        act(&mut t, P0, d, "draw", &[]).unwrap();
        let c = entity_choices(&t, P0).pop().unwrap();
        assert!(c.contains(&Entity::Object(art)) && c.contains(&Entity::Object(land)));
        assert!(!c.contains(&Entity::Object(bears)) && !c.contains(&Entity::Object(d)));
        assert!(!c.contains(&Entity::Object(theirs)));
        assert!(!t.on_battlefield(pick));
        assert!(t.on_battlefield(if sac_artifact { land } else { art }));
        assert!(t.obj_now(d).tapped);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), 1);
    }
}

#[test]
fn dirty_wererat_discards_to_regenerate() {
    cr!("602.2", "118.3", "701.9a", "701.19a", "514.2");
    // "{B}, Discard a card: Regenerate this creature."
    supported("Dirty Wererat");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Dirty Wererat");
    t.lands(P0, "Swamp", 2);
    assert!(act(&mut t, P0, w, "regenerate", &[]).is_err(), "no card to discard");
    t.clear_answers();
    let card = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(card)]);
    act(&mut t, P0, w, "regenerate", &[]).unwrap();
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "discarded as a cost");
    assert_eq!(t.hand_size(P0), 0);
    t.resolve_all();
    assert!(!t.g.destroy(w, None), "the shield replaces destruction");
    t.settle();
    assert!(t.on_battlefield(w));
    assert!(t.obj_now(w).tapped, "regenerated: tapped");
    assert!(t.g.destroy(w, None), "the shield is used up");
    assert!(t.in_graveyard(P0, "Dirty Wererat"));
    // An unused shield ends with the turn (CR 514.2).
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Dirty Wererat");
    t.lands(P0, "Swamp", 1);
    let card = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(card)]);
    act(&mut t, P0, w, "regenerate", &[]).unwrap();
    t.resolve_all();
    next_turn(&mut t);
    assert!(t.g.destroy(w, None), "no shield left next turn");
    assert!(t.in_graveyard(P0, "Dirty Wererat"));
}

#[test]
fn expendable_troops_sacrifice_to_damage_an_attacker() {
    cr!("602.2", "701.21a", "115.1", "120.3e");
    // "{T}, Sacrifice this creature: It deals 2 damage to target attacking or blocking
    // creature."
    supported("Expendable Troops");
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Grizzly Bears");
    let idle = t.battlefield(P0, "Grizzly Bears");
    let tr = t.battlefield(P1, "Expendable Troops");
    attack(&mut t, P0, P1, &[attacker]);
    act(&mut t, P1, tr, "damage", &[Entity::Object(attacker)]).unwrap();
    assert!(!t.on_battlefield(tr), "sacrificed");
    let c = target_choices(&t, P1).pop().unwrap();
    assert!(c.contains(&Entity::Object(attacker)));
    assert!(!c.contains(&Entity::Object(idle)));
    t.resolve_all();
    assert!(!t.on_battlefield(attacker), "2 damage kills the 2/2");
    assert!(t.on_battlefield(idle));
    // A blocking creature.
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P1, "Hill Giant");
    let tr = t.battlefield(P1, "Expendable Troops");
    let blocker = t.battlefield(P0, "Grizzly Bears");
    let idle = t.battlefield(P0, "Grizzly Bears");
    attack(&mut t, P1, P0, &[attacker]);
    block(&mut t, P0, &[(blocker, attacker)]);
    go_to(&mut t, Step::DeclareBlockers);
    act(&mut t, P1, tr, "damage", &[Entity::Object(blocker)]).unwrap();
    let c = target_choices(&t, P1).pop().unwrap();
    assert!(c.contains(&Entity::Object(blocker)) && c.contains(&Entity::Object(attacker)));
    assert!(!c.contains(&Entity::Object(idle)), "neither attacking nor blocking");
    t.resolve_all();
    assert!(!t.on_battlefield(blocker));
    assert!(t.on_battlefield(idle));
}

#[test]
fn sacrifice_to_destroy_an_enchantment() {
    cr!("602.2", "701.21a", "115.1", "701.8a");
    // Druid Lyrist: "{G}, {T}, Sacrifice this creature: Destroy target enchantment."
    // Elf Replica: "{1}{G}, Sacrifice this creature: Destroy target enchantment."
    for (card, lands, tap) in [("Druid Lyrist", 1, true), ("Elf Replica", 2, false)] {
        supported(card);
        let mut t = TestGame::new(2);
        let d = t.battlefield(P0, card);
        let anthem = t.battlefield(P1, "Glorious Anthem");
        let other = t.battlefield(P1, "Glorious Anthem");
        let art = t.battlefield(P1, "Ornithopter");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Forest", lands - 1);
        if tap {
            t.g.tap(d);
            assert!(
                act(&mut t, P0, d, "destroy", &[Entity::Object(anthem)]).is_err(),
                "{card}: tapped"
            );
            t.clear_answers();
            t.g.untap(d);
        }
        t.lands(P0, "Forest", 1);
        act(&mut t, P0, d, "destroy", &[Entity::Object(anthem)]).unwrap();
        assert!(!t.on_battlefield(d), "{card}: sacrificed");
        assert_eq!(untapped(&t, P0, "Forest"), 0, "{card}");
        let c = target_choices(&t, P0).pop().unwrap();
        assert!(c.contains(&Entity::Object(other)), "{card}");
        assert!(!c.contains(&Entity::Object(art)) && !c.contains(&Entity::Object(bears)));
        t.resolve_all();
        assert!(!t.on_battlefield(anthem), "{card}");
        assert!(t.on_battlefield(other), "{card}");
    }
}

#[test]
fn dwarven_miner_destroys_a_nonbasic_land() {
    cr!("602.2", "115.1", "205.4c", "701.8a");
    // "{2}{R}, {T}: Destroy target nonbasic land."
    supported("Dwarven Miner");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Dwarven Miner");
    let taiga = t.battlefield(P1, "Taiga");
    let mountain = t.battlefield(P1, "Mountain");
    t.lands(P0, "Mountain", 3);
    act(&mut t, P0, m, "destroy", &[Entity::Object(taiga)]).unwrap();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(taiga)));
    assert!(!c.contains(&Entity::Object(mountain)), "basic");
    t.resolve_all();
    assert!(!t.on_battlefield(taiga));
    assert!(t.in_graveyard(P1, "Taiga"));
    assert!(t.on_battlefield(mountain));
}

#[test]
fn cabal_torturer_shrinks_a_creature_until_end_of_turn() {
    cr!("602.2", "115.1", "611.2a", "514.2");
    // "{B}, {T}: Target creature gets -1/-1 until end of turn."
    supported("Cabal Torturer");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Cabal Torturer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    act(&mut t, P0, c, "-1/-1", &[Entity::Object(bears)]).unwrap();
    assert!(t.obj_now(c).tapped);
    t.resolve_all();
    assert_eq!(t.pt(bears), (1, 1));
    assert_eq!(t.pt(other), (2, 2));
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
    // A 1/1 dies.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Cabal Torturer");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Swamp", 1);
    act(&mut t, P0, c, "-1/-1", &[Entity::Object(elves)]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(elves));
}
