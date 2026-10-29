//! Rulings batch P098 — returning low-mana-value cards from graveyards and libraries:
//! illegal targets (General Leo Cristophe, Revoke Demise, Unforgiving One), Auras put onto
//! the battlefield without targeting (Shepherd of the Cosmos, Annie Flash, Matter
//! Reshaper), Call of the Death-Dweller's counters, Crypt Champion, Carnage's attack
//! requirement, Extraction Specialist, Kinscaer Sentry and Rebel searches.

use crate::r_s01_common::*;
use crate::r_s02_common::{can_attack, destroy, target_candidates};
use crate::r_s09_common::legal_attack;
use crate::r_s20_common::to_beginning_of_combat;
use crate::r_s21_common::legal_blocks;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Moves a card out of its graveyard (to exile), making it an illegal target.
fn exile_card(t: &mut TestGame, id: ObjectId) {
    crate::r_s05_common::move_to(t, id, Zone::Exile);
}

#[test]
fn returning_an_illegal_target_does_nothing_else() {
    cr!("608.2b");
    ruling!(
        "General Leo Cristophe",
        "If the target creature card is an illegal target as General Leo Cristophe's ability tries to resolve, it won't resolve and none of its effects will happen. You won't put +1/+1 counters on General Leo Cristophe."
    );
    ruling!(
        "Revoke Demise",
        "If the target creature card is an illegal target as Revoke Demise tries to resolve, it won't resolve and none of its effects will happen. You won't gain life."
    );
    supported("General Leo Cristophe");
    supported("Revoke Demise");
    for illegal in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Hill Giant");
        let bears = t.graveyard(P0, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        let leo = t.enter(P0, "General Leo Cristophe");
        t.settle();
        if illegal {
            exile_card(&mut t, bears);
        }
        t.resolve_all();
        // Leo, Hill Giant and the Bears: three counters; none if it didn't resolve.
        assert_eq!(
            t.counters(leo, counters::PLUS1),
            if illegal { 0 } else { 3 }
        );

        let mut t = TestGame::new(2);
        let bears = t.graveyard(P0, "Grizzly Bears");
        let spell = t.hand(P0, "Revoke Demise");
        t.lands(P0, "Swamp", 3);
        t.cast(P0, spell).target(bears).go();
        if illegal {
            exile_card(&mut t, bears);
        }
        t.resolve_all();
        assert_eq!(t.life(P0), if illegal { 20 } else { 22 });
    }
}

