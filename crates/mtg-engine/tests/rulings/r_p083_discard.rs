//! Rulings batch P083 — discard hate: cards that punish an opponent for making you
//! discard them (Quagnoth, Obstinate Baloth, Loxodon Smiter, Guerrilla Tactics, Nephalia
//! Academy; CR 701.9, 614.1a, 113.6k), replacement effects that compete
//! (CR 616.1), madness (CR 702.35), discard triggers (Spirit Cairn, Drake Haven, Rielle,
//! Glint-Horn Buccaneer) including during the cleanup step (CR 514.3a), and Tamiyo's
//! discard protection.

use crate::r_p008_common::damage_events;
use crate::r_p083_common::*;
use crate::r_s01_common::{attack_with, tokens};
use crate::r_s03_common::respond;
use crate::r_s29_common::replacement_choosers;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::{StackKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

/// On P1's turn, P1 casts Mind Rot ("Target player discards two cards.") at P0; it
/// resolves (only it).
fn mind_rot(t: &mut TestGame) {
    supported("Mind Rot");
    t.set_step(P1, Step::PrecombatMain);
    cast_new(t, P1, "Mind Rot", &[player(P0)]);
    t.resolve();
}

/// The cards named `name` in exile.
fn exiled(t: &TestGame, name: &str) -> Vec<ObjectId> {
    t.g.exile
        .iter()
        .copied()
        .filter(|id| t.g.obj(*id).chars.name == name)
        .collect()
}

// ---------------------------------------------------------------------------------
// Quagnoth: "When a spell or ability an opponent controls causes you to discard this
// card, return it to your hand."
// ---------------------------------------------------------------------------------

#[test]
fn quagnoth_triggers_from_the_graveyard_unless_cards_there_lose_their_abilities() {
    cr!("113.6k", "701.9a", "113.6");
    ruling!(
        "Quagnoth",
        "If Quagnoth is discarded normally (it moves from your hand to your graveyard), its last ability triggers from your graveyard. If Yixlid Jailer is on the battlefield, it won't trigger."
    );
    supported("Quagnoth");
    supported("Yixlid Jailer");
    for jailer in [false, true] {
        let mut t = TestGame::new(2);
        if jailer {
            t.battlefield(P1, "Yixlid Jailer");
        }
        t.hand(P0, "Quagnoth");
        mind_rot(&mut t);
        assert!(t.in_graveyard(P0, "Quagnoth"));
        assert_eq!(t.stack_len(), usize::from(!jailer), "jailer = {jailer}");
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "Quagnoth"), !jailer, "jailer = {jailer}");
    }
}

#[test]
fn quagnoth_that_leaves_its_new_zone_isnt_returned() {
    cr!("113.6k", "400.7", "608.2b");
    ruling!(
        "Quagnoth",
        "If an effect of a spell or ability an opponent controls causes Quagnoth to be discarded, but Quagnoth leaves its new zone after its ability triggers, the ability will have no effect when it resolves."
    );
    supported("Extirpate");
    let mut t = TestGame::new(2);
    t.hand(P0, "Quagnoth");
    mind_rot(&mut t);
    assert_eq!(t.stack_len(), 1, "Quagnoth's trigger");
    let in_gy = t.g.player(P0).graveyard[0];
    // In response, P1 exiles it from the graveyard with Extirpate.
    cast_resolve(&mut t, P1, "Extirpate", &[obj(in_gy)]);
    assert_eq!(exiled(&t, "Quagnoth").len(), 1);
    assert!(!t.in_hand(P0, "Quagnoth"));
}

#[test]
fn quagnoth_discarded_into_exile_returns_from_exile() {
    cr!("113.6k", "614.1a", "701.9a");
    ruling!(
        "Quagnoth",
        "If an effect of a spell or ability an opponent controls causes Quagnoth to be discarded, but Quagnoth moves from your hand directly to the Exile zone (due to Leyline of the Void, for example), the ability will trigger from that zone. It will return Quagnoth to your hand from that zone when it resolves."
    );
    supported("Leyline of the Void");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Leyline of the Void");
    t.hand(P0, "Quagnoth");
    mind_rot(&mut t);
    assert_eq!(exiled(&t, "Quagnoth").len(), 1);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_hand(P0, "Quagnoth"));
    assert!(exiled(&t, "Quagnoth").is_empty());
}

