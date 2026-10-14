//! Structure-coverage tests, round 1: triggered-ability structures shared by many cards
//! that no other test exercised (see `docs/STRUCTURE_COVERAGE.md`). Each test checks a
//! real card's ability does what its text says: the right objects and players affected
//! and not others, amounts, durations ending, and target legality.

use super::structure_seed_common::*;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::object::Zone;
use mtg_engine::*;

/// Puts `name` onto the battlefield through a real zone change and resolves its triggers.
fn etb(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.enter(p, name);
    t.resolve_all();
    id
}

/// Moves on to the opponent's upkeep, past the cleanup step that ends "until end of turn"
/// effects.
fn next_turn(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
}

/// Starts a combat on P0's turn with the given attackers (and P1's blocks) and runs it to
/// its end.
fn fight(t: &mut TestGame, attackers: &[ObjectId], blocks: &[(ObjectId, ObjectId)]) {
    let a: Vec<(ObjectId, Entity)> = attackers
        .iter()
        .map(|a| (*a, Entity::Player(P1)))
        .collect();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&a, blocks);
}

fn energy(t: &TestGame, p: PlayerId) -> u32 {
    t.g.player(p).counter(types::counters::ENERGY)
}

// ---------------------------------------------------------------------------
// "When ~ enters, ..." — effects on the controller
// ---------------------------------------------------------------------------

#[test]
fn ashioks_forerunner_finds_ashiok_in_library_or_graveyard() {
    cr!("603.6a", "701.23a");
    // "When this creature enters, you may search your library and/or graveyard for a card
    // named Ashiok, Sculptor of Fears, reveal it, and put it into your hand. If you search
    // your library this way, shuffle."
    supported("Ashiok's Forerunner");
    // From the graveyard.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Ashiok, Sculptor of Fears");
    t.graveyard(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    etb(&mut t, P0, "Ashiok's Forerunner");
    assert!(t.in_hand(P0, "Ashiok, Sculptor of Fears"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), 1);
    // From the library.
    let mut t = TestGame::new(2);
    t.library_top(P0, "Ashiok, Sculptor of Fears");
    t.g.players[P0.idx()].library.rotate_right(5);
    t.answer_yes(P0, true);
    etb(&mut t, P0, "Ashiok's Forerunner");
    assert!(t.in_hand(P0, "Ashiok, Sculptor of Fears"));
    assert_eq!(t.library_size(P0), 30);
    // Declining finds nothing.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Ashiok, Sculptor of Fears");
    t.answer_yes(P0, false);
    etb(&mut t, P0, "Ashiok's Forerunner");
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn theorists_proxy_empowers_jace_3() {
    cr!("603.6a", "701.71a");
    // "When this creature enters, empower Jace 3."
    supported("Theorist's Proxy");
    let mut t = TestGame::new(2);
    etb(&mut t, P0, "Theorist's Proxy");
    let jaces = subtype_ids(&t, P0, "Jace");
    assert_eq!(jaces.len(), 1);
    assert!(t.obj_now(jaces[0]).is_token());
    assert_eq!(t.counters(jaces[0], "loyalty"), 3);
    assert_eq!(count_subtype(&t, P1, "Jace"), 0);
    // A second one adds loyalty to the same Jace.
    etb(&mut t, P0, "Theorist's Proxy");
    assert_eq!(subtype_ids(&t, P0, "Jace"), jaces);
    assert_eq!(t.counters(jaces[0], "loyalty"), 6);
}

#[test]
fn foul_imp_makes_its_controller_lose_2_life() {
    cr!("603.6a", "119.3");
    // "When this creature enters, you lose 2 life."
    supported("Foul Imp");
    let mut t = TestGame::new(2);
    etb(&mut t, P1, "Foul Imp");
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn elite_guardmage_gains_3_and_draws() {
    cr!("603.6a", "119.3", "121.1");
    // "When this creature enters, you gain 3 life and draw a card."
    supported("Elite Guardmage");
    let mut t = TestGame::new(2);
    etb(&mut t, P0, "Elite Guardmage");
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P1), 0);
}

#[test]
fn blade_splicer_creates_a_3_3_golem() {
    cr!("603.6a", "111.1", "111.4");
    // "When this creature enters, create a 3/3 colorless Phyrexian Golem artifact creature
    // token."
    supported("Blade Splicer");
    let mut t = TestGame::new(2);
    etb(&mut t, P0, "Blade Splicer");
    let golems = subtype_ids(&t, P0, "Golem");
    assert_eq!(golems.len(), 1);
    let g = t.obj_now(golems[0]);
    assert!(g.is_token());
    assert!(g.is(types::CardType::Artifact) && g.is_creature());
    assert!(g.chars.has_subtype("Phyrexian"));
    assert!(g.chars.colors.is_colorless());
    assert_eq!(t.pt(golems[0]), (3, 3));
    assert_eq!(count_subtype(&t, P1, "Golem"), 0);
}

#[test]
fn gluttonous_guest_creates_a_blood_token() {
    cr!("603.6a", "111.10g");
    // "When this creature enters, create a Blood token."
    supported("Gluttonous Guest");
    let mut t = TestGame::new(2);
    etb(&mut t, P0, "Gluttonous Guest");
    assert_eq!(count_subtype(&t, P0, "Blood"), 1);
    assert_eq!(count_subtype(&t, P1, "Blood"), 0);
}

#[test]
fn nest_invader_creates_a_spawn_that_sacrifices_for_colorless() {
    cr!("603.6a", "111.4", "605.1a");
    // "When this creature enters, create a 0/1 colorless Eldrazi Spawn creature token. It
    // has "Sacrifice this token: Add {C}.""
    supported("Nest Invader");
    let mut t = TestGame::new(2);
    etb(&mut t, P0, "Nest Invader");
    let spawn = subtype_ids(&t, P0, "Spawn");
    assert_eq!(spawn.len(), 1);
    let s = spawn[0];
    assert!(t.obj_now(s).chars.has_subtype("Eldrazi"));
    assert!(t.obj_now(s).chars.colors.is_colorless());
    assert_eq!(t.pt(s), (0, 1));
    t.activate(P0, s, 0, &[]).unwrap();
    assert!(!t.on_battlefield(s), "sacrificed");
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::C), 1);
}

