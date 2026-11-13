//! Rulings batch P068 — casting exiled cards "without paying their mana costs" (CR 118.9):
//! no alternative cost can be combined with it (CR 118.9a), optional additional costs such
//! as kicker can be paid and mandatory ones must be (CR 601.2b, 601.2f). One scenario per
//! card that lets a player cast a card this way, run with a kicker spell, a spell with a
//! mandatory additional cost and a spell with an alternative cost of the kind the card can
//! cast.

use crate::r_p068_common::*;
use crate::r_s01_common::{attack_with, supported};
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType::*;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The cards that let a player cast a card without paying its mana cost.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Via {
    BringToLight,
    MindsDilation,
    SolsticeRevelations,
    ChaosWand,
    Jeleva,
    DauthiVoidwalker,
    Jace,
    CollectedConjuring,
    ReleaseToTheWind,
    Etrata,
    SynthesisPod,
    KeldonFlamesage,
    Ashiok,
    MonstrousVortex,
    HeraldOfAmity,
    Crabomination,
    IdolOfEndurance,
    LongRangeSensor,
    JacesMindseeker,
    SilentBladeOni,
}

/// The kinds of cards a card can cast.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    /// Instants of mana value 3 or less.
    Instant,
    /// Instants and sorceries of mana value 2 or less (Keldon Flamesage's power).
    Small,
    Sorcery,
    Creature,
    Aura,
}

