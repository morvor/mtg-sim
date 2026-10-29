//! Structure-coverage tests (round 8): spell-ability structures shared by many cards that
//! no other test exercised (see `docs/STRUCTURE_COVERAGE.md`). Each test checks that a
//! real card's spell does what its text says.

use super::structure_activated_2::{next_turn, shuffled, target_choices};
use super::structure_activated_6::entity_choices;
use super::structure_spell_6::hand_names;
use super::structure_seed_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

#[test]
fn unnatural_speed_grants_haste_until_end_of_turn() {
    cr!("608.2", "611.2a", "514.2", "702.10b", "302.6");
    // "Target creature gains haste until end of turn."
    supported("Unnatural Speed");
    let mut t = TestGame::new(2);
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    let other = t.battlefield_sick(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let s = t.hand(P0, "Unnatural Speed");
    t.cast(P0, s).target(bears).go();
    t.resolve_all();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Haste));
    assert!(!t.obj_now(other).has_keyword(KeywordKind::Haste));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    let options = t
        .asked()
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::DeclareAttackers { options } => Some(options),
            _ => None,
        })
        .unwrap();
    assert!(options.iter().any(|(c, _)| *c == bears));
    assert!(!options.iter().any(|(c, _)| *c == other), "summoning sick");
    assert_eq!(t.life(P1), 18, "the hasty creature attacked");
    next_turn(&mut t);
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Haste));
}

#[test]
fn expel_exiles_only_a_tapped_creature() {
    cr!("608.2", "115.1", "701.13a", "608.2b");
    // "Exile target tapped creature."
    supported("Expel");
    let mut t = TestGame::new(2);
    let tapped = t.battlefield(P1, "Grizzly Bears");
    let untapped = t.battlefield(P1, "Hill Giant");
    let tapped_land = t.battlefield(P1, "Forest");
    t.g.tap(tapped);
    t.g.tap(tapped_land);
    t.lands(P0, "Plains", 3);
    let s = t.hand(P0, "Expel");
    t.cast(P0, s).target(tapped).go();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(tapped)));
    assert!(!c.contains(&Entity::Object(untapped)), "untapped");
    assert!(!c.contains(&Entity::Object(tapped_land)), "not a creature");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.on_battlefield(untapped));
    // A target that untapped before resolution is illegal: the spell does nothing.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    t.lands(P0, "Plains", 3);
    let s = t.hand(P0, "Expel");
    t.cast(P0, s).target(bears).go();
    t.g.untap(bears);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Expel"));
}

#[test]
fn romantic_rendezvous_discards_then_draws_two() {
    cr!("608.2", "608.2c", "701.9a", "121.1");
    // "Discard a card, then draw two cards."
    supported("Romantic Rendezvous");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Forest");
    t.library_top(P0, "Island");
    let keep = t.hand(P0, "Grizzly Bears");
    let toss = t.hand(P0, "Hill Giant");
    t.lands(P0, "Mountain", 2);
    let s = t.hand(P0, "Romantic Rendezvous");
    t.answer_choose(P0, &[Entity::Object(toss)]);
    t.cast(P0, s).go();
    t.resolve_all();
    // Discard first, then draw: only the two cards already in hand are offered.
    let offered = entity_choices(&t, P0);
    assert_eq!(offered.len(), 1, "the caster chooses the discard");
    assert_eq!(offered[0].len(), 2);
    assert!(offered[0].contains(&Entity::Object(keep)) && offered[0].contains(&Entity::Object(toss)));
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.zone(keep), Zone::Hand(P0));
    let mut h = hand_names(&t, P0);
    h.sort();
    assert_eq!(h, vec!["Forest", "Grizzly Bears", "Island"]);
    assert_eq!(t.hand_size(P1), 0);
    // With an empty hand there's nothing to discard, but the draws still happen.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let s = t.hand(P0, "Romantic Rendezvous");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.graveyard_size(P0), 1, "only the spell");
}

