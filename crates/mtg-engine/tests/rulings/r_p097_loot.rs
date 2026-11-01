//! Rulings batch P097 — looting and its relatives: drawing then discarding all while the
//! spell or ability resolves (CR 608.2c), random discards and shared card types
//! (CR 205.2a, 205.4), "exile until this leaves the battlefield" (CR 610.3), and a spell
//! whose only target is gone (CR 608.2b).

use crate::r_p097_common::*;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn grasp_of_fate_exiled_permanents_lose_auras_equipment_and_counters() {
    cr!("610.3", "400.7", "122.2", "303.4c", "301.5c");
    ruling!(
        "Grasp of Fate",
        "Auras attached to exiled nonland permanents will be put into their owners' graveyards. Equipment attached to exiled creatures will become unattached and remain on the battlefield. Any counters on exiled nonland permanents will cease to exist."
    );
    ruling!(
        "Celebrate the Mountain-king",
        "Auras attached to exiled nonland permanents will be put into their owners' graveyards. Equipment attached to exiled creatures will become unattached and remain on the battlefield. Any counters on exiled nonland permanents will cease to exist."
    );
    for case in ["Grasp of Fate", "Celebrate the Mountain-king"] {
        supported(case);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        crate::r_s29_common::put_counters(&mut t, bears, "+1/+1", 2);
        let aura = crate::r_s06_common::attach_new(&mut t, P1, "Rancor", bears);
        let eq = crate::r_s06_common::attach_new(&mut t, P1, "Bonesplitter", bears);
        t.answer_targets(P0, &[Entity::Object(bears)]);
        let e = t.enter(P0, case);
        t.resolve_all();
        assert_eq!(t.zone(bears), Zone::Exile, "{case}");
        assert!(t.in_graveyard(P1, "Rancor") || t.in_hand(P1, "Rancor"));
        let _ = aura;
        assert!(t.on_battlefield(eq));
        assert_eq!(t.obj_now(eq).attached_to, None);
        // It leaves: the creature returns with no counters.
        t.g.destroy(e, None);
        t.resolve_all();
        let back = t.named_on_battlefield("Grizzly Bears");
        assert_eq!(back.len(), 1);
        assert_eq!(t.counters(back[0], "+1/+1"), 0);
        assert_eq!(t.pt(back[0]), (2, 2));
        assert_eq!(t.obj_now(back[0]).controller, P1);
    }
}

#[test]
fn celebrate_leaving_before_its_trigger_resolves_exiles_nothing() {
    cr!("610.3c");
    ruling!(
        "Celebrate the Mountain-king",
        "If Celebrate the Mountain-king leaves the battlefield before its triggered ability resolves, no nonland permanents will be exiled."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let e = t.enter(P0, "Celebrate the Mountain-king");
    t.settle();
    assert!(t.stack_len() >= 1);
    t.g.destroy(e, None);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn celebrate_exiled_cards_return_when_its_owner_leaves_the_game() {
    cr!("800.4a", "610.3");
    ruling!(
        "Celebrate the Mountain-king",
        "In a multiplayer game, if Celebrate the Mountain-king's owner leaves the game, the exiled cards will return to the battlefield."
    );
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P2, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.enter(P0, "Celebrate the Mountain-king");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(giant), Zone::Exile);
    t.g.lose_game(P0);
    t.settle();
    assert!(t.has_lost(P0));
    let b = t.named_on_battlefield("Grizzly Bears");
    let g = t.named_on_battlefield("Hill Giant");
    assert_eq!(b.len(), 1, "{}", t.dump_log());
    assert_eq!(g.len(), 1);
    assert_eq!(t.obj_now(b[0]).controller, P1);
    assert_eq!(t.obj_now(g[0]).controller, P2);
}

#[test]
fn refute_whose_target_is_gone_doesnt_loot() {
    cr!("608.2b");
    ruling!(
        "Refute",
        "If the target spell is an illegal target when Refute tries to resolve, it won't resolve and none of its effects will happen. You won't draw or discard."
    );
    supported("Refute");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    pool(&mut t, P1, &[(ManaType::R, 1)]);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(P0).go();
    pool(&mut t, P0, &[(ManaType::U, 2), (ManaType::C, 1)]);
    let refute = t.hand(P0, "Refute");
    t.cast(P0, refute).target(bolt).go();
    // The Bolt leaves the stack in response.
    t.g.move_object(bolt, Zone::Exile, events::MoveCause::Effect, None);
    let library = t.library_size(P0);
    t.resolve_all();
    assert_eq!(t.library_size(P0), library);
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.graveyard_size(P0), 1);
}

