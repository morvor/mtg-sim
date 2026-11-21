//! roundtrip-pairs-6 (docs/agents/items/roundtrip-pairs-6.md): untriaged round-trip
//! mismatches, checked in play where a mismatch could have meant the engine does something
//! other than what the card says.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn spider_punk_abilities_cant_be_countered() {
    cr!("701.6a");
    // "Spells and abilities can't be countered." is compiled as a restriction on every
    // object (`Restriction::CantBeCountered(Filter::Any)`, rendered "permanents"): an
    // activated ability on the stack can't be countered either.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Spider-Punk");
    let tim = t.battlefield(P0, "Prodigal Sorcerer");
    t.lands(P1, "Island", 1);
    let stifle = t.hand(P1, "Stifle");
    t.activate(P0, tim, 0, &[Entity::Player(P1)]).unwrap();
    let ability = *t.g.stack.last().unwrap();
    t.cast(P1, stifle).target(ability).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 19, "the ability resolved");
    assert!(t.in_graveyard(P1, "Stifle"));
}

#[test]
fn aura_graft_with_no_legal_place_leaves_the_aura_where_it_is() {
    cr!("701.3b");
    ruling!(
        "Aura Graft",
        "If there is no legal place to move the enchantment, then it doesn't move but you still control it."
    );
    // "Attach it to another permanent it can enchant": the choice is among permanents
    // other than the one it's attached to that it can legally enchant
    // (`Filter::CanBeAttachedBy`); with none, it stays, and you control it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = t.battlefield(P1, "Holy Strength");
    assert!(t.g.attach(aura, Entity::Object(bears)));
    let spell = t.hand(P0, "Aura Graft");
    t.lands(P0, "Island", 2);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, spell).target(Entity::Object(aura)).go();
    t.resolve_all();
    assert_eq!(t.obj_now(aura).controller, P0);
    assert_eq!(t.obj_now(aura).attached_to, Some(Entity::Object(bears)));
}

#[test]
fn galvanoth_leaves_a_noninstant_card_on_top() {
    ruling!(
        "Galvanoth",
        "If the card you look at isn't an instant or sorcery card, it remains on top of your library."
    );
    // "You may look at the top card of your library. You may cast it without paying its
    // mana cost if it's an instant or sorcery spell." (rendered as a condition on the top
    // card): a creature card stays where it is.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Galvanoth");
    let card = t.library_top(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.set_step(P0, Step::Untap);
    t.advance_to(P0, Step::Upkeep);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.g.library_top(P0), Some(t.g.current(card)));
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn blinding_beam_lasts_through_the_targeted_players_next_untap_step() {
    cr!("502.3", "611.2a");
    // "Creatures don't untap during target player's next untap step." is compiled as
    // "creatures target player controls don't untap" through the next untap step: the
    // targeted player's, whoever's turn it is cast on (`turn::untap_restricted_player`).
    let mut t = TestGame::new(2);
    for _ in 0..6 {
        t.library_top(P0, "Forest");
        t.library_top(P1, "Forest");
    }
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    let beam = t.hand(P0, "Blinding Beam");
    // Cast during P1's turn, after P1's untap step.
    t.set_step(P1, Step::PrecombatMain);
    t.g.tap(bears);
    t.cast(P0, beam).modes(&[1]).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    // P0's untap step passes: not P1's.
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(bears).tapped);
    // P1's next untap step: the Bears stay tapped.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(bears).tapped, "doesn't untap during P1's next untap step");
    // The untap step after that: they untap.
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn tinybones_checks_hands_as_the_ability_resolves() {
    cr!("608.2h");
    ruling!(
        "Tinybones, Trinket Thief",
        "Tinybones checks which opponents have no cards in hand only as its last ability resolves."
    );
    // "{4}{B}{B}: Each opponent with no cards in hand loses 10 life." (rendered "each
    // opponent ifs they have no cards in hand"): an opponent who casts their last card in
    // response has no cards in hand as it resolves; one who keeps a card doesn't lose.
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let tiny = t.battlefield(P0, "Tinybones, Trinket Thief");
    t.lands(P0, "Swamp", 6);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.hand(P2, "Grizzly Bears");
    t.activate(P0, tiny, 0, &[]).unwrap();
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.life(P1), 10, "no cards in hand as the ability resolves");
    assert_eq!(t.life(P2), 20, "a card in hand");
}

