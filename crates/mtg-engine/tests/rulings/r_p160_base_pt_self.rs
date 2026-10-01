//! Rulings batch P160 — "shapechange" abilities that set the source's own base power
//! (and toughness): "you may have ~'s base power and toughness become 4/2 until end of
//! turn", "~ has base power 4", "~'s base power becomes equal to target creature's
//! power", and similar (layer 7b, CR 613.4b).

use crate::r_p160_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn mirkwood_meditator_sets_its_base_pt_on_landfall() {
    cr!("613.4b", "613.4c", "613.4d", "613.7b", "120.6", "704.5g");
    ruling!(
        "Mirkwood Meditator",
        "The ability of Mirkwood Meditator overwrites any previous effects that set its power and/or toughness to specific values. Other effects that set these characteristics to specific values that start to apply after the ability resolves will overwrite that part of the effect."
    );
    ruling!(
        "Mirkwood Meditator",
        "Effects that modify the power or toughness of Mirkwood Meditator without setting it will apply to its new base power and toughness no matter when they started to take effect. The same is true for counters that change its power and toughness."
    );
    ruling!(
        "Mirkwood Meditator",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to Mirkwood Meditator may become lethal if you change its base toughness during that turn."
    );
    helpers_supported();
    supported("Mirkwood Meditator");
    let landfall = |t: &mut TestGame| {
        yes(t, P0);
        t.enter(P0, "Forest");
        t.resolve_all();
    };
    let mut t = TestGame::new(2);
    let med = t.battlefield(P0, "Mirkwood Meditator");
    layer7(&mut t, med, "Relic's Roar", true, landfall, (4, 2), "Square Up");

    // A 2/4 with 2 damage becomes a 4/2 and dies.
    let mut t = TestGame::new(2);
    let med = t.battlefield(P0, "Mirkwood Meditator");
    cast_resolve(&mut t, P1, "Shock", &[Entity::Object(med)]);
    assert!(t.on_battlefield(med));
    landfall(&mut t);
    assert!(!t.on_battlefield(med));
}

#[test]
fn master_of_winds_chooses_4_1_or_1_4() {
    cr!("613.4b", "613.4c", "613.7b", "120.6", "704.5g");
    ruling!(
        "Master of Winds",
        "The ability of Master of Winds overwrites any previous effects that set its power and/or toughness to specific values. Other effects that set these characteristics to specific values that start to apply after the ability resolves will overwrite that part of the effect."
    );
    ruling!(
        "Master of Winds",
        "Effects that modify the power or toughness of Master of Winds without setting it will apply to its new base power and toughness no matter when they started to take effect. The same is true for counters that change its power and toughness."
    );
    ruling!(
        "Master of Winds",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to Master of Winds may become lethal if you change its base toughness during that turn."
    );
    helpers_supported();
    supported("Master of Winds");
    supported("Opt");
    // "Whenever you cast an instant, sorcery, or Wizard spell, you may have this creature's
    // base power and toughness become 4/1 or 1/4 until end of turn." (The setup spells are
    // cast by the opponent so they don't trigger it.)
    let cast_opt = |option: usize| {
        move |t: &mut TestGame| {
            yes(t, P0);
            t.answer(P0, DecisionKind::Option, Answer::Index(option));
            cast_resolve(t, P0, "Opt", &[]);
        }
    };
    let mut t = TestGame::new(2);
    let master = t.battlefield(P0, "Master of Winds");
    layer7_by(&mut t, P1, master, "Relic's Roar", false, cast_opt(0), (4, 1), "Square Up");
    let mut t = TestGame::new(2);
    let master = t.battlefield(P0, "Master of Winds");
    layer7_by(&mut t, P1, master, "Relic's Roar", false, cast_opt(1), (1, 4), "Square Up");

    // A 1/4 with 2 damage that becomes a 4/1 dies.
    let mut t = TestGame::new(2);
    let master = t.battlefield(P0, "Master of Winds");
    cast_resolve(&mut t, P1, "Shock", &[Entity::Object(master)]);
    assert!(t.on_battlefield(master));
    cast_opt(0)(&mut t);
    assert!(!t.on_battlefield(master));
}

