//! Rulings batch P160 — "shapechange": effects that set base power and toughness apply in
//! layer 7b (CR 613.4b) in timestamp order (CR 613.7), so they overwrite earlier setting
//! effects and are overwritten by later ones, while effects and counters that modify
//! power and toughness (7c) and switch effects (7d) apply on top no matter when they
//! began.

use crate::r_p160_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The base power and toughness that a setting spell used as an "earlier" or "later"
/// effect gives its target.
fn setter_pt(name: &str) -> (i32, i32) {
    match name {
        "Square Up" => (4, 4),
        "Relic's Roar" => (4, 3),
        "Mind Transfer Protocol" => (4, 5),
        "Suit Up" => (4, 5),
        "Majestic Metamorphosis" => (4, 4),
        "Humble" => (0, 1),
        _ => panic!("unknown setter {name}"),
    }
}

/// The standard layer-7 scenario on `target` (P0's creature): it has a +1/+1 counter, an
/// earlier base-setting effect (`earlier`), a +3/+3 effect (Giant Growth) and, if
/// `switch`, a switch effect (Twisted Image) — all before the tested effect. `apply` then
/// applies the tested effect, which should set its base power and toughness to `set`: the
/// counter, the pump and the switch still apply on top. Finally the setting spell `later`
/// overwrites the tested effect.
fn layer7(
    t: &mut TestGame,
    target: ObjectId,
    earlier: &str,
    switch: bool,
    apply: impl FnOnce(&mut TestGame),
    set: (i32, i32),
    later: &str,
) {
    let target = t.g.current(target);
    plus_counters(t, target, 1);
    cast_resolve(t, P0, earlier, &[Entity::Object(target)]);
    cast_resolve(t, P0, "Giant Growth", &[Entity::Object(target)]);
    if switch {
        cast_resolve(t, P0, "Twisted Image", &[Entity::Object(target)]);
    }
    let sw = |(p, q): (i32, i32)| if switch { (q, p) } else { (p, q) };
    let e = setter_pt(earlier);
    assert_eq!(t.pt(target), sw((e.0 + 4, e.1 + 4)), "after {earlier}");
    apply(t);
    t.g.recompute();
    assert_eq!(t.pt(target), sw((set.0 + 4, set.1 + 4)), "after the tested effect");
    cast_resolve(t, P0, later, &[Entity::Object(target)]);
    let l = setter_pt(later);
    assert_eq!(t.pt(target), sw((l.0 + 4, l.1 + 4)), "after {later}");
}

fn targeted_spell(name: &'static str, target: ObjectId) -> impl FnOnce(&mut TestGame) {
    move |t: &mut TestGame| cast_resolve(t, P0, name, &[Entity::Object(target)])
}

fn helpers_supported() {
    for name in [
        "Square Up",
        "Relic's Roar",
        "Mind Transfer Protocol",
        "Giant Growth",
        "Twisted Image",
    ] {
        supported(name);
    }
}

