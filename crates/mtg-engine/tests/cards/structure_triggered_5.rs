//! Structure-coverage tests, round 3 (item 5): triggered-ability structures shared by many
//! cards that no other test exercised (see `docs/STRUCTURE_COVERAGE.md`). Each test checks
//! a real card's ability does what its text says: the right objects and players affected
//! and not others, amounts, durations ending, conditions true and false, and the targets
//! the engine offers. Combat triggers are in `structure_triggered_5_combat.rs`.

#![allow(dead_code)]

use super::structure_activated_2::{act, target_choices};
use super::structure_seed_common::*;
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::mana::ManaType;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// Puts `name` onto the battlefield through a real zone change and resolves its triggers.
pub fn etb(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.enter(p, name);
    t.resolve_all();
    t.g.current(id)
}

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Moves on to the opponent's upkeep, past the cleanup step that ends "until end of turn"
/// effects.
pub fn next_turn(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
}

/// Destroys `id` (it dies) and resolves the triggers.
fn kill(t: &mut TestGame, id: ObjectId) {
    let cur = t.g.current(id);
    t.g.destroy(cur, None);
    t.resolve_all();
}

// ---------------------------------------------------------------------------
// "When ~ enters, ..."
// ---------------------------------------------------------------------------

#[test]
fn leonin_snarecaster_may_tap_target_creature() {
    cr!("603.6a", "115.1", "603.5");
    // "When this creature enters, you may tap target creature."
    supported("Leonin Snarecaster");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Hill Giant");
    let forest = t.battlefield(P1, "Forest");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_yes(P0, true);
    let s = etb(&mut t, P0, "Leonin Snarecaster");
    assert!(t.obj_now(bears).tapped);
    assert!(!t.obj_now(other).tapped);
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(bears)) && offered.contains(&obj(other)));
    assert!(offered.contains(&obj(s)), "any creature, itself included");
    assert!(!offered.contains(&obj(forest)), "not a creature");
    // Declining leaves it untapped.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_yes(P0, false);
    etb(&mut t, P0, "Leonin Snarecaster");
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn messenger_hawk_creates_a_clue() {
    cr!("603.6a", "111.10f");
    // "When this creature enters, create a Clue token."
    supported("Messenger Hawk");
    let mut t = TestGame::new(2);
    etb(&mut t, P0, "Messenger Hawk");
    let clues = subtype_ids(&t, P0, "Clue");
    assert_eq!(clues.len(), 1);
    let c = t.obj_now(clues[0]);
    assert!(c.is_token() && c.is(CardType::Artifact) && !c.is_creature());
    assert_eq!(count_subtype(&t, P1, "Clue"), 0);
    // "{2}, Sacrifice this token: Draw a card."
    t.lands(P0, "Wastes", 2);
    t.activate(P0, clues[0], 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(count_subtype(&t, P0, "Clue"), 0);
}

#[test]
fn mournwhelk_makes_target_player_discard_two() {
    cr!("603.6a", "701.9a", "701.9b");
    // "When this creature enters, target player discards two cards."
    supported("Mournwhelk");
    let mut t = TestGame::new(3);
    let a = t.hand(P1, "Lightning Bolt");
    let b = t.hand(P1, "Forest");
    let c = t.hand(P1, "Grizzly Bears");
    t.hand(P2, "Forest");
    t.hand(P0, "Forest");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P1, &[obj(a), obj(c)]);
    etb(&mut t, P0, "Mournwhelk");
    assert_eq!(t.zone(a), Zone::Graveyard(P1));
    assert_eq!(t.zone(c), Zone::Graveyard(P1));
    assert_eq!(t.zone(b), Zone::Hand(P1), "the discarding player chose");
    assert_eq!((t.hand_size(P0), t.hand_size(P2)), (1, 1));
    let offered = &target_choices(&t, P0)[0];
    for p in [P0, P1, P2] {
        assert!(offered.contains(&Entity::Player(p)), "any player");
    }
    // With a single card, that card is discarded.
    let mut t = TestGame::new(2);
    t.hand(P1, "Forest");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    etb(&mut t, P0, "Mournwhelk");
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), 1);
}

