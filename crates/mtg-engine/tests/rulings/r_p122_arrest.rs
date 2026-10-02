//! Rulings batch P122 — Auras such as Faith's Fetters and Lawmage's Binding: activated
//! abilities (keyword abilities and loyalty abilities included) can't be activated, but
//! static and triggered abilities work and mana abilities may be "excepted" (CR 602.5,
//! 605.1a, 606.2); "can't crew"; Auras whose spells lose their target don't resolve
//! (CR 608.2b, 303.4a); Auras put onto the battlefield without being cast (CR 303.4f);
//! abilities of Auras that move them (CR 113.7a, 400.7, 602.2b).

use crate::r_p122_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s04_common::{crew, cycle};
use crate::r_s05_common::move_to;
use crate::r_s06_common::attach_new;
use crate::r_s20_common::tap_for_mana;
use crate::r_s27_common::can_activate_containing;
use crate::r_s28_common::cast_card;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn pool_all(t: &mut TestGame, p: PlayerId) {
    for ty in ManaType::ALL {
        mana(t, p, ty, 3);
    }
}

// ---------------------------------------------------------------------------------
// Activated abilities can't be activated; triggered abilities are unaffected.
// ---------------------------------------------------------------------------------

/// `aura` (attached by P1) stops Ainok Bond-Kin's outlast (a keyword that is an activated
/// ability) but not Soul Warden's triggered ability.
fn keyword_activated_stopped_triggered_works(aura: &str) {
    supported(aura);
    let mut t = TestGame::new(2);
    let ainok = t.battlefield(P0, "Ainok Bond-Kin");
    pool_all(&mut t, P0);
    assert!(can_activate_containing(&mut t, P0, ainok, "Outlast"));
    attach_new(&mut t, P1, aura, ainok);
    pool_all(&mut t, P0);
    assert!(
        !can_activate_containing(&mut t, P0, ainok, "Outlast"),
        "{aura}: outlast is an activated ability"
    );
    let warden = t.battlefield(P0, "Soul Warden");
    attach_new(&mut t, P1, aura, warden);
    t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 21, "{aura}: triggered abilities still trigger");
}

#[test]
fn nahiris_binding_activated_not_triggered() {
    cr!("602.5", "603.2");
    ruling!(
        "Nahiri's Binding",
        "Some keyword abilities are activated abilities and will have a colon in their reminder text. Triggered abilities (starting with \"when,\" \"whenever,\" or \"at\") are unaffected."
    );
    keyword_activated_stopped_triggered_works("Nahiri's Binding");
}

#[test]
fn deserts_hold_activated_not_triggered() {
    cr!("602.5", "603.2");
    ruling!(
        "Desert's Hold",
        "Triggered abilities (starting with \"when,\" \"whenever,\" or \"at\") are unaffected by the last ability of Desert's Hold."
    );
    keyword_activated_stopped_triggered_works("Desert's Hold");
}

#[test]
fn trapped_in_the_tower_activated_not_triggered() {
    cr!("602.5", "603.2");
    ruling!(
        "Trapped in the Tower",
        "Triggered abilities (starting with “when,” “whenever,” or “at”) are unaffected by Trapped in the Tower."
    );
    keyword_activated_stopped_triggered_works("Trapped in the Tower");
}

#[test]
fn demotion_activated_not_triggered() {
    cr!("602.5", "603.2");
    ruling!(
        "Demotion",
        "Triggered abilities (starting with “when,” “whenever,” or “at”) are unaffected by Demotion."
    );
    keyword_activated_stopped_triggered_works("Demotion");
}

#[test]
fn lawmages_binding_activated_not_triggered() {
    cr!("602.5", "603.2");
    ruling!(
        "Lawmage's Binding",
        "Triggered abilities (starting with “when,” “whenever,” or “at”) are unaffected by Lawmage’s Binding."
    );
    keyword_activated_stopped_triggered_works("Lawmage's Binding");
}

/// `aura` stops a planeswalker's loyalty abilities and an Equipment's equip ability.
fn loyalty_and_equip_stopped(aura: &str) {
    supported(aura);
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace Beleren");
    let bonesplitter = t.battlefield(P0, "Bonesplitter");
    t.battlefield(P0, "Grizzly Bears");
    pool_all(&mut t, P0);
    assert!(can_activate_containing(&mut t, P0, jace, "+2"));
    assert!(can_activate_containing(&mut t, P0, bonesplitter, "Equip"));
    attach_new(&mut t, P1, aura, jace);
    attach_new(&mut t, P1, aura, bonesplitter);
    pool_all(&mut t, P0);
    assert!(
        !can_activate_containing(&mut t, P0, jace, ""),
        "{aura}: loyalty abilities are activated abilities"
    );
    assert!(
        !can_activate_containing(&mut t, P0, bonesplitter, "Equip"),
        "{aura}: equip is an activated ability"
    );
}

