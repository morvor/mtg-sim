//! Rulings batch P034 — abilities that count the counters on their source as they
//! resolve: when the source has left the battlefield, its last known information is used
//! (CR 608.2h, 113.7a), and a counter the ability would put on it first isn't put
//! (CR 400.7: the card in the graveyard is a new object).

use crate::r_p034_common::*;
use crate::r_p038_common::{add, kill, next_p0_upkeep};
use crate::r_s01_common::{attack_with, stack_library, supported, tokens};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[test]
fn voodoo_doll_cost_x_is_counted_on_activation_and_the_damage_on_resolution() {
    cr!("608.2h", "107.3a");
    ruling!(
        "Voodoo Doll",
        "Each X in the cost of Voodoo Doll's third ability is the number of pin counters"
    );
    supported("Voodoo Doll");
    // Two pin counters: the cost is {2}{2}. A pin counter put on it in response raises the
    // damage to 3.
    let mut t = TestGame::new(2);
    let doll = t.battlefield(P0, "Voodoo Doll");
    put(&mut t, doll, "pin", 2);
    pool(&mut t, P0, ManaType::C, 3);
    assert!(
        t.activate(P0, doll, 0, &[Entity::Player(P1)]).is_err(),
        "{{X}}{{X}} with two pin counters costs 4"
    );
    pool(&mut t, P0, ManaType::C, 1);
    t.activate(P0, doll, 0, &[Entity::Player(P1)]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0, "paid {{2}}{{2}}");
    put(&mut t, doll, "pin", 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "the pin counters as it resolves");

    // The Doll is destroyed in response: its last existence on the battlefield counts.
    let mut t = TestGame::new(2);
    let doll = t.battlefield(P0, "Voodoo Doll");
    put(&mut t, doll, "pin", 2);
    pool(&mut t, P0, ManaType::C, 4);
    t.activate(P0, doll, 0, &[Entity::Player(P1)]).unwrap();
    kill(&mut t, doll);
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "two pin counters as it last existed");
}

#[test]
fn upkeep_counters_then_count_use_last_known_counters() {
    // "At the beginning of your upkeep, put a <kind> counter on ~, then <do something for
    // each counter on it>": with the source destroyed in response, no counter is added
    // and the counters it had as it left are counted.
    cr!("608.2h", "400.7");
    ruling!(
        "Assemble the Legion",
        "use the number of muster counters it had when it was last on the battlefield"
    );
    ruling!(
        "The Magic Mirror",
        "the ability will use the number of knowledge counters The Magic Mirror had before leaving"
    );
    supported("Assemble the Legion");
    supported("The Magic Mirror");
    // Assemble the Legion with two muster counters: two Soldiers, not three (or none).
    let mut t = TestGame::new(2);
    let atl = add(&mut t, P0, "Assemble the Legion");
    put(&mut t, atl, "muster", 2);
    next_p0_upkeep(&mut t);
    t.settle();
    assert!(top_is_trigger(&t), "{}", t.dump_log());
    kill(&mut t, atl);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
    assert_eq!(t.counters(atl, "muster"), 0, "the card in the graveyard has none");

    // The Magic Mirror with two knowledge counters draws two cards.
    let mut t = TestGame::new(2);
    let mirror = add(&mut t, P0, "The Magic Mirror");
    put(&mut t, mirror, "knowledge", 2);
    next_p0_upkeep(&mut t);
    t.settle();
    let hand = t.hand_size(P0);
    kill(&mut t, mirror);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn wilfred_mott_looks_at_as_many_cards_as_its_last_known_time_counters() {
    cr!("608.2h");
    ruling!(
        "Wilfred Mott",
        "use the number of time counters it had on it the last time it was on the battlefield"
    );
    supported("Wilfred Mott");
    // One time counter, destroyed in response: X is 1, so only the top card is looked at
    // and the Grizzly Bears second from the top can't be chosen.
    for (killed, bears_found) in [(true, false), (false, true)] {
        let mut t = TestGame::new(2);
        let wilfred = add(&mut t, P0, "Wilfred Mott");
        put(&mut t, wilfred, "time", 1);
        next_p0_upkeep(&mut t);
        t.settle();
    t.settle();
        // (The draw step comes after the upkeep: the library is set up now.)
        let bears = stack_library(&mut t, P0, &["Plains", "Grizzly Bears"])[1];
        if killed {
            kill(&mut t, wilfred);
        }
        t.answer_choose(P0, &[obj(bears)]);
        t.resolve_all();
        assert_eq!(
            !t.named_on_battlefield("Grizzly Bears").is_empty(),
            bears_found,
            "killed: {killed}"
        );
    }
}

#[test]
fn charging_cinderhorn_deals_damage_equal_to_its_last_known_fury_counters() {
    cr!("608.2h");
    ruling!(
        "Charging Cinderhorn",
        "it won’t get a new fury counter, but it will deal damage equal to the number of fury counters it had"
    );
    supported("Charging Cinderhorn");
    let mut t = TestGame::new(2);
    let ox = add(&mut t, P1, "Charging Cinderhorn");
    put(&mut t, ox, "fury", 2);
    t.advance_to(P0, Step::End);
    t.settle();
    assert!(top_is_trigger(&t));
    kill(&mut t, ox);
    t.resolve_all();
    assert_eq!(t.life(P0), 18, "two fury counters, not three");
}

#[test]
fn tap_abilities_that_count_their_source_use_last_known_counters() {
    cr!("608.2h");
    ruling!(
        "Grindclock",
        "use the number of charge counters on it when it left the battlefield to determine how many cards"
    );
    ruling!(
        "Insight Engine",
        "it won’t have a charge counter put on it when the ability resolves. Use the number of charge counters"
    );
    supported("Grindclock");
    supported("Insight Engine");
    // Grindclock with three charge counters: the target player mills three.
    let mut t = TestGame::new(2);
    let gc = t.battlefield(P0, "Grindclock");
    put(&mut t, gc, counters::CHARGE, 3);
    let lib = t.library_size(P1);
    t.activate(P0, gc, 1, &[Entity::Player(P1)]).unwrap();
    kill(&mut t, gc);
    t.resolve_all();
    assert_eq!(t.library_size(P1), lib - 3);

    // Insight Engine with one charge counter: one card (not two).
    let mut t = TestGame::new(2);
    let ie = t.battlefield(P0, "Insight Engine");
    put(&mut t, ie, counters::CHARGE, 1);
    pool(&mut t, P0, ManaType::C, 2);
    let hand = t.hand_size(P0);
    t.activate(P0, ie, 0, &[]).unwrap();
    kill(&mut t, ie);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn jhoira_still_puts_an_artifact_onto_the_battlefield_with_its_last_known_x() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Jhoira, Ageless Innovator",
        "you can still put an artifact card from your hand onto the battlefield. Use the number of ingenuity counters"
    );
    supported("Jhoira, Ageless Innovator");
    // Two ingenuity counters, destroyed in response: X is 2 (not 4, not 0).
    for (artifact, put_in) in [("Mind Stone", true), ("Pristine Talisman", false)] {
        let mut t = TestGame::new(2);
        let jhoira = t.battlefield(P0, "Jhoira, Ageless Innovator");
        put(&mut t, jhoira, "ingenuity", 2);
        let card = t.hand(P0, artifact);
        t.activate(P0, jhoira, 0, &[]).unwrap();
        kill(&mut t, jhoira);
        t.answer_choose(P0, &[obj(card)]);
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(
            !t.named_on_battlefield(artifact).is_empty(),
            put_in,
            "{artifact}"
        );
    }
}