#[test]
fn ogre_arsonist_destroys_target_land() {
    cr!("603.6a", "115.1", "701.8a");
    // "When this creature enters, destroy target land."
    supported("Ogre Arsonist");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    let keep = t.battlefield(P1, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(land)]);
    let o = etb(&mut t, P0, "Ogre Arsonist");
    assert!(!t.on_battlefield(land));
    assert!(t.in_graveyard(P1, "Forest"));
    assert!(t.on_battlefield(keep));
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(keep)));
    assert!(!offered.contains(&obj(bears)) && !offered.contains(&obj(o)));
}

#[test]
fn angel_of_despair_destroys_target_permanent() {
    cr!("603.6a", "115.1", "701.8a");
    // "When this creature enters, destroy target permanent."
    supported("Angel of Despair");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    let forest = t.battlefield(P1, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card_in_gy = t.graveyard(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(jace)]);
    etb(&mut t, P0, "Angel of Despair");
    assert!(!t.on_battlefield(jace));
    assert!(t.on_battlefield(forest) && t.on_battlefield(bears));
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(forest)) && offered.contains(&obj(bears)));
    assert!(!offered.contains(&obj(card_in_gy)), "permanents only");
    assert!(!offered.contains(&Entity::Player(P1)));
}

#[test]
fn professor_of_symbology_learns() {
    cr!("603.6a", "701.48a");
    // "When this creature enters, learn."
    supported("Professor of Symbology");
    // A Lesson from outside the game (options: nothing, rummage, Lesson).
    let mut t = TestGame::new(2);
    let lesson = t.custom(
        P0,
        (*card("Environmental Sciences")).clone(),
        Zone::Outside(P0),
    );
    let forest = t.hand(P0, "Forest");
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    etb(&mut t, P0, "Professor of Symbology");
    assert_eq!(t.zone(lesson), Zone::Hand(P0));
    assert_eq!(t.zone(forest), Zone::Hand(P0), "nothing discarded");
    assert_eq!(t.library_size(P0), 30);
    // Rummaging instead: discard, then draw.
    let mut t = TestGame::new(2);
    let lesson = t.custom(
        P0,
        (*card("Environmental Sciences")).clone(),
        Zone::Outside(P0),
    );
    let forest = t.hand(P0, "Forest");
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    etb(&mut t, P0, "Professor of Symbology");
    assert_eq!(t.zone(forest), Zone::Graveyard(P0));
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.library_size(P0), 29);
    assert_eq!(t.zone(lesson), Zone::Outside(P0));
}

#[test]
fn chainsaw_deals_3_to_up_to_one_target_creature() {
    cr!("603.6a", "603.3d", "120.3e", "704.5g");
    // "When this Equipment enters, it deals 3 damage to up to one target creature."
    supported("Chainsaw");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(giant)]);
    etb(&mut t, P0, "Chainsaw");
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.life(P1), 20);
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(bears)));
    assert!(!offered.contains(&Entity::Player(P1)), "creatures only");
    // Zero targets: the ability does nothing.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[]);
    let saw = etb(&mut t, P0, "Chainsaw");
    assert!(t.on_battlefield(bears) && t.on_battlefield(saw));
    assert_eq!(t.obj_now(bears).damage, 0);
}

#[test]
fn quick_draw_dagger_attaches_and_grants_first_strike() {
    cr!("603.6a", "701.3a", "611.2a");
    // "When this Equipment enters, attach it to target creature you control. That creature
    // gains first strike until end of turn. / Equipped creature gets +1/+1."
    supported("Quick-Draw Dagger");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let d = etb(&mut t, P0, "Quick-Draw Dagger");
    assert_eq!(t.obj_now(d).attached_to, Some(obj(bears)));
    assert_eq!(t.pt(bears), (3, 3));
    assert!(t.obj_now(bears).has_keyword(keywords::KeywordKind::FirstStrike));
    assert!(!t.obj_now(other).has_keyword(keywords::KeywordKind::FirstStrike));
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(other)));
    assert!(!offered.contains(&obj(theirs)), "a creature you control");
    next_turn(&mut t);
    assert!(!t.obj_now(bears).has_keyword(keywords::KeywordKind::FirstStrike));
    assert_eq!(t.pt(bears), (3, 3), "still equipped");
}

