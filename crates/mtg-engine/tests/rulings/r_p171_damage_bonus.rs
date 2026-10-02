//! Rulings batch P171 — replacement effects that increase damage ("it deals that much
//! damage plus N instead", "double", "triple"): Torbran, Thane of Red Fell; Jaya,
//! Venerated Firemage; The Flame of Keld; Mechanized Warfare; Embermaw Hellion; Sulfuric
//! Vapors; Fire Servant; City on Fire; and Ghostly Flame's colorless damage.

use crate::r_p171_common::*;
use crate::r_s03_common::respond;
use crate::r_s05_common::move_to;
use crate::r_s19_common::add_lore;
use crate::r_s29_common::replacement_choosers;
use crate::r_s30_common::pick_replacement;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The damage events this turn: (source, recipient, amount).
fn damage_events(t: &TestGame) -> Vec<(ObjectId, Entity, u32)> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Damage {
                source,
                target,
                amount,
                ..
            } => Some((*source, *target, *amount)),
            _ => None,
        })
        .collect()
}

/// P1 casts Remedy, preventing the next 5 damage that would be dealt to P1 this turn.
fn remedy_on_p1(t: &mut TestGame) {
    give_mana_for(t, P1, "Remedy");
    let card = t.hand(P1, "Remedy");
    t.answer_targets(P1, &[Entity::Player(P1)]);
    t.answer(P1, DecisionKind::Divide, Answer::Numbers(vec![5]));
    t.cast_with(P1, card, &[]).unwrap();
    t.resolve_all();
}

fn remedy_first(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    pick_replacement(g, d, "Remedy")
}

fn bonus_first(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    match d {
        Decision::ChooseReplacement { options } => options
            .iter()
            .position(|o| !o.contains("Remedy"))
            .map(Answer::Index),
        _ => None,
    }
}

/// P0 controls `name`; P1 prevents the next 5 damage to P1 (Remedy); P0 casts Lava Axe
/// (5 damage to target player) at P1, P1 ordering the effects with `choice`. Returns
/// P1's life and the players asked to order the effects.
fn lava_axe_through_remedy(
    name: &str,
    choice: fn(&mtg_engine::game::Game, &Decision) -> Option<Answer>,
) -> (i32, Vec<PlayerId>) {
    let mut t = TestGame::new(2);
    t.battlefield(P0, name);
    remedy_on_p1(&mut t);
    respond(&mut t, P1, choice);
    let from = t.asked().len();
    cast_targeting(&mut t, P0, "Lava Axe", &[Entity::Player(P1)]);
    t.resolve_all();
    (t.life(P1), replacement_choosers(&t, from))
}

#[test]
fn the_player_dealt_damage_orders_prevention_and_the_bonus() {
    cr!("616.1", "615.1a", "614.1a");
    ruling!(
        "Torbran, Thane of Red Fell",
        "If another effect modifies how much damage your red source would deal, including preventing some of it, the player being dealt damage or the controller of the permanent being dealt damage chooses an order in which to apply those effects. If all of the damage is prevented, Torbran's effect no longer applies."
    );
    ruling!(
        "Jaya, Venerated Firemage",
        "If another effect modifies how much damage your red source would deal, including preventing some of it, the player being dealt damage or the controller of the permanent being dealt damage chooses an order in which to apply those effects. If all of the damage is prevented, Jaya's effect no longer applies."
    );
    ruling!(
        "Mechanized Warfare",
        "If another effect modifies how much damage a source would deal to an opponent or a permanent they control, including preventing some of it, the player being dealt damage or the controller of the permanent being dealt damage chooses an order in which to apply those effects. If all of the damage is prevented, Mechanized Warfare's effect no longer applies."
    );
    ruling!(
        "Embermaw Hellion",
        "If the player or permanent being dealt damage is also affected by a damage prevention effect, that player or the controller of that permanent can apply that effect and Embermaw Hellion’s effect in any order. If all of the damage is prevented, Embermaw Hellion’s effect can’t apply to it."
    );
    supported("Remedy");
    supported("Lava Axe");
    for (name, bonus) in [
        ("Torbran, Thane of Red Fell", 2),
        ("Jaya, Venerated Firemage", 1),
        ("Mechanized Warfare", 1),
        ("Embermaw Hellion", 1),
    ] {
        supported(name);
        // Prevention first: all 5 damage is prevented, and the bonus no longer applies.
        let (life, asked) = lava_axe_through_remedy(name, remedy_first);
        assert_eq!(asked, vec![P1], "{name}");
        assert_eq!(life, 20, "{name}");
        // The bonus first: 5 + N, then 5 prevented.
        let (life, asked) = lava_axe_through_remedy(name, bonus_first);
        assert_eq!(asked, vec![P1], "{name}");
        assert_eq!(life, 20 - bonus, "{name}");
    }
}

