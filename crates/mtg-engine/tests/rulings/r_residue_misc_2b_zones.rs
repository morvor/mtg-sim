//! Item `rulings-residue-misc-2`, pass B — zone changes, copies and choices made as
//! effects resolve: what a resolving effect finds as it starts (CR 608.2c), tokens and
//! Auras that can't be attached (303.4i), copies of cards and of permanents (707.2,
//! 707.10), objects entering together (614.12), linked abilities with several answers
//! (607.5), and face-down permanents (708).

use crate::r_residue_misc_2b_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{counters, CardType};
use mtg_engine::*;

// --- Liberated Livestock, Smoke Spirits' Aid ---------------------------------------------------

#[test]
fn liberated_livestock_extra_tokens_may_get_auras_too() {
    cr!("614.1a", "303.4f");
    ruling!(
        "Liberated Livestock",
        "If Liberated Livestock's ability causes more than the expected number of tokens to be created (due to an effect such as that of Doubling Season), you may put an Aura card from your hand and/or graveyard onto the battlefield attached to each of the additional tokens as well."
    );
    supported("Liberated Livestock");
    supported("Doubling Season");
    let mut t = TestGame::new(2);
    let ll = t.battlefield(P0, "Liberated Livestock");
    t.battlefield(P0, "Doubling Season");
    for _ in 0..6 {
        t.graveyard(P0, "Holy Strength");
    }
    for _ in 0..6 {
        t.answer_yes(P0, true);
    }
    destroy(&mut t, ll);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 6);
    let hs = t.named_on_battlefield("Holy Strength");
    assert_eq!(hs.len(), 6, "{}", t.dump_log());
    let mut hosts: Vec<ObjectId> = hs
        .iter()
        .filter_map(|h| t.obj_now(*h).attached_to.and_then(|e| e.object()))
        .collect();
    hosts.sort();
    hosts.dedup();
    assert_eq!(hosts.len(), 6, "one on each token");
}

#[test]
fn smoke_spirits_aid_makes_no_blessing_that_cant_be_attached() {
    cr!("303.4f", "702.16c", "111.1");
    ruling!(
        "Smoke Spirits' Aid",
        "If a Smoke Blessing that would be created by Smoke Spirits' Aid can't be attached to that creature for any reason, the token isn't created."
    );
    supported("Azorius First-Wing");
    let mut t = TestGame::new(2);
    let wing = t.battlefield(P1, "Azorius First-Wing");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let s = t.hand(P0, "Smoke Spirits' Aid");
    t.cast(P0, s).x(2).targets(&[obj(wing), obj(bears)]).go();
    t.resolve_all();
    let blessings = t.named_on_battlefield("Smoke Blessing");
    assert_eq!(blessings.len(), 1, "{}", t.dump_log());
    assert_eq!(
        t.obj_now(blessings[0]).attached_to.and_then(|e| e.object()),
        Some(bears)
    );
}

// --- Dance of the Manse -------------------------------------------------------------------------

#[test]
fn dance_of_the_manse_four_four_x_zero_and_later_pt_setting() {
    cr!("202.3e", "613.4b", "613.7");
    ruling!(
        "Dance of the Manse",
        "An artifact creature or enchantment creature returned this way will be a 4/4 creature."
    );
    ruling!(
        "Dance of the Manse",
        "If a card in a player's graveyard has {X} in its mana cost, that X is considered to be 0."
    );
    ruling!(
        "Dance of the Manse",
        "If another effect causes one of the returned permanents to become a creature and sets its power and toughness as it does so, that creature will have that power and toughness; it won't be 4/4. Notably, crewing a Vehicle does not set its power and toughness, so a Vehicle will remain a 4/4 creature if you crew it."
    );
    supported("Dance of the Manse");
    supported("Ensoul Artifact");
    supported("Chalice of the Void");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let thopter = t.graveyard(P0, "Ornithopter");
    let copter = t.graveyard(P0, "Smuggler's Copter");
    let chalice = t.graveyard(P0, "Chalice of the Void");
    let stone = t.graveyard(P0, "Mind Stone");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 6);
    let d = t.hand(P0, "Dance of the Manse");
    t.cast(P0, d)
        .x(6)
        .targets(&[obj(thopter), obj(copter), obj(chalice), obj(stone)])
        .go();
    t.resolve_all();
    let now = |t: &TestGame, name: &str| t.named_on_battlefield(name)[0];
    // Ornithopter (an artifact creature, 0/2) and Chalice ({X}{X}: mana value 0) are 4/4s.
    assert_eq!(t.pt(now(&t, "Ornithopter")), (4, 4));
    let ch = now(&t, "Chalice of the Void");
    assert_eq!(t.pt(ch), (4, 4));
    // Crewed, the Copter is still a 4/4.
    let cop = now(&t, "Smuggler's Copter");
    assert_eq!(t.pt(cop), (4, 4));
    let crew = t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(crew)]);
    activate_containing(&mut t, P0, cop, "Crew").expect("crew");
    t.resolve_all();
    assert!(t.obj_now(cop).is(CardType::Creature));
    assert_eq!(t.pt(cop), (4, 4));
    // Ensoul Artifact on the Mind Stone: base 5/5 (later timestamp) instead.
    let ms = now(&t, "Mind Stone");
    t.lands(P0, "Island", 2);
    let ensoul = t.hand(P0, "Ensoul Artifact");
    t.cast(P0, ensoul).target(obj(ms)).go();
    t.resolve_all();
    assert_eq!(t.pt(ms), (5, 5), "{}", t.dump_log());
}