#[test]
fn reclaim_puts_a_card_from_your_graveyard_on_top_of_your_library() {
    cr!("608.2", "115.1");
    // "Put target card from your graveyard on top of your library."
    supported("Reclaim");
    let mut t = TestGame::new(2);
    let mine = t.graveyard(P0, "Hill Giant");
    let other = t.graveyard(P0, "Forest");
    let theirs = t.graveyard(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let s = t.hand(P0, "Reclaim");
    t.cast(P0, s).target(mine).go();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(mine)) && c.contains(&Entity::Object(other)));
    assert!(!c.contains(&Entity::Object(theirs)), "not your graveyard");
    t.resolve_all();
    let top = t.g.library_top(P0).unwrap();
    assert_eq!(t.obj(top).chars.name, "Hill Giant");
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn peer_review_creates_a_named_cadet_and_surveils() {
    cr!("608.2", "111.1", "111.4", "701.25a", "722.3c");
    // Prudent Fateseer // Peer Review: "Create a 2/2 colorless Wizard Soldier creature
    // token named Cadet. Surveil 1."
    const CARD: &str = "Prudent Fateseer // Peer Review";
    supported(CARD);
    let mut t = TestGame::new(2);
    let f = t.enter(P0, CARD);
    let f = t.g.current(f);
    let copy = t.obj_now(f).prepared.expect("enters prepared");
    let top = t.library_top(P0, "Forest");
    t.lands(P0, "Plains", 3);
    t.answer(P0, DecisionKind::Surveil, Answer::Split(vec![], vec![top]));
    t.cast(P0, copy).go();
    t.resolve_all();
    let cadets = t.named_on_battlefield("Cadet");
    assert_eq!(cadets.len(), 1);
    let o = t.obj_now(cadets[0]);
    assert!(o.is_token());
    assert_eq!(o.controller, P0);
    assert!(o.chars.is(CardType::Creature));
    assert!(o.chars.colors.is_colorless());
    assert!(o.chars.has_subtype("Wizard") && o.chars.has_subtype("Soldier"));
    // A 2/2, pumped +1/+0 by Prudent Fateseer's "whenever you scry or surveil" trigger.
    assert_eq!(t.pt(cadets[0]), (3, 2));
    assert_eq!(t.pt(f), (2, 4));
    next_turn(&mut t);
    assert_eq!(t.pt(cadets[0]), (2, 2));
    assert_eq!(surveils(&t), vec![1]);
    assert!(t.in_graveyard(P0, "Forest"), "surveilled into the graveyard");
}

#[test]
fn fiery_cannonade_damages_each_non_pirate_creature() {
    cr!("608.2", "120.3e");
    // "Fiery Cannonade deals 2 damage to each non-Pirate creature."
    supported("Fiery Cannonade");
    supported("Fathom Fleet Captain");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let pirate = t.battlefield(P1, "Fathom Fleet Captain");
    t.lands(P0, "Mountain", 3);
    let s = t.hand(P0, "Fiery Cannonade");
    t.cast(P0, s).go();
    t.resolve_all();
    assert!(!t.on_battlefield(mine) && !t.on_battlefield(theirs));
    assert_eq!(t.obj_now(giant).damage, 2);
    assert_eq!(t.obj_now(pirate).damage, 0, "a Pirate");
    assert!(t.on_battlefield(pirate));
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
}

#[test]
fn windstorm_damages_each_creature_with_flying() {
    cr!("608.2", "120.3e", "107.3a");
    // "Windstorm deals X damage to each creature with flying."
    supported("Windstorm");
    let mut t = TestGame::new(2);
    let bird = t.battlefield(P1, "Wind Drake");
    let mine = t.battlefield(P0, "Serra Angel");
    let ground = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 3);
    let s = t.hand(P0, "Windstorm");
    t.cast(P0, s).x(2).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bird), "a 2/2 flier dies to X=2");
    assert_eq!(t.obj_now(mine).damage, 2);
    assert_eq!(t.obj_now(ground).damage, 0, "no flying");
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
}

#[test]
fn sungold_barrage_destroys_only_a_creature_with_toughness_four_or_more() {
    cr!("608.2", "115.1", "701.8a");
    // "Destroy target creature with toughness 4 or greater."
    supported("Sungold Barrage");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P1, "Wall of Stone");
    let angel = t.battlefield(P1, "Serra Angel");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 3);
    let s = t.hand(P0, "Sungold Barrage");
    t.cast(P0, s).target(angel).go();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(wall)) && c.contains(&Entity::Object(angel)));
    assert!(!c.contains(&Entity::Object(giant)), "toughness 3");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Serra Angel"));
    assert!(t.on_battlefield(wall) && t.on_battlefield(giant));
}