impl Via {
    fn name(self) -> &'static str {
        match self {
            Via::BringToLight => "Bring to Light",
            Via::MindsDilation => "Mind's Dilation",
            Via::SolsticeRevelations => "Solstice Revelations",
            Via::ChaosWand => "Chaos Wand",
            Via::Jeleva => "Jeleva, Nephalia's Scourge",
            Via::DauthiVoidwalker => "Dauthi Voidwalker",
            Via::Jace => "Jace, Architect of Thought",
            Via::CollectedConjuring => "Collected Conjuring",
            Via::ReleaseToTheWind => "Release to the Wind",
            Via::Etrata => "Etrata, Deadly Fugitive",
            Via::SynthesisPod => "Synthesis Pod",
            Via::KeldonFlamesage => "Keldon Flamesage",
            Via::Ashiok => "Ashiok, Nightmare Muse",
            Via::MonstrousVortex => "Monstrous Vortex",
            Via::HeraldOfAmity => "Herald of Amity",
            Via::Crabomination => "Crabomination",
            Via::IdolOfEndurance => "Idol of Endurance",
            Via::LongRangeSensor => "Long-Range Sensor",
            Via::JacesMindseeker => "Jace's Mindseeker",
            Via::SilentBladeOni => "Silent-Blade Oni",
        }
    }

    fn kind(self) -> Kind {
        match self {
            Via::KeldonFlamesage => Kind::Small,
            Via::CollectedConjuring | Via::JacesMindseeker => Kind::Sorcery,
            Via::ReleaseToTheWind | Via::IdolOfEndurance | Via::SilentBladeOni => Kind::Creature,
            Via::HeraldOfAmity => Kind::Aura,
            _ => Kind::Instant,
        }
    }

    /// Whether the card is cast later with a permission (rather than during a resolution).
    fn permission(self) -> bool {
        matches!(
            self,
            Via::DauthiVoidwalker | Via::ReleaseToTheWind | Via::IdolOfEndurance
        )
    }

    /// Puts `spell` where the card finds it and queues the answers asked before the
    /// spell is cast. Returns the spell card.
    fn setup(self, t: &mut TestGame, spell: &str) -> ObjectId {
        let p0 = P0;
        match self {
            Via::BringToLight => {
                let c = t.library_top(p0, spell);
                t.hand(p0, "Bring to Light");
                t.answer_choose(p0, &[o(c)]);
                t.answer_yes(p0, true);
                c
            }
            Via::MindsDilation => {
                t.battlefield(p0, "Mind's Dilation");
                let c = t.library_top(P1, spell);
                t.answer_yes(p0, true);
                c
            }
            Via::SolsticeRevelations => {
                let c = t.library_top(p0, spell);
                t.hand(p0, "Solstice Revelations");
                t.lands(p0, "Mountain", 4);
                t.answer_yes(p0, true);
                c
            }
            Via::ChaosWand => {
                t.battlefield(p0, "Chaos Wand");
                let c = t.library_top(P1, spell);
                t.answer_yes(p0, true);
                c
            }
            Via::Jeleva => {
                let c = t.library_top(p0, spell);
                t.hand(p0, "Jeleva, Nephalia's Scourge");
                // Asked as Jeleva's attack trigger resolves.
                t.answer_choose_moved(p0, &[o(c)]);
                t.answer_yes(p0, true);
                c
            }
            Via::DauthiVoidwalker => {
                t.battlefield(p0, "Dauthi Voidwalker");
                // The card is put into P1's graveyard: exiled with a void counter instead.
                let c = t.hand(P1, spell);
                let c = move_to(t, c, Zone::Graveyard(P1));
                assert!(exiled(t, c), "{}", t.dump_log());
                assert_eq!(t.counters(c, "void"), 1);
                t.answer_choose(p0, &[o(c)]);
                c
            }
            Via::Jace => {
                let jace = t.battlefield(p0, "Jace, Architect of Thought");
                t.g.obj_mut(jace).counters.insert("loyalty".into(), 8);
                let c = t.library_top(p0, spell);
                t.answer_choose(p0, &[o(c)]);
                t.answer_choose(p0, &[]);
                t.answer_yes(p0, true);
                c
            }
            Via::CollectedConjuring => {
                let c = t.library_top(p0, spell);
                t.hand(p0, "Collected Conjuring");
                t.answer_choose_moved(p0, &[o(c)]);
                t.answer_yes(p0, true);
                c
            }
            Via::ReleaseToTheWind => {
                let c = t.battlefield(p0, spell);
                t.hand(p0, "Release to the Wind");
                c
            }
            Via::Etrata => {
                t.battlefield(p0, "Etrata, Deadly Fugitive");
                let c = t.library_top(p0, spell);
                t.enter(p0, "Cryptic Coat");
                t.resolve_all();
                assert!(t.obj_now(c).face_down, "cloaked");
                t.answer_yes(p0, true);
                c
            }
            Via::SynthesisPod => {
                t.battlefield(p0, "Synthesis Pod");
                let c = t.library_top(P1, spell);
                t.answer_yes(p0, true);
                c
            }
            Via::KeldonFlamesage => {
                t.battlefield(p0, "Keldon Flamesage");
                let c = t.library_top(p0, spell);
                // No enlisting.
                t.answer_choose(p0, &[]);
                t.answer_choose_moved(p0, &[o(c)]);
                t.answer_yes(p0, true);
                c
            }
            Via::Ashiok => {
                let a = t.battlefield(p0, "Ashiok, Nightmare Muse");
                t.g.obj_mut(a).counters.insert("loyalty".into(), 7);
                let c = t.exile(P1, spell);
                t.answer_choose(p0, &[o(c)]);
                t.answer_yes(p0, true);
                c
            }
            Via::MonstrousVortex => {
                t.battlefield(p0, "Monstrous Vortex");
                let c = t.library_top(p0, spell);
                t.hand(p0, "Colossal Dreadmaw");
                t.answer_yes(p0, true);
                c
            }
            Via::HeraldOfAmity => {
                let c = t.library_top(p0, spell);
                t.answer_choose_moved(p0, &[o(c)]);
                t.answer_yes(p0, true);
                c
            }
            Via::Crabomination => {
                let c = t.library_top(P1, spell);
                t.answer_targets(p0, &[Entity::Player(P1)]);
                t.answer_choose_moved(p0, &[o(c)]);
                t.answer_yes(p0, true);
                c
            }
            Via::IdolOfEndurance => t.graveyard(p0, spell),
            Via::LongRangeSensor => {
                let s = t.battlefield(p0, "Long-Range Sensor");
                t.g.obj_mut(s).counters.insert("charge".into(), 2);
                let c = t.library_top(p0, spell);
                t.answer_yes(p0, true);
                c
            }
            Via::JacesMindseeker => {
                let c = t.library_top(P1, spell);
                t.answer_targets(p0, &[Entity::Player(P1)]);
                t.answer_choose_moved(p0, &[o(c)]);
                t.answer_yes(p0, true);
                c
            }
            Via::SilentBladeOni => {
                t.battlefield(p0, "Silent-Blade Oni");
                let c = t.hand(P1, spell);
                t.answer_choose(p0, &[o(c)]);
                t.answer_yes(p0, true);
                c
            }
        }
    }

    /// Runs the card so that `spell` is cast (the spell's own answers are queued), and
    /// resolves everything.
    fn go(self, t: &mut TestGame, spell: ObjectId) {
        let p0 = P0;
        let find = |t: &TestGame, name: &str| -> ObjectId {
            *t.g.player(p0)
                .hand
                .iter()
                .chain(t.g.battlefield.iter())
                .find(|x| t.g.obj(**x).chars.name == name)
                .unwrap_or_else(|| panic!("{name} not found"))
        };
        match self {
            Via::BringToLight => {
                let btl = find(t, "Bring to Light");
                pool(t, p0, &[W, U, B, R, G]);
                t.cast(p0, btl).go();
            }
            Via::MindsDilation => {
                t.set_step(P1, Step::PrecombatMain);
                let orni = t.hand(P1, "Ornithopter");
                t.cast(P1, orni).go();
            }
            Via::SolsticeRevelations => {
                let sr = find(t, "Solstice Revelations");
                pool(t, p0, &[R, C, C]);
                t.cast(p0, sr).go();
            }
            Via::ChaosWand => {
                let w = find(t, "Chaos Wand");
                mana(t, p0, C, 4);
                t.activate(p0, w, 0, &[Entity::Player(P1)]).unwrap();
            }
            Via::Jeleva => {
                let j = find(t, "Jeleva, Nephalia's Scourge");
                pool(t, p0, &[U, B, R, C]);
                t.cast(p0, j).go();
                t.resolve_all();
                assert!(exiled(t, spell), "{}", t.dump_log());
                let j = t.g.current(j);
                t.g.obj_mut(j).summoning_sick = false;
                attack_with(t, &[(j, Entity::Player(P1))]);
            }
            Via::DauthiVoidwalker => {
                let d = find(t, "Dauthi Voidwalker");
                t.activate(p0, d, 0, &[]).unwrap();
                t.resolve_all();
                assert!(exiled(t, spell));
                assert_eq!(methods(t, p0, spell), vec![CastMethod::Free]);
                cast_free(t, p0, spell);
            }
            Via::Jace => {
                let jace = find(t, "Jace, Architect of Thought");
                t.activate(p0, jace, 2, &[]).unwrap();
            }
            Via::CollectedConjuring => {
                let cc = find(t, "Collected Conjuring");
                pool(t, p0, &[U, R, C, C]);
                t.cast(p0, cc).go();
            }
            Via::ReleaseToTheWind => {
                let rw = find(t, "Release to the Wind");
                pool(t, p0, &[U, C, C]);
                t.cast(p0, rw).target(spell).go();
                t.resolve_all();
                assert!(exiled(t, spell));
                assert_eq!(methods(t, p0, spell), vec![CastMethod::Free]);
                cast_free(t, p0, spell);
            }
            Via::Etrata => {
                let fd = t.g.current(spell);
                pool(t, p0, &[U, B, C, C]);
                t.activate(p0, fd, 0, &[]).unwrap();
            }
            Via::SynthesisPod => {
                let pod = find(t, "Synthesis Pod");
                // A spell of mana value one less than the card's.
                let mv = t.obj_now(spell).chars.mana_value();
                let fodder = t.hand(p0, if mv == 1 { "Ornithopter" } else { "Grizzly Bears" });
                if mv != 1 {
                    pool(t, p0, &[G, C]);
                }
                let fs = t.cast(p0, fodder).go();
                pool(t, p0, &[U, C]);
                t.answer_choose(p0, &[o(fs)]);
                t.activate(p0, pod, 0, &[Entity::Player(P1)]).unwrap();
            }
            Via::KeldonFlamesage => {
                let k = find(t, "Keldon Flamesage");
                attack_with(t, &[(k, Entity::Player(P1))]);
            }
            Via::Ashiok => {
                let a = find(t, "Ashiok, Nightmare Muse");
                t.activate(p0, a, 2, &[]).unwrap();
            }
            Via::MonstrousVortex => {
                let d = find(t, "Colossal Dreadmaw");
                pool(t, p0, &[G, G, C, C, C, C]);
                t.cast(p0, d).go();
            }
            Via::HeraldOfAmity => {
                t.enter(p0, "Herald of Amity");
            }
            Via::Crabomination => {
                t.enter(p0, "Crabomination");
            }
            Via::IdolOfEndurance => {
                let idol = t.enter(p0, "Idol of Endurance");
                t.resolve_all();
                assert!(exiled(t, spell), "{}", t.dump_log());
                pool(t, p0, &[W, C]);
                t.activate(p0, idol, 0, &[]).unwrap();
                t.resolve_all();
                assert_eq!(methods(t, p0, spell), vec![CastMethod::Free]);
                cast_free(t, p0, spell);
            }
            Via::LongRangeSensor => {
                let s = find(t, "Long-Range Sensor");
                mana(t, p0, C, 1);
                t.activate(p0, s, 0, &[]).unwrap();
            }
            Via::JacesMindseeker => {
                t.enter(p0, "Jace's Mindseeker");
            }
            Via::SilentBladeOni => {
                let oni = find(t, "Silent-Blade Oni");
                attack_with(t, &[(oni, Entity::Player(P1))]);
                t.advance_to(p0, Step::CombatDamage);
            }
        }
        t.resolve_all();
    }
}

