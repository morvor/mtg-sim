//! Rulings batch S10 — the Gods of Theros (indestructible): "As long as your devotion to
//! [color] is less than [N], [this God] isn't a creature." (CR 700.5 devotion, CR 613.1d
//! type-changing effects.)

use crate::r_s01_common::{attack_with, block_and_finish, give_mana_for, supported};
use crate::r_s03_common::to_blockers;
use crate::r_s04_common::spell_targets;
use crate::r_s05_common::{enter, move_to, run_from};
use crate::r_s10_common::*;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn is_creature(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).is(CardType::Creature)
}

#[test]
fn a_god_that_lost_all_abilities_still_stops_being_a_creature() {
    cr!("613.1d", "613.1f", "700.5");
    ruling!(
        "Erebos, God of the Dead",
        "If an effect causes a God to lose all abilities, its ability that causes it to stop being a creature still applies if appropriate."
    );
    supported("Erebos, God of the Dead");
    supported("Merfolk Trickster");
    // Erebos ({3}{B}) plus {B}{B}{B}{B}: devotion to black is five, it's a creature.
    let mut t = TestGame::new(2);
    let erebos = t.battlefield(P0, "Erebos, God of the Dead");
    let four = idol(&mut t, P0, "{B}{B}{B}{B}");
    assert!(is_creature(&t, erebos));
    // Merfolk Trickster: "tap target creature an opponent controls. It loses all
    // abilities until end of turn."
    t.answer_targets(P1, &[Entity::Object(erebos)]);
    enter(&mut t, P1, "Merfolk Trickster");
    t.resolve_all();
    assert!(!t.obj_now(erebos).has_keyword(KeywordKind::Indestructible));
    assert!(t.obj_now(erebos).tapped);
    assert!(is_creature(&t, erebos));
    // Devotion drops to one: the type-changing ability (layer 4) still applies although
    // the God has no abilities (layer 6).
    move_to(&mut t, four, Zone::Exile);
    assert!(!is_creature(&t, erebos));
    assert!(t.obj_now(erebos).is(CardType::Enchantment));
    assert!(!t.obj_now(erebos).chars.has_subtype("God"));
}

#[test]
fn a_god_entering_counts_its_own_mana_cost_for_enters_triggers_but_not_for_replacements() {
    cr!("700.5", "603.6a", "614.12");
    ruling!(
        "Erebos, God of the Dead",
        "When a God enters the battlefield, your devotion to its color (including the mana symbols in the mana cost of the God itself) will determine if a creature entered the battlefield or not"
    );
    ruling!(
        "Erebos, God of the Dead",
        "Because replacement effects are considered before the God is on the battlefield, the mana symbols in its mana cost won't be counted when determining this."
    );
    supported("Soul Warden");
    supported("Blind Obedience");
    // With {B}{B}{B} out, Erebos enters with a devotion of four: no creature entered.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul Warden");
    idol(&mut t, P0, "{B}{B}{B}");
    let erebos = enter(&mut t, P0, "Erebos, God of the Dead");
    t.resolve_all();
    assert!(!is_creature(&t, erebos));
    assert_eq!(t.life(P0), 20);
    // With {B}{B}{B}{B} out, its own {B} makes five as it's on the battlefield: a creature
    // entered and Soul Warden triggers. But as it was about to enter the devotion was
    // four, so the opponent's Blind Obedience ("Artifacts and creatures your opponents
    // control enter tapped") didn't apply to it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul Warden");
    t.battlefield(P1, "Blind Obedience");
    idol(&mut t, P0, "{B}{B}{B}{B}");
    let erebos = enter(&mut t, P0, "Erebos, God of the Dead");
    assert!(is_creature(&t, erebos));
    assert!(!t.obj_now(erebos).tapped);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    // With five other black mana symbols it's a creature as it's about to enter too.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Blind Obedience");
    idol(&mut t, P0, "{B}{B}{B}{B}{B}");
    let erebos = enter(&mut t, P0, "Erebos, God of the Dead");
    assert!(t.obj_now(erebos).tapped);
}

