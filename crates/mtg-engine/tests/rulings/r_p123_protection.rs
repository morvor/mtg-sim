//! Rulings batch P123 — protective spells and abilities and related rulings of the same
//! cards: phasing out makes targets illegal (CR 702.26b, 608.2b), "can't be countered"
//! and the rest of a counterspell (CR 101.2, 608.2c), "your life total can't change" and
//! damage (CR 119.10, 120.3), excess damage (CR 120.4a), effects that add abilities lock
//! in the affected set (CR 611.2c), protection from colors (CR 702.16), land creatures
//! (CR 205.1b), "controlled continuously since your turn began" (CR 302.6), creature
//! types (CR 205.3), and Mana Maze's color as a spell was cast (CR 601.2).

use crate::r_p123_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{CardType, Color, Supertype};
use mtg_engine::*;

fn choose_color(t: &mut TestGame, p: PlayerId, c: Color) {
    let i = Color::ALL.iter().position(|x| *x == c).unwrap();
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

#[test]
fn mana_maze_uses_the_color_the_spell_had_as_it_was_cast() {
    cr!("601.3", "601.2");
    ruling!(
        "Mana Maze",
        "If a spell’s color was changed while on the stack, use the color at the time it was when you started announcement"
    );
    supported("Mana Maze");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Mana Maze");
    cast_card(&mut t, P0, "Lightning Bolt");
    // Painter's Servant enters naming blue: the Bolt on the stack is blue too now.
    choose_color(&mut t, P0, Color::Blue);
    t.enter(P0, "Painter's Servant");
    let bolt = t.g.stack[0];
    assert!(t.g.obj(bolt).chars.colors.contains(Color::Blue));
    // A blue spell can still be cast: the Bolt was only red as it was cast. A red one
    // can't.
    let shock = t.hand(P0, "Shock");
    t.lands(P0, "Mountain", 1);
    assert!(!can_cast(&mut t, P0, shock, CastMethod::Normal), "red");
    let opt = t.hand(P0, "Opt");
    t.lands(P0, "Island", 1);
    assert!(can_cast(&mut t, P0, opt, CastMethod::Normal), "blue");
}

/// P1 casts Electrolyze dividing 2 damage among `targets` (one each, or 2 to one); then P0
/// casts Teferi's Protection in response. Returns P1's hand size before it resolved.
fn electrolyze_then_protection(t: &mut TestGame, targets: &[Entity]) -> usize {
    supported("Teferi's Protection");
    t.set_step(P1, Step::PrecombatMain);
    let amounts: Vec<i64> = if targets.len() == 2 { vec![1, 1] } else { vec![2] };
    t.answer_targets(P1, targets);
    t.answer(P1, DecisionKind::Divide, Answer::Numbers(amounts));
    cast_card(t, P1, "Electrolyze");
    cast_card(t, P0, "Teferi's Protection");
    t.resolve();
    let hand = t.hand_size(P1);
    t.resolve_all();
    hand
}

#[test]
fn phased_out_targets_are_illegal() {
    cr!("702.26b", "608.2b");
    ruling!(
        "Teferi's Protection",
        "A permanent phasing out causes a spell or ability on the stack to have an illegal target if it targets that permanent. As a spell or ability tries to resolve, if all its targets are illegal, that spell or ability doesn't resolve and none of its effects happen, including effects unrelated to the target."
    );
    // All targets illegal: no damage, no card drawn.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hand = electrolyze_then_protection(&mut t, &[obj(bears)]);
    assert!(t.obj_now(bears).phased_out);
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.hand_size(P1), hand, "no card drawn");
    // One target still legal: it's dealt its damage and P1 draws.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let hand = electrolyze_then_protection(&mut t, &[obj(bears), obj(wurm)]);
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.obj_now(wurm).damage, 1);
    assert_eq!(t.hand_size(P1), hand + 1);
}

#[test]
fn perch_protection_phases_out_only_if_the_gift_was_promised() {
    cr!("702.26b", "608.2b", "702.174a");
    ruling!(
        "Perch Protection",
        "A permanent phasing out causes a spell or ability on the stack to have an illegal target if it targets that permanent. As a spell or ability tries to resolve, if all its targets are illegal, that spell or ability doesn't resolve and none of its effects happen, including effects unrelated to the target."
    );
    supported("Perch Protection");
    for promised in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.set_step(P1, Step::PrecombatMain);
        t.answer_targets(P1, &[obj(bears)]);
        t.answer(P1, DecisionKind::Divide, Answer::Numbers(vec![2]));
        cast_card(&mut t, P1, "Electrolyze");
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(promised));
        cast_card(&mut t, P0, "Perch Protection");
        t.resolve();
        let hand = t.hand_size(P1);
        t.resolve_all();
        if promised {
            // The Bears (and the new Birds) phased out: Electrolyze's only target is
            // illegal, so it does nothing at all, not even the draw.
            assert!(t.obj_now(bears).phased_out);
            assert_eq!(t.obj_now(bears).damage, 0);
            assert_eq!(t.hand_size(P1), hand, "no card drawn");
        } else {
            // No gift: nothing phases out; the Bears are dealt 2 damage and die.
            assert!(!t.g.is_live(bears), "the Bears died");
            assert_eq!(t.hand_size(P1), hand + 1);
        }
    }
}

