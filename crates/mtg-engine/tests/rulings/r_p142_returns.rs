//! Rulings batch P142 — returning cards from the graveyard: targets chosen when the
//! ability is put on the stack (and illegal by resolution), choices made on resolution
//! (at random, by an opponent, without a target), owners versus controllers, and
//! intervening "if" clauses checked twice.

use crate::r_p142_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// P0 attacks P1 with Grizzly Bears equipped with Sword of Light and Shadow; stops with
/// the Sword's trigger on the stack (targets scripted by the caller).
fn sword_hits(t: &mut TestGame) {
    let b = t.battlefield(P0, "Grizzly Bears");
    attach_new(t, P0, "Sword of Light and Shadow", b);
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(b, pl(P1))]),
    );
    t.set_step(P0, Step::BeginningOfCombat);
    until_stack(t);
    assert_eq!(stacked_triggers(t), 1);
}

#[test]
fn sword_of_light_and_shadow_fizzles_only_with_an_illegal_chosen_target() {
    cr!("608.2b", "115.4");
    ruling!(
        "Sword of Light and Shadow",
        "If you choose a target creature card in your graveyard and that card is an illegal target by the time the triggered ability tries to resolve, the ability won't resolve. You won't gain 3 life. If you don't choose a target at all, you just gain 3 life."
    );
    supported("Sword of Light and Shadow");
    // The target leaves the graveyard in response: no life.
    let mut t = TestGame::new(2);
    let g = t.graveyard(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(g)]);
    sword_hits(&mut t);
    exile(&mut t, g);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // No target: 3 life.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Hill Giant");
    t.answer_targets(P0, &[]);
    sword_hits(&mut t);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert!(t.in_graveyard(P0, "Hill Giant"));
    // Positive control: a legal target is returned, and the life is gained.
    let mut t = TestGame::new(2);
    let g = t.graveyard(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(g)]);
    sword_hits(&mut t);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert!(t.in_hand(P0, "Hill Giant"));
}

#[test]
fn sword_of_light_and_shadow_can_target_a_creature_that_died_in_the_same_damage() {
    cr!("510.2", "704.3", "603.3b");
    ruling!(
        "Sword of Light and Shadow",
        "The triggered ability can target a creature card that was put into your graveyard as a result of state-based actions immediately after the combat damage that caused the ability to trigger."
    );
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Sword of Light and Shadow", b);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    target_moved(&mut t, P0, &[elves]);
    attack_with(&mut t, &[(b, pl(P1)), (elves, pl(P1))]);
    block_and_finish(&mut t, P1, &[(giant, elves)]);
    assert!(t.in_hand(P0, "Llanowar Elves"));
    assert_eq!(t.life(P0), 23);
}

