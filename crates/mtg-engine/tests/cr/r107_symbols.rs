//! CR 107.4–107.18: mana symbols and other symbols.

use super::r105_util::*;
use mtg_engine::ability::*;
use mtg_engine::mana::{find_payment, Mana, ManaCost, ManaSymbol, ManaType, SpendContext};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

use ManaType::*;

fn pool(types: &[ManaType]) -> Vec<Mana> {
    types.iter().map(|t| Mana::new(*t)).collect()
}

fn payable(cost: &str, types: &[ManaType], life: u32) -> Option<u32> {
    find_payment(
        &pool(types),
        &ManaCost::parse(cost).unwrap(),
        &SpendContext::default(),
        life,
    )
    .map(|p| p.life)
}

#[test]
fn all_mana_symbols_are_recognized() {
    cr!("107.4");
    let symbols = [
        "{W}", "{U}", "{B}", "{R}", "{G}", "{C}", "{0}", "{1}", "{2}", "{3}", "{4}", "{X}",
        "{W/U}", "{W/B}", "{U/B}", "{U/R}", "{B/R}", "{B/G}", "{R/G}", "{R/W}", "{G/W}", "{G/U}",
        "{2/W}", "{2/U}", "{2/B}", "{2/R}", "{2/G}", "{C/W}", "{C/U}", "{C/B}", "{C/R}", "{C/G}",
        "{W/P}", "{U/P}", "{B/P}", "{R/P}", "{G/P}", "{W/U/P}", "{W/B/P}", "{U/B/P}", "{U/R/P}",
        "{B/R/P}", "{B/G/P}", "{R/G/P}", "{R/W/P}", "{G/W/P}", "{G/U/P}", "{S}",
    ];
    for s in symbols {
        let c = ManaCost::parse(s).unwrap_or_else(|| panic!("{s} not parsed"));
        assert_eq!(c.symbols.len(), 1, "{s}");
        assert_eq!(c.to_string(), s);
    }
    // Snow is neither a color nor a type of mana (CR 107.4h).
    assert_eq!(ManaCost::parse("{S}").unwrap().colors(), ColorSet::NONE);
}

#[test]
fn colored_mana_symbols_are_paid_only_with_their_color() {
    cr!("107.4a");
    for (c, t) in [("{W}", W), ("{U}", U), ("{B}", B), ("{R}", R), ("{G}", G)] {
        let col = ManaCost::parse(c).unwrap().colors();
        assert!(col.is_monocolored());
        assert_eq!(ManaType::from_color(col.iter().next().unwrap()), t);
        assert!(payable(c, &[t], 0).is_some());
        for other in ManaType::ALL.iter().filter(|x| **x != t) {
            assert!(payable(c, &[*other], 0).is_none(), "{c} with {other:?}");
        }
    }
    // A real spell: Grizzly Bears ({1}{G}) can't be cast with red mana only.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
}