#[test]
fn ideas_unbound_discards_whole_hand_if_fewer_than_three() {
    cr!("701.9a", "603.7");
    ruling!(
        "Ideas Unbound",
        "If you don’t have three or more cards in hand at the end of turn, you discard what cards you have."
    );
    supported("Ideas Unbound");
    let mut t = TestGame::new(2);
    pool(&mut t, P0, &[(ManaType::U, 2)]);
    let s = t.hand(P0, "Ideas Unbound");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
    // Two of them leave the hand before the end step.
    for c in t.g.player(P0).hand.clone().into_iter().take(2) {
        t.g.move_object(c, Zone::Exile, events::MoveCause::Effect, None);
    }
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.graveyard_size(P0), 2);
}

#[test]
fn draw_then_discard_happens_in_one_resolution() {
    cr!("608.2c", "117.1");
    ruling!(
        "Tezzeret's Ambition",
        "No player may take actions between the time you draw three cards and the time you discard."
    );
    ruling!(
        "Rowdy Crew",
        "Once Rowdy Crew's triggered ability begins to resolve, no player may take other actions until it's done."
    );
    ruling!(
        "Splitskin Doll",
        "Once Splitskin Doll's ability begins to resolve, players can't take actions in between the time you draw a card and the time you would discard a card."
    );
    ruling!(
        "Bazaar Trademage",
        "You draw two cards and discard three cards all while Bazaar Trademage’s triggered ability is resolving. Nothing can happen between the two, and no player may choose to take actions."
    );
    ruling!(
        "Frantic Search",
        "You draw two cards and discard two cards all while Frantic Search is resolving. Nothing can happen between the two, and no player may choose to take actions."
    );
    // (card, starting hand size, the hand size after it resolves)
    for (case, hand, after) in [
        ("Tezzeret's Ambition", 0, 2),
        ("Rowdy Crew", 0, 1),
        ("Splitskin Doll", 0, 0),
        ("Bazaar Trademage", 2, 1),
        ("Frantic Search", 1, 1),
    ] {
        supported(case);
        let mut t = TestGame::new(2);
        // The drawn cards include an artifact with no mana cost and a cheap creature.
        stack_library(&mut t, P0, &["Ornithopter", "Memnite", "Grizzly Bears"]);
        // Splitskin Doll: the only other creature has power 3.
        t.battlefield(P0, "Hill Giant");
        for _ in 0..hand {
            t.hand(P0, "Island");
        }
        if case == "Tezzeret's Ambition" || case == "Frantic Search" {
            crate::r_s01_common::give_mana_for(&mut t, P0, case);
            let s = t.hand(P0, case);
            t.cast(P0, s).go();
            let from = t.asked().len();
            t.resolve();
            assert!(!priority_since(&t, from), "{case}");
        } else {
            t.enter(P0, case);
            t.settle();
            let from = t.asked().len();
            t.resolve();
            assert!(!priority_since(&t, from), "{case}");
        }
        assert_eq!(t.hand_size(P0), after, "{case}");
        assert!(t.named_on_battlefield("Ornithopter").is_empty());
    }
}

/// Rowdy Crew enters with the three `top` cards on top of the library and an empty hand.
/// The number of +1/+1 counters it gets.
fn rowdy_crew(top: &[&str]) -> u32 {
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, top);
    let crew = t.enter(P0, "Rowdy Crew");
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 2);
    t.counters(crew, "+1/+1")
}

#[test]
fn rowdy_crew_needs_two_discarded_cards_sharing_a_card_type() {
    cr!("205.2a", "205.4a", "205.3m");
    ruling!(
        "Rowdy Crew",
        "Rowdy Crew doesn't get more than two +1/+1 counters if the discarded cards happen to share more than one card type."
    );
    ruling!(
        "Rowdy Crew",
        "The card types that can appear on the discarded cards are artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Legendary is a supertype, not a card type."
    );
    ruling!(
        "Rowdy Crew",
        "The discarded cards just need to share one card type. For example, Rowdy Crew will get two +1/+1 counters if you discard an artifact creature and an enchantment creature."
    );
    // Any two of these share artifact and creature: still two counters.
    assert_eq!(
        rowdy_crew(&["Ornithopter", "Ornithopter", "Memnite"]),
        2
    );
    // Any two share only creature.
    assert_eq!(
        rowdy_crew(&["Ornithopter", "Boon Satyr", "Dryad Arbor"]),
        2
    );
    // Any two share only kindred.
    assert_eq!(
        rowdy_crew(&["Bitterblossom", "Nameless Inversion", "Morsel Theft"]),
        2
    );
    // Any two share only the legendary supertype: no counters.
    assert_eq!(
        rowdy_crew(&["Isamaru, Hound of Konda", "Mox Amber", "Urborg, Tomb of Yawgmoth"]),
        0
    );
}

