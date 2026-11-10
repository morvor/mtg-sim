//! Rulings batch P158 — the rest: where a spell is cast from (CR 601.1a, 702.34a), a
//! stolen creature phasing out (CR 702.26n), unearth (CR 702.84c), casting from a
//! revealed pile (CR 608.2c), Auras put onto the battlefield (CR 303.4f), mana abilities
//! (CR 605.3), mana restrictions (CR 609.4), a countered spell's destination (CR 701.6a,
//! 614.15), activating a bestowed Aura (CR 602.2), "played a land" (CR 305.1), searching
//! by subtype (CR 205.3d) and card types (CR 205.2a), and a countered delayed trigger.

use crate::r_p158_common::*;
use crate::r_s01_common::custom_card;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn fblthp_cast_from_exile_was_not_cast_from_the_library() {
    cr!("601.1a", "400.7", "614.1a");
    ruling!(
        "Fblthp, the Lost",
        "If an effect exiles Fblthp from your library and then lets you cast that card, it's cast from exile, not from your library."
    );
    supported("Fblthp, the Lost");
    supported("Light Up the Stage");
    supported("Experimental Frenzy");
    // Light Up the Stage exiles Fblthp from the top of the library; cast from exile, it
    // draws one card.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let f = t.library_top(P0, "Fblthp, the Lost");
    lots(&mut t, P0);
    cast_at(&mut t, P0, "Light Up the Stage", &[]);
    t.resolve_all();
    assert_eq!(t.zone(f), Zone::Exile);
    let hand = t.hand_size(P0);
    let f = t.g.current(f);
    t.cast(P0, f).go();
    t.resolve_all();
    assert!(t.on_battlefield(f));
    assert_eq!(t.hand_size(P0), hand + 1);
    // Cast from the top of the library (Experimental Frenzy): two cards.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Experimental Frenzy");
    let f = t.library_top(P0, "Fblthp, the Lost");
    lots(&mut t, P0);
    let hand = t.hand_size(P0);
    t.cast(P0, f).go();
    t.resolve_all();
    assert!(t.on_battlefield(f));
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn a_stolen_creature_phased_out_by_divine_smite_phases_in_under_its_owners_control() {
    cr!("702.26a", "702.26f", "702.26n", "800.4a");
    ruling!(
        "Divine Smite",
        "If an opponent gains control of your nonblack creature, you phase it out with Divine Smite, and the duration of the control-change effect expires before it phases back in, your creature phases in under your control as that opponent's next untap step begins. If they leave the game before their next untap step, it phases in as the next untap step begins after their turn would have begun."
    );
    supported("Divine Smite");
    supported("Ray of Command");
    // P1's turn: P1 takes P0's Bears until end of turn; P0 phases it out.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    cast_at(&mut t, P1, "Ray of Command", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(ctl(&t, bears), P1);
    cast_at(&mut t, P0, "Divine Smite", &[obj(bears)]);
    t.resolve_all();
    assert!(t.obj_now(bears).phased_out);
    // P0's untap step: it phased out under P1's control, so it stays out.
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(bears).phased_out);
    // P1's untap step: it phases in under P0's control.
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(bears).phased_out);
    assert_eq!(ctl(&t, bears), P0);
    // Three players: P1 steals it, P0 phases it out, P1 leaves the game. It phases in as
    // the untap step after P1's turn would have begun (P2's, after P0's turn) begins.
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    cast_at(&mut t, P1, "Ray of Command", &[obj(bears)]);
    t.resolve_all();
    cast_at(&mut t, P0, "Divine Smite", &[obj(bears)]);
    t.resolve_all();
    assert!(t.obj_now(bears).phased_out);
    crate::r_p053_common::concede(&mut t, P1);
    assert!(t.has_lost(P1));
    t.advance_to(P2, Step::Upkeep);
    assert!(t.obj_now(bears).phased_out, "P2's untap step right after P1 left");
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(bears).phased_out, "P0's untap step");
    t.advance_to(P2, Step::Upkeep);
    assert!(!t.obj_now(bears).phased_out, "the untap step after P1's turn would have begun");
    assert_eq!(ctl(&t, bears), P0);
    // A black creature is exiled instead.
    let mut t = TestGame::new(2);
    let evangel = t.battlefield(P1, "Cabal Evangel");
    cast_at(&mut t, P0, "Divine Smite", &[obj(evangel)]);
    t.resolve_all();
    assert!(t.in_exile("Cabal Evangel"));
}

