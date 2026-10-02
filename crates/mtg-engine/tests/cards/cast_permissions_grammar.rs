//! The permission grammar (`oracle/patterns/permission_grammar.rs`): permissions to play
//! the cards an earlier instruction moved ("You may play that card this turn"), some of
//! them ("one of those cards", "an instant or sorcery spell from among those exiled
//! cards"), for a while ("for as long as you control ~", "until you exile another card
//! with ~", "during your next turn"), on conditions ("if you control a Kavu"), and for
//! another player ("its owner may cast it").

use mtg_engine::object::{CastMethod, Zone};
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

fn bolt_p1(t: &mut TestGame, bolt: ObjectId) -> bool {
    let b = t.g.current(bolt);
    let ok = t.cast_with(P0, b, &[Entity::Player(P1)]).is_ok();
    if ok {
        t.resolve_all();
    }
    ok
}

#[test]
fn ark_of_hunger_plays_the_milled_card_this_turn_only() {
    cr!("601.3", "611.2a");
    assert_supported("Ark of Hunger");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let ark = t.battlefield(P0, "Ark of Hunger");
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.activate(P0, ark, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Graveyard(P0));
    assert!(bolt_p1(&mut t, bolt));
    // The Bolt left the graveyard: Ark's other ability dealt 1 damage too.
    assert_eq!(t.life(P1), 16);

    // Not played this turn: the permission ends.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let ark = t.battlefield(P0, "Ark of Hunger");
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.activate(P0, ark, 0, &[]).unwrap();
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(!bolt_p1(&mut t, bolt));
}

