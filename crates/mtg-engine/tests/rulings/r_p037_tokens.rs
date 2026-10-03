//! Rulings batch P037 — tokens with bigger bodies: characteristic-defining abilities that
//! count the token itself (CR 604.3, 613.4a), token names (CR 111.4), the legend rule (CR
//! 704.5j), values of X of the spell that triggered (CR 107.3), costs paid before anyone can
//! respond (CR 601.2h, 602.2), triggers whose conditions are checked twice (CR 603.4),
//! leaves-the-battlefield triggers and last known information (CR 603.10a, 608.2h),
//! delayed triggers (CR 603.7), and effects that count over a turn.

use crate::r_p037_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::create_token;
use crate::r_s06_common::{activate_containing, give_control};
use crate::r_s25_common::{abilities_from, cast_new};
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn tokens(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    crate::r_s01_common::tokens(t, p)
}

fn ready(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.objects[id.0 as usize].summoning_sick = false;
}

// --- Tokens that count themselves ----------------------------------------------------------

#[test]
fn tokens_that_count_themselves_are_at_least_1_1() {
    cr!("604.3", "613.4a");
    ruling!(
        "Doomed Artisan",
        "A Sculpture token will count itself, so it'll be at least 1/1."
    );
    ruling!(
        "Voice of Resurgence",
        "The power and toughness of the token change as the number of creatures you control changes. The token's ability counts itself, so it'll be at least 1/1."
    );
    ruling!(
        "Bonny Pall, Clearcutter",
        "Beau's power and toughness change as the number of lands you control changes."
    );
    ruling!(
        "Doomed Artisan",
        "Sculpture is a creature type, not an artifact type."
    );
    supported("Doomed Artisan");
    supported("Voice of Resurgence");
    // Doomed Artisan's end step Sculpture.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doomed Artisan");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let s = tokens(&t, P0);
    assert_eq!(s.len(), 1);
    assert_eq!(t.pt(s[0]), (1, 1));
    let o = t.obj_now(s[0]);
    assert!(o.chars.has_subtype("Sculpture"));
    assert!(o.is(CardType::Creature) && o.is(CardType::Artifact));
    // Voice of Resurgence dies: its Elemental counts itself, then more creatures.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Voice of Resurgence");
    t.g.destroy(v, None);
    t.resolve_all();
    let e = tokens(&t, P0)[0];
    assert_eq!(t.pt(e), (1, 1));
    t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(e), (2, 2));
    // Bonny Pall's Beau: lands you control.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    t.enter(P0, "Bonny Pall, Clearcutter");
    t.resolve_all();
    let beau = tokens_named(&t, P0, "Beau")[0];
    assert_eq!(t.pt(beau), (3, 3));
    t.lands(P0, "Island", 1);
    t.g.recompute();
    assert_eq!(t.pt(beau), (4, 4));
}

#[test]
fn doomed_artisan_s_sculptures_can_attack_once_it_s_gone() {
    cr!("508.1c", "611.3b");
    ruling!(
        "Doomed Artisan",
        "Once Doomed Artisan has left the battlefield, your Sculptures can attack and block."
    );
    let mut t = TestGame::new(2);
    let da = t.battlefield(P0, "Doomed Artisan");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let s = tokens(&t, P0)[0];
    ready(&mut t, s);
    assert!(!t.g.can_attack(s));
    t.g.destroy(da, None);
    t.settle();
    assert!(t.g.can_attack(s));
}

#[test]
fn digsite_engineer_s_construct_arrives_before_the_artifact_spell_resolves() {
    cr!("603.3", "604.3");
    ruling!(
        "Digsite Engineer",
        "The token gets created before the artifact spell that triggered its creation resolves. Fortunately, the token counts itself, so it's at least 1/1."
    );
    supported("Digsite Engineer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Digsite Engineer");
    t.lands(P0, "Wastes", 2);
    let orn = t.hand(P0, "Ornithopter");
    let spell = t.cast_with(P0, orn, &[]).unwrap();
    t.answer_yes(P0, true);
    t.resolve(); // the trigger
    assert!(t.stack.contains(&spell));
    let c = tokens(&t, P0)[0];
    assert_eq!(t.pt(c), (1, 1));
}

