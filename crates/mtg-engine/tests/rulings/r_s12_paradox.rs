//! Rulings batch S12 — paradox (an ability word): abilities that care about spells cast
//! "from anywhere other than your hand".

use crate::r_s01_common::*;
use crate::r_s02_common::can_cast;
use crate::r_s03_common::in_hand_with_mana;
use mtg_engine::decision::Answer;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

/// Iraxxa's 2/2 Alien Warrior tokens `p` controls.
fn aliens(t: &TestGame, p: PlayerId) -> usize {
    with_subtype(t, p, "Alien")
        .into_iter()
        .filter(|id| t.obj(*id).is_token())
        .count()
}

/// Puts Iraxxa, Empress of Mars ("Paradox — Whenever you cast a spell from anywhere other
/// than your hand, create a 2/2 red Alien Warrior creature token.") onto `p`'s battlefield.
fn iraxxa(t: &mut TestGame, p: PlayerId) -> ObjectId {
    supported("Iraxxa, Empress of Mars");
    t.battlefield(p, "Iraxxa, Empress of Mars")
}

#[test]
fn a_paradox_trigger_doesnt_see_its_own_spell_cast() {
    cr!("113.6", "603.2");
    ruling!(
        "Iraxxa, Empress of Mars",
        "A triggered ability that triggers when a spell is cast from anywhere other than your hand, such as that of The Thirteenth Doctor, functions only on the battlefield, so it doesn't trigger when you cast that spell from a zone other than your hand."
    );
    supported("Iraxxa, Empress of Mars");
    supported("Future Sight");
    // Future Sight: "You may play lands and cast spells from the top of your library."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Future Sight");
    give_mana_for(&mut t, P0, "Iraxxa, Empress of Mars");
    let card = t.library_top(P0, "Iraxxa, Empress of Mars");
    assert!(can_cast(&mut t, P0, card, CastMethod::Normal));
    t.cast(P0, card).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.on_battlefield(card));
    assert_eq!(aliens(&t, P0), 0);
    // Once it's on the battlefield, a spell cast from the library triggers it.
    give_mana_for(&mut t, P0, "Grizzly Bears");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(aliens(&t, P0), 1);
}

#[test]
fn a_spell_counting_spells_cast_from_elsewhere_counts_itself() {
    cr!("601.2i", "608.2h");
    ruling!(
        "Surge of Brilliance",
        "An instant or sorcery that counts how many spells you've cast from anywhere other than your hand, such as Surge of Brilliance, counts itself if it was cast from a zone other than your hand."
    );
    supported("Surge of Brilliance");
    // Surge of Brilliance: "Draw a card for each spell you've cast this turn from anywhere
    // other than your hand." Cast from the top of the library: it counts itself.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Future Sight");
    give_mana_for(&mut t, P0, "Surge of Brilliance");
    let surge = t.library_top(P0, "Surge of Brilliance");
    let hand = t.hand_size(P0);
    t.cast(P0, surge).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Cast from the hand: it doesn't count itself; a spell cast from the library earlier
    // this turn counts.
    let mut t = TestGame::new(2);
    let surge = in_hand_with_mana(&mut t, P0, "Surge of Brilliance");
    let hand = t.hand_size(P0);
    t.cast(P0, surge).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1);
    t.battlefield(P0, "Future Sight");
    give_mana_for(&mut t, P0, "Grizzly Bears");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    let surge = in_hand_with_mana(&mut t, P0, "Surge of Brilliance");
    let hand = t.hand_size(P0);
    t.cast(P0, surge).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 1);
    assert!(t.in_graveyard(P0, "Surge of Brilliance"));
}

#[test]
fn copying_a_spell_isnt_casting_it_but_casting_a_copy_is() {
    cr!("707.10", "707.12", "601.2i");
    ruling!(
        "Iraxxa, Empress of Mars",
        "If a spell or ability allows you to copy a spell on the stack but doesn't specify that the spell is cast, that spell wasn't cast and won't be counted by paradox abilities. However, if a spell or ability allows you to cast a copy of a spell, that spell will be counted for paradox abilities."
    );
    supported("Twincast");
    supported("Isochron Scepter");
    let mut t = TestGame::new(2);
    iraxxa(&mut t, P0);
    // Lightning Bolt, then Twincast ("Copy target instant or sorcery spell."), both from
    // the hand: the copy of Bolt wasn't cast.
    let bolt = in_hand_with_mana(&mut t, P0, "Lightning Bolt");
    let bolt = t.cast(P0, bolt).target(P1).go();
    let twin = in_hand_with_mana(&mut t, P0, "Twincast");
    t.cast(P0, twin).target(bolt).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert_eq!(aliens(&t, P0), 0);
    // Isochron Scepter: "You may copy the exiled card. If you do, you may cast the copy
    // without paying its mana cost." The copy is cast (not from a hand).
    let shock = t.hand(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(shock)]);
    let scepter = t.enter(P0, "Isochron Scepter");
    t.resolve_all();
    assert_eq!(t.zone(shock), Zone::Exile);
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, scepter, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 11);
    assert_eq!(aliens(&t, P0), 1);
}