#[test]
fn burning_fields_damages_an_opponent_or_planeswalker() {
    cr!("608.2", "115.1", "120.3a", "120.3c");
    // "Burning Fields deals 5 damage to target opponent or planeswalker."
    supported("Burning Fields");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let garruk = t.battlefield(P1, "Garruk Wildspeaker");
    t.lands(P0, "Mountain", 5);
    let s = t.hand(P0, "Burning Fields");
    t.cast(P0, s).target(Entity::Player(P1)).go();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Player(P1)) && c.contains(&Entity::Object(garruk)));
    assert!(!c.contains(&Entity::Player(P0)), "not an opponent");
    assert!(!c.contains(&Entity::Object(bears)));
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 15));
    let mut t = TestGame::new(2);
    let garruk = t.battlefield(P1, "Garruk Wildspeaker");
    t.g.add_counters(Entity::Object(garruk), "loyalty", 4, None);
    t.lands(P0, "Mountain", 5);
    let s = t.hand(P0, "Burning Fields");
    t.cast(P0, s).target(garruk).go();
    t.resolve_all();
    assert_eq!(t.counters(garruk, "loyalty"), 2);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn infinite_obliteration_exiles_the_chosen_cards_with_the_named_creature_name() {
    cr!("608.2", "115.1", "701.23a", "701.24a", "201.4a");
    ruling!(
        "Slaughter Games",
        "You can leave any cards with that name in the zone they are in"
    );
    // Infinite Obliteration: "Choose a creature card name. Search target opponent's
    // graveyard, hand, and library for any number of cards with that name and exile them.
    // Then that player shuffles." (Slaughter Games: the same, naming a nonland card.)
    for spell in ["Infinite Obliteration", "Slaughter Games"] {
        supported(spell);
        let mut t = TestGame::new(2);
        let yard = t.graveyard(P1, "Grizzly Bears");
        let yard2 = t.graveyard(P1, "Grizzly Bears");
        let hand = t.hand(P1, "Grizzly Bears");
        let lib = t.library_top(P1, "Grizzly Bears");
        let lib2 = t.library_top(P1, "Grizzly Bears");
        let other = t.hand(P1, "Hill Giant");
        let mine = t.graveyard(P0, "Grizzly Bears");
        t.lands(P0, "Swamp", 2);
        t.lands(P0, "Mountain", 2);
        let s = t.hand(P0, spell);
        t.answer(P0, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
        // Any number: leave one in the graveyard and one in the library.
        t.answer_choose(
            P0,
            &[Entity::Object(yard), Entity::Object(hand), Entity::Object(lib)],
        );
        t.cast(P0, s).target(Entity::Player(P1)).go();
        let c = target_choices(&t, P0).pop().unwrap();
        assert_eq!(c, vec![Entity::Player(P1)], "target opponent");
        t.resolve_all();
        let asked = t.asked();
        assert!(asked
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::NameCard { .. })));
        let offered = asked
            .iter()
            .find_map(|(p, d)| match d {
                Decision::ChooseEntities { candidates, .. } if *p == P0 => {
                    Some(candidates.clone())
                }
                _ => None,
            })
            .unwrap();
        assert_eq!(offered.len(), 5, "every Grizzly Bears of the opponent's");
        assert!(!offered.contains(&Entity::Object(other)));
        assert!(!offered.contains(&Entity::Object(mine)));
        for x in [yard, hand, lib] {
            assert_eq!(t.zone(t.g.current(x)), Zone::Exile);
        }
        assert_eq!(t.zone(yard2), Zone::Graveyard(P1), "{spell}: left behind");
        assert_eq!(t.zone(lib2), Zone::Library(P1), "{spell}: left behind");
        assert_eq!(t.zone(other), Zone::Hand(P1));
        assert_eq!(t.zone(mine), Zone::Graveyard(P0), "only the opponent's cards");
        assert!(shuffled(&t, P1));
        assert!(!shuffled(&t, P0));
    }
    // The name must match the spell's description (CR 201.4a): Infinite Obliteration
    // names a creature card, Slaughter Games a nonland card. A land name is neither; a
    // noncreature spell's name is only Slaughter Games's.
    for (spell, name, legal) in [
        ("Infinite Obliteration", "Forest", false),
        ("Slaughter Games", "Forest", false),
        ("Infinite Obliteration", "Lightning Bolt", false),
        ("Slaughter Games", "Lightning Bolt", true),
    ] {
        let mut t = TestGame::new(2);
        let card = t.hand(P1, name);
        t.lands(P0, "Swamp", 2);
        t.lands(P0, "Mountain", 2);
        let s = t.hand(P0, spell);
        t.answer(P0, DecisionKind::Name, Answer::Text(name.into()));
        t.answer_choose(P0, &[Entity::Object(card)]);
        t.cast(P0, s).target(Entity::Player(P1)).go();
        t.resolve_all();
        let want = if legal { Zone::Exile } else { Zone::Hand(P1) };
        assert_eq!(t.zone(t.g.current(card)), want, "{spell} naming {name}");
    }
}