#[test]
fn occult_epiphany_counts_card_types_not_supertypes_or_subtypes() {
    cr!("205.2a", "205.3", "205.4");
    ruling!(
        "Occult Epiphany",
        "The card types that you can discard include artifact, creature, enchantment, land, planeswalker, instant, and sorcery. Legendary, basic, and snow are supertypes, not card types. Human, Equipment, and Aura are subtypes, not card types."
    );
    supported("Occult Epiphany");
    for (cards, spirits) in [
        (vec!["Isamaru, Hound of Konda", "Snow-Covered Forest"], 2),
        (vec!["Bonesplitter", "Pacifism", "Grizzly Bears"], 3),
        (vec!["Grizzly Bears", "Hill Giant"], 1),
    ] {
        let mut t = TestGame::new(2);
        let x = cards.len() as u32;
        let ids: Vec<Entity> = cards
            .iter()
            .map(|c| Entity::Object(t.hand(P0, c)))
            .collect();
        pool(&mut t, P0, &[(ManaType::U, 1), (ManaType::C, x)]);
        let s = t.hand(P0, "Occult Epiphany");
        t.answer_choose(P0, &ids);
        t.cast(P0, s).x(x as i64).go();
        t.resolve_all();
        assert_eq!(t.graveyard_size(P0), x as usize + 1, "{cards:?}");
        assert_eq!(
            crate::r_s01_common::with_subtype(&t, P0, "Spirit").len(),
            spirits,
            "{cards:?}"
        );
    }
}

#[test]
fn read_the_runes_never_forces_a_sacrifice() {
    cr!("701.9a", "701.21a", "608.2c");
    ruling!(
        "Read the Runes",
        "You are never forced to sacrifice any permanents. You can choose to discard your entire hand if your hand is smaller than the number of cards you are required to discard."
    );
    ruling!(
        "Read the Runes",
        "You draw the cards, then sacrifice any permanents you want to sacrifice, then discard the required number of card from your hand. There is no chance to play any cards in between these actions."
    );
    supported("Read the Runes");
    // Not sacrificing: X discards (the drawn cards are in hand to discard).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    pool(&mut t, P0, &[(ManaType::U, 1), (ManaType::C, 2)]);
    let s = t.hand(P0, "Read the Runes");
    t.cast(P0, s).x(2).go();
    let from = t.asked().len();
    t.resolve_all();
    assert!(!priority_since(&t, from));
    assert!(t.on_battlefield(bears));
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.graveyard_size(P0), 3);
    // Sacrificing one permanent: one discard fewer.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    pool(&mut t, P0, &[(ManaType::U, 1), (ManaType::C, 2)]);
    let s = t.hand(P0, "Read the Runes");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, s).x(2).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears), "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn see_beyond_shuffles_any_card_from_hand_even_without_drawing() {
    cr!("121.6", "701.24a");
    ruling!(
        "See Beyond",
        "The card you shuffle into your library may be any card from your hand. It doesn’t have to be one of the cards you drew with the spell."
    );
    ruling!(
        "See Beyond",
        "You have to shuffle a card from your hand into your library even if you don’t draw any cards"
    );
    supported("See Beyond");
    supported("Maralen of the Mornsong");
    // A card that was already in hand.
    let mut t = TestGame::new(2);
    let old = t.hand(P0, "Grizzly Bears");
    pool(&mut t, P0, &[(ManaType::U, 1), (ManaType::C, 1)]);
    let s = t.hand(P0, "See Beyond");
    t.answer_choose(P0, &[Entity::Object(old)]);
    t.cast(P0, s).go();
    t.resolve_all();
    assert!(matches!(t.zone(old), Zone::Library(_)));
    assert_eq!(t.hand_size(P0), 2);
    // No draws (Maralen of the Mornsong): a card from hand is still shuffled away; with an
    // empty hand, nothing happens.
    for in_hand in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Maralen of the Mornsong");
        let old = in_hand.then(|| t.hand(P0, "Grizzly Bears"));
        let library = t.library_size(P0);
        pool(&mut t, P0, &[(ManaType::U, 1), (ManaType::C, 1)]);
        let s = t.hand(P0, "See Beyond");
        t.cast(P0, s).go();
        t.resolve_all();
        assert_eq!(t.hand_size(P0), 0);
        assert_eq!(t.library_size(P0), library + in_hand as usize);
        if let Some(o) = old {
            assert!(matches!(t.zone(o), Zone::Library(_)));
        }
    }
}
