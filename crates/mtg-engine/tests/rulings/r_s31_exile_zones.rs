//! Rulings batch S31 — cards in exile: a search of a player's graveyard, hand, and
//! library for cards with a name leaves the permanents with that name alone (CR 701.23,
//! 201.2), and face-down cards in exile have no characteristics (CR 406.3a).

use crate::r_s01_common::supported;
use crate::r_s04_common::add_mana;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn eradicate_doesnt_exile_permanents_with_the_same_name() {
    cr!("701.23a", "201.2", "608.2h");
    ruling!(
        "Eradicate",
        "Does not exile other cards of the same name that are on the battlefield. Just from the graveyard, hand, and library."
    );
    supported("Eradicate");
    // "Exile target nonblack creature. Search its controller's graveyard, hand, and
    // library for all cards with the same name as that creature and exile them. Then that
    // player shuffles."
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Grizzly Bears");
    let in_hand = t.hand(P1, "Grizzly Bears");
    let in_graveyard = t.graveyard(P1, "Grizzly Bears");
    let in_library = t.library_top(P1, "Grizzly Bears");
    let unrelated = t.graveyard(P1, "Hill Giant");
    // P0's own Grizzly Bears card isn't in that player's zones.
    let mine = t.graveyard(P0, "Grizzly Bears");
    add_mana(&mut t, P0, ManaType::B, 4);
    let spell = t.hand(P0, "Eradicate");
    t.cast(P0, spell).target(target).go();
    t.resolve_all();
    for c in [target, in_hand, in_graveyard, in_library] {
        assert_eq!(t.zone(c), Zone::Exile);
    }
    assert!(t.on_battlefield(other));
    assert_eq!(t.zone(unrelated), Zone::Graveyard(P1));
    assert_eq!(t.zone(mine), Zone::Graveyard(P0));
}

#[test]
fn eradicate_must_find_graveyard_copies_but_may_leave_hidden_ones() {
    cr!("701.23a", "701.23b", "400.2");
    ruling!(
        "Eradicate",
        "The copies must be found if they are in publicly viewable zones. Finding copies while searching private zones is optional."
    );
    supported("Eradicate");
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Grizzly Bears");
    let in_hand = t.hand(P1, "Grizzly Bears");
    let in_graveyard = t.graveyard(P1, "Grizzly Bears");
    let in_library = t.library_top(P1, "Grizzly Bears");
    add_mana(&mut t, P0, ManaType::B, 4);
    let spell = t.hand(P0, "Eradicate");
    // P0 chooses to find none of the copies in P1's hand and library.
    t.answer_choose(P0, &[]);
    t.answer_choose(P0, &[]);
    let from = t.asked().len();
    t.cast(P0, spell).target(target).go();
    t.resolve_all();
    assert_eq!(t.zone(target), Zone::Exile);
    // The graveyard copy is found regardless: it was never offered as a choice.
    assert_eq!(t.zone(in_graveyard), Zone::Exile);
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .filter(|(p, _)| *p == P0)
        .flat_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities { candidates, .. } => {
                candidates.clone()
            }
            _ => vec![],
        })
        .collect();
    assert!(offered.contains(&Entity::Object(in_hand)));
    assert!(offered.contains(&Entity::Object(in_library)));
    assert!(!offered.contains(&Entity::Object(in_graveyard)));
    // The hidden copies stay where they were.
    assert_eq!(t.zone(in_hand), Zone::Hand(P1));
    assert!(matches!(t.zone(in_library), Zone::Library(_)));
}

#[test]
fn splinter_searches_the_artifacts_controllers_zones() {
    cr!("701.23a", "201.2");
    ruling!(
        "Splinter",
        "Does not exile other cards of the same name that are on the battlefield. Just from the graveyard, hand, and library."
    );
    supported("Splinter");
    // "Exile target artifact. Search its controller's graveyard, hand, and library for all
    // cards with the same name as that artifact and exile them. Then that player
    // shuffles."
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Ornithopter");
    let other = t.battlefield(P0, "Ornithopter");
    let in_hand = t.hand(P1, "Ornithopter");
    add_mana(&mut t, P0, ManaType::G, 4);
    let spell = t.hand(P0, "Splinter");
    t.cast(P0, spell).target(target).go();
    t.resolve_all();
    assert_eq!(t.zone(target), Zone::Exile);
    assert_eq!(t.zone(in_hand), Zone::Exile);
    assert!(t.on_battlefield(other));
}