#[test]
fn rooftop_assassin_destroys_an_opponents_damaged_creature() {
    cr!("603.6a", "603.3d", "115.1", "701.8a");
    // "When this creature enters, destroy target creature an opponent controls that was
    // dealt damage this turn."
    supported("Rooftop Assassin");
    let mut t = TestGame::new(2);
    let src = t.battlefield(P0, "Grizzly Bears");
    let hurt = t.battlefield(P1, "Hill Giant");
    let fresh = t.battlefield(P1, "Hill Giant");
    let mine = t.battlefield(P0, "Hill Giant");
    t.g.deal_damage(src, obj(hurt), 1, false);
    t.g.deal_damage(src, obj(mine), 1, false);
    t.answer_targets(P0, &[obj(hurt)]);
    etb(&mut t, P0, "Rooftop Assassin");
    assert!(!t.on_battlefield(hurt));
    assert!(t.on_battlefield(fresh) && t.on_battlefield(mine));
    let offered = &target_choices(&t, P0)[0];
    assert_eq!(offered, &vec![obj(hurt)], "only a damaged opponent's creature");
    // Damage dealt last turn doesn't count: with no legal target, the ability is removed
    // from the stack (CR 603.3d).
    let mut t = TestGame::new(2);
    let src = t.battlefield(P0, "Grizzly Bears");
    let hurt = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(src, obj(hurt), 1, false);
    next_turn(&mut t);
    etb(&mut t, P0, "Rooftop Assassin");
    assert!(t.on_battlefield(hurt));
    assert!(target_choices(&t, P0).iter().all(|c| c.is_empty()));
}

#[test]
fn winter_eladrin_returns_up_to_one_other_creature() {
    cr!("603.6a", "603.3d", "115.1");
    // "When this creature enters, return up to one other target creature to its owner's
    // hand."
    supported("Winter Eladrin");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(theirs)]);
    let e = etb(&mut t, P0, "Winter Eladrin");
    assert_eq!(t.zone(theirs), Zone::Hand(P1), "to its owner's hand");
    assert!(t.on_battlefield(mine) && t.on_battlefield(e));
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(mine)));
    assert!(!offered.contains(&obj(e)), "another creature");
    // Zero targets.
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[]);
    etb(&mut t, P0, "Winter Eladrin");
    assert!(t.on_battlefield(theirs));
}

#[test]
fn voidwielder_may_return_target_creature() {
    cr!("603.6a", "603.5", "115.1");
    // "When this creature enters, you may return target creature to its owner's hand."
    supported("Voidwielder");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let forest = t.battlefield(P1, "Forest");
    t.answer_targets(P0, &[obj(theirs)]);
    t.answer_yes(P0, true);
    etb(&mut t, P0, "Voidwielder");
    assert_eq!(t.zone(theirs), Zone::Hand(P1));
    let offered = &target_choices(&t, P0)[0];
    assert!(!offered.contains(&obj(forest)));
    // Declining.
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(theirs)]);
    t.answer_yes(P0, false);
    etb(&mut t, P0, "Voidwielder");
    assert!(t.on_battlefield(theirs));
}

#[test]
fn aviary_mechanic_may_return_another_permanent_you_control() {
    cr!("603.6a", "603.5");
    // "When this creature enters, you may return another permanent you control to its
    // owner's hand."
    supported("Aviary Mechanic");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(forest)]);
    let m = etb(&mut t, P0, "Aviary Mechanic");
    assert_eq!(t.zone(forest), Zone::Hand(P0), "any permanent, a land included");
    assert!(t.on_battlefield(bears) && t.on_battlefield(theirs) && t.on_battlefield(m));
    let offered: Vec<Entity> = t
        .asked()
        .into_iter()
        .find_map(|(q, d)| match d {
            Decision::ChooseEntities { candidates, .. } if q == P0 => Some(candidates),
            _ => None,
        })
        .expect("asked which permanent to return");
    assert!(offered.contains(&obj(forest)) && offered.contains(&obj(bears)));
    assert!(!offered.contains(&obj(m)), "another permanent");
    assert!(!offered.contains(&obj(theirs)), "one you control");
    // Declining.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    t.answer_yes(P0, false);
    let m = etb(&mut t, P0, "Aviary Mechanic");
    assert!(t.on_battlefield(forest) && t.on_battlefield(m));
    // It never returns itself or an opponent's permanent, even with nothing else to pick.
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    let m = etb(&mut t, P0, "Aviary Mechanic");
    assert!(t.on_battlefield(m) && t.on_battlefield(theirs));
}