#[test]
fn enduring_renewal_is_a_trigger_that_misses_cards_that_left_and_tokens() {
    cr!("603.6a", "608.2b", "111.7");
    ruling!(
        "Enduring Renewal",
        "If, after the last ability triggers, the creature card is removed from your graveyard in response, it won’t be returned to your hand."
    );
    ruling!(
        "Enduring Renewal",
        "The last ability is now a triggered ability, not a replacement effect. That means your opponent has the opportunity to remove the card from your graveyard while the triggered ability is still on the stack."
    );
    ruling!(
        "Enduring Renewal",
        "Token creatures cease to exist if they leave the battlefield, so this effect will not let you get them in your hand."
    );
    supported("Enduring Renewal");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Enduring Renewal");
    let b = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, b);
    // The card is in the graveyard, with the trigger on the stack.
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(stacked_triggers(&t), 1);
    exile(&mut t, b);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(!t.in_hand(P0, "Grizzly Bears"));
    // Positive control: unanswered, it returns.
    let b = t.battlefield(P0, "Hill Giant");
    destroy(&mut t, b);
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    // A token creature.
    let hand = t.hand_size(P0);
    let tok = create_token(&mut t, P0, "Spirit");
    destroy(&mut t, tok);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn owner_triggers_see_cards_that_died_under_another_players_control() {
    cr!("108.3", "603.10a", "404.2");
    ruling!(
        "Athreos, God of Passage",
        "It doesn't matter who controlled the creature when it died. Athreos's last ability will trigger if you owned that creature."
    );
    ruling!(
        "Pia's Revolution",
        "It doesn’t matter who controlled the artifact when it was put into your graveyard. Pia’s Revolution’s ability will trigger if you own that artifact."
    );
    ruling!(
        "Sigil of the New Dawn",
        "Sigil of the New Dawn's ability triggers if a creature you own dies under another player's control."
    );
    supported("Athreos, God of Passage");
    supported("Pia's Revolution");
    supported("Sigil of the New Dawn");
    // Athreos: P1 declines to pay.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Athreos, God of Passage");
    let b = t.battlefield(P0, "Grizzly Bears");
    gain_control(&mut t, P1, b);
    no(&mut t, P1);
    destroy(&mut t, b);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // Pia's Revolution: P1 declines the damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pia's Revolution");
    let o = t.battlefield(P0, "Ornithopter");
    gain_control(&mut t, P1, o);
    no(&mut t, P1);
    destroy(&mut t, o);
    t.resolve_all();
    assert!(t.in_hand(P0, "Ornithopter"));
    assert_eq!(t.life(P1), 20);
    // Sigil of the New Dawn: P0 pays {1}{W}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sigil of the New Dawn");
    t.lands(P0, "Plains", 2);
    let b = t.battlefield(P0, "Grizzly Bears");
    gain_control(&mut t, P1, b);
    yes(&mut t, P0);
    destroy(&mut t, b);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // None of them triggers for a creature P0 controlled but P1 owns.
    for card in ["Athreos, God of Passage", "Sigil of the New Dawn"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, card);
        let b = t.battlefield(P1, "Grizzly Bears");
        gain_control(&mut t, P0, b);
        destroy(&mut t, b);
        assert_eq!(stacked_triggers(&t), 0, "{card}");
    }
}

#[test]
fn athreos_opponent_chooses_on_resolution_and_tokens_you_created_are_yours() {
    cr!("608.2c", "111.2", "108.3");
    ruling!(
        "Athreos, God of Passage",
        "The target opponent chooses whether to pay 3 life when Athreos's last ability resolves. You won't return the card to your hand if that player pays 3 life or if the card leaves the graveyard before the ability resolves."
    );
    ruling!(
        "Athreos, God of Passage",
        "You are a creature's owner if the card representing it began the game in your deck, or if it's a token that entered the battlefield under your control."
    );
    // P1 pays 3 life.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Athreos, God of Passage");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P1, true);
    destroy(&mut t, b);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // The card leaves the graveyard first: P1 isn't asked.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Athreos, God of Passage");
    let b = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, b);
    exile(&mut t, b);
    no(&mut t, P1);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(!t.in_hand(P0, "Grizzly Bears"));
    // A token P0 created dies under P1's control: Athreos still triggers for P0.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Athreos, God of Passage");
    let tok = create_token(&mut t, P0, "Spirit");
    gain_control(&mut t, P1, tok);
    destroy(&mut t, tok);
    assert_eq!(stacked_triggers(&t), 1);
}

