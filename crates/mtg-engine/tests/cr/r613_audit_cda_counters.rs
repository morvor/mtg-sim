//! CR 613 audit: characteristic-defining abilities (CR 604.3, 613.3, 613.4a) against
//! setting effects, counters and switches (CR 613.4b–d), and power/toughness
//! modifications whose values change (CR 613.4c, 611.2d), with real cards.

use crate::r613_audit_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn tarmogoyfs_defining_ability_works_in_every_zone_and_counts_itself() {
    cr!("604.3", "613.4a", "613.1");
    ruling!(
        "Tarmogoyf",
        "If Tarmogoyf is in your graveyard, it will count itself"
    );
    let mut t = TestGame::new(2);
    let in_hand = t.hand(P0, "Tarmogoyf");
    t.g.recompute();
    assert_eq!(t.pt(in_hand), (0, 1), "no cards in graveyards");
    let in_gy = t.graveyard(P0, "Tarmogoyf");
    t.g.recompute();
    assert_eq!(t.pt(in_gy), (1, 2), "a creature card: itself");
    assert_eq!(t.pt(in_hand), (1, 2));
    t.graveyard(P1, "Plains");
    t.graveyard(P1, "Giant Growth");
    t.g.recompute();
    assert_eq!(t.pt(in_gy), (3, 4));
    let on_bf = t.battlefield(P0, "Tarmogoyf");
    t.g.recompute();
    assert_eq!(t.pt(on_bf), (3, 4));
}

#[test]
fn a_setting_effect_overrides_tarmogoyfs_defining_ability_but_counters_still_apply() {
    // Layer 7a (the defining ability) is overridden by the later sublayer 7b (Ovinize)
    // whatever the timestamps; counters (7c) apply on top of either.
    cr!("613.4a", "613.4b", "613.4c", "604.3", "611.2a");
    ruling!(
        "Ovinize",
        "The same is true for counters that change the creature’s power and/or toughness"
    );
    let mut t = TestGame::new(2);
    let goyf = t.battlefield(P0, "Tarmogoyf");
    t.graveyard(P1, "Plains");
    t.graveyard(P1, "Grizzly Bears");
    put_counters(&mut t, goyf, counters::PLUS1, 1);
    assert_eq!(t.pt(goyf), (3, 4));
    cast_resolve(&mut t, P1, "Ovinize", &[Entity::Object(goyf)]);
    t.g.recompute();
    assert_eq!(t.pt(goyf), (1, 2), "0/1 plus the counter");
    assert_eq!(ability_count(&t, goyf), 0);
    // More card types in graveyards don't matter while Ovinize applies.
    t.graveyard(P1, "Giant Growth");
    t.g.recompute();
    assert_eq!(t.pt(goyf), (1, 2));
    end_turn(&mut t);
    assert_eq!(t.pt(goyf), (4, 5), "three types plus the counter");
    assert_eq!(ability_count(&t, goyf), 1);
}

#[test]
fn losing_all_abilities_removes_a_defining_ability_so_a_star_counts_as_zero() {
    // Dress Down removes the defining ability in layer 6, so it doesn't apply in layer
    // 7a: Tarmogoyf's */1+* is 0/1 (CR 208.5) and Nightmare's */* is 0/0 and dies.
    cr!("613.1f", "613.4a", "604.3", "208.2a", "208.5", "704.5f");
    interp!("star-without-defining-ability-is-zero");
    let mut t = TestGame::new(2);
    let goyf = t.battlefield(P0, "Tarmogoyf");
    let nightmare = t.battlefield(P0, "Nightmare");
    t.lands(P0, "Swamp", 3);
    t.graveyard(P1, "Plains");
    t.graveyard(P1, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(goyf), (2, 3));
    assert_eq!(t.pt(nightmare), (3, 3));
    t.battlefield(P1, "Dress Down");
    t.settle();
    t.g.recompute();
    assert_eq!(ability_count(&t, goyf), 0);
    assert_eq!(t.pt(goyf), (0, 1));
    assert!(t.on_battlefield(goyf));
    assert!(!t.on_battlefield(nightmare), "a 0/0 without its defining ability");
}