// ---------------------------------------------------------------------------------
// "If a spell or ability an opponent controls causes you to discard this card, put it
// onto the battlefield instead of putting it into your graveyard."
// ---------------------------------------------------------------------------------

fn battlefield_first(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    match d {
        Decision::ChooseReplacement { options } => options
            .iter()
            .position(|o| o.contains("battlefield"))
            .map(Answer::Index),
        _ => None,
    }
}

fn leyline_first(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    match d {
        Decision::ChooseReplacement { options } => options
            .iter()
            .position(|o| !o.contains("battlefield"))
            .map(Answer::Index),
        _ => None,
    }
}

#[test]
fn obstinate_baloth_against_leyline_of_the_void_its_owner_chooses() {
    cr!("616.1", "616.1a", "614.1a");
    ruling!(
        "Obstinate Baloth",
        "If a spell or ability an opponent controls causes you to discard Obstinate Baloth, and both Obstinate Baloth's ability and another ability (such as that of an opponent's Leyline of the Void) instruct you to put Obstinate Baloth somewhere else instead of putting it into your graveyard, you choose which one to apply."
    );
    supported("Obstinate Baloth");
    for battlefield in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Leyline of the Void");
        t.hand(P0, "Obstinate Baloth");
        respond(
            &mut t,
            P0,
            if battlefield {
                battlefield_first
            } else {
                leyline_first
            },
        );
        let from = t.asked().len();
        mind_rot(&mut t);
        t.resolve_all();
        assert!(replacement_choosers(&t, from).contains(&P0));
        assert!(!replacement_choosers(&t, from).contains(&P1));
        assert_eq!(
            t.named_on_battlefield("Obstinate Baloth").len(),
            usize::from(battlefield)
        );
        assert_eq!(exiled(&t, "Obstinate Baloth").len(), usize::from(!battlefield));
    }
}

#[test]
fn discarded_onto_the_battlefield_is_still_discarded() {
    cr!("701.9a", "614.1a", "614.6");
    ruling!(
        "Loxodon Smiter",
        "If you discard Loxodon Smiter and put it onto the battlefield, you've still discarded a card. Abilities that trigger when you discard a card (such as the one from Liliana's Caress) will still trigger."
    );
    ruling!(
        "Obstinate Baloth",
        "If you discard Obstinate Baloth and put it onto the battlefield, you've still discarded it. Abilities that trigger when you discard a card (such as that of Raiders' Wake) will still trigger."
    );
    ruling!(
        "Obstinate Baloth",
        "If you discard Obstinate Baloth and put it onto the battlefield, you've still discarded it. Abilities that trigger when you discard a card will still trigger."
    );
    supported("Loxodon Smiter");
    supported("Liliana's Caress");
    supported("Raiders' Wake");
    // Liliana's Caress: "Whenever an opponent discards a card, that player loses 2 life."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Liliana's Caress");
    t.hand(P0, "Loxodon Smiter");
    mind_rot(&mut t);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Loxodon Smiter").len(), 1);
    assert_eq!(t.life(P0), 18);
    // Raiders' Wake, the same; Obstinate Baloth enters and gains P0 4 life.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Raiders' Wake");
    t.hand(P0, "Obstinate Baloth");
    mind_rot(&mut t);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Obstinate Baloth").len(), 1);
    assert_eq!(t.life(P0), 20 - 2 + 4);
}

// ---------------------------------------------------------------------------------
// Nephalia Academy: "If a spell or ability an opponent controls causes you to discard a
// card, you may reveal that card and put it on top of your library instead of putting it
// anywhere else."
// ---------------------------------------------------------------------------------

#[test]
fn nephalia_academy_applies_when_you_choose_to_discard() {
    cr!("614.1a", "701.9a");
    ruling!(
        "Nephalia Academy",
        "If a spell or ability an opponent controls causes you to make a choice and you choose to discard a card, Nephalia Academy’s ability will apply."
    );
    supported("Nephalia Academy");
    supported("Painful Quandary");
    // Painful Quandary: "Whenever an opponent casts a spell, that player loses 5 life
    // unless they discard a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nephalia Academy");
    t.battlefield(P1, "Painful Quandary");
    let bolt = t.hand(P0, "Lightning Bolt");
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    // Discard rather than lose life; then use the Academy.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "discarded instead");
    let top = *t.g.player(P0).library.last().unwrap();
    assert_eq!(t.g.obj(top).chars.name, "Lightning Bolt", "on top of the library");
    assert!(!t.in_graveyard(P0, "Lightning Bolt"));
    let _ = bolt;
}

