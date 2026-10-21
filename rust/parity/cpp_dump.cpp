#include "chessembly.hpp"
#include <iostream>
#include <vector>
int main(){using namespace Chessembly;Board b(6,5);b.pieceNames={"","runner","target"};b.colorNames={"red","blue"};b.colorDirections={1,-1};b.at(2,2)={1,0,71};b.at(3,2)={2,1,72};b.at(2,3)={2,0,73};
std::vector<std::string> cases={"move(-1,0)","take-move(1,0)","take(1,0)","catch(1,0)","shift(1,0)","take(1,0) jump(1,0)","color(red) move(-1,0)","color(blue) move(-1,0)","piece-on(target,1,0) move(-1,0)","move(-1,0) repeat(1)","do move(-1,0) while","{ anchor(-1,0) move(-1,0) } move(0,1)","transition(target) move(-1,0)","anchor(-2,0) edge-left(-1,0) move(1,0)"};
for(size_t i=0;i<cases.size();++i){auto acts=Program(cases[i]).execute(b,{2,2});std::cout<<i<<":"<<acts.size();for(auto& a:acts){auto s=a.first;std::cout<<"|"<<static_cast<int>(s.effect)<<","<<s.from.first<<","<<s.from.second<<","<<s.take.first<<","<<s.take.second<<","<<s.moveTo.first<<","<<s.moveTo.second<<","<<s.transition;Board copy=b;bool ok=applyDefaultAction(copy,a);std::cout<<","<<ok<<":";for(auto piece:copy.cells)std::cout<<piece.id<<"/"<<piece.type<<"/"<<piece.owner<<",";}std::cout<<"\n";}}
