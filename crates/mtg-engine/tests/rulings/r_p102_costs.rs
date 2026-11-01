//! Rulings batch P102 — alternative costs ("rather than pay this spell's mana cost"),
//! cards with no mana cost, costs paid while casting, and choices made on casting versus
//! on resolution: Salvage Titan, Daze, Mogg Salvage, Land Grant, Commandeer, Pyrokinesis,
//! Cabal Therapy, Nevinyrral's Disk, Pact of the Titan, Leyline of the Void, Evermind and
//! Asmoranomardicadaistinaculdacar.

use crate::r_p102_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn tapped(t: &TestGame, p: PlayerId) -> usize {
    crate::r_s01_common::tapped_lands(t, p)
}

#[test]
fn salvage_titan_alternative_cost_plus_increase_keeps_mana_value() {
    cr!("118.9c", "601.2f", "202.3");
    ruling!(
        "Salvage Titan",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying (such as Salvage Titan's alternative cost), add any cost increases"
    );
    supported("Salvage Titan");
    // P1's Aura of Silence: "Artifact and enchantment spells your opponents cast cost {2}
    // more to cast." Sacrificing three artifacts, P0 still pays {2}; mana value stays 6.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Aura of Silence");
    let thopters: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Ornithopter")).collect();
    t.lands(P0, "Swamp", 3);
    let titan = t.hand(P0, "Salvage Titan");
    let alt = alternatives(&mut t, P0, titan);
    assert_eq!(alt.len(), 1);
    choose(&mut t, P0, &thopters);
    let spell = t.cast(P0, titan).method(alt[0].clone()).go();
    assert!(thopters.iter().all(|id| in_gy(&t, *id)));
    assert_eq!(tapped(&t, P0), 2);
    assert_eq!(mana_value(&t, spell), 6);
    t.resolve_all();
    assert!(t.on_battlefield(titan));
    assert_eq!(mana_value(&t, titan), 6);
}

#[test]
fn daze_alternative_cost_plus_increase_keeps_mana_value() {
    cr!("118.9c", "601.2f", "202.3");
    ruling!(
        "Daze",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying (such as the alternative cost of Daze), add any cost increases"
    );
    supported("Daze");
    // P1's Thalia makes noncreature spells cost {1} more: Daze cast by returning an
    // Island also costs {1}. It's still mana value 2.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P1, "Mountain", 2);
    let shock = t.hand(P1, "Shock");
    let s = t.cast(P1, shock).target(Entity::Player(P0)).go();
    let island = t.battlefield(P0, "Island");
    t.battlefield(P0, "Plains");
    let daze = t.hand(P0, "Daze");
    let alt = alternatives(&mut t, P0, daze);
    assert_eq!(alt.len(), 1);
    choose(&mut t, P0, &[island]);
    let spell = t
        .cast(P0, daze)
        .method(alt[0].clone())
        .target(Entity::Object(s))
        .go();
    assert_eq!(t.zone(island), Zone::Hand(P0));
    // One mana paid Thalia's {1} (the Island may tap for it before it's returned).
    let paid = t.obj(spell).stack.as_ref().unwrap().cast.mana_spent.len();
    assert_eq!(paid, 1);
    assert_eq!(mana_value(&t, spell), 2);
    t.resolve();
    assert!(t.in_graveyard(P1, "Shock"), "P1 couldn't pay {{1}}");
}

#[test]
fn mogg_salvage_checks_lands_only_as_it_is_cast() {
    cr!("118.9", "601.2f", "608.2b");
    ruling!(
        "Mogg Salvage",
        "Who controls what lands is important only as you cast Mogg Salvage."
    );
    supported("Mogg Salvage");
    let mut t = TestGame::new(2);
    let isl = t.battlefield(P1, "Island");
    let mtn = t.battlefield(P0, "Mountain");
    let thopter = t.battlefield(P1, "Ornithopter");
    let ms = t.hand(P0, "Mogg Salvage");
    let alt = alternatives(&mut t, P0, ms);
    assert_eq!(alt.len(), 1);
    t.cast(P0, ms)
        .method(alt[0].clone())
        .target(Entity::Object(thopter))
        .go();
    assert_eq!(tapped(&t, P0), 0);
    // Both lands leave before it resolves: it still resolves.
    crate::r_s02_common::destroy(&mut t, isl);
    crate::r_s02_common::destroy(&mut t, mtn);
    t.resolve_all();
    assert!(in_gy(&t, thopter));
    // Without the Island the alternative cost isn't available.
    let ms2 = t.hand(P0, "Mogg Salvage");
    t.battlefield(P0, "Mountain");
    assert!(alternatives(&mut t, P0, ms2).is_empty());
}

#[test]
fn land_grant_reveal_cost_payable_when_hand_already_revealed() {
    cr!("118.9", "701.20a");
    ruling!(
        "Land Grant",
        "You can pay the “reveal your hand” cost even if your hand is already revealed due to another effect."
    );
    supported("Land Grant");
    // P1's Telepathy: "Your opponents play with their hands revealed."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Telepathy");
    let forest = t.library_top(P0, "Forest");
    t.hand(P0, "Grizzly Bears");
    let lg = t.hand(P0, "Land Grant");
    let alt = alternatives(&mut t, P0, lg);
    assert_eq!(alt.len(), 1);
    t.cast(P0, lg).method(alt[0].clone()).go();
    choose(&mut t, P0, &[forest]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Forest"));
    // With a land card in hand, it can't be cast this way.
    let mut t = TestGame::new(2);
    t.hand(P0, "Forest");
    let lg = t.hand(P0, "Land Grant");
    assert!(alternatives(&mut t, P0, lg).is_empty());
}

/// P1 casts Shock at P0 in P1's main phase; returns the spell.
fn p1_shock(t: &mut TestGame) -> ObjectId {
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(Entity::Player(P0)).go()
}

#[test]
fn commandeer_cant_exile_itself_to_pay_for_itself() {
    cr!("601.2a", "601.2h", "118.9");
    ruling!(
        "Commandeer",
        "You can't exile a card from your hand to pay for itself."
    );
    supported("Commandeer");
    let mut t = TestGame::new(2);
    let s = p1_shock(&mut t);
    let other = t.hand(P0, "Counterspell");
    let cmd = t.hand(P0, "Commandeer");
    // Only one other blue card: the cost can't be paid.
    let alt = alternatives(&mut t, P0, cmd);
    let ok = alt
        .first()
        .map(|m| {
            choose(&mut t, P0, &[other, cmd]);
            t.cast(P0, cmd)
                .method(m.clone())
                .target(Entity::Object(s))
                .try_go()
                .is_ok()
        })
        .unwrap_or(false);
    assert!(!ok);
    assert_eq!(t.zone(cmd), Zone::Hand(P0));
    assert_eq!(t.zone(other), Zone::Hand(P0));
    t.clear_answers();
    // With a second other blue card it works.
    let other2 = t.hand(P0, "Daze");
    let alt = alternatives(&mut t, P0, cmd);
    assert_eq!(alt.len(), 1);
    choose(&mut t, P0, &[other, other2]);
    t.answer_yes(P0, false);
    t.cast(P0, cmd)
        .method(alt[0].clone())
        .target(Entity::Object(s))
        .go();
    assert_eq!(t.zone(other), Zone::Exile);
    assert_eq!(t.zone(other2), Zone::Exile);
}

#[test]
fn commandeer_alternative_cost_and_additional_costs_are_paid_together() {
    cr!("118.9", "118.8", "601.2f", "702.56a");
    ruling!(
        "Commandeer",
        "You may pay the alternative cost rather than the card's mana cost. Any additional costs are paid as normal."
    );
    // P0's Djinn Illuminatus gives Commandeer replicate {5}{U}{U} (an additional cost, CR
    // 702.56a). Exiling two blue cards for the alternative cost and replicating once,
    // P0 pays both: the two cards and the seven mana of the replicate cost.
    let mut t = TestGame::new(2);
    let s = p1_shock(&mut t);
    t.battlefield(P0, "Djinn Illuminatus");
    t.lands(P0, "Island", 8);
    let a = t.hand(P0, "Counterspell");
    let b = t.hand(P0, "Daze");
    let cmd = t.hand(P0, "Commandeer");
    let alt = alternatives(&mut t, P0, cmd);
    assert_eq!(alt.len(), 1);
    choose(&mut t, P0, &[a, b]);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(1));
    t.cast(P0, cmd)
        .method(alt[0].clone())
        .target(Entity::Object(s))
        .go();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
    assert_eq!(tapped(&t, P0), 7, "the replicate cost {{5}}{{U}}{{U}}");
    // The replicate trigger copies it once.
    t.settle();
    assert_eq!(t.stack_len(), 3);
    for _ in 0..3 {
        t.resolve();
    }
    assert_eq!(t.g.stack, vec![s]);
    assert_eq!(t.obj_now(s).controller, P0);
}

#[test]
fn commandeer_may_change_targets_only_to_legal_ones() {
    cr!("115.7d", "608.2b");
    ruling!(
        "Commandeer",
        "You may change any of the targeted spell's targets. If you change a target, you must choose a legal target for the spell."
    );
    // P0 takes P1's Shock and points it at P1.
    let mut t = TestGame::new(2);
    let s = p1_shock(&mut t);
    lots(&mut t, P0);
    let cmd = t.hand(P0, "Commandeer");
    t.cast(P0, cmd).target(Entity::Object(s)).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20);
    // P1's Disenchant targets P0's only artifact, which then leaves: there's no legal new
    // target, so the target stays (illegal) and Disenchant does nothing.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let thopter = t.battlefield(P0, "Ornithopter");
    lots(&mut t, P1);
    let dis = t.hand(P1, "Disenchant");
    let d = t.cast(P1, dis).target(Entity::Object(thopter)).go();
    lots(&mut t, P0);
    let cmd = t.hand(P0, "Commandeer");
    t.cast(P0, cmd).target(Entity::Object(d)).go();
    t.answer_yes(P0, true);
    crate::r_s02_common::destroy(&mut t, thopter);
    t.resolve();
    assert_eq!(t.obj_now(d).controller, P0);
    assert_eq!(stack_targets(&t, d), vec![Entity::Object(thopter)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Disenchant"));
}

#[test]
fn pyrokinesis_divides_on_casting_and_keeps_the_division() {
    cr!("601.2d", "608.2b", "115.7");
    ruling!(
        "Pyrokinesis",
        "You divide the damage as you cast Pyrokinesis, not as it resolves."
    );
    supported("Pyrokinesis");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let c = t.battlefield(P1, "Hill Giant");
    t.hand(P0, "Lightning Bolt");
    let pyro = t.hand(P0, "Pyrokinesis");
    let alt = alternatives(&mut t, P0, pyro);
    assert_eq!(alt.len(), 1);
    t.answer_targets(P0, &[o(a), o(b), o(c)]);
    crate::r_s29_common::divide(&mut t, P0, &[2, 1, 1]);
    let from = t.asked().len();
    let spell = t.cast(P0, pyro).method(alt[0].clone()).go();
    assert!(t.in_exile("Lightning Bolt"));
    assert!(t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Divide { .. })));
    // The bears leave: the giants still get 1 each (not a new division).
    crate::r_s02_common::destroy(&mut t, a);
    let after = t.asked().len();
    t.resolve();
    assert!(!t.asked()[after..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Divide { .. })));
    assert!(!t.g.stack.contains(&spell));
    assert_eq!(t.obj_now(b).damage, 1);
    assert_eq!(t.obj_now(c).damage, 1);
    // All targets illegal: it doesn't resolve (no damage, no effect).
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    lots(&mut t, P0);
    let pyro = t.hand(P0, "Pyrokinesis");
    t.answer_targets(P0, &[o(a)]);
    t.cast(P0, pyro).go();
    crate::r_s02_common::destroy(&mut t, a);
    let from = t.log.len();
    t.resolve();
    assert!(t.in_graveyard(P0, "Pyrokinesis"));
    assert!(!t.log[from..].iter().any(|l| l.text.contains("damage")));
}