#[test]
fn cant_be_countered_still_gets_the_counterspells_other_effects() {
    cr!("101.2", "608.2c", "701.6b");
    ruling!(
        "Hexing Squelcher",
        "A spell or ability that counters spells can still target any spell that can't be countered. When that spell or ability resolves, the spell that can't be countered won't be countered, but any additional effects of the countering spell or ability will still happen."
    );
    ruling!(
        "Veil of Summer",
        "A spell or ability that counters spells can still target your spells after Veil of Summer resolves. When that spell or ability resolves, your spell won't be countered, but any additional effects of the countering spell or ability will still happen."
    );
    supported("Hexing Squelcher");
    supported("Veil of Summer");
    supported("Exclude");
    for veil in [false, true] {
        let mut t = TestGame::new(2);
        if veil {
            cast_resolve_at(&mut t, P0, "Veil of Summer", &[]);
        } else {
            t.battlefield(P0, "Hexing Squelcher");
        }
        let bears = cast_card(&mut t, P0, "Grizzly Bears");
        // P1's Exclude ("Counter target creature spell. Draw a card.") targets it.
        let hand = t.hand_size(P1);
        cast_resolve_at(&mut t, P1, "Exclude", &[obj(bears)]);
        assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1, "veil {veil}");
        assert_eq!(t.hand_size(P1), hand + 1, "P1 drew a card");
    }
}

#[test]
fn flare_of_fortitude_damage_still_dealt() {
    cr!("119.10", "120.3a");
    ruling!(
        "Flare of Fortitude",
        "Abilities that trigger whenever damage is dealt to you will still trigger because that damage is still dealt, even though your life total doesn't change as a result."
    );
    supported("Flare of Fortitude");
    let mut t = TestGame::new(2);
    let watcher = custom_card(
        "Pain Diary",
        "Enchantment",
        "{1}",
        None,
        "Whenever a source deals damage to you, draw a card.",
    );
    t.custom(P0, watcher, Zone::Battlefield);
    cast_resolve_at(&mut t, P0, "Flare of Fortitude", &[]);
    let hand = t.hand_size(P0);
    t.set_step(P1, Step::PrecombatMain);
    cast_resolve_at(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.hand_size(P0), hand + 1, "the damage trigger triggered");
}

#[test]
fn spare_from_evil_non_human_creatures() {
    cr!("205.3", "702.16b");
    ruling!(
        "Spare from Evil",
        "A \"non-Human creature\" is any creature that doesn't have the creature type Human. A creature that is a Human in addition to other creature types is not a non-Human creature."
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let soldier = t.battlefield(P1, "Elite Vanguard");
    cast_resolve_at(&mut t, P0, "Spare from Evil", &[]);
    t.g.deal_damage(bears, obj(mine), 2, false);
    t.settle();
    assert_eq!(t.obj_now(mine).damage, 0, "a Bear: prevented");
    t.g.deal_damage(soldier, obj(mine), 2, false);
    t.settle();
    assert_eq!(t.obj_now(mine).damage, 2, "a Human Soldier: dealt");
}

#[test]
fn dragons_disciple_needs_a_dragon_card() {
    cr!("205.3", "614.12");
    ruling!(
        "Dragon's Disciple",
        "A Dragon card is a card with the creature type Dragon in its type line."
    );
    let mut t = TestGame::new(2);
    t.hand(P0, "Dragon's Disciple");
    t.answer_yes(P0, true);
    let a = t.enter(P0, "Dragon's Disciple");
    assert_eq!(plus1(&t, a), 0, "\"Dragon\" in the name isn't a Dragon card");
    // Nor is another Dragon's Disciple on the battlefield a Dragon.
    t.answer_yes(P0, true);
    let b = t.enter(P0, "Dragon's Disciple");
    assert_eq!(plus1(&t, b), 0);
    let shivan = t.hand(P0, "Shivan Dragon");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(shivan)]);
    let c = t.enter(P0, "Dragon's Disciple");
    assert_eq!(plus1(&t, c), 1, "revealed a Dragon card");
}