// --- Token names and the legend rule ------------------------------------------------------

#[test]
fn token_names_and_the_legend_rule() {
    cr!("111.4", "704.5j");
    ruling!(
        "Gimbal, Gremlin Prodigy",
        "A token’s name is its subtypes plus the word “Token” unless it’s copying another object or it was given a specific name by the effect that created it. For example, the tokens created by Gimbal’s last ability are named “Gremlin Token.” These tokens have the same name as the 2/2 red Gremlin creature tokens created by Release the Gremlins."
    );
    ruling!(
        "Tolsimir Wolfblood",
        "The token is named “Voja” and has creature type “Wolf.” This is different from most creature tokens, where the name and creature type are the same."
    );
    ruling!(
        "Tolsimir Wolfblood",
        "Other creatures you control that are both green and white, including Voja, get +2/+2."
    );
    ruling!(
        "Tolsimir Wolfblood",
        "The “legend rule” means that creating a second Voja while one is already under your control will result in one of them being put into its owner’s graveyard (where it promptly ceases to exist). You choose which of the two remains on the battlefield and which is put into the graveyard."
    );
    ruling!(
        "Tuktuk the Explorer",
        "Both Tuktuk the Explorer and Tuktuk the Returned are legendary creatures. However, since they have different names, one of each may coexist without being affected by the “legend rule.”"
    );
    supported("Gimbal, Gremlin Prodigy");
    supported("Tolsimir Wolfblood");
    supported("Tuktuk the Explorer");
    // Gimbal's Gremlin.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gimbal, Gremlin Prodigy");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Gremlin Token").len(), 1);
    // Voja.
    let mut t = TestGame::new(2);
    let tol = t.battlefield(P0, "Tolsimir Wolfblood");
    activate_containing(&mut t, P0, tol, "Voja").unwrap();
    t.resolve_all();
    let voja = tokens_named(&t, P0, "Voja");
    assert_eq!(voja.len(), 1);
    assert!(subtypes(&t, voja[0]).contains(&"Wolf".to_string()));
    assert!(t.obj_now(voja[0]).chars.supertypes.contains(Supertype::Legendary));
    assert_eq!(t.pt(voja[0]), (4, 4));
    t.g.untap(tol);
    activate_containing(&mut t, P0, tol, "Voja").unwrap();
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Voja").len(), 1);
    // Tuktuk the Returned and a new Tuktuk the Explorer.
    let mut t = TestGame::new(2);
    let tk = t.battlefield(P0, "Tuktuk the Explorer");
    t.g.destroy(tk, None);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Tuktuk the Returned").len(), 1);
    let tk2 = t.battlefield(P0, "Tuktuk the Explorer");
    t.settle();
    assert!(t.on_battlefield(tk2));
    assert_eq!(tokens_named(&t, P0, "Tuktuk the Returned").len(), 1);
}