#[test]
fn meticulous_excavation_returns_an_unearthed_creature_through_exile() {
    cr!("702.84a", "614.1a", "614.15");
    ruling!(
        "Meticulous Excavation",
        "If the target permanent has unearth and was put onto the battlefield using its unearth ability, this effect can still exile it and return it to its owner's hand."
    );
    supported("Meticulous Excavation");
    supported("Dregscape Zombie");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let ex = t.battlefield(P0, "Meticulous Excavation");
    let zombie = crate::r_s18_common::unearthed(&mut t, P0, "Dregscape Zombie", "{B}");
    act(&mut t, P0, ex, 0, &[obj(zombie)]);
    assert!(t.in_hand(P0, "Dregscape Zombie"));
    assert!(!t.in_exile("Dregscape Zombie"));
    // A Grizzly Bears without unearth: straight to the hand.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let ex = t.battlefield(P0, "Meticulous Excavation");
    let bears = t.battlefield(P0, "Grizzly Bears");
    act(&mut t, P0, ex, 0, &[obj(bears)]);
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn talent_of_the_telepath_casts_from_the_revealed_cards_and_the_rest_go_to_their_graveyard() {
    cr!("608.2c", "400.3", "608.2n");
    ruling!(
        "Talent of the Telepath",
        "If you can’t cast any instant or sorcery cards (perhaps because there are no legal targets available) or if you choose not to cast one, then Talent of the Telepath finishes resolving. Any of the revealed cards you didn’t cast will be put into that player’s graveyard."
    );
    ruling!(
        "Talent of the Telepath",
        "The cards will be put into their owner’s graveyard after they resolve, not yours."
    );
    supported("Talent of the Telepath");
    // Shock among the seven: cast at P1, it resolves and goes to P1's graveyard with the
    // other six.
    let mut t = TestGame::new(2);
    let top = stack_library(
        &mut t,
        P0,
        &["Grizzly Bears"],
    );
    let _ = top;
    let seven = stack_library(
        &mut t,
        P1,
        &["Grizzly Bears", "Shock", "Hill Giant", "Gray Ogre", "Forest", "Island", "Plains"],
    );
    let shock = seven[1];
    t.answer_choose(P0, &[obj(shock)]);
    t.answer_targets(P0, &[pl(P1)]);
    cast_at(&mut t, P0, "Talent of the Telepath", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(seven.iter().all(|c| t.zone(*c) == Zone::Graveyard(P1)), "all in P1's graveyard");
    assert_eq!(t.graveyard_size(P0), 1, "only the Talent");
    // No instant or sorcery among them: nothing is cast, all seven go to the graveyard.
    let mut t = TestGame::new(2);
    let seven = stack_library(
        &mut t,
        P1,
        &["Grizzly Bears", "Hill Giant", "Gray Ogre", "Forest", "Island", "Plains", "Swamp"],
    );
    let from = t.asked().len();
    cast_at(&mut t, P0, "Talent of the Telepath", &[pl(P1)]);
    t.resolve_all();
    assert!(seven.iter().all(|c| t.zone(*c) == Zone::Graveyard(P1)));
    assert!(
        !asked_since(&t, from)
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseEntities { .. } | Decision::YesNo { .. })),
        "{:?}",
        asked_since(&t, from)
    );
}