#[test]
fn liliana_the_last_hope_abilities() {
    cr!("115.1", "606.3", "114.4");
    ruling!(
        "Liliana, the Last Hope",
        "Liliana's second ability doesn't target the card to return to your hand. After putting the top two cards of your library into your graveyard, you may choose from among any creature cards there."
    );
    ruling!(
        "Liliana, the Last Hope",
        "You can activate Liliana's first ability without any targets just to add a loyalty counter to her."
    );
    ruling!(
        "Liliana, the Last Hope",
        "While resolving the triggered ability of Liliana's emblem, if you control zero Zombies, you'll get two Zombie tokens."
    );
    supported("Liliana, the Last Hope");
    // +1 with no creature on the battlefield.
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Liliana, the Last Hope");
    t.answer_targets(P0, &[]);
    t.activate(P0, l, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(l, "loyalty"), 4);
    // −2: a milled creature card can be chosen; no target was asked.
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Liliana, the Last Hope");
    t.graveyard(P0, "Grizzly Bears");
    let g = t.library_top(P0, "Hill Giant");
    t.answer_choose_moved(P0, &[obj(g)]);
    let from = n_asked(&t);
    t.activate(P0, l, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(target_candidates(&t, P0, from).is_empty());
    // −7: the emblem with no Zombies makes two.
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Liliana, the Last Hope");
    loyalty(&mut t, l, 7);
    t.activate(P0, l, 2, &[]).unwrap();
    t.resolve_all();
    end_step(&mut t, P0);
    assert_eq!(with_subtype(&t, P0, "Zombie").len(), 2);
}

#[test]
fn mtenda_griffin_needs_a_griffin_card_to_target() {
    cr!("602.2b", "601.2c");
    ruling!(
        "Mtenda Griffin",
        "Must have a Griffin in the graveyard to use this ability because it requires a target."
    );
    supported("Mtenda Griffin");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Mtenda Griffin");
    t.lands(P0, "Plains", 1);
    t.set_step(P0, Step::Upkeep);
    assert!(!can_activate(&mut t, P0, g));
    let other = t.graveyard(P0, "Mtenda Griffin");
    assert!(can_activate(&mut t, P0, g));
    t.activate(P0, g, 0, &[obj(other)]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn necromancers_magemark_returns_only_the_creature_without_dying() {
    cr!("614.6", "303.4c", "702.55a");
    ruling!(
        "Necromancer's Magemark",
        "Necromancer's Magemark returns only the creatures. The Auras attached to those creatures fall off and are put in their owner's graveyard."
    );
    ruling!(
        "Necromancer's Magemark",
        "Necromancer's Magemark's effect prevents the creature from ever reaching a graveyard, so abilities such as Haunt never trigger."
    );
    supported("Necromancer's Magemark");
    supported("Blind Hunter");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Necromancer's Magemark", b);
    attach_new(&mut t, P0, "Holy Strength", b);
    destroy(&mut t, b);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Necromancer's Magemark"));
    assert!(t.in_graveyard(P0, "Holy Strength"));
    // Blind Hunter (haunt) goes to the hand: no haunt trigger.
    for magemark in [true, false] {
        let mut t = TestGame::new(2);
        let h = t.battlefield(P0, "Blind Hunter");
        t.battlefield(P1, "Grizzly Bears");
        if magemark {
            attach_new(&mut t, P0, "Necromancer's Magemark", h);
        }
        destroy(&mut t, h);
        assert_eq!(stacked_triggers(&t), usize::from(!magemark), "{magemark}");
        t.resolve_all();
        if magemark {
            assert!(t.in_hand(P0, "Blind Hunter"));
        } else {
            assert!(t.in_exile("Blind Hunter"), "haunting");
        }
    }
}

#[test]
fn storrev_cant_target_cards_put_into_the_graveyard_this_combat() {
    cr!("115.1", "506.1");
    ruling!(
        "Storrev, Devkarin Lich",
        "Storrev's triggered ability can't target any card that was put into your graveyard during this combat phase. This includes creatures and planeswalkers that died from combat damage, creatures and planeswalkers that died for any other reason, and cards that were put there from any other zone."
    );
    supported("Storrev, Devkarin Lich");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Storrev, Devkarin Lich");
    let b = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let old = t.graveyard(P0, "Hill Giant");
    let discarded = t.hand(P0, "Craw Wurm");
    let blocker = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(s, pl(P1)), (b, pl(P1))]);
    // During combat: one creature dies to an effect, a card is discarded.
    destroy(&mut t, elves);
    discard(&mut t, P0, discarded);
    let from = n_asked(&t);
    t.answer_targets(P0, &[obj(old)]);
    block_and_finish(&mut t, P1, &[(blocker, b)]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    let offered = offered(&t, P0, from);
    assert_eq!(offered, vec![obj(old)], "only the card from before combat");
    assert!(t.in_hand(P0, "Hill Giant"));
}

#[test]
fn tasigur_doesnt_target_and_the_opponent_chooses_after_milling() {
    cr!("608.2c", "115.1");
    ruling!(
        "Tasigur, the Golden Fang",
        "Tasigur's last ability doesn't target any card or player. You choose an opponent as the ability resolves and that player chooses a nonland card after you put the top two cards of your library into your graveyard."
    );
    supported("Tasigur, the Golden Fang");
    let mut t = TestGame::new(2);
    let tg = t.battlefield(P0, "Tasigur, the Golden Fang");
    t.lands(P0, "Island", 4);
    t.graveyard(P0, "Hill Giant");
    let shock = t.library_top(P0, "Shock");
    t.library_top(P0, "Forest");
    t.answer_choose_moved(P1, &[obj(shock)]);
    let from = n_asked(&t);
    t.activate(P0, tg, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Shock"), "P1 chose the milled Shock");
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(target_candidates(&t, P0, from).is_empty());
}

#[test]
fn angel_of_flight_alabaster_needs_a_spirit_card_as_the_upkeep_begins() {
    cr!("603.3d", "503.1a");
    ruling!(
        "Angel of Flight Alabaster",
        "The Spirit card must already be in your graveyard when the ability triggers at the beginning of your upkeep. If there is no Spirit card in your graveyard when your upkeep begins, the ability will be removed from the stack with no effect."
    );
    supported("Angel of Flight Alabaster");
    for spirit in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Angel of Flight Alabaster");
        if spirit {
            t.graveyard(P0, "Kami of Ancient Law");
        }
        into_upkeep(&mut t, P0);
        assert_eq!(stacked_triggers(&t), usize::from(spirit));
        // A Spirit card arriving now is too late.
        if !spirit {
            t.graveyard(P0, "Kami of Ancient Law");
        }
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "Kami of Ancient Law"), spirit);
    }
}