/// `p` gains control of the permanent.
fn gain_control(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    ctx.targets = vec![vec![Entity::Object(t.g.current(id))]];
    t.g.exec(
        &ability::Effect::GainControl {
            what: ability::Sel::Target(0),
            who: ability::PlayerRef::You,
            duration: ability::Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

#[test]
fn lightning_plays_the_cards_only_while_you_control_it() {
    cr!("611.2b");
    ruling!(
        "Lightning, Security Sergeant",
        "If another player gains control of Lightning, you won't be able to play any of the exiled cards, even if you later regain control of Lightning."
    );
    assert_supported("Lightning, Security Sergeant");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let lightning = t.battlefield(P0, "Lightning, Security Sergeant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(lightning, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
    // Still there on a later turn.
    t.advance_to(P0, Step::PrecombatMain);
    assert!(bolt_p1(&mut t, bolt));
    assert_eq!(t.life(P1), 15);

    // Another player gains control of it: the permission ends, even after it's back.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let lightning = t.battlefield(P0, "Lightning, Security Sergeant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(lightning, Entity::Player(P1))], &[]);
    t.resolve_all();
    t.advance_to(P0, Step::PostcombatMain);
    gain_control(&mut t, P1, lightning);
    gain_control(&mut t, P0, lightning);
    assert!(!bolt_p1(&mut t, bolt));
}

#[test]
fn unstable_amulet_until_you_exile_another_card() {
    cr!("611.2b");
    ruling!(
        "Unstable Amulet",
        "If Unstable Amulet leaves the battlefield before you play the most recently exiled card, you can play that card for as long as it remains exiled."
    );
    assert_supported("Unstable Amulet");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let amulet = t.enter(P0, "Unstable Amulet");
    t.resolve_all();
    let shock = t.library_top(P0, "Shock");
    t.activate(P0, amulet, 0, &[]).unwrap();
    t.resolve_all();
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.g.add_counters(Entity::Player(P0), "energy", 2, None);
    t.g.obj_mut(amulet).tapped = false;
    t.activate(P0, amulet, 0, &[]).unwrap();
    t.resolve_all();
    // The earlier card's permission ended when another card was exiled with the Amulet.
    let shock_x = t.g.current(shock);
    assert_eq!(t.zone(shock_x), Zone::Exile);
    assert!(t.cast_with(P0, shock_x, &[Entity::Player(P1)]).is_err());
    // The latest one can be played, even after the Amulet is gone.
    t.g.destroy(amulet, None);
    t.settle();
    assert!(bolt_p1(&mut t, bolt));
    // Unstable Amulet's damage for the spell cast from exile isn't there any more.
    assert_eq!(t.life(P1), 17);
}

#[test]
fn flameskull_plays_one_of_those_cards() {
    cr!("611.2a");
    assert_supported("Flameskull");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let skull = t.battlefield(P0, "Flameskull");
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.g.destroy(skull, None);
    t.resolve_all();
    let skull_x = t.g.current(skull);
    assert_eq!(t.zone(skull_x), Zone::Exile);
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
    // Casting the Bolt uses the permission: Flameskull can't be cast this way any more.
    assert!(bolt_p1(&mut t, bolt));
    let skull_x = t.g.current(skull);
    assert!(t.cast_with(P0, skull_x, &[]).is_err());
    assert_eq!(t.zone(skull_x), Zone::Exile);
}

#[test]
fn chandra_hopes_beacon_casts_one_instant_or_sorcery_spell_from_among_them() {
    cr!("601.3e", "611.2a");
    ruling!(
        "Chandra, Hope's Beacon",
        "the spell you cast must be an instant or sorcery spell, although the exiled card doesn't necessarily have to be"
    );
    assert_supported("Chandra, Hope's Beacon");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 6);
    let chandra = t.battlefield(P0, "Chandra, Hope's Beacon");
    let bears = t.library_top(P0, "Grizzly Bears");
    let shock = t.library_top(P0, "Shock");
    let giant = t.library_top(P0, "Bonecrusher Giant");
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.library_top(P0, "Mountain");
    t.activate(P0, chandra, 1, &[]).unwrap();
    t.resolve_all();
    // A creature card isn't an instant or sorcery spell.
    let bears_x = t.g.current(bears);
    assert_eq!(t.zone(bears_x), Zone::Exile);
    assert!(t.cast_with(P0, bears_x, &[]).is_err());
    // The Adventure of a creature card is: Stomp is cast (and uses up the permission).
    let giant_x = t.g.current(giant);
    t.cast(P0, giant_x)
        .method(CastMethod::Half(1))
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    // Stomp and Chandra's copy of it.
    assert_eq!(t.life(P1), 16);
    // Only one spell: the others can't be cast any more.
    assert!(!bolt_p1(&mut t, bolt));
    let shock_x = t.g.current(shock);
    assert!(t.cast_with(P0, shock_x, &[Entity::Player(P1)]).is_err());
}

#[test]
fn galvanic_relay_only_during_your_next_turn() {
    cr!("611.2a");
    ruling!(
        "Galvanic Relay",
        "You can play the exiled card only on your next turn after Galvanic Relay resolves."
    );
    assert_supported("Galvanic Relay");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let relay = t.hand(P0, "Galvanic Relay");
    t.cast(P0, relay).go();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
    assert!(!bolt_p1(&mut t, bolt));
    // Not during the opponent's turn either.
    t.advance_to(P1, Step::Upkeep);
    let b = t.g.current(bolt);
    t.g.turn.priority = Some(P0);
    assert!(t.g.cast_spell(P0, b, CastMethod::Normal).is_err());
    t.advance_to(P0, Step::Upkeep);
    assert!(bolt_p1(&mut t, bolt));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn possibility_technician_plays_it_while_you_control_a_kavu() {
    cr!("611.2a");
    assert_supported("Possibility Technician");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let tech = t.enter(P0, "Possibility Technician");
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
    // Possibility Technician is a Kavu: the card may be played.
    let tech = t.g.current(tech);
    t.g.destroy(tech, None);
    t.settle();
    // Without a Kavu, it can't (the permission lasts while it's exiled).
    assert!(!bolt_p1(&mut t, bolt));
    t.battlefield(P0, "Possibility Technician");
    assert!(bolt_p1(&mut t, bolt));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn release_to_the_wind_its_owner_may_cast_it_for_free() {
    cr!("118.9", "611.2a");
    ruling!(
        "Release to the Wind",
        "Casting the exiled card follows the normal timing rules for casting that card."
    );
    assert_supported("Release to the Wind");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Release to the Wind");
    t.cast(P0, spell).target(bears).go();
    t.resolve_all();
    let bears_x = t.g.current(bears);
    assert_eq!(t.zone(bears_x), Zone::Exile);
    // Not its caster's: its owner's.
    t.g.turn.priority = Some(P0);
    assert!(t.g.cast_spell(P0, bears_x, CastMethod::Free).is_err());
    // The owner casts it without paying its mana cost, at sorcery speed.
    t.advance_to(P1, Step::PrecombatMain);
    t.g.turn.priority = Some(P1);
    t.g.cast_spell(P1, bears_x, CastMethod::Free)
        .expect("cast for free");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}
