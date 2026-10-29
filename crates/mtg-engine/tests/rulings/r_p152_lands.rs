//! Rulings batch P152 — lands that sacrifice themselves as part of an activated ability's
//! cost: targets chosen before costs are paid (CR 601.2c, 602.2b), activation restrictions
//! checked only on activation (CR 602.5b), copies of spells (CR 707.10), and linked
//! exiled cards (CR 607.2a).

use crate::r_p152_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

#[test]
fn ghost_quarter_targeting_itself_doesnt_resolve() {
    cr!("601.2c", "602.2b", "608.2b");
    ruling!(
        "Ghost Quarter",
        "If you target Ghost Quarter with its own ability, the ability won't resolve because its target is no longer on the battlefield."
    );
    supported("Ghost Quarter");
    let mut t = TestGame::new(2);
    let gq = t.battlefield(P0, "Ghost Quarter");
    t.library_top(P0, "Forest");
    act(&mut t, P0, gq, "Destroy target land", &[obj(gq)]).expect("Ghost Quarter");
    assert!(t.in_graveyard(P0, "Ghost Quarter"), "sacrificed as a cost");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(count_named(&t, "Forest") == 0, "no search");
    assert!(t
        .g
        .player(P0)
        .library
        .iter()
        .any(|c| t.g.obj(*c).chars.name == "Forest"));
}

#[test]
fn ghost_quarter_lets_the_controller_search_even_if_the_land_survives() {
    cr!("608.2c", "702.12b");
    ruling!(
        "Ghost Quarter",
        "The target land's controller gets to search for a basic land card even if that land wasn't destroyed"
    );
    supported("Ghost Quarter");
    for indestructible in [false, true] {
        let mut t = TestGame::new(2);
        let gq = t.battlefield(P0, "Ghost Quarter");
        let land = t.battlefield(
            P1,
            if indestructible {
                "Darksteel Citadel"
            } else {
                "Swamp"
            },
        );
        let forest = t.library_top(P1, "Forest");
        act(&mut t, P0, gq, "Destroy target land", &[obj(land)]).expect("Ghost Quarter");
        t.answer_yes(P1, true);
        t.answer_choose(P1, &[obj(forest)]);
        t.resolve_all();
        assert_eq!(t.on_battlefield(land), indestructible);
        assert_eq!(count_named(&t, "Forest"), 1, "searched ({indestructible})");
    }
}

#[test]
fn command_beacon_does_nothing_without_a_commander_in_the_command_zone() {
    cr!("903.3", "608.2c");
    ruling!(
        "Command Beacon",
        "If your commander isn't in the command zone (or you don't have a commander) as the last ability resolves, nothing happens."
    );
    ruling!(
        "Command Beacon",
        "If your commander isn't in the command zone (or you're not playing Commander) as the last ability resolves, nothing happens."
    );
    supported("Command Beacon");
    // In the command zone: put into your hand.
    let mut t = commander_game();
    let cmd = commander(&mut t, P0, "Grizzly Bears");
    let beacon = t.battlefield(P0, "Command Beacon");
    act(&mut t, P0, beacon, "Sacrifice", &[]).expect("Command Beacon");
    // Its owner doesn't put it back into the command zone instead (CR 903.9b).
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.zone(cmd), Zone::Hand(P0));
    // It left the command zone in response: nothing happens.
    let mut t = commander_game();
    let cmd = commander(&mut t, P0, "Grizzly Bears");
    let beacon = t.battlefield(P0, "Command Beacon");
    act(&mut t, P0, beacon, "Sacrifice", &[]).expect("Command Beacon");
    let id = t.g.current(cmd);
    t.g.move_object(id, Zone::Battlefield, mtg_engine::events::MoveCause::Effect, None);
    t.settle();
    t.resolve_all();
    assert_eq!(t.zone(cmd), Zone::Battlefield);
    assert_eq!(t.hand_size(P0), 0);
    // Not playing Commander: a card in the command zone that isn't a commander stays.
    let mut t = TestGame::new(2);
    let card = t.command(P0, "Grizzly Bears");
    let beacon = t.battlefield(P0, "Command Beacon");
    act(&mut t, P0, beacon, "Sacrifice", &[]).expect("Command Beacon");
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Command);
    assert!(t.in_graveyard(P0, "Command Beacon"));
}