#[test]
fn ghost_of_ramirez_targets_only_this_turns_discards_and_mills() {
    cr!("115.1", "700.4");
    ruling!(
        "Ghost of Ramirez DePietro",
        "The middle ability can't target cards that were discarded on previous turns or put into the graveyard from a library on previous turns."
    );
    supported("Ghost of Ramirez DePietro");
    let mut t = TestGame::new(2);
    let ghost = t.battlefield(P0, "Ghost of Ramirez DePietro");
    let old = t.hand(P0, "Hill Giant");
    discard(&mut t, P0, old);
    let old_mill = t.library_top(P1, "Craw Wurm");
    mill(&mut t, P1, 1);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    let new = t.hand(P0, "Grizzly Bears");
    discard(&mut t, P0, new);
    let from = n_asked(&t);
    t.answer_targets(P0, &[obj(now(&t, new))]);
    attack_with(&mut t, &[(ghost, pl(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    let offered = offered(&t, P0, from);
    assert!(!offered.contains(&obj(now(&t, old))));
    assert!(!offered.contains(&obj(now(&t, old_mill))));
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn colfenor_compares_to_the_dying_creatures_last_toughness() {
    cr!("603.10a", "115.1");
    ruling!(
        "Colfenor, the Last Yew",
        "Use the toughness of the dying creature as it last existed on the battlefield and the toughness of the creature cards in your graveyard to determine which of those cards are legal targets."
    );
    supported("Colfenor, the Last Yew");
    for pumped in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Colfenor, the Last Yew");
        let g = t.graveyard(P0, "Hill Giant");
        let b = t.battlefield(P0, "Grizzly Bears");
        if pumped {
            pump(&mut t, b, 0, 3);
        }
        let from = n_asked(&t);
        t.answer_targets(P0, &[obj(g)]);
        destroy(&mut t, b);
        assert_eq!(offered(&t, P0, from).contains(&obj(g)), pumped);
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "Hill Giant"), pumped);
    }
}

#[test]
fn gleancrawler_returns_creatures_that_died_before_it_entered() {
    cr!("400.7", "700.4");
    ruling!(
        "Gleancrawler",
        "When this ability resolves, it returns all creature cards currently in your graveyard that were put there directly from the battlefield sometime during the turn. This includes cards that were put into your graveyard before Gleancrawler entered the battlefield."
    );
    supported("Gleancrawler");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, b);
    let d = t.hand(P0, "Hill Giant");
    discard(&mut t, P0, d);
    t.battlefield(P0, "Gleancrawler");
    end_step(&mut t, P0);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn yomiji_returns_any_legendary_permanent_even_dying_with_it() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Yomiji, Who Bars the Way",
        "Yomiji's ability triggers even if Yomiji is going to the graveyard at the same time as the other legendary permanent."
    );
    ruling!(
        "Yomiji, Who Bars the Way",
        "Yomiji's ability triggers whenever any legendary permanent is put into a graveyard from the battlefield, not just legendary creatures."
    );
    supported("Yomiji, Who Bars the Way");
    let mut t = TestGame::new(2);
    let y = t.battlefield(P0, "Yomiji, Who Bars the Way");
    let r = t.battlefield(P1, "Ravos, Soultender");
    destroy_together(&mut t, &[y, r]);
    t.resolve_all();
    assert!(t.in_hand(P1, "Ravos, Soultender"), "to its owner's hand");
    assert!(t.in_graveyard(P0, "Yomiji, Who Bars the Way"));
    // A legendary artifact.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Yomiji, Who Bars the Way");
    let m = t.battlefield(P0, "Aetherworks Marvel");
    destroy(&mut t, m);
    t.resolve_all();
    assert!(t.in_hand(P0, "Aetherworks Marvel"));
}

