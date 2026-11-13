//! Rulings batch S04 — delirium (an ability word, CR 207.2c): abilities that care whether
//! there are four or more card types among cards in your graveyard.

use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, create_token};
use crate::r_s04_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts real cards into `p`'s graveyard.
fn bury(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    names.iter().map(|n| t.graveyard(p, n)).collect()
}

/// Whether P0's Deathcap Cultivator ("Delirium — This creature has deathtouch as long as
/// there are four or more card types among cards in your graveyard.") has deathtouch now.
fn delirium(t: &mut TestGame, cultivator: ObjectId) -> bool {
    t.g.recompute();
    t.obj_now(cultivator).has_keyword(KeywordKind::Deathtouch)
}

#[test]
fn delirium_counts_card_types_not_supertypes_or_subtypes() {
    cr!("205.2a", "205.4a", "205.3a", "207.2c");
    ruling!(
        "Deathcap Cultivator",
        "The card types in Magic are artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Supertypes (such as legendary and basic) and subtypes (such as Human and Equipment) are not counted."
    );
    supported("Deathcap Cultivator");
    // A legendary creature, a basic snow land, and an Equipment: three card types.
    let mut t = TestGame::new(2);
    let cultivator = t.battlefield(P0, "Deathcap Cultivator");
    bury(
        &mut t,
        P0,
        &["Isamaru, Hound of Konda", "Snow-Covered Forest", "Bonesplitter"],
    );
    assert!(!delirium(&mut t, cultivator));
    // A battle is a fourth.
    bury(&mut t, P0, &["Invasion of Tarkir"]);
    assert!(delirium(&mut t, cultivator));

    // A kindred instant is two card types (kindred and instant); with a creature, three.
    let mut t = TestGame::new(2);
    let cultivator = t.battlefield(P0, "Deathcap Cultivator");
    bury(&mut t, P0, &["Nameless Inversion", "Grizzly Bears"]);
    assert!(!delirium(&mut t, cultivator));
    // A planeswalker is a fourth.
    bury(&mut t, P0, &["Jace Beleren"]);
    assert!(delirium(&mut t, cultivator));

    // Enchantment, sorcery, land, and creature.
    let mut t = TestGame::new(2);
    let cultivator = t.battlefield(P0, "Deathcap Cultivator");
    bury(&mut t, P0, &["Faerie Tauntings", "Divination", "Forest"]);
    // Faerie Tauntings is a kindred enchantment: kindred, enchantment, sorcery, land.
    assert!(delirium(&mut t, cultivator));
}

#[test]
fn delirium_counts_card_types_not_cards() {
    cr!("205.2a", "207.2c");
    ruling!(
        "Deathcap Cultivator",
        "The number of card types matters, not the number of cards. For example, Wicker Witch (an artifact creature) along with Catalog (an instant) and Chaplain's Blessing (a sorcery) will enable delirium."
    );
    let mut t = TestGame::new(2);
    let cultivator = t.battlefield(P0, "Deathcap Cultivator");
    bury(&mut t, P0, &["Wicker Witch", "Catalog", "Chaplain's Blessing"]);
    assert_eq!(t.graveyard_size(P0), 3);
    assert!(delirium(&mut t, cultivator));
    // Many cards of the same types aren't enough.
    let mut t = TestGame::new(2);
    let cultivator = t.battlefield(P0, "Deathcap Cultivator");
    bury(
        &mut t,
        P0,
        &[
            "Grizzly Bears",
            "Hill Giant",
            "Craw Wurm",
            "Forest",
            "Swamp",
            "Lightning Bolt",
            "Shock",
        ],
    );
    assert!(!delirium(&mut t, cultivator));
}

#[test]
fn tokens_and_copies_in_a_graveyard_arent_cards() {
    cr!("108.2b", "111.7", "704.5d", "207.2c");
    ruling!(
        "Deathcap Cultivator",
        "In some rare cases, you can have a token or a copy of a spell in your graveyard at the moment that an object's delirium ability counts the card types among cards in your graveyard, before that token or copy ceases to exist. Because tokens and copies of spells are not cards, even if they are copies of cards, their types will never be counted."
    );
    let mut t = TestGame::new(2);
    let cultivator = t.battlefield(P0, "Deathcap Cultivator");
    bury(&mut t, P0, &["Grizzly Bears", "Forest", "Lightning Bolt"]);
    // A Treasure token (an artifact) is put into the graveyard; before state-based actions
    // make it cease to exist, it's there but not counted.
    let treasure = create_token(&mut t, P0, "Treasure");
    t.g.destroy(treasure, None);
    let token = t.g.current(treasure);
    assert_eq!(t.zone(token), Zone::Graveyard(P0));
    assert!(!delirium(&mut t, cultivator));
    t.settle();
    assert!(!t.g.player(P0).graveyard.contains(&token));
    // An artifact card is counted.
    bury(&mut t, P0, &["Mind Stone"]);
    assert!(delirium(&mut t, cultivator));
}

