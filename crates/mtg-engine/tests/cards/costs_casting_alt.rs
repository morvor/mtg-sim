//! A spell's own alternative costs (CR 118.9, 601.2b): "You may [cost] rather than pay
//! this spell's mana cost.", "If you control a Swamp, you may pay 4 life rather than pay
//! this spell's mana cost."

use mtg_engine::decision::Action;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// The alternative-cost ways of casting the card offered now.
fn alternatives(t: &TestGame, p: PlayerId, card: ObjectId) -> Vec<CastMethod> {
    t.cast_options(p, card)
        .into_iter()
        .map(|o| o.method)
        .filter(|m| matches!(m, CastMethod::Alternative(_)))
        .collect()
}

/// Whether the player may begin casting the card with its alternative cost now.
fn alt_castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    let saved = t.g.turn.priority;
    t.g.turn.priority = Some(p);
    let ok = t.g.legal_actions(p).iter().any(|a| {
        matches!(a, Action::Cast { card: c, method: CastMethod::Alternative(_) } if *c == card)
    });
    t.g.turn.priority = saved;
    ok
}

#[test]
fn pay_life_instead_if_you_control_a_swamp() {
    cr!("118.9", "601.2b");
    compiles("Snuff Out");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let snuff = t.hand(P0, "Snuff Out");
    t.lands(P0, "Mountain", 1);
    // No Swamp: only the mana cost.
    assert!(alternatives(&t, P0, snuff).is_empty());
    let swamp = t.lands(P0, "Swamp", 1)[0];
    let alts = alternatives(&t, P0, snuff);
    assert_eq!(alts.len(), 1);
    t.cast(P0, snuff)
        .method(alts[0].clone())
        .target(bears)
        .go();
    assert_eq!(t.life(P0), 16);
    // No mana was spent.
    assert!(!t.obj_now(swamp).tapped);
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn return_islands_instead_of_paying_mana() {
    cr!("118.9", "601.2h");
    compiles("Gush");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::Upkeep);
    let gush = t.hand(P0, "Gush");
    let islands = t.lands(P0, "Island", 1);
    // One Island isn't enough.
    assert!(!alt_castable(&mut t, P0, gush));
    let more = t.lands(P0, "Island", 1);
    assert!(alt_castable(&mut t, P0, gush));
    let alts = alternatives(&t, P0, gush);
    let hand_before = t.hand_size(P0);
    t.cast(P0, gush).method(alts[0].clone()).go();
    assert!(!t.on_battlefield(islands[0]));
    assert!(!t.on_battlefield(more[0]));
    // Gush left the hand; two Islands came back.
    assert_eq!(t.hand_size(P0), hand_before + 1);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand_before + 3);
}

#[test]
fn pay_life_and_exile_another_blue_card() {
    cr!("118.9", "118.9c");
    ruling!(
        "Force of Will",
        "start with the mana cost or alternative cost you're paying (such as the alternative cost of Force of Will)"
    );
    compiles("Force of Will");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    let fow = t.hand(P0, "Force of Will");
    let spell = t.cast(P1, bolt).target(P0).go();
    // Force of Will can't exile itself to pay its cost.
    assert!(!alt_castable(&mut t, P0, fow));
    t.hand(P0, "Opt");
    t.hand(P0, "Lightning Bolt");
    assert!(alt_castable(&mut t, P0, fow));
    let alts = alternatives(&t, P0, fow);
    let s = t.cast(P0, fow).method(alts[0].clone()).target(spell).go();
    // Mana value is still 5 (CR 118.9c).
    assert_eq!(t.obj(s).chars.mana_value(), 5);
    assert_eq!(t.life(P0), 19);
    assert!(t.in_exile("Opt"));
    assert!(t.in_hand(P0, "Lightning Bolt"));
    t.resolve();
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert_eq!(t.life(P0), 19);
}

#[test]
fn exile_a_card_instead_only_if_its_not_your_turn() {
    cr!("118.9", "601.2b");
    compiles("Force of Vigor");
    let mut t = TestGame::new(2);
    let vigor = t.hand(P0, "Force of Vigor");
    t.hand(P0, "Grizzly Bears");
    let a = t.battlefield(P1, "Ornithopter");
    t.set_step(P0, Step::PrecombatMain);
    assert!(alternatives(&t, P0, vigor).is_empty());
    t.set_step(P1, Step::PrecombatMain);
    let alts = alternatives(&t, P0, vigor);
    assert_eq!(alts.len(), 1);
    t.cast(P0, vigor)
        .method(alts[0].clone())
        .targets(&[a.into()])
        .go();
    assert!(t.in_exile("Grizzly Bears"));
    t.resolve();
    assert!(t.in_graveyard(P1, "Ornithopter"));
}

#[test]
fn tap_an_untapped_creature_instead_even_a_summoning_sick_one() {
    cr!("118.9", "302.6");
    compiles("Ramosian Rally");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let rally = t.hand(P0, "Ramosian Rally");
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    assert!(alternatives(&t, P0, rally).is_empty());
    t.lands(P0, "Plains", 1);
    let alts = alternatives(&t, P0, rally);
    assert_eq!(alts.len(), 1);
    t.cast(P0, rally).method(alts[0].clone()).go();
    assert!(t.obj_now(bears).tapped);
    t.resolve();
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn sacrifice_a_mountain_instead() {
    cr!("118.9");
    compiles("Thunderclap");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let clap = t.hand(P0, "Thunderclap");
    let m = t.lands(P0, "Mountain", 1)[0];
    let alts = alternatives(&t, P0, clap);
    t.cast(P0, clap).method(alts[0].clone()).target(bears).go();
    assert!(t.in_graveyard(P0, "Mountain"));
    assert!(!t.on_battlefield(m));
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn pay_mana_and_return_the_land_that_made_it() {
    cr!("118.9", "601.2g", "601.2h");
    ruling!(
        "Mistvein Borderpost",
        "you may tap a basic land for mana, then both spend that mana and return that land to your hand to pay this card's alternative cost"
    );
    ruling!(
        "Mistvein Borderpost",
        "Casting this card by paying its alternative cost doesn't change when you can cast it."
    );
    compiles("Mistvein Borderpost");
    let mut t = TestGame::new(2);
    let post = t.hand(P0, "Mistvein Borderpost");
    let island = t.lands(P0, "Island", 1)[0];
    // Only as an artifact spell could be cast.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!alt_castable(&mut t, P0, post));
    t.set_step(P0, Step::PrecombatMain);
    assert!(alt_castable(&mut t, P0, post));
    let alts = alternatives(&t, P0, post);
    t.cast(P0, post).method(alts[0].clone()).go();
    assert!(!t.on_battlefield(island));
    assert!(t.in_hand(P0, "Island"));
    t.resolve();
    assert_eq!(t.named_on_battlefield("Mistvein Borderpost").len(), 1);
}

#[test]
fn pay_five_colors_instead() {
    cr!("118.9");
    compiles("Bringer of the Red Dawn");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bringer = t.hand(P0, "Bringer of the Red Dawn");
    for n in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        t.lands(P0, n, 1);
    }
    // {7}{R}{R} can't be paid with five lands; {W}{U}{B}{R}{G} can.
    assert!(t.cast(P0, bringer).try_go().is_err());
    let alts = alternatives(&t, P0, bringer);
    t.cast(P0, bringer).method(alts[0].clone()).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Bringer of the Red Dawn").len(), 1);
}