/// The spell with kicker the card can cast, and how to check it was kicked.
fn kicker(t: &mut TestGame, via: Via) {
    let name = via.name();
    let spell = match via.kind() {
        Kind::Instant | Kind::Small => "Burst Lightning",
        Kind::Sorcery => "Saproling Migration",
        Kind::Creature => "Kavu Titan",
        Kind::Aura => "Gigantiform",
    };
    let card = via.setup(t, spell);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 4);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    match via.kind() {
        Kind::Instant | Kind::Small => {
            t.answer_targets(P0, &[Entity::Player(P1)]);
        }
        Kind::Aura => {
            t.answer_targets(P0, &[o(bears)]);
            t.answer_yes(P0, false);
        }
        _ => {}
    }
    let from = t.asked().len();
    via.go(t, card);
    let log = t.dump_log();
    assert!(!asked_method(t, from), "{name}");
    // Kicker {4}, or Kavu Titan's {2}{G}.
    let kick = if via.kind() == Kind::Creature { 3 } else { 4 };
    assert_eq!(tapped_lands(t, P0), kick, "{name}: the kicker was paid: {log}");
    match via.kind() {
        Kind::Instant | Kind::Small => assert_eq!(t.life(P1), 16, "{name}: {log}"),
        Kind::Sorcery => {
            assert_eq!(t.named_on_battlefield("Saproling Token").len(), 4, "{name}: {log}")
        }
        Kind::Creature => {
            let k = t.named_on_battlefield("Kavu Titan")[0];
            assert_eq!(t.counters(k, "+1/+1"), 3, "{name}: {log}");
        }
        Kind::Aura => assert_eq!(t.pt(bears), (8, 8), "{name}: {log}"),
    }
}

