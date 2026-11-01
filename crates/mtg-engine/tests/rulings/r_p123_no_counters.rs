//! Rulings batch P123 — "can't have counters put on it" (Tatterkite, Melira's Keepers)
//! and Solemnity ("Players can't get counters. Counters can't be put on artifacts,
//! creatures, enchantments, or lands."): counters that would be put are simply not put
//! (CR 614.1, 122.6), including those a permanent would enter with (CR 614.12, 122.6);
//! infect and wither damage then has no effect (CR 120.3, 702.80, 702.90); costs that put
//! counters can't be paid (CR 118.3).

use crate::r_p123_common::*;
use mtg_engine::ability::Modification;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn creature(name: &str, text: &str) -> mtg_engine::card::CardDef {
    custom_card(name, "Creature — Horror", "{2}{B}", Some((2, 2)), text)
}

/// The total number of counters of any kind on the object now.
fn all_counters(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).counters.values().sum()
}

#[test]
fn cant_have_any_kind_of_counter_put_on_it() {
    cr!("122.1", "614.1");
    ruling!(
        "Melira's Keepers",
        "Melira's Keepers can't have any kind of counters placed on it, not just -1/-1 counters."
    );
    ruling!(
        "Tatterkite",
        "Tatterkite's ability prevents any kind of counter from being placed on it, not just -1/-1 counters."
    );
    for name in ["Melira's Keepers", "Tatterkite"] {
        supported(name);
        let mut t = TestGame::new(2);
        let id = t.battlefield(P0, name);
        for kind in ["+1/+1", "-1/-1", "charge", "shield", "stun"] {
            put_counters_now(&mut t, id, kind);
        }
        assert_eq!(all_counters(&t, id), 0, "{name}");
    }
}

fn put_counters_now(t: &mut TestGame, id: ObjectId, kind: &str) {
    crate::r_p108_common::put_counters(t, id, kind, 1);
    t.settle();
}

#[test]
fn a_spell_putting_counters_still_affects_the_others() {
    cr!("614.1", "608.2c");
    ruling!(
        "Tatterkite",
        "A spell or ability whose effect would put a counter on Tatterkite simply doesn't affect it. The spell or ability isn't countered. If it would put counters on any other creatures, it continues to do so."
    );
    supported("Travel Preparations");
    let mut t = TestGame::new(2);
    let kite = t.battlefield(P0, "Tatterkite");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resolve_at(&mut t, P0, "Travel Preparations", &[obj(kite), obj(bears)]);
    assert_eq!(plus1(&t, kite), 0);
    assert_eq!(plus1(&t, bears), 1);
    assert!(t.in_graveyard(P0, "Travel Preparations"));
}

#[test]
fn entering_with_counters_enters_without_them() {
    cr!("614.12", "122.6", "702.79a");
    ruling!(
        "Melira's Keepers",
        "If Melira's Keepers would enter with a counter on it, it simply enters without that counter."
    );
    ruling!(
        "Tatterkite",
        "If Tatterkite would enter with counters on it, it enters but doesn't have those counters. For example, if an effect grants it persist and then it's destroyed, it returns to the battlefield without a -1/-1 counter on it."
    );
    ruling!(
        "Solemnity",
        "Solemnity stops counters from being put on an artifact, creature, enchantment, or land as it enters the battlefield, as well as stopping counters from being put on them later."
    );
    let mut t = TestGame::new(2);
    let keepers = crate::r_p119_common::enter_with(&mut t, P0, "Melira's Keepers", 2, false);
    assert!(t.on_battlefield(keepers));
    assert_eq!(all_counters(&t, keepers), 0);
    // Tatterkite granted persist returns without a -1/-1 counter, so it persists again.
    let mut t = TestGame::new(2);
    let kite = t.battlefield(P0, "Tatterkite");
    let persist = Modification::AddKeyword(Keyword::new(KeywordKind::Persist));
    modify_until_eot(&mut t, kite, vec![persist]);
    destroy(&mut t, kite);
    t.resolve_all();
    let back = t.named_on_battlefield("Tatterkite");
    assert_eq!(back.len(), 1, "returned by persist");
    assert_eq!(all_counters(&t, back[0]), 0);
    // Under Solemnity: Spike Feeder enters without its counters; Kitchen Finks persists
    // without a counter; nothing gets counters later.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Solemnity");
    let feeder = t.enter(P0, "Spike Feeder");
    t.settle();
    assert_eq!(all_counters(&t, feeder), 0);
    assert!(!t.on_battlefield(feeder), "a 0/0 dies");
    let finks = t.battlefield(P0, "Kitchen Finks");
    destroy(&mut t, finks);
    t.resolve_all();
    let finks = t.named_on_battlefield("Kitchen Finks");
    assert_eq!(finks.len(), 1);
    assert_eq!(all_counters(&t, finks[0]), 0);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters_now(&mut t, bears, "+1/+1");
    assert_eq!(all_counters(&t, bears), 0);
}

