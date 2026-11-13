//! Rulings batch P189 — abilities that count the creature (or permanent) cards in your
//! graveyard ("undergrowth"): one-shot effects count once, as they resolve (CR 608.2h);
//! static abilities keep counting (CR 611.3a); intervening "if" clauses are checked both
//! when the ability triggers and when it resolves (CR 603.4).

use crate::r_p057_common::into_upkeep;
use crate::r_p076_common::mana;
use crate::r_p189_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, supported, with_subtype};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------
// One-shot effects: counted as the spell or ability resolves.
// ---------------------------------------------------------------------------

#[test]
fn grim_flowering_counts_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Grim Flowering",
        "Count the number of creature cards in your graveyard when Grim Flowering resolves."
    );
    supported("Grim Flowering");
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P0, "Grizzly Bears", 2);
    mana(&mut t, P0, ManaType::G, 6);
    let gf = t.hand(P0, "Grim Flowering");
    t.cast(P0, gf).go();
    // In response, a third creature card is put into the graveyard.
    t.graveyard(P0, "Hill Giant");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
}

#[test]
fn spider_spawning_counts_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Spider Spawning",
        "The number of creature cards in your graveyard is counted when Spider Spawning resolves."
    );
    supported("Spider Spawning");
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P0, "Grizzly Bears", 3);
    mana(&mut t, P0, ManaType::G, 5);
    let ss = t.hand(P0, "Spider Spawning");
    t.cast(P0, ss).go();
    // In response, one of them is exiled.
    let first = t.g.player(P0).graveyard[0];
    t.g.move_object(first, Zone::Exile, events::MoveCause::Effect, None);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Spider").len(), 2);
}

#[test]
fn rise_of_the_varmints_x_determined_once() {
    cr!("608.2h");
    ruling!(
        "Rise of the Varmints",
        "The value of X is determined only once, as Rise of the Varmints resolves."
    );
    supported("Rise of the Varmints");
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P0, "Grizzly Bears", 1);
    mana(&mut t, P0, ManaType::G, 4);
    let rv = t.hand(P0, "Rise of the Varmints");
    t.cast(P0, rv).go();
    t.graveyard(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Varmint").len(), 2);
    // Later changes don't create or remove tokens.
    t.graveyard(P0, "Hill Giant");
    t.settle();
    assert_eq!(with_subtype(&t, P0, "Varmint").len(), 2);
}

#[test]
fn hallowed_spiritkeeper_counts_itself() {
    cr!("608.2h", "603.10a");
    ruling!(
        "Hallowed Spiritkeeper",
        "Count the number of creature cards in your graveyard as the triggered ability resolves, including Hallowed Spiritkeeper itself if it's still there"
    );
    supported("Hallowed Spiritkeeper");
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P0, "Grizzly Bears", 1);
    let hs = t.battlefield(P0, "Hallowed Spiritkeeper");
    t.g.destroy(hs, None);
    t.g.flush_events();
    t.settle();
    // A creature card joins the graveyard before the trigger resolves.
    t.graveyard(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Spirit").len(), 3);
}

#[test]
fn kessig_cagebreakers_counts_on_resolution() {
    cr!("608.2h", "508.4");
    ruling!(
        "Kessig Cagebreakers",
        "You count the number of creature cards in your graveyard when the triggered ability resolves."
    );
    supported("Kessig Cagebreakers");
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P0, "Grizzly Bears", 1);
    let kc = t.battlefield(P0, "Kessig Cagebreakers");
    attack_with(&mut t, &[(kc, Entity::Player(P1))]);
    assert_eq!(stack_triggers_from(&t, kc).len(), 1);
    t.graveyard(P0, "Hill Giant");
    t.resolve_all();
    let wolves = with_subtype(&t, P0, "Wolf");
    assert_eq!(wolves.len(), 2);
    for w in &wolves {
        assert!(t.obj(*w).tapped);
        assert!(t.g.combat.as_ref().unwrap().attackers.iter().any(|a| a.id == *w));
    }
}

#[test]
fn kessig_cagebreakers_tokens_choose_what_they_attack() {
    cr!("508.4");
    ruling!(
        "Kessig Cagebreakers",
        "You declare which player or planeswalker each token is attacking as you put it onto the battlefield. It doesn't have to be the same player or planeswalker Kessig Cagebreakers is attacking."
    );
    supported("Kessig Cagebreakers");
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P0, "Grizzly Bears", 1);
    let jace = t.battlefield(P1, "Jace Beleren");
    let kc = t.battlefield(P0, "Kessig Cagebreakers");
    attack_with(&mut t, &[(kc, Entity::Player(P1))]);
    t.answer_choose(P0, &[Entity::Object(jace)]);
    t.answer_targets(P0, &[Entity::Object(jace)]);
    t.resolve_all();
    let wolf = with_subtype(&t, P0, "Wolf")[0];
    let combat = t.g.combat.as_ref().unwrap();
    let a = combat.attackers.iter().find(|a| a.id == wolf).unwrap();
    assert_eq!(a.target, Some(Entity::Object(jace)));
    let k = combat.attackers.iter().find(|a| a.id == kc).unwrap();
    assert_eq!(k.target, Some(Entity::Player(P1)));
}