#[test]
fn into_the_core_exiles_two_target_artifacts() {
    cr!("608.2", "115.1", "701.13a", "601.2c");
    // "Exile two target artifacts."
    supported("Into the Core");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Sol Ring");
    let b = t.battlefield(P1, "Mind Stone");
    let c = t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 4);
    let s = t.hand(P0, "Into the Core");
    t.cast(P0, s)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    let offered = target_choices(&t, P0);
    assert!(offered
        .iter()
        .all(|c| !c.contains(&Entity::Object(bears))));
    assert!(offered.iter().any(|x| x.contains(&Entity::Object(c))));
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
    assert!(t.on_battlefield(c) && t.on_battlefield(bears));
}

#[test]
fn lash_of_thorns_pumps_and_grants_deathtouch() {
    cr!("608.2", "611.2a", "702.2b", "514.2");
    // "Target creature gets +2/+1 and gains deathtouch until end of turn."
    supported("Lash of Thorns");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let other = t.battlefield(P0, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.lands(P0, "Swamp", 1);
    let s = t.hand(P0, "Lash of Thorns");
    t.cast(P0, s).target(elves).go();
    t.resolve_all();
    assert_eq!(t.pt(elves), (3, 2));
    assert!(t.obj_now(elves).has_keyword(KeywordKind::Deathtouch));
    assert_eq!(t.pt(other), (1, 1));
    assert!(!t.obj_now(other).has_keyword(KeywordKind::Deathtouch));
    t.g.deal_damage(elves, Entity::Object(giant), 1, false);
    t.g.deal_damage(other, Entity::Object(wall), 1, false);
    t.settle();
    assert!(!t.on_battlefield(giant), "deathtouch");
    assert!(t.on_battlefield(wall));
    next_turn(&mut t);
    assert_eq!(t.pt(elves), (1, 1));
    assert!(!t.obj_now(elves).has_keyword(KeywordKind::Deathtouch));
}

#[test]
fn moment_of_craving_shrinks_a_creature_and_gains_two_life() {
    cr!("608.2", "611.2a", "514.2", "119.3", "704.5f");
    // "Target creature gets -2/-2 until end of turn. You gain 2 life."
    supported("Moment of Craving");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 2);
    let s = t.hand(P0, "Moment of Craving");
    t.cast(P0, s).target(bears).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.pt(giant), (3, 3));
    assert_eq!((t.life(P0), t.life(P1)), (22, 20));
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 2);
    let s = t.hand(P0, "Moment of Craving");
    t.cast(P0, s).target(giant).go();
    t.resolve_all();
    assert_eq!(t.pt(giant), (1, 1));
    next_turn(&mut t);
    assert_eq!(t.pt(giant), (3, 3));
}

#[test]
fn magic_damper_pumps_hexproofs_and_untaps_your_creature() {
    cr!("608.2", "115.1", "611.2a", "702.11b", "701.26b", "514.2");
    // "Target creature you control gets +1/+1 and gains hexproof until end of turn.
    // Untap it."
    supported("Magic Damper");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    t.g.tap(other);
    t.lands(P0, "Island", 1);
    let s = t.hand(P0, "Magic Damper");
    t.cast(P0, s).target(bears).go();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(bears)) && c.contains(&Entity::Object(other)));
    assert!(!c.contains(&Entity::Object(theirs)), "not yours");
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
    assert!(t.obj_now(other).tapped);
    assert_eq!(t.pt(bears), (3, 3));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Hexproof));
    assert!(!t.obj_now(other).has_keyword(KeywordKind::Hexproof));
    // Its controller can still target it; the opponent can't.
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(bears).go();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(bears)), "hexproof doesn't stop its controller");
    t.resolve_all();
    assert_eq!(t.pt(bears), (6, 6));
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, bolt).target(other).go();
    let c = target_choices(&t, P1).pop().unwrap();
    assert!(!c.contains(&Entity::Object(bears)), "hexproof");
    assert!(c.contains(&Entity::Object(other)));
    t.resolve_all();
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2), "both pumps ended");
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Hexproof));
}