#[test]
fn shepherds_returning_an_aura_choose_what_it_enchants_without_targeting() {
    cr!("303.4f", "303.4", "702.11b", "702.16b");
    ruling!(
        "Emeria Shepherd",
        "If you return an Aura to the battlefield this way, you choose what it will enchant as it enters the battlefield. This doesn't target any player or permanent, so you could attach an Aura to a permanent with hexproof controlled by an opponent, for example. If there's nothing the Aura can legally enchant, it stays in the graveyard."
    );
    ruling!(
        "Shepherd of the Clouds",
        "If you return an Aura to the battlefield this way, you choose what the Aura will enchant just before it enters the battlefield. An Aura returning to the battlefield this way doesn’t target the player or permanent it will enchant, so permanents or players with hexproof may be chosen; however, the chosen recipient must be able to legally be enchanted by the Aura, so a player or permanent with protection from one of the Aura’s qualities can’t be chosen this way. If there’s nothing legal for the Aura to enchant, it stays in the graveyard."
    );
    supported("Emeria Shepherd");
    supported("Shepherd of the Clouds");
    supported("Fortune, Loyal Steed");
    supported("Betrayal");
    supported("Gladecover Scout");
    supported("Scragnoth");
    // Betrayal ({U}, "Enchant creature an opponent controls"): the Shepherds' own
    // creatures don't qualify. (P1's only creature, whether Betrayal ends up on it.)
    for (creature, attached) in [
        (Some("Gladecover Scout"), true),
        (Some("Scragnoth"), false),
        (None, false),
    ] {
        // Shepherd of the Clouds with a Mount.
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Fortune, Loyal Steed");
        let target = creature.map(|c| t.battlefield(P1, c));
        let aura = t.graveyard(P0, "Betrayal");
        t.answer_targets(P0, &[obj(aura)]);
        let from = t.asked().len();
        t.enter(P0, "Shepherd of the Clouds");
        t.resolve_all();
        assert_eq!(t.on_battlefield(aura), attached, "Shepherd of the Clouds, {creature:?}: {:?}", asked_since(&t, from));
        if attached {
            assert_eq!(crate::r_s06_common::attached_to(&t, aura), target.map(obj));
        } else {
            assert!(t.in_graveyard(P0, "Betrayal"), "Shepherd of the Clouds, {creature:?}");
        }
        // Emeria Shepherd with a Plains entering.
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        t.battlefield(P0, "Emeria Shepherd");
        let target = creature.map(|c| t.battlefield(P1, c));
        let aura = t.graveyard(P0, "Betrayal");
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[obj(aura)]);
        t.answer_yes(P0, true);
        let plains = t.hand(P0, "Plains");
        t.play_land(P0, plains).expect("land");
        t.resolve_all();
        assert_eq!(t.on_battlefield(aura), attached, "Emeria Shepherd, {creature:?}");
        if attached {
            assert_eq!(crate::r_s06_common::attached_to(&t, aura), target.map(obj));
        } else {
            assert!(t.in_graveyard(P0, "Betrayal"), "Emeria Shepherd, {creature:?}");
        }
    }
}

#[test]
fn ilysian_caryatids_ability_is_a_mana_ability() {
    cr!("605.1a", "605.3", "605.3b");
    ruling!(
        "Ilysian Caryatid",
        "Ilysian Caryatid’s activated ability is a mana ability. It doesn’t use the stack and can’t be responded to."
    );
    supported("Ilysian Caryatid");
    for big in [false, true] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, "Ilysian Caryatid");
        if big {
            t.battlefield(P0, "Lowland Giant");
        }
        crate::r_p198_common::choose_words(&mut t, P0, &["Green", "{G}", "G"]);
        assert!(crate::r_s20_common::tap_for_mana(&mut t, P0, c, "Add"));
        assert_eq!(t.stack_len(), 0, "big {big}");
        assert_eq!(t.g.players[0].mana_pool.total(), if big { 2 } else { 1 }, "big {big}");
    }
}