#[test]
fn veteran_dungeoneer_ventures_into_the_dungeon() {
    cr!("603.6a", "701.49a");
    // "When this creature enters, venture into the dungeon."
    supported("Veteran Dungeoneer");
    let mut t = TestGame::new(2);
    assert!(dungeons::marker(&t.g, P0).is_none());
    etb(&mut t, P0, "Veteran Dungeoneer");
    let (_, room) = dungeons::marker(&t.g, P0).expect("P0 entered a dungeon");
    assert_eq!(room, 0, "the topmost room");
    assert!(dungeons::marker(&t.g, P1).is_none());
}

#[test]
fn marshals_pathcruiser_tutors_a_basic_land_to_hand() {
    cr!("603.6a", "701.23a");
    // "When this Vehicle enters, search your library for a basic land card, reveal it, put
    // it into your hand, then shuffle."
    supported("Marshals' Pathcruiser");
    let mut t = TestGame::new(2);
    let bears = t.library_top(P0, "Grizzly Bears");
    let forest = t.library_top(P0, "Forest");
    // A nonland card isn't a legal choice: the answer falls back to the land.
    t.answer_choose(P0, &[Entity::Object(bears)]);
    etb(&mut t, P0, "Marshals' Pathcruiser");
    assert!(t.in_hand(P0, "Forest"));
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.zone(forest), Zone::Hand(P0));
    assert_eq!(t.library_size(P0), 31);
}

