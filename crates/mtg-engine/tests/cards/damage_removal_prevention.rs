//! Prevention "to"/"by" objects and groups, static prevention, and "would die, exile it
//! instead" statics (patterns in `src/oracle/patterns/damage_removal_prevention.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn prevention_cards_compile() {
    assert_compiles(&[
        "Dawn Elemental",
        "Fog Bank",
        "Sandskin",
        "Inviolability",
        "Demonic Torment",
        "Maze of Ith",
        "Ethereal Haze",
        "Defend the Hearth",
        "Hunter's Ambush",
        "Blinding Fog",
        "Incendiary Oracle",
        "Bubble Matrix",
        "Champion Lancer",
        "Uncle Istvan",
        "Kor Haven",
        "Moonlight Geist",
        "Emmara Tandris",
        "Possessed Skaab",
        "Ebony Horse",
    ]);
}

fn bolt(t: &mut TestGame, target: impl Into<Entity>) {
    t.lands(P1, "Mountain", 1);
    let b = t.hand(P1, "Lightning Bolt");
    t.cast(P1, b).target(target).go();
    t.resolve();
}

#[test]
fn dawn_elemental_prevents_all_damage_to_itself() {
    cr!("615.1a");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Dawn Elemental");
    bolt(&mut t, e);
    assert!(t.on_battlefield(e));
    assert_eq!(t.obj_now(e).damage, 0);
}

#[test]
fn fog_bank_prevents_combat_damage_to_and_by_it_but_not_other_damage() {
    cr!("615.1a", "120.2a", "120.2b");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let fog = t.battlefield(P1, "Fog Bank");
    t.set_step(P0, Step::BeginningOfCombat);
    // Give the 0/2 Fog Bank some power so the damage it would deal is visible.
    t.lands(P1, "Forest", 1);
    let growth = t.hand(P1, "Giant Growth");
    t.cast(P1, growth).target(fog).go();
    t.resolve();
    assert_eq!(t.pt(fog), (3, 5));
    t.attack(&[(bears, Entity::Player(P1))], &[(fog, bears)]);
    assert!(t.on_battlefield(fog));
    assert_eq!(t.obj_now(fog).damage, 0);
    // The 3 combat damage Fog Bank would deal to the Bears is prevented too.
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
    // Noncombat damage isn't prevented (the pumped Fog Bank is a 3/5 this turn).
    bolt(&mut t, fog);
    assert_eq!(t.obj_now(fog).damage, 3);
}

#[test]
fn sandskin_prevents_combat_damage_to_and_by_the_enchanted_creature() {
    cr!("615.1a", "303.4");
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    let skin = t.battlefield(P0, "Sandskin");
    assert!(t.g.attach(skin, Entity::Object(attacker)));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(attacker, Entity::Player(P1))], &[(blocker, attacker)]);
    assert!(t.on_battlefield(attacker));
    assert!(t.on_battlefield(blocker));
    assert_eq!(t.obj_now(attacker).damage, 0);
    assert_eq!(t.obj_now(blocker).damage, 0);
}

