//! Rulings batch P018 — color and type changes: an effect that sets colors overwrites
//! earlier ones (CR 105.3, 613.1e, 613.7), setting land types (CR 305.7), "in addition to"
//! (CR 205.1b), effects that apply as a permanent enters (CR 613.1, 611.3c), colorless
//! isn't a card type (CR 105.2c), "the color or colors of your choice" (CR 105.1),
//! phasing doesn't make a permanent enter again (CR 702.26d), a target-zero spell
//! (CR 115.1, 601.2c), and the timestamp of a conditional static ability (CR 613.7a).

use crate::r_p018_common::*;
use crate::r_s01_common::{asked_since, supported};
use crate::r_s06_common::attach_new;
use crate::r_s20_common::tap_for_mana;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn cast_resolve(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) {
    plenty_of_mana(t, p);
    let card = t.hand(p, name);
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    t.cast(p, card).go();
    empty_pool(t, p);
    t.resolve_all();
}

#[test]
fn nightcreep_makes_creatures_black_only() {
    cr!("105.3", "613.1e");
    ruling!(
        "Nightcreep",
        "This effect overwrites all creatures' colors. Creatures will be black instead of any other color or colors."
    );
    supported("Nightcreep");
    let mut t = TestGame::new(2);
    let lions = t.battlefield(P1, "Savannah Lions");
    let ogre = t.battlefield(P0, "Rakdos Cackler");
    cast_resolve(&mut t, P0, "Nightcreep", &[]);
    for c in [lions, ogre] {
        assert_eq!(colors_now(&mut t, c), ColorSet::single(Color::Black));
    }
}

#[test]
fn nightcreep_makes_lands_swamps_keeping_supertypes() {
    cr!("305.7", "205.4a");
    ruling!(
        "Nightcreep",
        "This effect overwrites all lands' land types. Lands will be Swamps instead of any other land types. This won't affect a land's supertypes (such as basic or legendary). Lands will lose their normal abilities and have the ability \"{T}: Add {B}\" instead."
    );
    supported("Pendelhaven");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let falls = t.battlefield(P0, "Lumbering Falls");
    let haven = t.battlefield(P0, "Pendelhaven");
    cast_resolve(&mut t, P0, "Nightcreep", &[]);
    t.g.recompute();
    for land in [forest, falls, haven] {
        let o = t.obj_now(land);
        assert!(o.chars.has_subtype("Swamp"));
        assert!(!o.chars.has_subtype("Forest") && !o.chars.has_subtype("Island"));
        // Only "{T}: Add {B}".
        assert_eq!(o.chars.abilities.len(), 1, "{:?}", o.chars.name);
        assert!(o.chars.abilities[0].text.contains("{B}"));
    }
    assert!(t
        .obj_now(forest)
        .chars
        .supertypes
        .contains(Supertype::Basic));
    assert!(t
        .obj_now(haven)
        .chars
        .supertypes
        .contains(Supertype::Legendary));
    assert!(tap_for_mana(&mut t, P0, falls, "{B}"));
    assert_eq!(t.g.players[P0.idx()].mana_pool.count(ManaType::B), 1);
}

#[test]
fn dralnus_crusade_goblins_enter_as_black_zombies() {
    cr!("613.1", "611.3c", "205.1b", "105.3");
    ruling!(
        "Dralnu's Crusade",
        "Under Dralnu's Crusade, Goblins are black only, not black in addition to any other colors. They are, however, Zombies in addition to their other creature types. As one enters the battlefield, it is already a black Goblin Zombie (and perhaps some other creature types) with +1/+1; it doesn't enter and then become a black Goblin Zombie and get +1/+1."
    );
    supported("Dralnu's Crusade");
    supported("Noxious Ghoul");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dralnu's Crusade");
    // Noxious Ghoul: "Whenever this creature or another Zombie enters, all non-Zombie
    // creatures get -1/-1 until end of turn."
    t.battlefield(P1, "Noxious Ghoul");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let goblin = t.enter(P0, "Raging Goblin");
    t.settle();
    assert_eq!(t.stack_len(), 1, "the Goblin entered as a Zombie");
    t.resolve_all();
    let o = t.obj_now(goblin);
    assert_eq!(o.chars.colors, ColorSet::single(Color::Black));
    assert!(
        o.chars.has_subtype("Goblin")
            && o.chars.has_subtype("Zombie")
            && o.chars.has_subtype("Berserker")
    );
    // 1/1 +1/+1; a Zombie, so Noxious Ghoul's -1/-1 skips it.
    assert_eq!(t.pt(goblin), (2, 2));
    assert_eq!(t.pt(bears), (1, 1));
}