#[test]
fn godhead_of_awe_against_counters_and_earlier_and_later_setting_effects() {
    cr!("613.4b", "613.4c", "613.4d", "613.7", "613.7b");
    ruling!(
        "Godhead of Awe",
        "It will not overwrite effects that modify power or toughness (whether from a static ability, counters, or a resolved spell or ability), nor will it overwrite effects that set power and toughness which come into existence after the Godhead enters"
    );
    let mut t = TestGame::new(2);
    let spider = t.battlefield(P0, "Giant Spider");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let queen = t.battlefield(P1, "Sorceress Queen");
    // An earlier setting effect: the Spider has base 0/2 this turn.
    activate_containing(&mut t, P1, queen, "base power", &[Entity::Object(spider)]);
    put_counters(&mut t, bears, counters::PLUS1, 1);
    cast_resolve(&mut t, P0, "Giant Growth", &[Entity::Object(bears)]);
    t.g.recompute();
    assert_eq!(t.pt(spider), (0, 2));
    assert_eq!(t.pt(bears), (6, 6));
    let godhead = t.battlefield(P1, "Godhead of Awe");
    t.g.recompute();
    assert_eq!(t.pt(spider), (1, 1), "Godhead is later than the Queen's effect");
    assert_eq!(t.pt(bears), (5, 5), "1/1 with the counter and Giant Growth");
    assert_eq!(t.pt(godhead), (4, 4), "\"other creatures\"");
    assert!(has_kw(&t, spider, KeywordKind::Reach), "Godhead removes no abilities");
    // A later setting effect overrides the Godhead.
    cast_resolve(&mut t, P1, "Ovinize", &[Entity::Object(bears)]);
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 5), "0/1 with the counter and Giant Growth");
    // A switch applies after everything in layer 7.
    cast_resolve(&mut t, P1, "Inside Out", &[Entity::Object(bears)]);
    t.g.recompute();
    assert_eq!(t.pt(bears), (5, 4));
    // When the turn ends, the Godhead's 1/1 is all that's left (and the counter).
    end_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(spider), (1, 1));
}

#[test]
fn darksteel_mutation_against_counters_a_pump_and_a_later_transformation() {
    cr!("613.4b", "613.4c", "613.1d", "613.1f", "613.7e", "613.7");
    ruling!(
        "Darksteel Mutation",
        "It also won't overwrite the effect of counters"
    );
    ruling!(
        "Darksteel Mutation",
        "Any such effects that start to apply after Darksteel Mutation entered the battlefield will work normally"
    );
    ruling!(
        "Kenrith's Transformation",
        "loses any other card types it has (such as artifact)"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, bears, counters::PLUS1, 1);
    cast_resolve(&mut t, P0, "Giant Growth", &[Entity::Object(bears)]);
    let mutation = t.battlefield(P1, "Darksteel Mutation");
    assert!(t.g.attach(mutation, Entity::Object(bears)));
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 5), "0/1 with the counter and Giant Growth");
    assert!(has_kw(&t, bears, KeywordKind::Indestructible));
    assert_eq!(ability_count(&t, bears), 1);
    assert!(is_type(&t, bears, CardType::Artifact));
    assert!(has_subtype(&t, bears, "Insect"));
    assert!(!has_subtype(&t, bears, "Bear"));
    assert_eq!(colors(&t, bears), color_set(&[Color::Green]), "colors are untouched");
    // A later Kenrith's Transformation: a green Elk creature with base 3/3, no abilities
    // (the indestructible granted earlier included), no longer an artifact or an Insect.
    let elk = t.battlefield(P1, "Kenrith's Transformation");
    assert!(t.g.attach(elk, Entity::Object(bears)));
    t.g.recompute();
    assert_eq!(t.pt(bears), (7, 7), "3/3 with the counter and Giant Growth");
    assert!(!has_kw(&t, bears, KeywordKind::Indestructible));
    assert_eq!(ability_count(&t, bears), 0);
    assert!(!is_type(&t, bears, CardType::Artifact));
    assert!(has_subtype(&t, bears, "Elk"));
    assert!(!has_subtype(&t, bears, "Insect"));
    assert_eq!(colors(&t, bears), color_set(&[Color::Green]));
    // Re-attaching Darksteel Mutation makes it the later one again (CR 613.7e).
    let other = t.battlefield(P0, "Hill Giant");
    assert!(t.g.attach(mutation, Entity::Object(other)));
    assert!(t.g.attach(mutation, Entity::Object(bears)));
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 5));
    assert!(has_kw(&t, bears, KeywordKind::Indestructible));
    assert!(is_type(&t, bears, CardType::Artifact));
    assert!(has_subtype(&t, bears, "Insect"));
    assert!(!has_subtype(&t, bears, "Elk"));
}