#[test]
fn incubation_druids_mana_has_no_restrictions_of_the_land() {
    cr!("609.4", "605.1a", "106.6");
    ruling!(
        "Incubation Druid",
        "Incubation Druid doesn't care about any restrictions or riders your lands put on the mana they produce, such as those of Unclaimed Territory and Guildmages' Forum. It just produces one mana of the appropriate type, with no restrictions or riders."
    );
    supported("Incubation Druid");
    supported("Unclaimed Territory");
    // Unclaimed Territory (Elf chosen) is the only land: it could produce red mana, so
    // the Druid adds {R}, which, unlike the Territory's, can pay for Shock.
    let mut t = TestGame::new(2);
    crate::r_p198_common::choose_words(&mut t, P0, &["Elf", "R"]);
    let territory = t.enter(P0, "Unclaimed Territory");
    t.resolve_all();
    let druid = t.battlefield(P0, "Incubation Druid");
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P0, druid, "Add"));
    assert_eq!(t.g.players[0].mana_pool.count(ManaType::R), 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(pl(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // The Territory's own mana can't.
    crate::r_p198_common::choose_words(&mut t, P0, &["R"]);
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P0, territory, "any color"));
    assert_eq!(t.g.players[0].mana_pool.count(ManaType::R), 1);
    let shock = t.hand(P0, "Shock");
    assert!(t.cast(P0, shock).target(pl(P1)).try_go().is_err());
}

#[test]
fn memory_lapse_puts_the_spell_on_top_unless_flashback_exiles_it() {
    cr!("614.15", "701.6a", "702.34a", "616.1");
    ruling!(
        "Memory Lapse",
        "Memory Lapse has a self-replacement effect that replaces the spell going to the graveyard before any other effect can replace that event. If the spell was cast using flashback, however, flashback will change the spell's destination from its owner's library to exile."
    );
    supported("Memory Lapse");
    supported("Think Twice");
    for flashback in [false, true] {
        let mut t = TestGame::new(2);
        let tt = if flashback {
            let c = t.graveyard(P1, "Think Twice");
            cast_from_graveyard(&mut t, P1, c, &[])
        } else {
            cast_at(&mut t, P1, "Think Twice", &[])
        };
        cast_at(&mut t, P0, "Memory Lapse", &[obj(tt)]);
        t.resolve();
        assert_eq!(t.hand_size(P1), 0, "flashback {flashback}: countered");
        if flashback {
            assert!(t.in_exile("Think Twice"));
        } else {
            assert_eq!(crate::r_s33_common::library_top_first(&t, P1)[0], t.g.current(tt));
        }
    }
}

#[test]
fn witchs_oven_sacrifices_before_anyone_can_act() {
    cr!("602.2b", "601.2h", "117.3c");
    ruling!(
        "Witch's Oven",
        "No player may take actions between the time you announce you're activating Witch's Oven's ability and the time you sacrifice a creature. Notably, they can't try to remove or shrink the creature you'd like to bake."
    );
    supported("Witch's Oven");
    let mut t = TestGame::new(2);
    let oven = t.battlefield(P0, "Witch's Oven");
    let giant = t.battlefield(P0, "Serra Angel");
    let seen = crate::r_s01_common::watch(
        &mut t,
        P1,
        |_| true,
        |g| g.battlefield.iter().any(|o| g.obj(*o).chars.name == "Serra Angel"),
    );
    t.answer_choose(P0, &[obj(giant)]);
    let from = t.asked().len();
    act_only(&mut t, P0, oven, 0, &[]);
    assert!(!t.on_battlefield(giant), "sacrificed as a cost");
    assert!(
        crate::r_p046_common::others_asked_since(&t, from, P0).is_empty(),
        "{:?}",
        asked_since(&t, from)
    );
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Food").len(), 2, "toughness 4");
    // Whenever P1 decided anything, the Angel was already gone.
    assert!(seen.lock().unwrap().iter().all(|on| !on));
}

#[test]
fn only_the_bestowed_emissarys_controller_activates_its_ability() {
    cr!("602.2", "702.103b", "614.1a");
    ruling!(
        "Erebos's Emissary",
        "Only the controller of the Erebos's Emissary may activate the ability, even if Erebos's Emissary is attached to a creature another player controls."
    );
    supported("Erebos's Emissary");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P1, "Hill Giant");
    lots(&mut t, P0);
    let e = t.hand(P0, "Erebos's Emissary");
    let e = t
        .cast(P0, e)
        .method(CastMethod::Keyword(KeywordKind::Bestow))
        .target(obj(giant))
        .go();
    t.resolve_all();
    let e = t.g.current(e);
    assert_eq!(crate::r_s06_common::attached_to(&t, e), Some(obj(giant)));
    assert_eq!(t.pt(giant), (6, 6));
    // P1, with a creature card in hand, can't activate it; P0 can: the Giant gets +2/+2.
    t.hand(P1, "Grizzly Bears");
    assert!(t.activate(P1, e, 0, &[]).is_err());
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, e, 0, &[]).expect("P0 activates");
    t.resolve_all();
    assert_eq!(t.pt(giant), (8, 8));
}