#[test]
fn spells_cast_from_graveyard_command_zone_or_outside_the_game_count() {
    cr!("601.2i", "903.8", "400.11");
    ruling!(
        "Iraxxa, Empress of Mars",
        "Paradox abilities count any spells cast from zones other than your hand. These are usually spells cast from exile, the graveyard, or the command zone. They also count spells cast from outside the game, such as spells cast with Wish or Garth One-Eye's ability."
    );
    supported("Think Twice");
    supported("Garth One-Eye");
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Commander,
            ..Default::default()
        },
    );
    iraxxa(&mut t, P0);
    // From the graveyard: Think Twice's flashback.
    let tt = t.graveyard(P0, "Think Twice");
    t.lands(P0, "Island", 3);
    t.cast(P0, tt)
        .method(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Flashback))
        .go();
    t.resolve_all();
    assert_eq!(aliens(&t, P0), 1);
    // From the command zone: a commander.
    let cmd = t.command(P0, "Rograkh, Son of Rohgahh");
    t.g.objects[cmd.0 as usize].is_commander = true;
    t.g.players[0]
        .commander_names
        .push("Rograkh, Son of Rohgahh".into());
    t.cast(P0, cmd).go();
    t.resolve_all();
    assert!(t.on_battlefield(cmd));
    assert_eq!(aliens(&t, P0), 2);
    // From outside the game: Garth One-Eye's copy of Black Lotus.
    let garth = t.battlefield(P0, "Garth One-Eye");
    t.answer(P0, DecisionKind::Option, Answer::Index(5));
    t.answer_yes(P0, true);
    t.activate(P0, garth, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Black Lotus").len(), 1);
    assert_eq!(aliens(&t, P0), 3);
    // From the hand: no token.
    let bears = in_hand_with_mana(&mut t, P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(aliens(&t, P0), 3);
}

#[test]
fn aetherflux_reservoir_counts_the_spells_cast_this_turn_as_it_resolves() {
    cr!("601.2i", "608.2h", "603.2");
    ruling!(
        "Aetherflux Reservoir",
        "The first ability counts the spell that caused it to trigger plus any other spells you cast earlier in the turn."
    );
    ruling!(
        "Aetherflux Reservoir",
        "The first ability counts spells you cast earlier in the turn even if you didn't control Aetherflux Reservoir as you cast them, and even if those spells were countered."
    );
    ruling!(
        "Aetherflux Reservoir",
        "Aetherflux Reservoir's first ability doesn't trigger when you cast Aetherflux Reservoir itself."
    );
    ruling!(
        "Aetherflux Reservoir",
        "The number of spells you've cast is counted only as Aetherflux Reservoir's triggered ability resolves."
    );
    supported("Aetherflux Reservoir");
    // "Whenever you cast a spell, you gain 1 life for each spell you've cast this turn."
    let mut t = TestGame::new(2);
    // Before the Reservoir: a Lightning Bolt, countered.
    let bolt = in_hand_with_mana(&mut t, P0, "Lightning Bolt");
    let bolt = t.cast(P0, bolt).target(P1).go();
    let cs = in_hand_with_mana(&mut t, P1, "Counterspell");
    t.g.turn.priority = Some(P1);
    t.cast(P1, cs).target(bolt).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // The Reservoir itself: no trigger.
    let res = in_hand_with_mana(&mut t, P0, "Aetherflux Reservoir");
    t.cast(P0, res).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // Grizzly Bears, and in response to its trigger a Lightning Bolt: both triggers count
    // all four spells as they resolve.
    let bears = in_hand_with_mana(&mut t, P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.settle();
    let bolt = in_hand_with_mana(&mut t, P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 28);
    assert_eq!(t.life(P1), 17);
}
