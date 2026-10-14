//! Structure-coverage tests (round 8): activated-ability structures shared by many cards
//! that no other test exercised (see `docs/STRUCTURE_COVERAGE.md`). Each test checks that
//! a real card's ability does what its text says.

use super::structure_activated_2::{act, next_turn, shuffled, target_choices, untapped, with_p1p1};
use super::structure_seed_common::*;
use mtg_engine::combat::{block_declaration_legal, block_options};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{CardType, Color, ColorSet};
use mtg_engine::*;

/// The candidates offered by each "choose entities" decision asked of `p` so far.
fn entity_choices(t: &TestGame, p: PlayerId) -> Vec<Vec<Entity>> {
    t.asked()
        .into_iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseEntities { candidates, .. } if q == p => Some(candidates),
            _ => None,
        })
        .collect()
}

/// The sources of the damage dealt to `target` this turn.
fn damage_sources(t: &TestGame, target: Entity) -> Vec<(ObjectId, u32)> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Damage {
                source,
                target: tg,
                amount,
                ..
            } if *tg == target => Some((*source, *amount)),
            _ => None,
        })
        .collect()
}

/// Whether `decl` would be a legal declaration of blockers for `dp` now (CR 509.1a–c).
fn legal_blocks(t: &TestGame, dp: PlayerId, decl: &[(ObjectId, ObjectId)]) -> bool {
    let opts = block_options(&t.g, &[dp]);
    block_declaration_legal(&t.g, &opts, decl)
}

#[test]
fn hopeful_vigil_sacrifices_itself_on_resolution() {
    cr!("602.2", "701.21a", "118.3");
    // "{2}{W}: Sacrifice this enchantment." (and "When this enchantment is put into a
    // graveyard from the battlefield, scry 2.")
    supported("Hopeful Vigil");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Hopeful Vigil");
    t.lands(P0, "Plains", 2);
    assert!(act(&mut t, P0, v, "sacrifice", &[]).is_err(), "needs {{2}}{{W}}");
    t.clear_answers();
    t.lands(P0, "Plains", 1);
    act(&mut t, P0, v, "sacrifice", &[]).unwrap();
    assert_eq!(untapped(&t, P0, "Plains"), 0);
    assert!(t.on_battlefield(v), "the sacrifice is the effect, not the cost");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hopeful Vigil"));
    assert_eq!(scries(&t), vec![2], "its leaves-the-battlefield trigger");
}

#[test]
fn flailing_soldier_any_player_may_shrink_it() {
    cr!("602.2", "611.2a", "514.2", "704.5f");
    // "{1}: This creature gets -1/-1 until end of turn. Any player may activate this
    // ability."
    supported("Flailing Soldier");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Flailing Soldier");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Wastes", 2);
    t.lands(P0, "Wastes", 1);
    act(&mut t, P1, s, "-1/-1", &[]).unwrap();
    assert_eq!(untapped(&t, P1, "Wastes"), 1, "the opponent paid");
    assert_eq!(untapped(&t, P0, "Wastes"), 1);
    t.resolve_all();
    assert_eq!(t.pt(s), (1, 1));
    assert_eq!(t.pt(bears), (2, 2));
    next_turn(&mut t);
    assert_eq!(t.pt(s), (2, 2), "until end of turn");
    act(&mut t, P1, s, "-1/-1", &[]).unwrap();
    act(&mut t, P0, s, "-1/-1", &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Flailing Soldier"), "0/0");
}

#[test]
fn vulshok_replica_sacrifices_to_damage_a_player_or_planeswalker() {
    cr!("602.2", "115.1", "120.3a", "120.3c", "113.7a", "701.21a");
    // "{1}{R}, Sacrifice this creature: It deals 3 damage to target player or
    // planeswalker."
    supported("Vulshok Replica");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Vulshok Replica");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let garruk = t.battlefield(P1, "Garruk Wildspeaker");
    t.lands(P0, "Mountain", 2);
    act(&mut t, P0, r, "damage", &[Entity::Player(P1)]).unwrap();
    assert!(!t.on_battlefield(r), "sacrificed as a cost");
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Player(P0)) && c.contains(&Entity::Player(P1)));
    assert!(c.contains(&Entity::Object(garruk)));
    assert!(!c.contains(&Entity::Object(bears)), "not a creature");
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 17));
    assert_eq!(
        damage_sources(&t, Entity::Player(P1)),
        vec![(r, 3)],
        "the sacrificed Replica (last known information) is the source"
    );
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Vulshok Replica");
    let garruk = t.battlefield(P1, "Garruk Wildspeaker");
    t.g.add_counters(Entity::Object(garruk), "loyalty", 2, None);
    t.lands(P0, "Mountain", 2);
    act(&mut t, P0, r, "damage", &[Entity::Object(garruk)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(garruk, "loyalty"), 2);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn kelpie_guide_untaps_another_permanent_you_control() {
    cr!("602.2", "115.1", "701.26b");
    // "{T}: Untap another target permanent you control."
    supported("Kelpie Guide");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kelpie Guide");
    let land = t.battlefield(P0, "Island");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Island");
    t.g.tap(land);
    t.g.tap(bears);
    t.g.tap(theirs);
    act(&mut t, P0, k, "untap", &[Entity::Object(land)]).unwrap();
    assert!(t.obj_now(k).tapped);
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(land)) && c.contains(&Entity::Object(bears)));
    assert!(!c.contains(&Entity::Object(k)), "another");
    assert!(!c.contains(&Entity::Object(theirs)), "not yours");
    t.resolve_all();
    assert!(!t.obj_now(land).tapped);
    assert!(t.obj_now(bears).tapped && t.obj_now(theirs).tapped);
}

