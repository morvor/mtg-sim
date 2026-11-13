//! Rulings batch P150 — rummaging: discarding as a cost, "you may discard ... If you do"
//! on spells whose target became illegal, Blood tokens (with Arterial Alchemy), Nantuko
//! Cultivator's counts, Crypt Lurker, Dream Strix, Fear of Missing Out's combat and
//! Academic Dispute's block requirement.

use crate::r_s01_common::*;
use crate::r_s02_common::{create_token, target_candidates};
use crate::r_s05_common::move_to;
use crate::r_s21_common::{castable, go_to};
use crate::r_s25_common::cast_new;
use crate::r_s29_common::cast_and_resolve;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Fills `p`'s hand with `n` copies of `name`.
fn hand_of(t: &mut TestGame, p: PlayerId, name: &str, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.hand(p, name)).collect()
}

fn blood_tokens(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.has_subtype("Blood"))
        .map(|o| o.id)
        .collect()
}

// ---------------------------------------------------------------------------------------
// Discarding as an additional cost (Cathartic Reunion)
// ---------------------------------------------------------------------------------------

#[test]
fn cathartic_reunion_needs_two_other_cards_in_hand() {
    cr!("601.2b", "601.2h");
    ruling!(
        "Cathartic Reunion",
        "Because discarding two cards is an additional cost, you can't cast Cathartic Reunion if you don't have at least two other cards in hand."
    );
    supported("Cathartic Reunion");
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Cathartic Reunion");
    let reunion = t.hand(P0, "Cathartic Reunion");
    t.hand(P0, "Grizzly Bears");
    assert!(!castable(&mut t, P0, reunion));
    t.hand(P0, "Grizzly Bears");
    assert!(castable(&mut t, P0, reunion));
}

// ---------------------------------------------------------------------------------------
// Spells whose only target became illegal: none of the rummaging happens.
// ---------------------------------------------------------------------------------------

#[test]
fn rummage_spells_with_an_illegal_target_do_nothing() {
    cr!("608.2b", "701.9a");
    ruling!(
        "Incinerating Blast",
        "If the target creature is an illegal target when Incinerating Blast tries to resolve, it won't resolve and none of its effects will happen. You won't discard or draw."
    );
    ruling!(
        "Vengeful Possession",
        "If the target creature is an illegal target when Vengeful Possession tries to resolve, it won't resolve and none of its effects will happen. You won't have the option to discard a card in order to draw a card."
    );
    ruling!(
        "Tweeze",
        "If the target is illegal as Tweeze tries to resolve, it won't resolve and none of its effects will happen. You won't have the opportunity to discard a card to draw a card."
    );
    ruling!(
        "Manhole Missile",
        "If the target creature is an illegal target at the time Manhole Missile tries to resolve, it won't resolve and none of its effects will happen. You won't get to put a card from your hand on the bottom of your library, and you won't draw a card."
    );
    ruling!(
        "Fire Prophecy",
        "If the target creature is an illegal target by the time Fire Prophecy tries to resolve, the spell won't resolve. You won't have the option to put a card on the bottom of your library."
    );
    ruling!(
        "Fear of Missing Out",
        "If the target creature is an illegal target as Fear of Missing Out's last ability tries to resolve, it won't resolve and none of its effects will happen. There won't be an additional combat phase."
    );
    for name in [
        "Incinerating Blast",
        "Vengeful Possession",
        "Tweeze",
        "Manhole Missile",
        "Fire Prophecy",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let card = t.hand(P0, "Hill Giant");
        cast_new(&mut t, P0, name, &[obj(bears)]);
        move_to(&mut t, bears, Zone::Exile);
        let from = t.asked().len();
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(card)]);
        t.resolve_all();
        assert_eq!(t.zone(card), Zone::Hand(P0), "{name}");
        assert_eq!(t.hand_size(P0), 1, "{name}");
        assert!(
            !t.asked()[from..].iter().any(|(_, d)| matches!(
                d,
                Decision::YesNo { .. } | Decision::ChooseEntities { .. }
            )),
            "{name}: asked to rummage"
        );
        assert!(t.in_graveyard(P0, name), "{name}");
    }
}