#[test]
fn spider_man_puts_a_basic_land_onto_the_battlefield_tapped() {
    cr!("603.6a", "701.23a");
    // "When Spider-Man enters, search your library for a basic land card, put it onto the
    // battlefield tapped, then shuffle."
    supported("Spider-Man, Brooklyn Visionary");
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    t.answer_choose(P0, &[Entity::Object(forest)]);
    etb(&mut t, P0, "Spider-Man, Brooklyn Visionary");
    let lands = t.named_on_battlefield("Forest");
    assert_eq!(lands.len(), 1);
    assert_eq!(t.obj_now(lands[0]).controller, P0);
    assert!(t.obj_now(lands[0]).tapped);
    assert_eq!(t.library_size(P0), 30);
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn dune_mover_puts_a_basic_land_on_top() {
    cr!("603.6a", "701.23a");
    // "When this creature enters, you may search your library for a basic land card,
    // reveal it, then shuffle and put that card on top."
    supported("Dune Mover");
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    t.g.players[P0.idx()].library.rotate_left(1);
    assert_ne!(t.g.players[P0.idx()].library.last(), Some(&forest));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    etb(&mut t, P0, "Dune Mover");
    let top = *t.g.players[P0.idx()].library.last().unwrap();
    assert_eq!(t.obj_now(top).chars.name, "Forest");
    assert_eq!(t.library_size(P0), 31);
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn equilibrium_adept_exiles_the_top_card_playable_until_end_of_next_turn() {
    cr!("603.6a", "611.2a");
    // "When this creature enters, exile the top card of your library. Until the end of your
    // next turn, you may play that card."
    supported("Equilibrium Adept");
    let mut t = TestGame::new(2);
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    etb(&mut t, P0, "Equilibrium Adept");
    assert_eq!(t.zone(bolt), Zone::Exile);
    // Still playable on the opponent's turn (it's an instant).
    t.advance_to(P1, Step::Upkeep);
    let bolt = t.g.current(bolt);
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);

    // After the end of P0's next turn, the card can't be played any more.
    let mut t = TestGame::new(2);
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    etb(&mut t, P0, "Equilibrium Adept");
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    let bolt = t.g.current(bolt);
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert!(t.cast(P0, bolt).target(P1).try_go().is_err());
    assert_eq!(t.life(P1), 20);
}

// ---------------------------------------------------------------------------
// "When ~ enters, ..." — targeted effects
// ---------------------------------------------------------------------------

#[test]
fn baleful_ammit_puts_a_minus_counter_on_a_creature_you_control() {
    cr!("603.6a", "115.1", "122.1");
    // "When this creature enters, put a -1/-1 counter on target creature you control."
    supported("Baleful Ammit");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Hill Giant");
    // The opponent's creature isn't a legal target: the answer falls back to a legal one.
    t.answer_targets(P0, &[Entity::Object(theirs)]);
    t.answer_targets(P0, &[Entity::Object(mine)]);
    etb(&mut t, P0, "Baleful Ammit");
    assert_eq!(t.counters(theirs, "-1/-1"), 0);
    let ammit = t.named_on_battlefield("Baleful Ammit")[0];
    assert_eq!(
        t.counters(mine, "-1/-1") + t.counters(ammit, "-1/-1"),
        1,
        "the counter goes on a creature P0 controls"
    );
}

#[test]
fn baleful_ammit_counter_shrinks_the_chosen_creature() {
    cr!("603.6a", "122.1", "613.4c");
    supported("Baleful Ammit");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    etb(&mut t, P0, "Baleful Ammit");
    assert_eq!(t.counters(giant, "-1/-1"), 1);
    assert_eq!(t.pt(giant), (2, 2));
}

#[test]
fn returned_centaur_makes_target_player_mill_four() {
    cr!("603.6a", "701.17a");
    // "When this creature enters, target player mills four cards."
    supported("Returned Centaur");
    let mut t = TestGame::new(2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    etb(&mut t, P0, "Returned Centaur");
    assert_eq!(t.graveyard_size(P1), 4);
    assert_eq!(t.library_size(P1), 26);
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.library_size(P0), 30);
}

#[test]
fn briarpack_alpha_pumps_target_creature_until_end_of_turn() {
    cr!("603.6a", "611.2a", "514.2");
    // "When this creature enters, target creature gets +2/+2 until end of turn."
    supported("Briarpack Alpha");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let alpha = etb(&mut t, P0, "Briarpack Alpha");
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.pt(other), (2, 2));
    assert_eq!(t.pt(alpha), (3, 3));
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn eyeblight_assassin_shrinks_an_opponents_creature_until_end_of_turn() {
    cr!("603.6a", "115.1", "611.2a");
    // "When this creature enters, target creature an opponent controls gets -1/-1 until end
    // of turn."
    supported("Eyeblight Assassin");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    // P0's own creature isn't a legal target.
    t.answer_targets(P0, &[Entity::Object(mine)]);
    etb(&mut t, P0, "Eyeblight Assassin");
    assert_eq!(t.pt(mine), (2, 2));
    assert_eq!(t.pt(theirs), (1, 1));
    next_turn(&mut t);
    assert_eq!(t.pt(theirs), (2, 2));
}

#[test]
fn humbling_elder_gives_an_opponents_creature_minus_2_power() {
    cr!("603.6a", "115.1", "611.2a");
    // "When this creature enters, target creature an opponent controls gets -2/-0 until
    // end of turn."
    supported("Humbling Elder");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(mine)]);
    etb(&mut t, P0, "Humbling Elder");
    assert_eq!(t.pt(mine), (3, 3));
    assert_eq!(t.pt(theirs), (1, 3));
    next_turn(&mut t);
    assert_eq!(t.pt(theirs), (3, 3));
}

