//! Rulings batch P155 — rulings that restate what the card does: a characteristic-defining
//! ability in every zone (CR 604.3), a colorless emblem (CR 114.6, 105.2c), revealed
//! cards put on the bottom (CR 701.20), a land returned with no damage, a spell whose
//! target became illegal (CR 608.2b), a spell that can't be countered still the target
//! of a counterspell, a replaced draw, "your next first main phase" (CR 505.1), and
//! nothing happening between a spell's actions (CR 608.2).

use crate::r_p155_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::ObjKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn dodgy_jalopys_power_is_defined_in_the_graveyard_too() {
    cr!("604.3", "702.97a");
    ruling!(
        "Dodgy Jalopy",
        "The ability that defines Dodgy Jalopy's power works in all zones, not just on the battlefield."
    );
    supported("Dodgy Jalopy");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let jalopy = t.graveyard(P0, "Dodgy Jalopy");
    assert_eq!(
        t.pt(jalopy),
        (4, 5),
        "Hill Giant's mana value, in the graveyard"
    );
    mana(&mut t, P0, "2G");
    act(&mut t, P0, jalopy, "Scavenge", &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 4, "{}", t.dump_log());
    assert!(t.in_exile("Dodgy Jalopy"));
}

#[test]
fn narsets_emblem_is_colorless_and_deals_damage_from_a_colorless_source() {
    cr!("114.3", "105.2c", "114.1");
    ruling!(
        "Narset of the Ancient Way",
        "The emblem created by Narset's last ability is colorless. The damage it deals is from a colorless source."
    );
    supported("Narset of the Ancient Way");
    let mut t = TestGame::new(2);
    let narset = t.battlefield(P0, "Narset of the Ancient Way");
    t.g.objects[narset.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 6);
    t.activate(P0, narset, 2, &[]).unwrap();
    t.resolve_all();
    let emblem =
        *t.g.command
            .iter()
            .find(|id| t.g.obj(**id).kind == ObjKind::Emblem)
            .expect("the emblem");
    assert!(t.g.obj(emblem).chars.colors.is_colorless());
    let events = t.g.turn_events.len();
    t.answer_targets(P0, &[pl(P1)]);
    cast_resolve(&mut t, P0, "Opt", "U", &[]);
    assert_eq!(t.life(P1), 18, "{}", t.dump_log());
    let from_emblem = t.g.turn_events[events..].iter().any(|e| {
        matches!(
            e,
            mtg_engine::events::Event::Damage { source, .. } if *source == emblem
        )
    });
    assert!(from_emblem, "the emblem is the source");
}

#[test]
fn calibrated_blast_puts_the_nonland_card_on_the_bottom_with_the_others() {
    cr!("701.20a", "401.4");
    ruling!(
        "Calibrated Blast",
        "The nonland card revealed this way ends up on the bottom of the library with the other cards."
    );
    supported("Calibrated Blast");
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Forest", "Island", "Hill Giant", "Swamp"]);
    t.answer_targets(P0, &[pl(P1)]);
    cast_resolve(&mut t, P0, "Calibrated Blast", "2R", &[]);
    assert_eq!(t.life(P1), 16, "{}", t.dump_log());
    let lib = crate::r_s33_common::library_top_first(&t, P0);
    assert_eq!(lib[0], cards[3], "the Swamp wasn't revealed");
    let bottom: Vec<ObjectId> = lib[lib.len() - 3..].to_vec();
    for c in &cards[..3] {
        let now = t.g.current(*c);
        assert!(
            bottom.contains(&now) || bottom.contains(c),
            "{}",
            t.g.obj(*c).chars.name
        );
    }
}

#[test]
fn vengeful_rebirth_returning_a_land_deals_no_damage() {
    cr!("608.2c");
    ruling!(
        "Vengeful Rebirth",
        "You can return a land card to your hand with Vengeful Rebirth. It just won’t deal any damage to the target permanent or player if you do."
    );
    let mut t = TestGame::new(2);
    let forest = t.graveyard(P0, "Forest");
    cast_resolve(
        &mut t,
        P0,
        "Vengeful Rebirth",
        "4RG",
        &[obj(forest), pl(P1)],
    );
    assert!(t.in_hand(P0, "Forest"));
    assert_eq!(t.life(P1), 20);
    assert!(t.in_exile("Vengeful Rebirth"));
}

#[test]
fn spells_whose_only_target_became_illegal_do_nothing() {
    cr!("608.2b", "115.1");
    ruling!(
        "Spark of Creativity",
        "You can't cast Spark of Creativity without targeting a creature. If the creature becomes an illegal target for Spark of Creativity, none of its effects happen. You don't exile the top card of your library."
    );
    ruling!(
        "Soul's Majesty",
        "Soul's Majesty's only target is the creature. If that creature becomes an illegal target by the time Soul's Majesty would resolve, the entire spell doesn't resolve. You won't draw any cards."
    );
    // Spark of Creativity: no creature, no cast; the creature dies in response: no exile.
    let mut t = TestGame::new(2);
    let spark = t.hand(P0, "Spark of Creativity");
    mana(&mut t, P0, "R");
    assert!(t.cast(P0, spark).try_go().is_err(), "no creature to target");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let s = cast(&mut t, P0, "Spark of Creativity", "R", &[obj(bears)]);
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 30);
    assert!(!resolved(&t, s));
    // Soul's Majesty on a Hill Giant that leaves: no cards.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let s = cast(&mut t, P0, "Soul's Majesty", "4G", &[obj(giant)]);
    bounce(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1, "only the bounced Giant");
    assert!(!resolved(&t, s));
}

