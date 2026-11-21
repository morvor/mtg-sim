//! More of the permission grammar's cards: declining a spell cast as the effect resolves
//! ("You may cast it without paying its mana cost. If you don't, put it into your hand."),
//! the monarch's free spell from among cards exiled with Court of Locthwain, one spell from
//! among each player's milled cards (Locke), "until your next end step, you may play one
//! of those cards" (Unlucky Witness), restrictions on activating abilities and casting
//! (City of Solitude, Phyrexian Censor) and mana that can't pay for spells from a hand
//! (Karolina Dean).

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

fn try_cast_as(
    t: &mut TestGame,
    p: PlayerId,
    card: ObjectId,
    method: CastMethod,
    targets: &[Entity],
) -> bool {
    let c = t.g.current(card);
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    t.g.turn.priority = Some(p);
    let ok = t.g.cast_spell(p, c, method).is_ok();
    t.g.flush_events();
    if ok {
        t.resolve_all();
    } else {
        t.clear_answers();
    }
    ok
}

fn try_cast(t: &mut TestGame, p: PlayerId, card: ObjectId, targets: &[Entity]) -> bool {
    try_cast_as(t, p, card, CastMethod::Normal, targets)
}

/// Whether the card is the bottom card of its owner's library.
fn on_bottom(t: &TestGame, p: PlayerId, card: ObjectId) -> bool {
    let c = t.g.current(card);
    let lib = &t.g.player(p).library;
    let top = t.g.library_top(p).unwrap();
    let bottom = if lib[0] == top { lib[lib.len() - 1] } else { lib[0] };
    bottom == c
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
fn descendants_path_puts_a_declined_creature_card_on_the_bottom() {
    cr!("608.2c", "608.2d");
    ruling!(
        "Descendants' Path",
        "If the revealed card is a creature card that shares a creature type with a creature you control, but you choose not to cast it, it's put on the bottom of your library."
    );
    assert_supported("Descendants' Path");
    for cast in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Descendants' Path");
        t.battlefield(P0, "Grizzly Bears");
        t.advance_to(P1, Step::End);
        let bears = t.library_top(P0, "Grizzly Bears");
        t.answer_yes(P0, cast);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        if cast {
            assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
        } else {
            assert_eq!(t.zone(t.g.current(bears)), Zone::Library(P0));
            assert!(on_bottom(&t, P0, bears), "{}", t.dump_log());
        }
    }
}

#[test]
fn solstice_revelations_puts_a_declined_card_into_your_hand() {
    cr!("608.2c", "608.2d");
    assert_supported("Solstice Revelations");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let revelations = t.hand(P0, "Solstice Revelations");
    // Three Mountains: the Bolt may be cast for free; its controller declines.
    t.answer_yes(P0, false);
    t.cast(P0, revelations).go();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Hand(P0));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn breaching_dragonstorm_puts_a_card_it_doesnt_cast_into_your_hand() {
    cr!("608.2c", "608.2g");
    ruling!(
        "Breaching Dragonstorm",
        "If you choose to cast the exiled card, you do so while Breaching Dragonstorm’s first ability is resolving"
    );
    assert_supported("Breaching Dragonstorm");
    // Declined: into the hand.
    let mut t = TestGame::new(2);
    let bears = t.library_top(P0, "Grizzly Bears");
    t.answer_yes(P0, false);
    t.enter(P0, "Breaching Dragonstorm");
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bears)), Zone::Hand(P0));
    // Mana value 9 or more: it can't be cast this way, and goes into the hand too.
    let mut t = TestGame::new(2);
    let colossus = t.library_top(P0, "Darksteel Colossus");
    t.enter(P0, "Breaching Dragonstorm");
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(colossus)), Zone::Hand(P0));
    // Cast as the ability resolves: not later.
    let mut t = TestGame::new(2);
    let bears = t.library_top(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.enter(P0, "Breaching Dragonstorm");
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(bears)));
}

