//! Rulings batch P016 — permanents that enter as a copy (Sculpting Steel, Clever
//! Impersonator, Estrid's Invocation, Copy Enchantment, Mirrormade, Moritte of the Frost:
//! CR 707.2, 614.12), and assorted rulings on protection from each color, chosen colors,
//! triggered abilities and abilities targeting their own source.

use crate::r_p016_common::*;
use crate::r_p017_common::enter_copying;
use crate::r_s01_common::supported;
use crate::r_s24_common::choose_creature_type;
use crate::r_s26_common::{dress_up, fresh};
use mtg_engine::card::card;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------
// Enter as a copy: enters abilities and "as enters" / "enters with" abilities work
// ---------------------------------------------------------------------------

/// P0 casts `name` paying with exactly `mana`, choosing to copy `what`; returns the
/// permanent.
fn cast_copying(t: &mut TestGame, name: &str, mana: &[ManaType], what: ObjectId) -> ObjectId {
    supported(name);
    t.set_step(P0, Step::PrecombatMain);
    for m in mana {
        t.g.players[0].mana_pool.add_type(*m, 1);
    }
    let c = t.hand(P0, name);
    t.answer_choose(P0, &[o(what)]);
    t.cast(P0, c).go();
    t.resolve_all();
    t.g.current(c)
}

#[test]
fn sculpting_steel_copying_a_sunburst_artifact_counts_its_own_colors() {
    cr!("707.2", "702.44a", "614.12");
    ruling!(
        "Sculpting Steel",
        "Any \"enters\" abilities of the copied artifact will trigger when Sculpting Steel enters. Any \"as enters\" or \"enters with\" abilities of the chosen artifact will also work."
    );
    // Pentad Prism: "Sunburst" (charge counters, it's not a creature).
    let mut t = TestGame::new(2);
    let prism = t.battlefield(P1, "Pentad Prism");
    let s = cast_copying(
        &mut t,
        "Sculpting Steel",
        &[ManaType::W, ManaType::U, ManaType::B],
        prism,
    );
    assert_eq!(t.obj_now(s).chars.name, "Pentad Prism");
    assert_eq!(t.counters(s, counters::CHARGE), 3);
    // Ichor Wellspring: "When this artifact enters or is put into a graveyard from the
    // battlefield, draw a card."
    let mut t = TestGame::new(2);
    let well = t.battlefield(P1, "Ichor Wellspring");
    let hand = t.hand_size(P0);
    cast_copying(&mut t, "Sculpting Steel", &[ManaType::C; 3], well);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn mirrormade_copying_a_sunburst_artifact_counts_its_own_colors() {
    cr!("707.2", "702.44a", "614.12");
    ruling!(
        "Mirrormade",
        "Any enters-the-battlefield abilities of the copied permanent will trigger when Mirrormade enters the battlefield. Any \"as [this permanent] enters the battlefield\" or \"[this permanent] enters the battlefield with\" abilities of the chosen permanent will also work."
    );
    let mut t = TestGame::new(2);
    let prism = t.battlefield(P1, "Pentad Prism");
    let m = cast_copying(
        &mut t,
        "Mirrormade",
        &[ManaType::U, ManaType::U, ManaType::R],
        prism,
    );
    assert_eq!(t.counters(m, counters::CHARGE), 2);
    // An enters ability: Oblivion Ring's copy exiles a target.
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P1, "Oblivion Ring");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[o(bears)]);
    cast_copying(&mut t, "Mirrormade", &[ManaType::U; 3], ring);
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn clever_impersonator_gets_enters_abilities_and_enters_with_counters() {
    cr!("707.2", "614.12", "614.1c");
    ruling!(
        "Clever Impersonator",
        "Any enters-the-battlefield abilities of the chosen permanent will trigger when Clever Impersonator enters the battlefield. Any \"as [this permanent] enters the battlefield\" or \"[this permanent] enters the battlefield with\" abilities of the chosen permanent will also work."
    );
    supported("Clever Impersonator");
    // Wall of Omens: "When this creature enters, draw a card."
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P1, "Wall of Omens");
    let hand = t.hand_size(P0);
    enter_copying(&mut t, P0, "Clever Impersonator", Some(wall));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Kalonian Hydra: "enters with four +1/+1 counters on it."
    let mut t = TestGame::new(2);
    let hydra = t.battlefield(P1, "Kalonian Hydra");
    let c = enter_copying(&mut t, P0, "Clever Impersonator", Some(hydra));
    assert_eq!(t.counters(c, counters::PLUS1), 4);
    assert_eq!(t.pt(c), (4, 4));
}

