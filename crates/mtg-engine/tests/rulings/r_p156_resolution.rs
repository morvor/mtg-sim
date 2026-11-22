//! Rulings batch P156 — more cards with scry (grouped by the scry function): what happens
//! as a spell or ability resolves (die rolls after targets, choices that don't target,
//! sources that left, intervening "if" clauses, revealed cards that are drawn), copies of
//! spells (division, kicker and alternative costs carried over, no mana spent), and modes.

use crate::r_p062_common::is_tapped;
use crate::r_s01_common::*;
use crate::r_s03_common::run_effect;
use crate::r_s05_common::{enter, move_to};
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::{cast_new, lands_for_cost, targets_of};
use crate::r_s29_common::{choose_modes, divide, put_counters};
use mtg_engine::ability::{Effect, Sel, Value};
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn dice_rolled(t: &TestGame) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, Event::DieRolled { .. }))
        .count()
}

fn asked_count(t: &TestGame, from: usize, f: fn(&Decision) -> bool) -> usize {
    asked_since(t, from).iter().filter(|(_, d)| f(d)).count()
}

/// Copies the spell `spell` (as a resolving "copy target spell" would), keeping its targets.
fn copy_spell(t: &mut TestGame, spell: ObjectId) {
    t.answer_yes(P0, false);
    run_effect(
        t,
        None,
        P0,
        Effect::CopySpell {
            what: Sel::Target(0),
            count: Value::c(1),
            new_targets: true,
        },
        &[obj(spell)],
    );
}

/// The non-normal casting method `p` has for `card` (an alternative cost, aftermath, ...).
fn other_method(t: &mut TestGame, p: PlayerId, card: ObjectId) -> CastMethod {
    crate::r_s08_common::legal_cast_methods(t, p, card)
        .into_iter()
        .find(|m| *m != CastMethod::Normal)
        .expect("another casting method")
}

#[test]
fn choose_a_target_then_roll_rolls_as_it_resolves() {
    cr!("601.2c", "608.2", "706.1");
    ruling!(
        "Power of Persuasion",
        "An effect that says “choose a target, then roll a d20” or similar still uses the normal process of putting an ability on the stack and resolving it. Choosing targets is part of putting the ability on the stack and rolling the d20 happens later, as the ability resolves."
    );
    ruling!(
        "Scion of Stygia",
        "An effect that says “choose a target, then roll a d20” or similar still uses the normal process of putting an ability on the stack and resolving it. Choosing targets is part of putting the ability on the stack and rolling the d20 happens later, as the ability resolves."
    );
    supported("Power of Persuasion");
    supported("Scion of Stygia");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = cast_new(&mut t, P0, "Power of Persuasion", &[obj(bears)]);
    assert_eq!(targets_of(&t, spell), vec![obj(bears)]);
    assert_eq!(dice_rolled(&t), 0);
    t.resolve_all();
    assert_eq!(dice_rolled(&t), 1);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    enter(&mut t, P0, "Scion of Stygia");
    let trig = *t.g.stack.last().unwrap();
    assert_eq!(targets_of(&t, trig), vec![obj(bears)]);
    assert_eq!(dice_rolled(&t), 0);
    t.resolve_all();
    assert_eq!(dice_rolled(&t), 1);
    assert!(is_tapped(&t, bears));
}

#[test]
fn ignored_extra_dice_were_never_rolled() {
    cr!("706.2", "706.6", "614.1a");
    ruling!(
        "Contact Other Plane",
        "Some abilities, like that of Pixie Guide and Barbarian Class, replace rolling a die with rolling extra dice and ignoring the lowest roll. The ignored rolls are not considered for the effect that instructed you to roll a die, and do not cause abilities to trigger. For all intents and purposes, once you determine which dice count, any extra dice were never rolled."
    );
    supported("Contact Other Plane");
    supported("Pixie Guide");
    supported("Barbarian Class");
    for helper in ["Pixie Guide", "Barbarian Class"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, helper);
        let hand = t.hand_size(P0);
        cast_new(&mut t, P0, "Contact Other Plane", &[]);
        t.resolve_all();
        // One die counts: one roll event, and the card drew 2 or 3 cards once.
        assert_eq!(dice_rolled(&t), 1, "{helper}");
        let drawn = t.hand_size(P0) + 1 - hand;
        assert!(drawn == 2 || drawn == 3, "{helper}: drew {drawn}");
    }
}