/// The number of cards P0 drew (library fillers in hand).
fn drawn(t: &TestGame) -> usize {
    t.g.player(P0)
        .hand
        .iter()
        .filter(|x| t.g.obj(**x).chars.name == "Filler")
        .count()
}

/// The spell with a mandatory additional cost the card can cast; it's paid.
fn mandatory(t: &mut TestGame, via: Via) {
    let name = via.name();
    let spell = match via.kind() {
        Kind::Instant | Kind::Small => "Village Rites",
        Kind::Sorcery => "Tormenting Voice",
        Kind::Creature => "Silvergill Adept",
        Kind::Aura => "Grafted Identity",
    };
    let card = via.setup(t, spell);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mut discard = None;
    match via.kind() {
        Kind::Instant | Kind::Small => {
            t.answer_choose(P0, &[o(elves)]);
        }
        Kind::Sorcery => {
            let m = t.hand(P0, "Mountain");
            discard = Some(m);
            t.answer_choose(P0, &[o(m)]);
        }
        Kind::Creature => {
            t.lands(P0, "Island", 3);
        }
        Kind::Aura => {
            t.answer_choose(P0, &[o(elves)]);
            t.answer_targets(P0, &[o(theirs)]);
        }
    }
    via.go(t, card);
    let log = t.dump_log();
    match via.kind() {
        Kind::Instant | Kind::Small => {
            assert!(t.in_graveyard(P0, "Llanowar Elves"), "{name}: {log}");
            assert_eq!(drawn(t), 2, "{name}: {log}");
        }
        Kind::Sorcery => {
            assert_eq!(t.zone(discard.unwrap()), Zone::Graveyard(P0), "{name}: {log}");
            assert_eq!(drawn(t), 2, "{name}: {log}");
        }
        Kind::Creature => {
            assert_eq!(tapped_lands(t, P0), 3, "{name}: {{3}} was paid: {log}");
            assert_eq!(t.named_on_battlefield("Silvergill Adept").len(), 1, "{name}");
        }
        Kind::Aura => {
            assert!(t.in_graveyard(P0, "Llanowar Elves"), "{name}: {log}");
            assert_eq!(t.obj_now(theirs).controller, P0, "{name}: {log}");
        }
    }
}