#[test]
fn pardic_swordsmith_discards_at_random_to_pump_power() {
    cr!("602.2", "118.3", "701.9b", "611.2a", "514.2");
    // "{R}, Discard a card at random: This creature gets +2/+0 until end of turn."
    supported("Pardic Swordsmith");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Pardic Swordsmith");
    t.lands(P0, "Mountain", 2);
    assert!(act(&mut t, P0, s, "discard", &[]).is_err(), "no card to discard");
    t.clear_answers();
    t.hand(P0, "Forest");
    t.hand(P0, "Island");
    act(&mut t, P0, s, "discard", &[]).unwrap();
    assert_eq!(t.hand_size(P0), 1, "discarded as a cost");
    assert_eq!(t.graveyard_size(P0), 1);
    assert!(entity_choices(&t, P0).is_empty(), "at random: nobody chooses");
    t.resolve_all();
    assert_eq!(t.pt(s), (3, 1));
    next_turn(&mut t);
    assert_eq!(t.pt(s), (1, 1));
}

#[test]
fn goblin_artillery_damages_any_target_and_you() {
    cr!("602.2", "120.3a", "120.3e", "608.2b");
    ruling!(
        "Goblin Artillery",
        "the entire ability doesn't resolve. You won't be dealt any damage"
    );
    // "{T}: This creature deals 2 damage to any target and 3 damage to you."
    supported("Goblin Artillery");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Goblin Artillery");
    act(&mut t, P0, a, "damage", &[Entity::Player(P1)]).unwrap();
    assert!(t.obj_now(a).tapped);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (17, 18));
    // A creature target.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Goblin Artillery");
    let bears = t.battlefield(P1, "Grizzly Bears");
    act(&mut t, P0, a, "damage", &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!((t.life(P0), t.life(P1)), (17, 20));
    // The target left: nothing happens, not even the damage to you.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Goblin Artillery");
    let bears = t.battlefield(P1, "Grizzly Bears");
    act(&mut t, P0, a, "damage", &[Entity::Object(bears)]).unwrap();
    t.g.destroy(bears, None);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn phyrexian_vault_sacrifices_a_creature_to_draw() {
    cr!("602.2", "118.3", "701.21a", "121.1");
    // "{2}, {T}, Sacrifice a creature: Draw a card."
    supported("Phyrexian Vault");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Phyrexian Vault");
    t.lands(P0, "Wastes", 2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    assert!(act(&mut t, P0, v, "draw", &[]).is_err(), "no creature to sacrifice");
    t.clear_answers();
    let elves = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    act(&mut t, P0, v, "draw", &[]).unwrap();
    let c = entity_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(elves)) && c.contains(&Entity::Object(bears)));
    assert!(!c.contains(&Entity::Object(theirs)), "only yours");
    assert!(!t.on_battlefield(bears) && t.on_battlefield(elves));
    assert!(t.obj_now(v).tapped);
    assert_eq!(untapped(&t, P0, "Wastes"), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.hand_size(P1), 0);
}

#[test]
fn tap_to_grant_a_keyword_until_end_of_turn() {
    cr!("602.2", "115.1", "611.2a", "514.2", "702.10b", "702.19b");
    // Taxi Driver: "{1}, {T}: Target creature gains haste until end of turn."
    // War Chariot: "{3}, {T}: Target creature gains trample until end of turn."
    for (card, kw, mana) in [
        ("Taxi Driver", KeywordKind::Haste, 1),
        ("War Chariot", KeywordKind::Trample, 3),
    ] {
        supported(card);
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, card);
        let bears = if kw == KeywordKind::Haste {
            t.battlefield_sick(P0, "Grizzly Bears")
        } else {
            t.battlefield(P0, "Grizzly Bears")
        };
        let other = t.battlefield(P0, "Grizzly Bears");
        let theirs = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Wastes", mana - 1);
        let text = format!("{kw:?}").to_lowercase();
        assert!(act(&mut t, P0, src, &text, &[Entity::Object(bears)]).is_err());
        t.clear_answers();
        t.lands(P0, "Wastes", 1);
        act(&mut t, P0, src, &text, &[Entity::Object(bears)]).unwrap();
        assert!(t.obj_now(src).tapped, "{card}");
        assert_eq!(untapped(&t, P0, "Wastes"), 0);
        let c = target_choices(&t, P0).pop().unwrap();
        assert!(c.contains(&Entity::Object(theirs)), "{card}: any creature");
        t.resolve_all();
        assert!(t.obj_now(bears).has_keyword(kw), "{card}");
        assert!(!t.obj_now(other).has_keyword(kw), "{card}");
        if kw == KeywordKind::Haste {
            t.set_step(P0, Step::BeginningOfCombat);
            t.attack(&[(bears, Entity::Player(P1))], &[]);
            assert_eq!(t.life(P1), 18, "the summoning-sick creature attacked");
        } else {
            let wall = t.battlefield(P1, "Llanowar Elves");
            t.set_step(P0, Step::BeginningOfCombat);
            t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![1, 1]));
            t.attack(&[(bears, Entity::Player(P1))], &[(wall, bears)]);
            let trample = t.asked().into_iter().any(|(_, d)| {
                matches!(d, Decision::AssignCombatDamage { trample: true, recipients, .. }
                    if recipients.contains(&Entity::Player(P1)))
            });
            assert!(trample, "{card}: may assign damage to the player");
            assert!(!t.on_battlefield(wall));
            assert_eq!(t.life(P1), 19, "1 damage tramples over the 1/1 blocker");
        }
        next_turn(&mut t);
        assert!(!t.obj_now(bears).has_keyword(kw), "{card}: until end of turn");
    }
}

