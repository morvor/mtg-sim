//! CR 702.174 Gift: cards whose effects depend on whether the gift was promised
//! (CR 702.174k).

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_140_152::*;
use crate::common_k702_168_177::*;
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Casts `card` for P0 promising (or not) its gift; `targets` are answered in order.
fn cast_gift(t: &mut TestGame, card: ObjectId, promise: bool, targets: &[Entity]) -> ObjectId {
    pay_optional(t, P0, promise);
    for e in targets {
        t.answer_targets(P0, &[*e]);
    }
    t.cast(P0, card).go()
}

#[test]
fn parting_gust_returns_the_creature_only_if_the_gift_wasnt_promised() {
    cr!("702.174f", "702.174k");
    assert_supported("Parting Gust");
    ruling!(
        "Parting Gust",
        "Once the exiled permanent returns, it’s considered a new object with no relation to the object that it was."
    );
    // Parting Gust: {W}{W} instant, gift a tapped Fish, "Exile target nontoken creature.
    // If the gift wasn't promised, return that card to the battlefield under its owner's
    // control with a +1/+1 counter on it at the beginning of the next end step."
    for promise in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let giant = t.battlefield(P1, "Hill Giant");
        let gust = t.hand(P0, "Parting Gust");
        add_mana(&mut t, P0, ManaType::W, 2);
        cast_gift(&mut t, gust, promise, &[Entity::Object(giant)]);
        t.resolve_all();
        assert!(t.in_exile("Hill Giant"));
        assert_eq!(tokens_of_subtype(&t, P1, "Fish").len(), promise as usize);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        let back = t.named_on_battlefield("Hill Giant");
        if promise {
            assert!(back.is_empty());
            assert!(t.in_exile("Hill Giant"));
        } else {
            assert_eq!(back.len(), 1);
            assert_ne!(back[0], giant);
            assert_eq!(t.g.obj(back[0]).controller, P1);
            assert_eq!(t.counters(back[0], "+1/+1"), 1);
            assert_eq!(t.pt(back[0]), (4, 4));
        }
    }
}

#[test]
fn longstalk_brawl_adds_a_counter_before_the_fight_if_the_gift_was_promised() {
    cr!("702.174f", "702.174k", "701.14a");
    assert_supported("Longstalk Brawl");
    ruling!(
        "Longstalk Brawl",
        "If that creature is a legal target but the creature you don’t control isn’t, you’ll still put a +1/+1 counter"
    );
    // Longstalk Brawl: {G} sorcery, gift a tapped Fish, "Choose target creature you
    // control and target creature you don't control. Put a +1/+1 counter on the creature
    // you control if the gift was promised. Then those creatures fight each other."
    for promise in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let theirs = t.battlefield(P1, "Grizzly Bears");
        let brawl = t.hand(P0, "Longstalk Brawl");
        add_mana(&mut t, P0, ManaType::G, 1);
        cast_gift(
            &mut t,
            brawl,
            promise,
            &[Entity::Object(bears), Entity::Object(theirs)],
        );
        t.resolve_all();
        // 3/3 against 2/2: only the opponent's Bears die; unpromised, both do.
        assert!(!t.on_battlefield(theirs));
        assert_eq!(t.on_battlefield(bears), promise);
        if promise {
            assert_eq!(t.counters(bears, "+1/+1"), 1);
            assert_eq!(tokens_of_subtype(&t, P1, "Fish").len(), 1);
        }
    }
    // The creature you don't control became an illegal target: the counter is still put
    // on yours, and there's no fight.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let brawl = t.hand(P0, "Longstalk Brawl");
    add_mana(&mut t, P0, ManaType::G, 1);
    cast_gift(
        &mut t,
        brawl,
        true,
        &[Entity::Object(bears), Entity::Object(theirs)],
    );
    flicker(&mut t, theirs);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.g.obj(bears).damage, 0);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
}

#[test]
fn starfall_invocation_returns_a_creature_card_put_into_your_graveyard_this_way() {
    cr!("702.174e", "702.174k");
    assert_supported("Starfall Invocation");
    ruling!(
        "Starfall Invocation",
        "If the gift was promised, you choose which creature card to return to the battlefield while Starfall Invocation is resolving."
    );
    // Starfall Invocation: {3}{W}{W} sorcery, gift a card, "Destroy all creatures. If the
    // gift was promised, return a creature card put into your graveyard this way to the
    // battlefield under your control."
    for promise in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let giant = t.battlefield(P0, "Hill Giant");
        t.battlefield(P1, "Serra Angel");
        // A creature card that was already in your graveyard can't be returned.
        t.graveyard(P0, "Craw Wurm");
        let spell = t.hand(P0, "Starfall Invocation");
        add_mana(&mut t, P0, ManaType::W, 5);
        let hand = t.hand_size(P1);
        cast_gift(&mut t, spell, promise, &[]);
        t.resolve_all();
        assert_eq!(t.hand_size(P1), hand + promise as usize);
        assert!(t.in_graveyard(P1, "Serra Angel"));
        assert!(t.in_graveyard(P0, "Craw Wurm"));
        let back = t.named_on_battlefield("Hill Giant");
        assert_eq!(back.len(), promise as usize);
        if promise {
            assert_ne!(back[0], giant);
            assert_eq!(t.g.obj(back[0]).controller, P0);
        }
    }
}

