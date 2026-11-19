//! Rulings residue (batches P207–P213, Auras and Equipment, and the keywords grouped with
//! them): Shadow of the Second Sun's extra phases, The Masamune's extra triggers, Curse of
//! Shaken Faith, echo (Shah of Naar Isle), emerge (Herigast), encore (Eumidian Wastewaker),
//! entwine (Plunge into Darkness, Grab the Reins), Helm of Kaldra and Spy Kit.

use crate::r_p056_common::{count, steps_of_this_turn};
use crate::r_p210_common::lots_of_mana;
use crate::r_residue_enchant_equip_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s06_common::*;
use crate::r_s25_common::{cast_new, creature_tokens};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// --- Shadow of the Second Sun -------------------------------------------------------------

const SHADOW: &str = "Shadow of the Second Sun";

/// Shadow of the Second Sun enchanting P0, and the game in the precombat main phase of
/// P0's second turn (the starting player skips only the draw of their first turn).
fn shadowed() -> TestGame {
    supported(SHADOW);
    let mut t = TestGame::new(2);
    aura_on(&mut t, P1, SHADOW, P0);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t
}

#[test]
fn shadow_of_the_second_sun_triggers_without_an_attack() {
    cr!("505.1", "500.8");
    ruling!(
        "Shadow of the Second Sun",
        "Even if the enchanted player doesn't attack with any creatures during combat, they'll still get a postcombat main phase and Shadow of the Second Sun's ability will still trigger."
    );
    let mut t = shadowed();
    let steps = steps_of_this_turn(&mut t);
    assert_eq!(count(&steps, Step::Upkeep), 1 + 1, "{steps:?} {}", t.dump_log());
    let main2 = steps.iter().position(|s| *s == Step::PostcombatMain).unwrap();
    assert_eq!(steps[main2 + 1], Step::Untap, "{steps:?}");
}

#[test]
fn shadow_of_the_second_sun_with_an_added_combat_and_more_main_phases() {
    cr!("500.8", "505.1a", "505.1");
    ruling!(
        "Shadow of the Second Sun",
        "If multiple phases are added to the same point in your turn, the most recently created phase happens first."
    );
    ruling!(
        "Shadow of the Second Sun",
        "If the enchanted player somehow has more than two main phases in a turn, each main phase other than the first one is a postcombat main phase, and Shadow of the Second Sun's ability triggers at the beginning of each of them."
    );
    supported("Relentless Assault");
    // In P0's postcombat main phase, after the Shadow's trigger resolved, P0 casts
    // Relentless Assault ("Untap all creatures that attacked this turn. After this main
    // phase, there is an additional combat phase followed by an additional main phase."):
    // the combat comes first; its main phase is a postcombat main phase and the Shadow
    // triggers again.
    let mut t = shadowed();
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    t.g.turn.priority = Some(P0);
    cast_new(&mut t, P0, "Relentless Assault", &[]);
    t.resolve_all();
    let steps = steps_of_this_turn(&mut t);
    let main2 = steps.iter().position(|s| *s == Step::PostcombatMain).unwrap();
    assert_eq!(steps[main2 + 1], Step::BeginningOfCombat, "{steps:?}");
    assert_eq!(count(&steps, Step::PostcombatMain), 2, "{steps:?}");
    // Two additional beginning phases, both after the added combat.
    assert_eq!(count(&steps, Step::Upkeep), 3, "{steps:?}");
    let combat2 = steps
        .iter()
        .rposition(|s| *s == Step::BeginningOfCombat)
        .unwrap();
    let first_extra_untap = steps.iter().skip(1).position(|s| *s == Step::Untap).unwrap() + 1;
    assert!(first_extra_untap > combat2, "{steps:?}");
}

#[test]
fn shadow_of_the_second_sun_extra_beginning_phase_is_a_real_one() {
    cr!("500.8", "502.3", "503.1a", "504.1");
    ruling!(
        "Shadow of the Second Sun",
        "During the untap step, permanents will phase in or out as appropriate and they'll untap their tapped permanents. Anything they control that triggers \"at the beginning of your upkeep\" or similar will trigger during their upkeep step, and they'll eventually draw a card for their draw step."
    );
    supported("Phyrexian Arena");
    let mut t = shadowed();
    t.battlefield(P0, "Phyrexian Arena");
    let land = t.battlefield(P0, "Forest");
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    t.g.obj_mut(land).tapped = true;
    let (hand, life) = (t.hand_size(P0), t.life(P0));
    t.advance_to(P0, Step::End);
    // Phyrexian Arena ("At the beginning of your upkeep, you draw a card and you lose 1
    // life") and the draw step: two cards.
    assert!(!t.obj_now(land).tapped);
    assert_eq!(t.hand_size(P0), hand + 2, "{}", t.dump_log());
    assert_eq!(t.life(P0), life - 1);
}