#[test]
fn a_copy_of_geralf_is_put_into_the_graveyard_before_anyone_can_act() {
    cr!("704.5j", "117.5");
    ruling!(
        "Geralf, Visionary Stitcher",
        "If another permanent enters the battlefield under your control as a copy of Geralf, Visionary Stitcher, the legend rule requires you to put one of them in a graveyard before you can take any actions. You can't sacrifice the copy to pay the cost of the original Geralf's activated ability."
    );
    supported("Geralf, Visionary Stitcher");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Geralf, Visionary Stitcher");
    t.answer_choose(P0, &[obj(g)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Clone");
    t.settle();
    assert_eq!(
        t.named_on_battlefield("Geralf, Visionary Stitcher").len(),
        1
    );
}

#[test]
fn differently_named_artifact_tokens() {
    cr!("111.4", "201.2");
    ruling!(
        "Gimbal, Gremlin Prodigy",
        "To determine the number of differently named artifact tokens you control, count each artifact token you control once, but only if its English name isn’t exactly the same as another artifact token you’ve already counted this way."
    );
    ruling!(
        "Sandsteppe War Riders",
        "To determine the number of differently named artifact tokens you control, count each artifact token you control once, but only if its English name isn’t exactly the same as another artifact token you’ve already counted this way."
    );
    supported("Sandsteppe War Riders");
    // Two Treasures and a Food: two names, plus the new Gremlin itself.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gimbal, Gremlin Prodigy");
    create_token(&mut t, P0, "Treasure");
    create_token(&mut t, P0, "Treasure");
    create_token(&mut t, P0, "Food");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let g = tokens_named(&t, P0, "Gremlin Token")[0];
    assert_eq!(t.counters(g, counters::PLUS1), 3);
    // Sandsteppe War Riders bolsters 2.
    let mut t = TestGame::new(2);
    let wr = t.battlefield(P0, "Sandsteppe War Riders");
    create_token(&mut t, P0, "Treasure");
    create_token(&mut t, P0, "Treasure");
    create_token(&mut t, P0, "Food");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.counters(wr, counters::PLUS1), 2);
}

// --- Costs paid before anyone can respond -------------------------------------------------

#[test]
fn costs_are_paid_before_anyone_can_respond() {
    cr!("602.2b", "601.2h");
    ruling!(
        "Havengul Runebinder",
        "Although players may respond to the activated ability, they can't respond to the paying of its costs. The creature card will already be exiled by the time any player could respond."
    );
    ruling!(
        "Havengul Runebinder",
        "The Zombie creature token you put onto the battlefield will also get a +1/+1 counter."
    );
    ruling!(
        "Centaur's Herald",
        "Once you activate the ability of Centaur’s Herald, it’s too late for a player to respond by trying to destroy it or otherwise stop you from sacrificing it."
    );
    supported("Havengul Runebinder");
    supported("Centaur's Herald");
    let mut t = TestGame::new(2);
    let hr = t.battlefield(P0, "Havengul Runebinder");
    let gd = t.battlefield(P0, "Gravedigger");
    let card = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    t.answer_choose(P0, &[obj(card)]);
    activate_containing(&mut t, P0, hr, "Zombie").unwrap();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.zone(card), Zone::Exile);
    t.resolve_all();
    let z = tokens(&t, P0)[0];
    assert_eq!(t.pt(z), (3, 3));
    assert_eq!(t.counters(gd, counters::PLUS1), 1);

    let mut t = TestGame::new(2);
    let ch = t.battlefield(P0, "Centaur's Herald");
    t.lands(P0, "Forest", 3);
    activate_containing(&mut t, P0, ch, "Centaur").unwrap();
    assert_eq!(t.stack_len(), 1);
    assert!(t.in_graveyard(P0, "Centaur's Herald"));
}

// --- Stitcher's Apprentice ---------------------------------------------------------------

