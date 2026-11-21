//! roundtrip-pairs-5 (docs/agents/items/roundtrip-pairs-5.md): untriaged round-trip
//! mismatches (the 3+-card `gap:` clusters, two-card clusters and the single-card tail).

use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn the_deck_of_many_things_creature_owner_loses_when_it_dies() {
    cr!("603.7a", "104.3e", "108.3");
    // "20 | Put a creature card from any graveyard onto the battlefield under your control.
    // When that creature dies, its owner loses the game." "Its owner" is the owner of the
    // creature that died (the delayed ability has no targets): P1's Craw Wurm, so P1 loses.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let deck = t.battlefield(P0, "The Deck of Many Things");
    t.lands(P0, "Plains", 2);
    let wurm = t.graveyard(P1, "Craw Wurm");
    t.g.dice.loaded.push_back(20);
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    t.activate(P0, deck, 0, &[]).unwrap();
    t.resolve_all();
    let wurms = t.named_on_battlefield("Craw Wurm");
    assert_eq!(wurms.len(), 1);
    t.g.destroy(wurms[0], None);
    t.settle();
    t.resolve_all();
    assert!(t.has_lost(P1), "the Wurm's owner loses");
    assert!(!t.has_lost(P0));
}

/// Puts the real Aura or Equipment `name` onto the battlefield under `p`'s control attached
/// to `to`.
fn attach_new(t: &mut TestGame, p: PlayerId, name: &str, to: ObjectId) -> ObjectId {
    let id = t.battlefield(p, name);
    assert!(
        t.g.attach(id, Entity::Object(to)),
        "{name} couldn't be attached"
    );
    t.g.recompute();
    id
}

#[test]
fn pact_weapon_reveals_the_drawn_card_and_uses_its_mana_value() {
    cr!("508.1m", "603.2");
    // "Whenever equipped creature attacks, draw a card and reveal it. The creature gets
    // +X/+X until end of turn and you lose X life, where X is that card's mana value."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Pact Weapon", bears);
    t.library_top(P0, "Craw Wurm");
    t.set_step(P0, Step::PrecombatMain);
    let life = t.life(P0);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.resolve_all();
    assert!(t.in_hand(P0, "Craw Wurm"));
    assert_eq!(t.pt(bears), (8, 8), "Craw Wurm's mana value is 6");
    assert_eq!(t.life(P0), life - 6);
}

#[test]
fn pact_weapon_gives_no_bonus_when_the_draw_is_replaced_by_draws() {
    cr!("614.1a", "614.11");
    ruling!(
        "Pact Weapon",
        "This is true even if the card draw is replaced by one or more card draws."
    );
    // Thought Reflection: "If you would draw a card, draw two cards instead." The card
    // draw was replaced, so no card is "that card": no bonus, no life lost.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Pact Weapon", bears);
    t.battlefield(P0, "Thought Reflection");
    t.library_top(P0, "Craw Wurm");
    t.library_top(P0, "Craw Wurm");
    t.set_step(P0, Step::PrecombatMain);
    let life = t.life(P0);
    let hand = t.hand_size(P0);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2, "two cards drawn instead");
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.life(P0), life);
}

#[test]
fn chain_of_vapor_asks_the_bounced_permanents_controller_not_its_owner() {
    cr!("608.2h");
    // "Return target nonland permanent to its owner's hand. Then that permanent's
    // controller may sacrifice a land of their choice. If the player does, they may copy
    // ~": the controller as the permanent last existed on the battlefield (P0, through
    // Control Magic), not its owner (P1).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Control Magic", bears);
    assert_eq!(t.obj_now(bears).controller, P0);
    t.lands(P0, "Island", 1);
    t.lands(P1, "Island", 1);
    let chain = t.hand(P1, "Chain of Vapor");
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.cast(P1, chain).target(bears).go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Island"), "P0 sacrificed a land");
    assert!(!t.in_graveyard(P1, "Island"));
}