#[test]
fn symmetry_sage_sets_only_the_base_power() {
    cr!("613.4b", "613.4c", "613.7b");
    ruling!(
        "Symmetry Sage",
        "Symmetry Sage’s triggered ability overwrites all previous effects that set the target creature’s power to a specific value. Other effects that set its power to specific values that start to apply after this ability resolves will overwrite this effect."
    );
    ruling!(
        "Symmetry Sage",
        "Any effects that modify a creature’s power without setting it to a specific value (i.e. ones that don’t affect base power) will apply after its base power is set, regardless of the order those effects were created. The same is true for counters that modify its power."
    );
    helpers_supported();
    supported("Symmetry Sage");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Symmetry Sage");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let apply = move |t: &mut TestGame| {
        t.answer_targets(P0, &[Entity::Object(bears)]);
        cast_resolve(t, P0, "Opt", &[]);
    };
    // Base 4/3 (Relic's Roar) becomes base 2/3.
    layer7_by(&mut t, P1, bears, "Relic's Roar", false, apply, (2, 3), "Square Up");
}

#[test]
fn turtle_duck_sets_its_base_power() {
    cr!("613.4b", "613.4c", "613.7b");
    ruling!(
        "Turtle-Duck",
        "Effects that modify Turtle-Duck's power without setting it will apply no matter when they started to take effect. The same is true for counters that change its power."
    );
    supported("Turtle-Duck");
    supported("Gigantomancer");
    let mut t = TestGame::new(2);
    let duck = t.battlefield(P0, "Turtle-Duck");
    let mancer = t.battlefield(P0, "Gigantomancer");
    plus_counters(&mut t, duck, 1);
    cast_resolve(&mut t, P0, "Giant Growth", &[Entity::Object(duck)]);
    // An earlier setting effect: 7/7.
    t.lands(P0, "Wastes", 1);
    activate_resolve(&mut t, P0, mancer, 0, &[Entity::Object(duck)]);
    assert_eq!(t.pt(duck), (11, 11));
    t.lands(P0, "Wastes", 3);
    activate_resolve(&mut t, P0, duck, 0, &[]);
    assert_eq!(t.pt(duck), (8, 11));
    assert!(t
        .obj(duck)
        .has_keyword(mtg_engine::keywords::KeywordKind::Trample));
    cast_resolve(&mut t, P0, "Square Up", &[Entity::Object(duck)]);
    assert_eq!(t.pt(duck), (8, 8));
}

#[test]
fn riptide_mangler_copies_a_power_indefinitely() {
    cr!("613.4b", "613.4c", "613.4d", "611.2a");
    ruling!(
        "Riptide Mangler",
        "Effects that modify Riptide Mangler’s power, such as the effect of Giant Growth or Glorious Anthem, will apply to Riptide Mangler no matter when they started applying. The same is true for counters that affect Riptide Mangler’s power and effects that switch its power and toughness."
    );
    helpers_supported();
    supported("Riptide Mangler");
    let mut t = TestGame::new(2);
    let mangler = t.battlefield(P0, "Riptide Mangler");
    let bears = t.battlefield(P1, "Grizzly Bears");
    plus_counters(&mut t, mangler, 1);
    cast_resolve(&mut t, P0, "Giant Growth", &[Entity::Object(mangler)]);
    cast_resolve(&mut t, P0, "Twisted Image", &[Entity::Object(mangler)]);
    // 0/3 +1/+1 +3/+3, switched: 7/4.
    assert_eq!(t.pt(mangler), (7, 4));
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    activate_resolve(&mut t, P0, mangler, 0, &[Entity::Object(bears)]);
    // Base 2/3 +1/+1 +3/+3 = 6/7, switched: 7/6.
    assert_eq!(t.pt(mangler), (7, 6));
    // The effect lasts indefinitely: next turn it's a 3/4 (base 2/3 and the counter).
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(mangler), (3, 4));
}

#[test]
fn eldrazi_mimic_copies_the_entering_creatures_pt() {
    cr!("613.4b", "613.4c", "613.7b");
    ruling!(
        "Eldrazi Mimic",
        "Eldrazi Mimic’s ability will overwrite any other effect setting its base power and toughness. Such effects that begin to apply after that ability resolves will similarly overwrite the ability. Effects that modify Eldrazi Mimic’s power and/or toughness but don’t set its base power and/or toughness to specific values will apply no matter when they started to apply. The same is true for any +1/+1 counters it may have."
    );
    helpers_supported();
    supported("Eldrazi Mimic");
    let mut t = TestGame::new(2);
    let mimic = t.battlefield(P0, "Eldrazi Mimic");
    let apply = |t: &mut TestGame| {
        yes(t, P0);
        t.enter(P0, "Ornithopter");
        t.resolve_all();
    };
    layer7(&mut t, mimic, "Relic's Roar", true, apply, (0, 2), "Square Up");
}