#[test]
fn mirror_entity_with_x_zero_leaves_only_creatures_with_counters_alive() {
    cr!("613.4b", "613.4c", "613.1d", "704.5f");
    ruling!(
        "Mirror Entity",
        "Activating the ability with X = 0 will cause all creatures you control to become 0/0"
    );
    ruling!(
        "Mirror Entity",
        "Any existing effects or counters that raise or lower a creature's power and/or toughness continue to apply to the creature's newly-set power and toughness"
    );
    let mut t = TestGame::new(2);
    let entity = t.battlefield(P0, "Mirror Entity");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let pumped = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    put_counters(&mut t, bears, counters::PLUS1, 1);
    cast_resolve(&mut t, P0, "Giant Growth", &[Entity::Object(pumped)]);
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    t.activate(P0, entity, 0, &[]).expect("activation");
    t.resolve_all();
    t.g.recompute();
    assert!(!t.on_battlefield(elves));
    assert!(!t.on_battlefield(entity));
    assert!(t.on_battlefield(bears));
    assert_eq!(t.pt(bears), (1, 1));
    assert!(has_subtype(&t, bears, "Goblin") && has_subtype(&t, bears, "Bear"));
    assert!(t.on_battlefield(pumped));
    assert_eq!(t.pt(pumped), (3, 3), "0/0 and Giant Growth");
    assert_eq!(t.pt(theirs), (2, 2), "an opponent's creature is unaffected");
    assert!(!has_subtype(&t, theirs, "Goblin"));
}

#[test]
fn a_switch_applies_after_a_later_setting_effect_and_after_counters() {
    // Sublayer 7d applies after 7b and 7c whatever the timestamps (CR 613.4): the switch
    // cast first still switches the values set and modified later.
    cr!("613.4", "613.4b", "613.4c", "613.4d");
    ruling!(
        "Godhead of Awe",
        "Effects that switch the creature's power and toughness are always applied after any other power or toughness changing effects, including that of the Godhead"
    );
    let mut t = TestGame::new(2);
    let spider = t.battlefield(P0, "Giant Spider");
    cast_resolve(&mut t, P1, "Inside Out", &[Entity::Object(spider)]);
    t.g.recompute();
    assert_eq!(t.pt(spider), (4, 2));
    t.battlefield(P1, "Godhead of Awe");
    t.g.recompute();
    assert_eq!(t.pt(spider), (1, 1));
    let rancor = t.battlefield(P0, "Rancor");
    assert!(t.g.attach(rancor, Entity::Object(spider)));
    t.g.recompute();
    assert_eq!(t.pt(spider), (1, 3), "1/1, +2/+0, then switched");
    put_counters(&mut t, spider, counters::PLUS1, 1);
    assert_eq!(t.pt(spider), (2, 4), "unswitched 4/2");
    end_turn(&mut t);
    assert_eq!(t.pt(spider), (4, 2), "the switch ended");
}