// --- Indomitable Creativity -------------------------------------------------------------------

#[test]
fn indomitable_creativity_counts_only_what_it_destroyed() {
    cr!("701.8a", "608.2c", "614.6");
    ruling!(
        "Indomitable Creativity",
        "If a player controls no permanents destroyed this way, that player reveals no cards from their library and doesn't shuffle it."
    );
    ruling!(
        "Indomitable Creativity",
        "If an artifact or creature is targeted but not destroyed (perhaps because it gained indestructible or became an illegal target), it doesn't count as one of the artifacts or creatures destroyed this way. An artifact or creature that's destroyed but put into a zone other than a graveyard (for example, due to the effect of Rest in Peace) does count."
    );
    supported("Indomitable Creativity");
    supported("Rest in Peace");
    // P1's Darksteel Citadel (indestructible) isn't destroyed: P1 reveals nothing and
    // doesn't shuffle. P0's Ornithopter is destroyed (and exiled by Rest in Peace): P0
    // reveals until the Craw Wurm.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rest in Peace");
    let thopter = t.battlefield(P0, "Ornithopter");
    let citadel = t.battlefield(P1, "Darksteel Citadel");
    t.library_top(P0, "Craw Wurm");
    t.library_top(P0, "Island");
    t.library_top(P1, "Sol Ring");
    t.lands(P0, "Mountain", 5);
    let c = t.hand(P0, "Indomitable Creativity");
    let from = t.g.turn_events.len();
    t.cast(P0, c)
        .x(2)
        .targets(&[obj(thopter), obj(citadel)])
        .go();
    t.resolve_all();
    assert!(t.in_exile("Ornithopter"));
    assert!(t.on_battlefield(citadel));
    assert_eq!(n_named(&t, P0, "Craw Wurm"), 1, "{}", t.dump_log());
    assert!(t.named_on_battlefield("Sol Ring").is_empty());
    assert_eq!(shuffles_since(&t, P1, from), 0);
    assert_eq!(shuffles_since(&t, P0, from), 1);
}

#[test]
fn indomitable_creativity_with_no_artifact_or_creature_reveals_the_whole_library() {
    cr!("701.20a", "608.2c");
    ruling!(
        "Indomitable Creativity",
        "If a player's library no longer contains an artifact or creature card when instructed to reveal cards, that player reveals the entire library, exiles no cards, and then shuffles it."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // P1's library: lands only.
    t.g.players[P1.idx()].library.clear();
    for _ in 0..3 {
        t.library_top(P1, "Island");
    }
    t.lands(P0, "Mountain", 4);
    let c = t.hand(P0, "Indomitable Creativity");
    let from = t.g.turn_events.len();
    t.cast(P0, c).x(1).targets(&[obj(bears)]).go();
    t.resolve_all();
    assert_eq!(t.library_size(P1), 3, "{}", t.dump_log());
    assert!(!t.in_exile("Island"));
    assert_eq!(shuffles_since(&t, P1, from), 1);
}

// --- Hour of Eternity, Doppelgang, Replenish ----------------------------------------------------

#[test]
fn hour_of_eternity_tokens_copy_only_the_printed_card() {
    cr!("707.2", "707.9b", "400.7");
    ruling!(
        "Hour of Eternity",
        "Each token copies exactly what was printed on the original card and nothing else, except the characteristics it specifically modifies. It doesn't copy any information about the object the card was before it was put into your graveyard."
    );
    supported("Hour of Eternity");
    let mut t = TestGame::new(2);
    // A Grizzly Bears with +1/+1 counters that was enchanted, then died.
    let bears = t.battlefield(P0, "Grizzly Bears");
    plus1(&mut t, bears, 2);
    kill(&mut t, bears);
    let card = t.g.current(bears);
    t.lands(P0, "Island", 3);
    t.lands(P0, "Wastes", 2);
    let h = t.hand(P0, "Hour of Eternity");
    t.cast(P0, h).x(1).targets(&[obj(card)]).go();
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    let o = t.obj_now(toks[0]);
    assert_eq!(o.chars.name, "Grizzly Bears");
    assert!(o.chars.has_subtype("Zombie") && !o.chars.has_subtype("Bear"));
    assert_eq!(t.counters(toks[0], counters::PLUS1), 0);
    assert_eq!(t.pt(toks[0]), (4, 4));
}

#[test]
fn doppelgang_copies_trigger_and_enter_with_their_counters() {
    cr!("707.2", "614.1c", "603.6a");
    ruling!(
        "Doppelgang",
        "Any enters-the-battlefield abilities of the copied permanent will trigger when the tokens enter the battlefield. Any \"as [this permanent] enters the battlefield\" or \"[this permanent] enters the battlefield with\" abilities of the permanent will also work."
    );
    supported("Doppelgang");
    let mut t = TestGame::new(2);
    let fk = t.battlefield(P0, "Flametongue Kavu");
    let tri = t.battlefield(P0, "Triskelion");
    let wurms: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P1, "Craw Wurm")).collect();
    for w in &wurms {
        t.answer_targets(P0, &[obj(*w)]);
    }
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 6);
    let d = t.hand(P0, "Doppelgang");
    t.cast(P0, d).x(2).targets(&[obj(fk), obj(tri)]).go();
    t.resolve();
    t.settle();
    let toks = tokens(&t, P0);
    let kavus: Vec<&ObjectId> = toks
        .iter()
        .filter(|id| t.obj_now(**id).chars.name == "Flametongue Kavu")
        .collect();
    let tris: Vec<&ObjectId> = toks
        .iter()
        .filter(|id| t.obj_now(**id).chars.name == "Triskelion")
        .collect();
    assert_eq!((kavus.len(), tris.len()), (2, 2), "{}", t.dump_log());
    for tr in tris {
        assert_eq!(t.counters(*tr, counters::PLUS1), 3);
    }
    assert_eq!(triggers_of(&t, "Flametongue Kavu").len(), 2);
}

