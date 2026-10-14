//! Shuffling objects into libraries (patterns in
//! `src/oracle/patterns/control_exile_shuffle.rs`, CR 701.24).

use mtg_engine::object::Zone;
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
fn shuffle_cards_compile() {
    assert_compiles(&[
        "Clear the Mind",
        "Echo of Eons",
        "Game Plan",
        "Struggle // Survive",
        "Whirlpool Rider",
        "Kozilek, Butcher of Truth",
        "Blitz Hellion",
        "Deglamer",
        "Oblation",
        "Renewing Touch",
        "Memory's Journey",
        "Dwell on the Past",
    ]);
}

#[test]
fn target_player_shuffles_their_graveyard_into_their_library() {
    cr!("701.24a", "701.24d");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    // "Whenever an opponent shuffles their library, you may put a +1/+1 counter on this
    // creature."
    let trickster = t.battlefield(P0, "Cosi's Trickster");
    for _ in 0..3 {
        t.graveyard(P1, "Grizzly Bears");
    }
    let mine = t.graveyard(P0, "Grizzly Bears");
    let lib1 = t.library_size(P1);
    let hand0 = t.hand_size(P0);
    let spell = t.hand(P0, "Clear the Mind");
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 0);
    assert_eq!(t.library_size(P1), lib1 + 3);
    assert_eq!(t.counters(trickster, "+1/+1"), 1);
    // Only that player's graveyard; then "Draw a card."
    assert_eq!(t.zone(mine), Zone::Graveyard(P0));
    assert_eq!(t.hand_size(P0), hand0 + 1);

    // CR 701.24d: with an empty graveyard, the library is still shuffled.
    let spell = t.hand(P0, "Clear the Mind");
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.library_size(P1), lib1 + 3);
    assert_eq!(t.counters(trickster, "+1/+1"), 2);
}

#[test]
fn each_player_shuffles_hand_and_graveyard_then_draws_seven() {
    cr!("701.24a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    for p in [P0, P1] {
        t.hand(p, "Grizzly Bears");
        t.graveyard(p, "Grizzly Bears");
        t.graveyard(p, "Wall of Wood");
    }
    let total = |t: &TestGame, p| t.library_size(p) + t.hand_size(p) + t.graveyard_size(p);
    let before = [total(&t, P0), total(&t, P1)];
    let spell = t.hand(P0, "Echo of Eons");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 7);
    assert_eq!(t.hand_size(P1), 7);
    assert_eq!(t.graveyard_size(P1), 0);
    // Echo of Eons itself goes to the graveyard after resolving.
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(total(&t, P1), before[1]);
}

#[test]
fn exile_this_spell_as_it_resolves() {
    cr!("608.2n");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    t.graveyard(P0, "Grizzly Bears");
    let spell = t.hand(P0, "Game Plan");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 7);
    assert!(t.in_exile("Game Plan"));
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn shuffle_hand_into_library_then_draw_that_many() {
    cr!("701.24a", "608.2c");
    let mut t = TestGame::new(2);
    let hand: Vec<ObjectId> = (0..3).map(|_| t.hand(P0, "Grizzly Bears")).collect();
    let lib = t.library_size(P0);
    t.enter(P0, "Whirlpool Rider");
    t.resolve_all();
    // Every card that was in the hand went into the library (becoming a new object,
    // CR 400.7), and three cards were drawn.
    for c in &hand {
        assert!(!t.g.is_live(*c));
    }
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.library_size(P0), lib);
    assert_eq!(
        t.g.player(P0)
            .hand
            .iter()
            .filter(|c| hand.contains(c))
            .count(),
        0
    );
}

#[test]
fn each_player_shuffles_their_hand_then_draws_that_many() {
    cr!("701.24a", "701.24d");
    let mut t = TestGame::new(2);
    let trickster = t.battlefield(P0, "Cosi's Trickster");
    let mine: Vec<ObjectId> = (0..2).map(|_| t.hand(P0, "Grizzly Bears")).collect();
    let theirs: Vec<ObjectId> = (0..3).map(|_| t.hand(P1, "Grizzly Bears")).collect();
    let libs = [t.library_size(P0), t.library_size(P1)];
    let warrior = t.battlefield(P0, "Whirlpool Warrior");
    t.lands(P0, "Mountain", 1);
    // "{R}, Sacrifice this creature: Each player shuffles the cards from their hand into
    // their library, then draws that many cards."
    t.activate(P0, warrior, 0, &[]).unwrap();
    t.answer_yes(P0, true);
    t.resolve_all();
    // Each player draws the number of cards they shuffled away.
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.hand_size(P1), 3);
    assert_eq!(t.library_size(P0), libs[0]);
    assert_eq!(t.library_size(P1), libs[1]);
    assert!(mine.iter().chain(&theirs).all(|c| !t.g.is_live(*c)));
    assert_eq!(t.counters(trickster, "+1/+1"), 1);

    // A player with an empty hand still shuffles (and draws nothing).
    let warrior = t.battlefield(P0, "Whirlpool Warrior");
    t.lands(P0, "Mountain", 1);
    t.g.players[1].hand.clear();
    t.activate(P0, warrior, 0, &[]).unwrap();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.counters(trickster, "+1/+1"), 2);
}