#[test]
fn the_extra_damage_is_dealt_by_the_original_source() {
    cr!("614.1a", "614.2", "120.1");
    ruling!(
        "Torbran, Thane of Red Fell",
        "The additional 2 damage is dealt by the same source as the original source of damage. The damage isn't dealt by Torbran unless Torbran is the original source of damage."
    );
    ruling!(
        "Mechanized Warfare",
        "The additional 1 damage is dealt by the original source of damage. The additional damage isn't dealt by Mechanized Warfare."
    );
    ruling!(
        "Jaya, Venerated Firemage",
        "Jaya's first ability doesn't cause Jaya to deal damage; it affects the amount of damage dealt by the original red source."
    );
    ruling!(
        "Embermaw Hellion",
        "Embermaw Hellion’s last ability doesn’t cause Embermaw Hellion to deal damage; it affects the amount of damage dealt by the original red source."
    );
    for (name, bonus) in [
        ("Torbran, Thane of Red Fell", 2),
        ("Mechanized Warfare", 1),
        ("Jaya, Venerated Firemage", 1),
        ("Embermaw Hellion", 1),
    ] {
        let mut t = TestGame::new(2);
        let perm = t.battlefield(P0, name);
        let axe = cast_targeting(&mut t, P0, "Lava Axe", &[Entity::Player(P1)]);
        t.resolve_all();
        assert_eq!(t.life(P1), 15 - bonus, "{name}");
        // One damage event, from Lava Axe, for the whole amount; none from the permanent.
        assert_eq!(
            damage_events(&t),
            vec![(axe, Entity::Player(P1), 5 + bonus as u32)],
            "{name}"
        );
        assert!(damage_events(&t).iter().all(|(s, _, _)| *s != perm));
    }
    // Torbran is the original source when Torbran (a red source) itself deals damage.
    let mut t = TestGame::new(2);
    let torbran = t.battlefield(P0, "Torbran, Thane of Red Fell");
    t.attack(&[(torbran, Entity::Player(P1))], &[]);
    assert_eq!(damage_events(&t), vec![(torbran, Entity::Player(P1), 4)]);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn mechanized_warfare_divides_the_original_amount_before_adding() {
    cr!("614.1a", "601.2d");
    ruling!(
        "Mechanized Warfare",
        "If damage dealt by a red or artifact source you control is being divided or assigned among multiple opponents and/or permanents opponents' control, divide the original amount before adding 1."
    );
    supported("Mechanized Warfare");
    supported("Forked Bolt");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mechanized Warfare");
    let giant = t.battlefield(P1, "Hill Giant");
    // Forked Bolt: 2 damage divided as you choose among one or two targets: 1 and 1.
    give_mana_for(&mut t, P0, "Forked Bolt");
    let card = t.hand(P0, "Forked Bolt");
    t.answer_targets(P0, &[obj(giant), Entity::Player(P1)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    t.cast_with(P0, card, &[]).unwrap();
    t.resolve_all();
    assert_eq!(damage_on(&t, giant), 2);
    assert_eq!(t.life(P1), 18);
}

/// P0 attacks with Charging Monstrosaur (a red 5/5 trampler), P1 blocks with Grizzly
/// Bears, and P0 assigns 2 damage to the Bears and 3 to P1. Returns the damage dealt to
/// the Bears and to P1.
fn trample_2_and_3(t: &mut TestGame) -> (u32, u32) {
    let m = t.battlefield(P0, "Charging Monstrosaur");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![2, 3]));
    t.attack(&[(m, Entity::Player(P1))], &[(bears, m)]);
    let ev = damage_events(t);
    let to = |e: Entity| {
        ev.iter()
            .filter(|(s, r, _)| *s == m && *r == e)
            .map(|(_, _, n)| *n)
            .sum::<u32>()
    };
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    (to(obj(bears)), to(Entity::Player(P1)))
}

#[test]
fn trample_damage_is_assigned_before_the_bonus_is_added() {
    cr!("510.1c", "614.1a");
    ruling!(
        "Jaya, Venerated Firemage",
        "If damage dealt by a source you control is being divided or assigned among multiple permanents an opponent controls or among an opponent and one or more permanents they control simultaneously, divide the original amount before adding 1."
    );
    ruling!(
        "The Flame of Keld",
        "If damage dealt by a source you control is being divided or assigned among multiple permanents an opponent controls or among an opponent and one or more permanents they control simultaneously, divide the original amount before adding 2."
    );
    ruling!(
        "Torbran, Thane of Red Fell",
        "If damage dealt by a source you control is being divided or assigned among multiple permanents an opponent controls or among an opponent and one or more permanents they control, divide the original amount before adding 2."
    );
    supported("Charging Monstrosaur");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jaya, Venerated Firemage");
    assert_eq!(trample_2_and_3(&mut t), (3, 4));
    assert_eq!(t.life(P1), 16);

    let mut t = TestGame::new(2);
    t.battlefield(P0, "Torbran, Thane of Red Fell");
    assert_eq!(trample_2_and_3(&mut t), (4, 5));
    assert_eq!(t.life(P1), 15);

    supported("The Flame of Keld");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "The Flame of Keld");
    add_lore(&mut t, saga, 3);
    t.resolve_all();
    assert_eq!(trample_2_and_3(&mut t), (4, 5));
    assert_eq!(t.life(P1), 15);
}