#[test]
fn river_of_tears_counts_lands_played_not_lands_put_onto_the_battlefield() {
    cr!("305.1", "305.9", "614.1a");
    ruling!(
        "River of Tears",
        "River of Tears produces {B} only after you've played a land, not after you've put a land onto the battlefield (such as with Evolving Wilds)."
    );
    ruling!(
        "River of Tears",
        "The turn you play River of Tears, it will produce {B} when tapped for mana."
    );
    supported("River of Tears");
    supported("Evolving Wilds");
    // Played this turn: {B}.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let river = t.hand(P0, "River of Tears");
    t.play_land(P0, river).expect("land");
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P0, river, "Add"));
    assert_eq!(t.g.players[0].mana_pool.count(ManaType::B), 1);
    // Not played this turn: {U}; a land put onto the battlefield by Evolving Wilds
    // doesn't change that.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let river = t.battlefield(P0, "River of Tears");
    let wilds = t.battlefield(P0, "Evolving Wilds");
    let forest = t.library_top(P0, "Forest");
    t.answer_choose(P0, &[obj(forest)]);
    act(&mut t, P0, wilds, 0, &[]);
    assert!(t.on_battlefield(forest));
    t.g.players[0].mana_pool.mana.clear();
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P0, river, "Add"));
    assert_eq!(t.g.players[0].mana_pool.count(ManaType::U), 1);
    assert_eq!(t.g.players[0].mana_pool.count(ManaType::B), 0);
}

#[test]
fn see_the_truth_gives_no_permission_to_cast_it_from_elsewhere() {
    cr!("601.1a", "601.3");
    ruling!(
        "See the Truth",
        "See the Truth doesn't give you permission to cast it from a zone other than your hand. You'll need to find another way to do so."
    );
    supported("See the Truth");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    lots(&mut t, P0);
    let gy = t.graveyard(P0, "See the Truth");
    let ex = t.exile(P0, "See the Truth");
    let hand = t.hand(P0, "See the Truth");
    assert!(legal_cast_methods(&mut t, P0, gy).is_empty());
    assert!(legal_cast_methods(&mut t, P0, ex).is_empty());
    assert!(!legal_cast_methods(&mut t, P0, hand).is_empty());
}

#[test]
fn the_five_doctors_finds_cards_with_the_doctor_type_not_the_word() {
    cr!("205.3d", "701.23a");
    ruling!(
        "The Five Doctors",
        "The Five Doctors can find any card with the Doctor creature type, not cards with the word \"Doctor\" in the name."
    );
    supported("The Five Doctors");
    supported("Doctor Spectrum");
    let mut t = TestGame::new(2);
    let typed = t.custom(
        P0,
        custom_card("Time Lord", "Creature — Time Lord Doctor", "{1}{U}", Some((1, 1)), ""),
        Zone::Library(P0),
    );
    let named = t.library_top(P0, "Doctor Spectrum");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(typed)]);
    lots(&mut t, P0);
    let s = t.hand(P0, "The Five Doctors");
    t.cast(P0, s).kicked(false).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Time Lord"));
    assert!(!t.in_hand(P0, "Doctor Spectrum"));
    for (p, d) in asked_since(&t, from) {
        if let Decision::ChooseEntities { candidates, .. } = d {
            assert!(p == P0 && candidates.contains(&obj(typed)) && !candidates.contains(&obj(named)), "{candidates:?}");
        }
    }
}