#[test]
fn crackling_drake_doesnt_count_face_down_exiled_cards() {
    cr!("406.3a", "702.143a", "604.3");
    ruling!(
        "Crackling Drake",
        "If any exiled cards you own are face down, they have no characteristics. If they're normally instants or sorceries, they won't be counted."
    );
    supported("Crackling Drake");
    supported("Saw It Coming");
    // "Crackling Drake's power is equal to the total number of instant and sorcery cards
    // you own in exile and in your graveyard."
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P0, "Crackling Drake");
    t.graveyard(P0, "Lightning Bolt");
    t.exile(P0, "Shock");
    // Cards that aren't instants or sorceries, or aren't P0's, don't count.
    t.graveyard(P0, "Grizzly Bears");
    t.exile(P1, "Lightning Bolt");
    t.g.recompute();
    assert_eq!(t.pt(drake).0, 2);
    // Saw It Coming (an instant) foretold: exiled face down, it isn't counted.
    let card = t.hand(P0, "Saw It Coming");
    add_mana(&mut t, P0, ManaType::C, 2);
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, Action::Special(SpecialAction::Foretell { card }))
        .expect("foretell");
    assert_eq!(t.zone(card), Zone::Exile);
    assert!(t.obj_now(card).face_down);
    t.g.recompute();
    assert_eq!(t.pt(drake).0, 2);
}

#[test]
fn beacon_bolt_doesnt_count_face_down_exiled_cards() {
    cr!("406.3a", "702.143a");
    ruling!(
        "Beacon Bolt",
        "If any exiled cards you own are face down, they have no characteristics. If they're normally instants or sorceries, they won't be counted."
    );
    supported("Beacon Bolt");
    // "Beacon Bolt deals damage to target creature equal to the total number of instant
    // and sorcery cards you own in exile and in your graveyard."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.graveyard(P0, "Lightning Bolt");
    t.exile(P0, "Shock");
    let card = t.hand(P0, "Saw It Coming");
    add_mana(&mut t, P0, ManaType::C, 2);
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, Action::Special(SpecialAction::Foretell { card }))
        .expect("foretell");
    add_mana(&mut t, P0, ManaType::U, 1);
    add_mana(&mut t, P0, ManaType::R, 1);
    add_mana(&mut t, P0, ManaType::C, 1);
    let spell = t.hand(P0, "Beacon Bolt");
    t.cast(P0, spell).target(giant).go();
    t.resolve_all();
    // 2 damage: the face-down card isn't counted (nor Beacon Bolt itself, on the stack).
    assert_eq!(t.obj_now(giant).damage, 2);
}

#[test]
fn cemetery_gatekeeper_compares_card_types_not_supertypes() {
    cr!("205.2a", "205.4a", "603.4", "607.2a");
    ruling!(
        "Cemetery Gatekeeper",
        "Card types that can be exiled from a graveyard include artifact, creature, enchantment, land, planeswalker, instant, and sorcery. Legendary, basic, and snow are supertypes, not card types."
    );
    supported("Cemetery Gatekeeper");
    supported("Gaea's Cradle");
    // "When this creature enters, exile a card from a graveyard. Whenever a player plays a
    // land or casts a spell, if it shares a card type with the exiled card, this creature
    // deals 2 damage to that player." It exiles Isamaru, Hound of Konda (a legendary
    // creature card).
    let mut t = TestGame::new(2);
    let isamaru = t.graveyard(P1, "Isamaru, Hound of Konda");
    t.answer_choose(P0, &[Entity::Object(isamaru)]);
    t.enter(P0, "Cemetery Gatekeeper");
    t.resolve_all();
    assert_eq!(t.zone(isamaru), Zone::Exile);
    // P1 plays Gaea's Cradle, a legendary land: "legendary" is a supertype, not a card
    // type, so nothing happens.
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let cradle = t.hand(P1, "Gaea's Cradle");
    t.play_land(P1, cradle).expect("play a land");
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // P1 casts Grizzly Bears, a creature: 2 damage.
    add_mana(&mut t, P1, ManaType::G, 2);
    let bears = t.hand(P1, "Grizzly Bears");
    t.cast(P1, bears).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn cemetery_protector_makes_a_token_for_a_land_after_exiling_a_basic_land() {
    cr!("205.2a", "205.4a", "603.4");
    ruling!(
        "Cemetery Protector",
        "Card types that can be exiled from a graveyard include artifact, creature, enchantment, land, planeswalker, instant, and sorcery."
    );
    supported("Cemetery Protector");
    // "When this creature enters, exile a card from a graveyard. Whenever you play a land
    // or cast a spell, if it shares a card type with the exiled card, create a 1/1 white
    // Human creature token." It exiles a Snow-Covered Forest: any land shares its card
    // type (land), whatever its supertypes.
    let mut t = TestGame::new(2);
    let snowy = t.graveyard(P1, "Snow-Covered Forest");
    t.answer_choose(P0, &[Entity::Object(snowy)]);
    t.enter(P0, "Cemetery Protector");
    t.resolve_all();
    assert_eq!(t.zone(snowy), Zone::Exile);
    let wastes = t.hand(P0, "Wastes");
    t.play_land(P0, wastes).expect("play a land");
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 1);
    // An instant isn't a land: no token.
    add_mana(&mut t, P0, ManaType::R, 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 1);
}