#[test]
fn oath_of_chandra_deals_3_to_an_opponents_creature() {
    cr!("603.6a", "115.1", "120.3a");
    // "When Oath of Chandra enters, it deals 3 damage to target creature an opponent
    // controls."
    supported("Oath of Chandra");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(mine)]);
    etb(&mut t, P0, "Oath of Chandra");
    assert!(t.on_battlefield(mine));
    assert_eq!(t.obj_now(mine).damage, 0);
    assert!(!t.on_battlefield(theirs));
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn cloudchaser_kestrel_destroys_target_enchantment() {
    cr!("603.6a", "115.1", "701.8a");
    // "When this creature enters, destroy target enchantment."
    supported("Cloudchaser Kestrel");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ench = t.battlefield(P1, "Orcish Oriflamme");
    // A creature isn't a legal target.
    t.answer_targets(P0, &[Entity::Object(bears)]);
    etb(&mut t, P0, "Cloudchaser Kestrel");
    assert!(t.on_battlefield(bears));
    assert!(!t.on_battlefield(ench));
    assert!(t.in_graveyard(P1, "Orcish Oriflamme"));
}

#[test]
fn loran_destroys_up_to_one_artifact_or_enchantment() {
    cr!("603.6a", "115.1", "601.2c", "701.8a");
    // "When Loran enters, destroy up to one target artifact or enchantment."
    supported("Loran of the Third Path");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let art = t.battlefield(P1, "Ornithopter");
    let ench = t.battlefield(P1, "Orcish Oriflamme");
    t.answer_targets(P0, &[Entity::Object(art)]);
    etb(&mut t, P0, "Loran of the Third Path");
    assert!(!t.on_battlefield(art));
    assert!(t.on_battlefield(ench));
    assert!(t.on_battlefield(bears));
    // Zero targets is fine.
    let mut t = TestGame::new(2);
    let ench = t.battlefield(P1, "Orcish Oriflamme");
    t.answer_targets(P0, &[]);
    let loran = etb(&mut t, P0, "Loran of the Third Path");
    assert!(t.on_battlefield(ench));
    assert!(t.on_battlefield(loran));
}

#[test]
fn ravenous_rats_makes_target_opponent_discard() {
    cr!("603.6a", "701.9a");
    // "When this creature enters, target opponent discards a card."
    supported("Ravenous Rats");
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    t.hand(P1, "Lightning Bolt");
    t.hand(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    etb(&mut t, P0, "Ravenous Rats");
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(t.graveyard_size(P1), 1);
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn sibsig_muckdraggers_returns_a_creature_card_from_your_graveyard() {
    cr!("603.6a", "115.1");
    // "When this creature enters, return target creature card from your graveyard to your
    // hand."
    supported("Sibsig Muckdraggers");
    let mut t = TestGame::new(2);
    let theirs = t.graveyard(P1, "Hill Giant");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let mine = t.graveyard(P0, "Grizzly Bears");
    // Neither a card in an opponent's graveyard nor a noncreature card is legal.
    t.answer_targets(P0, &[Entity::Object(theirs)]);
    etb(&mut t, P0, "Sibsig Muckdraggers");
    assert_eq!(t.zone(mine), Zone::Hand(P0));
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0));
    assert_eq!(t.zone(theirs), Zone::Graveyard(P1));
}

#[test]
fn stromkirk_mentor_counters_another_vampire_you_control() {
    cr!("603.6a", "115.1", "122.6");
    // "When this creature enters, put a +1/+1 counter on another target Vampire you
    // control."
    supported("Stromkirk Mentor");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let their_vampire = t.battlefield(P1, "Gluttonous Guest");
    let vampire = t.battlefield(P0, "Gluttonous Guest");
    // Neither a non-Vampire nor an opponent's Vampire is legal (nor the Mentor itself,
    // though it's a Vampire).
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let mentor = etb(&mut t, P0, "Stromkirk Mentor");
    assert_eq!(t.counters(vampire, "+1/+1"), 1);
    assert_eq!(t.pt(vampire), (2, 5));
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    assert_eq!(t.counters(their_vampire, "+1/+1"), 0);
    assert_eq!(t.counters(mentor, "+1/+1"), 0);
}

#[test]
fn stromkirk_mentor_without_another_vampire_does_nothing() {
    cr!("603.3d");
    supported("Stromkirk Mentor");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let mentor = etb(&mut t, P0, "Stromkirk Mentor");
    assert_eq!(t.counters(mentor, "+1/+1"), 0);
    assert_eq!(t.counters(bears, "+1/+1"), 0);
}

// ---------------------------------------------------------------------------
// "When ~ enters, ..." — mass effects
// ---------------------------------------------------------------------------

