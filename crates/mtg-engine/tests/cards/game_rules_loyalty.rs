//! Loyalty-ability rules (CR 606) in Oracle text: who can't activate loyalty abilities,
//! activating them twice this turn, and countering a loyalty ability on the stack.

use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

#[test]
fn the_immortal_sun_stops_loyalty_abilities() {
    cr!("606.3");
    assert_supported(&["The Immortal Sun"]);
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P0, "Ajani Goldmane");
    // Its controller's opponent controls it: it affects every player.
    t.battlefield(P1, "The Immortal Sun");
    assert!(t.activate(P0, ajani, 0, &[]).is_err());
    assert_eq!(t.life(P0), 20);
}

#[test]
fn revel_in_silence_stops_opponents_spells_and_loyalty_abilities_this_turn() {
    cr!("606.3");
    assert_supported(&["Flamescroll Celebrant // Revel in Silence"]);
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P1, "Ajani Goldmane");
    t.lands(P0, "Plains", 2);
    let r = t.hand(P0, "Flamescroll Celebrant // Revel in Silence");
    t.cast(P0, r).method(CastMethod::Half(1)).go();
    t.resolve();
    // P1 can't activate it this turn, even at sorcery speed on their own turn.
    t.set_step(P1, turn::Step::PrecombatMain);
    assert!(t.activate(P1, ajani, 0, &[]).is_err());
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    assert!(t.cast(P1, bolt).target(P0).try_go().is_err());
    // P0 isn't affected.
    let ajani0 = t.battlefield(P0, "Ajani Goldmane");
    t.set_step(P0, turn::Step::PrecombatMain);
    t.activate(P0, ajani0, 0, &[]).unwrap();
}

#[test]
fn overwhelming_splendor_leaves_only_mana_and_loyalty_abilities() {
    cr!("605.1a", "606.3");
    assert_supported(&["Overwhelming Splendor"]);
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Overwhelming Splendor");
    t.g.objects[s.0 as usize].attached_to = Some(Entity::Player(P1));
    t.g.recompute();
    t.set_step(P1, turn::Step::PrecombatMain);
    let ajani = t.battlefield(P1, "Ajani Goldmane");
    let ring = t.battlefield(P1, "Sol Ring");
    // A noncreature artifact's ability that isn't a mana ability.
    let flask = t.battlefield(P1, "Mind Stone");
    t.lands(P1, "Plains", 1);
    assert!(t.activate(P1, flask, 1, &[]).is_err());
    t.activate(P1, ring, 0, &[]).unwrap();
    t.activate(P1, ajani, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 22);
}

#[test]
fn urza_assembles_the_titans_activates_loyalty_abilities_twice() {
    cr!("606.3");
    assert_supported(&["Urza Assembles the Titans"]);
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Urza Assembles the Titans");
    let ajani = t.battlefield(P0, "Ajani Goldmane");
    t.g.add_counters(Entity::Object(saga), "lore", 3, None);
    t.resolve_all();
    t.activate(P0, ajani, 0, &[]).unwrap();
    t.resolve_all();
    t.activate(P0, ajani, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    assert!(t.activate(P0, ajani, 0, &[]).is_err());
}

#[test]
fn verdant_command_counters_a_loyalty_ability() {
    cr!("606.3", "701.6b");
    assert_supported(&["Verdant Command"]);
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P1, "Ajani Goldmane");
    t.set_step(P1, turn::Step::PrecombatMain);
    let ab = t.activate(P1, ajani, 0, &[]).unwrap().unwrap();
    // Loyalty was paid as a cost.
    assert_eq!(t.counters(ajani, "loyalty"), 5);
    t.lands(P0, "Forest", 2);
    let vc = t.hand(P0, "Verdant Command");
    t.cast(P0, vc)
        .modes(&[1, 3])
        .targets(&[Entity::Object(ab), Entity::Player(P0)])
        .go();
    t.resolve();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 23);
    // CR 701.6b: the loyalty cost isn't refunded.
    assert_eq!(t.counters(ajani, "loyalty"), 5);
}
