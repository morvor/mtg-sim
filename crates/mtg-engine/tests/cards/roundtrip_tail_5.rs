//! roundtrip-tail-5 (docs/agents/items/roundtrip-tail-5.md): the second half of the 3-card
//! wording clusters, and the behavior doubts the roundtrip-clusters-4 review left open.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn vesuva_enters_untapped_when_it_copies_nothing() {
    cr!("614.1c", "707.9");
    ruling!(
        "Vesuva",
        "If you don't choose a land on the battlefield, Vesuva enters the battlefield untapped as itself"
    );
    // "You may have this land enter tapped as a copy of any land on the battlefield."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Forest");
    t.answer_choose(P0, &[]);
    let v = t.enter(P0, "Vesuva");
    t.settle();
    assert_eq!(t.obj_now(v).chars.name, "Vesuva");
    assert!(!t.obj_now(v).tapped, "{}", t.dump_log());
}

#[test]
fn vesuva_enters_tapped_as_a_copy() {
    cr!("614.1c", "707.9");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    t.answer_choose(P0, &[Entity::Object(forest)]);
    let v = t.enter(P0, "Vesuva");
    t.settle();
    assert_eq!(t.obj_now(v).chars.name, "Forest");
    assert!(t.obj_now(v).tapped, "{}", t.dump_log());
}

#[test]
fn echoing_deeps_enters_untapped_when_it_copies_nothing() {
    cr!("614.1c", "707.9");
    ruling!(
        "Echoing Deeps",
        "If you choose to have Echoing Deeps not copy anything, it enters the battlefield untapped."
    );
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Forest");
    t.answer_choose(P0, &[]);
    let d = t.enter(P0, "Echoing Deeps");
    t.settle();
    let o = t.obj_now(d);
    assert_eq!(o.chars.name, "Echoing Deeps");
    assert!(!o.tapped, "{}", t.dump_log());
}

#[test]
fn callidus_assassin_enters_untapped_when_it_copies_nothing() {
    cr!("614.1c", "707.9");
    ruling!(
        "Callidus Assassin",
        "You can choose not to copy anything. In that case, Callidus Assassin enters the battlefield as a 3/3 creature."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[]);
    let c = t.enter(P0, "Callidus Assassin");
    t.settle();
    assert_eq!(t.obj_now(c).chars.name, "Callidus Assassin");
    assert!(!t.obj_now(c).tapped, "{}", t.dump_log());
    // No "destroy up to one other target creature with the same name" trigger.
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn clackbridge_troll_sacrifices_the_chosen_creatures_at_the_same_time() {
    cr!("101.4", "603.10a", "701.21a");
    ruling!(
        "Clackbridge Troll",
        "Then all the chosen creatures are sacrificed at the same time."
    );
    // P1 sacrifices Blood Artist and P2 a Grizzly Bears: they die together, so Blood
    // Artist sees the Bears die too (CR 603.10a) and triggers twice.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Clackbridge Troll");
    let artist = t.battlefield(P1, "Blood Artist");
    let bears = t.battlefield(P2, "Grizzly Bears");
    t.answer_choose(P1, &[Entity::Object(artist)]);
    t.answer_choose(P2, &[Entity::Object(bears)]);
    t.set_step(P0, Step::PrecombatMain);
    let life = t.life(P1);
    t.advance_to_step(Step::BeginningOfCombat);
    t.resolve_all();
    assert!(!t.on_battlefield(artist) && !t.on_battlefield(bears));
    assert_eq!(t.life(P1), life + 2, "{}", t.dump_log());
}

#[test]
fn underworld_fires_exiles_only_permanents_it_actually_damaged() {
    cr!("614.1a", "702.16e");
    // "If a permanent dealt damage this way would die this turn, exile it instead": a
    // creature whose damage was prevented wasn't dealt damage this way.
    let mut t = TestGame::new(2);
    // Protection from red prevents the damage to Kor Firewalker (CR 702.16e).
    let a = t.battlefield(P1, "Kor Firewalker");
    let b = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 2);
    let fires = t.hand(P0, "Underworld Fires");
    if let Err(e) = t.cast_with(P0, fires, &[]) {
        panic!("cast: {e:?}\n{}", t.dump_log());
    }
    t.resolve_all();
    t.g.destroy(a, None);
    t.g.destroy(b, None);
    t.settle();
    assert!(t.in_graveyard(P1, "Kor Firewalker"), "{}", t.dump_log());
    assert!(t.in_exile("Hill Giant"), "{}", t.dump_log());
}

/// Whether P0 revealed a card with this name (the game log).
fn was_revealed(t: &TestGame, card: &str) -> bool {
    t.dump_log().contains(&format!("P0 reveals {card}"))
}

#[test]
fn primitive_etchings_reveals_the_first_card_drawn_whatever_it_is() {
    cr!("701.20a", "121.1");
    ruling!(
        "Rowen",
        "The first sentence is a continuous ability. You reveal the card when you draw it"
    );
    // "Reveal the first card you draw each turn. Whenever you reveal a creature card this
    // way, draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Primitive Etchings");
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Island");
    t.g.draw_cards(P0, 2);
    t.settle();
    assert!(was_revealed(&t, "Island"), "{}", t.dump_log());
    assert!(!was_revealed(&t, "Grizzly Bears"));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn keranos_reveals_only_on_your_turns() {
    cr!("701.20a", "121.1");
    // "Reveal the first card you draw on each of your turns."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Keranos, God of Storms");
    t.set_step(P1, Step::PrecombatMain);
    t.library_top(P0, "Island");
    t.g.draw_cards(P0, 1);
    t.settle();
    assert!(!was_revealed(&t, "Island"), "{}", t.dump_log());
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn ghosts_of_the_innocent_halves_damage_rounded_down() {
    cr!("614.1a");
    // "If a source would deal damage to a permanent or player, it deals half that damage,
    // rounded down, to that permanent or player instead."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ghosts of the Innocent");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let life = t.life(P1);
    if let Err(e) = t.cast_with(P0, bolt, &[Entity::Player(P1)]) {
        panic!("cast: {e:?}\n{}", t.dump_log());
    }
    t.resolve_all();
    assert_eq!(t.life(P1), life - 1, "{}", t.dump_log());
}

#[test]
fn plaguecrafter_each_player_who_cant_sacrifice_discards() {
    cr!("701.21a", "101.4");
    // "When this creature enters, each player sacrifices a creature or planeswalker of
    // their choice. Each player who can't discards a card."
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.hand(P1, "Island");
    t.hand(P2, "Island");
    t.answer_choose(P1, &[Entity::Object(bears)]);
    let pc = t.enter(P0, "Plaguecrafter");
    t.answer_choose(P0, &[Entity::Object(pc)]);
    t.settle();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    // P1 sacrificed a creature; P2 had none and discards.
    assert_eq!(t.hand_size(P1), 1, "{}", t.dump_log());
    assert_eq!(t.hand_size(P2), 0, "{}", t.dump_log());
}