#[test]
fn vengeful_possession_can_target_untapped_and_own_creatures() {
    cr!("115.1a", "115.2");
    ruling!(
        "Vengeful Possession",
        "Vengeful Possession can target any creature, even one that's untapped or one you already control."
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.g.tap(mine);
    let from = t.asked().len();
    let spell = cast_new(&mut t, P0, "Vengeful Possession", &[obj(mine)]);
    let offered = target_candidates(&t, P0, from);
    assert!(offered[0].contains(&obj(mine)) && offered[0].contains(&obj(theirs)));
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Vengeful Possession"));
    let _ = spell;
    assert!(!t.obj_now(mine).tapped, "untapped");
    assert!(t.obj_now(mine).chars.has_keyword(KeywordKind::Haste));
}

// ---------------------------------------------------------------------------------------
// One card only, and empty hands
// ---------------------------------------------------------------------------------------

#[test]
fn single_rummages_discard_or_bottom_only_one_card() {
    cr!("701.9a", "121.1");
    ruling!(
        "Fissure Wizard",
        "While resolving Fissure Wizard's ability, you can't discard more than one card."
    );
    ruling!(
        "Fire Prophecy",
        "While resolving Fire Prophecy, you can't put more than one card on the bottom of your library to draw more than one card."
    );
    supported("Fissure Wizard");
    // Fissure Wizard: an answer naming two cards discards no more than one.
    let mut t = TestGame::new(2);
    let cards = hand_of(&mut t, P0, "Hill Giant", 2);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(cards[0]), obj(cards[1])]);
    t.enter(P0, "Fissure Wizard");
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(t.hand_size(P0), 2);
    // Fire Prophecy.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let cards = hand_of(&mut t, P0, "Hill Giant", 2);
    let lib = t.library_size(P0);
    cast_new(&mut t, P0, "Fire Prophecy", &[obj(bears)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(cards[0]), obj(cards[1])]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.library_size(P0), lib);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn rummaging_with_too_few_cards_in_hand() {
    cr!("701.9a", "701.9b");
    ruling!(
        "Faithless Salvaging",
        "If you have no cards in hand as Faithless Salvaging resolves, you won't discard a card, but you will still draw a card."
    );
    ruling!(
        "Riptide Survivor",
        "If you have less than 2 cards in your hand, discard all the cards you have. If you have no cards in hand, you can discard zero and draw 3."
    );
    supported("Faithless Salvaging");
    supported("Riptide Survivor");
    let mut t = TestGame::new(2);
    cast_and_resolve(&mut t, P0, "Faithless Salvaging", &[]);
    assert_eq!(t.hand_size(P0), 1);
    for in_hand in [0usize, 1] {
        let mut t = TestGame::new(2);
        let survivor = crate::r_s12_common::morph(&mut t, P0, "Riptide Survivor");
        let cards = hand_of(&mut t, P0, "Hill Giant", in_hand);
        t.lands(P0, "Island", 3);
        assert!(crate::r_s11_common::turn_face_up(&mut t, P0, survivor));
        t.resolve_all();
        assert_eq!(t.hand_size(P0), 3, "{in_hand} in hand");
        for c in cards {
            assert_eq!(t.zone(c), Zone::Graveyard(P0));
        }
    }
}

// ---------------------------------------------------------------------------------------
// Nantuko Cultivator: "When this creature enters, you may discard any number of land
// cards. Put that many +1/+1 counters on this creature and draw that many cards."
// ---------------------------------------------------------------------------------------

#[test]
fn nantuko_cultivator_counters_and_draws_are_modified_separately() {
    cr!("614.1a", "608.2c");
    ruling!(
        "Nantuko Cultivator",
        "If an effect modifies how many +1/+1 counters you put on Nantuko Cultivator, the number of cards you draw isn't modified. Likewise, if an effect modifies how many cards you draw, the number of +1/+1 counters you put on Nantuko Cultivator won't be modified."
    );
    ruling!(
        "Nantuko Cultivator",
        "You discard, put counters, and draw all while Nantuko Cultivator's ability is resolving. Nothing can happen between the three, and no player may choose to take actions."
    );
    supported("Nantuko Cultivator");
    // (modifier, counters, cards drawn) for two land cards discarded.
    for (modifier, counters, drawn) in [("Hardened Scales", 3, 2), ("Thought Reflection", 2, 4)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, modifier);
        let lands = hand_of(&mut t, P0, "Forest", 2);
        let es: Vec<Entity> = lands.iter().map(|l| obj(*l)).collect();
        t.answer_choose(P0, &es);
        let c = t.enter(P0, "Nantuko Cultivator");
        t.settle();
        let from = t.asked().len();
        t.g.resolve_top();
        let priority_during = t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::Priority { .. }))
            .count();
        assert_eq!(priority_during, 0);
        t.settle();
        assert_eq!(t.counters(c, "+1/+1"), counters, "{modifier}");
        assert_eq!(t.hand_size(P0), drawn, "{modifier}");
    }
}