/// The spell with an alternative cost the card can cast: it isn't offered.
fn alternative(t: &mut TestGame, via: Via) {
    let name = via.name();
    let spell = match via.kind() {
        // "If you control a Forest, rather than pay this spell's mana cost, you may have an
        // opponent gain 3 life."
        Kind::Instant => "Invigorate",
        // Overload {4}{R}.
        Kind::Small | Kind::Sorcery => "Vandalblast",
        // Evoke {W}.
        Kind::Creature => "Wispmare",
        // "If you control a Plains, you may tap an untapped creature you control rather
        // than pay this spell's mana cost."
        Kind::Aura => "Lashknife",
    };
    let card = via.setup(t, spell);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let orni = [t.battlefield(P1, "Ornithopter"), t.battlefield(P1, "Ornithopter")];
    t.battlefield(P0, "Forest");
    t.battlefield(P0, "Plains");
    t.lands(P0, "Mountain", 5);
    match via.kind() {
        Kind::Instant | Kind::Aura => {
            t.answer_targets(P0, &[o(elves)]);
        }
        Kind::Small | Kind::Sorcery => {
            t.answer_targets(P0, &[o(orni[0])]);
        }
        Kind::Creature => {}
    }
    let from = t.asked().len();
    via.go(t, card);
    let log = t.dump_log();
    assert!(!asked_method(t, from), "{name}: {log}");
    match via.kind() {
        Kind::Instant => {
            assert_eq!(t.life(P1), 20, "{name}: {log}");
            assert_eq!(t.pt(elves), (5, 5), "{name}: {log}");
        }
        Kind::Small | Kind::Sorcery => {
            assert!(!t.on_battlefield(orni[0]), "{name}: {log}");
            assert!(t.on_battlefield(orni[1]), "{name}: not overloaded: {log}");
        }
        Kind::Creature => {
            assert_eq!(t.named_on_battlefield("Wispmare").len(), 1, "{name}: not evoked: {log}")
        }
        Kind::Aura => {
            assert!(!t.obj_now(elves).tapped, "{name}: {log}");
            assert_eq!(t.named_on_battlefield("Lashknife").len(), 1, "{name}: {log}");
        }
    }
    assert_eq!(tapped_lands(t, P0), 0, "{name}: nothing paid: {log}");
}

fn check_all(via: Via) {
    supported(via.name());
    for f in [kicker, mandatory, alternative] {
        let mut t = TestGame::new(2);
        f(&mut t, via);
    }
}