#[test]
fn believe_puts_the_card_into_hand_unrevealed_unless_put_onto_the_battlefield() {
    cr!("702.127a", "701.20a");
    ruling!(
        "Reason // Believe",
        "For Believe, you put the top card of your library into your hand if you don't put it onto the battlefield for any reason, whether it's not a creature card or whether you just didn't want to put it onto the battlefield."
    );
    ruling!(
        "Reason // Believe",
        "If you don't put the top card of your library onto the battlefield, you don't reveal it before putting it into your hand."
    );
    supported("Reason // Believe");
    for (top, put) in [("Grizzly Bears", false), ("Island", true), ("Grizzly Bears", true)] {
        let mut t = TestGame::new(2);
        let card = t.library_top(P0, top);
        let believe = t.graveyard(P0, "Reason // Believe");
        t.lands(P0, "Forest", 1);
        t.lands(P0, "Wastes", 4);
        let m = other_method(&mut t, P0, believe);
        t.answer_yes(P0, put);
        t.cast(P0, believe).method(m).go();
        t.resolve_all();
        let creature = top == "Grizzly Bears";
        if creature && put {
            assert!(t.on_battlefield(card));
        } else {
            assert_eq!(t.zone(card), Zone::Hand(P0), "{top} {put}");
            let revealed = t.g.turn_events.iter().any(|e| {
                matches!(e, Event::Custom { name, obj: Some(o), .. }
                    if name == mtg_engine::reveal::REVEALED && t.g.current(*o) == t.g.current(card))
            });
            assert!(!revealed, "{top} {put}: revealed");
        }
    }
}

/// Untaps P0's permanents (as P0's next untap step would, later this game).
fn untap_all(t: &mut TestGame) {
    for id in t.g.battlefield.clone() {
        if t.g.obj(id).controller == P0 {
            t.g.objects[id.0 as usize].tapped = false;
        }
    }
}

#[test]
fn glamorous_outlaws_granted_mana_ability() {
    cr!("611.2a", "605.1a", "601.2g");
    ruling!(
        "Glamorous Outlaw",
        "If Glamorous Outlaw is removed from exile without being cast, the land will continue to have this ability.."
    );
    ruling!(
        "Glamorous Outlaw",
        "You may use the mana ability Glamorous Outlaw grants to the land while casting Glamorous Outlaw from exile."
    );
    supported("Glamorous Outlaw");
    let setup = || {
        let mut t = TestGame::new(2);
        let land = t.battlefield(P0, "Wastes");
        t.lands(P0, "Wastes", 2);
        let outlaw = t.hand(P0, "Glamorous Outlaw");
        t.answer_targets(P0, &[obj(land)]);
        activate_containing(&mut t, P0, outlaw, "Exile ~ from your hand").expect("activate");
        t.resolve_all();
        assert_eq!(t.zone(outlaw), Zone::Exile);
        (t, land, outlaw)
    };
    // Removed from exile without being cast: the land keeps the ability.
    let (mut t, land, outlaw) = setup();
    move_to(&mut t, outlaw, Zone::Graveyard(P0));
    untap_all(&mut t);
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P0, land, "{R}"));
    // Cast from exile, paying {R} with the land's granted ability.
    let (mut t, land, outlaw) = setup();
    untap_all(&mut t);
    t.lands(P0, "Wastes", 3);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    let outlaw = t.g.current(outlaw);
    let m = crate::r_s08_common::legal_cast_methods(&mut t, P0, outlaw)
        .into_iter()
        .next()
        .expect("castable from exile with the land's mana");
    t.cast(P0, outlaw).method(m).go();
    assert!(is_tapped(&t, land));
    t.resolve_all();
    assert!(!t.named_on_battlefield("Glamorous Outlaw").is_empty());
}

#[test]
fn graven_lore_without_snow_mana_or_copied_just_draws_three() {
    cr!("107.4h", "707.10", "601.2h");
    ruling!(
        "Graven Lore",
        "If you cast Graven Lore without spending snow mana, you’ll just draw three cards."
    );
    ruling!(
        "Graven Lore",
        "If Graven Lore is copied, no mana was spent to cast the copy, so you’ll just draw three cards."
    );
    supported("Graven Lore");
    let is_scry = |d: &Decision| matches!(d, Decision::Scry { .. });
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    cast_new(&mut t, P0, "Graven Lore", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
    assert_eq!(asked_count(&t, from, is_scry), 0);
    // Cast with five snow mana, then copied: the copy draws three without scrying.
    let mut t = TestGame::new(2);
    t.lands(P0, "Snow-Covered Island", 5);
    let card = t.hand(P0, "Graven Lore");
    let hand = t.hand_size(P0);
    let spell = t.cast(P0, card).go();
    copy_spell(&mut t, spell);
    let from = t.asked().len();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand - 1 + 3);
    assert_eq!(asked_count(&t, from, is_scry), 0, "the copy scried");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 6);
    assert_eq!(asked_count(&t, from, is_scry), 1, "the original scries");
}

#[test]
fn a_source_that_left_before_its_trigger_resolved() {
    cr!("610.3c", "400.7", "608.2c");
    ruling!(
        "Mazemind Tome",
        "If Mazemind Tome leaves the battlefield while its triggered ability is on the stack, you can't exile it from the zone it's put into."
    );
    ruling!(
        "Prison Realm",
        "If Prison Realm leaves the battlefield before its first triggered ability resolves, the target permanent won’t be exiled."
    );
    supported("Mazemind Tome");
    supported("Prison Realm");
    for leave in [true, false] {
        let mut t = TestGame::new(2);
        let tome = t.battlefield(P0, "Mazemind Tome");
        put_counters(&mut t, tome, "page", 4);
        assert_eq!(triggers_on_stack(&t, "four or more"), 1);
        if leave {
            move_to(&mut t, tome, Zone::Graveyard(P0));
        }
        t.resolve_all();
        if leave {
            assert_eq!(t.zone(tome), Zone::Graveyard(P0));
            assert_eq!(t.life(P0), 20);
        } else {
            assert_eq!(t.zone(tome), Zone::Exile);
            assert_eq!(t.life(P0), 24);
        }
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        let realm = enter(&mut t, P0, "Prison Realm");
        if leave {
            move_to(&mut t, realm, Zone::Graveyard(P0));
        }
        t.resolve_all();
        assert_eq!(t.on_battlefield(bears), leave, "Prison Realm left: {leave}");
    }
}

