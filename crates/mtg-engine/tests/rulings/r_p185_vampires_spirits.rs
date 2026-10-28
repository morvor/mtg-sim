//! Rulings batch P185 — Vampire, Spirit, Treefolk and shared-type cards: simultaneous
//! deaths and "dies" triggers (CR 603.10a, 704.3), illegal targets on resolution (CR
//! 608.2b), what a "Vampire creature spell" is (CR 707.5), mana abilities (CR 605.1a),
//! counts and choices made on resolution (CR 608.2h), cost reductions with splice (CR
//! 601.2f, 702.47), leaving the battlefield in response (CR 400.7), and last known
//! information (CR 608.2h, 113.7a).

use crate::r_p044_common::damage_at_once;
use crate::r_p079_common::concede;
use crate::r_p185_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::{create_token, destroy, target_candidates};
use crate::r_s05_common::move_to;
use crate::r_s24_common::enter_together;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn bloodlord_of_vaasgoth_doesnt_give_bloodthirst_to_a_clone_spell() {
    cr!("707.5", "702.54a", "601.2i");
    ruling!(
        "Bloodlord of Vaasgoth",
        "A creature spell like Phantasmal Image that may enter as a copy of a creature is not a Vampire creature spell, even if you intend on copying a Vampire creature when that creature enters."
    );
    supported("Bloodlord of Vaasgoth");
    supported("Phantasmal Image");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bloodlord of Vaasgoth");
    let vampire = t.battlefield(P0, "Arrogant Vampire");
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(vampire)]);
    let img = cast_new(&mut t, P0, "Phantasmal Image", &[]);
    t.resolve_all();
    let img = t.g.current(img);
    assert!(t.obj_now(img).chars.has_subtype("Vampire"));
    assert_eq!(t.counters(img, counters::PLUS1), 0);
    // A real Vampire creature spell gains bloodthirst 3.
    let v = cast_new(&mut t, P0, "Highborn Vampire", &[]);
    t.resolve_all();
    assert_eq!(t.counters(v, counters::PLUS1), 3);
}

#[test]
fn patron_of_the_vein_counts_vampires_there_as_its_trigger_resolves() {
    cr!("603.10a", "608.2h", "704.3");
    ruling!(
        "Patron of the Vein",
        "Destroying a creature with Patron of the Vein's first ability causes its last ability to trigger if Patron of the Vein is still on the battlefield."
    );
    ruling!(
        "Patron of the Vein",
        "If Patron of the Vein and a creature an opponent controls die simultaneously (perhaps because they fought or were in combat together), Patron of the Vein's last ability triggers, but Patron of the Vein won't be on the battlefield as that ability resolves. It can't be saved by the +1/+1 counter that would have been put on it. The same is true of any other Vampires you control that die at the same time. Your remaining Vampires will each get a +1/+1 counter."
    );
    ruling!(
        "Patron of the Vein",
        "Patron of the Vein's last ability puts a +1/+1 counter on each Vampire you control, including itself, even if the creature that died can't be exiled (most likely because it was a token)."
    );
    supported("Patron of the Vein");
    // Its enters ability destroys the creature: the last ability triggers.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let patron = enter_with(&mut t, P0, "Patron of the Vein", &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.counters(patron, counters::PLUS1), 1);
    // A token dies: it can't be exiled, but the Vampires still get counters.
    let tok = create_token(&mut t, P1, "Soldier");
    destroy(&mut t, tok);
    t.resolve_all();
    assert_eq!(t.counters(patron, counters::PLUS1), 2);
    // Patron, another Vampire and P1's Bears die together: the surviving Vampire alone
    // gets a counter.
    let mut t = TestGame::new(2);
    let patron = t.battlefield(P0, "Patron of the Vein");
    let dying = t.battlefield(P0, "Highborn Vampire");
    let surviving = t.battlefield(P0, "Arrogant Vampire");
    let bears = t.battlefield(P1, "Grizzly Bears");
    damage_at_once(
        &mut t,
        &[
            (Entity::Object(patron), 4),
            (Entity::Object(dying), 3),
            (Entity::Object(bears), 2),
        ],
    );
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(patron) && !t.on_battlefield(dying));
    assert_eq!(t.counters(surviving, counters::PLUS1), 1);
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn cordial_vampire_triggers_for_each_creature_dying_with_it_but_cant_save_them() {
    cr!("603.10a", "603.2c", "704.3");
    ruling!(
        "Cordial Vampire",
        "If Cordial Vampire dies at the same time as one or more other creatures, its ability triggers for each of them."
    );
    ruling!(
        "Cordial Vampire",
        "If a Vampire you control is dealt lethal damage at the same time as another creature, they die at the same time. That Vampire can't receive a counter from Cordial Vampire's ability in time to save it."
    );
    supported("Cordial Vampire");
    let mut t = TestGame::new(2);
    let cordial = t.battlefield(P0, "Cordial Vampire");
    let dying = t.battlefield(P0, "Arrogant Vampire");
    let surviving = t.battlefield(P0, "Highborn Vampire");
    let bears = t.battlefield(P1, "Grizzly Bears");
    damage_at_once(
        &mut t,
        &[
            (Entity::Object(cordial), 1),
            (Entity::Object(dying), 3),
            (Entity::Object(bears), 2),
        ],
    );
    assert_eq!(t.stack_len(), 3);
    t.resolve_all();
    assert!(!t.on_battlefield(dying));
    assert_eq!(t.counters(surviving, counters::PLUS1), 3);
}

