use nazar_core::net_effect::{NetEffect, combine};
use nazar_core::observation::{Original, RegistryValueContent, Transition};

fn first_original() -> Original {
    Original::RegistryValue(RegistryValueContent {
        value_type: 1,
        bytes: vec![1],
    })
}

fn later_original() -> Original {
    Original::RegistryValue(RegistryValueContent {
        value_type: 1,
        bytes: vec![2],
    })
}

fn changed(original: Original) -> Transition {
    Transition::Changed { original }
}

fn vanished(original: Original) -> Transition {
    Transition::Vanished { original }
}

#[test]
fn untouched_object_takes_the_effect_of_its_first_transition() {
    assert_eq!(
        combine(None, Transition::Appeared),
        Some(NetEffect::Created)
    );
    assert_eq!(
        combine(None, changed(first_original())),
        Some(NetEffect::Modified {
            original: first_original()
        })
    );
    assert_eq!(
        combine(None, vanished(first_original())),
        Some(NetEffect::Deleted {
            original: first_original()
        })
    );
}

#[test]
fn created_object_that_vanishes_leaves_no_record() {
    let effect = combine(Some(NetEffect::Created), vanished(Original::NotCaptured));

    assert_eq!(effect, None);
}

#[test]
fn created_object_stays_created_however_often_it_changes() {
    let after_change = combine(Some(NetEffect::Created), changed(later_original()));
    let after_reappearing = combine(Some(NetEffect::Created), Transition::Appeared);

    assert_eq!(after_change, Some(NetEffect::Created));
    assert_eq!(after_reappearing, Some(NetEffect::Created));
}

#[test]
fn modified_object_keeps_its_first_original() {
    let modified = NetEffect::Modified {
        original: first_original(),
    };

    let after_change = combine(Some(modified.clone()), changed(later_original()));
    let after_vanishing = combine(Some(modified.clone()), vanished(later_original()));

    assert_eq!(after_change, Some(modified));
    assert_eq!(
        after_vanishing,
        Some(NetEffect::Deleted {
            original: first_original()
        })
    );
}

#[test]
fn deleted_object_that_reappears_counts_as_modified() {
    let deleted = NetEffect::Deleted {
        original: first_original(),
    };

    let after_reappearing = combine(Some(deleted.clone()), Transition::Appeared);
    let after_vanishing_again = combine(Some(deleted.clone()), vanished(later_original()));

    assert_eq!(
        after_reappearing,
        Some(NetEffect::Modified {
            original: first_original()
        })
    );
    assert_eq!(after_vanishing_again, Some(deleted));
}