#[test]
fn tectonic_edge_checks_the_opponents_lands_only_on_activation() {
    cr!("602.5b");
    ruling!(
        "Tectonic Edge",
        "Tectonic Edge’s second ability checks how many lands an opponent controls only at the time you activate the ability."
    );
    supported("Tectonic Edge");
    let mut t = TestGame::new(2);
    let edge = t.battlefield(P0, "Tectonic Edge");
    t.lands(P0, "Wastes", 1);
    let target = t.battlefield(P1, "Ghost Quarter");
    let basics = t.lands(P1, "Swamp", 3);
    act(&mut t, P0, edge, "Destroy target nonbasic", &[obj(target)]).expect("4 lands");
    // In response, the opponent drops to two lands.
    destroy(&mut t, basics[0]);
    t.resolve_all();
    assert!(!t.on_battlefield(target), "still destroyed");
    // Three lands: it can't be activated.
    let mut t = TestGame::new(2);
    let edge = t.battlefield(P0, "Tectonic Edge");
    t.lands(P0, "Wastes", 1);
    let target = t.battlefield(P1, "Ghost Quarter");
    t.lands(P1, "Swamp", 2);
    assert!(act(&mut t, P0, edge, "Destroy target nonbasic", &[obj(target)]).is_err());
}

#[test]
fn the_world_tree_doesnt_take_away_land_abilities_or_types() {
    cr!("613.1f", "305.6");
    ruling!(
        "The World Tree",
        "The World Tree’s third ability doesn’t cause lands you control to lose any of their abilities."
    );
    supported("The World Tree");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The World Tree");
    let forest = t.battlefield(P0, "Forest");
    t.lands(P0, "Swamp", 3);
    let wastes = t.battlefield(P0, "Wastes");
    t.g.recompute();
    // Six lands: every type of mana, still a Forest that taps for {G}.
    let f = t.g.current(forest);
    let produce = mtg_engine::mana_abilities::could_produce(&t.g, f);
    for ty in [
        ManaType::W,
        ManaType::U,
        ManaType::B,
        ManaType::R,
        ManaType::G,
    ] {
        assert!(produce.contains(&ty), "{ty:?}");
    }
    assert!(t.obj_now(forest).chars.has_subtype("Forest"));
    assert!(t
        .obj_now(forest)
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains("Add {G}")));
    // Wastes keeps its {C} ability and gains no land type.
    let w = t.g.current(wastes);
    assert!(mtg_engine::mana_abilities::could_produce(&t.g, w).contains(&ManaType::C));
    assert!(t.obj_now(wastes).chars.subtypes.is_empty());
}

#[test]
fn grove_of_the_guardian_taps_summoning_sick_creatures_but_not_itself_twice() {
    cr!("302.6", "602.2b");
    ruling!(
        "Grove of the Guardian",
        "you can tap any two untapped creatures you control, including ones you haven’t controlled continuously"
    );
    ruling!(
        "Grove of the Guardian",
        "You can’t tap Grove of the Guardian to activate both abilities at the same time."
    );
    supported("Grove of the Guardian");
    let mut t = TestGame::new(2);
    let grove = t.battlefield(P0, "Grove of the Guardian");
    t.battlefield_sick(P0, "Grizzly Bears");
    t.battlefield_sick(P0, "Grizzly Bears");
    pool(&mut t, P0, &[ManaType::G, ManaType::W], 1);
    pool(&mut t, P0, &[ManaType::C], 3);
    act(&mut t, P0, grove, "Create an 8/8", &[]).expect("tapping two new creatures");
    t.resolve_all();
    assert!(t
        .g
        .permanents()
        .any(|o| o.is_token() && o.power() == 8 && o.toughness() == 8));
    // Tapped for mana, it can't also pay the sacrifice ability's {T}.
    let mut t = TestGame::new(2);
    let grove = t.battlefield(P0, "Grove of the Guardian");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    assert!(act(&mut t, P0, grove, "Create an 8/8", &[]).is_err(), "only 4 other mana");
}