#[test]
fn anowon_doesnt_make_a_player_without_a_non_vampire_sacrifice() {
    cr!("701.21a", "101.3");
    ruling!(
        "Anowon, the Ruin Sage",
        "If a player controls no creatures, or if all creatures a player controls are Vampires, that player simply doesn't sacrifice anything."
    );
    supported("Anowon, the Ruin Sage");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Anowon, the Ruin Sage");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let vamp = t.battlefield(P1, "Highborn Vampire");
    t.set_step(P0, Step::Untap);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(vamp));
    assert_eq!(t.named_on_battlefield("Anowon, the Ruin Sage").len(), 1);
}

#[test]
fn kalastria_highborn_triggers_per_vampire_and_does_nothing_with_an_illegal_target() {
    cr!("603.10a", "608.2b", "800.4a");
    ruling!(
        "Kalastria Highborn",
        "If multiple Vampires you control (possibly including Kalastria Highborn itself) are put into their owners' graveyards at the same time, Kalastria Highborn's ability triggers that many times."
    );
    ruling!(
        "Kalastria Highborn",
        "If the targeted player is an illegal target by the time the ability resolves (for example, if that player has left a multiplayer game), the entire ability doesn't resolve. You can't pay {B} and you don't gain life."
    );
    supported("Kalastria Highborn");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kalastria Highborn");
    t.battlefield(P0, "Highborn Vampire");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    for _ in 0..2 {
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.answer_yes(P0, true);
    }
    damage_at_once_all(&mut t);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P0), 24);
    // The targeted player leaves the game first.
    let mut t = TestGame::new(3);
    let kh = t.battlefield(P0, "Kalastria Highborn");
    t.lands(P0, "Swamp", 1);
    t.answer_targets(P0, &[Entity::Player(P2)]);
    t.answer_yes(P0, true);
    destroy(&mut t, kh);
    assert_eq!(t.stack_len(), 1);
    concede(&mut t, P2);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(tapped_lands(&t, P0), 0);
}

/// Deals 3 damage to each creature on the battlefield at once.
fn damage_at_once_all(t: &mut TestGame) {
    let hits: Vec<(Entity, u32)> = t
        .g
        .permanents()
        .filter(|o| o.is(mtg_engine::types::CardType::Creature))
        .map(|o| (Entity::Object(o.id), 3))
        .collect();
    damage_at_once(t, &hits);
}

#[test]
fn feast_of_blood_with_an_illegal_target_gains_no_life() {
    cr!("608.2b");
    ruling!(
        "Feast of Blood",
        "If the targeted creature is an illegal target by the time Feast of Blood resolves, the entire spell doesn’t resolve. You don’t gain life."
    );
    supported("Feast of Blood");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Highborn Vampire");
    t.battlefield(P0, "Arrogant Vampire");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Feast of Blood", &[Entity::Object(bears)]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn midnight_arsonist_cant_target_artifacts_with_mana_abilities() {
    cr!("605.1a", "115.1");
    ruling!(
        "Midnight Arsonist",
        "Midnight Arsonist's triggered ability can't target artifacts that have either activated or triggered mana abilities. An activated mana ability is one that could add mana as it resolves, doesn't have a target, and isn't a loyalty ability. A triggered mana ability is one that could add mana as it resolves and triggers based on an activated mana ability."
    );
    supported("Midnight Arsonist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Highborn Vampire");
    let ring = t.battlefield(P1, "Sol Ring");
    let crypt = t.battlefield(P1, "Tormod's Crypt");
    let from = t.asked().len();
    enter_with(&mut t, P0, "Midnight Arsonist", &[Entity::Object(crypt)]);
    let cands: Vec<Entity> = target_candidates(&t, P0, from).concat();
    assert!(cands.contains(&Entity::Object(crypt)));
    assert!(!cands.contains(&Entity::Object(ring)));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Tormod's Crypt"));
    assert!(t.on_battlefield(ring));
}