#[test]
fn citizens_arrest_exiles_until_it_leaves() {
    cr!("603.6a", "610.3", "610.3c", "115.1");
    // "When this enchantment enters, exile target creature or planeswalker an opponent
    // controls until this enchantment leaves the battlefield."
    supported("Citizen's Arrest");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let jace = t.battlefield(P1, "Jace Beleren");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let forest = t.battlefield(P1, "Forest");
    t.answer_targets(P0, &[obj(bears)]);
    let ca = etb(&mut t, P0, "Citizen's Arrest");
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(t.on_battlefield(jace) && t.on_battlefield(mine));
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(jace)), "a planeswalker");
    assert!(!offered.contains(&obj(mine)) && !offered.contains(&obj(forest)));
    // When the enchantment leaves, the creature returns under its owner's control.
    kill(&mut t, ca);
    let back = t.g.current(bears);
    assert!(t.on_battlefield(back));
    assert_eq!(t.obj_now(back).controller, P1);
}

#[test]
fn elvish_regrower_returns_a_permanent_card_from_your_graveyard() {
    cr!("603.6a", "115.1", "110.4a");
    // "When this creature enters, return target permanent card from your graveyard to your
    // hand."
    supported("Elvish Regrower");
    let mut t = TestGame::new(2);
    let forest = t.graveyard(P0, "Forest");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let theirs = t.graveyard(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(forest)]);
    etb(&mut t, P0, "Elvish Regrower");
    assert_eq!(t.zone(forest), Zone::Hand(P0), "a land is a permanent card");
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(bears)));
    assert!(!offered.contains(&obj(bolt)), "an instant isn't a permanent card");
    assert!(!offered.contains(&obj(theirs)), "your graveyard only");
}

#[test]
fn ambush_wolf_exiles_up_to_one_card_from_a_graveyard() {
    cr!("603.6a", "603.3d", "701.13a");
    // "When this creature enters, exile up to one target card from a graveyard."
    supported("Ambush Wolf");
    let mut t = TestGame::new(2);
    let theirs = t.graveyard(P1, "Lightning Bolt");
    let mine = t.graveyard(P0, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(theirs)]);
    etb(&mut t, P0, "Ambush Wolf");
    assert_eq!(t.zone(theirs), Zone::Exile);
    assert_eq!(t.zone(mine), Zone::Graveyard(P0));
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(mine)), "any graveyard");
    assert!(!offered.contains(&obj(bears)));
    // Zero targets.
    let mut t = TestGame::new(2);
    let theirs = t.graveyard(P1, "Lightning Bolt");
    t.answer_targets(P0, &[]);
    etb(&mut t, P0, "Ambush Wolf");
    assert_eq!(t.zone(theirs), Zone::Graveyard(P1));
}

#[test]
fn chaos_maw_deals_3_to_each_other_creature() {
    cr!("603.6a", "120.3e", "704.5g");
    // "When this creature enters, it deals 3 damage to each other creature."
    supported("Chaos Maw");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let big = t.battlefield(P1, "Craw Wurm");
    let jace = t.battlefield(P1, "Jace Beleren");
    let m = etb(&mut t, P0, "Chaos Maw");
    assert!(!t.on_battlefield(mine) && !t.on_battlefield(theirs));
    assert_eq!(t.obj_now(big).damage, 3);
    assert_eq!(t.obj_now(m).damage, 0, "not itself");
    assert_eq!(t.counters(jace, "loyalty"), 3, "creatures only");
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
}

#[test]
fn dwarven_priest_gains_1_for_each_creature_you_control() {
    cr!("603.6a", "119.3");
    // "When this creature enters, you gain 1 life for each creature you control."
    supported("Dwarven Priest");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Forest");
    t.battlefield(P1, "Grizzly Bears");
    etb(&mut t, P0, "Dwarven Priest");
    assert_eq!(t.life(P0), 23, "two creatures plus the Priest itself");
    assert_eq!(t.life(P1), 20);
}

