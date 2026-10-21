//! Where the mana spent on a spell came from (cards3-costs-mana-replacements,
//! `oracle/patterns/mana_4.rs`, `kw/cast_history.rs`): "for each mana from a Cave spent to
//! cast it", "for each mana from a Desert spent to cast ~", "for each mana from a Treasure
//! that was spent to cast ~", "enter with an additional +1/+1 counter on them for each mana
//! from a Treasure spent to cast them", "... from an artifact source ...", "Whenever you
//! cast a permanent spell using mana produced by ~", and "If that mana is spent on an
//! instant or sorcery spell, that spell can't be countered."

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn treasure(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let spec = mtg_engine::tokens::predefined("Treasure").expect("Treasure token");
    let tc = mtg_engine::replacement::TokenCreate {
        chars: mtg_engine::tokens::token_characteristics(&spec),
        card: None,
        tapped: false,
        attacking: None,
        copy_of: None,
        copy_exceptions: vec![],
    };
    t.g.create_tokens(p, tc, 1, None)[0]
}

fn count_subtype(t: &TestGame, p: PlayerId, st: &str) -> usize {
    t.g.permanents_controlled_by(p)
        .into_iter()
        .filter(|o| t.g.obj(*o).chars.has_subtype(st))
        .count()
}

#[test]
fn bat_colony_makes_a_bat_for_each_mana_from_a_cave() {
    cr!("106.3", "601.2h");
    assert_supported("Bat Colony");
    // {2}{W}: two Caves and a Plains pay it.
    let mut t = TestGame::new(2);
    t.lands(P0, "Captivating Cave", 2);
    t.lands(P0, "Plains", 1);
    let colony = t.hand(P0, "Bat Colony");
    t.cast(P0, colony).go();
    t.resolve_all();
    assert_eq!(count_subtype(&t, P0, "Bat"), 2);
    // Paid with Plains only: no Bats.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    let colony = t.hand(P0, "Bat Colony");
    t.cast(P0, colony).go();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(colony)));
    assert_eq!(count_subtype(&t, P0, "Bat"), 0);
}

#[test]
fn cataclysmic_prospecting_makes_a_tapped_treasure_for_each_desert_mana() {
    cr!("106.3", "601.2h");
    assert_supported("Cataclysmic Prospecting");
    let mut t = TestGame::new(2);
    t.lands(P0, "Desert", 1);
    t.lands(P0, "Mountain", 2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Cataclysmic Prospecting");
    t.cast(P0, spell).x(1).go();
    t.resolve_all();
    // X = 1: 1 damage to each creature.
    assert_eq!(t.g.obj(t.g.current(bear)).damage, 1);
    let treasures: Vec<ObjectId> = t
        .g
        .permanents_controlled_by(P0)
        .into_iter()
        .filter(|o| t.g.obj(*o).chars.has_subtype("Treasure"))
        .collect();
    assert_eq!(treasures.len(), 1);
    assert!(t.g.obj(treasures[0]).tapped);
}

#[test]
fn spoils_of_the_hunt_counts_treasure_mana_and_the_pump_ends() {
    cr!("106.3", "601.2h", "514.2");
    assert_supported("Spoils of the Hunt");
    let mut t = TestGame::new(2);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    t.lands(P0, "Forest", 1);
    treasure(&mut t, P0);
    treasure(&mut t, P0);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let spoils = t.hand(P0, "Spoils of the Hunt");
    t.cast(P0, spoils)
        .targets(&[Entity::Object(bear), Entity::Object(giant)])
        .go();
    t.resolve_all();
    // Two Treasure mana: +2/+0; the 4-power Bears deals 4 damage to the 3/3 Giant.
    assert_eq!(t.pt(bear), (4, 2));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // "until end of turn".
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    assert_eq!(t.pt(bear), (2, 2));
}

#[test]
fn kalain_counts_treasure_mana_spent_on_each_other_creature() {
    cr!("106.3", "614.1c");
    assert_supported("Kalain, Reclusive Painter");
    let mut t = TestGame::new(2);
    let kalain = t.battlefield(P0, "Kalain, Reclusive Painter");
    t.lands(P0, "Forest", 1);
    treasure(&mut t, P0);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    let bears = t.g.current(bears);
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.counters(kalain, "+1/+1"), 0);
    // Without Treasure mana: none.
    t.lands(P0, "Forest", 2);
    let bears2 = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears2).go();
    t.resolve_all();
    assert_eq!(t.counters(t.g.current(bears2), "+1/+1"), 0);
}

#[test]
fn kalain_gives_nothing_to_a_creature_that_wasnt_cast() {
    cr!("614.1c");
    let mut t = TestGame::new(2);
    // Kalain itself was cast with Treasure mana.
    t.lands(P0, "Swamp", 1);
    treasure(&mut t, P0);
    let kalain = t.hand(P0, "Kalain, Reclusive Painter");
    t.cast(P0, kalain).go();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(kalain)));
    // A creature put onto the battlefield had no mana spent on it.
    let bears = t.enter(P0, "Grizzly Bears");
    assert_eq!(t.counters(bears, "+1/+1"), 0);
}