#[test]
fn inspiring_captain_pumps_your_creatures_until_end_of_turn() {
    cr!("603.6a", "611.2c", "514.2");
    // "When this creature enters, creatures you control get +1/+1 until end of turn."
    supported("Inspiring Captain");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let captain = etb(&mut t, P0, "Inspiring Captain");
    assert_eq!(t.pt(mine), (3, 3));
    assert_eq!(t.pt(captain), (4, 4));
    assert_eq!(t.pt(theirs), (2, 2));
    // Locked in: a creature entering later isn't affected (CR 611.2c).
    let late = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(late), (3, 3));
    next_turn(&mut t);
    assert_eq!(t.pt(mine), (2, 2));
    assert_eq!(t.pt(captain), (3, 3));
}

#[test]
fn supply_runners_counters_each_other_creature_you_control() {
    cr!("603.6a", "122.6");
    // "When this creature enters, put a +1/+1 counter on each other creature you control."
    supported("Supply Runners");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let runners = etb(&mut t, P0, "Supply Runners");
    assert_eq!(t.counters(a, "+1/+1"), 1);
    assert_eq!(t.counters(b, "+1/+1"), 1);
    assert_eq!(t.counters(theirs, "+1/+1"), 0);
    assert_eq!(t.counters(runners, "+1/+1"), 0);
}

// ---------------------------------------------------------------------------
// Dies triggers
// ---------------------------------------------------------------------------

#[test]
fn exquisite_huntmaster_makes_an_elf_warrior_when_it_dies() {
    cr!("603.6c", "700.4", "111.4");
    // "When this creature dies, create a 1/1 green Elf Warrior creature token."
    supported("Exquisite Huntmaster");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Exquisite Huntmaster");
    t.g.destroy(h, None);
    t.resolve_all();
    let elves = subtype_ids(&t, P0, "Elf");
    assert_eq!(elves.len(), 1);
    let e = elves[0];
    assert!(t.obj_now(e).is_token());
    assert!(t.obj_now(e).chars.has_subtype("Warrior"));
    assert_eq!(t.obj_now(e).chars.colors, types::ColorSet::single(types::Color::Green));
    assert_eq!(t.pt(e), (1, 1));
    assert_eq!(count_subtype(&t, P1, "Elf"), 0);
}

#[test]
fn guul_draz_mucklord_counters_a_creature_you_control_when_it_dies() {
    cr!("603.6c", "603.10a", "115.1");
    // "When this creature dies, put a +1/+1 counter on target creature you control."
    supported("Guul Draz Mucklord");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let m = t.battlefield(P0, "Guul Draz Mucklord");
    t.answer_targets(P0, &[Entity::Object(theirs)]);
    t.g.destroy(m, None);
    t.resolve_all();
    assert_eq!(t.counters(theirs, "+1/+1"), 0);
    assert_eq!(t.counters(mine, "+1/+1"), 1);
    assert_eq!(t.pt(mine), (3, 3));
}

#[test]
fn merfolk_seer_may_pay_to_draw_when_it_dies() {
    cr!("603.6c", "118.12");
    // "When this creature dies, you may pay {1}{U}. If you do, draw a card."
    supported("Merfolk Seer");
    for pay in [true, false] {
        let mut t = TestGame::new(2);
        let islands = t.lands(P0, "Island", 2);
        let s = t.battlefield(P0, "Merfolk Seer");
        t.answer_yes(P0, pay);
        t.g.destroy(s, None);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), pay as usize, "pay: {pay}");
        assert_eq!(
            islands.iter().filter(|i| t.obj_now(**i).tapped).count(),
            if pay { 2 } else { 0 }
        );
    }
    // Without the mana, nothing is drawn.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Merfolk Seer");
    t.answer_yes(P0, true);
    t.g.destroy(s, None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn unruly_mob_grows_when_another_creature_you_control_dies() {
    cr!("603.6c", "603.10a", "122.6");
    // "Whenever another creature you control dies, put a +1/+1 counter on this creature."
    supported("Unruly Mob");
    let mut t = TestGame::new(2);
    let mob = t.battlefield(P0, "Unruly Mob");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(theirs, None);
    t.resolve_all();
    assert_eq!(t.counters(mob, "+1/+1"), 0);
    t.g.destroy(mine, None);
    t.resolve_all();
    assert_eq!(t.counters(mob, "+1/+1"), 1);
    assert_eq!(t.pt(mob), (2, 2));
}

#[test]
fn skullclamp_draws_two_when_the_equipped_creature_dies() {
    cr!("603.6c", "603.10a", "301.5");
    // "Equipped creature gets +1/-1. / Whenever equipped creature dies, draw two cards."
    supported("Skullclamp");
    let mut t = TestGame::new(2);
    let clamp = t.battlefield(P0, "Skullclamp");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    t.g.attach(clamp, Entity::Object(bears));
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 1));
    // An unequipped creature dying draws nothing.
    t.g.destroy(other, None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    t.g.destroy(bears, None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.hand_size(P1), 0);
    // A 1/1 dies to the -1 toughness as soon as it's equipped.
    let elf = t.battlefield(P0, "Llanowar Elves");
    t.g.attach(clamp, Entity::Object(elf));
    t.resolve_all();
    assert!(!t.on_battlefield(elf));
    assert_eq!(t.hand_size(P0), 4);
}