#[test]
fn ashen_firebeast_damages_each_creature_without_flying() {
    cr!("602.2", "120.3e");
    // "{1}{R}: This creature deals 1 damage to each creature without flying."
    supported("Ashen Firebeast");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Ashen Firebeast");
    let mine = t.battlefield(P0, "Llanowar Elves");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let bird = t.battlefield(P1, "Wind Drake");
    t.lands(P0, "Mountain", 2);
    act(&mut t, P0, f, "damage", &[]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(mine), "your creatures too");
    assert_eq!(t.obj_now(theirs).damage, 1);
    assert_eq!(t.obj_now(f).damage, 1, "itself too");
    assert_eq!(t.obj_now(bird).damage, 0, "flying");
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
}

#[test]
fn noose_constrictor_discards_a_card_to_pump_itself() {
    cr!("602.2", "118.3", "701.9a", "611.2a", "514.2");
    // "Discard a card: This creature gets +1/+1 until end of turn."
    supported("Noose Constrictor");
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Noose Constrictor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(act(&mut t, P0, n, "discard", &[]).is_err(), "nothing to discard");
    t.clear_answers();
    let keep = t.hand(P0, "Forest");
    let toss = t.hand(P0, "Island");
    t.answer_choose(P0, &[Entity::Object(toss)]);
    act(&mut t, P0, n, "discard", &[]).unwrap();
    assert!(t.in_graveyard(P0, "Island"));
    assert_eq!(t.zone(keep), Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(t.pt(n), (3, 3));
    assert_eq!(t.pt(bears), (2, 2));
    next_turn(&mut t);
    assert_eq!(t.pt(n), (2, 2));
}

#[test]
fn seismic_assault_discards_a_land_card_to_deal_two_damage() {
    cr!("602.2", "118.3", "701.9a", "120.3a", "120.3e");
    // "Discard a land card: This enchantment deals 2 damage to any target."
    supported("Seismic Assault");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Seismic Assault");
    let spell = t.hand(P0, "Lightning Bolt");
    assert!(
        act(&mut t, P0, s, "damage", &[Entity::Player(P1)]).is_err(),
        "no land card to discard"
    );
    t.clear_answers();
    let forest = t.hand(P0, "Forest");
    act(&mut t, P0, s, "damage", &[Entity::Player(P1)]).unwrap();
    assert_eq!(t.zone(forest), Zone::Graveyard(P0));
    assert_eq!(t.zone(spell), Zone::Hand(P0), "not a land card");
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 18));
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.hand(P0, "Island");
    act(&mut t, P0, s, "damage", &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn knight_of_dawn_gains_protection_from_a_chosen_color() {
    cr!("602.2", "611.2a", "702.16b", "514.2");
    // "{W}{W}: This creature gains protection from the color of your choice until end of
    // turn."
    supported("Knight of Dawn");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Knight of Dawn");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    let red = Color::ALL.iter().position(|c| *c == Color::Red).unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(red));
    act(&mut t, P0, k, "protection", &[]).unwrap();
    t.resolve_all();
    t.lands(P1, "Mountain", 2);
    t.set_step(P1, Step::PrecombatMain);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(bears).go();
    let c = target_choices(&t, P1).pop().unwrap();
    assert!(!c.contains(&Entity::Object(k)), "protection from red");
    assert!(c.contains(&Entity::Object(bears)));
    t.resolve_all();
    // Green spells still can.
    t.lands(P1, "Forest", 1);
    let giant = t.hand(P1, "Giant Growth");
    t.cast(P1, giant).target(k).go();
    let c = target_choices(&t, P1).pop().unwrap();
    assert!(c.contains(&Entity::Object(k)), "not protection from green");
    t.resolve_all();
    next_turn(&mut t);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(k).go();
    let c = target_choices(&t, P1).pop().unwrap();
    assert!(c.contains(&Entity::Object(k)), "until end of turn");
}

#[test]
fn mindless_automaton_discards_for_a_counter() {
    cr!("602.2", "118.3", "701.9a", "122.1");
    // "{1}, Discard a card: Put a +1/+1 counter on this creature."
    supported("Mindless Automaton");
    let mut t = TestGame::new(2);
    let m = with_p1p1(&mut t, P0, "Mindless Automaton", 2);
    let other = with_p1p1(&mut t, P0, "Mindless Automaton", 2);
    t.lands(P0, "Wastes", 1);
    assert!(act(&mut t, P0, m, "discard", &[]).is_err(), "no card to discard");
    t.clear_answers();
    t.hand(P0, "Forest");
    act(&mut t, P0, m, "discard", &[]).unwrap();
    assert!(t.in_graveyard(P0, "Forest"));
    assert_eq!(untapped(&t, P0, "Wastes"), 0);
    t.resolve_all();
    assert_eq!(t.counters(m, "+1/+1"), 3);
    assert_eq!(t.pt(m), (3, 3));
    assert_eq!(t.counters(other, "+1/+1"), 2);
}

#[test]
fn ghost_warden_pumps_target_creature() {
    cr!("602.2", "115.1", "611.2a", "514.2");
    // "{T}: Target creature gets +1/+1 until end of turn."
    supported("Ghost Warden");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Ghost Warden");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    act(&mut t, P0, w, "+1/+1", &[Entity::Object(bears)]).unwrap();
    assert!(t.obj_now(w).tapped);
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3), "any creature, even an opponent's");
    assert_eq!(t.pt(other), (2, 2));
    assert_eq!(t.pt(w), (1, 1));
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn ral_storm_conduit_plus_two_scries_one() {
    cr!("606.3", "606.4", "701.22a");
    // "+2: Scry 1."
    supported("Ral, Storm Conduit");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Ral, Storm Conduit");
    let l = t.counters(r, "loyalty");
    let under = t.library_top(P0, "Island");
    let top = t.library_top(P0, "Forest");
    t.answer(P0, DecisionKind::Scry, Answer::Split(vec![], vec![top]));
    act(&mut t, P0, r, "+2", &[]).unwrap();
    assert_eq!(t.counters(r, "loyalty"), l + 2);
    t.resolve_all();
    assert_eq!(scries(&t), vec![1]);
    assert_eq!(t.g.library_top(P0), Some(under));
    assert_eq!(t.g.player(P0).library[0], top, "on the bottom");
    assert!(act(&mut t, P0, r, "+2", &[]).is_err(), "once per turn");
}