#[test]
fn cabal_therapy_names_the_card_on_resolution() {
    cr!("608.2", "608.2c", "701.9b");
    ruling!(
        "Cabal Therapy",
        "You name the card as Cabal Therapy resolves, not as you cast it."
    );
    supported("Cabal Therapy");
    let mut t = TestGame::new(2);
    t.hand(P1, "Shock");
    t.hand(P1, "Shock");
    t.hand(P1, "Grizzly Bears");
    t.battlefield(P0, "Swamp");
    let ct = t.hand(P0, "Cabal Therapy");
    let from = t.asked().len();
    t.cast(P0, ct).target(Entity::Player(P1)).go();
    let named = |t: &TestGame, from: usize| {
        t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::NameCard { .. }))
            .count()
    };
    assert_eq!(named(&t, from), 0, "nothing named while casting");
    t.answer(P0, DecisionKind::Name, Answer::Text("Shock".into()));
    t.resolve();
    assert_eq!(named(&t, from), 1);
    assert_eq!(t.hand_size(P1), 1);
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn nevinyrrals_disk_is_destroyed_on_resolution_unless_indestructible() {
    cr!("602.2", "608.2", "702.12b");
    ruling!(
        "Nevinyrral's Disk",
        "You don't sacrifice Nevinyrral's Disk to activate its ability. It's destroyed as part of the ability's resolution"
    );
    supported("Nevinyrral's Disk");
    let mut t = TestGame::new(2);
    let disk = t.battlefield(P0, "Nevinyrral's Disk");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    t.activate(P0, disk, 0, &[]).unwrap();
    assert!(t.on_battlefield(disk), "not a cost");
    t.resolve_all();
    assert!(in_gy(&t, disk));
    assert!(in_gy(&t, bears));
    // With indestructible (Darksteel Forge), the Disk stays.
    let mut t = TestGame::new(2);
    let disk = t.battlefield(P0, "Nevinyrral's Disk");
    let forge = t.battlefield(P0, "Darksteel Forge");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    t.activate(P0, disk, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(disk));
    assert!(t.on_battlefield(forge));
    assert!(in_gy(&t, bears));
}

