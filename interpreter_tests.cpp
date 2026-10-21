#include "chessembly.hpp"
#include <iostream>
#include <stdexcept>
#include <string>

using namespace Chessembly;
// A second rule module may use a different piece-definition schema.
struct MiniRules {
    struct Definition { int value; std::string name; };
    struct State { Core::Board board{3,4}; };
    static std::vector<Definition> pieceDefinitions() { return {{7,"hopper"}}; }
    static Core::Board boardBoundary() { return Core::Board(3,4); }
    static State initialPosition() {
        State s; s.board.pieceNames={"","hopper"}; s.board.colorNames={"amber"};
        s.board.at(1,1)={1,0,1}; return s;
    }
    static Core::GameResult victoryCondition(State&) { return {}; }
};
static_assert(Core::RuleModule<MiniRules>);
void expect(bool ok, const char* message) { if (!ok) throw std::runtime_error(message); }
void rejected(const std::string& code) {
    try { Program p(code); (void)p; } catch (const std::invalid_argument&) { return; }
    throw std::runtime_error("expected parse error: " + code);
}
int main() {
    try {
        auto mini=MiniRules::initialPosition();
        expect(chessemblyInterpreter(mini.board,{1,1},"color(amber) move(1,0)").size()==1,
               "second module shares interpreter");
        Board b(6,5);
        b.pieceNames={"","runner","target"};
        b.colorNames={"red","blue"};
        b.colorDirections={1,-1};
        b.at(2,2)={1,0,71};
        b.at(3,2)={2,1,72};
        b.at(2,3)={2,0,73};
        Board before=b;
        auto run=[&](std::string code){return chessemblyInterpreter(b,{2,2},std::move(code));};
        auto one=[&](std::string code,Effect effect,Coord take,Coord to){
            auto actions=run(std::move(code));
            expect(actions.size()==1 && actions[0].continuation.empty(),"one action");
            auto step=actions[0].first;
            expect(step.effect==effect && step.from==Coord{2,2} && step.take==take && step.moveTo==to,"step contract");
        };
        one("move(-1,0)",Effect::Move,{1,2},{1,2});
        one("take-move(1,0)",Effect::TakeMove,{3,2},{3,2});
        one("take(1,0)",Effect::Take,{3,2},{3,2});
        one("catch(1,0)",Effect::Catch,{3,2},{2,2});
        one("shift(1,0)",Effect::Shift,{3,2},{3,2});
        one("take(1,0) jump(1,0)",Effect::TakeJump,{3,2},{4,2});
        expect(run("move(1,0)").empty(),"occupied move blocked");
        expect(run("take-move(0,-1)").empty(),"friendly target blocked");
        expect(run("color(red) move(-1,0)").size()==1,"actor color true");
        expect(run("color(blue) move(-1,0)").empty(),"actor color false");
        expect(run("piece(runner) move(-1,0)").size()==1,"actor piece name");
        expect(run("piece-on(target,1,0) move(-1,0)").size()==1,"piece-on");
        expect(run("color-on(blue,1,0) move(-1,0)").size()==1,"color-on");
        expect(run("enemy(1,0) move(-1,0)").size()==1,"enemy condition");
        expect(run("friendly(0,-1) move(-1,0)").size()==1,"friendly condition");
        auto blocked=run("{ anchor(-1,0) move(-1,0) } move(0,1)");
        expect(blocked.size()==2 && blocked[1].first.moveTo==Coord{2,1},"block restores anchor");
        expect(run("transition(target) move(-1,0)")[0].first.transition=="target","transition payload");
        expect(run("transition(target) transition move(-1,0)")[0].first.transition.empty(),"transition erase");
        expect(run("absolute(0,0) move(1,0)").size()==1,"absolute positioning");
        auto paths=run("move(-1,0) move(-1,0)");
        expect(paths.size()==2 && paths[0].first.moveTo==Coord{1,2} && paths[1].first.moveTo==Coord{0,2},"consecutive emissions");
        expect(run("anchor(-2,0) bound(-1,0) move(1,0)").size()==1,"boundary condition");
        expect(run("anchor(-2,0) edge-left(-1,0) move(1,0)").size()==1,"edge condition");
        expect(run("anchor(-2,-2) corner-bottom-left(-1,-1) move(1,0)").size()==1,"corner condition");
        expect(run("peek(-1,0) move(-1,0)").size()==1,"peek advances without emitting");
        expect(run("observe(1,0) move(-1,0)").empty(),"observe occupied");
        expect(run("move(-1,0) repeat(1)").size()==2,"repeat until board edge");
        expect(run("do move(-1,0) while").size()==2,"do-while until board edge");
        expect(run("color(blue) { move(-1,0) } move(-1,0)").empty(),"false block remains false outside");
        expect(run("color(blue) not move(-1,0)").size()==1,"not condition");
        expect(run("color(red) jne(1) move(-1,0) label(1)").size()==1,"conditional jump");
        expect(run("color(blue) jne(1) move(-1,0) label(1) move(0,1)").size()==1,"conditional jump taken and state reset");
        auto applied=[&](std::string code) {
            Board copy=b;
            auto actions=run(std::move(code));
            expect(actions.size()==1 && applyDefaultAction(copy,actions[0]),"apply action");
            return copy;
        };
        auto moved=applied("move(-1,0)");
        expect(!moved.at(2,2).type && moved.at(1,2).id==71,"apply move");
        auto captured=applied("take-move(1,0)");
        expect(!captured.at(2,2).type && captured.at(3,2).id==71,"apply take-move");
        auto taken=applied("take(1,0)");
        expect(!taken.at(2,2).type && taken.at(3,2).id==71,"apply take");
        auto caught=applied("catch(1,0)");
        expect(caught.at(2,2).id==71 && !caught.at(3,2).type,"apply stationary capture");
        auto jumped=applied("take(1,0) jump(1,0)");
        expect(jumped.at(4,2).id==71 && !jumped.at(3,2).type && !jumped.at(2,2).type,"apply take-jump");
        auto shifted=applied("shift(1,0)");
        expect(shifted.at(2,2).id==72 && shifted.at(3,2).id==71,"apply swap");
        auto transformed=applied("transition(target) move(-1,0)");
        expect(transformed.at(1,2).type==2,"apply transition");
        Board atomic=b;
        auto first=run("move(-1,0)")[0];
        first.continuation.push_back({{1,2},{3,2},{3,2},Effect::Move,{}});
        expect(!applyDefaultAction(atomic,first) && atomic.cells==b.cells,"failed multi-step rolls back");
        expect(b.cells==before.cells && b.pieceNames==before.pieceNames && b.colorNames==before.colorNames,"pure input board");
        expect(run("move(-1,0)")==run("move(-1,0)"),"deterministic repeat call");
        b.at(4,3)={1,1,80};
        auto reverse=chessemblyInterpreter(b,{4,3},"color(blue) move(1,0)");
        expect(reverse.size()==1 && reverse[0].first.moveTo==Coord{3,3},"module-defined direction");
        Board shaped=b;
        shaped.boundary.assign(shaped.cells.size(),1);
        shaped.boundary[static_cast<size_t>(shaped.index(1,2))]=0;
        expect(chessemblyInterpreter(shaped,{2,2},"move(-1,0)").empty(),"playable mask");
        Board big(12,10);big.pieceNames={"","runner"};big.colorNames={"red"};big.at(10,8)={1,0,1};
        expect(chessemblyInterpreter(big,{10,8},"move(1,0)").size()==1,"non-8 board");
        rejected("check");rejected("set-state(flag,1)");rejected("if-state(flag,1)");
        rejected("bogus(1,2)");rejected("move(1)");rejected("move(1,2");
        std::cout << "Interpreter tests passed\n";
    } catch (const std::exception& e) { std::cerr << "Interpreter test failed: " << e.what() << '\n'; return 1; }
}