#[test]
fn maze_of_ith_untaps_and_neutralizes_an_attacker() {
    cr!("615.1a", "506.4");
    // (Untapping an attacking creature doesn't remove it from combat.)
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let maze = t.battlefield(P1, "Maze of Ith");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    assert!(t.obj_now(bears).tapped);
    t.activate(P1, maze, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    assert!(!t.obj_now(bears).tapped);
    assert!(t.g.is_attacking(bears));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn ethereal_haze_prevents_damage_dealt_by_creatures_only() {
    cr!("615.1a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Plains", 1);
    let haze = t.hand(P1, "Ethereal Haze");
    t.cast(P1, haze).go();
    t.resolve();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20);
    // A spell isn't a creature.
    t.lands(P0, "Mountain", 1);
    let b = t.hand(P0, "Lightning Bolt");
    t.cast(P0, b).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn defend_the_hearth_prevents_combat_damage_to_players() {
    cr!("615.1a", "120.2a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Llanowar Elves");
    let other = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Forest", 2);
    let d = t.hand(P1, "Defend the Hearth");
    t.cast(P1, d).go();
    t.resolve();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(
        &[(bears, Entity::Player(P1)), (other, Entity::Player(P1))],
        &[(blocker, other)],
    );
    assert_eq!(t.life(P1), 20);
    // Combat damage to creatures still happens.
    assert!(!t.on_battlefield(blocker));
}

#[test]
fn bubble_matrix_prevents_damage_to_creatures_but_not_players() {
    cr!("615.1a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bubble Matrix");
    let bears = t.battlefield(P0, "Grizzly Bears");
    bolt(&mut t, bears);
    assert!(t.on_battlefield(bears));
    bolt(&mut t, P0);
    assert_eq!(t.life(P0), 17, "players aren't protected");
}

#[test]
fn champion_lancer_prevents_damage_from_creatures_only() {
    cr!("615.1a");
    let mut t = TestGame::new(2);
    let lancer = t.battlefield(P0, "Champion Lancer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P0))], &[(lancer, bears)]);
    assert_eq!(t.obj_now(lancer).damage, 0);
    assert!(!t.on_battlefield(bears));
    bolt(&mut t, lancer);
    assert!(!t.on_battlefield(lancer), "3 damage from a spell to a 3/3");
}

#[test]
fn possessed_skaab_is_exiled_instead_of_dying() {
    cr!("614.1a", "700.4");
    let mut t = TestGame::new(2);
    let skaab = t.battlefield(P0, "Possessed Skaab");
    bolt(&mut t, skaab);
    assert!(!t.on_battlefield(skaab));
    assert!(t.in_exile("Possessed Skaab"));
    assert!(!t.in_graveyard(P0, "Possessed Skaab"));
}

#[test]
fn incendiary_oracle_exiles_creatures_it_dealt_damage_this_turn() {
    cr!("614.1a", "700.4");
    let mut t = TestGame::new(2);
    let oracle = t.battlefield(P0, "Incendiary Oracle");
    let blocker = t.battlefield(P1, "Llanowar Elves");
    let other = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(oracle, Entity::Player(P1))], &[(blocker, oracle)]);
    // The blocker was dealt damage by the Oracle and died: exiled.
    assert!(!t.on_battlefield(blocker));
    assert!(t.in_exile("Llanowar Elves"));
    assert!(t.on_battlefield(oracle), "the 2/2 Oracle survives the Elves' 1 damage");
    // With the Oracle still around, a creature it didn't deal damage to goes to the
    // graveyard as usual.
    bolt(&mut t, other);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(!t.in_exile("Grizzly Bears"));
}

// ---------------------------------------------------------------------------
// Damage that can't be prevented (CR 615.12)
// ---------------------------------------------------------------------------

#[test]
fn cant_be_prevented_cards_compile() {
    assert_compiles(&[
        "Leyline of Punishment",
        "Pinpoint Avalanche",
        "Combust",
        "Excruciator",
    ]);
}

#[test]
fn combusts_damage_ignores_prevention_shields() {
    cr!("615.12");
    let mut t = TestGame::new(2);
    let healer = t.battlefield(P1, "Master Healer");
    let angel = t.battlefield(P1, "Serra Angel");
    t.activate(P1, healer, 0, &[Entity::Object(angel)]).unwrap();
    t.resolve();
    t.lands(P0, "Mountain", 2);
    let c = t.hand(P0, "Combust");
    t.cast(P0, c).target(angel).go();
    t.resolve();
    assert!(!t.on_battlefield(angel));
}

#[test]
fn only_the_spells_own_damage_is_unpreventable() {
    cr!("615.12", "615.1a");
    let mut t = TestGame::new(2);
    let healer = t.battlefield(P1, "Master Healer");
    let angel = t.battlefield(P1, "Serra Angel");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let c = t.hand(P0, "Combust");
    t.cast(P0, c).target(angel).go();
    t.resolve();
    // After Combust, other damage is prevented as usual.
    t.activate(P1, healer, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    let b = t.hand(P0, "Lightning Bolt");
    t.cast(P0, b).target(bears).go();
    t.resolve();
    assert!(t.on_battlefield(bears));
}

#[test]
fn leyline_of_punishment_stops_all_prevention() {
    cr!("615.12");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Punishment");
    let healer = t.battlefield(P1, "Master Healer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P1, healer, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    bolt(&mut t, bears);
    assert!(!t.on_battlefield(bears));
}

#[test]
fn excruciators_damage_ignores_prevention_without_using_up_the_shield() {
    cr!("615.12");
    let mut t = TestGame::new(2);
    let ex = t.battlefield(P0, "Excruciator");
    let healer = t.battlefield(P1, "Master Healer");
    // Prevent the next 4 damage that would be dealt to P1 this turn.
    t.activate(P1, healer, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(ex, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 13, "all 7 damage is dealt");
    // The shield wasn't reduced: it still prevents damage from another source.
    t.lands(P0, "Mountain", 1);
    let b = t.hand(P0, "Lightning Bolt");
    t.cast(P0, b).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 13);
}