#[test]
fn nevinyrrals_disk_ruling_on_mogg_salvage_page() {
    cr!("602.2", "608.2");
    ruling!(
        "Mogg Salvage",
        "You don't sacrifice Nevinyrral's Disk to activate its ability."
    );
    // Mogg Salvage destroying the Disk in response doesn't stop the ability; the ability
    // then finds no Disk to destroy but destroys everything else.
    let mut t = TestGame::new(2);
    let disk = t.battlefield(P1, "Nevinyrral's Disk");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Wastes", 1);
    t.activate(P1, disk, 0, &[]).unwrap();
    t.battlefield(P1, "Island");
    t.battlefield(P0, "Mountain");
    let ms = t.hand(P0, "Mogg Salvage");
    let alt = alternatives(&mut t, P0, ms);
    t.cast(P0, ms)
        .method(alt[0].clone())
        .target(Entity::Object(disk))
        .go();
    t.resolve();
    assert!(in_gy(&t, disk));
    t.resolve_all();
    assert!(in_gy(&t, bears));
}

#[test]
fn pact_of_the_titan_must_be_paid_even_without_the_token() {
    cr!("603.7", "104.3a");
    ruling!(
        "Pact of the Titan",
        "You have to pay Pact of the Titan's cost during your next upkeep even if the Giant token doesn't survive that long."
    );
    supported("Pact of the Titan");
    for pay in [false, true] {
        let mut t = TestGame::new(2);
        let pact = t.hand(P0, "Pact of the Titan");
        t.cast(P0, pact).go();
        t.resolve_all();
        let giant = crate::r_s01_common::with_subtype(&t, P0, "Giant")[0];
        crate::r_s02_common::destroy(&mut t, giant);
        if pay {
            t.lands(P0, "Mountain", 5);
        }
        t.answer_yes(P0, pay);
        t.advance_to(P1, Step::Upkeep);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        assert_eq!(t.has_lost(P0), !pay, "paid: {pay}");
    }
}