// ---------------------------------------------------------------------------
// Dies and other zone-change triggers
// ---------------------------------------------------------------------------

#[test]
fn caustic_hound_makes_each_player_lose_4_when_it_dies() {
    cr!("603.6c", "700.4", "119.3");
    // "When this creature dies, each player loses 4 life."
    supported("Caustic Hound");
    let mut t = TestGame::new(3);
    let h = t.battlefield(P1, "Caustic Hound");
    kill(&mut t, h);
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (16, 16, 16));
    // Leaving another way (bounced) isn't dying.
    let mut t = TestGame::new(2);
    let h = t.battlefield(P1, "Caustic Hound");
    t.g.move_object(h, Zone::Hand(P1), MoveCause::Effect, None);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
}

#[test]
fn plague_dogs_gives_all_creatures_minus_1_when_it_dies() {
    cr!("603.6c", "700.4", "611.2a", "611.2c", "704.5f");
    // "When this creature dies, all creatures get -1/-1 until end of turn."
    supported("Plague Dogs");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Plague Dogs");
    let elf = t.battlefield(P0, "Llanowar Elves");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    kill(&mut t, d);
    assert!(!t.on_battlefield(elf));
    assert_eq!(t.pt(theirs), (1, 1));
    // A creature entering afterwards isn't affected (the set is locked in).
    let late = t.battlefield(P1, "Hill Giant");
    assert_eq!(t.pt(late), (3, 3));
    next_turn(&mut t);
    assert_eq!(t.pt(theirs), (2, 2));
}

#[test]
fn massacre_wurm_makes_a_player_lose_2_when_their_creature_dies() {
    cr!("603.6c", "603.10a", "700.4", "119.3");
    // "Whenever a creature an opponent controls dies, that player loses 2 life."
    supported("Massacre Wurm");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Massacre Wurm");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P2, "Grizzly Bears");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let token_like = t.battlefield(P1, "Forest");
    kill(&mut t, b1);
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (20, 18, 20));
    kill(&mut t, b2);
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (20, 18, 18));
    kill(&mut t, mine);
    kill(&mut t, token_like);
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (20, 18, 18), "not yours, not a land");
}

#[test]
fn crowded_crypt_counts_your_creatures_dying() {
    cr!("603.6c", "603.10a", "700.4", "122.1");
    // "Whenever a creature you control dies, put a corpse counter on this artifact."
    supported("Crowded Crypt");
    let mut t = TestGame::new(2);
    let crypt = t.battlefield(P0, "Crowded Crypt");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let land = t.battlefield(P0, "Forest");
    kill(&mut t, a);
    assert_eq!(t.counters(crypt, "corpse"), 1);
    kill(&mut t, theirs);
    kill(&mut t, land);
    assert_eq!(t.counters(crypt, "corpse"), 1);
    kill(&mut t, b);
    assert_eq!(t.counters(crypt, "corpse"), 2);
}

// ---------------------------------------------------------------------------
// Other events
// ---------------------------------------------------------------------------

#[test]
fn kor_celebrant_gains_1_for_itself_and_each_other_creature_you_control() {
    cr!("603.6a", "603.2", "119.3");
    // "Whenever this creature or another creature you control enters, you gain 1 life."
    supported("Kor Celebrant");
    let mut t = TestGame::new(2);
    etb(&mut t, P0, "Kor Celebrant");
    assert_eq!(t.life(P0), 21, "itself");
    etb(&mut t, P0, "Grizzly Bears");
    assert_eq!(t.life(P0), 22);
    etb(&mut t, P1, "Grizzly Bears");
    etb(&mut t, P0, "Forest");
    assert_eq!(t.life(P0), 22, "not an opponent's creature or a land");
    assert_eq!(t.life(P1), 20);
}