#[test]
fn sway_of_illusion_can_target_zero_creatures() {
    cr!("115.1", "601.2c");
    ruling!(
        "Sway of Illusion",
        "You can cast this and choose to target zero creatures and effectively just draw a card."
    );
    supported("Sway of Illusion");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.answer_targets(P0, &[]);
    cast_resolve(&mut t, P0, "Sway of Illusion", &[]);
    assert!(t.in_graveyard(P0, "Sway of Illusion"));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn scuttlemutt_chooses_one_or_more_of_the_five_colors() {
    cr!("105.1", "105.2c");
    ruling!(
        "Scuttlemutt",
        "You must choose one or more of the five colors of Magic while resolving Scuttlemutt’s last ability. You can’t choose “artifact,” “colorless,” or “chartreuse.”"
    );
    supported("Scuttlemutt");
    let mut t = TestGame::new(2);
    let mutt = t.battlefield(P0, "Scuttlemutt");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    // Choices are offered as the ability resolves: the sets of one or more colors.
    let ability = t.activate(P0, mutt, 1, &[Entity::Object(bears)]).unwrap();
    assert!(ability.is_some());
    assert!(asked_since(&t, from)
        .iter()
        .all(|(_, d)| !matches!(d, Decision::ChooseOption { .. })));
    let from = t.asked().len();
    // White and blue (the third combination).
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    t.resolve();
    let options: Vec<Vec<String>> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options),
            _ => None,
        })
        .collect();
    assert_eq!(options.len(), 1);
    assert_eq!(options[0].len(), 31);
    for o in &options[0] {
        assert!(!o.is_empty());
        assert!(o
            .split(" and ")
            .all(|w| ["white", "blue", "black", "red", "green"].contains(&w)));
    }
    let mut wu = ColorSet::single(Color::White);
    wu.insert(Color::Blue);
    assert_eq!(colors_now(&mut t, bears), wu);
}

#[test]
fn scrapbaskets_colors_are_overwritten_by_a_later_color_change() {
    cr!("613.1e", "613.7a");
    ruling!(
        "Scrapbasket",
        "After Scrapbasket's ability resolves, an effect that changes its colors will overwrite that ability's effect. For example, casting Cerulean Wisps on it will turn it just blue."
    );
    ruling!(
        "Scrapbasket",
        "Scrapbasket's ability won't stop it from being an artifact. It'll just be a colorful artifact."
    );
    supported("Scrapbasket");
    supported("Cerulean Wisps");
    let mut t = TestGame::new(2);
    let basket = t.battlefield(P0, "Scrapbasket");
    activate_and_resolve(&mut t, P0, basket, "becomes");
    assert_eq!(colors_now(&mut t, basket), ColorSet::ALL);
    assert!(t.obj_now(basket).is(CardType::Artifact));
    assert!(t.obj_now(basket).is(CardType::Creature));
    cast_resolve(&mut t, P0, "Cerulean Wisps", &[Entity::Object(basket)]);
    assert_eq!(colors_now(&mut t, basket), ColorSet::single(Color::Blue));
    // Activating it again afterwards makes it all colors again (a later timestamp).
    activate_and_resolve(&mut t, P0, basket, "becomes");
    assert_eq!(colors_now(&mut t, basket), ColorSet::ALL);
}

#[test]
fn ancient_kavu_becoming_colorless_doesnt_make_it_an_artifact() {
    cr!("105.2c");
    ruling!(
        "Ancient Kavu",
        "Making a permanent colorless does not make it into an artifact."
    );
    supported("Ancient Kavu");
    let mut t = TestGame::new(2);
    let kavu = t.battlefield(P0, "Ancient Kavu");
    activate_and_resolve(&mut t, P0, kavu, "colorless");
    assert!(colors_now(&mut t, kavu).is_colorless());
    let o = t.obj_now(kavu);
    assert!(!o.is(CardType::Artifact));
    assert!(o.is(CardType::Creature));
}

#[test]
fn alloy_golem_phasing_in_doesnt_choose_a_new_color() {
    cr!("702.26d");
    ruling!(
        "Alloy Golem",
        "If Alloy Golem phases out, you do not pick a new color when it phases in."
    );
    supported("Alloy Golem");
    let mut t = TestGame::new(2);
    // Green (the fifth color).
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    let golem = t.enter(P0, "Alloy Golem");
    t.settle();
    assert_eq!(colors_now(&mut t, golem), ColorSet::single(Color::Green));
    mtg_engine::keyword_impls::phase_out(&mut t.g, vec![golem]);
    t.g.recompute();
    let from = t.asked().len();
    mtg_engine::keyword_impls::phase_in(&mut t.g, golem);
    t.g.recompute();
    t.settle();
    assert!(asked_since(&t, from)
        .iter()
        .all(|(_, d)| !matches!(d, Decision::ChooseOption { .. })));
    assert!(t.on_battlefield(golem));
    assert!(!t.obj_now(golem).phased_out);
    assert_eq!(colors_now(&mut t, golem), ColorSet::single(Color::Green));
}

#[test]
fn tahngarths_rage_keeps_its_timestamp_as_attacking_changes() {
    cr!("613.7a", "611.3a");
    ruling!(
        "Tahngarth's Rage",
        "The effect does not receive a new timestamp when the creature goes from attacking to not attacking or the other way around."
    );
    supported("Tahngarth's Rage");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rage = attach_new(&mut t, P0, "Tahngarth's Rage", bears);
    let stamp = t.obj_now(rage).timestamp;
    assert_eq!(t.pt(bears), (0, 1));
    crate::r_s01_common::attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.g.recompute();
    assert_eq!(t.pt(bears), (5, 2));
    assert_eq!(t.obj_now(rage).timestamp, stamp);
    t.advance_to(P0, mtg_engine::turn::Step::PostcombatMain);
    assert_eq!(t.life(P1), 15);
    t.g.recompute();
    assert_eq!(t.pt(bears), (0, 1));
    assert_eq!(t.obj_now(rage).timestamp, stamp);
}
