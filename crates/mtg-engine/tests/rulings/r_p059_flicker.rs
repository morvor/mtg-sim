//! Rulings batch P059 — "exile it, then return it at the beginning of the next end step"
//! (CR 603.7, 610.3): the returned card is a new object (CR 400.7; Auras go to the
//! graveyard, Equipment stays unattached, counters cease to exist), a token doesn't come
//! back (CR 111.7, 111.8), the delayed trigger doesn't depend on its source staying on the
//! battlefield (CR 603.7c), exiling during an end step waits for the next turn's end step
//! (CR 513.1, 603.7), the returned permanent's own end step abilities miss that end step
//! (CR 603.2, 603.6a), and a returning Aura is attached without targeting (CR 303.4f).

use crate::r_p040_common::loyalty;
use crate::r_p058_common::{assert_new_object, decorate, mana};
use crate::r_p059_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::{create_token, destroy};
use crate::r_s05_common::move_to;
use crate::r_s06_common::{attach_new, attached_to, give_control};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// P0 uses `name` to exile `target` (an ETB trigger, an attack trigger, a loyalty ability
/// or a spell, as the card does it), and everything resolves. Returns the source of the
/// effect (the creature, enchantment or planeswalker; the spell's card for a spell).
fn exile_with(t: &mut TestGame, name: &str, target: ObjectId) -> ObjectId {
    supported(name);
    mana(t, P0, 2);
    match name {
        "Phelia, Exuberant Shepherd" => {
            let phelia = t.battlefield(P0, name);
            t.answer_targets(P0, &[o(target)]);
            attack_with(t, &[(phelia, pl(P1))]);
            t.resolve_all();
            phelia
        }
        "The Eternal Wanderer" => {
            let pw = t.battlefield(P0, name);
            loyalty(t, P0, pw, "+1", &[o(target)]);
            pw
        }
        "Teferi's Time Twist" | "Daydream" => {
            let spell = t.hand(P0, name);
            t.cast(P0, spell).target(target).go();
            t.resolve_all();
            spell
        }
        _ => {
            if name == "Blizzard Strix" {
                // "if you control another snow permanent"
                t.battlefield(P0, "Snow-Covered Forest");
            }
            if name == "Sentinel of the Pearl Trident" {
                t.answer_yes(P0, true);
            }
            t.answer_targets(P0, &[o(target)]);
            let source = t.enter(P0, name);
            t.resolve_all();
            source
        }
    }
}

/// The cards whose permanent returns at the beginning of the next end step (P0's own,
/// for The Eternal Wanderer: P0 owns the exiled card).
const END_STEP_FLICKERS: [&str; 6] = [
    "Flickerwisp",
    "Phelia, Exuberant Shepherd",
    "Sentinel of the Pearl Trident",
    "The Eternal Wanderer",
    "Blizzard Strix",
    "Teferi's Time Twist",
];