#[test]
fn rith_excess_damage_to_creatures_and_planeswalkers() {
    cr!("120.4a", "120.10");
    ruling!(
        "Rith, Liberated Primeval",
        "A creature has been dealt excess damage if one or more sources dealt more damage to it than the minimum amount of damage required to be lethal damage. In most cases, this means damage greater than its toughness, but consider the damage already dealt to it that turn."
    );
    ruling!(
        "Rith, Liberated Primeval",
        "A planeswalker is dealt excess damage if it’s dealt damage greater than its current loyalty."
    );
    supported("Rith, Liberated Primeval");
    let dragons = |t: &TestGame| crate::r_p108_common::tokens_with(t, P0, "Dragon");
    // A 3/3 already dealt 2 damage is dealt 2: 1 excess.
    let mut t = TestGame::new(2);
    let rith = t.battlefield(P0, "Rith, Liberated Primeval");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(rith, obj(giant), 2, false);
    t.g.deal_damage(rith, obj(giant), 2, false);
    t.settle();
    crate::r_p108_common::end_step(&mut t, P0);
    assert_eq!(dragons(&t), 1);
    // Exactly lethal (3 to a 3/3): no excess.
    let mut t = TestGame::new(2);
    let rith = t.battlefield(P0, "Rith, Liberated Primeval");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(rith, obj(giant), 3, false);
    t.settle();
    crate::r_p108_common::end_step(&mut t, P0);
    assert_eq!(dragons(&t), 0);
    // A planeswalker: more than its loyalty is excess, exactly its loyalty isn't.
    for (extra, excess) in [(0, 0), (1, 1)] {
        let mut t = TestGame::new(2);
        let rith = t.battlefield(P0, "Rith, Liberated Primeval");
        let pw = t.battlefield(P1, "Gideon, Ally of Zendikar");
        let loyalty = t.counters(pw, "loyalty");
        t.g.deal_damage(rith, obj(pw), loyalty + extra, false);
        t.settle();
        crate::r_p108_common::end_step(&mut t, P0);
        assert_eq!(dragons(&t), excess, "{extra}");
    }
}

#[test]
fn ready_willing_dont_affect_creatures_gained_later() {
    cr!("611.2c");
    ruling!(
        "Ready // Willing",
        "A creature that comes under your control after Ready resolves will not be granted by this effect."
    );
    ruling!(
        "Ready // Willing",
        "A creature that comes under your control after Willing resolves won't have deathtouch or lifelink, because adding abilities does change a creature's characteristics."
    );
    supported("Ready // Willing");
    for half in [0u8, 1] {
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Grizzly Bears");
        let theirs = t.battlefield(P1, "Hill Giant");
        t.lands(P0, "Forest", 1);
        t.lands(P0, "Plains", 2);
        t.lands(P0, "Swamp", 1);
        let card = t.hand(P0, "Ready // Willing");
        t.cast(P0, card).method(CastMethod::Half(half)).go();
        t.resolve_all();
        crate::r_p015_common::gain_control(&mut t, P0, theirs);
        let kw = if half == 0 {
            KeywordKind::Indestructible
        } else {
            KeywordKind::Lifelink
        };
        assert!(t.obj_now(mine).has_keyword(kw), "half {half}");
        assert!(!t.obj_now(theirs).has_keyword(kw), "half {half}");
    }
}

#[test]
fn legion_initiative_red_and_white_gets_both() {
    cr!("105.2", "613.4c");
    ruling!(
        "Legion's Initiative",
        "A creature that's both red and white will get +1/+1 from Legion's Initiative."
    );
    supported("Legion's Initiative");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Legion's Initiative");
    let blade = t.battlefield(P0, "Boros Swiftblade");
    assert_eq!(t.pt(blade), (2, 3));
}