#[test]
fn iron_hills_sacrifices_for_two_counters_on_your_dwarf() {
    cr!("602.2", "602.5d", "115.1", "122.1", "701.21a");
    // "{2}{R}{W}, {T}, Sacrifice this land: Put two +1/+1 counters on target Dwarf you
    // control. Activate only as a sorcery."
    supported("Iron Hills");
    supported("Dwarven Trader");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Iron Hills");
    let dwarf = t.battlefield(P0, "Dwarven Trader");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Dwarven Trader");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Plains", 2);
    t.set_step(P0, Step::Upkeep);
    assert!(
        act(&mut t, P0, h, "counters", &[Entity::Object(dwarf)]).is_err(),
        "only as a sorcery"
    );
    t.clear_answers();
    t.set_step(P0, Step::PrecombatMain);
    act(&mut t, P0, h, "counters", &[Entity::Object(dwarf)]).unwrap();
    assert!(t.in_graveyard(P0, "Iron Hills"), "sacrificed as a cost");
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(dwarf)));
    assert!(!c.contains(&Entity::Object(bears)), "not a Dwarf");
    assert!(!c.contains(&Entity::Object(theirs)), "not yours");
    t.resolve_all();
    assert_eq!(t.counters(dwarf, "+1/+1"), 2);
    assert_eq!(t.counters(theirs, "+1/+1"), 0);
}

#[test]
fn hardened_tactician_sacrifices_a_token_to_draw() {
    cr!("602.2", "118.3", "701.21a", "111.1");
    // "{1}, Sacrifice a token: Draw a card."
    supported("Hardened Tactician");
    supported("Raise the Alarm");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hardened Tactician");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    assert!(act(&mut t, P0, h, "draw", &[]).is_err(), "no token");
    t.clear_answers();
    t.lands(P0, "Plains", 2);
    let r = t.hand(P0, "Raise the Alarm");
    t.cast(P0, r).go();
    t.resolve_all();
    let soldiers = subtype_ids(&t, P0, "Soldier");
    assert_eq!(soldiers.len(), 2);
    t.answer_choose(P0, &[Entity::Object(soldiers[0])]);
    act(&mut t, P0, h, "draw", &[]).unwrap();
    let c = entity_choices(&t, P0).pop().unwrap();
    assert!(!c.contains(&Entity::Object(bears)) && !c.contains(&Entity::Object(h)));
    assert!(c.contains(&Entity::Object(soldiers[1])));
    assert!(!t.on_battlefield(soldiers[0]) && t.on_battlefield(soldiers[1]));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn lore_weaver_makes_target_player_draw_two() {
    cr!("602.2", "115.1", "121.1");
    // "{5}{U}{U}: Target player draws two cards."
    supported("Lore Weaver");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Lore Weaver");
    t.lands(P0, "Island", 7);
    act(&mut t, P0, w, "draw", &[Entity::Player(P1)]).unwrap();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Player(P0)) && c.contains(&Entity::Player(P1)));
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (0, 2));
}