#[test]
fn djinn_of_wishes_exiles_a_land_it_cant_play() {
    cr!("305.2b", "608.2g");
    ruling!(
        "Djinn of Wishes",
        "If the revealed card is a land, you can play it only if it's your turn and you haven't yet played a land this turn."
    );
    assert_supported("Djinn of Wishes");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let djinn = t.enter(P0, "Djinn of Wishes");
    t.resolve_all();
    let forest = t.hand(P0, "Forest");
    t.play_land(P0, forest).expect("the land play this turn");
    let top = t.library_top(P0, "Forest");
    // The player tries to play it: no land play left, so it isn't played and is exiled.
    t.answer_yes(P0, true);
    t.activate(P0, djinn, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(top)), Zone::Exile);
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
}

#[test]
fn court_of_locthwain_the_monarch_casts_one_spell_from_among_the_exiled_cards_free() {
    cr!("118.9", "611.2a");
    assert_supported("Court of Locthwain");
    let mut t = TestGame::new(2);
    // P0 becomes the monarch.
    t.enter(P0, "Court of Locthwain");
    t.resolve_all();
    t.advance_to(P1, Step::End);
    let shock = t.library_top(P1, "Shock");
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.zone(t.g.current(shock)), Zone::Exile);
    t.advance_to(P1, Step::End);
    let bolt = t.library_top(P1, "Lightning Bolt");
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
    // A card exiled on an earlier turn is among them; no lands: for free.
    assert!(try_cast_as(&mut t, P0, shock, CastMethod::Free, &[Entity::Player(P1)]));
    assert_eq!(t.life(P1), 18);
    // One spell: the other can't be cast for free...
    assert!(!try_cast_as(&mut t, P0, bolt, CastMethod::Free, &[Entity::Player(P1)]));
    // ... but its own permission lets it be cast paying its cost, with mana of any type.
    t.lands(P0, "Island", 1);
    assert!(try_cast(&mut t, P0, bolt, &[Entity::Player(P1)]));
    assert_eq!(t.life(P1), 15);
}

#[test]
fn court_of_locthwain_permission_stays_with_you() {
    cr!("109.5", "611.2a");
    ruling!(
        "Court of Locthwain",
        "You may play the exiled cards (and spend mana of any type to do so) even if Court of Locthwain leaves the battlefield. If another player gains control of Court of Locthwain, that player can't play the cards, and you still can."
    );
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        // Not the monarch: no free spell.
        let court = t.battlefield(P0, "Court of Locthwain");
        t.lands(P0, "Island", 1);
        t.lands(P1, "Mountain", 1);
        t.advance_to(P1, Step::End);
        let bolt = t.library_top(P1, "Lightning Bolt");
        t.advance_to(P0, Step::PrecombatMain);
        assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
        if leaves {
            t.g.destroy(court, None);
            t.settle();
        } else {
            gain_control(&mut t, P1, court);
            // The card's owner, now controlling Court, can't play it.
            assert!(!try_cast(&mut t, P1, bolt, &[Entity::Player(P0)]));
        }
        assert!(try_cast(&mut t, P0, bolt, &[Entity::Player(P1)]));
        assert_eq!(t.life(P1), 17);
    }
}

#[test]
fn locke_casts_one_spell_from_among_each_players_milled_cards() {
    cr!("117.1a", "601.3");
    ruling!(
        "Locke, Treasure Hunter",
        "You must follow the normal timing permissions and restrictions of the spell you cast from among those cards."
    );
    ruling!(
        "Locke, Treasure Hunter",
        "Losing control of Locke after the last ability has triggered won't affect your ability to cast a spell from among the milled cards."
    );
    assert_supported("Locke, Treasure Hunter");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let locke = t.battlefield(P0, "Locke, Treasure Hunter");
    let spike = t.library_top(P0, "Lava Spike");
    let bolt = t.library_top(P1, "Lightning Bolt");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(locke, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(spike)), Zone::Graveyard(P0));
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Graveyard(P1));
    // A sorcery: not during combat.
    assert!(!try_cast(&mut t, P0, spike, &[Entity::Player(P1)]));
    gain_control(&mut t, P1, locke);
    t.advance_to(P0, Step::PostcombatMain);
    assert!(try_cast(&mut t, P0, spike, &[Entity::Player(P1)]));
    assert_eq!(t.life(P1), 20 - 2 - 3);
    // One spell: the opponent's milled Bolt can't be cast any more.
    assert!(!try_cast(&mut t, P0, bolt, &[Entity::Player(P1)]));
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Graveyard(P1));
}