#[test]
fn spells_cast_during_a_resolution_without_paying_their_mana_costs() {
    cr!("118.9", "118.9a", "601.2b", "601.2f", "608.2g");
    ruling!(
        "Bring to Light",
        "If you cast a card \"without paying its mana cost,\" you can't choose to cast it for any alternative costs, such as awaken costs. You can, however, pay additional costs, such as kicker costs. If the card has any mandatory additional costs, you must pay those to cast the card."
    );
    ruling!(
        "Mind's Dilation",
        "If you cast a card \"without paying its mana cost,\" you can't choose to cast it for any alternative costs, such as emerge costs. You can, however, pay additional costs, such as escalate costs. If the card has any mandatory additional costs, you must pay those."
    );
    ruling!(
        "Solstice Revelations",
        "If you cast a card \"without paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, such as sacrificing a creature, those must be paid to cast the card."
    );
    ruling!(
        "Chaos Wand",
        "If you cast a card \"without paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, such as that of Tormenting Voice, those must be paid to cast the card."
    );
    ruling!(
        "Jeleva, Nephalia's Scourge",
        "If you cast a card \"without paying its mana cost,\" you can't pay alternative costs such as overload costs. However, you can pay additional costs such as kicker costs. If the card has mandatory additional costs, you must pay those."
    );
    ruling!(
        "Jace, Architect of Thought",
        "If you cast a card “without paying its mana cost,” you can't pay alternative costs such as overload costs. You can pay additional costs such as kicker costs. If the card has mandatory additional costs, you must pay those."
    );
    ruling!(
        "Collected Conjuring",
        "If you cast a card “without paying its mana cost,” you can’t choose to cast it for any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs (such as that of Tormenting Voice), you must pay those to cast the card."
    );
    ruling!(
        "Etrata, Deadly Fugitive",
        "If you cast a spell \"without paying its mana cost\", you can't choose to cast it for any alternative costs. You can, however, pay additional costs, such as kicker costs. If the card has any mandatory additional costs, such as that of Demand Answers, those must be paid to cast the spell."
    );
    ruling!(
        "Synthesis Pod",
        "If you cast a spell \"without paying its mana cost,\" you can't choose to cast it for any alternative costs. If the card has any mandatory additional costs, those must be paid to cast the card."
    );
    ruling!(
        "Keldon Flamesage",
        "If you cast a spell \"without paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs, such as a kicker cost. If the card has any mandatory additional costs, you must pay those to cast the spell."
    );
    ruling!(
        "Ashiok, Nightmare Muse",
        "If you cast a spell \"without paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs, such as kicker costs. If the card has any mandatory additional costs, such as that of Thrill of Possibility, those must be paid to cast the card."
    );
    ruling!(
        "Monstrous Vortex",
        "If you cast a spell \"without paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the spell has any mandatory additional costs, you must pay those to cast it."
    );
    ruling!(
        "Herald of Amity",
        "If you cast a spell \"without paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the spell has any mandatory additional costs, you must pay those to cast the spell."
    );
    ruling!(
        "Crabomination",
        "If you cast a spell for another cost \"without paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs, such as kicker costs. If the spell has any mandatory additional costs, those must be paid to cast it."
    );
    ruling!(
        "Long-Range Sensor",
        "If you cast a spell “without paying its mana cost,” you can’t choose to cast it for any alternative costs. You can, however, pay additional costs. If the spell has any mandatory additional costs, you must pay those to cast it."
    );
    for via in [
        Via::BringToLight,
        Via::MindsDilation,
        Via::SolsticeRevelations,
        Via::ChaosWand,
        Via::Jeleva,
        Via::Jace,
        Via::CollectedConjuring,
        Via::Etrata,
        Via::SynthesisPod,
        Via::KeldonFlamesage,
        Via::Ashiok,
        Via::MonstrousVortex,
        Via::HeraldOfAmity,
        Via::Crabomination,
        Via::LongRangeSensor,
    ] {
        check_all(via);
    }
}

#[test]
fn spells_cast_from_an_opponents_cards_without_paying_their_mana_costs() {
    cr!("118.9", "118.9a", "601.2b", "601.2f", "608.2g");
    // Jace's Mindseeker (an opponent mills five; cast an instant or sorcery among them),
    // with Vandalblast's overload as the alternative cost.
    ruling!(
        "Jace's Mindseeker",
        "If you cast a card “without paying its mana cost,” you can’t pay alternative costs such as overload costs. You can pay additional costs such as kicker costs. If the card has mandatory additional costs, you must pay those."
    );
    // Silent-Blade Oni (cast a spell from the damaged player's hand), with Wispmare's evoke
    // and Kavu Titan's kicker ({X} and Fling: `r_p175_free_casting.rs`).
    ruling!(
        "Silent-Blade Oni",
        "If you cast a card without paying its mana cost, you can't choose to cast it for any alternative costs, such as evoke or the alternative cost provided by the morph ability."
    );
    // Its other ruling's kicker part (its {X} and Fling parts: `r_p175_free_casting.rs`).
    ruling!(
        "Silent-Blade Oni",
        "However, you can pay optional additional costs, such as kicker, and you must still pay mandatory additional costs"
    );
    for via in [Via::JacesMindseeker, Via::SilentBladeOni] {
        check_all(via);
    }
}

#[test]
fn cards_cast_later_without_paying_their_mana_costs() {
    cr!("118.9", "118.9a", "601.2b", "601.2f");
    ruling!(
        "Dauthi Voidwalker",
        "If you cast a card this way, you may not cast it for any other alternative costs it has, but you may pay for additional costs, such as kicker costs. If the spell requires an additional cost, you must pay that cost."
    );
    ruling!(
        "Release to the Wind",
        "If you cast a card “without paying its mana cost,” you can’t pay any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, such as that of Silvergill Adept, those must be paid to cast the card."
    );
    ruling!(
        "Idol of Endurance",
        "If you cast a spell “without paying its mana cost,” you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, those must be paid to cast the spell."
    );
    for via in [
        Via::DauthiVoidwalker,
        Via::ReleaseToTheWind,
        Via::IdolOfEndurance,
    ] {
        assert!(via.permission());
        check_all(via);
    }
}