#[test]
fn coin_of_mastery_counts_mana_from_artifact_sources() {
    cr!("106.3", "614.1c");
    assert_supported("Coin of Mastery");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Coin of Mastery");
    t.lands(P0, "Forest", 1);
    t.battlefield(P0, "Mind Stone");
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.counters(t.g.current(bears), "+1/+1"), 1);
    // Land mana isn't from an artifact source.
    t.lands(P0, "Forest", 2);
    let bears2 = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears2).go();
    t.resolve_all();
    assert_eq!(t.counters(t.g.current(bears2), "+1/+1"), 0);
}

/// P1 casts Counterspell on the spell on top of the stack.
fn counter_top(t: &mut TestGame) -> ObjectId {
    let spell = *t.g.stack.last().expect("a spell on the stack");
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(spell).go();
    t.resolve();
    spell
}

#[test]
fn boseiju_mana_makes_an_instant_or_sorcery_uncounterable() {
    cr!("106.6", "101.2");
    ruling!(
        "Boseiju, Who Shelters All",
        "The spell can't be countered if the mana produced by Boseiju is spent to cover any cost of the spell"
    );
    assert_supported("Boseiju, Who Shelters All");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Boseiju, Who Shelters All");
    t.lands(P0, "Island", 2);
    let div = t.hand(P0, "Divination");
    let hand = t.hand_size(P0);
    t.cast(P0, div).go();
    assert_eq!(t.life(P0), 18);
    counter_top(&mut t);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Divination"));
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
    // Paid with other mana, it's countered.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let div = t.hand(P0, "Divination");
    let hand = t.hand_size(P0);
    t.cast(P0, div).go();
    counter_top(&mut t);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1);
}

#[test]
fn boseiju_mana_spent_on_a_creature_spell_does_nothing() {
    cr!("106.6");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Boseiju, Who Shelters All");
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    counter_top(&mut t);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

/// Brass's Tunnel-Grinder transformed into Tecutlan, the Searing Rift.
fn tecutlan(t: &mut TestGame) -> ObjectId {
    let g = t.battlefield(P0, "Brass's Tunnel-Grinder // Tecutlan, the Searing Rift");
    assert!(mtg_engine::dfc::transform(&mut t.g, g));
    t.g.recompute();
    assert_eq!(t.g.obj(g).chars.name, "Tecutlan, the Searing Rift");
    g
}

#[test]
fn tecutlan_discovers_for_a_permanent_spell_cast_with_its_mana() {
    cr!("106.3", "603.2");
    ruling!(
        "Brass's Tunnel-Grinder // Tecutlan, the Searing Rift",
        "Tecutlan, the Searing Rift's triggered ability will resolve before the spell that caused it to trigger."
    );
    assert_supported("Brass's Tunnel-Grinder // Tecutlan, the Searing Rift");
    let mut t = TestGame::new(2);
    tecutlan(&mut t);
    t.lands(P0, "Forest", 1);
    let memnite = t.library_top(P0, "Memnite");
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.settle();
    // The trigger is above the Bears.
    assert_eq!(t.stack_len(), 2);
    assert_eq!(*t.g.stack.first().unwrap(), t.g.current(bears));
    t.answer_yes(P0, true);
    t.resolve();
    // Discover 2: Memnite (mana value 0) is cast.
    assert_eq!(t.zone(t.g.current(memnite)), Zone::Stack);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(memnite)));
    assert!(t.on_battlefield(t.g.current(bears)));
}

#[test]
fn tecutlan_doesnt_trigger_without_its_mana_or_for_an_instant() {
    cr!("106.3");
    // Other mana.
    let mut t = TestGame::new(2);
    let rift = tecutlan(&mut t);
    t.g.tap(rift);
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // Its mana spent on an instant: not a permanent spell.
    let mut t = TestGame::new(2);
    tecutlan(&mut t);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn barracks_of_the_thousand_makes_a_gnome_soldier() {
    cr!("106.3", "603.2");
    ruling!(
        "Thousand Moons Smithy // Barracks of the Thousand",
        "The Gnome Soldier's ability will count the Gnome Soldier itself, so it will be at least 1/1."
    );
    assert_supported("Thousand Moons Smithy // Barracks of the Thousand");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Thousand Moons Smithy // Barracks of the Thousand");
    assert!(mtg_engine::dfc::transform(&mut t.g, b));
    t.g.recompute();
    // {1}{G}: Barracks's {W} and a Forest pay it.
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    let soldiers: Vec<ObjectId> = t
        .g
        .permanents_controlled_by(P0)
        .into_iter()
        .filter(|o| t.g.obj(*o).chars.has_subtype("Gnome"))
        .collect();
    assert_eq!(soldiers.len(), 1);
    // It counts itself and Barracks of the Thousand (an artifact land) until the Bears
    // resolves.
    t.g.recompute();
    assert_eq!(t.pt(soldiers[0]), (2, 2));
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.pt(soldiers[0]), (3, 3));
}
