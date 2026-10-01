//! Rulings batch P017 — coin flips (CR 705): flips happen as the spell or ability
//! resolves, after targets were chosen (CR 601.2c, 608.2b); coins flipped at once are
//! all replaced by Krark's Thumb and all fixed by Edgar (CR 705.3, 614.1a).

use crate::r_p017_common::*;
use crate::r_s01_common::supported;
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Ral Zarek with seven loyalty counters, ready to activate its −7.
fn ral_at_seven(t: &mut TestGame) -> ObjectId {
    supported("Ral Zarek");
    let ral = t.battlefield(P0, "Ral Zarek");
    let have = t.counters(ral, counters::LOYALTY);
    t.g.add_counters(Entity::Object(ral), counters::LOYALTY, 7 - have, None);
    t.settle();
    ral
}

/// Activates Ral Zarek's −7 ("Flip five coins. Take an extra turn after this one for
/// each coin that comes up heads.") and resolves it.
fn ral_ultimate(t: &mut TestGame, ral: ObjectId) {
    t.activate(P0, ral, 2, &[]).expect("activate the -7");
    t.resolve_all();
}

#[test]
fn krarks_thumb_replaces_each_of_several_coins_flipped_at_once() {
    cr!("705.1", "614.1a");
    ruling!(
        "Krark's Thumb",
        "If an effect tells you to flip more than one coin at once, this replaces each individual coin flip."
    );
    supported("Krark's Thumb");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Krark's Thumb");
    let ral = ral_at_seven(&mut t);
    // Five pairs of coins: P0 keeps heads from each pair that has one.
    load_coins(
        &mut t,
        &[
            false, true, false, true, true, false, false, false, true, true,
        ],
    );
    let from = t.asked().len();
    ral_ultimate(&mut t, ral);
    assert!(t.g.dice.loaded_coins.is_empty(), "ten coins flipped");
    assert_eq!(
        flip_results(&t),
        vec![true, true, true, false, true],
        "five flips happened"
    );
    assert_eq!(t.g.extra_turns.len(), 4);
    // P0 chose which to ignore once for each of the five pairs.
    let keeps = t.asked()[from..]
        .iter()
        .filter(|(_, d)| {
            matches!(d, mtg_engine::decision::Decision::ChooseOption { prompt, .. } if prompt == "Choose the flip to keep")
        })
        .count();
    assert_eq!(keeps, 5);
    // Coins flipped at once (with calls) are all called and flipped before P0 chooses
    // which flips to ignore.
    let mut spec = mtg_engine::dice::CoinFlip::new();
    spec.count = mtg_engine::ability::Value::c(2);
    load_coins(&mut t, &[true, false, false, true]);
    let from = t.asked().len();
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    t.g.exec(
        &mtg_engine::ability::Effect::FlipCoins(Box::new(spec)),
        &mut ctx,
    );
    let prompts: Vec<String> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseOption { prompt, .. } => Some(prompt.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        prompts,
        vec![
            "Call the coin flip",
            "Call the coin flip",
            "Choose the flip to keep",
            "Choose the flip to keep"
        ]
    );
}

#[test]
fn edgar_fixes_every_coin_of_the_first_set_flipped_in_a_turn() {
    cr!("705.3");
    ruling!(
        "Edgar, King of Figaro",
        "If an effect tells you to flip multiple coins at once and you haven't flipped one or more coins already during that turn, Edgar's last ability modifies that set of flips."
    );
    supported("Edgar, King of Figaro");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Edgar, King of Figaro");
    let ral = ral_at_seven(&mut t);
    load_coins(&mut t, &[false; 5]);
    ral_ultimate(&mut t, ral);
    assert_eq!(flip_results(&t), vec![true; 5]);
    assert_eq!(t.g.extra_turns.len(), 5);
}