#[test]
fn stitcher_s_apprentice_sacrifices_before_enters_triggers_resolve() {
    cr!("603.3", "608.2c", "701.21a");
    ruling!(
        "Stitcher's Apprentice",
        "Any abilities that trigger on the Homunculus token entering the battlefield will resolve after you’ve sacrificed a creature."
    );
    ruling!(
        "Stitcher's Apprentice",
        "The creature you sacrifice for the ability of Stitcher’s Apprentice could be the Homunculus you’ve just created. It could also be Stitcher’s Apprentice itself."
    );
    supported("Stitcher's Apprentice");
    supported("Soul Warden");
    let mut t = TestGame::new(2);
    // Soul Warden: "Whenever another creature enters, you gain 1 life."
    t.battlefield(P0, "Soul Warden");
    let sa = t.battlefield(P0, "Stitcher's Apprentice");
    t.lands(P0, "Island", 2);
    activate_containing(&mut t, P0, sa, "Homunculus").unwrap();
    // The sacrifice is chosen with the new token on the battlefield: sacrifice the
    // Apprentice itself.
    let watched = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| matches!(d, mtg_engine::decision::Decision::ChooseEntities { .. }),
        |g| g.permanents().filter(|o| o.is_token()).count(),
    );
    t.answer_choose(P0, &[obj(sa)]);
    let life = t.life(P0);
    t.resolve();
    assert_eq!(watched.lock().unwrap().clone(), vec![1]);
    assert!(t.in_graveyard(P0, "Stitcher's Apprentice"));
    assert_eq!(tokens(&t, P0).len(), 1);
    // The Warden's trigger goes on the stack only now.
    assert_eq!(t.life(P0), life);
    assert_eq!(
        crate::r_s01_common::triggers_on_stack(&t, "gain 1 life"),
        1
    );
    t.resolve_all();
    assert_eq!(t.life(P0), life + 1);
    // Sacrificing the new Homunculus itself.
    let mut t = TestGame::new(2);
    let sa = t.battlefield(P0, "Stitcher's Apprentice");
    t.lands(P0, "Island", 2);
    activate_containing(&mut t, P0, sa, "Homunculus").unwrap();
    let ab = abilities_from(&t, sa);
    assert_eq!(ab.len(), 1);
    // Answer the sacrifice with the newest object: the token (decided by the agent watch).
    struct PickToken;
    impl mtg_engine::decision::Agent for PickToken {
        fn decide(
            &mut self,
            g: &mtg_engine::game::Game,
            _p: PlayerId,
            d: &mtg_engine::decision::Decision,
        ) -> Answer {
            match d {
                mtg_engine::decision::Decision::ChooseEntities { .. } => {
                    let tok = g.permanents().find(|o| o.is_token()).map(|o| o.id);
                    Answer::Entities(tok.into_iter().map(Entity::Object).collect())
                }
                _ => Answer::Default,
            }
        }
    }
    t.g.agents.0.lock().unwrap()[P0.idx()] = Box::new(PickToken);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    assert!(t.on_battlefield(sa));
}

// --- Zaxara, the Exemplary ----------------------------------------------------------------

/// Zaxara is on the battlefield for P0 (with `extra` too); P0 casts the real spell `name`
/// with X = `x` (and target `targets`). Returns the game after everything resolves.
fn zaxara_casts(name: &str, x: i64, extra: Option<&str>, targets: &[Entity]) -> TestGame {
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zaxara, the Exemplary");
    if let Some(e) = extra {
        t.battlefield(P0, e);
    }
    let c = mtg_engine::card::card(name);
    let xs = format!("{}", c.front().chars.mana_cost.clone().unwrap_or_default())
        .matches("{X}")
        .count();
    t.lands(P0, "Wastes", xs * x as usize);
    t.answer(P0, DecisionKind::X, Answer::Number(x));
    cast_new(&mut t, P0, name, targets);
    t.resolve_all();
    t
}

fn hydras(t: &TestGame) -> Vec<ObjectId> {
    tokens_named(t, P0, "Hydra Token")
}

