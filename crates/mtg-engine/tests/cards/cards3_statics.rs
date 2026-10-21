//! Characteristic statics IV (CR 702.11d, 702.16): hexproof from colors for players (Veil
//! of Summer), protection from each color / mana value among objects (Pledge of Loyalty,
//! Rebbec), protection from each mana value but the chosen one (Haktos the Unscarred).

use crate::combat2_common::supported;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn veil_of_summer_gives_you_and_your_permanents_hexproof_from_blue_and_black() {
    cr!("702.11d", "702.11f");
    supported("Veil of Summer");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let black = t.battlefield(P1, "Hypnotic Specter");
    let blue = t.battlefield(P1, "Wind Drake");
    let red = t.battlefield(P1, "Goblin Piker");
    let veil = t.hand(P0, "Veil of Summer");
    t.lands(P0, "Forest", 1);
    t.set_step(P0, Step::PrecombatMain);
    assert!(!t.g.player_untargetable(P0, P1, Some(black)));
    t.cast(P0, veil).go();
    t.resolve_all();
    t.g.recompute();
    assert!(t.g.player_untargetable(P0, P1, Some(black)));
    assert!(t.g.player_untargetable(P0, P1, Some(blue)));
    assert!(!t.g.player_untargetable(P0, P1, Some(red)));
    // P0's own black or blue sources can still target P0.
    let mine = t.battlefield(P0, "Hypnotic Specter");
    assert!(!t.g.player_untargetable(P0, P0, Some(mine)));
    assert!(t.g.object_untargetable(bears, P1, Some(black)));
    assert!(!t.g.object_untargetable(bears, P1, Some(red)));
    // Until end of turn.
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.g.player_untargetable(P0, P1, Some(black)));
}

#[test]
fn rebbec_artifacts_have_protection_from_each_mana_value_among_artifacts() {
    cr!("702.16a", "702.16h");
    supported("Rebbec, Architect of Ascension");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rebbec, Architect of Ascension");
    let ring = t.battlefield(P0, "Sol Ring");
    let lions = t.battlefield(P1, "Savannah Lions");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let rebbec_mv4 = t.battlefield(P1, "Hill Giant");
    t.g.recompute();
    // Artifacts: Sol Ring (1).
    assert!(t.g.protected_from(ring, lions));
    assert!(!t.g.protected_from(ring, bears));
    // An artifact with mana value 2 adds that quality.
    t.battlefield(P0, "Mind Stone");
    t.g.recompute();
    assert!(t.g.protected_from(ring, bears));
    assert!(!t.g.protected_from(ring, rebbec_mv4));
    // Rebbec isn't an artifact: no protection.
    let r = t.named_on_battlefield("Rebbec, Architect of Ascension")[0];
    assert!(!t.g.protected_from(r, lions));
}

#[test]
fn pledge_of_loyalty_protection_from_your_colors_without_removing_itself() {
    cr!("702.16c", "702.16h", "702.16n");
    supported("Pledge of Loyalty");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let pledge = t.battlefield(P0, "Pledge of Loyalty");
    t.g.attach(pledge, Entity::Object(bears));
    let red = t.battlefield(P1, "Goblin Piker");
    let white = t.battlefield(P1, "Savannah Lions");
    let blue = t.battlefield(P1, "Wind Drake");
    t.settle();
    // P0 controls green (bears) and white (Pledge) permanents.
    assert!(t.g.protected_from(bears, white));
    assert!(!t.g.protected_from(bears, red));
    assert_eq!(t.g.obj(pledge).attached_to, Some(Entity::Object(bears)));
    // A red permanent of P0's adds red.
    t.battlefield(P0, "Goblin Piker");
    t.settle();
    assert!(t.g.protected_from(bears, red));
    assert!(!t.g.protected_from(bears, blue));
    assert!(t.on_battlefield(pledge));
}