#[test]
fn chain_of_plasma_lets_a_targeted_player_discard() {
    cr!("115.4");
    // "~ deals 3 damage to any target. Then that player or that permanent's controller
    // may discard a card.": `ControllerOf(Target(0))` of a player target is that player.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 2);
    let chain = t.hand(P0, "Chain of Plasma");
    t.hand(P1, "Grizzly Bears");
    t.answer_yes(P1, true);
    t.answer_yes(P1, false);
    t.cast(P0, chain).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "the targeted player discarded");
}

// Review: the "Remaining" cards whose ASTs or wordings left room for a different behavior,
// checked in play.

#[test]
fn rakdos_destroys_the_creatures_whose_coins_came_up_tails_at_the_same_time() {
    cr!("608.2f", "702.12b");
    ruling!(
        "Rakdos, the Showstopper",
        "All of the creatures whose coins come up tails are destroyed at the same time."
    );
    // "Destroy each creature whose coin comes up tails." is one action on all of them: a
    // Knight that Knight Exemplar gives indestructible still has it as the Exemplar is
    // destroyed with it, so it survives (it wouldn't if each creature were destroyed as
    // its coin came up tails).
    let mut t = TestGame::new(2);
    let exemplar = t.battlefield(P0, "Knight Exemplar");
    let knight = t.battlefield(P0, "White Knight");
    // Both coins come up tails (the Exemplar's flip first).
    t.g.dice.loaded_coins.extend([false, false]);
    let rakdos = t.enter(P0, "Rakdos, the Showstopper");
    t.resolve_all();
    assert!(t.g.dice.loaded_coins.is_empty(), "two flips");
    assert!(t.on_battlefield(rakdos));
    assert!(t.in_graveyard(P0, "Knight Exemplar"));
    assert!(!t.on_battlefield(exemplar));
    assert!(
        t.on_battlefield(knight),
        "indestructible as the Exemplar is destroyed with it"
    );
}

#[test]
fn princess_yue_doesnt_return_when_she_dies_as_the_land_moon() {
    cr!("603.4", "603.10a");
    ruling!("Princess Yue", "she is no longer a creature");
    // "When Princess Yue dies, if she was a nonland creature, return this card to the
    // battlefield tapped under your control. She's a land named Moon.": the intervening
    // "if" looks at her as she last existed on the battlefield.
    let mut t = TestGame::new(2);
    let yue = t.battlefield(P0, "Princess Yue");
    t.lands(P1, "Swamp", 3);
    t.lands(P1, "Mountain", 3);
    let murder = t.hand(P1, "Murder");
    let rain = t.hand(P1, "Stone Rain");
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, murder).target(Entity::Object(yue)).go();
    t.resolve_all();
    let moon = t.named_on_battlefield("Moon");
    assert_eq!(moon.len(), 1, "she returns as a land named Moon");
    let moon = moon[0];
    assert!(t.obj_now(moon).chars.is_land());
    assert!(!t.obj_now(moon).is_creature());
    assert!(t.obj_now(moon).tapped);
    assert_eq!(t.obj_now(moon).controller, P0);
    // Dying as a land, she wasn't a nonland creature: she stays in the graveyard.
    t.cast(P1, rain).target(Entity::Object(moon)).go();
    t.resolve_all();
    assert!(t.named_on_battlefield("Moon").is_empty());
    assert!(t.named_on_battlefield("Princess Yue").is_empty());
    assert!(t.in_graveyard(P0, "Princess Yue"));
}

#[test]
fn counterbalance_cant_reveal_the_top_card_of_an_empty_library() {
    cr!("608.2d");
    // "You may reveal the top card of your library. If you do, counter that spell if it
    // has the same mana value as the revealed card.": with no card to reveal, revealing
    // can't be chosen, so a spell with mana value 0 isn't countered.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Counterbalance");
    t.g.player_mut(P0).library.clear();
    t.answer_yes(P0, true);
    let thopter = t.hand(P1, "Ornithopter");
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, thopter).go();
    t.resolve_all();
    assert!(t.on_battlefield(thopter), "not countered");
}

