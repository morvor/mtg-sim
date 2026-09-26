//! CR 702.119 Emerge.

use crate::common_k702_111_124::*;
use mtg_engine::ability::Filter;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const EMERGE: CastMethod = CastMethod::Keyword(KeywordKind::Emerge);

#[test]
fn emerge_sacrifices_a_creature_and_reduces_the_cost_by_its_mana_value() {
    cr!("702.119", "702.119a");
    ruling!(
        "Wretched Gryff",
        "The mana value of a creature spell with emerge isn’t affected by whether its emerge cost is paid."
    );
    assert_supported_card("Wretched Gryff");
    let mut t = TestGame::new(2);
    // Wretched Gryff: {7}, emerge {5}{U}; "When you cast this spell, draw a card."
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Island", 2);
    let gryff = t.hand(P0, "Wretched Gryff");
    assert!(castable(&mut t, P0, gryff, EMERGE));
    let hand = t.hand_size(P0);
    let spell = t.cast(P0, gryff).method(EMERGE).go();
    // {5}{U} minus Hill Giant's mana value (4): {1}{U}.
    assert_eq!(untapped_lands(&t, P0), 0);
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.g.mana_value_of(spell), 7);
    t.resolve_all();
    assert!(t.on_battlefield(gryff));
    assert_eq!(t.hand_size(P0), hand, "drew a card for casting it");
}

#[test]
fn emerge_needs_a_creature_to_sacrifice() {
    cr!("702.119a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    let gryff = t.hand(P0, "Wretched Gryff");
    assert!(!castable(&mut t, P0, gryff, EMERGE));
    // A noncreature permanent won't do.
    t.battlefield(P0, "Sol Ring");
    assert!(!castable(&mut t, P0, gryff, EMERGE));
    t.battlefield(P0, "Grizzly Bears");
    assert!(castable(&mut t, P0, gryff, EMERGE));
}

#[test]
fn only_generic_mana_is_reduced() {
    cr!("702.119a");
    ruling!(
        "Wretched Gryff",
        "You may sacrifice a creature with mana value greater than or equal to the emerge cost. If you do, you’ll pay only the colored mana component of the emerge cost."
    );
    ruling!("Wretched Gryff", "Colored mana components of emerge costs can’t be reduced with emerge.");
    let mut t = TestGame::new(2);
    // Craw Wurm: mana value 6.
    t.battlefield(P0, "Craw Wurm");
    t.lands(P0, "Island", 1);
    let gryff = t.hand(P0, "Wretched Gryff");
    t.cast(P0, gryff).method(EMERGE).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert!(t.on_battlefield(gryff));
    // Without the {U} it can't be cast.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Craw Wurm");
    t.lands(P0, "Forest", 3);
    let gryff = t.hand(P0, "Wretched Gryff");
    assert!(!castable(&mut t, P0, gryff, EMERGE));
}

#[test]
fn a_creature_with_mana_value_zero_reduces_nothing() {
    cr!("702.119a");
    ruling!(
        "Wretched Gryff",
        "You may sacrifice a creature with a mana value of 0, such as a token creature that’s not a copy of another permanent, to cast a spell for its emerge cost. You’ll just pay the full emerge cost with no reduction."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ornithopter");
    t.lands(P0, "Island", 5);
    let gryff = t.hand(P0, "Wretched Gryff");
    assert!(!castable(&mut t, P0, gryff, EMERGE));
    t.lands(P0, "Island", 1);
    t.cast(P0, gryff).method(EMERGE).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert!(t.in_graveyard(P0, "Ornithopter"));
}

#[test]
fn the_player_chooses_which_creature_to_sacrifice() {
    cr!("702.119a", "702.119c");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 4);
    let gryff = t.hand(P0, "Wretched Gryff");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, gryff).method(EMERGE).go();
    // Grizzly Bears (mana value 2) was sacrificed: {3}{U} was paid.
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(!t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(untapped_lands(&t, P0), 0);
    let asked = asked_of(&t, P0, |d| {
        matches!(d, Decision::ChooseEntities { candidates, .. } if candidates.len() == 2)
    });
    assert_eq!(asked, 1);
}

#[test]
fn the_chosen_creature_can_be_tapped_for_mana_before_it_is_sacrificed() {
    cr!("702.119c");
    ruling!(
        "Wretched Gryff",
        "The creature chosen to be sacrificed is still on the battlefield as the cost of the emerge spell is determined and as you activate mana abilities to cast the emerge spell."
    );
    let mut t = TestGame::new(2);
    // Llanowar Elves (mana value 1) is chosen and taps for {G} toward the {4}{U} to pay.
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Island", 4);
    let gryff = t.hand(P0, "Wretched Gryff");
    t.answer_choose(P0, &[Entity::Object(elves)]);
    t.cast(P0, gryff).method(EMERGE).go();
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert!(t.on_battlefield(gryff));
}

#[test]
fn emerge_from_a_quality_sacrifices_a_permanent_with_that_quality() {
    cr!("702.119b");
    ruling!(
        "Crabomination",
        "Emerge from artifact is a variant of the emerge ability. It allows you to sacrifice an artifact rather than a creature, but otherwise functions identically to emerge."
    );
    let kws: Vec<_> = mtg_engine::card::card("Crabomination").faces[0]
        .chars
        .keywords()
        .filter(|k| k.kind == KeywordKind::Emerge)
        .cloned()
        .collect();
    assert_eq!(kws.len(), 1);
    assert!(matches!(kws[0].filter, Some(Filter::Type(CardType::Artifact))));
    let mut t = TestGame::new(2);
    // Crabomination: emerge from artifact {5}{B}{B}.
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 4);
    let crab = t.hand(P0, "Crabomination");
    assert!(!castable(&mut t, P0, crab, EMERGE), "a creature isn't an artifact");
    // Sol Ring: a noncreature artifact with mana value 1; Darksteel Myr: mana value 3.
    t.battlefield(P0, "Sol Ring");
    let myr = t.battlefield(P0, "Darksteel Myr");
    assert!(castable(&mut t, P0, crab, EMERGE));
    t.answer_choose(P0, &[Entity::Object(myr)]);
    t.cast(P0, crab).method(EMERGE).go();
    assert!(t.in_graveyard(P0, "Darksteel Myr"));
    assert!(t.on_battlefield(t.named_on_battlefield("Grizzly Bears")[0]));
}