#[test]
fn traverse_the_ulvenwald_counts_card_types_only() {
    cr!("205.2a", "205.4a", "205.3a", "614.1a");
    ruling!(
        "Traverse the Ulvenwald",
        "The card types that could appear in your graveyard are artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Supertypes (such as legendary and basic) and subtypes (such as Human and Equipment) are not counted."
    );
    supported("Traverse the Ulvenwald");
    supported("Eyeblight's Ending");
    // Candidates offered by the search (the Forest and the Bears are on top).
    let offered = |graveyard: &[&str]| -> (bool, bool) {
        let mut t = TestGame::new(2);
        for c in graveyard {
            t.graveyard(P0, c);
        }
        let forest = t.library_top(P0, "Forest");
        let bears = t.library_top(P0, "Grizzly Bears");
        let from = t.asked().len();
        cast_at(&mut t, P0, "Traverse the Ulvenwald", &[]);
        t.resolve_all();
        let cands: Vec<Vec<Entity>> = asked_since(&t, from)
            .iter()
            .filter_map(|(p, d)| match d {
                Decision::ChooseEntities { candidates, .. } if *p == P0 => Some(candidates.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(cands.len(), 1, "{graveyard:?}: {cands:?}");
        (cands[0].contains(&obj(forest)), cands[0].contains(&obj(bears)))
    };
    // Legendary enchantment creature, basic land, Human creature: three types.
    assert_eq!(offered(&["Sythis, Harvest's Hand", "Forest", "Cabal Evangel"]), (true, false));
    // Kindred instant (two types), artifact creature (two types): four types.
    assert_eq!(offered(&["Eyeblight's Ending", "Ornithopter"]), (true, true));
    // Legendary creature, basic land, instant: three types, however many supertypes.
    assert_eq!(offered(&["Sythis, Harvest's Hand", "Forest", "Shock"]), (true, true));
}

#[test]
fn defabricate_counters_the_unearth_delayed_trigger_but_not_the_exile_replacement() {
    cr!("702.84a", "603.7", "701.6a");
    ruling!(
        "Defabricate",
        "The last mode can target delayed triggered abilities. Notably, it can target the delayed triggered ability that exiles an unearthed permanent at the beginning of the next end step. If that delayed triggered ability is countered, the unearthed permanent remains on the battlefield indefinitely. It will still be exiled, however, if it would die or otherwise leave the battlefield for any other reason."
    );
    supported("Defabricate");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let zombie = crate::r_s18_common::unearthed(&mut t, P0, "Dregscape Zombie", "{B}");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1, "the delayed trigger");
    let trig = *t.g.stack.last().unwrap();
    lots(&mut t, P0);
    let d = t.hand(P0, "Defabricate");
    t.cast(P0, d).modes(&[1]).target(obj(trig)).go();
    t.resolve_all();
    assert!(t.on_battlefield(zombie), "the trigger was countered");
    pass_turn(&mut t);
    pass_turn(&mut t);
    assert!(t.on_battlefield(zombie), "it stays");
    kill(&mut t, zombie);
    assert!(t.in_exile("Dregscape Zombie"), "exiled instead of dying");
}

#[test]
fn forsaken_crossroads_may_untap_only_for_a_player_who_didnt_start() {
    cr!("103.1", "614.1a");
    // The ruling on Forsaken Crossroads ("it is highly recommended that you write down who
    // was the starting player") is bookkeeping advice (exempt in
    // docs/rulings-exemptions/rulings-P158.tsv); the game remembers the starting player,
    // which its "instead" needs: the starting player's Crossroads stays tapped, another
    // player's may untap.
    supported("Forsaken Crossroads");
    for (p, untaps) in [(P0, false), (P1, true)] {
        let mut t = TestGame::new(2);
        assert_eq!(t.g.turn.starting_player, P0);
        t.set_step(p, Step::PrecombatMain);
        let from = t.asked().len();
        crate::r_p198_common::choose_words(&mut t, p, &["Blue"]);
        t.answer_yes(p, true);
        let c = t.hand(p, "Forsaken Crossroads");
        t.play_land(p, c).expect("land");
        t.resolve_all();
        assert_eq!(!t.obj_now(c).tapped, untaps, "{p:?}");
        let offered = asked_since(&t, from)
            .iter()
            .any(|(q, d)| *q == p && matches!(d, Decision::YesNo { .. }));
        assert_eq!(offered, untaps, "{p:?}: {:?}", asked_since(&t, from));
    }
}