#[test]
fn replenish_returns_only_what_was_there_as_it_began() {
    cr!("608.2c", "614.12");
    ruling!(
        "Replenish",
        "It determines what enchantments to return when its resolution starts. If more enchantments go to the graveyard during the resolution, those additional enchantments are ignored."
    );
    supported("Replenish");
    // An enchantment that has its controller discard a card as it enters: the discarded
    // enchantment card goes to the graveyard during Replenish's resolution and stays there.
    let charm = custom_card(
        "Discarding Charm",
        "Enchantment",
        "{1}",
        None,
        "As this enchantment enters, discard a card.",
    );
    let mut t = TestGame::new(2);
    t.custom(P0, charm, Zone::Graveyard(P0));
    let anthem = t.hand(P0, "Glorious Anthem");
    t.answer_choose(P0, &[obj(anthem)]);
    t.lands(P0, "Plains", 4);
    let r = t.hand(P0, "Replenish");
    t.cast(P0, r).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Discarding Charm").len(), 1);
    assert!(t.in_graveyard(P0, "Glorious Anthem"), "{}", t.dump_log());
    assert!(t.named_on_battlefield("Glorious Anthem").is_empty());
}

// --- Repeated Reverberation, Devouring Hellion, The Elderspell ----------------------------------

#[test]
fn repeated_reverberation_copies_are_kicked_if_the_original_was() {
    cr!("707.10", "702.33d");
    ruling!(
        "Repeated Reverberation",
        "You can’t choose to pay any additional costs for the copies. However, effects based on any additional costs that were paid for the original spell are copied as though those same costs were paid for the copies too."
    );
    supported("Burst Lightning");
    for kicked in [true, false] {
        let mut t = TestGame::new(2);
        let rr = cast_card(&mut t, P0, "Repeated Reverberation", &[]);
        let _ = rr;
        t.resolve_all();
        t.lands(P0, "Mountain", 5);
        let bl = t.hand(P0, "Burst Lightning");
        let from = t.asked().len();
        t.cast(P0, bl).kicked(kicked).target(pl(P1)).go();
        t.resolve_all();
        let each = if kicked { 4 } else { 2 };
        assert_eq!(t.life(P1), 20 - 3 * each, "kicked: {kicked}");
        // No kicker decision was asked for the copies.
        let kicker_asks = asked_since(&t, from)
            .iter()
            .filter(|(_, d)| matches!(d, Decision::OptionalCost { .. }))
            .count();
        assert!(kicker_asks <= 1);
    }
}

