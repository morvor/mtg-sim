//! Keyword abilities (CR 702).
//!
//! [`KeywordKind`] enumerates every keyword ability defined in CR 702 (generated from
//! the rules text). A [`Keyword`] is an instance on an object, carrying its parameters
//! (e.g. the cost of Equip, the N of Bushido N, what Protection is from).

use crate::ability::{Cost, Filter, Value};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// A keyword ability instance with its parameters.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Keyword {
    pub kind: KeywordKind,
    /// Numeric parameter: Bushido N, Rampage N, Annihilator N, Toxic N, Modular N, ...
    pub n: Option<i32>,
    /// Cost parameter: Equip {2}, Ward {2}, Kicker {1}{R}, Cycling {2}, Flashback {3}{R}, ...
    pub cost: Option<Cost>,
    /// Additional costs (e.g. a second kicker cost).
    pub costs: Vec<Cost>,
    /// Filter parameter: Enchant [creature], Protection from [red], Landwalk [Island],
    /// Hexproof from [black], Affinity for [artifacts], Equip [legendary creature].
    pub filter: Option<Filter>,
    /// Raw parameter text (e.g. "Islandwalk", "Partner with Brallin", "Swampcycling").
    pub text: Option<SmolStr>,
    /// What X in the cost is equal to ("ward {X}, where X is ...", CR 702.21b), for a
    /// keyword whose ability determines X as it resolves; `costs` then holds the cost
    /// with X, while `cost` shows its current value (CR 702.1b).
    #[serde(default)]
    pub x: Option<Value>,
}

impl Keyword {
    pub fn new(kind: KeywordKind) -> Keyword {
        Keyword {
            kind,
            n: None,
            cost: None,
            costs: vec![],
            filter: None,
            text: None,
            x: None,
        }
    }
    pub fn with_n(kind: KeywordKind, n: i32) -> Keyword {
        Keyword {
            n: Some(n),
            ..Keyword::new(kind)
        }
    }
    pub fn with_cost(kind: KeywordKind, cost: Cost) -> Keyword {
        Keyword {
            cost: Some(cost),
            ..Keyword::new(kind)
        }
    }
    pub fn with_filter(kind: KeywordKind, filter: Filter) -> Keyword {
        Keyword {
            filter: Some(filter),
            ..Keyword::new(kind)
        }
    }
    pub fn text(mut self, t: impl Into<SmolStr>) -> Keyword {
        self.text = Some(t.into());
        self
    }
}

impl From<KeywordKind> for Keyword {
    fn from(k: KeywordKind) -> Self {
        Keyword::new(k)
    }
}

