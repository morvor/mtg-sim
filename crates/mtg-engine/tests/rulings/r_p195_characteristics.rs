//! Rulings batch P195 — effects that change a permanent's characteristics indefinitely:
//! base power/toughness set by an exchange (Evra, Tree of Perdition, Tree of Redemption)
//! or an ability (Ascendant Spirit, Trench Gorger) with other modifications applied
//! after it (CR 613.4b–c); colors and types added (Rise from the Grave, Butch DeLoria)
//! and overwritten later; decayed granted by Ghouls' Night Out; Opal Guardian becoming a
//! creature; Urza's Saga and the Urza's lands.

use crate::r_p195_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Attaches the real Aura or Equipment `name` (put onto P0's battlefield) to `to`.
fn attach(t: &mut TestGame, name: &str, to: ObjectId) -> ObjectId {
    let a = t.battlefield(P0, name);
    assert!(t.g.attach(a, obj(to)));
    t.g.recompute();
    a
}

#[test]
fn exchanged_base_value_is_modified_by_auras_and_equipment() {
    cr!("701.12g", "613.4b", "613.4c");
    ruling!(
        "Evra, Halcyon Witness",
        "Any power-modifying effects, counters, Auras, or Equipment will apply after Evra's power is set to your former life total. For example, say Evra is enchanted with Dub (which makes it 6/6) and your life total is 7. After the exchange, Evra would be a 9/6 creature (its power became 7, which was then modified by Dub) and your life total would be 6."
    );
    ruling!(
        "Tree of Perdition",
        "Any toughness-modifying effects, counters, Auras, or Equipment will apply after its toughness is set to the player's former life total. For example, say Tree of Perdition is equipped with Cultist's Staff (which makes it 2/15) and the player's life total is 7. After the exchange, Tree of Perdition would be a 2/9 creature (its toughness became 7, which was then modified by Cultist's Staff) and the player's life total would be 15."
    );
    ruling!(
        "Tree of Redemption",
        "Any toughness-modifying effects, counters, Auras, or Equipment will apply after its toughness is set to your former life total. For example, say Tree of Redemption is enchanted with Lunarch Mantle (which makes it 2/15) and your life total is 7. After the exchange, Tree of Redemption would be a 2/9 creature (its toughness became 7, which was then modified by Lunarch Mantle) and your life total would be 15."
    );
    // (creature, attachment, whose life, P/T after, life after)
    for (name, att, who, pt, life) in [
        ("Evra, Halcyon Witness", "Dub", P0, (9, 6), 6),
        ("Tree of Perdition", "Cultist's Staff", P1, (2, 9), 15),
        ("Tree of Redemption", "Lunarch Mantle", P0, (2, 9), 15),
    ] {
        supported(name);
        supported(att);
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        attach(&mut t, att, c);
        set_life(&mut t, who, 7);
        let targets: Vec<Entity> = if who == P1 { vec![pl(P1)] } else { vec![] };
        act(&mut t, P0, c, 0, &targets);
        assert_eq!(t.pt(c), pt, "{name}");
        assert_eq!(t.life(who), life, "{name}");
    }
}

/// P0's Ascendant Spirit with fourteen snow lands.
fn spirit(t: &mut TestGame) -> ObjectId {
    t.lands(P0, "Snow-Covered Island", 14);
    t.battlefield(P0, "Ascendant Spirit")
}

#[test]
fn ascendant_spirit_base_pt_and_repeated_activations() {
    cr!("613.4b", "613.4c", "122.1b", "113.2c");
    ruling!(
        "Ascendant Spirit",
        "Any effects that modify Ascendant Spirit's power or toughness without setting it to a specific value will apply after its base power and toughness are set, regardless of the order in which those effects were created. The same is true of counters that modify its power and toughness."
    );
    ruling!(
        "Ascendant Spirit",
        "Any of Ascendant Spirit's abilities may be activated repeatedly. Multiple flying counters on the same permanent provide no additional benefit; however, each time the last ability resolves while Ascendant Spirit is an Angel will give Ascendant Spirit two more +1/+1 counters and cause it to gain another instance of the triggered ability. All of those instances trigger separately."
    );
    supported("Ascendant Spirit");
    let mut t = TestGame::new(2);
    let s = spirit(&mut t);
    // Giant Growth first, then the base P/T is set: 2/3 + 3/3.
    cast_at(&mut t, P0, "Giant Growth", &[obj(s)]);
    t.resolve_all();
    t.activate(P0, s, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(s), (5, 6));
    t.activate(P0, s, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(s), (7, 7));
    assert!(t.obj_now(s).has_keyword(KeywordKind::Flying));
    t.activate(P0, s, 2, &[]).unwrap();
    t.resolve_all();
    t.activate(P0, s, 2, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(s, "+1/+1"), 4);
    assert_eq!(t.pt(s), (11, 11));
    // Two instances of the draw trigger: two cards for one hit.
    let hand = t.hand_size(P0);
    t.attack(&[(s, pl(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 9);
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn ascendant_spirit_flying_after_a_block_changes_nothing() {
    cr!("509.1b", "506.4");
    ruling!(
        "Ascendant Spirit",
        "Activating the second ability after Ascendant Spirit has already been legally blocked by a creature without flying or reach won't change or undo that block."
    );
    let mut t = TestGame::new(2);
    let s = spirit(&mut t);
    t.activate(P0, s, 0, &[]).unwrap();
    t.resolve_all();
    let wall = t.battlefield(P1, "Wall of Omens");
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![(s, pl(P1))]));
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(wall, s)]));
    t.advance_to(P0, Step::DeclareBlockers);
    t.activate(P0, s, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(s).has_keyword(KeywordKind::Flying));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Wall of Omens"));
}

