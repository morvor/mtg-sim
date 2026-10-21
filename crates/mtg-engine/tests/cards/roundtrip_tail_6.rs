//! roundtrip-tail-6 (docs/agents/items/roundtrip-tail-6.md): every 3-card round-trip
//! cluster, plus Celestial Dawn's spells you own that another player controls.

use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::Color;
use mtg_engine::*;

/// Activates the first activated ability of `source` whose text starts with `prefix`.
fn activate(t: &mut TestGame, p: PlayerId, source: ObjectId, prefix: &str) {
    t.g.recompute();
    let uid = t
        .g
        .obj(source)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.starts_with(prefix))
        .map(|a| a.uid)
        .expect("no such ability");
    t.g.turn.priority = Some(p);
    t.g.activate_ability(p, source, uid).expect("activation");
    t.g.flush_events();
}

#[test]
fn celestial_dawn_makes_a_spell_you_own_white_while_another_player_controls_it() {
    cr!("112.1", "611.3a");
    // "Nonland permanents you control are white. The same is true for spells you control
    // and nonland cards you own that aren't on the battlefield." A spell is a card on the
    // stack (CR 112.1): P0's card cast by P1 is a card P0 owns that isn't on the
    // battlefield.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Celestial Dawn");
    t.lands(P1, "Mountain", 1);
    let goblin = t.hand(P1, "Raging Goblin");
    t.g.obj_mut(goblin).owner = P0;
    let spell = t.cast(P1, goblin).go();
    t.g.recompute();
    let o = t.obj_now(spell);
    assert_eq!(o.zone, Zone::Stack);
    assert_eq!(o.controller, P1);
    assert!(o.chars.colors.contains(Color::White), "{:?}", o.chars.colors);
    assert!(!o.chars.colors.contains(Color::Red), "{:?}", o.chars.colors);
    // A spell P1 owns stays red.
    t.resolve_all();
    t.lands(P1, "Mountain", 1);
    let other = t.hand(P1, "Raging Goblin");
    let other = t.cast(P1, other).go();
    t.g.recompute();
    assert!(t.obj_now(other).chars.colors.contains(Color::Red));
}

#[test]
fn deafening_silence_counts_a_cast_copy_of_a_card() {
    cr!("112.1b", "707.12", "101.2");
    ruling!(
        "Deafening Silence",
        "Players may cast any number of creature spells plus one noncreature spell each turn."
    );
    // "Each player can't cast more than one noncreature spell each turn." A copy of a card
    // cast with Isochron Scepter is a noncreature spell (CR 112.1b): after it, no other
    // noncreature spell can be cast that turn.
    let mut t = TestGame::new(2);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    let scepter = t.enter(P0, "Isochron Scepter");
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Exile);
    t.battlefield(P1, "Deafening Silence");
    t.lands(P0, "Mountain", 3);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate(&mut t, P0, scepter, "{2}");
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "{}", t.dump_log());
    let shock = t.hand(P0, "Shock");
    assert!(
        t.cast(P0, shock).target(Entity::Player(P1)).try_go().is_err(),
        "{}",
        t.dump_log()
    );
    assert_eq!(t.zone(shock), Zone::Hand(P0));
}

#[test]
fn deafening_silence_stops_casting_a_copy_after_a_noncreature_spell() {
    cr!("112.1b", "707.12", "101.2");
    // The copy can't be cast after a noncreature spell this turn: it isn't cast (and
    // ceases to exist, CR 707.12, 704.5d).
    let mut t = TestGame::new(2);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    let scepter = t.enter(P0, "Isochron Scepter");
    t.resolve_all();
    t.battlefield(P1, "Deafening Silence");
    t.lands(P0, "Mountain", 3);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate(&mut t, P0, scepter, "{2}");
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "{}", t.dump_log());
}

#[test]
fn gluttonous_hellkite_counts_every_creature_sacrificed_tokens_included() {
    cr!("601.2i", "614.1c", "701.21a");
    // "When you cast this spell, each player sacrifices X creatures. This creature enters
    // with two +1/+1 counters on it for each creature sacrificed this way."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let token = t.battlefield(P1, "Grizzly Bears");
    t.g.obj_mut(token).kind = mtg_engine::object::ObjKind::Token;
    for l in ["Swamp", "Mountain", "Forest", "Wastes", "Wastes"] {
        t.lands(P0, l, 1);
    }
    let h = t.hand(P0, "Gluttonous Hellkite");
    t.cast(P0, h).x(1).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(token));
    let h = t.named_on_battlefield("Gluttonous Hellkite")[0];
    assert_eq!(t.counters(h, "+1/+1"), 4, "{}", t.dump_log());
}