#[test]
fn unlucky_witness_plays_one_card_paying_its_costs_with_normal_timing() {
    cr!("305.2b", "601.2h", "611.2a");
    ruling!(
        "Unlucky Witness",
        "You must pay all costs and follow all normal timing rules for a card played this way."
    );
    assert_supported("Unlucky Witness");
    let mut t = TestGame::new(2);
    let witness = t.battlefield(P0, "Unlucky Witness");
    let spike = t.library_top(P0, "Lava Spike");
    let forest = t.library_top(P0, "Forest");
    let mountain = t.hand(P0, "Mountain");
    t.g.destroy(witness, None);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(forest)), Zone::Exile);
    assert_eq!(t.zone(t.g.current(spike)), Zone::Exile);
    // The land play this turn is used: the Forest can't be played this way.
    t.play_land(P0, mountain).expect("the land play this turn");
    assert!(t.play_land(P0, t.g.current(forest)).is_err());
    // Lava Spike's cost must be paid: not with the Mountain tapped...
    let m = t.g.current(mountain);
    t.g.tap(m);
    assert!(!try_cast(&mut t, P0, spike, &[Entity::Player(P1)]));
    // ... but with it untapped.
    t.g.untap(m);
    assert!(try_cast(&mut t, P0, spike, &[Entity::Player(P1)]));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn unlucky_witness_plays_only_one_of_those_cards() {
    cr!("611.2a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let witness = t.battlefield(P0, "Unlucky Witness");
    let bolt = t.library_top(P0, "Lightning Bolt");
    let spike = t.library_top(P0, "Lava Spike");
    t.g.destroy(witness, None);
    t.resolve_all();
    assert!(try_cast(&mut t, P0, spike, &[Entity::Player(P1)]));
    assert!(!try_cast(&mut t, P0, bolt, &[Entity::Player(P1)]));
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
}

#[test]
fn unlucky_witness_permission_ends_as_your_next_end_step_begins() {
    cr!("500.4", "611.2a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let witness = t.battlefield(P0, "Unlucky Witness");
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.g.destroy(witness, None);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
    t.advance_to(P0, Step::End);
    assert!(!try_cast(&mut t, P0, bolt, &[Entity::Player(P1)]));
    // Dying during the opponent's turn: through the opponent's end step, until yours.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let witness = t.battlefield(P0, "Unlucky Witness");
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.advance_to(P1, Step::PrecombatMain);
    t.g.destroy(witness, None);
    t.resolve_all();
    t.advance_to(P1, Step::End);
    assert!(try_cast(&mut t, P0, bolt, &[Entity::Player(P1)]));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn city_of_solitude_stops_activating_abilities_including_mana_abilities() {
    cr!("602.5", "101.2");
    ruling!(
        "City of Solitude",
        "This stops players from activating mana abilities."
    );
    assert_supported("City of Solitude");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "City of Solitude");
    let gnomes = t.battlefield(P1, "Bottle Gnomes");
    let mountain = t.lands(P1, "Mountain", 1)[0];
    // P0's turn: P1 can't activate either ability.
    assert!(t.activate(P1, gnomes, 0, &[]).is_err());
    assert!(t.activate(P1, mountain, 0, &[]).is_err());
    assert!(!t.g.obj(mountain).tapped);
    // P1's turn: both can be activated.
    t.advance_to(P1, Step::PrecombatMain);
    t.activate(P1, mountain, 0, &[]).expect("a mana ability in P1's turn");
    t.activate(P1, gnomes, 0, &[]).expect("an ability in P1's turn");
    t.resolve_all();
    assert_eq!(t.life(P1), 23);
}

#[test]
fn phyrexian_censor_one_non_phyrexian_spell_each_turn() {
    cr!("601.3");
    ruling!(
        "Phyrexian Censor",
        "Phyrexian Censor looks at the entire turn to see if a player has cast a non-Phyrexian spell, even if Phyrexian Censor wasn’t on the battlefield when that spell was cast."
    );
    assert_supported("Phyrexian Censor");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    t.lands(P0, "Mountain", 2);
    let censor = t.hand(P0, "Phyrexian Censor");
    let bolt = t.hand(P0, "Lightning Bolt");
    let bolt2 = t.hand(P0, "Lightning Bolt");
    // Phyrexian Censor is a Phyrexian spell: a non-Phyrexian spell may still be cast.
    assert!(try_cast(&mut t, P0, censor, &[]));
    assert!(try_cast(&mut t, P0, bolt, &[Entity::Player(P1)]));
    assert!(!try_cast(&mut t, P0, bolt2, &[Entity::Player(P1)]));
    // A non-Phyrexian spell cast before another Censor entered counts.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    let bolt2 = t.hand(P0, "Lightning Bolt");
    assert!(try_cast(&mut t, P0, bolt, &[Entity::Player(P1)]));
    t.battlefield(P1, "Phyrexian Censor");
    assert!(!try_cast(&mut t, P0, bolt2, &[Entity::Player(P1)]));
}

#[test]
fn stolen_strategy_casts_the_exiled_cards_this_turn_with_normal_timing() {
    cr!("117.1a", "609.4b", "611.2a");
    ruling!(
        "Stolen Strategy",
        "Stolen Strategy doesn't change when you can cast the exiled card."
    );
    ruling!(
        "Stolen Strategy",
        "If you don't cast the exiled cards, they remain exiled. They can't be cast on later turns."
    );
    assert_supported("Stolen Strategy");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stolen Strategy");
    t.lands(P0, "Plains", 1);
    t.advance_to(P1, Step::End);
    let spike = t.library_top(P1, "Lava Spike");
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(spike)), Zone::Exile);
    // A sorcery: not in the upkeep.
    assert!(!try_cast(&mut t, P0, spike, &[Entity::Player(P1)]));
    t.advance_to(P0, Step::PrecombatMain);
    // Paid with a Plains (mana as though it were any color).
    assert!(try_cast(&mut t, P0, spike, &[Entity::Player(P1)]));
    assert_eq!(t.life(P1), 17);
    // Not cast this turn: not on a later turn either.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stolen Strategy");
    t.lands(P0, "Plains", 1);
    t.advance_to(P1, Step::End);
    let spike = t.library_top(P1, "Lava Spike");
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.zone(t.g.current(spike)), Zone::Exile);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(!try_cast(&mut t, P0, spike, &[Entity::Player(P1)]));
    assert_eq!(t.zone(t.g.current(spike)), Zone::Exile);
}