#[test]
fn yisan_searches_with_its_last_known_verse_counters_including_the_cost_counter() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Yisan, the Wanderer Bard",
        "This number will include the counter you put on Yisan to activate the ability."
    );
    supported("Yisan, the Wanderer Bard");
    let mut t = TestGame::new(2);
    let yisan = t.battlefield(P0, "Yisan, the Wanderer Bard");
    put(&mut t, yisan, "verse", 1);
    let elves = t.library_top(P0, "Llanowar Elves");
    let bears = t.library_top(P0, "Grizzly Bears");
    pool(&mut t, P0, ManaType::G, 3);
    t.activate(P0, yisan, 0, &[]).unwrap();
    assert_eq!(t.counters(yisan, "verse"), 2, "the cost put a verse counter");
    kill(&mut t, yisan);
    t.answer_choose(P0, &[obj(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears), "mana value 2 = its two verse counters");
    assert!(!t.on_battlefield(elves));
}

#[test]
fn herald_of_ilharg_deals_its_last_known_counters_in_damage() {
    cr!("608.2h");
    ruling!(
        "Herald of Ilharg",
        "use the number of counters that were on Herald of Ilharg as it last existed"
    );
    supported("Herald of Ilharg");
    let mut t = TestGame::new(2);
    let herald = t.battlefield(P0, "Herald of Ilharg");
    put(&mut t, herald, counters::PLUS1, 3);
    cast_card(&mut t, P0, "Craw Wurm", &[]);
    assert!(top_is_trigger(&t));
    kill(&mut t, herald);
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "three counters as it last existed, not five");
}