#[test]
fn graveblade_marauder_counts_creatures_that_died_in_combat() {
    cr!("608.2h", "510.2");
    ruling!(
        "Graveblade Marauder",
        "Count the number of creature cards in your graveyard as the ability resolves to determine how much life is lost. Notably, this could include any other creature you owned that died in combat"
    );
    supported("Graveblade Marauder");
    let mut t = TestGame::new(2);
    let gm = t.battlefield(P0, "Graveblade Marauder");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(
        &mut t,
        &[(gm, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    block_and_finish(&mut t, P1, &[(giant, bears)]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // 1 combat damage, then 1 life for the Bears that died in the same damage step.
    assert_eq!(t.life(P1), 18);
}

#[test]
fn bladewing_counts_creatures_that_died_in_the_same_damage_step() {
    cr!("608.2h", "510.2", "603.2");
    ruling!(
        "Bladewing, Deathless Tyrant",
        "If any creature cards are in your graveyard as a result of dying in the same combat damage step that the ability triggered in, they will be counted."
    );
    supported("Bladewing, Deathless Tyrant");
    let mut t = TestGame::new(2);
    let bw = t.battlefield(P0, "Bladewing, Deathless Tyrant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(
        &mut t,
        &[(bw, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    block_and_finish(&mut t, P1, &[(giant, bears)]);
    assert_eq!(t.life(P1), 14);
    assert_eq!(with_subtype(&t, P0, "Zombie").len(), 1);
}

#[test]
fn cloud_of_darkness_x_calculated_once() {
    cr!("608.2h");
    ruling!(
        "Cloud of Darkness",
        "The value of X is calculated only once, as Cloud of Darkness's last ability resolves."
    );
    supported("Cloud of Darkness");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Forest");
    t.graveyard(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    let cloud = t.enter(P0, "Cloud of Darkness");
    t.settle();
    assert_eq!(stack_triggers_from(&t, cloud).len(), 1);
    // A third permanent card arrives before it resolves: X is 3.
    t.graveyard(P0, "Sol Ring");
    t.resolve_all();
    assert_eq!(t.pt(wurm), (3, 1));
    // Later changes don't change the -X/-X.
    t.graveyard(P0, "Forest");
    t.g.recompute();
    assert_eq!(t.pt(wurm), (3, 1));
}

#[test]
fn terror_tide_x_checked_once() {
    cr!("608.2h");
    ruling!(
        "Terror Tide",
        "The value of X is checked only once, as Terror Tide resolves."
    );
    supported("Terror Tide");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Forest");
    t.graveyard(P0, "Sol Ring");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    mana(&mut t, P0, ManaType::B, 4);
    let tt = t.hand(P0, "Terror Tide");
    t.cast(P0, tt).go();
    t.resolve_all();
    // X was 2: the Bears died (becoming a third permanent card), but the Wurm keeps -2/-2.
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.pt(wurm), (4, 2));
}

#[test]
fn strength_from_the_fallen_x_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Strength from the Fallen",
        "The value of X is determined when the ability resolves."
    );
    supported("Strength from the Fallen");
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P0, "Grizzly Bears", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let sf = t.enter(P0, "Strength from the Fallen");
    t.settle();
    assert_eq!(stack_triggers_from(&t, sf).len(), 1);
    t.graveyard(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    t.graveyard(P0, "Hill Giant");
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
}

// ---------------------------------------------------------------------------
// Static abilities: constantly updated.
// ---------------------------------------------------------------------------

#[test]
fn wreath_of_geists_constantly_updated() {
    cr!("611.3a", "613.4c");
    ruling!(
        "Wreath of Geists",
        "The value of X is constantly updated as creature cards are put into or removed from your graveyard."
    );
    supported("Wreath of Geists");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::G, 1);
    let w = t.hand(P0, "Wreath of Geists");
    t.cast(P0, w).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
    fill_graveyard(&mut t, P0, "Hill Giant", 2);
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
    exile_graveyard(&mut t, P0);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn lilianas_elite_applies_only_on_the_battlefield() {
    cr!("611.3a", "113.6");
    ruling!(
        "Liliana's Elite",
        "The ability applies only while Liliana's Elite is on the battlefield."
    );
    supported("Liliana's Elite");
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P0, "Grizzly Bears", 2);
    let le = t.battlefield(P0, "Liliana's Elite");
    assert_eq!(t.pt(le), (3, 3));
    let in_gy = t.graveyard(P0, "Liliana's Elite");
    let in_hand = t.hand(P0, "Liliana's Elite");
    t.g.recompute();
    assert_eq!(t.pt(in_gy), (1, 1));
    assert_eq!(t.pt(in_hand), (1, 1));
    // The one on the battlefield now counts the Elite in the graveyard too.
    assert_eq!(t.pt(le), (4, 4));
}

#[test]
fn moon_vigil_adherents_damage_becomes_lethal() {
    cr!("120.6", "704.5g", "611.3a");
    ruling!(
        "Moon-Vigil Adherents",
        "nonlethal damage dealt to Moon-Vigil Adherents may become lethal if the number of creatures you control or creature cards in your graveyard changes."
    );
    supported("Moon-Vigil Adherents");
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P0, "Grizzly Bears", 2);
    let mva = t.battlefield(P0, "Moon-Vigil Adherents");
    let src = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(mva), (3, 3));
    t.g.deal_damage(src, Entity::Object(mva), 2, false);
    t.settle();
    assert!(t.on_battlefield(mva));
    exile_graveyard(&mut t, P0);
    t.settle();
    assert!(!t.on_battlefield(mva));
    assert!(t.in_graveyard(P0, "Moon-Vigil Adherents"));
}

// ---------------------------------------------------------------------------
// Intervening "if" clauses (CR 603.4).
// ---------------------------------------------------------------------------

#[test]
fn gixian_skullflayer_checks_on_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Gixian Skullflayer",
        "Gixian Skullflayer's triggered ability checks the number of cards in your graveyard both when it triggers and when it resolves."
    );
    supported("Gixian Skullflayer");
    // Two creature cards: no trigger.
    let mut t = TestGame::new(2);
    let gs = t.battlefield(P0, "Gixian Skullflayer");
    fill_graveyard(&mut t, P0, "Grizzly Bears", 2);
    into_upkeep(&mut t, P0);
    assert_eq!(triggers_of(&t, gs), 0);
    // Three: it triggers, but they're exiled before it resolves.
    let mut t = TestGame::new(2);
    let gs = t.battlefield(P0, "Gixian Skullflayer");
    fill_graveyard(&mut t, P0, "Grizzly Bears", 3);
    into_upkeep(&mut t, P0);
    assert_eq!(stack_triggers_from(&t, gs).len(), 1);
    exile_graveyard(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(gs, counters::PLUS1), 0);
    // Three both times: a counter.
    let mut t = TestGame::new(2);
    let gs = t.battlefield(P0, "Gixian Skullflayer");
    fill_graveyard(&mut t, P0, "Grizzly Bears", 3);
    into_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(gs, counters::PLUS1), 1);
}

