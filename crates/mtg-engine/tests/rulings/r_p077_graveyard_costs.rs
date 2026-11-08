//! Rulings batch P077 — exiling cards from a graveyard as a cost (CR 601.2h, 602.2b: no
//! player can respond while costs are paid), escape with other additional costs, Lich's
//! Mastery, From the Catacombs, and other graveyard-using cards.

use crate::r_s01_common::{supported, with_subtype};
use crate::r_s02_common::destroy;
use crate::r_s06_common::activate_containing;
use crate::r_s07_common::graveyard_n;
use crate::r_s08_common::priority_asks_of;
use crate::r_s21_common::castable;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const ESCAPE: CastMethod = CastMethod::Keyword(KeywordKind::Escape);

/// The cards in `p`'s graveyard with the given name.
fn yard_named(t: &TestGame, p: PlayerId, name: &str) -> usize {
    t.g.find_in_zone(Zone::Graveyard(p), name).len()
}

#[test]
fn graveyard_exile_costs_are_paid_before_anyone_can_respond() {
    cr!("602.2b", "601.2h", "117.1");
    ruling!(
        "Grim Lavamancer",
        "The two cards are exiled as the cost of Grim Lavamancer's ability is paid. Players can't respond to the paying of costs by trying to move those cards to another zone."
    );
    ruling!(
        "Dread Rider",
        "Although players may respond to the activated ability, they can’t respond to the paying of its activation cost. The creature card will already be exiled by the time anyone can respond."
    );
    ruling!(
        "Corpseweft",
        "Because exiling the creature cards from your graveyard is part of the activation cost, no player can respond to stop you from activating the ability by moving creature cards out of your graveyard."
    );
    ruling!(
        "Null Caller",
        "Once you activate the ability, it's too late for anyone to interrupt you by trying to remove the creature card from your graveyard."
    );
    ruling!(
        "Littjara Glade-Warden",
        "Once you announce that you’re activating the activated ability, no player may take actions until the ability has been paid for. Notably, opponents can’t try to remove creature cards from your graveyard to stop you from activating the ability."
    );
    ruling!(
        "Drudge Spell",
        "The creatures in the graveyard are exiled during activation as a cost."
    );
    // (card, the activated ability's text, creature cards it exiles, land for its mana,
    // its targets)
    let cases: [(&str, &str, usize, &str, bool); 6] = [
        ("Grim Lavamancer", "damage", 2, "Mountain", true),
        ("Dread Rider", "loses 3", 1, "Swamp", true),
        ("Corpseweft", "Zombie", 1, "Swamp", false),
        ("Null Caller", "Zombie", 1, "Swamp", false),
        ("Littjara Glade-Warden", "counters", 1, "Forest", true),
        ("Drudge Spell", "Skeleton", 2, "Swamp", false),
    ];
    for (name, needle, n, land, targeted) in cases {
        supported(name);
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, name);
        t.lands(P0, land, 4);
        graveyard_n(&mut t, P0, "Grizzly Bears", n);
        let bears = t.battlefield(P0, "Hill Giant");
        if targeted {
            let target = if needle == "counters" {
                Entity::Object(bears)
            } else {
                Entity::Player(P1)
            };
            t.answer_targets(P0, &[target]);
        }
        let from = t.asked().len();
        activate_containing(&mut t, P0, src, needle)
            .unwrap_or_else(|e| panic!("{name}: {e:?}"));
        // The ability is on the stack with its costs paid; no one had priority meanwhile.
        assert_eq!(t.stack_len(), 1, "{name}");
        assert_eq!(yard_named(&t, P0, "Grizzly Bears"), 0, "{name}");
        assert_eq!(t.g.exile.len(), n, "{name}");
        assert_eq!(priority_asks_of(&t, P1, from), 0, "{name}");
        // Removing cards from the graveyard now changes nothing.
        t.resolve_all();
        assert_eq!(t.stack_len(), 0);
    }
}

#[test]
fn corpseweft_counts_only_the_cards_exiled_for_that_activation() {
    cr!("607.2a", "602.2b");
    ruling!(
        "Corpseweft",
        "“Exiled this way” refers to that activation of the ability, not any creature cards that have been previously exiled."
    );
    let mut t = TestGame::new(2);
    let cw = t.battlefield(P0, "Corpseweft");
    t.lands(P0, "Swamp", 4);
    let cards = graveyard_n(&mut t, P0, "Grizzly Bears", 3);
    // "One or more": the number, then the cards.
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[Entity::Object(cards[0]), Entity::Object(cards[1])]);
    activate_containing(&mut t, P0, cw, "Zombie").unwrap();
    t.resolve_all();
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.answer_choose(P0, &[Entity::Object(cards[2])]);
    activate_containing(&mut t, P0, cw, "Zombie").unwrap();
    t.resolve_all();
    let mut pts: Vec<(i32, i32)> = with_subtype(&t, P0, "Zombie")
        .iter()
        .map(|z| t.pt(*z))
        .collect();
    pts.sort();
    assert_eq!(pts, vec![(2, 2), (4, 4)]);
}