#[test]
fn zaxara_s_hydra_gets_the_spell_s_x_counters() {
    cr!("107.3", "107.3f", "111.4");
    ruling!(
        "Zaxara, the Exemplary",
        "The value of X in Zaxara's last ability is the value of X of the spell you cast."
    );
    ruling!(
        "Zaxara, the Exemplary",
        "If the spell you cast has {X}{X} in its mana cost, the token gets only X counters, not twice X."
    );
    ruling!(
        "Zaxara, the Exemplary",
        "If an effect, such as that of Parallel Lives, causes Zaxara's ability to create multiple Hydra tokens, they each receive X +1/+1 counters."
    );
    ruling!(
        "Zaxara, the Exemplary",
        "If X is 0, the 0/0 Hydra token will die immediately after the ability's done resolving unless something else is raising its toughness."
    );
    supported("Zaxara, the Exemplary");
    // Endless One ({X}): X = 3.
    let t = zaxara_casts("Endless One", 3, None, &[]);
    let h = hydras(&t);
    assert_eq!(h.len(), 1);
    assert_eq!(t.counters(h[0], counters::PLUS1), 3);
    // Crackle with Power ({X}{X}{X}{R}), X = 1: one counter.
    let t = zaxara_casts("Crackle with Power", 1, None, &[Entity::Player(P1)]);
    let h = hydras(&t);
    assert_eq!(t.counters(h[0], counters::PLUS1), 1);
    assert_eq!(t.life(P1), 15);
    // Parallel Lives: two Hydras with two counters each.
    let t = zaxara_casts("Endless One", 2, Some("Parallel Lives"), &[]);
    let h = hydras(&t);
    assert_eq!(h.len(), 2);
    assert!(h.iter().all(|x| t.counters(*x, counters::PLUS1) == 2));
    // X = 0: the Hydra dies.
    let t = zaxara_casts("Endless One", 0, None, &[]);
    assert!(hydras(&t).is_empty());
}

// --- Druid of Horns ---------------------------------------------------------------------

#[test]
fn druid_of_horns_s_trigger_resolves_even_if_the_aura_is_countered() {
    cr!("603.3", "405.5");
    ruling!(
        "Druid of Horns",
        "Druid of Horns’s ability resolves before the spell that caused it to trigger. The ability resolves even if that spell is countered."
    );
    supported("Druid of Horns");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Druid of Horns");
    let rancor = cast_new(&mut t, P0, "Rancor", &[obj(d)]);
    t.settle();
    assert_eq!(abilities_from(&t, d).len(), 1);
    cast_new(&mut t, P1, "Counterspell", &[obj(rancor)]);
    t.resolve(); // Counterspell
    assert!(!t.stack.contains(&rancor));
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}

// --- Rukh Egg --------------------------------------------------------------------------

/// Rukh Egg dies (destroyed) under `controller`'s control; then the game goes to the end
/// step. Returns the game.
fn egg_dies(controller: PlayerId, text_change: bool, exile_card: bool) -> TestGame {
    let mut t = TestGame::new(2);
    let egg = t.battlefield(P0, "Rukh Egg");
    if controller != P0 {
        give_control(&mut t, egg, controller);
    }
    if text_change {
        modify_until_eot(
            &mut t,
            egg,
            vec![Modification::ChangeText {
                from: "red".into(),
                to: "blue".into(),
            }],
        );
    }
    t.g.destroy(t.g.current(egg), None);
    t.resolve_all();
    if exile_card {
        let card = t.g.current(egg);
        assert_eq!(t.zone(card), Zone::Graveyard(P0));
        t.g.exile_object(card, None);
    }
    t.advance_to(P0, Step::End);
    t.resolve_all();
    t
}

fn birds(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    tokens_named(t, p, "Bird Token")
}

#[test]
fn rukh_egg_s_bird() {
    cr!("603.7c", "603.10a", "612.2");
    ruling!(
        "Rukh Egg",
        "If the Egg is destroyed while under the control of another player, the controller of the Egg gets the Bird."
    );
    ruling!(
        "Rukh Egg",
        "If the Rukh Egg card is removed from the graveyard in the same turn it is put there, a Bird will still be put onto the battlefield."
    );
    ruling!(
        "Rukh Egg",
        "Text-changing effects can be used to change the color of the Bird that will be put onto the battlefield."
    );
    supported("Rukh Egg");
    let t = egg_dies(P1, false, false);
    assert_eq!(birds(&t, P1).len(), 1);
    assert!(birds(&t, P0).is_empty());
    let t = egg_dies(P0, false, true);
    assert_eq!(birds(&t, P0).len(), 1);
    let t = egg_dies(P0, true, false);
    let b = birds(&t, P0);
    assert_eq!(b.len(), 1);
    assert!(t.obj_now(b[0]).chars.colors.contains(Color::Blue));
    assert!(!t.obj_now(b[0]).chars.colors.contains(Color::Red));
}