// @generated from CR 702 by the bootstrap script; one variant per keyword ability.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum KeywordKind {
    /// CR 702.2 Deathtouch
    Deathtouch,
    /// CR 702.3 Defender
    Defender,
    /// CR 702.4 Double Strike
    DoubleStrike,
    /// CR 702.5 Enchant
    Enchant,
    /// CR 702.6 Equip
    Equip,
    /// CR 702.7 First Strike
    FirstStrike,
    /// CR 702.8 Flash
    Flash,
    /// CR 702.9 Flying
    Flying,
    /// CR 702.10 Haste
    Haste,
    /// CR 702.11 Hexproof
    Hexproof,
    /// CR 702.12 Indestructible
    Indestructible,
    /// CR 702.13 Intimidate
    Intimidate,
    /// CR 702.14 Landwalk
    Landwalk,
    /// CR 702.15 Lifelink
    Lifelink,
    /// CR 702.16 Protection
    Protection,
    /// CR 702.17 Reach
    Reach,
    /// CR 702.18 Shroud
    Shroud,
    /// CR 702.19 Trample
    Trample,
    /// CR 702.20 Vigilance
    Vigilance,
    /// CR 702.21 Ward
    Ward,
    /// CR 702.22 Banding
    Banding,
    /// CR 702.23 Rampage
    Rampage,
    /// CR 702.24 Cumulative Upkeep
    CumulativeUpkeep,
    /// CR 702.25 Flanking
    Flanking,
    /// CR 702.26 Phasing
    Phasing,
    /// CR 702.27 Buyback
    Buyback,
    /// CR 702.28 Shadow
    Shadow,
    /// CR 702.29 Cycling
    Cycling,
    /// CR 702.30 Echo
    Echo,
    /// CR 702.31 Horsemanship
    Horsemanship,
    /// CR 702.32 Fading
    Fading,
    /// CR 702.33 Kicker
    Kicker,
    /// CR 702.34 Flashback
    Flashback,
    /// CR 702.35 Madness
    Madness,
    /// CR 702.36 Fear
    Fear,
    /// CR 702.37 Morph
    Morph,
    /// CR 702.38 Amplify
    Amplify,
    /// CR 702.39 Provoke
    Provoke,
    /// CR 702.40 Storm
    Storm,
    /// CR 702.41 Affinity
    Affinity,
    /// CR 702.42 Entwine
    Entwine,
    /// CR 702.43 Modular
    Modular,
    /// CR 702.44 Sunburst
    Sunburst,
    /// CR 702.45 Bushido
    Bushido,
    /// CR 702.46 Soulshift
    Soulshift,
    /// CR 702.47 Splice
    Splice,
    /// CR 702.48 Offering
    Offering,
    /// CR 702.49 Ninjutsu
    Ninjutsu,
    /// CR 702.50 Epic
    Epic,
    /// CR 702.51 Convoke
    Convoke,
    /// CR 702.52 Dredge
    Dredge,
    /// CR 702.53 Transmute
    Transmute,
    /// CR 702.54 Bloodthirst
    Bloodthirst,
    /// CR 702.55 Haunt
    Haunt,
    /// CR 702.56 Replicate
    Replicate,
    /// CR 702.57 Forecast
    Forecast,
    /// CR 702.58 Graft
    Graft,
    /// CR 702.59 Recover
    Recover,
    /// CR 702.60 Ripple
    Ripple,
    /// CR 702.61 Split Second
    SplitSecond,
    /// CR 702.62 Suspend
    Suspend,
    /// CR 702.63 Vanishing
    Vanishing,
    /// CR 702.64 Absorb
    Absorb,
    /// CR 702.65 Aura Swap
    AuraSwap,
    /// CR 702.66 Delve
    Delve,
    /// CR 702.67 Fortify
    Fortify,
    /// CR 702.68 Frenzy
    Frenzy,
    /// CR 702.69 Gravestorm
    Gravestorm,
    /// CR 702.70 Poisonous
    Poisonous,
    /// CR 702.71 Transfigure
    Transfigure,
    /// CR 702.72 Champion
    Champion,
    /// CR 702.73 Changeling
    Changeling,
    /// CR 702.74 Evoke
    Evoke,
    /// CR 702.75 Hideaway
    Hideaway,
    /// CR 702.76 Prowl
    Prowl,
    /// CR 702.77 Reinforce
    Reinforce,
    /// CR 702.78 Conspire
    Conspire,
    /// CR 702.79 Persist
    Persist,
    /// CR 702.80 Wither
    Wither,
    /// CR 702.81 Retrace
    Retrace,
    /// CR 702.82 Devour
    Devour,
    /// CR 702.83 Exalted
    Exalted,
    /// CR 702.84 Unearth
    Unearth,
    /// CR 702.85 Cascade
    Cascade,
    /// CR 702.86 Annihilator
    Annihilator,
    /// CR 702.87 Level Up
    LevelUp,
    /// CR 702.88 Rebound
    Rebound,
    /// CR 702.89 Umbra Armor
    UmbraArmor,
    /// CR 702.90 Infect
    Infect,
    /// CR 702.91 Battle Cry
    BattleCry,
    /// CR 702.92 Living Weapon
    LivingWeapon,
    /// CR 702.93 Undying
    Undying,
    /// CR 702.94 Miracle
    Miracle,
    /// CR 702.95 Soulbond
    Soulbond,
    /// CR 702.96 Overload
    Overload,
    /// CR 702.97 Scavenge
    Scavenge,
    /// CR 702.98 Unleash
    Unleash,
    /// CR 702.99 Cipher
    Cipher,
    /// CR 702.100 Evolve
    Evolve,
    /// CR 702.101 Extort
    Extort,
    /// CR 702.102 Fuse
    Fuse,
    /// CR 702.103 Bestow
    Bestow,
    /// CR 702.104 Tribute
    Tribute,
    /// CR 702.105 Dethrone
    Dethrone,
    /// CR 702.106 Hidden Agenda
    HiddenAgenda,
    /// CR 702.107 Outlast
    Outlast,
    /// CR 702.108 Prowess
    Prowess,
    /// CR 702.109 Dash
    Dash,
    /// CR 702.110 Exploit
    Exploit,
    /// CR 702.111 Menace
    Menace,
    /// CR 702.112 Renown
    Renown,
    /// CR 702.113 Awaken
    Awaken,
    /// CR 702.114 Devoid
    Devoid,
    /// CR 702.115 Ingest
    Ingest,
    /// CR 702.116 Myriad
    Myriad,
    /// CR 702.117 Surge
    Surge,
    /// CR 702.118 Skulk
    Skulk,
    /// CR 702.119 Emerge
    Emerge,
    /// CR 702.120 Escalate
    Escalate,
    /// CR 702.121 Melee
    Melee,
    /// CR 702.122 Crew
    Crew,
    /// CR 702.123 Fabricate
    Fabricate,
    /// CR 702.124 Partner
    Partner,
    /// CR 702.125 Undaunted
    Undaunted,
    /// CR 702.126 Improvise
    Improvise,
    /// CR 702.127 Aftermath
    Aftermath,
    /// CR 702.128 Embalm
    Embalm,
    /// CR 702.129 Eternalize
    Eternalize,
    /// CR 702.130 Afflict
    Afflict,
    /// CR 702.131 Ascend
    Ascend,
    /// CR 702.132 Assist
    Assist,
    /// CR 702.133 Jump-Start
    JumpStart,
    /// CR 702.134 Mentor
    Mentor,
    /// CR 702.135 Afterlife
    Afterlife,
    /// CR 702.136 Riot
    Riot,
    /// CR 702.137 Spectacle
    Spectacle,
    /// CR 702.138 Escape
    Escape,
    /// CR 702.139 Companion
    Companion,
    /// CR 702.140 Mutate
    Mutate,
    /// CR 702.141 Encore
    Encore,
    /// CR 702.142 Boast
    Boast,
    /// CR 702.143 Foretell
    Foretell,
    /// CR 702.144 Demonstrate
    Demonstrate,
    /// CR 702.145 Daybound and Nightbound
    DayboundAndNightbound,
    /// CR 702.146 Disturb
    Disturb,
    /// CR 702.147 Decayed
    Decayed,
    /// CR 702.148 Cleave
    Cleave,
    /// CR 702.149 Training
    Training,
    /// CR 702.150 Compleated
    Compleated,
    /// CR 702.151 Reconfigure
    Reconfigure,
    /// CR 702.152 Blitz
    Blitz,
    /// CR 702.153 Casualty
    Casualty,
    /// CR 702.154 Enlist
    Enlist,
    /// CR 702.155 Read Ahead
    ReadAhead,
    /// CR 702.156 Ravenous
    Ravenous,
    /// CR 702.157 Squad
    Squad,
    /// CR 702.158 Space Sculptor
    SpaceSculptor,
    /// CR 702.159 Visit
    Visit,
    /// CR 702.160 Prototype
    Prototype,
    /// CR 702.161 Living Metal
    LivingMetal,
    /// CR 702.162 More Than Meets the Eye
    MoreThanMeetsTheEye,
    /// CR 702.163 For Mirrodin!
    ForMirrodin,
    /// CR 702.164 Toxic
    Toxic,
    /// CR 702.165 Backup
    Backup,
    /// CR 702.166 Bargain
    Bargain,
    /// CR 702.167 Craft
    Craft,
    /// CR 702.168 Disguise
    Disguise,
    /// CR 702.169 Solved
    Solved,
    /// CR 702.170 Plot
    Plot,
    /// CR 702.171 Saddle
    Saddle,
    /// CR 702.172 Spree
    Spree,
    /// CR 702.173 Freerunning
    Freerunning,
    /// CR 702.174 Gift
    Gift,
    /// CR 702.175 Offspring
    Offspring,
    /// CR 702.176 Impending
    Impending,
    /// CR 702.177 Exhaust
    Exhaust,
    /// CR 702.178 Max Speed
    MaxSpeed,
    /// CR 702.179 Start Your Engines!
    StartYourEngines,
    /// CR 702.180 Harmonize
    Harmonize,
    /// CR 702.181 Mobilize
    Mobilize,
    /// CR 702.182 Job Select
    JobSelect,
    /// CR 702.183 Tiered
    Tiered,
    /// CR 702.184 Station
    Station,
    /// CR 702.185 Warp
    Warp,
    /// CR 702.186 ∞ (Infinity)
    Infinity,
    /// CR 702.187 Mayhem
    Mayhem,
    /// CR 702.188 Web-slinging
    WebSlinging,
    /// CR 702.189 Firebending
    Firebending,
    /// CR 702.190 Sneak
    Sneak,
    /// CR 702.191 Increment
    Increment,
    /// CR 702.192 Paradigm
    Paradigm,
    /// CR 702.193 Power-up
    PowerUp,
    /// CR 702.194 Teamwork
    Teamwork,
    /// CR 702.195 Storied
    Storied,
}

