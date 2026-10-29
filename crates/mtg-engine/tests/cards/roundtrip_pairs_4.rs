//! roundtrip-pairs-4 (docs/agents/items/roundtrip-pairs-4.md): untriaged round-trip
//! mismatches from the long tail (the two-card clusters).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Puts the real Aura or Equipment `name` onto the battlefield under `p`'s control attached
/// to `to`.
fn attach_new(t: &mut TestGame, p: PlayerId, name: &str, to: ObjectId) -> ObjectId {
    let id = t.battlefield(p, name);
    assert!(t.g.attach(id, Entity::Object(to)), "{name} couldn't be attached");
    t.g.recompute();
    id
}

#[test]
fn seven_dwarves_counts_creatures_named_seven_dwarves_not_ones_sharing_another_name() {
    cr!("201.2", "201.2a", "201.5");
    // "~ gets +1/+1 for each other creature named ~ you control." The creatures named
    // Seven Dwarves, not the ones with the same name as this one: with Spy Kit ("has all
    // names of nonlegendary creature cards") it has Grizzly Bears' name too, but Grizzly
    // Bears isn't named Seven Dwarves.
    let mut t = TestGame::new(2);
    let dwarves = t.battlefield(P0, "Seven Dwarves");
    t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Spy Kit", dwarves);
    assert_eq!(t.pt(dwarves), (3, 3), "Spy Kit's +1/+1 only");
    // Another Seven Dwarves counts.
    t.battlefield(P0, "Seven Dwarves");
    t.g.recompute();
    assert_eq!(t.pt(dwarves), (4, 4));
}

#[test]
fn lake_of_the_dead_sacrifices_a_swamp_without_asking() {
    cr!("614.1c", "614.12", "608.2c");
    ruling!(
        "Scorched Ruins",
        "If you don't sacrifice the lands, Scorched Ruins never enters"
    );
    // "If ~ would enter, sacrifice a Swamp instead. If you do, put ~ onto the battlefield.
    // If you don't, put it into its owner's graveyard." An instruction, not an offer
    // (Mox Diamond's "you may discard a land card instead" is one): with a Swamp, it's
    // sacrificed and Lake of the Dead enters whatever the player would answer.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let lake = t.hand(P0, "Lake of the Dead");
    t.answer_yes(P0, false);
    t.play_land(P0, lake).unwrap();
    t.settle();
    assert_eq!(t.named_on_battlefield("Lake of the Dead").len(), 1);
    assert!(t.in_graveyard(P0, "Swamp"));
    // Without a Swamp, it never enters.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let lake = t.hand(P0, "Lake of the Dead");
    t.play_land(P0, lake).unwrap();
    t.settle();
    assert!(t.in_graveyard(P0, "Lake of the Dead"));
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
}

#[test]
fn lotus_vale_with_one_untapped_land_sacrifices_it_and_never_enters() {
    cr!("614.12", "609.3");
    interp!("enter-sacrifice-instead-partial");
    // "Sacrifice two untapped lands instead. If you do, ...": with only one, as much as
    // possible is done (it's sacrificed), but not all of it, so Lotus Vale goes to the
    // graveyard.
    let mut t = TestGame::new(2);
    let forests = t.lands(P0, "Forest", 2);
    t.g.tap(forests[0]);
    let vale = t.hand(P0, "Lotus Vale");
    t.play_land(P0, vale).unwrap();
    t.settle();
    assert!(t.in_graveyard(P0, "Lotus Vale"));
    assert_eq!(t.named_on_battlefield("Forest"), vec![forests[0]]);
}