// --- The Masamune -------------------------------------------------------------------------

const MASAMUNE: &str = "The Masamune";

#[test]
fn the_masamune_leaves_the_battlefield_triggers_twice_when_dying() {
    cr!("603.2d", "603.10a", "700.4");
    ruling!(
        "The Masamune",
        "An ability of the equipped creature or an emblem you own that triggers when a creature \"leaves the battlefield\" will trigger twice if that creature leaves the battlefield by dying."
    );
    ruling!(
        "The Masamune",
        "If a creature dying at the same time as the equipped creature (including the equipped creature itself dying) causes a triggered ability of the equipped creature or an emblem you own to trigger, that ability triggers an additional time."
    );
    supported(MASAMUNE);
    supported("Thragtusk");
    // Thragtusk: "When this creature leaves the battlefield, create a 3/3 green Beast
    // creature token." It dies equipped: two Beasts.
    let mut t = TestGame::new(2);
    let tusk = t.battlefield(P0, "Thragtusk");
    attach_new(&mut t, P0, MASAMUNE, tusk);
    destroy(&mut t, tusk);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Beast").len(), 2, "{}", t.dump_log());
    // Exiled: once.
    let mut t = TestGame::new(2);
    let tusk = t.battlefield(P0, "Thragtusk");
    attach_new(&mut t, P0, MASAMUNE, tusk);
    t.g.move_object(tusk, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Beast").len(), 1);
    // Blood Artist dying with the Bears: each of its two triggers twice.
    let mut t = TestGame::new(2);
    let artist = t.battlefield(P0, "Blood Artist");
    attach_new(&mut t, P0, MASAMUNE, artist);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy_all(vec![artist, bears], None, false);
    t.g.flush_events();
    t.settle();
    assert_eq!(crate::r_s14_common::triggers_from(&t, artist), 4, "{}", t.dump_log());
}

#[test]
fn the_masamune_put_into_a_graveyard_from_the_battlefield_triggers_twice() {
    cr!("603.2d", "700.4");
    ruling!(
        "The Masamune",
        "The trigger event doesn't have to specifically refer to \"creatures.\""
    );
    supported("Molder Beast");
    // Molder Beast: "Whenever an artifact is put into a graveyard from the battlefield,
    // this creature gets +2/+0 until end of turn." An artifact creature dies: +4/+0; a
    // noncreature artifact: +2/+0.
    let mut t = TestGame::new(2);
    let beast = t.battlefield(P0, "Molder Beast");
    attach_new(&mut t, P0, MASAMUNE, beast);
    let (p, _) = t.pt(beast);
    let thopter = t.battlefield(P1, "Ornithopter");
    destroy(&mut t, thopter);
    t.resolve_all();
    assert_eq!(t.pt(beast).0, p + 4, "{}", t.dump_log());
    let rod = t.battlefield(P1, "Rod of Ruin");
    destroy(&mut t, rod);
    t.resolve_all();
    assert_eq!(t.pt(beast).0, p + 6);
}

#[test]
fn two_masamunes_on_one_creature_trigger_three_times() {
    cr!("603.2d", "700.4", "704.5j");
    ruling!(
        "The Masamune",
        "If they're equipped to the same creature, the abilities of the equipped creature will also trigger three times."
    );
    supported("Mirror Gallery");
    // Mirror Gallery: "The "legend rule" doesn't apply."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirror Gallery");
    let artist = t.battlefield(P0, "Blood Artist");
    attach_new(&mut t, P0, MASAMUNE, artist);
    attach_new(&mut t, P0, MASAMUNE, artist);
    t.settle();
    assert_eq!(t.named_on_battlefield(MASAMUNE).len(), 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    assert_eq!(crate::r_s14_common::triggers_from(&t, artist), 3, "{}", t.dump_log());
}

// --- Curse of Shaken Faith ----------------------------------------------------------------

#[test]
fn curse_of_shaken_faith_counts_spells_cast_before_it_arrived() {
    cr!("603.2", "700.14");
    ruling!(
        "Curse of Shaken Faith",
        "If a player who has already cast a spell this turn becomes enchanted by Curse of Shaken Faith, casting more spells the same turn will cause the ability to trigger."
    );
    supported("Curse of Shaken Faith");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.g.turn.priority = Some(P1);
    cast_new(&mut t, P1, "Divination", &[]);
    t.resolve_all();
    aura_on(&mut t, P0, "Curse of Shaken Faith", P1);
    t.g.turn.priority = Some(P1);
    cast_new(&mut t, P1, "Divination", &[]);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "{}", t.dump_log());
}

// --- Echo, emerge -------------------------------------------------------------------------

#[test]
fn shah_of_naar_isle_paying_zero_with_thick_skinned_goblin_is_paying_echo() {
    cr!("702.30a", "118.9", "603.2");
    ruling!(
        "Shah of Naar Isle",
        "If you use Thick-Skinned Goblin’s ability to pay {0} instead of Shah of Naar Isle’s echo cost, the Shah’s last ability will trigger."
    );
    supported("Shah of Naar Isle");
    supported("Thick-Skinned Goblin");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thick-Skinned Goblin");
    t.set_step(P0, Step::PrecombatMain);
    t.enter(P0, "Shah of Naar Isle");
    t.resolve_all();
    let hand = t.hand_size(P1);
    respond_all(&mut t);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Shah of Naar Isle").len(), 1, "{}", t.dump_log());
    assert_eq!(t.hand_size(P1), hand + 3, "{}", t.dump_log());
}

/// Every player answers yes to yes/no questions and the largest number to number choices.
fn respond_all(t: &mut TestGame) {
    for p in [P0, P1] {
        crate::r_s03_common::respond(t, p, |_, d| match d {
            Decision::YesNo { .. } => Some(Answer::Bool(true)),
            Decision::ChooseNumber { max, .. } => Some(Answer::Number(*max as i64)),
            _ => None,
        });
    }
}

#[test]
fn herigast_a_spell_with_two_emerge_costs_uses_either() {
    cr!("702.119a", "601.2b", "118.9");
    ruling!(
        "Herigast, Erupting Nullkite",
        "If a spell has more than one emerge cost, you may choose to pay it for any one of those emerge costs."
    );
    supported("Elder Deep-Fiend");
    // Elder Deep-Fiend ({8}, emerge {5}{U}{U}) sacrificing Hill Giant (mana value 4): its
    // own emerge costs {1}{U}{U}; Herigast's (its mana cost) {4}.
    for (land, n) in [("Island", 3), ("Wastes", 4)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        t.battlefield(P0, "Herigast, Erupting Nullkite");
        let giant = t.battlefield(P0, "Hill Giant");
        t.lands(P0, land, n);
        let fiend = t.hand(P0, "Elder Deep-Fiend");
        t.answer_choose(P0, &[obj(giant)]);
        t.answer_yes(P0, false);
        let r = t
            .cast(P0, fiend)
            .method(CastMethod::Keyword(KeywordKind::Emerge))
            .try_go();
        assert!(r.is_ok(), "{land}: {r:?} {}", t.dump_log());
        assert!(!t.on_battlefield(giant));
    }
}

// --- Encore -------------------------------------------------------------------------------

#[test]
fn eumidian_wastewaker_encore_is_a_cost_and_makes_a_token_for_each_opponent() {
    cr!("702.141a", "602.2b", "601.2h");
    ruling!(
        "Eumidian Wastewaker",
        "Encore is an activated ability that functions from the graveyard."
    );
    ruling!(
        "Eumidian Wastewaker",
        "Exiling the card with encore is a cost to activate the encore ability."
    );
    supported("Eumidian Wastewaker");
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let card = t.graveyard(P0, "Eumidian Wastewaker");
    lots_of_mana(&mut t, P0);
    activate_containing(&mut t, P0, card, "Encore").expect("encore");
    // Paid: the card is already in exile, with the ability on the stack.
    assert_eq!(t.stack_len(), 1);
    assert!(t.in_exile("Eumidian Wastewaker"));
    t.resolve_all();
    let tokens = creature_tokens(&t, P0);
    assert_eq!(tokens, 2);
    for tok in crate::r_s01_common::tokens(&t, P0) {
        assert!(has_kw(&t, tok, KeywordKind::Haste));
        assert_eq!(name(&t, tok), "Eumidian Wastewaker");
    }
    // In combat, it can't be activated (only as a sorcery) — and not from the hand.
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Eumidian Wastewaker");
    t.set_step(P0, Step::BeginningOfCombat);
    lots_of_mana(&mut t, P0);
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, card));
    let in_hand = t.hand(P0, "Eumidian Wastewaker");
    t.set_step(P0, Step::PrecombatMain);
    lots_of_mana(&mut t, P0);
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, in_hand));
}