#[test]
fn sulfuric_vapors_adds_1_to_each_target_when_the_damage_would_be_dealt() {
    cr!("614.1a", "601.2d");
    ruling!(
        "Sulfuric Vapors",
        "If the red spell damages more than one target, add 1 to each target."
    );
    ruling!(
        "Sulfuric Vapors",
        "The ability is a replacement effect which is applied when damage would become dealt."
    );
    supported("Sulfuric Vapors");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sulfuric Vapors");
    let giant = t.battlefield(P1, "Indomitable Ancients");
    give_mana_for(&mut t, P0, "Forked Bolt");
    let card = t.hand(P0, "Forked Bolt");
    t.answer_targets(P0, &[obj(giant), Entity::Player(P1)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    t.cast_with(P0, card, &[]).unwrap();
    t.resolve_all();
    assert_eq!(damage_on(&t, giant), 2);
    assert_eq!(t.life(P1), 18);
    // The Vapors enter while Lightning Bolt is on the stack: the effect applies as the
    // damage would be dealt.
    let mut t = TestGame::new(2);
    cast_targeting(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.battlefield(P0, "Sulfuric Vapors");
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn fire_servant_applies_to_creatures_players_and_planeswalkers() {
    cr!("614.1a");
    ruling!(
        "Fire Servant",
        "Fire Servant’s ability applies no matter who or what the damage would be dealt to: a creature, a player, or a planeswalker."
    );
    supported("Fire Servant");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fire Servant");
    let wall = t.battlefield(P1, "Indomitable Ancients");
    let walker = t.battlefield(P1, "Ajani Goldmane");
    for target in [obj(wall), Entity::Player(P1), obj(walker)] {
        cast_targeting(&mut t, P0, "Lightning Bolt", &[target]);
        t.resolve_all();
    }
    let amounts: Vec<(Entity, u32)> = damage_events(&t)
        .into_iter()
        .map(|(_, r, n)| (r, n))
        .collect();
    assert_eq!(
        amounts,
        vec![(obj(wall), 6), (Entity::Player(P1), 6), (obj(walker), 6)]
    );
    assert_eq!(t.life(P1), 14);
    assert!(t.in_graveyard(P1, "Ajani Goldmane"));
}

#[test]
fn fire_servant_affects_only_damage_whose_source_is_the_spell() {
    cr!("614.1a", "614.2", "120.1");
    ruling!(
        "Fire Servant",
        "If a red instant or sorcery spell you control causes damage to be dealt, that spell will always identify the source of the damage."
    );
    supported("Soul's Fire");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fire Servant");
    // Lightning Bolt is the source of its damage: doubled.
    cast_targeting(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    // Soul's Fire: "Target creature you control on the battlefield deals damage equal to
    // its power to any target." The Hill Giant is the source: not doubled.
    let giant = t.battlefield(P0, "Hill Giant");
    cast_targeting(&mut t, P0, "Soul's Fire", &[obj(giant), Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 11);
}

#[test]
fn multiple_fire_servants_and_cities_on_fire_are_cumulative() {
    cr!("614.1a", "616.1");
    ruling!(
        "Fire Servant",
        "If you control more than one Fire Servant, the effects are cumulative. Two such effects will cause damage from red instant or sorcery spells you control to be multiplied by four; three such effects will cause damage from red instant or sorcery spells you control to be multiplied by eight."
    );
    ruling!(
        "City on Fire",
        "If you control two Cities on Fire, damage dealt by sources you control will be multiplied by 9. If you control three, it will be multiplied by 27, and so on."
    );
    supported("City on Fire");
    for (name, n, factor) in [
        ("Fire Servant", 2, 4),
        ("Fire Servant", 3, 8),
        ("City on Fire", 2, 9),
        ("City on Fire", 3, 27),
    ] {
        let mut t = TestGame::new(2);
        t.g.players[P1.idx()].life = 100;
        for _ in 0..n {
            t.battlefield(P0, name);
        }
        cast_targeting(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
        t.resolve_all();
        assert_eq!(t.life(P1), 100 - 3 * factor, "{n} × {name}");
    }
}

#[test]
fn ghostly_flame_multicolored_sources_with_red_or_black_deal_colorless_damage() {
    cr!("702.16e");
    ruling!(
        "Ghostly Flame",
        "If the source has more than one color but at least one is red or black, then the damage is colorless and all the other colors are forgotten."
    );
    // Boros Swiftblade (red and white) deals damage to Black Knight (protection from
    // white): with Ghostly Flame the damage is colorless and isn't prevented.
    for flame in [false, true] {
        let mut t = TestGame::new(2);
        if flame {
            t.battlefield(P0, "Ghostly Flame");
        }
        let blade = t.battlefield(P0, "Boros Swiftblade");
        let knight = t.battlefield(P1, "Black Knight");
        t.g.deal_damage(blade, obj(knight), 1, false);
        t.settle();
        assert_eq!(damage_on(&t, knight), u32::from(flame));
    }
}

#[test]
fn ghostly_flame_doesnt_change_the_sources_color() {
    cr!("105.2", "702.16e");
    ruling!(
        "Ghostly Flame",
        "It does not change the color of the source, so that things that trigger on a red spell doing damage will still trigger. The damage itself thinks it came from a colorless source, however."
    );
    ruling!(
        "Ghostly Flame",
        "The effect is continuous and applies whenever something looks at the damage. If this card leaves the battlefield, damage from red and black spells will appear as its normal color."
    );
    let mut t = TestGame::new(2);
    let flame = t.battlefield(P0, "Ghostly Flame");
    let paladin = t.battlefield(P1, "Paladin en-Vec");
    // The red spell stays red (God-Pharaoh's Faithful still sees a red spell), but its
    // damage is colorless: protection from red doesn't prevent it.
    t.battlefield(P0, "God-Pharaoh's Faithful");
    let shock = cast_targeting(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    assert!(t.obj(shock).chars.colors.contains(types::Color::Red));
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.deal_damage(giant, obj(paladin), 1, false);
    t.settle();
    assert_eq!(damage_on(&t, paladin), 1);
    // Once Ghostly Flame leaves the battlefield, the damage is red again: prevented.
    move_to(&mut t, flame, Zone::Graveyard(P0));
    t.recompute();
    t.g.deal_damage(giant, obj(paladin), 1, false);
    t.settle();
    assert_eq!(damage_on(&t, paladin), 1);
}