#[test]
fn haunted_fengraf_picks_the_card_at_random_on_resolution() {
    cr!("608.2c");
    ruling!(
        "Haunted Fengraf",
        "The creature card returned to your hand is chosen at random as the ability resolves."
    );
    supported("Haunted Fengraf");
    let mut t = TestGame::new(2);
    let fengraf = t.battlefield(P0, "Haunted Fengraf");
    t.lands(P0, "Wastes", 3);
    t.graveyard(P0, "Shock");
    act(&mut t, P0, fengraf, "Return a creature card", &[]).expect("Haunted Fengraf");
    // A creature card put into the graveyard in response is the only candidate.
    t.graveyard(P0, "Grizzly Bears");
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Shock"));
}

#[test]
fn sequestered_stash_can_put_back_a_card_it_milled() {
    cr!("701.17a", "115.10a");
    ruling!(
        "Sequestered Stash",
        "Sequestered Stash's ability doesn't target the artifact card in your graveyard. You may choose one of the five cards you put there from your library."
    );
    supported("Sequestered Stash");
    let mut t = TestGame::new(2);
    let stash = t.battlefield(P0, "Sequestered Stash");
    t.lands(P0, "Wastes", 4);
    let lib = stack_library(&mut t, P0, &["Grizzly Bears", "Ornithopter", "Shock"]);
    act(&mut t, P0, stash, "Mill five", &[]).expect("Sequestered Stash");
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[obj(lib[1])]);
    t.resolve_all();
    let top = *t.g.player(P0).library.last().unwrap();
    assert_eq!(t.g.obj(top).chars.name, "Ornithopter");
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn scavenger_grounds_exiles_the_desert_sacrificed_for_it() {
    cr!("601.2h", "608.2c");
    ruling!(
        "Scavenger Grounds",
        "The sacrificed Desert will be in your graveyard to be exiled by the last ability of Scavenger Grounds."
    );
    supported("Scavenger Grounds");
    let mut t = TestGame::new(2);
    let grounds = t.battlefield(P0, "Scavenger Grounds");
    t.lands(P0, "Wastes", 2);
    t.graveyard(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(grounds)]);
    act(&mut t, P0, grounds, "Exile all graveyards", &[]).expect("Scavenger Grounds");
    assert!(t.in_graveyard(P0, "Scavenger Grounds"));
    t.resolve_all();
    assert!(t.in_exile("Scavenger Grounds"));
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn mirrorpool_copies_any_instant_or_sorcery_and_the_copy_isnt_cast() {
    cr!("707.10", "707.10c", "601.2i");
    ruling!(
        "Mirrorpool",
        "Mirrorpool’s third ability can target (and copy) any instant or sorcery spell, not just one with targets."
    );
    ruling!(
        "Mirrorpool",
        "The copy is created on the stack, so it’s not cast. Abilities that trigger when a player casts a spell won’t trigger."
    );
    supported("Mirrorpool");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Pyromancer");
    let div = cast_new(&mut t, P0, "Divination", &[]);
    let pool_land = t.battlefield(P0, "Mirrorpool");
    t.resolve(); // Young Pyromancer's trigger for casting Divination.
    assert_eq!(tokens(&t, P0).len(), 1);
    pool(&mut t, P0, &[ManaType::C], 3);
    act(&mut t, P0, pool_land, "Copy target instant", &[obj(div)]).expect("Mirrorpool");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 4, "the copy drew two too");
    assert_eq!(tokens(&t, P0).len(), 1, "no cast trigger for the copy");
}

#[test]
fn mirrorpool_copies_fling_with_the_sacrificed_creatures_power() {
    cr!("707.10", "707.2");
    ruling!(
        "Mirrorpool",
        "effects based on any additional costs that were paid for the original spell are copied as though those same costs were paid for the copy, too."
    );
    supported("Mirrorpool");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(giant)]);
    let fling = cast_new(&mut t, P0, "Fling", &[Entity::Player(P1)]);
    let mp = t.battlefield(P0, "Mirrorpool");
    assert!(t.in_graveyard(P0, "Hill Giant"));
    pool(&mut t, P0, &[ManaType::C], 3);
    act(&mut t, P0, mp, "Copy target instant", &[obj(fling)]).expect("Mirrorpool");
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.life(P1), 14, "3 from the copy, 3 from Fling");
}

