//! Rulings batch P165 — colorless synergies (Abstruse Archaic, Titan's Presence, Tomb of
//! the Spirit Dragon, Ghostfire Blade, Glaring Fleshraker, Twins of Discord's bloodthirst,
//! Ugin's Construct) and Sylvan Echoes (clashing).

use crate::r_p189_common::triggers_of;
use crate::r_s01_common::{custom_card, supported, tokens};
use crate::r_s06_common::activate_containing;
use crate::r_s11_common::manifest_card;
use crate::r_s25_common::{cast_new, lands_for_cost, targets_of};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn o(id: ObjectId) -> Entity {
    Entity::Object(id)
}

// ---------------------------------------------------------------------------------------
// Sylvan Echoes
// ---------------------------------------------------------------------------------------

#[test]
fn sylvan_echoes_triggers_when_you_win_an_opponents_clash() {
    cr!("701.30d", "603.2");
    ruling!(
        "Sylvan Echoes",
        "If you win a clash initiated by a spell or ability an opponent controls, the ability will still trigger."
    );
    supported("Sylvan Echoes");
    supported("Lash Out");
    let mut t = TestGame::new(2);
    let echoes = t.battlefield(P0, "Sylvan Echoes");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // P0 reveals Hill Giant (4) and P1 a filler (0): P0 wins P1's clash.
    t.library_top(P0, "Hill Giant");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.answer_yes(P0, true);
    cast_new(&mut t, P1, "Lash Out", &[o(bears)]);
    t.resolve();
    assert_eq!(triggers_of(&t, echoes), 1);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // P0 won, so Lash Out's controller didn't: no 3 damage to P0.
    assert_eq!(t.life(P0), 20);
}

// ---------------------------------------------------------------------------------------
// Abstruse Archaic
// ---------------------------------------------------------------------------------------

#[test]
fn abstruse_archaic_creates_another_instance_without_granting_abilities() {
    cr!("707.10", "113.2c");
    ruling!(
        "Abstruse Archaic",
        "Abstruse Archaic's ability targets an ability that is on the stack and creates another instance of that ability on the stack. It doesn't cause any object to gain an ability."
    );
    supported("Abstruse Archaic");
    supported("Ichor Wellspring");
    let mut t = TestGame::new(2);
    let archaic = t.battlefield(P0, "Abstruse Archaic");
    let well = t.enter(P0, "Ichor Wellspring");
    t.settle();
    let abilities = t.obj_now(well).chars.abilities.len();
    let trig = t.g.stack[0];
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[o(trig)]);
    activate_containing(&mut t, P0, archaic, "Copy target").expect("archaic");
    t.resolve();
    // The copy and the original are on the stack.
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.obj_now(well).chars.abilities.len(), abilities);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn abstruse_archaic_can_copy_emblem_abilities_but_not_the_monarchs() {
    cr!("707.10", "114.4", "724.2");
    ruling!(
        "Abstruse Archaic",
        "Abstruse Archaic's ability can copy activated and triggered abilities of dungeons, emblems, planes, and phenomenons. It can't copy the Planechase-specific \"planeswalking ability\" or the triggered abilities associated with the monarch or initiative designations, as those abilities have no source."
    );
    // The monarch's end step draw trigger: no source, not a legal target.
    let mut t = TestGame::new(2);
    let archaic = t.battlefield(P0, "Abstruse Archaic");
    mtg_engine::designations::become_monarch(&mut t.g, P0);
    t.advance_to(P0, mtg_engine::turn::Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let trig = t.g.stack[0];
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[o(trig)]);
    let r = activate_containing(&mut t, P0, archaic, "Copy target");
    let copied = r.as_ref().is_ok_and(|a| a.is_some())
        && t.g
            .stack
            .iter()
            .any(|id| targets_of(&t, *id).contains(&o(trig)));
    assert!(!copied);
    // An emblem's triggered ability (an emblem is colorless): it can be copied.
    let mut t = TestGame::new(2);
    let archaic = t.battlefield(P0, "Abstruse Archaic");
    let def = custom_card(
        "Emblem Source",
        "Artifact",
        "",
        None,
        "Whenever you cast a spell, you gain 1 life.",
    );
    let abilities = def.front().chars.abilities.clone();
    let emblem = mtg_engine::tokens::create_emblem(&mut t.g, P0, abilities, None);
    t.g.recompute();
    cast_new(&mut t, P0, "Opt", &[]);
    t.settle();
    let trig = t.g.stack[t.g.stack.len() - 1];
    assert_ne!(trig, emblem);
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[o(trig)]);
    activate_containing(&mut t, P0, archaic, "Copy target")
        .expect("archaic")
        .expect("on the stack");
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn abstruse_archaic_copy_of_an_imprint_ability_gives_two_exiled_cards() {
    cr!("707.10", "607.2a");
    ruling!(
        "Abstruse Archaic",
        "In some cases involving linked abilities, an ability requires information about \"the exiled card.\" When this happens, the ability gets multiple answers. If these answers are being used to determine the value of a variable, the sum is used. If an ability attempts to create a token that is a copy of \"the exiled card,\" it creates a token for each card exiled this way that is a copy of that card."
    );
    supported("Soul Foundry");
    let mut t = TestGame::new(2);
    let archaic = t.battlefield(P0, "Abstruse Archaic");
    let bears = t.hand(P0, "Grizzly Bears");
    let giant = t.hand(P0, "Hill Giant");
    let foundry = t.enter(P0, "Soul Foundry");
    t.settle();
    let trig = t.g.stack[0];
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[o(trig)]);
    activate_containing(&mut t, P0, archaic, "Copy target").expect("archaic");
    t.resolve(); // Archaic's ability: the copy
    // The copy exiles the Bears, the original the Hill Giant.
    t.answer_choose(P0, &[o(bears)]);
    t.answer_choose(P0, &[o(giant)]);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Hill Giant"));
    // X is the sum of their mana values: 6.
    t.lands(P0, "Wastes", 6);
    t.answer(P0, DecisionKind::X, Answer::Number(6));
    activate_containing(&mut t, P0, foundry, "Create a token")
        .expect("foundry")
        .expect("on the stack");
    t.resolve_all();
    let toks = tokens(&t, P0);
    let mut names: Vec<String> = toks
        .iter()
        .map(|id| t.obj_now(*id).chars.name.to_string())
        .collect();
    names.sort();
    assert_eq!(names, vec!["Grizzly Bears", "Hill Giant"]);
}