#[test]
fn rage_of_purphoros_stops_regeneration_even_if_the_damage_is_prevented() {
    cr!("701.19c", "615.1", "608.2c");
    ruling!(
        "Rage of Purphoros",
        "If Rage of Purphoros resolves but the damage is redirected to a different creature or to a player, the target creature still won't be able to regenerate that turn. This is also true if the damage is prevented."
    );
    supported("Rage of Purphoros");
    supported("Mending Hands");
    supported("Drudge Skeletons");
    for rage in [true, false] {
        let mut t = TestGame::new(2);
        let sk = t.battlefield(P1, "Drudge Skeletons");
        t.lands(P1, "Swamp", 1);
        if rage {
            cast_new(&mut t, P0, "Rage of Purphoros", &[obj(sk)]);
            // P1 prevents the next 4 damage to it in response.
            cast_new(&mut t, P1, "Mending Hands", &[obj(sk)]);
            t.resolve_all();
            assert!(t.on_battlefield(sk));
            assert_eq!(t.obj_now(sk).damage, 0);
        }
        activate_containing(&mut t, P1, sk, "Regenerate").expect("regenerate");
        t.resolve_all();
        crate::r_s02_common::destroy(&mut t, sk);
        assert_eq!(t.on_battlefield(sk), !rage, "Rage cast: {rage}");
    }
}

#[test]
fn so_shiny_intervening_if_and_as_a_token() {
    cr!("603.4", "111.1");
    ruling!(
        "So Shiny",
        "So Shiny's triggered ability checks whether you control a token after it enters the battlefield to see if it triggers. If you don't, the ability won't trigger at all. The ability will check again as it tries to resolve. If you don't control a token at that time, the ability won't do anything. It doesn't have to be the same token for each check, however."
    );
    ruling!(
        "So Shiny",
        "If So Shiny is itself a token, its enters-the-battlefield ability will trigger."
    );
    supported("So Shiny");
    // (token at entry, swap tokens before it resolves, keep a token) -> tapped?
    for (token, swap, keep, tapped) in [
        (false, false, false, false),
        (true, true, true, true),
        (true, false, false, false),
    ] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let tok = token.then(|| crate::r_s02_common::create_token(&mut t, P0, "Soldier"));
        t.answer_choose(P0, &[obj(bears)]);
        enter(&mut t, P0, "So Shiny");
        assert_eq!(
            triggers_on_stack(&t, "token"),
            token as usize,
            "{token} {swap} {keep}"
        );
        if let Some(tok) = tok {
            move_to(&mut t, tok, Zone::Graveyard(P0));
            if swap && keep {
                crate::r_s02_common::create_token(&mut t, P0, "Soldier");
            }
        }
        t.resolve_all();
        assert_eq!(is_tapped(&t, bears), tapped, "{token} {swap} {keep}");
    }
    // A token copy of So Shiny: it's the token you control.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = t.exile(P0, "So Shiny");
    t.answer_choose(P0, &[obj(bears)]);
    let made = crate::r_s17_common::token_copy(&mut t, P0, card);
    assert_eq!(made.len(), 1);
    assert_eq!(triggers_on_stack(&t, "token"), 1);
    t.resolve_all();
    assert!(is_tapped(&t, bears));
}

/// Puts Gadwick's First Duel onto the battlefield and resolves chapter III.
fn gadwick_third_chapter(t: &mut TestGame) {
    let saga = t.battlefield(P0, "Gadwick's First Duel");
    let saga = t.g.current(saga);
    t.g.objects[saga.0 as usize]
        .counters
        .insert("lore".into(), 2);
    put_counters(t, saga, "lore", 1);
    t.resolve_all();
}