#[test]
fn replacement_effects_dont_count_the_entering_gods_own_mana_symbols() {
    cr!("700.5", "614.12");
    ruling!(
        "Erebos, Bleak-Hearted",
        "As a God enters the battlefield, your devotion to its color will determine whether any replacement effects that affect creatures entering the battlefield apply to that God."
    );
    supported("Erebos, Bleak-Hearted");
    supported("Grumgully, the Generous");
    // Grumgully: "Each other non-Human creature you control enters with an additional
    // +1/+1 counter on it." Four other black symbols: Erebos is a creature once it's on
    // the battlefield, but it didn't enter as one, so it gets no counter.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grumgully, the Generous");
    idol(&mut t, P0, "{B}{B}{B}{B}");
    let erebos = enter(&mut t, P0, "Erebos, Bleak-Hearted");
    assert!(is_creature(&t, erebos));
    assert_eq!(t.counters(erebos, counters::PLUS1), 0);
    // Five other black symbols: it enters as a creature and gets the counter.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grumgully, the Generous");
    idol(&mut t, P0, "{B}{B}{B}{B}{B}");
    let erebos = enter(&mut t, P0, "Erebos, Bleak-Hearted");
    assert_eq!(t.counters(erebos, counters::PLUS1), 1);
    assert_eq!(t.pt(erebos), (6, 7));
}

/// The God's type-changing ability works only on the battlefield: it's a creature card
/// in the hand and the graveyard, and a creature spell on the stack.
fn god_is_a_creature_outside_the_battlefield(name: &str) {
    supported(name);
    supported("Essence Scatter");
    let mut t = TestGame::new(2);
    let in_graveyard = t.graveyard(P0, name);
    assert!(is_creature(&t, in_graveyard));
    let god = t.hand(P0, name);
    assert!(is_creature(&t, god));
    give_mana_for(&mut t, P0, name);
    let spell = t.cast(P0, god).go();
    assert!(t.g.obj(spell).is(CardType::Creature));
    // "Counter target creature spell" can target it.
    assert!(spell_targets(&mut t, P1, "Essence Scatter").contains(&Entity::Object(spell)));
    t.resolve();
    assert!(t.on_battlefield(god));
    assert!(!is_creature(&t, god));
}

#[test]
fn a_god_is_always_a_creature_card_outside_the_battlefield() {
    cr!("700.5", "611.3b", "613.1d");
    ruling!(
        "Erebos, God of the Dead",
        "The type-changing ability that can make a God not be a creature functions only on the battlefield. It's always a creature card in other zones, regardless of your devotion to its color. It's always a creature spell while it's on the stack."
    );
    god_is_a_creature_outside_the_battlefield("Erebos, God of the Dead");
}

#[test]
fn a_god_is_always_a_creature_card_outside_the_battlefield_curly() {
    cr!("700.5", "611.3b", "613.1d");
    ruling!(
        "Purphoros, Bronze-Blooded",
        "The type-changing ability that can make a God not be a creature functions only on the battlefield. It’s always a creature card in other zones, regardless of your devotion to its color. It’s always a creature spell while it’s on the stack."
    );
    god_is_a_creature_outside_the_battlefield("Purphoros, Bronze-Blooded");
}

/// An attacking or blocking God that stops being a creature is removed from combat and
/// doesn't rejoin it when it becomes a creature again.
fn god_removed_from_combat(name: &str, devotion: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    let god = t.battlefield(P0, name);
    let idols = idol(&mut t, P0, devotion);
    assert!(is_creature(&t, god));
    attack_with(&mut t, &[(god, Entity::Player(P1))]);
    assert!(attacking(&t, god));
    move_to(&mut t, idols, Zone::Exile);
    assert!(!is_creature(&t, god));
    assert!(!attacking(&t, god));
    idol(&mut t, P0, devotion);
    t.settle();
    assert!(is_creature(&t, god));
    assert!(!attacking(&t, god));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 20);
    // A blocking God too: the attacker it blocked stays blocked and deals no damage.
    let mut t = TestGame::new(2);
    let god = t.battlefield(P0, name);
    let idols = idol(&mut t, P0, devotion);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    to_blockers(&mut t, &[(bears, Entity::Player(P0))], &[(god, bears)]);
    assert!(blocking(&t, god));
    move_to(&mut t, idols, Zone::Exile);
    assert!(!blocking(&t, god));
    idol(&mut t, P0, devotion);
    t.settle();
    assert!(is_creature(&t, god));
    assert!(!blocking(&t, god));
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.obj_now(god).damage, 0);
}

#[test]
fn a_god_that_stops_being_a_creature_is_removed_from_combat() {
    cr!("506.4", "509.1h", "700.5");
    ruling!(
        "Erebos, God of the Dead",
        "If a God is attacking or blocking and it stops being a creature, it will be removed from combat. It won't rejoin combat if it resumes being a creature later during that combat."
    );
    god_removed_from_combat("Erebos, God of the Dead", "{B}{B}{B}{B}");
}

#[test]
fn a_god_that_stops_being_a_creature_is_removed_from_combat_curly() {
    cr!("506.4", "509.1h", "700.5");
    ruling!(
        "Thassa, Deep-Dwelling",
        "If a God is attacking or blocking and it stops being a creature, it will be removed from combat. It won’t rejoin combat if it resumes being a creature later during that combat."
    );
    god_removed_from_combat("Thassa, Deep-Dwelling", "{U}{U}{U}{U}");
}