#[test]
fn nephalia_academy_or_madness() {
    cr!("616.1", "702.35a", "702.35b");
    ruling!(
        "Nephalia Academy",
        "If you discard a card with madness, you’ll choose whether to exile it or put it on top of your library. If you put it on top of your library, you can’t cast it using madness."
    );
    supported("Biting Rain");
    // P0 applies the Academy (and reveals) or madness first, then declines the other
    // effect if it's still offered; P0 casts the card for madness if asked.
    fn answer(d: &Decision, academy: bool) -> Option<Answer> {
        match d {
            Decision::ChooseReplacement { options } => options
                .iter()
                .position(|o| o.contains("library") == academy)
                .map(Answer::Index),
            Decision::YesNo { prompt, .. } if prompt.starts_with("Apply: Nephalia") => {
                Some(Answer::Bool(academy))
            }
            Decision::YesNo { prompt, .. } if prompt.contains("madness cost") => {
                Some(Answer::Bool(true))
            }
            _ => None,
        }
    }
    fn academy_first(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
        answer(d, true)
    }
    fn madness_first(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
        answer(d, false)
    }
    for academy in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Nephalia Academy");
        let rain = t.hand(P0, "Biting Rain");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Swamp", 3);
        respond(&mut t, P0, if academy { academy_first } else { madness_first });
        let from = t.asked().len();
        mind_rot(&mut t);
        t.resolve_all();
        assert!(replacement_choosers(&t, from).contains(&P0));
        let madness_asked = t.asked()[from..].iter().any(
            |(_, d)| matches!(d, Decision::YesNo { prompt, .. } if prompt.contains("madness cost")),
        );
        assert_eq!(madness_asked, !academy);
        if academy {
            let top = *t.g.player(P0).library.last().unwrap();
            assert_eq!(top, t.g.current(rain), "on top of the library");
            assert!(t.on_battlefield(bears), "not cast");
        } else {
            assert!(!t.on_battlefield(bears), "cast for madness");
            assert!(t.in_graveyard(P0, "Biting Rain"));
        }
    }
}

// ---------------------------------------------------------------------------------
// Tamiyo, Collector of Tales: "Spells and abilities your opponents control can't cause
// you to discard cards or sacrifice permanents."
// ---------------------------------------------------------------------------------

#[test]
fn tamiyo_doesnt_stop_the_cleanup_discard_with_a_smaller_hand_size() {
    cr!("514.1", "402.2");
    ruling!(
        "Tamiyo, Collector of Tales",
        "If a spell or ability your opponent controls reduces your maximum hand size, Tamiyo’s first ability won’t stop you from discarding cards when the game rules cause you to discard during your cleanup step."
    );
    supported("Tamiyo, Collector of Tales");
    supported("Gnat Miser");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tamiyo, Collector of Tales");
    t.battlefield(P1, "Gnat Miser");
    for _ in 0..7 {
        t.hand(P0, "Grizzly Bears");
    }
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.hand_size(P0), 6);
    assert_eq!(t.graveyard_size(P0), 1);
}

// ---------------------------------------------------------------------------------
// Discard triggers in the cleanup step.
// ---------------------------------------------------------------------------------

/// Advances P0's turn to its cleanup step at the point a player gets priority there.
fn cleanup_priority(t: &mut TestGame) -> bool {
    t.g.run_until(10_000, |g| {
        g.turn.step == Step::Cleanup && g.turn.stage == Stage::Priority && g.turn.priority.is_some()
            || g.turn.active != P0
    }) && t.g.turn.active == P0
}