#[test]
fn endless_sands_returns_only_the_cards_it_exiled() {
    cr!("607.2a", "400.7");
    ruling!(
        "Endless Sands",
        "The last ability of Endless Sands returns only creature cards exiled by that card. If Endless Sands leaves the battlefield before you activate that ability, the exiled creatures are lost in the dunes forever."
    );
    supported("Endless Sands");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Endless Sands");
    let b = t.battlefield(P0, "Endless Sands");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    pool(&mut t, P0, &[ManaType::C], 2);
    act(&mut t, P0, a, "Exile target creature", &[obj(bears)]).expect("exile");
    t.resolve_all();
    t.g.untap(t.g.current(a));
    pool(&mut t, P0, &[ManaType::C], 2);
    act(&mut t, P0, b, "Exile target creature", &[obj(giant)]).expect("exile");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Hill Giant"));
    // Sands A leaves; Sands B's last ability returns only the Giant.
    destroy(&mut t, a);
    t.g.untap(t.g.current(b));
    pool(&mut t, P0, &[ManaType::C], 4);
    act(&mut t, P0, b, "Return each creature card", &[]).expect("return");
    t.resolve_all();
    assert_eq!(count_named(&t, "Hill Giant"), 1);
    assert!(t.in_exile("Grizzly Bears"), "lost forever");
}

#[test]
fn haven_of_the_spirit_dragon_mana_pays_a_dragons_dash_cost() {
    cr!("106.6", "702.109a");
    ruling!(
        "Haven of the Spirit Dragon",
        "The mana produced by Haven of the Spirit Dragon's second ability can be used to pay for any additional or alternative costs"
    );
    supported("Haven of the Spirit Dragon");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Haven of the Spirit Dragon");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 2);
    let kolaghan = t.hand(P0, "Kolaghan, the Storm's Fury");
    // Dash {3}{R}{R}: five mana, one from the Haven.
    t.cast(P0, kolaghan)
        .method(mtg_engine::object::CastMethod::Keyword(
            mtg_engine::keywords::KeywordKind::Dash,
        ))
        .go();
    t.resolve_all();
    assert_eq!(count_named(&t, "Kolaghan, the Storm's Fury"), 1);
    // The Haven can't pay for a non-Dragon spell.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Haven of the Spirit Dragon");
    t.lands(P0, "Wastes", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
}

#[test]
fn ally_encampment_targets_an_ally_on_the_battlefield() {
    cr!("115.1", "115.4");
    ruling!(
        "Ally Encampment",
        "The last ability can only target an Ally on the battlefield."
    );
    supported("Ally Encampment");
    let mut t = TestGame::new(2);
    let camp = t.battlefield(P0, "Ally Encampment");
    t.lands(P0, "Wastes", 1);
    let ally = t.battlefield(P0, "Kazandu Blademaster");
    let not_ally = t.battlefield(P0, "Grizzly Bears");
    let dead = t.graveyard(P0, "Kazandu Blademaster");
    let n = t.asked().len();
    act(&mut t, P0, camp, "Return target Ally", &[obj(ally)]).expect("Ally Encampment");
    let cands = target_candidates(&t, P0, n);
    assert!(cands[0].contains(&obj(ally)));
    assert!(!cands[0].contains(&obj(not_ally)));
    assert!(!cands[0].contains(&obj(dead)));
    t.resolve_all();
    assert!(t.in_hand(P0, "Kazandu Blademaster"));
}

#[test]
fn lazotep_quarrys_token_copy_exception_is_copiable() {
    cr!("707.9b", "707.2");
    ruling!(
        "Lazotep Quarry",
        "The token is a Zombie instead of its other creature types, and it's black instead of its other colors. These are copiable values"
    );
    supported("Lazotep Quarry");
    let mut t = TestGame::new(2);
    let quarry = t.battlefield(P0, "Lazotep Quarry");
    let card = t.graveyard(P0, "Grizzly Bears");
    pool(&mut t, P0, &[ManaType::C], 4);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[obj(quarry)]);
    act(&mut t, P0, quarry, "Exile target creature card", &[obj(card)]).expect("Quarry");
    t.resolve_all();
    let tok = tokens(&t, P0)[0];
    let check = |t: &TestGame, id: ObjectId| {
        let o = t.obj_now(id);
        assert_eq!((o.power(), o.toughness()), (4, 4));
        assert!(o.chars.has_subtype("Zombie") && !o.chars.has_subtype("Bear"));
        assert_eq!(o.chars.colors, mtg_engine::types::ColorSet::single(mtg_engine::types::Color::Black));
    };
    check(&t, tok);
    // A copy of the token copies the exception.
    let mp = t.battlefield(P0, "Mirrorpool");
    pool(&mut t, P0, &[ManaType::C], 5);
    act(&mut t, P0, mp, "Create a token that's a copy", &[obj(tok)]).expect("Mirrorpool");
    t.resolve_all();
    let copies = tokens(&t, P0);
    assert_eq!(copies.len(), 2);
    for c in copies {
        check(&t, c);
    }
}