#[test]
fn belligerent_yearling_takes_the_latest_dinosaurs_power() {
    cr!("613.4b", "613.4c", "613.7b");
    ruling!(
        "Belligerent Yearling",
        "Belligerent Yearling's ability overwrites any previous effects that set its power, including previous instances of that ability. Other effects that set its power to specific values that start to apply after the ability resolves, including future instances of the ability, will overwrite that effect."
    );
    ruling!(
        "Belligerent Yearling",
        "Effects that modify the power of Belligerent Yearling without setting it to a specific value will apply to the new base power no matter when they started to take effect. The same is true for counters that change its power."
    );
    supported("Belligerent Yearling");
    supported("Charging Monstrosaur");
    supported("Rampaging Ferocidon");
    let mut t = TestGame::new(2);
    let yearling = t.battlefield(P0, "Belligerent Yearling");
    plus_counters(&mut t, yearling, 1);
    cast_resolve(&mut t, P0, "Giant Growth", &[Entity::Object(yearling)]);
    assert_eq!(t.pt(yearling), (7, 6));
    yes(&mut t, P0);
    t.enter(P0, "Charging Monstrosaur");
    t.resolve_all();
    assert_eq!(t.pt(yearling), (9, 6));
    yes(&mut t, P0);
    t.enter(P0, "Rampaging Ferocidon");
    t.resolve_all();
    assert_eq!(t.pt(yearling), (7, 6));
    cast_resolve(&mut t, P0, "Square Up", &[Entity::Object(yearling)]);
    assert_eq!(t.pt(yearling), (8, 8));
}

#[test]
fn arni_brokenbrow_boast_sets_its_base_power() {
    cr!("613.4b", "613.4c", "702.142a", "107.1b");
    ruling!(
        "Arni Brokenbrow",
        "Any effects that modify Arni Brokenbrow's power without setting it to a specific value will apply after Arni's base power is set, regardless of the order in which those effects were created. The same is true of counters that modify Arni's power."
    );
    ruling!(
        "Arni Brokenbrow",
        "In some unfortunate cases, the greatest power among other creatures you control may be negative. For example, if that greatest power is -3, you could change Arni's base power to -2 until end of turn."
    );
    supported("Arni Brokenbrow");
    supported("Weakness");
    // "Boast — {1}: You may have Arni's base power become 1 plus the greatest power among
    // other creatures you control until end of turn."
    let mut t = TestGame::new(2);
    let arni = t.battlefield(P0, "Arni Brokenbrow");
    t.battlefield(P0, "Hill Giant");
    plus_counters(&mut t, arni, 1);
    cast_resolve(&mut t, P0, "Giant Growth", &[Entity::Object(arni)]);
    assert_eq!(t.pt(arni), (7, 7));
    t.attack(&[(arni, Entity::Player(P1))], &[]);
    t.lands(P0, "Wastes", 1);
    yes(&mut t, P0);
    activate_resolve(&mut t, P0, arni, 0, &[]);
    // Base power 4 (1 plus Hill Giant's 3), +1 +3.
    assert_eq!(t.pt(arni), (8, 7));

    // The only other creature is a -2/1 (an Ornithopter with Weakness): base power -1.
    let mut t = TestGame::new(2);
    let arni = t.battlefield(P0, "Arni Brokenbrow");
    let thopter = t.battlefield(P0, "Ornithopter");
    cast_resolve(&mut t, P0, "Weakness", &[Entity::Object(thopter)]);
    assert_eq!(t.pt(thopter), (-2, 1));
    t.attack(&[(arni, Entity::Player(P1))], &[]);
    t.lands(P0, "Wastes", 1);
    yes(&mut t, P0);
    activate_resolve(&mut t, P0, arni, 0, &[]);
    assert_eq!(t.pt(arni), (-1, 3));
}