/// Counters stay on a God while it isn't a creature, and count again once it is.
fn god_keeps_counters(name: &str, devotion: &str, pt: (i32, i32)) {
    supported(name);
    let mut t = TestGame::new(2);
    let god = t.battlefield(P0, name);
    let idols = idol(&mut t, P0, devotion);
    t.g.add_counters(Entity::Object(god), counters::PLUS1, 2, None);
    t.settle();
    assert_eq!(t.pt(god), (pt.0 + 2, pt.1 + 2));
    move_to(&mut t, idols, Zone::Exile);
    assert!(!is_creature(&t, god));
    assert_eq!(t.counters(god, counters::PLUS1), 2);
    // The counters can still be added to it.
    t.g.add_counters(Entity::Object(god), counters::PLUS1, 1, None);
    t.settle();
    assert_eq!(t.counters(god, counters::PLUS1), 3);
    idol(&mut t, P0, devotion);
    t.settle();
    assert!(is_creature(&t, god));
    assert_eq!(t.pt(god), (pt.0 + 3, pt.1 + 3));
}

#[test]
fn counters_stay_on_a_god_that_isnt_a_creature() {
    cr!("122.1", "700.5");
    ruling!(
        "Erebos, God of the Dead",
        "Counters put on a God remain on it while it's not a creature, even if they have no effect."
    );
    god_keeps_counters("Erebos, God of the Dead", "{B}{B}{B}{B}", (5, 7));
}

#[test]
fn counters_stay_on_a_god_that_isnt_a_creature_curly() {
    cr!("122.1", "700.5");
    ruling!(
        "Erebos, Bleak-Hearted",
        "Counters put on a God remain on it while it’s not a creature, even if they have no effect."
    );
    god_keeps_counters("Erebos, Bleak-Hearted", "{B}{B}{B}{B}", (5, 6));
}

#[test]
fn a_gods_abilities_function_while_it_isnt_a_creature() {
    cr!("700.5", "611.3a");
    ruling!(
        "Erebos, God of the Dead",
        "The abilities of Gods function as long as they're on the battlefield, regardless of whether they're creatures."
    );
    supported("Erebos, God of the Dead");
    // Erebos (devotion one, not a creature): "Your opponents can't gain life."
    let mut t = TestGame::new(2);
    let erebos = t.battlefield(P0, "Erebos, God of the Dead");
    assert!(!is_creature(&t, erebos));
    run_from(
        &mut t,
        P1,
        None,
        Effect::GainLife {
            who: PlayerRef::You,
            n: Value::c(5),
        },
        &[],
    );
    assert_eq!(t.life(P1), 20);
    // Its activated ability works too: {1}{B}, Pay 2 life: Draw a card.
    t.lands(P0, "Swamp", 2);
    let hand = t.hand_size(P0);
    t.activate(P0, erebos, 0, &[]).expect("activate Erebos");
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn a_gods_abilities_function_while_it_isnt_a_creature_curly() {
    cr!("700.5", "611.3a");
    ruling!(
        "Purphoros, Bronze-Blooded",
        "The abilities of Gods function as long as they’re on the battlefield, regardless of whether they’re creatures."
    );
    supported("Purphoros, Bronze-Blooded");
    // Purphoros (not a creature): "Other creatures you control have haste."
    let mut t = TestGame::new(2);
    let purphoros = t.battlefield(P0, "Purphoros, Bronze-Blooded");
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    assert!(!is_creature(&t, purphoros));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Haste));
}

#[test]
fn devotion_to_two_colors_counts_a_hybrid_symbol_once() {
    cr!("700.5");
    ruling!(
        "Iroas, God of Victory",
        "If an effect counts your devotion to two colors, a hybrid symbol that is both of those colors is counted just once."
    );
    supported("Iroas, God of Victory");
    supported("Boros Reckoner");
    // Iroas ({2}{R}{W}) and Boros Reckoner ({R/W}{R/W}{R/W}): devotion to red and white is
    // 2 + 3 = 5, not 8. Less than seven: not a creature.
    let mut t = TestGame::new(2);
    let iroas = t.battlefield(P0, "Iroas, God of Victory");
    t.battlefield(P0, "Boros Reckoner");
    assert!(!is_creature(&t, iroas));
    // A symbol that's only red or only white counts too: {R}{W} makes seven.
    idol(&mut t, P0, "{R}{W}");
    assert!(is_creature(&t, iroas));
    assert_eq!(t.pt(iroas), (7, 4));
}