#[test]
fn gadwicks_first_duel_copies_keep_division_targets_and_paid_costs() {
    cr!("707.10", "707.10c", "601.2d", "702.33d");
    ruling!(
        "Gadwick's First Duel",
        "If the spell has damage divided as it was put onto the stack, the division can't be changed for the copy, although the targets receiving that damage still can. The same is true of spells that distribute counters."
    );
    ruling!(
        "Gadwick's First Duel",
        "The copy created by the third chapter ability will have the same targets as the spell it's copying unless you choose new ones. You may change any number of the targets, including all of them or none of them. The new targets must be legal. If, for any target, you can't choose a new legal target, then it remains unchanged (even if the current target is illegal)."
    );
    ruling!(
        "Gadwick's First Duel",
        "You can't choose to pay any additional costs for a copied spell. However, effects based on any additional costs that were paid for the original spell are copied as though the same costs were paid for the copy too."
    );
    supported("Gadwick's First Duel");
    supported("Forked Bolt");
    supported("Burst Lightning");
    // Forked Bolt divided 1/1 between two Hill Giants: the copy keeps the division.
    let mut t = TestGame::new(2);
    gadwick_third_chapter(&mut t);
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Hill Giant");
    divide(&mut t, P0, &[1, 1]);
    t.answer_yes(P0, false);
    let from = t.asked().len();
    lands_for_cost(&mut t, P0, "Forked Bolt");
    let card = t.hand(P0, "Forked Bolt");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.cast_with(P0, card, &[]).expect("cast");
    t.settle();
    assert_eq!(t.stack_len(), 2, "the delayed trigger");
    t.resolve_all();
    let divides = asked_count(&t, from, |d| matches!(d, Decision::Divide { .. }));
    assert_eq!(divides, 1, "the copy's division was asked again");
    assert_eq!(t.obj_now(a).damage, 2);
    assert_eq!(t.obj_now(b).damage, 2);
    // Shock: the copy's target changed to another creature.
    let mut t = TestGame::new(2);
    gadwick_third_chapter(&mut t);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Shock", &[obj(a)]);
    t.settle();
    crate::r_s25_common::change_copy_targets(&mut t, P0, &[Some(obj(b))]);
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Graveyard(P1));
    assert_eq!(t.zone(b), Zone::Graveyard(P1));
    // Kicked Burst Lightning: the copy is kicked too, without being asked.
    let mut t = TestGame::new(2);
    gadwick_third_chapter(&mut t);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 4);
    let card = t.hand(P0, "Burst Lightning");
    t.answer_yes(P0, false);
    let from = t.asked().len();
    t.cast(P0, card).kicked(true).target(Entity::Player(P1)).go();
    t.settle();
    t.resolve_all();
    let optional = asked_count(&t, from, |d| matches!(d, Decision::OptionalCost { .. }));
    assert_eq!(optional, 1);
    assert_eq!(t.life(P1), 12);
}

#[test]
fn medomais_prophecy_with_no_name_chosen_never_triggers() {
    cr!("714.2b", "603.7", "201.4");
    ruling!(
        "Medomai's Prophecy",
        "If the third chapter ability somehow resolves with no name chosen for the second chapter ability, it creates a delayed triggered ability that can’t possibly trigger, even if you cast a spell with no name."
    );
    supported("Medomai's Prophecy");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Medomai's Prophecy");
    let saga = t.g.current(saga);
    t.g.objects[saga.0 as usize]
        .counters
        .insert("lore".into(), 2);
    put_counters(&mut t, saga, "lore", 1);
    t.resolve_all();
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // Control: with Grizzly Bears named by chapter II, casting it draws two.
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Medomai's Prophecy");
    let saga = t.g.current(saga);
    t.g.objects[saga.0 as usize]
        .counters
        .insert("lore".into(), 1);
    t.answer(
        P0,
        DecisionKind::Name,
        mtg_engine::decision::Answer::Text("Grizzly Bears".into()),
    );
    put_counters(&mut t, saga, "lore", 1);
    t.resolve_all();
    put_counters(&mut t, saga, "lore", 1);
    t.resolve_all();
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn revealed_top_card_is_the_card_drawn() {
    cr!("121.1", "701.20a");
    ruling!(
        "Nissa's Revelation",
        "If the top card of your library isn’t a creature card or it’s a creature card with power 0 or less, you won’t draw any cards. Otherwise, the first card you draw will be the card you revealed."
    );
    ruling!(
        "Track Down",
        "The card you reveal will be the card you draw, if applicable."
    );
    ruling!(
        "Track Down",
        "If you don't draw the card you reveal, it remains on top of your library."
    );
    supported("Nissa's Revelation");
    supported("Track Down");
    for (top, draws, life) in [("Grizzly Bears", 2, 2), ("Ornithopter", 0, 2), ("Shock", 0, 0)] {
        let mut t = TestGame::new(2);
        let card = t.library_top(P0, top);
        let hand = t.hand_size(P0);
        cast_new(&mut t, P0, "Nissa's Revelation", &[]);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + draws, "{top}");
        assert_eq!(t.life(P0), 20 + life, "{top}");
        if draws > 0 {
            assert_eq!(t.zone(card), Zone::Hand(P0));
        }
    }
    for (top, drawn) in [("Grizzly Bears", true), ("Island", true), ("Shock", false)] {
        let mut t = TestGame::new(2);
        let card = t.library_top(P0, top);
        cast_new(&mut t, P0, "Track Down", &[]);
        t.resolve_all();
        if drawn {
            assert_eq!(t.zone(card), Zone::Hand(P0), "{top}");
        } else {
            assert_eq!(t.g.player(P0).library.last(), Some(&t.g.current(card)));
        }
    }
}