#[test]
fn setting_spells_overwrite_earlier_setting_effects_and_keep_modifiers() {
    cr!("613.4b", "613.4c", "613.4d", "613.7b");
    ruling!(
        "Square Up",
        "Square Up overwrites all previous effects that set the target creature’s power and toughness to specific values. Other effects that set its power or toughness to specific values that start to apply after Square Up resolves will overwrite this effect."
    );
    ruling!(
        "Relic's Roar",
        "Relic's Roar overwrites all previous effects that set the creature's base power and toughness to specific values. Any power- or toughness-setting effects that start to apply afterward will overwrite this effect."
    );
    ruling!(
        "Mind Transfer Protocol",
        "Mind Transfer Protocol overwrites all previous effects that set the creature or artifact's base power and toughness to specific values. Any power- or toughness-setting effects that start to apply afterward will overwrite this effect."
    );
    ruling!(
        "Majestic Metamorphosis",
        "Majestic Metamorphosis overwrites all previous effects that set the creature's base power and toughness to specific values. Any power- or toughness-setting effects that start to apply after Majestic Metamorphosis resolves will overwrite this effect."
    );
    ruling!(
        "Suit Up",
        "Suit Up will overwrite any previous effects that set the creature or Vehicle's power and toughness to specific numbers. Effects that otherwise modify its power and toughness will still apply no matter when they took effect. The same is true for +1/+1 counters."
    );
    ruling!(
        "Humble",
        "Humble overwrites all previous effects that set the creature’s base power and toughness to specific values. Any power- or toughness-setting effects that start to apply after Humble resolves will overwrite this effect."
    );
    ruling!(
        "Humble",
        "Effects that modify a creature’s power and/or toughness, such as the effect of Seal of Strength, will apply to the creature no matter when they started to take effect. The same is true for any counters that change its power and/or toughness and effects that switch its power and toughness."
    );
    ruling!(
        "Vengeant Earth",
        "If Vengeant Earth targets a creature you control, Vengeant Earth will overwrite any previous effects that set the creature’s power and toughness to specific values. Effects that otherwise modify the target creature’s power and toughness will still apply no matter when they took effect. The same is true for +1/+1 counters."
    );
    helpers_supported();
    let cases: [(&'static str, &str, &str, bool); 7] = [
        ("Square Up", "Relic's Roar", "Mind Transfer Protocol", false),
        ("Relic's Roar", "Square Up", "Mind Transfer Protocol", false),
        ("Mind Transfer Protocol", "Square Up", "Relic's Roar", false),
        ("Majestic Metamorphosis", "Relic's Roar", "Mind Transfer Protocol", false),
        ("Suit Up", "Square Up", "Relic's Roar", false),
        ("Humble", "Relic's Roar", "Mind Transfer Protocol", true),
        ("Vengeant Earth", "Relic's Roar", "Mind Transfer Protocol", false),
    ];
    for (name, earlier, later, switch) in cases {
        supported(name);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let set = if name == "Vengeant Earth" {
            (4, 4)
        } else {
            setter_pt(name)
        };
        layer7(
            &mut t,
            bears,
            earlier,
            switch,
            targeted_spell(name, bears),
            set,
            later,
        );
    }
}

#[test]
fn setting_activated_abilities_overwrite_earlier_setting_effects_and_keep_modifiers() {
    cr!("613.4b", "613.4c", "613.4d", "613.7b");
    ruling!(
        "Creeperhulk",
        "Effects that modify the target creature’s power and/or toughness, such as the effect of Giant Growth or Glorious Anthem, will apply to the creature no matter when they started applying. The same is true for counters that affect the target creature’s power and toughness and effects that switch its power and toughness."
    );
    ruling!(
        "Gigantomancer",
        "Effects that modify the targeted creature’s power or toughness, such as the effects of Giant Growth or Glorious Anthem, will apply to it no matter when they started to take effect. The same is true for counters that change the creature’s power or toughness (such as +1/+1 counters) and effects that switch its power and toughness."
    );
    ruling!(
        "Mirror Entity",
        "Mirror Entity's ability overwrites any effects that previously set a creature's base power and/or toughness. Any existing effects or counters that raise or lower a creature's power and/or toughness continue to apply to the creature's newly-set power and toughness."
    );
    ruling!(
        "Jolrael, Mwonvuli Recluse",
        "Jolrael's last ability overwrites all previous effects that set the affected creatures' power and/or toughness to specific values. If other effects that set these characteristics to specific values start to apply after Jolrael's ability resolves, they will overwrite that part of the effect."
    );
    ruling!(
        "Jolrael, Mwonvuli Recluse",
        "Effects that modify an affected creature's power and/or toughness without setting them to specific values will apply no matter when those effects began. The same is true for counters that change the creature's power and/or toughness."
    );
    helpers_supported();
    for name in [
        "Creeperhulk",
        "Gigantomancer",
        "Mirror Entity",
        "Jolrael, Mwonvuli Recluse",
    ] {
        supported(name);
    }

    // Creeperhulk: "{1}{G}: Until end of turn, target creature you control has base power
    // and toughness 5/5 and gains trample."
    let mut t = TestGame::new(2);
    let hulk = t.battlefield(P0, "Creeperhulk");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let apply = move |t: &mut TestGame| {
        t.lands(P0, "Forest", 2);
        activate_resolve(t, P0, hulk, 0, &[Entity::Object(bears)]);
    };
    layer7(&mut t, bears, "Relic's Roar", true, apply, (5, 5), "Square Up");

    // Gigantomancer: "{1}: Target creature you control has base power and toughness 7/7
    // until end of turn."
    let mut t = TestGame::new(2);
    let mancer = t.battlefield(P0, "Gigantomancer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let apply = move |t: &mut TestGame| {
        t.lands(P0, "Wastes", 1);
        activate_resolve(t, P0, mancer, 0, &[Entity::Object(bears)]);
    };
    layer7(&mut t, bears, "Relic's Roar", true, apply, (7, 7), "Square Up");

    // Mirror Entity with X = 3.
    let mut t = TestGame::new(2);
    let entity = t.battlefield(P0, "Mirror Entity");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let apply = move |t: &mut TestGame| {
        t.lands(P0, "Wastes", 3);
        t.answer(P0, DecisionKind::X, Answer::Number(3));
        activate_resolve(t, P0, entity, 0, &[]);
    };
    layer7(&mut t, bears, "Relic's Roar", false, apply, (3, 3), "Square Up");

    // Jolrael: "{4}{G}{G}: Until end of turn, creatures you control have base power and
    // toughness X/X, where X is the number of cards in your hand." Two cards in hand (the
    // card each spell's cast draws aside, Twisted Image isn't used here).
    let mut t = TestGame::new(2);
    let jolrael = t.battlefield(P0, "Jolrael, Mwonvuli Recluse");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let apply = move |t: &mut TestGame| {
        while t.hand_size(P0) < 2 {
            t.hand(P0, "Forest");
        }
        while t.hand_size(P0) > 2 {
            let c = t.g.player(P0).hand[0];
            t.g.move_object(c, mtg_engine::object::Zone::Graveyard(P0), mtg_engine::events::MoveCause::Effect, None);
        }
        t.lands(P0, "Forest", 6);
        activate_resolve(t, P0, jolrael, 0, &[]);
    };
    layer7(&mut t, bears, "Relic's Roar", false, apply, (2, 2), "Square Up");
}

#[test]
fn setting_auras_and_equipment_overwrite_earlier_setting_effects() {
    cr!("613.4b", "613.4c", "613.7a");
    ruling!(
        "Octopus Umbra",
        "Octopus Umbra overwrites all previous effects that set the creature's base power and toughness to specific values. Any power- or toughness-setting effects that start to apply to a creature after Octopus Umbra becomes attached to it will overwrite this effect."
    );
    ruling!(
        "Octopus Umbra",
        "Any power- or toughness-modifying effects (those that give +N/+N, for example) and counters will apply to the creature's new base power and toughness, even if they started to apply before Octopus Umbra became attached."
    );
    ruling!(
        "Blade of the Oni",
        "Blade of the Oni will overwrite any previous effects that set the equipped creature's power and toughness to specific numbers. Effects that otherwise modify the equipped creature's power and toughness will still apply no matter when they took effect. The same is true for +1/+1 counters."
    );
    helpers_supported();
    supported("Octopus Umbra");
    supported("Blade of the Oni");

    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    layer7(
        &mut t,
        bears,
        "Relic's Roar",
        false,
        targeted_spell("Octopus Umbra", bears),
        (8, 8),
        "Square Up",
    );

    // Blade of the Oni, reconfigured onto the creature: "Equipped creature has base power
    // and toughness 5/5 ...".
    let mut t = TestGame::new(2);
    let blade = t.battlefield(P0, "Blade of the Oni");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let apply = move |t: &mut TestGame| {
        t.lands(P0, "Swamp", 2);
        t.lands(P0, "Wastes", 2);
        activate_resolve(t, P0, blade, 0, &[Entity::Object(bears)]);
        assert_eq!(t.obj(blade).attached_to, Some(Entity::Object(bears)));
    };
    layer7(&mut t, bears, "Relic's Roar", false, apply, (5, 5), "Square Up");
}

#[test]
fn setting_triggered_abilities_overwrite_earlier_setting_effects() {
    cr!("613.4b", "613.4c", "613.7b");
    ruling!(
        "Glamer Gifter",
        "Glamer Gifter's last ability overwrites all previous effects that set the affected creature's base power and toughness to specific values. Any power- or toughness-setting effects that start to apply after the ability resolves will overwrite this effect."
    );
    ruling!(
        "Risen Riptide",
        "Risen Riptide’s ability overwrites any previous effects that set its power and/or toughness to specific values. Other effects that set these characteristics to specific values that start to apply after the ability resolves will overwrite that part of the effect."
    );
    ruling!(
        "Risen Riptide",
        "Effects that modify Risen Riptide’s power or toughness without setting it will apply to its new base power and toughness no matter when they started to take effect. The same is true for counters that change the creature’s power or toughness."
    );
    helpers_supported();
    for name in [
        "Glamer Gifter",
        "Risen Riptide",
        "Burst Lightning",
    ] {
        supported(name);
    }

    // Glamer Gifter: "When this creature enters, choose up to one other target creature.
    // Until end of turn, that creature has base power and toughness 4/4 ..." — tested
    // with an earlier 4/3 and a later 4/5.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let apply = move |t: &mut TestGame| {
        t.answer_targets(P0, &[Entity::Object(bears)]);
        t.enter(P0, "Glamer Gifter");
        t.resolve_all();
    };
    layer7(&mut t, bears, "Relic's Roar", false, apply, (4, 4), "Mind Transfer Protocol");

    // Risen Riptide: "Whenever you cast a kicked spell, this creature has base power and
    // toughness 5/5 until end of turn."
    let mut t = TestGame::new(2);
    let riptide = t.battlefield(P0, "Risen Riptide");
    let apply = move |t: &mut TestGame| {
        t.lands(P0, "Mountain", 1);
        t.lands(P0, "Wastes", 4);
        let bolt = t.hand(P0, "Burst Lightning");
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.cast(P0, bolt).kicked(true).go();
        t.resolve_all();
        assert_eq!(t.life(P1), 16);
    };
    layer7(&mut t, riptide, "Relic's Roar", false, apply, (5, 5), "Square Up");
}

