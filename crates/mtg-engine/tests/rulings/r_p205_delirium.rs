//! Rulings batch P205 — delirium (an ability word, CR 207.2c): four or more card types
//! among cards in your graveyard.

use crate::r_p205_common::*;
use crate::r_s01_common::*;
use crate::r_s04_common::next_upkeep;
use crate::r_s05_common::move_to;
use crate::r_s06_common::attach_new;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Casts Traverse the Ulvenwald with `buried` in P0's graveyard (choosing the Island) and
/// returns whether a creature card (Hill Giant) and a nonbasic land card (Azorius
/// Guildgate) in P0's library were offered.
fn traverse(buried: &[&str]) -> (bool, bool) {
    let mut t = TestGame::new(2);
    bury(&mut t, P0, buried);
    let giant = t.library_top(P0, "Hill Giant");
    let island = t.library_top(P0, "Island");
    let gate = t.library_top(P0, "Azorius Guildgate");
    t.lands(P0, "Forest", 1);
    let card = t.hand(P0, "Traverse the Ulvenwald");
    t.cast(P0, card).go();
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
    assert!(t.in_hand(P0, "Island"));
    // The upgraded effect replaces the basic one: only one card was found.
    assert_eq!(t.hand_size(P0), 1);
    (
        offered.contains(&Entity::Object(giant)),
        offered.contains(&Entity::Object(gate)),
    )
}

#[test]
fn traverse_the_ulvenwald_doesnt_count_back_faces_in_the_graveyard() {
    cr!("712.8a", "207.2c");
    ruling!(
        "Traverse the Ulvenwald",
        "Because you consider only the characteristics of a double-faced card’s front face while it’s not on the battlefield, the types of its back face won’t be counted for delirium."
    );
    supported("Traverse the Ulvenwald");
    // Search for Azcanta (enchantment // land) and Autumnal Gloom (enchantment //
    // creature), with an instant and an artifact: three card types.
    let three = [
        "Search for Azcanta",
        "Autumnal Gloom",
        "Lightning Bolt",
        "Mind Stone",
    ];
    assert_eq!(traverse(&three), (false, false));
    let mut four = three.to_vec();
    four.push("Forest");
    assert_eq!(traverse(&four), (true, true));
}

#[test]
fn traverse_the_ulvenwald_checks_as_it_resolves_and_gives_only_the_upgraded_effect() {
    cr!("608.2c", "207.2c");
    ruling!(
        "Traverse the Ulvenwald",
        "Some delirium abilities that appear on instants and sorceries use the word “instead.” These spells have an upgraded effect when they resolve if there are four or more card types among cards in your graveyard. They check that number only while they’re resolving and don’t count themselves, since they aren’t in your graveyard yet. You only get the upgraded effect, not both effects."
    );
    // Land, creature, and instant, plus the sorcery being cast: it doesn't count itself.
    assert_eq!(
        traverse(&["Forest", "Grizzly Bears", "Lightning Bolt"]),
        (false, false)
    );
    // With an artifact too: the upgraded search, and only one card.
    assert_eq!(
        traverse(&["Forest", "Grizzly Bears", "Lightning Bolt", "Mind Stone"]),
        (true, true)
    );
}

#[test]
fn traverse_the_ulvenwalds_delirium_search_can_find_a_nonbasic_land() {
    cr!("207.2c", "701.23a");
    ruling!(
        "Traverse the Ulvenwald",
        "The delirium ability of Traverse the Ulvenwald allows you to find a nonbasic land card."
    );
    let mut t = TestGame::new(2);
    delirium_graveyard(&mut t, P0);
    let gate = t.library_top(P0, "Azorius Guildgate");
    t.lands(P0, "Forest", 1);
    let card = t.hand(P0, "Traverse the Ulvenwald");
    t.cast(P0, card).go();
    t.answer_choose(P0, &[Entity::Object(gate)]);
    t.resolve();
    assert!(t.in_hand(P0, "Azorius Guildgate"));
    // Without delirium, it can't.
    assert_eq!(
        traverse(&["Forest", "Grizzly Bears", "Lightning Bolt"]).1,
        false
    );
}