#[test]
fn solemnity_doesnt_stop_counters_on_permanents_entering_with_it() {
    cr!("614.12", "122.6");
    ruling!(
        "Solemnity",
        "If an artifact, creature, enchantment, or land would enter the battlefield with counters on it at the same time that Solemnity enters the battlefield, Solemnity doesn’t stop it from getting those counters."
    );
    supported("Solemnity");
    supported("Spike Feeder");
    let mut t = TestGame::new(2);
    let ids = crate::r_p116_common::enter_together(&mut t, P0, &["Solemnity", "Spike Feeder"]);
    t.settle();
    assert_eq!(plus1(&t, ids[1]), 2);
}

#[test]
fn solemnity_doesnt_remove_counters() {
    cr!("122.1");
    ruling!(
        "Solemnity",
        "Solemnity doesn’t remove any counters players or permanents already have."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, bears, 2);
    t.g.add_counters(Entity::Player(P1), "poison", 3, None);
    t.battlefield(P0, "Solemnity");
    t.settle();
    assert_eq!(plus1(&t, bears), 2);
    assert_eq!(t.g.player(P1).poison(), 3);
}

#[test]
fn solemnity_suspended_cards_still_get_time_counters() {
    cr!("122.1", "702.62a");
    ruling!(
        "Solemnity",
        "Counters can be put on cards that aren't on the battlefield. Notably, suspended cards will still get time counters."
    );
    use mtg_engine::decision::{Action, SpecialAction};
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Solemnity");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Rift Bolt");
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, Action::Special(SpecialAction::Suspend { card: bolt }))
        .expect("suspended");
    let exiled = t.g.current(bolt);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert_eq!(t.counters(exiled, "time"), 1);
}

#[test]
fn infect_and_wither_damage_has_no_effect_but_is_dealt() {
    cr!("120.3b", "120.3d", "702.15b", "614.1");
    ruling!(
        "Melira's Keepers",
        "Damage from a source with infect has no effect on Melira's Keepers. No -1/-1 counters are placed on it, and no damage is marked on it. The damage is still dealt for purposes of effects that care about damage, such as lifelink."
    );
    ruling!(
        "Solemnity",
        "Damage from a source with infect has no effect on creatures or players. No -1/-1 counters are put on creatures, and no damage is marked on them. Players don’t get poison counters and they don’t lose life. The damage is still dealt for purposes of effects that care about damage, such as lifelink."
    );
    ruling!(
        "Solemnity",
        "Damage from a source with wither has no effect on creatures. No -1/-1 counters are put on them, and no damage is marked on them. The damage is still dealt for purposes of effects that care about damage, such as lifelink."
    );
    ruling!(
        "Tatterkite",
        "If Tatterkite is dealt damage by a creature with wither, that damage has no effect. It's not prevented; it just doesn't do anything. Abilities that trigger on damage being dealt will still trigger."
    );
    let infect = creature("Infect Leech", "Infect\nLifelink");
    let wither = creature(
        "Wither Leech",
        "Wither\nLifelink\nWhenever this creature deals damage, draw a card.",
    );
    // Melira's Keepers and infect.
    let mut t = TestGame::new(2);
    let leech = t.custom(P1, infect.clone(), Zone::Battlefield);
    let keepers = t.battlefield(P0, "Melira's Keepers");
    t.g.deal_damage(leech, obj(keepers), 2, false);
    t.settle();
    assert_eq!((all_counters(&t, keepers), t.obj_now(keepers).damage), (0, 0));
    assert_eq!(t.life(P1), 22, "lifelink");
    // Solemnity: infect to a creature and to a player; wither to a creature.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Solemnity");
    let leech = t.custom(P1, infect, Zone::Battlefield);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.deal_damage(leech, obj(bears), 2, false);
    t.settle();
    assert_eq!((all_counters(&t, bears), t.obj_now(bears).damage), (0, 0));
    t.g.deal_damage(leech, Entity::Player(P0), 2, false);
    t.settle();
    assert_eq!((t.g.player(P0).poison(), t.life(P0)), (0, 20));
    assert_eq!(t.life(P1), 24, "lifelink twice");
    let wl = t.custom(P1, wither.clone(), Zone::Battlefield);
    t.g.deal_damage(wl, obj(bears), 2, false);
    t.settle();
    assert_eq!((all_counters(&t, bears), t.obj_now(bears).damage), (0, 0));
    assert_eq!(t.life(P1), 26);
    // Tatterkite and wither: the damage-dealt trigger still triggers.
    let mut t = TestGame::new(2);
    let kite = t.battlefield(P0, "Tatterkite");
    let wl = t.custom(P1, wither, Zone::Battlefield);
    let hand = t.hand_size(P1);
    t.g.deal_damage(wl, obj(kite), 2, false);
    t.settle();
    assert!(t.on_battlefield(kite));
    assert_eq!((all_counters(&t, kite), t.obj_now(kite).damage), (0, 0));
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 1, "the damage trigger triggered");
    assert_eq!(t.life(P1), 22);
}