// ---------------------------------------------------------------------------
// Other creature/land enters
// ---------------------------------------------------------------------------

#[test]
fn griffin_protector_grows_when_another_creature_you_control_enters() {
    cr!("603.6a", "611.2a");
    // "Whenever another creature you control enters, this creature gets +1/+1 until end of
    // turn."
    supported("Griffin Protector");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Griffin Protector");
    etb(&mut t, P1, "Grizzly Bears");
    assert_eq!(t.pt(g), (2, 3));
    etb(&mut t, P0, "Grizzly Bears");
    assert_eq!(t.pt(g), (3, 4));
    etb(&mut t, P0, "Forest");
    assert_eq!(t.pt(g), (3, 4), "a land isn't a creature");
    next_turn(&mut t);
    assert_eq!(t.pt(g), (2, 3));
}

#[test]
fn remnant_elemental_landfall_gives_plus_2_power() {
    cr!("603.6a", "305.1", "611.2a");
    // "Landfall — Whenever a land you control enters, this creature gets +2/+0 until end
    // of turn."
    supported("Remnant Elemental");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Remnant Elemental");
    etb(&mut t, P1, "Forest");
    assert_eq!(t.pt(r), (0, 4));
    let land = t.hand(P0, "Mountain");
    t.play_land(P0, land).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(r), (2, 4));
    next_turn(&mut t);
    assert_eq!(t.pt(r), (0, 4));
}

#[test]
fn teeming_dragonstorm_returns_to_hand_when_your_dragon_enters() {
    cr!("603.6a", "400.7");
    // "When a Dragon you control enters, return this enchantment to its owner's hand."
    supported("Teeming Dragonstorm");
    let mut t = TestGame::new(2);
    let storm = t.battlefield(P0, "Teeming Dragonstorm");
    etb(&mut t, P1, "Shivan Dragon");
    etb(&mut t, P0, "Grizzly Bears");
    assert!(t.on_battlefield(storm));
    etb(&mut t, P0, "Shivan Dragon");
    assert_eq!(t.zone(storm), Zone::Hand(P0));
}

// ---------------------------------------------------------------------------
// Spell-cast triggers
// ---------------------------------------------------------------------------

#[test]
fn spellgorger_weird_grows_on_noncreature_spells() {
    cr!("603.2", "601.2i", "122.6");
    // "Whenever you cast a noncreature spell, put a +1/+1 counter on this creature."
    supported("Spellgorger Weird");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Spellgorger Weird");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 2);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.counters(w, "+1/+1"), 1);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.counters(w, "+1/+1"), 1, "a creature spell doesn't count");
    // The opponent's spells don't count.
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.resolve_all();
    assert_eq!(t.counters(w, "+1/+1"), 1);
    assert_eq!(t.pt(w), (3, 3));
}

#[test]
fn safehold_duo_grows_when_you_cast_a_green_spell() {
    cr!("603.2", "611.2a");
    // "Whenever you cast a green spell, this creature gets +1/+1 until end of turn."
    supported("Safehold Duo");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Safehold Duo");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.pt(d), (2, 4));
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.pt(d), (3, 5));
    next_turn(&mut t);
    assert_eq!(t.pt(d), (2, 4));
}

#[test]
fn nettle_sentinel_may_untap_when_you_cast_a_green_spell() {
    cr!("603.2", "701.26b");
    // "Whenever you cast a green spell, you may untap this creature."
    supported("Nettle Sentinel");
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Nettle Sentinel");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 4);
    t.g.tap(n);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert!(t.obj_now(n).tapped, "a red spell doesn't trigger it");
    // Declining leaves it tapped.
    t.answer_yes(P0, false);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    assert!(t.obj_now(n).tapped);
    t.answer_yes(P0, true);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    assert!(!t.obj_now(n).tapped);
}