#[test]
fn stalwart_valkyrie_alternative_cost() {
    cr!("118.9", "601.2b", "601.2h", "202.3a", "307.1");
    ruling!(
        "Stalwart Valkyrie",
        "Casting Stalwart Valkyrie for its alternative cost doesn't change when you can cast it."
    );
    ruling!(
        "Stalwart Valkyrie",
        "Once you announce that you're casting Stalwart Valkyrie using its alternative cost, no player may take other actions until the spell's been paid for. Notably, players can't try to remove creature cards from your graveyard to stop you from casting the spell."
    );
    ruling!(
        "Stalwart Valkyrie",
        "Stalwart Valkyrie has mana value 4, no matter what you actually paid to cast it."
    );
    supported("Stalwart Valkyrie");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    t.graveyard(P0, "Grizzly Bears");
    let valk = t.hand(P0, "Stalwart Valkyrie");
    let methods = crate::r_s08_common::legal_cast_methods(&mut t, P0, valk);
    let alt = methods
        .into_iter()
        .find(|m| matches!(m, CastMethod::Alternative(_)))
        .expect("the alternative cost");
    // Not at instant speed.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!castable(&mut t, P0, valk));
    t.set_step(P0, Step::PrecombatMain);
    let from = t.asked().len();
    let spell = t.cast(P0, valk).method(alt).go();
    assert!(!t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(priority_asks_of(&t, P1, from), 0);
    assert_eq!(crate::r_s08_common::mana_value(&t, spell), 4);
    t.resolve_all();
    assert_eq!(crate::r_s08_common::mana_value(&t, valk), 4);
}

#[test]
fn weigh_down_exiles_exactly_one_creature_card() {
    cr!("601.2b", "601.2f");
    ruling!(
        "Weigh Down",
        "You must exile exactly one creature card from your graveyard to cast this spell; you can’t cast it without exiling a creature card, and you can’t exile additional creature cards."
    );
    supported("Weigh Down");
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Hill Giant");
    let wd = crate::r_s03_common::in_hand_with_mana(&mut t, P0, "Weigh Down");
    t.graveyard(P0, "Forest");
    assert!(!castable(&mut t, P0, wd), "no creature card to exile");
    let cards = graveyard_n(&mut t, P0, "Grizzly Bears", 3);
    // Trying to exile two: only one is exiled.
    t.answer_choose(
        P0,
        &[Entity::Object(cards[0]), Entity::Object(cards[1])],
    );
    t.cast(P0, wd).target(Entity::Object(target)).go();
    assert_eq!(t.g.exile.len(), 1);
    assert_eq!(yard_named(&t, P0, "Grizzly Bears"), 2);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn horobis_whisper_targets_without_a_swamp_and_does_nothing() {
    cr!("608.2c", "115.1");
    ruling!(
        "Horobi's Whisper",
        "Horobi’s Whisper can target a creature even if you don’t control a Swamp, but the spell does nothing."
    );
    supported("Horobi's Whisper");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    t.g.players[0]
        .mana_pool
        .add_type(mtg_engine::mana::ManaType::B, 2);
    let hw = t.hand(P0, "Horobi's Whisper");
    t.cast(P0, hw).target(Entity::Object(bears)).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Horobi's Whisper"));
    // With a Swamp it destroys it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    let hw = t.hand(P0, "Horobi's Whisper");
    t.cast(P0, hw).target(Entity::Object(bears)).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn escape_may_exile_the_cards_discarded_or_sacrificed_for_its_additional_cost() {
    cr!("601.2h", "702.138a", "118.8");
    ruling!(
        "Underworld Breach",
        "If a spell you're casting with escape has an additional cost of discarding cards or sacrificing permanents, you may exile cards discarded or sacrificed this way to pay that part of its escape cost."
    );
    // Village Rites ({B}; "As an additional cost to cast this spell, sacrifice a
    // creature.") with only two other cards in the graveyard: the sacrificed creature is
    // the third card exiled.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Underworld Breach");
    t.lands(P0, "Swamp", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rites = t.graveyard(P0, "Village Rites");
    graveyard_n(&mut t, P0, "Llanowar Elves", 2);
    assert!(crate::r_s07_common::cast_methods(&mut t, P0, rites).contains(&ESCAPE));
    let hand = t.hand_size(P0);
    t.cast(P0, rites).method(ESCAPE).go();
    assert!(!t.on_battlefield(bears));
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.g.exile.len(), 3);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // Thrill of Possibility ({1}{R}; "As an additional cost to cast this spell, discard a
    // card.") with two other cards in the graveyard: the discarded card is exiled too.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Underworld Breach");
    t.lands(P0, "Mountain", 2);
    t.hand(P0, "Forest");
    let thrill = t.graveyard(P0, "Thrill of Possibility");
    graveyard_n(&mut t, P0, "Llanowar Elves", 2);
    assert!(crate::r_s07_common::cast_methods(&mut t, P0, thrill).contains(&ESCAPE));
    t.cast(P0, thrill).method(ESCAPE).go();
    assert!(t.in_exile("Forest"));
    assert_eq!(t.g.exile.len(), 3);
    // Still three cards in all: one other card and the sacrificed creature aren't enough,
    // nor is a sacrificed token (it isn't a card, CR 111.7).
    for (others, token) in [(1, false), (2, true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Underworld Breach");
        t.lands(P0, "Swamp", 1);
        if token {
            crate::r_s02_common::create_token(&mut t, P0, "Zombie");
        } else {
            t.battlefield(P0, "Grizzly Bears");
        }
        let rites = t.graveyard(P0, "Village Rites");
        graveyard_n(&mut t, P0, "Llanowar Elves", others);
        assert!(!crate::r_s07_common::cast_methods(&mut t, P0, rites).contains(&ESCAPE));
    }
}

#[test]
fn heated_argument_with_an_illegal_target_does_nothing() {
    cr!("608.2b");
    ruling!(
        "Heated Argument",
        "If the target creature is an illegal target as Heated Argument tries to resolve, it won't resolve and none of its effects will happen. You won't be able to exile any cards from your graveyard, and no player will be dealt damage."
    );
    supported("Heated Argument");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.graveyard(P0, "Forest");
    let ha = crate::r_s03_common::in_hand_with_mana(&mut t, P0, "Heated Argument");
    t.answer_yes(P0, true);
    t.cast(P0, ha).target(Entity::Object(bears)).go();
    crate::r_s05_common::move_to(&mut t, bears, Zone::Hand(P1));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(!t.in_exile("Forest"));
    assert_eq!(t.life(P1), 20);
    // With a legal target: 6 damage, and 2 to its controller for the exiled card.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.graveyard(P0, "Forest");
    let ha = crate::r_s03_common::in_hand_with_mana(&mut t, P0, "Heated Argument");
    t.answer_yes(P0, true);
    t.cast(P0, ha).target(Entity::Object(bears)).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.in_exile("Forest"));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn fleeting_spirit_leaves_combat_between_damage_steps() {
    cr!("510.4", "506.4");
    ruling!(
        "Fleeting Spirit",
        "If you activate Fleeting Spirit's first ability to give it first strike, you can later activate its last ability to remove it from combat after first strike combat damage is dealt but before normal combat damage is dealt."
    );
    supported("Fleeting Spirit");
    let mut t = TestGame::new(2);
    let spirit = t.battlefield(P0, "Fleeting Spirit");
    t.lands(P0, "Plains", 1);
    graveyard_n(&mut t, P0, "Forest", 3);
    t.hand(P0, "Island");
    // Craw Wurm: a 6/4.
    let giant = t.battlefield(P1, "Craw Wurm");
    activate_containing(&mut t, P0, spirit, "first strike").unwrap();
    t.resolve_all();
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(spirit, Entity::Player(P1))]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(giant, spirit)]),
    );
    t.advance_to(P0, Step::FirstStrikeDamage);
    t.settle();
    assert_eq!(crate::r_s07_common::damage_on(&t, giant), 3);
    activate_containing(&mut t, P0, spirit, "Discard").unwrap();
    t.resolve_all();
    assert_eq!(t.zone(spirit), Zone::Exile);
    t.advance_to(P0, Step::EndOfCombat);
    // It wasn't dealt the Giant's normal combat damage; it returns at the next end step.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let back = t.named_on_battlefield("Fleeting Spirit");
    assert_eq!(back.len(), 1);
    assert_eq!(t.obj_now(back[0]).damage, 0);
}

