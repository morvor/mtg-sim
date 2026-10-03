//! In-game tests for compiler misreads found by the Oracle round trip (follow-up item
//! `roundtrip-clusters-2`): each shows the behavior the corrected compilation has and
//! the old one didn't.

use mtg_engine::card::card;
use mtg_engine::testing::*;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn a_search_for_up_to_x_cards_may_find_fewer() {
    cr!("701.23d");
    // Diabolic Revelation: "Search your library for up to X cards, put those cards into
    // your hand, then shuffle." "Up to" was dropped, and a search for a quantity of cards
    // must find that many (CR 701.23d): the player had to take X cards.
    supported("Diabolic Revelation");
    let mut t = TestGame::new(2);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    let wanted = t.library_top(P0, "Grizzly Bears");
    for _ in 0..4 {
        t.library_top(P0, "Forest");
    }
    let spell = t.hand(P0, "Diabolic Revelation");
    t.lands(P0, "Swamp", 8);
    t.answer_choose(P0, &[Entity::Object(wanted)]);
    t.cast(P0, spell).x(3).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn yggdrasil_gives_haste_to_the_creature_it_puts_onto_the_battlefield() {
    cr!("400.7", "607.2a");
    // "Put a creature card exiled with ~ onto the battlefield under your control. It
    // gains haste until end of turn.": "it" is the creature put onto the battlefield; it
    // was compiled as Yggdrasil itself.
    supported("Yggdrasil, Rebirth Engine");
    let mut t = TestGame::new(2);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    t.graveyard(P0, "Grizzly Bears");
    let ygg = t.enter(P0, "Yggdrasil, Rebirth Engine");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    t.lands(P0, "Forest", 4);
    t.activate(P0, ygg, 1, &[]).expect("activate");
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    use mtg_engine::keywords::KeywordKind;
    assert!(t.obj_now(bears[0]).has_keyword(KeywordKind::Haste));
    assert!(!t.obj_now(ygg).has_keyword(KeywordKind::Haste));
}

#[test]
fn acolyte_of_the_inferno_deals_the_damage_to_its_blocker() {
    cr!("509.3d", "702.2b");
    // "Whenever ~ becomes blocked by a creature, it deals 2 damage to that creature.": "it"
    // is Acolyte; it was compiled as the blocker, which dealt the damage to itself. A
    // deathtouch blocker then destroyed itself before combat damage, and Acolyte lived.
    supported("Acolyte of the Inferno");
    let mut t = TestGame::new(2);
    let acolyte = t.battlefield(P0, "Acolyte of the Inferno");
    let hawk = t.battlefield(P1, "Vampire Nighthawk");
    t.attack(&[(acolyte, Entity::Player(P1))], &[(hawk, acolyte)]);
    // The trigger's 2 damage and 3 combat damage kill Nighthawk; its deathtouch combat
    // damage kills Acolyte.
    assert!(t.in_graveyard(P1, "Vampire Nighthawk"));
    assert!(t.in_graveyard(P0, "Acolyte of the Inferno"));
}

#[test]
fn the_one_ring_saga_mills_each_player_by_your_ring_bearers_power() {
    cr!("701.54e");
    // "Each player mills cards equal to your Ring-bearer's power.": the controller's
    // Ring-bearer; each player's own Ring-bearer was used, so an opponent without one
    // milled nothing.
    supported("One Ring to Rule Them All");
    let mut t = TestGame::new(2);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    t.battlefield(P0, "Hill Giant");
    let (l0, l1) = (t.library_size(P0), t.library_size(P1));
    t.enter(P0, "One Ring to Rule Them All");
    t.resolve_all();
    assert_eq!(t.library_size(P1), l1 - 3);
    assert_eq!(t.library_size(P0), l0 - 3);
}

#[test]
fn during_turns_other_than_yours_includes_a_teammates_turn() {
    cr!("102.3", "500.1");
    // Mesa Lynx: "~ gets +0/+2 during turns other than yours." It applied only while an
    // opponent was the active player, so not during a teammate's turn.
    supported("Mesa Lynx");
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            teams: Some(vec![0, 1, 0, 1]),
            ..Default::default()
        },
    );
    let lynx = t.battlefield(P0, "Mesa Lynx");
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    assert_eq!(t.pt(lynx), (2, 1));
    // P2 is P0's teammate.
    t.set_step(P2, mtg_engine::turn::Step::PrecombatMain);
    assert_eq!(t.pt(lynx), (2, 3));
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    assert_eq!(t.pt(lynx), (2, 3));
}

#[test]
fn patrician_geist_doesnt_reduce_spells_cast_from_another_players_graveyard() {
    cr!("404.1", "601.2f");
    // "Spells you cast from your graveyard cost {1} less to cast.": a spell cast from any
    // graveyard was reduced (a card cast from another player's graveyard, as Quistis Trepe
    // allows, isn't from yours).
    supported("Patrician Geist");
    let mut t = TestGame::new(2);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    t.battlefield(P0, "Patrician Geist");
    let in_hand = t.hand(P0, "Divination");
    let mine = t.graveyard(P0, "Divination");
    let theirs = t.graveyard(P1, "Divination");
    t.lands(P0, "Island", 3);
    t.g.turn.priority = Some(P0);
    let opts = t.g.cast_options(P0, in_hand);
    let cost = |t: &TestGame, card| {
        let chars = t.obj(card).chars.clone();
        let c = t.g.base_total_cost(P0, card, &chars, &opts[0], 0);
        c.mana.map(|m| m.mana_value())
    };
    assert_eq!(cost(&t, in_hand), Some(3));
    assert_eq!(cost(&t, mine), Some(2));
    assert_eq!(cost(&t, theirs), Some(3));
}

#[test]
fn feast_of_dreams_can_target_an_unenchanted_enchantment_creature() {
    cr!("115.1", "205.2a");
    // "Destroy target enchanted creature or enchantment creature.": it was read as
    // "enchanted (creature or enchantment) creature", leaving out enchantment creatures
    // that aren't enchanted.
    supported("Feast of Dreams");
    let mut t = TestGame::new(2);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    let ram = t.battlefield(P1, "Nyx-Fleece Ram");
    t.lands(P0, "Swamp", 2);
    let spell = t.hand(P0, "Feast of Dreams");
    t.cast(P0, spell).target(Entity::Object(ram)).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Nyx-Fleece Ram"));
}
