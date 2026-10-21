//! Permissions over the cards an instruction moved (cards3-cast-zones): in the body of a
//! delayed or reflexive triggered ability (End-Blaze Epiphany, Doom Reigns Supreme),
//! "from among the nonland cards exiled this way" (Etali, Primal Conqueror), "Choose one."
//! after exiling cards (Chandra, Flameshaper), the cards exiled from a player's hand
//! before they draw that many (Elder Brain), and "from among cards exiled with it" with a
//! rider sending the spell to the bottom of its owner's library (Kylox's Voltstrider).

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

/// Whether the card is the bottom card of its owner's library.
fn on_bottom(t: &TestGame, p: PlayerId, card: ObjectId) -> bool {
    let lib = &t.g.player(p).library;
    let top = t.g.library_top(p).unwrap();
    let bottom = if lib[0] == top { lib[lib.len() - 1] } else { lib[0] };
    bottom == card
}

fn can_cast_at(t: &mut TestGame, p: PlayerId, card: ObjectId, target: Entity) -> bool {
    let c = t.g.current(card);
    t.answer_targets(p, &[target]);
    t.g.turn.priority = Some(p);
    let ok = t.g.cast_spell(p, c, object::CastMethod::Normal).is_ok();
    t.g.flush_events();
    if ok {
        t.resolve_all();
    } else {
        t.clear_answers();
    }
    ok
}

#[test]
fn doom_reigns_supreme_casts_up_to_two_of_the_exiled_cards_free() {
    cr!("603.12", "608.2g", "118.9");
    assert_supported("Doom Reigns Supreme");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let doom = t.battlefield(P0, "Doom Reigns Supreme");
    t.library_top(P1, "Forest");
    let growth = t.library_top(P1, "Giant Growth");
    let bears = t.library_top(P1, "Grizzly Bears");
    let shock = t.library_top(P1, "Shock");
    let bolt = t.library_top(P1, "Lightning Bolt");
    t.g.add_counters(Entity::Object(doom), "plan", 4, None);
    t.settle();
    assert!(t.on_battlefield(doom));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[Entity::Object(bolt), Entity::Object(shock)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.add_counters(Entity::Object(doom), "plan", 1, None);
    t.resolve_all();
    assert!(!t.on_battlefield(doom));
    assert_eq!(t.life(P1), 15, "{}", t.dump_log());
    // The others stay exiled, and can't be cast later.
    let bears_x = t.g.current(bears);
    assert_eq!(t.zone(bears_x), Zone::Exile);
    assert_eq!(t.zone(t.g.current(growth)), Zone::Exile);
    t.lands(P0, "Forest", 2);
    assert!(t.cast_with(P0, bears_x, &[]).is_err());
}

#[test]
fn end_blaze_epiphany_plays_the_chosen_card_until_the_end_of_your_next_turn() {
    cr!("603.7", "611.2a");
    ruling!(
        "End-Blaze Epiphany",
        "You pay all costs and follow all timing rules for cards played this way."
    );
    assert_supported("End-Blaze Epiphany");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 4);
    let target = t.battlefield(P1, "Grizzly Bears");
    let mountain = t.library_top(P0, "Mountain");
    let bolt = t.library_top(P0, "Lightning Bolt");
    let epiphany = t.hand(P0, "End-Blaze Epiphany");
    // X = 2: the Bears die; two cards are exiled (its power), and the Bolt is chosen.
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.cast(P0, epiphany).x(2).target(target).go();
    t.resolve_all();
    let bolt_x = t.g.current(bolt);
    let mountain_x = t.g.current(mountain);
    assert_eq!(t.zone(bolt_x), Zone::Exile, "{}", t.dump_log());
    assert_eq!(t.zone(mountain_x), Zone::Exile);
    // The other exiled card can't be played.
    assert!(t.play_land(P0, mountain_x).is_err());
    // The Bolt can be cast during the next turn too.
    t.advance_to(P1, Step::Upkeep);
    assert!(can_cast_at(&mut t, P0, bolt_x, Entity::Player(P1)));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn end_blaze_epiphany_does_nothing_more_if_the_creature_survives() {
    cr!("603.7");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 2);
    let target = t.battlefield(P1, "Gray Ogre");
    let bolt = t.library_top(P0, "Lightning Bolt");
    let epiphany = t.hand(P0, "End-Blaze Epiphany");
    t.cast(P0, epiphany).x(1).target(target).go();
    t.resolve_all();
    assert!(t.on_battlefield(target));
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Library(P0));
}