#[test]
fn generic_mana_is_paid_with_any_type() {
    cr!("107.4b");
    for t in ManaType::ALL {
        assert!(payable("{1}", &[t], 0).is_some());
    }
    // {X} represents generic mana too: Blaze with X = 2 is paid with {C}{U}{R}.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    let blaze = t.hand(P0, "Blaze");
    t.cast(P0, blaze).x(2).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn colorless_symbol_is_paid_only_with_colorless_mana() {
    cr!("107.4c");
    assert!(payable("{C}", &[C], 0).is_some());
    for t in [W, U, B, R, G] {
        assert!(payable("{C}", &[t], 0).is_none());
    }
    // Thought-Knot Seer ({3}{C}) can't be cast with four colored mana.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    let seer = t.hand(P0, "Thought-Knot Seer");
    assert!(t.cast(P0, seer).try_go().is_err());
    let seer = t.g.current(seer);
    t.lands(P0, "Wastes", 1);
    t.cast(P0, seer).go();
}

#[test]
fn zero_symbol_costs_nothing() {
    cr!("107.4d");
    let mut t = TestGame::new(2);
    // Ornithopter ({0}) is cast with no mana and no other resources.
    let thopter = t.hand(P0, "Ornithopter");
    t.cast(P0, thopter).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 1);
    assert_eq!(t.life(P0), 20);
    // An ability with a {0} cost can be activated with nothing.
    let def = card_from_text(
        "Zero Engine",
        "{1}",
        "Artifact",
        None,
        "{0}: You gain 1 life.",
    );
    let z = put(&mut t, P0, def);
    t.activate(P0, z, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn hybrid_symbols() {
    cr!("107.4e");
    // Example: {G/W}{G/W} can be paid by spending {G}{G}, {G}{W}, or {W}{W}.
    for p in [[G, G], [G, W], [W, W]] {
        assert!(payable("{G/W}{G/W}", &p, 0).is_some());
    }
    assert!(payable("{G/W}{G/W}", &[G, U], 0).is_none());
    // {2/B}: one black mana or two mana of any type.
    assert!(payable("{2/B}", &[B], 0).is_some());
    assert!(payable("{2/B}", &[R, C], 0).is_some());
    assert!(payable("{2/B}", &[R], 0).is_none());
    // A hybrid symbol is all of its component colors, even with a colorless component.
    assert_eq!(ManaCost::parse("{G/W}").unwrap().colors(), cs("GW"));
    assert_eq!(ManaCost::parse("{2/B}").unwrap().colors(), cs("B"));
    assert_eq!(ManaCost::parse("{C/U}").unwrap().colors(), cs("U"));
    // Boros Recruit ({R/W}) cast with {W}.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let r = t.hand(P0, "Boros Recruit");
    t.cast(P0, r).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Boros Recruit").len(), 1);
}

#[test]
fn phyrexian_symbols() {
    cr!("107.4f");
    // Example: {W/P}{W/P} can be paid by spending {W}{W}, by spending {W} and paying 2
    // life, or by paying 4 life.
    assert_eq!(payable("{W/P}{W/P}", &[W, W], 20), Some(0));
    assert_eq!(payable("{W/P}{W/P}", &[W], 20), Some(2));
    assert_eq!(payable("{W/P}{W/P}", &[], 20), Some(4));
    // Hybrid Phyrexian: either color or 2 life; both colors.
    assert_eq!(payable("{W/U/P}", &[U], 20), Some(0));
    assert_eq!(payable("{W/U/P}", &[], 20), Some(2));
    assert_eq!(ManaCost::parse("{W/U/P}").unwrap().colors(), cs("WU"));
    assert_eq!(ManaCost::parse("{B/P}").unwrap().colors(), cs("B"));
    // Dismember ({1}{B/P}{B/P}) cast with one Swamp and a Mountain: {1} and one {B/P}
    // with mana, the other {B/P} with 2 life.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    let d = t.hand(P0, "Dismember");
    t.cast(P0, d).target(bears).go();
    assert_eq!(t.life(P0), 18);
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn h_symbol_means_any_phyrexian_mana_symbol() {
    cr!("107.4g");
    let mut t = TestGame::new(2);
    // Rage Extractor: "Whenever you cast a spell with {H} in its mana cost, this artifact
    // deals damage equal to that spell's mana value to any target."
    t.battlefield(P0, "Rage Extractor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Mutagenic Growth"); // {G/P}
    t.answer(
        P0,
        DecisionKind::Targets,
        Answer::Entities(vec![Entity::Object(bears)]),
    );
    t.answer(
        P0,
        DecisionKind::Targets,
        Answer::Entities(vec![Entity::Player(P1)]),
    );
    t.cast(P0, growth).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // A spell without Phyrexian symbols doesn't trigger it.
    t.lands(P0, "Forest", 1);
    let giant = t.hand(P0, "Giant Growth");
    t.cast(P0, giant).target(bears).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert!(ManaSymbol::parse("{G/P}").unwrap().is_phyrexian());
    assert!(ManaSymbol::parse("{W/U/P}").unwrap().is_phyrexian());
    assert!(!ManaSymbol::parse("{G}").unwrap().is_phyrexian());
}

#[test]
fn snow_symbol_is_paid_with_mana_from_a_snow_source() {
    cr!("107.4h");
    // Icehide Golem ({S}) can't be cast with mana from a Forest...
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let golem = t.hand(P0, "Icehide Golem");
    assert!(t.cast(P0, golem).try_go().is_err());
    // ...but can with mana of any type from a snow source.
    let golem = t.g.current(golem);
    let snow = t.battlefield(P0, "Snow-Covered Forest");
    t.activate(P0, snow, 0, &[]).unwrap();
    assert!(t.g.player(P0).mana_pool.mana[0].snow);
    t.cast(P0, golem).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Icehide Golem").len(), 1);
    // Generic cost reductions don't reduce {S}.
    let mut c = ManaCost::parse("{1}{S}").unwrap();
    c.reduce_generic(2);
    assert_eq!(c.to_string(), "{S}");
}

#[test]
fn tap_symbol_in_a_cost() {
    cr!("107.5");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.activate(P0, elves, 0, &[]).unwrap();
    assert!(t.obj_now(elves).tapped);
    // Already tapped: can't be tapped again to pay the cost.
    assert!(t.activate(P0, elves, 0, &[]).is_err());
    // A creature that hasn't been under its controller's control continuously since
    // their most recent turn began can't use {T} abilities.
    let sick = t.battlefield_sick(P0, "Llanowar Elves");
    assert!(t.activate(P0, sick, 0, &[]).is_err());
    // Haste lets it (CR 302.6); a noncreature permanent isn't affected.
    let hub = t.battlefield(P0, "Sol Ring");
    t.g.objects[hub.0 as usize].summoning_sick = true;
    t.activate(P0, hub, 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P0, C), 2);
}

#[test]
fn untap_symbol_in_a_cost() {
    cr!("107.6");
    let mut t = TestGame::new(2);
    // Patrol Signaler: "{1}{W}, {Q}: Create a 1/1 white Kithkin Soldier creature token."
    let sig = t.battlefield(P0, "Patrol Signaler");
    t.lands(P0, "Plains", 4);
    // Untapped: {Q} can't be paid.
    assert!(t.activate(P0, sig, 0, &[]).is_err());
    let cur = t.g.current(sig);
    t.g.objects[cur.0 as usize].tapped = true;
    t.activate(P0, sig, 0, &[]).unwrap();
    assert!(!t.obj_now(sig).tapped);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Kithkin Soldier Token").len(), 1);
    // A summoning-sick creature can't use {Q} abilities.
    let sick = t.battlefield_sick(P0, "Patrol Signaler");
    t.g.objects[sick.0 as usize].tapped = true;
    assert!(t.activate(P0, sick, 0, &[]).is_err());
}

#[test]
fn loyalty_symbols() {
    cr!("107.7");
    let mut t = TestGame::new(2);
    // Jace Beleren (3 loyalty): +2 puts two loyalty counters on it.
    let jace = t.battlefield(P0, "Jace Beleren");
    assert_eq!(t.counters(jace, "loyalty"), 3);
    t.activate(P0, jace, 0, &[]).unwrap();
    assert_eq!(t.counters(jace, "loyalty"), 5);
    t.resolve();
    // −1 removes one (next turn: once per turn).
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace Beleren");
    t.activate(P0, jace, 1, &[Entity::Player(P0)]).unwrap();
    assert_eq!(t.counters(jace, "loyalty"), 2);
    t.resolve();
    // [0] puts zero loyalty counters on it (Gideon, Ally of Zendikar: "0: Create a 2/2
    // white Knight Ally creature token.").
    let gideon = t.battlefield(P0, "Gideon, Ally of Zendikar");
    t.activate(P0, gideon, 0, &[]).unwrap();
    assert_eq!(t.counters(gideon, "loyalty"), 4);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Knight Ally Token").len(), 1);
    // [−X]: X loyalty counters are removed (Chandra, Flamecaller: "−X: Chandra deals X
    // damage to each creature.").
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra, Flamecaller");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, chandra, 0, &[]).unwrap();
    assert_eq!(t.counters(chandra, "loyalty"), 2);
    t.resolve();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn level_symbols() {
    cr!("107.8", "107.8a", "107.8b");
    let mut t = TestGame::new(2);
    // Student of Warfare: LEVEL 2-6 3/3 first strike; LEVEL 7+ 4/4 double strike.
    let s = t.battlefield(P0, "Student of Warfare");
    assert_eq!(t.pt(s), (1, 1));
    t.g.objects[s.0 as usize].counters.insert("level".into(), 1);
    t.g.recompute();
    assert_eq!(t.pt(s), (1, 1));
    assert!(!t
        .obj_now(s)
        .has_keyword(mtg_engine::keywords::KeywordKind::FirstStrike));
    t.g.objects[s.0 as usize].counters.insert("level".into(), 2);
    t.g.recompute();
    assert_eq!(t.pt(s), (3, 3));
    assert!(t
        .obj_now(s)
        .has_keyword(mtg_engine::keywords::KeywordKind::FirstStrike));
    t.g.objects[s.0 as usize].counters.insert("level".into(), 6);
    t.g.recompute();
    assert_eq!(t.pt(s), (3, 3));
    t.g.objects[s.0 as usize].counters.insert("level".into(), 7);
    t.g.recompute();
    assert_eq!(t.pt(s), (4, 4));
    assert!(t
        .obj_now(s)
        .has_keyword(mtg_engine::keywords::KeywordKind::DoubleStrike));
    assert!(!t
        .obj_now(s)
        .has_keyword(mtg_engine::keywords::KeywordKind::FirstStrike));
    t.g.objects[s.0 as usize]
        .counters
        .insert("level".into(), 12);
    t.g.recompute();
    assert_eq!(t.pt(s), (4, 4));
}

#[test]
fn color_indicator_defines_color() {
    cr!("107.13");
    let mut t = TestGame::new(2);
    let v = t.hand(P0, "Ancestral Vision");
    assert_eq!(t.obj_now(v).chars.color_indicator, Some(cs("U")));
    assert_eq!(colors(&t, v), cs("U"));
}

#[test]
fn energy_symbol() {
    cr!("107.14");
    let mut t = TestGame::new(2);
    // Aether Hub: "When this land enters, you get {E}." and "{T}, Pay {E}: Add one mana
    // of any color."
    let hub = t.enter(P0, "Aether Hub");
    t.resolve_all();
    assert_eq!(t.g.player(P0).counter("energy"), 1);
    t.g.objects[hub.0 as usize].summoning_sick = false;
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.activate(P0, hub, 1, &[]).unwrap();
    assert_eq!(t.g.player(P0).counter("energy"), 0);
    assert_eq!(pool_count(&t, P0, R), 1);
    // With no energy, {E} can't be paid.
    t.g.objects[hub.0 as usize].tapped = false;
    assert!(t.activate(P0, hub, 1, &[]).is_err());
}

#[test]
fn chapter_symbols() {
    cr!("107.15", "107.15a", "107.15b");
    let mut t = TestGame::new(2);
    // History of Benalia: "I, II — Create a 2/2 white Knight creature token with
    // vigilance. III — Knights you control get +2/+1 until end of turn."
    let saga = t.enter(P0, "History of Benalia");
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(saga, "lore"), 1);
    assert_eq!(t.named_on_battlefield("Knight Token").len(), 1);
    // Adding a lore counter when there's already one: the count goes from 1 to 2 — chapter
    // II triggers (the combined "I, II" ability works like two chapter abilities), chapter
    // I doesn't trigger again.
    t.g.add_counters(Entity::Object(saga), "lore", 1, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Knight Token").len(), 2);
    // A second Saga that gets two lore counters at once (0 → 2 in one event) triggers
    // both chapters I and II.
    let mut t = TestGame::new(2);
    let s2 = t.battlefield(P0, "History of Benalia");
    t.g.objects[s2.0 as usize].counters.clear();
    t.g.add_counters(Entity::Object(s2), "lore", 2, None);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Knight Token").len(), 2);
}

#[test]
fn class_level_bars() {
    cr!("107.16", "107.16a");
    let mut t = TestGame::new(2);
    // Druid Class: "{2}{G}: Level 2 — You may play an additional land on each of your
    // turns." / "{4}{G}: Level 3 — ..."
    let class = t.battlefield(P0, "Druid Class");
    assert_eq!(t.g.player(P0).land_plays, 1);
    t.lands(P0, "Forest", 5);
    // It's level 1: its level 3 bar requires level 2.
    let level2 = activated_uid(&t, class, 0);
    // Only as a sorcery: not during combat.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(t.g.activate_ability(P0, class, level2).is_err());
    t.set_step(P0, Step::PrecombatMain);
    t.g.activate_ability(P0, class, level2).unwrap();
    t.resolve();
    assert_eq!(t.obj_now(class).class_level, 2);
    t.g.recompute();
    assert_eq!(t.g.player(P0).land_plays, 2);
    // It can't gain level 2 again (it's not level 1).
    assert!(t.g.activate_ability(P0, class, level2).is_err());
}

#[test]
fn ticket_symbol() {
    cr!("107.17", "107.17a");
    let mut t = TestGame::new(2);
    // Prize Wall: "{U}, {T}: You get {TK}."
    let wall = t.battlefield(P0, "Prize Wall");
    t.lands(P0, "Island", 1);
    t.activate(P0, wall, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.g.player(P0).counter("ticket"), 1);
    // A ticket cost "{TK}{TK}" removes that many ticket counters from the player.
    let def = card_from_text(
        "Ticket Booth",
        "{1}",
        "Artifact",
        None,
        "{TK}{TK}: You gain 3 life.",
    );
    let booth = put(&mut t, P0, def);
    assert!(t.activate(P0, booth, 0, &[]).is_err());
    t.g.players[0].counters.insert("ticket".into(), 3);
    t.activate(P0, booth, 0, &[]).unwrap();
    assert_eq!(t.g.player(P0).counter("ticket"), 1);
    t.resolve();
    assert_eq!(t.life(P0), 23);
}

#[test]
fn pawprint_symbols_indicate_modes() {
    cr!("107.18");
    let text =
        "Choose up to five {P} worth of modes. You may choose the same mode more than once.\n\
                {P} — You gain 1 life.\n\
                {P}{P} — Draw a card.\n\
                {P}{P}{P} — Target player loses 3 life.";
    let def = card_from_text("Season Test", "{3}{W}", "Sorcery", None, text);
    // The pawprints aren't a cost, mana, or counters.
    assert_eq!(
        def.faces[0].chars.mana_cost.as_ref().unwrap().mana_value(),
        4
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    let s = put_in_hand(&mut t, P0, def.clone());
    // {P} + {P} + {P}{P} = four pawprints; the same mode twice.
    t.cast(P0, s).modes(&[0, 0, 1]).go();
    t.resolve();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.hand_size(P0), 1);
    // {P}{P}{P} + {P}{P} + {P} = six pawprints: too many.
    t.lands(P0, "Plains", 4);
    let s = put_in_hand(&mut t, P0, def);
    let id = t.cast(P0, s).modes(&[2, 1, 0]).target(P1).go();
    let chosen = t.g.obj(id).stack.as_ref().unwrap().chosen.clone();
    let pawprints: usize = chosen.iter().map(|c| [1, 2, 3][c.mode.unwrap()]).sum();
    assert!(pawprints <= 5, "{pawprints}");
    assert!(t.g.player(P0).counters.is_empty());
    let _ = Zone::Battlefield;
}
