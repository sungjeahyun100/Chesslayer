//! 보드를 변경하지 않고 체섬블리 액션 후보를 생성한다.
mod action;
mod board;
pub use action::{apply_default_action, ActionError, ActionStep, DefaultAction, Effect};
pub use board::{Board, BoardError, Piece, PieceId, PieceType, PlayerId, Position};
use std::collections::HashMap;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Parse(String),
    Board(BoardError),
    EmptyOrigin,
    UninitializedRegister(i32),
    UninitializedAnchor(i32),
    ExecutionLimit,
    BlockLimit,
}
impl From<BoardError> for Error {
    fn from(e: BoardError) -> Self {
        Self::Board(e)
    }
}
#[derive(Clone, Debug)]
struct Inst {
    name: String,
    args: Vec<String>,
    partner: usize,
}
#[derive(Clone, Debug)]
pub struct Program {
    // 세미콜론으로 구분한 체인들은 각각 독립적으로 실행된다.
    chains: Vec<Vec<Inst>>,
}
const SLOT_COUNT: usize = 256;
fn slot_op(name: &str) -> bool {
    matches!(
        name,
        "write" | "read" | "read-and" | "read-or" | "read-xor" | "write-anchor" | "read-anchor"
    )
}
fn arity(n: &str) -> Option<i8> {
    Some(match n {
        "move"
        | "take"
        | "take-move"
        | "catch"
        | "shift"
        | "jump"
        | "peek"
        | "observe"
        | "anchor"
        | "absolute"
        | "bound"
        | "edge"
        | "corner"
        | "edge-top"
        | "edge-bottom"
        | "edge-left"
        | "edge-right"
        | "corner-top-left"
        | "corner-top-right"
        | "corner-bottom-left"
        | "corner-bottom-right"
        | "enemy"
        | "friendly"
        | "barrier" => 2,
        "piece-on" | "color-on" => 3,
        "wall-piece" | "absolute-x" | "absolute-y" | "piece" | "color" | "repeat" | "label"
        | "jmp" | "jne" | "write" | "read" | "read-and" | "read-or" | "read-xor"
        | "write-anchor" | "read-anchor" => 1,
        "transition" => -1,
        "not" | "true" | "false" | "end" | "do" | "while" => 0,
        _ => return None,
    })
}
fn int(s: &str) -> Result<i32, Error> {
    s.parse()
        .map_err(|_| Error::Parse(format!("invalid integer: {s}")))
}
fn a(n: &Inst, i: usize) -> i32 {
    int(&n.args[i]).expect("parse validated")
}
impl Program {
    pub fn parse(src: &str) -> Result<Self, Error> {
        // 먼저 명령을 체인별로 나누고 인자 수와 정수 범위를 검사한다.
        let (mut chains, mut chain, mut p) = (Vec::new(), Vec::new(), 0);
        let bytes = src.as_bytes();
        while p < bytes.len() {
            match bytes[p] {
                b'#' => {
                    while p < bytes.len() && bytes[p] != b'\n' {
                        p += 1
                    }
                    continue;
                }
                b';' => {
                    if !chain.is_empty() {
                        chains.push(std::mem::take(&mut chain))
                    }
                    p += 1;
                    continue;
                }
                b'{' | b'}' => {
                    chain.push(Inst {
                        name: (bytes[p] as char).to_string(),
                        args: vec![],
                        partner: 0,
                    });
                    p += 1;
                    continue;
                }
                c if c.is_ascii_whitespace() => {
                    p += 1;
                    continue;
                }
                _ => (),
            }
            let start = p;
            while p < bytes.len() && (bytes[p].is_ascii_alphanumeric() || bytes[p] == b'-') {
                p += 1
            }
            if p == start {
                return Err(Error::Parse("invalid token".into()));
            }
            let name = &src[start..p];
            if name == "danger" {
                return Err(Error::Parse("unsupported expression: danger".into()));
            }
            let expected =
                arity(name).ok_or_else(|| Error::Parse(format!("unknown expression: {name}")))?;
            let mut args = vec![];
            if p < bytes.len() && bytes[p] == b'(' {
                p += 1;
                let begin = p;
                while p < bytes.len() && bytes[p] != b')' {
                    p += 1
                }
                if p == bytes.len() {
                    return Err(Error::Parse("unclosed arguments".into()));
                }
                if p > begin {
                    args = src[begin..p]
                        .split(',')
                        .map(|x| x.trim().to_string())
                        .collect()
                }
                p += 1
            }
            if (expected >= 0 && args.len() != expected as usize)
                || (expected < 0 && args.len() > 1)
            {
                return Err(Error::Parse(format!("wrong arity: {name}")));
            }
            for (i, v) in args.iter().enumerate() {
                if !matches!(name, "piece" | "color" | "transition" | "wall-piece")
                    && !(i == 0 && matches!(name, "piece-on" | "color-on"))
                {
                    int(v)?;
                }
            }
            if slot_op(name) && !(0..SLOT_COUNT as i32).contains(&int(&args[0])?) {
                return Err(Error::Parse(format!(
                    "invalid slot for {name}: {} (expected 0..={})",
                    args[0],
                    SLOT_COUNT - 1
                )));
            }
            chain.push(Inst {
                name: name.into(),
                args,
                partner: 0,
            });
        }
        if !chain.is_empty() {
            chains.push(chain)
        }
        // 블록·반복문·점프의 짝을 미리 연결해 실행 중 탐색을 줄인다.
        for ch in &mut chains {
            let (mut opens, mut dos, mut labels) = (vec![], vec![], HashMap::new());
            for i in 0..ch.len() {
                match ch[i].name.as_str() {
                    "{" => opens.push(i),
                    "}" => {
                        let j = opens
                            .pop()
                            .ok_or_else(|| Error::Parse("unmatched block".into()))?;
                        ch[j].partner = i;
                        ch[i].partner = j
                    }
                    "do" => dos.push(i),
                    "while" => {
                        ch[i].partner = dos
                            .pop()
                            .ok_or_else(|| Error::Parse("unmatched while".into()))?
                    }
                    "label" if labels.insert(a(&ch[i], 0), i).is_some() => {
                        return Err(Error::Parse("duplicate label".into()));
                    }
                    _ => (),
                }
            }
            if !opens.is_empty() || !dos.is_empty() {
                return Err(Error::Parse("unclosed block or loop".into()));
            }
            for n in ch {
                if matches!(n.name.as_str(), "jmp" | "jne") {
                    n.partner = *labels
                        .get(&a(n, 0))
                        .ok_or_else(|| Error::Parse("undefined label".into()))?
                }
            }
        }
        Ok(Self { chains })
    }
    pub fn execute(&self, b: &Board, origin: Position) -> Result<Vec<DefaultAction>, Error> {
        let actor = b.piece_at(origin)?.ok_or(Error::EmptyOrigin)?;
        b.piece_name(actor)?;
        b.color_name(actor)?;
        let d = i32::from(b.direction(actor.owner)?);
        let mut result = vec![];
        for ch in &self.chains {
            // 레지스터와 기준 좌표는 체인마다 원점에서 새로 시작한다.
            let mut context = ExecutionContext::new(origin);
            let ExecutionContext {
                frames,
                registers,
                anchors,
                trans,
                wall_piece,
                out,
            } = &mut context;
            let (mut pc, mut fuel) = (0usize, 0usize);
            while pc < ch.len() {
                // 무한 반복이 후보 생성 전체를 멈추지 않도록 실행 횟수를 제한한다.
                fuel += 1;
                if fuel > 1_000_000 {
                    return Err(Error::ExecutionLimit);
                }
                let n = &ch[pc];
                let op = n.name.as_str();
                let top = frames.len() - 1;
                if op == "}" {
                    frames.pop();
                    pc += 1;
                    continue;
                }
                if op == "{" {
                    if !frames[top].state {
                        pc = n.partner + 1;
                        continue;
                    }
                    if frames.len() == 32 {
                        return Err(Error::BlockLimit);
                    }
                    let mut child = frames[top].clone();
                    // 자식 블록은 현재 좌표를 이어받고 조건을 참으로 시작한다.
                    child.state = true;
                    child.close = n.partner;
                    frames.push(child);
                    pc += 1;
                    continue;
                }
                if !frames[top].state
                    && !matches!(
                        op,
                        "not"
                            | "true"
                            | "false"
                            | "write"
                            | "read"
                            | "read-and"
                            | "read-or"
                            | "read-xor"
                            | "while"
                            | "jmp"
                            | "jne"
                            | "label"
                    )
                {
                    // 조건이 거짓이면 일반 명령은 건너뛰되 제어 명령은 처리한다.
                    if top > 0 {
                        pc = frames[top].close
                    } else {
                        break;
                    }
                    continue;
                }
                match op {
                    "not" => frames[top].state = !frames[top].state,
                    "true" => frames[top].state = true,
                    "false" => frames[top].state = false,
                    "write" => registers[slot(n)] = Some(frames[top].state),
                    "read" | "read-and" | "read-or" | "read-xor" => {
                        let value = registers[slot(n)]
                            .ok_or_else(|| Error::UninitializedRegister(a(n, 0)))?;
                        frames[top].state = match op {
                            "read" => value,
                            "read-and" => frames[top].state && value,
                            "read-or" => frames[top].state || value,
                            "read-xor" => frames[top].state ^ value,
                            _ => unreachable!(),
                        };
                    }
                    "do" => {
                        if ch.get(pc + 1).is_some_and(|next| next.name == "while") {
                            pc += 2;
                            continue;
                        }
                    }
                    "label" => (),
                    "while" => {
                        if frames[top].state {
                            frames[top].state = true;
                            pc = n.partner + 1
                        } else {
                            frames[top].state = true;
                            pc += 1
                        }
                        continue;
                    }
                    "jmp" | "jne" => {
                        if op == "jne" {
                            frames[top].state = !frames[top].state
                        }
                        if frames[top].state {
                            pc = n.partner
                        } else {
                            frames[top].state = true;
                            pc += 1
                        }
                        continue;
                    }
                    "repeat" => {
                        let dest = pc as i64 - i64::from(a(n, 0));
                        if dest < 0 || dest >= ch.len() as i64 {
                            return Err(Error::Parse("invalid repeat".into()));
                        }
                        pc = dest as usize;
                        continue;
                    }
                    "end" => break,
                    "piece" => frames[top].state = b.piece_name(actor)? == n.args[0],
                    "color" => frames[top].state = b.color_name(actor)? == n.args[0],
                    "wall-piece" => *wall_piece = Some(n.args[0].clone()),
                    "transition" => *trans = n.args.first().cloned(),
                    "write-anchor" => anchors[slot(n)] = Some(frames[top].pos),
                    "read-anchor" => {
                        frames[top].pos =
                            anchors[slot(n)].ok_or_else(|| Error::UninitializedAnchor(a(n, 0)))?;
                        frames[top].route = None;
                    }
                    "absolute" | "absolute-x" | "absolute-y" => {
                        let current = frames[top].pos;
                        let x = if op == "absolute-y" {
                            current.file
                        } else {
                            absolute_component(a(n, 0), b.width(), d)
                        };
                        let y = if op == "absolute-x" {
                            current.rank
                        } else {
                            absolute_component(a(n, usize::from(op == "absolute")), b.height(), d)
                        };
                        let q = Position::new(x, y);
                        if b.contains(q) {
                            frames[top].pos = q;
                            // Absolute coordinates may teleport; no traversed route is known.
                            frames[top].route = None;
                        } else {
                            frames[top].state = false
                        }
                    }
                    _ => {
                        let offset = usize::from(matches!(op, "piece-on" | "color-on"));
                        let q = target(frames[top].pos, a(n, offset), a(n, offset + 1), d);
                        let w = wall(b, q, d);
                        let piece_wall = wall_piece.is_some()
                            && b.contains(q)
                            && b.piece_at(q)?.is_some_and(|piece| {
                                piece.owner == actor.owner
                                    && wall_piece.as_ref().is_some_and(|key| {
                                        b.piece_name(piece).is_ok_and(|name| name == key)
                                    })
                            });
                        if op == "barrier" {
                            frames[top].state = w != Wall::None || piece_wall;
                        } else if wall_op(op) {
                            frames[top].state = wall_test(op, w)
                        } else if w != Wall::None {
                            frames[top].state = false
                        } else {
                            let occupant = b.piece_at(q)?;
                            let friendly = occupant.is_some_and(|x| x.owner == actor.owner);
                            let enemy = occupant.is_some_and(|x| x.owner != actor.owner);
                            match op {
                                "enemy" => frames[top].state = enemy,
                                "friendly" => frames[top].state = friendly,
                                "piece-on" => {
                                    frames[top].state = occupant.is_some_and(|x| {
                                        b.piece_name(x).is_ok_and(|v| v == n.args[0])
                                    })
                                }
                                "color-on" => {
                                    frames[top].state = occupant.is_some_and(|x| {
                                        b.color_name(x).is_ok_and(|v| v == n.args[0])
                                    })
                                }
                                "observe" => {
                                    if occupant.is_some() {
                                        frames[top].state = false
                                    }
                                }
                                "anchor" => {
                                    frames[top].pos = q;
                                    frames[top].route = None;
                                }
                                "peek" => {
                                    if occupant.is_some() {
                                        frames[top].state = false
                                    } else {
                                        frames[top].pos = q;
                                        frames[top].route = None;
                                    }
                                }
                                "move" => {
                                    if occupant.is_some() {
                                        frames[top].state = false
                                    } else {
                                        frames[top].pos = q;
                                        extend_route(&mut frames[top].route, q);
                                        emit(
                                            out,
                                            origin,
                                            q,
                                            q,
                                            Effect::Move,
                                            trans,
                                            frames[top].route.as_deref(),
                                        )
                                    }
                                }
                                "take-move" => {
                                    if friendly || piece_wall {
                                        frames[top].state = false
                                    } else {
                                        frames[top].pos = q;
                                        extend_route(&mut frames[top].route, q);
                                        emit(
                                            out,
                                            origin,
                                            q,
                                            q,
                                            Effect::TakeMove,
                                            trans,
                                            frames[top].route.as_deref(),
                                        );
                                        if enemy {
                                            frames[top].state = false
                                        }
                                    }
                                }
                                "take" => {
                                    if friendly {
                                        frames[top].state = false
                                    } else {
                                        frames[top].pos = q;
                                        if enemy {
                                            let mut take_route = frames[top].route.clone();
                                            extend_route(&mut take_route, q);
                                            emit(
                                                out,
                                                origin,
                                                q,
                                                q,
                                                Effect::Take,
                                                trans,
                                                take_route.as_deref(),
                                            );
                                            frames[top].has_take = true;
                                            frames[top].take = q
                                        }
                                    }
                                }
                                "catch" => {
                                    if !enemy {
                                        frames[top].state = false
                                    } else {
                                        emit(out, origin, q, origin, Effect::Catch, trans, None)
                                    }
                                }
                                "shift" => {
                                    frames[top].pos = q;
                                    if occupant.is_some() {
                                        emit(out, origin, q, q, Effect::Shift, trans, None)
                                    }
                                }
                                "jump" => {
                                    if !frames[top].has_take
                                        || occupant.is_some()
                                        || q == frames[top].pos
                                    {
                                        frames[top].state = false
                                    } else {
                                        let victim = frames[top].take;
                                        frames[top].pos = q;
                                        extend_route(&mut frames[top].route, q);
                                        if let Some(i) = out.iter().position(|x: &DefaultAction| {
                                            x.continuation.is_empty()
                                                && x.first.effect == Effect::Take
                                                && x.first.take == victim
                                        }) {
                                            // 앞서 만든 단순 잡기 후보를 잡고 점프하는 후보로 교체한다.
                                            out.remove(i);
                                        }
                                        emit(
                                            out,
                                            origin,
                                            victim,
                                            q,
                                            Effect::TakeJump,
                                            trans,
                                            frames[top].route.as_deref(),
                                        )
                                    }
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                }
                pc += 1;
            }
            result.extend(std::mem::take(out));
        }
        Ok(result)
    }
}
#[derive(Clone)]
struct Frame {
    pos: Position,
    state: bool,
    has_take: bool,
    take: Position,
    close: usize,
    route: Option<Vec<Position>>,
}
struct ExecutionContext {
    frames: Vec<Frame>,
    registers: [Option<bool>; SLOT_COUNT],
    anchors: [Option<Position>; SLOT_COUNT],
    trans: Option<String>,
    wall_piece: Option<String>,
    out: Vec<DefaultAction>,
}
impl ExecutionContext {
    fn new(origin: Position) -> Self {
        Self {
            frames: vec![Frame {
                pos: origin,
                state: true,
                has_take: false,
                take: origin,
                close: 0,
                route: Some(vec![origin]),
            }],
            registers: [None; SLOT_COUNT],
            anchors: [None; SLOT_COUNT],
            trans: None,
            wall_piece: None,
            out: Vec::new(),
        }
    }
}
fn slot(n: &Inst) -> usize {
    a(n, 0) as usize // 파서가 슬롯 범위를 검사한다.
}
fn absolute_component(value: i32, extent: usize, direction: i32) -> i32 {
    if direction < 0 {
        (extent as i32 - 1).saturating_sub(value)
    } else {
        value
    }
}
fn target(p: Position, dx: i32, dy: i32, d: i32) -> Position {
    // 전진 방향을 기준으로 상대 좌표를 보드 좌표로 바꾼다.
    Position::new(
        p.file.saturating_add(dx.saturating_mul(d)),
        p.rank.saturating_sub(dy.saturating_mul(d)),
    )
}
fn emit(
    out: &mut Vec<DefaultAction>,
    from: Position,
    take: Position,
    to: Position,
    effect: Effect,
    t: &Option<String>,
    route: Option<&[Position]>,
) {
    let mut s = ActionStep::new(from, take, to, effect);
    s.transition = t.clone();
    let mut action = DefaultAction::one(s);
    action.route = route.map(|r| r.to_vec());
    out.push(action)
}

fn extend_route(route: &mut Option<Vec<Position>>, position: Position) {
    if let Some(route) = route {
        if route.last() != Some(&position) {
            route.push(position);
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Wall {
    None,
    Other,
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}
fn wall(b: &Board, p: Position, d: i32) -> Wall {
    let (mut l, mut r, mut t, mut bot) = (
        p.file < 0,
        p.file >= b.width() as i32,
        p.rank < 0,
        p.rank >= b.height() as i32,
    );
    if d < 0 {
        // 반대편 플레이어에게도 위·아래·좌·우가 같은 상대 방향을 뜻하게 한다.
        std::mem::swap(&mut l, &mut r);
        std::mem::swap(&mut t, &mut bot)
    }
    if !(l || r || t || bot) && !b.contains(p) {
        return Wall::Other;
    }
    match (t, bot, l, r) {
        (true, _, true, _) => Wall::TopLeft,
        (true, _, _, true) => Wall::TopRight,
        (_, true, true, _) => Wall::BottomLeft,
        (_, true, _, true) => Wall::BottomRight,
        (true, _, _, _) => Wall::Top,
        (_, true, _, _) => Wall::Bottom,
        (_, _, true, _) => Wall::Left,
        (_, _, _, true) => Wall::Right,
        _ => Wall::None,
    }
}
fn wall_op(n: &str) -> bool {
    matches!(n, "bound" | "edge" | "corner") || n.starts_with("edge-") || n.starts_with("corner-")
}
fn wall_test(n: &str, w: Wall) -> bool {
    match n {
        "bound" => w != Wall::None,
        "edge" => matches!(w, Wall::Top | Wall::Bottom | Wall::Left | Wall::Right),
        "corner" => matches!(
            w,
            Wall::TopLeft | Wall::TopRight | Wall::BottomLeft | Wall::BottomRight
        ),
        "edge-top" => w == Wall::Top,
        "edge-bottom" => w == Wall::Bottom,
        "edge-left" => w == Wall::Left,
        "edge-right" => w == Wall::Right,
        "corner-top-left" => w == Wall::TopLeft,
        "corner-top-right" => w == Wall::TopRight,
        "corner-bottom-left" => w == Wall::BottomLeft,
        "corner-bottom-right" => w == Wall::BottomRight,
        _ => false,
    }
}
pub fn interpret(b: &Board, origin: Position, source: &str) -> Result<Vec<DefaultAction>, Error> {
    Program::parse(source)?.execute(b, origin)
}