#[test]
fn crackdown_construct_ignores_abilities_of_cards_in_hand() {
    cr!("109.2", "602.1", "702.29a");
    ruling!(
        "Crackdown Construct",
        "doesn't trigger if you activate an ability of an artifact or creature card not on the battlefield"
    );
    // "Whenever you activate an ability of an artifact or creature that isn't a mana
    // ability, ~ gets +1/+1 until end of turn": of an artifact or creature permanent
    // (CR 109.2), not a creature card's cycling from a hand.
    let mut t = TestGame::new(2);
    let construct = t.battlefield(P0, "Crackdown Construct");
    t.lands(P0, "Forest", 3);
    t.library_top(P0, "Island");
    let tusker = t.hand(P0, "Krosan Tusker");
    t.activate(P0, tusker, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Krosan Tusker"), "cycled");
    assert_eq!(t.pt(construct), (2, 2));
    // A creature's ability on the battlefield.
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    t.g.recompute();
    t.activate(P0, sorcerer, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.pt(construct), (3, 3));
}

#[test]
fn rivalry_needs_more_lands_than_each_other_player() {
    cr!("103.1", "603.2");
    // "At the beginning of each player's upkeep, if that player controls more lands than
    // each other player, ~ deals 2 damage to them." (Compiled as "no other player
    // controls at least as many lands".) In a three-player game, more lands than one
    // opponent isn't enough.
    for (p1, p2, life) in [(3, 5, 20), (5, 5, 20), (6, 5, 18)] {
        let mut t = TestGame::new(3);
        t.battlefield(P0, "Rivalry");
        t.lands(P1, "Forest", p1);
        t.lands(P2, "Forest", p2);
        t.advance_to(P1, Step::Draw);
        t.resolve_all();
        assert_eq!(t.life(P1), life, "{p1} lands against {p2}");
    }
}

#[test]
fn shapeshifters_marrow_becomes_a_copy_of_the_revealed_creature_card() {
    cr!("707.2", "608.2c");
    // "At the beginning of each opponent's upkeep, that player reveals the top card of
    // their library. If it's a creature card, the player puts the card into their
    // graveyard and ~ becomes a copy of that card." The card is the one now in the
    // graveyard (CR 400.7).
    let mut t = TestGame::new(2);
    let marrow = t.battlefield(P0, "Shapeshifter's Marrow");
    t.library_top(P1, "Island");
    t.library_top(P1, "Grizzly Bears");
    t.advance_to(P1, Step::Draw);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let m = t.obj_now(marrow);
    assert_eq!(m.chars.name.as_str(), "Grizzly Bears");
    assert_eq!(t.pt(marrow), (2, 2));
}

#[test]
fn kambal_copies_each_of_the_tokens_that_entered() {
    cr!("111.10", "603.2c", "707.2");
    // "Whenever one or more tokens your opponents control enter, for each of them, create
    // a tapped token that's a copy of it."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kambal, Profiteering Mayor");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Plains", 2);
    let alarm = t.hand(P1, "Raise the Alarm");
    t.cast(P1, alarm).go();
    t.resolve_all();
    let copies: Vec<_> = t
        .g
        .permanents()
        .filter(|o| o.controller == P0 && o.is_token())
        .map(|o| (o.chars.name.to_string(), o.tapped))
        .collect();
    assert_eq!(copies.len(), 2, "{copies:?}");
    assert!(
        copies.iter().all(|(n, tapped)| n == "Soldier Token" && *tapped),
        "{copies:?}"
    );
    // Its other ability: those tokens entered under P0's control, once.
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
}

#[test]
fn healing_grace_prevents_three_damage_in_all() {
    cr!("615.7", "609.7a");
    // "Prevent the next 3 damage that would be dealt to any target this turn by a source
    // of your choice. You gain 3 life.": a shield of 3, used up as it prevents.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 1);
    let grace = t.hand(P0, "Healing Grace");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.cast(P0, grace).target(P0).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    t.g.deal_damage(giant, Entity::Player(P0), 2, false);
    t.settle();
    assert_eq!(t.life(P0), 23);
    t.g.deal_damage(giant, Entity::Player(P0), 2, false);
    t.settle();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn tanazir_quandrix_sets_base_power_and_toughness_once() {
    cr!("608.2h", "613.4b");
    // "Whenever ~ attacks, you may have the base power and toughness of other creatures
    // you control become equal to ~'s power and toughness until end of turn." The numbers
    // are determined as the ability resolves: Tanazir growing later doesn't change them.
    let mut t = TestGame::new(2);
    let tanazir = t.battlefield(P0, "Tanazir Quandrix");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.attack(&[(tanazir, Entity::Player(P1))], &[]);
    assert_eq!(t.pt(bears), (4, 4));
    t.g.add_counters(Entity::Object(tanazir), "+1/+1", 2, None);
    t.g.recompute();
    assert_eq!(t.pt(tanazir), (6, 6));
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn kurkesh_ignores_an_artifact_cards_cycling() {
    cr!("109.2", "602.1", "702.29a");
    // "Whenever you activate an ability of an artifact, if it isn't a mana ability, you
    // may pay {R}. If you do, copy that ability.": of an artifact permanent, not an
    // artifact card's cycling from a hand.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kurkesh, Onakke Ancient");
    t.lands(P0, "Mountain", 3);
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    let crystal = t.hand(P0, "Raugrin Crystal");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.activate(P0, crystal, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Raugrin Crystal"), "cycled");
    // One card drawn: the cycling wasn't copied.
    assert_eq!(t.hand_size(P0), hand);
}