#[test]
fn madcap_experiment_deals_damage_equal_to_the_cards_revealed() {
    cr!("701.20a", "608.2c");
    // "Reveal cards from the top of your library until you reveal an artifact card. Put
    // that card onto the battlefield and the rest on the bottom of your library in a
    // random order. ~ deals damage to you equal to the number of cards revealed this way."
    let mut t = TestGame::new(2);
    t.library_top(P0, "Ornithopter");
    t.library_top(P0, "Forest");
    t.library_top(P0, "Island");
    t.lands(P0, "Mountain", 4);
    let m = t.hand(P0, "Madcap Experiment");
    t.cast(P0, m).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 1);
    assert_eq!(t.life(P0), 17, "{}", t.dump_log());
}

#[test]
fn stonework_packbeast_is_also_a_wizard_in_every_zone() {
    cr!("604.3", "604.3a");
    ruling!(
        "Stonework Packbeast",
        "The ability that adds creature types to Stonework Packbeast applies in all zones, not just the battlefield."
    );
    ruling!(
        "Veteran Adventurer",
        "The ability that adds creature types to Veteran Adventurer applies in all zones, not just the battlefield."
    );
    ruling!(
        "Tajuru Paragon",
        "The ability that adds creature types to Tajuru Paragon applies in all zones, not just the battlefield."
    );
    let mut t = TestGame::new(2);
    let in_hand = t.hand(P0, "Stonework Packbeast");
    let in_yard = t.graveyard(P0, "Veteran Adventurer");
    let in_library = t.library_top(P0, "Tajuru Paragon");
    t.g.recompute();
    for id in [in_hand, in_yard, in_library] {
        let o = t.obj_now(id);
        for ty in ["Cleric", "Rogue", "Warrior", "Wizard"] {
            assert!(
                o.chars.subtypes.iter().any(|x| x.as_str() == ty),
                "{} in {:?} isn't a {ty}: {:?}",
                o.chars.name,
                o.zone,
                o.chars.subtypes
            );
        }
    }
}

#[test]
fn skeletonizes_token_regenerates_itself() {
    cr!("701.19a", "603.7c", "111.4");
    // "Skeletonize deals 3 damage to target creature. When a creature dealt damage this
    // way dies this turn, create a 1/1 black Skeleton creature token with "{B}: Regenerate
    // this token."": the token's ability regenerates the token.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 5);
    let s = t.hand(P0, "Skeletonize");
    t.cast(P0, s).target(bears).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let token = t
        .g
        .permanents()
        .find(|o| o.chars.subtypes.iter().any(|x| x.as_str() == "Skeleton"))
        .map(|o| o.id)
        .expect("Skeleton token");
    t.lands(P0, "Swamp", 1);
    t.activate(P0, token, 0, &[]).unwrap();
    t.resolve_all();
    t.g.destroy_all(vec![token], None, false);
    t.settle();
    assert!(t.on_battlefield(token), "{}", t.dump_log());
    assert!(t.obj_now(token).tapped);
}

#[test]
fn gollum_reveals_the_top_card_the_guess_was_about() {
    cr!("608.2c", "701.20a");
    // "look at the top two cards of your library, put them back in any order, then choose
    // land or nonland. An opponent guesses whether the top card of your library is the
    // chosen kind. Reveal that card. If they guessed right, remove Gollum from combat.
    // Otherwise, you draw a card ...": "that card" is the top card; the other card looked
    // at can't be revealed or put on top instead.
    let mut t = TestGame::new(2);
    let gollum = t.battlefield(P0, "Gollum, Scheming Guide");
    t.advance_to(P0, Step::BeginningOfCombat);
    let bears = t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Forest");
    t.answer(P0, DecisionKind::Option, Answer::Index(0)); // land
    t.answer(P1, DecisionKind::Option, Answer::Index(1)); // no: a wrong guess
    t.answer_choose(P0, &[Entity::Player(P1)]); // the opponent who guesses
    // (On the old code: the card "that card" was, chosen among the two looked at.)
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(gollum, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    // The top card is revealed, not the other one looked at.
    let log = t.dump_log();
    let revealed = |name: &str| {
        log.lines()
            .any(|l| l.contains(" reveals ") && l.contains(name))
    };
    assert!(revealed("Forest"), "{log}");
    assert!(!revealed("Grizzly Bears"), "{log}");
    // The guess was wrong: P0 draws the top card, the Forest.
    assert!(t.in_hand(P0, "Forest"), "{log}");
    assert!(!t.in_hand(P0, "Grizzly Bears"), "{log}");
}