// ---------------------------------------------------------------------------------------
// Crypt Lurker: "When this creature enters, you may sacrifice a creature or discard a
// creature card. If you do, draw a card."
// ---------------------------------------------------------------------------------------

#[test]
fn crypt_lurker_can_sacrifice_itself_but_only_one_thing() {
    cr!("118.12", "701.21a");
    ruling!(
        "Crypt Lurker",
        "Crypt Lurker can be sacrificed to pay the cost of its own triggered ability."
    );
    ruling!(
        "Crypt Lurker",
        "While resolving Crypt Lurker's ability, you can't sacrifice more than one creature, discard more than one card, or do both to draw more than one card."
    );
    supported("Crypt Lurker");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.hand(P0, "Hill Giant");
    let lurker = t.enter(P0, "Crypt Lurker");
    t.settle();
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lurker), obj(bears), obj(giant)]);
    t.answer_choose(P0, &[obj(lurker), obj(bears), obj(giant)]);
    t.resolve_all();
    let offered: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert!(
        offered.iter().any(|c| c.contains(&obj(lurker))),
        "{offered:?}"
    );
    // Exactly one thing was given up, and one card drawn.
    let gone = [lurker, bears, giant]
        .iter()
        .filter(|id| matches!(t.zone(**id), Zone::Graveyard(_)))
        .count();
    assert_eq!(gone, 1);
    assert_eq!(
        t.hand_size(P0),
        if t.zone(giant) == Zone::Hand(P0) {
            2
        } else {
            1
        }
    );
}

// ---------------------------------------------------------------------------------------
// Dream Strix: "When this creature becomes the target of a spell, sacrifice it."
// ---------------------------------------------------------------------------------------

#[test]
fn dream_strix_is_sacrificed_before_the_spell_resolves() {
    cr!("608.2b", "603.3");
    ruling!(
        "Dream Strix",
        "If Dream Strix becomes the target of a spell, it will be sacrificed before that spell tries to resolve. If it is the only target of that spell, it will be removed from the stack rather than resolving, and no part of its instructions will be performed."
    );
    supported("Dream Strix");
    supported("Vapor Snag");
    let mut t = TestGame::new(2);
    let strix = t.battlefield(P1, "Dream Strix");
    cast_new(&mut t, P0, "Vapor Snag", &[obj(strix)]);
    t.settle();
    // The trigger resolves first.
    t.resolve();
    assert!(t.in_graveyard(P1, "Dream Strix"));
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "Vapor Snag didn't resolve");
    assert!(!t.in_hand(P1, "Dream Strix"));
}

// ---------------------------------------------------------------------------------------
// Fear of Missing Out: "... untap target creature. After this phase, there is an
// additional combat phase."
// ---------------------------------------------------------------------------------------

#[test]
fn fear_of_missing_out_combat_follows_the_combat_directly() {
    cr!("500.8", "505.1a");
    ruling!(
        "Fear of Missing Out",
        "Fear of Missing Out's last ability doesn't give you any additional main phases. This means that you will move directly from the end of combat step of one combat phase to the beginning of combat step of the next one."
    );
    supported("Fear of Missing Out");
    let mut t = TestGame::new(2);
    let fomo = t.battlefield(P0, "Fear of Missing Out");
    // Delirium: four card types in the graveyard.
    for n in ["Grizzly Bears", "Forest", "Shock", "Divination"] {
        t.graveyard(P0, n);
    }
    t.set_step(P0, Step::PrecombatMain);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(fomo, Entity::Player(P1))]),
    );
    t.answer_targets(P0, &[obj(fomo)]);
    go_to(&mut t, Step::DeclareAttackers);
    t.resolve_all();
    go_to(&mut t, Step::PostcombatMain);
    let steps: Vec<Step> =
        t.g.turn_events
            .iter()
            .filter_map(|e| match e {
                mtg_engine::events::Event::StepBegan { step, .. } => Some(*step),
                _ => None,
            })
            .collect();
    let first_end = steps.iter().position(|s| *s == Step::EndOfCombat).unwrap();
    assert_eq!(steps[first_end + 1], Step::BeginningOfCombat, "{steps:?}");
    assert_eq!(steps.iter().filter(|s| s.is_main()).count(), 1, "{steps:?}");
}