#[test]
fn only_a_double_faced_cards_front_face_counts_in_the_graveyard() {
    cr!("712.8a", "207.2c");
    ruling!(
        "Deathcap Cultivator",
        "Because you consider only the characteristics of a double-faced card's front face while it's not on the battlefield, the types of its back face won't be counted for delirium."
    );
    // Search for Azcanta (an enchantment) transforms into a land; Autumnal Gloom (an
    // enchantment) into a creature. With an instant, only enchantment and instant count.
    let mut t = TestGame::new(2);
    let cultivator = t.battlefield(P0, "Deathcap Cultivator");
    bury(
        &mut t,
        P0,
        &["Search for Azcanta", "Autumnal Gloom", "Lightning Bolt"],
    );
    assert!(!delirium(&mut t, cultivator));
    bury(&mut t, P0, &["Forest"]);
    assert!(!delirium(&mut t, cultivator));
    bury(&mut t, P0, &["Grizzly Bears"]);
    assert!(delirium(&mut t, cultivator));
}

#[test]
fn a_delirium_trigger_checks_card_types_as_it_triggers_and_resolves() {
    cr!("603.4", "207.2c");
    ruling!(
        "Soul Swallower",
        "Most triggered delirium abilities use an intervening \"if\" clause. There must be four or more card types among cards in your graveyard in order for these abilities to trigger, otherwise they never trigger at all."
    );
    supported("Soul Swallower");
    supported("Relic of Progenitus");
    // Soul Swallower: "Delirium — At the beginning of your upkeep, if there are four or
    // more card types among cards in your graveyard, put three +1/+1 counters on this
    // creature."
    // Three card types: it doesn't trigger, even if a fourth is added during the upkeep.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Soul Swallower");
    bury(&mut t, P0, &["Forest", "Grizzly Bears", "Lightning Bolt"]);
    next_upkeep(&mut t, P0);
    assert_eq!(on_stack(&t, "card types"), 0, "{:?}", stack_items(&t));
    bury(&mut t, P0, &["Mind Stone"]);
    t.resolve_all();
    assert_eq!(t.counters(wurm, "+1/+1"), 0);

    // Four: it triggers. In response a card is exiled from the graveyard (Relic of
    // Progenitus), leaving three: it resolves doing nothing.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Soul Swallower");
    let relic = t.battlefield(P1, "Relic of Progenitus");
    let cards = bury(
        &mut t,
        P0,
        &["Forest", "Grizzly Bears", "Lightning Bolt", "Mind Stone"],
    );
    next_upkeep(&mut t, P0);
    assert_eq!(on_stack(&t, "card types"), 1, "{:?}", stack_items(&t));
    t.answer_choose(P0, &[Entity::Object(cards[3])]);
    t.activate(P1, relic, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve();
    assert_eq!(t.zone(cards[3]), Zone::Exile);
    t.resolve_all();
    assert_eq!(t.counters(wurm, "+1/+1"), 0);
}

#[test]
fn a_delirium_trigger_resolves_if_the_types_change_but_not_their_number() {
    cr!("603.4", "207.2c");
    ruling!(
        "Soul Swallower",
        "The number of card types is checked again as the trigger resolves, and if it has become too low somehow, the ability does nothing. If which card types are in your graveyard changes but the quantity of card types stays the same (or increases), then the delirium triggered ability will still resolve."
    );
    // Land, creature, instant, artifact. In response to the trigger, the artifact is
    // exiled and a sorcery (Akroma's Vengeance, cycled) is added: still four types.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Soul Swallower");
    let relic = t.battlefield(P1, "Relic of Progenitus");
    t.lands(P0, "Wastes", 3);
    let cards = bury(
        &mut t,
        P0,
        &["Forest", "Grizzly Bears", "Lightning Bolt", "Mind Stone"],
    );
    let vengeance = t.hand(P0, "Akroma's Vengeance");
    next_upkeep(&mut t, P0);
    assert_eq!(on_stack(&t, "card types"), 1, "{:?}", stack_items(&t));
    t.answer_choose(P0, &[Entity::Object(cards[3])]);
    t.activate(P1, relic, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve();
    cycle(&mut t, P0, vengeance, 0).unwrap();
    t.resolve();
    assert_eq!(
        graveyard_names(&t, P0),
        vec![
            "Forest",
            "Grizzly Bears",
            "Lightning Bolt",
            "Akroma's Vengeance"
        ]
    );
    t.resolve_all();
    assert_eq!(t.counters(wurm, "+1/+1"), 3);
}

#[test]
fn a_delirium_activated_ability_isnt_rechecked_as_it_resolves() {
    cr!("602.5", "207.2c");
    ruling!(
        "Stallion of Ashmouth",
        "Some delirium abilities are activated abilities of permanents. To activate such an ability, there must be four or more card types among cards in your graveyard. The number of card types is not rechecked as the ability resolves."
    );
    supported("Stallion of Ashmouth");
    supported("Tormod's Crypt");
    // Stallion of Ashmouth: "Delirium — {1}{B}: This creature gets +1/+1 until end of
    // turn. Activate only if there are four or more card types among cards in your
    // graveyard."
    let mut t = TestGame::new(2);
    let stallion = t.battlefield(P0, "Stallion of Ashmouth");
    t.lands(P0, "Swamp", 2);
    bury(&mut t, P0, &["Forest", "Grizzly Bears", "Lightning Bolt"]);
    assert!(!can_activate(&mut t, P0, stallion));
    assert!(t.activate(P0, stallion, 0, &[]).is_err());
    bury(&mut t, P0, &["Mind Stone"]);
    assert!(can_activate(&mut t, P0, stallion));
    t.activate(P0, stallion, 0, &[]).unwrap();
    // In response, P0's graveyard is exiled (Tormod's Crypt): it still resolves.
    let crypt = t.battlefield(P1, "Tormod's Crypt");
    t.activate(P1, crypt, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve();
    assert_eq!(t.graveyard_size(P0), 0);
    t.resolve();
    assert_eq!(t.pt(stallion), (4, 4));
}

#[test]
fn a_delirium_instead_spell_checks_as_it_resolves_without_counting_itself() {
    cr!("608.2c", "207.2c");
    ruling!(
        "Might Beyond Reason",
        "Some delirium abilities that appear on instants and sorceries use the word \"instead.\" These spells have an upgraded effect when they resolve if there are four or more card types among cards in your graveyard. They check that number only while they're resolving and don't count themselves, since they aren't in your graveyard yet. You only get the upgraded effect, not both effects."
    );
    supported("Might Beyond Reason");
    supported("Traverse the Ulvenwald");
    // Might Beyond Reason: "Put two +1/+1 counters on target creature. Delirium — Put three
    // +1/+1 counters on that creature instead if there are four or more card types among
    // cards in your graveyard."
    // Land, creature, artifact: the instant itself isn't counted.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    bury(&mut t, P0, &["Forest", "Hill Giant", "Mind Stone"]);
    give_mana_for(&mut t, P0, "Might Beyond Reason");
    let might = t.hand(P0, "Might Beyond Reason");
    t.cast(P0, might).target(bears).go();
    t.resolve();
    assert_eq!(t.counters(bears, "+1/+1"), 2);
    // Now there's an instant too: three counters, not five.
    let bears2 = t.battlefield(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Might Beyond Reason");
    let might = t.hand(P0, "Might Beyond Reason");
    t.cast(P0, might).target(bears2).go();
    t.resolve();
    assert_eq!(t.counters(bears2, "+1/+1"), 3);
    // Checked only as it resolves: a fourth type added in response counts.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    bury(&mut t, P0, &["Forest", "Hill Giant", "Mind Stone"]);
    give_mana_for(&mut t, P0, "Might Beyond Reason");
    t.lands(P0, "Wastes", 3);
    let might = t.hand(P0, "Might Beyond Reason");
    let vengeance = t.hand(P0, "Akroma's Vengeance");
    t.cast(P0, might).target(bears).go();
    cycle(&mut t, P0, vengeance, 0).unwrap();
    t.resolve();
    t.resolve();
    assert_eq!(t.counters(bears, "+1/+1"), 3);

    // Traverse the Ulvenwald (a sorcery): "Search your library for a basic land card ...
    // Delirium — If there are four or more card types among cards in your graveyard,
    // instead search your library for a creature or land card ..."
    for (buried, creature_allowed) in [
        (&["Forest", "Grizzly Bears", "Lightning Bolt"][..], false),
        (&["Forest", "Grizzly Bears", "Lightning Bolt", "Mind Stone"][..], true),
    ] {
        let mut t = TestGame::new(2);
        bury(&mut t, P0, buried);
        let giant = t.library_top(P0, "Hill Giant");
        let island = t.library_top(P0, "Island");
        let grave = t.library_top(P0, "Watery Grave");
        t.lands(P0, "Forest", 1);
        let traverse = t.hand(P0, "Traverse the Ulvenwald");
        t.cast(P0, traverse).go();
        let from = t.asked().len();
        t.answer_choose(P0, &[Entity::Object(island)]);
        t.resolve();
        let offered: Vec<Entity> = t.asked()[from..]
            .iter()
            .find_map(|(_, d)| match d {
                Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
                _ => None,
            })
            .unwrap_or_default();
        assert!(offered.contains(&Entity::Object(island)));
        assert_eq!(offered.contains(&Entity::Object(giant)), creature_allowed);
        assert_eq!(offered.contains(&Entity::Object(grave)), creature_allowed);
        assert!(t.in_hand(P0, "Island"));
    }
}