#[test]
fn pledge_of_loyalty_on_an_opponents_creature_uses_your_permanents() {
    cr!("702.16h");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Wind Drake");
    let pledge = t.battlefield(P0, "Pledge of Loyalty");
    t.g.attach(pledge, Entity::Object(theirs));
    // P0 controls white (Pledge), blue and green permanents; P1 controls blue and red.
    let blue = t.battlefield(P0, "Wind Drake");
    let green = t.battlefield(P0, "Grizzly Bears");
    let piker = t.battlefield(P1, "Goblin Piker");
    t.settle();
    assert!(t.g.protected_from(theirs, blue));
    assert!(t.g.protected_from(theirs, green));
    assert!(!t.g.protected_from(theirs, piker));
}

#[test]
fn haktos_has_protection_from_every_mana_value_but_the_chosen_one() {
    cr!("702.16a");
    ruling!(
        "Haktos the Unscarred",
        "protection from each possible mana value other than the chosen value"
    );
    supported("Haktos the Unscarred");
    let mut t = TestGame::new(2);
    let haktos = t.enter(P0, "Haktos the Unscarred");
    let mvs: Vec<ObjectId> = ["Savannah Lions", "Grizzly Bears", "Gray Ogre", "Hill Giant", "Craw Wurm"]
        .iter()
        .map(|n| t.battlefield(P1, n))
        .collect();
    t.g.recompute();
    let unprotected: Vec<usize> = (0..mvs.len())
        .filter(|i| !t.g.protected_from(haktos, mvs[*i]))
        .collect();
    assert_eq!(unprotected.len(), 1, "{unprotected:?}");
    assert!((1..=3).contains(&unprotected[0]));
}

#[test]
fn eon_frolicker_gives_you_and_your_planeswalkers_protection_from_that_player() {
    cr!("702.16k");
    ruling!(
        "Eon Frolicker",
        "Protection from a player means that you and each planeswalker you control have protection from each object controlled by that player"
    );
    supported("Eon Frolicker");
    let mut t = TestGame::with_config(3, Default::default());
    let otter = t.hand(P0, "Eon Frolicker");
    let pw = t.battlefield(P0, "Ajani Goldmane");
    let theirs = t.battlefield(P1, "Goblin Piker");
    let other = t.battlefield(P2, "Goblin Piker");
    t.lands(P0, "Island", 4);
    t.set_step(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, otter).go();
    t.resolve_all();
    t.g.recompute();
    assert!(t.g.player_untargetable(P0, P1, Some(theirs)));
    assert!(!t.g.player_untargetable(P0, P2, Some(other)));
    assert!(t.g.protected_from(pw, theirs));
    assert!(!t.g.protected_from(pw, other));
}

#[test]
fn excavator_grants_landwalk_of_the_sacrificed_lands_types() {
    cr!("702.14a", "702.14c");
    ruling!("Excavator", "will gain landwalk of all those types");
    supported("Excavator");
    let mut t = TestGame::new(2);
    let ex = t.battlefield(P0, "Excavator");
    t.battlefield(P0, "Island");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, ex, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Island"));
    crate::combat2_common::declare(&mut t, P0, &[(bears, Entity::Player(P1))]);
    assert!(crate::combat2_common::legal_blocks(&mut t, &[(blocker, bears)]));
    t.battlefield(P1, "Island");
    assert!(!crate::combat2_common::legal_blocks(&mut t, &[(blocker, bears)]));
}

/// The index of `word` among the options of the first "choose one of" instruction of
/// `src`'s first activated ability.
fn choice_index(t: &TestGame, src: ObjectId, word: &str) -> usize {
    fn find(e: &ability::Effect, word: &str) -> Option<usize> {
        match e {
            ability::Effect::Seq(v) => v.iter().find_map(|x| find(x, word)),
            ability::Effect::Choose {
                kind: ability::ChoiceKind::OneOf(words),
                ..
            } => words.iter().position(|w| w == word),
            _ => None,
        }
    }
    t.g.obj(src)
        .chars
        .abilities
        .iter()
        .find_map(|a| match &a.kind {
            ability::AbilityKind::Activated(act) => find(&act.body.effect, word),
            _ => None,
        })
        .expect("no such choice")
}