#[test]
fn karolina_dean_mana_cant_pay_for_spells_from_your_hand() {
    cr!("106.6");
    assert_supported("Karolina Dean, Runaway");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Karolina Dean, Runaway");
    let bolt = t.hand(P0, "Lightning Bolt");
    let think = t.graveyard(P0, "Think Twice");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    // {W}{U}{B}{R}{G} in the pool: not for the Bolt in the hand.
    assert!(!try_cast(&mut t, P0, bolt, &[Entity::Player(P1)]));
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Hand(P0));
    // Flashback {2}{U} from the graveyard: yes.
    assert!(try_cast_as(
        &mut t,
        P0,
        think,
        CastMethod::Keyword(keywords::KeywordKind::Flashback),
        &[]
    ));
}

#[test]
fn one_with_the_multiverse_casts_one_spell_free_each_turn_per_copy() {
    cr!("118.9", "601.3");
    ruling!(
        "One with the Multiverse",
        "each one's last ability applies independently, and you may cast a spell without paying its mana cost once for each of them during each of your turns"
    );
    assert_supported("One with the Multiverse");
    // From the hand or the top of the library: one use, each turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "One with the Multiverse");
    let bolt = t.hand(P0, "Lightning Bolt");
    let bolt2 = t.hand(P0, "Lightning Bolt");
    let top = t.library_top(P0, "Lightning Bolt");
    assert!(try_cast_as(&mut t, P0, bolt, CastMethod::Free, &[Entity::Player(P1)]));
    assert!(!try_cast_as(&mut t, P0, bolt2, CastMethod::Free, &[Entity::Player(P1)]));
    assert!(!try_cast_as(&mut t, P0, top, CastMethod::Free, &[Entity::Player(P1)]));
    assert_eq!(t.life(P1), 17);
    // Next turn: again.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(try_cast_as(&mut t, P0, bolt2, CastMethod::Free, &[Entity::Player(P1)]));
    assert_eq!(t.life(P1), 14);
    // Two of them: two free spells.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "One with the Multiverse");
    t.battlefield(P0, "One with the Multiverse");
    let bolts: Vec<ObjectId> = (0..3).map(|_| t.hand(P0, "Lightning Bolt")).collect();
    assert!(try_cast_as(&mut t, P0, bolts[0], CastMethod::Free, &[Entity::Player(P1)]));
    assert!(try_cast_as(&mut t, P0, bolts[1], CastMethod::Free, &[Entity::Player(P1)]));
    assert!(!try_cast_as(&mut t, P0, bolts[2], CastMethod::Free, &[Entity::Player(P1)]));
    assert_eq!(t.life(P1), 14);
}

