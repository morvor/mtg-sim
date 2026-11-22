//! "Choose [targets]." as a sentence of its own (CR 115.1, 601.2c): spells, several
//! targets of one instance of "target", and requirements on them taken together
//! ("controlled by the same player", "controlled by different players", CR 115.3). Later
//! sentences refer to them; "one of them" choices leave "the other".

use mtg_engine::testing::*;
use mtg_engine::object::Zone;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

fn e(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[test]
fn run_away_together_needs_creatures_of_different_controllers() {
    cr!("115.1", "601.2c");
    compiles("Run Away Together");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    let c = t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Run Away Together");
    t.cast(P0, spell).targets(&[e(a), e(c)]).go();
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Hand(P0));
    assert_eq!(t.zone(c), Zone::Hand(P1));
    assert!(t.on_battlefield(b));

    // Two creatures of one controller aren't a legal choice: with no other creatures,
    // the spell can't be cast.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Llanowar Elves");
    let spell = t.hand(P0, "Run Away Together");
    assert!(t.cast(P0, spell).try_go().is_err());
}

#[test]
fn incriminate_that_player_sacrifices_one_of_their_choice() {
    cr!("701.21a", "608.2d");
    compiles("Incriminate");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Llanowar Elves");
    let spell = t.hand(P0, "Incriminate");
    t.cast(P0, spell).targets(&[e(a), e(b)]).go();
    // The creatures' controller chooses which one.
    t.answer_choose(P1, &[e(b)]);
    t.resolve_all();
    assert!(t.on_battlefield(a));
    assert!(t.in_graveyard(P1, "Llanowar Elves"));

    // Creatures of different controllers aren't a legal choice.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Llanowar Elves");
    let spell = t.hand(P0, "Incriminate");
    assert!(t.cast(P0, spell).try_go().is_err());
}

#[test]
fn barrins_spite_sacrifice_one_return_the_other() {
    cr!("701.21a", "608.2d");
    compiles("Barrin's Spite");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Llanowar Elves");
    let spell = t.hand(P0, "Barrin's Spite");
    t.cast(P0, spell).targets(&[e(a), e(b)]).go();
    t.answer_choose(P1, &[e(a)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_hand(P1, "Llanowar Elves"));
}

#[test]
fn trial_of_agony_damage_to_the_chosen_and_the_other_cant_block() {
    cr!("608.2d", "509.1b");
    compiles("Trial of Agony");
    for blocks in [false, true] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Mountain", 1);
        let attacker = t.battlefield(P0, "Hill Giant");
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P1, "Llanowar Elves");
        let spell = t.hand(P0, "Trial of Agony");
        t.cast(P0, spell).targets(&[e(a), e(b)]).go();
        t.answer_choose(P1, &[e(b)]);
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Llanowar Elves"));
        assert!(t.on_battlefield(a));
        // The other can't block: the attack is unblocked.
        let blockers: &[(ObjectId, ObjectId)] = if blocks { &[(a, attacker)] } else { &[] };
        t.attack(&[(attacker, Entity::Player(P1))], blockers);
        assert_eq!(t.life(P1), 17);
    }
}

#[test]
fn trial_of_agony_needs_creatures_of_one_opponent() {
    cr!("115.1");
    let mut t = TestGame::new(3);
    t.lands(P0, "Mountain", 1);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P2, "Llanowar Elves");
    let spell = t.hand(P0, "Trial of Agony");
    assert!(t.cast(P0, spell).targets(&[e(a), e(b)]).try_go().is_err());
}

#[test]
fn swallowed_by_leviathan_counters_the_chosen_spell_unless_paid() {
    cr!("115.1", "701.6a");
    compiles("Swallowed by Leviathan");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.lands(P1, "Mountain", 1);
    t.graveyard(P0, "Grizzly Bears");
    let bolt = t.hand(P1, "Lightning Bolt");
    t.g.turn.priority = Some(P1);
    let spell = t.cast(P1, bolt).target(P0).go();
    let sw = t.hand(P0, "Swallowed by Leviathan");
    t.g.turn.priority = Some(P0);
    t.cast(P0, sw).target(spell).go();
    t.resolve();
    // P1 has no untapped mana left to pay {1} per card in P0's graveyard.
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Lightning Bolt"), "{}", t.dump_log());
    assert_eq!(t.life(P0), 20);
}

#[test]
fn teferi_who_slows_the_sunset_untaps_yours_and_taps_theirs() {
    cr!("115.1", "601.2c", "606.3");
    let t0 = card("Teferi, Who Slows the Sunset");
    assert!(
        t0.unsupported_text().iter().all(|u| !u.contains("+1")),
        "{:?}",
        t0.unsupported_text()
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    let teferi = t.battlefield(P0, "Teferi, Who Slows the Sunset");
    let mine = t.battlefield(P0, "Ornithopter");
    t.g.obj_mut(mine).tapped = true;
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let land = t.battlefield(P1, "Forest");
    t.answer_targets(P0, &[e(mine)]);
    t.answer_targets(P0, &[e(theirs)]);
    t.answer_targets(P0, &[e(land)]);
    t.activate(P0, teferi, 0, &[]).expect("activates");
    t.resolve_all();
    assert!(!t.obj_now(mine).tapped, "{}", t.dump_log());
    assert!(t.obj_now(theirs).tapped);
    assert!(t.obj_now(land).tapped);
    assert_eq!(t.life(P0), 22);
}