#[test]
fn devouring_hellion_sacrifices_as_it_enters_with_no_actions_between() {
    cr!("614.12", "614.1c", "117.1");
    ruling!(
        "Devouring Hellion",
        "You choose how many and which creatures and planeswalkers to sacrifice for Devouring Hellion’s ability while it’s entering the battlefield. No player may take actions between the time you choose which permanents to sacrifice and the time Devouring Hellion is on the battlefield with those counters."
    );
    supported("Devouring Hellion");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Jace Beleren");
    t.lands(P0, "Mountain", 3);
    let h = t.hand(P0, "Devouring Hellion");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.cast(P0, h).go();
    let from = t.asked().len();
    t.resolve();
    let asked = asked_since(&t, from);
    let choice = asked
        .iter()
        .position(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .expect("asked which to sacrifice");
    assert!(!asked[..choice]
        .iter()
        .chain(asked[choice..].iter())
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    let hel = t.g.current(h);
    assert!(t.on_battlefield(hel));
    assert_eq!(t.counters(hel, counters::PLUS1), 4);
}

#[test]
fn the_elderspell_chooses_the_planeswalker_for_counters_as_it_resolves() {
    cr!("601.2c", "608.2c");
    ruling!(
        "The Elderspell",
        "You choose the target planeswalkers to destroy as you cast The Elderspell, but you don’t choose a planeswalker to receive loyalty counters until The Elderspell is resolving."
    );
    supported("The Elderspell");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Jace Beleren");
    t.answer_targets(P0, &[obj(a)]);
    let spell = cast_card(&mut t, P0, "The Elderspell", &[]);
    let _ = spell;
    // P0's planeswalker arrives after the spell was cast.
    let mine = pw(&mut t, P0, "Ajani Goldmane", 4);
    t.answer_choose(P0, &[obj(mine)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Jace Beleren"));
    assert_eq!(t.counters(mine, counters::LOYALTY), 6, "{}", t.dump_log());
}

// --- Arsenal Thresher, Bruna, Reckless Crew -----------------------------------------------------

#[test]
fn arsenal_thresher_may_reveal_an_artifact_entering_with_it() {
    cr!("614.12", "614.1c");
    ruling!(
        "Arsenal Thresher",
        "If Arsenal Thresher and another artifact are entering from your hand at the same time, you may reveal that other artifact for Arsenal Thresher. If that artifact is a second Arsenal Thresher, you may reveal each Arsenal Thresher for the other."
    );
    supported("Arsenal Thresher");
    for other in ["Ornithopter", "Arsenal Thresher"] {
        let mut t = TestGame::new(2);
        let th = t.hand(P0, "Arsenal Thresher");
        let o = t.hand(P0, other);
        t.answer_choose(P0, &[obj(o)]);
        t.answer_choose(P0, &[obj(th)]);
        let moves = [th, o]
            .iter()
            .map(|c| mtg_engine::replacement::MoveEv {
                obj: *c,
                to: Zone::Battlefield,
                pos: mtg_engine::ability::LibraryPosition::Top,
                cause: mtg_engine::events::MoveCause::Effect,
                by: Some(P0),
                etb: Default::default(),
                source: None,
            })
            .collect();
        let from = t.asked().len();
        t.g.move_objects(moves);
        t.g.flush_events();
        let _ = from;
        t.settle();
        let th = t.g.current(th);
        assert_eq!(
            t.counters(th, counters::PLUS1),
            1,
            "{other}: {}",
            t.dump_log()
        );
        if other == "Arsenal Thresher" {
            assert_eq!(t.counters(t.g.current(o), counters::PLUS1), 1);
        }
    }
}

#[test]
fn bruna_attaches_auras_before_blockers_are_declared() {
    cr!("508.1m", "509.1");
    ruling!(
        "Bruna, Light of Alabaster",
        "All of these Auras will become attached to Bruna before combat damage is dealt. If Bruna is attacking, these Auras become attached before blockers are declared."
    );
    supported("Bruna, Light of Alabaster");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let b = t.battlefield(P0, "Bruna, Light of Alabaster");
    let hs = t.hand(P0, "Holy Strength");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(hs)]);
    attack_with(&mut t, &[(b, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    let hs = t.named_on_battlefield("Holy Strength");
    assert_eq!(hs.len(), 1, "{}", t.dump_log());
    assert_eq!(
        t.obj_now(hs[0]).attached_to.and_then(|e| e.object()),
        Some(b)
    );
}

#[test]
fn reckless_crew_counts_as_it_resolves_and_attaches_together() {
    cr!("608.2h", "701.3a");
    ruling!(
        "Reckless Crew",
        "Count the number of Vehicles and Equipment you control as Reckless Crew resolves to determine how many Dwarf Berserkers to create."
    );
    ruling!(
        "Reckless Crew",
        "All Equipment you decide to attach to tokens become attached at the same time."
    );
    supported("Reckless Crew");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let saw = t.battlefield(P0, "Bone Saw");
    let copter = t.battlefield(P0, "Smuggler's Copter");
    t.lands(P0, "Mountain", 4);
    let rc = t.hand(P0, "Reckless Crew");
    t.cast(P0, rc).go();
    // A second Equipment arrives before it resolves: three tokens.
    let split = t.battlefield(P0, "Bonesplitter");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(saw)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(split)]);
    let from = t.g.turn_events.len();
    t.resolve_all();
    let dwarves = with_subtype(&t, P0, "Dwarf");
    assert_eq!(dwarves.len(), 3, "{}", t.dump_log());
    let _ = copter;
    // Both attached by the same event batch: attached at once.
    let attach_events = t.g.turn_events[from..]
        .iter()
        .filter(|e| format!("{e:?}").starts_with("Attached"))
        .count();
    assert!(attach_events >= 2);
    for e in [saw, split] {
        let host = t.obj_now(e).attached_to.and_then(|x| x.object());
        assert!(
            host.is_some_and(|h| dwarves.contains(&h)),
            "{}",
            t.dump_log()
        );
    }
}

// --- Nahiri, Forged in Fury ------------------------------------------------------------------

#[test]
fn nahiri_triggers_once_per_equipped_attacker_and_equipment_may_be_paid_for() {
    cr!("508.1", "603.2", "601.2f");
    ruling!(
        "Nahiri, Forged in Fury",
        "An \"equipped creature you control\" is any creature you control with one or more Equipment attached to it."
    );
    ruling!(
        "Nahiri, Forged in Fury",
        "Nahiri's last ability will trigger once for each equipped creature you control that attacks, no matter how many Equipment are attached to any of them."
    );
    ruling!(
        "Nahiri, Forged in Fury",
        "If you cast an Equipment spell this way, you may choose to pay its mana cost if you want to."
    );
    supported("Nahiri, Forged in Fury");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Nahiri, Forged in Fury");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let plain = t.battlefield(P0, "Savannah Lions");
    crate::r_s06_common::attach_new(&mut t, P0, "Bonesplitter", bears);
    crate::r_s06_common::attach_new(&mut t, P0, "Bone Saw", bears);
    crate::r_s06_common::attach_new(&mut t, P0, "Bone Saw", elves);
    let sword = t.library_top(P0, "Vulshok Morningstar");
    t.library_top(P0, "Island");
    attack_with(&mut t, &[(bears, pl(P1)), (elves, pl(P1)), (plain, pl(P1))]);
    assert_eq!(triggers_of(&t, "Nahiri, Forged in Fury").len(), 2);
    t.resolve_all();
    // The Morningstar, exiled, is cast in the main phase paying its mana cost.
    t.advance_to(P0, Step::PostcombatMain);
    let sword = t.g.current(sword);
    assert_eq!(t.zone(sword), Zone::Exile);
    t.lands(P0, "Wastes", 2);
    t.cast(P0, sword).method(CastMethod::Normal).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Vulshok Morningstar").len(), 1);
    assert_eq!(tapped_lands(&t, P0), 2);
}

// --- Cloud, Midgar Mercenary ---------------------------------------------------------------------

#[test]
fn cloud_doubled_imprint_gives_two_exiled_cards_whose_names_both_count() {
    cr!("607.5", "603.2d");
    ruling!(
        "Cloud, Midgar Mercenary",
        "In some cases involving linked abilities, an ability requires information about \"the exiled card.\" When this happens, the ability gets multiple answers. If these answers are being used to determine the value of a variable, the sum is used. For example, if Strata Scythe's imprint ability (\"When this Equipment enters, search your library for a land card, exile it, then shuffle.\") triggers twice (probably because it somehow entered attached to Cloud), up to two total land cards can be exiled. Strata Scythe's last ability will give the equipped creature +1/+1 for each land on the battlefield with the same name as either of the exiled cards."
    );
    supported("Cloud, Midgar Mercenary");
    let mut t = TestGame::new(2);
    let cloud = t.battlefield(P0, "Cloud, Midgar Mercenary");
    let saw = t.battlefield(P0, "Bone Saw");
    t.g.attach(saw, obj(cloud));
    t.g.recompute();
    let island = t.library_top(P0, "Island");
    let forest = t.library_top(P0, "Forest");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Mountain", 1);
    // Strata Scythe enters attached to Cloud (it "somehow entered attached").
    let card = t.hand(P0, "Strata Scythe");
    let scythe =
        t.g.move_object(
            card,
            Zone::Battlefield,
            mtg_engine::events::MoveCause::Effect,
            Some(P0),
        )
        .unwrap();
    t.g.attach(scythe, obj(cloud));
    t.g.recompute();
    t.answer_choose(P0, &[obj(island)]);
    t.answer_choose(P0, &[obj(forest)]);
    t.g.flush_events();
    t.settle();
    assert_eq!(
        triggers_of(&t, "Strata Scythe").len(),
        2,
        "{}",
        t.dump_log()
    );
    t.resolve_all();
    assert!(t.in_exile("Island") && t.in_exile("Forest"));
    // Base 4/4 Cloud, Bone Saw +1/+0, two Islands and a Forest: +3/+3.
    let (p, tough) = t.pt(cloud);
    let base = crate::r_residue_misc_2b_common::base_pt("Cloud, Midgar Mercenary");
    assert_eq!((p, tough), (base.0 + 1 + 3, base.1 + 3), "{}", t.dump_log());
}

// --- Galea, Kindler of Hope ---------------------------------------------------------------------

#[test]
fn galea_doesnt_change_timing_and_the_new_top_card_waits_until_the_cast_is_done() {
    cr!("601.3", "401.5", "307.1");
    ruling!(
        "Galea, Kindler of Hope",
        "Galea, Kindler of Hope doesn't change when you can cast Aura and Equipment spells. Normally, this means during your main phase when the stack is empty, although flash may change this."
    );
    ruling!(
        "Galea, Kindler of Hope",
        "If the top card of your library changes while you're casting a spell, playing a land, or activating an ability, you can't look at the new top card until you finish doing so. This means that if you cast a spell from the top of your library, you can't look at the next card until you're done paying for that spell."
    );
    supported("Galea, Kindler of Hope");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Galea, Kindler of Hope");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let next = t.library_top(P0, "Bonesplitter");
    let hs = t.library_top(P0, "Holy Strength");
    t.lands(P0, "Plains", 1);
    // During combat the Aura can't be cast.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_cast(&mut t, P0, hs, CastMethod::Normal));
    t.set_step(P0, Step::PrecombatMain);
    assert!(can_cast(&mut t, P0, hs, CastMethod::Normal));
    // While Holy Strength is being cast (its target is chosen), P0 can't look at the
    // Bonesplitter that is now on top.
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseTargets { .. }),
        |g| {
            let top = *g.player(PlayerId(0)).library.last().unwrap();
            mtg_engine::facedown::can_look_at(g, PlayerId(0), top)
        },
    );
    t.answer_targets(P0, &[obj(bears)]);
    t.cast(P0, hs).go();
    assert_eq!(seen.lock().unwrap().clone(), vec![false]);
    // Done casting: it's the top card P0 may look at.
    assert!(mtg_engine::facedown::can_look_at(&t.g, P0, next));
    t.resolve_all();
}

// --- Chandra, Flame's Catalyst ------------------------------------------------------------------

#[test]
fn chandra_casts_from_the_graveyard_during_resolution_paying_costs() {
    cr!("608.2g", "601.2", "118.9a");
    ruling!(
        "Chandra, Flame's Catalyst",
        "If you wish to cast the target card, you must cast it while Chandra's second ability is resolving. You can't cast it later in the turn."
    );
    ruling!(
        "Chandra, Flame's Catalyst",
        "You must pay the costs for the spell you wish to cast with Chandra's second ability. If that spell has any alternative costs, you may pay those."
    );
    supported("Chandra, Flame's Catalyst");
    // Declined as it resolves: it can't be cast later.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let c = pw(&mut t, P0, "Chandra, Flame's Catalyst", 5);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.answer_yes(P0, false);
    activate_containing(&mut t, P0, c, "graveyard").expect("-2");
    t.resolve_all();
    t.lands(P0, "Mountain", 1);
    assert!(!can_cast(&mut t, P0, bolt, CastMethod::Normal));
    // Cast as it resolves, paying {R}; exiled as it would go to the graveyard.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let c = pw(&mut t, P0, "Chandra, Flame's Catalyst", 5);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bolt)]);
    t.answer_targets(P0, &[pl(P1)]);
    t.g.turn.priority = Some(P0);
    t.g.recompute();
    let uid = t
        .obj_now(c)
        .chars
        .abilities
        .iter()
        .find(|a| a.text.contains("graveyard"))
        .unwrap()
        .uid;
    t.g.activate_ability(P0, c, uid).expect("-2");
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "{}", t.dump_log());
    assert_eq!(tapped_lands(&t, P0), 1);
    assert!(t.in_exile("Lightning Bolt"));
}