#[test]
fn vren_exiles_only_dying_creatures_and_counts_them_at_end_step() {
    cr!("614.1a", "700.4", "701.9a", "701.17a");
    ruling!(
        "Vren, the Relentless",
        "Cards that would go to your opponent's graveyard for reasons other than dying, such as being discarded or milled, will still go to the graveyard and will not be exiled instead."
    );
    supported("Vren, the Relentless");
    // "If a creature an opponent controls would die, exile it instead. At the beginning of
    // each end step, create X 1/1 black Rat creature tokens with "This token gets +1/+1
    // for each other Rat you control," where X is the number of creatures that were
    // exiled under your opponents' control this turn."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vren, the Relentless");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let discarded = t.hand(P1, "Hill Giant");
    let milled = t.library_top(P1, "Craw Wurm");
    let mine = t.battlefield(P0, "Llanowar Elves");
    // P1's creature dies: exiled instead.
    crate::r_s02_common::destroy(&mut t, bears);
    assert_eq!(t.zone(bears), Zone::Exile);
    // A creature card P1 discards, or that's milled, goes to the graveyard.
    t.g.discard(P1, discarded, None);
    t.settle();
    assert_eq!(t.zone(discarded), Zone::Graveyard(P1));
    t.g.mill(P1, 1);
    t.settle();
    assert_eq!(t.zone(milled), Zone::Graveyard(P1));
    // P0's own creature dies normally.
    crate::r_s02_common::destroy(&mut t, mine);
    assert_eq!(t.zone(mine), Zone::Graveyard(P0));
    // One creature was exiled under an opponent's control this turn: one Rat token, 2/2
    // with Vren (a Rat) as the other Rat P0 controls.
    t.advance_to(P0, mtg_engine::turn::Step::End);
    t.resolve_all();
    let rats = crate::r_s01_common::tokens(&t, P0);
    assert_eq!(rats.len(), 1);
    assert_eq!(t.pt(rats[0]), (2, 2));
}

/// P0's Summoner's Sending ("At the beginning of your end step, you may exile target
/// creature card from a graveyard. If you do, create a 1/1 white Spirit creature token with
/// flying. Put a +1/+1 counter on it if the exiled card's mana value is 4 or greater.")
/// exiles `name` from P1's graveyard. Returns the Spirit token's P/T.
fn summoners_sending_exiles(name: &str) -> (i32, i32) {
    supported("Summoner's Sending");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Summoner's Sending");
    let card = t.graveyard(P1, name);
    t.answer_targets(P0, &[Entity::Object(card)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, mtg_engine::turn::Step::End);
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Exile);
    let spirits = crate::r_s01_common::tokens(&t, P0);
    assert_eq!(spirits.len(), 1);
    t.pt(spirits[0])
}

#[test]
fn summoners_sending_counts_x_as_0_in_the_exiled_cards_mana_value() {
    cr!("202.3e", "107.3g");
    ruling!(
        "Summoner's Sending",
        "If the exiled card has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    // Primordial Hydra ({X}{G}{G}) has mana value 2: no counter. Craw Wurm (mana value 6)
    // gets one.
    assert_eq!(summoners_sending_exiles("Primordial Hydra"), (1, 1));
    assert_eq!(summoners_sending_exiles("Craw Wurm"), (2, 2));
}

#[test]
fn exiling_the_top_cards_of_that_players_library() {
    cr!("406.1", "603.2", "510.2");
    // A creature with "Whenever this creature deals combat damage to a player, exile the
    // top two cards of that player's library.": the damaged player's cards.
    let def = crate::r_s01_common::custom_card(
        "Library Raider",
        "Creature — Rogue",
        "{2}",
        Some((2, 2)),
        "Whenever this creature deals combat damage to a player, exile the top two cards of that player's library.",
    );
    let mut t = TestGame::new(3);
    let raider = t.custom(P0, def, Zone::Battlefield);
    let theirs = crate::r_s01_common::stack_library(&mut t, P2, &["Grizzly Bears", "Hill Giant"]);
    let other = t.library_top(P1, "Craw Wurm");
    let mine = t.library_top(P0, "Forest");
    crate::r_s01_common::attack_with(&mut t, &[(raider, Entity::Player(P2))]);
    crate::r_s01_common::block_and_finish(&mut t, P2, &[]);
    t.resolve_all();
    assert_eq!(t.life(P2), 18);
    for c in &theirs {
        assert_eq!(t.zone(*c), Zone::Exile);
    }
    assert!(matches!(t.zone(other), Zone::Library(_)));
    assert!(matches!(t.zone(mine), Zone::Library(_)));
}