#[test]
fn earnest_fellowship_colorless_gets_no_protection() {
    cr!("702.16a", "105.2c");
    ruling!(
        "Earnest Fellowship",
        "A creature with no color does not gain any Protection ability."
    );
    supported("Earnest Fellowship");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Earnest Fellowship");
    let thopter = t.battlefield(P1, "Ornithopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.recompute();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Protection));
    // A green spell can't target the Bears; it can target Ornithopter.
    let from = t.asked().len();
    cast_resolve_at(&mut t, P0, "Giant Growth", &[obj(thopter)]);
    let c = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(c[0].contains(&obj(thopter)) && !c[0].contains(&obj(bears)));
}

#[test]
fn kamahls_will_land_creature_keeps_its_types_and_abilities() {
    cr!("205.1b", "613.1d");
    ruling!(
        "Kamahl's Will",
        "A land that becomes a creature because of the first mode will retain any other supertypes, card types, subtypes, and abilities it had."
    );
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    cast_resolve_at(&mut t, P0, "Kamahl's Will", &[obj(forest)]);
    let o = t.obj_now(forest);
    assert!(o.is(CardType::Creature) && o.is(CardType::Land));
    assert!(o.chars.supertypes.contains(Supertype::Basic));
    assert!(o.chars.has_subtype("Forest") && o.chars.has_subtype("Elemental"));
    assert_eq!(t.pt(forest), (1, 1));
    assert!(t.obj_now(forest).has_keyword(KeywordKind::Haste));
    // Still taps for mana.
    t.g.recompute();
    let produces = mtg_engine::mana_abilities::could_produce(&t.g, forest);
    assert!(produces.contains(&mtg_engine::mana::ManaType::G));
}

#[test]
fn sparkshaper_visionary_planeswalker_creature_needs_continuous_control() {
    cr!("302.6", "508.1a");
    ruling!(
        "Sparkshaper Visionary",
        "A planeswalker that becomes a creature due to Sparkshaper Visionary's ability can't attack unless you've controlled it continuously since your turn began."
    );
    supported("Sparkshaper Visionary");
    for sick in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Sparkshaper Visionary");
        let pw = if sick {
            t.battlefield_sick(P0, "Gideon, Ally of Zendikar")
        } else {
            t.battlefield(P0, "Gideon, Ally of Zendikar")
        };
        t.answer_targets(P0, &[obj(pw)]);
        t.set_step(P0, Step::Upkeep);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.resolve_all();
        assert!(t.obj_now(pw).is_creature());
        assert_eq!(crate::r_s02_common::can_attack(&mut t, pw), !sick);
    }
}

#[test]
fn deification_hexproof_planeswalker_can_be_attacked() {
    cr!("702.11a", "508.1b");
    ruling!("Deification", "A planeswalker with hexproof may still be attacked.");
    supported("Deification");
    let mut t = TestGame::new(2);
    // The planeswalker type chosen is the first one offered, Ajani.
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    let pw = t.battlefield(P1, "Ajani Goldmane");
    t.enter(P1, "Deification");
    t.g.recompute();
    assert!(t.obj_now(pw).has_keyword(KeywordKind::Hexproof));
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(bears, obj(pw))]);
    assert!(crate::r_s10_common::attacking(&t, bears));
}

#[test]
fn linvala_stops_activated_not_triggered_abilities() {
    cr!("602.5", "603.2");
    ruling!(
        "Linvala, Shield of Sea Gate",
        "Triggered abilities (starting with “when,” “whenever,” or “at”) are unaffected."
    );
    supported("Linvala, Shield of Sea Gate");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Linvala, Shield of Sea Gate");
    // A full party: Cleric, Rogue, Warrior, and Linvala the Wizard.
    t.battlefield(P0, "Scion of the Swarm");
    t.battlefield(P0, "Bloodthirsty Aerialist");
    t.battlefield(P0, "Merciless Javelineer");
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    t.answer_targets(P0, &[obj(pyro)]);
    t.set_step(P0, Step::Upkeep);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(!can_activate(&mut t, P1, pyro));
    // A triggered ability of the chosen permanent still triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Linvala, Shield of Sea Gate");
    t.battlefield(P0, "Scion of the Swarm");
    t.battlefield(P0, "Bloodthirsty Aerialist");
    t.battlefield(P0, "Merciless Javelineer");
    let mate = t.battlefield(P1, "Ajani's Pridemate");
    t.answer_targets(P0, &[obj(mate)]);
    t.set_step(P0, Step::Upkeep);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(!can_block(&mut t, mate));
    gain_life(&mut t, P1, 2);
    t.resolve_all();
    assert_eq!(plus1(&t, mate), 1);
}

#[test]
fn ultra_magnus_put_onto_the_battlefield_attacking_didnt_attack() {
    cr!("508.4", "603.2");
    ruling!(
        "Ultra Magnus, Tactician // Ultra Magnus, Armored Carrier",
        "An artifact creature card put onto the battlefield by Ultra Magnus, Tactician's last ability is an attacking creature, but since it wasn't declared as an attacker, it never \"attacked,\""
    );
    supported("Ultra Magnus, Tactician // Ultra Magnus, Armored Carrier");
    supported("Kiln Walker");
    let mut t = TestGame::new(2);
    let magnus = t.battlefield(P0, "Ultra Magnus, Tactician // Ultra Magnus, Armored Carrier");
    let walker = t.hand(P0, "Kiln Walker");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(walker)]);
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(magnus, Entity::Player(P1))]);
    t.resolve_all();
    let w = t.g.current(walker);
    assert!(t.on_battlefield(w));
    assert!(crate::r_s10_common::attacking(&t, w));
    // Kiln Walker's "whenever this creature attacks" (+3/+0) didn't trigger.
    assert_eq!(t.pt(w), (0, 3));
}