#[test]
fn leyline_of_the_void_exiles_dying_creatures_without_dies_triggers() {
    cr!("614.1a", "614.6", "700.4", "603.6c");
    ruling!(
        "Leyline of the Void",
        "While Leyline of the Void is on the battlefield, nontoken creatures your opponents control won't die."
    );
    supported("Leyline of the Void");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of the Void");
    t.battlefield(P0, "Blood Artist");
    let dt = t.battlefield(P1, "Doomed Traveler");
    crate::r_s02_common::destroy(&mut t, dt);
    t.resolve_all();
    assert_eq!(t.zone(dt), Zone::Exile);
    assert!(crate::r_s01_common::with_subtype(&t, P1, "Spirit").is_empty());
    assert_eq!(t.life(P1), 20, "Blood Artist didn't trigger");
    assert_eq!(t.life(P0), 20);
}

#[test]
fn evermind_has_no_mana_cost() {
    cr!("118.6", "202.1b", "202.3a", "105.2");
    ruling!(
        "Evermind",
        "Evermind has no mana cost, which means it can't normally be cast as a spell."
    );
    ruling!(
        "Evermind",
        "If a spell or effect, such as Parallectric Feedback, asks for Evermind's mana value, it's 0."
    );
    ruling!("Evermind", "Evermind is blue in all zones.");
    supported("Evermind");
    let mut t = TestGame::new(2);
    lots(&mut t, P0);
    let ev = t.hand(P0, "Evermind");
    assert!(!castable_normally(&mut t, P0, ev));
    assert_eq!(mana_value(&t, ev), 0);
    for zone in ["hand", "library", "graveyard", "exile", "battlefield"] {
        let id = match zone {
            "hand" => ev,
            "library" => t.library_top(P0, "Evermind"),
            "graveyard" => t.graveyard(P0, "Evermind"),
            "exile" => t.exile(P0, "Evermind"),
            _ => continue,
        };
        assert_eq!(
            t.obj(id).chars.colors,
            ColorSet::single(Color::Blue),
            "{zone}"
        );
    }
    // An alternate means (Omniscience: "without paying their mana costs") casts it.
    t.battlefield(P0, "Omniscience");
    assert!(crate::r_s07_common::cast_methods(&mut t, P0, ev).contains(&CastMethod::Free));
    let lib = t.library_size(P0);
    let spell = t.cast(P0, ev).method(CastMethod::Free).go();
    assert!(t.obj(spell).is(CardType::Instant));
    assert!(t.obj(spell).chars.has_subtype("Arcane"));
    t.resolve();
    assert_eq!(t.library_size(P0), lib - 1);
}