#[test]
fn cleanup_discard_triggers_get_priority_and_another_cleanup_step() {
    cr!("514.1", "514.3a");
    ruling!(
        "Spirit Cairn",
        "If a player discards a card during their cleanup step due to having too many cards in hand, any appropriate abilities that trigger on discarding that card (such as Spirit Cairn’s ability) trigger. If this happens, those triggered abilities are put onto the stack and players receive priority in that cleanup step to cast spells or activate abilities (normally, no players may take actions during a cleanup step). Another cleanup step is created following that one."
    );
    ruling!(
        "Rielle, the Everwise",
        "If the first time you discard cards during your turn is due to having too many cards in hand during your cleanup step, you'll draw that many cards. Players will receive priority during the cleanup step in this case, and a new cleanup step will happen after that one, during which you may have to discard cards again."
    );
    supported("Spirit Cairn");
    supported("Rielle, the Everwise");
    // Spirit Cairn: the trigger resolves in the cleanup step, with priority.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spirit Cairn");
    t.lands(P0, "Plains", 1);
    for _ in 0..8 {
        t.hand(P0, "Grizzly Bears");
    }
    t.answer_yes(P0, true);
    assert!(cleanup_priority(&mut t), "priority in the cleanup step");
    assert_eq!(t.hand_size(P0), 7);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(tokens(&t, P0).len(), 1, "the Spirit was created");
    // Rielle: the first discard is in the cleanup step: draw one; a new cleanup step
    // makes P0 discard again (Rielle doesn't trigger a second time this turn).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rielle, the Everwise");
    for _ in 0..8 {
        t.hand(P0, "Grizzly Bears");
    }
    let lib = t.library_size(P0);
    assert!(cleanup_priority(&mut t));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.library_size(P0), lib - 1, "Rielle drew one");
    assert_eq!(t.hand_size(P0), 7);
    assert_eq!(t.graveyard_size(P0), 2, "discarded in two cleanup steps");
}

#[test]
fn may_pay_triggers_make_one_token_each() {
    cr!("603.5", "608.2d");
    ruling!(
        "Spirit Cairn",
        "You can’t pay {W} multiple times to create more than one token each time you resolve Spirit Cairn’s triggered ability."
    );
    ruling!(
        "Drake Haven",
        "While resolving Drake Haven’s triggered ability, you can’t pay {1} multiple times to get multiple Drakes."
    );
    supported("Drake Haven");
    for (name, land) in [("Spirit Cairn", "Plains"), ("Drake Haven", "Wastes")] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        t.lands(P0, land, 5);
        let card = t.hand(P0, "Grizzly Bears");
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        crate::r_p146_common::discard(&mut t, P0, card);
        t.resolve_all();
        assert_eq!(tokens(&t, P0).len(), 1, "{name}");
        assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 4, "{name}: paid once");
    }
}

// ---------------------------------------------------------------------------------
// Rielle, the Everwise.
// ---------------------------------------------------------------------------------