#[test]
fn trench_gorger_keeps_its_base_pt_when_a_card_leaves_exile() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Trench Gorger",
        "After the ability resolves, Trench Gorger’s power and toughness won’t change if one of the exiled cards leaves exile."
    );
    supported("Trench Gorger");
    let mut t = TestGame::new(2);
    let lands: Vec<ObjectId> = (0..3).map(|_| t.library_top(P0, "Forest")).collect();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &lands.iter().map(|l| obj(*l)).collect::<Vec<_>>());
    let g = t.enter(P0, "Trench Gorger");
    t.resolve_all();
    assert_eq!(t.pt(g), (3, 3));
    let exiled = t.g.find_in_zone(Zone::Exile, "Forest");
    assert_eq!(exiled.len(), 3);
    t.g.move_object(exiled[0], Zone::Hand(P0), events::MoveCause::Effect, None);
    t.g.recompute();
    assert_eq!(t.pt(g), (3, 3));
}

#[test]
fn rise_from_the_grave_color_and_type_can_be_overwritten() {
    cr!("613.1d", "613.1e", "613.7");
    ruling!(
        "Rise from the Grave",
        "A later effect that changes the affected creature’s colors will overwrite that part of Rise from the Grave’s effect; the creature will be only the new color. The same is true about an effect that changes the affected creature’s types or subtypes."
    );
    supported("Rise from the Grave");
    supported("Turn to Frog");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P1, "Grizzly Bears");
    cast_at(&mut t, P0, "Rise from the Grave", &[obj(bears)]);
    t.resolve_all();
    let bears = t.g.current(bears);
    assert!(t.obj_now(bears).chars.colors.contains(Color::Black));
    assert!(t.obj_now(bears).chars.has_subtype("Zombie"));
    cast_at(&mut t, P0, "Turn to Frog", &[obj(bears)]);
    t.resolve_all();
    let c = &t.obj_now(bears).chars;
    assert!(c.colors.contains(Color::Blue));
    assert!(!c.colors.contains(Color::Black));
    assert!(!c.colors.contains(Color::Green));
    assert!(c.has_subtype("Frog"));
    assert!(!c.has_subtype("Zombie"));
    assert!(!c.has_subtype("Bear"));
}

#[test]
fn butch_deloria_counts_a_rogue_snake_once() {
    cr!("205.3", "608.2h");
    ruling!(
        "Butch DeLoria, Tunnel Snake",
        "A permanent that's both a Rogue and a Snake is counted only once for Butch DeLoria's second ability."
    );
    supported("Butch DeLoria, Tunnel Snake");
    supported("Ambush Viper");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Butch DeLoria, Tunnel Snake");
    let v = t.battlefield(P0, "Ambush Viper");
    act(&mut t, P0, b, 0, &[obj(v)]);
    assert!(t.obj_now(v).chars.has_subtype("Rogue"));
    assert!(t.obj_now(v).chars.has_subtype("Snake"));
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![(b, pl(P1))]));
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(t.pt(b), (3, 3));
}

/// P0 casts Ghouls' Night Out with a Grizzly Bears in each graveyard; returns them.
fn ghouls(t: &mut TestGame) -> Vec<ObjectId> {
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P1, "Hill Giant");
    t.answer_choose(P0, &[obj(a)]);
    t.answer_choose(P0, &[obj(b)]);
    cast_at(t, P0, "Ghouls' Night Out", &[]);
    t.resolve_all();
    let v = vec![t.g.current(a), t.g.current(b)];
    for c in &v {
        assert!(t.on_battlefield(*c));
        assert_eq!(ctl(t, *c), P0);
        assert!(t.obj_now(*c).has_keyword(KeywordKind::Decayed));
    }
    v
}