#[test]
fn graveyard_marshal_exiles_a_creature_card_for_a_tapped_zombie() {
    cr!("602.2", "118.3", "111.1", "111.4");
    // "{2}{B}, Exile a creature card from your graveyard: Create a tapped 2/2 black
    // Zombie creature token."
    supported("Graveyard Marshal");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Graveyard Marshal");
    t.lands(P0, "Swamp", 3);
    let land = t.graveyard(P0, "Forest");
    let theirs = t.graveyard(P1, "Grizzly Bears");
    assert!(act(&mut t, P0, m, "create", &[]).is_err(), "no creature card");
    t.clear_answers();
    let bears = t.graveyard(P0, "Grizzly Bears");
    act(&mut t, P0, m, "create", &[]).unwrap();
    assert_eq!(t.zone(t.g.current(bears)), Zone::Exile);
    assert_eq!(t.zone(land), Zone::Graveyard(P0));
    assert_eq!(t.zone(theirs), Zone::Graveyard(P1));
    t.resolve_all();
    let z = subtype_ids(&t, P0, "Zombie")
        .into_iter()
        .filter(|id| t.obj_now(*id).is_token())
        .collect::<Vec<_>>();
    assert_eq!(z.len(), 1);
    let o = t.obj_now(z[0]);
    assert!(o.tapped, "enters tapped");
    assert!(o.chars.is(CardType::Creature));
    assert!(o.chars.colors == ColorSet::single(Color::Black));
    assert_eq!(o.chars.name, "Zombie Token");
    assert_eq!(t.pt(z[0]), (2, 2));
}

#[test]
fn anurid_swarmsnapper_can_block_an_additional_creature() {
    cr!("602.2", "509.1a", "509.1b", "514.2");
    // "{1}{G}: This creature can block an additional creature this turn."
    supported("Anurid Swarmsnapper");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P1, "Anurid Swarmsnapper");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    let c = t.battlefield(P0, "Hill Giant");
    t.lands(P1, "Forest", 4);
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        &[
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
            (c, Entity::Player(P1)),
        ],
    );
    go_to(&mut t, Step::DeclareAttackers);
    assert!(legal_blocks(&t, P1, &[(s, a)]));
    assert!(!legal_blocks(&t, P1, &[(s, a), (s, b)]), "one block by default");
    act(&mut t, P1, s, "additional", &[]).unwrap();
    t.resolve_all();
    assert!(legal_blocks(&t, P1, &[(s, a), (s, b)]), "two now");
    assert!(!legal_blocks(&t, P1, &[(s, a), (s, b), (s, c)]), "not three");
    act(&mut t, P1, s, "additional", &[]).unwrap();
    t.resolve_all();
    assert!(legal_blocks(&t, P1, &[(s, a), (s, b), (s, c)]), "it stacks");
    // This turn only.
    let mut t2 = TestGame::new(2);
    let s = t2.battlefield(P1, "Anurid Swarmsnapper");
    let a = t2.battlefield(P0, "Grizzly Bears");
    let b = t2.battlefield(P0, "Llanowar Elves");
    t2.lands(P1, "Forest", 2);
    t2.set_step(P0, Step::PrecombatMain);
    act(&mut t2, P1, s, "additional", &[]).unwrap();
    t2.resolve_all();
    t2.advance_to(P1, Step::PrecombatMain);
    t2.advance_to(P0, Step::BeginningOfCombat);
    declare(&mut t2, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    go_to(&mut t2, Step::DeclareAttackers);
    assert!(!legal_blocks(&t2, P1, &[(s, a), (s, b)]), "the effect ended");
}

#[test]
fn elemental_masterpiece_discards_itself_for_a_treasure() {
    cr!("602.2", "118.3", "701.9a", "107.4e");
    // "{U/R}{U/R}, Discard this card: Create a Treasure token."
    supported("Elemental Masterpiece");
    let mut t = TestGame::new(2);
    let m = t.hand(P0, "Elemental Masterpiece");
    t.battlefield(P0, "Island");
    t.battlefield(P0, "Mountain");
    act(&mut t, P0, m, "treasure", &[]).unwrap();
    assert!(t.in_graveyard(P0, "Elemental Masterpiece"), "discarded as a cost");
    assert_eq!(untapped(&t, P0, "Island") + untapped(&t, P0, "Mountain"), 0);
    t.resolve_all();
    let tr = subtype_ids(&t, P0, "Treasure");
    assert_eq!(tr.len(), 1);
    assert!(t.obj_now(tr[0]).is_token());
    assert!(t.obj_now(tr[0]).chars.is(CardType::Artifact));
    assert_eq!(count_subtype(&t, P1, "Treasure"), 0);
    // Only from the hand: not from the graveyard.
    let gy = t.g.player(P0).graveyard[0];
    t.lands(P0, "Island", 2);
    assert!(act(&mut t, P0, gy, "treasure", &[]).is_err());
}

#[test]
fn cephalid_retainer_taps_a_creature_without_flying() {
    cr!("602.2", "115.1", "701.26a");
    // "{U}{U}: Tap target creature without flying."
    supported("Cephalid Retainer");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Cephalid Retainer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let drake = t.battlefield(P1, "Wind Drake");
    t.lands(P0, "Island", 2);
    act(&mut t, P0, r, "tap", &[Entity::Object(bears)]).unwrap();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(bears)) && c.contains(&Entity::Object(r)));
    assert!(!c.contains(&Entity::Object(drake)), "flying");
    assert!(!t.obj_now(r).tapped, "no {{T}} in the cost");
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert!(!t.obj_now(drake).tapped);
}

