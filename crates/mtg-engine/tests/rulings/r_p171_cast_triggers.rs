//! Rulings batch P171 — "whenever you cast" triggers: they resolve before the spell that
//! caused them to trigger, even if that spell is countered (Aven Wind Mage, Electrostatic
//! Field and Infantry, Erebor Flamesmith, Fire Urchin, Guttersnipe, Kiln Fiend, Weaver
//! of Lightning, Wee Dragonauts, God-Pharaoh's Faithful); spells of several colors (Tibor
//! and Lumia, God-Pharaoh's Faithful, Nightscape Familiar); Lightning Cloud; and
//! Two-Headed Giant.

use crate::r_p171_common::*;
use crate::r_s21_common::castable;
use crate::r_s25_common::targets_of;
use mtg_engine::decision::Answer;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 controls `name` and casts Lightning Bolt at P1 (with `extra` answers queued for the
/// trigger's targets); the trigger goes on the stack above the Bolt, P1 counters the Bolt
/// with Cancel, and everything resolves. Returns the permanent's id. P1 takes no damage
/// from the Bolt.
fn trigger_then_countered(t: &mut TestGame, name: &str, extra: &[Entity]) -> ObjectId {
    supported(name);
    let perm = t.battlefield(P0, name);
    let mut targets = vec![Entity::Player(P1)];
    targets.extend_from_slice(extra);
    let bolt = cast_targeting(t, P0, "Lightning Bolt", &targets);
    t.settle();
    let items = stack_items(t);
    assert_eq!(items.len(), 2, "{items:?}");
    assert_eq!(items[0], "Lightning Bolt");
    assert!(items[1].starts_with("ability"), "{items:?}");
    assert_eq!(triggers_from(t, perm), 1);
    t.answer_targets(P1, &[obj(bolt)]);
    cast_card(t, P1, "Cancel");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    perm
}

#[test]
fn pump_triggers_resolve_first_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5");
    ruling!(
        "Aven Wind Mage",
        "Aven Wind Mage’s triggered ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Fire Urchin",
        "Fire Urchin’s triggered ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Kiln Fiend",
        "Kiln Fiend's triggered ability resolves before the spell that causes it to trigger. The ability will resolve even if that spell is countered or otherwise leaves the stack."
    );
    ruling!(
        "Wee Dragonauts",
        "Wee Dragonauts's triggered ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Electrostatic Infantry",
        "Electrostatic Infantry’s triggered ability resolves before the spell that caused it to trigger. The ability will resolve even if that spell is countered."
    );
    for (name, pt) in [
        ("Aven Wind Mage", (3, 3)),
        ("Fire Urchin", (2, 3)),
        ("Kiln Fiend", (4, 2)),
        ("Wee Dragonauts", (3, 3)),
        ("Electrostatic Infantry", (2, 3)),
    ] {
        let mut t = TestGame::new(2);
        let perm = trigger_then_countered(&mut t, name, &[]);
        assert_eq!(t.pt(perm), pt, "{name}");
        assert_eq!(t.life(P1), 20, "{name}");
    }
}

#[test]
fn damage_triggers_resolve_first_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5");
    ruling!(
        "Electrostatic Field",
        "Electrostatic Field's triggered ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Erebor Flamesmith",
        "Erebor Flamesmith's triggered ability will resolve before the spell that caused the ability to trigger."
    );
    ruling!(
        "Guttersnipe",
        "Guttersnipe's triggered ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack without resolving."
    );
    for (name, life) in [
        ("Electrostatic Field", 19),
        ("Erebor Flamesmith", 19),
        ("Guttersnipe", 18),
    ] {
        let mut t = TestGame::new(2);
        trigger_then_countered(&mut t, name, &[]);
        assert_eq!(t.life(P1), life, "{name}");
    }
}

#[test]
fn god_pharaohs_faithful_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5");
    ruling!(
        "God-Pharaoh's Faithful",
        "The triggered ability of God-Pharaoh’s Faithful will resolve before the spell that caused it to trigger. The ability will resolve even if that spell is countered."
    );
    let mut t = TestGame::new(2);
    trigger_then_countered(&mut t, "God-Pharaoh's Faithful", &[]);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn weaver_of_lightning_resolves_after_the_spells_targets_are_chosen() {
    cr!("601.2c", "603.3", "405.5");
    ruling!(
        "Weaver of Lightning",
        "Weaver of Lightning's triggered ability resolves before the spell that caused it to trigger, but after targets for that spell have been chosen."
    );
    supported("Weaver of Lightning");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Weaver of Lightning");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ogre = t.battlefield(P1, "Hill Giant");
    // The Bolt targets the Bears; the trigger targets the Hill Giant.
    let bolt = cast_targeting(&mut t, P0, "Lightning Bolt", &[obj(bears), obj(ogre)]);
    t.settle();
    // The spell's target was chosen as it was cast, before the trigger went on the stack.
    assert_eq!(targets_of(&t, bolt), vec![obj(bears)]);
    assert_eq!(stack_items(&t)[0], "Lightning Bolt");
    t.resolve();
    // The trigger resolved first: the Giant has 1 damage, the Bolt is still on the stack.
    assert_eq!(damage_on(&t, ogre), 1);
    assert_eq!(t.stack_len(), 1);
    assert!(t.on_battlefield(bears));
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn tibor_and_lumia_a_blue_and_red_spell_triggers_both_in_either_order() {
    cr!("603.2", "603.3b");
    ruling!(
        "Tibor and Lumia",
        "A spell you cast that's blue and red will trigger both abilities. You can put them on the stack in either order."
    );
    supported("Tibor and Lumia");
    supported("Electrolyze");
    let mut tops = Vec::new();
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        let tibor = t.battlefield(P0, "Tibor and Lumia");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.answer(P0, DecisionKind::Order, Answer::Indices(order));
        // Electrolyze ({1}{U}{R}): 2 damage divided; its target is P1. Tibor's blue
        // trigger targets the Bears.
        cast_targeting(&mut t, P0, "Electrolyze", &[Entity::Player(P1), obj(bears)]);
        t.settle();
        assert_eq!(triggers_from(&t, tibor), 2);
        let top = stack_items(&t).last().unwrap().clone();
        let flying_on_top = top.contains("gains flying");
        tops.push(flying_on_top);
        t.resolve();
        // The top trigger resolved first. If it was the red one, the Bears (without
        // flying yet) were dealt 1 damage; if it was the blue one, the Bears now fly and
        // the red one doesn't damage them.
        t.resolve();
        let dmg = damage_on(&t, bears);
        assert_eq!(dmg, if flying_on_top { 0 } else { 1 }, "{top}");
        t.resolve_all();
    }
    assert_ne!(tops[0], tops[1]);
}

