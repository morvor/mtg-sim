//! Rulings batch P220 — open an Attraction (Command Performance), outlast (Envoy of the
//! Ancestors), partner (Akroma, Vision of Ixidor), phasing (Ertai's Familiar), plot (Aven
//! Interrupter, Fblthp, Lost on the Range, Make Your Own Luck, Lilah, Undefeated
//! Slickshot).

use crate::r_s01_common::*;
use crate::r_s02_common::can_activate;
use crate::r_s11_common::empty_library;
use crate::r_s29_common::choose_modes;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::kw::plot::plotted_turn;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
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

/// Whether plotting `card` is among `p`'s legal special actions now.
fn can_plot(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .contains(&Action::Special(SpecialAction::Plot { card }))
}

#[test]
fn command_performance_opens_an_attraction_before_rolling_to_visit() {
    cr!("700.2", "701.51b", "701.52a", "608.2c");
    ruling!(
        "Command Performance",
        "No matter which two modes you choose, you always follow the instructions in the order they're written. You'll open an Attraction before rolling to visit them, if applicable"
    );
    supported("Command Performance");
    supported("Information Booth");
    // P0's Attraction deck: Information Booth ("Visit — Draw a card.", lit up on 2 and
    // 6). Modes chosen: "Roll to visit your Attractions." and "Open an Attraction."; the
    // die shows 6. The Booth is opened first, then visited.
    let mut t = TestGame::new(2);
    let booth = t.command(P0, "Information Booth");
    t.g.objects[booth.0 as usize].face_down = true;
    t.g.recompute();
    t.lands(P0, "Island", 2);
    let cp = t.hand(P0, "Command Performance");
    t.g.dice.loaded.push_back(6);
    choose_modes(&mut t, P0, &[1, 0]);
    let hand = t.hand_size(P0);
    t.cast(P0, cp).go();
    t.resolve_all();
    let booth = t.g.current(booth);
    assert!(t.on_battlefield(booth));
    // The Command Performance left the hand; the Booth's visit drew a card.
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn envoy_of_the_ancestors_outlast_needs_control_since_the_turn_began() {
    cr!("702.107a", "302.6");
    ruling!(
        "Envoy of the Ancestors",
        "The cost to activate a creature's outlast ability includes the tap symbol ({T}). Envoy of the Ancestors's outlast ability can't be activated unless it has been under your control continuously since the beginning of your turn."
    );
    supported("Envoy of the Ancestors");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let sick = t.battlefield_sick(P0, "Envoy of the Ancestors");
    assert!(!can_activate(&mut t, P0, sick));
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let envoy = t.battlefield(P0, "Envoy of the Ancestors");
    assert!(can_activate(&mut t, P0, envoy));
    t.activate(P0, envoy, 0, &[]).expect("outlast");
    assert!(t.obj(envoy).tapped);
    t.resolve_all();
    assert_eq!(t.counters(envoy, counters::PLUS1), 1);
}

#[test]
fn akroma_counts_partner_and_partner_with_but_not_other_partner_variants() {
    cr!("702.124a", "702.124c", "702.124k");
    ruling!(
        "Akroma, Vision of Ixidor",
        "Akroma's ability refers only to creatures with partner or \"partner with\" abilities, not creatures with any other partner variant (such as choose a Background or friends forever)."
    );
    supported("Akroma, Vision of Ixidor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Akroma, Vision of Ixidor");
    // Thrasios (1/3, partner), Pir (1/1, partner with Toothy), Abdel Adrian (4/4, choose
    // a Background).
    let thrasios = t.battlefield(P0, "Thrasios, Triton Hero");
    let pir = t.battlefield(P0, "Pir, Imaginative Rascal");
    let abdel = t.battlefield(P0, "Abdel Adrian, Gorion's Ward");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(thrasios), (2, 4));
    assert_eq!(t.pt(pir), (2, 2));
    assert_eq!(t.pt(abdel), (4, 4));
}

#[test]
fn ertais_familiar_cant_phase_out_for_any_reason_after_its_ability() {
    cr!("702.26a", "702.26b");
    ruling!(
        "Ertai's Familiar",
        "Activating the activated ability will prevent Ertai’s Familiar from phasing out for any reason, not just due to its own phasing ability."
    );
    supported("Ertai's Familiar");
    supported("Slip Out the Back");
    // Slip Out the Back: "Put a +1/+1 counter on target creature. It phases out."
    for activated in [false, true] {
        let mut t = TestGame::new(2);
        let fam = t.battlefield(P0, "Ertai's Familiar");
        t.lands(P0, "Island", 2);
        if activated {
            t.activate(P0, fam, 0, &[]).expect("activate");
            t.resolve_all();
        }
        let slip = t.hand(P0, "Slip Out the Back");
        t.cast(P0, slip).target(fam).go();
        t.resolve_all();
        assert_eq!(t.obj(fam).phased_out, !activated, "activated: {activated}");
        assert_eq!(t.counters(fam, counters::PLUS1), 1);
    }
}