#[test]
fn evermind_is_a_blue_card_for_force_of_will() {
    cr!("118.9", "105.2");
    ruling!(
        "Evermind",
        "Evermind is a blue Arcane instant card. Cards that check whether a card has any of those attributes will affect Evermind."
    );
    let mut t = TestGame::new(2);
    let s = p1_shock(&mut t);
    let ev = t.hand(P0, "Evermind");
    let fow = t.hand(P0, "Force of Will");
    let alt = alternatives(&mut t, P0, fow);
    assert_eq!(alt.len(), 1);
    choose(&mut t, P0, &[ev]);
    t.cast(P0, fow)
        .method(alt[0].clone())
        .target(Entity::Object(s))
        .go();
    assert_eq!(t.zone(ev), Zone::Exile);
    assert_eq!(t.life(P0), 19, "and 1 life");
    t.resolve_all();
    assert_eq!(t.life(P0), 19, "Shock was countered");
}

#[test]
fn asmoranomardicadaistinaculdacar_needs_a_discard_from_elsewhere() {
    cr!("118.9", "202.1b", "601.2f");
    ruling!(
        "Asmoranomardicadaistinaculdacar",
        "Asmoranomardicadaistinaculdacar doesn't allow you to discard cards."
    );
    ruling!(
        "Asmoranomardicadaistinaculdacar",
        "Asmoranomardicadaistinaculdacar has no mana cost and its mana value is 0. It cannot be cast normally."
    );
    let mut t = TestGame::new(2);
    t.g.players[0].mana_pool.add_type(ManaType::B, 1);
    let shock = t.hand(P0, "Shock");
    let asmor = t.hand(P0, "Asmoranomardicadaistinaculdacar");
    assert_eq!(mana_value(&t, asmor), 0);
    assert!(!castable_normally(&mut t, P0, asmor));
    assert!(alternatives(&mut t, P0, asmor).is_empty());
    t.g.discard(P0, shock, None);
    t.g.flush_events();
    let alt = alternatives(&mut t, P0, asmor);
    assert_eq!(alt.len(), 1);
    let spell = t.cast(P0, asmor).method(alt[0].clone()).go();
    assert_eq!(mana_value(&t, spell), 0);
    t.resolve_all();
    assert!(t.on_battlefield(asmor));
}

#[test]
fn salvage_titan_returns_only_from_the_graveyard_and_not_if_exiled_for_the_cost() {
    cr!("113.6k", "602.2", "400.7");
    ruling!(
        "Salvage Titan",
        "You may activate its second ability only if Salvage Titan is in your graveyard."
    );
    // On the battlefield, with three artifact cards in the graveyard: not activatable.
    let mut t = TestGame::new(2);
    let titan = t.battlefield(P0, "Salvage Titan");
    for _ in 0..3 {
        t.graveyard(P0, "Ornithopter");
    }
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, titan));
    // In the graveyard with two other artifact cards: it may exile itself to pay the
    // cost, and then it isn't returned.
    let mut t = TestGame::new(2);
    let titan = t.graveyard(P0, "Salvage Titan");
    let a = t.graveyard(P0, "Ornithopter");
    let b = t.graveyard(P0, "Ornithopter");
    assert!(crate::r_s02_common::can_activate(&mut t, P0, titan));
    choose(&mut t, P0, &[titan, a, b]);
    t.activate(P0, titan, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(titan), Zone::Exile);
    assert!(!t.in_hand(P0, "Salvage Titan"));
    // With three other artifact cards it returns.
    let mut t = TestGame::new(2);
    let titan = t.graveyard(P0, "Salvage Titan");
    let others: Vec<ObjectId> = (0..3).map(|_| t.graveyard(P0, "Ornithopter")).collect();
    choose(&mut t, P0, &others);
    t.activate(P0, titan, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Salvage Titan"));
    assert!(others.iter().all(|id| t.zone(*id) == Zone::Exile));
}