#[test]
fn delirium_counts_card_types_not_cards() {
    cr!("207.2c", "205.2a");
    ruling!(
        "Traverse the Ulvenwald",
        "The number of card types matters, not the number of cards. For example, Epitaph Golem (an artifact creature) along with Memory Deluge (an instant) and Travel Preparations (a sorcery) will enable delirium."
    );
    assert_eq!(
        traverse(&["Epitaph Golem", "Memory Deluge", "Travel Preparations"]),
        (true, true)
    );
    // Many cards of fewer types don't.
    assert_eq!(
        traverse(&[
            "Grizzly Bears",
            "Hill Giant",
            "Lightning Bolt",
            "Shock",
            "Divination"
        ]),
        (false, false)
    );
}

#[test]
fn scour_the_laboratory_cast_from_the_graveyard_doesnt_count_itself() {
    cr!("601.2a", "601.2f", "207.2c", "702.34a");
    ruling!(
        "Scour the Laboratory",
        "If an effect allows you to cast Scour the Laboratory from your graveyard, count the number of card types among cards in your graveyard after moving Scour the Laboratory from your graveyard to the stack. If its delirium effect no longer applies, it costs {4}{U}{U} to cast."
    );
    supported("Scour the Laboratory");
    supported("Snapcaster Mage");
    for (with_bolt, lands, castable) in [(false, 4, false), (false, 6, true), (true, 4, true)] {
        let mut t = TestGame::new(2);
        // Scour (an instant) with a land, a creature, and an artifact: four card types
        // while it's in the graveyard, three once it's on the stack.
        let scour = t.graveyard(P0, "Scour the Laboratory");
        bury(&mut t, P0, &["Forest", "Grizzly Bears", "Mind Stone"]);
        if with_bolt {
            bury(&mut t, P0, &["Lightning Bolt"]);
        }
        t.answer_targets(P0, &[Entity::Object(scour)]);
        t.enter(P0, "Snapcaster Mage");
        t.resolve_all();
        t.lands(P0, "Island", lands);
        let r = t
            .cast(P0, scour)
            .method(CastMethod::Keyword(KeywordKind::Flashback))
            .try_go();
        assert_eq!(r.is_ok(), castable, "bolt {with_bolt}, {lands} lands");
        if castable {
            t.resolve_all();
            assert_eq!(t.hand_size(P0), 3);
        }
    }
}

#[test]
fn descend_upon_the_sinful_doesnt_count_auras_that_go_to_the_graveyard_afterward() {
    cr!("704.5m", "608.2c", "207.2c");
    ruling!(
        "Descend upon the Sinful",
        "If one of those creatures was enchanted, its Aura won't be put into a player's graveyard until after Descend upon the Sinful has finished resolving. If the controller of Descend upon the Sinful owned the Aura, it won't be in the graveyard in time to be counted for the delirium ability."
    );
    supported("Descend upon the Sinful");
    for aura_already_there in [false, true] {
        let mut t = TestGame::new(2);
        // Land, creature, and instant.
        bury(&mut t, P0, &["Forest", "Grizzly Bears", "Lightning Bolt"]);
        let giant = t.battlefield(P1, "Hill Giant");
        attach_new(&mut t, P0, "Pacifism", giant);
        if aura_already_there {
            bury(&mut t, P0, &["Pacifism"]);
        }
        let card = in_hand(&mut t, "Descend upon the Sinful");
        t.cast(P0, card).go();
        t.resolve_all();
        assert!(t.zone(giant) != Zone::Battlefield);
        assert!(t.in_graveyard(P0, "Pacifism"));
        assert_eq!(
            with_subtype(&t, P0, "Angel").len(),
            usize::from(aura_already_there)
        );
    }
}

