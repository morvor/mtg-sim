//! "[object] becomes [type words] with \"[ability]\"" and "[object] becomes ..., gets ...,
//! and gains \"[ability]\"" (`src/oracle/patterns/grant_grammar.rs`): one continuous
//! effect that changes what the object is and grants the quoted abilities for the same
//! duration (CR 611.2, 613.1d-f). "This creature" in a granted ability is the object
//! that has it (CR 113.6, 113.7); a granted "power and toughness are each equal to"
//! ability isn't characteristic-defining (CR 604.3a), so it applies in layer 7b in
//! timestamp order (CR 613.4b, 613.7a).

use mtg_engine::ability::AbilityKind;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

fn is_creature(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).chars.card_types.contains(CardType::Creature)
}

fn activated(t: &TestGame, id: ObjectId) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .count()
}

#[test]
fn raging_ravine_animated_land_has_the_granted_attack_trigger() {
    cr!("611.2a", "613.1f", "113.7");
    ruling!(
        "Raging Ravine",
        "it will get that many instances of the granted triggered ability"
    );
    ruling!(
        "Raging Ravine",
        "Any +1/+1 counters put on Raging Ravine remain on it"
    );
    assert_supported(&["Raging Ravine"]);
    let mut t = TestGame::new(2);
    let rr = t.battlefield(P0, "Raging Ravine");
    t.lands(P0, "Mountain", 4);
    t.lands(P0, "Forest", 4);
    t.settle();
    assert!(!is_creature(&t, rr));
    t.set_step(P0, Step::PrecombatMain);
    // Activated twice: two instances of the triggered ability.
    t.activate(P0, rr, 1, &[]).unwrap();
    t.resolve();
    t.activate(P0, rr, 1, &[]).unwrap();
    t.resolve();
    assert!(is_creature(&t, rr));
    assert!(t.obj_now(rr).chars.card_types.contains(CardType::Land));
    assert!(t.obj_now(rr).chars.has_subtype("Elemental"));
    assert_eq!(t.pt(rr), (3, 3));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(rr, Entity::Player(P1))], &[]);
    assert_eq!(t.counters(rr, "+1/+1"), 2);
    assert_eq!(t.life(P1), 15);
    // Next turn it's a land again; the counters stay.
    t.advance_to(P1, Step::Upkeep);
    assert!(!is_creature(&t, rr));
    assert_eq!(t.counters(rr, "+1/+1"), 2);
}

#[test]
fn spawning_pool_skeleton_regenerates_with_its_granted_ability() {
    cr!("611.2a", "113.7", "701.19a");
    assert_supported(&["Spawning Pool"]);
    let mut t = TestGame::new(2);
    let sp = t.battlefield(P0, "Spawning Pool");
    t.lands(P0, "Swamp", 3);
    t.settle();
    let before = activated(&t, sp);
    t.activate(P0, sp, 1, &[]).unwrap();
    t.resolve();
    assert_eq!(t.pt(sp), (1, 1));
    assert_eq!(activated(&t, sp), before + 1);
    // "{B}: Regenerate this creature" regenerates the land creature.
    t.activate(P0, sp, before, &[]).unwrap();
    t.resolve();
    t.g.destroy_all(vec![sp], None, false);
    t.settle();
    assert!(t.on_battlefield(sp));
    assert!(t.obj_now(sp).tapped);
}

#[test]
fn chimeric_mass_granted_pt_ability_applies_in_timestamp_order() {
    cr!("604.3a", "613.4b", "613.7a");
    ruling!(
        "Chimeric Mass",
        "If the number of charge counters on Chimeric Mass changes while it is a creature"
    );
    ruling!(
        "Chimeric Mass",
        "it will become a 0/0 creature and be put into its owner’s graveyard"
    );
    ruling!(
        "Chimeric Mass",
        "Activating the last ability while Chimeric Mass is a creature will override any effects that set its power"
    );
    assert_supported(&["Chimeric Mass", "Diminish"]);
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Chimeric Mass");
    t.g.add_counters(Entity::Object(m), "charge", 3, None);
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Island", 1);
    t.activate(P0, m, 0, &[]).unwrap();
    t.resolve();
    assert!(t.obj_now(m).chars.has_subtype("Construct"));
    assert_eq!(t.pt(m), (3, 3));
    t.g.add_counters(Entity::Object(m), "charge", 1, None);
    t.settle();
    assert_eq!(t.pt(m), (4, 4));
    // A later "base power and toughness 1/1" effect wins...
    let d = t.hand(P0, "Diminish");
    t.cast(P0, d).target(m).go();
    t.resolve();
    assert_eq!(t.pt(m), (1, 1));
    // ... until the ability is activated again.
    t.activate(P0, m, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.pt(m), (4, 4));
    // With no charge counters it's a 0/0 and dies.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Chimeric Mass");
    t.lands(P0, "Plains", 1);
    t.activate(P0, m, 0, &[]).unwrap();
    t.resolve();
    assert!(!t.on_battlefield(m));
}