#[test]
fn pupu_ufo_putting_a_land_isnt_playing_one() {
    cr!("305.4", "305.2");
    ruling!(
        "PuPu UFO",
        "Putting a land card onto the battlefield with PuPu UFO's first activated ability doesn't count as playing a land. You can put a land card onto the battlefield this way even if you've already played a land for the turn."
    );
    supported("PuPu UFO");
    // After a land play, the UFO can still put a land onto the battlefield.
    let mut t = TestGame::new(2);
    let ufo = t.battlefield(P0, "PuPu UFO");
    let first = t.hand(P0, "Forest");
    t.play_land(P0, first).unwrap();
    let second = t.hand(P0, "Island");
    yes(&mut t, P0);
    t.answer_choose(P0, &[Entity::Object(second)]);
    activate_resolve(&mut t, P0, ufo, 0, &[]);
    assert!(t.on_battlefield(second));
    // Putting a land with it first doesn't use up the land play.
    let mut t = TestGame::new(2);
    let ufo = t.battlefield(P0, "PuPu UFO");
    let first = t.hand(P0, "Island");
    yes(&mut t, P0);
    t.answer_choose(P0, &[Entity::Object(first)]);
    activate_resolve(&mut t, P0, ufo, 0, &[]);
    assert!(t.on_battlefield(first));
    let second = t.hand(P0, "Forest");
    t.play_land(P0, second).unwrap();
    assert!(t.on_battlefield(second));
}

#[test]
fn allosaurus_shepherd_sets_elves_base_pt_and_makes_them_dinosaurs() {
    cr!("613.4b", "613.4c", "613.1d", "613.7b", "611.2c");
    ruling!(
        "Allosaurus Shepherd",
        "Allosaurus Shepherd's last ability overwrites all previous effects that set the affected creatures' power and/or toughness to specific values. Other effects that set these characteristics to specific values that start to apply after the ability resolves will overwrite that part of the effect."
    );
    ruling!(
        "Allosaurus Shepherd",
        "Effects that modify an affected creature's power or toughness without setting it will apply no matter when they started to take effect. The same is true for counters that change the creature's power or toughness."
    );
    helpers_supported();
    supported("Allosaurus Shepherd");
    let mut t = TestGame::new(2);
    let shepherd = t.battlefield(P0, "Allosaurus Shepherd");
    let elf = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let apply = move |t: &mut TestGame| {
        t.lands(P0, "Forest", 6);
        activate_resolve(t, P0, shepherd, 0, &[]);
        assert!(t.obj(elf).chars.subtypes.iter().any(|s| s == "Dinosaur"));
        assert!(t.obj(elf).chars.subtypes.iter().any(|s| s == "Elf"));
        assert_eq!(t.pt(shepherd), (5, 5));
        // Not an Elf: unaffected.
        assert_eq!(t.pt(bears), (2, 2));
    };
    layer7(&mut t, elf, "Relic's Roar", true, apply, (5, 5), "Square Up");
}

#[test]
fn archon_of_the_wild_rose_sets_enchanted_creatures_base_pt() {
    cr!("613.4b", "613.4c", "613.7a", "303.4b");
    ruling!(
        "Archon of the Wild Rose",
        "Archon of the Wild Rose's ability overwrites all previous effects that set the affected creatures' power and/or toughness to specific values. Other effects that set these characteristics to specific values that start to apply after Archon of the Wild Rose enters the battlefield will overwrite this effect."
    );
    ruling!(
        "Archon of the Wild Rose",
        "Effects that modify a creature's power and/or toughness without setting it will apply to the affected creatures no matter when they started to take effect. The same is true for counters that change a creature's power and/or toughness."
    );
    helpers_supported();
    supported("Archon of the Wild Rose");
    supported("Pacifism");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let plain = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Pacifism", &[Entity::Object(bears)]);
    // An Aura an opponent controls doesn't count.
    t.set_step(P1, Step::PrecombatMain);
    cast_resolve(&mut t, P1, "Pacifism", &[Entity::Object(theirs)]);
    t.set_step(P0, Step::PrecombatMain);
    let apply = move |t: &mut TestGame| {
        t.battlefield(P0, "Archon of the Wild Rose");
        t.g.recompute();
        assert!(t
            .obj(bears)
            .has_keyword(mtg_engine::keywords::KeywordKind::Flying));
        assert_eq!(t.pt(plain), (2, 2));
        assert_eq!(t.pt(theirs), (2, 2));
    };
    layer7(&mut t, bears, "Relic's Roar", true, apply, (4, 4), "Mind Transfer Protocol");
}