#[test]
fn krakens_eye_may_gain_1_when_any_player_casts_a_blue_spell() {
    cr!("603.2", "119.3");
    // "Whenever a player casts a blue spell, you may gain 1 life."
    supported("Kraken's Eye");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kraken's Eye");
    t.lands(P1, "Island", 1);
    t.lands(P0, "Mountain", 1);
    t.answer_yes(P0, true);
    let opt = t.hand(P1, "Opt");
    t.cast(P1, opt).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.life(P1), 20);
    t.answer_yes(P0, true);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 21, "a red spell doesn't trigger it");
    assert_eq!(t.life(P1), 17);
}

#[test]
fn wooden_sphere_may_pay_1_to_gain_1_when_any_player_casts_a_green_spell() {
    cr!("603.2", "118.12", "119.3");
    // "Whenever a player casts a green spell, you may pay {1}. If you do, you gain 1 life."
    supported("Wooden Sphere");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wooden Sphere");
    let plains = t.lands(P0, "Plains", 1)[0];
    t.lands(P1, "Forest", 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    let growth = t.hand(P1, "Giant Growth");
    t.cast(P1, growth).target(bears).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.life(P1), 20);
    assert!(t.obj_now(plains).tapped, "paid {{1}}");
    // Without mana to pay, nothing happens.
    t.lands(P1, "Forest", 1);
    t.answer_yes(P0, true);
    let growth = t.hand(P1, "Giant Growth");
    t.cast(P1, growth).target(bears).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    // A nongreen spell doesn't trigger it.
    t.lands(P0, "Mountain", 2);
    t.answer_yes(P0, true);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn aurora_eidolon_returns_from_graveyard_when_you_cast_a_multicolored_spell() {
    cr!("603.2", "603.6", "105.4");
    // "Whenever you cast a multicolored spell, you may return this card from your
    // graveyard to your hand."
    supported("Aurora Eidolon");
    let mut t = TestGame::new(2);
    let e = t.graveyard(P0, "Aurora Eidolon");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Plains", 1);
    t.answer_yes(P0, true);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.zone(e), Zone::Graveyard(P0), "a monocolored spell");
    let helix = t.hand(P0, "Lightning Helix");
    t.answer_yes(P0, true);
    t.cast(P0, helix).target(P1).go();
    t.resolve_all();
    assert_eq!(t.zone(e), Zone::Hand(P0));
    assert_eq!(t.life(P1), 14);
}

// ---------------------------------------------------------------------------
// Upkeep and draw-step triggers
// ---------------------------------------------------------------------------

#[test]
fn grinning_demon_makes_its_controller_lose_2_each_upkeep() {
    cr!("503.1a", "603.2", "119.3");
    // "At the beginning of your upkeep, you lose 2 life."
    supported("Grinning Demon");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grinning Demon");
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "not on the opponent's upkeep");
}

#[test]
fn vilespawn_spider_mills_a_card_each_upkeep() {
    cr!("503.1a", "701.17a");
    // "At the beginning of your upkeep, mill a card."
    supported("Vilespawn Spider");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Vilespawn Spider");
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 1);
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn skull_collector_returns_a_black_creature_you_control() {
    cr!("503.1a", "603.2");
    // "At the beginning of your upkeep, return a black creature you control to its owner's
    // hand."
    supported("Skull Collector");
    let mut t = TestGame::new(2);
    let sc = t.battlefield(P1, "Skull Collector");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let rats = t.battlefield(P0, "Ravenous Rats");
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    // The only black creature P1 controls is the Collector itself.
    assert_eq!(t.zone(sc), Zone::Hand(P1));
    assert!(t.on_battlefield(bears));
    assert!(t.on_battlefield(rats));
}

#[test]
fn parasitic_bond_damages_the_enchanted_creatures_controller_each_upkeep() {
    cr!("503.1a", "603.2", "120.3a");
    // "At the beginning of the upkeep of enchanted creature's controller, this Aura deals
    // 2 damage to that player."
    supported("Parasitic Bond");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let bond = t.battlefield(P0, "Parasitic Bond");
    t.g.attach(bond, Entity::Object(bears));
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "not on the Aura controller's upkeep");
    assert_eq!(t.life(P1), 18);
}

#[test]
fn rites_of_flourishing_gives_each_player_an_extra_draw() {
    cr!("504.1", "603.2", "121.1");
    // "At the beginning of each player's draw step, that player draws an additional card."
    supported("Rites of Flourishing");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rites of Flourishing");
    t.advance_to(P1, Step::Draw);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 2);
    assert_eq!(t.hand_size(P0), 0);
}

// ---------------------------------------------------------------------------
// Combat triggers
// ---------------------------------------------------------------------------