#[test]
fn combat_damage_triggers_count_their_sources_last_known_counters() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Surrakar Spellblade",
        "its last existence on the battlefield is checked to determine how many charge counters"
    );
    ruling!(
        "Malcolm, Alluring Scoundrel",
        "You may still cast the discarded card without paying its mana cost if Malcolm had four or more"
    );
    supported("Surrakar Spellblade");
    supported("Malcolm, Alluring Scoundrel");
    // Surrakar Spellblade with two charge counters: draw two.
    let mut t = TestGame::new(2);
    let ss = t.battlefield(P0, "Surrakar Spellblade");
    put(&mut t, ss, counters::CHARGE, 2);
    attack_with(&mut t, &[(ss, Entity::Player(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert!(top_is_trigger(&t));
    kill(&mut t, ss);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);

    // Malcolm with four chorus counters (may cast the discard) and with three (can't:
    // the counter isn't put on it).
    for (chorus, cast) in [(4, true), (3, false)] {
        let mut t = TestGame::new(2);
        let m = t.battlefield(P0, "Malcolm, Alluring Scoundrel");
        put(&mut t, m, "chorus", chorus);
        let helix = t.library_top(P0, "Lightning Helix");
        attack_with(&mut t, &[(m, Entity::Player(P1))]);
        t.advance_to(P0, Step::CombatDamage);
        t.settle();
        assert!(top_is_trigger(&t));
        kill(&mut t, m);
        // Draw the Helix, discard it, and cast it at P1 if allowed.
        t.answer_choose(P0, &[obj(helix)]);
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.resolve_all();
        assert_eq!(t.life(P1), if cast { 20 - 2 - 3 } else { 18 }, "{chorus}");
        assert_eq!(t.life(P0), if cast { 23 } else { 20 }, "{chorus}");
    }
}

#[test]
fn sphinx_bone_wand_deals_no_damage_if_it_left_before_the_counter() {
    cr!("608.2c", "400.7");
    ruling!(
        "Sphinx-Bone Wand",
        "you won’t be able to put a charge counter on it. As a result, it won’t deal damage."
    );
    supported("Sphinx-Bone Wand");
    let mut t = TestGame::new(2);
    let wand = t.battlefield(P0, "Sphinx-Bone Wand");
    put(&mut t, wand, counters::CHARGE, 2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_card(&mut t, P0, "Divination", &[]);
    assert!(top_is_trigger(&t));
    kill(&mut t, wand);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "no counter was put on it, so no damage");
    // On the battlefield, it would have dealt 3.
    let mut t = TestGame::new(2);
    let wand = t.battlefield(P0, "Sphinx-Bone Wand");
    put(&mut t, wand, counters::CHARGE, 2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_card(&mut t, P0, "Divination", &[]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn nissa_uses_her_last_known_loyalty() {
    cr!("608.2h", "120.3c", "704.5i");
    ruling!(
        "Nissa, Steward of Elements",
        "If Nissa left the battlefield because she took enough damage to reduce her loyalty to 0, then her last known loyalty is 0."
    );
    supported("Nissa, Steward of Elements");
    // Loyalty 2, top card a mana value 2 creature. Destroyed in response: her last known
    // loyalty is 2. Dealt 2 damage in response: it is 0.
    for (how, put_in) in [("none", true), ("destroyed", true), ("damage", false)] {
        let mut t = TestGame::new(2);
        let nissa = t.battlefield(P0, "Nissa, Steward of Elements");
        t.g.objects[nissa.0 as usize]
            .counters
            .insert(counters::LOYALTY.into(), 2);
        let bears = t.library_top(P0, "Grizzly Bears");
        t.activate(P0, nissa, 1, &[]).unwrap();
        assert_eq!(t.counters(nissa, counters::LOYALTY), 2, "a 0 ability");
        match how {
            "destroyed" => kill(&mut t, nissa),
            "damage" => {
                let shock = t.hand(P1, "Shock");
                t.lands(P1, "Mountain", 1);
                t.cast(P1, shock).target(nissa).go();
                t.resolve();
                assert!(!t.on_battlefield(nissa));
            }
            _ => {}
        }
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(t.on_battlefield(bears), put_in, "{how}");
    }
}

#[test]
fn flay_essence_counts_every_kind_of_counter_the_creature_had() {
    cr!("608.2h");
    ruling!(
        "Flay Essence",
        "counts the number of counters of all kinds that the creature or planeswalker had on it"
    );
    supported("Flay Essence");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    put(&mut t, bears, counters::PLUS1, 2);
    put(&mut t, bears, "stun", 1);
    cast_card(&mut t, P0, "Flay Essence", &[obj(bears)]);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.life(P0), 23);
}

#[test]
fn galloping_lizrog_trigger_still_removes_counters_after_it_left() {
    cr!("113.7a");
    ruling!(
        "Galloping Lizrog",
        "you may remove any number of +1/+1 counters from among your creatures, even though you won’t put any on Galloping Lizrog"
    );
    supported("Galloping Lizrog");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, bears, counters::PLUS1, 2);
    let lizrog = crate::r_p038_common::enter(&mut t, P0, "Galloping Lizrog");
    assert!(top_is_trigger(&t));
    kill(&mut t, lizrog);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Number, mtg_engine::decision::Answer::Number(2));
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 0, "the counters were removed");
    assert_eq!(t.counters(lizrog, counters::PLUS1), 0);
}

#[test]
fn quarantine_field_exiles_nothing_if_it_left_before_its_trigger_resolved() {
    cr!("610.3b");
    ruling!(
        "Quarantine Field",
        "If Quarantine Field leaves the battlefield before its triggered ability resolves, no permanents will be exiled."
    );
    supported("Quarantine Field");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    pool(&mut t, P0, ManaType::W, 4);
    let qf = t.hand(P0, "Quarantine Field");
    t.cast(P0, qf).x(1).go();
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve();
    assert!(top_is_trigger(&t));
    let field = t.named_on_battlefield("Quarantine Field")[0];
    kill(&mut t, field);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