#[test]
fn chandra_ultimate_draws_seven_with_an_empty_hand_and_doesnt_change_timing() {
    cr!("608.2c", "601.3", "118.9");
    ruling!(
        "Chandra, Flame's Catalyst",
        "If you have no cards in hand while Chandra's last ability is resolving, you still draw seven cards and may cast spells from your hand without paying their mana costs this turn."
    );
    ruling!(
        "Chandra, Flame's Catalyst",
        "Chandra's last ability doesn't change when you may cast those spells. For example, you can't cast a sorcery spell from your hand during combat."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let c = pw(&mut t, P0, "Chandra, Flame's Catalyst", 8);
    stack_library(&mut t, P0, &["Divination"; 7]);
    activate_containing(&mut t, P0, c, "seven").expect("-8");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 7);
    let div = t.g.player(P0).hand[0];
    assert!(can_cast(&mut t, P0, div, CastMethod::Free));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_cast(&mut t, P0, div, CastMethod::Free));
}

// --- Selective Obliteration, Tolarian Contempt, Doomsday Excruciator, Ashnod's Cylix -------------

#[test]
fn selective_obliteration_each_permanent_is_judged_by_its_controllers_choice() {
    cr!("105.2", "105.2c");
    ruling!(
        "Selective Obliteration",
        "For example, say you control a colorless creature, a blue creature, a green creature, and a creature that's both blue and green. No matter what you choose, your colorless creature will not be exiled, and your creature that's both blue and green will be exiled. If you choose blue or green, the creature you control that is only that color will not be exiled. It doesn't matter what anyone else chooses."
    );
    supported("Selective Obliteration");
    // (P0's choice, P1's choice) -> which of P0's survive: colorless, blue, green, both.
    for (mine, theirs, alive) in [
        (1usize, 4usize, [true, true, false, false]),
        (4, 1, [true, false, true, false]),
        (0, 1, [true, false, false, false]),
    ] {
        let mut t = TestGame::new(2);
        let cs = [
            t.battlefield(P0, "Ornithopter"),
            t.battlefield(P0, "Storm Crow"),
            t.battlefield(P0, "Grizzly Bears"),
            t.battlefield(P0, "Coiling Oracle"),
        ];
        t.lands(P0, "Wastes", 5);
        let s = t.hand(P0, "Selective Obliteration");
        t.answer(P0, DecisionKind::Option, Answer::Index(mine));
        t.answer(P1, DecisionKind::Option, Answer::Index(theirs));
        t.cast(P0, s).go();
        t.resolve_all();
        let now: Vec<bool> = cs.iter().map(|c| t.on_battlefield(*c)).collect();
        assert_eq!(now, alive, "P0 chose {mine}, P1 {theirs}");
    }
}