#[test]
fn deepwood_ghoul_pays_life_to_regenerate() {
    cr!("602.2", "119.4", "701.19a", "118.3");
    // "Pay 2 life: Regenerate this creature."
    supported("Deepwood Ghoul");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Deepwood Ghoul");
    act(&mut t, P0, g, "regenerate", &[]).unwrap();
    assert_eq!(t.life(P0), 18, "paid as a cost");
    t.resolve_all();
    assert!(!t.g.destroy(g, None), "the shield replaces destruction");
    t.settle();
    assert!(t.on_battlefield(g));
    assert!(t.obj_now(g).tapped);
    assert_eq!(t.obj_now(g).damage, 0);
    assert!(t.g.destroy(g, None), "the shield is used up");
    // An unused shield expires at end of turn.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Deepwood Ghoul");
    act(&mut t, P0, g, "regenerate", &[]).unwrap();
    t.resolve_all();
    next_turn(&mut t);
    assert!(t.g.destroy(g, None), "the shield ended with the turn");
    // You can't pay more life than you have.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Deepwood Ghoul");
    t.g.lose_life(P0, 19);
    assert!(act(&mut t, P0, g, "regenerate", &[]).is_err());
    assert_eq!(t.life(P0), 1);
}

#[test]
fn goblin_sledder_sacrifices_a_goblin_to_pump() {
    cr!("602.2", "118.3", "701.21a", "611.2a", "514.2");
    // "Sacrifice a Goblin: Target creature gets +1/+1 until end of turn."
    supported("Goblin Sledder");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Goblin Sledder");
    let gob = t.battlefield(P0, "Raging Goblin");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Raging Goblin");
    t.answer_choose(P0, &[Entity::Object(gob)]);
    act(&mut t, P0, s, "+1/+1", &[Entity::Object(bears)]).unwrap();
    let c = entity_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(s)) && c.contains(&Entity::Object(gob)));
    assert!(!c.contains(&Entity::Object(bears)), "not a Goblin");
    assert!(!c.contains(&Entity::Object(theirs)), "not yours");
    assert!(!t.on_battlefield(gob));
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
    // It can sacrifice itself.
    t.answer_choose(P0, &[Entity::Object(s)]);
    act(&mut t, P0, s, "+1/+1", &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Goblin Sledder"));
    assert_eq!(t.pt(bears), (4, 4));
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn moggcatcher_puts_a_goblin_permanent_card_onto_the_battlefield() {
    cr!("602.2", "701.23a", "701.23b", "701.24a");
    ruling!(
        "Moggcatcher",
        "you don’t have to find the card if you don’t want to"
    );
    // "{3}, {T}: Search your library for a Goblin permanent card, put it onto the
    // battlefield, then shuffle."
    supported("Moggcatcher");
    supported("Tarfire");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Moggcatcher");
    t.lands(P0, "Wastes", 3);
    let gob = t.library_top(P0, "Raging Goblin");
    let tarfire = t.library_top(P0, "Tarfire");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(gob)]);
    act(&mut t, P0, m, "search", &[]).unwrap();
    assert!(t.obj_now(m).tapped);
    t.resolve_all();
    let c = entity_choices(&t, P0).pop().unwrap();
    assert_eq!(c, vec![Entity::Object(gob)], "a kindred Goblin instant isn't a permanent card");
    assert!(!c.contains(&Entity::Object(tarfire)) && !c.contains(&Entity::Object(bears)));
    let v = t.named_on_battlefield("Raging Goblin");
    assert_eq!(v.len(), 1);
    assert_eq!(t.obj_now(v[0]).controller, P0);
    assert!(shuffled(&t, P0));
    // Finding nothing.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Moggcatcher");
    t.lands(P0, "Wastes", 3);
    let gob = t.library_top(P0, "Raging Goblin");
    t.answer_choose(P0, &[]);
    act(&mut t, P0, m, "search", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(gob), Zone::Library(P0));
    assert!(shuffled(&t, P0));
}

#[test]
fn revered_elder_prevents_the_next_one_damage_to_itself() {
    cr!("602.2", "615.1a", "615.7");
    // "{1}: Prevent the next 1 damage that would be dealt to this creature this turn."
    supported("Revered Elder");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Revered Elder");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let src = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    act(&mut t, P0, e, "prevent", &[]).unwrap();
    t.resolve_all();
    act(&mut t, P0, e, "prevent", &[]).unwrap();
    t.resolve_all();
    t.g.deal_damage(src, Entity::Object(bears), 1, false);
    t.g.deal_damage(src, Entity::Object(e), 1, false);
    t.settle();
    assert_eq!(t.obj_now(bears).damage, 1, "only this creature is shielded");
    assert_eq!(t.obj_now(e).damage, 0);
    t.g.deal_damage(src, Entity::Object(e), 2, false);
    t.settle();
    assert_eq!(t.obj_now(e).damage, 1, "one shield left, then used up");
    // An unused shield ends with the turn.
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Revered Elder");
    let src = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    act(&mut t, P0, e, "prevent", &[]).unwrap();
    t.resolve_all();
    next_turn(&mut t);
    t.g.deal_damage(src, Entity::Object(e), 1, false);
    t.settle();
    assert_eq!(t.obj_now(e).damage, 1);
}

#[test]
fn thunderscape_master_drains_target_player() {
    cr!("602.2", "115.1", "119.3");
    // "{B}{B}, {T}: Target player loses 2 life and you gain 2 life."
    supported("Thunderscape Master");
    let mut t = TestGame::new(3);
    let m = t.battlefield(P0, "Thunderscape Master");
    t.lands(P0, "Swamp", 2);
    act(&mut t, P0, m, "loses", &[Entity::Player(P2)]).unwrap();
    assert!(t.obj_now(m).tapped);
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Player(P0)), "any player");
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (22, 20, 18));
}