#[test]
fn an_exiled_rukh_egg_makes_no_bird() {
    cr!("603.6c", "614.1a");
    ruling!(
        "Rukh Egg",
        "If the Egg is exiled instead of being put into the graveyard, no Bird is put onto the battlefield."
    );
    let mut t = TestGame::new(2);
    let egg = t.battlefield(P0, "Rukh Egg");
    t.g.exile_object(egg, None);
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(birds(&t, P0).is_empty());
}

// --- Psemilla, Meletian Poet ------------------------------------------------------------

#[test]
fn psemilla_s_first_enchantment_spell_counts_spells_before_it_arrived() {
    cr!("603.2", "700.14");
    ruling!(
        "Psemilla, Meletian Poet",
        "If you cast an enchantment spell during a turn before Psemilla is on the battlefield, its first ability won't trigger that turn even if you cast another enchantment spell later in the turn."
    );
    supported("Psemilla, Meletian Poet");
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Ghostly Prison", &[]);
    t.resolve_all();
    t.battlefield(P0, "Psemilla, Meletian Poet");
    cast_new(&mut t, P0, "Ghostly Prison", &[]);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
}

#[test]
fn psemilla_s_combat_trigger_checks_twice() {
    cr!("603.4");
    ruling!(
        "Psemilla, Meletian Poet",
        "Psemilla's last ability checks at the moment it would trigger to see if you control five or more enchantments. If you don't, the ability won't trigger at all. If it does trigger, the ability will check again as it tries to resolve. If you don't control five or more enchantments at that time, the ability won't resolve and none of its effects will happen. If you do, it doesn't matter what happens to those enchantments later in the turn."
    );
    // (enchantments, remove one before resolution, remove after, pumped)
    for (n, before, after, pumped) in [
        (4, false, false, false),
        (5, true, false, false),
        (5, false, true, true),
    ] {
        let mut t = TestGame::new(2);
        let ps = t.battlefield(P0, "Psemilla, Meletian Poet");
        let ench = t.lands(P0, "Ghostly Prison", n);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.settle();
        let stacked = !abilities_from(&t, ps).is_empty();
        assert_eq!(stacked, n >= 5, "{n} enchantments");
        if before {
            t.g.destroy(ench[0], None);
        }
        t.resolve_all();
        if after {
            t.g.destroy(ench[0], None);
            t.settle();
        }
        let p = if pumped { 5 } else { 1 };
        assert_eq!(t.pt(ps), (p, p), "{n} {before} {after}");
        assert_eq!(t.obj_now(ps).has_keyword(KeywordKind::Lifelink), pumped);
    }
}

// --- One-shot payments ----------------------------------------------------------------------

#[test]
fn aphemia_exiles_only_one_enchantment_card() {
    cr!("608.2c", "118.12");
    ruling!(
        "Aphemia, the Cacophony",
        "While resolving Aphemia’s triggered ability, you can’t exile more than one enchantment card to get more than one Zombie token."
    );
    supported("Aphemia, the Cacophony");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aphemia, the Cacophony");
    let a = t.graveyard(P0, "Ghostly Prison");
    let b = t.graveyard(P0, "Glorious Anthem");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    let exiled = [a, b].iter().filter(|x| t.zone(**x) == Zone::Exile).count();
    assert_eq!(exiled, 1);
}

// --- Other one-offs ------------------------------------------------------------------------