#[test]
fn myth_realized_and_svogthos_get_their_pt_from_the_granted_ability() {
    cr!("611.2a", "613.4b");
    assert_supported(&["Myth Realized", "Svogthos, the Restless Tomb"]);
    let mut t = TestGame::new(2);
    let mr = t.battlefield(P0, "Myth Realized");
    t.g.add_counters(Entity::Object(mr), "lore", 2, None);
    t.lands(P0, "Plains", 1);
    // Abilities: "{2}{W}: lore counter" is index 0, the animation index 1.
    t.activate(P0, mr, 1, &[]).unwrap();
    t.resolve();
    assert!(is_creature(&t, mr));
    assert!(t.obj_now(mr).chars.card_types.contains(CardType::Enchantment));
    assert!(t.obj_now(mr).chars.has_subtype("Monk"));
    assert_eq!(t.pt(mr), (2, 2));
    let mut t = TestGame::new(2);
    let sv = t.battlefield(P0, "Svogthos, the Restless Tomb");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Forest", 3);
    t.activate(P0, sv, 1, &[]).unwrap();
    t.resolve();
    assert_eq!(t.pt(sv), (2, 2));
}

#[test]
fn defiling_tears_becomes_gets_and_gains_a_quoted_ability() {
    cr!("611.2a", "613.1e", "113.7");
    assert_supported(&["Defiling Tears"]);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    let s = t.hand(P0, "Defiling Tears");
    t.cast(P0, s).target(bears).go();
    t.resolve();
    let o = t.obj_now(bears);
    assert_eq!(o.chars.colors, ColorSet::single(Color::Black));
    assert_eq!(t.pt(bears), (3, 1));
    // The creature's controller activates the granted ability.
    t.lands(P1, "Swamp", 1);
    t.activate(P1, bears, 0, &[]).unwrap();
    t.resolve();
    t.g.destroy_all(vec![bears], None, false);
    t.settle();
    assert!(t.on_battlefield(bears));
}

#[test]
fn mizzium_tank_becomes_a_creature_and_gets_a_bonus() {
    cr!("611.2a", "301.7");
    ruling!("Mizzium Tank", "triggers even if it's already a creature");
    assert_supported(&["Mizzium Tank"]);
    let mut t = TestGame::new(2);
    let tank = t.battlefield(P0, "Mizzium Tank");
    t.lands(P0, "Mountain", 2);
    let b = t.hand(P0, "Lightning Bolt");
    t.cast(P0, b).target(P1).go();
    t.resolve_all();
    assert!(is_creature(&t, tank));
    assert_eq!(t.pt(tank), (4, 3));
    let b = t.hand(P0, "Lightning Bolt");
    t.cast(P0, b).target(P1).go();
    t.resolve_all();
    assert_eq!(t.pt(tank), (5, 4));
    // A creature spell doesn't trigger it.
    let mut t = TestGame::new(2);
    let tank = t.battlefield(P0, "Mizzium Tank");
    t.lands(P0, "Forest", 2);
    let g = t.hand(P0, "Grizzly Bears");
    t.cast(P0, g).go();
    t.resolve_all();
    assert!(!is_creature(&t, tank));
}