/// Answers Lich's Mastery's choices of what to exile with anything but Lich's Mastery
/// while there's something else.
fn spare_lichs_mastery(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    let Decision::ChooseEntities {
        candidates,
        min,
        max,
        ..
    } = d
    else {
        return None;
    };
    let mastery = |e: &Entity| matches!(e, Entity::Object(o) if g.obj(*o).chars.name == "Lich's Mastery");
    let mut pick: Vec<Entity> = candidates.iter().filter(|e| !mastery(e)).copied().collect();
    pick.extend(candidates.iter().filter(|e| mastery(e)).copied());
    pick.truncate((*max).max(*min) as usize);
    Some(Answer::Entities(pick))
}

#[test]
fn lichs_mastery_life_changes_and_running_out_of_things_to_exile() {
    cr!("119.3", "104.3b", "603.10a");
    ruling!(
        "Lich's Mastery",
        "While you control Lich’s Mastery, your life total still changes. Lich’s Mastery’s effects don’t replace the life gain or life loss."
    );
    ruling!(
        "Lich's Mastery",
        "If you’re dealt more damage than you have cards to exile, you’ll just exile everything you can."
    );
    ruling!(
        "Lich's Mastery",
        "If you run out of other permanents, cards in hand, and cards in graveyard, you’ll have to exile Lich’s Mastery itself and lose the game."
    );
    supported("Lich's Mastery");
    let mut t = TestGame::new(2);
    let lm = t.battlefield(P0, "Lich's Mastery");
    // P0 exiles other things while they can.
    crate::r_s03_common::respond(&mut t, P0, spare_lichs_mastery);
    for _ in 0..2 {
        t.hand(P0, "Forest");
    }
    t.graveyard(P0, "Forest");
    let hand = t.hand_size(P0);
    t.g.gain_life(P0, 2);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.hand_size(P0), hand + 2);
    t.g.lose_life(P0, 3);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.g.exile.len(), 3);
    assert!(t.on_battlefield(lm));
    // More life lost than cards: everything is exiled, Lich's Mastery too, and P0 loses.
    let size = t.hand_size(P0) + t.graveyard_size(P0) + 1;
    t.g.lose_life(P0, 30);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P0), -11);
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.graveyard_size(P0), 0);
    assert!(t.in_exile("Lich's Mastery"));
    assert_eq!(t.g.exile.len(), 3 + size);
    assert!(t.has_lost(P0));
}

