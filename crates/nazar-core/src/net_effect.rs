use serde::{Deserialize, Serialize};

use crate::observation::{Original, Transition};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NetEffect {
    Created,
    Modified { original: Original },
    Deleted { original: Original },
}

impl NetEffect {
    pub fn is_deleted(&self) -> bool {
        matches!(self, Self::Deleted { .. })
    }
}

pub fn combine(current: Option<NetEffect>, transition: Transition) -> Option<NetEffect> {
    match current {
        None => Some(first_effect_of(transition)),
        Some(NetEffect::Created) => effect_after_creation(&transition),
        Some(NetEffect::Modified { original } | NetEffect::Deleted { original }) => {
            Some(effect_on_existing(original, &transition))
        }
    }
}

fn first_effect_of(transition: Transition) -> NetEffect {
    match transition {
        Transition::Appeared => NetEffect::Created,
        Transition::Changed { original } => NetEffect::Modified { original },
        Transition::Vanished { original } => NetEffect::Deleted { original },
    }
}

fn effect_after_creation(transition: &Transition) -> Option<NetEffect> {
    match transition {
        Transition::Vanished { .. } => None,
        Transition::Appeared | Transition::Changed { .. } => Some(NetEffect::Created),
    }
}

fn effect_on_existing(original: Original, transition: &Transition) -> NetEffect {
    match transition {
        Transition::Vanished { .. } => NetEffect::Deleted { original },
        Transition::Appeared | Transition::Changed { .. } => NetEffect::Modified { original },
    }
}
