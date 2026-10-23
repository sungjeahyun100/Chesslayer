use super::{ward, CardDefinition, CardKind};

pub fn definition() -> CardDefinition {
    CardDefinition {
        ethereal: true,
        ..CardDefinition::new(
            CardKind::FleetingWard,
            "찰나의 가호",
            0,
            "가호와 같은 효과. 쓰지 않고 턴을 마치면 소멸합니다.",
            ward::effect,
        )
        .with_targets(ward::targets)
    }
}