#[test]
fn unforgiving_one_x_is_checked_again_on_resolution() {
    cr!("608.2b", "601.2c");
    ruling!(
        "Unforgiving One",
        "The value of X needs to be determined both when the ability triggers (so you can choose a target) and again when the ability resolves (to check if that target is still legal). If the number of modified creatures you control has decreased enough in that time to make the target illegal, Unforgiving One's ability won't resolve."
    );
    supported("Unforgiving One");
    for decreased in [false, true] {
        let mut t = TestGame::new(2);
        let one = t.battlefield(P0, "Unforgiving One");
        let other = t.battlefield(P0, "Hill Giant");
        for id in [one, other] {
            t.g.add_counters(obj(id), counters::PLUS1, 1, None);
        }
        let bears = t.graveyard(P0, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        attack_with(&mut t, &[(one, Entity::Player(P1))]);
        assert_eq!(triggers_on_stack(&t, "return target creature card"), 1);
        if decreased {
            t.g.remove_counters(obj(other), counters::PLUS1, 1);
        }
        t.resolve_all();
        assert_eq!(
            t.in_graveyard(P0, "Grizzly Bears"),
            decreased,
            "{decreased}"
        );
    }
}

#[test]
fn an_aura_returned_without_targeting_may_enchant_hexproof_but_not_protection() {
    cr!("303.4f", "303.4d", "702.11b");
    ruling!(
        "Shepherd of the Cosmos",
        "If you return an Aura this way, you choose what the Aura will enchant just before it enters the battlefield. An Aura returning to the battlefield this way doesn’t target the player or permanent it will enchant, so permanents or players with hexproof may be chosen; however, the chosen recipient must be able to legally be enchanted by the Aura, so a player or permanent with protection from one of the Aura’s qualities can’t be chosen this way. If there’s nothing legal for the Aura to enchant, it stays in the graveyard."
    );
    ruling!(
        "Annie Flash, the Veteran",
        "If you return an Aura this way, you choose what the Aura will enchant just before it enters the battlefield. An Aura returning to the battlefield this way doesn't target the player or permanent it will enchant, so players or permanents with hexproof may be chosen; however, the chosen recipient must be able to legally be enchanted by the Aura, so a player or permanent with protection from one of the Aura's qualities can't be chosen this way. If there's nothing legal for the Aura to enchant, it stays in the graveyard."
    );
    supported("Shepherd of the Cosmos");
    supported("Annie Flash, the Veteran");
    // Holy Strength (white) with P1's Gladecover Scout (hexproof) and Black Knight
    // (protection from white).
    let returner = |t: &mut TestGame, name: &str| {
        if name == "Annie Flash, the Veteran" {
            let card = t.hand(P0, name);
            give_mana_for(t, P0, name);
            t.cast(P0, card).go();
            t.resolve_all();
        } else {
            t.enter(P0, name);
            t.resolve_all();
        }
    };
    for name in ["Shepherd of the Cosmos", "Annie Flash, the Veteran"] {
        let mut t = TestGame::new(2);
        let scout = t.battlefield(P1, "Gladecover Scout");
        let knight = t.battlefield(P1, "Black Knight");
        let aura = t.graveyard(P0, "Holy Strength");
        t.answer_targets(P0, &[obj(aura)]);
        let from = t.asked().len();
        t.answer_choose(P0, &[obj(scout)]);
        returner(&mut t, name);
        let offered: Vec<Entity> = crate::r_s03_common::choice_candidates(&t, from, "")
            .into_iter()
            .flatten()
            .collect();
        assert!(!offered.contains(&obj(knight)), "{name}");
        assert_eq!(t.pt(scout), (2, 3), "{name}");
        // Wild Growth ("Enchant land") with no land on the battlefield: nothing legal to
        // enchant, so it stays in the graveyard.
        let mut t = TestGame::new(2);
        let aura = t.graveyard(P0, "Wild Growth");
        t.answer_targets(P0, &[obj(aura)]);
        if name == "Annie Flash, the Veteran" {
            // Annie's mana: tapped lands it could enchant; take them away first.
            let card = t.hand(P0, name);
            give_mana_for(&mut t, P0, name);
            t.cast(P0, card).go();
            for l in t.g.battlefield.clone() {
                if t.obj(l).is(mtg_engine::types::CardType::Land) {
                    crate::r_s05_common::move_to(&mut t, l, Zone::Exile);
                }
            }
            t.resolve_all();
        } else {
            returner(&mut t, name);
        }
        assert_eq!(t.zone(aura), Zone::Graveyard(P0), "{name}");
    }
}

#[test]
fn matter_reshaper_aura_needs_something_to_enchant() {
    cr!("303.4f");
    ruling!(
        "Matter Reshaper",
        "If you put an Aura card onto the battlefield this way, you choose what it enchants as it enters the battlefield. You must be able to choose a legal player or permanent or you can’t put the Aura onto the battlefield."
    );
    supported("Matter Reshaper");
    for with_creature in [true, false] {
        let mut t = TestGame::new(2);
        let reshaper = t.battlefield(P0, "Matter Reshaper");
        let bears = with_creature.then(|| t.battlefield(P1, "Gladecover Scout"));
        let aura = t.library_top(P0, "Holy Strength");
        t.answer_yes(P0, true);
        if let Some(b) = bears {
            t.answer_choose(P0, &[obj(b)]);
        }
        destroy(&mut t, reshaper);
        t.resolve_all();
        let now = t.g.current(aura);
        if let Some(b) = bears {
            assert_eq!(t.zone(now), Zone::Battlefield);
            assert_eq!(t.pt(b), (2, 3));
        } else {
            assert_ne!(t.zone(now), Zone::Battlefield);
        }
    }
}

#[test]
fn call_of_the_death_dweller_counters_follow_the_returned_cards() {
    cr!("608.2c", "115.1");
    ruling!(
        "Call of the Death-Dweller",
        "If you return no creature cards to the battlefield, you won’t put any counters on any creatures."
    );
    ruling!(
        "Call of the Death-Dweller",
        "If you return two creature cards, you may put one counter on each or put both counters on one."
    );
    ruling!(
        "Call of the Death-Dweller",
        "You could target a single creature card with Call of the Death-Dweller. In this case, you’ll put both counters on that creature."
    );
    supported("Call of the Death-Dweller");
    let total =
        |t: &TestGame, id: ObjectId| t.counters(id, "deathtouch") + t.counters(id, "menace");
    // No targets: the creature already on the battlefield gets nothing.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let call = t.hand(P0, "Call of the Death-Dweller");
    t.lands(P0, "Swamp", 3);
    t.cast(P0, call).targets(&[]).go();
    t.resolve_all();
    assert_eq!(total(&t, giant), 0);
    // One target: both counters on it.
    let mut t = TestGame::new(2);
    let elves = t.graveyard(P0, "Llanowar Elves");
    let call = t.hand(P0, "Call of the Death-Dweller");
    t.lands(P0, "Swamp", 3);
    t.cast(P0, call).target(elves).go();
    t.resolve_all();
    let elves = t.g.current(elves);
    assert_eq!(t.counters(elves, "deathtouch"), 1);
    assert_eq!(t.counters(elves, "menace"), 1);
    // Two: on each, or both on one, as P0 chooses.
    for split in [true, false] {
        let mut t = TestGame::new(2);
        let elves = t.graveyard(P0, "Llanowar Elves");
        let bears = t.graveyard(P0, "Grizzly Bears");
        let call = t.hand(P0, "Call of the Death-Dweller");
        t.lands(P0, "Swamp", 3);
        t.cast(P0, call).targets(&[obj(elves), obj(bears)]).go();
        t.answer_choose_moved(P0, &[obj(elves)]);
        t.answer_choose_moved(P0, &[obj(if split { bears } else { elves })]);
        t.resolve_all();
        let (e, b) = (t.g.current(elves), t.g.current(bears));
        assert!(t.on_battlefield(e) && t.on_battlefield(b));
        assert_eq!(total(&t, e), if split { 1 } else { 2 }, "split {split}");
        assert_eq!(total(&t, b), if split { 1 } else { 0 }, "split {split}");
    }
}

#[test]
fn crypt_champion_each_player_returns_one_of_their_choice() {
    cr!("608.2c", "101.4");
    ruling!(
        "Crypt Champion",
        "The first triggered ability isn't targeted, so the creature cards aren't chosen until the ability resolves. Each player chooses which creature card from their own graveyard to return. If a player doesn't have an applicable creature card, that player does nothing. If a player does have any applicable cards, that player must return one to the battlefield."
    );
    supported("Crypt Champion");
    let mut t = TestGame::new(3);
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Llanowar Elves");
    let ogre = t.graveyard(P1, "Gray Ogre");
    t.graveyard(P1, "Air Elemental");
    t.graveyard(P2, "Air Elemental");
    // P1 chooses the Gray Ogre (cards in graveyards are chosen as it resolves).
    t.answer_choose_moved(P1, &[obj(ogre)]);
    let from = t.asked().len();
    t.enter(P0, "Crypt Champion");
    t.resolve_all();
    assert!(target_candidates(&t, P0, from).is_empty());
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Gray Ogre").len(), 1);
    assert!(t.named_on_battlefield("Llanowar Elves").is_empty());
    assert!(t.named_on_battlefield("Air Elemental").is_empty());
    assert_eq!(t.obj_now(t.g.current(ogre)).controller, P1);
    // P2 had no applicable card; Crypt Champion (not cast with {R}) was sacrificed.
    assert!(t.in_graveyard(P0, "Crypt Champion"));
}

#[test]
fn carnage_returned_creature_attacks_only_if_able_and_free() {
    cr!("508.1d", "302.6");
    ruling!(
        "Carnage, Crimson Chaos",
        "If the returned creature can't attack for any reason (such as being tapped or having come under your control that turn), then it doesn't attack. If there's a cost associated with having it attack, you're not forced to pay that cost, so it doesn't have to attack in that case either."
    );
    supported("Carnage, Crimson Chaos");
    let returned = |t: &mut TestGame| {
        let bears = t.graveyard(P0, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        t.enter(P0, "Carnage, Crimson Chaos");
        t.resolve_all();
        t.g.current(bears)
    };
    // The turn it returns: summoning sick, no attack required.
    let mut t = TestGame::new(2);
    let bears = returned(&mut t);
    to_beginning_of_combat(&mut t, P0);
    assert!(!can_attack(&mut t, bears));
    assert!(legal_attack(&mut t, &[]));
    // A later turn: it must attack...
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(!legal_attack(&mut t, &[]));
    // ...unless attacking costs something (Ghostly Prison).
    t.battlefield(P1, "Ghostly Prison");
    t.lands(P0, "Wastes", 2);
    assert!(legal_attack(&mut t, &[]));
}

#[test]
fn extraction_specialist_restriction_ends_when_it_leaves() {
    cr!("611.2b", "508.1c", "509.1b");
    ruling!(
        "Extraction Specialist",
        "Once Extraction Specialist leaves the battlefield, the creature that is returned can attack and block as normal. If Extraction Specialist leaves the battlefield before its ability resolves, the creature that is returned will be able to attack and block as normal right away."
    );
    supported("Extraction Specialist");
    for leaves_first in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.graveyard(P0, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        let spec = t.enter(P0, "Extraction Specialist");
        t.settle();
        if leaves_first {
            destroy(&mut t, spec);
        }
        t.resolve_all();
        let bears = t.g.current(bears);
        assert!(t.on_battlefield(bears));
        // P1 attacks with a creature; can the Bears block it?
        let attacker = t.battlefield(P1, "Hill Giant");
        to_beginning_of_combat(&mut t, P1);
        attack_with(&mut t, &[(attacker, Entity::Player(P0))]);
        assert_eq!(
            legal_blocks(&mut t, P0, &[(bears, attacker)]),
            leaves_first,
            "{leaves_first}"
        );
        if !leaves_first {
            destroy(&mut t, spec);
            assert!(legal_blocks(&mut t, P0, &[(bears, attacker)]));
        }
    }
}

#[test]
fn kinscaer_sentry_x_counts_attackers_once_and_the_creature_picks_its_own_defender() {
    cr!("508.4", "608.2h");
    ruling!(
        "Kinscaer Sentry",
        "The value of X is calculated only once, as Kinscaer Sentry's last ability resolves."
    );
    ruling!(
        "Kinscaer Sentry",
        "You choose the player, planeswalker, or battle the creature you put onto the battlefield is attacking. It doesn't have to attack the same player, planeswalker, or battle as Kinscaer Sentry."
    );
    supported("Kinscaer Sentry");
    let mut t = TestGame::new(3);
    let sentry = t.battlefield(P0, "Kinscaer Sentry");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let bears = t.hand(P0, "Grizzly Bears");
    let giant = t.hand(P0, "Hill Giant");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[obj(bears)]);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    attack_with(
        &mut t,
        &[(sentry, Entity::Player(P1)), (elves, Entity::Player(P1))],
    );
    t.resolve_all();
    // Two attackers as it resolves: X = 2, so Grizzly Bears but not Hill Giant.
    let offered: Vec<Entity> = crate::r_s03_common::choice_candidates(&t, from, "")
        .into_iter()
        .flatten()
        .collect();
    assert!(offered.contains(&obj(bears)));
    assert!(!offered.contains(&obj(giant)));
    let now = t.g.current(bears);
    let attacks = crate::r_s09_common::attacks_now(&t);
    let (_, defender) = attacks.iter().find(|(a, _)| *a == now).expect("attacking");
    assert!(t.obj_now(now).tapped);
    assert_eq!(*defender, Entity::Player(P2), "{attacks:?}");
}

#[test]
fn rebel_searches_may_find_nothing() {
    cr!("701.23b");
    ruling!(
        "Defiant Falcon",
        "You do not have to find a Rebel card if you do not want to."
    );
    ruling!(
        "Defiant Vanguard",
        "You do not have to find a Rebel card if you do not want to."
    );
    ruling!(
        "Lin Sivvi, Defiant Hero",
        "You do not have to find a Rebel card if you do not want to."
    );
    ruling!("Defiant Falcon", "This card is both a Rebel and a Bird.");
    for (name, index, mana) in [
        ("Defiant Falcon", 0usize, 4usize),
        ("Defiant Vanguard", 0, 5),
        ("Lin Sivvi, Defiant Hero", 0, 3),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, name);
        let rebel = t.library_top(P0, "Defiant Falcon");
        t.lands(P0, "Plains", mana);
        t.answer(P0, DecisionKind::X, Answer::Number(3));
        t.answer_choose(P0, &[]);
        t.activate(P0, src, index, &[]).expect("activate");
        t.resolve_all();
        assert_eq!(t.zone(t.g.current(rebel)), Zone::Library(P0), "{name}");
        assert!(
            t.named_on_battlefield("Defiant Falcon").len() == usize::from(name == "Defiant Falcon")
        );
    }
    let mut t = TestGame::new(2);
    let falcon = t.battlefield(P0, "Defiant Falcon");
    t.recompute();
    let c = &t.obj_now(falcon).chars;
    assert!(c.has_subtype("Rebel") && c.has_subtype("Bird"));
}