#[test]
fn the_returned_permanent_is_a_new_object() {
    cr!("400.7", "603.7", "704.5m", "704.5n", "122.2");
    ruling!(
        "Flickerwisp",
        "Auras attached to the exiled permanent will be put into their owners' graveyards. Equipment attached to the exiled permanent will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist. Once the exiled permanent returns, it's considered a new object with no relation to the object that it was."
    );
    ruling!(
        "Phelia, Exuberant Shepherd",
        "Auras attached to the exiled permanent will be put into their owners' graveyards. Equipment attached to the exiled permanent will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist. Once the exiled permanent returns, it's considered a new object with no relation to the object that it was."
    );
    ruling!(
        "Sentinel of the Pearl Trident",
        "Auras attached to the exiled permanent will be put into their owners' graveyards. Equipment attached to the exiled permanent will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist. Once the exiled permanent returns, it's considered a new object with no relation to the object that it was."
    );
    ruling!(
        "The Eternal Wanderer",
        "Auras attached to the exiled permanent will be put into their owners' graveyards. Equipment attached to the exiled permanent will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist. Once the exiled permanent returns, it's considered a new object with no relation to the object that it was."
    );
    ruling!(
        "Blizzard Strix",
        "Auras attached to the exiled permanent will be put into their owners’ graveyards. Equipment attached to the exiled permanent will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist. Once the exiled permanent returns, it’s considered a new object with no relation to the object that it was."
    );
    ruling!(
        "Teferi's Time Twist",
        "Auras attached to the exiled permanent will be put into their owners’ graveyards. Equipment attached to the exiled permanent will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist. Once the exiled permanent returns, it’s considered a new object with no relation to the object that it was."
    );
    ruling!(
        "Daydream",
        "Auras attached to the exiled permanent will be put into their owners' graveyards. Equipment attached to the exiled permanent will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist. Once the exiled permanent returns, it's considered a new object with no relation to the object that it was."
    );
    // Isamaru is legendary (historic, for Sentinel of the Pearl Trident).
    for name in END_STEP_FLICKERS {
        let mut t = TestGame::new(2);
        let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
        let d = decorate(&mut t, P0, isamaru);
        exile_with(&mut t, name, isamaru);
        assert!(t.in_exile("Isamaru, Hound of Konda"), "{name}: exiled");
        // The Aura and the Equipment were dealt with when it left.
        assert!(t.in_graveyard(P0, "Holy Strength"), "{name}");
        assert_eq!(attached_to(&t, d.equipment), None, "{name}");
        end_step_of(&mut t, P0);
        let now = t.g.current(isamaru);
        assert!(t.on_battlefield(now), "{name}: returned");
        if name == "Teferi's Time Twist" {
            // Only the counter it enters with ("an additional +1/+1 counter").
            assert_eq!(t.counters(now, counters::PLUS1), 1, "{name}");
            assert_ne!(now, d.id);
            assert!(t.on_battlefield(d.equipment));
            assert_eq!(attached_to(&t, d.equipment), None);
        } else {
            assert_new_object(&t, &d);
        }
    }
    // Daydream returns it at once, with only Daydream's counter.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let d = decorate(&mut t, P0, bears);
    exile_with(&mut t, "Daydream", bears);
    let now = t.g.current(bears);
    assert_ne!(now, bears);
    assert!(t.on_battlefield(now));
    assert_eq!(t.counters(now, counters::PLUS1), 1, "not 2");
    assert!(t.in_graveyard(P0, "Holy Strength"));
    assert!(t.on_battlefield(d.equipment));
    assert_eq!(attached_to(&t, d.equipment), None);
}

