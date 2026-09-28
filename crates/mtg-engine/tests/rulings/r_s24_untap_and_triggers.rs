//! Rulings batch S24 — effects on permanents you control at a later time: "Lands you
//! control don't untap during your next untap step" (CR 502.3), abilities that trigger an
//! additional time when a creature dies (CR 603.2d, 603.10a), keeping creatures with a
//! total power (CR 101.4, 107.1b), and conditions checked as an ability resolves
//! (CR 608.2h).

use crate::r_s01_common::supported;
use crate::r_s05_common::move_to;
use crate::r_s06_common::attach_new;
use crate::r_s24_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 casts the real spell `name` with lands made for it, and it resolves. Returns an
/// untapped Forest P0 got while the spell was on the stack.
fn cast_last(t: &mut TestGame, name: &str) -> ObjectId {
    crate::r_s01_common::give_mana_for(t, P0, name);
    let spell = t.hand(P0, name);
    t.cast(P0, spell).go();
    let forest = t.battlefield(P0, "Forest");
    t.resolve_all();
    forest
}

/// Taps every land `p` controls.
fn tap_all_lands(t: &mut TestGame, p: PlayerId) {
    let lands: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|o| o.controller == p && o.is(mtg_engine::types::CardType::Land))
        .map(|o| o.id)
        .collect();
    for l in lands {
        t.g.objects[l.0 as usize].tapped = true;
    }
}

#[test]
fn no_lands_you_control_untap_during_your_next_untap_step() {
    cr!("502.3", "611.2c");
    ruling!(
        "Rhonas's Last Stand",
        "No lands that you control will untap during your next untap step, even lands that aren’t tapped as this spell resolves. This includes lands that enter the battlefield after this spell resolves."
    );
    supported("Rhonas's Last Stand");
    // "Create a 5/4 green Snake creature token. Lands you control don't untap during your
    // next untap step."
    let mut t = TestGame::new(2);
    let untapped = cast_last(&mut t, "Rhonas's Last Stand");
    assert!(!tapped(&t, untapped));
    // A land that enters afterwards, and all of them tapped later this turn.
    let later = t.enter(P0, "Forest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    tap_all_lands(&mut t, P0);
    t.g.objects[bears.0 as usize].tapped = true;
    // The opponent's untap step isn't P0's: their lands untap.
    tap_all_lands(&mut t, P1);
    let theirs = t.battlefield(P1, "Forest");
    t.g.objects[theirs.0 as usize].tapped = true;
    t.advance_to(P1, Step::Upkeep);
    assert!(!tapped(&t, theirs));
    t.advance_to(P0, Step::Upkeep);
    assert!(tapped(&t, untapped));
    assert!(tapped(&t, later));
    assert!(!tapped(&t, bears), "only lands are affected");
    // The untap step after that one is normal.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!tapped(&t, untapped));
    assert!(!tapped(&t, later));
}

#[test]
fn several_dont_untap_effects_all_wear_off_during_that_untap_step() {
    cr!("502.3");
    ruling!(
        "Bontu's Last Reckoning",
        "If more than one spell says that lands you control don’t untap during your next untap step, the effects will all wear off during that untap step. You’ll untap lands you control during your untap step after that one."
    );
    supported("Bontu's Last Reckoning");
    let mut t = TestGame::new(2);
    let forest = cast_last(&mut t, "Rhonas's Last Stand");
    cast_last(&mut t, "Bontu's Last Reckoning");
    tap_all_lands(&mut t, P0);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(tapped(&t, forest));
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!tapped(&t, forest));
}