#[test]
fn mortal_combat_needs_twenty_as_upkeep_starts() {
    cr!("603.4");
    ruling!(
        "Mortal Combat",
        "If you don’t have twenty or more creature cards in your graveyard by the time your upkeep starts, the ability won’t trigger that turn."
    );
    supported("Mortal Combat");
    let mut t = TestGame::new(2);
    let mc = t.battlefield(P0, "Mortal Combat");
    fill_graveyard(&mut t, P0, "Grizzly Bears", 19);
    into_upkeep(&mut t, P0);
    assert_eq!(triggers_of(&t, mc), 0);
    // Reaching twenty during the upkeep is too late.
    t.graveyard(P0, "Grizzly Bears");
    t.resolve_all();
    t.advance_to(P0, Step::Draw);
    assert!(!t.has_lost(P1));
}

#[test]
fn mortal_combat_rechecks_on_resolution() {
    cr!("603.4", "104.2b");
    ruling!(
        "Mortal Combat",
        "If you don’t still have twenty creature cards in your graveyard, the ability does nothing."
    );
    supported("Mortal Combat");
    let mut t = TestGame::new(2);
    let mc = t.battlefield(P0, "Mortal Combat");
    fill_graveyard(&mut t, P0, "Grizzly Bears", 20);
    into_upkeep(&mut t, P0);
    assert_eq!(stack_triggers_from(&t, mc).len(), 1);
    let first = t.g.player(P0).graveyard[0];
    t.g.move_object(first, Zone::Exile, events::MoveCause::Effect, None);
    t.resolve_all();
    assert!(!t.has_lost(P1));
    // With twenty both times, you win.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mortal Combat");
    fill_graveyard(&mut t, P0, "Grizzly Bears", 20);
    into_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(t.has_lost(P1));
}