#[test]
fn a_returned_creature_without_counters_gets_one_counter() {
    cr!("614.1c", "122.6");
    ruling!(
        "Teferi's Time Twist",
        "A creature returned to the battlefield this way enters the battlefield with one +1/+1 counter if it would otherwise enter with no +1/+1 counters."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    exile_with(&mut t, "Teferi's Time Twist", bears);
    end_step_of(&mut t, P0);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn whether_it_enters_as_a_creature_counts_effects_that_will_apply() {
    cr!("614.12", "614.1c", "613.1d");
    ruling!(
        "Teferi's Time Twist",
        "To determine whether the entering permanent is entering as a creature, consider any effects that will modify that permanent’s characteristics once it’s on the battlefield, including that permanent’s own abilities that affect only itself and effects from other objects."
    );
    // A noncreature artifact returns without a counter.
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Millstone");
    exile_with(&mut t, "Teferi's Time Twist", stone);
    end_step_of(&mut t, P0);
    assert!(t.on_battlefield(stone));
    assert_eq!(t.counters(stone, counters::PLUS1), 0);
    // An effect from another object: March of the Machines ("Each noncreature artifact is
    // an artifact creature with power and toughness each equal to its mana value.").
    let mut t = TestGame::new(2);
    supported("March of the Machines");
    t.battlefield(P0, "March of the Machines");
    let stone = t.battlefield(P0, "Millstone");
    exile_with(&mut t, "Teferi's Time Twist", stone);
    end_step_of(&mut t, P0);
    assert!(t.on_battlefield(stone));
    assert_eq!(t.counters(stone, counters::PLUS1), 1, "{}", t.dump_log());
    assert_eq!(t.pt(stone), (3, 3));
    // Its own ability: Gideon Blackblade ("As long as it's your turn, Gideon Blackblade is
    // a 4/4 Human Soldier creature with indestructible that's still a planeswalker.")
    // returns during P0's end step, so it's a creature.
    let mut t = TestGame::new(2);
    let gideon = t.battlefield(P0, "Gideon Blackblade");
    exile_with(&mut t, "Teferi's Time Twist", gideon);
    end_step_of(&mut t, P0);
    assert!(t.on_battlefield(gideon));
    assert_eq!(t.counters(gideon, counters::PLUS1), 1, "{}", t.dump_log());
    assert_eq!(t.pt(gideon), (5, 5));
}

#[test]
fn teferis_time_twist_returns_it_untapped_unless_it_enters_tapped() {
    cr!("110.5b", "614.1c");
    ruling!(
        "Teferi's Time Twist",
        "The permanent returns untapped unless another effect causes it to enter the battlefield tapped."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let gate = t.battlefield(P0, "Azorius Guildgate");
    t.g.tap(bears);
    t.g.tap(gate);
    exile_with(&mut t, "Teferi's Time Twist", bears);
    let spell = t.hand(P0, "Teferi's Time Twist");
    t.cast(P0, spell).target(gate).go();
    t.resolve_all();
    end_step_of(&mut t, P0);
    assert!(t.on_battlefield(bears) && t.on_battlefield(gate));
    assert!(!t.obj_now(bears).tapped, "untapped");
    // "This land enters tapped."
    assert!(t.obj_now(gate).tapped, "enters tapped by its own ability");
}

#[test]
fn tokens_exiled_this_way_cease_to_exist() {
    cr!("111.7", "111.8");
    ruling!(
        "Glimmerpoint Stag",
        "If a token is exiled this way, it ceases to exist and won’t return to the battlefield."
    );
    ruling!(
        "Oath of Teferi",
        "If a token is exiled this way, it ceases to exist and won’t return to the battlefield."
    );
    for name in ["Glimmerpoint Stag", "Oath of Teferi"] {
        let mut t = TestGame::new(2);
        let soldier = create_token(&mut t, P0, "Soldier");
        exile_with(&mut t, name, soldier);
        assert!(token_gone(&t, soldier), "{name}");
        end_step_of(&mut t, P0);
        assert!(token_gone(&t, soldier), "{name}");
        assert!(t.g.permanents().all(|o| !o.is_token()), "{name}");
    }
}

#[test]
fn the_card_returns_even_if_its_exiler_is_gone() {
    cr!("603.7", "603.7e");
    ruling!(
        "Flickerwisp",
        "The exiled card will return to the battlefield at the beginning of the end step even if Flickerwisp is no longer on the battlefield."
    );
    ruling!(
        "Sentinel of the Pearl Trident",
        "The exiled card will return to the battlefield at the beginning of the end step even if Sentinel of the Pearl Trident is no longer on the battlefield."
    );
    ruling!(
        "Glimmerpoint Stag",
        "The exiled card will return to the battlefield at the beginning of the next end step even if Glimmerpoint Stag is no longer on the battlefield at that time."
    );
    ruling!(
        "Oath of Teferi",
        "The exiled card will return to the battlefield at the beginning of the next end step even if Oath of Teferi is no longer on the battlefield at that time."
    );
    for name in [
        "Flickerwisp",
        "Sentinel of the Pearl Trident",
        "Glimmerpoint Stag",
        "Oath of Teferi",
    ] {
        let mut t = TestGame::new(2);
        let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
        let source = exile_with(&mut t, name, isamaru);
        assert!(t.in_exile("Isamaru, Hound of Konda"), "{name}");
        move_to(&mut t, source, Zone::Graveyard(P0));
        assert!(!t.on_battlefield(source), "{name}");
        end_step_of(&mut t, P0);
        assert!(t.on_battlefield(isamaru), "{name}: {}", t.dump_log());
        assert_eq!(t.obj_now(isamaru).controller, P0);
    }
}

#[test]
fn end_step_abilities_of_the_returned_permanent_dont_trigger_that_turn() {
    cr!("603.7", "603.2b");
    ruling!(
        "Flickerwisp",
        "If the permanent that returns to the battlefield has any abilities that trigger at the beginning of the end step, those abilities won't trigger that turn."
    );
    ruling!(
        "Phelia, Exuberant Shepherd",
        "If the permanent that returns to the battlefield has any abilities that trigger at the beginning of the end step, those abilities won't trigger that turn."
    );
    ruling!(
        "Sentinel of the Pearl Trident",
        "If the permanent that returns to the battlefield has any abilities that trigger at the beginning of the end step, those abilities won't trigger that turn."
    );
    ruling!(
        "The Eternal Wanderer",
        "If the permanent that returns to the battlefield has any abilities that trigger at the beginning of the end step, those abilities won't trigger that turn."
    );
    for name in [
        "Flickerwisp",
        "Phelia, Exuberant Shepherd",
        "Sentinel of the Pearl Trident",
        "The Eternal Wanderer",
    ] {
        let mut t = TestGame::new(2);
        let keeper = end_step_keeper(&mut t, P0);
        exile_with(&mut t, name, keeper);
        assert!(t.in_exile("Dusk Keeper"), "{name}");
        let life = t.life(P0);
        end_step_of(&mut t, P0);
        assert!(t.on_battlefield(keeper), "{name}: returned");
        assert_eq!(t.life(P0), life, "{name}: didn't trigger this end step");
        // It does trigger at the next end step.
        end_step_of(&mut t, P1);
        assert_eq!(t.life(P0), life + 1, "{name}: next end step");
    }
}

#[test]
fn entering_during_an_end_step_waits_for_the_next_turns_end_step() {
    cr!("603.7a", "513.1");
    ruling!(
        "Blizzard Strix",
        "If Blizzard Strix enters the battlefield during a player’s end step, the exiled permanent will remain exiled until the next turn’s end step."
    );
    ruling!(
        "Glimmerpoint Stag",
        "If Glimmerpoint Stag somehow enters the battlefield during a turn’s end step, the exiled card won’t be returned to the battlefield until the beginning of the following turn’s end step."
    );
    for name in ["Blizzard Strix", "Glimmerpoint Stag"] {
        let mut t = TestGame::new(2);
        let ogre = t.battlefield(P1, "Gray Ogre");
        t.set_step(P0, Step::End);
        exile_with(&mut t, name, ogre);
        assert!(t.in_exile("Gray Ogre"), "{name}");
        t.advance_to(P1, Step::Upkeep);
        assert!(
            t.in_exile("Gray Ogre"),
            "{name}: not at this turn's cleanup"
        );
        end_step_of(&mut t, P1);
        assert!(t.on_battlefield(ogre), "{name}: the next turn's end step");
        assert_eq!(
            t.obj_now(ogre).controller,
            P1,
            "{name}: its owner's control"
        );
    }
}

#[test]
fn saltskitter_exiled_during_the_end_step_returns_the_next_turn() {
    cr!("603.7a", "513.1");
    ruling!(
        "Saltskitter",
        "If a creature enters after \"at the beginning of the end step\" abilities have triggered during that turn, Saltskitter won't return to the battlefield until the end of the following turn."
    );
    // "Whenever another creature enters, exile this creature. Return this creature to the
    // battlefield under its owner's control at the beginning of the next end step."
    supported("Saltskitter");
    let mut t = TestGame::new(2);
    let skitter = t.battlefield(P0, "Saltskitter");
    t.set_step(P0, Step::End);
    t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert!(t.in_exile("Saltskitter"));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.in_exile("Saltskitter"), "not this turn");
    end_step_of(&mut t, P1);
    assert!(t.on_battlefield(skitter), "the following turn's end step");
    assert_eq!(t.obj_now(skitter).controller, P0);
    // Entering earlier in the turn: it's back the same turn.
    let mut t = TestGame::new(2);
    let skitter = t.battlefield(P0, "Saltskitter");
    t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert!(t.in_exile("Saltskitter"));
    end_step_of(&mut t, P0);
    assert!(t.on_battlefield(skitter));
}

#[test]
fn a_returning_aura_is_attached_by_its_owner_without_targeting() {
    cr!("303.4f", "303.4g", "702.18a", "702.11a");
    ruling!(
        "Glimmerpoint Stag",
        "If the exiled card is an Aura, that card’s owner chooses what it will enchant as it comes back onto the battlefield. An Aura put onto the battlefield this way doesn’t target anything (so it could be attached to a permanent with shroud, for example), but the Aura’s enchant ability restricts what it can be attached to. If the Aura can’t legally be attached to anything, it remains exiled."
    );
    ruling!(
        "Oath of Teferi",
        "If the exiled card is an Aura, that card’s owner chooses what it will enchant as it returns to the battlefield. An Aura put onto the battlefield this way doesn’t target anything (so it could be attached to an opponent’s permanent with hexproof, for example), but the Aura’s enchant ability restricts what it can be attached to. If the Aura can’t legally be attached to anything, it remains exiled."
    );
    // Glimmerpoint Stag exiles P1's Holy Strength (on P1's Grizzly Bears); P1, its owner,
    // attaches it to P0's Argothian Enchantress (shroud) as it returns.
    supported("Argothian Enchantress");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let enchantress = t.battlefield(P0, "Argothian Enchantress");
    let aura = attach_new(&mut t, P1, "Holy Strength", bears);
    exile_with(&mut t, "Glimmerpoint Stag", aura);
    assert!(t.in_exile("Holy Strength"));
    t.advance_to(P0, Step::End);
    t.answer_choose(P1, &[o(enchantress)]);
    t.resolve_all();
    let now = t.g.current(aura);
    assert!(t.on_battlefield(now), "{}", t.dump_log());
    assert_eq!(t.obj_now(now).controller, P1);
    assert_eq!(attached_to(&t, now), Some(o(enchantress)));
    assert_eq!(t.pt(enchantress), (1, 3), "0/1 +1/+2");
    let p1_asked = t.asked().iter().any(|(p, d)| {
        *p == P1 && matches!(d, mtg_engine::decision::Decision::ChooseEntities { .. })
    });
    assert!(p1_asked, "the owner chose");

    // Oath of Teferi exiles P0's Holy Strength (on P0's Grizzly Bears); P0 attaches it to
    // P1's Gladecover Scout (hexproof).
    supported("Gladecover Scout");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let scout = t.battlefield(P1, "Gladecover Scout");
    let aura = attach_new(&mut t, P0, "Holy Strength", bears);
    exile_with(&mut t, "Oath of Teferi", aura);
    assert!(t.in_exile("Holy Strength"));
    t.advance_to(P0, Step::End);
    t.answer_choose(P0, &[o(scout)]);
    t.resolve_all();
    let now = t.g.current(aura);
    assert!(t.on_battlefield(now), "{}", t.dump_log());
    assert_eq!(attached_to(&t, now), Some(o(scout)));
    assert_eq!(t.obj_now(now).controller, P0);

    // Nothing it could enchant: it stays in exile.
    for name in ["Glimmerpoint Stag", "Oath of Teferi"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let aura = attach_new(&mut t, P0, "Holy Strength", bears);
        let source = exile_with(&mut t, name, aura);
        destroy(&mut t, bears);
        if name == "Glimmerpoint Stag" {
            destroy(&mut t, source);
        }
        assert!(
            t.g.permanents().all(|o| !o.is_creature()),
            "{name}: no creatures"
        );
        end_step_of(&mut t, P0);
        assert!(t.in_exile("Holy Strength"), "{name}: remains exiled");
        assert!(!t.on_battlefield(aura), "{name}");
    }
}

#[test]
fn argent_sphinx_returns_under_its_activators_control_for_good() {
    cr!("603.7e", "110.2a");
    ruling!(
        "Argent Sphinx",
        "If you control an Argent Sphinx owned by another player and activate its ability, Argent Sphinx will return to the battlefield under your control at the beginning of the next end step. You’ll retain control of it indefinitely."
    );
    // "Metalcraft — {U}: Exile this creature. Return it to the battlefield under your
    // control at the beginning of the next end step. Activate only if you control three
    // or more artifacts."
    supported("Argent Sphinx");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 1);
    for _ in 0..3 {
        t.battlefield(P0, "Ornithopter");
    }
    let sphinx = t.battlefield(P1, "Argent Sphinx");
    give_control(&mut t, sphinx, P0);
    assert_eq!(t.obj_now(sphinx).controller, P0);
    t.activate(P0, sphinx, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Argent Sphinx"));
    end_step_of(&mut t, P0);
    let now = t.g.current(sphinx);
    assert!(t.on_battlefield(now));
    assert_eq!(t.obj_now(now).owner, P1);
    assert_eq!(t.obj_now(now).controller, P0);
    // Still P0's on later turns.
    end_step_of(&mut t, P1);
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.obj_now(now).controller, P0);
}

#[test]
fn shifty_doppelganger_returns_only_if_it_sacrificed_the_creature() {
    cr!("603.7c");
    ruling!(
        "Shifty Doppelganger",
        "If the creature is not still on the battlefield at beginning of the End step, then you do not return this card to the battlefield."
    );
    // "{3}{U}, Exile this creature: You may put a creature card from your hand onto the
    // battlefield. If you do, that creature gains haste until end of turn. At the
    // beginning of the next end step, sacrifice that creature. If you do, return this card
    // to the battlefield."
    supported("Shifty Doppelganger");
    for creature_stays in [true, false] {
        let mut t = TestGame::new(2);
        mana(&mut t, P0, 2);
        let shifty = t.battlefield(P0, "Shifty Doppelganger");
        let wurm = t.hand(P0, "Craw Wurm");
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[o(wurm)]);
        t.activate(P0, shifty, 0, &[]).unwrap();
        t.resolve_all();
        assert!(t.in_exile("Shifty Doppelganger"));
        let wurm_now = t.g.current(wurm);
        assert!(t.on_battlefield(wurm_now), "{}", t.dump_log());
        assert!(crate::r_s06_common::has_kw(
            &t,
            wurm_now,
            mtg_engine::keywords::KeywordKind::Haste
        ));
        if !creature_stays {
            move_to(&mut t, wurm_now, Zone::Hand(P0));
        }
        end_step_of(&mut t, P0);
        if creature_stays {
            assert!(t.in_graveyard(P0, "Craw Wurm"), "sacrificed");
            assert!(t.on_battlefield(shifty), "returned");
        } else {
            assert!(t.in_hand(P0, "Craw Wurm"));
            assert!(t.in_exile("Shifty Doppelganger"), "not returned");
        }
    }
}

