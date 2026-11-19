//! Rulings residue — copying spells and permanents (batches P028, P030, P032): the
//! rulings that restate their card's own text, one case each (the interaction rulings are
//! in `r_residue_copy_instant.rs` and `r_residue_copy_instant_tokens.rs`).

use crate::r_p028_common::obj;
use crate::r_residue_copy_instant::{eldrazi_spell, kefnet_draw};
use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

#[test]
fn restatements() {
    let cases: &[(&str, fn())] = &[
        (
            "zada_triggers_only_for_a_spell_targeting_only_zada",
            zada_triggers_only_for_a_spell_targeting_only_zada,
        ),
        (
            "ulalek_must_be_on_the_battlefield_to_trigger",
            ulalek_must_be_on_the_battlefield_to_trigger,
        ),
        ("kefnet_s_reveal_is_optional", kefnet_s_reveal_is_optional),
        (
            "jace_reveals_the_drawn_card_as_it_resolves",
            jace_reveals_the_drawn_card_as_it_resolves,
        ),
        (
            "invasion_of_arcavios_searches_its_zones_in_one_choice",
            invasion_of_arcavios_searches_its_zones_in_one_choice,
        ),
    ];
    for (_name, case) in cases {
        case();
    }
}

fn zada_triggers_only_for_a_spell_targeting_only_zada() {
    cr!("603.2", "115.9c");
    ruling!(
        "Zada, Hedron Grinder",
        "Zada's ability triggers whenever you cast an instant or sorcery spell that targets only Zada and no other object or player."
    );
    supported("Zada, Hedron Grinder");
    // Electrolyze at Zada and P1: no trigger. At Zada alone: a trigger.
    for only in [false, true] {
        let mut t = TestGame::new(2);
        let zada = t.battlefield(P0, "Zada, Hedron Grinder");
        t.battlefield(P0, "Grizzly Bears");
        lands_for_cost(&mut t, P0, "Electrolyze");
        let card = t.hand(P0, "Electrolyze");
        if only {
            t.cast(P0, card).targets(&[obj(zada)]).go();
        } else {
            t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
            t.cast(P0, card).targets(&[obj(zada), pl(P1)]).go();
        }
        t.settle();
        assert_eq!(t.stack_len(), if only { 2 } else { 1 }, "only Zada: {only}");
    }
}

fn ulalek_must_be_on_the_battlefield_to_trigger() {
    cr!("603.2", "113.6");
    ruling!(
        "Ulalek, Fused Atrocity",
        "Ulalek must be on the battlefield in order for its triggered ability to trigger. Casting Ulalek itself won't cause that ability to trigger."
    );
    // Casting Ulalek (an Eldrazi spell): no trigger, and no {C}{C} asked for.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 7);
    let from = t.asked().len();
    let card = t.hand(P0, "Ulalek, Fused Atrocity");
    t.cast(P0, card).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::YesNo { .. })));
    t.resolve_all();
    // Once it's there, casting an Eldrazi spell triggers it.
    eldrazi_spell(&mut t);
    t.settle();
    assert_eq!(t.stack_len(), 2);
}

fn kefnet_s_reveal_is_optional() {
    cr!("701.20a", "603.2");
    ruling!(
        "God-Eternal Kefnet",
        "You don't have to reveal a drawn card if you don't wish to copy it at that time."
    );
    let (t, card) = kefnet_draw("Lightning Bolt", false, true);
    assert!(t.g.stack.is_empty());
    assert!(!mtg_engine::reveal::is_revealed(&t.g, t.g.current(card)));
    assert!(t.in_hand(P0, "Lightning Bolt"));
}

fn jace_reveals_the_drawn_card_as_it_resolves() {
    cr!("701.20a", "121.1");
    ruling!(
        "Jace, Mirror Mage",
        "While resolving Jace's last ability, you must reveal the card before it's mixed with the other cards in your hand."
    );
    supported("Jace, Mirror Mage");
    // The hand holds two other cards; the drawn Grizzly Bears is the card revealed, and
    // two loyalty counters (its mana value) are removed.
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace, Mirror Mage");
    t.hand(P0, "Island");
    t.hand(P0, "Lightning Bolt");
    let giant = t.library_top(P0, "Grizzly Bears");
    let from = t.g.turn_events.len();
    activate_containing(&mut t, P0, jace, "Draw a card").unwrap();
    t.resolve_all();
    let drawn = t.g.current(giant);
    assert_eq!(t.zone(drawn), Zone::Hand(P0));
    let revealed: Vec<ObjectId> = t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::Custom {
                name, obj: Some(o), ..
            } if name == mtg_engine::reveal::REVEALED => Some(*o),
            _ => None,
        })
        .collect();
    assert!(
        !revealed.is_empty()
            && revealed
                .iter()
                .all(|o| t.g.current(*o) == drawn || *o == giant),
        "{revealed:?}"
    );
    assert_eq!(t.counters(jace, "loyalty"), 2);
}

fn invasion_of_arcavios_searches_its_zones_in_one_choice() {
    cr!("701.23a", "400.11b");
    interp!("outside-the-game-is-the-sideboard");
    ruling!(
        "Invasion of Arcavios // Invocation of the Founders",
        "You don’t have to declare up front where you’re going to search. You may search your library, pause, sigh, check out your graveyard, frown meaningfully at your opponent, then finally grab a game-winning sorcery card from your sideboard. Bit dramatic though."
    );
    // A sorcery in each zone: one choice offers all three, and the one in the sideboard
    // can be taken.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let side =
        t.g.add_to_sideboard(P0, vec![mtg_engine::card::card("Divination")]);
    let lib = t.library_top(P0, "Lava Spike");
    let gy = t.graveyard(P0, "Lightning Bolt");
    t.answer_choose(P0, &[pl(P1)]);
    t.answer_choose(P0, &[obj(side[0])]);
    let from = t.asked().len();
    t.enter(P0, "Invasion of Arcavios // Invocation of the Founders");
    t.resolve_all();
    assert!(t.in_hand(P0, "Divination"));
    let offered: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert!(
        offered
            .iter()
            .any(|c| c.contains(&obj(lib)) && c.contains(&obj(gy)) && c.contains(&obj(side[0]))),
        "{offered:?}"
    );
}