/// `name` copies Oblivion Ring (its enters ability exiles a target) and Shared Triumph
/// (its "as enters" choice of a creature type is made for the copy).
fn enchantment_copy_works_like_the_original(name: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Oblivion Ring");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[o(bears)]);
    enter_copying(&mut t, P0, name, Some(ring));
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"), "{name}");
    // "As Shared Triumph enters, choose a creature type. Creatures of the chosen type get
    // +1/+1."
    let mut t = TestGame::new(2);
    let triumph = t.battlefield(P0, "Shared Triumph");
    let bears = t.battlefield(P0, "Grizzly Bears");
    choose_creature_type(&mut t, P0, "Bear");
    let c = enter_copying(&mut t, P0, name, Some(triumph));
    assert_eq!(t.obj_now(c).chars.name, "Shared Triumph", "{name}");
    assert_eq!(t.pt(bears), (3, 3), "{name}");
}

#[test]
fn copy_enchantment_and_estrids_invocation_get_enters_and_as_enters_abilities() {
    cr!("707.2", "614.12", "603.6a");
    ruling!(
        "Copy Enchantment",
        "Any enters-the-battlefield abilities of the copied permanent will trigger when Copy Enchantment enters the battlefield. Any \"as [this enchantment] enters the battlefield\" or \"[this enchantment] enters the battlefield with\" abilities of the chosen permanent will also work."
    );
    ruling!(
        "Estrid's Invocation",
        "Any enters-the-battlefield abilities of the copied enchantment will trigger when Estrid's Invocation enters the battlefield. Any \"as [this enchantment] enters the battlefield\" or \"[this enchantment] enters the battlefield with\" abilities of the chosen enchantment will also work."
    );
    enchantment_copy_works_like_the_original("Copy Enchantment");
    enchantment_copy_works_like_the_original("Estrid's Invocation");
}

#[test]
fn moritte_gets_the_enters_abilities_of_what_it_copies() {
    cr!("707.2", "707.9b", "614.12");
    ruling!(
        "Moritte of the Frost",
        "Any enters-the-battlefield abilities of the copied permanent will trigger when Moritte enters the battlefield. Any \"as [this permanent] enters the battlefield\" or \"[this permanent] enters the battlefield with\" abilities of the chosen permanent will also work."
    );
    supported("Moritte of the Frost");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Omens");
    let hand = t.hand_size(P0);
    let m = enter_copying(&mut t, P0, "Moritte of the Frost", Some(wall));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.counters(m, counters::PLUS1), 2);
    // An "enters with" ability of the copied permanent applies too.
    let mut t = TestGame::new(2);
    let hydra = t.battlefield(P0, "Kalonian Hydra");
    let m = enter_copying(&mut t, P0, "Moritte of the Frost", Some(hydra));
    assert_eq!(t.counters(m, counters::PLUS1), 6);
}

/// `name` copies `what` after it was tapped, given counters, enchanted by an Aura and
/// changed by non-copy effects: it gets exactly what's printed.
fn copies_only_whats_printed(name: &str, what: &str, aura: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    let orig = t.battlefield(P1, what);
    dress_up(&mut t, orig);
    let a = t.battlefield(P1, aura);
    t.g.attach(a, Entity::Object(orig));
    t.settle();
    assert!(!t.g.attachments_of(Entity::Object(orig)).is_empty());
    let c = enter_copying(&mut t, P0, name, Some(orig));
    assert_eq!(t.obj_now(c).chars.name, what, "{name}");
    assert!(fresh(&t, c), "{name}");
    let printed = card(what).front().chars.colors;
    assert_eq!(t.obj_now(c).chars.colors, printed, "{name}");
    if let (Some(p), Some(tg)) = (
        card(what).front().chars.power,
        card(what).front().chars.toughness,
    ) {
        assert_eq!(t.pt(c), (p, tg), "{name}");
    }
}