#[test]
fn mavren_fein_makes_a_token_even_if_the_attackers_leave() {
    cr!("508.1m", "603.2");
    ruling!(
        "Mavren Fein, Dusk Apostle",
        "Once you attack with one or more nontoken Vampires, Mavren Fein's ability will create a Vampire token even if some or all of those Vampires leave the battlefield or are removed from combat."
    );
    supported("Mavren Fein, Dusk Apostle");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mavren Fein, Dusk Apostle");
    let v = t.battlefield(P0, "Highborn Vampire");
    attack_with(&mut t, &[(v, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, v);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn blood_age_general_pumps_only_spirits_attacking_as_it_resolves_until_end_of_turn() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Blood Age General",
        "Blood Age General’s activated ability affects only Spirits that are attacking as it resolves. If you activate it while no Spirits are attacking, it will have no effect, even if you attack with one or more Spirits later in the turn."
    );
    ruling!(
        "Blood Age General",
        "Once the activated ability resolves, Spirits that get the +1/+0 bonus will retain it until end of turn, even after they stop attacking or if they stop being Spirits."
    );
    supported("Blood Age General");
    // Activated before combat: nothing.
    let mut t = TestGame::new(2);
    let gen = t.battlefield(P0, "Blood Age General");
    let mass = t.battlefield(P0, "Gnarled Mass");
    t.activate(P0, gen, 0, &[]).unwrap();
    t.resolve_all();
    attack_with(&mut t, &[(mass, Entity::Player(P1))]);
    assert_eq!(t.pt(mass), (3, 3));
    // Activated while it attacks: +1/+0 that stays after combat and after it stops being
    // a Spirit.
    let mut t = TestGame::new(2);
    let gen = t.battlefield(P0, "Blood Age General");
    let mass = t.battlefield(P0, "Gnarled Mass");
    let idle = t.battlefield(P0, "Raging Poltergeist");
    attack_with(&mut t, &[(mass, Entity::Player(P1))]);
    t.activate(P0, gen, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(mass), (4, 3));
    assert_eq!(t.pt(idle), (6, 1));
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.pt(mass), (4, 3));
    crate::r_s26_common::modify_until_eot(
        &mut t,
        mass,
        vec![mtg_engine::ability::Modification::RemoveSubtypes(vec![
            "Spirit".into()
        ])],
    );
    assert_eq!(t.pt(mass), (4, 3));
}

#[test]
fn clash_of_realities_grants_enters_triggers_and_typeless_creatures_are_non_spirits() {
    cr!("603.6a", "613.1f", "205.3m");
    ruling!(
        "Clash of Realities",
        "Clash of Realities's abilities apply as each creature enters and gives them an \"enters\" ability."
    );
    ruling!(
        "Clash of Realities",
        "Creatures with no creature type are non-Spirit creatures."
    );
    supported("Clash of Realities");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Clash of Realities");
    let typeless = vanilla(&mut t, P1, "Artifact Creature", (2, 2));
    t.answer_yes(P0, true);
    enter_with(&mut t, P0, "Gnarled Mass", &[Entity::Object(typeless)]);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(typeless));
    // A non-Spirit entering may deal 3 damage to a Spirit.
    let spirit = t.battlefield(P1, "Gnarled Mass");
    t.answer_yes(P0, true);
    enter_with(&mut t, P0, "Grizzly Bears", &[Entity::Object(spirit)]);
    t.resolve_all();
    assert!(!t.on_battlefield(spirit));
}

#[test]
fn nebelgast_herald_triggers_for_itself_and_a_spirit_entering_with_it() {
    cr!("603.6a", "603.2c");
    ruling!(
        "Nebelgast Herald",
        "If Nebelgast Herald and another Spirit enter at the same time, Nebelgast Herald’s last ability triggers once for each of them."
    );
    supported("Nebelgast Herald");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(a)]);
    t.answer_targets(P0, &[Entity::Object(b)]);
    enter_together(&mut t, &[(P0, "Nebelgast Herald"), (P0, "Gnarled Mass")]);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(t.obj_now(a).tapped && t.obj_now(b).tapped);
}