#[test]
fn copied_mastery_resolves_as_though_the_alternative_cost_was_paid() {
    cr!("707.10", "118.9");
    ruling!(
        "Ingenious Mastery",
        "If you copy a “Mastery” spell and the alternative cost was paid, the copy will resolve as though the cost was paid."
    );
    ruling!(
        "Ingenious Mastery",
        "In a multiplayer game, you choose which opponent takes the prescribed action as the spell resolves."
    );
    supported("Ingenious Mastery");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    let card = t.hand(P0, "Ingenious Mastery");
    let m = other_method(&mut t, P0, card);
    let hand = t.hand_size(P0);
    let spell = t.cast(P0, card).method(m).go();
    copy_spell(&mut t, spell);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 6);
    assert_eq!(tokens(&t, P1).len(), 4);
    // Three players: P0 chooses P2 as it resolves.
    let mut t = TestGame::new(3);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    let card = t.hand(P0, "Ingenious Mastery");
    let m = other_method(&mut t, P0, card);
    t.cast(P0, card).method(m).go();
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.resolve_all();
    assert_eq!(tokens(&t, P2).len(), 2);
    assert_eq!(tokens(&t, P1).len(), 0);
    assert!(asked_since(&t, from)
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseEntities { .. })));
}

#[test]
fn modes_cant_repeat_and_x_is_fixed_as_the_mode_is_performed() {
    cr!("700.2", "700.2d", "608.2h");
    ruling!(
        "Inscription of Insight",
        "If you kick Inscription of Insight, you can't choose any one mode more than once."
    );
    ruling!(
        "Inscription of Insight",
        "The value of X is determined only as you perform the third mode while Inscription of Insight is resolving. Once that happens, the value of X won't change even if the number of cards in the player's hand changes."
    );
    ruling!(
        "Bumi, King of Three Trials",
        "You can't choose the same mode of Bumi, King of Three Trials's triggered ability more than once. As there are only three modes, even if X is greater than three, you won't get additional effects after choosing each mode once."
    );
    supported("Inscription of Insight");
    supported("Bumi, King of Three Trials");
    let modes_asked = |t: &TestGame, from: usize| -> Vec<(u32, bool)> {
        asked_since(t, from)
            .into_iter()
            .filter_map(|(_, d)| match d {
                Decision::ChooseModes {
                    max, allow_repeat, ..
                } => Some((max, allow_repeat)),
                _ => None,
            })
            .collect()
    };
    // Kicked, the same mode twice: it's drawn once.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.lands(P0, "Wastes", 5);
    let card = t.hand(P0, "Inscription of Insight");
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    let _ = t.cast(P0, card).kicked(true).modes(&[1, 1]).try_go();
    t.resolve_all();
    let asked = modes_asked(&t, from);
    assert!(!asked.is_empty());
    assert!(asked.iter().all(|(_, rep)| !rep), "{asked:?}");
    assert!(t.hand_size(P0) <= hand - 1 + 2);
    // Kicked, modes II and III for P0: the token is X/X for the hand after drawing.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.lands(P0, "Wastes", 5);
    let card = t.hand(P0, "Inscription of Insight");
    let spell = t
        .cast(P0, card)
        .kicked(true)
        .modes(&[1, 2])
        .target(Entity::Player(P0))
        .go();
    assert!(t.zone(spell) == Zone::Stack);
    t.resolve_all();
    let x = t.hand_size(P0) as i32;
    assert_eq!(x, 2);
    let tok = tokens(&t, P0)[0];
    assert_eq!(t.pt(tok), (x, x));
    let c = t.g.player(P0).hand[0];
    move_to(&mut t, c, Zone::Graveyard(P0));
    t.g.recompute();
    assert_eq!(t.pt(tok), (x, x));
    // Bumi with four Lessons: each mode at most once.
    let mut t = TestGame::new(2);
    for _ in 0..4 {
        t.graveyard(P0, "Environmental Sciences");
    }
    choose_modes(&mut t, P0, &[0, 0, 0, 0]);
    let from = t.asked().len();
    let bumi = enter(&mut t, P0, "Bumi, King of Three Trials");
    t.resolve_all();
    let asked = modes_asked(&t, from);
    assert!(!asked.is_empty());
    assert!(asked.iter().all(|(max, rep)| *max <= 3 && !rep), "{asked:?}");
    assert!(t.counters(bumi, "+1/+1") <= 3);
    // Control: mode one once gives three counters.
    let mut t = TestGame::new(2);
    for _ in 0..4 {
        t.graveyard(P0, "Environmental Sciences");
    }
    choose_modes(&mut t, P0, &[0]);
    let bumi = enter(&mut t, P0, "Bumi, King of Three Trials");
    t.resolve_all();
    assert_eq!(t.counters(bumi, "+1/+1"), 3);
}

#[test]
fn inspire_awe_checks_for_enchanted_creatures_as_damage_is_dealt() {
    cr!("615.1", "510.2");
    ruling!(
        "Inspire Awe",
        "Inspire Awe doesn’t modify the characteristics of creatures, so check immediately before damage would be dealt whether a creature is enchanted or also an enchantment in order to determine if its combat damage should be prevented."
    );
    supported("Inspire Awe");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    cast_new(&mut t, P0, "Inspire Awe", &[]);
    t.resolve_all();
    // Enchanted after Inspire Awe resolved: its damage isn't prevented.
    let aura = t.battlefield(P0, "Holy Strength");
    t.g.attach(aura, Entity::Object(bears));
    t.g.recompute();
    t.attack(
        &[(bears, Entity::Player(P1)), (giant, Entity::Player(P1))],
        &[],
    );
    assert_eq!(t.life(P1), 17);
}