#[test]
fn fell_shepherd_can_target_the_creature_it_sacrifices() {
    cr!("601.2c", "601.2h", "608.2b");
    ruling!(
        "Fell Shepherd",
        "You can choose a creature as the target of the last ability and sacrifice that same creature to activate the ability."
    );
    supported("Fell Shepherd");
    let mut t = TestGame::new(2);
    let fs = t.battlefield(P0, "Fell Shepherd");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.answer_choose(P0, &[obj(b)]);
    let ab = t.activate(P0, fs, 0, &[obj(b)]).unwrap();
    assert!(ab.is_some());
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert_eq!(t.pt(fs), (8, 6));
}

#[test]
fn meren_puts_cheap_cards_onto_the_battlefield_not_the_hand() {
    cr!("608.2c", "122.1");
    ruling!(
        "Meren of Clan Nel Toth",
        "You can’t choose to put the creature card into your hand if its mana value is less than or equal to the number of experience counters you have as the ability resolves."
    );
    supported("Meren of Clan Nel Toth");
    for (card, on_bf) in [("Llanowar Elves", true), ("Grizzly Bears", false)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Meren of Clan Nel Toth");
        give_player_counters(&mut t, P0, "experience", 1);
        t.graveyard(P0, card);
        end_step(&mut t, P0);
        assert_eq!(t.named_on_battlefield(card).len() == 1, on_bf, "{card}");
        assert_eq!(t.in_hand(P0, card), !on_bf, "{card}");
    }
}