#[test]
fn enter_as_copy_cards_copy_only_whats_printed() {
    cr!("707.2", "613.2");
    ruling!(
        "Clever Impersonator",
        "Clever Impersonator copies exactly what is printed on the chosen permanent (unless that permanent is copying something else or is a token; see below). It doesn't copy whether that permanent is tapped or untapped"
    );
    ruling!(
        "Copy Enchantment",
        "Copy Enchantment copies exactly what was printed on the original permanent and nothing more (unless that permanent is itself copying something or is a token; see below). It doesn't copy whether that permanent is tapped or untapped"
    );
    ruling!(
        "Estrid's Invocation",
        "Estrid's Invocation copies exactly what was printed on the original enchantment (unless that enchantment is copying something else or is a token; see below). It doesn't copy whether that enchantment is tapped or untapped"
    );
    copies_only_whats_printed("Clever Impersonator", "Grizzly Bears", "Rancor");
    // An enchantment creature enchanted by an Aura.
    copies_only_whats_printed("Copy Enchantment", "Enduring Courage", "Rancor");
    let mut t = TestGame::new(2);
    let orig = t.battlefield(P0, "Glorious Anthem");
    dress_up(&mut t, orig);
    let c = enter_copying(&mut t, P0, "Estrid's Invocation", Some(orig));
    assert!(fresh(&t, c));
    assert_eq!(t.obj_now(c).chars.colors, ColorSet::single(Color::White));
}

// ---------------------------------------------------------------------------
// Assorted
// ---------------------------------------------------------------------------

#[test]
fn etched_champion_has_protection_from_each_color() {
    cr!("702.16a", "702.16b");
    ruling!(
        "Etched Champion",
        "\"Protection from all colors\" means protection from white, from blue, from black, from red, and from green."
    );
    supported("Etched Champion");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let champ = t.battlefield(P0, "Etched Champion");
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Monocolored red and black spells can't target it.
    for name in ["Lightning Bolt", "Doom Blade"] {
        lots(&mut t, P1);
        let s = t.hand(P1, name);
        let from = t.asked().len();
        t.answer_targets(P1, &[o(bears)]);
        t.cast(P1, s).go();
        let offered = &target_choices(&t, P1, from, "")[0];
        assert!(!offered.contains(&o(champ)), "{name}");
        assert!(offered.contains(&o(bears)), "{name}");
    }
}

/// Puts Diamond Knight onto the battlefield for P0 choosing `color`.
fn diamond_knight(t: &mut TestGame, color: Color) -> ObjectId {
    supported("Diamond Knight");
    let i = Color::ALL.iter().position(|c| *c == color).unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
    let k = t.enter(P0, "Diamond Knight");
    t.settle();
    k
}

#[test]
fn diamond_knight_chooses_one_of_the_five_colors() {
    cr!("105.1", "614.12");
    ruling!(
        "Diamond Knight",
        "You must choose one of the five colors of Magic as Diamond Knight asks you to choose a color. You can’t choose “artifact,” “colorless,” or “chartreuse.”"
    );
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    diamond_knight(&mut t, Color::Red);
    let options: Vec<Vec<String>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(options.len(), 1);
    assert_eq!(options[0].len(), 5);
    assert!(options[0]
        .iter()
        .all(|o| Color::from_word(&o.to_lowercase()).is_some()));
}