#[test]
fn ghouls_night_out_decayed() {
    cr!("702.147a", "302.6", "508.1d", "603.7c");
    ruling!(
        "Ghouls' Night Out",
        "Decayed does not grant haste. Creatures with decayed that enter the battlefield during your turn may not attack until your next turn."
    );
    ruling!(
        "Ghouls' Night Out",
        "Decayed does not create any attacking requirements. You may choose not to attack with a creature that has decayed."
    );
    ruling!(
        "Ghouls' Night Out",
        "Decayed represents a static ability and a triggered ability. \"Decayed\" means \"This creature can't block\" and \"When this creature attacks, sacrifice it at end of combat.\""
    );
    ruling!(
        "Ghouls' Night Out",
        "Once a creature with decayed attacks, it will be sacrificed at end of combat, even if it no longer has decayed at that time."
    );
    supported("Ghouls' Night Out");
    // No haste: they can't attack this turn.
    let mut t = TestGame::new(2);
    let v = ghouls(&mut t);
    let from = t.asked().len();
    t.advance_to(P0, Step::DeclareAttackers);
    let options: Vec<ObjectId> = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::DeclareAttackers { options } => Some(options.iter().map(|o| o.0).collect()),
            _ => None,
        })
        .unwrap_or_default();
    assert!(!options.contains(&v[0]) && !options.contains(&v[1]));
    // Next turn: can attack, but needn't (no requirement): declaring none is legal.
    t.advance_to(P1, Step::End);
    let from = t.asked().len();
    t.advance_to(P0, Step::DeclareAttackers);
    let options: Vec<ObjectId> = t.asked()[from..]
        .iter()
        .find_map(|(p, d)| match d {
            Decision::DeclareAttackers { options } if *p == P0 => {
                Some(options.iter().map(|o| o.0).collect())
            }
            _ => None,
        })
        .unwrap();
    assert!(options.contains(&v[0]));
    assert!(t.g.combat.as_ref().is_none_or(|c| c.attackers.is_empty()));
    // Can't block.
    t.advance_to(P1, Step::BeginningOfCombat);
    let attacker = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.answer(P1, DecisionKind::Attackers, Answer::Attackers(vec![(attacker, pl(P0))]));
    t.advance_to(P1, Step::DeclareBlockers);
    let blockers: Vec<ObjectId> = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::DeclareBlockers { options } => Some(options.iter().map(|o| o.0).collect()),
            _ => None,
        })
        .unwrap_or_default();
    assert!(!blockers.contains(&v[0]) && !blockers.contains(&v[1]));
    // Attacks, loses decayed (Turn to Frog), is still sacrificed at end of combat.
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![(v[1], pl(P1))]));
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    cast_at(&mut t, P0, "Turn to Frog", &[obj(v[1])]);
    t.resolve_all();
    assert!(!t.obj_now(v[1]).has_keyword(KeywordKind::Decayed));
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.on_battlefield(v[0]));
}

#[test]
fn opal_guardian_becomes_a_creature_on_a_face_down_spell() {
    cr!("708.4", "603.4", "205.1b");
    ruling!(
        "Opal Guardian",
        "A face-down creature spell will cause Opal Guardian's ability to trigger (if it's an enchantment)."
    );
    ruling!(
        "Opal Guardian",
        "After the triggered ability resolves, Opal Guardian is no longer an enchantment."
    );
    supported("Opal Guardian");
    supported("Exalted Angel");
    let mut t = TestGame::new(2);
    let og = t.battlefield(P0, "Opal Guardian");
    t.set_step(P1, Step::PrecombatMain);
    let angel = t.hand(P1, "Exalted Angel");
    lots(&mut t, P1);
    t.cast(P1, angel)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    t.settle();
    assert!(top_is_trigger_of(&t, og));
    t.resolve();
    let o = t.obj_now(og);
    assert!(o.is(CardType::Creature));
    assert!(!o.is(CardType::Enchantment));
    assert_eq!((o.power(), o.toughness()), (3, 4));
    assert!(o.chars.has_subtype("Gargoyle"));
    assert!(o.has_keyword(KeywordKind::Flying));
    // No longer an enchantment: another creature spell doesn't trigger it.
    t.resolve_all();
    let bears = t.hand(P1, "Grizzly Bears");
    lots(&mut t, P1);
    t.cast(P1, bears).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn urzas_saga_is_not_an_urzas_mine_or_power_plant() {
    cr!("205.3i", "106.1");
    ruling!(
        "Urza's Saga",
        "Although Urza's Saga has the Urza's land type, it doesn't interact with Urza's Tower, Urza's Mine, or Urza's Power Plant."
    );
    supported("Urza's Saga");
    supported("Urza's Tower");
    let mut t = TestGame::new(2);
    let tower = t.battlefield(P0, "Urza's Tower");
    t.battlefield(P0, "Urza's Mine");
    let saga = t.battlefield(P0, "Urza's Saga");
    assert!(t.obj_now(saga).chars.has_subtype("Urza's"));
    t.activate(P0, tower, 0, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    // With a real Power Plant: three.
    t.battlefield(P0, "Urza's Power Plant");
    t.g.untap(tower);
    t.activate(P0, tower, 0, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.total(), 4);
    let _ = ManaType::C;
}