#[test]
fn tortured_existence_cant_return_the_card_discarded_for_it() {
    cr!("601.2c", "601.2h", "602.2b");
    ruling!(
        "Tortured Existence",
        "You can’t return the same card you are discarding. This is because you pick the target before paying the costs."
    );
    supported("Tortured Existence");
    let mut t = TestGame::new(2);
    let te = t.battlefield(P0, "Tortured Existence");
    t.lands(P0, "Swamp", 1);
    let g = t.graveyard(P0, "Hill Giant");
    let b = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(b)]);
    let from = n_asked(&t);
    t.activate(P0, te, 0, &[obj(g)]).unwrap();
    assert_eq!(offered(&t, P0, from), vec![obj(g)]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn spawnbed_protector_makes_scions_unless_its_chosen_target_is_illegal() {
    cr!("608.2b", "115.4");
    ruling!(
        "Spawnbed Protector",
        "You do not have to choose a target for Spawnbed Protector's ability, and if you don't, you will still create Eldrazi Scion tokens. If you do choose a target but the target Eldrazi creature card is illegal as Spawnbed Protector's ability tries to resolve, it won't resolve and none of its effects will happen. You won't create Eldrazi Scion tokens."
    );
    supported("Spawnbed Protector");
    // No target.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spawnbed Protector");
    t.answer_targets(P0, &[]);
    end_step(&mut t, P0);
    assert_eq!(tokens(&t, P0).len(), 2);
    // A target that leaves in response.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spawnbed Protector");
    let e = t.graveyard(P0, "Spawnbed Protector");
    t.answer_targets(P0, &[obj(e)]);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(stacked_triggers(&t), 1);
    exile(&mut t, e);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 0);
    // Positive control: a legal target is returned, with the tokens.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spawnbed Protector");
    let e = t.graveyard(P0, "Spawnbed Protector");
    t.answer_targets(P0, &[obj(e)]);
    end_step(&mut t, P0);
    assert!(t.in_hand(P0, "Spawnbed Protector"));
    assert_eq!(tokens(&t, P0).len(), 2);
}

/// Liliana, the Necromancer with 7 loyalty for P0.
fn necromancer(t: &mut TestGame) -> ObjectId {
    let l = t.battlefield(P0, "Liliana, the Necromancer");
    loyalty(t, l, 7);
    l
}

#[test]
fn liliana_the_necromancer_ultimate_targets() {
    cr!("608.2b", "115.4");
    ruling!(
        "Liliana, the Necromancer",
        "You may activate Liliana’s last ability without choosing any target creatures. You’ll just put up to two creature cards onto the battlefield from graveyards. However, if you choose any targets and all of those targets become illegal before the ability tries to resolve, the ability won’t resolve and you won’t put any cards onto the battlefield."
    );
    supported("Liliana, the Necromancer");
    // No targets.
    let mut t = TestGame::new(2);
    let l = necromancer(&mut t);
    let g = t.graveyard(P1, "Hill Giant");
    t.answer_targets(P0, &[]);
    t.answer_choose(P0, &[obj(g)]);
    t.activate(P0, l, 2, &[]).unwrap();
    t.resolve_all();
    let giant = t.named_on_battlefield("Hill Giant");
    assert_eq!(giant.len(), 1);
    assert_eq!(t.obj_now(giant[0]).controller, P0);
    // The only target leaves.
    let mut t = TestGame::new(2);
    let l = necromancer(&mut t);
    let g = t.graveyard(P1, "Hill Giant");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(b)]);
    t.answer_choose(P0, &[obj(g)]);
    t.activate(P0, l, 2, &[]).unwrap();
    move_to(&mut t, b, Zone::Hand(P1));
    t.resolve_all();
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn liliana_the_necromancer_reanimated_cards_are_exiled_when_she_leaves_the_game() {
    cr!("800.4a");
    ruling!(
        "Liliana, the Necromancer",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, any permanents you control from Liliana’s third ability that came from other players’ graveyards are exiled."
    );
    let mut t = TestGame::new(3);
    let l = necromancer(&mut t);
    let g = t.graveyard(P1, "Hill Giant");
    t.answer_targets(P0, &[]);
    t.answer_choose(P0, &[obj(g)]);
    t.activate(P0, l, 2, &[]).unwrap();
    t.resolve_all();
    let giant = t.named_on_battlefield("Hill Giant");
    assert_eq!(giant.len(), 1);
    t.g.lose_game(P0);
    t.settle();
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
    assert!(t.in_exile("Hill Giant"));
}