#[test]
fn god_pharaohs_faithful_a_multicolored_spell_gains_only_1_life() {
    cr!("603.2", "105.2b");
    ruling!(
        "God-Pharaoh's Faithful",
        "If you cast a spell that’s two or more of these colors, you gain only 1 life."
    );
    supported("God-Pharaoh's Faithful");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "God-Pharaoh's Faithful");
    cast_targeting(&mut t, P0, "Electrolyze", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn nightscape_familiar_a_blue_and_red_spell_costs_only_1_less() {
    cr!("601.2f");
    ruling!(
        "Nightscape Familiar",
        "A spell you cast that’s blue and red costs {1} less, not {2} less."
    );
    supported("Nightscape Familiar");
    supported("Prophetic Bolt");
    // Prophetic Bolt costs {3}{U}{R}: with the Familiar, {2}{U}{R}.
    for (generic, ok) in [(1usize, false), (2, true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Nightscape Familiar");
        t.lands(P0, "Island", 1);
        t.lands(P0, "Mountain", 1);
        t.lands(P0, "Wastes", generic);
        let bolt = t.hand(P0, "Prophetic Bolt");
        assert_eq!(castable(&mut t, P0, bolt), ok, "{generic} generic");
    }
}

#[test]
fn lightning_cloud_triggers_on_cast_and_asks_for_payment_on_resolution() {
    cr!("603.2", "603.3d", "603.5", "118.12");
    ruling!("Lightning Cloud", "Triggers when the spell is cast.");
    ruling!(
        "Lightning Cloud",
        "You pick the target when putting the triggered ability on the stack, but you don’t choose whether or not to pay {R} until the triggered ability resolves."
    );
    supported("Lightning Cloud");
    for pay in [true, false] {
        let mut t = TestGame::new(2);
        let cloud = t.battlefield(P0, "Lightning Cloud");
        let mountain = t.battlefield(P0, "Mountain");
        let bears = t.battlefield(P1, "Grizzly Bears");
        // P1 casts Shock at P0; the Cloud (controlled by P0) triggers on that cast.
        t.answer_targets(P0, &[obj(bears)]);
        let shock = cast_targeting(&mut t, P1, "Shock", &[Entity::Player(P0)]);
        let before = t.asked().len();
        t.settle();
        assert_eq!(triggers_from(&t, cloud), 1);
        assert_eq!(t.stack[0], shock);
        // The trigger's target was chosen as it went on the stack; no payment yet.
        let trig = *t.stack.last().unwrap();
        assert_eq!(targets_of(&t, trig), vec![obj(bears)]);
        assert!(!t.asked()[before..]
            .iter()
            .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::YesNo { .. })));
        assert!(!t.obj(mountain).tapped);
        t.answer_yes(P0, pay);
        t.resolve();
        assert_eq!(damage_on(&t, bears), u32::from(pay));
        assert_eq!(t.obj(mountain).tapped, pay);
        t.resolve_all();
    }
}

fn two_headed_giant() -> TestGame {
    TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    )
}

#[test]
fn two_headed_giant_each_opponent_triggers_hit_the_team_twice() {
    cr!("810.9", "810.4");
    ruling!(
        "Electrostatic Field",
        "In a Two-Headed Giant game, Electrostatic Field's triggered ability causes the opposing team to lose 2 life."
    );
    ruling!(
        "Guttersnipe",
        "In a Two-Headed Giant game, Guttersnipe's ability causes the opposing team to lose 4 life."
    );
    for (name, loss) in [("Electrostatic Field", 2), ("Guttersnipe", 4)] {
        supported(name);
        let mut t = two_headed_giant();
        assert_eq!(t.life(P2), 30);
        t.battlefield(P0, name);
        let bears = t.battlefield(P0, "Grizzly Bears");
        cast_targeting(&mut t, P0, "Giant Growth", &[obj(bears)]);
        t.resolve_all();
        assert_eq!(t.life(P2), 30 - loss, "{name}");
        assert_eq!(t.life(P3), 30 - loss, "{name}");
        assert_eq!(t.life(P0), 30, "{name}");
    }
}