impl KeywordKind {
    pub const ALL: &'static [KeywordKind] = &[
        KeywordKind::Deathtouch,
        KeywordKind::Defender,
        KeywordKind::DoubleStrike,
        KeywordKind::Enchant,
        KeywordKind::Equip,
        KeywordKind::FirstStrike,
        KeywordKind::Flash,
        KeywordKind::Flying,
        KeywordKind::Haste,
        KeywordKind::Hexproof,
        KeywordKind::Indestructible,
        KeywordKind::Intimidate,
        KeywordKind::Landwalk,
        KeywordKind::Lifelink,
        KeywordKind::Protection,
        KeywordKind::Reach,
        KeywordKind::Shroud,
        KeywordKind::Trample,
        KeywordKind::Vigilance,
        KeywordKind::Ward,
        KeywordKind::Banding,
        KeywordKind::Rampage,
        KeywordKind::CumulativeUpkeep,
        KeywordKind::Flanking,
        KeywordKind::Phasing,
        KeywordKind::Buyback,
        KeywordKind::Shadow,
        KeywordKind::Cycling,
        KeywordKind::Echo,
        KeywordKind::Horsemanship,
        KeywordKind::Fading,
        KeywordKind::Kicker,
        KeywordKind::Flashback,
        KeywordKind::Madness,
        KeywordKind::Fear,
        KeywordKind::Morph,
        KeywordKind::Amplify,
        KeywordKind::Provoke,
        KeywordKind::Storm,
        KeywordKind::Affinity,
        KeywordKind::Entwine,
        KeywordKind::Modular,
        KeywordKind::Sunburst,
        KeywordKind::Bushido,
        KeywordKind::Soulshift,
        KeywordKind::Splice,
        KeywordKind::Offering,
        KeywordKind::Ninjutsu,
        KeywordKind::Epic,
        KeywordKind::Convoke,
        KeywordKind::Dredge,
        KeywordKind::Transmute,
        KeywordKind::Bloodthirst,
        KeywordKind::Haunt,
        KeywordKind::Replicate,
        KeywordKind::Forecast,
        KeywordKind::Graft,
        KeywordKind::Recover,
        KeywordKind::Ripple,
        KeywordKind::SplitSecond,
        KeywordKind::Suspend,
        KeywordKind::Vanishing,
        KeywordKind::Absorb,
        KeywordKind::AuraSwap,
        KeywordKind::Delve,
        KeywordKind::Fortify,
        KeywordKind::Frenzy,
        KeywordKind::Gravestorm,
        KeywordKind::Poisonous,
        KeywordKind::Transfigure,
        KeywordKind::Champion,
        KeywordKind::Changeling,
        KeywordKind::Evoke,
        KeywordKind::Hideaway,
        KeywordKind::Prowl,
        KeywordKind::Reinforce,
        KeywordKind::Conspire,
        KeywordKind::Persist,
        KeywordKind::Wither,
        KeywordKind::Retrace,
        KeywordKind::Devour,
        KeywordKind::Exalted,
        KeywordKind::Unearth,
        KeywordKind::Cascade,
        KeywordKind::Annihilator,
        KeywordKind::LevelUp,
        KeywordKind::Rebound,
        KeywordKind::UmbraArmor,
        KeywordKind::Infect,
        KeywordKind::BattleCry,
        KeywordKind::LivingWeapon,
        KeywordKind::Undying,
        KeywordKind::Miracle,
        KeywordKind::Soulbond,
        KeywordKind::Overload,
        KeywordKind::Scavenge,
        KeywordKind::Unleash,
        KeywordKind::Cipher,
        KeywordKind::Evolve,
        KeywordKind::Extort,
        KeywordKind::Fuse,
        KeywordKind::Bestow,
        KeywordKind::Tribute,
        KeywordKind::Dethrone,
        KeywordKind::HiddenAgenda,
        KeywordKind::Outlast,
        KeywordKind::Prowess,
        KeywordKind::Dash,
        KeywordKind::Exploit,
        KeywordKind::Menace,
        KeywordKind::Renown,
        KeywordKind::Awaken,
        KeywordKind::Devoid,
        KeywordKind::Ingest,
        KeywordKind::Myriad,
        KeywordKind::Surge,
        KeywordKind::Skulk,
        KeywordKind::Emerge,
        KeywordKind::Escalate,
        KeywordKind::Melee,
        KeywordKind::Crew,
        KeywordKind::Fabricate,
        KeywordKind::Partner,
        KeywordKind::Undaunted,
        KeywordKind::Improvise,
        KeywordKind::Aftermath,
        KeywordKind::Embalm,
        KeywordKind::Eternalize,
        KeywordKind::Afflict,
        KeywordKind::Ascend,
        KeywordKind::Assist,
        KeywordKind::JumpStart,
        KeywordKind::Mentor,
        KeywordKind::Afterlife,
        KeywordKind::Riot,
        KeywordKind::Spectacle,
        KeywordKind::Escape,
        KeywordKind::Companion,
        KeywordKind::Mutate,
        KeywordKind::Encore,
        KeywordKind::Boast,
        KeywordKind::Foretell,
        KeywordKind::Demonstrate,
        KeywordKind::DayboundAndNightbound,
        KeywordKind::Disturb,
        KeywordKind::Decayed,
        KeywordKind::Cleave,
        KeywordKind::Training,
        KeywordKind::Compleated,
        KeywordKind::Reconfigure,
        KeywordKind::Blitz,
        KeywordKind::Casualty,
        KeywordKind::Enlist,
        KeywordKind::ReadAhead,
        KeywordKind::Ravenous,
        KeywordKind::Squad,
        KeywordKind::SpaceSculptor,
        KeywordKind::Visit,
        KeywordKind::Prototype,
        KeywordKind::LivingMetal,
        KeywordKind::MoreThanMeetsTheEye,
        KeywordKind::ForMirrodin,
        KeywordKind::Toxic,
        KeywordKind::Backup,
        KeywordKind::Bargain,
        KeywordKind::Craft,
        KeywordKind::Disguise,
        KeywordKind::Solved,
        KeywordKind::Plot,
        KeywordKind::Saddle,
        KeywordKind::Spree,
        KeywordKind::Freerunning,
        KeywordKind::Gift,
        KeywordKind::Offspring,
        KeywordKind::Impending,
        KeywordKind::Exhaust,
        KeywordKind::MaxSpeed,
        KeywordKind::StartYourEngines,
        KeywordKind::Harmonize,
        KeywordKind::Mobilize,
        KeywordKind::JobSelect,
        KeywordKind::Tiered,
        KeywordKind::Station,
        KeywordKind::Warp,
        KeywordKind::Infinity,
        KeywordKind::Mayhem,
        KeywordKind::WebSlinging,
        KeywordKind::Firebending,
        KeywordKind::Sneak,
        KeywordKind::Increment,
        KeywordKind::Paradigm,
        KeywordKind::PowerUp,
        KeywordKind::Teamwork,
        KeywordKind::Storied,
    ];

    /// The Comprehensive Rules section defining this keyword.
    pub fn rule(self) -> &'static str {
        match self {
            KeywordKind::Deathtouch => "702.2",
            KeywordKind::Defender => "702.3",
            KeywordKind::DoubleStrike => "702.4",
            KeywordKind::Enchant => "702.5",
            KeywordKind::Equip => "702.6",
            KeywordKind::FirstStrike => "702.7",
            KeywordKind::Flash => "702.8",
            KeywordKind::Flying => "702.9",
            KeywordKind::Haste => "702.10",
            KeywordKind::Hexproof => "702.11",
            KeywordKind::Indestructible => "702.12",
            KeywordKind::Intimidate => "702.13",
            KeywordKind::Landwalk => "702.14",
            KeywordKind::Lifelink => "702.15",
            KeywordKind::Protection => "702.16",
            KeywordKind::Reach => "702.17",
            KeywordKind::Shroud => "702.18",
            KeywordKind::Trample => "702.19",
            KeywordKind::Vigilance => "702.20",
            KeywordKind::Ward => "702.21",
            KeywordKind::Banding => "702.22",
            KeywordKind::Rampage => "702.23",
            KeywordKind::CumulativeUpkeep => "702.24",
            KeywordKind::Flanking => "702.25",
            KeywordKind::Phasing => "702.26",
            KeywordKind::Buyback => "702.27",
            KeywordKind::Shadow => "702.28",
            KeywordKind::Cycling => "702.29",
            KeywordKind::Echo => "702.30",
            KeywordKind::Horsemanship => "702.31",
            KeywordKind::Fading => "702.32",
            KeywordKind::Kicker => "702.33",
            KeywordKind::Flashback => "702.34",
            KeywordKind::Madness => "702.35",
            KeywordKind::Fear => "702.36",
            KeywordKind::Morph => "702.37",
            KeywordKind::Amplify => "702.38",
            KeywordKind::Provoke => "702.39",
            KeywordKind::Storm => "702.40",
            KeywordKind::Affinity => "702.41",
            KeywordKind::Entwine => "702.42",
            KeywordKind::Modular => "702.43",
            KeywordKind::Sunburst => "702.44",
            KeywordKind::Bushido => "702.45",
            KeywordKind::Soulshift => "702.46",
            KeywordKind::Splice => "702.47",
            KeywordKind::Offering => "702.48",
            KeywordKind::Ninjutsu => "702.49",
            KeywordKind::Epic => "702.50",
            KeywordKind::Convoke => "702.51",
            KeywordKind::Dredge => "702.52",
            KeywordKind::Transmute => "702.53",
            KeywordKind::Bloodthirst => "702.54",
            KeywordKind::Haunt => "702.55",
            KeywordKind::Replicate => "702.56",
            KeywordKind::Forecast => "702.57",
            KeywordKind::Graft => "702.58",
            KeywordKind::Recover => "702.59",
            KeywordKind::Ripple => "702.60",
            KeywordKind::SplitSecond => "702.61",
            KeywordKind::Suspend => "702.62",
            KeywordKind::Vanishing => "702.63",
            KeywordKind::Absorb => "702.64",
            KeywordKind::AuraSwap => "702.65",
            KeywordKind::Delve => "702.66",
            KeywordKind::Fortify => "702.67",
            KeywordKind::Frenzy => "702.68",
            KeywordKind::Gravestorm => "702.69",
            KeywordKind::Poisonous => "702.70",
            KeywordKind::Transfigure => "702.71",
            KeywordKind::Champion => "702.72",
            KeywordKind::Changeling => "702.73",
            KeywordKind::Evoke => "702.74",
            KeywordKind::Hideaway => "702.75",
            KeywordKind::Prowl => "702.76",
            KeywordKind::Reinforce => "702.77",
            KeywordKind::Conspire => "702.78",
            KeywordKind::Persist => "702.79",
            KeywordKind::Wither => "702.80",
            KeywordKind::Retrace => "702.81",
            KeywordKind::Devour => "702.82",
            KeywordKind::Exalted => "702.83",
            KeywordKind::Unearth => "702.84",
            KeywordKind::Cascade => "702.85",
            KeywordKind::Annihilator => "702.86",
            KeywordKind::LevelUp => "702.87",
            KeywordKind::Rebound => "702.88",
            KeywordKind::UmbraArmor => "702.89",
            KeywordKind::Infect => "702.90",
            KeywordKind::BattleCry => "702.91",
            KeywordKind::LivingWeapon => "702.92",
            KeywordKind::Undying => "702.93",
            KeywordKind::Miracle => "702.94",
            KeywordKind::Soulbond => "702.95",
            KeywordKind::Overload => "702.96",
            KeywordKind::Scavenge => "702.97",
            KeywordKind::Unleash => "702.98",
            KeywordKind::Cipher => "702.99",
            KeywordKind::Evolve => "702.100",
            KeywordKind::Extort => "702.101",
            KeywordKind::Fuse => "702.102",
            KeywordKind::Bestow => "702.103",
            KeywordKind::Tribute => "702.104",
            KeywordKind::Dethrone => "702.105",
            KeywordKind::HiddenAgenda => "702.106",
            KeywordKind::Outlast => "702.107",
            KeywordKind::Prowess => "702.108",
            KeywordKind::Dash => "702.109",
            KeywordKind::Exploit => "702.110",
            KeywordKind::Menace => "702.111",
            KeywordKind::Renown => "702.112",
            KeywordKind::Awaken => "702.113",
            KeywordKind::Devoid => "702.114",
            KeywordKind::Ingest => "702.115",
            KeywordKind::Myriad => "702.116",
            KeywordKind::Surge => "702.117",
            KeywordKind::Skulk => "702.118",
            KeywordKind::Emerge => "702.119",
            KeywordKind::Escalate => "702.120",
            KeywordKind::Melee => "702.121",
            KeywordKind::Crew => "702.122",
            KeywordKind::Fabricate => "702.123",
            KeywordKind::Partner => "702.124",
            KeywordKind::Undaunted => "702.125",
            KeywordKind::Improvise => "702.126",
            KeywordKind::Aftermath => "702.127",
            KeywordKind::Embalm => "702.128",
            KeywordKind::Eternalize => "702.129",
            KeywordKind::Afflict => "702.130",
            KeywordKind::Ascend => "702.131",
            KeywordKind::Assist => "702.132",
            KeywordKind::JumpStart => "702.133",
            KeywordKind::Mentor => "702.134",
            KeywordKind::Afterlife => "702.135",
            KeywordKind::Riot => "702.136",
            KeywordKind::Spectacle => "702.137",
            KeywordKind::Escape => "702.138",
            KeywordKind::Companion => "702.139",
            KeywordKind::Mutate => "702.140",
            KeywordKind::Encore => "702.141",
            KeywordKind::Boast => "702.142",
            KeywordKind::Foretell => "702.143",
            KeywordKind::Demonstrate => "702.144",
            KeywordKind::DayboundAndNightbound => "702.145",
            KeywordKind::Disturb => "702.146",
            KeywordKind::Decayed => "702.147",
            KeywordKind::Cleave => "702.148",
            KeywordKind::Training => "702.149",
            KeywordKind::Compleated => "702.150",
            KeywordKind::Reconfigure => "702.151",
            KeywordKind::Blitz => "702.152",
            KeywordKind::Casualty => "702.153",
            KeywordKind::Enlist => "702.154",
            KeywordKind::ReadAhead => "702.155",
            KeywordKind::Ravenous => "702.156",
            KeywordKind::Squad => "702.157",
            KeywordKind::SpaceSculptor => "702.158",
            KeywordKind::Visit => "702.159",
            KeywordKind::Prototype => "702.160",
            KeywordKind::LivingMetal => "702.161",
            KeywordKind::MoreThanMeetsTheEye => "702.162",
            KeywordKind::ForMirrodin => "702.163",
            KeywordKind::Toxic => "702.164",
            KeywordKind::Backup => "702.165",
            KeywordKind::Bargain => "702.166",
            KeywordKind::Craft => "702.167",
            KeywordKind::Disguise => "702.168",
            KeywordKind::Solved => "702.169",
            KeywordKind::Plot => "702.170",
            KeywordKind::Saddle => "702.171",
            KeywordKind::Spree => "702.172",
            KeywordKind::Freerunning => "702.173",
            KeywordKind::Gift => "702.174",
            KeywordKind::Offspring => "702.175",
            KeywordKind::Impending => "702.176",
            KeywordKind::Exhaust => "702.177",
            KeywordKind::MaxSpeed => "702.178",
            KeywordKind::StartYourEngines => "702.179",
            KeywordKind::Harmonize => "702.180",
            KeywordKind::Mobilize => "702.181",
            KeywordKind::JobSelect => "702.182",
            KeywordKind::Tiered => "702.183",
            KeywordKind::Station => "702.184",
            KeywordKind::Warp => "702.185",
            KeywordKind::Infinity => "702.186",
            KeywordKind::Mayhem => "702.187",
            KeywordKind::WebSlinging => "702.188",
            KeywordKind::Firebending => "702.189",
            KeywordKind::Sneak => "702.190",
            KeywordKind::Increment => "702.191",
            KeywordKind::Paradigm => "702.192",
            KeywordKind::PowerUp => "702.193",
            KeywordKind::Teamwork => "702.194",
            KeywordKind::Storied => "702.195",
        }
    }

    /// The keyword's name as printed.
    pub fn name(self) -> &'static str {
        match self {
            KeywordKind::Deathtouch => "Deathtouch",
            KeywordKind::Defender => "Defender",
            KeywordKind::DoubleStrike => "Double Strike",
            KeywordKind::Enchant => "Enchant",
            KeywordKind::Equip => "Equip",
            KeywordKind::FirstStrike => "First Strike",
            KeywordKind::Flash => "Flash",
            KeywordKind::Flying => "Flying",
            KeywordKind::Haste => "Haste",
            KeywordKind::Hexproof => "Hexproof",
            KeywordKind::Indestructible => "Indestructible",
            KeywordKind::Intimidate => "Intimidate",
            KeywordKind::Landwalk => "Landwalk",
            KeywordKind::Lifelink => "Lifelink",
            KeywordKind::Protection => "Protection",
            KeywordKind::Reach => "Reach",
            KeywordKind::Shroud => "Shroud",
            KeywordKind::Trample => "Trample",
            KeywordKind::Vigilance => "Vigilance",
            KeywordKind::Ward => "Ward",
            KeywordKind::Banding => "Banding",
            KeywordKind::Rampage => "Rampage",
            KeywordKind::CumulativeUpkeep => "Cumulative Upkeep",
            KeywordKind::Flanking => "Flanking",
            KeywordKind::Phasing => "Phasing",
            KeywordKind::Buyback => "Buyback",
            KeywordKind::Shadow => "Shadow",
            KeywordKind::Cycling => "Cycling",
            KeywordKind::Echo => "Echo",
            KeywordKind::Horsemanship => "Horsemanship",
            KeywordKind::Fading => "Fading",
            KeywordKind::Kicker => "Kicker",
            KeywordKind::Flashback => "Flashback",
            KeywordKind::Madness => "Madness",
            KeywordKind::Fear => "Fear",
            KeywordKind::Morph => "Morph",
            KeywordKind::Amplify => "Amplify",
            KeywordKind::Provoke => "Provoke",
            KeywordKind::Storm => "Storm",
            KeywordKind::Affinity => "Affinity",
            KeywordKind::Entwine => "Entwine",
            KeywordKind::Modular => "Modular",
            KeywordKind::Sunburst => "Sunburst",
            KeywordKind::Bushido => "Bushido",
            KeywordKind::Soulshift => "Soulshift",
            KeywordKind::Splice => "Splice",
            KeywordKind::Offering => "Offering",
            KeywordKind::Ninjutsu => "Ninjutsu",
            KeywordKind::Epic => "Epic",
            KeywordKind::Convoke => "Convoke",
            KeywordKind::Dredge => "Dredge",
            KeywordKind::Transmute => "Transmute",
            KeywordKind::Bloodthirst => "Bloodthirst",
            KeywordKind::Haunt => "Haunt",
            KeywordKind::Replicate => "Replicate",
            KeywordKind::Forecast => "Forecast",
            KeywordKind::Graft => "Graft",
            KeywordKind::Recover => "Recover",
            KeywordKind::Ripple => "Ripple",
            KeywordKind::SplitSecond => "Split Second",
            KeywordKind::Suspend => "Suspend",
            KeywordKind::Vanishing => "Vanishing",
            KeywordKind::Absorb => "Absorb",
            KeywordKind::AuraSwap => "Aura Swap",
            KeywordKind::Delve => "Delve",
            KeywordKind::Fortify => "Fortify",
            KeywordKind::Frenzy => "Frenzy",
            KeywordKind::Gravestorm => "Gravestorm",
            KeywordKind::Poisonous => "Poisonous",
            KeywordKind::Transfigure => "Transfigure",
            KeywordKind::Champion => "Champion",
            KeywordKind::Changeling => "Changeling",
            KeywordKind::Evoke => "Evoke",
            KeywordKind::Hideaway => "Hideaway",
            KeywordKind::Prowl => "Prowl",
            KeywordKind::Reinforce => "Reinforce",
            KeywordKind::Conspire => "Conspire",
            KeywordKind::Persist => "Persist",
            KeywordKind::Wither => "Wither",
            KeywordKind::Retrace => "Retrace",
            KeywordKind::Devour => "Devour",
            KeywordKind::Exalted => "Exalted",
            KeywordKind::Unearth => "Unearth",
            KeywordKind::Cascade => "Cascade",
            KeywordKind::Annihilator => "Annihilator",
            KeywordKind::LevelUp => "Level Up",
            KeywordKind::Rebound => "Rebound",
            KeywordKind::UmbraArmor => "Umbra Armor",
            KeywordKind::Infect => "Infect",
            KeywordKind::BattleCry => "Battle Cry",
            KeywordKind::LivingWeapon => "Living Weapon",
            KeywordKind::Undying => "Undying",
            KeywordKind::Miracle => "Miracle",
            KeywordKind::Soulbond => "Soulbond",
            KeywordKind::Overload => "Overload",
            KeywordKind::Scavenge => "Scavenge",
            KeywordKind::Unleash => "Unleash",
            KeywordKind::Cipher => "Cipher",
            KeywordKind::Evolve => "Evolve",
            KeywordKind::Extort => "Extort",
            KeywordKind::Fuse => "Fuse",
            KeywordKind::Bestow => "Bestow",
            KeywordKind::Tribute => "Tribute",
            KeywordKind::Dethrone => "Dethrone",
            KeywordKind::HiddenAgenda => "Hidden Agenda",
            KeywordKind::Outlast => "Outlast",
            KeywordKind::Prowess => "Prowess",
            KeywordKind::Dash => "Dash",
            KeywordKind::Exploit => "Exploit",
            KeywordKind::Menace => "Menace",
            KeywordKind::Renown => "Renown",
            KeywordKind::Awaken => "Awaken",
            KeywordKind::Devoid => "Devoid",
            KeywordKind::Ingest => "Ingest",
            KeywordKind::Myriad => "Myriad",
            KeywordKind::Surge => "Surge",
            KeywordKind::Skulk => "Skulk",
            KeywordKind::Emerge => "Emerge",
            KeywordKind::Escalate => "Escalate",
            KeywordKind::Melee => "Melee",
            KeywordKind::Crew => "Crew",
            KeywordKind::Fabricate => "Fabricate",
            KeywordKind::Partner => "Partner",
            KeywordKind::Undaunted => "Undaunted",
            KeywordKind::Improvise => "Improvise",
            KeywordKind::Aftermath => "Aftermath",
            KeywordKind::Embalm => "Embalm",
            KeywordKind::Eternalize => "Eternalize",
            KeywordKind::Afflict => "Afflict",
            KeywordKind::Ascend => "Ascend",
            KeywordKind::Assist => "Assist",
            KeywordKind::JumpStart => "Jump-Start",
            KeywordKind::Mentor => "Mentor",
            KeywordKind::Afterlife => "Afterlife",
            KeywordKind::Riot => "Riot",
            KeywordKind::Spectacle => "Spectacle",
            KeywordKind::Escape => "Escape",
            KeywordKind::Companion => "Companion",
            KeywordKind::Mutate => "Mutate",
            KeywordKind::Encore => "Encore",
            KeywordKind::Boast => "Boast",
            KeywordKind::Foretell => "Foretell",
            KeywordKind::Demonstrate => "Demonstrate",
            KeywordKind::DayboundAndNightbound => "Daybound and Nightbound",
            KeywordKind::Disturb => "Disturb",
            KeywordKind::Decayed => "Decayed",
            KeywordKind::Cleave => "Cleave",
            KeywordKind::Training => "Training",
            KeywordKind::Compleated => "Compleated",
            KeywordKind::Reconfigure => "Reconfigure",
            KeywordKind::Blitz => "Blitz",
            KeywordKind::Casualty => "Casualty",
            KeywordKind::Enlist => "Enlist",
            KeywordKind::ReadAhead => "Read Ahead",
            KeywordKind::Ravenous => "Ravenous",
            KeywordKind::Squad => "Squad",
            KeywordKind::SpaceSculptor => "Space Sculptor",
            KeywordKind::Visit => "Visit",
            KeywordKind::Prototype => "Prototype",
            KeywordKind::LivingMetal => "Living Metal",
            KeywordKind::MoreThanMeetsTheEye => "More Than Meets the Eye",
            KeywordKind::ForMirrodin => "For Mirrodin!",
            KeywordKind::Toxic => "Toxic",
            KeywordKind::Backup => "Backup",
            KeywordKind::Bargain => "Bargain",
            KeywordKind::Craft => "Craft",
            KeywordKind::Disguise => "Disguise",
            KeywordKind::Solved => "Solved",
            KeywordKind::Plot => "Plot",
            KeywordKind::Saddle => "Saddle",
            KeywordKind::Spree => "Spree",
            KeywordKind::Freerunning => "Freerunning",
            KeywordKind::Gift => "Gift",
            KeywordKind::Offspring => "Offspring",
            KeywordKind::Impending => "Impending",
            KeywordKind::Exhaust => "Exhaust",
            KeywordKind::MaxSpeed => "Max Speed",
            KeywordKind::StartYourEngines => "Start Your Engines!",
            KeywordKind::Harmonize => "Harmonize",
            KeywordKind::Mobilize => "Mobilize",
            KeywordKind::JobSelect => "Job Select",
            KeywordKind::Tiered => "Tiered",
            KeywordKind::Station => "Station",
            KeywordKind::Warp => "Warp",
            KeywordKind::Infinity => "∞ (Infinity)",
            KeywordKind::Mayhem => "Mayhem",
            KeywordKind::WebSlinging => "Web-slinging",
            KeywordKind::Firebending => "Firebending",
            KeywordKind::Sneak => "Sneak",
            KeywordKind::Increment => "Increment",
            KeywordKind::Paradigm => "Paradigm",
            KeywordKind::PowerUp => "Power-up",
            KeywordKind::Teamwork => "Teamwork",
            KeywordKind::Storied => "Storied",
        }
    }
}

impl KeywordKind {
    /// Looks a keyword up by its printed name (case-insensitive).
    pub fn from_name(name: &str) -> Option<KeywordKind> {
        let n = name.trim().to_lowercase();
        KeywordKind::ALL
            .iter()
            .copied()
            .find(|k| k.name().to_lowercase() == n)
    }

    /// Evasion abilities restrict what can block (CR 509.1b).
    pub fn is_evasion(self) -> bool {
        use KeywordKind::*;
        matches!(
            self,
            Flying | Fear | Intimidate | Landwalk | Shadow | Horsemanship | Skulk | Menace
        )
    }
}