#[test]
fn deeproot_warrior_grows_when_blocked() {
    cr!("509.3c", "611.2a");
    // "Whenever this creature becomes blocked, it gets +1/+1 until end of turn."
    supported("Deeproot Warrior");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Deeproot Warrior");
    let bears = t.battlefield(P1, "Grizzly Bears");
    fight(&mut t, &[w], &[(bears, w)]);
    assert_eq!(t.pt(w), (3, 3));
    assert!(t.on_battlefield(w), "survives 2 damage as a 3/3");
    assert!(!t.on_battlefield(bears));
    next_turn(&mut t);
    assert_eq!(t.pt(w), (2, 2));
    // Unblocked, it doesn't grow.
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Deeproot Warrior");
    fight(&mut t, &[w], &[]);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.pt(w), (2, 2));
}

#[test]
fn charging_griffin_gets_plus_one_when_it_attacks() {
    cr!("508.1m", "611.2a");
    // "Whenever this creature attacks, it gets +1/+1 until end of turn."
    supported("Charging Griffin");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Charging Griffin");
    let other = t.battlefield(P0, "Charging Griffin");
    fight(&mut t, &[g], &[]);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.pt(g), (3, 3));
    assert_eq!(t.pt(other), (2, 2));
    next_turn(&mut t);
    assert_eq!(t.pt(g), (2, 2));
}

#[test]
fn nightveil_sprite_surveils_when_it_attacks() {
    cr!("508.1m", "701.25a");
    // "Whenever this creature attacks, surveil 1."
    supported("Nightveil Sprite");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Nightveil Sprite");
    assert!(surveils(&t).is_empty());
    fight(&mut t, &[s], &[]);
    assert_eq!(surveils(&t), vec![1]);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn aether_chaser_may_pay_two_energy_for_a_servo_when_it_attacks() {
    cr!("508.1m", "107.14", "118.12");
    // "Whenever this creature attacks, you may pay {E}{E}. If you do, create a 1/1
    // colorless Servo artifact creature token."
    supported("Aether Chaser");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Aether Chaser");
    t.g.players[P0.idx()]
        .counters
        .insert(types::counters::ENERGY.into(), 3);
    t.answer_yes(P0, true);
    fight(&mut t, &[c], &[]);
    let servos = subtype_ids(&t, P0, "Servo");
    assert_eq!(servos.len(), 1);
    assert_eq!(t.pt(servos[0]), (1, 1));
    assert!(t.obj_now(servos[0]).is(types::CardType::Artifact));
    assert_eq!(energy(&t, P0), 1);
    // With only one energy, no Servo.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Aether Chaser");
    t.g.players[P0.idx()]
        .counters
        .insert(types::counters::ENERGY.into(), 1);
    t.answer_yes(P0, true);
    fight(&mut t, &[c], &[]);
    assert_eq!(count_subtype(&t, P0, "Servo"), 0);
    assert_eq!(energy(&t, P0), 1);
}

#[test]
fn wharf_infiltrator_may_loot_on_combat_damage_to_a_player() {
    cr!("510.3a", "603.2", "701.9a");
    // "Whenever this creature deals combat damage to a player, you may draw a card. If you
    // do, discard a card."
    supported("Wharf Infiltrator");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Wharf Infiltrator");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    fight(&mut t, &[w], &[]);
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.library_size(P0), 29);
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0));
    // Declining: no draw, no discard.
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Wharf Infiltrator");
    t.hand(P0, "Lightning Bolt");
    t.answer_yes(P0, false);
    fight(&mut t, &[w], &[]);
    assert_eq!(t.library_size(P0), 30);
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.graveyard_size(P0), 0);
}

// ---------------------------------------------------------------------------
// Mana triggers
// ---------------------------------------------------------------------------

#[test]
fn verdant_haven_adds_one_mana_of_any_color_for_the_lands_controller() {
    cr!("605.1b", "605.4a", "106.12a");
    // "Whenever enchanted land is tapped for mana, its controller adds an additional one
    // mana of any color."
    supported("Verdant Haven");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    let other = t.battlefield(P1, "Forest");
    let haven = t.battlefield(P0, "Verdant Haven");
    t.g.attach(haven, Entity::Object(forest));
    t.g.recompute();
    t.activate(P1, other, 0, &[]).unwrap();
    assert_eq!(t.g.player(P1).mana_pool.mana.len(), 1, "unenchanted land");
    t.activate(P1, forest, 0, &[]).unwrap();
    t.settle();
    assert_eq!(t.g.player(P1).mana_pool.mana.len(), 3);
    assert_eq!(t.g.player(P1).mana_pool.count(ManaType::C), 0);
    assert_eq!(t.g.player(P0).mana_pool.mana.len(), 0);
    assert_eq!(t.stack_len(), 0, "a mana ability doesn't use the stack");
}