// ---------------------------------------------------------------------------------------
// Blood tokens and Arterial Alchemy ("Blood tokens you control are Equipment in addition
// to their other types and have "Equipped creature gets +2/+0" and equip {2}.")
// ---------------------------------------------------------------------------------------

fn equip(t: &mut TestGame, blood: ObjectId, to: ObjectId) {
    t.answer_targets(P0, &[obj(to)]);
    crate::r_s06_common::activate_containing(t, P0, blood, "Equip").unwrap();
    t.resolve_all();
}

#[test]
fn arterial_alchemy_blood_equipment() {
    cr!("301.5c", "704.5n", "613.1d");
    ruling!(
        "Arterial Alchemy",
        "Blood tokens stop being Equipment if Arterial Alchemy leaves the battlefield. If any of them are attached to creatures at that time, they become unattached."
    );
    ruling!(
        "Arterial Alchemy",
        "If you control more than one Arterial Alchemy, creatures equipped with Blood Equipment will get +2/+0 for each Arterial Alchemy. The equip cost is still {2}."
    );
    ruling!(
        "Arterial Alchemy",
        "If a Blood token is somehow both a creature and an Equipment, it can't be attached to a creature."
    );
    supported("Arterial Alchemy");
    let mut t = TestGame::new(2);
    let a1 = t.battlefield(P0, "Arterial Alchemy");
    t.battlefield(P0, "Arterial Alchemy");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blood = create_token(&mut t, P0, "Blood");
    t.lands(P0, "Wastes", 2);
    equip(&mut t, blood, bears);
    assert_eq!(
        crate::r_s06_common::attached_to(&t, blood),
        Some(obj(bears))
    );
    assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 0);
    t.g.recompute();
    assert_eq!(t.pt(bears), (6, 2));
    // One Alchemy leaves: still Equipment, +2/+0.
    move_to(&mut t, a1, Zone::Exile);
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 2));
    // The other leaves: not Equipment, unattached.
    let a2 = t.named_on_battlefield("Arterial Alchemy")[0];
    move_to(&mut t, a2, Zone::Exile);
    t.g.recompute();
    assert_eq!(crate::r_s06_common::attached_to(&t, blood), None);
    assert_eq!(t.pt(bears), (2, 2));
    // A Blood Equipment that's also a creature becomes unattached.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Arterial Alchemy");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blood = create_token(&mut t, P0, "Blood");
    t.lands(P0, "Wastes", 2);
    equip(&mut t, blood, bears);
    crate::r_s26_common::modify_until_eot(
        &mut t,
        blood,
        vec![
            Modification::AddTypes(vec![CardType::Creature]),
            Modification::SetPT(Some(Value::c(1)), Some(Value::c(1))),
        ],
    );
    t.settle();
    assert_eq!(crate::r_s06_common::attached_to(&t, blood), None);
}

#[test]
fn blood_tokens_with_other_subtypes_are_still_blood_tokens() {
    cr!("111.10g", "205.3g");
    ruling!(
        "Bloodtithe Harvester",
        "If an effect refers to a Blood token, it means any artifact token with the subtype Blood, even if it has gained other subtypes."
    );
    ruling!(
        "Bloodtithe Harvester",
        "The value of X is determined only once, as Bloodtithe Harvester’s last ability resolves."
    );
    supported("Bloodtithe Harvester");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // Arterial Alchemy makes them Equipment too.
    t.battlefield(P0, "Arterial Alchemy");
    let harvester = t.battlefield(P0, "Bloodtithe Harvester");
    let b1 = create_token(&mut t, P0, "Blood");
    create_token(&mut t, P0, "Blood");
    t.g.recompute();
    assert!(t.obj_now(b1).chars.has_subtype("Equipment"));
    let giant = t.battlefield(P1, "Craw Wurm");
    t.activate(P0, harvester, 0, &[obj(giant)]).unwrap();
    // In response, one is sacrificed: X is 2 as the ability resolves.
    t.g.sacrifice(b1, P0);
    t.settle();
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.pt(giant), (4, 2));
    // More Blood tokens later don't change it.
    create_token(&mut t, P0, "Blood");
    t.g.recompute();
    assert_eq!(t.pt(giant), (4, 2));
}