#[test]
fn shifty_doppelganger_may_put_nothing_onto_the_battlefield() {
    cr!("608.2d");
    ruling!(
        "Shifty Doppelganger",
        "This card has received errata such that putting a creature card from your hand onto the battlefield is now optional. This means you no longer need a third-party to verify that you don't have any creature cards in your hand if you don't put something onto the battlefield."
    );
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let shifty = t.battlefield(P0, "Shifty Doppelganger");
    t.hand(P0, "Craw Wurm");
    t.answer_yes(P0, false);
    t.activate(P0, shifty, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Craw Wurm"), "kept in hand");
    assert!(t.named_on_battlefield("Craw Wurm").is_empty());
    end_step_of(&mut t, P0);
    assert!(t.in_exile("Shifty Doppelganger"), "nothing was sacrificed");
}

// ---------------------------------------------------------------------------------------
// Interactions of the same abilities (no ruling of their own).
// ---------------------------------------------------------------------------------------

#[test]
fn the_eternal_wanderer_returns_the_card_at_its_owners_next_end_step() {
    cr!("603.7", "603.7b");
    // "+1: Exile up to one target artifact or creature. Return that card to the
    // battlefield under its owner's control at the beginning of that player's next end
    // step."
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P1, "Gray Ogre");
    exile_with(&mut t, "The Eternal Wanderer", ogre);
    end_step_of(&mut t, P0);
    assert!(t.in_exile("Gray Ogre"), "not at P0's end step");
    end_step_of(&mut t, P1);
    assert!(t.on_battlefield(ogre));
    assert_eq!(t.obj_now(ogre).controller, P1);
}