#[test]
fn zaffai_casts_one_instant_or_sorcery_from_hand_free_each_of_your_turns() {
    cr!("118.9", "601.3");
    assert_supported("Zaffai and the Tempests");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zaffai and the Tempests");
    let bolt = t.hand(P0, "Lightning Bolt");
    let bolt2 = t.hand(P0, "Lightning Bolt");
    let goblin = t.hand(P0, "Raging Goblin");
    // Not a creature spell.
    assert!(!try_cast_as(&mut t, P0, goblin, CastMethod::Free, &[]));
    assert!(try_cast_as(&mut t, P0, bolt, CastMethod::Free, &[Entity::Player(P1)]));
    assert!(!try_cast_as(&mut t, P0, bolt2, CastMethod::Free, &[Entity::Player(P1)]));
    // Not during another player's turn.
    t.advance_to(P1, Step::Upkeep);
    assert!(!try_cast_as(&mut t, P0, bolt2, CastMethod::Free, &[Entity::Player(P1)]));
    t.advance_to(P0, Step::Upkeep);
    assert!(try_cast_as(&mut t, P0, bolt2, CastMethod::Free, &[Entity::Player(P1)]));
    assert_eq!(t.life(P1), 14);
}

#[test]
fn kethis_lets_legendary_cards_in_your_graveyard_be_played_this_turn() {
    cr!("611.2c", "601.3");
    ruling!(
        "Kethis, the Hidden Hand",
        "Kethis’s last ability affects only legendary cards that are in your graveyard at the time it resolves."
    );
    ruling!(
        "Kethis, the Hidden Hand",
        "Kethis’s last ability doesn’t change when you can play the legendary cards."
    );
    assert_supported("Kethis, the Hidden Hand");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let kethis = t.battlefield(P0, "Kethis, the Hidden Hand");
    let a = t.graveyard(P0, "Isamaru, Hound of Konda");
    let b = t.graveyard(P0, "Isamaru, Hound of Konda");
    let squee = t.graveyard(P0, "Squee, Goblin Nabob");
    let goblin = t.graveyard(P0, "Raging Goblin");
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.activate(P0, kethis, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(a)), Zone::Exile);
    // A legendary card put there later doesn't gain it.
    let later = t.graveyard(P0, "Isamaru, Hound of Konda");
    assert!(!try_cast(&mut t, P0, later, &[]));
    // Not a nonlegendary card.
    assert!(!try_cast(&mut t, P0, goblin, &[]));
    // Normal timing: not during combat.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!try_cast(&mut t, P0, squee, &[]));
    t.set_step(P0, Step::PostcombatMain);
    assert!(try_cast(&mut t, P0, squee, &[]));
    assert_eq!(t.named_on_battlefield("Squee, Goblin Nabob").len(), 1);
}