#[test]
fn old_fat_spider_draws_when_an_opponent_targets_it() {
    cr!("603.2", "115.1");
    // "Whenever this creature becomes the target of a spell or ability an opponent
    // controls, draw a card."
    supported("Old Fat Spider");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Old Fat Spider");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, bolt).target(s).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.obj_now(s).damage, 3);
    // An opponent's activated ability counts too.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Old Fat Spider");
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    t.set_step(P1, Step::PrecombatMain);
    act(&mut t, P1, pyro, "1 damage", &[obj(s)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(s).damage, 1);
    assert_eq!(t.hand_size(P0), 1);
    // Your own spell doesn't count.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Old Fat Spider");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(s).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    // An opponent's spell targeting another creature doesn't either.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Old Fat Spider");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, bolt).target(bears).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn enchantresss_presence_draws_on_enchantment_spells() {
    cr!("601.2i", "603.2");
    // "Whenever you cast an enchantment spell, draw a card."
    supported("Enchantress's Presence");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Enchantress's Presence");
    t.lands(P0, "Mountain", 2);
    let e = t.hand(P0, "Mass Hysteria");
    t.cast(P0, e).go();
    t.settle();
    assert_eq!(t.hand_size(P0), 0, "the trigger hasn't resolved yet");
    assert_eq!(t.stack_len(), 2, "the spell and the trigger above it");
    t.resolve();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.stack_len(), 1, "drawn while the spell is still on the stack");
    t.resolve_all();
    // A creature spell, or an opponent's enchantment, doesn't draw.
    let g = t.hand(P0, "Raging Goblin");
    t.cast(P0, g).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    t.lands(P1, "Mountain", 1);
    let theirs = t.hand(P1, "Mass Hysteria");
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, theirs).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.hand_size(P1), 0);
}

#[test]
fn dragons_rage_channeler_surveils_on_noncreature_spells() {
    cr!("601.2i", "603.2", "701.25a");
    // "Whenever you cast a noncreature spell, surveil 1."
    supported("Dragon's Rage Channeler");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dragon's Rage Channeler");
    t.lands(P0, "Mountain", 2);
    let top = t.library_top(P0, "Forest");
    t.answer(P0, DecisionKind::Surveil, Answer::Split(vec![], vec![top]));
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(surveils(&t), vec![1]);
    assert_eq!(t.zone(top), Zone::Graveyard(P0), "surveilled into the graveyard");
    let g = t.hand(P0, "Raging Goblin");
    t.cast(P0, g).go();
    t.resolve_all();
    assert_eq!(surveils(&t), vec![1], "not on a creature spell");
}

#[test]
fn comforting_counsel_counts_life_gain_events() {
    cr!("603.2", "119.9", "122.1");
    // "Whenever you gain life, put a growth counter on this enchantment. / As long as there
    // are five or more growth counters on this enchantment, creatures you control get
    // +3/+3."
    supported("Comforting Counsel");
    let mut t = TestGame::new(2);
    let cc = t.battlefield(P0, "Comforting Counsel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.gain_life(P0, 3);
    t.resolve_all();
    assert_eq!(t.counters(cc, "growth"), 1, "one counter per event, not per life");
    t.g.gain_life(P1, 2);
    t.resolve_all();
    assert_eq!(t.counters(cc, "growth"), 1, "not an opponent's life gain");
    for _ in 0..3 {
        t.g.gain_life(P0, 1);
        t.resolve_all();
    }
    assert_eq!(t.counters(cc, "growth"), 4);
    assert_eq!(t.pt(bears), (2, 2));
    t.g.gain_life(P0, 1);
    t.resolve_all();
    assert_eq!(t.counters(cc, "growth"), 5);
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn zendikar_resurgent_adds_one_more_mana_of_a_type_the_land_produced() {
    cr!("605.1b", "605.4a", "106.12a");
    // "Whenever you tap a land for mana, add one mana of any type that land produced."
    supported("Zendikar Resurgent");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zendikar Resurgent");
    let forest = t.battlefield(P0, "Forest");
    t.activate(P0, forest, 0, &[]).unwrap();
    t.settle();
    let pool = &t.g.player(P0).mana_pool;
    assert_eq!(pool.mana.len(), 2);
    assert_eq!(pool.count(ManaType::G), 2);
    assert_eq!(t.stack_len(), 0, "a mana ability");
    // A nonland mana source, or an opponent's land, doesn't trigger it.
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.activate(P0, elves, 0, &[]).unwrap();
    t.settle();
    assert_eq!(t.g.player(P0).mana_pool.mana.len(), 3);
    let theirs = t.battlefield(P1, "Mountain");
    t.activate(P1, theirs, 0, &[]).unwrap();
    t.settle();
    assert_eq!(t.g.player(P1).mana_pool.mana.len(), 1);
    assert_eq!(t.g.player(P0).mana_pool.mana.len(), 3);
    // A dual land tapped for {U} adds another {U}, not {G}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zendikar Resurgent");
    let pool = t.battlefield(P0, "Breeding Pool");
    act(&mut t, P0, pool, "{u}", &[]).unwrap();
    t.settle();
    let m = &t.g.player(P0).mana_pool;
    assert_eq!((m.count(ManaType::U), m.count(ManaType::G)), (2, 0));
}

// ---------------------------------------------------------------------------
// Upkeep and beginning-of-combat triggers
// ---------------------------------------------------------------------------

#[test]
fn crop_sigil_may_mill_a_card_each_upkeep() {
    cr!("503.1a", "603.5", "701.17a");
    // "At the beginning of your upkeep, you may mill a card."
    supported("Crop Sigil");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Crop Sigil");
    t.answer_yes(P1, true);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 1);
    assert_eq!(t.library_size(P1), 29);
    assert_eq!(t.graveyard_size(P0), 0);
    // Declining.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Crop Sigil");
    t.answer_yes(P1, false);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 0);
    assert_eq!(t.library_size(P1), 30);
}