#[test]
fn choices_made_as_the_ability_resolves_dont_target() {
    cr!("608.2d", "603.7", "702.171a");
    ruling!(
        "Fortune, Loyal Steed",
        "The delayed triggered ability doesn't target any creature. You choose which other creature to exile, if any, as that ability is resolving. At that point, it's too late for any player to respond."
    );
    ruling!(
        "Biblioplex Kraken",
        "You choose whether to return a creature and which creature to return as the triggered ability resolves. This doesn't target any creature."
    );
    supported("Fortune, Loyal Steed");
    supported("Biblioplex Kraken");
    // Biblioplex Kraken: return the hexproof Gladecover Scout (not a target).
    let mut t = TestGame::new(2);
    let kraken = t.battlefield(P0, "Biblioplex Kraken");
    let scout = t.battlefield(P0, "Gladecover Scout");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    crate::r_s01_common::attack_with(&mut t, &[(kraken, Entity::Player(P1))]);
    let trig = *t.g.stack.last().expect("attack trigger");
    assert!(targets_of(&t, trig).is_empty());
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(scout)]);
    t.resolve_all();
    assert_eq!(t.zone(scout), Zone::Hand(P0));
    block_and_finish(&mut t, P1, &[(blocker, kraken)]);
    assert_eq!(t.life(P1), 16);
    // Fortune: saddled by Grizzly Bears, attacks; at end of combat, the Bears is chosen.
    let mut t = TestGame::new(2);
    let fortune = t.battlefield(P0, "Fortune, Loyal Steed");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, fortune, "Saddle").expect("saddle");
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    crate::r_s01_common::attack_with(&mut t, &[(fortune, Entity::Player(P1))]);
    t.resolve_all();
    t.answer_choose(P0, &[obj(bears)]);
    t.advance_to(P0, Step::EndOfCombat);
    t.settle();
    let trig = *t.g.stack.last().expect("delayed trigger");
    assert!(targets_of(&t, trig).is_empty());
    t.resolve_all();
    let bears_now = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears_now.len(), 1);
    assert_ne!(bears_now[0], bears, "the Bears was exiled and returned");
    assert!(!t.named_on_battlefield("Fortune, Loyal Steed").is_empty());
}

#[test]
fn inga_counts_every_creature_that_died_this_turn_as_it_resolves() {
    cr!("608.2h", "700.4", "603.10a");
    ruling!(
        "Inga Rune-Eyes",
        "The last ability will count any creature that died, no matter who controlled it."
    );
    ruling!(
        "Inga Rune-Eyes",
        "The number of creatures that have died that turn is checked as the last ability resolves. Inga Rune-Eyes will count, as will creatures that die at the same time as Inga Rune-Eyes and creatures that die while that ability is on the stack but has not yet resolved."
    );
    supported("Inga Rune-Eyes");
    supported("Wrath of God");
    // P1's Bears died earlier; Inga and P0's Hill Giant die together: three.
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    crate::r_s02_common::destroy(&mut t, theirs);
    t.battlefield(P0, "Inga Rune-Eyes");
    t.battlefield(P0, "Hill Giant");
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Wrath of God", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
    // Inga dies after one other; a third creature dies while the trigger waits.
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    crate::r_s02_common::destroy(&mut t, theirs);
    let inga = t.battlefield(P0, "Inga Rune-Eyes");
    let giant = t.battlefield(P0, "Hill Giant");
    crate::r_s02_common::destroy(&mut t, inga);
    assert_eq!(triggers_on_stack(&t, "three or more"), 1);
    crate::r_s02_common::destroy(&mut t, giant);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
    // Control: only two died.
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    crate::r_s02_common::destroy(&mut t, theirs);
    let inga = t.battlefield(P0, "Inga Rune-Eyes");
    crate::r_s02_common::destroy(&mut t, inga);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn littjara_kinseekers_three_creatures_share_a_type() {
    cr!("205.3m", "702.73a", "603.4");
    ruling!(
        "Littjara Kinseekers",
        "Three creatures share a creature type if there’s at least one creature type all three have, no matter what other creature types they have. For example, Littjara Kinseekers (which has all creature types), an Elf Warrior, and an Elf Wizard share a creature type because they’re all Elves."
    );
    supported("Littjara Kinseekers");
    for (a, b, counter) in [
        ("Llanowar Elves", "Elvish Visionary", 1),
        ("Grizzly Bears", "Hill Giant", 0),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, a);
        t.battlefield(P0, b);
        let k = enter(&mut t, P0, "Littjara Kinseekers");
        t.resolve_all();
        assert_eq!(t.counters(k, "+1/+1"), counter, "{a} {b}");
    }
}