#[test]
fn aven_interrupters_plotted_spell_is_cast_anew() {
    cr!("702.170d", "400.7", "601.2c");
    ruling!(
        "Aven Interrupter",
        "If the plotted card's owner casts it, the spell has no relation to the spell that player originally cast. Any choices made for the original spell or effects affecting the original spell aren't carried over to the new one."
    );
    supported("Aven Interrupter");
    supported("Burst Lightning");
    // P1 casts Burst Lightning kicked ({R} + kicker {4}: 4 damage) at P0; P0 flashes in
    // Aven Interrupter and exiles it plotted.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Mountain", 5);
    let burst = t.hand(P1, "Burst Lightning");
    let spell = t
        .cast(P1, burst)
        .kicked(true)
        .target(Entity::Player(P0))
        .go();
    t.lands(P0, "Plains", 3);
    let aven = t.hand(P0, "Aven Interrupter");
    t.answer_targets(P0, &[Entity::Object(spell)]);
    t.g.turn.priority = Some(P0);
    t.cast(P0, aven).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    let card = exiled(&t, "Burst Lightning");
    assert!(plotted_turn(&t.g, card).is_some());
    // P1's next turn: cast without kicker (the Interrupter adds {2}), at a new target.
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::PrecombatMain);
    t.cast(P1, card)
        .method(PLOTTED)
        .kicked(false)
        .target(bears)
        .go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn fblthp_a_card_with_plot_on_top_can_use_either_plot_cost() {
    cr!("702.170a", "702.170f");
    ruling!(
        "Fblthp, Lost on the Range",
        "If the top card of your library normally has plot, you may plot that card using its own plot cost or the plot cost given to it by Fblthp."
    );
    supported("Fblthp, Lost on the Range");
    supported("Freestrider Commando");
    supported("Djinn of Fool's Fall");
    // Freestrider Commando: {2}{G}, plot {3}{G}. With three mana, it can be plotted for
    // its mana cost (Fblthp's plot cost).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fblthp, Lost on the Range");
    let c = t.library_top(P0, "Freestrider Commando");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 2);
    assert!(can_plot(&mut t, P0, c));
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, Action::Special(SpecialAction::Plot { card: c }))
        .expect("plot");
    assert!(plotted_turn(&t.g, exiled(&t, "Freestrider Commando")).is_some());
    // Djinn of Fool's Fall: {4}{U}, plot {3}{U}. With four mana, it can be plotted for
    // its own plot cost.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fblthp, Lost on the Range");
    let d = t.library_top(P0, "Djinn of Fool's Fall");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    assert!(!can_plot(&mut t, P0, d));
    t.lands(P0, "Wastes", 1);
    assert!(can_plot(&mut t, P0, d));
    // With both costs payable, the player chooses: the mana cost {4}{U}.
    t.lands(P0, "Wastes", 1);
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(1),
    );
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, Action::Special(SpecialAction::Plot { card: d }))
        .expect("plot");
    assert!(plotted_turn(&t.g, exiled(&t, "Djinn of Fool's Fall")).is_some());
    assert_eq!(tapped_lands(&t, P0), 5);
}

#[test]
fn make_your_own_luck_with_two_cards_in_library() {
    cr!("701.20b", "702.170a");
    ruling!(
        "Make Your Own Luck",
        "If there are fewer than three cards in your library, look at all of the cards in your library and follow the instructions in order. For example, if you had only two cards in your library, you would look at both, exile and plot up to one nonland card from among them, and then put the rest into your hand."
    );
    supported("Make Your Own Luck");
    let mut t = TestGame::new(2);
    empty_library(&mut t, P0);
    let bears = t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Island");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 3);
    let m = t.hand(P0, "Make Your Own Luck");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, m).go();
    t.resolve_all();
    assert!(plotted_turn(&t.g, exiled(&t, "Grizzly Bears")).is_some());
    assert!(t.in_hand(P0, "Island"));
    assert_eq!(t.library_size(P0), 0);
}

#[test]
fn lilah_doesnt_plot_a_spell_that_exiles_itself() {
    cr!("608.2n", "614.1a", "702.170a");
    ruling!(
        "Lilah, Undefeated Slickshot",
        "The spell will have all its normal effects before being exiled. If the spell's normal effects include exiling itself, Lilah's effect won't apply to it and it won't become plotted."
    );
    supported("Lilah, Undefeated Slickshot");
    supported("Reap the Past");
    // Reap the Past ({X}{R}{G}, X = 1): "Return X cards at random from your graveyard to
    // your hand. Exile Reap the Past." It exiles itself: not plotted.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lilah, Undefeated Slickshot");
    t.graveyard(P0, "Island");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 1);
    let r = t.hand(P0, "Reap the Past");
    t.cast(P0, r).x(1).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Island"));
    let card = exiled(&t, "Reap the Past");
    assert!(plotted_turn(&t.g, card).is_none());
    // Lightning Helix (multicolored, from hand) doesn't exile itself: it's plotted.
    supported("Lightning Helix");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lilah, Undefeated Slickshot");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Plains", 1);
    let h = t.hand(P0, "Lightning Helix");
    t.cast(P0, h).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(plotted_turn(&t.g, exiled(&t, "Lightning Helix")).is_some());
}