#[test]
fn coiling_rebirth_copies_a_nonlegendary_creature_if_the_gift_was_promised() {
    cr!("702.174e", "702.174k");
    assert_supported("Coiling Rebirth");
    ruling!(
        "Coiling Rebirth",
        "The token created by the additional effect of Coiling Rebirth isn’t “cast,”"
    );
    // Coiling Rebirth: {3}{B}{B} sorcery, gift a card, "Return target creature card from
    // your graveyard to the battlefield. Then if the gift was promised and that creature
    // isn't legendary, create a token that's a copy of that creature, except it's 1/1."
    for (name, promise, copies) in [
        ("Hill Giant", false, 0),
        ("Hill Giant", true, 1),
        ("Isamaru, Hound of Konda", true, 0),
    ] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let card = t.graveyard(P0, name);
        let spell = t.hand(P0, "Coiling Rebirth");
        add_mana(&mut t, P0, ManaType::B, 5);
        let hand = t.hand_size(P1);
        cast_gift(&mut t, spell, promise, &[Entity::Object(card)]);
        t.resolve_all();
        // The gift: the opponent draws a card.
        assert_eq!(t.hand_size(P1), hand + promise as usize);
        let all = t.named_on_battlefield(name);
        assert_eq!(all.len(), 1 + copies, "{name} {promise}");
        let tokens: Vec<_> = all.iter().filter(|o| t.g.obj(**o).is_token()).collect();
        assert_eq!(tokens.len(), copies);
        for tok in tokens {
            assert_eq!(t.pt(*tok), (1, 1));
        }
    }
}

#[test]
fn cruelclaws_heist_lets_you_cast_the_exiled_card_if_the_gift_was_promised() {
    cr!("702.174e", "702.174k");
    assert_supported("Cruelclaw's Heist");
    ruling!(
        "Cruelclaw's Heist",
        "You pay all costs and follow all timing rules for a spell cast this way."
    );
    // Cruelclaw's Heist: {B}{B} sorcery, gift a card, "Target opponent reveals their hand.
    // You choose a nonland card from it. Exile that card. If the gift was promised, you
    // may cast that card for as long as it remains exiled, and mana of any type can be
    // spent to cast it."
    for promise in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let bolt = t.hand(P1, "Lightning Bolt");
        t.hand(P1, "Mountain");
        let spell = t.hand(P0, "Cruelclaw's Heist");
        add_mana(&mut t, P0, ManaType::B, 2);
        let hand = t.hand_size(P1);
        cast_gift(&mut t, spell, promise, &[Entity::Player(P1)]);
        t.resolve_all();
        assert!(t.in_exile("Lightning Bolt"));
        assert!(t.in_hand(P1, "Mountain"));
        // The gift (a card, drawn before the rest) and the exiled Bolt.
        assert_eq!(t.hand_size(P1), hand + promise as usize - 1);
        let exiled = t.g.current(bolt);
        assert_eq!(t.zone(exiled), Zone::Exile);
        // Green mana can pay for {R}.
        add_mana(&mut t, P0, ManaType::G, 1);
        assert_eq!(!cast_methods_now(&mut t, P0, exiled).is_empty(), promise);
        if promise {
            t.cast(P0, exiled).target(Entity::Player(P1)).go();
            assert_eq!(pool(&t, P0), 0);
            t.resolve_all();
            assert_eq!(t.life(P1), 17);
            assert!(t.in_graveyard(P1, "Lightning Bolt"));
        }
    }
}

#[test]
fn consumed_by_greed_edict_and_return_if_the_gift_was_promised() {
    cr!("702.174e", "702.174m");
    assert_supported("Consumed by Greed");
    ruling!(
        "Consumed by Greed",
        "If the target opponent has multiple creatures tied for the greatest power, that player chooses which one to sacrifice."
    );
    // Consumed by Greed: {1}{B}{B} instant, gift a card, "Target opponent sacrifices a
    // creature with the greatest power among creatures they control. If the gift was
    // promised, return target creature card from your graveyard to your hand."
    for promise in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        let other_giant = t.battlefield(P1, "Hill Giant");
        let wurm = t.graveyard(P0, "Craw Wurm");
        let spell = t.hand(P0, "Consumed by Greed");
        add_mana(&mut t, P0, ManaType::B, 3);
        let mut targets = vec![Entity::Player(P1)];
        if promise {
            targets.push(Entity::Object(wurm));
        }
        let hand = t.hand_size(P1);
        let cast = cast_gift(&mut t, spell, promise, &targets);
        // The graveyard card is targeted only if the gift was promised (CR 702.174m).
        let chosen: usize = t
            .obj(cast)
            .stack
            .as_ref()
            .unwrap()
            .chosen
            .iter()
            .flat_map(|c| c.targets.iter())
            .map(Vec::len)
            .sum();
        assert_eq!(chosen, targets.len(), "promised: {promise}");
        // The opponent chooses between the two tied Hill Giants: the Bears, with less
        // power, can't be chosen.
        t.answer_choose(P1, &[Entity::Object(other_giant)]);
        t.resolve_all();
        let mut offered = t
            .asked()
            .into_iter()
            .find_map(|(p, d)| match d {
                Decision::ChooseEntities { candidates, .. } if p == P1 => Some(candidates),
                _ => None,
            })
            .expect("the opponent chose a creature to sacrifice");
        offered.sort();
        let mut giants = vec![Entity::Object(giant), Entity::Object(other_giant)];
        giants.sort();
        assert_eq!(offered, giants);
        assert!(t.on_battlefield(bears) && t.on_battlefield(giant));
        assert!(!t.on_battlefield(other_giant));
        assert_eq!(t.in_hand(P0, "Craw Wurm"), promise);
        assert_eq!(t.hand_size(P1), hand + promise as usize);
    }
}