#[test]
fn setting_static_abilities_overwrite_earlier_setting_effects() {
    cr!("613.4b", "613.4c", "613.4d", "613.7a");
    ruling!(
        "Harmonious Archon",
        "Harmonious Archon overwrites all previous effects that set a creature's base power and toughness to specific values. Any power- or toughness-setting effects that start to apply after Harmonious Archon enters the battlefield will overwrite this effect."
    );
    ruling!(
        "Kudo, King Among Bears",
        "Kudo, King Among Bears's ability overwrites any effects that set a creature's power and toughness. Any existing effects or counters that raise, lower, or switch a creature's power and/or toughness continue to apply to the creature's newly set power and toughness. Any power- or toughness-setting effects that start to apply after Kudo enters the battlefield will overwrite this effect."
    );
    ruling!(
        "Dollmaker's Shop // Porcelain Gallery",
        "Porcelain Gallery's ability overrides all previous effects that set the base power and toughness of creatures you control to specific values. Any power- or toughness-setting effects that start to apply to a creature you control after Porcelain Gallery is unlocked will overwrite this effect on that creature."
    );
    ruling!(
        "Dollmaker's Shop // Porcelain Gallery",
        "Effects that modify a creature's power and/or toughness, such as the effect of Jump Scare, will apply to that creature no matter when they started to take effect. The same is true for any counters that change its power and/or toughness and effects that switch its power and toughness."
    );
    helpers_supported();
    for name in [
        "Harmonious Archon",
        "Kudo, King Among Bears",
        "Dollmaker's Shop // Porcelain Gallery",
    ] {
        supported(name);
    }

    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let apply = |t: &mut TestGame| {
        t.battlefield(P1, "Harmonious Archon");
    };
    layer7(&mut t, bears, "Relic's Roar", false, apply, (3, 3), "Square Up");

    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Hill Giant");
    let apply = |t: &mut TestGame| {
        t.battlefield(P1, "Kudo, King Among Bears");
    };
    layer7(&mut t, bears, "Relic's Roar", true, apply, (2, 2), "Square Up");

    // Porcelain Gallery: "Creatures you control have base power and toughness each equal to
    // the number of creatures you control." Three creatures.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    let apply = |t: &mut TestGame| {
        let room = t.battlefield(P0, "Dollmaker's Shop // Porcelain Gallery");
        mtg_engine::rooms::unlock(&mut t.g, room, 1, P0);
        t.settle();
    };
    layer7(&mut t, bears, "Relic's Roar", true, apply, (3, 3), "Square Up");
}

