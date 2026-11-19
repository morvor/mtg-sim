//! Rulings batch P088 — Yixlid Jailer ("Cards in graveyards lose all abilities."):
//! characteristic-defining abilities (CR 604.3, 613.1f), abilities that let a card be
//! cast from a graveyard (CR 601.2, 702.34), abilities that trigger from a graveyard
//! (CR 603.6c, 603.10), and abilities of the card in other zones.

use crate::r_p088_common::*;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn tarmogoyf_in_a_graveyard_is_0_1_under_yixlid_jailer() {
    cr!("604.3", "613.1f", "208.2a");
    ruling!(
        "Yixlid Jailer",
        "If a card in a graveyard has an ability that defines a * in its power or toughness, that * is 0. For example, Tarmogoyf is a 0/1 creature card in your graveyard while you control Yixlid Jailer."
    );
    supported("Yixlid Jailer");
    supported("Tarmogoyf");
    let mut t = TestGame::new(2);
    let goyf = t.graveyard(P0, "Tarmogoyf");
    // Without Jailer: a creature card is in a graveyard (Tarmogoyf itself), 1/2.
    assert_eq!(t.pt(goyf), (1, 2));
    t.battlefield(P0, "Yixlid Jailer");
    assert_eq!(t.pt(goyf), (0, 1));
}

#[test]
fn yixlid_jailer_stops_flashback_but_not_other_permissions() {
    cr!("601.2", "601.2a", "601.2b", "601.2f", "702.34a", "613.1f");
    ruling!(
        "Yixlid Jailer",
        "If a spell or ability allows you to cast a spell from your graveyard, the first step in doing so is to move it to the stack. That spell's additional and alternative costs may be applied. However, if a card's own ability allows you to cast it from your graveyard (such as a flashback ability) Yixlid Jailer stops that ability."
    );
    supported("Think Twice");
    supported("Muldrotha, the Gravetide");
    supported("Kavu Titan");
    // Think Twice ("Draw a card. Flashback {2}{U}"): can't be cast from the graveyard.
    let mut t = TestGame::new(2);
    let tt = t.graveyard(P0, "Think Twice");
    pool(&mut t, P0, &[(ManaType::U, 3)]);
    assert!(can_cast(
        &mut t,
        P0,
        tt,
        CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Flashback)
    ));
    t.battlefield(P1, "Yixlid Jailer");
    assert!(!can_cast(
        &mut t,
        P0,
        tt,
        CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Flashback)
    ));
    // Muldrotha's permission still lets P0 cast Kavu Titan from the graveyard, and its
    // kicker (an additional cost, functioning on the stack) applies.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Yixlid Jailer");
    t.battlefield(P0, "Muldrotha, the Gravetide");
    let titan = t.graveyard(P0, "Kavu Titan");
    t.lands(P0, "Forest", 5);
    t.cast(P0, titan).kicked(true).go();
    t.resolve_all();
    assert_eq!(t.zone(titan), Zone::Battlefield);
    assert_eq!(t.counters(titan, counters::PLUS1), 3);
}

#[test]
fn yixlid_jailer_stops_abilities_that_trigger_from_the_graveyard() {
    cr!("603.6c", "603.10", "613.1f", "702.29a");
    ruling!(
        "Yixlid Jailer",
        "If an ability triggers when the object that has it is put into a graveyard from anywhere other than the battlefield, such as Krosan Tusker or Narcomoeba, that ability triggers from the graveyard. Yixlid Jailer stops those abilities from triggering at all. This includes abilities that trigger when a card is put into a graveyard \"from anywhere,\" even if that card was on the battlefield."
    );
    supported("Narcomoeba");
    supported("Krosan Tusker");
    supported("Dread");
    for jailer in [false, true] {
        let mut t = TestGame::new(2);
        if jailer {
            t.battlefield(P1, "Yixlid Jailer");
        }
        // Narcomoeba: "When this card is put into your graveyard from your library, you
        // may put it onto the battlefield."
        let n = t.library_top(P0, "Narcomoeba");
        t.answer_yes(P0, true);
        t.g.mill(P0, 1);
        t.resolve_all();
        assert_eq!(
            t.zone(n) == Zone::Battlefield,
            !jailer,
            "Narcomoeba {jailer}"
        );
        // Krosan Tusker: "When you cycle this card, you may search your library for a
        // basic land card ..."
        let tusker = t.hand(P0, "Krosan Tusker");
        pool(&mut t, P0, &[(ManaType::G, 1), (ManaType::C, 2)]);
        crate::r_s04_common::cycle(&mut t, P0, tusker, 0).expect("cycle");
        t.settle();
        assert_eq!(
            triggers_on_stack(&t, "search"),
            usize::from(!jailer),
            "Tusker {jailer}"
        );
        t.resolve_all();
        // Dread: "When Dread is put into a graveyard from anywhere, shuffle it into its
        // owner's library." — from the battlefield too.
        let dread = t.battlefield(P0, "Dread");
        destroy(&mut t, dread);
        t.resolve_all();
        let want = if jailer {
            Zone::Graveyard(P0)
        } else {
            Zone::Library(P0)
        };
        assert_eq!(t.zone(dread), want, "Dread {jailer}");
    }
}

#[test]
fn yixlid_jailer_doesnt_stop_dies_abilities() {
    cr!("603.6c", "603.10a");
    ruling!(
        "Yixlid Jailer",
        "If an ability triggers when the object that has it is put into a graveyard from the battlefield, that ability triggers from the battlefield and isn't affected by Yixlid Jailer."
    );
    supported("Doomed Dissenter");
    // Doomed Dissenter: "When this creature dies, create a 2/2 black Zombie creature
    // token."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Yixlid Jailer");
    let d = t.battlefield(P0, "Doomed Dissenter");
    destroy(&mut t, d);
    t.resolve_all();
    assert_eq!(crate::r_s25_common::creature_tokens(&t, P0), 1);
}

#[test]
fn as_enters_abilities_apply_to_cards_returned_from_the_graveyard() {
    cr!("614.1c", "614.12", "613.1f");
    ruling!(
        "Yixlid Jailer",
        "Some cards have abilities that apply \"as [this card] enters the battlefield\" or state that \"[this card] enters the battlefield with\" counters. Although these cards won't have these abilities in the graveyard, they will be applied if the cards are put onto the battlefield from the graveyard (due to Zombify, perhaps). What matters is that these cards will have these abilities on the battlefield."
    );
    supported("Zombify");
    supported("Servant of the Scale");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Yixlid Jailer");
    let servant = t.graveyard(P0, "Servant of the Scale");
    assert!(t.obj_now(servant).chars.abilities.is_empty());
    cast_new(&mut t, P0, "Zombify", &[obj(servant)]);
    t.resolve_all();
    assert_eq!(t.zone(servant), Zone::Battlefield);
    assert_eq!(t.counters(servant, counters::PLUS1), 1);
}

#[test]
fn darksteel_colossus_never_reaches_the_graveyard_under_yixlid_jailer() {
    cr!("614.1a", "614.6", "613.1f");
    ruling!(
        "Yixlid Jailer",
        "Some replacement effects cause a card to be put somewhere else instead of being put into a graveyard (such as that of Darksteel Colossus). These effects mean the card is never actually put into the graveyard, so Yixlid Jailer doesn't affect that ability."
    );
    supported("Darksteel Colossus");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Yixlid Jailer");
    let colossus = t.hand(P0, "Darksteel Colossus");
    t.g.discard(P0, colossus, None);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.zone(colossus), Zone::Library(P0));
    assert_eq!(t.graveyard_size(P0), 0);
}