#[test]
fn shuffle_target_cards_from_a_graveyard() {
    cr!("701.24a", "115.1");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P1, "Wall of Wood");
    let c = t.graveyard(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Memory's Journey");
    t.cast(P0, spell)
        .target(Entity::Player(P1))
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Library(P1));
    assert_eq!(t.zone(b), Zone::Library(P1));
    assert_eq!(t.zone(c), Zone::Graveyard(P1));
    // The cards must be in the target player's graveyard.
    let mine = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let spell = t.hand(P0, "Memory's Journey");
    t.cast(P0, spell)
        .target(Entity::Player(P1))
        .targets(&[Entity::Object(mine)])
        .go();
    t.resolve_all();
    assert_eq!(t.zone(mine), Zone::Graveyard(P0));

    // "Shuffle any number of target creature cards from your graveyard into your library."
    let bear = t.graveyard(P0, "Grizzly Bears");
    let land = t.graveyard(P0, "Forest");
    t.lands(P0, "Forest", 1);
    let touch = t.hand(P0, "Renewing Touch");
    t.cast(P0, touch).targets(&[Entity::Object(bear)]).go();
    t.resolve_all();
    assert_eq!(t.zone(bear), Zone::Library(P0));
    assert_eq!(t.zone(land), Zone::Graveyard(P0));
}

#[test]
fn owner_shuffles_it_into_their_library() {
    cr!("701.24a", "115.1a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let hand1 = t.hand_size(P1);
    let spell = t.hand(P0, "Oblation");
    t.cast(P0, spell).target(bear).go();
    t.resolve_all();
    assert_eq!(t.zone(bear), Zone::Library(P1));
    // "then draws two cards": the owner draws.
    assert_eq!(t.hand_size(P1), hand1 + 2);

    // "Choose target artifact or enchantment. Its owner shuffles it into their library."
    t.lands(P0, "Forest", 2);
    let relic = t.battlefield(P1, "Ornithopter");
    let deglamer = t.hand(P0, "Deglamer");
    t.cast(P0, deglamer).target(relic).go();
    t.resolve_all();
    assert_eq!(t.zone(relic), Zone::Library(P1));
}

#[test]
fn chosen_target_stays_that_creature_after_a_reveal() {
    cr!("115.1a", "608.2c");
    let mut t = TestGame::new(2);
    // Every card in the library has mana value 4, however the scry goes.
    t.g.players[0].library.clear();
    for _ in 0..4 {
        t.library_top(P0, "Hill Giant");
    }
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer(
        P1,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(giant, Entity::Player(P0))]),
    );
    t.set_step(P1, Step::BeginningOfCombat);
    t.advance_to(P1, Step::DeclareAttackers);
    t.lands(P0, "Plains", 2);
    // "Choose target attacking or blocking creature. Scry 3, then reveal the top card of
    // your library. ~ deals damage equal to that card's mana value to that creature."
    let judge = t.hand(P0, "Judge Unworthy");
    t.cast(P0, judge).target(giant).go();
    t.resolve();
    assert!(!t.on_battlefield(giant));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.library_size(P0), 4);
}

#[test]
fn put_into_a_graveyard_from_anywhere_shuffles_the_graveyard() {
    cr!("603.6c", "701.24a");
    // Discarded from a hand.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    let kozilek = t.hand(P0, "Kozilek, Butcher of Truth");
    let lib = t.library_size(P0);
    t.g.discard(P0, kozilek, None);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.library_size(P0), lib + 2);

    // Sacrificed while it has no abilities on the battlefield: a "from anywhere" ability
    // isn't a leaves-the-battlefield ability, so it doesn't look back in time. The card
    // in the graveyard has the ability, and it triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Humility");
    let kozilek = t.battlefield(P0, "Kozilek, Butcher of Truth");
    assert!(t.obj_now(kozilek).chars.abilities.is_empty());
    t.graveyard(P0, "Grizzly Bears");
    let lib = t.library_size(P0);
    t.lands(P1, "Swamp", 2);
    let edict = t.hand(P1, "Diabolic Edict");
    t.cast(P1, edict).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.zone(kozilek), Zone::Library(P0));
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.library_size(P0), lib + 2);
}

#[test]
fn owner_shuffles_it_at_the_beginning_of_the_end_step() {
    cr!("701.24a");
    let mut t = TestGame::new(2);
    let hellion = t.battlefield(P0, "Blitz Hellion");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.zone(hellion), Zone::Library(P0));
}