#[test]
fn reaver_ambush_exiles_only_a_creature_with_power_three_or_less() {
    cr!("608.2", "115.1", "701.13a");
    // "Exile target creature with power 3 or less."
    supported("Reaver Ambush");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let angel = t.battlefield(P1, "Serra Angel");
    let big = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Swamp", 3);
    let s = t.hand(P0, "Reaver Ambush");
    t.cast(P0, s).target(giant).go();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(giant)));
    assert!(!c.contains(&Entity::Object(angel)), "power 4");
    assert!(!c.contains(&Entity::Object(big)), "power 6");
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert!(!t.in_graveyard(P1, "Hill Giant"));
    assert!(t.on_battlefield(angel) && t.on_battlefield(big));
}

#[test]
fn eyeblights_ending_destroys_a_non_elf_creature() {
    cr!("608.2", "115.1", "701.8a");
    // "Destroy target non-Elf creature."
    supported("Eyeblight's Ending");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elf = t.battlefield(P1, "Llanowar Elves");
    let land = t.battlefield(P1, "Forest");
    t.lands(P0, "Swamp", 3);
    let s = t.hand(P0, "Eyeblight's Ending");
    t.cast(P0, s).target(bears).go();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(bears)));
    assert!(!c.contains(&Entity::Object(elf)), "an Elf");
    assert!(!c.contains(&Entity::Object(land)));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(elf));
}

#[test]
fn hymn_to_tourach_discards_two_cards_at_random() {
    cr!("608.2", "115.1", "701.9a", "701.9b");
    // "Target player discards two cards at random."
    supported("Hymn to Tourach");
    let mut t = TestGame::new(2);
    for name in ["Forest", "Island", "Swamp"] {
        t.hand(P1, name);
    }
    let mine = t.hand(P0, "Plains");
    t.lands(P0, "Swamp", 2);
    let s = t.hand(P0, "Hymn to Tourach");
    t.cast(P0, s).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(t.graveyard_size(P1), 2);
    assert_eq!(t.zone(mine), Zone::Hand(P0));
    assert!(
        entity_choices(&t, P0).is_empty() && entity_choices(&t, P1).is_empty(),
        "at random"
    );
    // With one card, it discards just that.
    let mut t = TestGame::new(2);
    t.hand(P1, "Forest");
    t.lands(P0, "Swamp", 2);
    let s = t.hand(P0, "Hymn to Tourach");
    t.cast(P0, s).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    assert!(t.in_graveyard(P1, "Forest"));
}

const CHANDRA: &str = "Chandra, Bold Pyromancer";

#[test]
fn chandras_outburst_finds_chandra_in_the_graveyard_or_library() {
    cr!("608.2", "701.23a", "701.24a");
    // "Chandra's Outburst deals 4 damage to target player or planeswalker. Search your
    // library and/or graveyard for a card named Chandra, Bold Pyromancer, reveal it, and
    // put it into your hand. If you search your library this way, shuffle."
    supported("Chandra's Outburst");
    // In the graveyard: search only the graveyard, no shuffle.
    let mut t = TestGame::new(2);
    let c = t.graveyard(P0, CHANDRA);
    let in_lib = t.library_top(P0, CHANDRA);
    t.graveyard(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 5);
    let s = t.hand(P0, "Chandra's Outburst");
    t.answer_yes(P0, false); // don't search the library
    t.answer_choose(P0, &[Entity::Object(c)]);
    t.cast(P0, s).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(hand_names(&t, P0), vec![CHANDRA.to_string()]);
    let offered = t
        .asked()
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .unwrap();
    assert_eq!(offered, vec![Entity::Object(c)], "only the named card, only the graveyard");
    assert_eq!(t.zone(in_lib), Zone::Library(P0));
    assert!(!shuffled(&t, P0), "the library wasn't searched");
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    // In the library: found there, then the library is shuffled.
    let mut t = TestGame::new(2);
    let c = t.library_top(P0, CHANDRA);
    t.library_top(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 5);
    let s = t.hand(P0, "Chandra's Outburst");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(c)]);
    t.cast(P0, s).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(hand_names(&t, P0), vec![CHANDRA.to_string()]);
    assert!(shuffled(&t, P0));
    assert!(!shuffled(&t, P1));
}