// --- Entwine ------------------------------------------------------------------------------

#[test]
fn plunge_into_darkness_entwined_order_and_choices_on_resolution() {
    cr!("702.42a", "608.2c", "119.4");
    ruling!(
        "Plunge into Darkness",
        "Assuming the entwine cost has been paid, the effect is (1) sacrifice any number of creatures, (2) gain 3 life for each sacrificed creature, (3) choose X, (4) pay X life, and (5) look at the top X cards of your library, put one into your hand, and exile the rest."
    );
    ruling!(
        "Plunge into Darkness",
        "Everything other than the mode/entwine choice (and paying the {1}{B}, of course) is done on resolution."
    );
    supported("Plunge into Darkness");
    // At 2 life, P0 sacrifices two Bears (6 life), then pays 5 life: possible only because
    // the life is gained first.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.g.players[0].life = 2;
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Island", "Forest", "Swamp", "Plains", "Mountain"]);
    t.lands(P0, "Swamp", 3);
    let plunge = t.hand(P0, "Plunge into Darkness");
    let hand = t.hand_size(P0);
    t.g.turn.priority = Some(P0);
    t.cast(P0, plunge).kicked(true).go();
    // Nothing chosen yet but the entwine.
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    assert_eq!(t.life(P0), 3, "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.in_exile("Forest") || t.in_hand(P0, "Forest"));
}

#[test]
fn grab_the_reins_entwined_sacrifices_the_stolen_creature_without_priority() {
    cr!("702.42a", "608.2c", "117.1");
    ruling!(
        "Grab the Reins",
        "If you pay the entwine cost, you can sacrifice the creature you gain control of with Grab the Reins."
    );
    ruling!(
        "Grab the Reins",
        "If you cast this with Entwine, you don’t get priority in the middle of resolution."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P1, "Hill Giant");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 7);
    let grab = t.hand(P0, "Grab the Reins");
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[obj(giant)]);
    t.g.turn.priority = Some(P0);
    t.cast(P0, grab).kicked(true).go();
    let from = t.asked().len();
    t.g.resolve_top();
    // No priority while it resolved.
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    t.settle();
    assert!(t.in_graveyard(P1, "Hill Giant"), "{}", t.dump_log());
    assert_eq!(t.life(P1), 17);
    assert_eq!(creatures(&t, P0).len(), 1);
}