fn in_hand(t: &mut TestGame, name: &str) -> ObjectId {
    give_mana_for(t, P0, name);
    t.hand(P0, name)
}

#[test]
fn mindwrack_demons_trigger_has_no_intervening_if_and_checks_on_resolution() {
    cr!("603.4", "608.2c", "207.2c");
    ruling!(
        "Mindwrack Demon",
        "Mindwrack Demon's delirium triggered ability does not include an intervening \"if\" clause. This ability triggers at the beginning of your upkeep regardless of the number of types in your graveyard, and it checks that number as it resolves to determine whether you lose 4 life or not."
    );
    supported("Mindwrack Demon");
    // No delirium as it triggers, delirium as it resolves: no life lost.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mindwrack Demon");
    bury(&mut t, P0, &["Forest", "Grizzly Bears", "Lightning Bolt"]);
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "you lose 4 life"), 1);
    bury(&mut t, P0, &["Mind Stone"]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // Delirium as it triggers, none as it resolves: 4 life lost.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mindwrack Demon");
    let cards = delirium_graveyard(&mut t, P0);
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "you lose 4 life"), 1);
    move_to(&mut t, cards[0], Zone::Exile);
    t.resolve_all();
    assert_eq!(t.life(P0), 16);
}

#[test]
fn bloodbraid_marauder_counts_permanents_sacrificed_while_casting_it() {
    cr!("601.2g", "601.2i", "702.85a", "207.2c");
    ruling!(
        "Bloodbraid Marauder",
        "The delirium ability will check your graveyard at the moment you finish casting Bloodbraid Marauder to determine if it has cascade. Notably, any permanents you sacrificed while casting the spell (such as a permanent you sacrificed to activate a mana ability) will be in the graveyard at that time."
    );
    supported("Bloodbraid Marauder");
    for petal in [false, true] {
        let mut t = TestGame::new(2);
        // Land, creature, and instant.
        bury(&mut t, P0, &["Forest", "Grizzly Bears", "Lightning Bolt"]);
        t.library_top(P0, "Shock");
        t.lands(P0, "Mountain", 1);
        if petal {
            // Lotus Petal ("{T}, Sacrifice this artifact: Add one mana of any color."),
            // sacrificed to pay for the spell, is an artifact in the graveyard by the time
            // it's cast.
            t.battlefield(P0, "Lotus Petal");
        } else {
            t.lands(P0, "Wastes", 1);
        }
        let card = t.hand(P0, "Bloodbraid Marauder");
        let spell = t.cast(P0, card).go();
        t.settle();
        t.g.recompute();
        assert_eq!(t.in_graveyard(P0, "Lotus Petal"), petal);
        assert_eq!(
            t.obj(spell).chars.has_keyword(KeywordKind::Cascade),
            petal,
            "petal {petal}"
        );
        assert_eq!(triggers_on_stack(&t, "Cascade"), usize::from(petal));
    }
}

#[test]
fn manic_scribe_mills_during_upkeep_before_the_draw() {
    cr!("503.1a", "504.1", "207.2c");
    ruling!(
        "Manic Scribe",
        "The upkeep step is before the draw step, after the untap step. Manic Scribe's delirium ability mills an opponent's library before that player draws a card during their draw step."
    );
    supported("Manic Scribe");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Manic Scribe");
    delirium_graveyard(&mut t, P0);
    let lib = t.library_size(P1);
    let hand = t.hand_size(P1);
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "mills three cards"), 1);
    t.resolve_all();
    assert_eq!(t.library_size(P1), lib - 3);
    assert_eq!(t.graveyard_size(P1), 3);
    assert_eq!(t.hand_size(P1), hand);
    t.advance_to(P1, Step::Draw);
    assert_eq!(t.library_size(P1), lib - 4);
    assert_eq!(t.hand_size(P1), hand + 1);
}