#[test]
fn sandstorm_salvager_s_trample_without_counters() {
    cr!("608.2c", "122.6");
    ruling!(
        "Sandstorm Salvager",
        "In the rare case where +1/+1 counters can’t be put on one or more creature tokens you control, those creature tokens still gain trample until end of turn."
    );
    supported("Sandstorm Salvager");
    let mut t = TestGame::new(2);
    let ss = t.enter(P0, "Sandstorm Salvager");
    t.resolve_all();
    ready(&mut t, ss);
    let golem = tokens(&t, P0)[0];
    t.battlefield(P0, "Solemnity");
    t.lands(P0, "Wastes", 2);
    activate_containing(&mut t, P0, ss, "+1/+1").unwrap();
    t.resolve_all();
    assert_eq!(t.counters(golem, counters::PLUS1), 0);
    assert!(t.obj_now(golem).has_keyword(KeywordKind::Trample));
}

#[test]
fn malcator_counts_artifacts_that_entered_before_it_but_not_ones_that_became_artifacts() {
    cr!("603.4", "700.14");
    ruling!(
        "Malcator, Purity Overseer",
        "Malcator will count any artifacts that entered the battlefield under your control during the turn, even if you didn't control Malcator at the time. Malcator won't count non-artifact permanents you controlled that became artifacts."
    );
    supported("Malcator, Purity Overseer");
    for (orns, animate, made) in [(3, false, true), (2, true, false)] {
        let mut t = TestGame::new(2);
        for _ in 0..orns {
            t.enter(P0, "Ornithopter");
        }
        if animate {
            let bears = t.enter(P0, "Grizzly Bears");
            modify_until_eot(
                &mut t,
                bears,
                vec![Modification::AddTypes(vec![CardType::Artifact])],
            );
        }
        t.battlefield(P0, "Malcator, Purity Overseer");
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(tokens(&t, P0).len(), made as usize, "{orns} {animate}");
    }
}

#[test]
fn baru_s_cost_reduction_stops_at_green() {
    cr!("601.2f", "118.7d");
    ruling!(
        "Baru, Wurmspeaker",
        "The cost reduction of Baru’s last ability can’t reduce the cost to activate it to less than {G}, even if you control a Wurm with power greater than 7."
    );
    supported("Baru, Wurmspeaker");
    let mut t = TestGame::new(2);
    let baru = t.battlefield(P0, "Baru, Wurmspeaker");
    let w = create_token(&mut t, P0, "Wurm");
    t.g.add_counters(Entity::Object(w), counters::PLUS1, 6, None);
    t.g.recompute();
    assert!(t.pt(w).0 > 7);
    // No mana: can't activate. One Forest: can.
    assert!(activate_containing(&mut t, P0, baru, "Wurm creature token").is_err());
    let f = t.lands(P0, "Forest", 1);
    activate_containing(&mut t, P0, baru, "Wurm creature token").unwrap();
    assert!(t.obj(f[0]).tapped);
}

#[test]
fn azog_uses_the_destroyed_creature_s_last_known_power() {
    cr!("608.2h", "701.47a");
    ruling!(
        "Azog, Moria's Ruin",
        "Use the power of the creature as it last existed on the battlefield to determine the value of X."
    );
    supported("Azog, Moria's Ruin");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    modify_until_eot(
        &mut t,
        giant,
        vec![Modification::ModifyPT(Value::c(2), Value::c(2))],
    );
    t.answer_targets(P0, &[obj(giant)]);
    t.enter(P0, "Azog, Moria's Ruin");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    let army = tokens(&t, P1);
    assert_eq!(army.len(), 1);
    assert_eq!(t.counters(army[0], counters::PLUS1), 5);
}