#[test]
fn goblin_bangchuckers_targets_on_activation_and_flips_on_resolution() {
    cr!("602.2b", "601.2c", "608.2b", "705.2");
    ruling!(
        "Goblin Bangchuckers",
        "You choose the target when you activate the ability. You don’t flip the coin until the ability resolves. Players may respond to the ability, but they won’t know the results of the coin flip."
    );
    ruling!(
        "Goblin Bangchuckers",
        "If the permanent or player is an illegal target when Goblin Bangchuckers’ ability tries to resolve, it won’t resolve and none of its effects will happen. You won’t flip a coin and no damage will be dealt."
    );
    supported("Goblin Bangchuckers");
    let mut t = TestGame::new(2);
    let bang = t.battlefield(P0, "Goblin Bangchuckers");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, bang, 0, &[Entity::Object(bears)])
        .expect("activate");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert!(flip_results(&t).is_empty());
    // In response, the target leaves the battlefield: the ability doesn't resolve.
    t.g.destroy(bears, None);
    t.settle();
    load_coins(&mut t, &[false]);
    t.resolve_all();
    assert!(flip_results(&t).is_empty());
    assert_eq!(t.obj_now(bang).damage, 0);
    assert_eq!(t.g.dice.loaded_coins.len(), 1);
}

#[test]
fn orcish_captain_targets_on_activation_and_flips_on_resolution() {
    cr!("602.2b", "608.2b", "705.2");
    ruling!(
        "Orcish Captain",
        "You choose the target when you activate the ability. You don’t flip a coin until the ability resolves."
    );
    supported("Orcish Captain");
    let mut t = TestGame::new(2);
    let captain = t.battlefield(P0, "Orcish Captain");
    t.lands(P0, "Mountain", 1);
    t.activate(P0, captain, 0, &[Entity::Object(captain)])
        .expect("activate");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert!(flip_results(&t).is_empty());
    // P0 calls heads, and it comes up heads.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    load_coins(&mut t, &[true]);
    t.resolve_all();
    assert_eq!(flip_results(&t), vec![true]);
    assert_eq!(t.pt(captain), (3, 1));
}

#[test]
fn aleatory_targets_on_casting_and_flips_on_resolution() {
    cr!("601.2c", "608.2b", "705.2");
    ruling!(
        "Aleatory",
        "You pick the target on announcement and flip the coin on resolution."
    );
    supported("Aleatory");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    let spell = t.hand(P0, "Aleatory");
    let r = t.cast(P0, spell).target(bears).try_go();
    assert!(r.is_ok(), "{r:?}");
    t.settle();
    assert!(flip_results(&t).is_empty());
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    load_coins(&mut t, &[true]);
    t.resolve_all();
    assert_eq!(flip_results(&t), vec![true]);
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn boompile_destroys_itself_along_with_everything_else() {
    cr!("705.2", "608.2h");
    ruling!(
        "Boompile",
        "If you win the flip, Boompile is destroyed along with all the other nonland permanents."
    );
    ruling!(
        "Boompile",
        "You flip a coin as Boompile’s ability resolves. No player may take actions between seeing the result of the flip and all nonland permanents being destroyed."
    );
    supported("Boompile");
    let mut t = TestGame::new(2);
    let pile = t.battlefield(P0, "Boompile");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let land = t.battlefield(P1, "Forest");
    t.activate(P0, pile, 0, &[]).expect("activate");
    t.settle();
    assert!(flip_results(&t).is_empty());
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    load_coins(&mut t, &[true]);
    // The ability resolves in one go: the flip and the destruction.
    t.g.resolve_top();
    assert_eq!(flip_results(&t), vec![true]);
    assert!(!t.on_battlefield(pile) && !t.on_battlefield(bears));
    assert!(t.on_battlefield(land));
    assert!(t.in_graveyard(P0, "Boompile"));
}

#[test]
fn molten_birth_goes_from_the_stack_to_its_owners_hand() {
    cr!("705.2", "608.2n");
    ruling!(
        "Molten Birth",
        "If you win the flip, Molten Birth goes directly from the stack to its owner’s hand. It never goes to a graveyard."
    );
    supported("Molten Birth");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let spell = t.hand(P0, "Molten Birth");
    t.cast(P0, spell).go();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    load_coins(&mut t, &[true]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Molten Birth"));
    assert_eq!(t.graveyard_size(P0), 0);
    let to_graveyard = t.turn_events.iter().any(|e| {
        matches!(
            e,
            Event::ZoneChange {
                to: Zone::Graveyard(_),
                from: Zone::Stack,
                ..
            }
        )
    });
    assert!(!to_graveyard);
    assert_eq!(t.g.permanents().filter(|o| o.is_token()).count(), 2);
}

/// P1's `attacker` attacks and P0's Creepy Doll blocks it; returns once combat damage
/// was dealt, with Creepy Doll's trigger on the stack.
fn doll_blocks(t: &mut TestGame, attacker: ObjectId) -> ObjectId {
    supported("Creepy Doll");
    let doll = t.battlefield(P0, "Creepy Doll");
    t.set_step(P1, Step::BeginningOfCombat);
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(attacker, Entity::Player(P0))]),
    );
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(doll, attacker)]),
    );
    t.advance_to(P1, Step::CombatDamage);
    t.settle();
    doll
}