#[test]
fn a_creature_dying_with_a_leaving_permanent_makes_its_ability_trigger_again() {
    cr!("603.2d", "603.10a");
    ruling!(
        "Teysa Karlov",
        "If a creature dying at the same time that another permanent you control leaves the battlefield causes a triggered ability of that permanent to trigger, that ability triggers an additional time."
    );
    supported("Teysa Karlov");
    supported("Blood Artist");
    supported("Pyroclasm");
    // Blood Artist: "Whenever this creature or another creature dies, target player loses
    // 1 life and you gain 1 life." Pyroclasm deals 2 damage to each creature: Blood
    // Artist and the Bears die together; Teysa (2/4) survives.
    for teysa in [false, true] {
        let mut t = TestGame::new(2);
        if teysa {
            t.battlefield(P0, "Teysa Karlov");
        }
        t.battlefield(P0, "Blood Artist");
        t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, "Mountain", 2);
        for _ in 0..4 {
            t.answer_targets(P0, &[Entity::Player(P1)]);
        }
        let pyroclasm = t.hand(P0, "Pyroclasm");
        t.cast(P0, pyroclasm).go();
        t.resolve();
        assert!(t.in_graveyard(P0, "Blood Artist"));
        assert_eq!(t.stack_len(), if teysa { 4 } else { 2 });
        t.resolve_all();
        assert_eq!(t.life(P1), if teysa { 16 } else { 18 });
    }
}

#[test]
fn a_negative_power_subtracts_from_the_total_power_kept() {
    cr!("107.1b", "101.4");
    ruling!(
        "Slaughter the Strong",
        "If a creature's power is somehow less than 0, it subtracts from the total power of the other creatures its controller chooses. This can cause creatures with power 5 or greater to survive."
    );
    supported("Slaughter the Strong");
    supported("Weakness");
    // "Each player chooses any number of creatures they control with total power 4 or
    // less, then sacrifices all other creatures they control."
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Colossal Dreadmaw");
    let wall = t.battlefield(P0, "Wall of Stone");
    // Weakness: "Enchanted creature gets -2/-1." The Wall is -2/7.
    attach_new(&mut t, P0, "Weakness", wall);
    assert_eq!(t.pt(wall).0, -2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let lions = t.battlefield(P1, "Savannah Lions");
    t.battlefield(P1, "Grizzly Bears");
    // P0 keeps the Dreadmaw (6) and the Wall (-2): a total of 4.
    t.answer_choose(P0, &[Entity::Object(wurm), Entity::Object(wall)]);
    // P1 tries to keep more than 4 total power: not a legal choice.
    t.answer_choose(P1, &[Entity::Object(giant), Entity::Object(lions)]);
    crate::r_s01_common::give_mana_for(&mut t, P0, "Slaughter the Strong");
    let slaughter = t.hand(P0, "Slaughter the Strong");
    t.cast(P0, slaughter).go();
    t.resolve_all();
    assert!(t.on_battlefield(wurm));
    assert!(t.on_battlefield(wall));
    assert!(!t.on_battlefield(bears));
    // P1 kept creatures with total power 4 or less.
    let kept: i32 = t
        .g
        .permanents()
        .filter(|o| o.controller == P1 && o.is_creature())
        .map(|o| o.power())
        .sum();
    assert!(kept <= 4);
    assert!(!t.on_battlefield(giant) || !t.on_battlefield(lions));
}

#[test]
fn the_empires_artifacts_are_checked_as_the_ability_resolves() {
    cr!("608.2h", "608.2c");
    ruling!(
        "Scepter of Empires",
        "Whether or not you control the correct artifacts is determined when the ability resolves."
    );
    supported("Scepter of Empires");
    // "{T}: Scepter of Empires deals 1 damage to target player or planeswalker. It deals 3
    // damage instead if you control artifacts named Crown of Empires and Throne of
    // Empires."
    let mut t = TestGame::new(2);
    let scepter = t.battlefield(P0, "Scepter of Empires");
    t.battlefield(P0, "Crown of Empires");
    let throne = t.battlefield(P0, "Throne of Empires");
    t.activate(P0, scepter, 0, &[Entity::Player(P1)])
        .expect("activate");
    move_to(&mut t, throne, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // Without the Throne as it's activated, but with it as it resolves.
    let mut t = TestGame::new(2);
    let scepter = t.battlefield(P0, "Scepter of Empires");
    t.battlefield(P0, "Crown of Empires");
    t.activate(P0, scepter, 0, &[Entity::Player(P1)])
        .expect("activate");
    t.battlefield(P0, "Throne of Empires");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // An opponent's Throne doesn't count.
    let mut t = TestGame::new(2);
    let scepter = t.battlefield(P0, "Scepter of Empires");
    t.battlefield(P0, "Crown of Empires");
    t.battlefield(P1, "Throne of Empires");
    t.activate(P0, scepter, 0, &[Entity::Player(P1)])
        .expect("activate");
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}