#[test]
fn etali_casts_any_number_of_the_nonland_cards_exiled_free() {
    cr!("608.2g", "118.9");
    ruling!(
        "Etali, Primal Conqueror // Etali, Primal Sickness",
        "Any cards not cast, including land cards, remain in exile. They can’t be cast on later turns."
    );
    ruling!(
        "Etali, Primal Conqueror // Etali, Primal Sickness",
        "If you cast any of the exiled cards, you do so as part of the resolution of the triggered ability."
    );
    assert_supported("Etali, Primal Conqueror // Etali, Primal Sickness");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let shock = t.library_top(P0, "Shock");
    let bolt = t.library_top(P1, "Lightning Bolt");
    let mountain = t.library_top(P1, "Mountain");
    t.answer_choose(P0, &[Entity::Object(shock), Entity::Object(bolt)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Etali, Primal Conqueror // Etali, Primal Sickness");
    t.resolve_all();
    assert_eq!(t.life(P1), 15, "{}\n{:#?}", t.dump_log(), t.asked());
    // P1's Bolt was cast by P0 and went to P1's graveyard; the Mountain stays exiled.
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert_eq!(t.zone(t.g.current(mountain)), Zone::Exile);
}

#[test]
fn chandra_flameshaper_plays_the_one_card_chosen_this_turn() {
    cr!("611.2a", "606.3");
    ruling!(
        "Chandra, Flameshaper",
        "You pay all costs and follow all timing rules for cards played with the permission granted by Chandra's first ability."
    );
    assert_supported("Chandra, Flameshaper");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let chandra = t.battlefield(P0, "Chandra, Flameshaper");
    let shock = t.library_top(P0, "Shock");
    t.library_top(P0, "Mountain");
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.activate(P0, chandra, 0, &[]).unwrap();
    t.resolve_all();
    let bolt_x = t.g.current(bolt);
    let shock_x = t.g.current(shock);
    assert_eq!(t.zone(shock_x), Zone::Exile);
    // Paid for with the {R}{R}{R} the ability added.
    assert!(!can_cast_at(&mut t, P0, shock_x, Entity::Player(P1)));
    assert!(can_cast_at(&mut t, P0, bolt_x, Entity::Player(P1)));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn elder_brain_plays_the_cards_exiled_from_that_players_hand() {
    cr!("611.2a", "609.4b");
    ruling!("Elder Brain", "The cards are exiled face up.");
    assert_supported("Elder Brain");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let brain = t.battlefield(P0, "Elder Brain");
    t.lands(P0, "Swamp", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let forest = t.hand(P1, "Forest");
    t.attack(&[(brain, Entity::Player(P1))], &[]);
    let bolt_x = t.g.current(bolt);
    let forest_x = t.g.current(forest);
    assert_eq!(t.zone(bolt_x), Zone::Exile);
    assert_eq!(t.zone(forest_x), Zone::Exile);
    // P1 drew as many cards as were exiled.
    assert_eq!(t.hand_size(P1), 2);
    assert_eq!(t.life(P1), 14);
    // The Bolt is cast with a Swamp's mana (as though it were red); the Forest is played.
    t.advance_to(P0, Step::PostcombatMain);
    assert!(can_cast_at(&mut t, P0, bolt_x, Entity::Player(P1)));
    assert_eq!(t.life(P1), 11);
    t.play_land(P0, forest_x).expect("play the exiled Forest");
    assert_eq!(t.zone(t.g.current(forest)), Zone::Battlefield);
}

#[test]
fn kylox_voltstrider_casts_a_spell_exiled_with_it_and_puts_it_on_the_bottom() {
    cr!("607.2a", "608.2g", "614.1a");
    ruling!(
        "Kylox's Voltstrider",
        "You choose whether or not to cast a spell from among the exiled cards as Kylox's Voltstrider's triggered ability resolves."
    );
    assert_supported("Kylox's Voltstrider");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let kylox = t.battlefield(P0, "Kylox's Voltstrider");
    t.lands(P0, "Mountain", 1);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Divination");
    let other = t.exile(P0, "Shock");
    // Collect evidence 6: the three cards (total mana value 6) are exiled with it.
    t.activate(P0, kylox, 0, &[]).expect("collect evidence");
    t.resolve_all();
    let bolt_x = t.g.current(bolt);
    assert_eq!(t.zone(bolt_x), Zone::Exile, "{}", t.dump_log());
    t.answer_choose(P0, &[Entity::Object(bolt_x)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.attack(&[(kylox, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 13, "{}", t.dump_log());
    // The Bolt went to the bottom of its owner's library, not the graveyard.
    let bolt_now = t.g.current(bolt);
    assert_eq!(t.zone(bolt_now), Zone::Library(P0));
    assert!(on_bottom(&t, P0, bolt_now));
    // A card exiled some other way isn't among them.
    assert_eq!(t.zone(other), Zone::Exile);
}

#[test]
fn arvinox_casts_permanent_spells_from_among_the_face_down_exiled_cards() {
    cr!("406.3b", "611.2a", "609.4b");
    assert_supported("Arvinox, the Mind Flail");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.battlefield(P0, "Arvinox, the Mind Flail");
    t.lands(P0, "Island", 2);
    let bears = t.library_top(P1, "Grizzly Bears");
    mtg_engine::library::put_on_bottom(&mut t.g, P1, &[bears]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let bears_x = t.g.current(bears);
    assert_eq!(t.zone(bears_x), Zone::Exile);
    assert!(t.g.obj(bears_x).face_down);
    // On a later turn, the Bears are cast with Islands (as though any color).
    t.advance_to(P0, Step::PrecombatMain);
    t.g.turn.priority = Some(P0);
    assert!(t.cast_with(P0, bears_x, &[]).is_ok(), "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn arvinox_doesnt_cast_nonpermanent_spells() {
    cr!("601.3");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.battlefield(P0, "Arvinox, the Mind Flail");
    t.lands(P0, "Mountain", 1);
    let bolt = t.library_top(P1, "Lightning Bolt");
    mtg_engine::library::put_on_bottom(&mut t.g, P1, &[bolt]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let bolt_x = t.g.current(bolt);
    assert_eq!(t.zone(bolt_x), Zone::Exile);
    t.advance_to(P1, Step::Upkeep);
    assert!(!can_cast_at(&mut t, P0, bolt_x, Entity::Player(P1)));
}