#[test]
fn the_antiquities_war_makes_artifacts_5_5_keeping_modifiers() {
    cr!("613.4b", "613.4c", "613.4d", "714.2b");
    ruling!(
        "The Antiquities War",
        "Effects that modify an artifact creature’s power and/or toughness, such as the effect of Titanic Growth, will apply to the creature no matter when they started to take effect. The same is true for any counters that change its power and/or toughness and effects that switch its power and toughness."
    );
    helpers_supported();
    supported("The Antiquities War");
    supported("Ornithopter");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let saga = t.battlefield(P0, "The Antiquities War");
    let apply = move |t: &mut TestGame| {
        let saga = t.g.current(saga);
        t.g.add_counters(Entity::Object(saga), counters::LORE, 3, None);
        t.g.flush_events();
        t.resolve_all();
    };
    layer7(&mut t, thopter, "Relic's Roar", true, apply, (5, 5), "Square Up");
}

#[test]
fn damage_stays_marked_so_a_lower_base_toughness_can_be_lethal() {
    cr!("120.6", "704.5g", "613.4b");
    ruling!(
        "Harmonious Archon",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to creatures may become lethal if Harmonious Archon enters or leaves the battlefield during that turn."
    );
    ruling!(
        "Kudo, King Among Bears",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to creatures may become lethal if Kudo enters or leaves the battlefield during that turn."
    );
    ruling!(
        "Trickster's Elk",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to a creature may become lethal if Trickster's Elk becomes attached to it during that turn or becomes unattached later in the turn."
    );
    ruling!(
        "Dollmaker's Shop // Porcelain Gallery",
        "Because damage remains marked on creatures until the damage is removed as the turn ends, nonlethal damage dealt to creatures you control while Porcelain Gallery is unlocked may become lethal if other creatures you control leave the battlefield during that turn."
    );
    for name in [
        "Harmonious Archon",
        "Kudo, King Among Bears",
        "Trickster's Elk",
        "Dollmaker's Shop // Porcelain Gallery",
        "Shock",
    ] {
        supported(name);
    }

    // Harmonious Archon enters: a 1/4 with 3 damage becomes a 3/3.
    let mut t = TestGame::new(2);
    let turtle = t.battlefield(P1, "Horned Turtle");
    cast_resolve(&mut t, P0, "Lightning Bolt", &[Entity::Object(turtle)]);
    assert!(t.on_battlefield(turtle));
    t.battlefield(P0, "Harmonious Archon");
    t.settle();
    assert!(!t.on_battlefield(turtle));

    // Harmonious Archon leaves: a damaged creature that was 3/3 (a 1/1 with 2 damage) is a
    // 1/1 again.
    let mut t = TestGame::new(2);
    let archon = t.battlefield(P0, "Harmonious Archon");
    let elf = t.battlefield(P1, "Llanowar Elves");
    cast_resolve(&mut t, P0, "Shock", &[Entity::Object(elf)]);
    assert!(t.on_battlefield(elf));
    t.g.destroy(archon, None);
    t.settle();
    assert!(!t.on_battlefield(elf));

    // Kudo enters: a 1/4 with 3 damage becomes a 2/2.
    let mut t = TestGame::new(2);
    let turtle = t.battlefield(P1, "Horned Turtle");
    cast_resolve(&mut t, P0, "Lightning Bolt", &[Entity::Object(turtle)]);
    assert!(t.on_battlefield(turtle));
    t.battlefield(P0, "Kudo, King Among Bears");
    t.settle();
    assert!(!t.on_battlefield(turtle));

    // Trickster's Elk bestowed on a 1/4 with 3 damage: it's a 3/3 and dies.
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P0, "Horned Turtle");
    cast_resolve(&mut t, P0, "Lightning Bolt", &[Entity::Object(ogre)]);
    assert!(t.on_battlefield(ogre));
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 1);
    let elk = t.hand(P0, "Trickster's Elk");
    t.answer_targets(P0, &[Entity::Object(ogre)]);
    t.cast(P0, elk).method(mtg_engine::object::CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Bestow)).go();
    t.resolve_all();
    assert!(!t.on_battlefield(ogre));

    // Trickster's Elk becoming unattached: a 1/1 with 2 damage was a 3/3, and is a 1/1 again.
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 1);
    let elk = t.hand(P0, "Trickster's Elk");
    t.answer_targets(P0, &[Entity::Object(elf)]);
    t.cast(P0, elk).method(mtg_engine::object::CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Bestow)).go();
    t.resolve_all();
    assert_eq!(t.pt(elf), (3, 3));
    cast_resolve(&mut t, P0, "Shock", &[Entity::Object(elf)]);
    assert!(t.on_battlefield(elf));
    let elk = t.named_on_battlefield("Trickster's Elk")[0];
    t.g.unattach(elk);
    t.settle();
    assert!(!t.on_battlefield(elf));

    // Porcelain Gallery: with three creatures, each is 3/3; one with 2 damage dies when
    // another leaves.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    let room = t.battlefield(P0, "Dollmaker's Shop // Porcelain Gallery");
    mtg_engine::rooms::unlock(&mut t.g, room, 1, P0);
    t.settle();
    assert_eq!(t.pt(a), (3, 3));
    cast_resolve(&mut t, P0, "Shock", &[Entity::Object(a)]);
    assert!(t.on_battlefield(a));
    t.g.destroy(b, None);
    t.settle();
    assert!(!t.on_battlefield(a));
}