#[test]
fn a_static_plus_x_follows_x_while_a_resolved_one_is_locked() {
    // Nighthowler's "+X/+X, where X is the number of creature cards in all graveyards"
    // is a static ability's effect, reevaluated continuously (CR 611.3a, 613.4c); Might
    // of the Masses' X is determined once, on resolution (CR 611.2d, 608.2h).
    cr!("613.4c", "611.3a", "611.2d", "608.2h", "704.5f");
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Grizzly Bears");
    let howler = t.battlefield(P0, "Nighthowler");
    t.g.recompute();
    assert_eq!(t.pt(howler), (1, 1));
    t.graveyard(P0, "Llanowar Elves");
    t.graveyard(P0, "Giant Growth");
    t.g.recompute();
    assert_eq!(t.pt(howler), (2, 2), "two creature cards; the instant doesn't count");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Might of the Masses", &[Entity::Object(bears)]);
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4), "two creatures as it resolved");
    t.battlefield(P0, "Hill Giant");
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4), "X is locked");
    // Exiling the creature cards from the graveyards shrinks Nighthowler to 0/0.
    for p in [P0, P1] {
        for id in t.g.players[p.idx()].graveyard.clone() {
            if t.g.obj(id).chars.is_creature() {
                t.g.move_object(id, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
            }
        }
    }
    t.settle();
    t.g.recompute();
    assert!(!t.on_battlefield(howler), "0/0");
}

#[test]
fn master_of_etherium_counts_artifacts_after_mycosynth_lattice_changed_types() {
    // Layer 4 (the Lattice) is applied before the defining ability (7a) and the anthem
    // (7c) are evaluated.
    cr!("613.1d", "613.4a", "613.4c", "604.3", "613.1");
    ruling!(
        "Mycosynth Lattice",
        "Mycosynth Lattice turns all permanents on the battlefield into artifacts"
    );
    let mut t = TestGame::new(2);
    let master = t.battlefield(P0, "Master of Etherium");
    let stone = t.battlefield(P0, "Mind Stone");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(master), (2, 2), "itself and Mind Stone");
    assert_eq!(t.pt(bears), (2, 2));
    t.battlefield(P1, "Mycosynth Lattice");
    t.g.recompute();
    assert_eq!(t.pt(master), (3, 3), "the Bears is an artifact too");
    assert_eq!(t.pt(bears), (3, 3), "an artifact creature now: +1/+1");
    assert!(is_type(&t, stone, CardType::Artifact));
    assert_eq!(colors(&t, bears), ColorSet::NONE);
}

#[test]
fn transguild_couriers_color_defining_ability_applies_first_in_layer_5() {
    // CR 613.3: the defining ability applies before other layer 5 effects, whatever the
    // timestamps, so an older "all permanents are colorless" and an older "becomes blue"
    // both override it.
    cr!("613.3", "613.1e", "604.3", "613.7");
    ruling!(
        "Transguild Courier",
        "This card is white, blue, black, red, and green in all zones"
    );
    ruling!(
        "Painter's Servant",
        "an effect that changes an object's colors will overwrite Painter's Servant's effect"
    );
    let mut t = TestGame::new(2);
    let lattice = t.battlefield(P1, "Mycosynth Lattice");
    let courier = t.battlefield(P0, "Transguild Courier");
    let in_hand = t.hand(P0, "Transguild Courier");
    t.g.recompute();
    assert_eq!(colors(&t, courier), ColorSet::NONE, "the Lattice's older effect still wins");
    assert_eq!(colors(&t, in_hand), ColorSet::NONE, "cards not on the battlefield too");
    t.g.destroy(lattice, None);
    t.settle();
    t.g.recompute();
    assert_eq!(colors(&t, courier), ColorSet::ALL);
    assert_eq!(colors(&t, in_hand), ColorSet::ALL);
    // Painter's Servant adds a color it already has; Cerulean Wisps overrides the
    // Servant and the defining ability.
    choose_color(&mut t, P1, Color::Red);
    t.enter(P1, "Painter's Servant");
    t.resolve_all();
    t.g.recompute();
    assert_eq!(colors(&t, courier), ColorSet::ALL);
    cast_resolve(&mut t, P1, "Cerulean Wisps", &[Entity::Object(courier)]);
    t.g.recompute();
    assert_eq!(colors(&t, courier), color_set(&[Color::Blue]));
    end_turn(&mut t);
    assert_eq!(colors(&t, courier), ColorSet::ALL);
}