#[test]
fn tolarian_contempt_the_owner_chooses_top_or_bottom() {
    cr!("401.4", "108.3");
    ruling!(
        "Tolarian Contempt",
        "Each affected creature's owner chooses whether to put it on the top or bottom of their library."
    );
    supported("Tolarian Contempt");
    // P1 controls P2's Hill Giant (with a rejection counter): P2 is asked.
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Tolarian Contempt");
    let giant = t.battlefield(P2, "Hill Giant");
    give_control(&mut t, giant, P1);
    t.g.add_counters(obj(t.g.current(giant)), "rejection", 1, None);
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_targets(P0, &[]);
    let from = t.asked().len();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let chooser = asked_since(&t, from)
        .into_iter()
        .find_map(|(p, d)| match d {
            Decision::ChooseOption { options, .. } if options.iter().any(|o| o.contains("Top")) => {
                Some(p)
            }
            _ => None,
        });
    assert_eq!(chooser, Some(P2), "{:?}", asked_since(&t, from));
    assert!(t
        .g
        .player(P2)
        .library
        .iter()
        .any(|c| t.g.obj(*c).chars.name == "Hill Giant"));
}

#[test]
fn doomsday_excruciator_exiles_nothing_from_six_or_fewer_cards() {
    cr!("608.2c", "406.3");
    ruling!(
        "Doomsday Excruciator",
        "A player with six or fewer cards remaining in their library when Doomsday Excruciator's second ability resolves won't exile any cards."
    );
    supported("Doomsday Excruciator");
    let mut t = TestGame::new(2);
    t.g.players[P1.idx()].library.truncate(5);
    t.g.players[P0.idx()].library.truncate(10);
    t.lands(P0, "Swamp", 6);
    let d = t.hand(P0, "Doomsday Excruciator");
    t.cast(P0, d).go();
    t.resolve_all();
    assert_eq!(t.library_size(P1), 5, "{}", t.dump_log());
    assert_eq!(t.library_size(P0), 6);
}

