//! The permission grammar for target cards ("You may cast target instant or sorcery card
//! from your graveyard this turn. If that spell would be put into your graveyard, exile it
//! instead."), cards looked at ("You may cast a spell from among them without paying its
//! mana cost. Put the rest on the bottom ..."), static riders ("You can't cast more than
//! one spell this way each turn."), conditions on playing the card ("During any turn you
//! attacked with ~"), and mana flexibility ("mana of any type can be spent", "as though it
//! were mana of any color to cast planeswalker spells", "Spend this mana only to cast
//! spells from your graveyard").

use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn can_cast_at(t: &mut TestGame, p: PlayerId, card: ObjectId, targets: &[Entity]) -> bool {
    let c = t.g.current(card);
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    t.g.turn.priority = Some(p);
    let ok = t.g.cast_spell(p, c, CastMethod::Normal).is_ok();
    t.g.flush_events();
    if ok {
        t.resolve_all();
    } else {
        t.clear_answers();
    }
    ok
}

#[test]
fn vohar_casts_target_card_from_your_graveyard_this_turn_then_exiles_it() {
    cr!("601.3", "614.1a");
    assert_supported("Vohar, Vodalian Desecrator");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let vohar = t.battlefield(P0, "Vohar, Vodalian Desecrator");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let other = t.graveyard(P0, "Lightning Bolt");
    t.activate(P0, vohar, 1, &[Entity::Object(bolt)]).unwrap();
    t.resolve_all();
    // Only the target.
    assert!(!can_cast_at(&mut t, P0, other, &[Entity::Player(P1)]));
    assert!(can_cast_at(&mut t, P0, bolt, &[Entity::Player(P1)]));
    assert_eq!(t.life(P1), 17);
    // Exiled instead of put into the graveyard.
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
}

#[test]
fn quistis_trepe_casts_a_card_from_a_graveyard_with_mana_of_any_type() {
    cr!("118.14", "608.2g", "614.1a");
    assert_supported("Quistis Trepe");
    let mut t = TestGame::new(2);
    // Only Islands: mana of any type can pay for the Bolt's {R}.
    t.lands(P0, "Island", 4);
    let bolt = t.graveyard(P1, "Lightning Bolt");
    let quistis = t.hand(P0, "Quistis Trepe");
    t.cast(P0, quistis).go();
    t.answer_targets(P0, &[Entity::Object(bolt)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "{}", t.dump_log());
    // The opponent's card, exiled instead of put into its owner's graveyard.
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
}

#[test]
fn hedonists_trove_casts_one_spell_a_turn_from_among_the_exiled_cards() {
    cr!("601.3", "607.2a");
    assert_supported("Hedonist's Trove");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 9);
    let bolt = t.graveyard(P1, "Lightning Bolt");
    let shock = t.graveyard(P1, "Shock");
    let forest = t.graveyard(P1, "Forest");
    let trove = t.hand(P0, "Hedonist's Trove");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, trove).go();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
    t.lands(P0, "Mountain", 2);
    assert!(can_cast_at(&mut t, P0, bolt, &[Entity::Player(P1)]));
    assert_eq!(t.life(P1), 17);
    // One spell this way each turn; lands are another permission.
    assert!(!can_cast_at(&mut t, P0, shock, &[Entity::Player(P1)]));
    t.play_land(P0, t.g.current(forest))
        .expect("play a land exiled with it");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(can_cast_at(&mut t, P0, shock, &[Entity::Player(P1)]));
    assert_eq!(t.life(P1), 15);
}