// --- Helm of Kaldra, Spy Kit --------------------------------------------------------------

#[test]
fn helm_of_kaldra_needs_all_three_as_it_resolves() {
    cr!("608.2c", "603.4");
    ruling!(
        "Helm of Kaldra",
        "The token isn’t put onto the battlefield unless you control all three Equipment when the ability resolves."
    );
    supported("Helm of Kaldra");
    supported("Sword of Kaldra");
    supported("Shield of Kaldra");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let helm = t.battlefield(P0, "Helm of Kaldra");
    let sword = t.battlefield(P0, "Sword of Kaldra");
    t.battlefield(P0, "Shield of Kaldra");
    t.lands(P0, "Wastes", 2);
    activate_containing(&mut t, P0, helm, "Kaldra, a legendary").expect("activate");
    // (Shield of Kaldra makes them indestructible: the Sword is bounced.)
    t.g.move_object(sword, Zone::Hand(P0), mtg_engine::events::MoveCause::Effect, None);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert!(t.named_on_battlefield("Kaldra").is_empty(), "{}", t.dump_log());
    // With all three: the token, with them attached.
    let sword = t.battlefield(P0, "Sword of Kaldra");
    activate_containing(&mut t, P0, helm, "Kaldra, a legendary").expect("activate");
    t.resolve_all();
    let kaldra = t.named_on_battlefield("Kaldra");
    assert_eq!(kaldra.len(), 1, "{}", t.dump_log());
    assert_eq!(attached_to(&t, sword), Some(obj(kaldra[0])));
}

#[test]
fn spy_kit_doesnt_change_the_draft_information_of_the_creature() {
    cr!("905.2b", "607.1", "201.2");
    ruling!(
        "Spy Kit",
        "These abilities are linked, so equipping Spy Kit to such a creature won't affect the set of cards it exiled or the information you noted."
    );
    supported("Animus of Predation");
    // Animus of Predation: "this creature has flying if you removed a creature card with
    // flying from the draft with cards named Animus of Predation" (and so on). P0 removed
    // a Serra Angel with a card named Cogwork Grinder: equipped with Spy Kit (it has that
    // name too), the Animus doesn't gain flying.
    let mut t = TestGame::new(2);
    t.g.start
        .draft
        .removed
        .push(mtg_engine::draft::RemovedCard {
            player: P0,
            with: "Cogwork Grinder".into(),
            card: card("Serra Angel"),
            face_up: true,
        });
    let animus = t.battlefield(P0, "Animus of Predation");
    attach_new(&mut t, P0, "Spy Kit", animus);
    assert!(t.obj_now(animus).chars.has_name("Cogwork Grinder"));
    assert!(!has_kw(&t, animus, KeywordKind::Flying));
    // Removed with a card named Animus of Predation: flying.
    t.g.start.draft.removed[0].with = "Animus of Predation".into();
    t.g.recompute();
    assert!(has_kw(&t, animus, KeywordKind::Flying));
}