#[test]
fn ashnods_cylix_with_fewer_than_three_cards() {
    cr!("401.4", "608.2c");
    ruling!(
        "Ashnod's Cylix",
        "If the player has less than 3 cards in their library, look at as many as they have, put one back, and exile the others."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let cy = t.battlefield(P0, "Ashnod's Cylix");
    t.g.players[P1.idx()].library.clear();
    let a = t.library_top(P1, "Island");
    let b = t.library_top(P1, "Forest");
    t.lands(P0, "Wastes", 3);
    t.answer_choose(P1, &[obj(a)]);
    t.activate(P0, cy, 0, &[pl(P1)]).expect("Cylix");
    t.resolve_all();
    assert_eq!(t.library_size(P1), 1, "{}", t.dump_log());
    assert_eq!(t.zone(b), Zone::Exile);
}

// --- Illusionary Mask --------------------------------------------------------------------------

/// P0 activates Illusionary Mask with X paid by `n` `lands`, casting `card` face down.
fn mask(t: &mut TestGame, lands: &str, n: usize, card: ObjectId) -> ObjectId {
    let m = t.battlefield(P0, "Illusionary Mask");
    t.lands(P0, lands, n);
    t.answer(P0, DecisionKind::X, Answer::Number(n as i64));
    t.answer_choose(P0, &[obj(card)]);
    t.answer_yes(P0, true);
    t.activate(P0, m, 0, &[]).unwrap();
    t.resolve_all();
    t.g.recompute();
    t.g.battlefield
        .iter()
        .copied()
        .find(|o| t.obj(*o).face_down)
        .expect("a face-down creature")
}

#[test]
fn illusionary_mask_enters_abilities_dont_apply_face_down_or_when_turned_up() {
    cr!("708.3", "708.8", "614.12");
    ruling!(
        "Illusionary Mask",
        "The creature enters face down, so none of its \"enters\" abilities will trigger or have any effect. Also none of its \"As this enters\" abilities apply."
    );
    ruling!(
        "Illusionary Mask",
        "The creature's \"enters\" abilities (and any other abilities relating to the creature entering) do not trigger when it turns face up."
    );
    supported("Illusionary Mask");
    // Flametongue Kavu: no trigger entering, none when it's turned face up by tapping.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P1, "Grizzly Bears");
    let fk = t.hand(P0, "Flametongue Kavu");
    let fd = mask(&mut t, "Mountain", 4, fk);
    assert!(t.g.stack.is_empty());
    tap(&mut t, fd);
    assert!(!t.obj_now(fd).face_down);
    assert!(t.g.stack.is_empty(), "{}", t.dump_log());
    // Triskelion: no counters.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let tri = t.hand(P0, "Triskelion");
    let fd = mask(&mut t, "Wastes", 6, tri);
    assert_eq!(t.counters(fd, counters::PLUS1), 0);
    tap(&mut t, fd);
    assert_eq!(t.counters(fd, counters::PLUS1), 0);
}

#[test]
fn illusionary_mask_face_down_creature_is_seen_only_by_its_controller() {
    cr!("708.5");
    ruling!(
        "Illusionary Mask",
        "Only the controller of the face down creature can look at it."
    );
    ruling!(
        "Illusionary Mask",
        "You may look at a face-down spell or permanent you control at any time."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.hand(P0, "Hill Giant");
    let fd = mask(&mut t, "Mountain", 4, giant);
    assert!(mtg_engine::facedown::can_look_at(&t.g, P0, fd));
    assert!(!mtg_engine::facedown::can_look_at(&t.g, P1, fd));
}

#[test]
fn illusionary_mask_turns_up_only_for_damage_or_tapping_or_by_morph() {
    cr!("708.7", "702.37e", "708.8");
    ruling!(
        "Illusionary Mask",
        "You may not turn a face-down spell face up. You may not turn a face-down permanent face up unless it would have morph while face up or an effect specifically allows you to turn it face up. Illusionary Mask's ability has you turn a face-down creature face up if it would assign damage, deal damage, be dealt damage, or become tapped, but not for any other reason. For example, if you use Illusionary Mask's ability to cast a black creature face down, you can't turn it face up just because it's being targeted by Terror."
    );
    ruling!(
        "Illusionary Mask",
        "You can turn a face-down permanent face up if it would have morph while face up. This applies to creatures you cast face down as a result of Illusionary Mask's effect. The rest of Illusionary Mask's effect applies to it as well."
    );
    supported("Terror");
    // A black creature (Drudge Skeletons) face down: no special action to turn it up;
    // Terror destroys it face down.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let sk = t.hand(P0, "Drudge Skeletons");
    let fd = mask(&mut t, "Swamp", 2, sk);
    assert!(!crate::r_s11_common::can_turn_face_up(&mut t, P0, fd));
    t.lands(P1, "Swamp", 2);
    let terror = t.hand(P1, "Terror");
    t.cast(P1, terror).target(obj(fd)).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Drudge Skeletons"), "{}", t.dump_log());
    // A morph creature (Exalted Angel): it can be turned face up for its morph cost.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let angel = t.hand(P0, "Exalted Angel");
    let fd = mask(&mut t, "Plains", 6, angel);
    t.lands(P0, "Plains", 4);
    assert!(crate::r_s11_common::can_turn_face_up(&mut t, P0, fd));
}

#[test]
fn illusionary_mask_card_is_revealed_when_it_leaves() {
    cr!("708.9");
    ruling!(
        "Illusionary Mask",
        "If you use the ability to cast a creature card face down, you must keep track of the amount and types of mana you spent on {X}. If that creature spell is moved from the stack to anywhere other than the battlefield, the resulting creature leaves the battlefield, or the game ends, the face-down card is revealed. If its mana cost couldn't be paid by some amount of, or all of, the mana you spent on {X}, all applicable penalties for casting a card illegally are assessed."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.hand(P0, "Hill Giant");
    let fd = mask(&mut t, "Mountain", 4, giant);
    let from = t.g.turn_events.len();
    destroy(&mut t, fd);
    let card = t.g.current(fd);
    assert!(!t.obj_now(card).face_down);
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(mtg_engine::facedown::can_look_at(&t.g, P1, card));
    let _ = from;
}

// --- Ferris Wheel -------------------------------------------------------------------------

#[test]
fn ferris_wheel_must_target_if_it_can_and_keeps_counters_while_phased_out() {
    cr!("603.3d", "702.26d", "702.26e");
    ruling!(
        "Ferris Wheel",
        "If you visit Ferris Wheel and there are no legal targets to choose, the ability is removed from the stack and has no effect. However, if there’s at least one legal target, you must choose one, even if it’s a creature you don’t want to phase out."
    );
    ruling!(
        "Ferris Wheel",
        "Permanents usually phase back in during their controller’s untap step, immediately before that player untaps their permanents. But Ferris Wheel isn’t so forgiving. The phased-out creatures will stay phased out until a player rolls a 3 or less. If a permanent had counters or stickers on it when it phased out, it will have them when it phases back in."
    );
    supported("Ferris Wheel");
    let roll = |t: &mut TestGame, r: u32| {
        t.g.dice.loaded.push_back(r);
        mtg_engine::variants::roll_to_visit(&mut t.g, P0);
        t.g.flush_events();
        t.settle();
    };
    // No creatures: nothing goes on the stack.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ferris Wheel");
    roll(&mut t, 5);
    assert!(t.g.stack.is_empty());
    // Only P0's own creature: it must be chosen (no answer given).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ferris Wheel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    plus1(&mut t, bears, 2);
    roll(&mut t, 5);
    t.resolve_all();
    assert!(t.obj_now(bears).phased_out);
    // It stays phased out through P0's untap step, then a 3 brings it back, counters on.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(bears).phased_out);
    roll(&mut t, 3);
    assert!(!t.obj_now(bears).phased_out);
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
}