#[test]
fn sacrificing_a_blood_token_for_any_reason_triggers() {
    cr!("701.21a", "603.2");
    ruling!(
        "Gluttonous Guest",
        "Some triggered abilities trigger \"whenever you sacrifice a Blood token.\" These abilities trigger regardless of why you sacrificed that Blood token."
    );
    supported("Gluttonous Guest");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gluttonous Guest");
    // Its own ability.
    let b = create_token(&mut t, P0, "Blood");
    t.hand(P0, "Hill Giant");
    t.lands(P0, "Wastes", 1);
    t.answer_choose(P0, &[]);
    t.activate(P0, b, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    // Deadly Dispute's additional cost.
    let b = create_token(&mut t, P0, "Blood");
    t.answer_choose(P0, &[obj(b)]);
    cast_new(&mut t, P0, "Deadly Dispute", &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // An effect's sacrifice.
    let b = create_token(&mut t, P0, "Blood");
    t.g.sacrifice(b, P0);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}

#[test]
fn one_blood_token_pays_only_one_cost() {
    cr!("118.3", "701.21a");
    ruling!(
        "Wedding Security",
        "You can't sacrifice a Blood token to pay multiple costs."
    );
    supported("Wedding Security");
    supported("Bloodcrazed Socialite");
    let mut t = TestGame::new(2);
    let sec = t.battlefield(P0, "Wedding Security");
    let soc = t.battlefield(P0, "Bloodcrazed Socialite");
    let b = create_token(&mut t, P0, "Blood");
    for _ in 0..4 {
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(b)]);
    }
    t.attack(&[(sec, Entity::Player(P1)), (soc, Entity::Player(P1))], &[]);
    // Wedding Security 4/4 (+1/+1 counter: 5), Bloodcrazed Socialite 3/3 (+2/+2: 5): one
    // of them benefited, not both.
    let dealt = 20 - t.life(P1);
    let sec_paid = t.counters(sec, "+1/+1") == 1;
    assert_eq!(
        dealt,
        if sec_paid { 5 + 3 } else { 4 + 5 },
        "{}",
        t.dump_log()
    );
    assert!(blood_tokens(&t, P0).is_empty());
    // Bloodcrazed Socialite alone: "If you do, it gets +2/+2" pumps the Socialite, not the
    // sacrificed token.
    let mut t = TestGame::new(2);
    let soc = t.battlefield(P0, "Bloodcrazed Socialite");
    let b = create_token(&mut t, P0, "Blood");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(b)]);
    t.attack(&[(soc, Entity::Player(P1))], &[]);
    assert!(blood_tokens(&t, P0).is_empty());
    assert_eq!(t.life(P1), 15);
}

// ---------------------------------------------------------------------------------------
// Academic Dispute: "Target creature blocks this turn if able. You may have it gain reach
// until end of turn."
// ---------------------------------------------------------------------------------------

#[test]
fn academic_dispute_reach_is_chosen_on_resolution_and_block_only_if_able() {
    cr!("509.1c", "608.2d");
    ruling!(
        "Academic Dispute",
        "You choose whether you want the creature to gain reach until end of turn as Academic Dispute resolves."
    );
    ruling!(
        "Academic Dispute",
        "The creature blocks only if it's able to do so as the declare blockers step begins. If, at that time, the creature is tapped, it's affected by a spell or ability that says it can't block, or no creatures are attacking its controller or a planeswalker controlled by that player, then it doesn't block."
    );
    supported("Academic Dispute");
    for (reach, tap_it) in [(true, false), (false, false), (false, true)] {
        let mut t = TestGame::new(2);
        let attacker = t.battlefield(P0, "Grizzly Bears");
        let blocker = t.battlefield(P1, "Hill Giant");
        t.set_step(P0, Step::PrecombatMain);
        let from = t.asked().len();
        cast_new(&mut t, P0, "Academic Dispute", &[obj(blocker)]);
        t.answer_yes(P0, reach);
        t.answer_yes(P0, false); // learn: nothing
        t.resolve_all();
        assert!(
            t.asked()[from..]
                .iter()
                .any(|(_, d)| matches!(d, Decision::YesNo { .. })),
            "asked on resolution"
        );
        assert_eq!(
            t.obj_now(blocker).chars.has_keyword(KeywordKind::Reach),
            reach
        );
        if tap_it {
            t.g.tap(blocker);
        }
        // P1 declares no blocks; the requirement makes the Giant block if able.
        crate::r_s03_common::to_blockers(&mut t, &[(attacker, Entity::Player(P1))], &[]);
        assert_eq!(crate::r_s10_common::blocking(&t, blocker), !tap_it);
    }
}