#[test]
fn faiths_fetters_stops_equip_and_loyalty_abilities() {
    cr!("602.5", "606.2", "702.6a");
    ruling!(
        "Faith's Fetters",
        "Some keywords (such as equip) are activated abilities and will have colons in their reminder text. Loyalty abilities of planeswalkers are activated abilities."
    );
    loyalty_and_equip_stopped("Faith's Fetters");
}

#[test]
fn planar_disruption_stops_keyword_and_loyalty_abilities() {
    cr!("602.5", "606.2", "702.6a");
    ruling!(
        "Planar Disruption",
        "Some keywords are activated abilities and will have colons in their reminder text. Notably, loyalty abilities of planeswalkers are activated abilities."
    );
    loyalty_and_equip_stopped("Planar Disruption");
}

#[test]
fn suppression_bonds_stops_keyword_and_loyalty_abilities() {
    cr!("602.5", "606.2", "702.6a");
    ruling!(
        "Suppression Bonds",
        "Some keywords are activated abilities and will have colons in their reminder texts. The loyalty abilities of planeswalkers are activated abilities."
    );
    loyalty_and_equip_stopped("Suppression Bonds");
}

/// `aura` on Llanowar Elves (mana ability) and Soul Warden (triggered), and on Glorious
/// Anthem (static): only "unless they're mana abilities" Auras.
fn static_triggered_and_mana_abilities_work(aura: &str) {
    supported(aura);
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    attach_new(&mut t, P1, aura, elves);
    assert!(tap_for_mana(&mut t, P0, elves, "{G}"), "{aura}: mana ability");
    let anthem = t.battlefield(P0, "Glorious Anthem");
    attach_new(&mut t, P1, aura, anthem);
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(bears), (3, 3), "{aura}: static ability");
    let warden = t.battlefield(P0, "Soul Warden");
    attach_new(&mut t, P1, aura, warden);
    t.enter(P1, "Savannah Lions");
    t.resolve_all();
    assert_eq!(t.life(P0), 21, "{aura}: triggered ability");
}

#[test]
fn bound_in_gold_static_triggered_and_mana_abilities() {
    cr!("605.1a", "602.5", "603.2");
    ruling!(
        "Bound in Gold",
        "Bound in Gold doesn’t stop static abilities from affecting the game, and it doesn’t stop triggered abilities from triggering. It also doesn’t stop mana abilities from being activated."
    );
    static_triggered_and_mana_abilities_work("Bound in Gold");
}

#[test]
fn intercessors_arrest_static_triggered_and_mana_abilities() {
    cr!("605.1a", "602.5", "603.2");
    ruling!(
        "Intercessor's Arrest",
        "Intercessor's Arrest doesn't stop static abilities from affecting the game, and it doesn't stop triggered abilities from triggering. It also doesn't stop mana abilities from being activated."
    );
    static_triggered_and_mana_abilities_work("Intercessor's Arrest");
}

#[test]
fn faiths_fetters_static_triggered_and_mana_abilities() {
    cr!("605.1a", "602.5", "603.2");
    ruling!(
        "Faith's Fetters",
        "Faith's Fetters doesn't stop static abilities, triggered abilities, or mana abilities from working. A mana ability is an ability that produces mana, not an ability that costs mana."
    );
    static_triggered_and_mana_abilities_work("Faith's Fetters");
    // An ability that costs mana isn't a mana ability: Prodigal Sorcerer's is stopped.
    let mut t = TestGame::new(2);
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    attach_new(&mut t, P1, "Faith's Fetters", sorcerer);
    assert!(!can_activate_containing(&mut t, P0, sorcerer, "damage"));
}

/// The creature enchanted by `aura` can't be tapped to crew a Vehicle.
fn cant_crew(aura: &str) {
    supported(aura);
    let mut t = TestGame::new(2);
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P1, aura, bears);
    assert!(!crew(&mut t, P0, copter, &[bears]), "{aura}");
    assert!(!t.obj_now(bears).tapped);
    let lions = t.battlefield(P0, "Savannah Lions");
    assert!(crew(&mut t, P0, copter, &[lions]));
}

#[test]
fn intercessors_arrest_cant_crew() {
    cr!("702.122a");
    ruling!(
        "Intercessor's Arrest",
        "A creature that “can't crew Vehicles” can't be tapped to pay the crew cost of a Vehicle."
    );
    cant_crew("Intercessor's Arrest");
}

