//! CR 702.170 Plot.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_140_152::*;
use crate::common_k702_168_177::*;
use mtg_engine::decision::SpecialAction;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const PLOTTED: CastMethod = CastMethod::Keyword(KeywordKind::Plot);

/// The card named `name` in exile.
fn exiled(t: &TestGame, name: &str) -> ObjectId {
    t.g.find_in_zone(Zone::Exile, name)
        .first()
        .copied()
        .unwrap_or_else(|| panic!("{name} isn't in exile"))
}

/// Goes to P0's next precombat main phase (a later turn).
fn next_turn_main(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
}

#[test]
fn plotting_is_a_special_action_that_exiles_the_card_from_hand() {
    cr!("702.170", "702.170a", "702.170b");
    assert_supported("Djinn of Fool's Fall");
    ruling!(
        "Slickshot Show-Off",
        "Exiling a card using its plot ability is a special action. Once you announce you’re taking that action, no other player can respond by trying to remove that card from your hand."
    );
    // Djinn of Fool's Fall: {4}{U} 4/3 flying, plot {3}{U}.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let djinn = t.hand(P0, "Djinn of Fool's Fall");
    add_mana(&mut t, P0, ManaType::U, 3);
    // It can't be paid yet.
    assert!(!special_actions(&mut t, P0).contains(&SpecialAction::Plot { card: djinn }));
    add_mana(&mut t, P0, ManaType::U, 1);
    assert!(special_actions(&mut t, P0).contains(&SpecialAction::Plot { card: djinn }));
    take_special(&mut t, P0, SpecialAction::Plot { card: djinn }).unwrap();
    // No stack involved: it's in exile at once, plotted, and the cost was paid.
    assert_eq!(t.stack_len(), 0);
    assert_eq!(pool(&t, P0), 0);
    let plotted = exiled(&t, "Djinn of Fool's Fall");
    assert!(mtg_engine::kw::plot::plotted_turn(&t.g, plotted).is_some());
    // Only from its owner's hand: not from a graveyard.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let djinn = t.graveyard(P0, "Djinn of Fool's Fall");
    add_mana(&mut t, P0, ManaType::U, 4);
    assert!(!special_actions(&mut t, P0).contains(&SpecialAction::Plot { card: djinn }));
    assert!(take_special(&mut t, P0, SpecialAction::Plot { card: djinn }).is_err());
}

