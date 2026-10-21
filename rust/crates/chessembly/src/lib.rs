//! Reusable, read-only Chessembly candidate generator.
use chesslayer_core::{ActionStep, Board, BoardError, DefaultAction, Effect, Position};
use std::collections::HashMap;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Parse(String),
    Board(BoardError),
    EmptyOrigin,
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
    chains: Vec<Vec<Inst>>,
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
        | "danger" => 2,
        "piece-on" | "color-on" => 3,
        "absolute-x" | "absolute-y" | "piece" | "color" | "repeat" | "label" | "jmp" | "jne"
        | "write" | "read" | "read-and" | "read-or" | "read-xor" | "write-anchor"
        | "read-anchor" => 1,
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
                if !matches!(name, "piece" | "color" | "transition")
                    && !(i == 0 && matches!(name, "piece-on" | "color-on"))
                {
                    int(v)?;
                }
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
            let mut frames = vec![Frame {
                pos: origin,
                state: true,
                has_take: false,
                take: origin,
                close: 0,
            }];
            let mut trans: Option<String> = None;
            let mut out = vec![];
            let (mut pc, mut fuel) = (0usize, 0usize);
            while pc < ch.len() {
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
                    let mut child = frames[top];
                    child.state = true;
                    child.close = n.partner;
                    frames.push(child);
                    pc += 1;
                    continue;
                }
                if !frames[top].state && !matches!(op, "not" | "while" | "jmp" | "jne" | "label") {
                    if top > 0 {
                        pc = frames[top].close
                    } else {
                        break;
                    }
                    continue;
                }
                match op {
                    "not" => frames[top].state = !frames[top].state,
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
                    "transition" => trans = n.args.first().cloned(),
                    // These opcodes intentionally have no execution case in the C++ interpreter.
                    "true" | "false" | "write" | "read" | "read-and" | "read-or" | "read-xor"
                    | "write-anchor" | "read-anchor" | "danger" | "absolute-x" | "absolute-y" => (),
                    "absolute" => {
                        let mut q = Position::new(a(n, 0), a(n, 1));
                        if d < 0 {
                            q = Position::new(
                                b.width() as i32 - 1 - q.file,
                                b.height() as i32 - 1 - q.rank,
                            )
                        }
                        if b.contains(q) {
                            frames[top].pos = q
                        } else {
                            frames[top].state = false
                        }
                    }
                    _ => {
                        let offset = usize::from(matches!(op, "piece-on" | "color-on"));
                        let q = target(frames[top].pos, a(n, offset), a(n, offset + 1), d);
                        let w = wall(b, q, d);
                        if wall_op(op) {
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
                                "anchor" => frames[top].pos = q,
                                "peek" => {
                                    if occupant.is_some() {
                                        frames[top].state = false
                                    } else {
                                        frames[top].pos = q
                                    }
                                }
                                "move" => {
                                    if occupant.is_some() {
                                        frames[top].state = false
                                    } else {
                                        frames[top].pos = q;
                                        emit(&mut out, origin, q, q, Effect::Move, &trans)
                                    }
                                }
                                "take-move" => {
                                    if friendly {
                                        frames[top].state = false
                                    } else {
                                        frames[top].pos = q;
                                        emit(&mut out, origin, q, q, Effect::TakeMove, &trans);
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
                                            emit(&mut out, origin, q, q, Effect::Take, &trans);
                                            frames[top].has_take = true;
                                            frames[top].take = q
                                        }
                                    }
                                }
                                "catch" => {
                                    if !enemy {
                                        frames[top].state = false
                                    } else {
                                        emit(&mut out, origin, q, origin, Effect::Catch, &trans)
                                    }
                                }
                                "shift" => {
                                    frames[top].pos = q;
                                    if occupant.is_some() {
                                        emit(&mut out, origin, q, q, Effect::Shift, &trans)
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
                                        if let Some(i) = out.iter().position(|x: &DefaultAction| {
                                            x.continuation.is_empty()
                                                && x.first.effect == Effect::Take
                                                && x.first.take == victim
                                        }) {
                                            out.remove(i);
                                        }
                                        emit(&mut out, origin, victim, q, Effect::TakeJump, &trans)
                                    }
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                }
                pc += 1;
            }
            result.extend(out);
        }
        Ok(result)
    }
}
#[derive(Clone, Copy)]
struct Frame {
    pos: Position,
    state: bool,
    has_take: bool,
    take: Position,
    close: usize,
}
fn target(p: Position, dx: i32, dy: i32, d: i32) -> Position {
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
) {
    let mut s = ActionStep::new(from, take, to, effect);
    s.transition = t.clone();
    out.push(DefaultAction::one(s))
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