#[test]
fn phelia_gets_a_counter_only_for_a_card_that_returns_under_your_control() {
    cr!("603.7", "110.2");
    // "Whenever Phelia attacks, exile up to one other target nonland permanent. At the
    // beginning of the next end step, return that card to the battlefield under its
    // owner's control. If it entered under your control, put a +1/+1 counter on Phelia."
    for mine in [true, false] {
        let mut t = TestGame::new(2);
        let owner = if mine { P0 } else { P1 };
        let ogre = t.battlefield(owner, "Gray Ogre");
        let phelia = exile_with(&mut t, "Phelia, Exuberant Shepherd", ogre);
        assert!(t.in_exile("Gray Ogre"));
        end_step_of(&mut t, P0);
        assert!(t.on_battlefield(ogre));
        assert_eq!(t.obj_now(ogre).controller, owner);
        assert_eq!(
            t.counters(phelia, counters::PLUS1),
            u32::from(mine),
            "mine: {mine}"
        );
    }
}

#[test]
fn teferis_time_twist_adds_a_counter_to_ones_it_would_enter_with() {
    cr!("614.1c", "122.6");
    // Walking Ballista ("This creature enters with X +1/+1 counters on it.") isn't cast:
    // X is 0, so it would enter as a 0/0; with Teferi's Time Twist's counter it's a 1/1.
    let mut t = TestGame::new(2);
    let ballista = t.battlefield(P0, "Walking Ballista");
    put(&mut t, ballista, counters::PLUS1, 2);
    exile_with(&mut t, "Teferi's Time Twist", ballista);
    end_step_of(&mut t, P0);
    assert!(t.on_battlefield(ballista), "{}", t.dump_log());
    assert_eq!(t.counters(ballista, counters::PLUS1), 1);
    assert_eq!(t.pt(ballista), (1, 1));
}

#[test]
fn a_card_whose_owner_left_the_game_doesnt_return() {
    cr!("800.4a", "603.7c");
    let mut t = TestGame::new(3);
    let ogre = t.battlefield(P2, "Gray Ogre");
    exile_with(&mut t, "Flickerwisp", ogre);
    assert!(t.in_exile("Gray Ogre"));
    crate::r_p079_common::concede(&mut t, P2);
    end_step_of(&mut t, P0);
    assert!(!t.on_battlefield(ogre));
    assert!(t.g.permanents().all(|o| o.chars.name != "Gray Ogre"));
}