#[test]
fn drowner_of_secrets_taps_an_untapped_merfolk_to_mill() {
    cr!("602.2", "118.3", "701.17a", "302.6");
    // "Tap an untapped Merfolk you control: Target player mills a card."
    supported("Drowner of Secrets");
    let mut t = TestGame::new(2);
    let d = t.battlefield_sick(P0, "Drowner of Secrets");
    let fish = t.battlefield(P0, "Coral Merfolk");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Coral Merfolk");
    let lib = t.library_size(P1);
    t.answer_choose(P0, &[Entity::Object(d)]);
    act(&mut t, P0, d, "mills", &[Entity::Player(P1)]).unwrap();
    let c = entity_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(d)), "itself, even summoning sick");
    assert!(c.contains(&Entity::Object(fish)));
    assert!(!c.contains(&Entity::Object(bears)) && !c.contains(&Entity::Object(theirs)));
    assert!(t.obj_now(d).tapped);
    t.resolve_all();
    assert_eq!(t.library_size(P1), lib - 1);
    assert_eq!(t.graveyard_size(P1), 1);
    act(&mut t, P0, d, "mills", &[Entity::Player(P0)]).unwrap();
    assert!(t.obj_now(fish).tapped, "the only untapped Merfolk left");
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 1);
    assert!(
        act(&mut t, P0, d, "mills", &[Entity::Player(P1)]).is_err(),
        "no untapped Merfolk"
    );
}

#[test]
fn drowner_of_secrets_can_tap_a_kindred_merfolk() {
    cr!("602.2", "118.3", "308.2");
    ruling!(
        "Drowner of Secrets",
        "including noncreature kindred permanents such as Merrow Commerce"
    );
    supported("Drowner of Secrets");
    supported("Merrow Commerce");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Drowner of Secrets");
    let mc = t.battlefield(P0, "Merrow Commerce");
    t.g.tap(d);
    act(&mut t, P0, d, "mills", &[Entity::Player(P1)]).unwrap();
    assert!(t.obj_now(mc).tapped);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 1);
}

#[test]
fn phyrexian_debaser_sacrifices_to_shrink_target_creature() {
    cr!("602.2", "701.21a", "611.2a", "514.2", "704.5f");
    // "{T}, Sacrifice this creature: Target creature gets -2/-2 until end of turn."
    supported("Phyrexian Debaser");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Phyrexian Debaser");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    act(&mut t, P0, d, "-2/-2", &[Entity::Object(giant)]).unwrap();
    assert!(t.in_graveyard(P0, "Phyrexian Debaser"));
    t.resolve_all();
    assert_eq!(t.pt(giant), (1, 1));
    assert_eq!(t.pt(bears), (2, 2));
    next_turn(&mut t);
    assert_eq!(t.pt(giant), (3, 3));
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Phyrexian Debaser");
    t.battlefield(P1, "Grizzly Bears");
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    act(&mut t, P0, d, "-2/-2", &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn tap_to_create_an_artifact_token() {
    cr!("602.2", "111.1", "111.4");
    // Nuka-Cola Vending Machine: "{1}, {T}: Create a Food token."
    supported("Nuka-Cola Vending Machine");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Nuka-Cola Vending Machine");
    t.lands(P0, "Wastes", 1);
    act(&mut t, P0, m, "food", &[]).unwrap();
    assert!(t.obj_now(m).tapped);
    assert_eq!(untapped(&t, P0, "Wastes"), 0);
    t.resolve_all();
    let food = subtype_ids(&t, P0, "Food");
    assert_eq!(food.len(), 1);
    let o = t.obj_now(food[0]);
    assert!(o.is_token() && o.chars.is(CardType::Artifact));
    assert!(!o.chars.is(CardType::Creature));
    assert_eq!(o.chars.name, "Food Token");
    assert!(act(&mut t, P0, m, "food", &[]).is_err(), "it's tapped");
    // Nuisance Engine: "{2}, {T}: Create a 0/1 colorless Pest artifact creature token."
    supported("Nuisance Engine");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Nuisance Engine");
    t.lands(P0, "Wastes", 2);
    act(&mut t, P0, e, "pest", &[]).unwrap();
    assert!(t.obj_now(e).tapped);
    t.resolve_all();
    let pest = subtype_ids(&t, P0, "Pest");
    assert_eq!(pest.len(), 1);
    let o = t.obj_now(pest[0]);
    assert!(o.is_token());
    assert!(o.chars.is(CardType::Artifact) && o.chars.is(CardType::Creature));
    assert!(o.chars.colors.is_colorless());
    assert_eq!(o.chars.name, "Pest Token");
    assert!(o.chars.abilities.is_empty(), "a vanilla Pest");
    assert_eq!(t.pt(pest[0]), (0, 1));
    assert_eq!(count_subtype(&t, P1, "Pest"), 0);
}

#[test]
fn sandsower_taps_three_creatures_to_tap_target_creature() {
    cr!("602.2", "118.3", "701.26a", "302.6");
    ruling!(
        "Sandsower",
        "you can tap any three untapped creatures you control (including Sandsower itself)"
    );
    // "Tap three untapped creatures you control: Tap target creature."
    supported("Sandsower");
    let mut t = TestGame::new(2);
    let s = t.battlefield_sick(P0, "Sandsower");
    let a = t.battlefield_sick(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(s), Entity::Object(a), Entity::Object(b)]);
    act(&mut t, P0, s, "tap target", &[Entity::Object(theirs)]).unwrap();
    assert!(t.obj_now(s).tapped && t.obj_now(a).tapped && t.obj_now(b).tapped);
    t.resolve_all();
    assert!(t.obj_now(theirs).tapped);
    // Only two untapped creatures: can't pay.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sandsower");
    t.battlefield(P0, "Grizzly Bears");
    let tapped = t.battlefield(P0, "Llanowar Elves");
    t.g.tap(tapped);
    let theirs = t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Grizzly Bears");
    assert!(act(&mut t, P0, s, "tap target", &[Entity::Object(theirs)]).is_err());
    assert!(!t.obj_now(theirs).tapped);
}

#[test]
fn scalding_devil_pings_a_player_or_planeswalker() {
    cr!("602.2", "115.1", "120.3a", "120.3c");
    // "{2}{R}: This creature deals 1 damage to target player or planeswalker."
    supported("Scalding Devil");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Scalding Devil");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let garruk = t.battlefield(P1, "Garruk Wildspeaker");
    t.lands(P0, "Mountain", 6);
    act(&mut t, P0, d, "damage", &[Entity::Player(P1)]).unwrap();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Player(P0)) && c.contains(&Entity::Player(P1)));
    assert!(c.contains(&Entity::Object(garruk)));
    assert!(!c.contains(&Entity::Object(bears)) && !c.contains(&Entity::Object(d)));
    assert!(!t.obj_now(d).tapped);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 19));
    act(&mut t, P0, d, "damage", &[Entity::Object(garruk)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(garruk, "loyalty"), 2);
}

