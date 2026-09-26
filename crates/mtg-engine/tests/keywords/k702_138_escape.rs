//! CR 702.138 Escape.

use crate::common_k702_125_139::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const ESCAPE: CastMethod = CastMethod::Keyword(KeywordKind::Escape);

fn exile_count(t: &TestGame) -> usize {
    t.g.exile.len()
}

#[test]
fn escape_casts_the_card_from_the_graveyard_for_its_escape_cost() {
    cr!("702.138", "702.138a");
    ruling!(
        "Phoenix of Ash",
        "The mana value of the spell remains unchanged, no matter what the total cost to cast it was and no matter whether an alternative cost was paid."
    );
    ruling!(
        "Phoenix of Ash",
        "After an escaped spell resolves, it returns to its owner's graveyard if it's not a permanent spell."
    );
    assert_supported_card("Fruit of Tizerus");
    let mut t = TestGame::new(2);
    // Fruit of Tizerus: {B} sorcery, "Target player loses 2 life."; escape—{3}{B}, exile
    // three other cards from your graveyard.
    t.lands(P0, "Swamp", 4);
    let fruit = t.graveyard(P0, "Fruit of Tizerus");
    graveyard_n(&mut t, P0, "Grizzly Bears", 2);
    assert!(!castable(&mut t, P0, fruit, ESCAPE), "only two other cards");
    t.graveyard(P0, "Grizzly Bears");
    assert!(castable(&mut t, P0, fruit, ESCAPE));
    let spell = t
        .cast(P0, fruit)
        .method(ESCAPE)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(t.g.mana_value_of(spell), 1);
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(exile_count(&t), 3);
    assert_eq!(t.graveyard_size(P0), 0);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // An escaped instant or sorcery goes back to the graveyard.
    assert!(t.in_graveyard(P0, "Fruit of Tizerus"));
}

#[test]
fn escape_doesnt_change_when_the_card_can_be_cast() {
    cr!("702.138a");
    ruling!(
        "Phoenix of Ash",
        "Escape's permission doesn't change when you may cast the spell from your graveyard."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let fruit = t.graveyard(P0, "Fruit of Tizerus");
    graveyard_n(&mut t, P0, "Grizzly Bears", 3);
    t.set_step(P1, Step::PrecombatMain);
    assert!(!castable(&mut t, P0, fruit, ESCAPE));
    // Only from its owner's graveyard: not from the hand.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let fruit = t.hand(P0, "Fruit of Tizerus");
    graveyard_n(&mut t, P0, "Grizzly Bears", 3);
    assert!(!castable(&mut t, P0, fruit, ESCAPE));
}

#[test]
fn a_permanent_escapes_with_counters_only_if_it_escaped() {
    cr!("702.138b", "702.138c");
    assert_supported_card("Phoenix of Ash");
    // Phoenix of Ash: 2/2; escape—{2}{R}{R}, exile three other cards; "This creature
    // escapes with a +1/+1 counter on it."
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let phoenix = t.graveyard(P0, "Phoenix of Ash");
    graveyard_n(&mut t, P0, "Grizzly Bears", 3);
    t.cast(P0, phoenix).method(ESCAPE).go();
    t.resolve_all();
    assert!(t.on_battlefield(phoenix));
    assert_eq!(plus1(&t, phoenix), 1);
    assert_eq!(t.pt(phoenix), (3, 3));
    // Cast from the hand, it didn't escape.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let phoenix = t.hand(P0, "Phoenix of Ash");
    t.cast(P0, phoenix).go();
    t.resolve_all();
    assert_eq!(plus1(&t, phoenix), 0);
    // Put onto the battlefield from the graveyard without being cast: it didn't escape.
    let mut t = TestGame::new(2);
    let phoenix = t.graveyard(P0, "Phoenix of Ash");
    t.g.move_object(
        phoenix,
        Zone::Battlefield,
        mtg_engine::events::MoveCause::Effect,
        Some(P0),
    );
    t.g.flush_events();
    assert_eq!(plus1(&t, phoenix), 0);
}

#[test]
fn sacrifice_it_unless_it_escaped() {
    cr!("702.138b");
    assert_line_supported("Uro, Titan of Nature's Wrath", "unless it escaped");
    assert_line_supported("Kroxa, Titan of Death's Hunger", "unless it escaped");
    ruling!(
        "Kroxa, Titan of Death's Hunger",
        "Kroxa's first ability causes you to sacrifice it if you didn't cast it, or if it was cast using any permission other than an escape ability."
    );
    // Uro, Titan of Nature's Wrath: "When Uro enters, sacrifice it unless it escaped."
    // Cast from the hand: sacrificed.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 2);
    let uro = t.hand(P0, "Uro, Titan of Nature's Wrath");
    t.cast(P0, uro).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Uro, Titan of Nature's Wrath"));
    // Escaped: it stays.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Island", 2);
    let uro = t.graveyard(P0, "Uro, Titan of Nature's Wrath");
    graveyard_n(&mut t, P0, "Grizzly Bears", 5);
    t.cast(P0, uro).method(ESCAPE).go();
    t.resolve_all();
    assert!(t.on_battlefield(uro));
    // Cast from the graveyard with another permission, it didn't escape.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 2);
    let uro = t.graveyard(P0, "Uro, Titan of Nature's Wrath");
    mtg_engine::casting::grant_play_permission(
        &mut t.g,
        P0,
        vec![uro],
        mtg_engine::ability::Duration::EndOfTurn,
        false,
        None,
    );
    assert!(castable(&mut t, P0, uro, CastMethod::Normal));
    t.cast(P0, uro).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Uro, Titan of Nature's Wrath"));
}

#[test]
fn enters_with_counters_but_escapes_with_more_instead() {
    cr!("702.138c");
    assert_line_supported("Polukranos, Unchained", "escapes with");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Forest", 2);
    let p = t.hand(P0, "Polukranos, Unchained");
    t.cast(P0, p).go();
    t.resolve_all();
    assert_eq!(plus1(&t, p), 6);
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    t.lands(P0, "Forest", 3);
    let p = t.graveyard(P0, "Polukranos, Unchained");
    graveyard_n(&mut t, P0, "Grizzly Bears", 6);
    t.cast(P0, p).method(ESCAPE).go();
    t.resolve_all();
    assert_eq!(plus1(&t, p), 12);
}

#[test]
fn when_it_enters_this_way_triggers_only_if_it_escaped() {
    cr!("702.138c");
    assert_supported_card("Pharika's Spawn");
    // Pharika's Spawn: "This creature escapes with two +1/+1 counters on it. When it
    // enters this way, each opponent sacrifices a non-Gorgon creature of their choice."
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 6);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spawn = t.graveyard(P0, "Pharika's Spawn");
    graveyard_n(&mut t, P0, "Grizzly Bears", 3);
    t.cast(P0, spawn).method(ESCAPE).go();
    t.resolve_all();
    assert_eq!(plus1(&t, spawn), 2);
    assert!(!t.on_battlefield(bears));
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spawn = t.hand(P0, "Pharika's Spawn");
    t.cast(P0, spawn).go();
    t.resolve_all();
    assert_eq!(plus1(&t, spawn), 0);
    assert!(t.on_battlefield(bears));
}