#[test]
fn nut_collector_may_create_a_squirrel_each_upkeep() {
    cr!("503.1a", "603.5", "111.1");
    // "At the beginning of your upkeep, you may create a 1/1 green Squirrel creature
    // token."
    supported("Nut Collector");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Nut Collector");
    t.answer_yes(P1, true);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    let sq = subtype_ids(&t, P1, "Squirrel");
    assert_eq!(sq.len(), 1);
    assert_eq!(t.pt(sq[0]), (1, 1));
    assert!(t.obj_now(sq[0]).is_token());
    assert!(t.obj_now(sq[0]).chars.colors.contains(types::Color::Green));
    // Not on another player's upkeep (even if its controller would accept).
    t.answer_yes(P0, true);
    t.answer_yes(P1, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(count_subtype(&t, P1, "Squirrel"), 1);
    assert_eq!(count_subtype(&t, P0, "Squirrel"), 0);
    // Declining.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Nut Collector");
    t.answer_yes(P1, false);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(count_subtype(&t, P1, "Squirrel"), 0);
}

#[test]
fn siege_veteran_counters_a_creature_you_control_at_beginning_of_combat() {
    cr!("603.2", "115.1", "122.1");
    // "At the beginning of combat on your turn, put a +1/+1 counter on target creature you
    // control."
    supported("Siege Veteran");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Siege Veteran");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.counters(v, "+1/+1"), 0);
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(v)));
    assert!(!offered.contains(&obj(theirs)));
    // Not on the opponent's turn.
    t.advance_to(P1, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1") + t.counters(v, "+1/+1"), 1);
}

#[test]
fn valor_singer_pumps_a_creature_you_control_at_beginning_of_combat() {
    cr!("603.2", "115.1", "611.2a");
    // "At the beginning of combat on your turn, target creature you control gets +1/+0
    // until end of turn."
    supported("Valor Singer");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Valor Singer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 2));
    assert_eq!(t.pt(v), (2, 3));
    let offered = &target_choices(&t, P0)[0];
    assert!(!offered.contains(&obj(theirs)));
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn ascendant_dustspeaker_exiles_up_to_one_graveyard_card_at_beginning_of_combat() {
    cr!("603.2", "603.3d", "701.13a");
    // "At the beginning of combat on your turn, exile up to one target card from a
    // graveyard."
    supported("Ascendant Dustspeaker");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ascendant Dustspeaker");
    let theirs = t.graveyard(P1, "Lightning Bolt");
    let mine = t.graveyard(P0, "Forest");
    t.answer_targets(P0, &[obj(theirs)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.zone(theirs), Zone::Exile);
    assert_eq!(t.zone(mine), Zone::Graveyard(P0));
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(mine)));
    // Choosing no target.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ascendant Dustspeaker");
    let theirs = t.graveyard(P1, "Lightning Bolt");
    t.answer_targets(P0, &[]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.zone(theirs), Zone::Graveyard(P1));
}