#[test]
fn mystic_genesis_can_target_an_uncounterable_spell_and_still_makes_its_ooze() {
    cr!("608.2c", "115.1");
    ruling!(
        "Mystic Genesis",
        "You may target a spell that can’t be countered. When Mystic Genesis resolves, the target spell will be unaffected, but you’ll still get an Ooze token."
    );
    supported("Mystic Genesis");
    supported("Abrupt Decay");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let decay = cast_at(&mut t, P1, "Abrupt Decay", &[obj(bears)]);
    cast_resolve(&mut t, P0, "Mystic Genesis", "2GUU", &[obj(decay)]);
    let oozes = tokens(&t, P0);
    assert_eq!(oozes.len(), 1, "{}", t.dump_log());
    assert_eq!(t.pt(oozes[0]), (2, 2));
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "Abrupt Decay resolved");
}

#[test]
fn spellbound_dragon_discards_even_if_the_draw_was_replaced() {
    cr!("608.2c", "614.1a");
    ruling!(
        "Spellbound Dragon",
        "You'll discard a card even if some other effect replaces the draw."
    );
    supported("Spellbound Dragon");
    supported("Words of War");
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Spellbound Dragon");
    let words = t.battlefield(P0, "Words of War");
    let giant = t.hand(P0, "Hill Giant");
    mana(&mut t, P0, "1");
    t.answer_targets(P0, &[pl(P1)]);
    act(&mut t, P0, words, "next time you would draw", &[]).unwrap();
    t.resolve_all();
    t.answer_choose(P0, &[obj(giant)]);
    attack_with(&mut t, &[(dragon, pl(P1))]);
    t.resolve_all();
    assert_eq!(
        t.life(P1),
        18,
        "Words of War's 2 damage instead of the draw: {}",
        t.dump_log()
    );
    assert!(t.in_graveyard(P0, "Hill Giant"), "discarded all the same");
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.pt(dragon), (7, 5));
}

#[test]
fn plasm_captures_mana_comes_at_the_next_precombat_main_phase() {
    cr!("505.1", "505.1a", "603.7");
    ruling!(
        "Plasm Capture",
        "Your precombat main phase is the main phase that happens immediately after your draw step, not any other main phase."
    );
    supported("Plasm Capture");
    let mut t = TestGame::new(2);
    let bolt = cast_at(&mut t, P1, "Lightning Bolt", &[pl(P0)]);
    cast_resolve(&mut t, P0, "Plasm Capture", "GGUU", &[obj(bolt)]);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    // Not in this turn's postcombat main phase.
    t.advance_to(P0, Step::PostcombatMain);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(pool(&t, P0), 0);
    // At the next precombat main phase: X (1) mana.
    t.advance_to(P0, Step::PrecombatMain);
    t.settle();
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(pool(&t, P0), 1, "{}", t.dump_log());
}

#[test]
fn nothing_happens_between_the_actions_of_a_resolving_spell_or_ability() {
    cr!("608.2", "117.3b");
    ruling!(
        "Combustible Gearhulk",
        "While Combustible Gearhulk's triggered ability is resolving, no player may take any actions other than those specified by the Gearhulk. Once the opponent makes a choice, players can't cast spells or activate abilities until after the cards are drawn or put into the graveyard and damage has been dealt (if applicable)."
    );
    ruling!(
        "Erratic Mutation",
        "You reveal cards and the creature gets +X/-X all while Erratic Mutation is resolving. Nothing can happen in between, and no player can take actions."
    );
    supported("Combustible Gearhulk");
    supported("Erratic Mutation");
    // The Gearhulk: P1 declines the draw; the three cards are milled and 7 damage dealt
    // with no priority in between.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Hill Giant", "Centaur Courser", "Forest"]);
    t.hand(P1, "Shock");
    mana(&mut t, P1, "R");
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_yes(P1, false);
    let from = t.asked().len();
    t.enter(P0, "Combustible Gearhulk");
    t.settle();
    t.resolve();
    assert_eq!(t.library_size(P0), 30);
    assert_eq!(t.life(P1), 13, "{}", t.dump_log());
    assert!(asked_of(&t, P1, from)
        .iter()
        .all(|d| !matches!(d, Decision::Priority { .. })));
    // Erratic Mutation: the revealed Hill Giant gives +4/-4 at once: the Bears die.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Forest", "Hill Giant", "Island"]);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    cast(&mut t, P0, "Erratic Mutation", "2U", &[obj(bears)]);
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "{}", t.dump_log());
    assert!(t.asked()[from..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::Priority { .. })));
}