#[test]
fn shades_breath_turns_your_creatures_into_shades_with_a_pump() {
    cr!("611.2c", "205.1a", "113.7");
    assert_supported(&["Shade's Breath"]);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    let s = t.hand(P0, "Shade's Breath");
    t.cast(P0, s).go();
    t.resolve();
    // Its creature types are replaced; it's black.
    let o = t.obj_now(bears);
    assert!(o.chars.has_subtype("Shade") && !o.chars.has_subtype("Bear"));
    assert_eq!(o.chars.colors, ColorSet::single(Color::Black));
    assert!(t.obj_now(theirs).chars.has_subtype("Bear"));
    // A creature that enters later isn't affected (CR 611.2c).
    let later = t.battlefield(P0, "Grizzly Bears");
    t.settle();
    assert!(t.obj_now(later).chars.has_subtype("Bear"));
    t.activate(P0, bears, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn veiled_serpent_becomes_a_creature_with_an_attack_restriction() {
    cr!("611.2a", "603.4", "508.1c");
    assert_supported(&["Veiled Serpent"]);
    let mut t = TestGame::new(2);
    let vs = t.battlefield(P0, "Veiled Serpent");
    t.lands(P1, "Mountain", 1);
    let b = t.hand(P1, "Lightning Bolt");
    t.cast(P1, b).target(P0).go();
    t.resolve_all();
    assert!(is_creature(&t, vs));
    assert!(!t.obj_now(vs).chars.card_types.contains(CardType::Enchantment));
    assert_eq!(t.pt(vs), (4, 4));
    // It can't attack a player who controls no Island.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!t.g.can_attack_target(vs, Entity::Player(P1)));
    t.battlefield(P1, "Island");
    t.settle();
    assert!(t.g.can_attack_target(vs, Entity::Player(P1)));
}

#[test]
fn kellan_becomes_a_detective_only_if_its_a_scout() {
    cr!("611.2a", "205.1a", "113.7");
    ruling!("Kellan, Planar Trailblazer", "Kellan's abilities overwrite its existing creature types");
    ruling!("Kellan, Planar Trailblazer", "Neither of these abilities have durations");
    assert_supported(&["Kellan, Planar Trailblazer"]);
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kellan, Planar Trailblazer");
    t.lands(P0, "Mountain", 4);
    t.activate(P0, k, 0, &[]).unwrap();
    t.resolve();
    let o = t.obj_now(k);
    assert!(o.chars.has_subtype("Detective") && !o.chars.has_subtype("Scout"));
    let triggers = |t: &TestGame| {
        t.obj_now(k)
            .chars
            .abilities
            .iter()
            .filter(|a| matches!(a.kind, AbilityKind::Triggered(_)))
            .count()
    };
    assert_eq!(triggers(&t), 1);
    // Activated again: it's no longer a Scout, so nothing happens.
    t.activate(P0, k, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(triggers(&t), 1);
    // No duration: still a Detective next turn.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(k).chars.has_subtype("Detective"));
    // Its combat damage trigger: exile the top card, which may be played this turn.
    t.advance_to(P0, Step::BeginningOfCombat);
    t.library_top(P0, "Grizzly Bears");
    t.attack(&[(k, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn song_of_totentanz_rats_cant_block_and_creatures_gain_haste() {
    cr!("611.2c", "111.1");
    ruling!(
        "Song of Totentanz",
        "Only creatures you control at the time Song of Totentanz resolves"
    );
    assert_supported(&["Song of Totentanz"]);
    let mut t = TestGame::new(2);
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let s = t.hand(P0, "Song of Totentanz");
    t.cast(P0, s).x(2).go();
    t.resolve();
    let rats: Vec<ObjectId> = t
        .g
        .battlefield
        .iter()
        .copied()
        .filter(|id| t.obj_now(*id).chars.has_subtype("Rat"))
        .collect();
    assert_eq!(rats.len(), 2);
    for r in &rats {
        assert!(t.obj_now(*r).has_keyword(KeywordKind::Haste));
    }
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Haste));
    let later = t.battlefield_sick(P0, "Grizzly Bears");
    t.settle();
    assert!(!t.obj_now(later).has_keyword(KeywordKind::Haste));
    // "This token can't block."
    t.set_step(P1, Step::BeginningOfCombat);
    let attacker = t.battlefield(P1, "Grizzly Bears");
    t.attack(&[(attacker, Entity::Player(P0))], &[(rats[0], attacker)]);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn jem_lightfoote_draws_unless_you_cast_a_spell_from_your_hand() {
    cr!("603.4", "601.2a");
    ruling!(
        "Jem Lightfoote, Sky Explorer",
        "will check to see if you’ve cast a spell from your hand this turn"
    );
    assert_supported(&["Jem Lightfoote, Sky Explorer"]);
    for cast_from_hand in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Jem Lightfoote, Sky Explorer");
        for _ in 0..3 {
            t.library_top(P0, "Island");
        }
        t.set_step(P0, Step::PrecombatMain);
        if cast_from_hand {
            t.lands(P0, "Mountain", 1);
            let b = t.hand(P0, "Lightning Bolt");
            t.cast(P0, b).target(P1).go();
            t.resolve();
        }
        let hand = t.hand_size(P0);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        let drew = t.hand_size(P0) - hand;
        assert_eq!(drew, if cast_from_hand { 0 } else { 1 });
    }
}

#[test]
fn genju_of_the_fields_animates_the_enchanted_plains() {
    cr!("303.4a", "611.2a", "113.7");
    ruling!(
        "Genju of the Fields",
        "When activated multiple times, it will have the triggered ability multiple times"
    );
    let mut t = TestGame::new(2);
    let plains = t.battlefield(P0, "Plains");
    let other = t.battlefield(P0, "Plains");
    let genju = t.battlefield(P0, "Genju of the Fields");
    t.g.attach(genju, Entity::Object(plains));
    t.lands(P0, "Mountain", 4);
    t.settle();
    t.activate(P0, genju, 0, &[]).unwrap();
    t.resolve();
    t.activate(P0, genju, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.pt(plains), (2, 5));
    assert!(!is_creature(&t, other), "only the enchanted Plains");
    // (The mana payment may have tapped it.)
    t.g.untap(plains);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(plains, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Two instances of "Whenever this creature deals damage, its controller gains that
    // much life."
    assert_eq!(t.life(P0), 24);
}
