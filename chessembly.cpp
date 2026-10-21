#include "chessembly.hpp"
#include <algorithm>
#include <array>
#include <charconv>
#include <cctype>
#include <stdexcept>
#include <string_view>
#include <unordered_map>

namespace Chessembly {
namespace {
std::string trim(std::string_view s) {
    while (!s.empty() && std::isspace(static_cast<unsigned char>(s.front()))) s.remove_prefix(1);
    while (!s.empty() && std::isspace(static_cast<unsigned char>(s.back()))) s.remove_suffix(1);
    return std::string(s);
}
int integer(const std::string& s) {
    int v = 0;
    auto [p, e] = std::from_chars(s.data(), s.data() + s.size(), v);
    if (e != std::errc{} || p != s.data() + s.size()) throw std::invalid_argument("invalid Chessembly integer: " + s);
    return v;
}
int arg(const Instruction& i, size_t n) { return integer(i.args.at(n)); }
std::vector<std::string> splitArgs(std::string_view s) {
    std::vector<std::string> out;
    if (s.empty()) return out;
    while (true) {
        auto comma = s.find(',');
        out.push_back(trim(s.substr(0, comma)));
        if (comma == s.npos) return out;
        s.remove_prefix(comma + 1);
    }
}
const std::unordered_map<std::string, int>& arity() {
    static const std::unordered_map<std::string, int> a = {
        {"move",2},{"take",2},{"take-move",2},{"catch",2},{"shift",2},{"jump",2},
        {"peek",2},{"observe",2},{"anchor",2},{"absolute",2},{"absolute-x",1},{"absolute-y",1},
        {"bound",2},{"edge",2},{"corner",2},{"edge-top",2},{"edge-bottom",2},
        {"edge-left",2},{"edge-right",2},{"corner-top-left",2},{"corner-top-right",2},
        {"corner-bottom-left",2},{"corner-bottom-right",2},{"enemy",2},{"friendly",2},
        {"piece-on",3},{"color-on",3},{"piece",1},{"color",1},{"transition",-1},
        {"not",0},{"true",0},{"false",0},{"end",0},{"do",0},{"while",0},
        {"repeat",1},{"label",1},{"jmp",1},{"jne",1},
        {"write",1},{"read",1},{"read-and",1},{"read-or",1},{"read-xor",1},
        {"write-anchor",1},{"read-anchor",1},{"danger",2}
    };
    return a;
}
struct Frame { Coord pos; bool state = true; bool hasTake = false; Coord take{}; int close = -1; };
struct Context {
    const Board& board;
    Coord origin;
    Core::Piece actor;
    std::array<Frame,32> frames{};
    size_t depth = 1;
    std::string transition;
    std::vector<defaultAction> out;
    int direction;
    Context(const Board& b, Coord o, Core::Piece p) : board(b), origin(o), actor(p), direction(b.direction(p)) { frames[0].pos=o; }
    Frame& top() { return frames[depth-1]; }
    bool occupied(Coord p) const { return board.at(p.first,p.second).type != 0; }
    bool friendly(Coord p) const { auto q=board.at(p.first,p.second); return q.type && q.owner==actor.owner; }
    bool enemy(Coord p) const { auto q=board.at(p.first,p.second); return q.type && q.owner!=actor.owner; }
    Coord target(int dx,int dy) const { return {topConst().pos.first+dx*direction,topConst().pos.second-dy*direction}; }
    const Frame& topConst() const { return frames[depth-1]; }
    void emit(Effect e, Coord to, Coord take) { out.push_back({{origin,take,to,e,transition},{}}); }
};
enum class Wall { None, Other, Top, Bottom, Left, Right, TopLeft, TopRight, BottomLeft, BottomRight };
Wall wall(const Board& b, Coord p, int direction) {
    bool left=p.first<0, right=p.first>=b.width, top=p.second<0, bottom=p.second>=b.height;
    if (direction<0) { std::swap(left,right); std::swap(top,bottom); }
    if (!left && !right && !top && !bottom && !b.inside(p.first,p.second)) return Wall::Other;
    if (top && left) return Wall::TopLeft;
    if (top && right) return Wall::TopRight;
    if (bottom && left) return Wall::BottomLeft;
    if (bottom && right) return Wall::BottomRight;
    if (top) return Wall::Top;
    if (bottom) return Wall::Bottom;
    if (left) return Wall::Left;
    if (right) return Wall::Right;
    return Wall::None;
}
bool wallPredicate(std::string_view n, Wall w) {
    if (n=="bound") return w!=Wall::None;
    if (n=="edge") return w==Wall::Top||w==Wall::Bottom||w==Wall::Left||w==Wall::Right;
    if (n=="corner") return w==Wall::TopLeft||w==Wall::TopRight||w==Wall::BottomLeft||w==Wall::BottomRight;
    return (n=="edge-top"&&w==Wall::Top)||(n=="edge-bottom"&&w==Wall::Bottom)||
        (n=="edge-left"&&w==Wall::Left)||(n=="edge-right"&&w==Wall::Right)||
        (n=="corner-top-left"&&w==Wall::TopLeft)||(n=="corner-top-right"&&w==Wall::TopRight)||
        (n=="corner-bottom-left"&&w==Wall::BottomLeft)||(n=="corner-bottom-right"&&w==Wall::BottomRight);
}
} // namespace

Program::Program(const std::string& source) {
    std::vector<Instruction> chain;
    auto flush=[&]{ if (!chain.empty()) { chains_.push_back(std::move(chain)); chain.clear(); } };
    size_t p=0;
    while (p<source.size()) {
        char c=source[p];
        if (std::isspace(static_cast<unsigned char>(c))) { ++p; continue; }
        if (c=='#') { while (p<source.size() && source[p]!='\n') ++p; continue; }
        if (c==';') { flush(); ++p; continue; }
        if (c=='{'||c=='}') { chain.push_back({std::string(1,c),{}}); ++p; continue; }
        size_t start=p;
        while (p<source.size() && (std::isalnum(static_cast<unsigned char>(source[p]))||source[p]=='-')) ++p;
        if (p==start) throw std::invalid_argument("invalid Chessembly token");
        std::string name=source.substr(start,p-start);
        if (name=="check"||name=="set-state"||name=="if-state"||name=="state")
            throw std::invalid_argument("removed Chessembly expression: "+name);
        auto it=arity().find(name);
        if (it==arity().end()) throw std::invalid_argument("unknown Chessembly expression: "+name);
        std::vector<std::string> args;
        if (p<source.size()&&source[p]=='(') {
            ++p; size_t begin=p;
            while (p<source.size()&&source[p]!=')') ++p;
            if (p==source.size()) throw std::invalid_argument("unclosed Chessembly arguments");
            args=splitArgs(std::string_view(source).substr(begin,p-begin)); ++p;
        }
        if ((it->second>=0 && static_cast<int>(args.size())!=it->second) || (it->second<0 && args.size()>1))
            throw std::invalid_argument("wrong Chessembly arity: "+name);
        for (size_t k=0;k<args.size();++k) if (name!="piece"&&name!="color"&&name!="transition"&&
            !(k==0&&(name=="piece-on"||name=="color-on"))) (void)integer(args[k]);
        chain.push_back({std::move(name),std::move(args)});
    }
    flush();
    for (auto& ch:chains_) {
        std::vector<int> opens, dos;
        std::unordered_map<int,int> labels;
        for (int i=0;i<static_cast<int>(ch.size());++i) {
            auto& n=ch[static_cast<size_t>(i)];
            if (n.name=="{") opens.push_back(i);
            else if (n.name=="}") {
                if (opens.empty()) throw std::invalid_argument("unmatched Chessembly block");
                int o=opens.back(); opens.pop_back(); n.partner=o; ch[static_cast<size_t>(o)].partner=i;
            } else if (n.name=="do") dos.push_back(i);
            else if (n.name=="while") {
                if (dos.empty()) throw std::invalid_argument("unmatched Chessembly while");
                int d=dos.back();dos.pop_back();n.partner=d;
            } else if (n.name=="label") {
                if (!labels.emplace(arg(n,0),i).second) throw std::invalid_argument("duplicate Chessembly label");
            }
        }
        if (!opens.empty()||!dos.empty()) throw std::invalid_argument("unclosed Chessembly block/loop");
        for (auto& n:ch) if (n.name=="jmp"||n.name=="jne") {
            auto it=labels.find(arg(n,0));
            if (it==labels.end()) throw std::invalid_argument("undefined Chessembly label");
            n.partner=it->second;
        }
    }
}

std::vector<defaultAction> Program::execute(const Board& b, Coord origin) const {
    if (!b.inside(origin.first,origin.second)) throw std::out_of_range("Chessembly origin");
    auto actor=b.at(origin.first,origin.second);
    if (!actor.type) throw std::invalid_argument("Chessembly origin is empty");
    (void)b.pieceName(actor); (void)b.colorName(actor);
    std::vector<defaultAction> result;
    for (const auto& chain:chains_) {
        Context ctx(b,origin,actor);
        size_t pc=0, fuel=0;
        while (pc<chain.size()) {
            if (++fuel>1000000) throw std::runtime_error("Chessembly execution limit exceeded");
            const auto& n=chain[pc]; const auto& op=n.name;
            if (op=="}") { if (ctx.depth<2) throw std::logic_error("block stack"); --ctx.depth; ++pc; continue; }
            if (op=="{") { if (!ctx.top().state) { pc=static_cast<size_t>(n.partner+1); continue; } if(ctx.depth==ctx.frames.size()) throw std::runtime_error("Chessembly block depth exceeded"); ctx.frames[ctx.depth]=ctx.top(); ++ctx.depth; ctx.top().state=true; ctx.top().close=n.partner; ++pc; continue; }
            if (!ctx.top().state && op!="not" && op!="while" && op!="jmp" && op!="jne" && op!="label") { if (ctx.depth>1) pc=static_cast<size_t>(ctx.top().close); else break; continue; }
            if (op=="not") { ctx.top().state=!ctx.top().state; ++pc; continue; }
            if (op=="do") { if (pc+1<chain.size()&&chain[pc+1].name=="while") {++pc;continue;} ++pc;continue; }
            if (op=="while") { bool state=ctx.top().state; if (state) { ctx.top().state=true; pc=static_cast<size_t>(n.partner+1); } else { ctx.top().state=true;++pc; } continue; }
            if (op=="label") {++pc;continue;}
            if (op=="jmp"||op=="jne") { if (op=="jne") ctx.top().state=!ctx.top().state; bool jump=ctx.top().state; if (jump) pc=static_cast<size_t>(n.partner); else {ctx.top().state=true;++pc;} continue; }
            if (op=="repeat") { int dest=static_cast<int>(pc)-arg(n,0); if (dest<0||dest>=static_cast<int>(chain.size())) throw std::invalid_argument("invalid Chessembly repeat"); pc=static_cast<size_t>(dest);continue; }
            if (op=="end") break;
            if (!ctx.top().state) { ++pc; continue; }
            if (op=="piece") ctx.top().state=b.pieceName(actor)==n.args[0];
            else if (op=="color") ctx.top().state=b.colorName(actor)==n.args[0];
            else if (op=="transition") ctx.transition=n.args.empty()?"":n.args[0];
            else if (op=="true"||op=="false"||op=="write"||op=="read"||op=="read-and"||op=="read-or"||op=="read-xor"||op=="write-anchor"||op=="read-anchor"||op=="danger"||op=="absolute-x"||op=="absolute-y") {
                // Parsed in the source, but the original JIT has no execution case for these opcodes.
            } else if (op=="absolute") {
                Coord q{arg(n,0),arg(n,1)};
                if (ctx.direction<0) q={b.width-1-q.first,b.height-1-q.second};
                if (!b.inside(q.first,q.second)) ctx.top().state=false; else ctx.top().pos=q;
            } else {
                size_t deltaOffset=(op=="piece-on"||op=="color-on")?1:0;
                Coord q=ctx.target(arg(n,deltaOffset),arg(n,deltaOffset+1));
                Wall w=wall(b,q,ctx.direction); bool inside=w==Wall::None;
                if (op=="bound"||op=="edge"||op=="corner"||op.starts_with("edge-")||op.starts_with("corner-")) ctx.top().state=wallPredicate(op,w);
                else if (!inside) ctx.top().state=false;
                else if (op=="enemy") ctx.top().state=ctx.enemy(q);
                else if (op=="friendly") ctx.top().state=ctx.friendly(q);
                else if (op=="piece-on") ctx.top().state=ctx.occupied(q)&&b.pieceName(b.at(q.first,q.second))==n.args[0];
                else if (op=="color-on") ctx.top().state=ctx.occupied(q)&&b.colorName(b.at(q.first,q.second))==n.args[0];
                else if (op=="observe") { if (ctx.occupied(q)) ctx.top().state=false; }
                else if (op=="anchor") ctx.top().pos=q;
                else if (op=="peek") { if (ctx.occupied(q)) ctx.top().state=false; else ctx.top().pos=q; }
                else if (op=="move") { if (ctx.occupied(q)) ctx.top().state=false; else { ctx.top().pos=q;ctx.emit(Effect::Move,q,q); } }
                else if (op=="take-move") { if (ctx.friendly(q)) ctx.top().state=false; else { ctx.top().pos=q;ctx.emit(Effect::TakeMove,q,q); if (ctx.enemy(q)) ctx.top().state=false; } }
                else if (op=="take") { if (ctx.friendly(q)) ctx.top().state=false; else {ctx.top().pos=q;if (ctx.enemy(q)) {ctx.emit(Effect::Take,q,q);ctx.top().hasTake=true;ctx.top().take=q;} } }
                else if (op=="catch") { if (!ctx.enemy(q)) ctx.top().state=false; else ctx.emit(Effect::Catch,origin,q); }
                else if (op=="shift") { ctx.top().pos=q;if(ctx.occupied(q)) ctx.emit(Effect::Shift,q,q); }
                else if (op=="jump") {
                    if (!ctx.top().hasTake || ctx.occupied(q) || q==ctx.top().pos) ctx.top().state=false;
                    else { Coord victim=ctx.top().take; ctx.top().pos=q;
                        auto it=std::find_if(ctx.out.begin(),ctx.out.end(),[&](const defaultAction& a){return a.continuation.empty()&&a.first.effect==Effect::Take&&a.first.take==victim;});
                        if(it!=ctx.out.end()) ctx.out.erase(it);
                        ctx.emit(Effect::TakeJump,q,victim);
                    }
                }
            }
            ++pc;
        }
        result.insert(result.end(),std::make_move_iterator(ctx.out.begin()),std::make_move_iterator(ctx.out.end()));
    }
    return result;
}
bool applyDefaultAction(Board& b, const defaultAction& action) {
    std::vector<std::pair<int,Core::Piece>> undo;
    undo.reserve((1+action.continuation.size())*3);
    auto remember=[&](Coord p) {
        int index=b.index(p.first,p.second);
        if (std::none_of(undo.begin(),undo.end(),[&](const auto& e){return e.first==index;}))
            undo.emplace_back(index,b.cells[static_cast<size_t>(index)]);
    };
    auto rollback=[&] {for(auto it=undo.rbegin();it!=undo.rend();++it) b.cells[static_cast<size_t>(it->first)]=it->second;};
    auto applyStep=[&](const ActionStep& s) -> bool {
        if (!b.inside(s.from.first,s.from.second)||!b.inside(s.moveTo.first,s.moveTo.second)||
            !b.inside(s.take.first,s.take.second)) return false;
        auto mover=b.at(s.from.first,s.from.second);
        if (!mover.type) return false;
        auto victim=b.at(s.take.first,s.take.second);
        auto destination=b.at(s.moveTo.first,s.moveTo.second);
        bool valid=false;
        switch(s.effect) {
            case Effect::Move:
                valid=s.take==s.moveTo && !destination.type && s.from!=s.moveTo;break;
            case Effect::TakeMove:
                valid=s.take==s.moveTo && s.from!=s.moveTo &&
                    (!destination.type || destination.owner!=mover.owner);break;
            case Effect::Take:
                valid=s.take==s.moveTo && s.from!=s.moveTo && victim.type && victim.owner!=mover.owner;break;
            case Effect::TakeJump:
                valid=s.take!=s.moveTo && s.from!=s.moveTo && victim.type &&
                    victim.owner!=mover.owner && !destination.type;break;
            case Effect::Catch:
                valid=s.moveTo==s.from && s.take!=s.from && victim.type && victim.owner!=mover.owner;break;
            case Effect::Shift:
                valid=s.take==s.moveTo && s.from!=s.moveTo && destination.type;break;
        }
        if (!valid) return false;
        if (!s.transition.empty()) {
            auto it=std::find(b.pieceNames.begin(),b.pieceNames.end(),s.transition);
            if(it==b.pieceNames.end()||it==b.pieceNames.begin()) return false;
            mover.type=static_cast<int>(it-b.pieceNames.begin());
        }
        remember(s.from);remember(s.take);remember(s.moveTo);
        if(s.effect==Effect::Shift) {
            b.at(s.from.first,s.from.second)=destination;
            b.at(s.moveTo.first,s.moveTo.second)=mover;
        } else if(s.effect==Effect::Catch) {
            b.at(s.take.first,s.take.second)={};
            b.at(s.from.first,s.from.second)=mover;
        } else {
            b.at(s.from.first,s.from.second)={};
            if(s.effect==Effect::TakeJump) b.at(s.take.first,s.take.second)={};
            b.at(s.moveTo.first,s.moveTo.second)=mover;
        }
        return true;
    };
    if (!applyStep(action.first)) {rollback();return false;}
    for (const auto& step:action.continuation) if (!applyStep(step)) {rollback();return false;}
    return true;
}
std::vector<defaultAction> chessemblyInterpreter(Board board, Coord origin, std::string code) {
    return Program(code).execute(board,origin);
}
} // namespace Chessembly