#[test]
fn escapes_with_your_choice_of_counters_chosen_as_it_enters() {
    cr!("702.138c");
    ruling!(
        "Tizerus Charger",
        "You make the choice between a +1/+1 counter and a flying counter as Tizerus Charger enters the battlefield, not as you cast it with escape."
    );
    assert_supported_card("Tizerus Charger");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    let c = t.graveyard(P0, "Tizerus Charger");
    graveyard_n(&mut t, P0, "Grizzly Bears", 5);
    t.cast(P0, c).method(ESCAPE).go();
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(1),
    );
    t.resolve_all();
    assert_eq!(t.counters(c, "flying"), 1);
    assert!(has(&t, c, KeywordKind::Flying));
}

#[test]
fn escapes_with_an_ability() {
    cr!("702.138d");
    let mut def = custom_card(
        "Hasty Escapee",
        "Creature — Elemental",
        Some((2, 2)),
        "Escape—{1}{R}, Exile two other cards from your graveyard.\nThis creature escapes with haste.",
    );
    def.faces[0].chars.mana_cost = mtg_engine::mana::ManaCost::parse("{R}");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let c = t.custom(P0, def.clone(), Zone::Graveyard(P0));
    graveyard_n(&mut t, P0, "Grizzly Bears", 2);
    t.cast(P0, c).method(ESCAPE).go();
    t.resolve_all();
    assert!(t.on_battlefield(c));
    assert!(has(&t, c, KeywordKind::Haste));
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let c = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, c).go();
    t.resolve_all();
    assert!(!has(&t, c, KeywordKind::Haste));
}

#[test]
fn a_granted_escape_costs_the_cards_mana_cost_plus_more() {
    cr!("702.138a");
    ruling!(
        "Underworld Breach",
        "If a card has no mana cost, its escape cost is an unpayable cost, so you can't cast it for that cost."
    );
    assert_supported_card("Underworld Breach");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Underworld Breach");
    t.lands(P0, "Mountain", 1);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    graveyard_n(&mut t, P0, "Grizzly Bears", 2);
    assert!(!castable(&mut t, P0, bolt, ESCAPE));
    t.graveyard(P0, "Grizzly Bears");
    assert!(castable(&mut t, P0, bolt, ESCAPE));
    t.cast(P0, bolt)
        .method(ESCAPE)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(exile_count(&t), 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    // A land card has no mana cost (and isn't a nonland card anyway); a card with no mana
    // cost can't be escaped.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Underworld Breach");
    t.lands(P0, "Mountain", 3);
    let ancestral = t.graveyard(P0, "Ancestral Vision");
    graveyard_n(&mut t, P0, "Grizzly Bears", 3);
    assert!(!castable(&mut t, P0, ancestral, ESCAPE));
}

#[test]
fn a_card_can_gain_escape_until_end_of_turn() {
    cr!("702.138a", "702.138b");
    assert_supported_card("Confession Dial");
    let mut t = TestGame::new(2);
    // Confession Dial: "{T}: Target legendary creature card in your graveyard gains escape
    // until end of turn. The escape cost is equal to its mana cost plus exile three other
    // cards from your graveyard."
    let dial = t.battlefield(P0, "Confession Dial");
    let isamaru = t.graveyard(P0, "Isamaru, Hound of Konda");
    graveyard_n(&mut t, P0, "Grizzly Bears", 3);
    t.lands(P0, "Plains", 1);
    assert!(!castable(&mut t, P0, isamaru, ESCAPE));
    t.activate(P0, dial, 0, &[Entity::Object(isamaru)]).unwrap();
    t.resolve_all();
    assert!(castable(&mut t, P0, isamaru, ESCAPE));
    t.cast(P0, isamaru).method(ESCAPE).go();
    assert_eq!(exile_count(&t), 3);
    t.resolve_all();
    assert!(t.on_battlefield(isamaru));
    // It escaped (the ability was granted to the card; the spell keeps it, CR 400.7g).
    let perm = t.g.current(isamaru);
    assert!(t
        .g
        .obj(perm)
        .cast
        .as_deref()
        .is_some_and(|c| c.paid.iter().any(|p| p == "escape")));
}