#[test]
fn ogre_battlecaster_pumps_itself_only_when_the_spell_is_cast() {
    cr!("603.12", "614.1a");
    // "Whenever this creature attacks, you may cast target instant or sorcery card from
    // your graveyard by paying {R}{R} in addition to its other costs. If that spell would
    // be put into a graveyard, exile it instead. When you cast that spell, this creature
    // gets +X/+0 until end of turn, where X is that spell's mana value."
    for cast in [false, true] {
        let mut t = TestGame::new(2);
        let ogre = t.battlefield(P0, "Ogre Battlecaster");
        t.lands(P0, "Mountain", 3);
        let bolt = t.graveyard(P0, "Lightning Bolt");
        t.set_step(P0, Step::PrecombatMain);
        t.answer_targets(P0, &[Entity::Object(bolt)]);
        t.answer_yes(P0, cast);
        if cast {
            t.answer_targets(P0, &[Entity::Player(P1)]);
        }
        t.attack(&[(ogre, Entity::Player(P1))], &[]);
        if cast {
            assert_eq!(t.pt(ogre).0, 4, "+1/+0: the Bolt's mana value");
            assert!(t.in_exile("Lightning Bolt"), "exiled instead of going to the graveyard");
            assert!(!t.in_graveyard(P0, "Lightning Bolt"));
            assert_eq!(t.life(P1), 20 - 3 - 4);
        } else {
            assert_eq!(t.pt(ogre).0, 3, "not cast: no bonus");
            assert!(t.in_graveyard(P0, "Lightning Bolt"));
            assert_eq!(t.life(P1), 17);
        }
    }
}

#[test]
fn kalitas_token_has_the_destroyed_creatures_last_known_power_and_toughness() {
    cr!("608.2h");
    ruling!("Kalitas, Bloodchief of Ghet", "last existence on the battlefield");
    ruling!(
        "Kalitas, Bloodchief of Ghet",
        "You get the token, not the player who controlled the creature."
    );
    ruling!(
        "Kalitas, Bloodchief of Ghet",
        "If the targeted creature has indestructible or regenerates, you won't get a Vampire token."
    );
    // "Destroy target creature. If that creature dies this way, create a black Vampire
    // creature token. Its power is equal to that creature's power and its toughness is
    // equal to that creature's toughness."
    let mut t = TestGame::new(2);
    let kalitas = t.battlefield(P0, "Kalitas, Bloodchief of Ghet");
    t.lands(P0, "Swamp", 3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), "+1/+1", 2, None);
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, kalitas, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let tokens = t.named_on_battlefield("Vampire Token");
    assert_eq!(tokens.len(), 1);
    assert_eq!(t.pt(tokens[0]), (4, 4), "as the Bears last existed on the battlefield");
    assert_eq!(t.obj_now(tokens[0]).controller, P0);
    // An indestructible creature isn't destroyed: no token.
    let mut t = TestGame::new(2);
    let kalitas = t.battlefield(P0, "Kalitas, Bloodchief of Ghet");
    t.lands(P0, "Swamp", 3);
    let myr = t.battlefield(P1, "Darksteel Myr");
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, kalitas, 0, &[Entity::Object(myr)]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(myr));
    assert!(t.named_on_battlefield("Vampire Token").is_empty());
}

#[test]
fn rashmi_puts_a_revealed_land_into_your_hand() {
    ruling!(
        "Rashmi, Eternities Crafter",
        "If the revealed card is a land card, you put it into your hand."
    );
    // "reveal the top card of your library. You may cast it without paying its mana cost
    // if it's a spell with lesser mana value. If you don't cast it, put it into your
    // hand.": a land has mana value 0 but isn't a spell.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rashmi, Eternities Crafter");
    t.lands(P0, "Forest", 2);
    let forest = t.library_top(P0, "Forest");
    let bears = t.hand(P0, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    // Wanting to cast it changes nothing: a land can't be cast.
    t.answer_yes(P0, true);
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.zone(forest), mtg_engine::object::Zone::Hand(P0));
    assert_eq!(t.named_on_battlefield("Forest").len(), 2, "not put onto the battlefield");
    assert!(t.on_battlefield(bears));
}