#[test]
fn skyserpent_seeker_puts_only_the_land_cards_revealed_onto_the_battlefield() {
    cr!("701.20a", "608.2c");
    // "Exhaust — {4}: Reveal cards from the top of your library until you reveal two land
    // cards. Put those land cards onto the battlefield tapped and the rest on the bottom of
    // your library in a random order." The nonland cards revealed go to the bottom, not
    // onto the battlefield.
    let mut t = TestGame::new(2);
    let seeker = t.battlefield(P0, "Skyserpent Seeker");
    t.lands(P0, "Mountain", 4);
    // From the top: Grizzly Bears, Forest, Lightning Bolt, Island, Plains.
    let plains = t.library_top(P0, "Plains");
    for n in ["Island", "Lightning Bolt", "Forest", "Grizzly Bears"] {
        t.library_top(P0, n);
    }
    let before = t.library_size(P0);
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, seeker, 0, &[]).unwrap();
    t.resolve_all();
    for n in ["Forest", "Island"] {
        let l = t.named_on_battlefield(n);
        assert_eq!(l.len(), 1, "{n}");
        assert!(t.obj_now(l[0]).tapped, "{n} enters tapped");
    }
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.named_on_battlefield("Plains").is_empty());
    let lib = t.g.player(P0).library.clone();
    assert_eq!(lib.len(), before - 2, "only the two lands left the library");
    assert_eq!(
        lib.last(),
        Some(&plains),
        "the unrevealed card stays on top"
    );
    let bottom: Vec<String> = lib[..2]
        .iter()
        .map(|id| t.obj_now(*id).chars.name.to_string())
        .collect();
    assert!(bottom.contains(&"Grizzly Bears".to_string()));
    assert!(bottom.contains(&"Lightning Bolt".to_string()));
    assert_eq!(t.counters(seeker, "+1/+1"), 1);
}

#[test]
fn kaito_dancing_shadow_returns_only_one_of_the_creatures() {
    cr!("608.2c");
    // "Whenever one or more creatures you control deal combat damage to a player, you may
    // return one of them to its owner's hand."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kaito, Dancing Shadow");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(elves)]);
    t.attack(
        &[(bears, Entity::Player(P1)), (elves, Entity::Player(P1))],
        &[],
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_hand(P0, "Llanowar Elves"));
    assert!(t.on_battlefield(bears), "the other creature stays");
}

#[test]
fn taster_of_wares_lets_you_cast_only_the_exiled_card_not_the_others_revealed() {
    cr!("608.2c", "601.3");
    // "target opponent reveals X cards from their hand, where X is the number of Goblins
    // you control. You choose one of those cards. That player exiles it. If an instant or
    // sorcery card is exiled this way, you may cast it ...": the other revealed card stays
    // in the opponent's hand, and you can't cast it.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Goblin Piker");
    t.lands(P0, "Mountain", 2);
    let bolt = t.hand(P1, "Lightning Bolt");
    let shock = t.hand(P1, "Shock");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P1, &[Entity::Object(bolt), Entity::Object(shock)]);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.enter(P0, "Taster of Wares");
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
    assert!(t.in_hand(P1, "Shock"));
    let cast = |t: &mut TestGame, card: ObjectId| {
        let c = t.g.current(card);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.g.turn.priority = Some(P0);
        let ok = t.g.cast_spell(P0, c, CastMethod::Normal).is_ok();
        t.g.flush_events();
        if ok {
            t.resolve_all();
        } else {
            t.clear_answers();
        }
        ok
    };
    assert!(!cast(&mut t, shock), "the Shock wasn't exiled");
    assert!(cast(&mut t, bolt));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn parallax_wave_each_player_returns_the_cards_they_own() {
    cr!("110.2a");
    // "When ~ leaves the battlefield, each player returns to the battlefield all cards they
    // own exiled with it.": P1's Bears return under P1's control, P0's Elves under P0's.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let wave = t.enter(P0, "Parallax Wave");
    assert_eq!(t.counters(wave, "fade"), 5);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.activate(P0, wave, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    t.activate(P0, wave, 0, &[Entity::Object(elves)]).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Llanowar Elves"));
    t.g.destroy(wave, None);
    t.settle();
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears");
    let e = t.named_on_battlefield("Llanowar Elves");
    assert_eq!((b.len(), e.len()), (1, 1));
    assert_eq!(t.obj_now(b[0]).controller, P1);
    assert_eq!(t.obj_now(e[0]).controller, P0);
}