#[test]
fn bound_in_gold_cant_crew() {
    cr!("702.122a");
    ruling!(
        "Bound in Gold",
        "A creature that “can’t crew Vehicles” can’t be tapped to pay the crew cost of a Vehicle."
    );
    cant_crew("Bound in Gold");
}

// ---------------------------------------------------------------------------------
// Zirda, the Dawnwaker.
// ---------------------------------------------------------------------------------

#[test]
fn zirda_reduces_cycling_a_keyword_activated_ability() {
    cr!("602.2b", "702.29a");
    ruling!(
        "Zirda, the Dawnwaker",
        "Some keyword abilities are activated abilities (such as cycling) and will have colons in their reminder text."
    );
    supported("Zirda, the Dawnwaker");
    let mut t = TestGame::new(2);
    let meadow = t.hand(P0, "Drifting Meadow");
    // Cycling {2} costs {1} (not less than one mana).
    assert!(cycle(&mut t, P0, meadow, 0).is_err());
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zirda, the Dawnwaker");
    let meadow = t.hand(P0, "Drifting Meadow");
    assert!(cycle(&mut t, P0, meadow, 0).is_err(), "at least one mana");
    mana(&mut t, P0, ManaType::C, 1);
    cycle(&mut t, P0, meadow, 0).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Drifting Meadow"));
}

#[test]
fn zirda_companion_basic_lands_have_activated_mana_abilities() {
    cr!("702.139a", "305.6");
    ruling!(
        "Zirda, the Dawnwaker",
        "Land cards with basic land types have intrinsic activated mana abilities associated with those types."
    );
    use mtg_engine::card::{card, CardDef};
    use mtg_engine::game::GameConfig;
    use std::sync::Arc;
    let copies = |name: &str, n: usize| -> Vec<Arc<CardDef>> { (0..n).map(|_| card(name)).collect() };
    let revealable = |deck: Vec<Arc<CardDef>>| -> bool {
        let config = GameConfig {
            starting_player: Some(P0),
            ..Default::default()
        };
        let mut t = crate::r_s13_common::pregame(config, vec![deck, copies("Forest", 60)]);
        let side = t.g.add_to_sideboard(P0, vec![card("Zirda, the Dawnwaker")]);
        t.answer_choose(P0, &[Entity::Object(side[0])]);
        t.g.start();
        t.g.companion_of(P0) == Some(side[0])
    };
    // Forests (the intrinsic "{T}: Add {G}") and Llanowar Elves: each permanent card has
    // an activated ability.
    let mut deck = copies("Forest", 24);
    deck.extend(copies("Llanowar Elves", 36));
    assert!(revealable(deck));
    let mut deck = copies("Forest", 24);
    deck.extend(copies("Grizzly Bears", 36));
    assert!(!revealable(deck));
}

// ---------------------------------------------------------------------------------
// Aura spells and Aura abilities.
// ---------------------------------------------------------------------------------

#[test]
fn faiths_fetters_with_an_illegal_target_doesnt_enter() {
    cr!("608.2b", "303.4a");
    ruling!(
        "Faith's Fetters",
        "If the target permanent is an illegal target by the time Faith's Fetters tries to resolve, it doesn't resolve. It won't enter the battlefield, so its enters-the-battlefield ability won't trigger."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    cast_card(&mut t, P0, "Faith's Fetters");
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Faith's Fetters"));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn undying_rage_with_an_illegal_target_isnt_returned() {
    cr!("608.2b", "303.4a");
    ruling!(
        "Undying Rage",
        "It's put into its owner's graveyard from the stack rather than from the battlefield, so its last ability won't trigger."
    );
    supported("Undying Rage");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    cast_card(&mut t, P0, "Undying Rage");
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Undying Rage"));
    // From the battlefield, it returns to its owner's hand.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Undying Rage", bears);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.in_hand(P0, "Undying Rage"));
}

#[test]
fn bound_in_silence_put_onto_the_battlefield_attaches_without_targeting() {
    cr!("303.4f", "303.4g");
    ruling!(
        "Bound in Silence",
        "This doesn't target that creature, so you could have it enter the battlefield attached to a creature an opponent controls with hexproof, for example. If there's no creature on the battlefield it can be attached to, it stays in whatever zone it was in."
    );
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P1, "Gladecover Scout");
    t.answer_choose(P0, &[obj(scout)]);
    let aura = t.enter(P0, "Bound in Silence");
    assert_eq!(t.g.obj(aura).attached_to, Some(obj(scout)));
    assert!(!t.g.can_block_at_all(scout));
    // No creature to attach it to: it stays in the graveyard.
    let mut t = TestGame::new(2);
    let aura = t.graveyard(P0, "Bound in Silence");
    move_to(&mut t, aura, Zone::Battlefield);
    assert!(t.in_graveyard(P0, "Bound in Silence"));
}