#[test]
fn rielle_trigger_after_the_cost_and_after_the_resolving_spell() {
    cr!("603.3", "601.2c", "601.2h", "608.2");
    ruling!(
        "Rielle, the Everwise",
        "If you discard a card as a cost to cast a spell or activate an ability, Rielle's triggered ability resolves before that spell or ability but after you've chosen targets for it. If you discard a card while a spell or ability is resolving, that spell or ability finishes resolving before Rielle's triggered ability does.'"
    );
    supported("Lightning Axe");
    supported("Faithless Looting");
    // Lightning Axe, discarding a card as its additional cost: the trigger goes on the
    // stack above the Axe, which already has its target.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rielle, the Everwise");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.hand(P0, "Grizzly Bears");
    let axe = cast_new(&mut t, P0, "Lightning Axe", &[obj(bears)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.g.stack[0], axe);
    assert_eq!(crate::r_s25_common::targets_of(&t, axe), vec![obj(bears)]);
    assert_eq!(t.hand_size(P0), 0);
    t.resolve();
    assert_eq!(t.hand_size(P0), 1, "Rielle drew first");
    assert!(on_stack(&t, axe) && t.on_battlefield(bears));
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    // Faithless Looting ("Draw two cards. Then discard two cards."): it finishes
    // resolving, then Rielle's trigger resolves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rielle, the Everwise");
    cast_new(&mut t, P0, "Faithless Looting", &[]);
    t.resolve();
    assert_eq!(t.hand_size(P0), 0, "drew two, discarded two");
    assert_eq!(t.stack_len(), 1, "Rielle's trigger waits");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn rielle_gets_its_bonus_only_on_the_battlefield() {
    cr!("113.6", "611.3a");
    ruling!(
        "Rielle, the Everwise",
        "Rielle's first ability applies only while it's on the battlefield."
    );
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P0, "Lava Spike");
    let in_hand = t.hand(P0, "Rielle, the Everwise");
    t.g.recompute();
    assert_eq!(t.obj_now(in_hand).chars.power, Some(0));
    let r = t.battlefield(P0, "Rielle, the Everwise");
    assert_eq!(t.pt(r).0, 2);
}

// ---------------------------------------------------------------------------------
// Glint-Horn Buccaneer: "Whenever you discard a card, this creature deals 1 damage to
// each opponent. {1}{R}, Discard a card: Draw a card. Activate only if this creature is
// attacking."
// ---------------------------------------------------------------------------------

#[test]
fn glint_horn_buccaneer_trigger_is_not_activated() {
    cr!("603.1", "602.5b", "701.9a");
    ruling!(
        "Glint-Horn Buccaneer",
        "The middle ability of Glint-Horn Buccaneer is a triggered ability, not an activated ability. It doesn’t allow you to discard a card whenever you want; rather, you need some other way of discarding a card, such as by activating its last ability."
    );
    supported("Glint-Horn Buccaneer");
    let mut t = TestGame::new(2);
    let gb = t.battlefield(P0, "Glint-Horn Buccaneer");
    t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    // Its only activated ability needs it to be attacking.
    assert_eq!(
        t.g.obj(gb)
            .chars
            .abilities
            .iter()
            .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
            .count(),
        1
    );
    assert!(t.activate(P0, gb, 0, &[]).is_err(), "not attacking");
    // Discarding some other way triggers it: P1's Mind Rot.
    let mut t2 = TestGame::new(2);
    t2.battlefield(P0, "Glint-Horn Buccaneer");
    t2.hand(P0, "Grizzly Bears");
    t2.hand(P0, "Grizzly Bears");
    mind_rot(&mut t2);
    t2.resolve_all();
    assert_eq!(t2.life(P1), 18);
    // Attacking, its ability discards a card: the trigger deals 1 damage.
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(gb, player(P1))]);
    t.activate(P0, gb, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn glint_horn_buccaneer_in_two_headed_giant() {
    cr!("810.4", "810.9", "120.3a");
    ruling!(
        "Glint-Horn Buccaneer",
        "In a Two-Headed Giant game, Glint-Horn Buccaneer’s triggered ability causes the opposing team to be dealt 1 damage twice."
    );
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    let gb = t.battlefield(P0, "Glint-Horn Buccaneer");
    let card = t.hand(P0, "Grizzly Bears");
    let life = t.life(P2);
    let from = t.g.turn_events.len();
    crate::r_p146_common::discard(&mut t, P0, card);
    t.resolve_all();
    let hits: Vec<Entity> = damage_events(&t, from)
        .into_iter()
        .filter(|d| d.0 == gb)
        .map(|d| d.1)
        .collect();
    assert_eq!(hits.len(), 2, "1 damage to each opponent");
    assert!(hits.contains(&player(P2)) && hits.contains(&player(P3)));
    assert_eq!(t.life(P2), life - 2, "the team's shared life total");
    assert_eq!(t.life(P3), t.life(P2));
}

// ---------------------------------------------------------------------------------
// Guerrilla Tactics: "When a spell or ability an opponent controls causes you to discard
// this card, it deals 4 damage to any target."
// ---------------------------------------------------------------------------------

#[test]
fn guerrilla_tactics_deals_damage_as_a_card_with_an_ability() {
    cr!("113.6k", "113.7a", "120.3");
    ruling!(
        "Guerrilla Tactics",
        "If it is discarded, the damage done by it is done by this card due to an ability and not by a spell."
    );
    supported("Guerrilla Tactics");
    let mut t = TestGame::new(2);
    t.hand(P0, "Guerrilla Tactics");
    t.answer_targets(P0, &[player(P1)]);
    mind_rot(&mut t);
    // The trigger: an ability whose source is the card in the graveyard.
    assert_eq!(t.stack_len(), 1);
    let ab = *t.g.stack.last().unwrap();
    let card = t.g.player(P0).graveyard[0];
    assert!(matches!(
        t.g.obj(ab).stack.as_deref().map(|s| &s.kind),
        Some(StackKind::Triggered { source, .. }) if *source == card
    ));
    let from = t.g.turn_events.len();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    let dmg = damage_events(&t, from);
    assert_eq!(dmg.len(), 1);
    assert_eq!(dmg[0].0, card, "dealt by the card");
    assert_eq!(t.g.obj(card).zone, Zone::Graveyard(P0), "not a spell");
}

// ---------------------------------------------------------------------------------
// Ignorant Bliss: "Exile all cards from your hand face down. At the beginning of the next
// end step, return those cards to your hand, then draw a card."
// ---------------------------------------------------------------------------------

#[test]
fn ignorant_bliss_cards_stay_exiled_if_the_delayed_trigger_is_countered() {
    cr!("603.7", "701.6b", "406.3");
    ruling!(
        "Ignorant Bliss",
        "If the delayed triggered ability is countered (for example, with Voidslime), the cards remain exiled forever."
    );
    ruling!(
        "Ignorant Bliss",
        "No player may look at the cards while they’re face down."
    );
    supported("Ignorant Bliss");
    supported("Voidslime");
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Craw Wurm");
    cast_resolve(&mut t, P0, "Ignorant Bliss", &[]);
    assert_eq!(t.hand_size(P0), 0);
    let gone: Vec<ObjectId> = t
        .g
        .exile
        .iter()
        .copied()
        .filter(|id| t.g.obj(*id).owner == P0)
        .collect();
    assert_eq!(gone.len(), 2);
    for id in &gone {
        assert!(t.g.obj(*id).face_down);
        assert!(!mtg_engine::facedown::can_look_at(&t.g, P0, *id));
        assert!(!mtg_engine::facedown::can_look_at(&t.g, P1, *id));
    }
    // The end step: P1 counters the delayed trigger with Voidslime.
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let trig = *t.g.stack.last().unwrap();
    cast_new(&mut t, P1, "Voidslime", &[obj(trig)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    // Later end steps don't return them.
    t.advance_to(P1, Step::End);
    t.resolve_all();
    t.advance_to(P0, Step::Upkeep);
    assert!(gone.iter().all(|id| t.g.obj(*id).zone == Zone::Exile && t.g.is_live(*id)));
}

#[test]
fn ignorant_bliss_cast_in_the_end_step_waits_for_the_next_one() {
    cr!("603.7", "603.7b", "513.1");
    ruling!(
        "Ignorant Bliss",
        "If you cast this during the End step, you won’t return the cards to your hand or draw a card until the next turn’s End step."
    );
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.settle();
    cast_resolve(&mut t, P0, "Ignorant Bliss", &[]);
    assert_eq!(t.hand_size(P0), 0);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.hand_size(P0), 0, "not this end step");
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2, "the Bears and a drawn card");
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

// ---------------------------------------------------------------------------------
// Bottled Cloister.
// ---------------------------------------------------------------------------------

#[test]
fn bottled_cloister_returns_only_cards_its_controller_owns() {
    cr!("607.2a", "108.3", "110.2");
    ruling!(
        "Bottled Cloister",
        "Errata’d to make it clear that only the cards you own go into your hand, so if you somehow gain control of the Cloister, your opponent’s cards won’t go to your hand."
    );
    supported("Bottled Cloister");
    let mut t = TestGame::new(2);
    let cloister = t.battlefield(P0, "Bottled Cloister");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Craw Wurm");
    // P1's upkeep: P0's hand is exiled face down.
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    // P1 gains control of the Cloister.
    crate::r_s06_common::give_control(&mut t, cloister, P1);
    t.hand(P1, "Lightning Bolt");
    // P0's upkeep (P1's opponent): P1's hand is exiled.
    let p1_cards = t.hand_size(P1);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    // P1's upkeep: P1's own cards return and P1 draws; P0's cards stay exiled.
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_hand(P1, "Lightning Bolt"));
    // (P1's cards: those in hand at its first upkeep and the card from its draw step.)
    assert_eq!(t.hand_size(P1), p1_cards + 2, "P1's cards and a drawn card");
    assert!(!t.in_hand(P1, "Grizzly Bears") && !t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(
        t.g.exile
            .iter()
            .filter(|id| t.g.obj(**id).owner == P0)
            .count(),
        2
    );
}

// ---------------------------------------------------------------------------------
// Madness cards.
// ---------------------------------------------------------------------------------

#[test]
fn nagging_thoughts_with_one_card_left() {
    cr!("704.5b", "121.4");
    ruling!(
        "Nagging Thoughts",
        "If there is only one card left in your library, you'll put it into your hand. You won't lose the game for having zero cards in your library until you're required to draw a card."
    );
    supported("Nagging Thoughts");
    let mut t = TestGame::new(2);
    t.g.players[0].library.clear();
    t.library_top(P0, "Craw Wurm");
    cast_resolve(&mut t, P0, "Nagging Thoughts", &[]);
    assert!(t.in_hand(P0, "Craw Wurm"));
    assert_eq!(t.library_size(P0), 0);
    t.settle();
    assert!(!t.has_lost(P0));
}

#[test]
fn biting_rain_affects_only_creatures_there_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Biting Rain",
        "The set of creatures affected by Biting Rain is determined as the spell resolves. Creatures that enter the battlefield later in the turn won't get -2/-2."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_resolve(&mut t, P0, "Biting Rain", &[]);
    assert_eq!(t.pt(wurm), (4, 2));
    let later = t.battlefield(P1, "Craw Wurm");
    assert_eq!(t.pt(later), (6, 4));
}

#[test]
fn insatiable_gorgers_attacks_only_when_able_and_without_paying() {
    cr!("508.1d", "508.1h", "302.6");
    ruling!(
        "Insatiable Gorgers",
        "If Insatiable Gorgers hasn't been under your control since the turn began, is tapped, or is affected by a spell or ability that says it can't attack, then it doesn't attack. If there's a cost associated with having it attack, you aren't forced to pay that cost, so it doesn't have to attack in that case either."
    );
    supported("Insatiable Gorgers");
    supported("Ghostly Prison");
    supported("Pacifism");
    /// P0's combat with no attackers chosen: P1's life after it.
    fn combat(setup: fn(&mut TestGame, ObjectId)) -> i32 {
        let mut t = TestGame::new(2);
        let g = t.battlefield(P0, "Insatiable Gorgers");
        setup(&mut t, g);
        t.lands(P0, "Wastes", 2);
        t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![]));
        t.advance_to(P0, Step::EndOfCombat);
        t.life(P1)
    }
    assert_eq!(combat(|_, _| {}), 15, "it must attack");
    assert_eq!(combat(|t, g| t.g.obj_mut(g).summoning_sick = true), 20);
    assert_eq!(combat(|t, g| t.g.obj_mut(g).tapped = true), 20);
    assert_eq!(
        combat(|t, g| {
            crate::r_s06_common::attach_new(t, P1, "Pacifism", g);
        }),
        20
    );
    assert_eq!(
        combat(|t, _| {
            t.battlefield(P1, "Ghostly Prison");
        }),
        20,
        "not forced to pay {{2}}"
    );
}

