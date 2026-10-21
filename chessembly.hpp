#pragma once
#include "core.hpp"
#include <string>
#include <utility>
#include <vector>

namespace Chessembly {
using Board = Core::Board;
using Coord = std::pair<int, int>;

// One step has one invariant meaning in every rule module. Coordinates are (file, rank).
// Move: relocate to empty moveTo. TakeMove: relocate and capture occupant of take
// (take == moveTo); Take: same capture/relocation, but records the take instruction.
// TakeJump: capture at take, relocate to distinct empty moveTo.
// Catch: capture at take without relocating (moveTo == from).
// Shift: swap occupants of from and moveTo; take identifies the swap target.
// A module may reject a candidate, but may not reinterpret these effects.
enum class Effect { Move, TakeMove, Take, TakeJump, Catch, Shift };
struct ActionStep {
    Coord from{}, take{}, moveTo{};
    Effect effect = Effect::Move;
    std::string transition; // optional resulting piece name; empty means no transition
    bool operator==(const ActionStep&) const = default;
};
struct defaultAction {
    ActionStep first; // common single-step case needs no heap allocation
    std::vector<ActionStep> continuation; // ordered additional steps, only when needed
    bool operator==(const defaultAction&) const = default;
};

// Applies the shared effect contract atomically. Rule modules may filter actions
// before calling this and update their own game state afterward.
bool applyDefaultAction(Board& board, const defaultAction& action);

struct Instruction {
    std::string name;
    std::vector<std::string> args;
    int partner = -1;
};
class Program {
public:
    explicit Program(const std::string& source);
    std::vector<defaultAction> execute(const Board& board, Coord origin) const;
private:
    std::vector<std::vector<Instruction>> chains_;
};

// Requested compatibility entry point. Pass by value preserves the specified signature.
std::vector<defaultAction> chessemblyInterpreter(Board board, Coord initPieceCoord, std::string chessemblyCode);
} // namespace Chessembly

using Chessembly::defaultAction;
using Chessembly::chessemblyInterpreter;
