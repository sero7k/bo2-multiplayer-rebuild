use crate::ZoneGame;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AssetNamespace {
    #[default]
    Iw4,
    T5,
    Iw5,
    /// bo2zm: Black Ops II.
    T6,
    /// bo2mp: Call of Duty: Ghosts (guest guns only).
    Iw6,
    /// bo2mp: Call of Duty: World at War (guest guns only; its maps are
    /// read into Black Ops II's own namespace).
    T4,
    /// bo2mp: Call of Duty 4 (guest guns only, as World at War).
    Iw3,
    /// bo2mp: Call of Duty: Advanced Warfare (guest guns only).
    S1,
    /// bo2mp: Call of Duty: Black Ops III (guest guns only; its maps are
    /// read into Black Ops II's own namespace).
    T7,
    /// bo2mp: Call of Duty: Modern Warfare III (2023, "jup"): Resurgence's
    /// sounds on Rebirth Island so far.
    Jup,
}

impl AssetNamespace {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Iw4 => "iw4",
            Self::T5 => "t5",
            Self::Iw5 => "iw5",
            Self::T6 => "t6",
            Self::Iw6 => "iw6",
            Self::T4 => "t4",
            Self::Iw3 => "iw3",
            Self::S1 => "s1",
            Self::T7 => "t7",
            Self::Jup => "jup",
        }
    }

    /// bo2mp: the prefix another game's guns carry in a Black Ops II match
    /// (`bo1_ak74u_mp`, `mw2_m4_mp`), so they never take a BO2 gun's name;
    /// empty for BO2's own.
    pub const fn guest_prefix(self) -> &'static str {
        match self {
            Self::Iw4 => "mw2_",
            Self::T5 => "bo1_",
            Self::Iw5 => "mw3_",
            Self::Iw6 => "gh_",
            Self::T4 => "waw_",
            Self::Iw3 => "cod4_",
            Self::S1 => "aw_",
            Self::T7 => "bo3_",
            Self::Jup => "jup_",
            Self::T6 => "",
        }
    }

    /// bo2mp: the game a guest gun's script name says it is from.
    pub fn from_guest_name(name: &str) -> Option<Self> {
        [Self::T5, Self::Iw4, Self::Iw5, Self::Iw6, Self::T4, Self::Iw3, Self::S1, Self::T7].into_iter().find(|ns| name.starts_with(ns.guest_prefix()))
    }

    /// bo2mp: the attachment words of this game's gun variants
    /// (`ak74u_reflex_extclip`: `reflex`, `extclip`).
    pub const fn guest_attachment_tokens(self) -> &'static [&'static str] {
        match self {
            Self::T5 => &[
                "acog", "auto", "dualclip", "elbit", "extclip", "ft", "gl", "grip", "ir", "lps", "mk", "reflex", "rf",
                "silencer", "snub", "speed", "upgradesight", "vzoom",
            ],
            Self::Iw4 => &[
                "acog", "akimbo", "eotech", "fmj", "gl", "grip", "heartbeat", "reflex", "rof", "shotgun", "silencer",
                "tactical", "thermal", "xmags",
            ],
            // (World at War's bipod variants are deployed modes, not
            // attachments: `bar_bipod` stays a gun of its own.)
            Self::T4 => &[
                "aperture", "bayonet", "bigammo", "flash", "gl", "grip", "sawoff", "scoped", "silenced", "telescopic",
            ],
            Self::Iw3 => &["acog", "gl", "grip", "reflex", "silencer"],
            Self::Iw5 | Self::T6 | Self::Iw6 | Self::S1 | Self::T7 | Self::Jup => &[],
        }
    }

    /// bo2mp: an attachment word of this game's variants as a guest gun's
    /// name carries it: where it is one Black Ops II has, BO2's word
    /// (World at War's `silenced` is `silencer`, `bigammo` `extclip`).
    pub fn guest_attachment_word(self, native: &str) -> &str {
        match (self, native) {
            (Self::T4, "silenced") => "silencer",
            (Self::T4, "bigammo") => "extclip",
            _ => native,
        }
    }

    /// bo2mp: an MW3 attachment as a guest gun's name carries it: MW3's
    /// per-class variants of one attachment (`reflexsmg`, `reflexlmg`,
    /// `eotechsmg`, `acogsmg`, `thermalsmg`, `silencer02`, `silencer03`)
    /// are the one word Create-a-Class lists (`reflex`, `silencer`).
    pub fn iw5_guest_attachment(native: &str) -> &str {
        match native {
            "reflexsmg" | "reflexlmg" => "reflex",
            "eotechsmg" | "eotechlmg" => "eotech",
            "acogsmg" => "acog",
            "thermalsmg" => "thermal",
            "silencer02" | "silencer03" => "silencer",
            "gp25" | "m320" => "gl",
            other => other,
        }
    }

    pub fn from_zone_game(game: ZoneGame) -> Self {
        match game {
            ZoneGame::Iw4 => Self::Iw4,
            ZoneGame::T5 => Self::T5,
            ZoneGame::Iw5 => Self::Iw5,
            ZoneGame::T6 => Self::T6,
        }
    }

    pub fn parse(token: &str) -> Option<Self> {
        match token {
            "iw4" => Some(Self::Iw4),
            "t5" => Some(Self::T5),
            "iw5" => Some(Self::Iw5),
            "t6" => Some(Self::T6),
            "iw6" => Some(Self::Iw6),
            "t4" => Some(Self::T4),
            "iw3" => Some(Self::Iw3),
            "s1" => Some(Self::S1),
            "t7" => Some(Self::T7),
            "jup" => Some(Self::Jup),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AssetKind {
    Weapon,
    Map,
    XModel,
    Anim,
    Material,
    Localize,
}

impl AssetKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Weapon => "weapon",
            Self::Map => "map",
            Self::XModel => "xmodel",
            Self::Anim => "anim",
            Self::Material => "material",
            Self::Localize => "localize",
        }
    }

    pub fn parse(token: &str) -> Option<Self> {
        match token {
            "weapon" => Some(Self::Weapon),
            "map" => Some(Self::Map),
            "xmodel" => Some(Self::XModel),
            "anim" => Some(Self::Anim),
            "material" => Some(Self::Material),
            "localize" => Some(Self::Localize),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MaterialKey {
    pub namespace: AssetNamespace,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AssetKey {
    pub namespace: AssetNamespace,
    pub kind: AssetKind,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssetKeyError {
    MissingNamespace,

    UnknownNamespace,

    UnknownKind,

    EmptyName,
}

impl core::fmt::Display for AssetKeyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingNamespace => {
                write!(f, "asset key missing namespace (want iw4:/t5:/iw5:)")
            }
            Self::UnknownNamespace => write!(f, "asset key unknown namespace"),
            Self::UnknownKind => write!(f, "asset key missing or unknown kind"),
            Self::EmptyName => write!(f, "asset key empty logical name"),
        }
    }
}

impl std::error::Error for AssetKeyError {}

impl AssetKey {
    pub fn new(
        namespace: AssetNamespace,
        kind: AssetKind,
        name: impl Into<String>,
    ) -> Result<Self, AssetKeyError> {
        let name = name.into();
        if name.is_empty() || name.contains(':') || name.contains('/') {
            return Err(AssetKeyError::EmptyName);
        }
        Ok(Self {
            namespace,
            kind,
            name,
        })
    }

    pub fn parse(raw: &str) -> Result<Self, AssetKeyError> {
        let Some((ns, rest)) = raw.split_once(':') else {
            return Err(AssetKeyError::MissingNamespace);
        };
        let namespace = AssetNamespace::parse(ns).ok_or(AssetKeyError::UnknownNamespace)?;
        let Some((kind_tok, name)) = rest.split_once('/') else {
            return Err(AssetKeyError::UnknownKind);
        };
        let kind = AssetKind::parse(kind_tok).ok_or(AssetKeyError::UnknownKind)?;
        if name.is_empty() {
            return Err(AssetKeyError::EmptyName);
        }
        Ok(Self {
            namespace,
            kind,
            name: name.to_owned(),
        })
    }

    pub fn display(&self) -> String {
        format!(
            "{}:{}/{}",
            self.namespace.as_str(),
            self.kind.as_str(),
            self.name
        )
    }

    pub fn logical_name(&self) -> &str {
        &self.name
    }
}

impl core::fmt::Display for AssetKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}:{}/{}",
            self.namespace.as_str(),
            self.kind.as_str(),
            self.name
        )
    }
}