#[test]
fn inventors_fair_checks_its_artifact_count_as_upkeep_begins_and_on_activation() {
    cr!("603.4", "602.5b");
    ruling!(
        "Inventors' Fair",
        "No player may take actions in a turn before Inventors' Fair's triggered ability checks to see if it should trigger."
    );
    ruling!(
        "Inventors' Fair",
        "When using Inventors' Fair's activated ability, the number of artifacts you control is checked only as you activate it."
    );
    supported("Inventors' Fair");
    for n in [2, 3] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Inventors' Fair");
        for _ in 0..n {
            t.battlefield(P0, "Ornithopter");
        }
        next_upkeep(&mut t);
        assert_eq!(triggers_on_stack(&t, "gain 1 life"), (n == 3) as usize, "{n}");
        t.resolve_all();
        assert_eq!(t.life(P0), 20 + (n == 3) as i32);
    }
    // Activated with three artifacts, one destroyed in response: it still resolves.
    let mut t = TestGame::new(2);
    let fair = t.battlefield(P0, "Inventors' Fair");
    let thopters: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Ornithopter")).collect();
    t.lands(P0, "Wastes", 4);
    let target = t.library_top(P0, "Sol Ring");
    act(&mut t, P0, fair, "Search your library", &[]).expect("3 artifacts");
    destroy(&mut t, thopters[0]);
    t.answer_choose(P0, &[obj(target)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Sol Ring"));
}

#[test]
fn eden_chooses_its_reflexive_target_after_milling() {
    cr!("603.12");
    ruling!(
        "Eden, Seat of the Sanctum",
        "You don't choose a target for Eden, Seat of the Sanctum's last ability at the time you activate it. Rather, a second \"reflexive\" ability triggers"
    );
    supported("Eden, Seat of the Sanctum");
    let mut t = TestGame::new(2);
    let eden = t.battlefield(P0, "Eden, Seat of the Sanctum");
    t.lands(P0, "Wastes", 5);
    let lib = stack_library(&mut t, P0, &["Grizzly Bears", "Shock"]);
    act(&mut t, P0, eden, "Mill two", &[]).expect("Eden");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(eden)]);
    t.resolve();
    // The reflexive trigger is on the stack; it can target a card just milled.
    let bears = t.g.current(lib[0]);
    assert_eq!(t.g.obj(bears).zone, Zone::Graveyard(P0));
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Eden, Seat of the Sanctum"));
}

#[test]
fn tyrite_sanctum_can_target_a_legendary_god() {
    cr!("115.1");
    ruling!(
        "Tyrite Sanctum",
        "You can activate the middle ability targeting a legendary creature that's already a God. You'll put a +1/+1 counter on it."
    );
    supported("Tyrite Sanctum");
    let mut t = TestGame::new(2);
    let sanctum = t.battlefield(P0, "Tyrite Sanctum");
    t.lands(P0, "Wastes", 2);
    let god = t.battlefield(P0, "Ilharg, the Raze-Boar");
    act(&mut t, P0, sanctum, "becomes a God", &[obj(god)]).expect("Tyrite Sanctum");
    t.resolve_all();
    assert_eq!(t.counters(god, "+1/+1"), 1);
    assert!(t.obj_now(god).chars.has_subtype("God"));
}

#[test]
fn seal_of_primordium_can_target_itself() {
    cr!("601.2c", "602.2b", "608.2b");
    ruling!(
        "Seal of Primordium",
        "You can activate Seal of Primordium's ability targeting Seal of Primordium itself"
    );
    supported("Seal of Primordium");
    let mut t = TestGame::new(2);
    let seal = t.battlefield(P0, "Seal of Primordium");
    let other = t.battlefield(P1, "Ornithopter");
    act(&mut t, P0, seal, "Destroy target", &[obj(seal)]).expect("targets itself");
    assert!(t.in_graveyard(P0, "Seal of Primordium"));
    t.resolve_all();
    assert!(t.on_battlefield(other));
    assert!(t.stack_len() == 0);
}