/// `aura` (attached by P0 to P1's creature) is activated, and destroyed in response: it
/// stays in the graveyard (CR 400.7). It can't be activated from the graveyard.
fn return_ability_after_aura_left(aura_name: &str, needle: &str) {
    supported(aura_name);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = attach_new(&mut t, P0, aura_name, bears);
    pool_all(&mut t, P0);
    activate_containing_ok(&mut t, aura, needle);
    destroy(&mut t, aura);
    t.resolve_all();
    assert!(t.in_graveyard(P0, aura_name), "{aura_name}");
    assert!(!t.in_hand(P0, aura_name));
    pool_all(&mut t, P0);
    let in_gy = t.g.find_in_zone(Zone::Graveyard(P0), aura_name)[0];
    assert!(!can_activate_containing(&mut t, P0, in_gy, needle));
}

fn activate_containing_ok(t: &mut TestGame, source: ObjectId, needle: &str) {
    crate::r_s06_common::activate_containing(t, P0, source, needle).unwrap();
}

#[test]
fn cage_of_hands_gone_before_its_ability_resolves() {
    cr!("400.7", "113.7a");
    ruling!(
        "Cage of Hands",
        "If Cage of Hands is no longer on the battlefield when the ability resolves, Cage of Hands remains in its new zone and isn't returned to its owner's hand."
    );
    return_ability_after_aura_left("Cage of Hands", "Return");
}

#[test]
fn forced_worship_gone_before_its_ability_resolves() {
    cr!("400.7", "113.7a");
    ruling!(
        "Forced Worship",
        "Forced Worship's activated ability may only be activated if Forced Worship is on the battlefield. If it's no longer on the battlefield when the ability resolves, the ability has no effect."
    );
    return_ability_after_aura_left("Forced Worship", "Return");
}

/// Only the controller of `aura_name` (P0) may activate its ability, not the enchanted
/// creature's controller.
fn only_controller_activates(aura_name: &str) {
    supported(aura_name);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = attach_new(&mut t, P0, aura_name, bears);
    pool_all(&mut t, P1);
    assert!(!can_activate_containing(&mut t, P1, aura, "Exile"), "{aura_name}");
    pool_all(&mut t, P0);
    assert!(can_activate_containing(&mut t, P0, aura, "Exile"), "{aura_name}");
}

#[test]
fn cooped_up_only_its_controller_activates() {
    cr!("602.2");
    ruling!("Cooped Up", "Only the controller of Cooped Up may activate its ability.");
    only_controller_activates("Cooped Up");
}

#[test]
fn sigardas_imprisonment_only_its_controller_activates() {
    cr!("602.2");
    ruling!(
        "Sigarda's Imprisonment",
        "Only the controller of Sigarda's Imprisonment may activate its activated ability."
    );
    only_controller_activates("Sigarda's Imprisonment");
}

#[test]
fn krasis_incubation_returned_as_a_cost() {
    cr!("602.2b", "601.2h", "113.7a");
    ruling!(
        "Krasis Incubation",
        "Returning Krasis Incubation to its owner's hand is part of the cost to activate its last ability."
    );
    supported("Krasis Incubation");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = attach_new(&mut t, P0, "Krasis Incubation", bears);
    pool_all(&mut t, P0);
    activate_containing_ok(&mut t, aura, "counters");
    // Paid as the ability is activated, before anyone can respond.
    assert!(t.in_hand(P0, "Krasis Incubation"));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 2);
}

#[test]
fn arachnus_web_checks_each_end_step() {
    cr!("603.4");
    ruling!(
        "Arachnus Web",
        "Arachnus Web's triggered ability checks at the beginning of each end step, not just yours."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Craw Wurm");
    let web = attach_new(&mut t, P0, "Arachnus Web", giant);
    t.set_step(P1, Step::PostcombatMain);
    t.advance_to_step(Step::End);
    t.resolve_all();
    assert_eq!(t.g.turn.active, P1);
    assert!(!t.on_battlefield(web));
    assert!(t.in_graveyard(P0, "Arachnus Web"));
}

#[test]
fn bribers_purse_with_no_gem_counters() {
    cr!("602.2b", "107.3m");
    ruling!(
        "Briber's Purse",
        "If Briber’s Purse has no gem counters on it, it remains on the battlefield, although you can’t activate its last ability."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let purse = t.hand(P0, "Briber's Purse");
    t.cast(P0, purse).x(0).go();
    t.resolve_all();
    let purse = t.g.current(purse);
    assert!(t.on_battlefield(purse));
    assert_eq!(t.counters(purse, "gem"), 0);
    pool_all(&mut t, P0);
    assert!(!can_activate_containing(&mut t, P0, purse, "gem"));
}