#[test]
fn barbarian_guides_snow_landwalk_of_the_chosen_type() {
    cr!("702.14c");
    ruling!(
        "Barbarian Guides",
        "if the defending player controls any snow lands with that subtype"
    );
    supported("Barbarian Guides");
    let mut t = TestGame::new(2);
    let guides = t.battlefield(P0, "Barbarian Guides");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let blocker = t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Swamp");
    t.set_step(P0, Step::PrecombatMain);
    let i = choice_index(&t, guides, "Swamp");
    t.answer(P0, DecisionKind::Option, mtg_engine::decision::Answer::Index(i));
    t.activate(P0, guides, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    crate::combat2_common::declare(&mut t, P0, &[(bears, Entity::Player(P1))]);
    // A Swamp that isn't snow doesn't count.
    assert!(crate::combat2_common::legal_blocks(&mut t, &[(blocker, bears)]));
    t.battlefield(P1, "Snow-Covered Swamp");
    assert!(!crate::combat2_common::legal_blocks(&mut t, &[(blocker, bears)]));
    // The creature returns to its owner's hand at the beginning of the next end step.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn teferi_phased_out_creatures_stay_out_until_after_your_next_turn() {
    cr!("702.26a");
    ruling!(
        "Teferi, Timeless Voyager",
        "they'll phase in during their controller's next untap step after your next turn"
    );
    supported("Teferi, Timeless Voyager");
    let mut t = TestGame::new(2);
    let teferi = t.battlefield(P0, "Teferi, Timeless Voyager");
    t.g.objects[teferi.0 as usize]
        .counters
        .insert("loyalty".into(), 8);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, teferi, 2, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(t.g.obj(bears).phased_out);
    // P1's next untap step: still phased out.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.g.obj(t.g.current(bears)).phased_out);
    // After P0's next turn, P1's following untap step phases it in.
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.g.obj(t.g.current(bears)).phased_out);
}

#[test]
fn viral_spawning_has_flashback_only_while_an_opponent_is_corrupted() {
    cr!("113.6b", "702.34a");
    supported("Viral Spawning");
    use mtg_engine::object::CastMethod;
    use mtg_engine::keywords::KeywordKind;
    let mut t = TestGame::new(2);
    let spawn = t.graveyard(P0, "Viral Spawning");
    t.lands(P0, "Forest", 3);
    t.set_step(P0, Step::PrecombatMain);
    t.g.recompute();
    assert!(t
        .cast(P0, spawn)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .try_go()
        .is_err());
    t.g.player_mut(P1).counters.insert("poison".into(), 3);
    t.g.recompute();
    t.cast(P0, spawn)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .go();
    t.resolve_all();
    assert_eq!(
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.chars.has_subtype("Beast"))
            .count(),
        1
    );
    assert!(t.in_exile("Viral Spawning"));
}

#[test]
fn spark_rupture_turns_planeswalkers_into_creatures_sized_by_loyalty() {
    cr!("613.1d", "613.4b");
    ruling!(
        "Spark Rupture",
        "A planeswalker that becomes a creature due to Spark Rupture is no longer a planeswalker"
    );
    ruling!(
        "Spark Rupture",
        "The resulting creature's power and toughness will change if the number of loyalty counters on it changes"
    );
    supported("Spark Rupture");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spark Rupture");
    let pw = t.battlefield(P1, "Ajani Goldmane");
    t.g.objects[pw.0 as usize]
        .counters
        .insert("loyalty".into(), 4);
    t.settle();
    let o = t.g.obj(pw);
    assert!(o.is_creature());
    assert!(!o.chars.is(mtg_engine::types::CardType::Planeswalker));
    assert!(o.chars.abilities.is_empty());
    assert_eq!(t.pt(pw), (4, 4));
    t.g.objects[pw.0 as usize]
        .counters
        .insert("loyalty".into(), 2);
    t.g.dirty = true;
    t.settle();
    assert_eq!(t.pt(pw), (2, 2));
}