#[test]
fn a_copy_of_melira_keepers_keeps_its_counters() {
    cr!("707.2", "122.1");
    ruling!(
        "Melira's Keepers",
        "If a creature with a counter on it becomes a copy of Melira's Keepers, those counters will remain."
    );
    let mut t = TestGame::new(2);
    let keepers = t.battlefield(P0, "Melira's Keepers");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, bears, 2);
    crate::r_s17_common::become_copy(&mut t, bears, keepers);
    assert_eq!(t.obj_now(bears).name(), "Melira's Keepers");
    assert_eq!(plus1(&t, bears), 2);
    assert_eq!(t.pt(bears), (6, 6));
}

#[test]
fn solemnity_tribute_cant_be_paid() {
    cr!("702.104a", "614.1");
    ruling!(
        "Solemnity",
        "If a creature with tribute is entering the battlefield, the chosen opponent can’t pay tribute even if they want to."
    );
    supported("Fanatic of Xenagos");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Solemnity");
    t.answer_yes(P1, true);
    t.answer_yes(P1, true);
    let fanatic = t.enter(P0, "Fanatic of Xenagos");
    t.resolve_all();
    assert_eq!(all_counters(&t, fanatic), 0);
    // Tribute wasn't paid: its "if tribute wasn't paid" ability happened.
    assert_eq!(t.pt(fanatic), (4, 4));
    assert!(t.obj_now(fanatic).has_keyword(KeywordKind::Haste));
    // Likewise riot offers no counter: the creature gains haste (Frenzied Arynx).
    t.answer_yes(P0, true);
    let arynx = t.enter(P0, "Frenzied Arynx");
    t.settle();
    assert_eq!(all_counters(&t, arynx), 0);
    assert!(t.obj_now(arynx).has_keyword(KeywordKind::Haste));
}

#[test]
fn solemnity_and_a_replacement_effect_that_puts_counters() {
    cr!("614.1", "616.1");
    ruling!(
        "Solemnity",
        "If a replacement effect allows a player to modify or replace an event by putting counters on an artifact, creature, enchantment, or land, that player may apply that replacement effect."
    );
    supported("Soul-Scar Mage");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Solemnity");
    t.battlefield(P0, "Soul-Scar Mage");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_resolve_at(&mut t, P0, "Shock", &[obj(bears)]);
    // The damage was replaced by -1/-1 counters, which weren't put: nothing happened.
    assert!(t.on_battlefield(bears));
    assert_eq!((all_counters(&t, bears), t.obj_now(bears).damage), (0, 0));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn costs_that_put_counters_cant_be_paid() {
    cr!("118.3", "601.2h");
    ruling!(
        "Solemnity",
        "If the cost of an ability or an additional cost of a spell requires putting counters on an artifact, creature, enchantment, or land, that cost can’t be paid."
    );
    ruling!(
        "Tatterkite",
        "You can't pay a cost that includes putting a counter on Tatterkite. For example, if Tatterkite is the only creature you control, you can't cast Scarscale Ritual because you can't pay the additional cost."
    );
    supported("Scarscale Ritual");
    let ritual = |t: &mut TestGame| {
        lands_for_cost(t, P0, "Scarscale Ritual");
        let card = t.hand(P0, "Scarscale Ritual");
        can_cast(t, P0, card, CastMethod::Normal)
    };
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tatterkite");
    assert!(!ritual(&mut t), "Tatterkite is the only creature");
    t.battlefield(P0, "Grizzly Bears");
    assert!(ritual(&mut t), "the Bears can take the counter");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Solemnity");
    t.battlefield(P0, "Grizzly Bears");
    assert!(!ritual(&mut t), "Solemnity");
    // "You may put a counter on ...": the player can't choose to do so.
    let def = custom_card(
        "Optional Growth",
        "Instant",
        "{G}",
        None,
        "You may put a +1/+1 counter on target creature. If you do, draw a card.",
    );
    let bears = t.battlefield(P0, "Grizzly Bears");
    let card = t.custom(P0, def, Zone::Hand(P0));
    t.lands(P0, "Forest", 1);
    t.answer_yes(P0, true);
    t.cast(P0, card).target(bears).go();
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!((plus1(&t, bears), t.hand_size(P0)), (0, hand));
}

#[test]
fn solemnity_cumulative_upkeep_pays_for_no_age_counters() {
    cr!("702.24a", "614.1");
    ruling!(
        "Solemnity",
        "While resolving a cumulative upkeep trigger of a permanent, you’ll fail to put a counter on that permanent, then you may pay for the age counters already on it. If it has no age counters on it, you may pay {0}."
    );
    supported("Mystic Remora");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Solemnity");
    let remora = t.battlefield(P0, "Mystic Remora");
    for _ in 0..2 {
        t.answer_yes(P0, true);
        crate::r_s04_common::next_upkeep(&mut t, P0);
        t.resolve_all();
        assert!(t.on_battlefield(remora), "paid {{0}}");
        assert_eq!(t.counters(remora, "age"), 0);
    }
    // With an age counter already on it: pay {1}.
    t.g.add_counters(Entity::Object(remora), "age", 1, None);
    assert_eq!(t.counters(remora, "age"), 0, "Solemnity stops this too");
}