#[test]
fn from_the_catacombs_replacement_ignores_the_corpse_counter() {
    cr!("614.1a", "614.12");
    ruling!(
        "From the Catacombs",
        "The corpse counter is only to help remind you which creatures will be exiled if they would leave the battlefield. If you somehow remove a corpse counter from a creature that was put onto the battlefield this way, the replacement effect that will exile it continues to apply."
    );
    supported("From the Catacombs");
    let mut t = TestGame::new(2);
    let card = t.graveyard(P1, "Hill Giant");
    let other = t.battlefield(P0, "Grizzly Bears");
    crate::r_s25_common::cast_new(&mut t, P0, "From the Catacombs", &[Entity::Object(card)]);
    t.resolve_all();
    let giant = t.named_on_battlefield("Hill Giant")[0];
    assert_eq!(t.obj(giant).controller, P0);
    assert_eq!(t.counters(giant, "corpse"), 1);
    // The counter moves to the Bears.
    t.g.remove_counters(Entity::Object(giant), "corpse", 1);
    crate::r_s29_common::put_counters(&mut t, other, "corpse", 1);
    destroy(&mut t, giant);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert!(!t.in_graveyard(P1, "Hill Giant"));
    destroy(&mut t, other);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn taigam_legend_rule_needs_the_exact_name() {
    cr!("704.5j", "201.2");
    ruling!(
        "Taigam, Sidisi's Hand",
        "The “legend rule” cares about legendary permanents with the exact same English name. You can control Taigam, Sidisi’s Hand and Taigam, Ojutai Master at the same time."
    );
    supported("Taigam, Sidisi's Hand");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Taigam, Sidisi's Hand");
    let b = t.battlefield(P0, "Taigam, Ojutai Master");
    t.settle();
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
    assert!(crate::r_s25_common::legendary(&t, a));
    t.battlefield(P0, "Taigam, Sidisi's Hand");
    t.settle();
    assert_eq!(t.named_on_battlefield("Taigam, Sidisi's Hand").len(), 1);
    assert!(t.on_battlefield(b));
}