#[test]
fn bello_animates_big_non_equipment_artifacts_and_non_aura_enchantments_on_your_turn() {
    cr!("205.1b", "613.1d");
    supported("Bello, Bard of the Brambles");
    use mtg_engine::keywords::KeywordKind;
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bello, Bard of the Brambles");
    let lotus = t.battlefield(P0, "Gilded Lotus");
    let sphere = t.battlefield(P0, "Sphere of Safety");
    let skull = t.battlefield(P0, "Batterskull");
    let ring = t.battlefield(P0, "Sol Ring");
    let theirs = t.battlefield(P1, "Gilded Lotus");
    t.set_step(P0, Step::PrecombatMain);
    t.g.recompute();
    for o in [lotus, sphere] {
        assert!(t.g.obj(o).is_creature());
        assert_eq!(t.pt(o), (4, 4));
        assert!(t.g.obj(o).chars.has_subtype("Elemental"));
        assert!(t.g.obj(o).has_keyword(KeywordKind::Indestructible));
        assert!(t.g.obj(o).has_keyword(KeywordKind::Haste));
    }
    for o in [skull, ring, theirs] {
        assert!(!t.g.obj(o).is_creature(), "{:?}", t.g.obj(o).chars.name);
    }
    // Not during an opponent's turn.
    t.advance_to(P1, Step::Upkeep);
    t.g.recompute();
    assert!(!t.g.obj(lotus).is_creature());
}

#[test]
fn harold_and_bob_return_as_an_aura_on_a_forest_with_only_the_granted_ability() {
    cr!("303.4f", "613.1f");
    supported("Harold and Bob, First Numens");
    use mtg_engine::keywords::KeywordKind;
    let mut t = TestGame::new(2);
    let hb = t.battlefield(P0, "Harold and Bob, First Numens");
    let forest = t.battlefield(P0, "Forest");
    let before = t.g.obj(forest).chars.abilities.len();
    t.g.destroy(hb, None);
    t.settle();
    t.resolve_all();
    let now = t.named_on_battlefield("Harold and Bob, First Numens");
    assert_eq!(now.len(), 1);
    let o = t.g.obj(now[0]);
    assert!(!o.is_creature());
    assert!(o.chars.has_subtype("Aura"));
    assert!(!o.has_keyword(KeywordKind::Reach));
    assert!(!o.has_keyword(KeywordKind::Vigilance));
    assert_eq!(o.attached_to, Some(Entity::Object(forest)));
    assert_eq!(t.g.obj(forest).chars.abilities.len(), before + 1);
}

#[test]
fn old_growth_troll_returns_as_an_aura_whose_forest_can_make_a_troll() {
    cr!("303.4f", "613.1f");
    supported("Old-Growth Troll");
    let mut t = TestGame::new(2);
    let troll = t.battlefield(P0, "Old-Growth Troll");
    let forest = t.battlefield(P0, "Forest");
    t.battlefield(P0, "Mountain");
    t.g.destroy(troll, None);
    t.settle();
    t.resolve_all();
    let aura = t.named_on_battlefield("Old-Growth Troll");
    assert_eq!(aura.len(), 1);
    assert!(!t.g.obj(aura[0]).is_creature());
    assert_eq!(t.g.obj(aura[0]).attached_to, Some(Entity::Object(forest)));
    t.set_step(P0, Step::PrecombatMain);
    // The Forest's own mana ability, "{T}: Add {G}{G}", then the Troll-making ability.
    t.activate(P0, forest, 2, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forest"));
    let tokens: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|o| o.chars.has_subtype("Warrior") && o.chars.has_subtype("Troll"))
        .map(|o| o.id)
        .collect();
    assert_eq!(tokens.len(), 1);
    assert!(t.g.obj(tokens[0]).tapped);
    assert_eq!(t.pt(tokens[0]), (4, 4));
    // The Aura, attached to nothing, is put into the graveyard.
    assert!(t.in_graveyard(P0, "Old-Growth Troll"));
}