#[test]
fn diamond_knights_trigger_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "601.2i");
    ruling!(
        "Diamond Knight",
        "Diamond Knight’s ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let k = diamond_knight(&mut t, Color::Red);
    let bolt = cast_at(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // The trigger is above the Bolt; the Bolt is countered in response.
    cast_at(&mut t, P1, "Counterspell", &[o(bolt)]);
    t.resolve(); // Counterspell
    assert!(!t.g.stack.contains(&bolt));
    t.resolve_all();
    assert_eq!(t.counters(k, counters::PLUS1), 1);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn diamond_knight_with_no_color_chosen_never_triggers() {
    cr!("607.2a");
    ruling!(
        "Diamond Knight",
        "If you somehow control a Diamond Knight for which no color was chosen, its last ability will never trigger."
    );
    // Put onto the battlefield without entering (no choice made).
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let k = t.battlefield(P0, "Diamond Knight");
    for name in ["Lightning Bolt", "Giant Growth"] {
        let target = if name == "Giant Growth" {
            o(k)
        } else {
            Entity::Player(P1)
        };
        cast_at(&mut t, P0, name, &[target]);
        t.settle();
        assert_eq!(t.stack_len(), 1, "{name}");
        t.resolve_all();
    }
    assert_eq!(t.counters(k, counters::PLUS1), 0);
}

#[test]
fn consign_to_memory_counters_a_keyword_triggered_ability() {
    cr!("603.1", "702.108a");
    ruling!(
        "Consign to Memory",
        "Triggered abilities use the word \"when,\" \"whenever,\" or \"at.\" They're often written as \"[Trigger condition], [effect].\" Some keywords (such as prowess) are triggered abilities"
    );
    supported("Consign to Memory");
    // Prowess triggers; Consign to Memory counters the trigger.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let monk = t.battlefield(P1, "Monastery Swiftspear");
    cast_at(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    t.settle();
    let trig = *t.g.stack.last().unwrap();
    assert!(t.obj(trig).stack.is_some());
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    cast_at(&mut t, P0, "Consign to Memory", &[o(trig)]);
    t.resolve_all();
    assert_eq!(t.pt(monk), (1, 2));
    assert_eq!(t.life(P0), 18);
}

#[test]
fn sylvok_replica_targeting_itself_does_nothing() {
    cr!("608.2b", "601.2c");
    ruling!(
        "Sylvok Replica",
        "You may target Sylvok Replica with its own ability. However, if you do, the ability won’t resolve for having an illegal target."
    );
    supported("Sylvok Replica");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let r = t.battlefield(P0, "Sylvok Replica");
    let ring = t.battlefield(P0, "Sol Ring");
    lots(&mut t, P0);
    let ab = t.activate(P0, r, 0, &[o(r)]).unwrap().unwrap();
    assert_eq!(stack_targets(&t, ab), vec![o(r)]);
    assert!(t.in_graveyard(P0, "Sylvok Replica"));
    t.resolve_all();
    assert!(t.on_battlefield(ring));
    assert!(t.in_graveyard(P0, "Sylvok Replica"));
}

#[test]
fn gogo_copy_of_a_linked_ability_is_linked_too() {
    cr!("607.2a", "707.10", "702.75a");
    ruling!(
        "Gogo, Master of Mimicry",
        "If an ability is linked to a second ability, copies of that first ability are also linked to that second ability. If the second ability refers to \"the exiled card,\" it refers to all cards exiled by the first ability and the copy."
    );
    supported("Gogo, Master of Mimicry");
    supported("Clive's Hideaway");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let gogo = t.battlefield(P0, "Gogo, Master of Mimicry");
    for l in [
        "Thalia, Guardian of Thraben",
        "Isamaru, Hound of Konda",
        "Ragavan, Nimble Pilferer",
    ] {
        t.battlefield(P0, l);
    }
    // Eight cards on top: each hideaway looks at four and exiles one.
    let cards = crate::r_s01_common::stack_library(
        &mut t,
        P0,
        &[
            "Grizzly Bears",
            "Island",
            "Island",
            "Island",
            "Hill Giant",
            "Island",
            "Island",
            "Island",
        ],
    );
    let hideaway = crate::r_s05_common::enter(&mut t, P0, "Clive's Hideaway");
    let trig = *t.g.stack.last().expect("hideaway trigger");
    lots(&mut t, P0);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.activate(P0, gogo, 0, &[o(trig)]).unwrap();
    t.resolve_all();
    let exiled: Vec<ObjectId> = cards
        .iter()
        .map(|c| t.g.current(*c))
        .filter(|c| t.zone(*c) == mtg_engine::object::Zone::Exile)
        .collect();
    assert_eq!(exiled.len(), 2);
    lots(&mut t, P0);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    crate::r_s06_common::activate_containing(&mut t, P0, hideaway, "{2}").unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}