#[test]
fn abilities_can_trigger_on_sacrificing_while_casting_a_spell_with_emerge() {
    // "When you sacrifice ~" looks back in time (CR 603.10a): the sacrificed permanent's
    // ability triggers.
    cr!("702.119a", "702.119c", "603.10a");
    ruling!(
        "Foul Emissary",
        "Foul Emissary’s last ability triggers if it’s sacrificed for any reason while you’re casting a spell with emerge, whether or not you’re casting that spell for its emerge cost."
    );
    assert_supported_card("Foul Emissary");
    // Sacrificed for the emerge cost: a 3/2 Eldrazi Horror.
    let mut t = TestGame::new(2);
    let emissary = t.battlefield(P0, "Foul Emissary");
    t.lands(P0, "Island", 3);
    let gryff = t.hand(P0, "Wretched Gryff");
    t.answer_choose(P0, &[Entity::Object(emissary)]);
    t.cast(P0, gryff).method(EMERGE).go();
    t.resolve_all();
    let horrors = tokens_of(&t, P0);
    assert_eq!(horrors.len(), 1);
    assert_eq!(t.pt(horrors[0]), (3, 2));
    // Sacrificed while not casting a spell with emerge: nothing.
    let mut t = TestGame::new(2);
    let emissary = t.battlefield(P0, "Foul Emissary");
    t.g.sacrifice(emissary, P0);
    t.resolve_all();
    assert!(tokens_of(&t, P0).is_empty());
}

#[test]
fn the_sacrificed_creature_is_gone_before_cast_triggers() {
    cr!("702.119c");
    ruling!(
        "Wretched Gryff",
        "However, if it has an ability that triggers when a spell is cast, it will have been sacrificed before that ability can trigger."
    );
    let mut t = TestGame::new(2);
    // Beast Whisperer (mana value 4): "Whenever you cast a creature spell, draw a card."
    t.battlefield(P0, "Beast Whisperer");
    t.lands(P0, "Island", 2);
    let gryff = t.hand(P0, "Wretched Gryff");
    let hand = t.hand_size(P0);
    t.cast(P0, gryff).method(EMERGE).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Beast Whisperer"));
    // Only Wretched Gryff's own cast trigger drew a card.
    assert_eq!(t.hand_size(P0), hand);
}