#[test]
fn svella_casts_one_of_the_cards_looked_at_and_puts_the_rest_on_the_bottom() {
    cr!("118.9", "608.2g");
    assert_supported("Svella, Ice Shaper");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    t.lands(P0, "Mountain", 4);
    let svella = t.battlefield(P0, "Svella, Ice Shaper");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    let giant = t.library_top(P0, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.activate(P0, svella, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    // The rest went to the bottom: the top card isn't one of them.
    let top = t.g.library_top(P0).unwrap();
    assert_ne!(t.g.obj(top).chars.name.as_str(), "Island");
    assert_eq!(t.zone(t.g.current(bears)), Zone::Library(P0));
    let lib = t.g.player(P0).library.clone();
    let pos = |id: ObjectId| lib.iter().position(|x| *x == id).unwrap();
    let from_top = |id: ObjectId| {
        if pos(top) == 0 {
            pos(id)
        } else {
            lib.len() - 1 - pos(id)
        }
    };
    // Among the bottom three cards.
    assert!(from_top(t.g.current(bears)) >= lib.len() - 3);
}

#[test]
fn goblin_researcher_plays_the_card_during_any_turn_it_attacked() {
    cr!("611.2a");
    ruling!(
        "Goblin Researcher",
        "You can play the exiled card if Goblin Researcher attacked and is still in combat, has left combat, has left the battlefield, or even if combat is over."
    );
    assert_supported("Goblin Researcher");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let researcher = t.enter(P0, "Goblin Researcher");
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
    // Not before it attacked.
    assert!(!can_cast_at(&mut t, P0, bolt, &[Entity::Player(P1)]));
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(researcher, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    // It attacked this turn; it has even left the battlefield since.
    t.g.destroy(researcher, None);
    t.settle();
    assert!(can_cast_at(&mut t, P0, bolt, &[Entity::Player(P1)]));
    assert_eq!(t.life(P1), 14);
}

#[test]
fn squee_can_be_cast_from_your_graveyard_or_from_exile() {
    cr!("601.3");
    assert_supported("Squee, the Immortal");
    for from_exile in [false, true] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Mountain", 3);
        let squee = if from_exile {
            t.exile(P0, "Squee, the Immortal")
        } else {
            t.graveyard(P0, "Squee, the Immortal")
        };
        assert!(can_cast_at(&mut t, P0, squee, &[]));
        assert!(t.on_battlefield(t.g.current(squee)));
    }
    // Not from the library.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let squee = t.library_top(P0, "Squee, the Immortal");
    assert!(!can_cast_at(&mut t, P0, squee, &[]));
}

#[test]
fn tegwylls_scouring_has_flash_timing_by_tapping_three_flyers() {
    cr!("601.3c");
    assert_supported("Tegwyll's Scouring");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 6);
    let scouring = t.hand(P0, "Tegwyll's Scouring");
    let bird = t.battlefield(P0, "Birds of Paradise");
    t.advance_to(P1, Step::Upkeep);
    // Two flyers aren't enough.
    t.battlefield(P0, "Birds of Paradise");
    let flash_way = |t: &TestGame| {
        t.g.cast_options(P0, scouring)
            .into_iter()
            .find(|o| o.flash)
            .expect("a way to cast it as though it had flash")
            .method
    };
    // Not as a sorcery in the opponent's upkeep, and not with only two flyers.
    assert!(!can_cast_at(&mut t, P0, scouring, &[]));
    t.g.turn.priority = Some(P0);
    assert!(t.g.cast_spell(P0, scouring, flash_way(&t)).is_err());
    t.battlefield(P0, "Birds of Paradise");
    t.battlefield(P1, "Grizzly Bears");
    t.g.turn.priority = Some(P0);
    t.g.cast_spell(P0, scouring, flash_way(&t))
        .expect("cast by tapping three flyers");
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bird)), Zone::Graveyard(P0));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn oath_of_nissa_spends_mana_as_though_any_color_for_planeswalker_spells() {
    cr!("609.4b");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oath of Nissa");
    t.lands(P0, "Forest", 4);
    let jace = t.hand(P0, "Jace Beleren");
    let bolt = t.hand(P0, "Lightning Bolt");
    // A planeswalker spell: {1}{U}{U} with Forests.
    assert!(can_cast_at(&mut t, P0, jace, &[]));
    // Not another spell.
    assert!(!can_cast_at(&mut t, P0, bolt, &[Entity::Player(P1)]));
}

#[test]
fn rootcoil_creeper_mana_only_for_spells_from_your_graveyard() {
    cr!("106.6");
    assert!(card("Rootcoil Creeper")
        .unsupported_text()
        .iter()
        .all(|u| !u.contains("Spend this mana")));
    let mut t = TestGame::new(2);
    let creeper = t.battlefield(P0, "Rootcoil Creeper");
    let think = t.graveyard(P0, "Think Twice");
    let opt = t.hand(P0, "Opt");
    // Two blue mana that can only pay for a spell cast from the graveyard.
    t.answer(P0, DecisionKind::Option, decision::Answer::Index(1));
    t.activate(P0, creeper, 1, &[]).unwrap();
    t.resolve_all();
    assert!(!can_cast_at(&mut t, P0, opt, &[]));
    t.lands(P0, "Island", 1);
    // Flashback {2}{U}: the restricted mana pays for it.
    t.g.turn.priority = Some(P0);
    let think = t.g.current(think);
    t.g.cast_spell(
        P0,
        think,
        CastMethod::Keyword(keywords::KeywordKind::Flashback),
    )
    .expect("flashback from the graveyard");
}