#[test]
fn utter_end_exiles_a_nonland_permanent() {
    cr!("608.2", "115.1", "701.13a");
    // "Exile target nonland permanent."
    supported("Utter End");
    let mut t = TestGame::new(2);
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ring = t.battlefield(P1, "Sol Ring");
    let land = t.battlefield(P1, "Forest");
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Swamp", 2);
    let s = t.hand(P0, "Utter End");
    t.cast(P0, s).target(anthem).go();
    let c = target_choices(&t, P0).pop().unwrap();
    for x in [anthem, bears, ring] {
        assert!(c.contains(&Entity::Object(x)));
    }
    assert!(!c.contains(&Entity::Object(land)), "a land");
    t.resolve_all();
    assert!(t.in_exile("Glorious Anthem"));
    assert!(!t.in_graveyard(P1, "Glorious Anthem"));
    assert_eq!(t.pt(bears), (2, 2), "the anthem is gone");
}

#[test]
fn roiling_regrowth_sacrifices_a_land_for_two_tapped_basics() {
    cr!("608.2", "701.21a", "701.23a", "701.24a", "205.4c");
    // "Sacrifice a land. Search your library for up to two basic land cards, put them
    // onto the battlefield tapped, then shuffle."
    supported("Roiling Regrowth");
    let mut t = TestGame::new(2);
    let lands = t.lands(P0, "Forest", 3);
    let spare = t.battlefield(P0, "Taiga");
    let theirs = t.battlefield(P1, "Forest");
    let i = t.library_top(P0, "Island");
    let p = t.library_top(P0, "Plains");
    let nonbasic = t.library_top(P0, "Savannah");
    let s = t.hand(P0, "Roiling Regrowth");
    t.cast(P0, s).go();
    assert!(lands.iter().all(|l| t.obj_now(*l).tapped));
    t.answer_choose(P0, &[Entity::Object(spare)]);
    t.answer_choose(P0, &[Entity::Object(i), Entity::Object(p)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Taiga"), "the chosen land was sacrificed");
    assert!(t.on_battlefield(theirs));
    let offered: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseEntities { candidates, .. } if q == P0 => Some(candidates),
            _ => None,
        })
        .collect();
    let sac = &offered[0];
    assert!(!sac.contains(&Entity::Object(theirs)), "only your lands");
    let search = offered.last().unwrap();
    assert!(search.contains(&Entity::Object(i)) && search.contains(&Entity::Object(p)));
    assert!(!search.contains(&Entity::Object(nonbasic)), "not basic");
    for name in ["Island", "Plains"] {
        let v = t.named_on_battlefield(name);
        assert_eq!(v.len(), 1, "{name}");
        assert!(t.obj_now(v[0]).tapped, "enters tapped");
        assert_eq!(t.obj_now(v[0]).controller, P0);
    }
    assert!(t.named_on_battlefield("Savannah").is_empty());
    assert!(shuffled(&t, P0));
}

#[test]
fn silk_net_pumps_and_grants_reach() {
    cr!("608.2", "611.2a", "702.17b", "514.2");
    // "Target creature gets +1/+1 and gains reach until end of turn."
    supported("Silk Net");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Grizzly Bears");
    let drake = t.battlefield(P0, "Wind Drake");
    t.lands(P1, "Forest", 1);
    let s = t.hand(P1, "Silk Net");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(drake, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    t.cast(P1, s).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
    assert_eq!(t.pt(other), (2, 2));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Reach));
    assert!(!t.obj_now(other).has_keyword(KeywordKind::Reach));
    assert!(t.g.can_block(bears, drake), "reach blocks a flier");
    assert!(!t.g.can_block(other, drake));
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Reach));
}