#[test]
fn veteran_motorists_bonus_resolves_first_and_survives_it_leaving() {
    cr!("702.122a", "603.2", "611.2c", "613.4c");
    ruling!(
        "Veteran Motorist",
        "Veteran Motorist's last ability resolves before the crew ability that it was tapped to activate. The +1/+1 bonus will be in effect as the Vehicle becomes an artifact creature."
    );
    ruling!(
        "Veteran Motorist",
        "Veteran Motorist's last ability triggers when it becomes tapped to activate a crew ability. Even if Veteran Motorist leaves the battlefield before its triggered ability resolves, that Vehicle still gets +1/+1 until end of turn."
    );
    supported("Veteran Motorist");
    supported("Smuggler's Copter");
    for leave in [false, true] {
        let mut t = TestGame::new(2);
        let motorist = t.battlefield(P0, "Veteran Motorist");
        let copter = t.battlefield(P0, "Smuggler's Copter");
        assert!(crate::r_s04_common::crew(&mut t, P0, copter, &[motorist]));
        t.settle();
        assert_eq!(t.stack_len(), 2);
        if leave {
            move_to(&mut t, motorist, Zone::Graveyard(P0));
        }
        // The trigger is on top: it resolves first, before the Copter is a creature.
        t.resolve();
        t.g.recompute();
        assert!(!t.obj_now(copter).is(CardType::Creature));
        t.resolve();
        t.g.recompute();
        assert!(t.obj_now(copter).is(CardType::Creature));
        assert_eq!(t.pt(copter), (4, 4), "left: {leave}");
    }
}

#[test]
fn living_lectern_draws_without_a_target_but_not_with_an_illegal_one() {
    cr!("608.2b", "115.4");
    ruling!(
        "Living Lectern",
        "You can activate Living Lectern's ability without a target just to draw a card. However, if you do choose a target, and that target is illegal at the time the ability tries to resolve, the ability won't resolve and none of its effects will happen. You won't draw a card."
    );
    supported("Living Lectern");
    let mut t = TestGame::new(2);
    let lectern = t.battlefield(P0, "Living Lectern");
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[]);
    let hand = t.hand_size(P0);
    activate_containing(&mut t, P0, lectern, "Draw").expect("activate");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    let mut t = TestGame::new(2);
    let lectern = t.battlefield(P0, "Living Lectern");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, lectern, "Draw").expect("activate");
    move_to(&mut t, bears, Zone::Graveyard(P0));
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn gods_willing_chooses_a_color_as_it_resolves() {
    cr!("105.1", "608.2d", "702.16a");
    ruling!(
        "Gods Willing",
        "You can't choose \"artifact\" or \"colorless\" as Gods Willing asks you to choose a color because those are not colors."
    );
    ruling!(
        "Gods Willing",
        "You choose the color as Gods Willing resolves. Once the color is chosen, it's too late for players to respond."
    );
    supported("Gods Willing");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let is_option = |d: &Decision| matches!(d, Decision::ChooseOption { .. });
    let from = t.asked().len();
    cast_new(&mut t, P0, "Gods Willing", &[obj(bears)]);
    assert_eq!(asked_count(&t, from, is_option), 0, "chosen while casting");
    let from = t.asked().len();
    t.resolve_all();
    let options: Vec<String> = asked_since(&t, from)
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options),
            _ => None,
        })
        .expect("color asked as it resolves");
    assert_eq!(options.len(), 5, "{options:?}");
    for o in &options {
        let o = o.to_lowercase();
        assert!(!o.contains("colorless") && !o.contains("artifact"), "{o}");
    }
}

#[test]
fn biolume_egg_and_serpent() {
    cr!("701.21a", "712.14a", "205.3i");
    ruling!(
        "Biolume Egg // Biolume Serpent",
        "You can't sacrifice Biolume Egg unless another effect allows you to."
    );
    ruling!(
        "Biolume Egg // Biolume Serpent",
        "You may sacrifice any two lands with the subtype Island to activate Biolume Serpent's ability, even if they aren't basic or aren't named Island."
    );
    supported("Biolume Egg // Biolume Serpent");
    let mut t = TestGame::new(2);
    let egg = t.battlefield(P0, "Biolume Egg // Biolume Serpent");
    assert!(crate::r_s27_common::activatable(&mut t, P0, egg).is_empty());
    // Sacrificed by an effect: back transformed at the next end step.
    let egg = t.g.current(egg);
    t.g.sacrifice(egg, P0);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let serpent = t.named_on_battlefield("Biolume Serpent");
    assert_eq!(serpent.len(), 1);
    let serpent = serpent[0];
    let a = t.battlefield(P0, "Tropical Island");
    let b = t.battlefield(P0, "Breeding Pool");
    t.answer_choose(P0, &[obj(a), obj(b)]);
    activate_containing(&mut t, P0, serpent, "Sacrifice two Islands").expect("activate");
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Graveyard(P0));
    assert_eq!(t.zone(b), Zone::Graveyard(P0));
}