#[test]
fn random_cards_are_chosen_on_resolution() {
    cr!("608.2c", "603.3");
    ruling!(
        "Deadbridge Chant",
        "The card is chosen at random as the ability resolves. If any player responds to the ability, that player won't yet know what card will be chosen."
    );
    ruling!(
        "Charmbreaker Devils",
        "The instant or sorcery card returned to your hand is chosen at random as Charmbreaker Devils's first ability resolves. If any player responds to the ability, that player won't yet know what card will be returned."
    );
    supported("Deadbridge Chant");
    supported("Charmbreaker Devils");
    // The graveyards are empty as the upkeep begins; the only card arrives in response.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Deadbridge Chant");
    into_upkeep(&mut t, P0);
    assert_eq!(stacked_triggers(&t), 1);
    t.graveyard(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Charmbreaker Devils");
    into_upkeep(&mut t, P0);
    assert_eq!(stacked_triggers(&t), 1);
    t.graveyard(P0, "Shock");
    t.resolve_all();
    assert!(t.in_hand(P0, "Shock"));
}

#[test]
fn mirkwood_elk_gains_zero_for_an_elf_card_without_power() {
    cr!("208.3", "107.1b");
    ruling!(
        "Mirkwood Elk",
        "In the rare case that the Elf card you return to your hand doesn't have power, you'll gain 0 life."
    );
    supported("Mirkwood Elk");
    for (card, life) in [("Elvish Promenade", 20), ("Llanowar Elves", 21)] {
        let mut t = TestGame::new(2);
        let c = t.graveyard(P0, card);
        t.answer_targets(P0, &[obj(c)]);
        enter(&mut t, P0, "Mirkwood Elk");
        t.resolve_all();
        assert!(t.in_hand(P0, card));
        assert_eq!(t.life(P0), life, "{card}");
    }
}

#[test]
fn intervening_if_graveyard_counts_are_checked_twice() {
    cr!("603.4");
    ruling!(
        "Oversold Cemetery",
        "Oversold Cemetery’s ability will check your graveyard as your upkeep begins, before any player has the opportunity to take any actions. If you have fewer than four creature cards in your graveyard at that time, the ability won’t trigger at all. If the ability does trigger, it will check again as it tries to resolve. If you have fewer than four creature cards in your graveyard at that point, the ability won’t have any effect."
    );
    ruling!(
        "Tersa Lightshatter",
        "Tersa Lightshatter’s last ability checks your graveyard at the moment it would trigger to see if you have seven or more cards in your graveyard. If you don’t, the ability won’t trigger at all. If it does trigger, the ability will check again as it tries to resolve. If you don’t have seven or more cards in your graveyard at that time, the ability won’t resolve and none of its effects will happen."
    );
    supported("Oversold Cemetery");
    supported("Tersa Lightshatter");
    let names = ["Grizzly Bears", "Hill Giant", "Craw Wurm", "Llanowar Elves"];
    // Oversold Cemetery: (creature cards, remove one in response) -> returned?
    for (n, respond, returned) in [(3, false, false), (4, true, false), (4, false, true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Oversold Cemetery");
        let cards: Vec<_> = names[..n].iter().map(|c| t.graveyard(P0, c)).collect();
        yes(&mut t, P0);
        t.answer_targets(P0, &[obj(cards[0])]);
        into_upkeep(&mut t, P0);
        assert_eq!(stacked_triggers(&t), usize::from(n == 4), "{n}");
        if respond {
            exile(&mut t, cards[3]);
        }
        t.resolve_all();
        assert_eq!(t.in_hand(P0, names[0]), returned, "{n} {respond}");
    }
    // Tersa: (cards, remove one in response) -> a card exiled?
    for (n, respond, exiled) in [(6, false, false), (7, true, false), (7, false, true)] {
        let mut t = TestGame::new(2);
        let tersa = t.battlefield(P0, "Tersa Lightshatter");
        let cards: Vec<_> = (0..n).map(|_| t.graveyard(P0, "Shock")).collect();
        attack_with(&mut t, &[(tersa, pl(P1))]);
        assert_eq!(stacked_triggers(&t), usize::from(n == 7), "{n}");
        if respond {
            move_to(&mut t, cards[0], Zone::Hand(P0));
        }
        t.resolve_all();
        let in_exile = t.g.exile.len();
        assert_eq!(in_exile > 0, exiled, "{n} {respond}");
    }
}

#[test]
fn forbidden_crypt_replaces_graveyards_and_draws() {
    cr!("614.1a", "614.6", "104.3c");
    ruling!(
        "Forbidden Crypt",
        "It will replace itself being sent to the graveyard, and exile itself."
    );
    ruling!(
        "Forbidden Crypt",
        "The second ability is a replacement effect, not a triggered ability. The card never gets to the graveyard."
    );
    ruling!(
        "Forbidden Crypt",
        "The “if you can’t” in the first ability refers to putting the card in your hand, as well as being unable to choose a card. Thus, if you can’t choose a card during resolution, you lose the game."
    );
    supported("Forbidden Crypt");
    // Itself.
    let mut t = TestGame::new(2);
    let fc = t.battlefield(P0, "Forbidden Crypt");
    destroy(&mut t, fc);
    assert!(t.in_exile("Forbidden Crypt"));
    // A dying creature never reaches the graveyard: Enduring Renewal doesn't trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forbidden Crypt");
    t.battlefield(P0, "Enduring Renewal");
    let b = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, b);
    assert_eq!(stacked_triggers(&t), 0);
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.graveyard_size(P0), 0);
    // Drawing with a card in the graveyard returns it (positive control) …
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forbidden Crypt");
    let g = t.custom(P0, (*mtg_engine::card::card("Hill Giant")).clone(), Zone::Graveyard(P0));
    let lib = t.library_size(P0);
    t.answer_choose(P0, &[obj(g)]);
    t.g.draw_cards(P0, 1);
    t.settle();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(t.library_size(P0), lib);
    assert!(!t.has_lost(P0));
    // … and with an empty graveyard, P0 loses.
    t.g.draw_cards(P0, 1);
    t.settle();
    assert!(t.has_lost(P0));
}