#[test]
fn safe_haven_tokens_cease_to_exist_and_cards_return_as_new_objects() {
    cr!("111.7", "704.5d", "400.7");
    ruling!(
        "Safe Haven",
        "Token creatures cease to exist when they leave the battlefield, so this effect just exiles them"
    );
    ruling!(
        "Safe Haven",
        "When the creature leaves the battlefield any damage or “will be destroyed at some future time” effects are removed from the creature."
    );
    ruling!(
        "Safe Haven",
        "Creatures with X in the mana cost are treated as X is zero."
    );
    supported("Safe Haven");
    let mut t = TestGame::new(2);
    let haven = t.battlefield(P0, "Safe Haven");
    let tok = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let endless = t.battlefield(P0, "Endless One");
    put_counters(&mut t, endless, "+1/+1", 3);
    damage(&mut t, haven, obj(bears), 1);
    for c in [tok, bears, endless] {
        pool(&mut t, P0, &[ManaType::C], 2);
        let h = t.g.current(haven);
        t.g.untap(h);
        act(&mut t, P0, haven, "Exile target creature", &[obj(c)]).expect("exile");
        t.resolve_all();
    }
    assert!(tokens(&t, P0).is_empty());
    next_upkeep(&mut t);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Safe Haven"));
    assert!(tokens(&t, P0).is_empty(), "the token didn't come back");
    let b = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(b.len(), 1);
    assert_eq!(t.g.obj(b[0]).damage, 0);
    // Endless One returns with X = 0: a 0/0 that dies.
    assert!(t.in_graveyard(P0, "Endless One"));
}

#[test]
fn pilot_tokens_from_roads_dont_change_power_when_crewing() {
    cr!("702.122a");
    ruling!(
        "Foul Roads",
        "Saddling a Mount or crewing a Vehicle doesn’t cause the token’s power to change."
    );
    ruling!(
        "Roadside Assistance",
        "Saddling a Mount or crewing a Vehicle doesn’t cause the token’s power to change."
    );
    supported("Foul Roads");
    supported("Roadside Assistance");
    for roads in [true, false] {
        let mut t = TestGame::new(2);
        if roads {
            let fr = t.battlefield(P0, "Foul Roads");
            pool(&mut t, P0, &[ManaType::B], 2);
            act(&mut t, P0, fr, "Create a 1/1", &[]).expect("Foul Roads");
        } else {
            let bears = t.battlefield(P0, "Grizzly Bears");
            let aura = t.hand(P0, "Roadside Assistance");
            pool(&mut t, P0, &[ManaType::W], 3);
            t.cast(P0, aura).target(obj(bears)).go();
        }
        t.resolve_all();
        let pilot = tokens(&t, P0)[0];
        t.g.objects[pilot.0 as usize].summoning_sick = false;
        let wl = t.battlefield(P0, "Weatherlight");
        assert!(crate::r_s04_common::crew(&mut t, P0, wl, &[pilot]), "crew 3");
        t.resolve_all();
        assert!(t.obj_now(wl).is(CardType::Creature));
        assert_eq!(t.pt(pilot), (1, 1));
    }
}

#[test]
fn cactus_preserve_follows_changes_to_your_lands() {
    cr!("106.7", "305.7");
    ruling!(
        "Cactus Preserve",
        "Any change to a land’s type or abilities gained by a land can affect the types of mana a land can produce."
    );
    supported("Cactus Preserve");
    let mut t = TestGame::new(2);
    let cactus = t.battlefield(P0, "Cactus Preserve");
    let forest = t.battlefield(P0, "Forest");
    let c = t.g.current(cactus);
    let before = mtg_engine::mana_abilities::could_produce(&t.g, c);
    assert!(before.contains(&ManaType::G) && !before.contains(&ManaType::U));
    // Spreading Seas makes the Forest an Island.
    let seas = t.hand(P1, "Spreading Seas");
    t.lands(P1, "Island", 3);
    t.g.turn.active = P1;
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, seas).target(obj(forest)).go();
    t.resolve_all();
    let c = t.g.current(cactus);
    let after = mtg_engine::mana_abilities::could_produce(&t.g, c);
    assert!(after.contains(&ManaType::U) && !after.contains(&ManaType::G));
}