#[test]
fn shadowborn_demon_upkeep_checks_twice() {
    cr!("603.4");
    ruling!(
        "Shadowborn Demon",
        "The last ability checks whether you have fewer than six creature cards in your graveyard when it would trigger."
    );
    supported("Shadowborn Demon");
    // Six: no trigger.
    let mut t = TestGame::new(2);
    let sd = t.battlefield(P0, "Shadowborn Demon");
    fill_graveyard(&mut t, P0, "Grizzly Bears", 6);
    into_upkeep(&mut t, P0);
    assert_eq!(triggers_of(&t, sd), 0);
    // Five, then six before it resolves: nothing is sacrificed.
    let mut t = TestGame::new(2);
    let sd = t.battlefield(P0, "Shadowborn Demon");
    fill_graveyard(&mut t, P0, "Grizzly Bears", 5);
    into_upkeep(&mut t, P0);
    assert_eq!(stack_triggers_from(&t, sd).len(), 1);
    t.graveyard(P0, "Grizzly Bears");
    t.resolve_all();
    assert!(t.on_battlefield(sd));
    // Five both times: a creature is sacrificed.
    let mut t = TestGame::new(2);
    let sd = t.battlefield(P0, "Shadowborn Demon");
    fill_graveyard(&mut t, P0, "Grizzly Bears", 5);
    into_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(!t.on_battlefield(sd));
}

#[test]
fn shadowborn_demon_etb_is_mandatory() {
    cr!("603.3d", "115.1");
    ruling!(
        "Shadowborn Demon",
        "Shadowborn Demon’s enters-the-battlefield ability is mandatory. If you control the only non-Demon creature, you must choose it as the target."
    );
    supported("Shadowborn Demon");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sd = t.enter(P0, "Shadowborn Demon");
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(sd), "the Demon itself isn't a legal target");
}

// ---------------------------------------------------------------------------
// Aatchik, Emerald Radian; Necrotic Wound.
// ---------------------------------------------------------------------------

#[test]
fn aatchik_counts_artifact_creature_cards_once() {
    cr!("608.2h", "205.2a");
    ruling!(
        "Aatchik, Emerald Radian",
        "If a card in your graveyard is an artifact creature card, count it only once when determining how many Insect tokens to create"
    );
    supported("Aatchik, Emerald Radian");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Ornithopter");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Sol Ring");
    t.graveyard(P0, "Forest");
    t.enter(P0, "Aatchik, Emerald Radian");
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Insect").len(), 4, "Aatchik and three tokens");
}

#[test]
fn aatchik_dies_with_another_insect() {
    cr!("603.10a", "704.5g", "704.3");
    ruling!(
        "Aatchik, Emerald Radian",
        "If Aatchik is dealt lethal damage at the same time as another Insect you control, Aatchik’s last ability will still trigger"
    );
    supported("Aatchik, Emerald Radian");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    let a = t.enter(P0, "Aatchik, Emerald Radian");
    t.resolve_all();
    let insect = with_subtype(&t, P0, "Insect")
        .into_iter()
        .find(|i| *i != a)
        .unwrap();
    let src = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage_batch(
        vec![
            (src, Entity::Object(a), 3),
            (src, Entity::Object(insect), 1),
        ],
        false,
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert!(!t.on_battlefield(a));
    assert!(t.in_graveyard(P0, "Aatchik, Emerald Radian"));
}

#[test]
fn necrotic_wound_with_no_creature_cards() {
    cr!("614.1a", "608.2h");
    ruling!(
        "Necrotic Wound",
        "If you have no creature cards in your graveyard, the target creature gets -0/-0, and it will still be exiled if it would die this turn."
    );
    supported("Necrotic Wound");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    mana(&mut t, P0, ManaType::B, 1);
    let nw = t.hand(P0, "Necrotic Wound");
    t.cast(P0, nw).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
    t.g.destroy(bears, None);
    t.settle();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn necrotic_wound_exiles_if_it_dies_later_for_any_reason() {
    cr!("614.1a", "701.21a");
    ruling!(
        "Necrotic Wound",
        "Necrotic Wound's replacement effect will exile the target creature if it would die this turn for any reason"
    );
    supported("Necrotic Wound");
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P0, "Grizzly Bears", 1);
    let giant = t.battlefield(P1, "Hill Giant");
    mana(&mut t, P0, ManaType::B, 1);
    let nw = t.hand(P0, "Necrotic Wound");
    t.cast(P0, nw).target(giant).go();
    t.resolve_all();
    assert_eq!(t.pt(giant), (2, 2));
    // Later in the turn, its controller sacrifices it.
    t.g.sacrifice(giant, P1);
    t.settle();
    assert!(t.in_exile("Hill Giant"));
}