#[test]
fn creepy_doll_flips_even_if_its_damage_was_lethal() {
    cr!("603.2", "701.19a", "705.2");
    ruling!(
        "Creepy Doll",
        "If the combat damage Creepy Doll deals to a creature is lethal, you’ll still flip a coin. If the creature is still on the battlefield (perhaps because it regenerated), it could be destroyed a second time"
    );
    supported("Drudge Skeletons");
    let mut t = TestGame::new(2);
    let skeletons = t.battlefield(P1, "Drudge Skeletons");
    t.lands(P1, "Swamp", 1);
    t.activate(P1, skeletons, 0, &[]).expect("regenerate");
    t.resolve_all();
    let _doll = doll_blocks(&mut t, skeletons);
    // The lethal damage destroyed it, but it regenerated; the trigger still flips.
    assert!(t.on_battlefield(skeletons));
    assert_eq!(t.stack_len(), 1);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    load_coins(&mut t, &[true]);
    t.resolve_all();
    assert_eq!(flip_results(&t), vec![true]);
    assert!(!t.on_battlefield(skeletons));
    assert!(t.in_graveyard(P1, "Drudge Skeletons"));
}

#[test]
fn creepy_doll_flips_on_resolution_so_responses_come_first() {
    cr!("603.3", "608.2b", "705.2", "701.19a");
    ruling!(
        "Creepy Doll",
        "You don’t flip the coin until the ability resolves. If you want to respond to the ability, perhaps by regenerating the damaged creature, you’ll have to do so before you know the outcome of the flip."
    );
    supported("Cudgel Troll");
    let mut t = TestGame::new(2);
    let troll = t.battlefield(P1, "Cudgel Troll");
    t.lands(P1, "Forest", 1);
    doll_blocks(&mut t, troll);
    assert_eq!(t.stack_len(), 1);
    assert!(flip_results(&t).is_empty());
    // P1 regenerates the troll in response, not knowing the result.
    t.activate(P1, troll, 0, &[]).expect("regenerate");
    t.resolve();
    assert!(flip_results(&t).is_empty());
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    load_coins(&mut t, &[true]);
    t.resolve_all();
    assert_eq!(flip_results(&t), vec![true]);
    assert!(t.on_battlefield(troll));
    assert!(t.obj_now(troll).tapped);
}

#[test]
fn krarks_thumb_sees_the_opponents_simultaneous_flip_before_choosing() {
    cr!("705.1", "614.1a", "101.4");
    ruling!(
        "Krark's Thumb",
        "If you and your opponent both flip at the same time, you can see your opponent's result before choosing which result to keep."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Krark's Thumb");
    // "Each player flips a coin."
    let mut spec = mtg_engine::dice::CoinFlip::new();
    spec.who = mtg_engine::ability::PlayerRef::EachPlayer;
    load_coins(&mut t, &[true, false, true]);
    let from = t.asked().len();
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    t.g.exec(
        &mtg_engine::ability::Effect::FlipCoins(Box::new(spec)),
        &mut ctx,
    );
    let asked: Vec<(PlayerId, String)> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            mtg_engine::decision::Decision::ChooseOption { prompt, .. } => {
                Some((*p, prompt.clone()))
            }
            _ => None,
        })
        .collect();
    let keep = asked
        .iter()
        .position(|(p, x)| *p == P0 && x == "Choose the flip to keep")
        .expect("P0 chooses a flip to keep");
    let p1_call = asked
        .iter()
        .position(|(p, x)| *p == P1 && x == "Call the coin flip")
        .expect("P1 calls its flip");
    assert!(p1_call < keep, "{asked:?}");
    t.g.flush_events();
    // All three coins were flipped: two for P0, one for P1.
    assert!(t.g.dice.loaded_coins.is_empty());
    assert_eq!(flip_results(&t).len(), 2);
}