#[test]
fn quintorius_triggers_once_per_event_of_cards_leaving_the_graveyard() {
    cr!("603.2c", "603.10a");
    ruling!(
        "Quintorius, Field Historian",
        "If two or more cards leave your graveyard as a single event, Quintorius's last ability will trigger once. If they leave your graveyard as separate events, it will trigger once for each event."
    );
    supported("Quintorius, Field Historian");
    // One event: Tormod's Crypt exiles the whole graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Quintorius, Field Historian");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    let crypt = t.battlefield(P1, "Tormod's Crypt");
    t.set_step(P1, Step::PrecombatMain);
    t.activate(P1, crypt, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(tokens(&t, P0).len(), 1);
    // Two events: two tokens.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Quintorius, Field Historian");
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Hill Giant");
    move_to(&mut t, a, Zone::Exile);
    move_to(&mut t, b, Zone::Exile);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
}

#[test]
fn hikari_may_exile_itself_leaving_before_the_spell_resolves() {
    cr!("608.2b", "400.7", "603.7a");
    ruling!(
        "Hikari, Twilight Guardian",
        "If you exile Hikari, it leaves the battlefield before the spell that triggered it resolves its ability, so it will be unaffected by that spell."
    );
    ruling!("Hikari, Twilight Guardian", "You may choose not to exile Hikari.");
    supported("Hikari, Twilight Guardian");
    for exile in [true, false] {
        let mut t = TestGame::new(2);
        let hikari = t.battlefield(P0, "Hikari, Twilight Guardian");
        // P0's own Arcane Glacial Ray targets Hikari.
        t.answer_yes(P0, exile);
        let ray = cast_new(&mut t, P0, "Glacial Ray", &[Entity::Object(hikari)]);
        t.settle();
        t.resolve();
        assert_eq!(t.zone(ray), Zone::Stack);
        assert_eq!(t.zone(hikari) == Zone::Exile, exile);
        t.resolve_all();
        if exile {
            // The spell has no legal target; Hikari returns at the next end step as a new,
            // undamaged object.
            t.advance_to(P0, Step::End);
            t.resolve_all();
            let back = t.named_on_battlefield("Hikari, Twilight Guardian");
            assert_eq!(back.len(), 1);
            assert_eq!(marked(&t, back[0]), 0);
        } else {
            assert_eq!(marked(&t, hikari), 2);
        }
    }
}

#[test]
fn drogskol_cavalrys_own_spirit_tokens_trigger_its_life_gain() {
    cr!("603.6a", "111.1");
    ruling!(
        "Drogskol Cavalry",
        "The Spirit tokens created by Drogskol Cavalry will cause its second ability to trigger, as well as any other Spirits you cast or Spirit tokens you put onto the battlefield another way."
    );
    supported("Drogskol Cavalry");
    let mut t = TestGame::new(2);
    let dc = t.battlefield(P0, "Drogskol Cavalry");
    t.lands(P0, "Plains", 4);
    t.activate(P0, dc, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    enter_with(&mut t, P0, "Gnarled Mass", &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
}

#[test]
fn long_forgotten_gohei_reduces_the_total_cost_with_splice_once() {
    cr!("601.2f", "702.47a");
    ruling!(
        "Long-Forgotten Gohei",
        "The cost reduction applies to the total cost of the Arcane spell, including any additional costs from cards spliced onto it. However, the cost reduction applies only once to each Arcane spell. For example, if you cast an Arcane spell with mana cost {R} and splice onto it a card whose splice cost is {1}{U}, then the spell costs {U}{R} to cast. If you cast an Arcane spell with mana cost {4}{U} and splice onto it a card whose splice cost is {1}{R}, the spell costs {4}{U}{R} to cast."
    );
    supported("Long-Forgotten Gohei");
    // Lava Spike ({R}, 3 damage) with Glacial Ray ({1}{R}, 2 damage) spliced costs
    // {R}{R}; Glacial Ray with another Glacial Ray spliced costs {1}{R}{R} (reduced once).
    for (main, mountains, spliced) in [
        ("Lava Spike", 1, false),
        ("Lava Spike", 2, true),
        ("Glacial Ray", 2, false),
        ("Glacial Ray", 3, true),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Long-Forgotten Gohei");
        t.lands(P0, "Mountain", mountains);
        let spell = t.hand(P0, main);
        t.hand(P0, "Glacial Ray");
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
        let r = t
            .cast(P0, spell)
            .targets(&[Entity::Player(P1)])
            .targets(&[Entity::Player(P1)])
            .try_go();
        if r.is_ok() {
            t.resolve_all();
        }
        let dealt = 20 - t.life(P1);
        let alone = if main == "Lava Spike" { 3 } else { 2 };
        assert_eq!(dealt == alone + 2, spliced, "{main} with {mountains}: {r:?} {dealt}");
        if spliced {
            assert_eq!(tapped_lands(&t, P0), mountains);
        }
    }
}

#[test]
fn teller_of_tales_chooses_tap_or_untap_on_resolution() {
    cr!("608.2c", "603.3");
    ruling!(
        "Teller of Tales",
        "You choose whether or not to tap or untap the creature when the ability resolves."
    );
    supported("Teller of Tales");
    for (word, tapped_before, tapped_after) in [("untap", true, false), ("tap", false, true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Teller of Tales");
        let bears = t.battlefield(P1, "Grizzly Bears");
        if tapped_before {
            t.g.tap(bears);
        }
        let seen = pick_option_containing(&mut t, P0, word);
        t.answer_targets(P0, &[Entity::Object(bears)]);
        cast_new(&mut t, P0, "Gnarled Mass", &[]);
        t.settle();
        assert_eq!(t.stack_len(), 2);
        assert!(seen.lock().unwrap().is_empty(), "no choice before resolution");
        t.resolve();
        assert_eq!(seen.lock().unwrap().len(), 1, "{word}");
        assert_eq!(t.obj_now(bears).tapped, tapped_after, "{word}");
    }
}

#[test]
fn faces_of_the_past_taps_or_untaps_all_sharing_a_type_chosen_on_resolution() {
    cr!("608.2c");
    ruling!(
        "Faces of the Past",
        "You choose whether to tap or untap during resolution."
    );
    ruling!(
        "Faces of the Past",
        "You either tap all of them or untap all of them. You can’t tap some and untap others."
    );
    supported("Faces of the Past");
    for (word, after) in [("tap", [true, true, false]), ("untap", [false, false, false])] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Faces of the Past");
        let dying = t.battlefield(P1, "Grizzly Bears");
        let up = t.battlefield(P0, "Grizzly Bears");
        let down = t.battlefield(P1, "Runeclaw Bear");
        let other = t.battlefield(P1, "Hill Giant");
        t.g.tap(down);
        let seen = pick_option_containing(&mut t, P0, word);
        destroy(&mut t, dying);
        assert_eq!(t.stack_len(), 1);
        assert!(seen.lock().unwrap().is_empty());
        t.resolve_all();
        assert_eq!(seen.lock().unwrap().len(), 1, "{word}");
        let got = [
            t.obj_now(up).tapped,
            t.obj_now(down).tapped,
            t.obj_now(other).tapped,
        ];
        assert_eq!(got, after, "{word}");
    }
}

#[test]
fn orchard_warden_uses_the_treefolks_toughness_as_it_resolves_or_its_last_known() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Orchard Warden",
        "The creature’s toughness is checked as this ability resolves. If the creature has left the battlefield, use its last known information."
    );
    supported("Orchard Warden");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Orchard Warden");
    t.answer_yes(P0, true);
    let tree = enter_with(&mut t, P0, "Redwood Treefolk", &[]);
    assert_eq!(t.stack_len(), 1);
    pump(&mut t, tree, 0, 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 28);
    t.answer_yes(P0, true);
    let tree = enter_with(&mut t, P0, "Redwood Treefolk", &[]);
    pump(&mut t, tree, 0, 1);
    destroy(&mut t, tree);
    t.resolve_all();
    assert_eq!(t.life(P0), 35);
}

#[test]
fn basalt_ravager_counts_the_largest_shared_type_group_as_it_resolves() {
    cr!("608.2h");
    ruling!(
        "Basalt Ravager",
        "The amount of damage dealt is calculated as the triggered ability resolves based on the creatures you control at that time, including Basalt Ravager itself if it’s still under your control. You’ll always use the greatest number possible. You don’t need to choose any creature types. For example, if you control a Giant Wizard, a Giant Warrior, an Elf Warrior, and a Kor Warrior at that time, Basalt Ravager will deal 3 damage."
    );
    supported("Basalt Ravager");
    let mut t = TestGame::new(2);
    vanilla(&mut t, P0, "Creature — Giant Warrior", (1, 1));
    vanilla(&mut t, P0, "Creature — Elf Warrior", (1, 1));
    enter_with(&mut t, P0, "Basalt Ravager", &[Entity::Player(P1)]);
    assert_eq!(t.stack_len(), 1);
    // A Kor Warrior arrives before it resolves: three Warriors (and two Giants).
    vanilla(&mut t, P0, "Creature — Kor Warrior", (1, 1));
    // An opponent's Warrior isn't counted.
    vanilla(&mut t, P1, "Creature — Human Warrior", (1, 1));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}