#[test]
fn summer_bloom_plays_up_to_three_additional_lands() {
    cr!("305.2");
    assert_supported("Summer Bloom");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let bloom = t.hand(P0, "Summer Bloom");
    let lands: Vec<ObjectId> = (0..5).map(|_| t.hand(P0, "Forest")).collect();
    t.cast(P0, bloom).go();
    t.resolve_all();
    for l in &lands[..4] {
        t.play_land(P0, *l).expect("a land play");
    }
    assert!(t.play_land(P0, lands[4]).is_err());
}

/// Chooses as many cards with the name as it may (up to the maximum) when choosing among
/// objects; the script answers the other decisions.
struct ChooseNamed {
    name: &'static str,
    other: ScriptedAgent,
}

impl decision::Agent for ChooseNamed {
    fn decide(&mut self, g: &game::Game, p: PlayerId, d: &decision::Decision) -> decision::Answer {
        if let decision::Decision::ChooseEntities {
            candidates, max, ..
        } = d
        {
            // Logged as asked (nothing is queued for it).
            let _ = self.other.decide(g, p, d);
            let named: Vec<Entity> = candidates
                .iter()
                .copied()
                .filter(|e| {
                    e.object()
                        .is_some_and(|o| g.obj(o).chars.name.as_str() == self.name)
                })
                .take(*max as usize)
                .collect();
            return decision::Answer::Entities(named);
        }
        self.other.decide(g, p, d)
    }
}

#[test]
fn collected_conjuring_casts_up_to_two_sorceries_and_puts_the_others_on_the_bottom() {
    cr!("608.2g", "118.9");
    ruling!(
        "Collected Conjuring",
        "You must cast any of the exiled cards you wish to cast while Collected Conjuring is resolving."
    );
    ruling!(
        "Collected Conjuring",
        "Each individual spell you cast this way must have mana value 3 or less."
    );
    assert_supported("Collected Conjuring");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Mountain", 2);
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    let overrun = t.library_top(P0, "Overrun");
    let bears = t.library_top(P0, "Grizzly Bears");
    let mountain = t.library_top(P0, "Mountain");
    let spikes: Vec<ObjectId> = (0..3).map(|_| t.library_top(P0, "Lava Spike")).collect();
    let conjuring = t.hand(P0, "Collected Conjuring");
    // Choose as many Lava Spikes as allowed among the exiled cards (new objects).
    t.g.agents.0.lock().unwrap()[P0.idx()] = Box::new(ChooseNamed {
        name: "Lava Spike",
        other: ScriptedAgent {
            player: P0,
            script: t.script.clone(),
        },
    });
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, conjuring).go();
    t.resolve_all();
    // Two of them, cast for free.
    assert_eq!(t.life(P1), 14, "{}", t.dump_log());
    // Chosen among the sorcery cards with mana value 3 or less: not Overrun (5).
    let candidates = t
        .asked()
        .into_iter()
        .find_map(|(_, d)| match d {
            decision::Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .expect("a choice among the exiled cards");
    assert_eq!(candidates.len(), 3);
    for e in candidates {
        let o = e.object().unwrap();
        assert_eq!(t.g.obj(o).chars.name.as_str(), "Lava Spike");
    }
    // The others went to the bottom: none can be cast later.
    for c in [overrun, bears, mountain] {
        assert_eq!(t.zone(t.g.current(c)), Zone::Library(P0));
    }
    let zones: Vec<Zone> = spikes.iter().map(|c| t.zone(t.g.current(*c))).collect();
    assert_eq!(
        zones.iter().filter(|z| **z == Zone::Graveyard(P0)).count(),
        2
    );
    assert_eq!(zones.iter().filter(|z| **z == Zone::Library(P0)).count(), 1);
    assert!(!t.in_exile("Lava Spike"));
    let top = t.g.library_top(P0).unwrap();
    assert_eq!(t.g.obj(top).chars.name.as_str(), "Island");
}