#[test]
fn seal_of_cleansing_sacrifices_to_destroy_an_artifact_or_enchantment() {
    cr!("602.2", "115.1", "701.8a", "701.21a", "601.2c", "601.2h");
    ruling!(
        "Seal of Cleansing",
        "Sacrificing Seal of Cleansing is the cost to activate its ability"
    );
    // "Sacrifice this enchantment: Destroy target artifact or enchantment."
    supported("Seal of Cleansing");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Seal of Cleansing");
    let ring = t.battlefield(P1, "Sol Ring");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let land = t.battlefield(P1, "Forest");
    act(&mut t, P0, s, "destroy", &[Entity::Object(anthem)]).unwrap();
    assert!(t.in_graveyard(P0, "Seal of Cleansing"), "sacrificed as the cost");
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(ring)) && c.contains(&Entity::Object(anthem)));
    assert!(!c.contains(&Entity::Object(bears)) && !c.contains(&Entity::Object(land)));
    // Targets are chosen before costs are paid (CR 601.2c, 601.2h): the Seal could target
    // itself.
    assert!(c.contains(&Entity::Object(s)));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Glorious Anthem"));
    assert!(t.on_battlefield(ring));
}

#[test]
fn skola_grovedancer_mills_its_controller() {
    cr!("602.2", "701.17a");
    // "{2}{G}: Mill a card." (and "Whenever a land card is put into your graveyard from
    // anywhere, you gain 1 life.")
    supported("Skola Grovedancer");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Skola Grovedancer");
    t.lands(P0, "Forest", 6);
    t.library_top(P0, "Grizzly Bears");
    let (l0, l1) = (t.library_size(P0), t.library_size(P1));
    act(&mut t, P0, s, "mill", &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.library_size(P0), l0 - 1);
    assert_eq!(t.library_size(P1), l1, "only you mill");
    assert_eq!(t.life(P0), 20, "not a land card");
    t.library_top(P0, "Forest");
    act(&mut t, P0, s, "mill", &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forest"));
    assert_eq!(t.life(P0), 21, "a land card: its trigger");
}

#[test]
fn slag_strider_sacrifices_an_artifact_to_ping() {
    cr!("602.2", "118.3", "701.21a", "120.3a");
    // "{1}, Sacrifice an artifact: This creature deals 1 damage to any target."
    supported("Slag Strider");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Slag Strider");
    t.lands(P0, "Wastes", 2);
    let theirs = t.battlefield(P1, "Sol Ring");
    assert!(
        act(&mut t, P0, s, "damage", &[Entity::Player(P1)]).is_err(),
        "no artifact to sacrifice"
    );
    t.clear_answers();
    let ring = t.battlefield(P0, "Sol Ring");
    act(&mut t, P0, s, "damage", &[Entity::Player(P1)]).unwrap();
    assert!(t.in_graveyard(P0, "Sol Ring"));
    assert!(t.on_battlefield(theirs));
    assert!(!t.on_battlefield(ring));
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(damage_sources(&t, Entity::Player(P1)), vec![(s, 1)]);
}

#[test]
fn specter_of_the_fens_drains_target_opponent() {
    cr!("602.2", "115.1", "119.3");
    // "{5}{B}: Target opponent loses 2 life and you gain 2 life."
    supported("Specter of the Fens");
    let mut t = TestGame::new(3);
    let s = t.battlefield(P0, "Specter of the Fens");
    t.lands(P0, "Swamp", 6);
    act(&mut t, P0, s, "loses", &[Entity::Player(P2)]).unwrap();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Player(P1)) && c.contains(&Entity::Player(P2)));
    assert!(!c.contains(&Entity::Player(P0)), "an opponent");
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (22, 20, 18));
}