// ---------------------------------------------------------------------------------------
// Titan's Presence
// ---------------------------------------------------------------------------------------

/// P0 casts Titan's Presence revealing Matter Reshaper (3/2) and targeting `target`.
fn titans_presence(t: &mut TestGame, target: ObjectId) -> (ObjectId, ObjectId) {
    supported("Titan's Presence");
    supported("Matter Reshaper");
    let reshaper = t.hand(P0, "Matter Reshaper");
    t.answer_choose(P0, &[o(reshaper)]);
    let tp = cast_new(t, P0, "Titan's Presence", &[o(target)]);
    (tp, reshaper)
}

#[test]
fn titans_presence_compares_power_on_resolution() {
    cr!("608.2h", "601.2b");
    ruling!(
        "Titan's Presence",
        "Check the target creature’s power as Titan’s Presence resolves to see if it’s less than or equal to revealed creature card’s power. If it’s greater, Titan’s Presence will finish resolving with no effect."
    );
    for pump in [false, true] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        titans_presence(&mut t, giant);
        if pump {
            cast_new(&mut t, P1, "Giant Growth", &[o(giant)]);
            t.resolve();
        }
        t.resolve_all();
        assert_eq!(t.on_battlefield(giant), pump, "pumped: {pump}");
    }
}

#[test]
fn titans_presence_keeps_the_card_revealed_and_uses_its_last_known_power() {
    cr!("701.20a", "608.2h");
    ruling!(
        "Titan's Presence",
        "The colorless creature card stays revealed until Titan’s Presence leaves the stack."
    );
    ruling!(
        "Titan's Presence",
        "The colorless creature card you reveal doesn’t have to still be in your hand as Titan’s Presence resolves. If it’s not, use its power as it last existed in your hand."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let (_, reshaper) = titans_presence(&mut t, giant);
    assert!(mtg_engine::reveal::is_revealed(&t.g, reshaper));
    // The revealed card leaves the hand (discarded) before resolution.
    t.g.move_object(
        reshaper,
        Zone::Graveyard(P0),
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.g.flush_events();
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    // Another one stays revealed only while the spell is on the stack.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let (_, reshaper) = titans_presence(&mut t, bears);
    assert!(mtg_engine::reveal::is_revealed(&t.g, reshaper));
    t.resolve_all();
    assert!(!mtg_engine::reveal::is_revealed(&t.g, reshaper));
}

// ---------------------------------------------------------------------------------------
// Tomb of the Spirit Dragon, Ghostfire Blade
// ---------------------------------------------------------------------------------------

#[test]
fn tomb_of_the_spirit_dragon_counts_colorless_creatures_on_resolution() {
    cr!("608.2h", "708.2");
    ruling!(
        "Tomb of the Spirit Dragon",
        "Count the number of colorless creatures you control (including face-down creatures) as the last ability resolves to determine how much life you gain."
    );
    supported("Tomb of the Spirit Dragon");
    let mut t = TestGame::new(2);
    let tomb = t.battlefield(P0, "Tomb of the Spirit Dragon");
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    activate_containing(&mut t, P0, tomb, "gain").expect("tomb");
    // A face-down creature arrives in response.
    manifest_card(&mut t, P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn face_down_and_artifact_creatures_are_colorless_for_ghostfire_blade() {
    cr!("708.2", "105.2c");
    ruling!(
        "Ghostfire Blade",
        "Face-down creatures and most artifact creatures are colorless."
    );
    supported("Ghostfire Blade");
    for which in ["face-down", "artifact", "green"] {
        let mut t = TestGame::new(2);
        let blade = t.battlefield(P0, "Ghostfire Blade");
        let creature = match which {
            "face-down" => manifest_card(&mut t, P0, "Grizzly Bears"),
            "artifact" => t.battlefield(P0, "Ornithopter"),
            _ => t.battlefield(P0, "Grizzly Bears"),
        };
        // {1}: enough only for the reduced equip cost.
        t.lands(P0, "Wastes", 1);
        let ok = t.activate(P0, blade, 0, &[o(creature)]).is_ok();
        assert_eq!(ok, which != "green", "{which}");
    }
}

// ---------------------------------------------------------------------------------------
// Glaring Fleshraker
// ---------------------------------------------------------------------------------------

#[test]
fn glaring_fleshrakers_cast_trigger_resolves_even_if_the_spell_is_countered() {
    cr!("603.3", "601.2i");
    ruling!(
        "Glaring Fleshraker",
        "Glaring Fleshraker's first ability goes on the stack above the spell that caused it to trigger and resolves first. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    supported("Glaring Fleshraker");
    let mut t = TestGame::new(2);
    let gf = t.battlefield(P0, "Glaring Fleshraker");
    let thopter = cast_new(&mut t, P0, "Ornithopter", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.g.stack[0], thopter);
    assert_eq!(triggers_of(&t, gf), 1);
    cast_new(&mut t, P1, "Counterspell", &[o(thopter)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Ornithopter"));
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}

// ---------------------------------------------------------------------------------------
// Twins of Discord, Ugin's Construct
// ---------------------------------------------------------------------------------------

#[test]
fn twins_of_discord_bloodthirst_is_cumulative() {
    cr!("702.54c", "702.54a");
    ruling!(
        "Twins of Discord",
        "Multiple instances of bloodthirst are cumulative. For example, if you control Twins of Discord and a colorless creature with bloodthirst 1 enters the battlefield under your control, that creature will enter the battlefield with three +1/+1 counters on it if an opponent has been dealt damage that turn."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Twins of Discord");
    let def = custom_card(
        "Thirsty Construct",
        "Artifact Creature — Construct",
        "{2}",
        Some((1, 1)),
        "Bloodthirst 1",
    );
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    let card = t.custom(P0, def, Zone::Hand(P0));
    t.lands(P0, "Wastes", 2);
    t.cast(P0, card).go();
    t.resolve_all();
    let c = t.g.current(card);
    assert_eq!(t.counters(c, "+1/+1"), 3);
}

#[test]
fn ugins_construct_sacrifices_nothing_if_you_control_only_colorless_permanents() {
    cr!("701.21a", "105.2c");
    ruling!(
        "Ugin's Construct",
        "If you control only colorless permanents as the ability resolves (for example, basic lands and face-down permanents), you won’t sacrifice anything."
    );
    supported("Ugin's Construct");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let fd = manifest_card(&mut t, P0, "Grizzly Bears");
    let from = t.asked().len();
    let construct = t.enter(P0, "Ugin's Construct");
    t.resolve_all();
    assert!(t.on_battlefield(construct));
    assert!(t.on_battlefield(fd));
    assert_eq!(t.named_on_battlefield("Forest").len(), 2);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseEntities { .. })));
    // With a colored permanent, it must go.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.enter(P0, "Ugin's Construct");
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    let _ = lands_for_cost;
}
