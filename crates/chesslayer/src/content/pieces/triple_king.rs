use super::PieceDefinition;
use crate::PieceType;

pub const TRIPLE_KING: PieceType = 97;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 10,
    description: "한 행동으로 킹 이동을 최대 3단계 수행합니다. 두 번째 단계부터 직전 방향으로 직진하거나 좌우 45도로 꺾으며, 각 단계에서 멈출 수 있습니다. 아군을 통과할 수 없고 포획하면 이동을 마칩니다.",
    ..PieceDefinition::new(TRIPLE_KING, "triple_king", "트리플 킹", movement)
};

fn movement() -> String {
    r#"take-move(1,0) {take-move(1,0) {take-move(1,0)} {take-move(1,1)} {take-move(1,-1)}} {take-move(1,1) {take-move(1,1)} {take-move(1,0)} {take-move(0,1)}} {take-move(1,-1) {take-move(1,-1)} {take-move(1,0)} {take-move(0,-1)}};
take-move(-1,0) {take-move(-1,0) {take-move(-1,0)} {take-move(-1,1)} {take-move(-1,-1)}} {take-move(-1,1) {take-move(-1,1)} {take-move(-1,0)} {take-move(0,1)}} {take-move(-1,-1) {take-move(-1,-1)} {take-move(-1,0)} {take-move(0,-1)}};
take-move(0,-1) {take-move(0,-1) {take-move(0,-1)} {take-move(1,-1)} {take-move(-1,-1)}} {take-move(1,-1) {take-move(1,-1)} {take-move(1,0)} {take-move(0,-1)}} {take-move(-1,-1) {take-move(-1,-1)} {take-move(-1,0)} {take-move(0,-1)}};
take-move(0,1) {take-move(0,1) {take-move(0,1)} {take-move(1,1)} {take-move(-1,1)}} {take-move(1,1) {take-move(1,1)} {take-move(1,0)} {take-move(0,1)}} {take-move(-1,1) {take-move(-1,1)} {take-move(-1,0)} {take-move(0,1)}};
take-move(1,1) {take-move(1,1) {take-move(1,1)} {take-move(1,0)} {take-move(0,1)}} {take-move(1,0) {take-move(1,0)} {take-move(1,1)} {take-move(1,-1)}} {take-move(0,1) {take-move(0,1)} {take-move(1,1)} {take-move(-1,1)}};
take-move(1,-1) {take-move(1,-1) {take-move(1,-1)} {take-move(1,0)} {take-move(0,-1)}} {take-move(1,0) {take-move(1,0)} {take-move(1,1)} {take-move(1,-1)}} {take-move(0,-1) {take-move(0,-1)} {take-move(1,-1)} {take-move(-1,-1)}};
take-move(-1,1) {take-move(-1,1) {take-move(-1,1)} {take-move(-1,0)} {take-move(0,1)}} {take-move(-1,0) {take-move(-1,0)} {take-move(-1,1)} {take-move(-1,-1)}} {take-move(0,1) {take-move(0,1)} {take-move(1,1)} {take-move(-1,1)}};
take-move(-1,-1) {take-move(-1,-1) {take-move(-1,-1)} {take-move(-1,0)} {take-move(0,-1)}} {take-move(-1,0) {take-move(-1,0)} {take-move(-1,1)} {take-move(-1,-1)}} {take-move(0,-1) {take-move(0,-1)} {take-move(1,-1)} {take-move(-1,-1)}};"#.into()
}