#[test]
fn petals_of_insight_returns_from_the_stack_directly() {
    cr!("608.2n");
    ruling!(
        "Petals of Insight",
        "If you choose to put the three cards on the bottom of your library, Petals of Insight will return to your hand directly from the stack. It never goes to a graveyard."
    );
    supported("Petals of Insight");
    let mut t = TestGame::new(2);
    t.answer_yes(P0, true);
    let spell = cast_new(&mut t, P0, "Petals of Insight", &[]);
    t.resolve_all();
    assert_eq!(t.zone(spell), Zone::Hand(P0));
    let to_gy = t.g.turn_events.iter().any(|e| {
        matches!(e, Event::ZoneChange { from: Zone::Stack, to: Zone::Graveyard(_), .. })
    });
    assert!(!to_gy);
    let gy = t.g.turn_events.iter().any(|e| {
        matches!(e, Event::ZoneChange { from: Zone::Stack, to: Zone::Hand(_), .. })
    });
    assert!(gy);
}

#[test]
fn threefold_signal_adds_a_second_replicate_ability() {
    cr!("702.56a", "702.56b");
    ruling!(
        "Threefold Signal",
        "If you cast a spell that already has replicate while you have this permanent on the battlefield, the spell will have two replicate abilities: one with a replicate cost as printed on it, and one with a replicate cost of {3}. You may use either ability, or both abilities, to replicate the spell."
    );
    supported("Threefold Signal");
    // A three-color spell with replicate (no real card has both).
    let bolt = || {
        let mut def = custom_card(
            "Tricolor Echo",
            "Sorcery",
            "{U}{B}{R}",
            None,
            "Replicate {1}\n~ deals 1 damage to any target.",
        );
        let chars = &mut def.faces[0].chars;
        chars.colors = chars.mana_cost.as_ref().unwrap().colors();
        assert_eq!(chars.colors.count(), 3);
        def
    };
    // (printed paid, {3} paid) -> copies.
    for (printed, granted) in [(1, 1), (0, 1), (1, 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Threefold Signal");
        t.lands(P0, "Island", 1);
        t.lands(P0, "Swamp", 1);
        t.lands(P0, "Mountain", 1);
        t.lands(P0, "Wastes", 4);
        let spell = t.custom(P0, bolt(), Zone::Hand(P0));
        t.answer(
            P0,
            DecisionKind::OptionalCost,
            mtg_engine::decision::Answer::Number(printed),
        );
        t.answer(
            P0,
            DecisionKind::OptionalCost,
            mtg_engine::decision::Answer::Number(granted),
        );
        for _ in 0..(printed + granted) {
            t.answer_yes(P0, false);
        }
        t.cast(P0, spell).target(Entity::Player(P1)).go();
        t.settle();
        t.resolve_all();
        assert_eq!(
            t.life(P1),
            20 - 1 - (printed + granted) as i32,
            "{printed} {granted}"
        );
        let spent = 3 + printed * 1 + granted * 3;
        assert_eq!(crate::r_s01_common::tapped_lands(&t, P0) as i64, spent);
    }
}

#[test]
fn vault_13_owner_orders_their_cards_put_on_the_bottom() {
    cr!("401.4", "714.2b");
    ruling!(
        "Vault 13: Dweller's Journey",
        "In the unusual case where one player owns multiple cards exiled with Vault 13 that are being put on the bottom of their library with its third chapter ability, the owner of those cards chooses the order in which they are put on the bottom of their library."
    );
    supported("Vault 13: Dweller's Journey");
    // Four players; P1 owns a creature controlled by each player.
    let mut t = TestGame::new(4);
    let mut owned = vec![];
    for (p, name) in [
        (P0, "Grizzly Bears"),
        (P1, "Hill Giant"),
        (P2, "Craw Wurm"),
        (P3, "Llanowar Elves"),
    ] {
        let id = t.battlefield(P1, name);
        if p != P1 {
            crate::r_p147_common::take_control(&mut t, p, id);
        }
        owned.push(id);
    }
    t.g.recompute();
    let saga = t.battlefield(P0, "Vault 13: Dweller's Journey");
    let targets: Vec<Entity> = owned.iter().map(|id| obj(*id)).collect();
    for e in &targets {
        t.answer_targets(P0, &[*e]);
    }
    put_counters(&mut t, saga, "lore", 1);
    t.resolve_all();
    for id in &owned {
        assert_eq!(t.zone(*id), Zone::Exile);
    }
    put_counters(&mut t, saga, "lore", 1);
    t.resolve_all();
    let from = t.asked().len();
    put_counters(&mut t, saga, "lore", 1);
    t.resolve_all();
    // Two returned, two on the bottom of P1's library; P1 ordered them.
    let back = owned.iter().filter(|id| t.on_battlefield(**id)).count();
    assert_eq!(back, 2);
    let lib = &t.g.player(P1).library;
    let bottom: Vec<String> = lib[..2]
        .iter()
        .map(|id| t.g.obj(*id).chars.name.to_string())
        .collect();
    assert!(bottom.iter().all(|n| n != "Filler"), "{bottom:?}");
    let orderers: Vec<PlayerId> = asked_since(&t, from)
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::Order { prompt, .. } if prompt.contains("library")))
        .map(|(p, _)| p)
        .collect();
    assert_eq!(orderers, vec![P1], "{:?}", asked_since(&t, from));
}