#[test]
fn skyship_weatherlight_returns_to_the_owner_and_may_find_nothing() {
    cr!("701.23b", "108.3");
    ruling!(
        "Skyship Weatherlight",
        "The card goes to its owner’s hand as per the errata."
    );
    ruling!(
        "Skyship Weatherlight",
        "You may choose to not find anything when searching your library."
    );
    supported("Skyship Weatherlight");
    // Find nothing.
    let mut t = TestGame::new(2);
    t.library_top(P0, "Grizzly Bears");
    let lib = t.library_size(P0);
    t.answer_choose(P0, &[]);
    enter(&mut t, P0, "Skyship Weatherlight");
    t.resolve_all();
    assert_eq!(t.library_size(P0), lib);
    assert!(!t.in_exile("Grizzly Bears"));
    // Exile a card, then P1 controls the Skyship and activates it: the card goes to
    // P0's hand.
    let mut t = TestGame::new(2);
    let b = t.library_top(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(b)]);
    let s = enter(&mut t, P0, "Skyship Weatherlight");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    gain_control(&mut t, P1, s);
    t.lands(P1, "Wastes", 4);
    t.activate(P1, s, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(!t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn citanul_flute_x_can_be_zero_and_finding_is_optional() {
    cr!("107.3", "701.23b");
    ruling!("Citanul Flute", "X can be zero if you want.");
    ruling!(
        "Citanul Flute",
        "You look in your library but you do not have to choose a creature even if you have one that meets the criteria."
    );
    supported("Citanul Flute");
    // X = 0 finds a mana value 0 creature.
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Citanul Flute");
    let m = t.library_top(P0, "Memnite");
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(0));
    t.answer_choose(P0, &[obj(m)]);
    t.activate(P0, f, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Memnite"));
    // Choosing nothing although a creature qualifies.
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Citanul Flute");
    t.lands(P0, "Wastes", 2);
    t.library_top(P0, "Grizzly Bears");
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(2));
    t.answer_choose(P0, &[]);
    t.activate(P0, f, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), 0);
}