// ---------------------------------------------------------------------------------
// Nullhide Ferox: "{2}: This creature loses all abilities until end of turn. Any player
// may activate this ability."
// ---------------------------------------------------------------------------------

fn ferox_activatable(t: &mut TestGame, p: PlayerId, ferox: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let f = t.g.current(ferox);
    t.g.activatable_abilities(p).iter().any(|(s, _)| *s == f)
}

#[test]
fn nullhide_ferox_ability_only_on_the_battlefield_and_once() {
    cr!("602.1", "113.6", "613.1f");
    ruling!(
        "Nullhide Ferox",
        "Nullhide Ferox's activated ability can be activated only while it's on the battlefield. Players can't, for example, activate it to try to stop its replacement effect from putting it onto the battlefield."
    );
    ruling!(
        "Nullhide Ferox",
        "Once Nullhide Ferox has lost its activated ability that makes it lose its abilities, that ability can't be activated again until the turn's over."
    );
    ruling!(
        "Nullhide Ferox",
        "Once Nullhide Ferox's activated ability has resolved, it can gain new abilities. It won't lose those abilities."
    );
    supported("Nullhide Ferox");
    // In hand: no player can activate it, and an opponent's Mind Rot puts it onto the
    // battlefield.
    let mut t = TestGame::new(2);
    let ferox = t.hand(P0, "Nullhide Ferox");
    t.lands(P0, "Wastes", 2);
    t.lands(P1, "Wastes", 4);
    assert!(!ferox_activatable(&mut t, P0, ferox));
    assert!(!ferox_activatable(&mut t, P1, ferox));
    mind_rot(&mut t);
    t.resolve_all();
    let ferox = t.named_on_battlefield("Nullhide Ferox")[0];
    // On the battlefield: P1 (any player) activates it.
    assert!(ferox_activatable(&mut t, P1, ferox));
    t.activate(P1, ferox, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(ferox).chars.abilities.is_empty());
    assert!(!ferox_activatable(&mut t, P1, ferox), "it lost that ability too");
    // New abilities afterwards stay.
    cast_resolve(&mut t, P1, "Jump", &[obj(ferox)]);
    assert!(t
        .obj_now(ferox)
        .has_keyword(mtg_engine::keywords::KeywordKind::Flying));
}