#[test]
fn muraganda_petroglyphs_counts_abilities_as_they_stand_after_layer_6() {
    cr!("613.1", "613.1f", "613.4c", "305.6", "613.9");
    ruling!(
        "Muraganda Petroglyphs",
        "If a land with a basic land type becomes a creature, the intrinsic mana ability it has stops Muraganda Petroglyphs from applying to it"
    );
    ruling!(
        "Humility",
        "Animated lands will also lose the ability to tap for mana"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Muraganda Petroglyphs");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let angel = t.battlefield(P0, "Serra Angel");
    let forest = t.battlefield(P0, "Forest");
    t.battlefield(P0, "Life and Limb");
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.pt(angel), (4, 4));
    assert!(t.obj_now(forest).is_creature());
    assert_eq!(t.pt(forest), (1, 1), "its intrinsic mana ability is an ability");
    // Turn to Frog: a 1/1 with no abilities gets +2/+2.
    cast_resolve(&mut t, P1, "Turn to Frog", &[Entity::Object(angel)]);
    t.g.recompute();
    assert_eq!(t.pt(angel), (3, 3));
    // A keyword counter is an ability.
    put_counters(&mut t, bears, "flying", 1);
    assert_eq!(t.pt(bears), (2, 2));
    // Humility: every creature has no abilities (the Forest's mana ability included).
    t.battlefield(P1, "Humility");
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 3));
    assert_eq!(t.pt(angel), (3, 3));
    assert_eq!(t.pt(forest), (3, 3));
    assert!(!has_ability_text(&t, forest, "Add {G}"));
}

#[test]
fn counters_stay_and_a_pump_resumes_when_an_artifact_stops_and_restarts_being_a_creature() {
    // Giant Growth's effect is locked to the object (CR 611.2c) and keeps applying when
    // it's a creature again; a noncreature has no power or toughness (CR 208.3) but keeps
    // its counters.
    cr!("611.2c", "613.4c", "613.4b", "208.3", "208.3a");
    let mut t = TestGame::new(2);
    let citadel = t.battlefield(P0, "Darksteel Citadel");
    let ensoul = t.battlefield(P0, "Ensoul Artifact");
    assert!(t.g.attach(ensoul, Entity::Object(citadel)));
    put_counters(&mut t, citadel, counters::PLUS1, 1);
    cast_resolve(&mut t, P0, "Giant Growth", &[Entity::Object(citadel)]);
    t.g.recompute();
    assert!(t.obj_now(citadel).is_creature());
    assert_eq!(t.pt(citadel), (9, 9));
    cast_resolve(&mut t, P0, "Disenchant", &[Entity::Object(ensoul)]);
    t.g.recompute();
    assert!(!t.obj_now(citadel).is_creature());
    assert!(t.obj_now(citadel).chars.power.is_none());
    assert!(t.obj_now(citadel).chars.toughness.is_none());
    assert_eq!(t.counters(citadel, counters::PLUS1), 1);
    assert!(is_type(&t, citadel, CardType::Land));
    let ensoul2 = t.battlefield(P0, "Ensoul Artifact");
    assert!(t.g.attach(ensoul2, Entity::Object(citadel)));
    t.g.recompute();
    assert_eq!(t.pt(citadel), (9, 9), "5/5, the counter, and Giant Growth again");
    end_turn(&mut t);
    assert_eq!(t.pt(citadel), (6, 6));
}