#[test]
fn budoka_gardener_flips_with_ten_lands_on_resolution() {
    cr!("710.2", "704.5j");
    ruling!(
        "Budoka Gardener // Dokai, Weaver of Life",
        "Budoka Gardener flips if you control ten or more lands when its ability resolves, even if you don’t use its ability to put a land onto the battlefield."
    );
    ruling!(
        "Budoka Gardener // Dokai, Weaver of Life",
        "Budoka Gardener flips even if you put a second copy of a legendary land onto the battlefield. You’ll have ten lands on the battlefield while the ability is resolving, so the Gardener flips. Then you will choose one of the legendary lands to keep and put the other into its owner’s graveyard as a state-based action."
    );
    supported("Budoka Gardener // Dokai, Weaver of Life");
    // Ten lands, no land put.
    let mut t = TestGame::new(2);
    let bg = t.battlefield(P0, "Budoka Gardener // Dokai, Weaver of Life");
    t.lands(P0, "Forest", 10);
    t.answer_yes(P0, false);
    activate_containing(&mut t, P0, bg, "flip").unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(bg).chars.name, "Dokai, Weaver of Life");
    // Nine lands with Gaea's Cradle; a second Cradle is put onto the battlefield.
    let mut t = TestGame::new(2);
    let bg = t.battlefield(P0, "Budoka Gardener // Dokai, Weaver of Life");
    t.lands(P0, "Forest", 8);
    t.battlefield(P0, "Gaea's Cradle");
    let c2 = t.hand(P0, "Gaea's Cradle");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(c2)]);
    activate_containing(&mut t, P0, bg, "flip").unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(bg).chars.name, "Dokai, Weaver of Life");
    assert_eq!(t.named_on_battlefield("Gaea's Cradle").len(), 1);
    assert!(t.in_graveyard(P0, "Gaea's Cradle"));
}

#[test]
fn shagrat_attaches_without_changing_control_of_the_equipment() {
    cr!("301.5", "301.5c", "702.6a");
    ruling!(
        "Shagrat, Loot Bearer",
        "Control of the Equipment doesn't change. If you target an Equipment an opponent controls, you won't be able to activate its equip ability, for example, and they can activate its equip ability as a sorcery to reattach it to a creature they control."
    );
    supported("Shagrat, Loot Bearer");
    let mut t = TestGame::new(2);
    let sh = t.battlefield(P0, "Shagrat, Loot Bearer");
    let bs = t.battlefield(P1, "Bonesplitter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bs)]);
    crate::r_s01_common::attack_with(&mut t, &[(sh, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.obj_now(bs).attached_to, Some(obj(sh)));
    assert_eq!(t.obj_now(bs).controller, P1);
    crate::r_s01_common::block_and_finish(&mut t, P1, &[]);
    // P0 can't activate the equip ability.
    t.lands(P0, "Wastes", 1);
    assert!(activate_containing(&mut t, P0, bs, "Equip").is_err());
    // P1 reattaches it on its turn.
    t.advance_to(P1, Step::PrecombatMain);
    t.lands(P1, "Wastes", 1);
    t.answer_targets(P1, &[obj(bears)]);
    activate_containing(&mut t, P1, bs, "Equip").unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(bs).attached_to, Some(obj(bears)));
}

#[test]
fn setzer_s_coin_flip_has_no_winner() {
    cr!("705.2", "705.3");
    ruling!(
        "Setzer, Wandering Gambler",
        "Some effects that instruct a player to flip a coin care only about whether the coin comes up heads or tails. These effects don't normally cause any player to win or lose that coin flip."
    );
    supported("Setzer, Wandering Gambler");
    supported("Ral Zarek");
    // Ral Zarek's −7: "Flip five coins. Take an extra turn after this one for each coin
    // that comes up heads." — only heads or tails: nobody wins, so Setzer's "Whenever you
    // win a coin flip" never triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Setzer, Wandering Gambler");
    let ral = t.battlefield(P0, "Ral Zarek");
    t.g.add_counters(Entity::Object(ral), counters::LOYALTY, 3, None);
    activate_containing(&mut t, P0, ral, "Flip five coins").unwrap();
    t.resolve();
    assert_eq!(t.stack_len(), 0, "no win trigger");
    assert!(crate::r_s01_common::with_subtype(&t, P0, "Treasure").is_empty());
}