#[test]
fn creatures_set_to_zero_toughness_die() {
    cr!("704.5f", "613.4b");
    ruling!(
        "Mirror Entity",
        "Activating the ability with X = 0 will cause all creatures you control to become 0/0 and be put into the graveyard."
    );
    ruling!(
        "Biomass Mutation",
        "Choosing 0 as the value for X will likely cause creatures you control to become 0/0 and be put into the graveyard."
    );
    supported("Mirror Entity");
    supported("Biomass Mutation");
    let mut t = TestGame::new(2);
    let entity = t.battlefield(P0, "Mirror Entity");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    activate_resolve(&mut t, P0, entity, 0, &[]);
    assert!(!t.on_battlefield(entity));
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(theirs));

    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let pumped = t.battlefield(P0, "Grizzly Bears");
    plus_counters(&mut t, pumped, 1);
    t.lands(P0, "Forest", 2);
    let spell = t.hand(P0, "Biomass Mutation");
    t.cast(P0, spell).x(0).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    // A creature with a +1/+1 counter survives as a 1/1.
    assert_eq!(t.pt(pumped), (1, 1));
}

#[test]
fn biomass_mutation_example() {
    cr!("613.4b", "613.4c", "613.4d");
    ruling!(
        "Biomass Mutation",
        "Biomass Mutation overwrites any effects that set the power and/or toughness of a creature you control to a specific value. Effects that modify power and/or toughness but don't set them to a specific value (like the one created by Giant Growth), power/toughness changes from counters, and effects that switch a creature's power and toughness will continue to apply."
    );
    helpers_supported();
    supported("Biomass Mutation");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let apply = |t: &mut TestGame| {
        t.lands(P0, "Forest", 6);
        let spell = t.hand(P0, "Biomass Mutation");
        t.cast(P0, spell).x(4).go();
        t.resolve_all();
    };
    layer7(&mut t, bears, "Relic's Roar", true, apply, (4, 4), "Square Up");
}

#[test]
fn bramblefort_fink_sets_its_base_pt_under_its_modifiers() {
    cr!("613.4b", "613.4c", "613.4d");
    ruling!(
        "Bramblefort Fink",
        "Any effects that modify Bramblefort Fink's power and/or toughness without setting them to a specific number or value will apply after its base power and toughness are set, regardless of the order in which those effects were created. The same is true of counters that modify its power and toughness."
    );
    helpers_supported();
    supported("Bramblefort Fink");
    supported("Oko, Thief of Crowns");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oko, Thief of Crowns");
    let fink = t.battlefield(P0, "Bramblefort Fink");
    let apply = move |t: &mut TestGame| {
        t.lands(P0, "Wastes", 8);
        activate_resolve(t, P0, fink, 0, &[]);
    };
    layer7(&mut t, fink, "Relic's Roar", true, apply, (10, 10), "Square Up");
}