#[test]
fn an_effect_can_make_a_card_in_exile_plotted() {
    cr!("702.170c", "702.170d");
    assert_supported("Kellan Joins Up");
    ruling!(
        "Kellan Joins Up",
        "You can't cast a plotted card on the same turn it became plotted. On any future turn, you may cast that card from exile without paying its mana cost during your main phase while the stack is empty."
    );
    // Kellan Joins Up: "When Kellan Joins Up enters, you may exile a nonland card with mana
    // value 3 or less from your hand. If you do, it becomes plotted."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let kellan = t.hand(P0, "Kellan Joins Up");
    let bears = t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Hill Giant");
    add_mana(&mut t, P0, ManaType::G, 1);
    add_mana(&mut t, P0, ManaType::W, 1);
    add_mana(&mut t, P0, ManaType::U, 1);
    t.cast(P0, kellan).go();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    // The Bears (mana value 2) is plotted even though it has no plot ability.
    let plotted = exiled(&t, "Grizzly Bears");
    assert!(mtg_engine::kw::plot::plotted_turn(&t.g, plotted).is_some());
    assert!(t.in_hand(P0, "Hill Giant"));
    // Not this turn.
    assert!(!cast_methods_now(&mut t, P0, plotted).contains(&PLOTTED));
    // A later turn, in the main phase with an empty stack: cast without paying its mana
    // cost.
    next_turn_main(&mut t);
    assert!(cast_methods_now(&mut t, P0, plotted).contains(&PLOTTED));
    let before = pool(&t, P0);
    t.cast(P0, plotted).method(PLOTTED).go();
    assert_eq!(pool(&t, P0), before);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn a_plotted_card_is_cast_only_by_its_owner_as_a_sorcery_on_a_later_turn() {
    cr!("702.170d");
    ruling!(
        "Kellan Joins Up",
        "If an instant or a card with flash is plotted this way, you can still cast it only when you have priority during your main phase while the stack is empty."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let helix = t.exile(P0, "Lightning Helix");
    mtg_engine::kw::plot::make_plotted(&mut t.g, helix);
    next_turn_main(&mut t);
    // An instant, plotted: not during combat, not with a spell on the stack.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!cast_methods_now(&mut t, P0, helix).contains(&PLOTTED));
    t.set_step(P0, Step::PostcombatMain);
    let bolt = t.hand(P1, "Lightning Bolt");
    add_mana(&mut t, P1, ManaType::R, 1);
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    assert!(!cast_methods_now(&mut t, P0, helix).contains(&PLOTTED));
    t.resolve_all();
    // Not in the opponent's turn, and not by another player.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!cast_methods_now(&mut t, P0, helix).contains(&PLOTTED));
    assert!(!cast_methods_now(&mut t, P1, helix).contains(&PLOTTED));
    t.set_step(P0, Step::PostcombatMain);
    assert!(cast_methods_now(&mut t, P0, helix).contains(&PLOTTED));
    t.cast(P0, helix)
        .method(PLOTTED)
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn effects_that_refer_to_plotting_mean_the_special_action() {
    cr!("702.170e");
    ruling!(
        "Doc Aurlock, Grizzled Genius",
        "Doc Aurlock’s abilities don’t change the mana cost or mana value of any spell. They change only the total cost you pay to cast spells or plot cards from the appropriate zones."
    );
    // Doc Aurlock: "Plotting cards from your hand costs {2} less."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Doc Aurlock, Grizzled Genius");
    let djinn = t.hand(P0, "Djinn of Fool's Fall");
    // Plot {3}{U} costs {1}{U}.
    add_mana(&mut t, P0, ManaType::U, 2);
    take_special(&mut t, P0, SpecialAction::Plot { card: djinn }).unwrap();
    assert_eq!(pool(&t, P0), 0);
    exiled(&t, "Djinn of Fool's Fall");
    // Longhorn Sharpshooter: "When this card becomes plotted, it deals 2 damage to any
    // target." — whether plotted by its plot ability or by an effect.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let sharp = t.hand(P0, "Longhorn Sharpshooter");
    add_mana(&mut t, P0, ManaType::R, 4);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    take_special(&mut t, P0, SpecialAction::Plot { card: sharp }).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let kellan = t.hand(P0, "Kellan Joins Up");
    let sharp = t.hand(P0, "Longhorn Sharpshooter");
    add_mana(&mut t, P0, ManaType::G, 1);
    add_mana(&mut t, P0, ManaType::W, 1);
    add_mana(&mut t, P0, ManaType::U, 1);
    t.cast(P0, kellan).go();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(sharp)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn an_effect_can_let_plot_function_from_the_top_of_the_library() {
    cr!("702.170f");
    assert_supported("Fblthp, Lost on the Range");
    ruling!(
        "Fblthp, Lost on the Range",
        "If you plot the top card of your library, you must exile that card and pay its plot cost before you may look at the new top card of your library."
    );
    // Fblthp: "The top card of your library has plot. The plot cost is equal to its mana
    // cost." "You may plot nonland cards from the top of your library."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.library_top(P0, "Island");
    let giant = t.library_top(P0, "Hill Giant");
    // Without Fblthp, a card on top of the library can't be plotted.
    add_mana(&mut t, P0, ManaType::R, 4);
    assert!(!special_actions(&mut t, P0).contains(&SpecialAction::Plot { card: giant }));
    t.battlefield(P0, "Fblthp, Lost on the Range");
    assert!(special_actions(&mut t, P0).contains(&SpecialAction::Plot { card: giant }));
    // Its plot cost is its mana cost, {3}{R}; it's exiled from the library.
    take_special(&mut t, P0, SpecialAction::Plot { card: giant }).unwrap();
    assert_eq!(pool(&t, P0), 0);
    let plotted = exiled(&t, "Hill Giant");
    assert!(mtg_engine::kw::plot::plotted_turn(&t.g, plotted).is_some());
    // The new top card is a land: it can't be plotted.
    let island = *t.g.player(P0).library.last().unwrap();
    add_mana(&mut t, P0, ManaType::C, 4);
    assert!(!special_actions(&mut t, P0).contains(&SpecialAction::Plot { card: island }));
    next_turn_main(&mut t);
    t.cast(P0, plotted).method(PLOTTED).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}
